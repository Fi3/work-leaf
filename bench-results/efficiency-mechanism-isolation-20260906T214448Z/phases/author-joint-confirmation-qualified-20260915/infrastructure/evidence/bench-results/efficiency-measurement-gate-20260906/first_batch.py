#!/usr/bin/env python3
"""Freeze and launch one Direct/normal-Work-Leaf pair, without replacement runs."""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import random
import shutil
import signal
import subprocess
import sys
import time


REPO = Path(__file__).resolve().parents[2]
BASE_COMMIT = "c92a0b7060a36eac6db2d869b85e589a7a9480f9"
DRIVERS = (
    "bench-three-features", "bench-three-features-sequential", "bench-three-features-direct-common",
    "bench-candidate-common", "bench-validation-common", "bench-agent-profile-common", "bench-progress-common",
)
AUTH_ENV = {"OPENAI_API_KEY", "CODEX_API_KEY", "OPENAI_BASE_URL", "OPENAI_API_BASE", "CODEX_BASE_URL", "CODEX_ACCESS_TOKEN"}


def now():
    return datetime.now(timezone.utc).isoformat()


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_new(path, value):
    with Path(path).open("x", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2, sort_keys=True, allow_nan=False)
        handle.write("\n")
        handle.flush()
        os.fsync(handle.fileno())


def git(source, *arguments):
    return subprocess.run(["git", "-C", str(source), *arguments], check=True, text=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout.strip()


def schedule(batch_root, source_repo, runtime_root):
    rows = []
    for condition, driver, stem, workflow in (
        ("direct", "bench-three-features-sequential", "three-feature-sequential-bench", "direct-sequential"),
        ("work-leaf", "bench-three-features", "three-feature-bench", "work-leaf-concurrent"),
    ):
        run_id = f"{condition}-001"
        results = batch_root / "runs" / run_id
        artifact = results / f"{run_id}-{stem}-artifacts"
        rows.append({"run_id": run_id, "condition": condition, "workflow": workflow,
                     "driver": str(source_repo / driver), "results_dir": str(results),
                     "runtime_dir": str(runtime_root / run_id), "artifact": str(artifact),
                     "report": str(artifact / "report.json")})
    return rows


def run_environment(manifest, row, inherited):
    env = {key: value for key, value in inherited.items()
           if not key.startswith("WORK_LEAF_") and key not in AUTH_ENV}
    env.update({
        "PATH": manifest["provider_dir"] + os.pathsep + env.get("PATH", os.defpath),
        "WORK_LEAF_BENCH_OBSERVER_BIN": str(Path(manifest["bin_dir"]) / "bench-observer"),
        "WORK_LEAF_BENCH_STUDY_ID": manifest["study"], "WORK_LEAF_BENCH_PAIR_ID": "first-pair",
    })
    if row["condition"] == "direct":
        prefix = "WORK_LEAF_DIRECT_BENCH_"
        env.update({prefix + "REVIEW_ROUNDS": "0", prefix + "AGENT": "codex",
                    prefix + "SANDBOX": "workspace-write", prefix + "REVIEW_SANDBOX": "read-only",
                    prefix + "LINEARIZE_SANDBOX": "danger-full-access"})
    elif row["condition"] == "work-leaf":
        prefix = "WORK_LEAF_BENCH_"
        env.update({prefix + "BUSY_STALL_SECS": "1800", prefix + "IDLE_STALL_SECS": "300",
                    prefix + "NO_READ_PERMISSION": "0", prefix + "WEB_UI": "0",
                    prefix + "DISABLE_TMUX_SUPERVISOR": "1", prefix + "LISTEN": "127.0.0.1:0",
                    "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS": "1000",
                    "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME": "forward",
                    "WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE": "1"})
    else:
        raise ValueError("only Direct and normal Work Leaf are allowed")
    env.update({prefix + "MODEL": "gpt-5.5", prefix + "REASONING_EFFORT": "xhigh",
                prefix + "TMPDIR": row["runtime_dir"], prefix + "RESULTS_DIR": row["results_dir"],
                prefix + "SKIP_BUILD": "1", prefix + "BIN_DIR": manifest["bin_dir"],
                prefix + "TIMEOUT_SECS": "7200", prefix + "RUN_ID": row["run_id"]})
    return env


def validate_schedule(rows):
    if (not isinstance(rows, list) or len(rows) != 2
            or [row.get("condition") for row in rows] != ["direct", "work-leaf"]
            or [row.get("run_id") for row in rows] != ["direct-001", "work-leaf-001"]):
        raise ValueError("schedule must contain exactly the declared Direct/normal Work Leaf pair")


def claim_batch(batch_root):
    (Path(batch_root) / "RUN-ONCE").mkdir()


def choose_launch_order(rows, rng):
    order = [row["run_id"] for row in rows]
    rng.shuffle(order)
    return order


def prepare(args):
    batch = args.batch_root.resolve()
    source = args.source_repo.resolve()
    runtime = args.runtime_root.resolve()
    if git(source, "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("source snapshot must be clean; do not copy the dirty working-tree config")
    commit = git(source, "rev-parse", "HEAD")
    git(source, "cat-file", "-e", BASE_COMMIT + "^{commit}")
    if len(args.task_list_sha256) != 64 or any(c not in "0123456789abcdef" for c in args.task_list_sha256):
        raise ValueError("task-list SHA-256 must be explicit")
    batch.mkdir(parents=True, exist_ok=True)
    for name in ("FIRST-BATCH-MANIFEST.json", "SCHEDULE.json", "infrastructure", "RUN-ONCE"):
        if (batch / name).exists():
            raise FileExistsError(f"prepared state already exists: {name}")
    infrastructure = batch / "infrastructure"
    infrastructure.mkdir()
    binary_dir = infrastructure / "bin"
    provider_dir = infrastructure / "provider"
    binary_dir.mkdir()
    provider_dir.mkdir()
    files = []

    def record(path, role):
        path = Path(path).resolve()
        files.append({"path": str(path), "role": role, "sha256": sha256(path)})

    def freeze(path, destination, role, executable=False):
        path = Path(path).resolve()
        if not path.is_file() or (executable and not os.access(path, os.X_OK)):
            raise ValueError(f"required {role} is not a regular runnable file")
        destination.parent.mkdir(parents=True, exist_ok=True)
        with path.open("rb") as original, destination.open("xb") as copied:
            shutil.copyfileobj(original, copied)
        destination.chmod(0o500 if executable else 0o400)
        record(destination, role)

    for name in ("work-leaf", "work-leaf-orchestrator"):
        freeze(args.bin_dir / name, binary_dir / name, "runtime", True)
    freeze(args.observer_bin, binary_dir / "bench-observer", "observer", True)
    canonical_wrapper = Path(__file__).with_name("subscription-codex")
    if sha256(args.subscription_wrapper) != sha256(canonical_wrapper):
        raise ValueError("subscription wrapper does not match this study's reviewed wrapper")
    freeze(args.subscription_wrapper, provider_dir / "codex", "subscription-only-provider", True)
    for name in DRIVERS:
        record(source / name, "source-driver")
        freeze(source / name, infrastructure / "drivers" / name, "frozen-driver")
    for path in args.identity_file:
        record(path, "external-identity-only")
    evidence = {Path(path).resolve() for path in args.evidence}
    evidence.add(Path(__file__).resolve())
    for path in sorted(evidence):
        relative = path.relative_to(REPO)
        if relative.parts[0] == ".codex" or path.name in {"auth.json", ".env"}:
            raise ValueError("credential/config files are not copied as evidence; use identity hashes only")
        freeze(path, infrastructure / "evidence" / relative, "frozen-evidence")
    manifest = {"schema_version": 1, "study": batch.name, "prepared_at": now(),
                "source_repo": str(source), "source_commit": commit, "base_commit": BASE_COMMIT,
                "runtime_root": str(runtime), "bin_dir": str(binary_dir), "provider_dir": str(provider_dir),
                "model": "gpt-5.5", "reasoning_effort": "xhigh", "provider_route": "existing-codex-chatgpt-subscription",
                "task_list_sha256": args.task_list_sha256, "maximum_concurrent_workflows": 2,
                "replacements_allowed": False, "pause_after_pair": True, "files": files,
                "runner_sha256": sha256(Path(__file__)), "supervisor_wall_timeout_seconds": 86400,
                "schedule": schedule(batch, source, runtime)}
    for row in manifest["schedule"]:
        row["environment_overrides"] = run_environment(manifest, row, {"PATH": "<inherited PATH>"})
    manifest["launch_order"] = choose_launch_order(manifest["schedule"], random.SystemRandom())
    write_new(batch / "SCHEDULE.json", {"runs": manifest["schedule"], "launch_order": manifest["launch_order"]})
    manifest["schedule_sha256"] = sha256(batch / "SCHEDULE.json")
    write_new(batch / "FIRST-BATCH-MANIFEST.json", manifest)
    return manifest


def verify_manifest(batch):
    manifest_path = batch / "FIRST-BATCH-MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    validate_schedule(manifest["schedule"])
    if (manifest.get("maximum_concurrent_workflows") != 2 or manifest.get("replacements_allowed") is not False
            or manifest.get("pause_after_pair") is not True
            or manifest.get("supervisor_wall_timeout_seconds") != 86400):
        raise ValueError("manifest does not declare the one-shot pair limit")
    if sorted(manifest.get("launch_order", [])) != ["direct-001", "work-leaf-001"]:
        raise ValueError("launch order must be a frozen permutation of the pair")
    if manifest.get("runner_sha256") != sha256(Path(__file__)):
        raise ValueError("runner differs from its frozen implementation")
    if (manifest["bin_dir"] != str(batch / "infrastructure" / "bin")
            or manifest["provider_dir"] != str(batch / "infrastructure" / "provider")):
        raise ValueError("runtime executables must come from the batch's frozen directories")
    if manifest["schedule_sha256"] != sha256(batch / "SCHEDULE.json"):
        raise ValueError("frozen schedule changed")
    if json.loads((batch / "SCHEDULE.json").read_text(encoding="utf-8")) != {
            "runs": manifest["schedule"], "launch_order": manifest["launch_order"]}:
        raise ValueError("manifest and frozen schedule disagree")
    source = Path(manifest["source_repo"])
    if git(source, "rev-parse", "HEAD") != manifest["source_commit"] or git(source, "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("frozen source snapshot changed")
    required = {str((batch / "infrastructure" / "bin" / name).resolve())
                for name in ("work-leaf", "work-leaf-orchestrator", "bench-observer")}
    required.add(str((batch / "infrastructure" / "provider" / "codex").resolve()))
    required.update(str((source / name).resolve()) for name in DRIVERS)
    indexed = {entry["path"] for entry in manifest["files"]}
    if not required <= indexed:
        raise ValueError("required frozen executable or driver digest is absent")
    for entry in manifest["files"]:
        if sha256(entry["path"]) != entry["sha256"]:
            raise ValueError(f"frozen {entry['role']} digest changed")
    expected_rows = schedule(batch, source, Path(manifest["runtime_root"]))
    for row, expected in zip(manifest["schedule"], expected_rows, strict=True):
        overrides = run_environment(manifest, row, {"PATH": "<inherited PATH>"})
        if row != {**expected, "environment_overrides": overrides}:
            raise ValueError("schedule paths or environment differ from the declared normal workflows")
    return manifest


def run_batch(batch_root, popen=None):
    batch = Path(batch_root).resolve()
    manifest = verify_manifest(batch)
    for row in manifest["schedule"]:
        for path in (Path(row["results_dir"]), Path(row["runtime_dir"]), batch / "logs" / (row["run_id"] + ".log")):
            if path.exists():
                raise FileExistsError(f"run state already exists: {path}")
    claim_batch(batch)
    (batch / "logs").mkdir(exist_ok=True)
    popen = subprocess.Popen if popen is None else popen
    active = []
    results = [{"id": row["run_id"], "condition": row["condition"], "workflow": row["workflow"],
                "artifact": row["artifact"], "report": row["report"], "launcher_exit_code": None,
                "launch_status": "not_launched", "started_at": None, "finished_at": None}
               for row in manifest["schedule"]]
    by_id = {row["run_id"]: row for row in manifest["schedule"]}
    results_by_id = {row["id"]: row for row in results}
    interrupted = []
    infrastructure_failed = False
    wall_timeout = False
    supervisor_started = time.monotonic()
    old_handlers = {}

    def stop(signum, _frame):
        interrupted.append(signum)
        for process, _, _ in active:
            if process.poll() is None:
                try:
                    os.killpg(process.pid, signum)
                except ProcessLookupError:
                    pass

    for signum in (signal.SIGINT, signal.SIGTERM):
        old_handlers[signum] = signal.signal(signum, stop)
    write_new(batch / "RUN-ONCE" / "admission.json", {
        "started_at": now(), "manifest_sha256": sha256(batch / "FIRST-BATCH-MANIFEST.json"),
        "schedule_sha256": manifest["schedule_sha256"], "workflow_count": 2,
    })
    try:
        for run_id in manifest["launch_order"]:
            row = by_id[run_id]
            result = results_by_id[run_id]
            if interrupted:
                result["launch_status"] = "not_launched_after_signal"
                continue
            # A report does not make an observed launcher failure safe to ignore.
            # Incomplete token accounting alone does not give these drivers a nonzero exit.
            if any(process.poll() not in (None, 0) for process, _, _ in active):
                infrastructure_failed = True
            if infrastructure_failed:
                result["launch_status"] = "not_launched_after_infrastructure_failure"
                continue
            Path(row["results_dir"]).mkdir(parents=True)
            Path(row["runtime_dir"]).mkdir(parents=True)
            log = (batch / "logs" / (row["run_id"] + ".log")).open("xb")
            try:
                process = popen([row["driver"]], cwd=manifest["source_repo"],
                                env=run_environment(manifest, row, os.environ),
                                stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
                                start_new_session=True)
            except OSError as error:
                log.close()
                result.update(launch_status="failed_to_start", error_type=type(error).__name__, finished_at=now())
                infrastructure_failed = True
                continue
            result.update(launch_status="running", started_at=now(), pid=process.pid)
            active.append((process, result, log))
            if interrupted:
                stop(interrupted[-1], None)
        stopping_at = None
        while any(process.poll() is None for process, _, _ in active):
            if not wall_timeout and time.monotonic() - supervisor_started >= manifest["supervisor_wall_timeout_seconds"]:
                wall_timeout = True
                stop(signal.SIGTERM, None)
            if interrupted:
                stopping_at = time.monotonic() if stopping_at is None else stopping_at
                if time.monotonic() - stopping_at > 20:
                    for process, _, _ in active:
                        if process.poll() is None:
                            try:
                                os.killpg(process.pid, signal.SIGKILL)
                            except ProcessLookupError:
                                pass
            time.sleep(0.25)
        for process, result, log in active:
            result.update(launcher_exit_code=process.wait(), launch_status="completed", finished_at=now())
            log.close()
            write_new(batch / "logs" / (result["id"] + ".exit.json"), result)
    finally:
        for signum, handler in old_handlers.items():
            signal.signal(signum, handler)
        # On an unexpected supervisor exception, no started workflow is silently left running.
        for process, result, log in active:
            if process.poll() is None:
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                    process.wait(timeout=20)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                except ProcessLookupError:
                    pass
                result.update(launcher_exit_code=process.poll(), launch_status="supervisor_stopped", finished_at=now())
            log.close()
        report = {"schema_version": 1, "study": manifest["study"], "finished_at": now(),
                  "manifest_sha256": sha256(batch / "FIRST-BATCH-MANIFEST.json"),
                  "runs": results, "signals_received": interrupted, "collection_paused": True,
                  "supervisor_wall_timeout": wall_timeout,
                  "measurement_and_quality_assessed_separately": True}
        write_new(batch / "FIRST-BATCH-RESULT.json", report)
        write_new(batch / "score-manifest.json", {
            **{key: manifest[key] for key in ("study", "base_commit", "task_list_sha256", "model", "reasoning_effort", "source_repo")},
            "schema_version": 1, "runs": results,
        })
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="mode", required=True)
    prep = subparsers.add_parser("prepare", help="freeze inputs; no provider work")
    for name in ("batch-root", "source-repo", "bin-dir", "observer-bin", "subscription-wrapper", "runtime-root"):
        prep.add_argument("--" + name, type=Path, required=True)
    prep.add_argument("--task-list-sha256", required=True)
    prep.add_argument("--evidence", type=Path, action="append", default=[])
    prep.add_argument("--identity-file", type=Path, action="append", default=[])
    for name in ("verify", "run"):
        subparsers.add_parser(name).add_argument("--batch-root", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.mode == "prepare":
            prepare(args)
        elif args.mode == "verify":
            verify_manifest(args.batch_root.resolve())
        else:
            report = run_batch(args.batch_root)
            print(json.dumps({"batch_root": str(args.batch_root), "collection_paused": True, "runs": report["runs"]}))
            return 0 if all(row["launcher_exit_code"] == 0 for row in report["runs"]) else 1
        print(json.dumps({"batch_root": str(args.batch_root), "mode": args.mode, "provider_work_started": False}))
        return 0
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"first batch: {type(error).__name__}: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
