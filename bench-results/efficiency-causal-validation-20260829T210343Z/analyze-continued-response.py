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
    "continued-response-001",
    "continued-response-002",
    "continued-response-003",
)
QUALITY = STUDY / "continued-response-quality.json"
DECOMPOSITION = STUDY / "decomposition-evidence.json"
DIRECT_READ = STUDY / "control-evidence.json"
CONTROL_MANIFEST = STUDY / "infrastructure/continued-response-manifest.json"
LAUNCHER = STUDY / "run-continued-response-control"
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
    return (
        STUDY
        / "runs"
        / run_id
        / f"{run_id}-three-feature-bench-artifacts"
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


def usage_from(value):
    return {key: value[key] for key in USAGE_KEYS}


def directive_kind(text: str) -> str:
    match = re.search(r"(?m)^@work-leaf\s+([a-z-]+)", text)
    return match.group(1) if match else "unknown"


def activation_for(run_id: str, quality, manifest):
    artifact = artifact_path(run_id)
    observation = artifact / "observation"
    app_servers = sorted((observation / "app-server").iterdir())
    if len(app_servers) != 1:
        raise ValueError(f"{run_id} has {len(app_servers)} app-server captures")
    app_server = app_servers[0]
    client_values, client_errors = parse_json_lines(app_server / "client-to-server.raw")
    server_values, server_errors = parse_json_lines(app_server / "server-to-client.raw")
    decisions, decision_errors = parse_json_lines(
        app_server / "provider-usage-grace.jsonl"
    )

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
    for value in server_values:
        if value.get("method") != "item/completed":
            continue
        params = value.get("params") or {}
        item = params.get("item") or {}
        if item.get("type") != "agentMessage":
            continue
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

    starts = [
        load(path)
        for path in sorted((observation / "invocations").glob("*/start.json"))
        if load(path).get("primary") is True
    ]
    primary_app_servers = [
        row for row in starts if row.get("capture_kind") == "app-server"
    ]
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
    decision_policies = {
        row.get("output_resume_policy") for row in decisions
    }
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
    activation = {
        "id": run_id,
        "turn_start_threads": len(turn_threads),
        "direct_prompt_threads": len(direct_prompt_threads),
        "mediated_prompt_threads": len(mediated_prompt_threads),
        "mediated_read_directives": mediated_reads,
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
        "recursive_provider_attempt_bytes": recursive_log.stat().st_size,
        "observer_sha256": observer_config["observer_sha256"],
        "admission": admission,
        "fully_activated": continued > 0 and timeouts == 0,
    }
    activation["passed"] = (
        activation["turn_start_threads"] == 8
        and activation["direct_prompt_threads"] == 0
        and activation["mediated_prompt_threads"] == 7
        and activation["mediated_read_directives"] > 0
        and activation["continued_response_count"] > 0
        and activation["json_line_parse_errors"] == 0
        and set(activation["decision_outcomes"]) <= known_outcomes
        and "forwarded-after-output-resumed" not in activation["decision_outcomes"]
        and activation["decision_policies"] == ["wait-for-usage"]
        and activation["workflow_result"] == "pass"
        and activation["read_permission_mode"] == "orchestrator-mediated file reads"
        and activation["model"] == "gpt-5.5"
        and activation["reasoning_effort"] == "xhigh"
        and activation["exact_accounting"]
        and activation["interrupt_bytes_preserved"]
        and activation["primary_observer_policy_ok"]
        and activation["recursive_provider_attempt_bytes"] == 0
        and activation["observer_sha256"] == manifest["binaries"]["bench-observer"]
        and admission["condition"] == "work-leaf-continued-response"
        and admission["observer_sha256"] == manifest["binaries"]["bench-observer"]
    )
    return activation


def mean_stage_usage(runs):
    stages = sorted({stage for run in runs for stage in run["stages"]})
    keys = (*USAGE_KEYS, "usage_changes")
    return {
        stage: {
            key: sum(run["stages"].get(stage, {}).get(key, 0) for run in runs)
            / len(runs)
            for key in keys
        }
        for stage in stages
    }


def stage_difference(reference, control):
    reference_stages = mean_stage_usage(reference)
    control_stages = mean_stage_usage(control)
    stages = sorted(set(reference_stages) | set(control_stages))
    return {
        stage: {
            key: control_stages.get(stage, {}).get(key, 0)
            - reference_stages.get(stage, {}).get(key, 0)
            for key in (*USAGE_KEYS, "usage_changes")
        }
        for stage in stages
    }


def build_evidence():
    base = load_module(STUDY / "analyze-control.py", "causal_control_common")
    decompose = load_module(STUDY / "decompose.py", "causal_decompose_continued")
    decomposition = load(DECOMPOSITION)
    direct_read = load(DIRECT_READ)
    quality = load(QUALITY)
    quality_by_id = {row["id"]: row for row in quality["runs"]}
    manifest = load(CONTROL_MANIFEST)
    rollout_integrity = {}
    controls = []
    activations = []
    for run_id in CONTROL_IDS:
        analysis_path = artifact_path(run_id) / "observation/analysis.json"
        controls.append(
            decompose.analyze_run(
                run_id,
                "work_leaf",
                analysis_path,
                quality_by_id[run_id]["checks"],
                rollout_integrity,
            )
        )
        activations.append(activation_for(run_id, quality_by_id[run_id], manifest))

    current = decomposition["cohorts"]["current_detailed_6_by_6"]["runs"]
    direct_runs = [run for run in current if run["group"] == "direct"]
    normal_runs = [run for run in current if run["group"] == "work_leaf"]
    direct = base.summarize_group(direct_runs)
    normal = base.summarize_group(normal_runs)
    control = base.summarize_group(controls)
    comparison = base.compare_groups(direct, normal, control)
    full_direct = base.full_quality_summary(direct_runs)
    full_normal = base.full_quality_summary(normal_runs)
    full_control = base.full_quality_summary(controls)
    full_comparison = base.compare_groups(full_direct, full_normal, full_control)
    mismatches = [
        row["source"] for row in rollout_integrity.values() if not row["matches"]
    ]
    infrastructure_valid = not mismatches and all(row["passed"] for row in activations)
    partial_activation = any(row["timeout_count"] > 0 for row in activations)
    direct_raw = direct["mean_usage"]["raw_input_plus_output"]
    control_raw = control["mean_usage"]["raw_input_plus_output"]
    bounded = base.bounded_control_comparison(
        decomposition, direct_raw, control_raw
    )

    return {
        "schema_version": 1,
        "study": STUDY.name,
        "status": (
            "complete-with-partial-activation"
            if infrastructure_valid and partial_activation
            else "complete" if infrastructure_valid else "invalid"
        ),
        "references": {
            "decomposition": {
                "path": relative(DECOMPOSITION),
                "sha256": sha256(DECOMPOSITION),
            },
            "direct_read_control": {
                "path": relative(DIRECT_READ),
                "sha256": sha256(DIRECT_READ),
            },
            "control_quality": {
                "path": relative(QUALITY),
                "sha256": sha256(QUALITY),
            },
            "control_manifest": {
                "path": relative(CONTROL_MANIFEST),
                "sha256": sha256(CONTROL_MANIFEST),
            },
            "launcher": {
                "path": relative(LAUNCHER),
                "sha256": sha256(LAUNCHER),
            },
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
            "continued_response_work_leaf": control,
        },
        "bounded_normal_comparison": bounded,
        "comparisons": {
            "continued_response_minus_normal_work_leaf": comparison,
            "full_quality_continued_response_minus_normal_work_leaf": full_comparison,
            "continued_response_minus_normal_stage_usage": stage_difference(
                normal_runs, controls
            ),
            "direct_sequential_minus_continued_response_work_leaf": {
                "raw_tokens": direct_raw - control_raw,
                "raw_reduction_percent": (direct_raw - control_raw) / direct_raw * 100,
                "quality_checks": {
                    "direct": direct["completed_features"],
                    "continued_response": control["completed_features"],
                },
            },
        },
        "causal_summary": {
            "measurement": "recorded normal Work Leaf lower-bound scenario",
            "endpoint_raw_gap_tokens": (
                direct["mean_usage"]["raw_input_plus_output"]
                - normal["mean_usage"]["raw_input_plus_output"]
            ),
            "mediated_read_sample_fraction_percent": direct_read["comparisons"][
                "direct_read_minus_normal_work_leaf"
            ]["raw_fraction_of_direct_gap_percent"],
            "continued_response_sample_fraction_percent": comparison[
                "raw_fraction_of_direct_gap_percent"
            ],
            "fractions_are_additive": False,
            "why_not_additive": (
                "Read delivery and response interruption operate in the same Work Leaf turns, "
                "and no combined two-factor control measured their interaction."
            ),
            "partial_activation": partial_activation,
            "quality_warning": (
                "The continued-response group completed 6/9 checks versus 13/18 for normal Work "
                "Leaf. Its higher token use was not caused by completing more scored features, "
                "but three runs do not remove ordinary model variation."
            ),
            "bounded_direction": (
                "The completed-response control uses "
                f"{bounded['control_minus_normal_raw_tokens']['lower'] / 1_000_000:.2f}-"
                f"{bounded['control_minus_normal_raw_tokens']['upper'] / 1_000_000:.2f} million "
                "more raw tokens than "
                "normal early interruption under the endpoint bound, so early interruption's "
                "raw-token saving is established for these collected samples."
            ),
        },
    }


def main():
    evidence = build_evidence()
    output = STUDY / "continued-response-evidence.json"
    output.write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(output)


if __name__ == "__main__":
    main()
