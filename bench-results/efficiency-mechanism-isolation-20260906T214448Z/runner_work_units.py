#!/usr/bin/env python3
"""Freeze and supervise the v2 buildable-work-unit experiment, three workflows per wave.

This separate successor preserves the v1 supervisor and frozen studies. Its only
experimental condition is the declared complete buildable-work-unit policy.

`phase_schedule_exhausted` means the schedule loop reached its natural end, including
rows explicitly withheld after an integrity failure; it is false on an exception.
`all_scheduled_workflows_launched` independently requires a start timestamp for every row.
Neither field asserts workflow, measurement, or experiment success.

Raw, parsed, behavioral hashes and the legacy behavioral drift flag are retained.
Only the separately frozen own-workflow trust classifier can explain a prescribed
trust addition; unexplained drift stops new admissions while active workflows finish.

Frozen-input verification costs O(W * E) for W waves and E frozen evidence bytes.
E includes per-run manifests, so metadata work can be quadratic in growing run count.
The admitted study phases are bounded at three screening or twelve confirmation runs.
"""

import argparse
from collections import Counter, defaultdict
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import random
import re
import shutil
import signal
import subprocess
import sys
import time
import tomllib
import types


REPO = Path(__file__).resolve().parents[2]
TRUST_PATH = Path(__file__).resolve().with_name("trust_work_units.py")
_trust_source = TRUST_PATH.read_bytes()
TRUST_LOADED_SHA256 = hashlib.sha256(_trust_source).hexdigest()
_trust_module = types.ModuleType("work_units_trust")
_trust_module.__file__ = str(TRUST_PATH)
exec(compile(_trust_source, str(TRUST_PATH), "exec"), _trust_module.__dict__)
TrustClassifier = _trust_module.TrustClassifier
CANONICAL_WRAPPER = REPO / "bench-results/efficiency-measurement-gate-20260906/subscription-codex"
WRAPPER_SHA256 = "0977db361b4477e2bf68ea08c05571a4a35fb1e5c958cc9a953f2cc738de7078"
BASE_COMMIT = "c92a0b7060a36eac6db2d869b85e589a7a9480f9"
CONDITIONS = ("control", "buildable-work-unit-incremental")
EXPERIMENT_SCHEMA = "work-leaf-bench-experiment-v2"
DRIVERS = ("bench-three-features", "bench-candidate-common", "bench-validation-common",
           "bench-agent-profile-common", "bench-progress-common")
AUTH_ENV = {"OPENAI_API_KEY", "CODEX_API_KEY", "OPENAI_BASE_URL", "OPENAI_API_BASE",
            "CODEX_BASE_URL", "CODEX_ACCESS_TOKEN"}
BOOKKEEPING_NOTICE_KEYS = {"hide_full_access_warning", "hide_gpt5_1_migration_prompt",
                           "hide_rate_limit_model_nudge", "model_migrations"}
SAFE_ENUMS = {
    "model": {"gpt-5.5"}, "model_provider": {"openai"},
    "forced_login_method": {"chatgpt", "api"},
    "model_reasoning_effort": {"none", "minimal", "low", "medium", "high", "xhigh"},
    "model_reasoning_summary": {"auto", "concise", "detailed", "none"},
    "model_verbosity": {"low", "medium", "high"},
    "approval_policy": {"untrusted", "on-failure", "on-request", "never"},
    "sandbox_mode": {"read-only", "workspace-write", "danger-full-access"},
    "web_search": {"disabled", "cached", "live"},
    "service_tier": {"fast", "flex"},
}
PINNED_SETTINGS = {"forced_login_method": "chatgpt", "model_provider": "openai",
                   "model": "gpt-5.5", "model_reasoning_effort": "xhigh"}


def now():
    return datetime.now(timezone.utc).isoformat()


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def digest_value(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"),
                                     default=str, allow_nan=False).encode()).hexdigest()


def write_new(path, value):
    with Path(path).open("x", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2, sort_keys=True, allow_nan=False)
        handle.write("\n")
        handle.flush()
        os.fsync(handle.fileno())


def git(source, *arguments):
    return subprocess.run(["git", "-C", str(source), *arguments], check=True, text=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout.strip()


def config_snapshot(path):
    """Hash all config; serialize only fixed nonsecret enum values, never arbitrary values."""
    record = {"path": str(Path(path).resolve()), "captured_at": now(), "parse_valid": False,
              "safe_global_settings": {}, "pinned_command_overrides": PINNED_SETTINGS,
              "scope": "global file plus declared CLI overrides; not a full effective-config resolver"}
    try:
        content = Path(path).read_bytes()
        record["sha256"] = hashlib.sha256(content).hexdigest()
        parsed = tomllib.loads(content.decode("utf-8"))
        record["parsed_sha256"] = digest_value(parsed)
        record["safe_global_settings"] = {
            key: value for key, allowed in SAFE_ENUMS.items()
            if isinstance(value := parsed.get(key), str) and value in allowed}
        behavior = dict(parsed)
        notice = behavior.get("notice")
        if isinstance(notice, dict):
            remaining = {key: value for key, value in notice.items() if key not in BOOKKEEPING_NOTICE_KEYS}
            if remaining:
                behavior["notice"] = remaining
            else:
                behavior.pop("notice")
        record.update(parse_valid=True, behavioral_sha256=digest_value(behavior))
    except (OSError, UnicodeError, ValueError) as error:
        record["error_type"] = type(error).__name__
    return record


def config_drift(before, after):
    if not before.get("parse_valid") or not after.get("parse_valid"):
        return "unreadable_or_invalid"
    if before["sha256"] == after["sha256"]:
        return "unchanged"
    if before["parsed_sha256"] == after["parsed_sha256"]:
        return "formatting_only"
    if before["behavioral_sha256"] == after["behavioral_sha256"]:
        return "known_bookkeeping_only"
    return "behavioral_or_unknown"


def identifier(value):
    return isinstance(value, str) and re.fullmatch(r"[a-zA-Z0-9][a-zA-Z0-9_-]{0,95}", value)


def validate_plan(plan):
    if not isinstance(plan, dict) or not identifier(plan.get("phase")):
        raise ValueError("phase must be a safe explicit identifier")
    if plan.get("phase_kind") not in {"screening", "confirmation"}:
        raise ValueError("phase_kind must explicitly separate screening from confirmation")
    rows = plan.get("runs")
    if not isinstance(rows, list) or not rows:
        raise ValueError("an explicit nonempty run schedule is required")
    identifiers = set()
    waves = defaultdict(list)
    counts = defaultdict(Counter)
    for row in rows:
        if (not isinstance(row, dict) or set(row) != {"run_id", "condition", "wave", "block_id"}
                or not identifier(row["run_id"]) or row["run_id"] in identifiers
                or not identifier(row["block_id"]) or row["condition"] not in CONDITIONS
                or type(row["wave"]) is not int or row["wave"] < 1):
            raise ValueError("invalid, duplicate, or undeclared scheduled workflow")
        identifiers.add(row["run_id"])
        waves[row["wave"]].append(row)
        counts[row["block_id"]][row["condition"]] += 1
    if sorted(waves) != list(range(1, len(waves) + 1)):
        raise ValueError("wave numbers must be contiguous starting at one")
    for index, members in waves.items():
        if len(members) > 3 or (index < len(waves) and len(members) != 3):
            raise ValueError("use three top-level workflows per wave, except a smaller final remainder")
    allocation = plan.get("randomization")
    if (not isinstance(allocation, dict) or not isinstance(allocation.get("method"), str)
            or not allocation["method"] or type(allocation.get("mixed_waves")) is not bool
            or allocation.get("unit") != "workflow" or allocation.get("scheme") != "within_block"
            or allocation.get("block_condition_counts") != {key: dict(value) for key, value in counts.items()}):
        raise ValueError("randomization metadata must identify the exact scheduled block counts")
    if allocation["mixed_waves"] and any(len({row["condition"] for row in members}) < 2
                                         for members in waves.values()):
        raise ValueError("declared mixed waves must contain at least two conditions")


def schedule(phase_root, source_repo, runtime_root, plan):
    validate_plan(plan)
    rows = []
    for declaration in plan["runs"]:
        run_id = declaration["run_id"]
        results = phase_root / "runs" / run_id
        artifact = results / f"{run_id}-three-feature-bench-artifacts"
        rows.append({**declaration, "phase": plan["phase"], "workflow": "work-leaf-concurrent",
                     "driver": str(source_repo / "bench-three-features"), "results_dir": str(results),
                     "runtime_dir": str(runtime_root / run_id), "artifact": str(artifact),
                     "report": str(artifact / "report.json"),
                     "prompt_trace": str(phase_root / "prompt-events" / (run_id + ".jsonl")),
                     "experiment_manifest": str(phase_root / "experiments" / (run_id + ".json"))})
    return rows


def run_environment(manifest, row, inherited):
    if row["condition"] not in CONDITIONS:
        raise ValueError("only declared Work Leaf conditions are allowed")
    env = {key: value for key, value in inherited.items()
           if not key.startswith("WORK_LEAF_") and key not in AUTH_ENV}
    env.update({
        "PATH": manifest["provider_dir"] + os.pathsep + env.get("PATH", os.defpath),
        "WORK_LEAF_BENCH_OBSERVER_BIN": str(Path(manifest["bin_dir"]) / "bench-observer"),
        "WORK_LEAF_BENCH_STUDY_ID": manifest["study"], "WORK_LEAF_BENCH_PAIR_ID": row["block_id"],
        "WORK_LEAF_BENCH_MODEL": "gpt-5.5", "WORK_LEAF_BENCH_REASONING_EFFORT": "xhigh",
        "WORK_LEAF_BENCH_TMPDIR": row["runtime_dir"], "WORK_LEAF_BENCH_RESULTS_DIR": row["results_dir"],
        "WORK_LEAF_BENCH_SKIP_BUILD": "1", "WORK_LEAF_BENCH_BIN_DIR": manifest["bin_dir"],
        "WORK_LEAF_BENCH_TIMEOUT_SECS": "7200", "WORK_LEAF_BENCH_BUSY_STALL_SECS": "1800",
        "WORK_LEAF_BENCH_IDLE_STALL_SECS": "300", "WORK_LEAF_BENCH_RUN_ID": row["run_id"],
        "WORK_LEAF_BENCH_NO_READ_PERMISSION": "0", "WORK_LEAF_BENCH_WEB_UI": "0",
        "WORK_LEAF_BENCH_DISABLE_TMUX_SUPERVISOR": "1", "WORK_LEAF_BENCH_LISTEN": "127.0.0.1:0",
        "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS": "1000",
        "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME": "forward",
        "WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE": "1", "WORK_LEAF_BENCH_EXPERIMENT": "1",
        "WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY": "1",
        "WORK_LEAF_BENCH_EXPERIMENT_MANIFEST": row["experiment_manifest"],
    })
    return env


def claim_phase(phase_root):
    (phase_root / "RUN-ONCE").mkdir()


def classify_outcome(row, exit_code):
    """Separate published workflow failures and accounting gaps from broken run integrity."""
    failure = "infrastructure_or_measurement_integrity_failure"
    try:
        report = json.loads(Path(row["report"]).read_text(encoding="utf-8"))
        analysis = json.loads((Path(row["artifact"]) / "observation/analysis.json").read_text(encoding="utf-8"))
        with Path(row["prompt_trace"]).open(encoding="utf-8") as trace:
            activation = json.loads(trace.readline())
        if (activation.get("event") != "activation" or activation.get("schema") != EXPERIMENT_SCHEMA
                or activation.get("run_id") != row["run_id"] or activation.get("condition") != row["condition"]):
            return failure
        required = {"run_id": row["run_id"], "bench_mode": "work-leaf", "feature_schedule": "concurrent",
                    "agent_backend": "codex", "agent_transport": "app-server", "agent_model": "gpt-5.5",
                    "agent_reasoning_effort": "xhigh", "codex_cli_version": "codex-cli 0.153.4",
                    "base_commit": BASE_COMMIT}
        if any(report.get(key) != value for key, value in required.items()):
            return failure
        outcome = report.get("workflow_result")
        if outcome not in {"pass", "fail"} or report.get("result") != outcome:
            return failure
        if (outcome == "pass" and exit_code != 0) or not isinstance(exit_code, int) or exit_code < 0:
            return failure
        errors = analysis.get("errors")
        if not isinstance(errors, list) or any(not isinstance(error, str) or not re.fullmatch(
                r"interrupted provider turn has no complete usage: count=[1-9][0-9]*", error) for error in errors):
            return failure
        complete = analysis.get("capture_complete")
        if ((complete is True and (errors or report.get("measurement_status") != "complete"))
                or (complete is False and (not errors or report.get("measurement_status") != "incomplete"))
                or type(complete) is not bool):
            return failure
        usage = analysis["usage_scopes"]["total_workflow"]
        keys = ("input_tokens", "cached_input_tokens", "uncached_input_tokens", "output_tokens",
                "reasoning_output_tokens", "raw_input_plus_output", "uncached_input_plus_output")
        if any(type(usage.get(key)) is not int or usage[key] < 0 for key in keys):
            return failure
        if (usage["input_tokens"] != usage["cached_input_tokens"] + usage["uncached_input_tokens"]
                or usage["raw_input_plus_output"] != usage["input_tokens"] + usage["output_tokens"]
                or usage["uncached_input_plus_output"] != usage["uncached_input_tokens"] + usage["output_tokens"]
                or usage["reasoning_output_tokens"] > usage["output_tokens"]
                or any(report["total_workflow_usage"].get(key) != usage[key] for key in keys)):
            return failure
        return "recorded_workflow_success" if outcome == "pass" else "recorded_workflow_failure"
    except (OSError, ValueError, KeyError, TypeError, AttributeError):
        return failure


def prepare(args):
    if sha256(TRUST_PATH) != TRUST_LOADED_SHA256:
        raise ValueError("trust helper source differs from executing code")
    phase = args.phase_root.resolve()
    source = args.source_repo.resolve()
    runtime = args.runtime_root.resolve()
    plan = json.loads(args.schedule.read_text(encoding="utf-8"))
    validate_plan(plan)
    if git(source, "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("driver source must be a clean snapshot; freeze experimental source separately")
    commit = git(source, "rev-parse", "HEAD")
    git(source, "cat-file", "-e", BASE_COMMIT + "^{commit}")
    if not re.fullmatch(r"[0-9a-f]{64}", args.task_list_sha256):
        raise ValueError("task SHA-256 must be explicit")
    if sha256(args.subscription_wrapper) != WRAPPER_SHA256:
        raise ValueError("subscription wrapper differs from the reviewed subscription-only route")
    initial_config = config_snapshot(args.global_config)
    if not initial_config["parse_valid"]:
        raise ValueError("global configuration is unreadable or invalid before preparation")
    phase.mkdir(parents=True, exist_ok=True)
    for name in ("PHASE-MANIFEST.json", "SCHEDULE.json", "infrastructure", "experiments", "RUN-ONCE"):
        if (phase / name).exists():
            raise FileExistsError(f"prepared phase already exists: {name}")
    infrastructure = phase / "infrastructure"
    binary_dir = infrastructure / "bin"
    provider_dir = infrastructure / "provider"
    binary_dir.mkdir(parents=True)
    provider_dir.mkdir()
    (phase / "experiments").mkdir()
    (phase / "prompt-events").mkdir(exist_ok=True)
    files = []

    def record(path, role):
        path = Path(path).resolve()
        files.append({"path": str(path), "role": role, "sha256": sha256(path)})

    def freeze(path, destination, role, executable=False):
        path = Path(path).resolve()
        if not path.is_file() or (executable and not os.access(path, os.X_OK)):
            raise ValueError(f"required {role} must be a regular runnable file")
        if ".codex" in path.parts or path.name in {"auth.json", ".env", "config.toml"}:
            raise ValueError("configuration or credential files cannot be copied as evidence")
        destination.parent.mkdir(parents=True, exist_ok=True)
        with path.open("rb") as original, destination.open("xb") as copied:
            shutil.copyfileobj(original, copied)
        destination.chmod(0o500 if executable else 0o400)
        record(destination, role)

    for name in ("work-leaf", "work-leaf-orchestrator"):
        freeze(args.bin_dir / name, binary_dir / name, "runtime", True)
    freeze(args.observer_bin, binary_dir / "bench-observer", "observer", True)
    freeze(args.subscription_wrapper, provider_dir / "codex", "subscription-only-provider", True)
    for name in DRIVERS:
        record(source / name, "source-driver")
        freeze(source / name, infrastructure / "drivers" / name, "frozen-driver")
    for path in args.identity_file:
        record(path, "external-identity-only")
    freeze(args.protocol, infrastructure / "PROTOCOL.md", "protocol")
    freeze(args.scorer_config, infrastructure / "SCORER.json", "scorer-config")
    evidence = {Path(path).resolve() for path in args.evidence}
    evidence.add(Path(__file__).resolve())
    evidence.add(TRUST_PATH)
    for path in sorted(evidence):
        relative = path.relative_to(args.evidence_root.resolve())
        freeze(path, infrastructure / "evidence" / relative, "frozen-evidence")
    manifest = {"schema_version": 1, "study": phase.parent.parent.name if phase.parent.name == "phases" else phase.name,
                "phase": plan["phase"], "phase_kind": plan["phase_kind"], "prepared_at": now(), "plan": plan,
                "source_repo": str(source), "source_commit": commit, "base_commit": BASE_COMMIT,
                "runtime_root": str(runtime), "bin_dir": str(binary_dir), "provider_dir": str(provider_dir),
                "model": "gpt-5.5", "reasoning_effort": "xhigh", "compiled_feature": "bench-experiments",
                "provider_route": "existing-codex-chatgpt-subscription", "task_list_sha256": args.task_list_sha256,
                "maximum_concurrent_workflows": 3, "replacements_allowed": False,
                "supervisor_wall_timeout_seconds": 86400, "runner_sha256": sha256(__file__),
                "trust_classifier_sha256": TRUST_LOADED_SHA256,
                "randomization": {**plan["randomization"], "launch_order_method": "system-random-shuffle-within-declared-wave"},
                "global_config_baseline": initial_config, "files": files,
                "schedule": schedule(phase, source, runtime, plan)}
    for row in manifest["schedule"]:
        write_new(row["experiment_manifest"], {"schema": EXPERIMENT_SCHEMA,
                  "run_id": row["run_id"], "condition": row["condition"], "evidence_path": row["prompt_trace"]})
        Path(row["experiment_manifest"]).chmod(0o400)
        record(row["experiment_manifest"], "experiment-manifest")
        row["environment_overrides"] = run_environment(manifest, row, {"PATH": "<inherited PATH>"})
    waves = defaultdict(list)
    for row in manifest["schedule"]:
        waves[row["wave"]].append(row["run_id"])
    manifest["launch_order"] = [waves[index] for index in sorted(waves)]
    for wave in manifest["launch_order"]:
        random.SystemRandom().shuffle(wave)
    write_new(phase / "SCHEDULE.json", {"runs": manifest["schedule"], "launch_order": manifest["launch_order"],
                                       "randomization": manifest["randomization"]})
    manifest["schedule_sha256"] = sha256(phase / "SCHEDULE.json")
    write_new(phase / "PHASE-MANIFEST.json", manifest)
    return manifest


def verify_manifest(phase):
    manifest = json.loads((phase / "PHASE-MANIFEST.json").read_text(encoding="utf-8"))
    validate_plan(manifest["plan"])
    if (manifest.get("maximum_concurrent_workflows") != 3 or manifest.get("replacements_allowed") is not False
            or manifest.get("compiled_feature") != "bench-experiments"
            or manifest.get("model") != "gpt-5.5" or manifest.get("reasoning_effort") != "xhigh"
            or manifest.get("provider_route") != "existing-codex-chatgpt-subscription"
            or manifest.get("supervisor_wall_timeout_seconds") != 86400
            or manifest.get("runner_sha256") != sha256(__file__)
            or manifest.get("trust_classifier_sha256") != TRUST_LOADED_SHA256
            or TRUST_LOADED_SHA256 != sha256(TRUST_PATH)):
        raise ValueError("manifest differs from the frozen study workflow or runner")
    if (manifest["bin_dir"] != str(phase / "infrastructure" / "bin")
            or manifest["provider_dir"] != str(phase / "infrastructure" / "provider")):
        raise ValueError("runtime executables must use frozen phase directories")
    if manifest["schedule_sha256"] != sha256(phase / "SCHEDULE.json"):
        raise ValueError("frozen schedule changed")
    if json.loads((phase / "SCHEDULE.json").read_text(encoding="utf-8")) != {
            "runs": manifest["schedule"], "launch_order": manifest["launch_order"],
            "randomization": manifest["randomization"]}:
        raise ValueError("schedule and manifest disagree")
    source = Path(manifest["source_repo"])
    if git(source, "rev-parse", "HEAD") != manifest["source_commit"] or git(source, "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("clean driver source snapshot changed")
    expected_rows = schedule(phase, source, Path(manifest["runtime_root"]), manifest["plan"])
    if len(manifest["schedule"]) != len(expected_rows):
        raise ValueError("scheduled row count changed")
    for row, expected in zip(manifest["schedule"], expected_rows, strict=True):
        if row != {**expected, "environment_overrides": run_environment(manifest, expected, {"PATH": "<inherited PATH>"})}:
            raise ValueError("schedule paths or environments differ from the declared Work Leaf workflow")
    expected_waves = defaultdict(list)
    for row in manifest["schedule"]:
        expected_waves[row["wave"]].append(row["run_id"])
    if ([sorted(wave) for wave in manifest["launch_order"]]
            != [sorted(expected_waves[index]) for index in sorted(expected_waves)]):
        raise ValueError("launch order must be a permutation within each declared wave")
    if manifest["randomization"] != {**manifest["plan"]["randomization"],
                                       "launch_order_method": "system-random-shuffle-within-declared-wave"}:
        raise ValueError("randomization metadata changed")
    required = {str((phase / "infrastructure" / "bin" / name).resolve())
                for name in ("work-leaf", "work-leaf-orchestrator", "bench-observer")}
    required.update(str((source / name).resolve()) for name in DRIVERS)
    required.update(row["experiment_manifest"] for row in manifest["schedule"])
    required.update(str(phase / "infrastructure" / name) for name in ("PROTOCOL.md", "SCORER.json"))
    required.add(str(phase / "infrastructure" / "provider" / "codex"))
    required.add(str(phase / "infrastructure" / "evidence" / TRUST_PATH.relative_to(REPO)))
    indexed = {entry["path"] for entry in manifest["files"]}
    if not required <= indexed or len(indexed) != len(manifest["files"]):
        raise ValueError("required frozen identity is absent or duplicated")
    for entry in manifest["files"]:
        if sha256(entry["path"]) != entry["sha256"]:
            raise ValueError(f"frozen {entry['role']} digest changed")
    return manifest


def run_phase(phase_root, popen=None):
    phase = Path(phase_root).resolve()
    manifest = verify_manifest(phase)
    for row in manifest["schedule"]:
        for name in (row["results_dir"], row["runtime_dir"], row["prompt_trace"],
                     phase / "logs" / (row["run_id"] + ".log")):
            if Path(name).exists():
                raise FileExistsError(f"run state already exists: {name}")
    for name in ("PHASE-RESULT.json", "score-manifest.json", "config-history.json", "config-history.jsonl",
                 "trust-evidence.jsonl", "trust-pending.jsonl", "trust-final.json"):
        if (phase / name).exists():
            raise FileExistsError(f"result already exists: {name}")
    trust_classifier = TrustClassifier(manifest, phase)
    claim_phase(phase)
    (phase / "logs").mkdir(exist_ok=True)
    config_journal = (phase / "config-history.jsonl").open("x", encoding="utf-8")
    popen = subprocess.Popen if popen is None else popen
    results = [{**{key: row[key] for key in ("run_id", "condition", "phase", "block_id", "wave", "workflow",
                 "artifact", "report", "prompt_trace", "experiment_manifest")}, "id": row["run_id"], "launcher_exit_code": None,
                "launch_status": "not_launched", "started_at": None, "finished_at": None}
               for row in manifest["schedule"]]
    by_id = {row["run_id"]: row for row in manifest["schedule"]}
    result_by_id = {row["id"]: row for row in results}
    active = []
    interrupted = []
    config_history = []
    infrastructure_failed = False
    behavioral_drift = False
    unexplained_drift = False
    pending_config = False
    integrity_errors = []
    wall_timeout = False
    supervisor_error = None
    schedule_exhausted = False
    supervisor_started = time.monotonic()
    stopping_at = None
    last_config_poll = supervisor_started
    old_handlers = {}
    admitted_manifest_sha256 = sha256(phase / "PHASE-MANIFEST.json")

    def verify_frozen_inputs(reason):
        nonlocal infrastructure_failed
        try:
            verified = verify_manifest(phase)
            if verified != manifest:
                raise ValueError("admitted manifest changed")
        except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
            integrity_errors.append({"reason": reason, "captured_at": now(), "error_type": type(error).__name__})
            infrastructure_failed = True

    def snapshot_config(reason):
        nonlocal behavioral_drift, unexplained_drift, pending_config
        snapshot = config_snapshot(manifest["global_config_baseline"]["path"])
        drift = config_drift(manifest["global_config_baseline"], snapshot)
        classification = trust_classifier.classify(snapshot, drift,
                            {result["id"] for result in results if result["started_at"] is not None})
        config_history.append({"reason": reason, "drift_from_baseline": drift, **snapshot,
                               "trust_classification": classification})
        config_journal.write(json.dumps(config_history[-1], sort_keys=True, allow_nan=False) + "\n")
        config_journal.flush()
        os.fsync(config_journal.fileno())
        if drift in {"behavioral_or_unknown", "unreadable_or_invalid"}:
            behavioral_drift = True
        pending_config = classification.get("pending_transition", False)
        unexplained_drift |= not classification["allows_admission"] and not pending_config
        return len(config_history) - 1

    def stop(signum, _frame):
        interrupted.append(signum)
        for process, _result, _log in active:
            if process.poll() is None:
                try:
                    os.killpg(process.pid, signum)
                except ProcessLookupError:
                    pass

    for signum in (signal.SIGINT, signal.SIGTERM):
        old_handlers[signum] = signal.signal(signum, stop)
    write_new(phase / "RUN-ONCE" / "admission.json", {"started_at": now(),
              "manifest_sha256": sha256(phase / "PHASE-MANIFEST.json"),
              "schedule_sha256": manifest["schedule_sha256"], "workflow_count": len(results)})
    try:
        for wave in manifest["launch_order"]:
            verify_frozen_inputs("before-wave:" + str(by_id[wave[0]]["wave"]))
            for run_id in wave:
                row = by_id[run_id]
                result = result_by_id[run_id]
                result["prelaunch_config_snapshot"] = snapshot_config("before-launch:" + run_id)
                while pending_config and not (interrupted or unexplained_drift or infrastructure_failed):
                    # Only future admission waits; active provider work and its streams are untouched.
                    time.sleep(0.25)
                    result["prelaunch_config_snapshot"] = snapshot_config("awaiting-native-attestation:" + run_id)
                for process, previous, _log in active:
                    exit_code = process.poll()
                    if exit_code is not None:
                        previous["outcome_classification"] = classify_outcome(by_id[previous["id"]], exit_code)
                        infrastructure_failed |= previous["outcome_classification"] == "infrastructure_or_measurement_integrity_failure"
                if interrupted or unexplained_drift or infrastructure_failed:
                    suffix = "signal" if interrupted else "config_drift" if unexplained_drift else "infrastructure_failure"
                    result["launch_status"] = "not_launched_after_" + suffix
                    continue
                Path(row["results_dir"]).mkdir(parents=True)
                Path(row["runtime_dir"]).mkdir(parents=True)
                log = (phase / "logs" / (run_id + ".log")).open("xb")
                try:
                    process = popen([row["driver"]], cwd=manifest["source_repo"],
                                    env=run_environment(manifest, row, os.environ), stdin=subprocess.DEVNULL,
                                    stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
                except OSError as error:
                    log.close()
                    result.update(launch_status="failed_to_start", error_type=type(error).__name__, finished_at=now())
                    infrastructure_failed = True
                    continue
                result.update(launch_status="running", started_at=now(), pid=process.pid)
                active.append((process, result, log))
                if interrupted:
                    stop(interrupted[-1], None)
            while active:
                elapsed = time.monotonic()
                if not wall_timeout and elapsed - supervisor_started >= manifest["supervisor_wall_timeout_seconds"]:
                    wall_timeout = True
                    stop(signal.SIGTERM, None)
                if elapsed - last_config_poll >= 10:
                    snapshot_config("active-workflows")
                    last_config_poll = elapsed
                if interrupted:
                    stopping_at = elapsed if stopping_at is None else stopping_at
                    if elapsed - stopping_at > 20:
                        for process, _result, _log in active:
                            if process.poll() is None:
                                try:
                                    os.killpg(process.pid, signal.SIGKILL)
                                except ProcessLookupError:
                                    pass
                pending = []
                for process, result, log in active:
                    if process.poll() is None:
                        pending.append((process, result, log))
                        continue
                    result.update(launcher_exit_code=process.wait(), launch_status="completed", finished_at=now())
                    result["completion_config_snapshot"] = snapshot_config("completed:" + result["id"])
                    result["outcome_classification"] = classify_outcome(by_id[result["id"]], result["launcher_exit_code"])
                    infrastructure_failed |= result["outcome_classification"] == "infrastructure_or_measurement_integrity_failure"
                    log.close()
                    write_new(phase / "logs" / (result["id"] + ".exit.json"), result)
                active = pending
                if active:
                    time.sleep(0.25)
        schedule_exhausted = True
    except BaseException as error:
        supervisor_error = type(error).__name__
        raise
    finally:
        for signum, handler in old_handlers.items():
            signal.signal(signum, handler)
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
            if result["launch_status"] == "running":
                result.update(launcher_exit_code=process.poll(), launch_status="supervisor_stopped", finished_at=now())
            log.close()
        for result in results:
            if result["launch_status"] == "not_launched":
                result["launch_status"] = "not_launched_after_supervisor_error"
        snapshot_config("phase-finished")
        while pending_config and not (interrupted or unexplained_drift):
            time.sleep(0.25)
            snapshot_config("phase-finished-awaiting-native-attestation")
        config_journal.close()
        verify_frozen_inputs("phase-finished")
        report = {"schema_version": 1, "study": manifest["study"], "phase": manifest["phase"],
                  "phase_kind": manifest["phase_kind"],
                  "finished_at": now(), "manifest_sha256": admitted_manifest_sha256,
                  "runs": results, "signals_received": interrupted, "phase_schedule_exhausted": schedule_exhausted,
                  "all_scheduled_workflows_launched": all(row["started_at"] is not None for row in results),
                  "supervisor_wall_timeout": wall_timeout, "supervisor_error": supervisor_error,
                  "behavioral_config_drift_detected": behavioral_drift,
                  "unexplained_config_drift_detected": unexplained_drift,
                  "pending_config_attestation_at_finish": pending_config,
                  "trust_classifier_sha256": manifest.get("trust_classifier_sha256"),
                  "frozen_input_integrity_errors": integrity_errors,
                  "measurement_and_quality_assessed_separately": True}
        write_new(phase / "PHASE-RESULT.json", report)
        write_new(phase / "config-history.json", {"baseline": manifest["global_config_baseline"],
                                                 "snapshots": config_history})
        trust_classifier.finish(config_history)
        write_new(phase / "score-manifest.json", {
            **{key: manifest[key] for key in ("study", "phase", "phase_kind", "base_commit", "task_list_sha256", "model",
                                              "reasoning_effort", "source_repo", "source_commit", "randomization")},
            "schema_version": 1, "phase_manifest": str(phase / "PHASE-MANIFEST.json"),
            "phase_manifest_sha256": report["manifest_sha256"], "runs": results})
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_subparsers(dest="mode", required=True)
    prep = modes.add_parser("prepare", help="freeze explicit inputs without provider work")
    for name in ("phase-root", "source-repo", "bin-dir", "observer-bin", "subscription-wrapper",
                 "runtime-root", "schedule", "protocol", "scorer-config", "global-config"):
        prep.add_argument("--" + name, type=Path, required=True)
    prep.add_argument("--task-list-sha256", required=True)
    prep.add_argument("--evidence-root", type=Path, default=REPO)
    prep.add_argument("--evidence", type=Path, action="append", default=[])
    prep.add_argument("--identity-file", type=Path, action="append", default=[])
    for mode in ("verify", "run"):
        modes.add_parser(mode).add_argument("--phase-root", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.mode == "prepare":
            prepare(args)
        elif args.mode == "verify":
            verify_manifest(args.phase_root.resolve())
        else:
            result = run_phase(args.phase_root)
            print(json.dumps(result, sort_keys=True))
            return 0 if (all(row["launcher_exit_code"] == 0 for row in result["runs"])
                         and not result["unexplained_config_drift_detected"]
                         and not result["pending_config_attestation_at_finish"]
                         and not result["frozen_input_integrity_errors"]) else 1
        print(json.dumps({"phase_root": str(args.phase_root), "mode": args.mode, "provider_work_started": False}))
        return 0
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"mechanism phase: {type(error).__name__}: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
