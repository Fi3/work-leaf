#!/usr/bin/env python3
"""Build the six-run normal Work Leaf comparison from frozen evidence."""

from __future__ import annotations

import hashlib
import json
import statistics
from collections import Counter
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
RESPONSE_BOUND = STUDY_DIR / "response-bound.json"
CORRECTED_OBSERVER_SOURCE = REPO_ROOT / "bench-observer" / "src" / "lib.rs"
RUN_IDS = [f"exact-normal-{index:03d}" for index in range(1, 7)]
FEATURES = ("visual", "status", "completion")
MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE = 386_400
EXACT_USAGE_GRACE_OUTCOMES = {
    "forwarded-after-exact-usage",
    "forwarded-after-resumed-output-usage",
}
UNRESOLVED_USAGE_GRACE_OUTCOMES = {
    "forwarded-after-output-resumed",
    "forwarded-after-timeout",
}
ALLOWED_UNCOVERED_ITEM_TYPES = {"userMessage", "reasoning", "agentMessage"}
ALLOWED_POST_DIRECTIVE_ITEM_TYPES = {"reasoning", "agentMessage"}
ALLOWED_UNCOVERED_METHODS = {
    "turn/started",
    "item/started",
    "item/completed",
    "item/agentMessage/delta",
    "mcpServer/startupStatus/updated",
}
ALLOWED_POST_DIRECTIVE_METHODS = {
    "item/started",
    "item/agentMessage/delta",
    "thread/tokenUsage/updated",
    "thread/status/changed",
    "turn/completed",
}


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


def read_json_lines(path: Path) -> list[tuple[str, dict[str, Any]]]:
    records: list[tuple[str, dict[str, Any]]] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line:
            continue
        value = json.loads(line)
        if not isinstance(value, dict):
            raise ValueError(f"{path}:{line_number} is not a JSON object")
        records.append((line, value))
    return records


def extract_thread_id(value: dict[str, Any]) -> str | None:
    params = value.get("params")
    if not isinstance(params, dict):
        return None
    thread_id = params.get("threadId")
    return thread_id if isinstance(thread_id, str) else None


def extract_turn_id(value: dict[str, Any]) -> str | None:
    params = value.get("params")
    if not isinstance(params, dict):
        return None
    turn_id = params.get("turnId")
    if isinstance(turn_id, str):
        return turn_id
    turn = params.get("turn")
    if not isinstance(turn, dict):
        return None
    turn_id = turn.get("id")
    return turn_id if isinstance(turn_id, str) else None


def extract_item(value: dict[str, Any]) -> dict[str, Any] | None:
    params = value.get("params")
    if not isinstance(params, dict):
        return None
    item = params.get("item")
    return item if isinstance(item, dict) else None


def usage_total(value: dict[str, Any]) -> tuple[int, int, int, int] | None:
    if value.get("method") != "thread/tokenUsage/updated":
        return None
    params = value.get("params")
    if not isinstance(params, dict):
        return None
    token_usage = params.get("tokenUsage")
    if not isinstance(token_usage, dict):
        return None
    total = token_usage.get("total")
    if not isinstance(total, dict):
        return None
    return tuple(
        int(total.get(field, 0))
        for field in (
            "inputTokens",
            "cachedInputTokens",
            "outputTokens",
            "reasoningOutputTokens",
        )
    )


def is_work_leaf_directive_message(value: dict[str, Any]) -> bool:
    if value.get("method") != "item/completed":
        return False
    item = extract_item(value)
    if item is None or item.get("type") != "agentMessage":
        return False
    text = item.get("text")
    return isinstance(text, str) and any(
        line.startswith("@work-leaf ") for line in text.splitlines()
    )


def audit_unresolved_response_tails(
    run_id: str,
    app_server: Path,
    expected_unresolved_responses: int,
) -> dict[str, Any]:
    client_records = [value for _, value in read_json_lines(app_server / "client-to-server.raw")]
    server_records = [value for _, value in read_json_lines(app_server / "server-to-client.raw")]
    grace_records = [
        value for _, value in read_json_lines(app_server / "provider-usage-grace.jsonl")
    ]
    unresolved = [
        value
        for value in grace_records
        if value.get("outcome") not in EXACT_USAGE_GRACE_OUTCOMES
    ]
    if len(unresolved) != expected_unresolved_responses:
        raise ValueError(
            f"{run_id} has {len(unresolved)} non-exact grace outcomes but the strict observer "
            f"reports {expected_unresolved_responses} unresolved responses"
        )
    unexpected_grace_outcomes = sorted(
        {
            str(value.get("outcome"))
            for value in unresolved
            if value.get("outcome") not in UNRESOLVED_USAGE_GRACE_OUTCOMES
        }
    )
    if unexpected_grace_outcomes:
        raise ValueError(
            f"{run_id} has unexpected unresolved grace outcomes: "
            f"{unexpected_grace_outcomes}"
        )

    client_interrupts: dict[tuple[str, str], int] = {}
    for value in client_records:
        if value.get("method") != "turn/interrupt":
            continue
        thread_id = extract_thread_id(value)
        turn_id = extract_turn_id(value)
        if thread_id is not None and turn_id is not None:
            key = (thread_id, turn_id)
            client_interrupts[key] = client_interrupts.get(key, 0) + 1

    usage_by_thread: dict[str, list[tuple[int, tuple[int, int, int, int]]]] = {}
    events_by_turn: dict[str, list[tuple[int, dict[str, Any]]]] = {}
    for sequence, value in enumerate(server_records):
        thread_id = extract_thread_id(value)
        turn_id = extract_turn_id(value)
        total = usage_total(value)
        if thread_id is not None and total is not None:
            usage_by_thread.setdefault(thread_id, []).append((sequence, total))
        if turn_id is not None:
            events_by_turn.setdefault(turn_id, []).append((sequence, value))

    details: list[dict[str, Any]] = []
    duplicate_usage_after_directive = 0
    no_usage_after_directive = 0
    for grace in unresolved:
        thread_id = grace.get("thread_id")
        turn_id = grace.get("turn_id")
        if not isinstance(thread_id, str) or not isinstance(turn_id, str):
            raise ValueError(f"{run_id} has a grace record without thread/turn identity")
        key = (thread_id, turn_id)
        if client_interrupts.get(key) != 1:
            raise ValueError(f"{run_id} {turn_id} does not have exactly one client interrupt")

        turn_events = events_by_turn.get(turn_id, [])
        directives = [
            (sequence, value)
            for sequence, value in turn_events
            if is_work_leaf_directive_message(value)
        ]
        if len(directives) != 1:
            raise ValueError(
                f"{run_id} {turn_id} has {len(directives)} completed directive messages"
            )
        directive_sequence = directives[0][0]
        turn_started_sequences = [
            sequence
            for sequence, value in turn_events
            if value.get("method") == "turn/started"
        ]
        if len(turn_started_sequences) != 1:
            raise ValueError(
                f"{run_id} {turn_id} has {len(turn_started_sequences)} turn starts"
            )
        turn_started_sequence = turn_started_sequences[0]
        prior_usage = [
            (sequence, total)
            for sequence, total in usage_by_thread.get(thread_id, [])
            if sequence < directive_sequence
        ]
        prior_usage_sequence = prior_usage[-1][0] if prior_usage else -1
        prior_usage_total = prior_usage[-1][1] if prior_usage else (0, 0, 0, 0)
        response_start_sequence = max(
            turn_started_sequence,
            prior_usage_sequence + 1,
        )
        uncovered = [
            (sequence, value)
            for sequence, value in enumerate(server_records)
            if response_start_sequence <= sequence <= directive_sequence
            and (
                extract_thread_id(value) == thread_id
                or extract_turn_id(value) == turn_id
            )
        ]
        foreign_uncovered_turns = sorted(
            {
                event_turn_id
                for _, value in uncovered
                if (event_turn_id := extract_turn_id(value)) is not None
                and event_turn_id != turn_id
            }
        )
        uncovered_items = [
            extract_item(value)
            for _, value in uncovered
            if value.get("method") in {"item/started", "item/completed"}
        ]
        uncovered_item_types = [
            str(item.get("type")) for item in uncovered_items if item is not None
        ]
        uncovered_methods = Counter(
            str(value.get("method")) for _, value in uncovered
        )
        unexpected_uncovered_methods = sorted(
            set(uncovered_methods) - ALLOWED_UNCOVERED_METHODS
        )
        unexpected_item_types = sorted(
            set(uncovered_item_types) - ALLOWED_UNCOVERED_ITEM_TYPES
        )
        started_items = Counter(
            (str(item.get("type")), str(item.get("id")))
            for _, value in uncovered
            if value.get("method") == "item/started"
            and (item := extract_item(value)) is not None
        )
        completed_items = Counter(
            (str(item.get("type")), str(item.get("id")))
            for _, value in uncovered
            if value.get("method") == "item/completed"
            and (item := extract_item(value)) is not None
        )
        completed_agent_messages = sum(
            value.get("method") == "item/completed"
            and (extract_item(value) or {}).get("type") == "agentMessage"
            for _, value in uncovered
        )
        if (
            unexpected_uncovered_methods
            or foreign_uncovered_turns
            or unexpected_item_types
            or started_items != completed_items
            or completed_agent_messages != 1
        ):
            raise ValueError(
                f"{run_id} {turn_id} does not isolate one response: "
                f"unexpected_protocol_events={unexpected_uncovered_methods}, "
                f"foreign_turns={foreign_uncovered_turns}, "
                f"unexpected_items={unexpected_item_types}, "
                f"unpaired_items={started_items != completed_items}, "
                f"completed_agent_messages={completed_agent_messages}"
            )

        completed_turns = [
            (sequence, value)
            for sequence, value in turn_events
            if value.get("method") == "turn/completed"
        ]
        if len(completed_turns) != 1:
            raise ValueError(f"{run_id} {turn_id} does not complete exactly once")
        completion_sequence, completed_turn = completed_turns[0]
        completed_params = completed_turn.get("params") or {}
        completed_turn = completed_params.get("turn") or {}
        if completed_turn.get("status") != "interrupted":
            raise ValueError(f"{run_id} {turn_id} was not completed as interrupted")

        post_directive = [
            (sequence, value)
            for sequence, value in enumerate(server_records)
            if directive_sequence < sequence <= completion_sequence
            and (
                extract_thread_id(value) == thread_id
                or extract_turn_id(value) == turn_id
            )
        ]
        foreign_post_turns = sorted(
            {
                event_turn_id
                for _, value in post_directive
                if (event_turn_id := extract_turn_id(value)) is not None
                and event_turn_id != turn_id
            }
        )
        post_methods = Counter(
            str(value.get("method")) for _, value in post_directive
        )
        unexpected_post_methods = sorted(
            set(post_methods) - ALLOWED_POST_DIRECTIVE_METHODS
        )
        post_item_types = [
            str(item.get("type"))
            for _, value in post_directive
            if (item := extract_item(value)) is not None
        ]
        unexpected_post_items = sorted(
            set(post_item_types) - ALLOWED_POST_DIRECTIVE_ITEM_TYPES
        )
        if unexpected_post_methods or foreign_post_turns or unexpected_post_items:
            raise ValueError(
                f"{run_id} {turn_id} has unexpected protocol events after its directive: "
                f"methods={unexpected_post_methods}, foreign_turns={foreign_post_turns}, "
                f"items={unexpected_post_items}"
            )
        post_started_items = sum(
            value.get("method") == "item/started" for _, value in post_directive
        )
        if post_started_items > 1:
            raise ValueError(
                f"{run_id} {turn_id} starts {post_started_items} items after its directive"
            )
        if (
            grace.get("outcome") == "forwarded-after-output-resumed"
            and post_started_items != 1
        ):
            raise ValueError(
                f"{run_id} {turn_id} reports resumed output without one started item"
            )

        post_usage = [
            (sequence, usage_total(value))
            for sequence, value in post_directive
            if usage_total(value) is not None
        ]
        if post_usage:
            for _, total in post_usage:
                if total != prior_usage_total:
                    raise ValueError(
                        f"{run_id} {turn_id} has advancing usage after the directive"
                    )
            duplicate_usage_after_directive += 1
        else:
            no_usage_after_directive += 1

        details.append(
            {
                "thread_id": thread_id,
                "turn_id": turn_id,
                "grace_outcome": grace.get("outcome"),
                "previous_usage_sequence": prior_usage_sequence,
                "response_start_sequence": response_start_sequence,
                "directive_sequence": directive_sequence,
                "completion_sequence": completion_sequence,
                "completed_agent_messages_in_uncovered_tail": (
                    completed_agent_messages
                ),
                "uncovered_protocol_methods": dict(sorted(uncovered_methods.items())),
                "tool_boundaries_in_uncovered_tail": len(unexpected_item_types),
                "post_directive_protocol_methods": dict(sorted(post_methods.items())),
                "unfinished_items_after_directive": post_started_items,
                "post_directive_usage": "duplicate" if post_usage else "absent",
            }
        )

    return {
        "run_id": run_id,
        "audited_responses": len(details),
        "single_response_bound_proven": True,
        "tool_boundaries_in_uncovered_tails": 0,
        "unfinished_items_after_directive": sum(
            int(detail["unfinished_items_after_directive"]) for detail in details
        ),
        "grace_outcomes": dict(
            sorted(Counter(str(detail["grace_outcome"]) for detail in details).items())
        ),
        "duplicate_usage_after_directive": duplicate_usage_after_directive,
        "no_usage_after_directive": no_usage_after_directive,
        "details": details,
        "grace_stream": str(
            (app_server / "provider-usage-grace.jsonl").relative_to(REPO_ROOT)
        ),
        "grace_stream_sha256": sha256_file(
            app_server / "provider-usage-grace.jsonl"
        ),
    }


def capture_bound_audit(
    run_id: str,
    app_server: Path,
    expected_unresolved_responses: int,
) -> dict[str, Any]:
    client_path = app_server / "client-to-server.raw"
    server_path = app_server / "server-to-client.raw"
    turn_starts = 0
    context_windows: set[int] = set()
    usage_events = 0
    observed_last_response_raw_tokens: list[int] = []

    for _, value in read_json_lines(client_path):
        if value.get("method") == "turn/start":
            turn_starts += 1

    server_records = read_json_lines(server_path)
    for _, value in server_records:
        params = value.get("params")
        if not isinstance(params, dict):
            continue
        if value.get("method") == "thread/tokenUsage/updated":
            token_usage = params.get("tokenUsage")
            if not isinstance(token_usage, dict):
                continue
            context_window = token_usage.get("modelContextWindow")
            if context_window is not None:
                context_windows.add(int(context_window))
                usage_events += 1
            last = token_usage.get("last")
            if isinstance(last, dict):
                observed_last_response_raw_tokens.append(
                    int(last.get("inputTokens", 0)) + int(last.get("outputTokens", 0))
                )

    if len(context_windows) != 1:
        raise ValueError(
            f"{run_id} does not have one effective context window: {context_windows}"
        )
    context_window = next(iter(context_windows))
    response_bound = read_json(RESPONSE_BOUND)
    frozen_client = response_bound["frozen_client"]
    model_bound = response_bound["model"]
    declared_bound = response_bound["bound"]
    maximum_context_window = int(
        frozen_client["catalog_max_context_window_tokens"]
    )
    effective_percent = int(frozen_client["effective_context_window_percent"])
    hard_active_context_window = int(
        frozen_client["hard_active_context_window_tokens"]
    )
    maximum_output = int(model_bound["maximum_output_tokens"])
    maximum_observed_response = max(observed_last_response_raw_tokens, default=0)
    expected_effective_context_window = maximum_context_window * effective_percent // 100
    maximum_single_response = hard_active_context_window + maximum_output
    if hard_active_context_window != expected_effective_context_window:
        raise ValueError(
            f"{run_id} declared hard active context {hard_active_context_window} does not match "
            f"the catalog limit and effective percentage"
        )
    if context_window != hard_active_context_window:
        raise ValueError(
            f"{run_id} effective context {context_window} does not match the frozen client limit"
        )
    if maximum_single_response != MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE:
        raise ValueError(
            f"{run_id} frozen client/model maximum is {maximum_single_response}, not "
            f"{MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE}"
        )
    if int(declared_bound["maximum_raw_tokens_per_response"]) != maximum_single_response:
        raise ValueError(f"{run_id} response-bound evidence has inconsistent arithmetic")
    if maximum_observed_response > MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE:
        raise ValueError(
            f"{run_id} observed a {maximum_observed_response}-token response but the declared "
            f"cap is {MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE}"
        )
    return {
        "run_id": run_id,
        "turns": turn_starts,
        "usage_events_with_context_window": usage_events,
        "effective_context_window_tokens": context_window,
        "catalog_max_context_window_tokens": maximum_context_window,
        "hard_active_context_window_tokens": hard_active_context_window,
        "maximum_output_tokens": maximum_output,
        "maximum_observed_last_response_raw_tokens": maximum_observed_response,
        "maximum_single_response_raw_tokens": maximum_single_response,
        "declared_response_cap": MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE,
        "response_cap_headroom_tokens": (
            MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE - maximum_single_response
        ),
        "unresolved_response_tail_audit": audit_unresolved_response_tails(
            run_id,
            app_server,
            expected_unresolved_responses,
        ),
        "client_stream": str(client_path.relative_to(REPO_ROOT)),
        "client_stream_sha256": sha256_file(client_path),
        "server_stream": str(server_path.relative_to(REPO_ROOT)),
        "server_stream_sha256": sha256_file(server_path),
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


def load_work_leaf_rows() -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    quality = read_json(QUALITY)
    scored = {row["id"]: row for row in quality["runs"]}
    rows: list[dict[str, Any]] = []
    bound_audits: list[dict[str, Any]] = []

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
            "agent_backend": "codex",
            "agent_transport": "app-server",
            "agent_model": "gpt-5.5",
            "agent_reasoning_effort": "xhigh",
            "codex_cli_version": "codex-cli 0.150.1",
            "actual_codex_sha256": read_json(RESPONSE_BOUND)["frozen_client"][
                "binary_sha256"
            ],
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
        bound_audits.append(capture_bound_audit(run_id, app_servers[0], missing))
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
    return rows, bound_audits


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
    work_leaf_rows, bound_audits = load_work_leaf_rows()
    direct = summarize(direct_rows)
    work_leaf = summarize(work_leaf_rows)
    comparison = compare_groups(direct, work_leaf)
    tail_audits = [audit["unresolved_response_tail_audit"] for audit in bound_audits]
    grace_outcomes: Counter[str] = Counter()
    for audit in tail_audits:
        grace_outcomes.update(audit["grace_outcomes"])

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
                "One capture is exact. Thirty-five interrupted responses across the other five "
                "captures have no provable terminal usage. Recorded totals are lower bounds; each "
                "unresolved final response receives a 386,400-token cap. All 35 raw event tails "
                "contain one completed directive response and no intervening tool boundary. The "
                "frozen Codex 0.150.1 client enforces a 258,400-token hard active-context limit "
                "and the model limits output to 128,000 tokens, so one response can contribute no "
                "more than 386,400 raw tokens. The same cap is added to the uncached upper bound because "
                "the missing cached-input split is unknown."
            ),
            "maximum_raw_tokens_per_unresolved_response": (
                MAXIMUM_RAW_TOKENS_PER_UNRESOLVED_RESPONSE
            ),
            "maximum_output_source": read_json(RESPONSE_BOUND)["model"]["source"],
            "effective_context_window_tokens": max(
                audit["effective_context_window_tokens"] for audit in bound_audits
            ),
            "catalog_max_context_window_tokens": max(
                audit["catalog_max_context_window_tokens"]
                for audit in bound_audits
            ),
            "hard_active_context_window_tokens": max(
                audit["hard_active_context_window_tokens"]
                for audit in bound_audits
            ),
            "maximum_output_tokens": max(
                audit["maximum_output_tokens"] for audit in bound_audits
            ),
            "maximum_observed_last_response_raw_tokens": max(
                audit["maximum_observed_last_response_raw_tokens"]
                for audit in bound_audits
            ),
            "maximum_single_response_raw_tokens": max(
                audit["maximum_single_response_raw_tokens"] for audit in bound_audits
            ),
            "response_cap_headroom_tokens": min(
                audit["response_cap_headroom_tokens"] for audit in bound_audits
            ),
            "capture_bound_audits": bound_audits,
            "unresolved_response_tail_audit": {
                "audited_responses": sum(
                    audit["audited_responses"] for audit in tail_audits
                ),
                "single_response_bound_proven": all(
                    audit["single_response_bound_proven"] for audit in tail_audits
                ),
                "tool_boundaries_in_uncovered_tails": sum(
                    audit["tool_boundaries_in_uncovered_tails"]
                    for audit in tail_audits
                ),
                "unfinished_items_after_directive": sum(
                    audit["unfinished_items_after_directive"]
                    for audit in tail_audits
                ),
                "grace_outcomes": dict(sorted(grace_outcomes.items())),
                "duplicate_usage_after_directive": sum(
                    audit["duplicate_usage_after_directive"] for audit in tail_audits
                ),
                "no_usage_after_directive": sum(
                    audit["no_usage_after_directive"] for audit in tail_audits
                ),
                "runs": tail_audits,
            },
            "response_bound_source": str(RESPONSE_BOUND.relative_to(REPO_ROOT)),
            "response_bound_source_sha256": sha256_file(RESPONSE_BOUND),
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
            str(RESPONSE_BOUND.relative_to(REPO_ROOT)): sha256_file(RESPONSE_BOUND),
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
