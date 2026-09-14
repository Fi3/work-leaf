"""Private P01 startup-only delay; no prompt text or provider implementation edit."""
import os
from pathlib import Path
import sys


DELAY = ('hooks.SessionStart=[{matcher="^startup$",hooks=[{type="command",'
         'command="/usr/bin/sleep 2",timeout=5}]}]')


def build_argv(argv, env):
    if env.get("WORK_LEAF_BENCH_P01_DELAY") != "1":
        raise ValueError("P01 delay requires explicit benchmark opt-in")
    if env.get("WORK_LEAF_BENCH_CODEX_ACTIVE") == "1":
        raise ValueError("benchmark policy blocks recursive Codex provider launches")
    if "exec" not in argv:
        raise ValueError("P01 qualification supports only native exec")
    position = argv.index("exec")
    if argv[position + 1:position + 2] == ["resume"]:
        return list(argv)
    return ["--dangerously-bypass-hook-trust", "-c", DELAY, *argv]


def main():
    try:
        argv = build_argv(sys.argv[1:], os.environ)
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 86
    os.environ["WORK_LEAF_BENCH_CODEX_ACTIVE"] = "1"
    study = Path(__file__).resolve().parents[3]
    provider = study / "phases/standalone-global-hunk-pilot-01/infrastructure/provider/codex"
    os.execv(str(provider), [str(provider), *argv])


if __name__ == "__main__":
    raise SystemExit(main())
