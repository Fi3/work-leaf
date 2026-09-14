"""Opt-in benchmark catalog view; native CLI, credentials and resume stay unchanged."""
import ctypes
import json
import os
from pathlib import Path
import sys


def needs_view(argv, env):
    if env.get("WORK_LEAF_BENCH_P01_CATALOG") != "1":
        raise ValueError("private catalog requires explicit benchmark opt-in")
    if env.get("WORK_LEAF_BENCH_CODEX_ACTIVE") == "1":
        raise ValueError("recursive benchmark provider launch is forbidden")
    if "exec" not in argv:
        raise ValueError("only native exec is supported")
    i = argv.index("exec")
    return argv[i + 1:i + 2] != ["resume"]


def enter_view(view):
    if str(os.stat("/proc/self/ns/mnt").st_ino) == os.environ.get("WORK_LEAF_P01_PARENT_MNT"):
        raise ValueError("catalog mounts require a distinct private mount namespace")
    lib = ctypes.CDLL(None, use_errno=True)
    for source, target in view["mounts"]:
        if lib.mount(source.encode(), target.encode(), None, 4096, None) != 0:
            raise OSError(ctypes.get_errno(), "private catalog mount failed")
    if os.getuid() != view["uid"]:
        raise ValueError("preserve the original user identity")
    # The mounts belong to this namespace. The provider receives no ambient,
    # effective, permitted or inheritable capabilities from mount preparation.
    if lib.prctl(47, 4, 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), "cannot clear ambient capabilities")
    header = (ctypes.c_uint32 * 2)(0x20080522, 0)
    data = (ctypes.c_uint32 * 6)()
    if lib.capset(ctypes.byref(header), ctypes.byref(data)) != 0:
        raise OSError(ctypes.get_errno(), "cannot drop preparation capabilities")


def main():
    argv = sys.argv[1:]
    inside = argv[:1] == ["--inside"]
    if inside:
        argv = argv[1:]
    fresh = needs_view(argv, os.environ)
    root = Path(__file__).resolve().parent
    view = json.loads((root / "PRIVATE-VIEW.json").read_text())
    if fresh and not inside:
        os.environ["WORK_LEAF_P01_PARENT_MNT"] = str(os.stat("/proc/self/ns/mnt").st_ino)
        os.execvp("unshare", ["unshare", "--user", "--map-current-user", "--keep-caps",
            "--mount", "--propagation", "private", sys.executable, str(Path(__file__).resolve()),
            "--inside", *argv])
    if inside:
        if not fresh:
            raise ValueError("resume cannot enter the private startup view")
        enter_view(view)
    os.environ.pop("WORK_LEAF_P01_PARENT_MNT", None)
    os.environ["WORK_LEAF_BENCH_CODEX_ACTIVE"] = "1"
    provider = root.parents[2] / "phases/standalone-global-hunk-pilot-01/infrastructure/provider/codex"
    os.execv(str(provider), [str(provider), *argv])


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(86)
