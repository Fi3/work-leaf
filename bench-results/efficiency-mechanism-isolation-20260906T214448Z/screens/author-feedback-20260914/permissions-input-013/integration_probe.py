"""One admitted integration-profile launch/resume probe; no feature work."""
import argparse
import json
import os
from pathlib import Path
import sys
import time
import uuid

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "full-workflow-009"))
import full_workflow as f


def command(provider, repo, thread):
    if thread is not None and str(uuid.UUID(thread)) != thread:
        raise ValueError("invalid native resume identity")
    result = [str(provider), "--cd", str(repo), "--sandbox", "danger-full-access",
              "--ask-for-approval", "never", "--model", "gpt-5.5", "exec", "--color", "never"]
    return result + (["--json", "-"] if thread is None else ["resume", "--json", thread, "-"])


def terminal(rows, expected, marker):
    starts = [r["thread_id"] for r in rows if r.get("type") == "thread.started"]
    ends = [r for r in rows if r.get("type") == "turn.completed"]
    if (len(starts) != 1 or len(ends) != 1 or expected is not None and starts != [expected]
            or any(r.get("type") in ("turn.failed", "error") for r in rows)):
        raise ValueError("probe lacks one owned complete public turn")
    messages = [r["item"].get("text", "") for r in rows if r.get("type") == "item.completed"
                and r.get("item", {}).get("type") == "agent_message"]
    if not messages or messages[-1].strip() != marker:
        raise ValueError("probe final marker differs")
    usage = ends[0]["usage"]
    values = [usage[k] for k in ("input_tokens", "output_tokens")]
    if any(type(v) is not int or v < 0 for v in values):
        raise ValueError("invalid public usage")
    return starts[0], sum(values)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--artifact-dir", type=Path, required=True)
    args = parser.parse_args()
    if os.environ.get("WORK_LEAF_BENCH_FULL_INVERSE") != "1":
        raise ValueError("explicit benchmark opt-in required")
    root, repo = args.artifact_dir.resolve(), args.repo.resolve()
    host = f.load_host()
    initial = host.snapshot(repo)
    host.save_json(root / "INITIAL-STATE.json", host.state_summary(initial))
    deadline, thread, total = time.monotonic() + 60, None, 0
    for stage, marker in (("linearize-plan", "WORK_LEAF_INTEGRATION_INPUT_OK"),
                          ("linearize-accept", "WORK_LEAF_INTEGRATION_RESUME_OK")):
        if total >= 100000:
            raise ValueError("completed-public usage tripwire; no further turn")
        output = root / stage
        output.mkdir()
        env = dict(os.environ)
        env["WORK_LEAF_OBSERVER_ROLE"] = stage
        argv = command(f.HERE / "input_provider", repo, thread)
        host.save_json(output / "REQUEST.json", {"argv":argv,"expected_thread":thread})
        result = host.execute_child(argv, repo, root / (stage + ".prompt.txt"),
            output / "stdout.jsonl", output / "stderr.txt", deadline-time.monotonic(), env)
        host.save_json(output / "EXIT.json", result)
        if result["exit_code"] or result["timed_out"] or result["cancelled_signal"]:
            raise ValueError("probe invocation failed; no retry")
        rows = [json.loads(line) for line in (output / "stdout.jsonl").read_text().splitlines()]
        thread, raw = terminal(rows, thread, marker)
        total += raw
        host.save_json(output / "PUBLIC-RESULT.json", {"thread_id":thread,"raw":raw})
        if host.snapshot(repo) != initial:
            raise ValueError("read-only task changed repository state")
    host.save_json(root / "DIAGNOSTIC-RESULT.json", {"completed":True,"thread_id":thread,
        "public_raw":total,"scope":"actual integration capability profile; input comparison remains separate",
        "repository_unchanged":True})


if __name__ == "__main__":
    main()
