#!/usr/bin/env python3

import hashlib
import importlib.util
import itertools
import json
from pathlib import Path


STUDY = Path(__file__).resolve().parent
ROOT = STUDY.parents[1]
NORMAL_STUDY = ROOT / "bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z"
DECOMPOSITION = STUDY / "decomposition-evidence.json"
CONTROL_QUALITY = STUDY / "control-quality.json"
CONTROL_IDS = ("direct-read-001", "direct-read-002", "direct-read-003")
USAGE_KEYS = (
    "input_tokens",
    "cached_input_tokens",
    "uncached_input_tokens",
    "output_tokens",
    "reasoning_output_tokens",
    "raw_input_plus_output",
    "uncached_input_plus_output",
)


def load(path: Path):
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def mean(values):
    return sum(values) / len(values)


def relative(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def load_decompose_module():
    path = STUDY / "decompose.py"
    specification = importlib.util.spec_from_file_location("causal_decompose", path)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load decomposition code: {path}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def usage_from(value):
    return {key: value[key] for key in USAGE_KEYS}


def exact_permutation_greater(reference, control):
    values = list(reference) + list(control)
    control_size = len(control)
    observed = mean(control) - mean(reference)
    differences = []
    for selected in itertools.combinations(range(len(values)), control_size):
        selected = set(selected)
        candidate_control = [value for index, value in enumerate(values) if index in selected]
        candidate_reference = [value for index, value in enumerate(values) if index not in selected]
        differences.append(mean(candidate_control) - mean(candidate_reference))
    return {
        "alternative": "control is greater than normal Work Leaf",
        "observed_difference": observed,
        "assignments": len(differences),
        "p_value": sum(value >= observed - 1e-9 for value in differences) / len(differences),
    }


def summarize_group(runs):
    usage = {
        key: mean([run["usage"][key] for run in runs])
        for key in USAGE_KEYS
    }
    changes = mean([run["usage_changes"] for run in runs])
    return {
        "runs": len(runs),
        "completed_features": sum(run["completed_features"] for run in runs),
        "full_quality_runs": sum(run["completed_features"] == 3 for run in runs),
        "mean_usage": usage,
        "range": {
            key: {
                "minimum": min(run["usage"][key] for run in runs),
                "maximum": max(run["usage"][key] for run in runs),
            }
            for key in ("raw_input_plus_output", "uncached_input_plus_output")
        },
        "mean_usage_changes": changes,
        "mean_input_context_per_usage_change": usage["input_tokens"] / changes,
        "run_rows": runs,
    }


def full_quality_summary(runs):
    selected = [run for run in runs if run["completed_features"] == 3]
    return summarize_group(selected) if selected else None


def observation_path(run_id: str) -> Path:
    return (
        STUDY
        / "runs"
        / run_id
        / f"{run_id}-three-feature-bench-artifacts"
        / "observation"
    )


def parse_json_lines(path: Path):
    values = []
    errors = 0
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if not line.strip():
                continue
            try:
                values.append(json.loads(line))
            except json.JSONDecodeError:
                errors += 1
    return values, errors


def activation_for(run_id, quality):
    artifact = observation_path(run_id).parent
    observation = artifact / "observation"
    direct_threads = set()
    mediated_threads = set()
    turn_threads = set()
    mediated_reads = 0
    command_executions = 0
    direct_read_commands = 0
    parse_errors = 0
    raw_files = []

    for path in sorted((observation / "app-server").glob("*/client-to-server.raw")):
        values, errors = parse_json_lines(path)
        parse_errors += errors
        raw_files.append({"path": relative(path), "sha256": sha256(path)})
        for value in values:
            if value.get("method") != "turn/start":
                continue
            params = value.get("params") or {}
            thread_id = params.get("threadId")
            if thread_id:
                turn_threads.add(thread_id)
            text = "\n".join(
                item.get("text", "")
                for item in params.get("input", [])
                if isinstance(item, dict)
            )
            if "You may read repository files directly from the filesystem." in text:
                direct_threads.add(thread_id)
            if "You are not allowed to read files directly" in text:
                mediated_threads.add(thread_id)

    for path in sorted((observation / "app-server").glob("*/server-to-client.raw")):
        values, errors = parse_json_lines(path)
        parse_errors += errors
        raw_files.append({"path": relative(path), "sha256": sha256(path)})
        for value in values:
            if value.get("method") != "item/completed":
                continue
            item = ((value.get("params") or {}).get("item") or {})
            if item.get("type") == "commandExecution":
                command_executions += 1
                if any(
                    action.get("type") == "read"
                    for action in item.get("commandActions") or []
                    if isinstance(action, dict)
                ):
                    direct_read_commands += 1
            elif item.get("type") == "agentMessage":
                mediated_reads += sum(
                    line.startswith("@work-leaf read ")
                    for line in (item.get("text") or "").splitlines()
                )

    original = load(observation / "analysis.json")
    cumulative = load(observation / "analysis-cumulative.json")
    report = load(artifact / "report.json")
    admission = load(STUDY / "runs" / run_id / "admission.json")
    original_usage = usage_from(original["usage_scopes"]["total_workflow"])
    cumulative_usage = usage_from(cumulative["usage_scopes"]["total_workflow"])
    report_usage = usage_from(report["total_workflow_usage"])
    exact_stratum = cumulative.get("model_strata") == [
        {
            "model": "gpt-5.5",
            "effort": "xhigh",
            "thread_count": 8,
            "primary_threads": 8,
            "visible_threads": 7,
            "descendant_threads": 0,
            "usage": {
                "input_tokens": cumulative_usage["input_tokens"],
                "cached_input_tokens": cumulative_usage["cached_input_tokens"],
                "output_tokens": cumulative_usage["output_tokens"],
                "reasoning_output_tokens": cumulative_usage["reasoning_output_tokens"],
            },
        }
    ]
    recursive_log = artifact / "recursive-codex-attempts.log"
    exact_accounting = (
        original.get("capture_complete") is True
        and cumulative.get("capture_complete") is True
        and original.get("errors") == []
        and cumulative.get("errors") == []
        and len(cumulative.get("threads", [])) == 8
        and exact_stratum
        and original_usage == cumulative_usage == report_usage
    )
    activation = {
        "id": run_id,
        "turn_start_threads": len(turn_threads),
        "direct_prompt_threads": len(direct_threads),
        "mediated_prompt_threads": len(mediated_threads),
        "mediated_read_directives": mediated_reads,
        "command_executions": command_executions,
        "direct_read_commands": direct_read_commands,
        "json_line_parse_errors": parse_errors,
        "workflow_result": report["workflow_result"],
        "read_permission_mode": report["read_permission_mode"],
        "model": report["agent_model"],
        "reasoning_effort": report["agent_reasoning_effort"],
        "completed_features": quality["completed_features"],
        "feature_checks": quality["checks"],
        "exact_accounting": exact_accounting,
        "original_and_cumulative_usage_match": original_usage == cumulative_usage,
        "recursive_provider_attempt_bytes": recursive_log.stat().st_size,
        "admission": admission,
        "raw_trace_files": raw_files,
    }
    activation["passed"] = (
        activation["turn_start_threads"] == 8
        and activation["direct_prompt_threads"] == 7
        and activation["mediated_prompt_threads"] == 0
        and activation["mediated_read_directives"] == 0
        and activation["direct_read_commands"] > 0
        and activation["json_line_parse_errors"] == 0
        and activation["workflow_result"] == "pass"
        and activation["read_permission_mode"]
        == "direct agent file reads enabled (--no-read-permission)"
        and activation["model"] == "gpt-5.5"
        and activation["reasoning_effort"] == "xhigh"
        and activation["completed_features"] == 3
        and activation["exact_accounting"]
        and activation["recursive_provider_attempt_bytes"] == 0
        and admission["condition"] == "work-leaf-direct-read"
    )
    return activation


def compare_groups(direct, normal, control):
    direct_gap_raw = (
        direct["mean_usage"]["raw_input_plus_output"]
        - normal["mean_usage"]["raw_input_plus_output"]
    )
    direct_gap_uncached = (
        direct["mean_usage"]["uncached_input_plus_output"]
        - normal["mean_usage"]["uncached_input_plus_output"]
    )
    raw_delta = (
        control["mean_usage"]["raw_input_plus_output"]
        - normal["mean_usage"]["raw_input_plus_output"]
    )
    uncached_delta = (
        control["mean_usage"]["uncached_input_plus_output"]
        - normal["mean_usage"]["uncached_input_plus_output"]
    )
    input_delta = control["mean_usage"]["input_tokens"] - normal["mean_usage"]["input_tokens"]
    changes_delta = control["mean_usage_changes"] - normal["mean_usage_changes"]
    context_delta = (
        control["mean_input_context_per_usage_change"]
        - normal["mean_input_context_per_usage_change"]
    )
    normal_changes = normal["mean_usage_changes"]
    control_changes = control["mean_usage_changes"]
    normal_context = normal["mean_input_context_per_usage_change"]
    control_context = control["mean_input_context_per_usage_change"]
    count_contribution = (control_changes - normal_changes) * (
        control_context + normal_context
    ) / 2
    context_contribution = (control_context - normal_context) * (
        control_changes + normal_changes
    ) / 2

    normal_raw = [run["usage"]["raw_input_plus_output"] for run in normal["run_rows"]]
    control_raw = [run["usage"]["raw_input_plus_output"] for run in control["run_rows"]]
    normal_uncached = [
        run["usage"]["uncached_input_plus_output"] for run in normal["run_rows"]
    ]
    control_uncached = [
        run["usage"]["uncached_input_plus_output"] for run in control["run_rows"]
    ]
    return {
        "raw_tokens": raw_delta,
        "uncached_tokens": uncached_delta,
        "input_tokens": input_delta,
        "usage_changes": changes_delta,
        "input_context_per_usage_change": context_delta,
        "raw_increase_over_normal_percent": raw_delta
        / normal["mean_usage"]["raw_input_plus_output"]
        * 100,
        "uncached_increase_over_normal_percent": uncached_delta
        / normal["mean_usage"]["uncached_input_plus_output"]
        * 100,
        "raw_fraction_of_direct_gap_percent": raw_delta / direct_gap_raw * 100,
        "uncached_fraction_of_direct_gap_percent": uncached_delta
        / direct_gap_uncached
        * 100,
        "input_gap_factorization": {
            "fewer_usage_changes_tokens": count_contribution,
            "larger_context_tokens": context_contribution,
            "sum_tokens": count_contribution + context_contribution,
            "observed_input_delta": input_delta,
        },
        "exact_permutation": {
            "raw_tokens": exact_permutation_greater(normal_raw, control_raw),
            "uncached_tokens": exact_permutation_greater(normal_uncached, control_uncached),
        },
    }


def build_evidence():
    decompose = load_decompose_module()
    decomposition = load(DECOMPOSITION)
    quality = load(CONTROL_QUALITY)
    quality_by_id = {row["id"]: row for row in quality["runs"]}
    rollout_integrity = {}
    controls = []
    activations = []
    for run_id in CONTROL_IDS:
        analysis = observation_path(run_id) / "analysis-cumulative.json"
        controls.append(
            decompose.analyze_run(
                run_id,
                "work_leaf",
                analysis,
                quality_by_id[run_id]["checks"],
                rollout_integrity,
            )
        )
        activations.append(activation_for(run_id, quality_by_id[run_id]))

    current = decomposition["cohorts"]["current_detailed_6_by_6"]["runs"]
    direct_runs = [run for run in current if run["group"] == "direct"]
    normal_runs = [run for run in current if run["group"] == "work_leaf"]
    direct = summarize_group(direct_runs)
    normal = summarize_group(normal_runs)
    control = summarize_group(controls)
    full_direct = full_quality_summary(direct_runs)
    full_normal = full_quality_summary(normal_runs)
    full_control = full_quality_summary(controls)
    full_comparison = compare_groups(full_direct, full_normal, full_control)
    comparison = compare_groups(direct, normal, control)
    mismatches = [row["source"] for row in rollout_integrity.values() if not row["matches"]]
    complete = not mismatches and all(row["passed"] for row in activations)

    direct_read_raw = control["mean_usage"]["raw_input_plus_output"]
    direct_raw = direct["mean_usage"]["raw_input_plus_output"]
    direct_read_uncached = control["mean_usage"]["uncached_input_plus_output"]
    direct_uncached = direct["mean_usage"]["uncached_input_plus_output"]
    return {
        "schema_version": 1,
        "study": STUDY.name,
        "status": "complete" if complete else "invalid",
        "references": {
            "decomposition": {
                "path": relative(DECOMPOSITION),
                "sha256": sha256(DECOMPOSITION),
            },
            "normal_quality": {
                "path": relative(NORMAL_STUDY / "quality.json"),
                "sha256": sha256(NORMAL_STUDY / "quality.json"),
            },
            "control_quality": {
                "path": relative(CONTROL_QUALITY),
                "sha256": sha256(CONTROL_QUALITY),
            },
            "launcher_commit": "cb875a0",
            "launcher_sha256": sha256(STUDY / "run-direct-read-control"),
        },
        "activation": activations,
        "rollout_integrity": {
            "checked_files": len(rollout_integrity),
            "hash_mismatches": mismatches,
            "files": list(rollout_integrity.values()),
        },
        "groups": {
            "direct_sequential": direct,
            "normal_work_leaf": normal,
            "direct_read_work_leaf": control,
        },
        "comparisons": {
            "direct_read_minus_normal_work_leaf": comparison,
            "full_quality_direct_read_minus_normal_work_leaf": full_comparison,
            "direct_sequential_minus_direct_read_work_leaf": {
                "raw_tokens": direct_raw - direct_read_raw,
                "uncached_tokens": direct_uncached - direct_read_uncached,
                "direct_read_raw_reduction_percent": (direct_raw - direct_read_raw)
                / direct_raw
                * 100,
                "direct_read_uncached_reduction_percent": (
                    direct_uncached - direct_read_uncached
                )
                / direct_uncached
                * 100,
                "usage_changes": direct["mean_usage_changes"] - control["mean_usage_changes"],
                "input_context_per_usage_change": direct[
                    "mean_input_context_per_usage_change"
                ]
                - control["mean_input_context_per_usage_change"],
            },
        },
        "interpretation": {
            "supported": "Mediated reads reduce uncached context and increase context efficiency in this sample.",
            "raw_limit": "The read route explains only a minority of the raw-token gap; its all-run sample allocation is 9.38% and its full-quality subset allocation is larger but imprecise.",
            "remaining": "Direct-read Work Leaf still has far fewer usage changes than direct sequential Codex, so a separate Work Leaf mechanism causes most cached-input savings.",
            "next_hypothesis": "Immediate interruption after complete orchestrator directives is the next isolated candidate; command-output compaction and review or linearization alone lack enough observed magnitude.",
            "statistical_limit": "The control has three runs and the normal reference has six. Exact permutation results and ranges are descriptive, not a population estimate.",
        },
    }


def main():
    output = STUDY / "control-evidence.json"
    output.write_text(json.dumps(build_evidence(), indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(output)


if __name__ == "__main__":
    main()
