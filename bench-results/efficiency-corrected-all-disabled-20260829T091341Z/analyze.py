#!/usr/bin/env python3
"""Verify and summarize the corrected all-disabled control cohort."""

from __future__ import annotations

import hashlib
import json
import statistics
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Sequence


STUDY_DIR = Path(__file__).resolve().parent
REPO_ROOT = STUDY_DIR.parents[1]
PRIOR_EVIDENCE = (
    REPO_ROOT / "bench-results/efficiency-points8-9-20260828T145556Z/evidence.json"
)
RUN_IDS = tuple(f"corrected-all-disabled-00{number}" for number in range(1, 4))
BASE_COMMIT = "c92a0b7060a36eac6db2d869b85e589a7a9480f9"
MODEL = "gpt-5.5"
EFFORT = "xhigh"
TASK_LIST_SHA256 = "45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a"
MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE = 400_000
EFFECTIVE_CONTEXT_WINDOW = 258_400
DOCUMENTED_MAXIMUM_OUTPUT = 128_000
EXPECTED_CONTROLS = {
    "WORK_LEAF_EXPERIMENT_CHANGED_REPEAT_DELIVERY": "full",
    "WORK_LEAF_EXPERIMENT_REVIEW_PROVENANCE": "git-reconstruct",
    "WORK_LEAF_EXPERIMENT_UNCHANGED_REPEAT_DELIVERY": "full",
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


def mean_interval(intervals: Sequence[dict[str, int | float]]) -> dict[str, float]:
    require(bool(intervals), "cannot average an empty interval group")
    return {
        "lower": statistics.fmean(float(value["lower"]) for value in intervals),
        "upper": statistics.fmean(float(value["upper"]) for value in intervals),
    }


def difference_interval(
    left: dict[str, int | float], right: dict[str, int | float]
) -> dict[str, float]:
    return {
        "lower": float(left["lower"]) - float(right["upper"]),
        "upper": float(left["upper"]) - float(right["lower"]),
    }


def cap_audit(interrupted: int, prompt_bytes: int) -> dict[str, int]:
    require(interrupted >= 0 and prompt_bytes >= 0, "invalid interruption values")
    rounded = interrupted * MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE
    context_output_prompt = (
        interrupted * (EFFECTIVE_CONTEXT_WINDOW + DOCUMENTED_MAXIMUM_OUTPUT)
        + prompt_bytes
    )
    applied = max(rounded, context_output_prompt)
    return {
        "rounded_missing_raw_token_cap": rounded,
        "context_output_and_prompt_upper_bound": context_output_prompt,
        "applied_missing_raw_token_cap": applied,
        "cap_headroom": applied - context_output_prompt,
    }


def audit_app_server_session(client_path: Path, server_path: Path) -> dict[str, int]:
    starts: dict[str, tuple[str, int]] = {}
    interrupts: set[tuple[str, str]] = set()
    for row in read_jsonl(client_path):
        method = row.get("method")
        params = row.get("params", {})
        if method == "turn/start":
            request_id = str(row["id"])
            thread_id = params.get("threadId")
            prompt = params.get("input", [])
            require(isinstance(thread_id, str), "turn/start has no thread ID")
            require(isinstance(prompt, list), "turn/start has invalid input")
            prompt_bytes = len(
                json.dumps(prompt, ensure_ascii=False, separators=(",", ":")).encode()
            )
            require(request_id not in starts, f"duplicate turn/start ID: {request_id}")
            starts[request_id] = (thread_id, prompt_bytes)
        elif method == "turn/interrupt":
            identity = (params.get("threadId"), params.get("turnId"))
            require(all(isinstance(value, str) for value in identity), "invalid interrupt")
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
            require(isinstance(turn_id, str), "turn/start response has no turn ID")
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
            require(all(isinstance(value, str) for value in identity), "invalid outcome")
            require(status in {"completed", "interrupted"}, "invalid turn status")
            outcomes[identity] = status  # type: ignore[index]
        if row.get("method") == "thread/tokenUsage/updated":
            window = row.get("params", {}).get("tokenUsage", {}).get("modelContextWindow")
            if isinstance(window, int):
                context_windows.add(window)

    require(rpc_errors == 0, "app-server transcript contains an RPC error")
    require(set(started) == set(outcomes), "started and completed turns differ")
    interrupted = {identity for identity, status in outcomes.items() if status == "interrupted"}
    require(interrupts == interrupted, "interrupt requests and outcomes differ")
    require(context_windows == {EFFECTIVE_CONTEXT_WINDOW}, "unexpected context window")
    return {
        "started_turns": len(started),
        "completed_turns": sum(status == "completed" for status in outcomes.values()),
        "interrupted_turns": len(interrupted),
        "interrupted_prompt_json_bytes": sum(started[item] for item in interrupted),
    }


def audit_app_server(artifact: Path) -> dict[str, int]:
    sessions = sorted((artifact / "observation/app-server").glob("*"))
    require(bool(sessions), f"no app-server capture in {artifact}")
    audits = [
        audit_app_server_session(
            session / "client-to-server.raw", session / "server-to-client.raw"
        )
        for session in sessions
    ]
    return {
        key: sum(audit[key] for audit in audits)
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
        if line.startswith("WORK_LEAF_EXPERIMENT_") and "=" in line:
            name, value = line.split("=", 1)
            controls[name] = value
    return controls


def delivery_summary(mechanism: dict[str, Any]) -> dict[str, Any]:
    rows = mechanism["mechanisms"].get("counterfactuals", [])
    result = {}
    for hypothesis, label in (("H1", "unchanged_reread"), ("H2", "changed_reread")):
        selected = [row for row in rows if row.get("hypothesis") == hypothesis]
        statuses: dict[str, int] = {}
        for row in selected:
            status = str(row.get("status"))
            statuses[status] = statuses.get(status, 0) + 1
        result[label] = {
            "events": len(selected),
            "statuses": statuses,
            "full_current_events": statuses.get("full-current-delivery", 0),
            "actual_component_bytes": sum(
                int(row["actual_component_bytes"])
                for row in selected
                if isinstance(row.get("actual_component_bytes"), int)
            ),
        }
    return result


def review_summary(state: dict[str, Any]) -> dict[str, Any]:
    sessions = [
        session
        for session in state["snapshot"]["sessions"]
        if session["id"].startswith("review-")
    ]
    require(len(sessions) == 3, "expected exactly three review sessions")
    rows = []
    for session in sessions:
        lines = [str(line) for line in session["lines"]]
        git_commands = [
            line
            for line in lines
            if line.startswith("@work-leaf locks run") and "git " in line
        ]
        markers = [
            line
            for line in lines
            if line.strip().startswith(("NO_FINDINGS", "FINDINGS"))
        ]
        require(bool(git_commands), f"{session['id']} did not reconstruct from Git")
        require(bool(markers), f"{session['id']} has no review marker")
        for marker in markers:
            stripped = marker.strip()
            require("@work-leaf done" in stripped, f"{session['id']} marker lacks done")
            require(
                stripped.find("@work-leaf done") > 0,
                f"{session['id']} put done before the review marker",
            )
        require(markers[-1].strip().startswith("NO_FINDINGS"), "final review was not clean")
        rows.append(
            {
                "agent_id": session["id"],
                "mediated_git_commands": len(git_commands),
                "review_responses": len(markers),
                "finding_responses": sum(
                    marker.strip().startswith("FINDINGS") for marker in markers
                ),
                "final_marker": "NO_FINDINGS",
                "marker_before_done": True,
            }
        )
    return {"sessions": rows, "all_reconstructed_from_git": True}


def observer_error_is_explained(error: str) -> bool:
    return error.startswith(
        (
            "interrupted provider turn has no complete usage:",
            "invocation ",
            "rollout: ",
            "controller usage row ",
            "controller usage for ",
        )
    )


def usage_add(left: dict[str, int], right: dict[str, int]) -> dict[str, int]:
    return {key: int(left.get(key, 0)) + int(right.get(key, 0)) for key in left | right}


def audit_controller_mapping(analysis: dict[str, Any]) -> dict[str, Any]:
    rows = analysis.get("controller_usage_reconciliation", [])
    missing = [
        row
        for row in rows
        if row.get("controller_streamed_usage") is not None
        and not row.get("provider_thread_ids")
    ]
    mismatched = [
        row
        for row in rows
        if row.get("controller_streamed_usage") is not None
        and row.get("provider_largest_cumulative_usage") is not None
        and row.get("controller_matches_replay") is False
    ]
    covered = []
    for missing_row in missing:
        missing_usage = missing_row["controller_streamed_usage"]
        match = next(
            (
                row
                for row in mismatched
                if usage_add(row["controller_streamed_usage"], missing_usage)
                == row["provider_largest_cumulative_usage"]
            ),
            None,
        )
        require(
            match is not None,
            f"unaccounted controller usage for {missing_row['agent_id']}",
        )
        covered.append(
            {
                "controller_agent_id": missing_row["agent_id"],
                "provider_total_row": match["agent_id"],
                "usage_already_in_provider_total": True,
            }
        )
    return {"identity_mismatches": covered, "all_usage_covered": True}


def audit_rollout_mapping(artifact: Path) -> dict[str, Any]:
    audit = read_json(artifact / "observation/rollout-audit.json")
    unobserved = set(audit.get("unobserved_cwd_threads", []))
    outcomes: dict[str, Counter[str]] = defaultdict(Counter)
    usage_events: Counter[str] = Counter()
    for session in sorted((artifact / "observation/app-server").glob("*")):
        for row in read_jsonl(session / "server-to-client.raw"):
            if row.get("method") == "turn/completed":
                params = row.get("params", {})
                turn = params.get("turn", {})
                thread_id = params.get("threadId", turn.get("threadId"))
                status = turn.get("status", params.get("status"))
                if isinstance(thread_id, str) and isinstance(status, str):
                    outcomes[thread_id][status] += 1
            if row.get("method") == "thread/tokenUsage/updated":
                thread_id = row.get("params", {}).get("threadId")
                if isinstance(thread_id, str):
                    usage_events[thread_id] += 1
    rows = []
    for thread_id in sorted(unobserved):
        statuses = dict(outcomes[thread_id])
        require(statuses and set(statuses) == {"interrupted"}, "unmapped completed thread")
        require(usage_events[thread_id] == 0, "unmapped thread has unmerged usage")
        rows.append(
            {
                "thread_id": thread_id,
                "interrupted_turns": statuses["interrupted"],
                "usage_events": 0,
                "covered_by_interruption_cap": True,
            }
        )
    return {
        "unmapped_threads": rows,
        "all_unmapped_usage_covered": True,
    }


def audit_unfinished_invocations(artifact: Path) -> dict[str, Any]:
    unfinished = [
        row
        for row in read_jsonl(artifact / "observation/process-invocations.jsonl")
        if row.get("end") is None
    ]
    require(
        all(row.get("capture_kind") == "locked-command" for row in unfinished),
        "unfinished provider invocation",
    )
    return {
        "count": len(unfinished),
        "capture_kinds": sorted({str(row.get("capture_kind")) for row in unfinished}),
        "provider_invocations": 0,
    }


def artifact_for(run_id: str) -> Path:
    artifacts = sorted((STUDY_DIR / "runs" / run_id).glob("*-artifacts"))
    require(len(artifacts) == 1, f"expected one artifact for {run_id}")
    return artifacts[0]


def verify_run(run_id: str, quality: dict[str, Any]) -> dict[str, Any]:
    artifact = artifact_for(run_id)
    report_path = artifact / "report.json"
    analysis_path = artifact / "observation/analysis.json"
    mechanism_path = artifact / "observation/mechanism-summary.json"
    state_path = artifact / "final-state.json"
    report = read_json(report_path)
    analysis = read_json(analysis_path)
    mechanism = read_json(mechanism_path)
    scored = next((run for run in quality["runs"] if run["id"] == run_id), None)
    require(scored is not None, f"missing quality score for {run_id}")
    require(report["workflow_result"] == "pass", f"workflow failed: {run_id}")
    require(report["base_commit"] == BASE_COMMIT, f"wrong base: {run_id}")
    require(
        report["agent_model"] == MODEL and report["agent_reasoning_effort"] == EFFORT,
        f"wrong provider profile: {run_id}",
    )
    require(
        report["bench_mode"] == "work-leaf" and report["feature_schedule"] == "concurrent",
        f"wrong workflow: {run_id}",
    )
    require(report["review_completed"] == "yes", f"review incomplete: {run_id}")
    require(report["linearize_completed"] == "yes", f"linearization incomplete: {run_id}")
    for command in ("cargo fmt", "cargo clippy", "cargo test"):
        require(command in report["code_quality"], f"missing {command}: {run_id}")
    require(read_controls(artifact / "daemon-env.txt") == EXPECTED_CONTROLS, "wrong controls")
    require((artifact / "recursive-codex-attempts.log").stat().st_size == 0, "recursive call")
    require(report["measurement_status"] == "incomplete", "unexpected exact measurement")
    require(analysis["capture_complete"] is False, "unexpected complete observer capture")
    require(all(observer_error_is_explained(error) for error in analysis["errors"]), "unknown observer error")
    require(
        report["total_workflow_usage"] == scored["measurement"]["usage"],
        f"quality/report usage differs: {run_id}",
    )
    observed = int(report["total_workflow_usage"]["raw_input_plus_output"])
    turns = audit_app_server(artifact)
    require(
        turns["interrupted_turns"] == int(analysis["interrupted_provider_turns"]),
        f"interruption count differs: {run_id}",
    )
    cap = cap_audit(turns["interrupted_turns"], turns["interrupted_prompt_json_bytes"])
    delivery = delivery_summary(mechanism)
    review = review_summary(read_json(state_path))
    warning_audit = {
        "controller_identity_mapping": audit_controller_mapping(analysis),
        "rollout_mapping": audit_rollout_mapping(artifact),
        "unfinished_invocations": audit_unfinished_invocations(artifact),
    }
    source_paths = (
        STUDY_DIR / "runs" / run_id / "admission.json",
        report_path,
        analysis_path,
        mechanism_path,
        state_path,
    )
    return {
        "run_id": run_id,
        "workflow_result": "pass",
        "feature_checks": scored["checks"],
        "completed_features": scored["completed_features"],
        "raw_tokens": {
            "lower": observed,
            "upper": observed + cap["applied_missing_raw_token_cap"],
        },
        "observed_uncached_tokens": int(
            report["total_workflow_usage"]["uncached_input_plus_output"]
        ),
        "turn_audit": turns,
        "cap_audit": cap,
        "controls": EXPECTED_CONTROLS,
        "delivery_events": delivery,
        "review_control": review,
        "observer_warnings": analysis["errors"],
        "observer_warning_audit": warning_audit,
        "source_sha256": {
            str(path.relative_to(REPO_ROOT)): sha256_file(path) for path in source_paths
        },
    }


def build_evidence() -> dict[str, Any]:
    quality = read_json(STUDY_DIR / "quality.json")
    require(quality["complete"] is True, "quality scoring is incomplete")
    require(quality["base_commit"] == BASE_COMMIT, "quality base differs")
    require(quality["task_list_sha256"] == TASK_LIST_SHA256, "quality task differs")
    require(quality["model"] == MODEL and quality["reasoning_effort"] == EFFORT, "quality profile differs")
    runs = [verify_run(run_id, quality) for run_id in RUN_IDS]
    current_interval = mean_interval([run["raw_tokens"] for run in runs])
    prior = read_json(PRIOR_EVIDENCE)
    require(prior["status"] == "complete", "prior endpoint evidence is incomplete")
    require(prior["frozen_setup"]["base_commit"] == BASE_COMMIT, "prior base differs")
    require(prior["frozen_setup"]["model"] == MODEL, "prior model differs")
    require(prior["frozen_setup"]["reasoning_effort"] == EFFORT, "prior effort differs")
    direct = prior["endpoint_groups"]["direct_sequential"]
    normal = prior["endpoint_groups"]["normal_concurrent_work_leaf"]
    direct_interval = direct["raw_token_mean_interval"]
    normal_interval = normal["raw_token_mean_interval"]
    feature_pass_counts = {
        feature: sum(run["feature_checks"][feature] == "pass" for run in runs)
        for feature in ("visual", "status", "completion")
    }
    return {
        "schema_version": 1,
        "status": "complete",
        "study": STUDY_DIR.name,
        "scope": "Steps 1 through 3: correct, verify, and run three all-disabled Work Leaf controls",
        "frozen_setup": {
            "base_commit": BASE_COMMIT,
            "task_list_sha256": TASK_LIST_SHA256,
            "model": MODEL,
            "reasoning_effort": EFFORT,
            "workflow": "normal concurrent Work Leaf",
            "conditions_are_independent_groups": True,
            "maximum_tokens_per_interrupted_response": MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE,
        },
        "runs": runs,
        "cohort": {
            "observations": len(runs),
            "feature_pass_counts": feature_pass_counts,
            "mean_completed_features": statistics.fmean(
                float(run["completed_features"]) for run in runs
            ),
            "raw_token_mean_interval": current_interval,
            "mean_observed_uncached_tokens": statistics.fmean(
                float(run["observed_uncached_tokens"]) for run in runs
            ),
            "changed_reread_exercised_runs": sum(
                run["delivery_events"]["changed_reread"]["full_current_events"] > 0
                for run in runs
            ),
            "unchanged_reread_exercised_runs": sum(
                run["delivery_events"]["unchanged_reread"]["full_current_events"] > 0
                for run in runs
            ),
            "git_review_exercised_runs": sum(
                run["review_control"]["all_reconstructed_from_git"] for run in runs
            ),
        },
        "prior_comparators": {
            "direct_sequential": {
                "observations": direct["observations"],
                "mean_completed_features": direct["mean_completed_features"],
                "raw_token_mean_interval": direct_interval,
            },
            "normal_concurrent_work_leaf": {
                "observations": normal["observations"],
                "mean_completed_features": normal["mean_completed_features"],
                "raw_token_mean_interval": normal_interval,
            },
            "source": str(PRIOR_EVIDENCE.relative_to(REPO_ROOT)),
        },
        "comparisons": {
            "all_disabled_minus_normal_work_leaf_raw_tokens": difference_interval(
                current_interval, normal_interval
            ),
            "direct_minus_all_disabled_raw_tokens": difference_interval(
                direct_interval, current_interval
            ),
        },
        "conclusions": {
            "corrected_control_operational": True,
            "combined_mechanism_effect_known": False,
            "savings_disappear_when_three_mechanisms_are_disabled": "not established",
            "reason": "Both token-difference intervals cross zero; interrupted-turn ceilings overlap, and mean feature completion is 2.33/3 for this control versus 2.67/3 in each prior endpoint group.",
        },
        "source_sha256": {
            str(path.relative_to(REPO_ROOT)): sha256_file(path)
            for path in (
                STUDY_DIR / "score-manifest.json",
                STUDY_DIR / "quality.json",
                STUDY_DIR / "scorer/score.py",
                PRIOR_EVIDENCE,
            )
        },
    }


def main() -> int:
    evidence = build_evidence()
    output = STUDY_DIR / "evidence.json"
    temporary = output.with_suffix(".json.tmp")
    temporary.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(output)
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
