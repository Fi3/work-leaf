#!/usr/bin/env python3

import hashlib
import itertools
import json
from pathlib import Path


STUDY = Path(__file__).resolve().parent
ROOT = STUDY.parents[1]
BASE_COMMIT = "c92a0b7060a36eac6db2d869b85e589a7a9480f9"
TASK_SHA256 = "45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a"
QUALITY_COMMAND = (
    "passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- "
    "-D warnings; cargo test --all-targets --all-features"
)

POINTS = ROOT / "bench-results/efficiency-points8-9-20260828T145556Z"
POINT7 = ROOT / "bench-results/efficiency-point7-exact-accounting-20260828T113610Z"
RESIDUAL = ROOT / "bench-results/efficiency-residual-cause-20260828T070112Z"


def load(path):
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def mean(values):
    return sum(values) / len(values)


def usage_from_report(report):
    return report["total_workflow_usage"]


def usage_from_analysis(analysis):
    return analysis["usage_scopes"]["total_workflow"]


def feature_count(features):
    return sum(status == "pass" for status in features.values())


def exact_permutation_p(left, right):
    pooled = list(left) + list(right)
    observed = mean(left) - mean(right)
    differences = []
    for indexes in itertools.combinations(range(len(pooled)), len(left)):
        selected = set(indexes)
        candidate_left = [value for index, value in enumerate(pooled) if index in selected]
        candidate_right = [value for index, value in enumerate(pooled) if index not in selected]
        differences.append(mean(candidate_left) - mean(candidate_right))
    one_sided = sum(value >= observed for value in differences) / len(differences)
    two_sided = sum(abs(value) >= abs(observed) for value in differences) / len(differences)
    return {"one_sided": one_sided, "two_sided": two_sided, "partitions": len(differences)}


def direct_rows(points_evidence, point7_result):
    definitions = [
        {
            "id": "point7-exact-direct",
            "report": POINT7
            / "runs/direct/point7-exact-direct-three-feature-sequential-bench-artifacts/report.json",
            "environment": POINT7
            / "runs/direct/point7-exact-direct-three-feature-sequential-bench-artifacts/driver-env.txt",
            "recursive": POINT7
            / "runs/direct/point7-exact-direct-three-feature-sequential-bench-artifacts/recursive-codex-attempts.log",
            "quality": point7_result["runs"][0]["checks"],
        },
        {
            "id": "direct-003",
            "report": POINTS
            / "runs/direct-003/points89-direct-003-three-feature-sequential-bench-artifacts/report.json",
            "environment": POINTS
            / "runs/direct-003/points89-direct-003-three-feature-sequential-bench-artifacts/driver-env.txt",
            "recursive": POINTS
            / "runs/direct-003/points89-direct-003-three-feature-sequential-bench-artifacts/recursive-codex-attempts.log",
            "quality": points_evidence["observations"]["direct-003"]["features"],
        },
        {
            "id": "direct-002",
            "report": POINTS
            / "runs/direct-002/points89-direct-002-three-feature-sequential-bench-artifacts/report.json",
            "environment": POINTS
            / "runs/direct-002/points89-direct-002-three-feature-sequential-bench-artifacts/driver-env.txt",
            "recursive": POINTS
            / "runs/direct-002/points89-direct-002-three-feature-sequential-bench-artifacts/recursive-codex-attempts.log",
            "quality": points_evidence["observations"]["direct-002"]["features"],
        },
    ]
    rows = []
    for definition in definitions:
        report = load(definition["report"])
        rows.append(make_row(definition, report, usage_from_report(report), None))
    return rows


def work_leaf_rows(points_evidence, residual_result):
    definitions = [
        {
            "id": "wl-000-002",
            "report": POINTS
            / "runs/wl-000-002/points89-wl-000-002-three-feature-bench-artifacts/report.json",
            "environment": POINTS
            / "runs/wl-000-002/points89-wl-000-002-three-feature-bench-artifacts/daemon-env.txt",
            "recursive": POINTS
            / "runs/wl-000-002/points89-wl-000-002-three-feature-bench-artifacts/recursive-codex-attempts.log",
            "analysis": STUDY / "derived/observations/wl-000-002/analysis-cumulative.json",
            "quality": points_evidence["observations"]["wl-000-002"]["features"],
        },
        {
            "id": "wl-000-003",
            "report": POINTS
            / "runs/wl-000-003/points89-wl-000-003-three-feature-bench-artifacts/report.json",
            "environment": POINTS
            / "runs/wl-000-003/points89-wl-000-003-three-feature-bench-artifacts/daemon-env.txt",
            "recursive": POINTS
            / "runs/wl-000-003/points89-wl-000-003-three-feature-bench-artifacts/recursive-codex-attempts.log",
            "analysis": STUDY / "derived/observations/wl-000-003/analysis-cumulative.json",
            "quality": points_evidence["observations"]["wl-000-003"]["features"],
        },
        {
            "id": "wl-normal-003",
            "report": RESIDUAL
            / "runs/wl-normal-003/residual-wl-normal-003-three-feature-bench-artifacts/report.json",
            "environment": RESIDUAL
            / "runs/wl-normal-003/residual-wl-normal-003-three-feature-bench-artifacts/daemon-env.txt",
            "recursive": RESIDUAL
            / "runs/wl-normal-003/residual-wl-normal-003-three-feature-bench-artifacts/recursive-codex-attempts.log",
            "analysis": STUDY / "derived/observations/wl-normal-003/analysis-cumulative.json",
            "quality": residual_result["runs"][0]["checks"],
        },
    ]
    rows = []
    for definition in definitions:
        report = load(definition["report"])
        analysis = load(definition["analysis"])
        rows.append(make_row(definition, report, usage_from_analysis(analysis), analysis))
    return rows


def make_row(definition, report, usage, analysis):
    source_paths = [definition["report"], definition["environment"], definition["recursive"]]
    if definition.get("analysis"):
        source_paths.append(definition["analysis"])
    return {
        "id": definition["id"],
        "workflow": "work-leaf-concurrent" if report["bench_mode"] == "work-leaf" else "direct-sequential",
        "report_result": report["result"],
        "workflow_result": report["workflow_result"],
        "base_commit": report["base_commit"],
        "model": report["agent_model"],
        "reasoning_effort": report["agent_reasoning_effort"],
        "bench_mode": report["bench_mode"],
        "feature_schedule": report["feature_schedule"],
        "read_permission_mode": report["read_permission_mode"],
        "no_read_permission": report["no_read_permission"],
        "agent_transport": report["agent_transport"],
        "code_quality": report["code_quality"],
        "review_completed": report["review_completed"],
        "linearize_completed": report["linearize_completed"],
        "recursive_provider_log_empty": definition["recursive"].stat().st_size == 0,
        "features": definition["quality"],
        "completed_features": feature_count(definition["quality"]),
        "usage": {
            key: usage[key]
            for key in (
                "input_tokens",
                "cached_input_tokens",
                "uncached_input_tokens",
                "output_tokens",
                "reasoning_output_tokens",
                "raw_input_plus_output",
                "uncached_input_plus_output",
            )
        },
        "provider_capture_complete": analysis is None or analysis["capture_complete"],
        "unresolved_provider_turns": 0 if analysis is None else analysis["interrupted_provider_turns"],
        "provider_threads": None if analysis is None else len(analysis["threads"]),
        "controller_provider_differences": (
            0
            if analysis is None
            else sum(
                row["controller_streamed_usage"] != row["provider_largest_cumulative_usage"]
                for row in analysis["controller_usage_reconciliation"]
            )
        ),
        "source_sha256": {
            str(path.relative_to(ROOT)): sha256(path) for path in source_paths
        },
    }


def group_summary(rows):
    raw = [row["usage"]["raw_input_plus_output"] for row in rows]
    uncached = [row["usage"]["uncached_input_plus_output"] for row in rows]
    features = {
        feature: sum(row["features"][feature] == "pass" for row in rows)
        for feature in ("visual", "status", "completion")
    }
    return {
        "run_count": len(rows),
        "completed_features": sum(row["completed_features"] for row in rows),
        "feature_passes": features,
        "mean_completed_features": mean([row["completed_features"] for row in rows]),
        "mean_raw_tokens": mean(raw),
        "mean_uncached_tokens": mean(uncached),
        "minimum_raw_tokens": min(raw),
        "maximum_raw_tokens": max(raw),
        "minimum_uncached_tokens": min(uncached),
        "maximum_uncached_tokens": max(uncached),
        "runs": rows,
    }


def fairness_failures(direct, work_leaf, task_hashes):
    failures = []
    for row in direct + work_leaf:
        expected = {
            "report_result": "pass",
            "workflow_result": "pass",
            "base_commit": BASE_COMMIT,
            "model": "gpt-5.5",
            "reasoning_effort": "xhigh",
            "code_quality": QUALITY_COMMAND,
            "review_completed": "yes",
            "linearize_completed": "yes",
            "recursive_provider_log_empty": True,
            "provider_capture_complete": True,
            "unresolved_provider_turns": 0,
        }
        for key, value in expected.items():
            if row[key] != value:
                failures.append(f"{row['id']}: {key}={row[key]!r}, expected {value!r}")
    for row in direct:
        expected = {
            "workflow": "direct-sequential",
            "bench_mode": "sequential",
            "feature_schedule": "sequential",
            "no_read_permission": "n/a-direct-agent",
            "agent_transport": "direct-codex-cli",
        }
        for key, value in expected.items():
            if row[key] != value:
                failures.append(f"{row['id']}: {key}={row[key]!r}, expected {value!r}")
    for row in work_leaf:
        expected = {
            "workflow": "work-leaf-concurrent",
            "bench_mode": "work-leaf",
            "feature_schedule": "concurrent",
            "no_read_permission": "0",
            "agent_transport": "app-server",
        }
        for key, value in expected.items():
            if row[key] != value:
                failures.append(f"{row['id']}: {key}={row[key]!r}, expected {value!r}")
    for label, value in task_hashes.items():
        if value != TASK_SHA256:
            failures.append(f"{label}: task SHA-256 {value}, expected {TASK_SHA256}")
    return failures


def build_evidence():
    points_evidence = load(POINTS / "evidence.json")
    point7_result = load(POINT7 / "direct-result.json")
    residual_result = load(RESIDUAL / "quality/wl-normal-003/result.json")
    direct = direct_rows(points_evidence, point7_result)
    work_leaf = work_leaf_rows(points_evidence, residual_result)
    direct_summary = group_summary(direct)
    work_leaf_summary = group_summary(work_leaf)
    raw_gap = direct_summary["mean_raw_tokens"] - work_leaf_summary["mean_raw_tokens"]
    uncached_gap = (
        direct_summary["mean_uncached_tokens"] - work_leaf_summary["mean_uncached_tokens"]
    )
    task_hashes = {
        "points8-9": points_evidence["frozen_setup"]["task_list_sha256"],
        "point7": point7_result["task_list_sha256"],
        "residual": residual_result["task_list_sha256"],
    }
    failures = fairness_failures(direct, work_leaf, task_hashes)
    inexact = [
        row["id"]
        for row in direct + work_leaf
        if not row["provider_capture_complete"] or row["unresolved_provider_turns"]
    ]
    direct_raw = [row["usage"]["raw_input_plus_output"] for row in direct]
    work_leaf_raw = [row["usage"]["raw_input_plus_output"] for row in work_leaf]
    direct_uncached = [row["usage"]["uncached_input_plus_output"] for row in direct]
    work_leaf_uncached = [row["usage"]["uncached_input_plus_output"] for row in work_leaf]
    return {
        "schema_version": 1,
        "study": STUDY.name,
        "status": "complete" if not failures and not inexact else "invalid",
        "scope": "frozen three-feature Rust benchmark",
        "fairness": {
            "base_commit": BASE_COMMIT,
            "task_sha256": TASK_SHA256,
            "task_hashes": task_hashes,
            "model": "gpt-5.5",
            "reasoning_effort": "xhigh",
            "validation": QUALITY_COMMAND,
            "groups_are_independent": True,
            "failed_checks": failures,
        },
        "accounting": {
            "authority": "final cumulative provider totals from hash-verified rollout records",
            "controller_counter_role": "audit only; repeated stale last-turn values can overcount it",
            "inexact_runs": inexact,
        },
        "groups": {"direct": direct_summary, "work_leaf": work_leaf_summary},
        "comparison": {
            "raw_difference_tokens": raw_gap,
            "uncached_difference_tokens": uncached_gap,
            "raw_reduction_percent": raw_gap / direct_summary["mean_raw_tokens"] * 100,
            "uncached_reduction_percent": (
                uncached_gap / direct_summary["mean_uncached_tokens"] * 100
            ),
            "complete_raw_separation": max(work_leaf_raw) < min(direct_raw),
            "complete_uncached_separation": max(work_leaf_uncached) < min(direct_uncached),
            "raw_exact_permutation": exact_permutation_p(direct_raw, work_leaf_raw),
            "uncached_exact_permutation": exact_permutation_p(direct_uncached, work_leaf_uncached),
        },
        "limitations": [
            "Only three observations per group are admitted in this quality-balanced endpoint check.",
            "The feature totals match, but the failed feature differs between groups.",
            "The Work Leaf runner commits differ across the historical runs; all use normal concurrent behavior, but this cohort is a historical sanity check rather than a frozen current-version trial.",
            "The result is specific to one repository and one three-feature task.",
        ],
    }


def main():
    evidence = build_evidence()
    output = STUDY / "endpoint-evidence.json"
    output.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(output)


if __name__ == "__main__":
    main()
