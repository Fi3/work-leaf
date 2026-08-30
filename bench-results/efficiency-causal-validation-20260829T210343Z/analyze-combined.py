#!/usr/bin/env python3

import hashlib
import importlib.util
import json
import re
from collections import Counter
from pathlib import Path


STUDY = Path(__file__).resolve().parent
ROOT = STUDY.parents[1]
CONTROL_IDS = (
    "combined-control-001",
    "combined-control-002",
    "combined-control-003",
)
QUALITY = STUDY / "combined-quality.json"
DECOMPOSITION = STUDY / "decomposition-evidence.json"
DIRECT_READ = STUDY / "control-evidence.json"
CONTINUED_RESPONSE = STUDY / "continued-response-evidence.json"
CONTROL_MANIFEST = STUDY / "infrastructure/combined-control-manifest.json"
LAUNCHER = STUDY / "run-combined-control"
USAGE_KEYS = (
    "input_tokens",
    "cached_input_tokens",
    "uncached_input_tokens",
    "output_tokens",
    "reasoning_output_tokens",
    "raw_input_plus_output",
    "uncached_input_plus_output",
)
APP_SERVER_USAGE_KEYS = (
    "inputTokens",
    "cachedInputTokens",
    "outputTokens",
    "reasoningOutputTokens",
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


def raw_values(runs):
    return [run["usage"]["raw_input_plus_output"] for run in runs]


def relative(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def load_module(path: Path, name: str):
    specification = importlib.util.spec_from_file_location(name, path)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load analysis module: {path}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def artifact_path(run_id: str) -> Path:
    return STUDY / "runs" / run_id / f"{run_id}-three-feature-bench-artifacts"


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


def usage_from(value):
    return {key: value[key] for key in USAGE_KEYS}


def directive_kind(text: str) -> str:
    match = re.search(r"(?m)^@work-leaf\s+([a-z-]+)", text)
    return match.group(1) if match else "unknown"


def app_server_incremental_usage_reconciles(server_values):
    previous = {}
    checked = 0
    failures = []
    derived_total_token_anomalies = []
    for value in server_values:
        if value.get("method") != "thread/tokenUsage/updated":
            continue
        params = value.get("params") or {}
        thread_id = params.get("threadId")
        token_usage = params.get("tokenUsage") or {}
        total = token_usage.get("total") or {}
        last = token_usage.get("last") or {}
        if not thread_id or not all(key in total and key in last for key in APP_SERVER_USAGE_KEYS):
            failures.append({"thread_id": thread_id, "reason": "missing total or last usage"})
            continue
        prior = previous.get(thread_id, {key: 0 for key in APP_SERVER_USAGE_KEYS})
        mismatched = {
            key: {"prior": prior[key], "last": last[key], "total": total[key]}
            for key in APP_SERVER_USAGE_KEYS
            if prior[key] + last[key] != total[key]
        }
        if mismatched:
            failures.append({"thread_id": thread_id, "mismatched": mismatched})
        expected_last_total = last["inputTokens"] + last["outputTokens"]
        if last.get("totalTokens") != expected_last_total:
            derived_total_token_anomalies.append(
                {
                    "thread_id": thread_id,
                    "reported_last_total_tokens": last.get("totalTokens"),
                    "component_last_total_tokens": expected_last_total,
                    "authoritative_components_unchanged": all(
                        prior[key] == total[key] for key in APP_SERVER_USAGE_KEYS
                    ),
                }
            )
        previous[thread_id] = {key: total[key] for key in APP_SERVER_USAGE_KEYS}
        checked += 1
    return {
        "checked_updates": checked,
        "failures": failures,
        "derived_total_token_anomalies": derived_total_token_anomalies,
        "passed": checked > 0 and not failures,
    }


def activation_for(run_id: str, quality, manifest):
    artifact = artifact_path(run_id)
    observation = artifact / "observation"
    app_servers = sorted((observation / "app-server").iterdir())
    if len(app_servers) != 1:
        raise ValueError(f"{run_id} has {len(app_servers)} app-server captures")
    app_server = app_servers[0]
    client_values, client_errors = parse_json_lines(app_server / "client-to-server.raw")
    server_values, server_errors = parse_json_lines(app_server / "server-to-client.raw")
    decisions, decision_errors = parse_json_lines(app_server / "provider-usage-grace.jsonl")

    turn_threads = set()
    direct_prompt_threads = set()
    mediated_prompt_threads = set()
    for value in client_values:
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
            direct_prompt_threads.add(thread_id)
        if "You are not allowed to read files directly" in text:
            mediated_prompt_threads.add(thread_id)

    directive_text = {}
    mediated_reads = 0
    command_executions = 0
    direct_read_commands = 0
    for value in server_values:
        if value.get("method") != "item/completed":
            continue
        params = value.get("params") or {}
        item = params.get("item") or {}
        if item.get("type") == "commandExecution":
            command_executions += 1
            if any(
                action.get("type") == "read"
                for action in item.get("commandActions") or []
                if isinstance(action, dict)
            ):
                direct_read_commands += 1
        elif item.get("type") == "agentMessage":
            text = item.get("text") or ""
            turn_id = params.get("turnId")
            if turn_id and "@work-leaf" in text:
                directive_text.setdefault(turn_id, text)
            mediated_reads += sum(
                line.startswith("@work-leaf read ") for line in text.splitlines()
            )

    outcomes = Counter(row.get("outcome") for row in decisions)
    continued = outcomes["forwarded-after-resumed-output-usage"]
    timeouts = outcomes["forwarded-after-timeout"]
    by_directive = Counter()
    for row in decisions:
        if row.get("outcome") not in {
            "forwarded-after-resumed-output-usage",
            "forwarded-after-timeout",
        }:
            continue
        by_directive[
            f"{directive_kind(directive_text.get(row.get('turn_id'), ''))}:{row['outcome']}"
        ] += 1

    starts = []
    for path in sorted((observation / "invocations").glob("*/start.json")):
        row = load(path)
        if row.get("primary") is True:
            starts.append(row)
    primary_app_servers = [row for row in starts if row.get("capture_kind") == "app-server"]
    report = load(artifact / "report.json")
    analysis = load(observation / "analysis.json")
    admission = load(STUDY / "runs" / run_id / "admission.json")
    observer_config = load(observation / "observer-config.json")
    report_usage = usage_from(report["total_workflow_usage"])
    analysis_usage = usage_from(analysis["usage_scopes"]["total_workflow"])
    expected_usage = {
        key: analysis_usage[key]
        for key in (
            "input_tokens",
            "cached_input_tokens",
            "output_tokens",
            "reasoning_output_tokens",
        )
    }
    exact_stratum = analysis.get("model_strata") == [
        {
            "model": "gpt-5.5",
            "effort": "xhigh",
            "thread_count": 8,
            "primary_threads": 8,
            "visible_threads": 7,
            "descendant_threads": 0,
            "usage": expected_usage,
        }
    ]
    recursive_log = artifact / "recursive-codex-attempts.log"
    interrupt_bytes_preserved = (
        (app_server / "client-to-server.raw").read_bytes()
        == (app_server / "client-to-server.forwarded.raw").read_bytes()
    )
    decision_policies = {row.get("output_resume_policy") for row in decisions}
    known_outcomes = {
        "forwarded-after-exact-usage",
        "forwarded-after-resumed-output-usage",
        "forwarded-after-turn-completed",
        "forwarded-after-timeout",
        "not-eligible",
    }
    exact_accounting = (
        analysis.get("capture_complete") is True
        and analysis.get("errors") == []
        and len(analysis.get("threads", [])) == 8
        and exact_stratum
        and report_usage == analysis_usage
    )
    primary_policy_ok = (
        len(primary_app_servers) == 1
        and primary_app_servers[0].get("provider_usage_grace_ms") == 120000
        and primary_app_servers[0].get("provider_usage_grace_output_resume")
        == "wait-for-usage"
    )
    incremental_reconciliation = app_server_incremental_usage_reconciles(server_values)
    activation = {
        "id": run_id,
        "turn_start_threads": len(turn_threads),
        "direct_prompt_threads": len(direct_prompt_threads),
        "mediated_prompt_threads": len(mediated_prompt_threads),
        "mediated_read_directives": mediated_reads,
        "command_executions": command_executions,
        "direct_read_commands": direct_read_commands,
        "decision_count": len(decisions),
        "decision_outcomes": dict(sorted(outcomes.items())),
        "decision_policies": sorted(str(value) for value in decision_policies),
        "continued_response_count": continued,
        "timeout_count": timeouts,
        "continued_wait_ms": [
            row["waited_ms"]
            for row in decisions
            if row.get("outcome") == "forwarded-after-resumed-output-usage"
        ],
        "affected_directives": dict(sorted(by_directive.items())),
        "json_line_parse_errors": client_errors + server_errors + decision_errors,
        "workflow_result": report["workflow_result"],
        "read_permission_mode": report["read_permission_mode"],
        "model": report["agent_model"],
        "reasoning_effort": report["agent_reasoning_effort"],
        "completed_features": quality["completed_features"],
        "feature_checks": quality["checks"],
        "exact_accounting": exact_accounting,
        "interrupt_bytes_preserved": interrupt_bytes_preserved,
        "primary_observer_policy_ok": primary_policy_ok,
        "app_server_incremental_usage_reconciles": incremental_reconciliation["passed"],
        "app_server_usage_reconciliation": incremental_reconciliation,
        "recursive_provider_attempt_bytes": recursive_log.stat().st_size,
        "observer_sha256": observer_config["observer_sha256"],
        "admission": admission,
        "fully_activated": continued > 0 and timeouts == 0,
    }
    activation["passed"] = (
        activation["turn_start_threads"] == 8
        and activation["direct_prompt_threads"] == 7
        and activation["mediated_prompt_threads"] == 0
        and activation["mediated_read_directives"] == 0
        and activation["direct_read_commands"] > 0
        and activation["continued_response_count"] > 0
        and activation["timeout_count"] == 0
        and activation["json_line_parse_errors"] == 0
        and set(activation["decision_outcomes"]) <= known_outcomes
        and activation["decision_policies"] == ["wait-for-usage"]
        and activation["workflow_result"] == "pass"
        and activation["read_permission_mode"]
        == "direct agent file reads enabled (--no-read-permission)"
        and activation["model"] == "gpt-5.5"
        and activation["reasoning_effort"] == "xhigh"
        and activation["exact_accounting"]
        and activation["interrupt_bytes_preserved"]
        and activation["primary_observer_policy_ok"]
        and activation["app_server_incremental_usage_reconciles"]
        and activation["recursive_provider_attempt_bytes"] == 0
        and activation["observer_sha256"] == manifest["binaries"]["bench-observer"]
        and admission["condition"] == "work-leaf-direct-read-continued-response"
        and admission["observer_sha256"] == manifest["binaries"]["bench-observer"]
    )
    return activation


def factorial_for(key, direct, normal, direct_read, continued, combined):
    direct_mean = direct["mean_usage"][key]
    normal_mean = normal["mean_usage"][key]
    read_mean = direct_read["mean_usage"][key]
    continued_mean = continued["mean_usage"][key]
    combined_mean = combined["mean_usage"][key]
    endpoint_gap = direct_mean - normal_mean
    return {
        "direct_sequential_mean": direct_mean,
        "normal_work_leaf_mean": normal_mean,
        "direct_read_mean": read_mean,
        "continued_response_mean": continued_mean,
        "combined_mean": combined_mean,
        "direct_read_minus_normal": read_mean - normal_mean,
        "continued_response_minus_normal": continued_mean - normal_mean,
        "direct_read_under_continued_response": combined_mean - continued_mean,
        "continued_response_under_direct_read": combined_mean - read_mean,
        "interaction": combined_mean - read_mean - continued_mean + normal_mean,
        "combined_minus_normal": combined_mean - normal_mean,
        "endpoint_gap": endpoint_gap,
        "combined_fraction_of_endpoint_gap_percent": (combined_mean - normal_mean)
        / endpoint_gap
        * 100,
    }


def review_rounds_for_direct(run):
    artifact = ROOT / run["report"]
    files = sorted(artifact.parent.glob("review-results/*.review"))
    rounds = []
    for path in files:
        match = re.search(r"(?m)^feature=(\d+)\s*$.*?^rounds=(\d+)\s*$.*?^result=clean\s*$", path.read_text(encoding="utf-8"), re.DOTALL)
        if match is None:
            raise ValueError(f"cannot parse clean direct review result: {path}")
        rounds.append(int(match.group(2)))
    if len(rounds) != 3:
        raise ValueError(f"{run['id']} has {len(rounds)} direct review results")
    return sum(rounds)


def review_rounds_for_work_leaf(run):
    report = ROOT / run["report"]
    state = load(report.parent / "final-state.json")
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
        raise ValueError(f"{run['id']} has {len(rounds)} Work Leaf review results")
    return sum(rounds)


def combined_review_rounds(run_id):
    report = artifact_path(run_id) / "report.json"
    return review_rounds_for_work_leaf({"id": run_id, "report": relative(report)})


def direct_rollout_reconciliation(direct_runs):
    rows = []
    for run in direct_runs:
        analysis_path = ROOT / run["analysis"]
        analysis = load(analysis_path)
        by_thread = {thread["thread_id"]: thread["usage"] for thread in analysis["threads"]}
        metadata, errors = parse_json_lines(analysis_path.parent / "rollout-metadata.jsonl")
        mismatches = []
        for row in metadata:
            expected = by_thread.get(row["thread_id"])
            actual = row["usage"]
            if expected is None or any(
                expected[key] != actual[key]
                for key in (
                    "input_tokens",
                    "cached_input_tokens",
                    "output_tokens",
                    "reasoning_output_tokens",
                )
            ):
                mismatches.append(row["thread_id"])
        rows.append(
            {
                "id": run["id"],
                "threads": len(metadata),
                "json_line_parse_errors": errors,
                "mismatched_threads": mismatches,
                "passed": errors == 0 and len(metadata) == len(by_thread) and not mismatches,
            }
        )
    return rows


def mean_activity(runs):
    keys = runs[0]["workflow_activity"].keys()
    return {key: mean([run["workflow_activity"][key] for run in runs]) for key in keys}


def rollout_action_rows(runs, sessions_root: Path):
    result = []
    for run in runs:
        metadata_path = (ROOT / run["analysis"]).parent / "rollout-metadata.jsonl"
        metadata, errors = parse_json_lines(metadata_path)
        counts = Counter()
        for row in metadata:
            source = sessions_root / row["source_relative_path"]
            with source.open(encoding="utf-8", errors="replace") as handle:
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
                        counts[payload.get("name") or "unnamed_tool"] += 1
                    elif payload_type == "web_search_call":
                        counts["web_search"] += 1
        result.append(
            {
                "id": run["id"],
                "metadata_parse_errors": errors,
                "actions": dict(sorted(counts.items())),
            }
        )
    return result


def mean_actions(rows):
    keys = sorted({key for row in rows for key in row["actions"]})
    return {
        key: mean([row["actions"].get(key, 0) for row in rows]) for key in keys
    }


def work_leaf_directive_counts(run_id):
    state = load(artifact_path(run_id) / "final-state.json")
    lines = [line for session in state["snapshot"]["sessions"] for line in session["lines"]]
    return {
        "edit": sum(line.startswith("@work-leaf edit ") for line in lines),
        "locks_run": sum(line.startswith("@work-leaf locks run ") for line in lines),
        "read": sum(line.startswith("@work-leaf read ") for line in lines),
    }


def residual_decomposition(direct, combined, direct_runs, combined_runs, stage_difference):
    direct_changes = direct["mean_usage_changes"]
    combined_changes = combined["mean_usage_changes"]
    direct_context = direct["mean_input_context_per_usage_change"]
    combined_context = combined["mean_input_context_per_usage_change"]
    count_contribution = (direct_changes - combined_changes) * (
        direct_context + combined_context
    ) / 2
    context_contribution = (direct_context - combined_context) * (
        direct_changes + combined_changes
    ) / 2
    input_gap = direct["mean_usage"]["input_tokens"] - combined["mean_usage"]["input_tokens"]
    return {
        "token_class_gap": {
            key: direct["mean_usage"][key] - combined["mean_usage"][key]
            for key in USAGE_KEYS
        },
        "usage_changes": {
            "direct_mean": direct_changes,
            "combined_mean": combined_changes,
            "combined_reduction_percent": (direct_changes - combined_changes)
            / direct_changes
            * 100,
        },
        "context_per_usage_change": {
            "direct_mean": direct_context,
            "combined_mean": combined_context,
            "combined_reduction_percent": (direct_context - combined_context)
            / direct_context
            * 100,
        },
        "input_gap_factorization": {
            "fewer_usage_changes_tokens": count_contribution,
            "smaller_context_tokens": context_contribution,
            "sum_tokens": count_contribution + context_contribution,
            "observed_input_gap": input_gap,
            "fewer_usage_changes_share_percent": count_contribution / input_gap * 100,
            "smaller_context_share_percent": context_contribution / input_gap * 100,
        },
        "direct_minus_combined_stage_usage": stage_difference(combined_runs, direct_runs),
    }


def build_evidence():
    base = load_module(STUDY / "analyze-control.py", "combined_control_common")
    decompose = load_module(STUDY / "decompose.py", "combined_decompose")
    continued_module = load_module(
        STUDY / "analyze-continued-response.py", "combined_stage_common"
    )
    decomposition = load(DECOMPOSITION)
    direct_read_evidence = load(DIRECT_READ)
    continued_evidence = load(CONTINUED_RESPONSE)
    quality = load(QUALITY)
    quality_by_id = {row["id"]: row for row in quality["runs"]}
    manifest = load(CONTROL_MANIFEST)
    rollout_integrity = {}
    combined_runs = []
    activations = []
    for run_id in CONTROL_IDS:
        analysis_path = artifact_path(run_id) / "observation/analysis.json"
        combined_runs.append(
            decompose.analyze_run(
                run_id,
                "work_leaf",
                analysis_path,
                quality_by_id[run_id]["checks"],
                rollout_integrity,
            )
        )
        combined_runs[-1]["report"] = relative(artifact_path(run_id) / "report.json")
        activations.append(activation_for(run_id, quality_by_id[run_id], manifest))

    current = decomposition["cohorts"]["current_detailed_6_by_6"]["runs"]
    direct_runs = [run for run in current if run["group"] == "direct"]
    normal_runs = [run for run in current if run["group"] == "work_leaf"]
    direct = base.summarize_group(direct_runs)
    normal = base.summarize_group(normal_runs)
    normal_missing_responses = sum(
        int(run.get("unresolved_provider_responses", 0)) for run in normal_runs
    )
    direct_read = direct_read_evidence["groups"]["direct_read_work_leaf"]
    continued = continued_evidence["groups"]["continued_response_work_leaf"]
    combined = base.summarize_group(combined_runs)
    combined_comparison = base.compare_groups(direct, normal, combined)

    full_direct = base.full_quality_summary(direct_runs)
    full_normal = base.full_quality_summary(normal_runs)
    full_combined = base.full_quality_summary(combined_runs)
    full_comparison = base.compare_groups(full_direct, full_normal, full_combined)

    direct_reconciliation = direct_rollout_reconciliation(direct_runs)
    mismatches = [row["source"] for row in rollout_integrity.values() if not row["matches"]]
    infrastructure_valid = (
        not mismatches
        and all(row["passed"] for row in activations)
        and all(row["passed"] for row in direct_reconciliation)
    )

    direct_rounds = [review_rounds_for_direct(run) for run in direct_runs]
    normal_rounds = [review_rounds_for_work_leaf(run) for run in normal_runs]
    combined_rounds = [combined_review_rounds(run_id) for run_id in CONTROL_IDS]

    same_cli_runs = []
    for run in direct_runs:
        report = load(ROOT / run["report"])
        if "codex-cli 0.150.1" in report["agent_cli_version"]:
            same_cli_runs.append(run)
    same_cli_direct = base.summarize_group(same_cli_runs)
    same_cli_full_combined = base.full_quality_summary(combined_runs)

    direct_action_rows = rollout_action_rows(direct_runs, decompose.SESSIONS)
    combined_action_rows = rollout_action_rows(combined_runs, decompose.SESSIONS)
    if any(row["metadata_parse_errors"] for row in direct_action_rows + combined_action_rows):
        infrastructure_valid = False
    direct_action_mean = mean_actions(direct_action_rows)
    combined_action_mean = mean_actions(combined_action_rows)
    directive_rows = [
        {"id": run_id, **work_leaf_directive_counts(run_id)} for run_id in CONTROL_IDS
    ]
    structured_edit_rows = []
    for run in combined_runs:
        structured = load(ROOT / run["analysis"])["mechanisms"]["structured_edits"]
        structured_edit_rows.append(
            {
                "id": run["id"],
                "submissions": structured["submissions"],
                "duplicate_submissions": structured["duplicate_submissions"],
                "acknowledgements": structured["acknowledgements"],
                "rejections": structured["rejections"],
            }
        )
    structured_edit_mean = mean([row["submissions"] for row in structured_edit_rows])

    factorial = {
        key: factorial_for(key, direct, normal, direct_read, continued, combined)
        for key in USAGE_KEYS
    }
    for value in factorial.values():
        value["measurement"] = "recorded normal Work Leaf lower-bound scenario"
    raw_factorial = factorial["raw_input_plus_output"]
    uncached_factorial = factorial["uncached_input_plus_output"]

    return {
        "schema_version": 1,
        "study": STUDY.name,
        "status": (
            "complete_controls_with_bounded_normal_endpoint"
            if infrastructure_valid
            else "invalid"
        ),
        "normal_endpoint_accounting": {
            "measurement": "bounded",
            "unresolved_provider_responses": normal_missing_responses,
            "recorded_mean_raw_tokens": normal["mean_usage"][
                "raw_input_plus_output"
            ],
            "factorial_values_use": "recorded lower-bound scenario",
        },
        "references": {
            "decomposition": {"path": relative(DECOMPOSITION), "sha256": sha256(DECOMPOSITION)},
            "direct_read": {"path": relative(DIRECT_READ), "sha256": sha256(DIRECT_READ)},
            "continued_response": {
                "path": relative(CONTINUED_RESPONSE),
                "sha256": sha256(CONTINUED_RESPONSE),
            },
            "quality": {"path": relative(QUALITY), "sha256": sha256(QUALITY)},
            "manifest": {
                "path": relative(CONTROL_MANIFEST),
                "sha256": sha256(CONTROL_MANIFEST),
            },
            "launcher": {"path": relative(LAUNCHER), "sha256": sha256(LAUNCHER)},
        },
        "activation": activations,
        "rollout_integrity": {
            "checked_files": len(rollout_integrity),
            "hash_mismatches": mismatches,
            "files": list(rollout_integrity.values()),
        },
        "direct_rollout_reconciliation": direct_reconciliation,
        "groups": {
            "direct_sequential": direct,
            "normal_work_leaf": normal,
            "direct_read_work_leaf": direct_read,
            "continued_response_work_leaf": continued,
            "combined_work_leaf": combined,
        },
        "factorial": factorial,
        "comparisons": {
            "combined_minus_normal_work_leaf": combined_comparison,
            "full_quality_combined_minus_normal_work_leaf": full_comparison,
            "combined_minus_normal_stage_usage": continued_module.stage_difference(
                normal_runs, combined_runs
            ),
            "mean_workflow_activity": {
                "direct_sequential": mean_activity(direct_runs),
                "normal_work_leaf": mean_activity(normal_runs),
                "combined_work_leaf": mean_activity(combined_runs),
            },
        },
        "residual_decomposition": residual_decomposition(
            direct,
            combined,
            direct_runs,
            combined_runs,
            continued_module.stage_difference,
        ),
        "counterchecks": {
            "same_cli_direct_vs_combined": {
                "cli": "codex-cli 0.150.1",
                "direct_runs": len(same_cli_runs),
                "direct_features": same_cli_direct["completed_features"],
                "direct_mean_raw": same_cli_direct["mean_usage"]["raw_input_plus_output"],
                "combined_runs": combined["runs"],
                "combined_features": combined["completed_features"],
                "combined_mean_raw": combined["mean_usage"]["raw_input_plus_output"],
                "combined_raw_reduction_percent": (
                    same_cli_direct["mean_usage"]["raw_input_plus_output"]
                    - combined["mean_usage"]["raw_input_plus_output"]
                )
                / same_cli_direct["mean_usage"]["raw_input_plus_output"]
                * 100,
                "full_quality_combined_mean_raw": same_cli_full_combined["mean_usage"][
                    "raw_input_plus_output"
                ],
            },
            "review_rounds": {
                "direct": direct_rounds,
                "direct_mean": mean(direct_rounds),
                "normal_work_leaf": normal_rounds,
                "normal_work_leaf_mean": mean(normal_rounds),
                "combined": combined_rounds,
                "combined_mean": mean(combined_rounds),
            },
            "candidate_size": {
                "direct_changed_lines": [
                    load(ROOT / run["report"])["changed_lines_total"] for run in direct_runs
                ],
                "combined_changed_lines": [
                    load(artifact_path(run_id) / "report.json")["changed_lines_total"]
                    for run_id in CONTROL_IDS
                ],
            },
            "provider_actions": {
                "direct_rows": direct_action_rows,
                "direct_mean": direct_action_mean,
                "combined_rows": combined_action_rows,
                "combined_mean": combined_action_mean,
                "combined_directive_rows": directive_rows,
                "combined_structured_edit_rows": structured_edit_rows,
                "combined_structured_edit_mean": structured_edit_mean,
                "direct_total_write_submission_mean": direct_action_mean.get(
                    "apply_patch", 0
                ),
                "combined_total_write_submission_mean": structured_edit_mean
                + combined_action_mean.get("apply_patch", 0),
            },
            "raw_rank_separation": {
                "direct_vs_normal": base.exact_permutation_greater(
                    raw_values(normal_runs), raw_values(direct_runs)
                ),
                "direct_vs_combined": base.exact_permutation_greater(
                    raw_values(combined_runs), raw_values(direct_runs)
                ),
                "same_cli_direct_vs_combined": base.exact_permutation_greater(
                    raw_values(combined_runs), raw_values(same_cli_runs)
                ),
            },
        },
        "causal_summary": {
            "endpoint_raw_gap_tokens": raw_factorial["endpoint_gap"],
            "combined_raw_movement_tokens": raw_factorial["combined_minus_normal"],
            "combined_raw_fraction_of_endpoint_gap_percent": raw_factorial[
                "combined_fraction_of_endpoint_gap_percent"
            ],
            "combined_uncached_movement_tokens": uncached_factorial[
                "combined_minus_normal"
            ],
            "combined_uncached_fraction_of_endpoint_gap_percent": uncached_factorial[
                "combined_fraction_of_endpoint_gap_percent"
            ],
            "raw_interaction_tokens": raw_factorial["interaction"],
            "separate_effects_are_additive": False,
            "why_not_additive": (
                "The combined control has a large negative raw-token interaction. Direct reads "
                "make post-directive continuation much more common, and the two changes replace "
                "some of the same later provider cycles rather than adding independent costs."
            ),
            "residual_raw_gap_after_combined_control": (
                direct["mean_usage"]["raw_input_plus_output"]
                - combined["mean_usage"]["raw_input_plus_output"]
            ),
            "residual_raw_reduction_percent_vs_direct": (
                direct["mean_usage"]["raw_input_plus_output"]
                - combined["mean_usage"]["raw_input_plus_output"]
            )
            / direct["mean_usage"]["raw_input_plus_output"]
            * 100,
            "quality": {
                "direct": direct["completed_features"],
                "normal_work_leaf": normal["completed_features"],
                "combined_work_leaf": combined["completed_features"],
            },
        },
    }


def main():
    evidence = build_evidence()
    output = STUDY / "combined-evidence.json"
    output.write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(output)


if __name__ == "__main__":
    main()
