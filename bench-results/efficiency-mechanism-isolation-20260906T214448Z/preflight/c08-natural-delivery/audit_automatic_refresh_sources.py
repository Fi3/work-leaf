#!/usr/bin/env python3
"""Finite closed C08 source joins. No provider, analysis or accounting command."""
import argparse
from collections import Counter, defaultdict
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import types

HERE = Path(__file__).resolve().parent
STUDY = HERE.parents[1]
FRAME = HERE.parent / "review-evidence-native-diagnostic-001/postcapture/observer-frame-correction/audit_observer_frames.py"
OLD = FRAME.parents[2] / "infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z"
COLLECTOR_SHA = "d3fd8c80746bf4bce565cb5f0a2e2eab29681b3aa40f89196cf95ebf344ed9ee"
PRIMITIVE_SHA = "34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257"
EVENT_SHA = "c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f"
JOIN_SHA = "614ca2faab8790a24ec02b20cec5dcb2997ed7ae79273ec86e5304570565a402"
FRAME_SHA = "ccfb4cc3fe2a24c6496f4a749e6c95f147d6b5fefa7fd4e91dedc6f40be1961f"
CONDITION = "automatic-changed-refresh-full"
RUNS = tuple(f"automatic-refresh-01-workflow-{n:03d}" for n in (1, 2, 3))
MAX_ROWS = 100000
MAX_BYTES = 32 * 1024 * 1024
LIMITS = {"max_file_bytes": 1024**3, "max_total_source_bytes": 32*1024**3, "max_sources": 20000,
          "max_output_rows": MAX_ROWS, "max_output_bytes": MAX_BYTES}
KINDS = {"app-server": "app-server", "exec-json": "exec-json", "locked-command": "locked-commands"}


class Invalid(ValueError):
    pass


def require(value, code):
    if not value:
        raise Invalid(code)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":"), allow_nan=False).encode()


def equal(a, b):
    if type(a) is not type(b): return False
    if type(a) is dict: return a.keys() == b.keys() and all(equal(a[k], b[k]) for k in a)
    if type(a) is list: return len(a) == len(b) and all(equal(x, y) for x, y in zip(a, b))
    return a == b


def identity(value):
    require(type(value) is str and bool(value), "identity-shape")
    value.encode("utf-8")
    return value


def integer(value):
    require(type(value) is int and value >= 0, "integer-shape")
    return value


def rpc(frame):
    value = frame.get("id")
    require(type(value) in (str, int) and (type(value) is int or bool(value)), "rpc-identity")
    return type(value).__name__, value


def source_module(path, expected, sources=None):
    require(path.is_absolute() and path.resolve() == path, "code-path")
    if sources is None:
        before = path.stat()
        require(stat.S_ISREG(before.st_mode) and before.st_size <= 1024 * 1024, "bootstrap-size")
        with path.open("rb") as stream: raw = stream.read(1024 * 1024 + 1)
        require(len(raw) <= 1024 * 1024, "bootstrap-size")
    else:
        raw = sources.read({"path": str(path), "sha256": expected})
    require(sha(raw) == expected, "code-source-pin")
    module = types.ModuleType("closed_" + path.stem); module.__file__ = str(path)
    exec(compile(raw, str(path), "exec"), module.__dict__)
    return module


def dependencies(sources):
    for path, expected in ((OLD / "audit_review_evidence_sources.py", COLLECTOR_SHA),
                           (OLD / "audit_review_evidence.py", PRIMITIVE_SHA)):
        sources.read({"path": str(path), "sha256": expected})
    primitive = sources.read({"path": str(STUDY / "audit_review_evidence.py"), "sha256": PRIMITIVE_SHA})
    event = sources.read({"path": str(HERE / "validate_automatic_refresh.py"), "sha256": EVENT_SHA})
    frame = source_module(FRAME, FRAME_SHA, sources)
    join = source_module(HERE / "join_automatic_refresh_delivery.py", JOIN_SHA, sources)
    return frame, join, primitive, event


def directory_names(path, maximum):
    require(path.is_absolute() and path.resolve() == path and path.is_dir(), "directory-path")
    names = set()
    with os.scandir(path) as entries:
        for entry in entries:
            require(len(names) < maximum and entry.is_dir(follow_symlinks=False), "directory-population-type-or-limit")
            names.add(entry.name)
    return names


def check_limits(scope):
    require(equal(scope.get("limits"), LIMITS), "scope-source-output-limits")
    return dict(LIMITS)


def collect_closed_captures(manifest, sources, collector, frame_helper):
    result = {"errors": [], "frames": [], "captures": [], "invocations": [], "directories": {}}
    directories = {}
    try:
        observation = Path(manifest["invocations"]["path"]).parent
        rows = sources.records(manifest["invocations"]); logged = {}
        for _, row in rows:
            key = identity(row.get("invocation_id")); require(key not in logged, "duplicate-invocation")
            logged[key] = row
        expected_apps = {key for key, row in logged.items() if row.get("capture_kind") == "app-server"}
        for path, expected in ((observation / "invocations", set(logged)), (observation / "app-server", expected_apps)):
            actual = directory_names(path, collector.MAX_SOURCES); directories[path] = actual
            require(actual == expected, "directory-population-differs")
        require(all(row.get("capture_kind") in KINDS for row in logged.values()), "unknown-capture-kind")
        kinds = defaultdict(set)
        for key, row in logged.items(): kinds[row["capture_kind"]].add(key)
        for kind in ("exec-json", "locked-command"):
            path = observation / KINDS[kind]
            if path.exists() or kinds[kind]:
                actual = directory_names(path, collector.MAX_SOURCES); directories[path] = actual
                require(actual == kinds[kind], "nonapp-directory-population")
        seen = set()
        for refs in manifest["invocation_metadata"]:
            path = Path(refs["start"]["path"]).parent; key = path.name
            require(key in logged and key not in seen and path == observation / "invocations" / key, "invocation-metadata-population")
            seen.add(key)
            for name in ("start", "child", "end"):
                require(refs[name]["path"] == str(path / (name + ".json")), "invocation-metadata-path")
            start, child, end = (sources.json(refs[name]) for name in ("start", "child", "end"))
            require(start.get("invocation_id") == end.get("invocation_id") == child.get("invocation_id") == key, "invocation-metadata-identity")
            require(integer(start["start_monotonic_ns"]) <= integer(child["started_monotonic_ns"]) <= integer(end["end_monotonic_ns"]), "child-lifetime")
            require(integer(start["start_unix_ns"]) <= integer(end["end_unix_ns"]), "invocation-wall-time")
            require(type(end.get("exit_code")) is int or type(end.get("terminating_signal")) is int, "invocation-not-closed")
            projected = {k: v for k, v in start.items() if k not in ("raw_response_usage", "project_layer_inventory_required")}
            require(equal({k: v for k, v in logged[key].items() if k != "end"}, projected) and equal(logged[key].get("end"), end), "logged-invocation-differs")
            result["invocations"].append({"invocation_id": key, "start": start, "child": child, "end": end})
        require(seen == set(logged), "missing-invocation-metadata")
        # Preserve the unchanged collector: its acceptance is not v7 ownership.
        captured = collector.collect_captures(manifest, sources)
        indexed = {cap["path"]: cap for cap in manifest["captures"]}
        for cap in captured:
            refs = indexed[cap["path"]]; path = Path(cap["path"])
            try:
                start, end = sources.json(refs["start"]), sources.json(refs["end"])
                require(start.get("raw_response_usage") is True and start.get("primary") is True and
                        type(start.get("provider_usage_grace_ms")) is int and start["provider_usage_grace_ms"] == 1000 and
                        start.get("provider_usage_grace_output_resume") == "forward", "raw-grace-settings")
                for name, filename in (("settings", "raw-response-usage.json"), ("journal", "raw-response-rewrites.jsonl"), ("grace", "provider-usage-grace.jsonl")):
                    require(refs[name]["path"] == str(path / filename) and end.get("raw_response_usage_sha256", {}).get(filename) == refs[name]["sha256"], "raw-extra-path-or-end-pin")
                original, forwarded = sources.read(refs["clients"]), sources.read(refs["forwarded"])
                proof = frame_helper.prove_frames(original, forwarded,
                    [row for _, row in sources.records(refs["journal"])], sources.json(refs["settings"]))
                require(frame_helper.matches_proof(proof, cap["clients"], cap["forwarded"]), "decoded-frame-proof-differs")
                grace = sources.records(refs["grace"])
                result["frames"].append({"capture_id": path.name, **proof, "grace_record_count": len(grace)})
                decoded = {"capture_id": path.name,
                    "clients": list(zip(cap["client_lines"], cap["clients"])),
                    "forwarded": sources.records(refs["forwarded"]), "servers": list(zip(cap["server_lines"], cap["servers"]))}
                closure = check_turn_closure(decoded, grace)
                result["frames"][-1]["turn_closure"] = closure
                result["errors"].extend(closure["errors"])
                result["captures"].append(decoded)
            except (ValueError, KeyError, TypeError, OSError, UnicodeError):
                result["errors"].append({"code": "capture-frame-source-invalid", "capture_id": path.name})
    except (ValueError, KeyError, TypeError, OSError, UnicodeError):
        result["errors"].append({"code": "capture-population-or-source-invalid"})
    for path, before in directories.items():
        try: require(directory_names(path, collector.MAX_SOURCES) == before, "directory-endpoint-drift")
        except (ValueError, OSError): result["errors"].append({"code": "directory-endpoint-invalid", "path": str(path)})
        result["directories"][str(path)] = sorted(before)
    return result


def check_invocation_streams(refs, invocation, observation, sources):
    result = {"errors": [], "invocation_id": invocation["invocation_id"]}
    try:
        start, end = invocation["start"], invocation["end"]
        kind = start["capture_kind"]; path = observation / KINDS[kind] / invocation["invocation_id"]
        names = ("client-to-server.raw", "server-to-client.raw", "server-stderr.raw") if kind == "app-server" else ("stdin.raw", "stdout.raw", "stderr.raw")
        for key, name in zip(("stdin", "stdout", "stderr"), names):
            ref = refs["streams"][key]
            require(ref["path"] == str(path / name) and ref["sha256"] == end[key + "_sha256"], "invocation-stream-pin")
            sources.read(ref)
        require(refs["meta"]["path"] == str(path / "meta.json"), "capture-meta-path")
        require(equal(sources.json(refs["meta"]), {"start": start, "end": end}), "capture-meta-identity")
    except (ValueError, KeyError, TypeError, OSError): result["errors"].append("invocation-stream-or-meta-invalid")
    return result


def check_native_membership(captures, native_sources, expected):
    result = {"errors": [], "threads": [], "contexts": []}; accepted = set(); turns = set(); by_turn = defaultdict(set)
    try:
        for cap in captures:
            requests = {}; replies = defaultdict(list)
            for _, row in cap["clients"]:
                if "id" not in row: continue
                key = rpc(row); require(key not in requests, "duplicate-request")
                requests[key] = row
            for _, row in cap["servers"]:
                if "id" in row and "method" not in row: replies[rpc(row)].append(row)
            for key, request in requests.items():
                if request.get("method") not in ("thread/start", "thread/resume", "turn/start"): continue
                answers = replies.get(key, []); require(len(answers) == 1, "missing-or-duplicate-membership-reply")
                reply = answers[0]; require(not ("error" in reply and "result" in reply), "mixed-membership-reply")
                if "error" in reply: continue
                value = reply["result"]; require(type(value) is dict, "membership-result")
                if request["method"] in ("thread/start", "thread/resume"):
                    accepted.add(identity(value["thread"]["id"]))
                else:
                    thread = identity(request["params"]["threadId"]); turn = identity(value["turn"]["id"])
                    require((thread, turn) not in turns, "duplicate-accepted-turn")
                    accepted.add(thread); turns.add((thread, turn)); by_turn[thread].add(turn)
        by_thread = {}; source_names = set()
        for source in native_sources:
            thread = identity(source["thread_id"]); name = identity(source["source"])
            require(thread not in by_thread and name not in source_names, "duplicate-native-source")
            by_thread[thread] = source; source_names.add(name)
        require(set(by_thread) == accepted, "native-thread-population")
        for thread, source in by_thread.items():
            session = []; contexts = {}; previous = 0
            for line, row in source["rows"]:
                require(type(line) is int and line > previous and type(row) is dict, "native-physical-rows"); previous = line
                payload = row.get("payload"); require(type(payload) is dict, "native-payload")
                if row.get("type") == "session_meta": session.append(payload)
                if row.get("type") != "turn_context": continue
                turn = identity(payload.get("turn_id"))
                require((thread, turn) in turns, "unaccepted-native-context")
                require(turn not in contexts or equal(contexts[turn], payload), "conflicting-native-context")
                for key in ("cwd", "model", "effort"): require(equal(payload.get(key), expected[key]), "native-context-setting")
                contexts[turn] = payload
            require(len(session) == 1 and session[0].get("id") == thread, "native-session")
            for key in ("cwd", "cli_version"): require(equal(session[0].get(key), expected[key]), "native-session-setting")
            required = by_turn[thread]
            require(set(contexts) == required, "native-context-population")
            result["threads"].append({"thread_id": thread, "source": source["source"], "accepted_turn_count": len(required)})
            result["contexts"].extend({"thread_id": thread, "turn_id": turn} for turn in contexts)
    except (ValueError, KeyError, TypeError, UnicodeError): result["errors"].append("native-membership-or-context-invalid")
    return result


def check_turn_closure(capture, grace):
    result = {"errors": [], "turns": [], "interrupts": []}
    try:
        replies = defaultdict(list); terminals = defaultdict(list); requests = {}; interrupts = []
        for line, row in capture["servers"]:
            if "id" in row and "method" not in row: replies[rpc(row)].append(row)
            if row.get("method") not in ("turn/completed", "turn/interrupted", "turn/failed"): continue
            params = row["params"]; thread = identity(params.get("threadId"))
            nested = params.get("turn", {})
            require(type(nested) is dict, "terminal-turn-shape")
            direct, embedded = params.get("turnId"), nested.get("id")
            require(direct is None or embedded is None or equal(direct, embedded), "terminal-turn-conflict")
            turn = identity(direct if direct is not None else embedded)
            terminals[thread, turn].append({"server_line": line, "method": row["method"], "status": nested.get("status")})
        accepted = set()
        for line, row in capture["clients"]:
            if "id" not in row: continue
            key = rpc(row); require(key not in requests, "duplicate-closure-request"); requests[key] = row
            method = row.get("method")
            if method not in ("turn/start", "turn/interrupt"): continue
            answers = replies.get(key, []); require(len(answers) == 1, "missing-closure-reply")
            reply = answers[0]; require(not ("result" in reply and "error" in reply), "mixed-closure-reply")
            if "error" in reply:
                require(method != "turn/interrupt", "interrupt-rejected")
                continue
            require(type(reply.get("result")) is dict, "closure-result")
            thread = identity(row["params"].get("threadId"))
            if method == "turn/start":
                turn = identity(reply["result"]["turn"]["id"])
                require((thread, turn) not in accepted, "duplicate-closure-turn"); accepted.add((thread, turn))
            else:
                turn = identity(row["params"].get("turnId")); interrupts.append((line, thread, turn))
        require(set(terminals) == accepted and all(len(v) == 1 for v in terminals.values()), "terminal-turn-population")
        result["turns"] = [{"thread_id": t, "turn_id": u, **values[0]} for (t, u), values in terminals.items()]
        require(len(grace) == len(interrupts), "grace-occurrence-population")
        allowed = {"not-eligible", "forwarded-after-exact-usage", "forwarded-after-resumed-output-usage",
                   "forwarded-after-output-resumed", "forwarded-after-turn-completed", "forwarded-after-timeout"}
        for (line, thread, turn), (grace_line, row) in zip(interrupts, grace):
            require((thread, turn) in accepted and type(row) is dict and set(row) == {"thread_id", "turn_id", "configured_grace_ms", "output_resume_policy", "waited_ms", "outcome"}, "grace-shape")
            require(row["thread_id"] == thread and row["turn_id"] == turn and type(row["configured_grace_ms"]) is int and
                    row["configured_grace_ms"] == 1000 and row["output_resume_policy"] == "forward" and row["outcome"] in allowed, "grace-identity-setting")
            integer(row["waited_ms"])
            result["interrupts"].append({"client_line": line, "grace_line": grace_line, **row})
    except (ValueError, KeyError, TypeError, UnicodeError): result["errors"].append("turn-or-grace-closure-invalid")
    return result


def check_project_inventory(refs, sources, config, invocations, published_observation=None):
    result = {"errors": [], "snapshots": []}; root = None; before = None
    def inventory_files(directory):
        require(directory.is_absolute() and directory.resolve() == directory and directory.is_dir(), "project-directory")
        names = set()
        with os.scandir(directory) as children:
            for child in children:
                require(len(names) < 66 and child.is_file(follow_symlinks=False), "project-file-population")
                names.add(child.name)
        return names
    try:
        root = Path(refs["manifest"]["path"]).parent
        observation = Path(published_observation if published_observation is not None else config["root"])
        require(root == observation / "project-layer-inventory" and refs["manifest"]["path"] == str(root / "manifest.jsonl") and
                refs["baseline"]["path"] == str(root / "pre-spawn-baseline.json"), "project-source-path")
        before = inventory_files(root)
        names = {"manifest.jsonl", "pre-spawn-baseline.json"}
        snapshots = []
        for ref in refs["snapshots"]:
            path = Path(ref["path"])
            require(path.parent == root and path.name not in names, "project-snapshot-path")
            names.add(path.name); snapshots.append(sources.json(ref))
        require(before == names, "project-source-population")
        rows = sources.records(refs["manifest"]); baseline = sources.json(refs["baseline"])
        require(0 < len(rows) <= 64 and Counter(sha(canonical(row)) for _, row in rows) == Counter(sha(canonical(row)) for row in snapshots), "project-manifest-snapshot-bijection")
        pre = defaultdict(list)
        for line, row in rows:
            require(row.get("schema") == "work-leaf-project-layer-inventory-v1" and row.get("run_id") == config["run_id"] and row.get("study_id") == config["study_id"], "project-identity")
            require(type(row.get("entries")) is list and row.get("inventory_sha256") == sha(canonical(row["entries"])), "project-entry-digest")
            require(row.get("valid") is True and row.get("errors") == [] and row.get("matches_pre_spawn") is not False, "project-original-invalid")
            require(row.get("repository") == baseline.get("repository") and row["inventory_sha256"] == baseline.get("inventory_sha256"), "project-boundary-drift")
            for key in ("started_monotonic_ns", "completed_monotonic_ns"):
                require(type(row.get(key)) is str and re.fullmatch(r"[0-9]+", row[key]), "project-time-type")
            require(int(row["started_monotonic_ns"]) <= int(row["completed_monotonic_ns"]), "project-time-order")
            if row.get("label") == "pre-spawn": pre[row.get("invocation_id")].append(row)
            result["snapshots"].append({"manifest_line": line, "label": row.get("label"), "invocation_id": row.get("invocation_id"),
                "inventory_sha256": row["inventory_sha256"], "record_sha256": sha(canonical(row)), "entry_count": len(row["entries"])})
        first = next((row for _, row in rows if row.get("label") == "pre-spawn"), None)
        require(first is not None and equal(first, baseline), "project-baseline-copy")
        for invocation in invocations:
            start = invocation["start"]
            if start.get("capture_kind") != "app-server" or start.get("primary") is not True: continue
            require(start.get("project_layer_inventory_required") is True, "project-start-required")
            candidates = pre.get(invocation["invocation_id"], [])
            require(len(candidates) == 1 and candidates[0]["repository"] == start.get("cwd") and
                    int(candidates[0]["completed_monotonic_ns"]) <= integer(invocation["child"]["started_monotonic_ns"]), "project-before-child")
    except (ValueError, KeyError, TypeError, OSError, UnicodeError): result["errors"].append("project-inventory-invalid")
    if root is not None and before is not None:
        try: require(inventory_files(root) == before, "project-directory-drift")
        except (ValueError, OSError): result["errors"].append("project-directory-endpoint-invalid")
    return result


def check_phase_documents(run, phase, result, score, phase_ref, collector):
    require(run in RUNS and phase.get("phase") == "automatic-refresh-01", "phase-run-scope")
    require(result.get("phase") == score.get("phase") == phase["phase"] and
            result.get("manifest_sha256") == score.get("phase_manifest_sha256") == phase_ref["sha256"] and
            score.get("phase_manifest") == phase_ref["path"], "phase-source-identity")
    def index(rows, key):
        require(type(rows) is list, "phase-rows")
        values = {}
        for row in rows:
            require(type(row) is dict and row.get(key) in RUNS and row[key] not in values, "phase-row-identity")
            values[row[key]] = row
        require(set(values) == set(RUNS), "phase-population")
        return values
    scheduled, ended, scored = index(phase["schedule"], "run_id"), index(result["runs"], "id"), index(score["runs"], "id")
    terminals = {}
    fields = ("run_id", "condition", "phase", "block_id", "wave", "workflow", "artifact", "report", "prompt_trace", "experiment_manifest")
    for key in RUNS:
        require(scheduled[key].get("condition") == CONDITION and type(scheduled[key].get("wave")) is int and scheduled[key]["wave"] == 1, "phase-condition-wave")
        require(all(equal(scheduled[key].get(field), ended[key].get(field)) for field in fields) and equal(ended[key], scored[key]), "phase-terminal-score-mismatch")
        terminals[key] = collector.terminal(ended[key], key, CONDITION)
    return {"entry": scheduled[run], "terminal": terminals[run], "all_terminal": terminals}


def check_publication_root(original, entry, published):
    parent = Path(entry["results_dir"]); run = entry["run_id"]
    require(parent.is_absolute() and parent.resolve() == parent and run in RUNS, "publication-parent")
    require(entry["artifact"] == str(parent / (run + "-three-feature-bench-artifacts")) and published == Path(entry["artifact"]) / "observation", "publication-final-path")
    old = Path(original)
    require(type(original) is str and old.is_absolute() and str(old) == original and old.name == "observation" and old.parent.parent == parent and
            re.fullmatch(r"\.bench-artifact-publish\.[0-9a-f]{32}", old.parent.name), "publication-original-path")
    return {"original_observation_root": original, "published_observation_root": str(published),
            "qualification": "pinned-driver-publication-contract; not independent historical inode proof"}


def finish_result(result, max_rows=MAX_ROWS, max_bytes=MAX_BYTES):
    raw = canonical(result); digest = sha(raw)
    count = 0; stack = [result]
    while stack:
        value = stack.pop()
        if type(value) is list: count += len(value); stack.extend(value)
        elif type(value) is dict: stack.extend(value.values())
    require(type(max_rows) is int and 0 <= max_rows <= MAX_ROWS and type(max_bytes) is int and 0 < max_bytes <= MAX_BYTES, "output-bound-declaration")
    require(count <= max_rows and len(raw) <= max_bytes, "output-limit-no-clipping")
    result.update(full_result_sha256=digest, full_result_bytes=len(raw), metadata_list_entries=count)
    require(len(canonical(result)) + 1 <= max_bytes, "published-output-limit-no-clipping")
    return result


def preserve_original_report(item, sources, collector):
    raw = sources.read(item["source"])
    saved = {"kind": item["kind"], "source": item["source"], "bytes": len(raw)}
    if item["format"] == "json":
        report = collector.decode(raw)
        require(type(report) in (dict, list), "original-report-shape")
        if type(report) is list:
            saved.update(json_type="array", record_count=len(report))
        else:
            saved["flags"] = {key: report[key] for key in ("errors", "capture_complete", "measurement_status", "status", "exit_code") if key in report}
    elif item["format"] == "jsonl": saved["record_count"] = len(sources.records(item["source"]))
    else: require(item["format"] == "text", "original-report-format")
    return saved


def check_generated_profile(refs, sources, config, project, artifact, report, approved):
    renderer = refs["renderer"]
    require(renderer["sha256"] == "de9803658bc8ac41a9356436be2c8b8edfce9d7bef9176841e067b3478633d4f" and
            Path(renderer["path"]).name == "bench-agent-profile-common", "profile-renderer-source")
    sources.read(renderer)
    require(refs["profile"]["path"] == str(Path(artifact) / "agent-profile.txt") and
            refs["recursive_log"]["path"] == str(Path(artifact) / "recursive-codex-attempts.log"), "retained-profile-paths")
    raw = sources.read(refs["profile"])
    require(raw.endswith(b"\n") and len(raw) <= 16384, "profile-text-boundary")
    fields = {}
    for line in raw[:-1].decode("utf-8").split("\n"):
        key, separator, value = line.partition("=")
        require(separator and key not in fields and bool(value), "profile-fields")
        fields[key] = value
    require(set(fields) == {"agent_model", "agent_reasoning_effort", "actual_codex", "actual_codex_sha256",
        "profiled_codex", "profiled_codex_sha256", "codex_cli_version", "recursive_codex_attempt_log"}, "profile-field-population")
    require(fields["agent_model"] == "gpt-5.5" and fields["agent_reasoning_effort"] == "xhigh" and
            fields["codex_cli_version"] == config["real_codex_version"] == report["codex_cli_version"] == "codex-cli 0.153.4", "profile-model-cli")
    for key in ("actual_codex", "profiled_codex", "recursive_codex_attempt_log"):
        value = fields[key]
        # Only Bash %q's unchanged ASCII-path subset is supported; no shell runs.
        require(re.fullmatch(r"/[A-Za-z0-9_./-]+", value) and str(Path(value)) == value and ".." not in Path(value).parts, "profile-path-quoting")
    generated = str(Path(project).parent / "codex-profile" / "codex")
    log = str(Path(generated).parent / "recursive-codex-attempts.log")
    require(fields["profiled_codex"] == config["real_codex"] == generated and fields["recursive_codex_attempt_log"] == log, "generated-profile-identity")
    actual = {"path": fields["actual_codex"], "sha256": fields["actual_codex_sha256"]}
    require(approved.get(actual["path"]) == actual["sha256"] == report["actual_codex_sha256"] and
            actual["path"] == report["codex_cli_path"], "profile-actual-binary-source")
    sources.read(actual)
    rendered = ("#!/usr/bin/env bash\n"
        'if [[ "${WORK_LEAF_BENCH_CODEX_ACTIVE:-}" == "1" ]]; then\n'
        "  printf 'blocked recursive Codex launch pid=%s\\n' \"$$\" >> " + log + "\n"
        "  printf 'benchmark policy blocks recursive Codex provider launches; do not retry; continue with automated validation\\n' >&2\n"
        "  exit 86\nfi\nexport WORK_LEAF_BENCH_CODEX_ACTIVE=1\n"
        'exec ' + actual["path"] + ' -c model=\\"gpt-5.5\\" -c model_reasoning_effort=\\"xhigh\\" "$@"\n').encode()
    digest = sha(rendered)
    require(digest == fields["profiled_codex_sha256"] == config["real_codex_sha256"] == report["profiled_codex_sha256"], "generated-profile-byte-hash")
    recursive = sources.read(refs["recursive_log"])
    require(not recursive, "recorded-recursive-provider-attempt")
    return {"generated_path": generated, "rendered_sha256": digest, "rendered_bytes": len(rendered),
        "actual_binary": actual, "recursive_attempt_bytes": len(recursive), "sources": refs,
        "qualification": "source-rendered bytes bound to retained profile, report and capture; not an extant generated-file endpoint"}


def execute(input_ref, output_path=None):
    result = {"schema": "work-leaf-c08-natural-source-result-v1", "status": "unverifiable", "errors": [], "witness_provider_calls": 0}
    sources = None; directories = {}; resolutions = []
    try:
        collector = source_module(STUDY / "audit_review_evidence_sources.py", COLLECTOR_SHA)
        sources = collector.Sources()
        sources.read({"path": str(STUDY / "audit_review_evidence_sources.py"), "sha256": COLLECTOR_SHA})
        value = sources.json(input_ref)
        require(value.get("schema") == "work-leaf-c08-natural-source-input-v1", "input-schema")
        sources.read({"path": str(Path(__file__).resolve()), "sha256": value["helper_sha256"]})
        frame, join, primitive, event = dependencies(sources)
        scope = sources.json(value["scope"])
        require(scope.get("schema") == "work-leaf-c08-natural-source-scope-v1" and scope.get("run_ids") == list(RUNS) and scope.get("condition") == CONDITION, "fixed-scope")
        require(equal(scope["phase_manifest"], value["phase_manifest"]), "scope-phase-pin")
        result["declared_limits"] = check_limits(scope)
        run = identity(value["run_id"]); result["run_id"] = run
        require(run in RUNS and scope["input_paths"][run] == input_ref["path"] and scope["output_paths"][run] == output_path, "scope-input-output-path")
        phase, ended, score = (sources.json(value[key]) for key in ("phase_manifest", "phase_result", "score_manifest"))
        checked = check_phase_documents(run, phase, ended, score, value["phase_manifest"], collector)
        result.update(terminal=checked["terminal"], all_terminal=checked["all_terminal"])
        result["original_phase_flags"] = {key: ended.get(key) for key in ("signals_received", "phase_schedule_exhausted", "all_scheduled_workflows_launched", "supervisor_wall_timeout", "supervisor_error", "behavioral_config_drift_detected", "unexplained_config_drift_detected", "pending_config_attestation_at_finish", "frozen_input_integrity_errors")}
        require(phase.get("model") == "gpt-5.5" and phase.get("reasoning_effort") == "xhigh" and phase.get("provider_route") == "existing-codex-chatgpt-subscription", "phase-model-route")
        approved = {}
        for ref in [*phase["files"], *value["additional_sources"]]:
            target = {"path": ref["path"], "sha256": ref["sha256"]}
            require(target["path"] not in approved or approved[target["path"]] == target["sha256"], "conflicting-approved-source")
            approved[target["path"]] = target["sha256"]; sources.read(target)
        def executable(path, digest):
            resolved = str(Path(path).resolve())
            require(approved.get(resolved) == digest, "undeclared-executable-source")
            sources.read({"path": resolved, "sha256": digest})
            resolutions.append({"configured_path": path, "resolved_path": resolved, "sha256": digest})
        for name, expected in (("bench-candidate-common", "2971cb9729626ddc7a12bd688df8d80ee20f904bb48d52ab846c1e9f9751711f"),
                               ("bench-three-features", "d2487780c63c14021904b8a3c882d54fe231c5846f4a7f57fe955f50201f5644")):
            ref = value["publication_sources"][name]
            require(ref["sha256"] == expected and Path(ref["path"]).name == name, "publication-source-pin")
            sources.read(ref)
        entry = checked["entry"]
        for key, scheduled in (("experiment", "experiment_manifest"), ("trace", "prompt_trace")):
            require(value[key]["path"] == entry[scheduled], "scheduled-source-path")
        experiment = sources.json(value["experiment"])
        require(equal(experiment, dict(schema="work-leaf-bench-experiment-v7", run_id=run, condition=CONDITION, evidence_path=entry["prompt_trace"])), "v7-experiment")
        config = sources.json(value["observer_config"])
        observation = Path(value["invocations"]["path"]).parent
        require(value["observer_config"]["path"] == str(observation / "observer-config.json") and config.get("run_id") == run and config.get("condition") == "work-leaf", "observer-config-source")
        result["publication"] = check_publication_root(config["root"], entry, observation)
        require(config.get("model") == "gpt-5.5" and config.get("effort") == "xhigh" and config.get("real_codex_version") == "codex-cli 0.153.4", "observer-model-cli")
        result["original_reports"] = [preserve_original_report(item, sources, collector) for item in value["original_reports"]]
        reports = [item for item in value["original_reports"] if item["kind"] == "benchmark-report"]
        require(len(reports) == 1 and reports[0]["source"]["path"] == entry["report"], "published-report-source")
        report = sources.json(reports[0]["source"])
        require(report.get("run_id") == run and report.get("observation") == str(observation), "published-report-identity")
        generated = check_generated_profile(value["generated_profile"], sources, config, value["project_cwd"], entry["artifact"], report, approved)
        result["generated_profile"] = generated
        executable(generated["actual_binary"]["path"], generated["actual_binary"]["sha256"])
        for prefix in ("observer", "real_sh"):
            key = "observer_executable" if prefix == "observer" else prefix
            executable(config[key], config[prefix + "_sha256"])
        require(config["observer_executable"] == str(Path(phase["bin_dir"]) / "bench-observer"), "observer-phase-binary")
        closed = collect_closed_captures(value, sources, collector, frame)
        directories = closed["directories"]
        result["errors"].extend(closed["errors"]); result["frame_proofs"] = closed["frames"]
        metadata = {Path(row["start"]["path"]).parent.name: row for row in value["invocation_metadata"]}
        result["invocation_streams"] = []
        for invocation in closed["invocations"]:
            checked_streams = check_invocation_streams(metadata[invocation["invocation_id"]], invocation, observation, sources)
            result["invocation_streams"].append(checked_streams); result["errors"].extend(checked_streams["errors"])
            start = invocation["start"]
            if start["capture_kind"] in ("app-server", "exec-json"):
                require(start["real_executable"] == generated["generated_path"] and
                        start["real_executable_sha256"] == generated["rendered_sha256"], "invocation-generated-profile-identity")
            else:
                executable(start["real_executable"], start["real_executable_sha256"])
            if start["capture_kind"] == "app-server": require(start.get("cwd") == value["project_cwd"], "appserver-project-cwd")
        native = [{"thread_id": item["thread_id"], "source": item["source"]["path"], "rows": sources.records(item["source"])} for item in value["native_sessions"]]
        expected = {"cwd": value["project_cwd"], "cli_version": "0.153.4", "model": "gpt-5.5", "effort": "xhigh"}
        membership = check_native_membership(closed["captures"], native, expected)
        result["native_membership"] = membership; result["errors"].extend(membership["errors"])
        traced = sources.records(value["trace"])
        joined = join.join_delivery([row for _, row in traced], closed["captures"], native, run, primitive, event)
        for row in joined["trace"]: row["physical_trace_line"] = traced[row["row_index"]][0]
        result["delivery"] = joined; result["errors"].extend(joined["errors"])
        project = check_project_inventory(value["project_inventory"], sources, config, closed["invocations"], str(observation))
        result["project_inventory"] = project; result["errors"].extend(project["errors"])
    except (ValueError, KeyError, TypeError, OSError, UnicodeError) as error:
        result["errors"].append(str(error) if isinstance(error, Invalid) else "source-execution-invalid")
    if sources is not None:
        try: sources.finish()
        except (ValueError, OSError): result["errors"].append("source-endpoint-invalid")
        result["source_sha256"] = sources.hashes
        for path, names in directories.items():
            try: require(directory_names(Path(path), collector.MAX_SOURCES) == set(names), "final-directory-drift")
            except (ValueError, OSError): result["errors"].append("directory-final-endpoint-invalid")
        for resolution in resolutions:
            if str(Path(resolution["configured_path"]).resolve()) != resolution["resolved_path"]: result["errors"].append("executable-resolution-endpoint-invalid")
        result["directory_inventory"] = directories; result["executable_sources"] = resolutions
    if not result["errors"]: result["status"] = "available"
    return result


def attempt_path(output):
    return output.with_name(output.name + ".attempt.json")


def prepare_publication(input_ref, output):
    source = Path(input_ref["path"]); attempt = attempt_path(output)
    require(source.is_absolute() and source.resolve() == source and source.is_file() and not source.is_symlink(), "input-path")
    require(type(input_ref["sha256"]) is str and re.fullmatch(r"[0-9a-f]{64}", input_ref["sha256"]), "input-digest-shape")
    require(output.is_absolute() and output.parent.resolve() == output.parent and output.parent.is_dir(), "output-parent")
    require(source not in (output, attempt), "publication-source-overlap")
    for path in (output, attempt): require(not path.exists() and not path.is_symlink(), "publication-exists")
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW
    stream = os.fdopen(os.open(output, flags, 0o600), "wb")
    try:
        with os.fdopen(os.open(attempt, flags, 0o600), "wb") as marker:
            marker.write(canonical({"schema": "work-leaf-c08-source-attempt-v1", "input": input_ref, "output": str(output), "recorded_at": datetime.now(timezone.utc).isoformat(), "automatic_retry_allowed": False}) + b"\n")
            marker.flush(); os.fsync(marker.fileno())
    except BaseException:
        stream.close(); raise
    return stream


def publish_result(stream, result):
    stream.write(canonical(result) + b"\n"); stream.flush(); os.fsync(stream.fileno())


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True); parser.add_argument("--input-sha256", required=True); parser.add_argument("--output", required=True)
    args = parser.parse_args(argv); ref = {"path": args.input, "sha256": args.input_sha256}
    with prepare_publication(ref, Path(args.output)) as stream:
        result = execute(ref, args.output)
        # A pre-scope failure still needs its bounded, unqualified receipt.
        limits = result.get("declared_limits", LIMITS)
        result = finish_result(result, max_rows=limits["max_output_rows"], max_bytes=limits["max_output_bytes"])
        publish_result(stream, result)
    return 0 if result["status"] == "available" else 1


if __name__ == "__main__":
    raise SystemExit(main())
