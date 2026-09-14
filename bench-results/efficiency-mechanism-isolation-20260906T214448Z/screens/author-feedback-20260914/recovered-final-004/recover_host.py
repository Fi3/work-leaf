"""Isolated acceptance overlay for a completed native turn with diagnostics."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import sys
import time
import types

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "resume-input-003"))
import run_author


def load_host(arm):
    source = run_author.author_inverse.replace_once(
        run_author.scheduled_source(arm),
        '            elif kind in ("error", "turn.failed"):\n'
        '                raise Fatal("native invocation failed")',
        '            elif kind == "turn.failed":\n'
        '                raise Fatal("native invocation failed")\n'
        '            elif kind == "error":\n'
        '                if not thread or starts != 1 or ends:\n'
        '                    raise Fatal("diagnostic outside owned active turn")',
    )
    module = types.ModuleType("recovered_private_benchmark_host")
    module.__file__ = str(run_author.author_inverse.BASE_HOST)
    exec(compile(source, module.__file__, "exec"), module.__dict__)
    module._reference_cursor = run_author.catalog_schedule.Cursor()
    return module


def require(value, message):
    if not value:
        raise ValueError(message)


def recover_first(module, host, prompt, seed):
    """Restore only an unconsumed first response; never regenerate its input."""
    require(seed["schema"] == 1, "unsupported recovery seed")
    require(not host.invocations and host.thread is None, "recovery already consumed")
    required = {"raw", "final", "request", "result", "exit", "events"}
    require(set(seed["files"]) == required == set(seed["sha256"]), "incomplete seed files")
    payload = {}
    for name, filename in seed["files"].items():
        path = Path(filename)
        require(not path.is_symlink() and path.is_file(), "nonregular seed file")
        payload[name] = path.read_bytes()
        require(hashlib.sha256(payload[name]).hexdigest() == seed["sha256"][name],
                "seed file digest differs")
    request, result, status = (json.loads(payload[name]) for name in ("request", "result", "exit"))
    require(status["exit_code"] == 0 and status["timed_out"] is False
            and status["cancelled_signal"] is None, "saved native child did not succeed")
    require(request["expected_thread"] is None and request["prompt_sha256"] ==
            hashlib.sha256(prompt.encode()).hexdigest(), "saved launch differs")
    require(result["invocations"] == [request] and result["completed"] is False
            and result["clean"] is True and not result["accepted_commits"]
            and not result["pending_changes"] and not result["unprocessed_replies"]
            and not payload["events"].strip(), "saved host contains consumed work")
    require(result["initial_head"] == result["final_head"] == host.initial_head
            and Path(result["repo"]).resolve() == host.repo
            and result["feature"] == host.feature and result["stage"] == host.stage
            and result["serialized_feedback"] == host.serialized_feedback,
            "recovery host state differs")
    native = Path(seed["native_path"])
    require(not native.is_symlink() and native.is_file(), "nonregular native prefix")
    prefix = native.read_bytes()
    require(len(prefix) == seed["native_prefix_bytes"] and
            hashlib.sha256(prefix).hexdigest() == seed["native_prefix_sha256"],
            "native prefix changed or already resumed")
    host.unchanged()
    thread, final, author = module.owned_final(seed["files"]["raw"], seed["files"]["final"], None)
    require(module._reference_cursor.ordinal == 0, "delivery cursor already used")
    # An exclusive claim survives even if subsequent host processing fails.
    with Path(seed["claim_path"]).open("x", encoding="utf-8") as claim:
        json.dump({"thread": thread, "artifacts": str(host.artifacts),
                   "final_sha256": seed["sha256"]["final"]}, claim)
        claim.flush()
        os.fsync(claim.fileno())
    module._reference_cursor.enter(1, None)
    host.thread = thread
    host.invocations.append(request)
    for text in author:
        host.evidence("AUTHOR PUBLIC TEXT", text)
    host.event("owned_final", invocation=1, thread_id=thread,
               final_sha256=module.digest(final.encode("utf-8")))
    return final


def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--arm", required=True, choices=("R03", "R04"))
    for name in ("repo", "artifact-dir", "prompt-file", "codex-bin", "seed"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--wall-seconds", type=int, required=True)
    args = parser.parse_args(argv)
    require(os.environ.get("WORK_LEAF_BENCH_INVERSE") == "1", "private benchmark opt-in required")
    require(0 < args.wall_seconds <= 900, "recovery wall must be between 1 and 900 seconds")
    seed = json.loads(Path(args.seed).read_text())
    module = load_host(args.arm)
    original_invoke = module.invoke
    def invoke(host, arguments, prompt):
        if not host.invocations:
            return recover_first(module, host, prompt, seed)
        return original_invoke(host, arguments, prompt)
    module.invoke = invoke
    root, repo = Path(args.artifact_dir).resolve(), Path(args.repo).resolve()
    deadline = time.monotonic() + args.wall_seconds
    print(f"supervisor_pid={os.getpid()} arm={args.arm} artifacts={root} wall_seconds={args.wall_seconds}", flush=True)
    return module.run_stage(argparse.Namespace(
        repo=str(repo), artifact_dir=str(root / "host"), feature="1",
        stage="initial-author", deadline=deadline, serialized_feedback=True,
        prompt_file=args.prompt_file, resume_thread=None, model="gpt-5.5",
        codex_bin=args.codex_bin))


if __name__ == "__main__":
    raise SystemExit(main())
