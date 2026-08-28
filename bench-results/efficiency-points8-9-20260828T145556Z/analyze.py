#!/usr/bin/env python3

from __future__ import annotations

import hashlib
import json
import statistics
from pathlib import Path
from typing import Any, Iterable, Sequence


STUDY_DIR = Path(__file__).resolve().parent
REPO_ROOT = STUDY_DIR.parents[1]
BASE_COMMIT = "c92a0b7060a36eac6db2d869b85e589a7a9480f9"
MODEL = "gpt-5.5"
EFFORT = "xhigh"
TASK_LIST_SHA256 = "45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a"
MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE = 400_000
EFFECTIVE_CONTEXT_WINDOW = 258_400
DOCUMENTED_MAXIMUM_OUTPUT = 128_000
USAGE_FIELDS = (
    "input_tokens",
    "cached_input_tokens",
    "uncached_input_tokens",
    "output_tokens",
    "reasoning_output_tokens",
    "raw_input_plus_output",
    "uncached_input_plus_output",
)

POINT7_DIR = REPO_ROOT / "bench-results/efficiency-point7-bounded-accounting-20260828T142614Z"
POINT7_EVIDENCE = POINT7_DIR / "evidence.json"
POINT7_DIRECT_MECHANISM = REPO_ROOT / (
    "bench-results/efficiency-point7-exact-accounting-20260828T113610Z/runs/direct/"
    "point7-exact-direct-three-feature-sequential-bench-artifacts/observation/"
    "mechanism-summary.json"
)
POINT7_NORMAL_MECHANISM = REPO_ROOT / (
    "bench-results/efficiency-residual-cause-20260828T070112Z/runs/wl-normal-003/"
    "residual-wl-normal-003-three-feature-bench-artifacts/observation/"
    "mechanism-summary.json"
)
POINT7_ALL_OFF_MECHANISM = REPO_ROOT / (
    "bench-results/efficiency-residual-cause-20260828T070112Z/runs/wl-all-off-002/"
    "residual-wl-all-off-002-three-feature-bench-artifacts/observation/"
    "mechanism-summary.json"
)

EXPECTED_COMPLETED = {
    "wl-100-001": "wl-100",
    "wl-000-003": "wl-000",
    "wl-010-001": "wl-010",
    "direct-003": "direct",
    "direct-002": "direct",
    "wl-110-001": "wl-110",
    "wl-000-002": "wl-000",
}
FAILED_ATTEMPTS = {"wl-111-003": "systematic review-routing loop"}
WITHHELD_ATTEMPTS = {
    "wl-001-001": "Git-reconstruction control has a systematic review-routing failure",
    "wl-011-001": "Git-reconstruction control has a systematic review-routing failure",
    "wl-111-002": "Git-reconstruction control has a systematic review-routing failure",
    "wl-101-001": "Git-reconstruction control has a systematic review-routing failure",
}
FACTORIAL_ATTEMPTS = {
    "wl-000": "wl-000-003",
    "wl-010": "wl-010-001",
    "wl-100": "wl-100-001",
    "wl-110": "wl-110-001",
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    rows = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise ValueError(f"{path}:{line_number}: invalid JSON: {error}") from error
    return rows


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def raw_interval(
    observed_raw_tokens: int,
    interrupted_responses: int,
    cap: int = MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE,
) -> dict[str, int]:
    if observed_raw_tokens < 0 or interrupted_responses < 0 or cap < 0:
        raise ValueError("token values and interruption counts must be nonnegative")
    return {
        "lower": observed_raw_tokens,
        "upper": observed_raw_tokens + interrupted_responses * cap,
    }


def conservative_missing_usage_cap(interrupted: int, prompt_bytes: int) -> int:
    if interrupted < 0 or prompt_bytes < 0:
        raise ValueError("interruption count and prompt bytes must be nonnegative")
    rounded_cap = interrupted * MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE
    context_output_prompt_cap = (
        interrupted * (EFFECTIVE_CONTEXT_WINDOW + DOCUMENTED_MAXIMUM_OUTPUT)
        + prompt_bytes
    )
    return max(rounded_cap, context_output_prompt_cap)


def mean_interval(intervals: Sequence[dict[str, int | float]]) -> dict[str, float]:
    if not intervals:
        raise ValueError("cannot average an empty interval group")
    return {
        "lower": statistics.fmean(float(item["lower"]) for item in intervals),
        "upper": statistics.fmean(float(item["upper"]) for item in intervals),
    }


def difference_interval(
    left: dict[str, int | float], right: dict[str, int | float]
) -> dict[str, float]:
    return {
        "lower": float(left["lower"]) - float(right["upper"]),
        "upper": float(left["upper"]) - float(right["lower"]),
    }


def minimum_reduction_percent(exact_reference: int | float, candidate_upper: int | float) -> float:
    if exact_reference <= 0:
        raise ValueError("reference token total must be positive")
    saved = max(float(exact_reference) - float(candidate_upper), 0.0)
    return round(saved * 100.0 / float(exact_reference), 6)


def interval_direction(interval: dict[str, float]) -> str:
    if interval["lower"] > 0:
        return "positive throughout the conservative interval"
    if interval["upper"] < 0:
        return "negative throughout the conservative interval"
    return "inconclusive because the conservative interval crosses zero"


def mean(values: Iterable[int | float]) -> float:
    materialized = [float(value) for value in values]
    if not materialized:
        raise ValueError("cannot average an empty collection")
    return statistics.fmean(materialized)


def token_usage(value: dict[str, Any]) -> dict[str, int]:
    return {field: int(value[field]) for field in USAGE_FIELDS}


def interruption_count(value: Any) -> int:
    if not isinstance(value, int) or value < 0:
        raise ValueError("observer interruption count must be a nonnegative integer")
    return value


def mechanism_metrics(summary: dict[str, Any]) -> dict[str, Any]:
    mechanisms = summary["mechanisms"]
    model_strata = summary["model_strata"]
    require(len(model_strata) == 1, "expected one model stratum")
    stratum = model_strata[0]
    require(stratum["model"] == MODEL, "mechanism summary has the wrong model")
    require(stratum["effort"] == EFFORT, "mechanism summary has the wrong reasoning effort")
    require(stratum["descendant_threads"] == 0, "recursive provider thread found")
    return {
        "provider_threads": stratum["thread_count"],
        "provider_invocations": summary["invocation_count"],
        "command_count": mechanisms["command_count"],
        "validation_commands": mechanisms["validation"]["validation_commands"],
        "repeated_commands": mechanisms["repeated_commands"],
        "protocol_bytes": mechanisms["protocol_bytes"],
    }


def delivery_summary(summary: dict[str, Any]) -> dict[str, Any]:
    rows = [
        row
        for row in summary["mechanisms"].get("counterfactuals", [])
        if row.get("hypothesis") in {"H1", "H2"}
    ]
    result = {}
    for hypothesis, label in (("H1", "unchanged_reread"), ("H2", "changed_reread")):
        selected = [row for row in rows if row["hypothesis"] == hypothesis]
        statuses: dict[str, int] = {}
        for row in selected:
            status = str(row.get("status"))
            statuses[status] = statuses.get(status, 0) + 1
        result[label] = {
            "events": len(selected),
            "statuses": statuses,
            "actual_component_bytes": sum(
                int(row["actual_component_bytes"])
                for row in selected
                if isinstance(row.get("actual_component_bytes"), int)
            ),
            "measured_avoided_bytes": sum(
                int(row["avoided_bytes"])
                for row in selected
                if isinstance(row.get("avoided_bytes"), int)
            ),
            "events_with_measured_counterfactual": sum(
                isinstance(row.get("avoided_bytes"), int) for row in selected
            ),
        }
    return result


def audit_app_server_session(client_path: Path, server_path: Path) -> dict[str, int]:
    starts: dict[str, tuple[str, int]] = {}
    interrupts: set[tuple[str, str]] = set()
    for row in read_jsonl(client_path):
        method = row.get("method")
        params = row.get("params", {})
        if method == "turn/start":
            request_id = str(row["id"])
            thread_id = params.get("threadId")
            require(isinstance(thread_id, str), "turn/start has no thread identity")
            prompt = params.get("input", [])
            require(isinstance(prompt, list), "turn/start has invalid prompt input")
            prompt_bytes = len(
                json.dumps(prompt, ensure_ascii=False, separators=(",", ":")).encode()
            )
            require(request_id not in starts, f"duplicate turn/start request {request_id}")
            starts[request_id] = (thread_id, prompt_bytes)
        elif method == "turn/interrupt":
            identity = (params.get("threadId"), params.get("turnId"))
            require(all(isinstance(value, str) for value in identity), "invalid interrupt identity")
            interrupts.add(identity)  # type: ignore[arg-type]

    started: dict[tuple[str, str], int] = {}
    outcomes: dict[tuple[str, str], str] = {}
    context_windows = set()
    rpc_errors = 0
    for row in read_jsonl(server_path):
        rpc_errors += int("error" in row)
        request_id = str(row["id"]) if "id" in row else None
        if request_id in starts:
            turn_id = row.get("result", {}).get("turn", {}).get("id")
            require(isinstance(turn_id, str), "turn/start response has no turn identity")
            thread_id, prompt_bytes = starts[request_id]
            started[(thread_id, turn_id)] = prompt_bytes
        if row.get("method") == "turn/completed":
            params = row.get("params", {})
            turn = params.get("turn", {})
            identity = (
                params.get("threadId", turn.get("threadId")),
                params.get("turnId", turn.get("id")),
            )
            status = turn.get("status", params.get("status"))
            require(all(isinstance(value, str) for value in identity), "invalid completed identity")
            require(status in {"completed", "interrupted"}, "invalid completed outcome")
            outcomes[identity] = status  # type: ignore[index]
        if row.get("method") == "thread/tokenUsage/updated":
            window = row.get("params", {}).get("tokenUsage", {}).get("modelContextWindow")
            if isinstance(window, int):
                context_windows.add(window)

    require(rpc_errors == 0, "app-server transcript contains JSON-RPC errors")
    require(set(started) == set(outcomes), "started and completed app-server turns differ")
    interrupted = {identity for identity, status in outcomes.items() if status == "interrupted"}
    require(interrupts == interrupted, "interrupt requests and outcomes differ")
    require(context_windows == {EFFECTIVE_CONTEXT_WINDOW}, "unexpected model context window")
    return {
        "started_turns": len(started),
        "completed_turns": sum(status == "completed" for status in outcomes.values()),
        "interrupted_turns": len(interrupted),
        "interrupted_prompt_json_bytes": sum(started[item] for item in interrupted),
    }


def audit_app_server(artifact: Path) -> dict[str, int]:
    sessions = sorted((artifact / "observation/app-server").glob("*"))
    require(bool(sessions), f"no app-server capture in {artifact}")
    audits = []
    for session in sessions:
        client = session / "client-to-server.raw"
        server = session / "server-to-client.raw"
        require(client.is_file() and server.is_file(), f"incomplete app-server session {session}")
        audits.append(audit_app_server_session(client, server))
    return {
        key: sum(item[key] for item in audits)
        for key in (
            "started_turns",
            "completed_turns",
            "interrupted_turns",
            "interrupted_prompt_json_bytes",
        )
    }


def read_controls(path: Path) -> dict[str, str]:
    controls = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if "=" not in line:
            continue
        name, value = line.split("=", 1)
        if name.startswith("WORK_LEAF_EXPERIMENT_"):
            controls[name] = value
    return controls


def quality_index() -> dict[str, dict[str, Any]]:
    indexed = {}
    for path in sorted((STUDY_DIR / "quality").glob("batch*.json")):
        result = read_json(path)
        require(result.get("complete") is True, f"incomplete quality result: {path}")
        require(result.get("base_commit") == BASE_COMMIT, f"wrong quality base: {path}")
        require(result.get("model") == MODEL, f"wrong quality model: {path}")
        require(result.get("reasoning_effort") == EFFORT, f"wrong quality effort: {path}")
        require(result.get("task_list_sha256") == TASK_LIST_SHA256, f"wrong quality task: {path}")
        for run in result["runs"]:
            run_id = run["id"]
            require(run_id not in indexed, f"duplicate quality score for {run_id}")
            for detail in run.get("check_details", {}).values():
                log = STUDY_DIR / detail["log"]
                require(log.is_file(), f"missing quality log {log}")
                require(sha256_file(log) == detail["log_sha256"], f"changed quality log {log}")
            indexed[run_id] = run
    return indexed


def artifact_for(attempt_id: str) -> Path:
    artifacts = sorted((STUDY_DIR / "runs" / attempt_id).glob("*-artifacts"))
    require(len(artifacts) == 1, f"expected one artifact for {attempt_id}")
    return artifacts[0]


def expected_controls(condition: str) -> dict[str, str]:
    bits = condition.removeprefix("wl-")
    return {
        "WORK_LEAF_EXPERIMENT_CHANGED_REPEAT_DELIVERY": (
            "full" if bits[0] == "1" else "normal"
        ),
        "WORK_LEAF_EXPERIMENT_UNCHANGED_REPEAT_DELIVERY": (
            "full" if bits[1] == "1" else "normal"
        ),
        "WORK_LEAF_EXPERIMENT_REVIEW_PROVENANCE": (
            "git-reconstruct" if bits[2] == "1" else "inline-exact"
        ),
    }


def cap_audit(interrupted: int, prompt_bytes: int) -> dict[str, int]:
    declared = interrupted * MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE
    required = interrupted * (EFFECTIVE_CONTEXT_WINDOW + DOCUMENTED_MAXIMUM_OUTPUT) + prompt_bytes
    applied = conservative_missing_usage_cap(interrupted, prompt_bytes)
    return {
        "declared_missing_raw_token_cap": declared,
        "context_output_and_prompt_upper_bound": required,
        "applied_missing_raw_token_cap": applied,
        "rounded_cap_headroom": declared - required,
        "extra_cap_required": applied - declared,
    }


def verify_current_attempt(
    attempt_id: str, condition: str, quality: dict[str, dict[str, Any]]
) -> dict[str, Any]:
    artifact = artifact_for(attempt_id)
    admission_path = STUDY_DIR / "runs" / attempt_id / "admission.json"
    report_path = artifact / "report.json"
    analysis_path = artifact / "observation/analysis.json"
    mechanism_path = artifact / "observation/mechanism-summary.json"
    recursive_path = artifact / "recursive-codex-attempts.log"
    admission = read_json(admission_path)
    report = read_json(report_path)
    analysis = read_json(analysis_path)
    mechanism = read_json(mechanism_path)
    scored = quality.get(attempt_id)
    require(scored is not None, f"missing quality score for {attempt_id}")
    require(admission["condition"] == condition, f"wrong condition for {attempt_id}")
    require(admission["model"] == MODEL and admission["reasoning_effort"] == EFFORT, f"wrong admission profile for {attempt_id}")
    require(report["base_commit"] == BASE_COMMIT, f"wrong candidate base for {attempt_id}")
    require(report["agent_model"] == MODEL and report["agent_reasoning_effort"] == EFFORT, f"wrong report profile for {attempt_id}")
    require(report["workflow_result"] == "pass", f"workflow did not pass for {attempt_id}")
    require(report["review_completed"] == "yes" and report["linearize_completed"] == "yes", f"incomplete workflow stages for {attempt_id}")
    for command in ("cargo fmt", "cargo clippy", "cargo test"):
        require(command in report["code_quality"], f"missing final {command} for {attempt_id}")
    require(recursive_path.is_file() and recursive_path.stat().st_size == 0, f"recursive provider attempt in {attempt_id}")
    require(scored["workflow_result"] == "pass", f"quality scorer saw failed workflow for {attempt_id}")
    require(scored["checks"] == {"completion": "pass", "status": "pass", "visual": "pass"} or scored["completed_features"] < 3, f"inconsistent feature score for {attempt_id}")
    usage = token_usage(report["total_workflow_usage"])
    require(
        usage == token_usage(analysis["usage_scopes"]["total_workflow"]),
        f"report and observer usage differ for {attempt_id}",
    )
    require(
        usage == token_usage(scored["measurement"]["usage"]),
        f"scorer and report usage differ for {attempt_id}",
    )
    metrics = mechanism_metrics(mechanism)
    hashes = {
        str(path.relative_to(REPO_ROOT)): sha256_file(path)
        for path in (admission_path, report_path, analysis_path, mechanism_path, recursive_path)
    }

    raw = int(usage["raw_input_plus_output"])
    if condition == "direct":
        require(report["bench_mode"] == "sequential" and report["feature_schedule"] == "sequential", f"wrong direct workflow for {attempt_id}")
        require(report["measurement_status"] == "complete", f"direct usage is incomplete for {attempt_id}")
        require(analysis["capture_complete"] is True and not analysis["errors"], f"direct observer is incomplete for {attempt_id}")
        require(
            interruption_count(analysis["interrupted_provider_turns"]) == 0,
            f"direct run has interrupted turns: {attempt_id}",
        )
        interval = {"lower": raw, "upper": raw}
        turn_audit = None
        controls = {}
        cap = None
        delivery = {"unchanged_reread": {"events": 0}, "changed_reread": {"events": 0}}
    else:
        require(report["bench_mode"] == "work-leaf" and report["feature_schedule"] == "concurrent", f"wrong Work Leaf workflow for {attempt_id}")
        controls = read_controls(artifact / "daemon-env.txt")
        require(controls == expected_controls(condition), f"wrong controls for {attempt_id}")
        turn_audit = audit_app_server(artifact)
        interrupted = interruption_count(analysis["interrupted_provider_turns"])
        require(interrupted == turn_audit["interrupted_turns"], f"interruption count differs for {attempt_id}")
        require(report["measurement_status"] == "incomplete", f"Work Leaf usage unexpectedly marked exact for {attempt_id}")
        require(analysis["capture_complete"] is False, f"Work Leaf observer unexpectedly complete for {attempt_id}")
        require(analysis["errors"] == [f"interrupted provider turn has no complete usage: count={interrupted}"], f"unexpected observer error for {attempt_id}")
        cap = cap_audit(interrupted, turn_audit["interrupted_prompt_json_bytes"])
        interval = {
            "lower": raw,
            "upper": raw + cap["applied_missing_raw_token_cap"],
        }
        delivery = delivery_summary(mechanism)

    return {
        "run_id": attempt_id,
        "condition": condition,
        "workflow": admission["workflow"],
        "workflow_result": report["workflow_result"],
        "features": scored["checks"],
        "completed_features": scored["completed_features"],
        "raw_tokens": interval,
        "observed_uncached_tokens": int(usage["uncached_input_plus_output"]),
        "interrupted_responses": 0 if turn_audit is None else turn_audit["interrupted_turns"],
        "turn_audit": turn_audit,
        "cap_audit": cap,
        "controls": controls,
        "delivery_events": delivery,
        "workflow_metrics": metrics,
        "source_sha256": hashes,
    }


def point7_observations() -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, str]]:
    evidence = read_json(POINT7_EVIDENCE)
    require(evidence["status"] == "complete", "Point 7 evidence is incomplete")
    require(evidence["frozen_setup"]["base_commit"] == BASE_COMMIT, "Point 7 base differs")
    require(evidence["frozen_setup"]["model"] == MODEL, "Point 7 model differs")
    require(evidence["frozen_setup"]["reasoning_effort"] == EFFORT, "Point 7 effort differs")

    def quality() -> dict[str, str]:
        return {"completion": "pass", "status": "pass", "visual": "pass"}

    direct_source = evidence["direct_sequential"]
    direct_raw = int(direct_source["raw_tokens_exact"])
    direct_mechanism = read_json(POINT7_DIRECT_MECHANISM)
    direct = {
        "run_id": direct_source["run_id"],
        "condition": "direct",
        "workflow": "direct-sequential",
        "workflow_result": "pass",
        "features": quality(),
        "completed_features": 3,
        "raw_tokens": {"lower": direct_raw, "upper": direct_raw},
        "observed_uncached_tokens": direct_source["uncached_tokens_exact"],
        "interrupted_responses": 0,
        "workflow_metrics": mechanism_metrics(direct_mechanism),
        "source": "Point 7 accepted bounded-accounting evidence",
    }

    def bounded(name: str, condition: str, mechanism_path: Path) -> dict[str, Any]:
        source = evidence[name]
        bound = source["raw_bound"]
        require(bound["maximum_tokens_per_interrupted_turn"] == MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE, "Point 7 cap differs")
        require(bound["raw_token_upper_bound"] == bound["observed_raw_tokens"] + bound["interrupted_turns"] * MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE, "Point 7 bound arithmetic differs")
        return {
            "run_id": source["run_id"],
            "condition": condition,
            "workflow": "work-leaf-concurrent",
            "workflow_result": "pass",
            "features": quality(),
            "completed_features": 3,
            "raw_tokens": {
                "lower": bound["observed_raw_tokens"],
                "upper": bound["raw_token_upper_bound"],
            },
            "observed_uncached_tokens": None,
            "interrupted_responses": bound["interrupted_turns"],
            "workflow_metrics": mechanism_metrics(read_json(mechanism_path)),
            "source": "Point 7 accepted bounded-accounting evidence",
        }

    normal = bounded("normal_work_leaf", "wl-000", POINT7_NORMAL_MECHANISM)
    all_off = bounded("all_three_disabled_work_leaf", "wl-111", POINT7_ALL_OFF_MECHANISM)
    hashes = {
        str(path.relative_to(REPO_ROOT)): sha256_file(path)
        for path in (
            POINT7_EVIDENCE,
            POINT7_DIRECT_MECHANISM,
            POINT7_NORMAL_MECHANISM,
            POINT7_ALL_OFF_MECHANISM,
        )
    }
    return direct, normal, all_off, hashes


def summarize_group(observations: Sequence[dict[str, Any]]) -> dict[str, Any]:
    raw = mean_interval([item["raw_tokens"] for item in observations])
    metrics = {
        name: mean(item["workflow_metrics"][name] for item in observations)
        for name in (
            "provider_threads",
            "provider_invocations",
            "command_count",
            "validation_commands",
            "repeated_commands",
            "protocol_bytes",
        )
    }
    return {
        "observations": len(observations),
        "run_ids": [item["run_id"] for item in observations],
        "raw_token_mean_interval": raw,
        "raw_token_observation_intervals": {
            item["run_id"]: item["raw_tokens"] for item in observations
        },
        "feature_pass_counts": {
            feature: sum(item["features"][feature] == "pass" for item in observations)
            for feature in ("visual", "status", "completion")
        },
        "mean_completed_features": mean(item["completed_features"] for item in observations),
        "mean_workflow_metrics": metrics,
    }


def factorial_screen(by_id: dict[str, dict[str, Any]]) -> dict[str, Any]:
    cells = {condition: by_id[attempt] for condition, attempt in FACTORIAL_ATTEMPTS.items()}

    def group_interval(conditions: Sequence[str]) -> dict[str, float]:
        return mean_interval([cells[condition]["raw_tokens"] for condition in conditions])

    changed_compact = group_interval(("wl-000", "wl-010"))
    changed_full = group_interval(("wl-100", "wl-110"))
    changed_difference = difference_interval(changed_full, changed_compact)
    unchanged_compact = group_interval(("wl-000", "wl-100"))
    unchanged_full = group_interval(("wl-010", "wl-110"))
    unchanged_difference = difference_interval(unchanged_full, unchanged_compact)
    all_feature_counts = {item["completed_features"] for item in cells.values()}
    return {
        "cells": {
            condition: {
                "run_id": item["run_id"],
                "raw_tokens": item["raw_tokens"],
                "completed_features": item["completed_features"],
                "delivery_events": item["delivery_events"],
            }
            for condition, item in cells.items()
        },
        "quality_equal_across_cells": len(all_feature_counts) == 1,
        "changed_reread_full_minus_diff": {
            "raw_token_interval": changed_difference,
            "direction": interval_direction(changed_difference),
            "interpretation": "One balanced four-cell screen; not a population estimate.",
        },
        "unchanged_reread_full_minus_digest": {
            "raw_token_interval": unchanged_difference,
            "direction": interval_direction(unchanged_difference),
            "interpretation": "One balanced four-cell screen; not a population estimate.",
        },
        "review_context": {
            "result": "not estimable",
            "reason": "Git reconstruction caused a repeated review-routing loop, so the remaining review-control rows were withheld under the stop rule.",
        },
    }


def analyze() -> dict[str, Any]:
    quality = quality_index()
    current = {
        attempt: verify_current_attempt(attempt, condition, quality)
        for attempt, condition in EXPECTED_COMPLETED.items()
    }
    require(set(quality) == set(EXPECTED_COMPLETED), "quality results do not match completed attempts")
    for attempt in FAILED_ATTEMPTS:
        require((STUDY_DIR / "runs" / attempt / "admission.json").is_file(), f"missing failed admission {attempt}")
        require(not list((STUDY_DIR / "runs" / attempt).glob("*-artifacts")), f"failed attempt unexpectedly has a completed artifact: {attempt}")
    for attempt in WITHHELD_ATTEMPTS:
        require(not (STUDY_DIR / "runs" / attempt).exists(), f"withheld attempt was launched: {attempt}")

    point7_direct, point7_normal, point7_all_off, point7_hashes = point7_observations()
    direct_observations = [
        point7_direct,
        current["direct-003"],
        current["direct-002"],
    ]
    normal_observations = [
        point7_normal,
        current["wl-000-003"],
        current["wl-000-002"],
    ]
    direct_group = summarize_group(direct_observations)
    normal_group = summarize_group(normal_observations)
    savings = difference_interval(
        direct_group["raw_token_mean_interval"],
        normal_group["raw_token_mean_interval"],
    )
    direct_mean = direct_group["raw_token_mean_interval"]["lower"]
    minimum_reduction = minimum_reduction_percent(
        direct_mean, normal_group["raw_token_mean_interval"]["upper"]
    )
    every_normal_upper_below_every_direct = max(
        item["raw_tokens"]["upper"] for item in normal_observations
    ) < min(item["raw_tokens"]["lower"] for item in direct_observations)

    metric_differences = {}
    for metric in ("command_count", "validation_commands", "repeated_commands"):
        direct_value = direct_group["mean_workflow_metrics"][metric]
        work_leaf_value = normal_group["mean_workflow_metrics"][metric]
        metric_differences[metric] = {
            "direct_mean": direct_value,
            "normal_work_leaf_mean": work_leaf_value,
            "work_leaf_fewer_percent": round(
                max(direct_value - work_leaf_value, 0.0) * 100.0 / direct_value, 6
            ),
        }

    source_hashes = dict(point7_hashes)
    for item in current.values():
        source_hashes.update(item["source_sha256"])
    return {
        "schema_version": 1,
        "study": STUDY_DIR.name,
        "status": "complete",
        "frozen_setup": {
            "base_commit": BASE_COMMIT,
            "task_list_sha256": TASK_LIST_SHA256,
            "model": MODEL,
            "reasoning_effort": EFFORT,
            "maximum_tokens_per_interrupted_response": MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE,
            "conditions_are_independent_groups": True,
        },
        "collection": {
            "completed_attempts": sorted(EXPECTED_COMPLETED),
            "failed_attempts": FAILED_ATTEMPTS,
            "withheld_attempts": WITHHELD_ATTEMPTS,
            "failed_attempts_remain_reliability_evidence": True,
        },
        "observations": current,
        "endpoint_groups": {
            "direct_sequential": direct_group,
            "normal_concurrent_work_leaf": normal_group,
            "all_three_disabled_work_leaf": {
                **summarize_group([point7_all_off]),
                "replication_complete": False,
                "reliability_failure": FAILED_ATTEMPTS,
            },
        },
        "overall_raw_saving": {
            "direct_mean_minus_work_leaf_mean_interval": savings,
            "minimum_mean_reduction_percent": minimum_reduction,
            "every_normal_work_leaf_upper_bound_below_every_direct_total": every_normal_upper_below_every_direct,
            "direction": interval_direction(savings),
            "exact_percentage_known": False,
            "uncached_reduction_known": False,
        },
        "factorial_screen": factorial_screen(current),
        "associated_workflow_cycle_difference": {
            "metrics": metric_differences,
            "causal_limit": "These cycle counts are an observed workflow difference, not an isolated ablation. They can explain why raw usage differs but cannot assign an exact token fraction.",
        },
        "conclusions": {
            "overall_saving_repeated_under_conservative_accounting": savings["lower"] > 0,
            "three_delivery_mechanisms_exact_fraction_known": False,
            "review_context_effect_known": False,
            "all_disabled_residual_replicated": False,
            "most_supported_remaining_explanation": "fewer repeated model/tool and validation cycles, if the endpoint bound remains positive",
        },
        "source_sha256": dict(sorted(source_hashes.items())),
    }


if __name__ == "__main__":
    result = analyze()
    output = STUDY_DIR / "evidence.json"
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(output)
