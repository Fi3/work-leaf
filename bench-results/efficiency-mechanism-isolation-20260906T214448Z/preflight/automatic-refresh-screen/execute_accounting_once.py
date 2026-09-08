"""One fixed original-accounting publication; no baseline or derivative calls."""
import argparse
import copy
from datetime import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import stat
import subprocess
import sys
import time
import types


HELPER_SHA = "c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a"
DEPENDENCY_PINS = {"analyze.py": "2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78",
    "audit_compaction.py": "dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153",
    "../efficiency-measurement-gate-20260906/batch_analysis.py": "dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3"}
NEW_IDS = [f"automatic-refresh-01-workflow-{n:03}" for n in (1, 2, 3)]
BASELINE_IDS = [f"work-units-01-workflow-{n:03}" for n in (2, 4, 6, 9, 10, 12)]
METRICS = ("raw_input_plus_output", "uncached_input_plus_output")


def require(value, reason):
    if not value:
        raise ValueError(reason)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False).encode()


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def decode(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate JSON field")
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs,
                      parse_constant=lambda _: (_ for _ in ()).throw(ValueError("nonfinite JSON number")))


def sha_value(value):
    require(type(value) is str and len(value) == 64 and all(c in "0123456789abcdef" for c in value), "invalid SHA256")
    return value


def identity(value):
    require(type(value) is str and bool(value), "invalid identity")
    return value


def exact_path(value, directory=False):
    path = Path(value)
    require(path.is_absolute() and path.resolve() == path and not path.is_symlink(), "noncanonical or symlink path")
    mode = path.stat().st_mode
    require(stat.S_ISDIR(mode) if directory else stat.S_ISREG(mode), "wrong path type")
    return path


def read_exact(path, expected):
    path = exact_path(path); raw = path.read_bytes()
    require(digest(raw) == sha_value(expected), "source SHA256 mismatch")
    return raw


def remember_ref(ref, pins):
    require(type(ref) is dict and type(ref.get("path")) is str, "invalid source reference")
    path = str(exact_path(ref["path"])); expected = sha_value(ref.get("sha256"))
    require(path not in pins or pins[path] == expected, "conflicting source pins")
    raw = read_exact(path, expected); pins[path] = expected
    return decode(raw)


def verify(pins):
    for path, expected in pins.items():
        read_exact(path, expected)


def compile_accounting(actual_path, captured_exact_bytes, parsed_manifest):
    path = exact_path(actual_path)
    require(digest(captured_exact_bytes) == HELPER_SHA and path.read_bytes() == captured_exact_bytes, "accounting helper source differs")
    matches = [row for row in parsed_manifest["files"] if Path(row["path"]).resolve() == path]
    require(len(matches) == 1 and matches[0].get("role") == "frozen-evidence"
            and matches[0].get("sha256") == HELPER_SHA and Path(matches[0]["path"]) == path, "helper lacks unique actual manifest identity")
    for relative, expected in DEPENDENCY_PINS.items():
        read_exact((path.parent / relative).resolve(), expected)
    module = types.ModuleType("once_original_accounting"); module.__file__ = str(path)
    exec(compile(captured_exact_bytes, str(path), "exec"), module.__dict__)
    return module


class WorkerFailure(RuntimeError):
    def __init__(self, kind, closed, exit_code, evidence=None):
        super().__init__(kind)
        self.kind = kind; self.closed = closed; self.exit_code = exit_code
        self.evidence = evidence or {}


def group_exists(pid):
    try:
        os.killpg(pid, 0)
        return True
    except ProcessLookupError:
        return False


def signal_group(pid, value):
    try: os.killpg(pid, value)
    except ProcessLookupError: pass


def exclusive(path):
    path = Path(path); exact_path(path.parent, directory=True)
    require(not os.path.lexists(path), "publication path already exists")
    return os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)


def write_fd(fd, raw):
    view = memoryview(raw)
    while view:
        count = os.write(fd, view); require(count > 0, "short publication write")
        view = view[count:]
    os.fsync(fd)


def supervise_worker(argv, stdout_path, stderr_path, timeout_seconds, grace_seconds, *, pass_fds=()):
    require(timeout_seconds > 0 and grace_seconds > 0, "worker deadlines must be positive")
    start = time.monotonic(); process = None; timed_out = False; escalated = False
    out = exclusive(stdout_path)
    try:
        err = exclusive(stderr_path)
    except BaseException:
        os.close(out); raise
    try:
        process = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=out, stderr=err,
                                   start_new_session=True, pass_fds=pass_fds)
        try: process.wait(timeout=timeout_seconds)
        except subprocess.TimeoutExpired:
            timed_out = True; signal_group(process.pid, signal.SIGTERM)
            try: process.wait(timeout=grace_seconds)
            except subprocess.TimeoutExpired:
                escalated = True; signal_group(process.pid, signal.SIGKILL)
                try: process.wait(timeout=grace_seconds)
                except subprocess.TimeoutExpired: pass
        if process.poll() is not None and group_exists(process.pid):
            escalated = True; signal_group(process.pid, signal.SIGKILL)
        closed = process.poll() is not None and not group_exists(process.pid)
        os.fsync(out); os.fsync(err)
        return {"exit_code": process.poll(), "closed": closed, "timed_out": timed_out, "escalated": escalated,
                "elapsed_seconds": time.monotonic() - start, "stdout_path": str(stdout_path), "stderr_path": str(stderr_path)}
    except BaseException:
        if process is not None and process.poll() is None:
            signal_group(process.pid, signal.SIGKILL)
            try: process.wait(timeout=grace_seconds)
            except subprocess.TimeoutExpired: pass
        raise
    finally:
        os.close(out); os.close(err)


BOOTSTRAP = r'''
import hashlib,json,os,sys,types
from pathlib import Path
raw=Path(sys.argv[1]).read_bytes()
if hashlib.sha256(raw).hexdigest()!=sys.argv[2]: raise ValueError('worker input hash differs')
v=json.loads(raw); p=Path(v['helper']['path']); data=p.read_bytes()
if not p.is_absolute() or p.resolve()!=p or p.is_symlink(): raise ValueError('worker helper path differs')
if hashlib.sha256(data).hexdigest()!=v['helper']['sha256']: raise ValueError('worker helper bytes differ')
module=types.ModuleType('original_accounting');module.__file__=str(p)
exec(compile(data,str(p),'exec'),module.__dict__)
result=module.audit_run(v['entry'],v['manifest'],Path(v['sessions_root']))
raw=json.dumps(result,sort_keys=True,separators=(',',':'),ensure_ascii=True,allow_nan=False).encode()
fd=int(sys.argv[3]);view=memoryview(raw)
while view:
    count=os.write(fd,view)
    if count<=0: raise OSError('worker short result write')
    view=view[count:]
os.fsync(fd)
'''


def invoke_accounting(path, raw, entry, frozen, sessions_root, timeout_seconds, operation_root):
    root = exact_path(operation_root, directory=True)
    require(timeout_seconds == 600 and digest(raw) == HELPER_SHA, "worker invocation scope differs")
    value = {"helper": {"path": str(path), "sha256": HELPER_SHA}, "entry": entry,
             "manifest": frozen, "sessions_root": str(sessions_root)}
    data = canonical(value); input_path = root / "INPUT.json"
    fd = exclusive(input_path)
    try: write_fd(fd, data)
    finally: os.close(fd)
    result_path = root / "RESULT.json"; output = exclusive(result_path)
    try:
        execution = supervise_worker([str(Path(sys.executable).resolve()), "-I", "-B", "-c", BOOTSTRAP,
                                      str(input_path), digest(data), str(output)], root / "stdout", root / "stderr",
                                     timeout_seconds, 5, pass_fds=(output,))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        raise WorkerFailure("worker_exception", False, None, {"error_type": type(error).__name__}) from error
    finally: os.close(output)
    execution["input_sha256"] = digest(data)
    evidence_path = root / "EXECUTION.json"; fd = exclusive(evidence_path)
    try: write_fd(fd, canonical(execution))
    finally: os.close(fd)
    if not execution["closed"] or execution["exit_code"] != 0:
        raise WorkerFailure("worker_timeout" if execution["timed_out"] else "worker_exception",
                            execution["closed"], execution["exit_code"], execution)
    try: return decode(result_path.read_bytes())
    except (ValueError, OSError) as error:
        raise WorkerFailure("worker_exception", True, execution["exit_code"], execution) from error


def checked_map(value):
    require(type(value) is dict, "response map missing")
    for response, row in value.items():
        identity(response); require(type(row) is dict, "response record missing")
        identity(row.get("thread_id")); identity(row.get("turn_id"))
        usage = row.get("usage"); require(type(usage) is dict, "response usage missing")
        fields = ("input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens")
        require(all(type(usage.get(k)) is int and usage[k] >= 0 for k in fields), "response counters invalid")
        i, c, o, r = (usage[k] for k in fields)
        require(c <= i and r <= o, "response parent counters invalid")
        for key, expected in (("uncached_input_tokens", i-c), ("raw_input_plus_output", i+o), ("uncached_input_plus_output", i-c+o)):
            require(key not in usage or (type(usage[key]) is int and usage[key] == expected), "derived response counter invalid")
        require(type(row.get("sources")) is list and bool(row["sources"]), "response source locators missing")
    return value


def project(full):
    value = copy.deepcopy(full); value["exact_helper_result_sha256"] = digest(canonical(full))
    evidence = value["exact_response_evidence"]
    value["exact_response_evidence"] = {"count": len(evidence), "sha256": digest(canonical(evidence)),
        "retention": "Complete unchanged helper result retained in the exact FULL sidecar."}
    inventory = value.get("measurement", {}).get("gap_inventory")
    if type(inventory) is dict and "raw_responses" in inventory:
        raw = inventory.pop("raw_responses")
        inventory["raw_response_inventory_identity"] = {"count": len(raw), "sha256": digest(canonical(raw))}
    scope = value.get("corrected_scope")
    if type(scope) is dict:
        for capture in scope.get("captures", []):
            if "proof_view_server_lines" in capture:
                lines = capture.pop("proof_view_server_lines")
                capture["proof_view_server_lines_identity"] = {"count": len(lines), "sha256": digest(canonical(lines)),
                    "retention": "Original physical source and deterministic frozen predicate retained; no rewritten stream."}
    return value


def validate_result(value, run_id):
    require(type(value) is dict and value.get("schema") == "work-leaf-read-identity-accounting-v1"
            and value.get("run_id") == run_id, "original result identity differs")
    require(value.get("status") in ("unknown", "validated") and type(value.get("errors")) is list
            and type(value.get("warnings")) is list, "original result status differs")
    require(type(value.get("source_sha256")) is dict and value.get("whole_workflow_hidden_call_completeness_proven") is False,
            "original result provenance differs")
    checked_map(value.get("exact_response_evidence"))
    measurement = value.get("measurement"); require(type(measurement) is dict, "measurement missing")
    status = measurement.get("status"); bounds = measurement.get("bounds")
    require(status in ("exact", "bounded", "unbounded_accounting_gap", "ineligible"), "measurement status differs")
    if status == "ineligible": require(bounds is None, "ineligible bounds must stay null")
    else:
        require(type(bounds) is dict and set(bounds) == set(METRICS), "measurement metric scope differs")
        for bound in bounds.values():
            require(type(bound) is dict and set(bound) == {"lower", "upper"}, "bound shape invalid")
            lower, upper = bound["lower"], bound["upper"]
            require(type(lower) is int and lower >= 0, "lower endpoint invalid")
            if status == "unbounded_accounting_gap": require(upper is None, "unbounded endpoint must stay null")
            else: require(type(upper) is int and upper >= lower, "upper endpoint invalid")
            if status == "exact": require(lower == upper, "exact endpoints differ")


def declared_outputs(output_root):
    root = Path(output_root)
    paths = {"attempt": root / "ACCOUNTING-ORIGINAL-ATTEMPT.json", "final": root / "COMMON-ACCOUNTING-ORIGINAL.json",
             "ownership": root / "COMMON-ACCOUNTING-ORIGINAL-RESPONSE-OWNERSHIP.json"}
    for run_id in NEW_IDS:
        directory = root / run_id; worker = directory / "worker"
        paths[run_id + ":full"] = directory / "COMMON-ACCOUNTING-ORIGINAL-FULL.json"
        paths[run_id + ":projection"] = directory / "COMMON-ACCOUNTING-ORIGINAL.json"
        for name in ("INPUT.json", "RESULT.json", "stdout", "stderr", "EXECUTION.json"):
            paths[run_id + ":worker:" + name] = worker / name
    return {key: str(path) for key, path in paths.items()}


def publication_preflight(output_root):
    root = exact_path(output_root, directory=True)
    paths = {key: Path(path) for key, path in declared_outputs(root).items()}
    directories = []
    for run_id in NEW_IDS:
        directory = root / run_id; worker = directory / "worker"
        directories.append(directory); directories.append(worker)
    for path in paths.values():
        require(not os.path.lexists(path), "reserved output already exists")
    for directory in directories:
        if os.path.lexists(directory): exact_path(directory, directory=True)
    for directory in directories:
        if not directory.exists(): directory.mkdir()
    handles = {}
    try:
        for key, path in paths.items():
            if ":worker:" not in key: handles[key] = exclusive(path)
    except BaseException:
        for fd in handles.values(): os.close(fd)
        raise
    return paths, handles


def publish_result(fd, value):
    raw = canonical(value); write_fd(fd, raw)
    return digest(raw)


def preflight(scope_path, expected_scope_sha256):
    scope_path = exact_path(scope_path); raw = read_exact(scope_path, expected_scope_sha256); scope = decode(raw)
    require(scope.get("schema") == "work-leaf-c08-original-accounting-once-v1", "scope schema differs")
    require(scope.get("new_run_ids") == NEW_IDS and scope.get("baseline_run_ids") == BASELINE_IDS, "fixed population differs")
    require(type(scope.get("per_run_timeout_seconds")) is int and scope["per_run_timeout_seconds"] == 600, "fixed deadline differs")
    pins = dict(scope["source_sha256"]); pins[str(scope_path)] = expected_scope_sha256
    for source, expected in pins.items(): exact_path(source); sha_value(expected)
    frozen = remember_ref(scope["phase_manifest"], pins)
    require(type(frozen.get("files")) is list, "frozen file census missing")
    file_paths = set()
    for row in frozen["files"]:
        path = str(exact_path(row["path"])); require(path not in file_paths, "duplicate frozen file")
        file_paths.add(path); expected = sha_value(row["sha256"])
        require(path not in pins or pins[path] == expected, "frozen source pin conflicts"); pins[path] = expected
    schedule = remember_ref(scope["schedule"], pins)
    require(scope["schedule"]["sha256"] == frozen.get("schedule_sha256")
            and canonical(schedule) == canonical({"runs": frozen.get("schedule"), "launch_order": frozen.get("launch_order"),
                                                  "randomization": frozen.get("randomization")}), "frozen schedule differs")
    planned = {row["run_id"]: row for row in schedule["runs"]}
    require(len(schedule["runs"]) == 3 and set(planned) == set(NEW_IDS), "scheduled population differs")
    score = remember_ref(scope["score_manifest"], pins); closed = remember_ref(scope["phase_result"], pins)
    require(score.get("phase_manifest") == scope["phase_manifest"]["path"]
            and score.get("phase_manifest_sha256") == closed.get("manifest_sha256") == scope["phase_manifest"]["sha256"], "phase closure manifest differs")
    identity(closed.get("finished_at"))
    entries = score.get("runs"); require(type(entries) is list and len(entries) == 3, "terminal population differs")
    by_id = {entry.get("id", entry.get("run_id")): entry for entry in entries}
    require(set(by_id) == set(NEW_IDS) and canonical(entries) == canonical(closed.get("runs")), "terminal run rows differ")
    require(set(scope["terminals"]) == set(NEW_IDS), "terminal receipt population differs")
    for run_id in NEW_IDS:
        entry = by_id[run_id]
        declaration = planned[run_id]
        require(all(canonical(entry.get(key)) == canonical(declaration.get(key)) for key in ("condition", "artifact", "report"))
                and all(canonical(value) == canonical(entry[key]) for key, value in declaration.items() if key in entry),
                "terminal row differs from schedule")
        require(entry.get("condition") == "automatic-changed-refresh-full" and entry.get("launch_status") == "completed"
                and type(entry.get("launcher_exit_code")) is int, "run is not terminal")
        start = datetime.fromisoformat(identity(entry.get("started_at")))
        end = datetime.fromisoformat(identity(entry.get("finished_at")))
        require(start <= end, "terminal chronology differs")
        require(canonical(remember_ref(scope["terminals"][run_id], pins)) == canonical(entry), "terminal receipt differs")
    helper = exact_path(scope["helper"]["path"])
    require(scope["helper"]["sha256"] == HELPER_SHA, "original helper pin differs")
    helper_raw = read_exact(helper, HELPER_SHA)
    matches = [row for row in frozen["files"] if row["path"] == str(helper)]
    require(len(matches) == 1 and matches[0].get("role") == "frozen-evidence", "helper manifest role differs")
    for relative, expected in DEPENDENCY_PINS.items():
        path = (helper.parent / relative).resolve()
        require(pins.get(str(path)) == expected, "dependency absent from predeclared source union"); read_exact(path, expected)
    require(str(Path(__file__).resolve()) in pins and str(Path(sys.executable).resolve()) in pins, "orchestrator/interpreter identity missing")
    sessions_root = exact_path(scope["sessions_root"], directory=True)
    output_root = exact_path(scope["output_root"], directory=True)
    require(output_root == Path(scope["phase_manifest"]["path"]).parent / "postcapture", "output root differs from phase")
    outputs = declared_outputs(output_root)
    require(canonical(scope.get("outputs")) == canonical(outputs), "output reservation declaration differs")
    baselines = scope["baselines"]
    require(type(baselines) is list and [row.get("run_id") for row in baselines] == BASELINE_IDS, "baseline population differs")
    retained = []; maps = {}
    for baseline in baselines:
        run_id = baseline["run_id"]; receipt = remember_ref(baseline["receipt"], pins)
        original_entry = receipt["original_entry"]
        require(original_entry.get("id", original_entry.get("run_id")) == run_id, "baseline receipt ownership differs")
        supplement = remember_ref(baseline["response_supplement"], pins)
        require(supplement.get("schema") == "work-leaf-admitted-provider-ledger-execution-v1"
                and supplement["result"].get("schema") == "work-leaf-provider-ledger-qualification-v1", "baseline supplement schema differs")
        original = supplement["result"]["original"]
        owner = supplement["original_entry"]
        require(original.get("run_id") == run_id and owner.get("id", owner.get("run_id")) == run_id,
                "baseline supplement ownership differs")
        evidence = checked_map(supplement.get("response_evidence")); projection = receipt["accounting"]["exact_response_evidence"]
        count = baseline["expected_response_count"]
        evidence_sha = digest(canonical(evidence))
        for projected in (projection, original["exact_response_evidence"]):
            require(type(count) is int and count >= 0 and type(projected.get("count")) is int
                    and len(evidence) == count == projected["count"] and evidence_sha == projected.get("sha256"), "baseline response identity projection differs")
        retained.append({"run_id": run_id, "kind": "retained", "execution_status": "reused_without_call",
                         "original_entry": original_entry, "accounting": receipt["accounting"],
                         "receipt": baseline["receipt"], "response_supplement": baseline["response_supplement"]})
        maps[run_id] = evidence
    # Baseline identity failure prevents any new computation, not six reruns.
    prior_ids = set()
    for evidence in maps.values():
        require(not prior_ids.intersection(evidence), "baseline response identity reused")
        prior_ids.update(evidence)
    output_paths = set(outputs.values())
    worker_roots = [output_root / run / "worker" for run in NEW_IDS]
    require(not any(path in output_paths or any(Path(path).is_relative_to(worker) for worker in worker_roots)
                    for path in pins), "source/output namespaces overlap")
    verify(pins)
    return scope, pins, frozen, [by_id[run] for run in NEW_IDS], helper, helper_raw, sessions_root, output_root, retained, maps


def execute(scope_path, expected_scope_sha256):
    scope, pins, frozen, entries, helper, raw, sessions, root, retained, maps = preflight(scope_path, expected_scope_sha256)
    require(scope.get("execution_authorized") is True, "scope is not authorized for execution")
    paths, handles = publication_preflight(root)
    result = {"schema": "work-leaf-c08-original-accounting-publication-v1", "scope_sha256": expected_scope_sha256,
        "runs": [], "integrity_errors": [], "identity_errors": [], "publication_errors": [], "execution_errors": [],
        "response_ownership": {}, "accounting_call_attempts": [], "baseline_accounting_calls": 0,
        "whole_workflow_hidden_call_completeness_proven": False}
    def publication(key, value):
        try:
            actual = publish_result(handles[key], value)
            return {"path": str(paths[key]), "sha256": actual}
        except (OSError, ValueError, TypeError) as error:
            result["publication_errors"].append({"path": str(paths[key]), "error_type": type(error).__name__})
            return None
    blocked = None
    try:
        write_fd(handles["attempt"], canonical({"scope_sha256": expected_scope_sha256, "new_run_ids": NEW_IDS,
            "baseline_run_ids": BASELINE_IDS, "pid": os.getpid(), "started_at": datetime.now().astimezone().isoformat(),
            "retry_allowed": False, "outputs": {key: str(path) for key, path in paths.items()}}))
        for entry in entries:
            run_id = entry.get("id", entry.get("run_id"))
            row = {"run_id": run_id, "kind": "new", "original_entry": entry, "accounting": None,
                   "execution_status": blocked or "not_called"}
            result["runs"].append(row)
            if blocked:
                row["receipt"] = publication(run_id + ":projection", {"scope_sha256": expected_scope_sha256, "kind": "run", **row})
                continue
            try: verify(pins)
            except (OSError, ValueError) as error:
                blocked = "not_called_after_source_failure"; row["execution_status"] = blocked
                result["integrity_errors"].append(type(error).__name__)
                row["receipt"] = publication(run_id + ":projection", {"scope_sha256": expected_scope_sha256, "kind": "run", **row})
                continue
            operation_root = root / run_id / "worker"
            result["accounting_call_attempts"].append(run_id)
            try:
                full = invoke_accounting(helper, raw, entry, frozen, sessions, 600, operation_root)
                row["execution_status"] = "returned"
                row["full_result"] = publication(run_id + ":full", full)
                validate_result(full, run_id)
                row["accounting"] = project(full)
                maps[run_id] = full["exact_response_evidence"]
                for path, expected in full["source_sha256"].items():
                    require(pins.get(path) == sha_value(expected), "returned source absent from declared union")
                    read_exact(path, expected)
            except WorkerFailure as error:
                row["execution_status"] = error.kind
                row["worker_execution"] = {"closed": error.closed, "exit_code": error.exit_code, **error.evidence}
                result["execution_errors"].append({"run_id": run_id, "kind": error.kind, "closed": error.closed})
                if not error.closed: blocked = "not_called_after_worker_uncertainty"
            except (OSError, ValueError, TypeError, KeyError, RuntimeError) as error:
                result["integrity_errors"].append({"run_id": run_id, "error_type": type(error).__name__})
                blocked = "not_called_after_source_failure"
            try: verify(pins)
            except (OSError, ValueError) as error:
                result["integrity_errors"].append({"run_id": run_id, "error_type": type(error).__name__})
                blocked = "not_called_after_source_failure"
            row["receipt"] = publication(run_id + ":projection", {"scope_sha256": expected_scope_sha256, "kind": "run", **row})
        result["runs"].extend(retained)
        for run_id in NEW_IDS + BASELINE_IDS:
            for response_id, record in maps.get(run_id, {}).items():
                owner = {"run_id": run_id, "thread_id": record["thread_id"], "turn_id": record["turn_id"]}
                if response_id in result["response_ownership"]:
                    result["identity_errors"].append({"response_id": response_id, "first_run": result["response_ownership"][response_id]["run_id"], "second_run": run_id})
                else: result["response_ownership"][response_id] = owner
        try: verify(pins)
        except (OSError, ValueError) as error: result["integrity_errors"].append(type(error).__name__)
        result["source_sha256"] = pins
        result["ownership_receipt"] = publication("ownership", {"scope_sha256": expected_scope_sha256,
            "response_ownership": result["response_ownership"], "identity_errors": result["identity_errors"],
            "returned_maps_are_not_hidden_call_completeness_proof": True})
        publication("final", result)
        return result
    finally:
        for fd in handles.values(): os.close(fd)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scope", type=Path, required=True); parser.add_argument("--scope-sha256", required=True)
    args = parser.parse_args()
    result = execute(args.scope, args.scope_sha256)
    print(json.dumps({"runs_retained": len(result["runs"]), "new_attempts": len(result["accounting_call_attempts"]),
                      "integrity_error_count": len(result["integrity_errors"]), "identity_error_count": len(result["identity_errors"]),
                      "publication_error_count": len(result["publication_errors"]), "execution_error_count": len(result["execution_errors"])}))
    return int(any(result[key] for key in ("integrity_errors", "identity_errors", "publication_errors", "execution_errors")))


if __name__ == "__main__":
    try: sys.exit(main())
    except (ValueError, OSError, TypeError, KeyError) as error:
        print(json.dumps({"status": "preflight_failed", "error_type": type(error).__name__}), file=sys.stderr)
        sys.exit(2)
