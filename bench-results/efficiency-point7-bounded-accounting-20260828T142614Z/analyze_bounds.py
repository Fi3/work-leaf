#!/usr/bin/env python3

import hashlib
import json
from pathlib import Path
from typing import Any


STUDY_DIR = Path(__file__).resolve().parent
REPO_ROOT = STUDY_DIR.parents[1]
BASE_COMMIT = "c92a0b7060a36eac6db2d869b85e589a7a9480f9"
MODEL = "gpt-5.5"
EFFORT = "xhigh"
TASK_LIST_SHA256 = "45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a"
EFFECTIVE_CONTEXT_WINDOW = 258_400
DOCUMENTED_MAXIMUM_OUTPUT = 128_000
MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE = 400_000
MODEL_DOCUMENTATION = "https://developers.openai.com/api/docs/models/gpt-5.5"
CODEX_CONTEXT_LIMIT_SOURCE = (
    "https://github.com/openai/codex/blob/rust-v0.149.1/"
    "codex-rs/core/src/session/context_window.rs#L53-L76"
)

DIRECT_RESULT = REPO_ROOT / (
    "bench-results/efficiency-point7-exact-accounting-20260828T113610Z/"
    "direct-result.json"
)
DIRECT_REPORT = REPO_ROOT / (
    "bench-results/efficiency-point7-exact-accounting-20260828T113610Z/runs/"
    "direct/point7-exact-direct-three-feature-sequential-bench-artifacts/report.json"
)
RESIDUAL_ROOT = REPO_ROOT / "bench-results/efficiency-residual-cause-20260828T070112Z"
NORMAL_ARTIFACT = RESIDUAL_ROOT / (
    "runs/wl-normal-003/"
    "residual-wl-normal-003-three-feature-bench-artifacts"
)
ALL_OFF_ARTIFACT = RESIDUAL_ROOT / (
    "runs/wl-all-off-002/"
    "residual-wl-all-off-002-three-feature-bench-artifacts"
)
NORMAL_QUALITY = RESIDUAL_ROOT / "quality/wl-normal-003/result.json"
ALL_OFF_QUALITY = RESIDUAL_ROOT / "quality/wl-all-off-002/result.json"

SOURCE_HASHES = {
    DIRECT_RESULT: "0e31fa8b26e6199115e965abd9338c8c19775f0f25ca050eabd3b2c281e26cd2",
    DIRECT_REPORT: "99075f4699c8ed01f36c794d607a685381e05daf0f6c35b158ac06e47eba8f61",
    NORMAL_QUALITY: "97b147bcd19f1635ce9c9913b602b379dd18b03795901f5d42d21448c4370f5a",
    NORMAL_ARTIFACT / "report.json": "efdaac9c70f606d36ba05b8d81575073f57bb60fbdd1f80a04e4a8411ffa4b18",
    NORMAL_ARTIFACT
    / "observation/app-server/00002551049505474822-3299243/client-to-server.raw": "beb311c9ee7cde2c51b320431709e8959bb50ab79edec623d07c38f2975c6408",
    NORMAL_ARTIFACT
    / "observation/app-server/00002551049505474822-3299243/server-to-client.raw": "5663e71c1f5fe9ca6ae4f1a61c7d3437d54a9511ba9631bfc70146fbf82eb503",
    NORMAL_ARTIFACT
    / "observation/rollout-metadata.jsonl": "0686bc6cf7577148ec51aabe6192679f44700f34c9e4410f08d47ae9845e49b3",
    NORMAL_ARTIFACT
    / "daemon-env.txt": "d3b29fb111cab60e4ecfae9777757ce62ca38999c429abc3cd980f4873698c29",
    ALL_OFF_QUALITY: "1eb96bd4ee01608c20fc5cc26cd1968484b486d07bfb9c2603804f2ba402e30f",
    ALL_OFF_ARTIFACT / "report.json": "6d54fefe6383e45f397f080ae9451add8acf2aac63d8e498aef1f248995c020c",
    ALL_OFF_ARTIFACT
    / "observation/app-server/00002553926897852129-3388596/client-to-server.raw": "8a11f68f26d07a3beed2f52ba6e937063666b6f0d795b5fe905d89cb65559c29",
    ALL_OFF_ARTIFACT
    / "observation/app-server/00002553926897852129-3388596/server-to-client.raw": "75cf78471aeaf33c4d6fabccaf0d03c9fafe6e2e7d360761639e61d1ee5eecea",
    ALL_OFF_ARTIFACT
    / "observation/rollout-metadata.jsonl": "e3db65e2d4995ebd0f94fec02bd336b4d5dbcd45ee7315fc6982d0540fbf3543",
    ALL_OFF_ARTIFACT
    / "observation/process-invocations.jsonl": "478777fd3865933501364e8daa0eef3d4fa3d8057ca269d13b50014b4d08c8a1",
    ALL_OFF_ARTIFACT
    / "daemon-env.txt": "f17470120cdd70720cf541a22ea1d023eb07d2460d5a1cf34a573a7383f17e6f",
}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    rows = []
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise ValueError(f"{path}:{line_number}: invalid JSON: {error}") from error
    return rows


def rpc_id(row: dict[str, Any]) -> str | None:
    value = row.get("id")
    return None if value is None else str(value)


def audit_app_server(client_path: Path, server_path: Path) -> dict[str, int]:
    client_rows = read_jsonl(client_path)
    server_rows = read_jsonl(server_path)
    starts_by_request: dict[str, tuple[str, int]] = {}
    interrupt_requests: set[tuple[str, str]] = set()

    for row in client_rows:
        method = row.get("method")
        params = row.get("params", {})
        if method == "turn/start":
            request_id = rpc_id(row)
            thread_id = params.get("threadId")
            if request_id is None or not isinstance(thread_id, str):
                raise ValueError("turn/start has no request or thread identity")
            if request_id in starts_by_request:
                raise ValueError(f"duplicate turn/start request {request_id}")
            prompt_input = params.get("input", [])
            if not isinstance(prompt_input, list) or any(
                not isinstance(item, dict) or item.get("type") != "text"
                for item in prompt_input
            ):
                raise ValueError("turn/start contains a non-text prompt item")
            prompt_json = json.dumps(
                prompt_input, ensure_ascii=False, separators=(",", ":")
            ).encode()
            starts_by_request[request_id] = (thread_id, len(prompt_json))
        elif method == "turn/interrupt":
            thread_id = params.get("threadId")
            turn_id = params.get("turnId")
            if not isinstance(thread_id, str) or not isinstance(turn_id, str):
                raise ValueError("turn/interrupt has no thread or turn identity")
            identity = (thread_id, turn_id)
            if identity in interrupt_requests:
                raise ValueError(f"duplicate turn/interrupt for {identity}")
            interrupt_requests.add(identity)

    started_turns: set[tuple[str, str]] = set()
    prompt_bytes_by_turn: dict[tuple[str, str], int] = {}
    outcomes: dict[tuple[str, str], str] = {}
    rpc_errors = 0
    for row in server_rows:
        if "error" in row:
            rpc_errors += 1
        request_id = rpc_id(row)
        if request_id in starts_by_request:
            turn = row.get("result", {}).get("turn", {})
            turn_id = turn.get("id")
            if not isinstance(turn_id, str):
                raise ValueError(f"turn/start response {request_id} has no turn identity")
            thread_id, prompt_bytes = starts_by_request[request_id]
            identity = (thread_id, turn_id)
            if identity in started_turns:
                raise ValueError(f"duplicate turn/start response for {identity}")
            started_turns.add(identity)
            prompt_bytes_by_turn[identity] = prompt_bytes
        if row.get("method") == "turn/completed":
            params = row.get("params", {})
            turn = params.get("turn", {})
            thread_id = params.get("threadId", turn.get("threadId"))
            turn_id = params.get("turnId", turn.get("id"))
            status = turn.get("status", params.get("status"))
            if not all(isinstance(value, str) for value in (thread_id, turn_id, status)):
                raise ValueError("turn/completed has incomplete identity or outcome")
            identity = (thread_id, turn_id)
            if identity in outcomes:
                raise ValueError(f"duplicate turn/completed for {identity}")
            if status not in {"completed", "interrupted"}:
                raise ValueError(f"unsupported terminal outcome {status} for {identity}")
            outcomes[identity] = status

    if rpc_errors:
        raise ValueError(f"app-server transcript contains {rpc_errors} JSON-RPC errors")
    if len(started_turns) != len(starts_by_request) or started_turns != set(outcomes):
        raise ValueError(
            "started/completed turn mismatch: "
            f"requests={len(starts_by_request)} starts={len(started_turns)} "
            f"outcomes={len(outcomes)}"
        )
    interrupted_outcomes = {
        identity for identity, status in outcomes.items() if status == "interrupted"
    }
    if interrupt_requests != interrupted_outcomes:
        raise ValueError(
            "interrupt request/outcome mismatch: "
            f"requests={len(interrupt_requests)} outcomes={len(interrupted_outcomes)}"
        )
    return {
        "started_turns": len(started_turns),
        "completed_turns": sum(status == "completed" for status in outcomes.values()),
        "interrupted_turns": len(interrupted_outcomes),
        "interrupted_prompt_json_bytes": sum(
            prompt_bytes_by_turn[identity] for identity in interrupted_outcomes
        ),
        "rpc_errors": rpc_errors,
    }


def usage_window_evidence(server_path: Path) -> dict[str, int]:
    windows = [
        row.get("params", {}).get("tokenUsage", {}).get("modelContextWindow")
        for row in read_jsonl(server_path)
        if row.get("method") == "thread/tokenUsage/updated"
    ]
    require(windows, f"no provider usage events: {server_path}")
    require(
        set(windows) == {EFFECTIVE_CONTEXT_WINDOW},
        f"unexpected model context window: {server_path}",
    )
    return {
        "usage_events": len(windows),
        "effective_context_window": EFFECTIVE_CONTEXT_WINDOW,
    }


def condition_bound(
    *,
    direct_raw_tokens: int,
    observed_raw_tokens: int,
    interrupted_turns: int,
    maximum_tokens_per_interrupted_turn: int,
) -> dict[str, int | float]:
    values = (
        direct_raw_tokens,
        observed_raw_tokens,
        interrupted_turns,
        maximum_tokens_per_interrupted_turn,
    )
    if any(value < 0 for value in values) or direct_raw_tokens == 0:
        raise ValueError("token totals and interruption count must be nonnegative")
    upper_bound = observed_raw_tokens + (
        interrupted_turns * maximum_tokens_per_interrupted_turn
    )
    minimum_saved = max(direct_raw_tokens - upper_bound, 0)
    return {
        "observed_raw_tokens": observed_raw_tokens,
        "interrupted_turns": interrupted_turns,
        "maximum_tokens_per_interrupted_turn": maximum_tokens_per_interrupted_turn,
        "raw_token_upper_bound": upper_bound,
        "minimum_raw_tokens_saved": minimum_saved,
        "minimum_raw_reduction_percent": round(
            minimum_saved * 100 / direct_raw_tokens, 6
        ),
    }


def missing_usage_cap_audit(
    *,
    interrupted_turns: int,
    interrupted_prompt_json_bytes: int,
    effective_context_window: int,
    maximum_output_tokens: int,
    declared_tokens_per_interruption: int,
) -> dict[str, int]:
    declared = interrupted_turns * declared_tokens_per_interruption
    required = interrupted_turns * (
        effective_context_window + maximum_output_tokens
    ) + interrupted_prompt_json_bytes
    if declared < required:
        raise ValueError("aggregate interrupted-response cap has insufficient prompt headroom")
    return {
        "declared_missing_raw_token_cap": declared,
        "context_output_and_prompt_upper_bound": required,
        "remaining_headroom": declared - required,
    }


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def verify_source_hashes() -> dict[str, str]:
    verified = {}
    for path, expected in SOURCE_HASHES.items():
        actual = sha256_file(path)
        require(actual == expected, f"source hash changed for {path}: {actual}")
        try:
            label = str(path.relative_to(REPO_ROOT))
        except ValueError:
            label = str(path)
        verified[label] = actual
    return verified


def verify_quality(path: Path, expected_id: str) -> dict[str, Any]:
    result = read_json(path)
    require(result.get("complete") is True, f"quality result is incomplete: {path}")
    require(result.get("base_commit") == BASE_COMMIT, f"wrong base commit: {path}")
    require(result.get("model") == MODEL, f"wrong model: {path}")
    require(result.get("reasoning_effort") == EFFORT, f"wrong effort: {path}")
    require(
        result.get("task_list_sha256") == TASK_LIST_SHA256,
        f"wrong task list: {path}",
    )
    runs = result.get("runs", [])
    require(len(runs) == 1, f"expected one scored candidate: {path}")
    run = runs[0]
    require(run.get("id") == expected_id, f"wrong scored run: {path}")
    require(run.get("workflow_result") == "pass", f"workflow did not pass: {path}")
    require(run.get("completed_features") == 3, f"candidate is not 3/3: {path}")
    require(
        run.get("checks")
        == {"completion": "pass", "status": "pass", "visual": "pass"},
        f"feature checks did not all pass: {path}",
    )
    return run


def verify_report(
    path: Path, *, mode: str, schedule: str, transport: str
) -> dict[str, Any]:
    report = read_json(path)
    require(report.get("base_commit") == BASE_COMMIT, f"wrong base commit: {path}")
    require(report.get("workflow_result") == "pass", f"workflow did not pass: {path}")
    require(report.get("bench_mode") == mode, f"wrong workflow mode: {path}")
    require(report.get("feature_schedule") == schedule, f"wrong feature schedule: {path}")
    require(report.get("agent_backend") == "codex", f"wrong agent backend: {path}")
    require(report.get("agent_transport") == transport, f"wrong transport: {path}")
    require(report.get("agent_model") == MODEL, f"wrong model: {path}")
    require(report.get("agent_reasoning_effort") == EFFORT, f"wrong effort: {path}")
    require(
        report.get("requested_agent_model") == MODEL,
        f"wrong requested model: {path}",
    )
    require(
        report.get("requested_agent_reasoning_effort") == EFFORT,
        f"wrong requested effort: {path}",
    )
    checks = report.get("code_quality", "")
    for command in ("cargo fmt", "cargo clippy", "cargo test"):
        require(command in checks, f"missing final {command} result: {path}")
    return report


def verify_rollout_profile(path: Path) -> dict[str, Any]:
    rows = read_jsonl(path)
    require(rows, f"no rollout metadata: {path}")
    require({row.get("model") for row in rows} == {MODEL}, f"mixed models: {path}")
    require({row.get("effort") for row in rows} == {EFFORT}, f"mixed efforts: {path}")
    require(
        not any(row.get("descendant") for row in rows),
        f"recursive provider thread found: {path}",
    )
    return {
        "provider_threads": len(rows),
        "models": [MODEL],
        "reasoning_efforts": [EFFORT],
        "recursive_provider_threads": 0,
    }


def read_experiment_controls(path: Path) -> dict[str, str]:
    values = {}
    for line in path.read_text().splitlines():
        if "=" not in line:
            continue
        name, value = line.split("=", 1)
        if name.startswith("WORK_LEAF_EXPERIMENT_"):
            values[name] = value
    return values


def verify_all_off_non_provider_gap() -> dict[str, Any]:
    rows = read_jsonl(ALL_OFF_ARTIFACT / "observation/process-invocations.jsonl")
    incomplete = [row for row in rows if row.get("end") is None]
    require(len(incomplete) == 1, "unexpected all-off incomplete process count")
    require(
        incomplete[0].get("capture_kind") == "locked-command",
        "all-off has an incomplete provider invocation",
    )
    return {
        "incomplete_processes": 1,
        "incomplete_process_capture_kind": "locked-command",
        "provider_usage_affected": False,
    }


def analyze() -> dict[str, Any]:
    hashes = verify_source_hashes()
    direct_quality = verify_quality(DIRECT_RESULT, "point7-exact-direct")
    normal_quality = verify_quality(NORMAL_QUALITY, "wl-normal-003")
    all_off_quality = verify_quality(ALL_OFF_QUALITY, "wl-all-off-002")
    direct_report = verify_report(
        DIRECT_REPORT,
        mode="sequential",
        schedule="sequential",
        transport="direct-codex-cli",
    )
    normal_report = verify_report(
        NORMAL_ARTIFACT / "report.json",
        mode="work-leaf",
        schedule="concurrent",
        transport="app-server",
    )
    all_off_report = verify_report(
        ALL_OFF_ARTIFACT / "report.json",
        mode="work-leaf",
        schedule="concurrent",
        transport="app-server",
    )
    direct_raw = direct_report["total_workflow_usage"]["raw_input_plus_output"]
    require(direct_report.get("measurement_status") == "complete", "direct usage is not exact")
    require(
        direct_quality["measurement"]["usable"] is True
        and direct_quality["measurement"]["usage"]["raw_input_plus_output"] == direct_raw,
        "direct scorer and report usage disagree",
    )

    normal_client = NORMAL_ARTIFACT / (
        "observation/app-server/00002551049505474822-3299243/client-to-server.raw"
    )
    normal_server = NORMAL_ARTIFACT / (
        "observation/app-server/00002551049505474822-3299243/server-to-client.raw"
    )
    all_off_client = ALL_OFF_ARTIFACT / (
        "observation/app-server/00002553926897852129-3388596/client-to-server.raw"
    )
    all_off_server = ALL_OFF_ARTIFACT / (
        "observation/app-server/00002553926897852129-3388596/server-to-client.raw"
    )
    normal_turns = audit_app_server(normal_client, normal_server)
    all_off_turns = audit_app_server(all_off_client, all_off_server)
    normal_raw = normal_report["total_workflow_usage"]["raw_input_plus_output"]
    all_off_raw = all_off_report["total_workflow_usage"]["raw_input_plus_output"]
    require(
        normal_quality["measurement"]["usage"]["raw_input_plus_output"] == normal_raw,
        "normal Work Leaf scorer and report usage disagree",
    )
    require(
        all_off_quality["measurement"]["usage"]["raw_input_plus_output"]
        == all_off_raw,
        "all-off Work Leaf scorer and report usage disagree",
    )

    normal_controls = read_experiment_controls(NORMAL_ARTIFACT / "daemon-env.txt")
    all_off_controls = read_experiment_controls(ALL_OFF_ARTIFACT / "daemon-env.txt")
    require(
        normal_controls
        == {
            "WORK_LEAF_EXPERIMENT_CHANGED_REPEAT_DELIVERY": "normal",
            "WORK_LEAF_EXPERIMENT_REVIEW_PROVENANCE": "inline-exact",
            "WORK_LEAF_EXPERIMENT_UNCHANGED_REPEAT_DELIVERY": "normal",
        },
        "normal Work Leaf controls are not normal",
    )
    require(
        all_off_controls
        == {
            "WORK_LEAF_EXPERIMENT_CHANGED_REPEAT_DELIVERY": "full",
            "WORK_LEAF_EXPERIMENT_REVIEW_PROVENANCE": "git-reconstruct",
            "WORK_LEAF_EXPERIMENT_UNCHANGED_REPEAT_DELIVERY": "full",
        },
        "all-off Work Leaf controls are not all disabled",
    )
    require(
        MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE
        >= EFFECTIVE_CONTEXT_WINDOW + DOCUMENTED_MAXIMUM_OUTPUT,
        "interrupted-response cap is not conservative",
    )
    normal_cap_audit = missing_usage_cap_audit(
        interrupted_turns=normal_turns["interrupted_turns"],
        interrupted_prompt_json_bytes=normal_turns["interrupted_prompt_json_bytes"],
        effective_context_window=EFFECTIVE_CONTEXT_WINDOW,
        maximum_output_tokens=DOCUMENTED_MAXIMUM_OUTPUT,
        declared_tokens_per_interruption=MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE,
    )
    all_off_cap_audit = missing_usage_cap_audit(
        interrupted_turns=all_off_turns["interrupted_turns"],
        interrupted_prompt_json_bytes=all_off_turns["interrupted_prompt_json_bytes"],
        effective_context_window=EFFECTIVE_CONTEXT_WINDOW,
        maximum_output_tokens=DOCUMENTED_MAXIMUM_OUTPUT,
        declared_tokens_per_interruption=MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE,
    )

    normal_bound = condition_bound(
        direct_raw_tokens=direct_raw,
        observed_raw_tokens=normal_raw,
        interrupted_turns=normal_turns["interrupted_turns"],
        maximum_tokens_per_interrupted_turn=MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE,
    )
    all_off_bound = condition_bound(
        direct_raw_tokens=direct_raw,
        observed_raw_tokens=all_off_raw,
        interrupted_turns=all_off_turns["interrupted_turns"],
        maximum_tokens_per_interrupted_turn=MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE,
    )
    require(normal_bound["minimum_raw_tokens_saved"] > 0, "normal saving is not proven")
    require(all_off_bound["minimum_raw_tokens_saved"] > 0, "all-off saving is not proven")

    return {
        "schema_version": 1,
        "study": STUDY_DIR.name,
        "status": "complete",
        "provider_workflows_launched": 0,
        "question": "Does a raw-token saving remain under conservative accounting, including when all three candidate mechanisms are disabled?",
        "frozen_setup": {
            "base_commit": BASE_COMMIT,
            "task_list_sha256": TASK_LIST_SHA256,
            "model": MODEL,
            "reasoning_effort": EFFORT,
            "quality_checks": ["visual", "status", "completion"],
            "required_completed_features": 3,
        },
        "bound": {
            "effective_context_window_observed_in_saved_runs": EFFECTIVE_CONTEXT_WINDOW,
            "documented_maximum_output_tokens": DOCUMENTED_MAXIMUM_OUTPUT,
            "model_documentation": MODEL_DOCUMENTATION,
            "codex_context_limit_source": CODEX_CONTEXT_LIMIT_SOURCE,
            "sum": EFFECTIVE_CONTEXT_WINDOW + DOCUMENTED_MAXIMUM_OUTPUT,
            "rounded_maximum_tokens_per_interrupted_response": MAXIMUM_TOKENS_PER_INTERRUPTED_RESPONSE,
            "interpretation": "Each interrupted turn is charged the full rounded maximum even though only its final response lacks terminal usage. Aggregate headroom also covers the captured new-turn prompt JSON bytes.",
        },
        "direct_sequential": {
            "run_id": "point7-exact-direct",
            "workflow_result": "pass",
            "completed_features": 3,
            "raw_tokens_exact": direct_raw,
            "uncached_tokens_exact": direct_report["total_workflow_usage"][
                "uncached_input_plus_output"
            ],
            "measurement": "exact",
        },
        "normal_work_leaf": {
            "run_id": "wl-normal-003",
            "workflow_result": "pass",
            "completed_features": 3,
            "turn_audit": normal_turns,
            "rollout_profile": verify_rollout_profile(
                NORMAL_ARTIFACT / "observation/rollout-metadata.jsonl"
            ),
            "context_window_evidence": usage_window_evidence(normal_server),
            "missing_usage_cap_audit": normal_cap_audit,
            "controls": normal_controls,
            "raw_bound": normal_bound,
        },
        "all_three_disabled_work_leaf": {
            "run_id": "wl-all-off-002",
            "workflow_result": "pass",
            "completed_features": 3,
            "turn_audit": all_off_turns,
            "rollout_profile": verify_rollout_profile(
                ALL_OFF_ARTIFACT / "observation/rollout-metadata.jsonl"
            ),
            "context_window_evidence": usage_window_evidence(all_off_server),
            "missing_usage_cap_audit": all_off_cap_audit,
            "controls": all_off_controls,
            "non_provider_capture_gap": verify_all_off_non_provider_gap(),
            "raw_bound": all_off_bound,
        },
        "conclusion": {
            "normal_work_leaf_raw_saving_proven_for_selected_run": True,
            "all_three_disabled_raw_saving_proven_for_selected_run": True,
            "saving_fully_explained_by_three_candidate_mechanisms": False,
            "exact_raw_reduction_known": False,
            "uncached_reduction_known": False,
            "population_average_known": False,
        },
        "source_sha256": hashes,
    }


if __name__ == "__main__":
    output = analyze()
    (STUDY_DIR / "evidence.json").write_text(json.dumps(output, indent=2) + "\n")
    print(json.dumps(output, indent=2))
