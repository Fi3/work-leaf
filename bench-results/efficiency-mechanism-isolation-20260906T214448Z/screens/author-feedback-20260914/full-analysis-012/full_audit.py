"""Terminal-only full-workflow ledger using the retained audit core exactly once."""
import argparse
from contextlib import redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import sys
import uuid

HERE = Path(__file__).resolve().parent
STUDY = HERE.parents[2]
TEMPLATE = STUDY / "phases/standalone-global-hunk-pilot-01/postcapture/NATIVE-SUPPLEMENT-COMMAND.json"
TEMPLATE_SHA = "64f05749b9b793f4087eefba4cb81f7f4ef306a509e6c6ab690304ea242545ea"
CORE_SHA = "dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def publish(path, value):
    with Path(path).open("xb") as handle:
        handle.write(encoded(value) + b"\n")
        handle.flush()


class NativeCache:
    def __init__(self, root, core_sha):
        self.root, self.core_sha = Path(root), core_sha
        self.root.mkdir(parents=True, exist_ok=True)

    def run(self, core, rows, metadata, model, effort):
        thread = metadata["thread_id"]
        if str(uuid.UUID(thread)) != thread:
            raise ValueError("invalid native thread identity")
        identity = digest(encoded([rows, metadata, model, effort, self.core_sha]))
        claim = self.root / (thread + ".claim.json")
        result = self.root / (thread + ".result.json")
        if claim.exists():
            if json.loads(claim.read_bytes())["input_sha256"] != identity:
                raise ValueError("cached native input identity differs")
            if not result.is_file():
                raise ValueError("claimed native audit has no result; do not execute twice")
            saved = json.loads(result.read_bytes())
            if saved["input_sha256"] != identity or digest(encoded(saved["result"])) != saved["result_sha256"]:
                raise ValueError("cached native result identity differs")
            return saved["result"]
        publish(claim, {"input_sha256": identity, "core_sha256": self.core_sha})
        try:
            value = core(rows, metadata, model, effort)
        except Exception as error:
            publish(self.root / (thread + ".failure.json"),
                    {"error": type(error).__name__ + ": " + str(error), "input_sha256": identity})
            raise
        publish(result, {"input_sha256": identity, "result": value,
                         "result_sha256": digest(encoded(value))})
        return value


def require_terminal(batch, manifest):
    path = Path(batch) / "PHASE-RESULT.json"
    if not path.is_file():
        raise ValueError("actual analysis waits for every phase outcome")
    result = json.loads(path.read_bytes())
    expected = [r["run_id"] for r in manifest["schedule"]]
    rows = result.get("runs", [])
    if (result.get("collection_paused") is not True or len(rows) != len(expected)
            or {r.get("id") for r in rows} != set(expected)
            or any(type(r.get("launcher_exit_code")) is not int or not r.get("finished_at") for r in rows)):
        raise ValueError("terminal population is incomplete or foreign")
    return result


def public_lifecycle(rows):
    starts = [i for i, r in enumerate(rows) if r.get("type") == "thread.started"]
    ends = [i for i, r in enumerate(rows) if r.get("type") == "turn.completed"]
    bad = [(i+1, r.get("type")) for i, r in enumerate(rows) if r.get("type") == "turn.failed"]
    valid = len(starts) == len(ends) == 1 and starts[0] < ends[0]
    for i, row in enumerate(rows):
        if row.get("type") == "error" and not (valid and starts[0] < i < ends[0]):
            bad.append((i+1, "error-outside-owned-complete-turn"))
    if not valid:
        bad.append((0, "incomplete-public-lifecycle"))
    return bad


def replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError("retained aggregate source anchor differs: " + before)
    return source.replace(before, after, 1)


def aggregate_source(batch, run_id, expected_cwd, output, sessions_root=Path("/home/user/.codex/sessions")):
    raw = TEMPLATE.read_bytes()
    if digest(raw) != TEMPLATE_SHA:
        raise ValueError("retained full audit template changed")
    source = json.loads(raw)["stdin"]
    setup_start = source.index("S=Path(")
    setup_end = source.index("core=S/'audit_compaction.py'")
    setup = ("S=Path(" + repr(str(STUDY)) + ")\nP=Path(" + repr(str(batch)) + ")\n"
             "run_id=" + repr(run_id) + "\nA=P/'runs'/run_id/(run_id+'-three-feature-sequential-bench-artifacts')\n"
             "expected_cwd=" + repr(expected_cwd) + "\n")
    source = source[:setup_start] + setup + source[setup_end:]
    before = "for p in [core,P/'logs'/f'{run_id}.exit.json',S/'phases/standalone-completion-pilot-01/POSTCAPTURE-HANDOFF.md',S/'phases/standalone-completion-pilot-01/postcapture/NATIVE-SUPPLEMENT-COMMAND03.json',S/'PROTOCOL-STANDALONE-GLOBAL-HUNK-PILOT.md']: read(p)"
    source = replace_once(source, before,
        "for p in [core,P/'logs'/f'{run_id}.exit.json',P/'PHASE-MANIFEST.json',P/'PHASE-RESULT.json',P/'PROTOCOL.md']: read(p)")
    before = "exec(compile(read(core),str(core),'exec'),mod.__dict__)\n"
    after = before + ("cached_core=NativeCache(Path(" + repr(str(output / "native-core")) + "),"
        "originals[str(core)])\noriginal_core=mod.audit_rollout\n"
        "mod.audit_rollout=lambda rows,metadata,model,effort: cached_core.run(original_core,rows,metadata,model,effort)\n")
    source = replace_once(source, before, after)
    source = replace_once(source,
        "    bad=[(n,r.get('type')) for n,r in enumerate(rr,1) if r.get('type') in ['error','turn.failed']]",
        "    bad=public_lifecycle(rr)")
    source = replace_once(source,
        "    if len(starts)!=1 or len(ends)!=1 or bad:\n        issue('public lifecycle incomplete: '+str(p))\n        continue",
        "    if len(starts)!=1:\n        issue('public lifecycle has no unique owner: '+str(p))\n        continue\n"
        "    if bad: issue('public lifecycle incomplete: '+str(p))")
    source = replace_once(source,
        "'public_usage':mod.metrics(mod.checked_usage(ends[0][1].get('usage'))),\n        'public_completion_line':ends[0][0]",
        "'public_usage':mod.metrics(mod.checked_usage(ends[0][1].get('usage'))) if len(ends)==1 else None,\n"
        "        'public_completion_line':ends[0][0] if len(ends)==1 else None")
    source = replace_once(source,
        "difference={k:native_usage[k]-pub['public_usage'][k] for k in mod.FIELDS}",
        "difference={k:native_usage[k]-(pub['public_usage'] or {}).get(k,0) for k in mod.FIELDS}")
    source = replace_once(source,
        "classification='exact_public_native_match' if not any(difference.values())",
        "classification='recorded_native_public_unavailable' if pub['public_usage'] is None else 'exact_public_native_match' if not any(difference.values())")
    source = replace_once(source,
        "'native_usage':native_usage,'native_minus_public':mod.metrics(difference),'reconciliation':classification",
        "'native_usage':native_usage,'native_minus_public':mod.metrics(difference) if pub['public_usage'] is not None else None,'reconciliation':classification")
    source = replace_once(source,
        "x['public_usage'] for x in joined", "x['public_usage'] for x in joined if x['public_usage'] is not None")
    source = replace_once(source,
        "x['public_usage'] for x in selected", "x['public_usage'] for x in selected if x['public_usage'] is not None")
    token = "Path('/home/user/.codex/sessions')"
    if source.count(token) != 2:
        raise ValueError("retained native-root source anchors differ")
    source = source.replace(token, "Path(" + repr(str(sessions_root)) + ")")
    return source


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--batch", type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--expected-cwd", required=True)
    args = parser.parse_args()
    batch = args.batch.resolve()
    manifest = json.loads((batch / "PHASE-MANIFEST.json").read_bytes())
    require_terminal(batch, manifest)
    if args.run_id not in [r["run_id"] for r in manifest["schedule"]]:
        raise ValueError("run is outside the admitted population")
    output = batch / "postcapture" / args.run_id
    output.mkdir(parents=True, exist_ok=True)
    result_path = output / "NATIVE-USAGE.json"
    if result_path.exists():
        raise ValueError("aggregate already published; reuse the saved result")
    source = aggregate_source(batch, args.run_id, args.expected_cwd, output)
    command_path = output / "EXECUTED-AGGREGATE.json"
    if not command_path.exists():
        publish(command_path, {"source": source, "template_sha256": TEMPLATE_SHA,
                              "adapter_sha256": digest(Path(__file__).read_bytes())})
    capture = io.StringIO()
    with redirect_stdout(capture):
        exec(compile(source, str(command_path), "exec"),
             {"NativeCache": NativeCache, "public_lifecycle": public_lifecycle})
    result = json.loads(capture.getvalue())
    publish(result_path, result)
    print(json.dumps({k: result[k] for k in
        ("run_id", "status", "errors", "native_usage", "category_totals", "capture_qualification")}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
