"""Benchmark-only prompt/feedback inverse; no normal Work Leaf imports or edits."""
import argparse
import hashlib
import os
from pathlib import Path
import sys
import time
import types

STUDY = Path(__file__).resolve().parents[2]
PHASE = STUDY / "phases/standalone-global-hunk-pilot-01"
ARTIFACT = PHASE / "runs/standalone-global-hunk-pilot-001/standalone-global-hunk-pilot-001-three-feature-sequential-bench-artifacts"
REFERENCE_PROMPT = ARTIFACT / "runs/sequential-feature-1-implement.prompt.txt"
BASE_HOST = PHASE / "infrastructure/drivers/host_custody.py"
HOST_SHA = "5e5be14ae50574e1d3ee81991cb4fee2d33e923ca825655644a1c33aac18b537"

A_SPANS = [
    ("""Design the required tests first, but submit a buildable implementation and those tests together.
This custody procedure supersedes an AGENTS.md requirement to execute a failing test before implementation;
do not manufacture a separate RED run solely to satisfy that procedural timing. All required tests remain required.
Use actual failing checks when genuinely needed to diagnose or repair the assigned work, and preserve their results.""",
     """Before production implementation, publish the required new tests through the host and run the relevant
focused checks to observe their failure for the missing behavior or API. Then implement the feature and
rerun the checks. Follow the repository's executed-test-first requirement, not only test design.
All required tests and genuine failures remain; retain actual failure output and repair real defects."""),
    ("""Add File and Delete File operations are also supported. Submit a cohesive implementation and its
focused tests when both are known; handle genuinely needed repairs after actual feedback.""",
     """Add File and Delete File operations are also supported. Publish each genuinely ready test or implementation
unit through the ordinary edit operation; handle genuinely needed repairs after actual feedback."""),
]
B_LAUNCH = (
    """When the assigned work is complete, send exactly @standalone done. This is not an obligation to stop
after one green check: complete any genuinely remaining task work first. Do not emit directives in""",
    """When the assigned work is complete, send exactly @standalone done. Do not emit directives in""",
)
B_SOURCE = [
    ('. Each accepted edit has its provisional commit already saved. Do not resend or restate accepted patches. Run at most one relevant focused validation step for this accepted work; after it passes, send @standalone done. Send another edit for a concrete issue in your own patch. The complete requested work remains required; concrete unresolved requirements may still be addressed.")',
     '. Each accepted edit has its provisional commit already saved.")'),
    ('+ pending + "\\nUse another requested command or an ordinary edit for concrete remaining work. When the required work and focused validation are complete, send @standalone done."',
     '+ pending'),
    ('deliver("Pending output explicitly discarded. Continue concrete required work or send @standalone done when complete.")',
     'deliver("Pending output explicitly discarded.")'),
]


def replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError("owned source span must occur exactly once")
    return source.replace(before, after, 1)


def transform_prompt(original, arm):
    if arm not in ("identity", "R02", "R03", "R04", "R06"):
        raise ValueError("unsupported or unqualified arm")
    if arm == "R06":
        prefix, marker, _ = original.partition("\n\nThis author session has read-only native tools.")
        if not marker:
            raise ValueError("missing custody prompt boundary")
        return replace_once(
            prefix,
            "Implement the feature with focused tests through the standalone host custody protocol below.\n"
            "The host owns writes, commits, and verification commands. Avoid unrelated refactors.",
            "Implement the feature with focused tests. Leave the changes uncommitted; the benchmark driver will\n"
            "make the commit after your turn. Avoid unrelated refactors.",
        ) + "\n"
    changed = original
    if arm in ("R02", "R04"):
        for before, after in A_SPANS:
            changed = replace_once(changed, before, after)
    if arm in ("R03", "R04"):
        changed = replace_once(changed, *B_LAUNCH)
    return changed


def host_source(arm):
    if arm not in ("identity", "R02", "R03", "R04", "R06"):
        raise ValueError("unsupported or unqualified arm")
    original = BASE_HOST.read_bytes()
    if hashlib.sha256(original).hexdigest() != HOST_SHA:
        raise ValueError("frozen host source digest mismatch")
    changed = original.decode()
    if arm in ("R03", "R04"):
        for before, after in B_SOURCE:
            changed = replace_once(changed, before, after)
    return changed


def load_host(arm):
    module = types.ModuleType("private_benchmark_host")
    module.__file__ = str(BASE_HOST)
    exec(compile(host_source(arm), str(BASE_HOST), "exec"), module.__dict__)
    return module


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--arm", required=True, choices=("R02", "R03", "R04", "R06"))
    for name in ("repo", "artifact-dir", "prompt-file", "codex-bin"):
        parser.add_argument("--" + name, required=True)
    args = parser.parse_args()
    if os.environ.get("WORK_LEAF_BENCH_INVERSE") != "1":
        parser.error("private benchmark opt-in is required")
    root = Path(args.artifact_dir).resolve()
    repo = Path(args.repo).resolve()
    module = load_host(args.arm)
    deadline = time.monotonic() + 900
    print(f"supervisor_pid={os.getpid()} arm={args.arm} artifacts={root} wall_seconds=900", flush=True)
    if args.arm != "R06":
        return module.run_stage(argparse.Namespace(
            repo=str(repo), artifact_dir=str(root / "host"), feature="1",
            stage="initial-author", deadline=deadline, serialized_feedback=True,
            prompt_file=args.prompt_file, resume_thread=None, model="gpt-5.5",
            codex_bin=args.codex_bin,
        ))
    root.mkdir(parents=True, exist_ok=True)
    initial = module.snapshot(repo)
    final_path = root / "native.final.txt"
    argv = [args.codex_bin, "--cd", str(repo), "--sandbox", "workspace-write",
            "--ask-for-approval", "never", "--model", "gpt-5.5",
            "exec", "--color", "never", "--json", "-o", str(final_path), "-"]
    outcome = module.execute_child(
        argv, repo, Path(args.prompt_file), root / "native.stdout.jsonl",
        root / "native.stderr.txt", max(1, int(deadline - time.monotonic())),
        os.environ.copy(),
    )
    module.save_json(root / "native.exit.json", outcome)
    error = None
    thread = None
    if outcome["exit_code"] == 0 and not outcome["timed_out"] and not outcome["cancelled_signal"]:
        try:
            thread, _, _ = module.owned_final(root / "native.stdout.jsonl", final_path, None)
            current = module.snapshot(repo)
            if current["head"] != initial["head"]:
                error = "native author changed HEAD despite the uncommitted-output policy"
            elif not module.git(repo, "status", "--porcelain"):
                error = "native author left no implementation changes"
            else:
                module.git(repo, "add", "-A")
                module.git(repo, "commit", "-m", "UPDATE feature 1: implement the assigned behavior")
        except (module.Fatal, module.Rejected, OSError) as exc:
            error = str(exc)
    else:
        error = "native author invocation did not finish successfully"
    result = {
        "arm": args.arm, "thread_id": thread, "completed": error is None,
        "invocation": outcome, "error": error, "initial_head": initial["head"],
        "final_head": module.snapshot(repo)["head"], "final_file": str(final_path),
        "scope": "one native initial-author episode; no reviews or integration",
    }
    module.save_json(root / "native.result.json", result)
    return 0 if result["completed"] else 1


if __name__ == "__main__":
    sys.exit(main())
