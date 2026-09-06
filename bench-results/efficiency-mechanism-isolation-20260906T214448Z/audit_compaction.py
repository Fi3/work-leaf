#!/usr/bin/env python3
"""Supplemental, provider-free native response/compaction inventory (version 1).

Only sources named and SHA-256 matched by the run's rollout metadata are read.
Every manifest outcome is retained. Exact response identities, not token-counter
advances, define the observed native ledger. An adjacent compaction marker is an
explicit association candidate, not proof of the provider's internal call type.
The audit never rewrites or automatically retotals the frozen primary analysis.
Equal observed prefixes cannot establish absent hidden work or completed tails.
"""
import argparse
import hashlib
import json
from pathlib import Path

FIELDS = ("input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens")
CAMEL = dict(zip(FIELDS, ("inputTokens", "cachedInputTokens", "outputTokens", "reasoningOutputTokens")))


def valid_identity(value):
    return isinstance(value, str) and bool(value)


def checked_usage(value, total_required=False):
    if not isinstance(value, dict):
        raise ValueError("missing usage object")
    usage = {key: value.get(key) for key in FIELDS}
    if any(type(v) is not int or v < 0 for v in usage.values()):
        raise ValueError("four usage fields must be nonnegative integers")
    if usage["cached_input_tokens"] > usage["input_tokens"] or usage["reasoning_output_tokens"] > usage["output_tokens"]:
        raise ValueError("cached/reasoning usage exceeds containing field")
    if total_required or "total_tokens" in value:
        if type(value.get("total_tokens")) is not int or value["total_tokens"] != usage["input_tokens"] + usage["output_tokens"]:
            raise ValueError("total_tokens must equal input plus output")
    if "cache_write_input_tokens" in value and (type(value["cache_write_input_tokens"]) is not int or value["cache_write_input_tokens"] != 0):
        raise ValueError("nonzero or invalid cache-write scope is unsupported")
    return usage


def summed(values):
    result = dict.fromkeys(FIELDS, 0)
    for value in values:
        for key in FIELDS:
            result[key] += value[key]
    return result


def metrics(usage):
    return {**usage, "raw_input_plus_output": usage["input_tokens"] + usage["output_tokens"],
            "uncached_input_plus_output": usage["input_tokens"] - usage["cached_input_tokens"] + usage["output_tokens"]}


def audit_rollout(rows, metadata, model, effort):
    """One-pass identity and cumulative checks; content bodies are never exported."""
    thread = metadata.get("thread_id")
    errors, records, markers, contexts, turns = [], {}, [], {}, {}
    thread_sum = dict.fromkeys(FIELDS, 0)
    duplicates, session_count, previous = 0, 0, None
    if not valid_identity(thread) or metadata.get("model") != model or metadata.get("effort") != effort:
        errors.append("metadata thread/model/effort scope mismatch")
    for line, row in enumerate(rows, 1):
        kind, payload = row.get("type"), row.get("payload", {})
        prefix = f"line {line}: "
        adjacent = previous
        previous = None
        if kind in ("session_meta", "turn_context", "token_usage_record", "compacted") and not isinstance(payload, dict):
            errors.append(prefix + "unsupported native payload shape")
            continue
        if kind == "session_meta":
            session_count += 1
            if (not valid_identity(payload.get("id")) or not valid_identity(payload.get("session_id", thread))
                    or payload.get("id") != thread or payload.get("session_id", thread) != thread):
                errors.append(prefix + "session identity mismatch")
        elif kind == "turn_context":
            turn = payload.get("turn_id")
            scope = (payload.get("model"), payload.get("effort"))
            if not valid_identity(turn):
                errors.append(prefix + "turn identity must be a nonempty string")
                continue
            if scope != (model, effort) or (turn in contexts and contexts[turn] != scope):
                errors.append(prefix + "turn model/effort scope mismatch")
            contexts[turn] = scope
        elif kind == "token_usage_record":
            try:
                response_id, turn = payload.get("response_id"), payload.get("turn_id")
                if not valid_identity(response_id):
                    raise ValueError("missing response identity")
                if (not valid_identity(payload.get("thread_id")) or not valid_identity(payload.get("session_id"))
                        or payload.get("thread_id") != thread or payload.get("session_id") != thread):
                    raise ValueError("native thread/session identity mismatch")
                if not valid_identity(turn) or contexts.get(turn) != (model, effort):
                    raise ValueError("native response lacks matching turn model/effort")
                usage = checked_usage(payload.get("usage"), True)
                cumulative = checked_usage(payload.get("thread_token_usage"), True)
                turn_cumulative = checked_usage(payload.get("turn_token_usage"), True)
                record = {"response_id": response_id, "thread_id": thread, "turn_id": turn, "usage": usage}
                if response_id in records:
                    if any(records[response_id][key] != record[key] for key in record):
                        raise ValueError("conflicting response identity")
                    duplicates += 1
                else:
                    records[response_id] = {**record, "line": line}
                    thread_sum = summed((thread_sum, usage))
                    turns[turn] = summed((turns.get(turn, dict.fromkeys(FIELDS, 0)), usage))
                if cumulative != thread_sum or turn_cumulative != turns[turn]:
                    errors.append(prefix + "native cumulative differs from unique response prefix")
                previous = response_id
            except (ValueError, TypeError) as error:
                errors.append(prefix + str(error))
        elif kind == "compacted":
            markers.append({"line": line, "adjacent_native_response_id": adjacent,
                            "association": "immediately_preceding_usage_record" if adjacent else "unknown"})
            if adjacent is None:
                errors.append(prefix + "unlinked compaction marker")
    if session_count != 1:
        errors.append("expected exactly one native session identity record")
    if not records:
        errors.append("no native response-usage records; coverage unknown")
    return {"thread_id": thread, "errors": errors, "records": records,
            "response_count": len(records), "duplicate_response_records": duplicates,
            "native_usage": metrics(thread_sum), "compaction_markers": markers}


def read_source(path, sources):
    data = path.read_bytes()
    sources.append({"path": str(path.resolve()), "sha256": hashlib.sha256(data).hexdigest()})
    return data


def json_rows(data):
    for line in data.splitlines():
        if line.strip():
            row = json.loads(line)
            if not isinstance(row, dict):
                raise ValueError("JSONL record must be an object")
            yield row


def raw_records(observation, sources, errors):
    """Index raw identities once; full capture provenance remains a primary gate."""
    records = {}
    paths = sorted((observation / "app-server").glob("*/server-to-client.raw"))
    for path in paths:
        try:
            for line, row in enumerate(json_rows(read_source(path, sources)), 1):
                if row.get("method") != "rawResponse/completed":
                    continue
                params = row.get("params", {})
                if not isinstance(params, dict) or not isinstance(params.get("usage"), dict):
                    raise ValueError("unsupported raw response payload shape")
                raw = params.get("usage", {})
                value = {key: raw.get(camel) for key, camel in CAMEL.items()}
                value["total_tokens"] = raw.get("totalTokens")
                if "cacheWriteInputTokens" in raw:
                    value["cache_write_input_tokens"] = raw["cacheWriteInputTokens"]
                usage = checked_usage(value, True)
                response_id = params.get("responseId")
                if not all(valid_identity(params.get(key)) for key in ("responseId", "threadId", "turnId")):
                    raise ValueError("raw response missing identity")
                record = {"response_id": response_id, "thread_id": params["threadId"],
                          "turn_id": params["turnId"], "usage": usage}
                if response_id in records and any(records[response_id][key] != record[key] for key in record):
                    raise ValueError("conflicting raw response identity")
                records.setdefault(response_id, {**record, "source": str(path), "line": line})
        except (OSError, ValueError, TypeError) as error:
            errors.append(f"raw capture {path}: {error}")
    return records, "available" if paths else "unavailable"


def audit_run(run, model, effort, sessions_root):
    result = {"run_id": run.get("run_id", run.get("id")), "condition": run.get("condition"),
              "launch_status": run.get("launch_status"), "launcher_exit_code": run.get("launcher_exit_code"),
              "status": "unknown", "errors": [], "threads": [], "sources": [],
              "adjusted_primary_total": None, "whole_workflow_coverage_established": False,
              "coverage_limit": "Observed identity reconciliation only; hidden work and unfinished tails require the frozen primary gates."}
    errors, sources = result["errors"], result["sources"]
    try:
        if not run.get("artifact"):
            raise ValueError("no artifact; admitted outcome retained without coverage")
        observation = Path(run["artifact"]) / "observation"
        analysis = json.loads(read_source(observation / "analysis.json", sources))
        if not isinstance(analysis, dict):
            raise ValueError("unsupported observer analysis shape")
        metadata_rows = list(json_rows(read_source(observation / "rollout-metadata.jsonl", sources)))
        observer = {}
        for row in analysis.get("threads", []):
            if not isinstance(row, dict):
                raise ValueError("unsupported observer thread row shape")
            if not valid_identity(row.get("thread_id")):
                raise ValueError("observer thread identity must be a nonempty string")
            if row["thread_id"] in observer:
                raise ValueError("duplicate observer thread identity")
            observer[row["thread_id"]] = row
        if not observer or not metadata_rows:
            errors.append("empty observer/native inventory; coverage unknown")
        if analysis.get("session_only_threads"):
            errors.append("observer session-only threads require separate scope adjudication")
        raw, result["app_raw_status"] = raw_records(observation, sources, errors)
        native_ids, matched_ids, seen_threads = set(), set(), set()
        native_total = dict.fromkeys(FIELDS, 0)
        sessions_root = Path(sessions_root).resolve()
        for metadata in metadata_rows:
            thread = metadata.get("thread_id")
            if not valid_identity(thread) or thread in seen_threads:
                errors.append("missing or duplicate metadata thread identity")
                continue
            seen_threads.add(thread)
            try:
                if thread not in observer:
                    raise ValueError("native metadata thread absent from observer scope")
                relative = Path(metadata["source_relative_path"])
                source = (sessions_root / relative).resolve()
                if relative.is_absolute() or not source.is_relative_to(sessions_root):
                    raise ValueError("native source path escapes sessions root")
                data = read_source(source, sources)
                if hashlib.sha256(data).hexdigest() != metadata.get("source_sha256"):
                    raise ValueError("native source SHA-256 mismatch")
                item = audit_rollout(json_rows(data), metadata, model, effort)
                item["source"] = str(source)
                item["scope"] = {key: metadata.get(key) for key in ("primary", "visible", "descendant")}
                for key in ("primary", "visible"):
                    if metadata.get(key) != observer[thread].get(key):
                        item["errors"].append(f"observer/metadata {key} scope mismatch")
                if metadata.get("descendant"):
                    item["errors"].append("descendant scope requires separate adjudication")
                observed = checked_usage(observer[thread].get("usage"))
                item["observer_usage"] = metrics(observed)
                item["native_minus_observer"] = metrics({key: item["native_usage"][key] - observed[key] for key in FIELDS})
                item["native_only_response_ids"] = []
                for identity, record in item["records"].items():
                    if identity in native_ids:
                        item["errors"].append("response identity repeated across native threads")
                    native_ids.add(identity)
                    counterpart = raw.get(identity)
                    if counterpart is None:
                        item["native_only_response_ids"].append(identity)
                    elif all(record[key] == counterpart[key] for key in ("thread_id", "turn_id", "usage")):
                        matched_ids.add(identity)
                    else:
                        item["errors"].append(f"native/raw identity usage or scope mismatch: {identity}")
                item["native_only_usage"] = metrics(summed(item["records"][identity]["usage"] for identity in item["native_only_response_ids"]))
                for marker in item["compaction_markers"]:
                    marker["raw_identity_match"] = marker["adjacent_native_response_id"] in matched_ids
                native_total = summed((native_total, item["native_usage"]))
                errors.extend(f"thread {thread}: {error}" for error in item["errors"])
                result["threads"].append(item)
            except (OSError, ValueError, KeyError, TypeError) as error:
                errors.append(f"thread {thread}: {error}")
                result["threads"].append({"thread_id": thread, "status": "unknown", "errors": [str(error)]})
        result["observer_threads_without_native_source"] = sorted(set(observer) - seen_threads)
        if result["observer_threads_without_native_source"]:
            errors.append("observer threads lack exact native source")
        result["raw_only_response_ids"] = sorted(set(raw) - native_ids)
        result["native_response_count"] = len(native_ids)
        result["raw_response_count"] = len(raw)
        result["identity_matched_response_count"] = len(matched_ids)
        result["native_usage"] = metrics(native_total)
        observer_total = summed(checked_usage(row.get("usage")) for row in observer.values())
        result["observer_usage"] = metrics(observer_total)
        result["native_minus_observer"] = metrics({key: native_total[key] - observer_total[key] for key in FIELDS})
        result["observed_counter_scope"] = "unknown" if errors else ("matching_prefix" if native_total == observer_total else "different")
        result["observer_capture_complete"] = analysis.get("capture_complete")
        result["compaction_marker_count"] = sum(len(row.get("compaction_markers", [])) for row in result["threads"])
        result["compaction_exercised"] = result["compaction_marker_count"] > 0
        markers = [marker for row in result["threads"] for marker in row.get("compaction_markers", [])]
        result["compaction_raw_coverage"] = (
            "unknown" if errors else "not_exercised" if not markers else
            "raw_capture_unavailable" if result["app_raw_status"] != "available" else
            "adjacent_candidates_identity_matched" if all(marker["raw_identity_match"] for marker in markers) else
            "native_candidates_outside_raw_ledger")
        result["status"] = "unknown" if errors else "observed_prefix_audited"
    except (OSError, ValueError, KeyError, TypeError) as error:
        errors.append(str(error))
    return result


def audit_manifest(path, sessions_root):
    sources = []
    manifest = json.loads(read_source(Path(path), sources))
    if not manifest.get("model") or not manifest.get("reasoning_effort") or not isinstance(manifest.get("runs"), list):
        raise ValueError("manifest requires frozen model/effort and runs array")
    runs = [audit_run(row, manifest["model"], manifest["reasoning_effort"], sessions_root) for row in manifest["runs"]]
    read_source(Path(__file__), sources)
    return {"schema": "work-leaf-supplemental-compaction-audit-v1", "phase": manifest.get("phase"),
            "scope": "Supplemental observed native response inventory; no primary retotal or causal estimates.",
            "sources": sources, "runs": runs}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--sessions-root", type=Path, default=Path.home() / ".codex" / "sessions")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = audit_manifest(args.manifest, args.sessions_root)
    with args.output.open("x") as output:
        json.dump(result, output, indent=2)
        output.write("\n")
    print(json.dumps({"output": str(args.output), "runs": [{"run_id": row["run_id"], "status": row["status"], "errors": row["errors"]} for row in result["runs"]]}))


if __name__ == "__main__":
    main()
