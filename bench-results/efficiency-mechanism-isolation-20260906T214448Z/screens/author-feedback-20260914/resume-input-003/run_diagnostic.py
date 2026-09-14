"""One bounded subscription diagnostic for full/inherited/short catalog delivery."""
import argparse
import json
import os
from pathlib import Path
import sys
import time

import catalog_schedule

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "author-inverse-20260914"))
import author_inverse


def main():
    parser = argparse.ArgumentParser()
    for name in ("arm", "repo", "artifact-dir", "codex-bin"):
        parser.add_argument("--" + name, required=True)
    args = parser.parse_args()
    if args.arm != "P03" or os.environ.get("WORK_LEAF_BENCH_INVERSE") != "1":
        parser.error("only the opted-in resumed-input diagnostic is admitted")
    root, repo = Path(args.artifact_dir), Path(args.repo)
    host = author_inverse.load_host("identity")
    cursor = catalog_schedule.Cursor()
    thread, records, total = None, [], 0
    deadline = time.monotonic() + 60
    print(f"supervisor_pid={os.getpid()} arm=P03 artifacts={root} wall_seconds=60", flush=True)
    for number in range(1, 4):
        folder = root / f"turn-{number:04d}"
        cursor.enter(number, thread)
        env = dict(os.environ)
        env["WORK_LEAF_BENCH_INPUT_BOUNDARY"] = str(number)
        argv = [args.codex_bin, "--cd", str(repo), "--sandbox", "read-only",
                "--ask-for-approval", "never", "--model", "gpt-5.5", "exec", "--color", "never"]
        if thread:
            argv += ["resume", "--json", "-o", str(folder / "final.txt"), thread, "-"]
        else:
            argv += ["--json", "-o", str(folder / "final.txt"), "-"]
        host.save_json(folder / "request.json", {"argv": argv, "expected_thread": thread,
                       "catalog_boundary": number,
                       "prompt_sha256": host.digest((folder / "prompt.txt").read_bytes())})
        result = host.execute_child(argv, repo, folder / "prompt.txt", folder / "stdout.jsonl",
                                    folder / "stderr.txt", max(0, deadline - time.monotonic()), env)
        host.save_json(folder / "exit.json", result)
        records.append(result)
        if result["exit_code"] != 0 or result["timed_out"] or result["cancelled_signal"]:
            return 1
        thread, final, _ = host.owned_final(folder / "stdout.jsonl", folder / "final.txt", thread)
        rows = [json.loads(line) for line in (folder / "stdout.jsonl").read_text().splitlines()]
        completed = [r for r in rows if r.get("type") == "turn.completed"]
        if len(completed) != 1 or final.strip() != "WORK_LEAF_RESUME_INPUT_OK":
            raise ValueError("diagnostic response did not satisfy its bounded read-only instruction")
        usage = completed[0]["usage"]
        total += usage["input_tokens"] + usage["output_tokens"]
        if total >= 100000 or time.monotonic() >= deadline:
            return 1
    host.save_json(root / "DIAGNOSTIC-RESULT.json", {"thread_id": thread,
                   "completed": True, "invocations": records,
                   "public_recorded_raw": total, "scope": "input delivery only; no causal token effect"})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
