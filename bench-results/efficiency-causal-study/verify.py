#!/usr/bin/env python3
"""Recompute the compact efficiency study without launching candidates or agents."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path
from types import ModuleType
from typing import Any, Iterable


class AuditError(RuntimeError):
    """Raised when compact evidence is incomplete or inconsistent."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


def require_equal(actual: Any, expected: Any, message: str) -> None:
    if actual != expected:
        raise AuditError(f"{message}: expected {expected!r}, found {actual!r}")


def read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise AuditError(f"cannot read JSON evidence {path}: {error}") from error
    require(isinstance(value, dict), f"JSON evidence must be an object: {path}")
    return value


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    try:
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
    except OSError as error:
        raise AuditError(f"cannot hash {path}: {error}") from error
    return digest.hexdigest()


def relative_path(value: str, label: str) -> Path:
    path = Path(value)
    require(not path.is_absolute(), f"{label} must be relative: {value}")
    require(".." not in path.parts, f"{label} must not escape its root: {value}")
    return path


def reduction_percent(control: int, normal: int, digits: int) -> float:
    require(control > 0, "reduction control must be positive")
    return round(100 * (control - normal) / control, digits)


def increase_percent(control: int, normal: int, digits: int) -> float:
    require(control > 0, "increase control must be positive")
    return round(100 * (normal - control) / control, digits)


def prompt_identity(tasks: list[str]) -> dict[str, Any]:
    require(all(isinstance(task, str) for task in tasks), "prompt tasks must be strings")
    payload = json.dumps(tasks, ensure_ascii=False, separators=(",", ":")).encode()
    payload += b"\n"
    return {
        "lengths": [len(task.encode()) for task in tasks],
        "json_lf_bytes": len(payload),
        "json_lf_sha256": sha256_bytes(payload),
    }


def load_frozen_scorer(study_dir: Path) -> ModuleType:
    scorer_path = study_dir / "frozen-original-task-scorer" / "quality_audit.py"
    module_name = f"efficiency_quality_audit_{sha256_bytes(str(scorer_path).encode())[:12]}"
    spec = importlib.util.spec_from_file_location(module_name, scorer_path)
    require(spec is not None and spec.loader is not None, "cannot load frozen scorer")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_frozen_files(
    study_dir: Path, provenance: dict[str, Any], result: dict[str, Any]
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    scorer = provenance["frozen_scorer"]
    checked: list[dict[str, Any]] = []
    for record in scorer["files"]:
        local_relative = relative_path(record["local_copy"], "frozen scorer local_copy")
        local_path = study_dir / local_relative
        actual = sha256_file(local_path)
        checked.append(
            {
                "path": str(local_relative),
                "sha256": actual,
                "expected_sha256": record["sha256"],
                "matches": actual == record["sha256"],
            }
        )
    require_equal(len(checked), 7, "frozen scorer file count")
    require(all(item["matches"] for item in checked), "frozen scorer file hash mismatch")

    log_root = study_dir / "frozen-original-task-scorer" / "logs"
    expected_logs: dict[Path, str] = {}
    for run in result["runs"]:
        details = run.get("check_details", {})
        for detail in details.values():
            log_relative = relative_path(detail["log"], "frozen result log")
            require(
                log_relative.parts and log_relative.parts[0] == "logs",
                f"frozen result log is outside logs/: {log_relative}",
            )
            require(log_relative not in expected_logs, f"duplicate scorer log {log_relative}")
            expected_logs[log_relative] = detail["log_sha256"]

    manifest_lines: list[str] = []
    total_bytes = 0
    for log_relative in sorted(expected_logs, key=lambda path: path.as_posix()):
        local_path = study_dir / "frozen-original-task-scorer" / log_relative
        actual = sha256_file(local_path)
        require_equal(actual, expected_logs[log_relative], f"scorer log hash {log_relative}")
        total_bytes += local_path.stat().st_size
        manifest_lines.append(f"{actual}  {Path(*log_relative.parts[1:]).as_posix()}\n")

    actual_paths = {
        path.relative_to(study_dir / "frozen-original-task-scorer")
        for path in log_root.rglob("*")
        if path.is_file()
    }
    require_equal(actual_paths, set(expected_logs), "scorer log path set")
    log_manifest_sha256 = sha256_bytes("".join(manifest_lines).encode())
    log_record = scorer["log_directory"]
    require_equal(len(expected_logs), log_record["regular_file_count"], "scorer log count")
    require_equal(total_bytes, log_record["regular_file_bytes"], "scorer log bytes")
    require_equal(
        log_manifest_sha256,
        log_record["sha256_manifest_sha256"],
        "scorer log manifest hash",
    )
    return checked, {
        "files": len(expected_logs),
        "bytes": total_bytes,
        "sha256_manifest_sha256": log_manifest_sha256,
        "all_result_hashes_match": True,
    }


def quality_stats(rows: list[dict[str, Any]]) -> dict[str, Any]:
    scores = [row["completed_features"] for row in rows]
    return {
        "runs": len(rows),
        "score_sum": sum(scores),
        "mean": sum(scores) / len(scores),
        "status_passes": sum(row["checks"]["slash_status"] == "pass" for row in rows),
        "distribution": [scores.count(score) for score in range(4)],
    }


def verify_quality(
    evidence: dict[str, Any], result: dict[str, Any], scorer: ModuleType
) -> dict[str, Any]:
    quality = evidence["quality"]
    require(result["complete"] is True, "frozen scorer result is incomplete")
    require_equal(
        result["scoring_contract"]["original_task"]["source_commit"],
        "e70c933ff0313fafb771ff214d06734845537b86",
        "original-task scorer source commit",
    )
    require_equal(
        result["scoring_contract"]["supplemental_checks"],
        ["slash_fork_continuation"],
        "supplemental scorer checks",
    )

    current = [row for row in result["runs"] if row["cohort"] == "current"]
    require_equal(
        [row["id"] for row in current],
        quality["current_expected_ids"],
        "current quality row ids",
    )
    for row in current:
        recomputed_features = scorer.feature_scores(row["checks"])
        require_equal(recomputed_features, row["features"], f"feature map for {row['id']}")
        require_equal(
            sum(recomputed_features.values()),
            row["completed_features"],
            f"feature score for {row['id']}",
        )
        require_equal(row["model"], evidence["fair_comparison"]["model"], f"model for {row['id']}")
        require_equal(
            row["reasoning_effort"],
            evidence["fair_comparison"]["reasoning_effort"],
            f"reasoning effort for {row['id']}",
        )

    sequential_rows = [row for row in current if row["workflow"] == "sequential"]
    work_leaf_rows = [row for row in current if row["workflow"] == "work-leaf"]
    sequential = quality_stats(sequential_rows)
    work_leaf = quality_stats(work_leaf_rows)
    require_equal(sequential["runs"], 6, "sequential row count")
    require_equal(work_leaf["runs"], 4, "Work Leaf row count")
    require_equal(sequential["mean"], 2.0, "sequential mean")
    require_equal(work_leaf["mean"], 2.25, "Work Leaf mean")
    require_equal(sequential["status_passes"], 4, "sequential /status passes")
    require_equal(work_leaf["status_passes"], 4, "Work Leaf /status passes")

    grouped: dict[tuple[str, int], dict[str, dict[str, Any]]] = defaultdict(dict)
    for row in current:
        key = (row["block"], row["attempt"])
        require(row["workflow"] not in grouped[key], f"duplicate pair row {key}")
        grouped[key][row["workflow"]] = row
    pair_rows: list[dict[str, Any]] = []
    unmatched_ids: list[str] = []
    for (block, attempt), bucket in sorted(grouped.items()):
        if set(bucket) != {"sequential", "work-leaf"}:
            unmatched_ids.extend(row["id"] for row in bucket.values())
            continue
        sequential_row = bucket["sequential"]
        work_leaf_row = bucket["work-leaf"]
        pair_rows.append(
            {
                "block": block,
                "attempt": attempt,
                "sequential": sequential_row["completed_features"],
                "work_leaf": work_leaf_row["completed_features"],
                "difference": (
                    work_leaf_row["completed_features"]
                    - sequential_row["completed_features"]
                ),
            }
        )
    differences = [row["difference"] for row in pair_rows]
    require_equal(differences, [-1, 2, 1, -1], "same-block pair differences")
    require_equal(
        differences,
        [
            row["difference_work_leaf_minus_sequential"]
            for row in result["same_block_comparison"]["pair_results"]
        ],
        "recomputed pair order",
    )

    historical = [
        row for row in result["runs"] if row["cohort"] == "historical-work-leaf-sanity"
    ]
    require_equal(
        [row["id"] for row in historical],
        quality["historical_work_leaf_expected_ids"],
        "historical Work Leaf sanity ids",
    )
    historical_scores = [row["completed_features"] for row in historical]
    require_equal(historical_scores, [3, 2, 2], "historical Work Leaf scores")
    historical_status = sum(row["checks"]["slash_status"] == "pass" for row in historical)
    require_equal(historical_status, 3, "historical Work Leaf /status passes")

    observed_workflow_results = sorted({row["workflow_result"] for row in current})
    observed_scores = sorted({row["completed_features"] for row in current})
    observed_attempts = sorted({row["attempt"] for row in current})
    outcomes = quality["retained_outcomes"]
    retry = quality["retry_and_row_admission"]
    require_equal(observed_workflow_results, outcomes["saved_workflow_results"], "workflow outcomes")
    require_equal(observed_scores, outcomes["observed_feature_scores"], "observed feature scores")
    require_equal(scores_zero := sum(row["completed_features"] == 0 for row in current), 0, "zero score count")
    require_equal(scores_zero, outcomes["zero_feature_rows_observed"], "recorded zero score count")
    require(outcomes["zero_feature_rows_are_eligible"] is True, "zero scores must remain eligible")
    require_equal(observed_attempts, retry["observed_attempt_numbers"], "observed attempt numbers")
    require(retry["uniform_predeclared_retry_cap"] is False, "retry cap must not be invented")

    pair_sequential = [row["sequential"] for row in pair_rows]
    pair_work_leaf = [row["work_leaf"] for row in pair_rows]
    return {
        "retained_current_rows": len(current),
        "sequential": sequential,
        "work_leaf": work_leaf,
        "pairs": {
            "count": len(pair_rows),
            "differences": differences,
            "sequential_mean": sum(pair_sequential) / len(pair_sequential),
            "work_leaf_mean": sum(pair_work_leaf) / len(pair_work_leaf),
            "unmatched_ids": sorted(unmatched_ids),
        },
        "historical_work_leaf": {
            "scores": historical_scores,
            "status_passes": historical_status,
        },
        "outcomes": {
            **outcomes,
            "observed_attempt_numbers": observed_attempts,
            "uniform_predeclared_retry_cap": retry["uniform_predeclared_retry_cap"],
        },
    }


def verify_exact_pair(
    evidence: dict[str, Any], result: dict[str, Any]
) -> dict[str, Any]:
    pair = evidence["exact_token_pair"]
    rows = {row["id"]: row for row in result["runs"]}
    sequential_id = pair["sequential"]["quality_row_id"]
    work_leaf_id = pair["work_leaf"]["quality_row_id"]
    sequential_row = rows[sequential_id]
    work_leaf_row = rows[work_leaf_id]
    require_equal(
        pair["sequential"]["analysis_sha256"],
        sequential_row["analysis_sha256"],
        "R19 sequential analysis identity",
    )
    require_equal(
        pair["sequential"]["report_sha256"],
        sequential_row["report_sha256"],
        "R19 sequential report identity",
    )
    require_equal(
        pair["work_leaf"]["analysis_sha256"],
        work_leaf_row["analysis_sha256"],
        "R19 Work Leaf analysis identity",
    )
    require_equal(
        pair["work_leaf"]["report_sha256"],
        work_leaf_row["report_sha256"],
        "R19 Work Leaf report identity",
    )
    return {
        "scope": pair["accounting_scope"],
        "sequential_raw": pair["sequential"]["raw"],
        "sequential_uncached": pair["sequential"]["uncached"],
        "work_leaf_raw": pair["work_leaf"]["raw"],
        "work_leaf_uncached": pair["work_leaf"]["uncached"],
        "raw_reduction_percent_4dp": reduction_percent(
            pair["sequential"]["raw"], pair["work_leaf"]["raw"], 4
        ),
        "uncached_reduction_percent_4dp": reduction_percent(
            pair["sequential"]["uncached"], pair["work_leaf"]["uncached"], 4
        ),
        "scores": {
            "sequential": sequential_row["completed_features"],
            "work_leaf": work_leaf_row["completed_features"],
        },
    }


def verify_mechanisms(evidence: dict[str, Any]) -> dict[str, Any]:
    mechanisms = evidence["isolated_mechanisms"]
    specifications = {
        "changed_repeated_read": ("normal", "control_full_current"),
        "unchanged_repeated_read": ("normal", "control_full_resend"),
        "inline_review_provenance": ("normal_inline_exact", "control_git_reconstruct"),
    }
    require_equal(set(mechanisms), set(specifications), "isolated mechanism set")
    result: dict[str, Any] = {}
    scopes: list[str] = []
    for name, (normal_name, control_name) in specifications.items():
        mechanism = mechanisms[name]
        normal = mechanism[normal_name]
        control = mechanism[control_name]
        scopes.append(mechanism["scope"])
        result[name] = {
            "scope": mechanism["scope"],
            "reductions_4dp": {
                "raw": reduction_percent(control["raw"], normal["raw"], 4),
                "uncached": reduction_percent(control["uncached"], normal["uncached"], 4),
            },
        }
    require_equal(len(scopes), len(set(scopes)), "isolated mechanism scopes must differ")
    return result


def verify_screens(evidence: dict[str, Any]) -> dict[str, Any]:
    screens = evidence["screens"]
    large = screens["large_read_bundle"]
    raw_increase = increase_percent(large["inline"]["raw"], large["bundle"]["raw"], 3)
    uncached_reduction = reduction_percent(
        large["inline"]["uncached"], large["bundle"]["uncached"], 3
    )
    require_equal(raw_increase, 14.638, "large bundle raw increase")
    require_equal(uncached_reduction, 59.242, "large bundle uncached reduction")
    require_equal(
        large["archive_bundle_vs_inline_direction"],
        {"raw_increase_percent": raw_increase, "uncached_reduction_percent": uncached_reduction},
        "large bundle archive direction",
    )
    requested_direction = {
        "raw_reduction_percent": raw_increase,
        "uncached_increase_percent": uncached_reduction,
    }
    require_equal(
        large["requested_feature_off_direction"],
        requested_direction,
        "large bundle requested feature-off direction",
    )

    patch = screens["patch_acknowledgement"]
    raw_benefit = patch["normal"]["raw"] < patch["neutral"]["raw"]
    behavior_benefit = (
        patch["normal"]["duplicate_patches"] < patch["neutral"]["duplicate_patches"]
    )
    linear = screens["linearization_compaction"]
    linear_activated = (
        linear["grouped_reviewed_commit_markers"] > 0
        or linear["verified_h7_records"] > 0
        or linear["rendered_targets"] != linear["unique_agent_ids"]
    )
    command = screens["command_output_compaction"]
    opportunity = sum(command["counterfactual_component_sizes"]) - sum(
        command["actual_component_sizes"]
    )
    directive = screens["directive_interruption"]
    return {
        "large_read_bundle": {
            "raw_change_percent_3dp": raw_increase,
            "uncached_reduction_percent_3dp": uncached_reduction,
            "requested_feature_off_direction": requested_direction,
        },
        "patch_acknowledgement": {
            "raw_benefit_observed": raw_benefit,
            "behavior_benefit_observed": behavior_benefit,
        },
        "linearization_compaction": {"activated": linear_activated},
        "command_output_compaction": {"opportunity_bytes": opportunity},
        "directive_interruption": {
            "post_directive_generation": directive["interrupt"]["post_directive_generation"],
            "usage_available": directive["interrupt"]["usage_available"],
        },
    }


def verify_factorial(evidence: dict[str, Any]) -> dict[str, Any]:
    factorial = evidence["factorial"]
    required = factorial["required_cells"]
    ledger = factorial["attempt_ledger"]
    exact = sorted({row["condition"] for row in ledger if row["exact"]})
    recorded_exact = sorted(factorial["exact_cells"])
    require_equal(exact, recorded_exact, "exact factorial cells")
    missing = sorted(set(required) - set(exact))
    require_equal(missing, factorial["missing_exact_cells"], "missing factorial cells")
    admission = factorial["admission_and_retries"]
    require_equal(len(ledger), admission["completed_attempts"], "completed R19 attempts")
    require_equal(
        sum(row["workflow_pass"] for row in ledger),
        admission["workflow_passes"],
        "R19 workflow passes",
    )
    require_equal(
        sum(row["exact"] for row in ledger),
        admission["exact_accounted_attempts"],
        "R19 exact attempts",
    )
    require_equal(
        sum(row["exact"] and row["workflow_pass"] for row in ledger),
        admission["exact_accounted_workflow_passes"],
        "R19 exact passing attempts",
    )
    attempts_per_condition = Counter(row["condition"] for row in ledger)
    require_equal(
        sorted(set(attempts_per_condition.values())),
        admission["observed_completed_attempts_per_condition"],
        "R19 completed attempts per condition",
    )
    require_equal(
        factorial["incomplete_attempt_directories"],
        ["step190-full-workflow-attribution-r19/runs/07-wl-110/attempt-02"],
        "excluded interrupted attempt",
    )
    return {
        "required_cells": len(required),
        "exact_cells": len(exact),
        "missing_exact_cells": missing,
        "exact_allocation_available": len(exact) == len(required),
        "mixed_block_substitution_valid": factorial["mixed_block_substitution_valid"],
        "completed_attempts": len(ledger),
        "workflow_passes": sum(row["workflow_pass"] for row in ledger),
        "exact_accounted_attempts": sum(row["exact"] for row in ledger),
        "exact_accounted_workflow_passes": sum(
            row["exact"] and row["workflow_pass"] for row in ledger
        ),
        "uniform_predeclared_retry_cap": admission["uniform_predeclared_retry_cap"],
        "current_paid_authorization": admission["current_paid_authorization"],
    }


def run_git(repo_root: Path, *arguments: str) -> str:
    completed = subprocess.run(
        ["git", *arguments],
        cwd=repo_root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(
        completed.returncode == 0,
        f"git {' '.join(arguments)} failed: {completed.stderr.strip()}",
    )
    return completed.stdout


def verify_prompts(
    study_dir: Path, evidence: dict[str, Any], manifest: dict[str, Any]
) -> dict[str, Any]:
    contracts = evidence["prompt_contracts"]
    driver = contracts["driver"]
    driver_identity = prompt_identity(driver["tasks"])
    require_equal(driver_identity["lengths"], driver["lengths"], "driver prompt lengths")
    require_equal(driver_identity["json_lf_bytes"], driver["json_lf_bytes"], "driver prompt bytes")
    require_equal(
        driver_identity["json_lf_sha256"], driver["json_lf_sha256"], "driver prompt hash"
    )

    scorer_tasks = manifest["original_task"]["features"]
    scorer_identity = prompt_identity(scorer_tasks)
    analysis = contracts["analysis_scorer"]
    require_equal(scorer_identity["lengths"], analysis["lengths"], "scorer prompt lengths")
    require_equal(
        scorer_identity["json_lf_sha256"],
        analysis["json_lf_sha256"],
        "scorer prompt hash",
    )
    require_equal(
        manifest["original_task"]["source_commit"],
        analysis["source_commit"],
        "scorer source commit",
    )
    require("/fork execution" in analysis["supplemental_only"], "/fork must remain supplemental")

    repo_root = study_dir.parents[1]
    source_commit = driver["source_commit"]
    origin_source = run_git(repo_root, "show", f"{source_commit}:bench-three-features")
    origin_tasks = re.findall(
        r"^post_command 'new (.*)' \|\| fail_bench", origin_source, flags=re.MULTILINE
    )
    require_equal(origin_tasks, driver["tasks"], "origin/master driver prompts")
    require_equal(
        run_git(repo_root, "rev-parse", "origin/master").strip(),
        source_commit,
        "origin/master commit",
    )

    work_leaf_driver = (repo_root / "bench-three-features").read_text(encoding="utf-8")
    direct_driver = (repo_root / "bench-three-features-direct-common").read_text(
        encoding="utf-8"
    )
    for task in driver["tasks"]:
        require_equal(work_leaf_driver.count(task), 1, "Work Leaf committed driver prompt count")
        require_equal(direct_driver.count(task), 1, "direct committed driver prompt count")

    return {
        "driver": {
            **driver_identity,
            "origin_commit_matches": True,
            "committed_drivers_match": True,
        },
        "analysis_scorer": {
            **scorer_identity,
            "fork_is_supplemental": True,
        },
    }


def archive_references(provenance: dict[str, Any]) -> list[dict[str, str]]:
    root = Path(provenance["archive"]["expected_root"])
    require(root.is_absolute(), "archive expected_root must be absolute")
    references: list[dict[str, str]] = []
    seen_ids: set[str] = set()
    seen_paths: set[Path] = set()
    for record in provenance["sources"]:
        require(record["id"] not in seen_ids, f"duplicate archive reference id {record['id']}")
        source_relative = relative_path(record["archive_relative_path"], "archive source")
        source_path = root / source_relative
        require(source_path not in seen_paths, f"duplicate archive source path {source_path}")
        require(re.fullmatch(r"[0-9a-f]{64}", record["sha256"]) is not None, f"bad SHA-256 for {record['id']}")
        seen_ids.add(record["id"])
        seen_paths.add(source_path)
        references.append(
            {"id": record["id"], "path": str(source_path), "sha256": record["sha256"]}
        )
    return references


def verify_external_archive(
    evidence: dict[str, Any],
    provenance: dict[str, Any],
    result: dict[str, Any],
    references: list[dict[str, str]],
) -> int:
    checked_paths: set[Path] = set()

    def check(path: Path, expected: str, label: str) -> None:
        require(path not in checked_paths, f"duplicate external archive check {path}")
        require_equal(sha256_file(path), expected, f"external archive hash for {label}")
        checked_paths.add(path)

    for reference in references:
        check(Path(reference["path"]), reference["sha256"], reference["id"])

    root = Path(provenance["archive"]["expected_root"])
    for record in provenance["frozen_scorer"]["files"]:
        relative = relative_path(record["archive_relative_path"], "frozen scorer archive source")
        check(root / relative, record["sha256"], f"frozen scorer {relative.name}")

    source_directory = root / relative_path(
        provenance["frozen_scorer"]["source_directory"], "frozen scorer source directory"
    )
    log_records: dict[Path, str] = {}
    for run in result["runs"]:
        for detail in run.get("check_details", {}).values():
            relative = relative_path(detail["log"], "external scorer log")
            log_records[relative] = detail["log_sha256"]
    for relative, expected in sorted(log_records.items(), key=lambda item: item[0].as_posix()):
        check(source_directory / relative, expected, f"frozen scorer log {relative}")

    bundle = provenance["archive"]["investigation_bundle"]
    bundle_path = root / relative_path(bundle["archive_relative_path"], "investigation bundle")
    check(bundle_path, bundle["sha256"], "investigation commit bundle")
    require_equal(bundle_path.stat().st_size, bundle["bytes"], "investigation bundle bytes")

    references_by_id = {reference["id"]: Path(reference["path"]) for reference in references}
    summary = read_json(references_by_id["recovery_summary"])
    verification = read_json(references_by_id["recovery_verification"])
    require_equal(summary["verified"], True, "archive summary verification")
    require_equal(summary["payload_tree_sha256"], provenance["archive"]["payload_tree_sha256"], "payload tree hash")
    require_equal(verification["result"], "pass", "archive verification result")
    require_equal(
        verification["source_archive_checksum_mismatch_count"],
        provenance["archive"]["source_checksum_mismatches"],
        "archive checksum mismatches",
    )

    reanalysis = read_json(references_by_id["step226_reanalysis"])
    product = reanalysis["product_comparison"]
    pair = evidence["exact_token_pair"]
    require_equal(product["direct"]["usage"], {
        "raw": pair["sequential"]["raw"],
        "uncached": pair["sequential"]["uncached"],
    }, "external R19 direct usage")
    require_equal(product["work_leaf"]["usage"], {
        "raw": pair["work_leaf"]["raw"],
        "uncached": pair["work_leaf"]["uncached"],
    }, "external R19 Work Leaf usage")
    require_equal(
        round(product["savings"]["raw"]["reduction_percent"], 4),
        reduction_percent(pair["sequential"]["raw"], pair["work_leaf"]["raw"], 4),
        "external R19 raw reduction",
    )
    require_equal(
        round(product["savings"]["uncached"]["reduction_percent"], 4),
        reduction_percent(
            pair["sequential"]["uncached"], pair["work_leaf"]["uncached"], 4
        ),
        "external R19 uncached reduction",
    )
    source_attempts = []
    for row in reanalysis["r19"]["attempts"]:
        usage = row["measurement"]["usage"]
        source_attempts.append(
            {
                "condition": row["condition"],
                "attempt": row["attempt"],
                "workflow_pass": row["workflow"]["passed"],
                "exact": row["measurement"]["exact"],
                "raw": usage["raw"] if usage else None,
                "uncached": usage["uncached"] if usage else None,
            }
        )
    require_equal(source_attempts, evidence["factorial"]["attempt_ledger"], "external R19 ledger")

    changed_source = read_json(references_by_id["step56_changed_read_comparison"])
    changed = evidence["isolated_mechanisms"]["changed_repeated_read"]
    changed_normal_source = changed_source["primary_treatment_turn"]["normal_diff"]
    changed_control_source = changed_source["primary_treatment_turn"]["full_current"]
    require_equal(
        changed["normal"],
        {
            "prompt_bytes": changed_normal_source["prompt_bytes"],
            "input": changed_normal_source["input_tokens"],
            "cached_input": changed_normal_source["cached_input_tokens"],
            "output": changed_normal_source["output_tokens"],
            "raw": changed_normal_source["raw_input_plus_output"],
            "uncached": changed_normal_source["uncached_input_plus_output"],
        },
        "external changed-reread normal",
    )
    require_equal(
        changed["control_full_current"],
        {
            "prompt_bytes": changed_control_source["prompt_bytes"],
            "input": changed_control_source["input_tokens"],
            "cached_input": changed_control_source["cached_input_tokens"],
            "output": changed_control_source["output_tokens"],
            "raw": changed_control_source["raw_input_plus_output"],
            "uncached": changed_control_source["uncached_input_plus_output"],
        },
        "external changed-reread control",
    )
    unchanged_source = read_json(references_by_id["step63_unchanged_read_comparison"])
    unchanged = evidence["isolated_mechanisms"]["unchanged_repeated_read"]
    unchanged_normal_source = unchanged_source["treated_turn"]["normal_digest"]
    unchanged_control_source = unchanged_source["treated_turn"]["full_resend"]
    require_equal(
        unchanged["normal"],
        {
            "prompt_bytes": unchanged_source["treatment"]["normal_prompt_bytes"],
            "input": unchanged_normal_source["input_tokens"],
            "cached_input": unchanged_normal_source["cached_input_tokens"],
            "output": unchanged_normal_source["output_tokens"],
            "raw": unchanged_normal_source["raw_input_plus_output"],
            "uncached": unchanged_normal_source["uncached_input_plus_output"],
        },
        "external unchanged-reread normal",
    )
    require_equal(
        unchanged["control_full_resend"],
        {
            "prompt_bytes": unchanged_source["treatment"]["full_prompt_bytes"],
            "input": unchanged_control_source["input_tokens"],
            "cached_input": unchanged_control_source["cached_input_tokens"],
            "output": unchanged_control_source["output_tokens"],
            "raw": unchanged_control_source["raw_input_plus_output"],
            "uncached": unchanged_control_source["uncached_input_plus_output"],
        },
        "external unchanged-reread control",
    )
    review_source = read_json(references_by_id["step86_review_provenance_comparison"])
    review = evidence["isolated_mechanisms"]["inline_review_provenance"]
    inline_source = review_source["usage"]["inline-exact"]
    reconstruct_source = review_source["usage"]["git-reconstruct"]
    require_equal(
        review["normal_inline_exact"],
        {
            "provider_turns": review_source["provider_turn_count"]["inline-exact"],
            "prompt_bytes": review_source["total_provider_prompt_bytes"]["inline-exact"],
            "input": inline_source["input_tokens"],
            "cached_input": inline_source["cached_input_tokens"],
            "output": inline_source["output_tokens"],
            "raw": inline_source["raw_input_plus_output"],
            "uncached": inline_source["uncached_input_plus_output"],
        },
        "external inline review",
    )
    require_equal(
        review["control_git_reconstruct"],
        {
            "provider_turns": review_source["provider_turn_count"]["git-reconstruct"],
            "prompt_bytes": review_source["total_provider_prompt_bytes"]["git-reconstruct"],
            "input": reconstruct_source["input_tokens"],
            "cached_input": reconstruct_source["cached_input_tokens"],
            "output": reconstruct_source["output_tokens"],
            "raw": reconstruct_source["raw_input_plus_output"],
            "uncached": reconstruct_source["uncached_input_plus_output"],
        },
        "external reconstructed review",
    )

    large_source = read_json(references_by_id["step50_large_read_comparison"])
    large_metrics = large_source["aggregate_user_agent_metrics"]
    large = evidence["screens"]["large_read_bundle"]
    for condition in ("inline", "bundle"):
        require_equal(
            large[condition],
            {
                "input": large_metrics["input_tokens"][condition],
                "cached_input": large_metrics["cached_input_tokens"][condition],
                "output": large_metrics["output_tokens"][condition],
                "raw": large_metrics["raw_input_plus_output"][condition],
                "uncached": large_metrics["uncached_input_plus_output"][condition],
            },
            f"external large-read {condition}",
        )
    require_equal(
        round(large_metrics["raw_input_plus_output"]["bundle_percent_change_vs_inline"], 3),
        large["archive_bundle_vs_inline_direction"]["raw_increase_percent"],
        "external bundle raw direction",
    )
    patch_source = read_json(references_by_id["step70_patch_ack_decision"])
    patch = evidence["screens"]["patch_acknowledgement"]
    require_equal(
        patch_source["classification"], patch["classification"], "external patch-ack result"
    )
    normal_patch_source = patch_source["treatment_usage"]["normal-guidance"]
    neutral_patch_source = patch_source["treatment_usage"]["neutral-confirmation"]
    require_equal(
        patch["normal"],
        {
            "prompt_bytes": patch["normal"]["prompt_bytes"],
            "raw": normal_patch_source["raw_input_plus_output"],
            "uncached": normal_patch_source["uncached_input_plus_output"],
            "duplicate_patches": normal_patch_source["duplicate_patch_directives"],
            "output": normal_patch_source["output_tokens"],
        },
        "external normal patch acknowledgement",
    )
    require_equal(
        patch["neutral"],
        {
            "prompt_bytes": patch["neutral"]["prompt_bytes"],
            "raw": neutral_patch_source["raw_input_plus_output"],
            "uncached": neutral_patch_source["uncached_input_plus_output"],
            "duplicate_patches": neutral_patch_source["duplicate_patch_directives"],
            "output": neutral_patch_source["output_tokens"],
        },
        "external neutral patch acknowledgement",
    )
    linear_source = read_json(references_by_id["step77_linearization_decision"])
    linear = evidence["screens"]["linearization_compaction"]
    require_equal(
        linear_source["decision"],
        linear["classification"],
        "external linearization screen",
    )
    require_equal(linear_source["accepted_trace"]["prompt_bytes"], linear["complete_prompt_bytes"], "external linearization prompt bytes")
    require_equal(linear_source["accepted_trace"]["target_block_bytes"], linear["target_block_bytes"], "external linearization target bytes")
    require_equal(linear_source["study_trace_audit"]["verified_h7_records"], linear["verified_h7_records"], "external linearization activations")

    command_source = read_json(references_by_id["step36_mechanism_summary"])
    h4 = [
        row
        for row in command_source["mechanisms"]["counterfactuals"]
        if row["hypothesis"] == "H4" and row["status"] == "verified"
    ]
    command = evidence["screens"]["command_output_compaction"]
    require_equal([row["actual_component_bytes"] for row in h4], command["actual_component_sizes"], "external command output actual bytes")
    require_equal([row["counterfactual_component_bytes"] for row in h4], command["counterfactual_component_sizes"], "external command output counterfactual bytes")

    interrupt_source = read_json(references_by_id["step40_interrupt_result"])
    natural_source = read_json(references_by_id["step40_natural_result"])
    directive = evidence["screens"]["directive_interruption"]
    require_equal(interrupt_source["prompt_sha256"], directive["canonical_prompt_sha256"], "external directive prompt")
    require_equal(interrupt_source["turn_status"], directive["interrupt"]["turn_status"], "external interrupt status")
    require_equal(
        interrupt_source["post_directive_generation_event_count"],
        directive["interrupt"]["post_directive_generation"],
        "external interrupt continuation",
    )
    require_equal(
        interrupt_source["reported_usage_available"],
        directive["interrupt"]["usage_available"],
        "external interrupted usage availability",
    )
    require_equal(natural_source["turn_status"], directive["natural"]["turn_status"], "external natural status")
    require_equal(natural_source["post_directive_generation_event_count"], directive["natural"]["post_directive_generation"], "external natural continuation")
    require_equal(
        natural_source["usage"]["totalTokens"], directive["natural"]["raw"], "external natural raw"
    )

    replay = read_json(references_by_id["step3_replay_ledger"])
    require_equal(replay["schema"], provenance["replay"]["schema"], "Step 3 replay schema")
    require_equal(replay["passed"], True, "Step 3 replay outcome")
    require_equal(len(replay["candidate_replays"]), 66, "Step 3 replay candidate count")
    require(all(row["passed"] for row in replay["candidate_replays"]), "Step 3 replay failure")
    require_equal(
        replay["real_agent_or_model_execution_permitted"],
        False,
        "Step 3 real-agent permission",
    )
    require_equal(
        replay["asset_evidence"]["strict_asset_count"],
        provenance["replay"]["strict_asset_file_count"],
        "Step 3 strict asset count",
    )
    require_equal(
        replay["asset_evidence"]["strict_asset_bytes"],
        provenance["replay"]["strict_asset_bytes"],
        "Step 3 strict asset bytes",
    )
    return len(checked_paths)


def verify_limits(evidence: dict[str, Any]) -> dict[str, Any]:
    mechanism_limits = evidence["isolated_mechanism_limits"]
    limitations = evidence["limitations"]
    require(mechanism_limits["separate_scopes"] is True, "mechanism scopes must be separate")
    require(mechanism_limits["percentages_can_be_added"] is False, "mechanism percentages cannot be added")
    require(
        mechanism_limits["percentages_are_whole_gap_shares"] is False,
        "mechanism percentages cannot be whole-gap shares",
    )
    require(
        limitations["formal_quality_equivalence_available"] is False,
        "formal quality equivalence must remain unavailable",
    )
    require(
        limitations["exact_whole_gap_allocation_available"] is False,
        "exact whole-gap allocation must remain unavailable",
    )
    require_equal(limitations["cross_project_generalization"], "deferred", "cross-project limit")
    return {
        "percentages_can_be_added": mechanism_limits["percentages_can_be_added"],
        "percentages_are_whole_gap_shares": mechanism_limits[
            "percentages_are_whole_gap_shares"
        ],
        "formal_quality_equivalence_available": limitations[
            "formal_quality_equivalence_available"
        ],
        "exact_whole_gap_allocation_available": limitations[
            "exact_whole_gap_allocation_available"
        ],
        "cross_project_generalization": limitations["cross_project_generalization"],
    }


def verify_study(study_dir: Path, *, check_archive: bool = False) -> dict[str, Any]:
    study_dir = study_dir.resolve()
    evidence = read_json(study_dir / "evidence.json")
    provenance = read_json(study_dir / "provenance.json")
    scorer_dir = study_dir / "frozen-original-task-scorer"
    manifest = read_json(scorer_dir / "manifest.json")
    result = read_json(scorer_dir / "result.json")
    require_equal(evidence["schema"], "work-leaf-efficiency-causal-study-v1", "evidence schema")
    require_equal(
        evidence["accepted_step_3_head"],
        "81d28af5d5e506175941956326c21e4787b10367",
        "accepted Step 3 head",
    )
    require_equal(
        provenance["schema"],
        "work-leaf-efficiency-causal-study-provenance-v1",
        "provenance schema",
    )
    require_equal(provenance["archive"]["verification_result"], "pass", "archive result")
    require_equal(provenance["archive"]["source_checksum_mismatches"], 0, "archive mismatches")
    require_equal(provenance["replay"]["candidate_count"], 66, "replay candidate count")
    require_equal(provenance["replay"]["passing_candidates"], 66, "passing replay candidates")
    require(
        provenance["replay"]["real_agent_or_model_execution_permitted"] is False,
        "Step 3 replay must forbid real agents and models",
    )
    require(
        all(value is False for key, value in provenance["step_4_consolidation"].items() if key != "archive_access"),
        "Step 4 consolidation must not execute candidates, agents, providers, or paid runs",
    )

    frozen_files, frozen_logs = verify_frozen_files(study_dir, provenance, result)
    scorer = load_frozen_scorer(study_dir)
    quality = verify_quality(evidence, result, scorer)
    exact_pair = verify_exact_pair(evidence, result)
    mechanisms = verify_mechanisms(evidence)
    screens = verify_screens(evidence)
    factorial = verify_factorial(evidence)
    limits = verify_limits(evidence)
    prompts = verify_prompts(study_dir, evidence, manifest)
    references = archive_references(provenance)
    external_checked = (
        verify_external_archive(evidence, provenance, result, references)
        if check_archive
        else 0
    )
    return {
        "quality": quality,
        "exact_token_pair": exact_pair,
        "isolated_mechanisms": mechanisms,
        "screens": screens,
        "factorial": factorial,
        "limits": limits,
        "prompts": prompts,
        "frozen_scorer_files": frozen_files,
        "frozen_scorer_logs": frozen_logs,
        "archive_references": references,
        "external_archive_files_checked": external_checked,
    }


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Recompute the compact Work Leaf efficiency evidence."
    )
    parser.add_argument(
        "--check-archive",
        action="store_true",
        help="also hash every referenced file at the frozen external archive path",
    )
    arguments = parser.parse_args(list(argv) if argv is not None else None)
    study_dir = Path(__file__).resolve().parent
    try:
        result = verify_study(study_dir, check_archive=arguments.check_archive)
    except AuditError as error:
        print(f"efficiency causal study: FAIL: {error}", file=sys.stderr)
        return 1

    quality = result["quality"]
    pair = result["exact_token_pair"]
    factorial = result["factorial"]
    print("efficiency causal study: PASS")
    print(
        "quality: sequential 2.00/3 (n=6); Work Leaf 2.25/3 (n=4)"
    )
    print(
        f"status: sequential {quality['sequential']['status_passes']}/6; "
        f"Work Leaf {quality['work_leaf']['status_passes']}/4"
    )
    print(
        f"R19 attempt 2: raw reduction {pair['raw_reduction_percent_4dp']:.4f}%; "
        f"uncached reduction {pair['uncached_reduction_percent_4dp']:.4f}%"
    )
    print(
        f"factorial: {factorial['exact_cells']}/{factorial['required_cells']} exact; "
        f"missing {','.join(factorial['missing_exact_cells'])}"
    )
    if arguments.check_archive:
        print(f"external archive: {result['external_archive_files_checked']} files match")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
