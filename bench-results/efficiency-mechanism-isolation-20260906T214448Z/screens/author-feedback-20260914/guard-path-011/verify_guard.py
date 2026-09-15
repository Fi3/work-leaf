"""Actual artifact guard + full observer setup + private host route, bounded."""
import argparse
import json
import os
from pathlib import Path
import shlex
import sys
import time

import repair_driver as r


def shell_function(source, name):
    start = source.index(name + "() {")
    return source[start:source.index("\n}\n", start) + 3]


def main():
    parser = argparse.ArgumentParser()
    for name in ("arm", "artifact-dir", "repo"):
        parser.add_argument("--" + name, required=True)
    args = parser.parse_args()
    if args.arm != "P09":
        raise ValueError("explicit P09 diagnostic required")
    root, repo = Path(args.artifact_dir).resolve(), Path(args.repo).resolve()
    settings = json.loads((root / "SETUP.json").read_text())
    host = r.original.load_host()
    source = r.driver_source()
    command = [sys.executable, str(r.original.HERE/"host_custody.py"),
        "--repo", str(repo), "--artifact-dir", str(root/"host"), "--feature", "diagnostic",
        "--stage", "sequential-feature-1-implement", "--prompt-file", str(root/"PROMPT.txt"),
        "--codex-bin", str(r.original.HERE/"input_provider"), "--serialized-feedback",
        "--model", "gpt-5.5", "--deadline", str(time.monotonic()+60)]
    script = 'set -euo pipefail\nsource ' + shlex.quote(settings["artifact_common"]) + '\n'
    script += 'fail_bench() { printf "%s\\n" "$*" >&2; exit 86; }\n'
    for name in ("setup_observer", "observer_timeline"):
        script += shell_function(source, name)
    for key, value in settings["variables"].items():
        script += key + "=" + shlex.quote(value) + "\n"
    script += "bench_begin_artifact_transaction " + shlex.quote(str(root)) + " guard-artifacts guard.md guard.tsv\n"
    script += "trap 'bench_close_artifact_transaction_guard keep' EXIT\n"
    script += 'artifact_dir="$bench_artifact_transaction_stage_fd_path"\nsetup_observer\n'
    script += 'printf "GUARD=%s\\nCANONICAL_OBSERVER=%s\\n" "$artifact_dir" "$observer_root"\n'
    script += 'export WORK_LEAF_OBSERVER_PRIMARY_MARKER="$observer_primary_marker"\n'
    script += 'export WORK_LEAF_OBSERVER_ROLE=sequential-feature-1-implement\n'
    script += shlex.join(command) + "\n"
    # This shell script contains no credentials; preserve the exact executed setup.
    with (root/"EXECUTED-SHELL.txt").open("x") as output:
        output.write(script)
    status = host.execute_child(["bash", "-c", script], repo, root/"EMPTY-STDIN.txt",
        root/"guard.stdout", root/"guard.stderr", 65, dict(os.environ))
    host.save_json(root/"GUARD-EXIT.json", status)
    if status["exit_code"] or status["timed_out"] or status["cancelled_signal"]:
        raise ValueError("guard route diagnostic failed; no retry")
    result = json.loads((root/"host/result.json").read_text())
    if not result["completed"] or not result["clean"] or result["accepted_commits"]:
        raise ValueError("guard verification did not complete without changes")
    host.save_json(root/"DIAGNOSTIC-RESULT.json", {
        "completed": True, "thread_id": result["thread_id"],
        "scope": "real artifact guard, full observer setup, actual command feedback, same-thread continuation",
        "preservation": "original hidden guard stage retained; not published as benchmark"})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
