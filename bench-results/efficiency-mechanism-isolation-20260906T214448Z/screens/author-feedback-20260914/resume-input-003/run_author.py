"""Existing A/B author experiment with explicit reference input delivery only."""
import argparse
import os
from pathlib import Path
import sys
import time
import types

import catalog_schedule

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "author-inverse-20260914"))
import author_inverse


SPAN = '    env = dict(os.environ)\n    if env.get("WORK_LEAF_BENCH_CODEX_ACTIVE")'
REPLACEMENT = ('    _reference_cursor.enter(number, host.thread)\n'
               '    env = dict(os.environ)\n'
               '    env["WORK_LEAF_BENCH_INPUT_BOUNDARY"] = str(number)\n'
               '    if env.get("WORK_LEAF_BENCH_CODEX_ACTIVE")')


def scheduled_source(arm):
    return author_inverse.replace_once(author_inverse.host_source(arm), SPAN, REPLACEMENT)


def load_host(arm):
    module = types.ModuleType("scheduled_private_benchmark_host")
    module.__file__ = str(author_inverse.BASE_HOST)
    exec(compile(scheduled_source(arm), str(author_inverse.BASE_HOST), "exec"), module.__dict__)
    module._reference_cursor = catalog_schedule.Cursor()
    return module


def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--arm", required=True, choices=("R03", "R04"))
    for name in ("repo", "artifact-dir", "prompt-file", "codex-bin"):
        parser.add_argument("--" + name, required=True)
    args = parser.parse_args(argv)
    if os.environ.get("WORK_LEAF_BENCH_INVERSE") != "1":
        parser.error("private benchmark opt-in is required")
    root, repo = Path(args.artifact_dir).resolve(), Path(args.repo).resolve()
    module = load_host(args.arm)
    deadline = time.monotonic() + 1200
    print(f"supervisor_pid={os.getpid()} arm={args.arm} artifacts={root} wall_seconds=1200", flush=True)
    return module.run_stage(argparse.Namespace(
        repo=str(repo), artifact_dir=str(root / "host"), feature="1",
        stage="initial-author", deadline=deadline, serialized_feedback=True,
        prompt_file=args.prompt_file, resume_thread=None, model="gpt-5.5",
        codex_bin=args.codex_bin,
    ))


if __name__ == "__main__":
    raise SystemExit(main())
