#!/usr/bin/env python3

from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import Counter, defaultdict
from itertools import combinations
from pathlib import Path
from typing import Any, Iterable


STUDY = Path(__file__).resolve().parent
ROOT = STUDY.parents[1]
SESSIONS = Path.home() / ".codex/sessions"
PRIOR = ROOT / "bench-results/efficiency-causal-validation-20260829T210343Z"
USAGE_FIELDS = (
    "input_tokens",
    "cached_input_tokens",
    "uncached_input_tokens",
    "output_tokens",
    "reasoning_output_tokens",
    "raw_input_plus_output",
    "uncached_input_plus_output",
)
BRIDGE_STEPS = (
    ("compact_linearization", "D", "L"),
    ("work_leaf_orchestration", "L", "S"),
    ("concurrent_scheduling", "S", "C"),
    ("mediated_reads_and_interruption", "C", "W"),
)


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def mean(values: Iterable[float]) -> float:
    collected = list(values)
    if not collected:
        raise ValueError("cannot calculate the mean of an empty collection")
    return sum(collected) / len(collected)


def exact_permutation_greater(
    lower: list[float], higher: list[float]
) -> dict[str, Any]:
    combined = lower + higher
    lower_size = len(lower)
    observed = mean(higher) - mean(lower)
    at_least_observed = 0
    partitions = 0
    indexes = range(len(combined))
    for lower_indexes in combinations(indexes, lower_size):
        selected = set(lower_indexes)
        permuted_lower = [value for index, value in enumerate(combined) if index in selected]
        permuted_higher = [value for index, value in enumerate(combined) if index not in selected]
        difference = mean(permuted_higher) - mean(permuted_lower)
        partitions += 1
        if difference >= observed - 1e-9:
            at_least_observed += 1
    return {
        "observed_mean_difference": observed,
        "partitions": partitions,
        "one_sided_p": at_least_observed / partitions,
        "complete_separation": max(lower) < min(higher),
    }


def ordered_bridge(values: dict[str, float]) -> dict[str, Any]:
    missing = sorted({"D", "L", "S", "C", "W"} - values.keys())
    if missing:
        raise ValueError(f"bridge is missing conditions: {', '.join(missing)}")
    endpoint_gap = values["D"] - values["W"]
    if endpoint_gap == 0:
        raise ValueError("endpoint gap is zero")
    steps = []
    for name, left, right in BRIDGE_STEPS:
        tokens = values[left] - values[right]
        steps.append(
            {
                "name": name,
                "left": left,
                "right": right,
                "tokens": tokens,
                "share_of_endpoint_gap_percent": tokens / endpoint_gap * 100.0,
            }
        )
    allocated = sum(step["tokens"] for step in steps)
    return {
        "conditions": values,
        "endpoint_gap": endpoint_gap,
        "steps": steps,
        "allocated_tokens": allocated,
        "unallocated_tokens": endpoint_gap - allocated,
    }


def selected_causal_coverage(
    bridge: dict[str, Any], mechanisms: tuple[str, ...]
) -> dict[str, Any]:
    by_name = {step["name"]: step for step in bridge["steps"]}
    unknown = [name for name in mechanisms if name not in by_name]
    if unknown:
        raise ValueError(f"unknown bridge mechanism: {', '.join(unknown)}")
    tokens = sum(float(by_name[name]["tokens"]) for name in mechanisms)
    return {
        "mechanisms": list(mechanisms),
        "tokens": tokens,
        "share_of_endpoint_gap_percent": tokens / float(bridge["endpoint_gap"]) * 100.0,
    }


def stage_difference(
    left: dict[str, float], right: dict[str, float]
) -> dict[str, float]:
    return {
        stage: left.get(stage, 0.0) - right.get(stage, 0.0)
        for stage in sorted(set(left) | set(right))
    }


def stage_for(thread: dict[str, Any], condition: str) -> str:
    identity = str(thread.get("agent_id") or thread.get("role") or "")
    if identity == "title-agent":
        return "title"
    if identity == "linearize" or "linearize" in identity:
        return "linearization"
    if identity.startswith("review-") or "review" in identity:
        return "review"
    if condition == "compact-direct" and not identity:
        raise ValueError(f"direct thread {thread.get('thread_id')} has no saved role")
    return "implementation"


def usage_with_derived(values: Counter[str]) -> dict[str, int]:
    values["uncached_input_tokens"] = (
        values["input_tokens"] - values["cached_input_tokens"]
    )
    values["raw_input_plus_output"] = values["input_tokens"] + values["output_tokens"]
    values["uncached_input_plus_output"] = (
        values["uncached_input_tokens"] + values["output_tokens"]
    )
    return {field: int(values[field]) for field in USAGE_FIELDS}


def stage_usage(analysis: dict[str, Any], condition: str) -> dict[str, dict[str, int]]:
    stages: dict[str, Counter[str]] = defaultdict(Counter)
    for thread in analysis["threads"]:
        stage = stage_for(thread, condition)
        for field in (
            "input_tokens",
            "cached_input_tokens",
            "output_tokens",
            "reasoning_output_tokens",
        ):
            stages[stage][field] += int(thread["usage"][field])
    return {
        stage: usage_with_derived(values)
        for stage, values in sorted(stages.items())
    }


def token_tuple(usage: dict[str, Any]) -> tuple[int, int, int, int]:
    return tuple(
        int(usage.get(field, 0))
        for field in (
            "input_tokens",
            "cached_input_tokens",
            "output_tokens",
            "reasoning_output_tokens",
        )
    )


def rollout_usage_changes(path: Path) -> tuple[int, int]:
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
            usage = (payload.get("info") or {}).get("total_token_usage")
            if not isinstance(usage, dict):
                continue
            events += 1
            current = token_tuple(usage)
            if current != previous:
                changes += 1
                previous = current
    return changes, events


def rollout_actions(path: Path) -> Counter[str]:
    actions: Counter[str] = Counter()
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            try:
                value = json.loads(line)
            except json.JSONDecodeError:
                continue
            if value.get("type") != "response_item":
                continue
            payload = value.get("payload") or {}
            payload_type = payload.get("type")
            if payload_type in {"function_call", "custom_tool_call"}:
                actions[str(payload.get("name") or "unnamed_tool")] += 1
            elif payload_type == "web_search_call":
                actions["web_search"] += 1
    return actions


def rollout_activity(
    analysis_path: Path, analysis: dict[str, Any], condition: str
) -> dict[str, Any]:
    metadata_path = analysis_path.parent / "rollout-metadata.jsonl"
    metadata = [
        json.loads(line)
        for line in metadata_path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    thread_by_id = {thread["thread_id"]: thread for thread in analysis["threads"]}
    changes_by_stage: Counter[str] = Counter()
    events_by_stage: Counter[str] = Counter()
    actions: Counter[str] = Counter()
    actions_by_stage: dict[str, Counter[str]] = defaultdict(Counter)
    files = []
    for row in metadata:
        source = SESSIONS / row["source_relative_path"]
        actual = sha256(source)
        if actual != row["source_sha256"]:
            raise ValueError(f"saved rollout hash changed: {source}")
        changes, events = rollout_usage_changes(source)
        source_actions = rollout_actions(source)
        stage = stage_for(thread_by_id[row["thread_id"]], condition)
        changes_by_stage[stage] += changes
        events_by_stage[stage] += events
        actions.update(source_actions)
        actions_by_stage[stage].update(source_actions)
        files.append(
            {
                "thread_id": row["thread_id"],
                "stage": stage,
                "source": str(source),
                "sha256": actual,
                "usage_changes": changes,
                "usage_events": events,
            }
        )
    return {
        "usage_changes": sum(changes_by_stage.values()),
        "usage_events": sum(events_by_stage.values()),
        "usage_changes_by_stage": dict(sorted(changes_by_stage.items())),
        "usage_events_by_stage": dict(sorted(events_by_stage.items())),
        "actions": dict(sorted(actions.items())),
        "actions_by_stage": {
            stage: dict(sorted(counts.items()))
            for stage, counts in sorted(actions_by_stage.items())
        },
        "rollouts": files,
    }


def review_rounds(artifact: Path, condition: str) -> int:
    if condition == "compact-direct":
        rounds = []
        for path in sorted((artifact / "review-results").glob("*.review")):
            match = re.search(
                r"(?m)^rounds=(\d+)\s*$.*?^result=clean\s*$",
                path.read_text(encoding="utf-8"),
                re.DOTALL,
            )
            if match is None:
                raise ValueError(f"cannot parse clean direct review result: {path}")
            rounds.append(int(match.group(1)))
    else:
        state = load_json(artifact / "final-state.json")
        rows = [
            line
            for line in state["snapshot"]["command_transcript"]
            if " reviewed by " in line
        ]
        rounds = []
        for row in rows:
            match = re.search(r"rounds=(\d+)\s+resolved=yes", row)
            if match is None:
                raise ValueError(f"cannot parse Work Leaf review result: {row}")
            rounds.append(int(match.group(1)))
    if len(rounds) != 3:
        raise ValueError(f"expected three reviewed features, found {len(rounds)}")
    return sum(rounds)


def directive_counts(artifact: Path, condition: str) -> dict[str, int]:
    if condition == "compact-direct":
        return {"edit": 0, "locks_run": 0, "read": 0}
    state = load_json(artifact / "final-state.json")
    lines = [
        line
        for session in state["snapshot"]["sessions"]
        for line in session["lines"]
    ]
    return {
        "edit": sum(line.startswith("@work-leaf edit ") for line in lines),
        "locks_run": sum(line.startswith("@work-leaf locks run ") for line in lines),
        "read": sum(line.startswith("@work-leaf read ") for line in lines),
    }


def compact_target_activation(artifact: Path) -> dict[str, Any]:
    prompt = artifact / "runs/linearize-plan.prompt.txt"
    text = prompt.read_text(encoding="utf-8")
    return {
        "active": (
            "Exact reviewed provisional targets:" in text
            and "The list above is the complete reviewed target set." in text
            and text.count("Feature target ") == 3
            and text.count("  - Commit: ") >= 3
        ),
        "prompt": str(prompt),
        "prompt_sha256": sha256(prompt),
        "feature_target_count": text.count("Feature target "),
        "commit_count": text.count("  - Commit: "),
    }


def sequential_activation(artifact: Path, report: dict[str, Any]) -> dict[str, Any]:
    log = artifact.parent.parent / f"{report['run_id']}.log"
    if not log.is_file():
        log = STUDY / "logs" / f"{report['run_id']}.log"
    lines = log.read_text(encoding="utf-8").splitlines()
    starts = [index for index, line in enumerate(lines) if "started sequential diagnostic patch agent" in line]
    gates = []
    for index in starts[1:]:
        previous = lines[index - 1] if index else ""
        gates.append("busy=false" in previous and "NeedsDecision" in previous)
    analysis = load_json(artifact / "observation/analysis.json")
    return {
        "active": (
            report.get("feature_schedule") == "sequential-diagnostic"
            and report.get("no_read_permission") == "1"
            and len(starts) == 3
            and gates == [True, True]
            and int(analysis.get("interrupted_provider_turns", 0)) == 0
        ),
        "feature_starts": len(starts),
        "prior_feature_terminal_at_later_starts": gates,
        "direct_read_mode": report.get("no_read_permission") == "1",
        "interrupted_provider_turns": int(analysis.get("interrupted_provider_turns", 0)),
        "log": str(log),
        "log_sha256": sha256(log),
    }


def analyze_new_run(row: dict[str, Any]) -> dict[str, Any]:
    if not row["measurement"]["usable"]:
        raise ValueError(f"{row['id']} has unusable measurement: {row['measurement']['reasons']}")
    report_path = ROOT / row["report"]
    report = load_json(report_path)
    analysis_path = Path(row["measurement"]["analysis"])
    analysis = load_json(analysis_path)
    if not analysis.get("capture_complete") or analysis.get("errors"):
        raise ValueError(f"{row['id']} observer capture is incomplete")
    if sha256(analysis_path) != row["measurement"]["analysis_sha256"]:
        raise ValueError(f"{row['id']} observer analysis hash changed after scoring")
    artifact = report_path.parent
    rollout_audit = load_json(artifact / "observation/rollout-audit.json")
    if any(
        rollout_audit.get(field)
        for field in (
            "errors",
            "missing_threads",
            "session_only_threads",
            "unobserved_cwd_threads",
        )
    ):
        raise ValueError(f"{row['id']} rollout audit is not clean")
    recursive = artifact / "recursive-codex-attempts.log"
    if not recursive.is_file() or recursive.stat().st_size:
        raise ValueError(f"{row['id']} has a recursive provider attempt")
    condition = row["condition"]
    activation = (
        compact_target_activation(artifact)
        if condition == "compact-direct"
        else sequential_activation(artifact, report)
    )
    if not activation["active"]:
        raise ValueError(f"{row['id']} did not activate its intended control")
    activity = rollout_activity(analysis_path, analysis, condition)
    mechanisms = analysis["mechanisms"]
    return {
        "id": row["id"],
        "condition": condition,
        "workflow_result": row["workflow_result"],
        "checks": row["checks"],
        "completed_features": int(row["completed_features"]),
        "usage": {field: int(row["measurement"]["usage"][field]) for field in USAGE_FIELDS},
        "stages": stage_usage(analysis, condition),
        "usage_changes": activity["usage_changes"],
        "usage_events": activity["usage_events"],
        "usage_changes_by_stage": activity["usage_changes_by_stage"],
        "provider_actions": activity["actions"],
        "provider_actions_by_stage": activity["actions_by_stage"],
        "work_leaf_directives": directive_counts(artifact, condition),
        "review_rounds": review_rounds(artifact, condition),
        "workflow_activity": {
            "command_count": int(mechanisms["command_count"]),
            "repeated_commands": int(mechanisms["repeated_commands"]),
            "validation_commands": int(mechanisms["validation"]["validation_commands"]),
            "command_output_bytes": int(mechanisms["command_output_bytes"]),
            "protocol_bytes": int(mechanisms["protocol_bytes"]),
            "structured_edit_submissions": int(
                mechanisms["structured_edits"]["submissions"]
            ),
        },
        "changed_lines_total": int(report["changed_lines_total"]),
        "activation": activation,
        "report": row["report"],
        "report_sha256": sha256(report_path),
        "analysis": str(analysis_path),
        "analysis_sha256": sha256(analysis_path),
        "rollout_integrity": activity["rollouts"],
    }


def mean_usage(runs: list[dict[str, Any]]) -> dict[str, float]:
    return {
        field: mean(float(run["usage"][field]) for run in runs)
        for field in USAGE_FIELDS
    }


def mean_stages(runs: list[dict[str, Any]]) -> dict[str, dict[str, float]]:
    stages = sorted({stage for run in runs for stage in run["stages"]})
    return {
        stage: {
            field: mean(float(run["stages"].get(stage, {}).get(field, 0)) for run in runs)
            for field in USAGE_FIELDS
        }
        for stage in stages
    }


def summarize_new_group(runs: list[dict[str, Any]]) -> dict[str, Any]:
    usage = mean_usage(runs)
    usage_change_stages = sorted(
        {stage for run in runs for stage in run["usage_changes_by_stage"]}
    )
    action_stages = sorted(
        {stage for run in runs for stage in run["provider_actions_by_stage"]}
    )
    return {
        "runs": len(runs),
        "completed_features": sum(run["completed_features"] for run in runs),
        "full_quality_runs": sum(run["completed_features"] == 3 for run in runs),
        "mean_usage": usage,
        "range": {
            "raw_input_plus_output": {
                "minimum": min(run["usage"]["raw_input_plus_output"] for run in runs),
                "maximum": max(run["usage"]["raw_input_plus_output"] for run in runs),
            },
            "uncached_input_plus_output": {
                "minimum": min(run["usage"]["uncached_input_plus_output"] for run in runs),
                "maximum": max(run["usage"]["uncached_input_plus_output"] for run in runs),
            },
        },
        "mean_stage_usage": mean_stages(runs),
        "mean_usage_changes": mean(float(run["usage_changes"]) for run in runs),
        "mean_usage_changes_by_stage": {
            stage: mean(
                float(run["usage_changes_by_stage"].get(stage, 0)) for run in runs
            )
            for stage in usage_change_stages
        },
        "mean_workflow_activity": {
            field: mean(float(run["workflow_activity"][field]) for run in runs)
            for field in runs[0]["workflow_activity"]
        },
        "mean_provider_actions": {
            action: mean(float(run["provider_actions"].get(action, 0)) for run in runs)
            for action in sorted(
                {action for run in runs for action in run["provider_actions"]}
            )
        },
        "mean_provider_actions_by_stage": {
            stage: {
                action: mean(
                    float(
                        run["provider_actions_by_stage"]
                        .get(stage, {})
                        .get(action, 0)
                    )
                    for run in runs
                )
                for action in sorted(
                    {
                        action
                        for run in runs
                        for action in run["provider_actions_by_stage"].get(stage, {})
                    }
                )
            }
            for stage in action_stages
        },
        "mean_work_leaf_directives": {
            directive: mean(
                float(run["work_leaf_directives"][directive]) for run in runs
            )
            for directive in ("edit", "locks_run", "read")
        },
        "mean_review_rounds": mean(float(run["review_rounds"]) for run in runs),
        "changed_lines": [run["changed_lines_total"] for run in runs],
        "run_rows": runs,
    }


def build_evidence(quality_path: Path) -> dict[str, Any]:
    quality = load_json(quality_path)
    runs = [analyze_new_run(row) for row in quality["runs"]]
    compact = [run for run in runs if run["condition"] == "compact-direct"]
    sequential = [
        run for run in runs if run["condition"] == "sequential-work-leaf-combined"
    ]
    if len(compact) != 3 or len(sequential) != 3:
        raise ValueError("final analysis requires exactly three L and three S observations")
    prior = load_json(PRIOR / "combined-evidence.json")
    groups = {
        "D": prior["groups"]["direct_sequential"],
        "L": summarize_new_group(compact),
        "S": summarize_new_group(sequential),
        "C": prior["groups"]["combined_work_leaf"],
        "W": prior["groups"]["normal_work_leaf"],
    }
    raw = ordered_bridge(
        {
            symbol: float(group["mean_usage"]["raw_input_plus_output"])
            for symbol, group in groups.items()
        }
    )
    uncached = ordered_bridge(
        {
            symbol: float(group["mean_usage"]["uncached_input_plus_output"])
            for symbol, group in groups.items()
        }
    )
    l_stage = {
        stage: values["raw_input_plus_output"]
        for stage, values in groups["L"]["mean_stage_usage"].items()
    }
    s_stage = {
        stage: values["raw_input_plus_output"]
        for stage, values in groups["S"]["mean_stage_usage"].items()
    }
    l_minus_s_stage = stage_difference(l_stage, s_stage)
    full_compact = [run for run in compact if run["completed_features"] == 3]
    full_sequential = [run for run in sequential if run["completed_features"] == 3]
    full_quality_test = None
    if full_compact and full_sequential:
        full_quality_test = exact_permutation_greater(
            lower=[run["usage"]["raw_input_plus_output"] for run in full_sequential],
            higher=[run["usage"]["raw_input_plus_output"] for run in full_compact],
        )
    causal_coverage = selected_causal_coverage(
        raw,
        ("work_leaf_orchestration", "mediated_reads_and_interruption"),
    )
    l_usage = groups["L"]["mean_usage"]
    s_usage = groups["S"]["mean_usage"]
    l_changes = float(groups["L"]["mean_usage_changes"])
    s_changes = float(groups["S"]["mean_usage_changes"])
    l_actions = groups["L"]["mean_provider_actions_by_stage"]
    s_actions = groups["S"]["mean_provider_actions_by_stage"]
    return {
        "schema_version": 1,
        "study": STUDY.name,
        "status": "complete",
        "quality_file": str(quality_path),
        "quality_file_sha256": sha256(quality_path),
        "groups": groups,
        "ordered_attribution": {"raw": raw, "uncached": uncached},
        "causal_coverage": causal_coverage,
        "work_leaf_orchestration_behavior": {
            "raw_reduction_relative_to_compact_direct_percent": (
                (l_usage["raw_input_plus_output"] - s_usage["raw_input_plus_output"])
                / l_usage["raw_input_plus_output"]
                * 100.0
            ),
            "token_class_difference": {
                field: float(l_usage[field]) - float(s_usage[field])
                for field in USAGE_FIELDS
            },
            "provider_usage_changes": {
                "compact_direct": l_changes,
                "sequential_work_leaf": s_changes,
                "difference": l_changes - s_changes,
                "reduction_percent": (l_changes - s_changes) / l_changes * 100.0,
                "by_stage": {
                    stage: {
                        "compact_direct": float(
                            groups["L"]["mean_usage_changes_by_stage"].get(stage, 0)
                        ),
                        "sequential_work_leaf": float(
                            groups["S"]["mean_usage_changes_by_stage"].get(stage, 0)
                        ),
                    }
                    for stage in sorted(
                        set(groups["L"]["mean_usage_changes_by_stage"])
                        | set(groups["S"]["mean_usage_changes_by_stage"])
                    )
                },
            },
            "patch_agent_writes": {
                "compact_direct_native_apply_patch_calls": float(
                    l_actions.get("implementation", {}).get("apply_patch", 0)
                ),
                "sequential_work_leaf_native_apply_patch_calls": float(
                    s_actions.get("implementation", {}).get("apply_patch", 0)
                ),
                "sequential_work_leaf_structured_edit_submissions": float(
                    groups["S"]["mean_work_leaf_directives"]["edit"]
                ),
            },
            "native_exec_calls": {
                stage: {
                    "compact_direct": float(
                        l_actions.get(stage, {}).get("exec_command", 0)
                    ),
                    "sequential_work_leaf": float(
                        s_actions.get(stage, {}).get("exec_command", 0)
                    ),
                }
                for stage in ("implementation", "review", "linearization")
            },
            "review_rounds": {
                "compact_direct": float(groups["L"]["mean_review_rounds"]),
                "sequential_work_leaf": float(groups["S"]["mean_review_rounds"]),
            },
        },
        "work_leaf_orchestration_stage_attribution": {
            "raw_tokens": l_minus_s_stage,
            "sum_raw_tokens": sum(l_minus_s_stage.values()),
            "transition_raw_tokens": next(
                step["tokens"]
                for step in raw["steps"]
                if step["name"] == "work_leaf_orchestration"
            ),
            "raw_share_of_endpoint_gap_percent": {
                stage: tokens / raw["endpoint_gap"] * 100.0
                for stage, tokens in l_minus_s_stage.items()
            },
        },
        "counterchecks": {
            "compact_direct_vs_sequential_work_leaf_raw": exact_permutation_greater(
                lower=[run["usage"]["raw_input_plus_output"] for run in sequential],
                higher=[run["usage"]["raw_input_plus_output"] for run in compact],
            ),
            "compact_direct_vs_sequential_work_leaf_usage_changes": exact_permutation_greater(
                lower=[run["usage_changes"] for run in sequential],
                higher=[run["usage_changes"] for run in compact],
            ),
            "full_quality_raw": full_quality_test,
            "quality": {
                "compact_direct": {
                    "completed_features": groups["L"]["completed_features"],
                    "possible_features": len(compact) * 3,
                    "full_quality_runs": len(full_compact),
                },
                "sequential_work_leaf": {
                    "completed_features": groups["S"]["completed_features"],
                    "possible_features": len(sequential) * 3,
                    "full_quality_runs": len(full_sequential),
                },
            },
        },
        "source_evidence": {
            "prior": str(PRIOR / "combined-evidence.json"),
            "prior_sha256": sha256(PRIOR / "combined-evidence.json"),
        },
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--quality", type=Path, default=STUDY / "quality.json")
    parser.add_argument("--output", type=Path, default=STUDY / "evidence.json")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    evidence = build_evidence(args.quality.resolve())
    args.output.resolve().write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(args.output.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
