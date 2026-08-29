#!/usr/bin/env python3

import hashlib
import json
import math
from collections import Counter, defaultdict
from pathlib import Path


STUDY = Path(__file__).resolve().parent
ROOT = STUDY.parents[1]
SESSIONS = Path.home() / ".codex/sessions"
CURRENT = ROOT / "bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z"

USAGE_KEYS = (
    "input_tokens",
    "cached_input_tokens",
    "uncached_input_tokens",
    "output_tokens",
    "reasoning_output_tokens",
    "raw_input_plus_output",
    "uncached_input_plus_output",
)


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


def relative(path):
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def token_tuple(usage):
    return tuple(
        usage.get(key, 0)
        for key in (
            "input_tokens",
            "cached_input_tokens",
            "output_tokens",
            "reasoning_output_tokens",
        )
    )


def rollout_usage_changes(path):
    previous = None
    changes = 0
    events = 0
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            try:
                value = json.loads(line)
            except json.JSONDecodeError:
                continue
            payload = value.get("payload") or {}
            if value.get("type") != "event_msg" or payload.get("type") != "token_count":
                continue
            usage = ((payload.get("info") or {}).get("total_token_usage"))
            if not isinstance(usage, dict):
                continue
            events += 1
            current = token_tuple(usage)
            if current != previous:
                changes += 1
                previous = current
    return changes, events


def stage_for(thread, group):
    identity = thread.get("agent_id") or thread.get("role") or ""
    if identity == "title-agent":
        return "title"
    if identity == "linearize" or "linearize" in identity:
        return "linearization"
    if identity.startswith("review-") or "review" in identity:
        return "review"
    if group == "direct" and not identity:
        raise ValueError(f"direct thread {thread['thread_id']} has no role")
    return "implementation"


def analyze_run(run_id, group, analysis_path, quality, rollout_integrity):
    analysis = load(analysis_path)
    if not analysis["capture_complete"]:
        raise ValueError(f"{run_id} capture is incomplete: {analysis['errors']}")
    metadata_path = analysis_path.parent / "rollout-metadata.jsonl"
    metadata = [json.loads(line) for line in metadata_path.read_text().splitlines() if line.strip()]
    thread_by_id = {thread["thread_id"]: thread for thread in analysis["threads"]}
    changes_by_stage = Counter()
    total_changes = 0
    total_events = 0
    for row in metadata:
        source = SESSIONS / row["source_relative_path"]
        actual_sha = sha256(source)
        matches = actual_sha == row["source_sha256"]
        changes, events = rollout_usage_changes(source)
        stage = stage_for(thread_by_id[row["thread_id"]], group)
        changes_by_stage[stage] += changes
        total_changes += changes
        total_events += events
        key = (str(source), row["source_sha256"])
        rollout_integrity[key] = {
            "source": str(source),
            "expected_sha256": row["source_sha256"],
            "actual_sha256": actual_sha,
            "matches": matches,
            "usage_changes": changes,
            "usage_events": events,
        }
    stages = defaultdict(Counter)
    for thread in analysis["threads"]:
        stage = stage_for(thread, group)
        usage = thread["usage"]
        for key in ("input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens"):
            stages[stage][key] += usage[key]
    for stage, usage in stages.items():
        usage["uncached_input_tokens"] = usage["input_tokens"] - usage["cached_input_tokens"]
        usage["raw_input_plus_output"] = usage["input_tokens"] + usage["output_tokens"]
        usage["uncached_input_plus_output"] = (
            usage["uncached_input_tokens"] + usage["output_tokens"]
        )
        usage["usage_changes"] = changes_by_stage[stage]
    workflow_usage = analysis["usage_scopes"]["total_workflow"]
    mechanisms = analysis["mechanisms"]
    return {
        "id": run_id,
        "group": group,
        "quality": quality,
        "completed_features": sum(status == "pass" for status in quality.values()),
        "usage": {key: workflow_usage[key] for key in USAGE_KEYS},
        "usage_changes": total_changes,
        "usage_events": total_events,
        "input_tokens_per_usage_change": workflow_usage["input_tokens"] / total_changes,
        "stages": {stage: dict(values) for stage, values in sorted(stages.items())},
        "workflow_activity": {
            "command_count": mechanisms["command_count"],
            "repeated_commands": mechanisms["repeated_commands"],
            "validation_commands": mechanisms["validation"]["validation_commands"],
            "command_output_bytes": mechanisms["command_output_bytes"],
            "protocol_bytes": mechanisms["protocol_bytes"],
        },
        "analysis": relative(analysis_path),
        "analysis_sha256": sha256(analysis_path),
    }


def current_runs(rollout_integrity):
    evidence = load(CURRENT / "evidence.json")
    direct_report_overrides = {
        "direct-002": ROOT
        / "bench-results/efficiency-points8-9-20260828T145556Z/runs/direct-002/points89-direct-002-three-feature-sequential-bench-artifacts/report.json",
        "direct-003": ROOT
        / "bench-results/efficiency-points8-9-20260828T145556Z/runs/direct-003/points89-direct-003-three-feature-sequential-bench-artifacts/report.json",
    }
    runs = []
    for observation in evidence["observations"]:
        report_path = direct_report_overrides.get(
            observation["run_id"], ROOT / observation["source"]
        )
        if observation["group"] == "normal_work_leaf":
            analysis_path = ROOT / observation["analysis"]
            group = "work_leaf"
        else:
            analysis_path = report_path.parent / "observation/analysis.json"
            group = "direct"
        run = analyze_run(
            observation["run_id"],
            group,
            analysis_path,
            observation["feature_checks"],
            rollout_integrity,
        )
        expected_raw = observation["raw_tokens"]["lower"]
        if run["usage"]["raw_input_plus_output"] != expected_raw:
            raise ValueError(f"{run['id']} raw usage does not match frozen evidence")
        run["report"] = relative(report_path)
        run["report_sha256"] = sha256(report_path)
        runs.append(run)
    return runs


def historical_runs(current, rollout_integrity):
    endpoint = load(STUDY / "endpoint-evidence.json")
    current_by_id = {run["id"]: run for run in current}
    runs = [current_by_id[run_id] for run_id in ("point7-exact-direct", "direct-003", "direct-002")]
    for source in endpoint["groups"]["work_leaf"]["runs"]:
        run_id = source["id"]
        analysis_path = STUDY / f"derived/observations/{run_id}/analysis-cumulative.json"
        run = analyze_run(
            run_id,
            "work_leaf",
            analysis_path,
            source["features"],
            rollout_integrity,
        )
        if run["usage"] != source["usage"]:
            raise ValueError(f"{run_id} usage does not match endpoint evidence")
        runs.append(run)
    return runs


def group_means(runs):
    return {
        key: mean([run["usage"][key] for run in runs])
        for key in USAGE_KEYS
    }


def mean_activity(runs):
    keys = runs[0]["workflow_activity"].keys()
    return {key: mean([run["workflow_activity"][key] for run in runs]) for key in keys}


def stage_means(runs):
    stages = sorted({stage for run in runs for stage in run["stages"]})
    result = {}
    for stage in stages:
        result[stage] = {}
        for key in USAGE_KEYS + ("usage_changes",):
            result[stage][key] = mean(
                [run["stages"].get(stage, {}).get(key, 0) for run in runs]
            )
    return result


def pearson(left, right):
    left_mean = mean(left)
    right_mean = mean(right)
    numerator = sum((x - left_mean) * (y - right_mean) for x, y in zip(left, right))
    left_scale = math.sqrt(sum((x - left_mean) ** 2 for x in left))
    right_scale = math.sqrt(sum((y - right_mean) ** 2 for y in right))
    if left_scale == 0 or right_scale == 0:
        return None
    return numerator / (left_scale * right_scale)


def summarize_cohort(runs):
    direct = [run for run in runs if run["group"] == "direct"]
    work_leaf = [run for run in runs if run["group"] == "work_leaf"]
    direct_usage = group_means(direct)
    work_leaf_usage = group_means(work_leaf)
    gap = {key: direct_usage[key] - work_leaf_usage[key] for key in USAGE_KEYS}
    gap["cached_input_share_of_raw_gap_percent"] = (
        gap["cached_input_tokens"] / gap["raw_input_plus_output"] * 100
    )
    direct_changes = mean([run["usage_changes"] for run in direct])
    work_leaf_changes = mean([run["usage_changes"] for run in work_leaf])
    direct_context = direct_usage["input_tokens"] / direct_changes
    work_leaf_context = work_leaf_usage["input_tokens"] / work_leaf_changes
    count_contribution = (direct_changes - work_leaf_changes) * (
        direct_context + work_leaf_context
    ) / 2
    context_contribution = (direct_context - work_leaf_context) * (
        direct_changes + work_leaf_changes
    ) / 2
    stage_direct = stage_means(direct)
    stage_work_leaf = stage_means(work_leaf)
    all_stages = sorted(set(stage_direct) | set(stage_work_leaf))
    stage_gap = {}
    for stage in all_stages:
        raw = stage_direct.get(stage, {}).get("raw_input_plus_output", 0) - stage_work_leaf.get(
            stage, {}
        ).get("raw_input_plus_output", 0)
        uncached = stage_direct.get(stage, {}).get(
            "uncached_input_plus_output", 0
        ) - stage_work_leaf.get(stage, {}).get("uncached_input_plus_output", 0)
        stage_gap[stage] = {
            "raw_tokens": raw,
            "uncached_tokens": uncached,
            "raw_share_of_total_gap_percent": raw / gap["raw_input_plus_output"] * 100,
        }
    raw_values = [run["usage"]["raw_input_plus_output"] for run in runs]
    changes = [run["usage_changes"] for run in runs]
    contexts = [run["input_tokens_per_usage_change"] for run in runs]
    return {
        "direct": {
            "runs": len(direct),
            "completed_features": sum(run["completed_features"] for run in direct),
            "mean_usage": direct_usage,
            "mean_activity": mean_activity(direct),
            "mean_stage_usage": stage_direct,
        },
        "work_leaf": {
            "runs": len(work_leaf),
            "completed_features": sum(run["completed_features"] for run in work_leaf),
            "mean_usage": work_leaf_usage,
            "mean_activity": mean_activity(work_leaf),
            "mean_stage_usage": stage_work_leaf,
        },
        "token_gap": {
            "input_tokens": gap["input_tokens"],
            "cached_input_tokens": gap["cached_input_tokens"],
            "uncached_input_tokens": gap["uncached_input_tokens"],
            "output_tokens": gap["output_tokens"],
            "reasoning_output_tokens": gap["reasoning_output_tokens"],
            "raw_tokens": gap["raw_input_plus_output"],
            "uncached_tokens": gap["uncached_input_plus_output"],
            "cached_input_share_of_raw_gap_percent": gap[
                "cached_input_share_of_raw_gap_percent"
            ],
        },
        "usage_changes": {
            "direct_mean": direct_changes,
            "work_leaf_mean": work_leaf_changes,
            "work_leaf_reduction_percent": (direct_changes - work_leaf_changes)
            / direct_changes
            * 100,
        },
        "context_per_change": {
            "direct_mean_input_tokens": direct_context,
            "work_leaf_mean_input_tokens": work_leaf_context,
            "work_leaf_reduction_percent": (direct_context - work_leaf_context)
            / direct_context
            * 100,
        },
        "input_gap_factorization": {
            "usage_change_count_tokens": count_contribution,
            "context_size_tokens": context_contribution,
            "sum_tokens": count_contribution + context_contribution,
            "usage_change_count_share_percent": count_contribution / gap["input_tokens"] * 100,
            "context_size_share_percent": context_contribution / gap["input_tokens"] * 100,
            "meaning": "descriptive arithmetic only; it does not prove which product feature caused either factor",
        },
        "stage_gap": stage_gap,
        "descriptive_correlations": {
            "raw_tokens_vs_usage_changes": pearson(raw_values, changes),
            "raw_tokens_vs_context_per_change": pearson(raw_values, contexts),
        },
        "runs": runs,
    }


def aggregate_work_leaf_mechanisms(runs):
    summaries = [load(ROOT / run["analysis"])["mechanisms"] for run in runs]
    counterfactuals = defaultdict(lambda: {"events": 0, "statuses": Counter(), "avoided_bytes": 0})
    for summary in summaries:
        for record in summary["counterfactuals"]:
            item = counterfactuals[record["hypothesis"]]
            item["events"] += 1
            item["statuses"][record["status"]] += 1
            item["avoided_bytes"] += record.get("avoided_bytes") or 0
    bundles = [bundle for summary in summaries for bundle in summary["bundles"]]
    return {
        "counterfactuals": {
            hypothesis: {
                "events": value["events"],
                "statuses": dict(sorted(value["statuses"].items())),
                "avoided_bytes": value["avoided_bytes"],
            }
            for hypothesis, value in sorted(counterfactuals.items())
        },
        "context_bundles": {
            "events": len(bundles),
            "payload_bytes": sum(bundle["payload_bytes"] for bundle in bundles),
            "manifest_bytes": sum(bundle["manifest_bytes"] for bundle in bundles),
            "observed_follow_up_bytes": sum(bundle["observed_follow_up_bytes"] for bundle in bundles),
            "deferred_bytes": sum(bundle["deferred_bytes"] for bundle in bundles),
            "observed_path_net_bytes": sum(bundle["observed_path_net_bytes"] for bundle in bundles),
        },
        "terminal_directives": {
            key: sum(summary["terminal_directives"][key] for summary in summaries)
            for key in summaries[0]["terminal_directives"]
        },
        "structured_edits": {
            key: sum(summary["structured_edits"][key] for summary in summaries)
            for key in summaries[0]["structured_edits"]
        },
        "reviews": {
            key: sum(summary["reviews"][key] for summary in summaries)
            for key in summaries[0]["reviews"]
        },
    }


def build_evidence():
    rollout_integrity = {}
    current = current_runs(rollout_integrity)
    historical = historical_runs(current, rollout_integrity)
    mismatches = [
        row["source"] for row in rollout_integrity.values() if not row["matches"]
    ]
    current_summary = summarize_cohort(current)
    historical_summary = summarize_cohort(historical)
    work_leaf_current = [run for run in current if run["group"] == "work_leaf"]
    return {
        "schema_version": 1,
        "study": STUDY.name,
        "status": "complete" if not mismatches else "invalid",
        "rollout_integrity": {
            "checked_files": len(rollout_integrity),
            "hash_mismatches": mismatches,
            "files": list(rollout_integrity.values()),
        },
        "cohorts": {
            "current_detailed_6_by_6": current_summary,
            "historical_quality_balanced_3_by_3": historical_summary,
        },
        "normal_work_leaf_mechanisms": aggregate_work_leaf_mechanisms(work_leaf_current),
        "interpretation": {
            "proximate_source": "Work Leaf replays less input because it has fewer provider usage changes and less input context per change.",
            "not_yet_causal": "The decomposition does not identify which Work Leaf mechanism changes the cycle count or context size.",
            "command_count_warning": "Direct shell commands and Work Leaf orchestrator directives are different event types, so their counts are descriptive and not a causal allocation.",
        },
    }


def main():
    evidence = build_evidence()
    output = STUDY / "decomposition-evidence.json"
    output.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(output)


if __name__ == "__main__":
    main()
