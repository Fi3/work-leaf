"""Read owned public/native resource fields only; never add their representations."""
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import signal
import sys
import uuid


def events(path):
    rows = []
    for line in Path(path).read_bytes().splitlines(keepends=True):
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            if line.endswith(b"\n"):
                raise
    return rows


def raw(usage):
    fields = [usage["input_tokens"], usage["output_tokens"]]
    if not all(type(x) is int and x >= 0 for x in fields):
        raise ValueError("invalid resource fields")
    return sum(fields)


def native_totals(rows, threads):
    responses = {}
    for event in rows:
        if event.get("type") != "token_usage_record":
            continue
        payload = event["payload"]
        if payload["thread_id"] not in threads:
            continue
        identity = payload["response_id"]
        if not isinstance(identity, str) or not identity:
            raise ValueError("missing response identity")
        value = raw(payload["usage"])
        if identity in responses and responses[identity] != value:
            raise ValueError("conflicting duplicate response charge")
        responses[identity] = value
    return sum(responses.values()), len(responses)


def sample_arm(root, sessions):
    root = Path(root)
    files = sorted(set(root.glob("native.stdout.jsonl")) |
                   set(root.glob("host/invocation-*/stdout.jsonl")))
    threads, completed, public = set(), 0, 0
    for path in files:
        for event in events(path):
            if event.get("type") == "thread.started":
                thread = event["thread_id"]
                if str(uuid.UUID(thread)) != thread:
                    raise ValueError("invalid owned thread")
                threads.add(thread)
            if event.get("type") == "turn.completed":
                public += raw(event["usage"])
                completed += 1
    records, paths, missing = [], [], []
    for thread in sorted(threads):
        matches = list(Path(sessions).glob("*" + thread + "*.jsonl"))
        if len(matches) > 1:
            raise ValueError("ambiguous owned rollout")
        if not matches:
            missing.append(thread)
        for path in matches:
            paths.append(str(path))
            records.extend(events(path))
    native, responses = native_totals(records, threads)
    return {"arm": root.name, "native_recorded_raw": native,
            "distinct_responses": responses, "public_completed_raw": public,
            "completed_public_turns": completed, "threads": sorted(threads),
            "native_paths": paths, "native_pending_threads": missing,
            "terminal": (root / "TERMINAL.json").exists(),
            "stop_threshold_reached": max(native, public) >= 3000000}


def stop_owned(script, arm, root):
    expected = [str(Path(script).resolve()), "--arm", arm,
                "--artifact-dir", str(Path(root).resolve())]
    matches = []
    for proc in Path("/proc").glob("[0-9]*"):
        try:
            argv = (proc / "cmdline").read_bytes().split(b"\0")
            argv = [part.decode() for part in argv if part]
            if expected[0] not in argv:
                continue
            if argv[argv.index("--arm") + 1] != arm:
                continue
            if argv[argv.index("--artifact-dir") + 1] != expected[-1]:
                continue
            matches.append((int(proc.name), argv))
        except (OSError, ValueError, IndexError):
            continue
    if len(matches) != 1:
        raise ValueError(f"expected one owned supervisor, found {len(matches)}")
    pid, argv = matches[0]
    descriptor = os.pidfd_open(pid)
    try:
        if Path(f"/proc/{pid}/cmdline").read_bytes().split(b"\0")[:-1] != [
                item.encode() for item in argv]:
            raise ValueError("supervisor identity changed")
        signal.pidfd_send_signal(descriptor, signal.SIGINT)
    finally:
        os.close(descriptor)
    return {"signalled_pid": pid, "signal": "SIGINT", "arm": arm,
            "artifact_root": str(root), "argv": argv}


if __name__ == "__main__":
    root = Path(__file__).resolve().parent
    if len(sys.argv) > 1:
        result = stop_owned(root / "author_inverse.py", sys.argv[1], root / sys.argv[1])
    else:
        sessions = Path("/home/user/.codex/sessions/2026/09/14")
        result = {"at_utc": datetime.now(timezone.utc).isoformat(),
                  "representation": "distinct native and completed public reported separately",
                  "runs": [sample_arm(root / arm, sessions) for arm in ("R02", "R04", "R06")]}
    print(json.dumps(result, indent=2))
