"""Resource discovery from explicit public paths and possible native directories.

The caller owns which files belong to its admission. Native and public totals
are alternative representations, never summed. No prompt or provider mutation.
"""
from pathlib import Path
import sys
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "author-inverse-20260914"))
import resource_sample as existing


def sample(root, public_paths, session_directories):
    root = Path(root)
    files = [Path(path).resolve() for path in public_paths]
    if len(set(files)) != len(files):
        raise ValueError("duplicate owned public source")
    threads, public, completed = set(), 0, 0
    for path in files:
        for event in existing.events(path):
            if event.get("type") == "thread.started":
                thread = event["thread_id"]
                if str(uuid.UUID(thread)) != thread:
                    raise ValueError("invalid owned thread")
                threads.add(thread)
            if event.get("type") == "turn.completed":
                public += existing.raw(event["usage"])
                completed += 1
    records, paths, missing = [], [], []
    for thread in sorted(threads):
        matches = {path.resolve() for directory in session_directories
                   for path in Path(directory).glob("*" + thread + "*.jsonl")}
        if len(matches) > 1:
            raise ValueError("ambiguous owned rollout")
        if not matches:
            missing.append(thread)
        for path in sorted(matches):
            paths.append(str(path))
            records.extend(existing.events(path))
    native, responses = existing.native_totals(records, threads)
    return {"arm": root.name, "native_recorded_raw": native,
            "distinct_responses": responses, "public_completed_raw": public,
            "completed_public_turns": completed, "threads": sorted(threads),
            "native_paths": paths, "native_pending_threads": missing,
            "terminal": (root / "TERMINAL.json").exists(),
            "stop_threshold_reached": max(native, public) >= 3000000}
