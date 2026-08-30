#!/usr/bin/env python3
"""Build the six-run normal Work Leaf comparison from frozen evidence."""

from __future__ import annotations

import hashlib
import json
import statistics
from pathlib import Path
from typing import Any


STUDY_DIR = Path(__file__).resolve().parent
REPO_ROOT = STUDY_DIR.parents[1]
DIRECT_EVIDENCE = (
    REPO_ROOT
    / "bench-results"
    / "efficiency-corrected-all-disabled-20260829T091341Z"
    / "final-evidence.json"
)
QUALITY = STUDY_DIR / "quality.json"
BOUND_EVIDENCE = (
    REPO_ROOT
    / "bench-results"
    / "efficiency-point7-bounded-accounting-20260828T142614Z"
    / "evidence.json"
)
CORRECTED_OBSERVER_SOURCE = REPO_ROOT / "bench-observer" / "src" / "lib.rs"
RUN_IDS = [f"exact-normal-{index:03d}" for index in range(1, 7)]
FEATURES = ("visual", "status", "completion")
MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE = 400_000


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def mean_interval(rows: list[dict[str, Any]], field: str) -> dict[str, float]:
    return {
        "lower": statistics.fmean(float(row[field]["lower"]) for row in rows),
        "upper": statistics.fmean(float(row[field]["upper"]) for row in rows),
    }


def summarize(rows: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "observations": len(rows),
        "run_ids": [row["run_id"] for row in rows],
        "workflow_passes": sum(row["workflow_result"] == "pass" for row in rows),
        "total_completed_features": sum(int(row["completed_features"]) for row in rows),
        "mean_completed_features": statistics.fmean(
            float(row["completed_features"]) for row in rows
        ),
        "feature_pass_counts": {
            feature: sum(row["feature_checks"][feature] == "pass" for row in rows)
            for feature in FEATURES
        },
        "exact_token_observations": sum(row["measurement"] == "exact" for row in rows),
        "bounded_token_observations": sum(row["measurement"] == "bounded" for row in rows),
        "missing_provider_responses": sum(int(row["missing_responses"]) for row in rows),
        "raw_token_mean_interval": mean_interval(rows, "raw_tokens"),
        "uncached_token_mean_interval": mean_interval(rows, "uncached_tokens"),
    }


def difference_interval(
    direct: dict[str, float], work_leaf: dict[str, float]
) -> dict[str, float]:
    return {
        "lower": direct["lower"] - work_leaf["upper"],
        "upper": direct["upper"] - work_leaf["lower"],
    }


def reduction_interval(
    difference: dict[str, float], direct: dict[str, float]
) -> dict[str, float]:
    if direct["lower"] <= 0:
        raise ValueError("direct token mean must be positive")
    return {
        "lower": 100.0 * difference["lower"] / direct["upper"],
        "upper": 100.0 * difference["upper"] / direct["lower"],
    }


def load_direct_rows() -> list[dict[str, Any]]:
    evidence = read_json(DIRECT_EVIDENCE)
    rows = [row for row in evidence["observations"] if row["group"] == "direct"]
    if len(rows) != 6:
        raise ValueError(f"expected six direct observations, found {len(rows)}")
    for row in rows:
        if row["measurement"] != "exact" or row["interrupted_turns"] != 0:
            raise ValueError(f"direct observation is not exact: {row['run_id']}")
        row["missing_responses"] = 0
    return rows


def load_work_leaf_rows() -> list[dict[str, Any]]:
    quality = read_json(QUALITY)
    scored = {row["id"]: row for row in quality["runs"]}
    rows: list[dict[str, Any]] = []

    if set(scored) != set(RUN_IDS):
        raise ValueError("quality evidence does not contain the declared six runs")

    for run_id in RUN_IDS:
        artifact = STUDY_DIR / "runs" / run_id / f"{run_id}-three-feature-bench-artifacts"
        report_path = artifact / "report.json"
        analysis_path = artifact / "observation" / "analysis-request-accounting.json"
        report = read_json(report_path)
        analysis = read_json(analysis_path)

        expected_report = {
            "result": "pass",
            "feature_schedule": "concurrent",
            "agent_model": "gpt-5.5",
            "agent_reasoning_effort": "xhigh",
            "base_commit": "c92a0b7060a36eac6db2d869b85e589a7a9480f9",
        }
        for key, value in expected_report.items():
            if report.get(key) != value:
                raise ValueError(f"{run_id} report has unexpected {key}: {report.get(key)!r}")

        app_servers = [path for path in (artifact / "observation" / "app-server").iterdir() if path.is_dir()]
        if len(app_servers) != 1:
            raise ValueError(f"{run_id} has {len(app_servers)} app-server captures")
        incoming = app_servers[0] / "client-to-server.raw"
        forwarded = app_servers[0] / "client-to-server.forwarded.raw"
        if incoming.read_bytes() != forwarded.read_bytes():
            raise ValueError(f"{run_id} observer changed client request bytes")

        usage = analysis["usage_scopes"]["total_workflow"]
        missing = int(analysis["interrupted_provider_turns"])
        expected_complete = missing == 0
        if bool(analysis["capture_complete"]) != expected_complete:
            raise ValueError(f"{run_id} has inconsistent capture completeness")
        if expected_complete and analysis["errors"]:
            raise ValueError(f"{run_id} exact analysis reports errors")
        if not expected_complete and analysis["errors"] != [
            f"interrupted provider turn has no complete usage: count={missing}"
        ]:
            raise ValueError(f"{run_id} bounded analysis reports unexpected errors")
        raw_lower = int(usage["raw_input_plus_output"])
        uncached_lower = int(usage["uncached_input_plus_output"])
        missing_raw_cap = (
            missing * MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE
        )
        score = scored[run_id]
        rows.append(
            {
                "group": "normal_work_leaf",
                "run_id": run_id,
                "workflow_result": score["workflow_result"],
                "completed_features": int(score["completed_features"]),
                "feature_checks": score["checks"],
                "measurement": "exact" if expected_complete else "bounded",
                "missing_responses": missing,
                "maximum_raw_tokens_per_missing_response": (
                    MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE
                ),
                "raw_tokens": {
                    "lower": raw_lower,
                    "upper": raw_lower + missing_raw_cap,
                },
                "uncached_tokens": {
                    "lower": uncached_lower,
                    "upper": uncached_lower + missing_raw_cap,
                },
                "source": str(report_path.relative_to(REPO_ROOT)),
                "analysis": str(analysis_path.relative_to(REPO_ROOT)),
            }
        )
    return rows


def compare_groups(
    direct_summary: dict[str, Any], work_leaf_summary: dict[str, Any]
) -> dict[str, Any]:
    raw_difference = difference_interval(
        direct_summary["raw_token_mean_interval"],
        work_leaf_summary["raw_token_mean_interval"],
    )
    uncached_difference = difference_interval(
        direct_summary["uncached_token_mean_interval"],
        work_leaf_summary["uncached_token_mean_interval"],
    )
    return {
        "raw_tokens": raw_difference,
        "raw_work_leaf_reduction_percent": reduction_interval(
            raw_difference, direct_summary["raw_token_mean_interval"]
        ),
        "uncached_tokens": uncached_difference,
        "uncached_work_leaf_reduction_percent": reduction_interval(
            uncached_difference, direct_summary["uncached_token_mean_interval"]
        ),
        "completed_feature_mean_difference": (
            direct_summary["mean_completed_features"]
            - work_leaf_summary["mean_completed_features"]
        ),
    }


def build_evidence() -> dict[str, Any]:
    direct_rows = load_direct_rows()
    work_leaf_rows = load_work_leaf_rows()
    direct = summarize(direct_rows)
    work_leaf = summarize(work_leaf_rows)
    comparison = compare_groups(direct, work_leaf)

    direct_full_rows = [row for row in direct_rows if row["completed_features"] == 3]
    work_leaf_full_rows = [row for row in work_leaf_rows if row["completed_features"] == 3]
    direct_full = summarize(direct_full_rows)
    work_leaf_full = summarize(work_leaf_full_rows)
    full_comparison = compare_groups(direct_full, work_leaf_full)

    return {
        "schema_version": 1,
        "study": STUDY_DIR.name,
        "status": "complete_with_bounded_work_leaf_usage",
        "question": (
            "Does concurrent Work Leaf under the recorded interrupt grace use fewer tokens than "
            "normal direct sequential Codex on the frozen three-feature task after unresolved "
            "interrupted responses receive a conservative token cap?"
        ),
        "accounting": {
            "method": (
                "One capture is exact. Ten interrupted responses across the other five captures "
                "have no terminal usage. Recorded totals are lower bounds; each unresolved "
                "response adds at most 400,000 raw tokens to the upper bound. The same amount is "
                "added to the uncached upper bound because the missing cached-input split is "
                "unknown."
            ),
            "maximum_raw_tokens_per_unresolved_response": (
                MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE
            ),
            "bound_source": str(BOUND_EVIDENCE.relative_to(REPO_ROOT)),
            "corrected_observer_source": str(
                CORRECTED_OBSERVER_SOURCE.relative_to(REPO_ROOT)
            ),
            "corrected_observer_source_sha256": sha256_file(
                CORRECTED_OBSERVER_SOURCE
            ),
            "timing_caveat": (
                "The observer held each complete-directive interrupt for at most 1000 milliseconds. "
                "This can add post-directive generation before interruption; all such usage is "
                "counted, but later thread behavior may differ from an uninstrumented run."
            ),
        },
        "groups": {"direct": direct, "normal_work_leaf": work_leaf},
        "observations": direct_rows + work_leaf_rows,
        "comparisons": {"direct_minus_normal_work_leaf": comparison},
        "quality_matched_full_feature_subset": {
            "warning": (
                "This post-hoc subset contains only runs that completed all three features. It is "
                "descriptive and cannot replace the primary all-run quality comparison."
            ),
            "direct": direct_full,
            "normal_work_leaf": work_leaf_full,
            "direct_minus_normal_work_leaf": full_comparison,
        },
        "conclusions": {
            "raw_saving_proven_under_conservative_bound": (
                comparison["raw_tokens"]["lower"] > 0
            ),
            "uncached_saving_proven_under_conservative_bound": (
                comparison["uncached_tokens"]["lower"] > 0
            ),
            "equal_quality_average_claim_supported": (
                direct["total_completed_features"] == work_leaf["total_completed_features"]
                and direct["feature_pass_counts"] == work_leaf["feature_pass_counts"]
            ),
            "full_feature_subset_raw_saving_proven_under_conservative_bound": (
                full_comparison["raw_tokens"]["lower"] > 0
            ),
        },
        "source_sha256": {
            str(DIRECT_EVIDENCE.relative_to(REPO_ROOT)): sha256_file(DIRECT_EVIDENCE),
            str(QUALITY.relative_to(REPO_ROOT)): sha256_file(QUALITY),
            str(BOUND_EVIDENCE.relative_to(REPO_ROOT)): sha256_file(BOUND_EVIDENCE),
            str(CORRECTED_OBSERVER_SOURCE.relative_to(REPO_ROOT)): sha256_file(
                CORRECTED_OBSERVER_SOURCE
            ),
            **{
                row["analysis"]: sha256_file(REPO_ROOT / row["analysis"])
                for row in work_leaf_rows
            },
        },
    }


def main() -> int:
    output = STUDY_DIR / "evidence.json"
    output.write_text(json.dumps(build_evidence(), indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
