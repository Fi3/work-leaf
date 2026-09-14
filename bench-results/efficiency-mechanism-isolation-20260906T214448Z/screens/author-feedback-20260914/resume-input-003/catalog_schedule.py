"""Opt-in native catalog delivery from a frozen benchmark reference-state plan.

This module controls input setup, not agent prompts, model responses or workflow
decisions. Reference ordinals are delivery data, not proof of semantic matching.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent
source = ROOT.parent / "P02/catalog_dispatch.py"
spec = importlib.util.spec_from_file_location("p02_catalog_dispatch", source)
dispatch = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dispatch)


def select(plan, ordinal, argv, env):
    if env.get("WORK_LEAF_BENCH_INPUT_SCHEDULE") != "1":
        raise ValueError("reference catalog delivery requires benchmark opt-in")
    fresh = dispatch.needs_view(argv, env)
    if type(ordinal) is not int or ordinal < 1:
        raise ValueError("reference boundary must be a positive integer")
    if fresh != (ordinal == 1):
        raise ValueError("fresh/resume command disagrees with reference boundary")
    if plan.get("schema") != 1 or plan.get("tail") != "hold_last":
        raise ValueError("unsupported reference-state contract")
    states, profiles = plan.get("states"), plan.get("profiles")
    if not isinstance(states, list) or not states or not isinstance(profiles, dict):
        raise ValueError("reference states and profiles are required")
    for name in states:
        profile = profiles.get(name) if isinstance(name, str) else None
        if not isinstance(profile, dict) or not isinstance(profile.get("view"), dict):
            raise ValueError("missing reference profile/view")
        if not re.fullmatch(r"[0-9a-f]{64}", profile.get("catalog_sha256", "")):
            raise ValueError("missing reference catalog digest")
    name = states[min(ordinal, len(states)) - 1]
    return {"name": name, "ordinal": ordinal, "profile": profiles[name],
            "extrapolated": ordinal > len(states)}


class Cursor:
    """One serial host's invocation boundary; no shared cross-workflow state."""
    def __init__(self):
        self.ordinal = 0
        self.thread = None

    def enter(self, ordinal, thread):
        if type(ordinal) is not int or ordinal != self.ordinal + 1:
            raise ValueError("duplicate, skipped or stale host boundary")
        if ordinal == 1 and thread is not None:
            raise ValueError("a scheduled author starts with a fresh native thread")
        if ordinal > 1 and (not isinstance(thread, str) or not thread):
            raise ValueError("resume requires its existing native thread")
        if self.thread is not None and thread != self.thread:
            raise ValueError("native thread changed during the scheduled author")
        self.ordinal, self.thread = ordinal, thread


def main():
    argv = sys.argv[1:]
    inside = argv[:1] == ["--inside"]
    if inside:
        argv = argv[1:]
    path = Path(os.environ["WORK_LEAF_BENCH_INPUT_PLAN"])
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != os.environ["WORK_LEAF_BENCH_INPUT_PLAN_SHA256"]:
        raise ValueError("reference plan digest mismatch")
    plan = json.loads(raw)
    selected = select(plan, int(os.environ["WORK_LEAF_BENCH_INPUT_BOUNDARY"]), argv, os.environ)
    for filename, expected in plan["sources"].items():
        if hashlib.sha256(Path(filename).read_bytes()).hexdigest() != expected:
            raise ValueError("reference input source changed")
    if not inside:
        os.environ["WORK_LEAF_P01_PARENT_MNT"] = str(os.stat("/proc/self/ns/mnt").st_ino)
        os.execvp("unshare", ["unshare", "--user", "--map-current-user", "--keep-caps",
                  "--mount", "--propagation", "private", sys.executable, str(ROOT / "catalog_schedule.py"),
                  "--inside", *argv])
    dispatch.qualified.enter_view(selected["profile"]["view"])
    for key in ("WORK_LEAF_P01_PARENT_MNT", "WORK_LEAF_BENCH_INPUT_SCHEDULE",
                "WORK_LEAF_BENCH_INPUT_PLAN", "WORK_LEAF_BENCH_INPUT_PLAN_SHA256",
                "WORK_LEAF_BENCH_INPUT_BOUNDARY"):
        os.environ.pop(key, None)
    os.environ["WORK_LEAF_BENCH_CODEX_ACTIVE"] = "1"
    provider = ROOT.parents[2] / "phases/standalone-global-hunk-pilot-01/infrastructure/provider/codex"
    os.execv(str(provider), [str(provider), *argv])


if __name__ == "__main__":
    try:
        main()
    except (KeyError, ValueError, OSError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(86)
