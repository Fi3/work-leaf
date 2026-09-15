#!/usr/bin/env python3
"""Opt-in, source-pinned private catalog view for the retained WL app-server."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
SOURCE = HERE.parents[1] / "P01/private_catalog.py"
spec = importlib.util.spec_from_file_location("p06_private_catalog", SOURCE)
qualified = importlib.util.module_from_spec(spec)
spec.loader.exec_module(qualified)
ALLOWED_CONFIG = {'model="gpt-5.5"', 'model_reasoning_effort="xhigh"'}

def classify(argv):
    offset = 0
    while offset < len(argv) and argv[offset] == "-c":
        if offset + 1 >= len(argv) or argv[offset + 1] not in ALLOWED_CONFIG:
            raise ValueError("only frozen model/effort profile overrides are admitted")
        offset += 2
    rest = argv[offset:]
    if rest == ["--version"]:
        return "version"
    if rest != ["app-server", "--listen", "stdio://"]:
        raise ValueError("only the frozen stdio app-server route is admitted")
    return "app-server"

def main():
    argv = list(sys.argv[1:])
    inside = argv[:1] == ["--inside"]
    if inside:
        argv = argv[1:]
    kind = classify(argv)
    if os.environ.get("WORK_LEAF_BENCH_P06_INPUT") != "1":
        raise ValueError("explicit P06 input opt-in required")
    if os.environ.get("WORK_LEAF_BENCH_CODEX_ACTIVE") != "1":
        raise ValueError("the unchanged outer profile must own the recursive launch guard")
    raw = Path(os.environ["WORK_LEAF_BENCH_P06_INPUT_PLAN"]).read_bytes()
    if hashlib.sha256(raw).hexdigest() != os.environ["WORK_LEAF_BENCH_P06_INPUT_PLAN_SHA256"]:
        raise ValueError("P06 input plan digest differs")
    plan = json.loads(raw)
    if plan["schema"] != "work-leaf-p06-appserver-input-v1":
        raise ValueError("unsupported P06 input plan")
    for filename, expected in plan["sources"].items():
        if hashlib.sha256(Path(filename).read_bytes()).hexdigest() != expected:
            raise ValueError("P06 input source differs: " + filename)
    provider = plan["provider"]
    if not Path(provider).is_absolute() or not os.access(provider, os.X_OK):
        raise ValueError("provider is not an absolute executable")
    if kind == "app-server" and not inside:
        receipt = Path(os.environ["WORK_LEAF_BENCH_P06_INPUT_RECEIPT"])
        with receipt.open("x") as output:
            json.dump({"argv":argv,"plan_sha256":hashlib.sha256(raw).hexdigest(),
                       "uid":os.getuid(),"pid":os.getpid(),"view":plan["view"]}, output)
        os.environ["WORK_LEAF_P01_PARENT_MNT"] = str(os.stat("/proc/self/ns/mnt").st_ino)
        os.execvp("unshare", ["unshare", "--user", "--map-current-user", "--keep-caps",
            "--mount", "--propagation", "private", sys.executable, str(Path(__file__).resolve()),
            "--inside", *argv])
    if inside:
        if kind != "app-server":
            raise ValueError("non-agent route cannot enter the private namespace")
        qualified.enter_view(plan["view"])
    for key in list(os.environ):
        if key.startswith("WORK_LEAF_BENCH_P06_INPUT") or key == "WORK_LEAF_P01_PARENT_MNT":
            os.environ.pop(key)
    os.execv(provider, [provider, *argv])

if __name__ == "__main__":
    try:
        main()
    except (KeyError, ValueError, OSError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(86)
