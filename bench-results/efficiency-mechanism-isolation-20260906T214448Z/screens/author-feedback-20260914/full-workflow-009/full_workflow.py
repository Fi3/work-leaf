"""Opt-in A+B full-workflow adapter; reuse the frozen standalone host/driver."""
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "recovered-final-004"))
import recover_host

inverse = recover_host.run_author.author_inverse
dispatch = recover_host.run_author.catalog_schedule.dispatch
DRIVER = inverse.BASE_HOST.with_name("bench-three-features-serialized-host-custody")
DRIVER_SHA = "a621b4ee2ae74dbef03d6c3b29b8dbaaf5842d27e186f61089ce16d658d1526f"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def driver_replacements():
    return [
        ("set -euo pipefail\n", "set -euo pipefail\n"
         '[[ "${WORK_LEAF_BENCH_FULL_INVERSE:-}" == "1" ]] || exit 86\n'),
        ('host_custody_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/host_custody.py"',
         'host_custody_script="${WORK_LEAF_BENCH_FULL_HOST:?private full-workflow host required}"'),
        ('source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/bench-candidate-common"',
         "source " + shlex.quote(str(DRIVER.with_name("bench-candidate-common")))),
        ('  agent_bin="$observer_proxy_dir/codex"',
         '  export WORK_LEAF_BENCH_INPUT_NEXT="$observer_proxy_dir/codex"\n'
         '  agent_bin="${WORK_LEAF_BENCH_FULL_PROVIDER:?private input wrapper required}"'),
        *inverse.A_SPANS,
        inverse.B_LAUNCH,
    ]


def driver_source():
    raw = DRIVER.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == DRIVER_SHA, "frozen full driver changed")
    result = raw.decode()
    for before, after in driver_replacements():
        result = inverse.replace_once(result, before, after)
    return result


class StageCursor:
    """Serial invocation ownership, including a stage's first resumed turn."""
    def __init__(self, resume_thread):
        require(resume_thread is None or isinstance(resume_thread, str) and bool(resume_thread),
                "invalid stage resume thread")
        self.ordinal = 0
        self.thread = resume_thread
        self.initial = resume_thread

    def enter(self, ordinal, thread):
        require(type(ordinal) is int and ordinal == self.ordinal + 1,
                "duplicate, skipped or stale stage boundary")
        if ordinal == 1:
            require(thread == self.initial, "stage initial thread differs")
        else:
            require(isinstance(thread, str) and bool(thread), "resume requires native thread")
        require(self.thread is None or self.thread == thread, "foreign stage thread")
        self.ordinal, self.thread = ordinal, thread


def select(plan, stage, ordinal, argv, env):
    require(env.get("WORK_LEAF_BENCH_FULL_INVERSE") == "1", "full inverse opt-in required")
    fresh = dispatch.needs_view(argv, env)
    require(type(ordinal) is int and ordinal >= 1, "invalid invocation ordinal")
    require(isinstance(stage, str) and re.fullmatch(r"[A-Za-z0-9_-]+", stage), "invalid stage")
    require(plan.get("schema") == 1 and plan.get("tail") == "hold_last", "unsupported plan")
    stages, profiles = plan.get("stages"), plan.get("profiles")
    require(isinstance(stages, dict) and isinstance(profiles, dict), "missing stage profiles")
    entry = stages.get(stage)
    fallback = entry is None
    if fallback:
        matches = [r for r in plan.get("fallbacks", []) if re.fullmatch(r["pattern"], stage)]
        require(len(matches) == 1, "unknown or ambiguous fallback stage")
        entry = matches[0]
    require(entry.get("first") in ("fresh", "resume") and entry.get("kind") in ("host", "direct"),
            "invalid stage mode")
    states = entry.get("states")
    require(isinstance(states, list) and bool(states), "missing stage states")
    for name in states:
        profile = profiles.get(name) if isinstance(name, str) else None
        require(isinstance(profile, dict) and isinstance(profile.get("view"), dict),
                "missing profile view")
        require(re.fullmatch(r"[0-9a-f]{64}", profile.get("catalog_sha256", "")),
                "missing catalog digest")
    require(fresh == (ordinal == 1 and entry["first"] == "fresh"), "stage fresh/resume mismatch")
    if entry["kind"] == "direct":
        require(ordinal == 1, "direct stage contains exactly one invocation")
        expected_role = stage
    else:
        expected_role = f"{stage}-host-{ordinal:04d}"
    require(env.get("WORK_LEAF_OBSERVER_ROLE") == expected_role, "observer stage ownership differs")
    name = states[min(ordinal, len(states)) - 1]
    return {"name": name, "profile": profiles[name], "stage": stage, "ordinal": ordinal,
            "fallback": fallback, "extrapolated": ordinal > len(states),
            "fresh": fresh, "kind": entry["kind"]}


def load_host():
    # Reuse the separately fail-first and real-agent verified recovered-diagnostic
    # parser. Recovery of old responses is NOT used by the full workflow.
    return recover_host.load_host("R04")


def host_main():
    require(os.environ.get("WORK_LEAF_BENCH_FULL_INVERSE") == "1", "full inverse opt-in required")
    module = load_host()
    original = module.run_stage

    def run_stage(args):
        require(os.environ.get("WORK_LEAF_OBSERVER_ROLE") == args.stage, "foreign host stage")
        os.environ["WORK_LEAF_BENCH_INPUT_STAGE"] = args.stage
        module._reference_cursor = StageCursor(args.resume_thread)
        return original(args)

    module.run_stage = run_stage
    return module.main()


def provider_main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    inside = argv[:1] == ["--inside"]
    if inside:
        argv = argv[1:]
    env = os.environ
    plan_path = Path(env["WORK_LEAF_BENCH_INPUT_PLAN"])
    raw = plan_path.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == env["WORK_LEAF_BENCH_INPUT_PLAN_SHA256"],
            "input plan digest differs")
    plan = json.loads(raw)
    for filename, expected in plan["sources"].items():
        require(hashlib.sha256(Path(filename).read_bytes()).hexdigest() == expected,
                "input plan source changed: " + filename)
    stage = env.get("WORK_LEAF_BENCH_INPUT_STAGE", env["WORK_LEAF_OBSERVER_ROLE"])
    ordinal = int(env.get("WORK_LEAF_BENCH_INPUT_BOUNDARY", "1"))
    selected = select(plan, stage, ordinal, argv, env)
    next_provider = env["WORK_LEAF_BENCH_INPUT_NEXT"]
    require(Path(next_provider).is_absolute() and os.access(next_provider, os.X_OK),
            "original observer provider is not executable")
    require(Path(next_provider).resolve() != Path(env["WORK_LEAF_BENCH_FULL_PROVIDER"]).resolve(),
            "input wrapper cannot forward to itself")
    if not inside:
        root = Path(env["WORK_LEAF_BENCH_INPUT_RECEIPTS"])
        root.mkdir(parents=True, exist_ok=True)
        with (root / f"{stage}-{ordinal:04d}.json").open("x") as handle:
            json.dump({"selected": selected, "argv": argv, "next_provider": next_provider,
                       "plan_sha256": hashlib.sha256(raw).hexdigest()}, handle, indent=2)
        env["WORK_LEAF_P01_PARENT_MNT"] = str(os.stat("/proc/self/ns/mnt").st_ino)
        os.execvp("unshare", ["unshare", "--user", "--map-current-user", "--keep-caps",
                  "--mount", "--propagation", "private", sys.executable,
                  str(HERE / "input_provider"), "--inside", *argv])
    dispatch.qualified.enter_view(selected["profile"]["view"])
    for key in list(env):
        if key.startswith("WORK_LEAF_BENCH_INPUT_") or key == "WORK_LEAF_P01_PARENT_MNT":
            env.pop(key, None)
    # The unchanged observer/profile, not this wrapper, owns the ACTIVE guard.
    os.execv(next_provider, [next_provider, *argv])


if __name__ == "__main__":
    if sys.argv[1:] != ["driver"]:
        raise SystemExit("expected driver")
    print(driver_source(), end="")
