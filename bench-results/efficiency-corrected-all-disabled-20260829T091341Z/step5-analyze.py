#!/usr/bin/env python3
"""Build the final independent-group efficiency analysis without provider calls."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import random
import statistics
from pathlib import Path
from typing import Any, Iterable, Sequence


STUDY_DIR = Path(__file__).resolve().parent
REPO_ROOT = STUDY_DIR.parents[1]
PRIOR_EVIDENCE_PATH = (
    REPO_ROOT / "bench-results/efficiency-points8-9-20260828T145556Z/evidence.json"
)
POINT7_DIRECT_RESULT_PATH = (
    REPO_ROOT
    / "bench-results/efficiency-point7-exact-accounting-20260828T113610Z/direct-result.json"
)
POINT7_DIRECT_REPORT_PATH = (
    REPO_ROOT
    / "bench-results/efficiency-point7-exact-accounting-20260828T113610Z/runs/direct/point7-exact-direct-three-feature-sequential-bench-artifacts/report.json"
)
POINT7_BOUND_PATH = (
    REPO_ROOT
    / "bench-results/efficiency-point7-bounded-accounting-20260828T142614Z/evidence.json"
)
WL_NORMAL_QUALITY_PATH = (
    REPO_ROOT
    / "bench-results/efficiency-residual-cause-20260828T070112Z/quality/wl-normal-003/result.json"
)
WL_NORMAL_REPORT_PATH = (
    REPO_ROOT
    / "bench-results/efficiency-residual-cause-20260828T070112Z/runs/wl-normal-003/residual-wl-normal-003-three-feature-bench-artifacts/report.json"
)
BASE_COMMIT = "c92a0b7060a36eac6db2d869b85e589a7a9480f9"
TASK_LIST_SHA256 = "45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a"
MODEL = "gpt-5.5"
EFFORT = "xhigh"
FEATURES = ("visual", "status", "completion")
NORMAL_CONTROLS = {
    "WORK_LEAF_EXPERIMENT_CHANGED_REPEAT_DELIVERY": "normal",
    "WORK_LEAF_EXPERIMENT_REVIEW_PROVENANCE": "inline-exact",
    "WORK_LEAF_EXPERIMENT_UNCHANGED_REPEAT_DELIVERY": "normal",
}
DISABLED_CONTROLS = {
    "WORK_LEAF_EXPERIMENT_CHANGED_REPEAT_DELIVERY": "full",
    "WORK_LEAF_EXPERIMENT_REVIEW_PROVENANCE": "git-reconstruct",
    "WORK_LEAF_EXPERIMENT_UNCHANGED_REPEAT_DELIVERY": "full",
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_initial_analyzer():
    path = STUDY_DIR / "analyze.py"
    specification = importlib.util.spec_from_file_location("initial_control_analyzer", path)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def artifact_for(run_id: str) -> Path:
    artifacts = sorted((STUDY_DIR / "runs" / run_id).glob("*-artifacts"))
    require(len(artifacts) == 1, f"expected one artifact for {run_id}")
    return artifacts[0]


def quality_index() -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for name in (
        "step4-batch2-quality.json",
        "step4-batch3-quality.json",
        "step4-batch4-quality.json",
    ):
        document = read_json(STUDY_DIR / name)
        require(document["base_commit"] == BASE_COMMIT, f"wrong scorer base in {name}")
        require(document["task_list_sha256"] == TASK_LIST_SHA256, f"wrong task in {name}")
        require(document["model"] == MODEL, f"wrong model in {name}")
        require(document["reasoning_effort"] == EFFORT, f"wrong effort in {name}")
        for run in document["runs"]:
            require(run["id"] not in result, f"duplicate score: {run['id']}")
            result[run["id"]] = run
    return result


def observation(
    *,
    run_id: str,
    group: str,
    workflow_result: str,
    checks: dict[str, str],
    raw_lower: int,
    raw_upper: int,
    uncached_lower: int,
    uncached_upper: int,
    interrupted_turns: int,
    source: str,
    outcome_note: str | None = None,
) -> dict[str, Any]:
    require(set(checks) == set(FEATURES), f"wrong feature checks: {run_id}")
    require(raw_lower > 0 and raw_upper >= raw_lower, f"invalid raw interval: {run_id}")
    require(
        uncached_lower > 0 and uncached_upper >= uncached_lower,
        f"invalid uncached interval: {run_id}",
    )
    completed = sum(checks[feature] == "pass" for feature in FEATURES)
    return {
        "run_id": run_id,
        "group": group,
        "workflow_result": workflow_result,
        "outcome_note": outcome_note,
        "feature_checks": checks,
        "completed_features": completed,
        "raw_tokens": {"lower": raw_lower, "upper": raw_upper},
        "uncached_tokens": {"lower": uncached_lower, "upper": uncached_upper},
        "interrupted_turns": interrupted_turns,
        "measurement": "exact" if raw_lower == raw_upper else "conservative interval",
        "source": source,
    }


def prior_endpoint_observations() -> list[dict[str, Any]]:
    prior = read_json(PRIOR_EVIDENCE_PATH)
    require(prior["status"] == "complete", "prior endpoint evidence is incomplete")
    frozen = prior["frozen_setup"]
    require(frozen["base_commit"] == BASE_COMMIT, "prior endpoint base differs")
    require(frozen["model"] == MODEL and frozen["reasoning_effort"] == EFFORT, "prior profile differs")
    require(frozen["task_list_sha256"] == TASK_LIST_SHA256, "prior task differs")
    rows: list[dict[str, Any]] = []

    direct_result = read_json(POINT7_DIRECT_RESULT_PATH)
    direct_report = read_json(POINT7_DIRECT_REPORT_PATH)
    direct_score = direct_result["runs"][0]
    require(direct_score["id"] == "point7-exact-direct", "wrong point-7 direct score")
    require(direct_report["measurement_status"] == "complete", "point-7 direct is not exact")
    direct_usage = direct_report["total_workflow_usage"]
    rows.append(
        observation(
            run_id="point7-exact-direct",
            group="direct",
            workflow_result=direct_report["workflow_result"],
            checks=direct_score["checks"],
            raw_lower=int(direct_usage["raw_input_plus_output"]),
            raw_upper=int(direct_usage["raw_input_plus_output"]),
            uncached_lower=int(direct_usage["uncached_input_plus_output"]),
            uncached_upper=int(direct_usage["uncached_input_plus_output"]),
            interrupted_turns=0,
            source=str(POINT7_DIRECT_REPORT_PATH.relative_to(REPO_ROOT)),
        )
    )

    for run_id in ("direct-003", "direct-002"):
        saved = prior["observations"][run_id]
        rows.append(
            observation(
                run_id=run_id,
                group="direct",
                workflow_result=saved["workflow_result"],
                checks=saved["features"],
                raw_lower=int(saved["raw_tokens"]["lower"]),
                raw_upper=int(saved["raw_tokens"]["upper"]),
                uncached_lower=int(saved["observed_uncached_tokens"]),
                uncached_upper=int(saved["observed_uncached_tokens"]),
                interrupted_turns=0,
                source=str(PRIOR_EVIDENCE_PATH.relative_to(REPO_ROOT)),
            )
        )

    bounded = read_json(POINT7_BOUND_PATH)
    normal_bound = bounded["normal_work_leaf"]
    normal_quality_document = read_json(WL_NORMAL_QUALITY_PATH)
    normal_quality = normal_quality_document["runs"][0]
    require(normal_quality["id"] == "wl-normal-003", "wrong prior normal score")
    normal_report = read_json(WL_NORMAL_REPORT_PATH)
    raw_lower = int(normal_bound["raw_bound"]["observed_raw_tokens"])
    raw_upper = int(normal_bound["raw_bound"]["raw_token_upper_bound"])
    missing_cap = raw_upper - raw_lower
    uncached_lower = int(normal_report["total_workflow_usage"]["uncached_input_plus_output"])
    rows.append(
        observation(
            run_id="wl-normal-003",
            group="normal_work_leaf",
            workflow_result=normal_report["workflow_result"],
            checks=normal_quality["checks"],
            raw_lower=raw_lower,
            raw_upper=raw_upper,
            uncached_lower=uncached_lower,
            uncached_upper=uncached_lower + missing_cap,
            interrupted_turns=int(normal_bound["turn_audit"]["interrupted_turns"]),
            source=str(POINT7_BOUND_PATH.relative_to(REPO_ROOT)),
        )
    )

    for run_id in ("wl-000-003", "wl-000-002"):
        saved = prior["observations"][run_id]
        missing_cap = int(saved["cap_audit"]["applied_missing_raw_token_cap"])
        uncached_lower = int(saved["observed_uncached_tokens"])
        rows.append(
            observation(
                run_id=run_id,
                group="normal_work_leaf",
                workflow_result=saved["workflow_result"],
                checks=saved["features"],
                raw_lower=int(saved["raw_tokens"]["lower"]),
                raw_upper=int(saved["raw_tokens"]["upper"]),
                uncached_lower=uncached_lower,
                uncached_upper=uncached_lower + missing_cap,
                interrupted_turns=int(saved["interrupted_responses"]),
                source=str(PRIOR_EVIDENCE_PATH.relative_to(REPO_ROOT)),
            )
        )
    return rows


def verify_current_report(report: dict[str, Any], run_id: str, group: str) -> None:
    require(report["base_commit"] == BASE_COMMIT, f"wrong base: {run_id}")
    require(report["agent_model"] == MODEL, f"wrong model: {run_id}")
    require(report["agent_reasoning_effort"] == EFFORT, f"wrong effort: {run_id}")
    if group == "direct":
        require(report["feature_schedule"] == "sequential", f"wrong schedule: {run_id}")
    else:
        require(report["bench_mode"] == "work-leaf", f"wrong mode: {run_id}")
        require(report["feature_schedule"] == "concurrent", f"wrong schedule: {run_id}")


def review_git_reconstruction_exercised(state: dict[str, Any], run_id: str) -> bool:
    sessions = [
        session
        for session in state["snapshot"]["sessions"]
        if session["id"].startswith("review-")
    ]
    require(len(sessions) == 3, f"wrong review-session count: {run_id}")
    for session in sessions:
        lines = [str(line) for line in session["lines"]]
        require(
            any(
                line.startswith("@work-leaf locks run") and "git " in line
                for line in lines
            ),
            f"review did not reconstruct from Git: {run_id}/{session['id']}",
        )
    return True


def current_step4_observations() -> tuple[list[dict[str, Any]], dict[str, Any]]:
    initial = load_initial_analyzer()
    scores = quality_index()
    rows: list[dict[str, Any]] = []
    activation = {
        "changed_reread_events": 0,
        "changed_reread_exercised_runs": 0,
        "unchanged_reread_events": 0,
        "unchanged_reread_exercised_runs": 0,
        "git_review_exercised_runs": 0,
    }

    for run_id in ("step4-direct-001", "step4-direct-002", "step4-direct-003"):
        artifact = artifact_for(run_id)
        report = read_json(artifact / "report.json")
        score = scores[run_id]
        verify_current_report(report, run_id, "direct")
        require(report["measurement_status"] == "complete", f"direct usage is not exact: {run_id}")
        usage = report["total_workflow_usage"]
        rows.append(
            observation(
                run_id=run_id,
                group="direct",
                workflow_result=report["workflow_result"],
                checks=score["checks"],
                raw_lower=int(usage["raw_input_plus_output"]),
                raw_upper=int(usage["raw_input_plus_output"]),
                uncached_lower=int(usage["uncached_input_plus_output"]),
                uncached_upper=int(usage["uncached_input_plus_output"]),
                interrupted_turns=0,
                source=str((artifact / "report.json").relative_to(REPO_ROOT)),
                outcome_note=(
                    "outer launcher failed after the child report was published"
                    if run_id == "step4-direct-001"
                    else None
                ),
            )
        )

    for run_id, group in (
        ("step4-normal-002", "normal_work_leaf"),
        ("step4-normal-003", "normal_work_leaf"),
        ("step4-control-001", "all_disabled_work_leaf"),
        ("step4-control-002", "all_disabled_work_leaf"),
        ("step4-control-003", "all_disabled_work_leaf"),
    ):
        artifact = artifact_for(run_id)
        report = read_json(artifact / "report.json")
        score = scores[run_id]
        verify_current_report(report, run_id, group)
        expected_controls = NORMAL_CONTROLS if group == "normal_work_leaf" else DISABLED_CONTROLS
        require(initial.read_controls(artifact / "daemon-env.txt") == expected_controls, f"wrong controls: {run_id}")
        recursive_log = artifact / "recursive-codex-attempts.log"
        require(recursive_log.is_file() and recursive_log.stat().st_size == 0, f"recursive provider call: {run_id}")
        turns = initial.audit_app_server(artifact)
        cap = initial.cap_audit(
            turns["interrupted_turns"], turns["interrupted_prompt_json_bytes"]
        )["applied_missing_raw_token_cap"]
        usage = report["total_workflow_usage"]
        raw_lower = int(usage["raw_input_plus_output"])
        uncached_lower = int(usage["uncached_input_plus_output"])
        note = None
        if run_id == "step4-normal-002":
            note = "feature workflow completed; benchmark failed while removing its temporary AGENTS.md policy"
        elif run_id == "step4-control-003":
            note = "feature workflow completed; one final repository test failed"
        elif run_id == "step4-control-001":
            note = "outer launcher failed after the child report was published"
        rows.append(
            observation(
                run_id=run_id,
                group=group,
                workflow_result=report["workflow_result"],
                checks=score["checks"],
                raw_lower=raw_lower,
                raw_upper=raw_lower + cap,
                uncached_lower=uncached_lower,
                uncached_upper=uncached_lower + cap,
                interrupted_turns=int(turns["interrupted_turns"]),
                source=str((artifact / "report.json").relative_to(REPO_ROOT)),
                outcome_note=note,
            )
        )
        if group == "all_disabled_work_leaf":
            delivery = initial.delivery_summary(
                read_json(artifact / "observation/mechanism-summary.json")
            )
            changed = int(delivery["changed_reread"]["full_current_events"])
            unchanged = int(delivery["unchanged_reread"]["full_current_events"])
            activation["changed_reread_events"] += changed
            activation["changed_reread_exercised_runs"] += changed > 0
            activation["unchanged_reread_events"] += unchanged
            activation["unchanged_reread_exercised_runs"] += unchanged > 0
            activation["git_review_exercised_runs"] += review_git_reconstruction_exercised(
                read_json(artifact / "final-state.json"), run_id
            )
    return rows, activation


def initial_control_observations() -> tuple[list[dict[str, Any]], dict[str, int]]:
    saved = read_json(STUDY_DIR / "evidence.json")
    rows = []
    activation = {
        "changed_reread_events": 0,
        "changed_reread_exercised_runs": 0,
        "unchanged_reread_events": 0,
        "unchanged_reread_exercised_runs": 0,
        "git_review_exercised_runs": 0,
    }
    for run in saved["runs"]:
        cap = int(run["cap_audit"]["applied_missing_raw_token_cap"])
        uncached_lower = int(run["observed_uncached_tokens"])
        rows.append(
            observation(
                run_id=run["run_id"],
                group="all_disabled_work_leaf",
                workflow_result=run["workflow_result"],
                checks=run["feature_checks"],
                raw_lower=int(run["raw_tokens"]["lower"]),
                raw_upper=int(run["raw_tokens"]["upper"]),
                uncached_lower=uncached_lower,
                uncached_upper=uncached_lower + cap,
                interrupted_turns=int(run["turn_audit"]["interrupted_turns"]),
                source=str((STUDY_DIR / "evidence.json").relative_to(REPO_ROOT)),
            )
        )
        changed = int(run["delivery_events"]["changed_reread"]["full_current_events"])
        unchanged = int(run["delivery_events"]["unchanged_reread"]["full_current_events"])
        activation["changed_reread_events"] += changed
        activation["changed_reread_exercised_runs"] += changed > 0
        activation["unchanged_reread_events"] += unchanged
        activation["unchanged_reread_exercised_runs"] += unchanged > 0
        activation["git_review_exercised_runs"] += run["review_control"][
            "all_reconstructed_from_git"
        ]
    return rows, activation


def fmean(values: Iterable[int | float]) -> float:
    return statistics.fmean(float(value) for value in values)


def percentile(values: Sequence[float], percentage: float) -> float:
    ordered = sorted(values)
    require(bool(ordered), "cannot take a percentile of no values")
    position = (len(ordered) - 1) * percentage
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    fraction = position - lower
    return ordered[lower] * (1.0 - fraction) + ordered[upper] * fraction


def bootstrap_mean_interval(values: Sequence[float], samples: int, seed: int) -> dict[str, float]:
    require(samples > 0 and values, "invalid bootstrap request")
    generator = random.Random(seed)
    means = [
        fmean(generator.choice(values) for _ in values)
        for _ in range(samples)
    ]
    return {"lower": percentile(means, 0.025), "upper": percentile(means, 0.975)}


def summarize_group(rows: Sequence[dict[str, Any]], samples: int, seed: int) -> dict[str, Any]:
    raw_lower = [float(row["raw_tokens"]["lower"]) for row in rows]
    raw_upper = [float(row["raw_tokens"]["upper"]) for row in rows]
    uncached_lower = [float(row["uncached_tokens"]["lower"]) for row in rows]
    uncached_upper = [float(row["uncached_tokens"]["upper"]) for row in rows]
    quality = [float(row["completed_features"]) for row in rows]
    return {
        "observations": len(rows),
        "run_ids": [row["run_id"] for row in rows],
        "workflow_passes": sum(row["workflow_result"] == "pass" for row in rows),
        "bounded_token_observations": sum(row["measurement"] != "exact" for row in rows),
        "feature_pass_counts": {
            feature: sum(row["feature_checks"][feature] == "pass" for row in rows)
            for feature in FEATURES
        },
        "total_completed_features": sum(int(value) for value in quality),
        "mean_completed_features": fmean(quality),
        "full_feature_observations": sum(value == 3 for value in quality),
        "raw_token_mean_interval": {"lower": fmean(raw_lower), "upper": fmean(raw_upper)},
        "raw_observed_mean_bootstrap_95_percent": bootstrap_mean_interval(
            raw_lower, samples, seed
        ),
        "raw_observed_median": statistics.median(raw_lower),
        "raw_observed_range": {"minimum": min(raw_lower), "maximum": max(raw_lower)},
        "uncached_token_mean_interval": {
            "lower": fmean(uncached_lower),
            "upper": fmean(uncached_upper),
        },
        "quality_mean_bootstrap_95_percent": bootstrap_mean_interval(
            quality, samples, seed + 1
        ),
    }


def difference_interval(left: dict[str, float], right: dict[str, float]) -> dict[str, float]:
    return {
        "lower": left["lower"] - right["upper"],
        "upper": left["upper"] - right["lower"],
    }


def bootstrap_difference_envelope(
    left: Sequence[dict[str, Any]],
    right: Sequence[dict[str, Any]],
    key: str,
    samples: int,
    seed: int,
) -> dict[str, float]:
    generator = random.Random(seed)
    minimum_differences = []
    maximum_differences = []
    observed_differences = []
    for _ in range(samples):
        left_sample = [generator.choice(left) for _ in left]
        right_sample = [generator.choice(right) for _ in right]
        minimum_differences.append(
            fmean(row[key]["lower"] for row in left_sample)
            - fmean(row[key]["upper"] for row in right_sample)
        )
        maximum_differences.append(
            fmean(row[key]["upper"] for row in left_sample)
            - fmean(row[key]["lower"] for row in right_sample)
        )
        observed_differences.append(
            fmean(row[key]["lower"] for row in left_sample)
            - fmean(row[key]["lower"] for row in right_sample)
        )
    return {
        "conservative_lower": percentile(minimum_differences, 0.025),
        "conservative_upper": percentile(maximum_differences, 0.975),
        "observed_lower_bound_ci_lower": percentile(observed_differences, 0.025),
        "observed_lower_bound_ci_upper": percentile(observed_differences, 0.975),
    }


def bootstrap_scalar_difference(
    left: Sequence[float],
    right: Sequence[float],
    samples: int,
    seed: int,
) -> dict[str, float]:
    generator = random.Random(seed)
    differences = []
    for _ in range(samples):
        left_sample = [generator.choice(left) for _ in left]
        right_sample = [generator.choice(right) for _ in right]
        differences.append(fmean(left_sample) - fmean(right_sample))
    return {
        "lower": percentile(differences, 0.025),
        "upper": percentile(differences, 0.975),
    }


def compare_groups(
    left_rows: Sequence[dict[str, Any]],
    right_rows: Sequence[dict[str, Any]],
    left_summary: dict[str, Any],
    right_summary: dict[str, Any],
    samples: int,
    seed: int,
) -> dict[str, Any]:
    raw_difference = difference_interval(
        left_summary["raw_token_mean_interval"], right_summary["raw_token_mean_interval"]
    )
    uncached_difference = difference_interval(
        left_summary["uncached_token_mean_interval"],
        right_summary["uncached_token_mean_interval"],
    )
    left_quality = [float(row["completed_features"]) for row in left_rows]
    right_quality = [float(row["completed_features"]) for row in right_rows]
    return {
        "raw_mean_difference_interval": raw_difference,
        "raw_observed_mean_difference": (
            left_summary["raw_token_mean_interval"]["lower"]
            - right_summary["raw_token_mean_interval"]["lower"]
        ),
        "uncached_mean_difference_interval": uncached_difference,
        "mean_completed_feature_difference": fmean(left_quality) - fmean(right_quality),
        "raw_descriptive_bootstrap_95_percent": bootstrap_difference_envelope(
            left_rows, right_rows, "raw_tokens", samples, seed
        ),
        "quality_descriptive_bootstrap_95_percent": bootstrap_scalar_difference(
            left_quality, right_quality, samples, seed + 1
        ),
    }


def add_activation(left: dict[str, int], right: dict[str, int]) -> dict[str, int]:
    return {key: int(left[key]) + int(right[key]) for key in left}


def source_hashes() -> dict[str, str]:
    paths = [
        PRIOR_EVIDENCE_PATH,
        POINT7_DIRECT_RESULT_PATH,
        POINT7_DIRECT_REPORT_PATH,
        POINT7_BOUND_PATH,
        WL_NORMAL_QUALITY_PATH,
        WL_NORMAL_REPORT_PATH,
        STUDY_DIR / "evidence.json",
        STUDY_DIR / "step4-batch2-quality.json",
        STUDY_DIR / "step4-batch3-quality.json",
        STUDY_DIR / "step4-batch4-quality.json",
        STUDY_DIR / "scorer/score.py",
        STUDY_DIR / "step5-analyze.py",
        STUDY_DIR / "test_step5_analyze.py",
    ]
    for run_id in (
        "step4-direct-001",
        "step4-direct-002",
        "step4-direct-003",
        "step4-normal-002",
        "step4-normal-003",
        "step4-control-001",
        "step4-control-002",
        "step4-control-003",
    ):
        artifact = artifact_for(run_id)
        paths.extend((artifact / "report.json", artifact / "observation/analysis.json"))
        if run_id.startswith(("step4-normal", "step4-control")):
            paths.extend(
                (
                    artifact / "daemon-env.txt",
                    artifact / "recursive-codex-attempts.log",
                )
            )
            for session in sorted((artifact / "observation/app-server").glob("*")):
                paths.extend(
                    (
                        session / "client-to-server.raw",
                        session / "server-to-client.raw",
                    )
                )
        if run_id.startswith("step4-control"):
            paths.extend(
                (
                    artifact / "final-state.json",
                    artifact / "observation/mechanism-summary.json",
                )
            )
    return {
        str(path.relative_to(REPO_ROOT)): sha256_file(path)
        for path in paths
    }


def build_final_evidence(bootstrap_samples: int = 50_000) -> dict[str, Any]:
    prior_rows = prior_endpoint_observations()
    initial_rows, initial_activation = initial_control_observations()
    step4_rows, step4_activation = current_step4_observations()
    rows = prior_rows + initial_rows + step4_rows
    run_ids = [row["run_id"] for row in rows]
    require(len(run_ids) == len(set(run_ids)), "duplicate observations")
    grouped = {
        group: [row for row in rows if row["group"] == group]
        for group in ("direct", "normal_work_leaf", "all_disabled_work_leaf")
    }
    require({key: len(value) for key, value in grouped.items()} == {
        "direct": 6,
        "normal_work_leaf": 5,
        "all_disabled_work_leaf": 6,
    }, "unexpected final group sizes")
    summaries = {
        name: summarize_group(group, bootstrap_samples, 100 + index * 10)
        for index, (name, group) in enumerate(grouped.items())
    }
    overall = compare_groups(
        grouped["direct"],
        grouped["normal_work_leaf"],
        summaries["direct"],
        summaries["normal_work_leaf"],
        bootstrap_samples,
        700,
    )
    overall["raw_average_saving_proven_under_cap"] = (
        overall["raw_mean_difference_interval"]["lower"] > 0
    )
    direct_mean = summaries["direct"]["raw_token_mean_interval"]["lower"]
    overall["raw_mean_reduction_percent_interval"] = {
        "lower": overall["raw_mean_difference_interval"]["lower"] / direct_mean * 100.0,
        "upper": overall["raw_mean_difference_interval"]["upper"] / direct_mean * 100.0,
    }
    mechanism = compare_groups(
        grouped["all_disabled_work_leaf"],
        grouped["normal_work_leaf"],
        summaries["all_disabled_work_leaf"],
        summaries["normal_work_leaf"],
        bootstrap_samples,
        900,
    )
    mechanism["combined_mechanism_effect_proven"] = not (
        mechanism["raw_mean_difference_interval"]["lower"] <= 0
        <= mechanism["raw_mean_difference_interval"]["upper"]
    )
    direct_full = [row for row in grouped["direct"] if row["completed_features"] == 3]
    normal_full = [
        row for row in grouped["normal_work_leaf"] if row["completed_features"] == 3
    ]
    direct_full_summary = summarize_group(direct_full, bootstrap_samples, 1_100)
    normal_full_summary = summarize_group(normal_full, bootstrap_samples, 1_120)
    full_difference = difference_interval(
        direct_full_summary["raw_token_mean_interval"],
        normal_full_summary["raw_token_mean_interval"],
    )
    prior = read_json(PRIOR_EVIDENCE_PATH)
    activation = add_activation(initial_activation, step4_activation)
    return {
        "schema_version": 1,
        "status": "complete",
        "study": STUDY_DIR.name,
        "question": "Is normal concurrent Work Leaf's raw-token saving real, and do the three tested context-delivery mechanisms explain it?",
        "frozen_setup": {
            "base_commit": BASE_COMMIT,
            "task_list_sha256": TASK_LIST_SHA256,
            "model": MODEL,
            "reasoning_effort": EFFORT,
            "conditions_are_independent_groups": True,
            "production_work_leaf_modified": False,
            "normal_work_leaf_is_concurrent": True,
            "direct_codex_is_sequential": True,
            "maximum_tokens_per_interrupted_response": 400_000,
        },
        "observations": rows,
        "groups": summaries,
        "excluded_infrastructure_attempts": [
            {
                "run_id": "step4-normal-001",
                "reason": "invalid environment values stopped normal Work Leaf before implementation, review, or linearization",
                "evidence_retained": True,
            }
        ],
        "comparisons": {
            "direct_minus_normal_work_leaf": overall,
            "all_disabled_minus_normal_work_leaf": mechanism,
        },
        "quality_matched_full_feature_subset": {
            "direct_observations": len(direct_full),
            "normal_work_leaf_observations": len(normal_full),
            "direct_run_ids": [row["run_id"] for row in direct_full],
            "normal_work_leaf_run_ids": [row["run_id"] for row in normal_full],
            "direct_raw_token_mean_interval": direct_full_summary[
                "raw_token_mean_interval"
            ],
            "normal_work_leaf_raw_token_mean_interval": normal_full_summary[
                "raw_token_mean_interval"
            ],
            "raw_mean_difference_interval": full_difference,
            "raw_average_saving_proven_under_cap": full_difference["lower"] > 0,
            "interpretation": "Supporting subset only; the primary analysis retains every partial and failed workflow.",
        },
        "control_activation": {
            **activation,
            "observations": len(grouped["all_disabled_work_leaf"]),
            "all_three_controls_configured_in_every_observation": True,
        },
        "associated_workflow_cycle_difference": prior[
            "associated_workflow_cycle_difference"
        ],
        "conclusions": {
            "collected_sample_average_raw_saving_survives_conservative_cap": overall[
                "raw_average_saving_proven_under_cap"
            ],
            "population_average_raw_saving_statistically_established": overall[
                "raw_descriptive_bootstrap_95_percent"
            ]["conservative_lower"] > 0,
            "same_result_holds_in_full_feature_subset": full_difference["lower"] > 0,
            "uncached_saving_proven": overall["uncached_mean_difference_interval"]["lower"] > 0,
            "three_tested_mechanisms_explain_the_saving": "not established",
            "exact_mechanism_fraction_known": False,
            "most_supported_remaining_explanation": "normal concurrent Work Leaf used fewer total, repeated, and validation command cycles; this is associated evidence, not an isolated causal allocation",
            "statistical_limit": "The groups are small and interrupted Work Leaf turns require conservative bounds. Bootstrap ranges are descriptive and do not establish cross-project generalization.",
        },
        "source_sha256": source_hashes(),
    }


def main() -> int:
    evidence = build_final_evidence()
    output = STUDY_DIR / "final-evidence.json"
    temporary = output.with_suffix(".json.tmp")
    temporary.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(output)
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
