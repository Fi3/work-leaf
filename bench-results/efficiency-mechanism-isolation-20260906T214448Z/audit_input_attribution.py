#!/usr/bin/env python3
"""Supplemental exact observed retained-item charges; no primary retotal.

Sum top-level attribution items and request_fields, never their nested content.
Repeated charges mean the same recorded thread/item identity appears as input in
multiple completed response records. They are not an exhaustive call inventory,
a counterfactual cue-token price, or a causal share of between-run savings.
"""
import argparse
from collections import defaultdict
from functools import lru_cache
import hashlib
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIELDS = ("input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens", "cache_write_input_tokens")
COUNTS = {"input_tokens": "input_tokens", "cached_tokens": "cached_input_tokens",
          "output_tokens": "output_tokens", "cache_write_tokens": "cache_write_input_tokens"}
CAMEL = dict(zip(FIELDS, ("inputTokens", "cachedInputTokens", "outputTokens", "reasoningOutputTokens", "cacheWriteInputTokens")))
NONREASONING = {"message", "function_call", "custom_tool_call", "web_search_call"}
ACTION_FIELDS = ("kind", "role", "name", "call_id", "turn_id", "directive_names", "source", "line", "payload_sha256")
INPUT_FIELDS = ("kind", "role", "name", "call_id", "source", "line", "payload_sha256")
PINS = {"audit_compaction.py": "dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153",
        "analyze.py": "2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78"}


def valid_id(value):
    return isinstance(value, str) and bool(value)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def read_source(path, sources):
    data = Path(path).read_bytes()
    sources[str(Path(path).resolve())] = digest(data)
    return data


def json_rows(data):
    for line, text in enumerate(data.splitlines(), 1):
        if text.strip():
            row = json.loads(text)
            if not isinstance(row, dict):
                raise ValueError(f"line {line}: unsupported JSONL shape")
            yield line, row


@lru_cache(maxsize=1)
def dependencies():
    modules = []
    for filename, expected in PINS.items():
        path = HERE / filename
        if digest(path.read_bytes()) != expected:
            raise ValueError(f"pinned supplemental dependency changed: {filename}")
        spec = importlib.util.spec_from_file_location("input_attribution_" + path.stem, path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        modules.append(module)
    return tuple(modules)


def observed_usage(raw):
    if not isinstance(raw, dict):
        raise ValueError("missing recorded usage")
    value = {field: raw.get(camel) for field, camel in CAMEL.items()}
    if any(type(count) is not int or count < 0 for count in value.values()):
        raise ValueError("recorded usage must contain nonnegative integer fields")
    if value["cached_input_tokens"] + value["cache_write_input_tokens"] > value["input_tokens"] or value["reasoning_output_tokens"] > value["output_tokens"]:
        raise ValueError("recorded usage subsets exceed their containing field")
    if type(raw.get("totalTokens")) is not int or raw["totalTokens"] != value["input_tokens"] + value["output_tokens"]:
        raise ValueError("recorded total differs from input plus output")
    return value


def item_counts(raw):
    if not isinstance(raw, dict) or set(raw) - set(COUNTS) - {"content"}:
        raise ValueError("unsupported attribution counter shape")
    result = {target: raw.get(source) for source, target in COUNTS.items()}
    if any(type(count) is not int or count < 0 for count in result.values()):
        raise ValueError("attribution fields must be nonnegative integers")
    if result["cached_input_tokens"] + result["cache_write_input_tokens"] > result["input_tokens"]:
        raise ValueError("attribution cached/write counts exceed input")
    return result


def add_counts(target, value):
    for key, count in value.items():
        target[key] += count


def native_input_link(native):
    if not isinstance(native, dict) or not valid_id(native.get("kind")):
        return {"status": "unlinked", "metadata": None}
    return {"status": "linked", "metadata": {key: native[key] for key in INPUT_FIELDS if key in native}}


def extract_records(servers, native_items, native_ledger):
    """Linear in captured metadata size; identity-indexed joins, no text matching."""
    errors, raw, conflicts, duplicates, anonymous = [], {}, set(), 0, []
    for sequence, row in enumerate(servers):
        if row.get("method") != "rawResponse/completed":
            continue
        params = row.get("params")
        try:
            if not isinstance(params, dict) or not all(valid_id(params.get(key)) for key in ("threadId", "turnId", "responseId")):
                raise ValueError("response/thread/turn identities must be nonempty strings")
            response_id = params["responseId"]
            metadata = params.get("usageMetadata")
            attribution = metadata.get("metadata", {}).get("attribution") if isinstance(metadata, dict) and isinstance(metadata.get("metadata"), dict) else None
            fingerprint = digest(canonical([params["threadId"], params["turnId"], params.get("usage"), attribution]))
            entry = {"params": params, "attribution": attribution, "fingerprint": fingerprint,
                     "sequence": sequence, "source": row.get("_audit_source"), "line": row.get("_audit_line")}
            if response_id in raw:
                if raw[response_id]["fingerprint"] != fingerprint:
                    conflicts.add(response_id)
                    errors.append(f"conflicting response identity: {response_id}")
                else:
                    duplicates += 1
            else:
                raw[response_id] = entry
        except (ValueError, TypeError) as error:
            errors.append(str(error))
            anonymous.append({"status": "unknown", "sequence": sequence, "errors": [str(error)]})
    output_owners, output_conflicts = {}, set()
    for response_id, entry in raw.items():
        attribution = entry["attribution"]
        items = attribution.get("items") if isinstance(attribution, dict) else None
        if not isinstance(items, dict):
            continue
        for item_id, item in items.items():
            if not valid_id(item_id) or not isinstance(item, dict) or type(item.get("output_tokens")) is not int or item["output_tokens"] <= 0:
                continue
            key = entry["params"]["threadId"], item_id
            prior = output_owners.setdefault(key, response_id)
            if prior != response_id:
                output_conflicts.update((prior, response_id))
    responses, repeated = [], {}
    for response_id, entry in raw.items():
        params, attribution = entry["params"], entry["attribution"]
        thread, turn = params["threadId"], params["turnId"]
        record = {"response_id": response_id, "thread_id": thread, "turn_id": turn,
                  "source": entry["source"], "line": entry["line"], "sequence": entry["sequence"],
                  "attribution_sha256": digest(canonical(attribution)), "status": "unknown", "errors": [],
                  "input_items": [], "output_items": [], "request_fields": {}}
        issues = record["errors"]
        try:
            usage = observed_usage(params.get("usage"))
            record["usage"] = usage
            native = native_ledger.get(response_id)
            if (not isinstance(native, dict) or native.get("thread_id") != thread or native.get("turn_id") != turn
                    or not isinstance(native.get("usage"), dict)
                    or any(type(native["usage"].get(key)) is not int or native["usage"][key] != usage[key] for key in FIELDS[:4])):
                issues.append("response lacks matching native identity/scope/four-field usage")
            if response_id in conflicts:
                issues.append("conflicting duplicate response metadata")
            if response_id in output_conflicts:
                issues.append("output item assigned to multiple response identities")
            if not isinstance(attribution, dict) or set(attribution) != {"items", "request_fields"} or not all(isinstance(attribution[key], dict) for key in attribution):
                raise ValueError("missing/unsupported attribution items or request_fields")
            totals = dict.fromkeys(FIELDS, 0)
            overhead = dict.fromkeys(FIELDS, 0)
            reasoning_known = True
            for item_id, item in attribution["items"].items():
                if not valid_id(item_id):
                    raise ValueError("attribution item identity must be a nonempty string")
                counts = item_counts(item)
                add_counts(totals, counts)
                if counts["input_tokens"]:
                    link = native_input_link(native_items.get((thread, item_id)))
                    record["input_items"].append({"item_id": item_id, **counts, "native_link_status": link["status"]})
                if counts["output_tokens"]:
                    native_item = native_items.get((thread, item_id))
                    action = {key: native_item[key] for key in ACTION_FIELDS if key in native_item} if isinstance(native_item, dict) else None
                    record["output_items"].append({"item_id": item_id, "output_tokens": counts["output_tokens"], "action": action})
                    if (not action or action.get("turn_id") != turn or action.get("kind") not in NONREASONING | {"reasoning"}
                            or (action.get("kind") == "message" and action.get("role") != "assistant")):
                        reasoning_known = False
                        issues.append(f"unmatched/unsupported output item scope: {item_id}")
                    elif action["kind"] == "reasoning":
                        totals["reasoning_output_tokens"] += counts["output_tokens"]
            for field, item in attribution["request_fields"].items():
                if not valid_id(field):
                    raise ValueError("request-field identity must be a nonempty string")
                counts = item_counts(item)
                record["request_fields"][field] = counts
                add_counts(overhead, counts)
                add_counts(totals, counts)
                if counts["output_tokens"]:
                    reasoning_known = False
                    issues.append("request-field output has unknown reasoning classification")
            record["request_field_usage"] = overhead
            record["attributed_usage"] = {**totals, "reasoning_output_tokens": totals["reasoning_output_tokens"] if reasoning_known else None}
            record["residual"] = {key: usage[key] - value if value is not None else None for key, value in record["attributed_usage"].items()}
            if any(value != 0 for value in record["residual"].values()):
                issues.append("attribution does not exactly reconcile to recorded usage")
            record["status"] = "exact_observed_attribution" if not issues else "unknown"
        except (ValueError, TypeError, KeyError) as error:
            issues.append(str(error))
        for charge in record["input_items"]:
            key = (thread, charge["item_id"])
            if key not in repeated:
                repeated[key] = {"thread_id": thread, "item_id": charge["item_id"], "response_charge_count": 0,
                                 "input_tokens": 0, "cached_input_tokens": 0, "cache_write_input_tokens": 0,
                                 "repeated_after_first_input_tokens": 0, "first_observed_response_id": response_id,
                                 "last_observed_response_id": response_id, "all_charge_responses_exact": True,
                                 "native_link": native_input_link(native_items.get(key))}
            summary = repeated[key]
            if summary["response_charge_count"]:
                summary["repeated_after_first_input_tokens"] += charge["input_tokens"]
            summary["response_charge_count"] += 1
            summary["last_observed_response_id"] = response_id
            for field in ("input_tokens", "cached_input_tokens", "cache_write_input_tokens"):
                summary[field] += charge[field]
            summary["all_charge_responses_exact"] &= record["status"] == "exact_observed_attribution"
        errors.extend(f"response {response_id}: {error}" for error in issues)
        responses.append(record)
    if not raw:
        errors.append("no completed raw response attribution; coverage unknown")
    return {"status": "unknown" if errors else "exact_observed_attribution", "errors": errors,
            "responses": responses, "unidentified_response_records": anonymous,
            "duplicate_responses": duplicates, "native_response_ids_without_raw": sorted(set(native_ledger) - set(raw)),
            "repeated_input_items": list(repeated.values()),
            "interpretation": "Observed per-response charges only; no primary retotal, exhaustive call count, or causal savings share."}


def native_item_inventory(base, sources):
    items, ledger, errors = {}, {}, []
    for thread in base.get("threads", []):
        if thread.get("errors") or not thread.get("source"):
            continue
        thread_id, source = thread["thread_id"], Path(thread["source"])
        ledger.update(thread.get("records", {}))
        expected = sources.get(str(source.resolve()))
        data = read_source(source, sources)
        if expected is None or digest(data) != expected:
            errors.append(f"native source differs from identity audit: {source}")
            continue
        turn = None
        for line, row in json_rows(data):
            payload = row.get("payload")
            if row.get("type") == "turn_context" and isinstance(payload, dict):
                turn = payload.get("turn_id")
            if row.get("type") != "response_item":
                continue
            if not isinstance(payload, dict):
                errors.append(f"native item payload shape unknown: {source}:{line}")
                continue
            item_id = payload.get("id")
            if item_id is None:
                continue  # Some input/tool-result items have no native ID; no guessed identity.
            if not valid_id(item_id):
                errors.append(f"native item identity is not a string: {source}:{line}")
                continue
            item = {"kind": payload.get("type"), "turn_id": turn, "source": str(source),
                    "line": line, "payload_sha256": digest(canonical(payload))}
            for key in ("name", "call_id", "role"):
                if key in payload:
                    if not valid_id(payload[key]):
                        errors.append(f"native item {key} has unsupported identity: {source}:{line}")
                    else:
                        item[key] = payload[key]
            if payload.get("type") == "message":
                directives = []
                content = payload.get("content", [])
                if isinstance(content, list):
                    for part in content:
                        if not isinstance(part, dict) or not isinstance(part.get("text"), str):
                            continue
                        for text in part["text"].splitlines():
                            if text.startswith("@work-leaf "):
                                rest = text[len("@work-leaf "):]
                                name = "locks run" if rest.startswith("locks run ") else rest.split(" ", 1)[0]
                                if name in {"read", "classify", "locks run", "edit", "patch", "done", "discard-command-changes"}:
                                    directives.append(name)
                item["directive_names"] = directives
            key = thread_id, item_id
            if key in items and items[key]["payload_sha256"] != item["payload_sha256"]:
                errors.append(f"conflicting native item identity: {thread_id}/{item_id}")
            else:
                items.setdefault(key, item)
    return items, ledger, errors


def audit_run(run, model, effort, sessions_root):
    result = {"run_id": run.get("run_id", run.get("id")), "condition": run.get("condition"),
              "launch_status": run.get("launch_status"), "launcher_exit_code": run.get("launcher_exit_code"),
              "status": "unknown", "errors": [], "source_sha256": {}, "attribution": None}
    errors, sources = result["errors"], result["source_sha256"]
    try:
        if not run.get("artifact"):
            raise ValueError("no artifact; admitted outcome retained")
        compaction, primary = dependencies()
        for filename, expected in PINS.items():
            sources[str(HERE / filename)] = expected
        base = compaction.audit_run(run, model, effort, sessions_root)
        sources.update({row["path"]: row["sha256"] for row in base["sources"]})
        errors.extend(base["errors"])
        result["native_compaction_scope"] = {key: base.get(key) for key in (
            "status", "app_raw_status", "native_response_count", "raw_response_count",
            "identity_matched_response_count", "compaction_marker_count", "compaction_raw_coverage", "observed_counter_scope")}
        native, ledger, native_errors = native_item_inventory(base, sources)
        errors.extend(native_errors)
        paths = sorted((Path(run["artifact"]) / "observation" / "app-server").glob("*/server-to-client.raw"))
        for path in paths:
            provenance = primary.capture_provenance(path.parent)
            sources.update(provenance["source_sha256"])
            errors.extend(provenance["errors"])
        def servers():
            for path in paths:
                expected = sources.get(str(path.resolve()))
                data = read_source(path, sources)
                if expected is None or digest(data) != expected:
                    raise ValueError(f"raw capture differs from provenance audit: {path}")
                for line, row in json_rows(data):
                    if row.get("method") == "rawResponse/completed":
                        yield {**row, "_audit_source": str(path), "_audit_line": line}
        attribution = extract_records(servers(), native, ledger)
        errors.extend(attribution["errors"])
        result["attribution"] = attribution
        result["status"] = "unknown" if errors else "exact_observed_attribution"
        result["summary"] = {"completed_response_records": len(attribution["responses"]),
                             "exact_attribution_response_records": sum(row["status"] == "exact_observed_attribution" for row in attribution["responses"]),
                             "input_item_identities": len(attribution["repeated_input_items"]),
                             "input_item_identities_charged_multiple_times": sum(row["response_charge_count"] > 1 for row in attribution["repeated_input_items"]),
                             "most_repeated_items": sorted(attribution["repeated_input_items"], key=lambda row: row["repeated_after_first_input_tokens"], reverse=True)[:10]}
    except (OSError, ValueError, TypeError, KeyError) as error:
        errors.append(str(error))
    return result


def audit_manifest(path, sessions_root):
    sources = {}
    manifest = json.loads(read_source(path, sources))
    if not isinstance(manifest, dict) or not valid_id(manifest.get("model")) or not valid_id(manifest.get("reasoning_effort")) or not isinstance(manifest.get("runs"), list):
        raise ValueError("manifest requires model, effort, and admitted runs")
    runs = []
    for row in manifest["runs"]:
        if not isinstance(row, dict):
            runs.append({"run_id": None, "status": "unknown", "errors": ["unsupported manifest run shape"]})
        else:
            runs.append(audit_run(row, manifest["model"], manifest["reasoning_effort"], sessions_root))
    read_source(Path(__file__), sources)
    return {"schema": "work-leaf-supplemental-input-attribution-v1", "phase": manifest.get("phase"),
            "source_sha256": sources, "runs": runs, "adjusted_primary_total": None,
            "causal_savings_share": None, "whole_workflow_coverage_established": False,
            "scope": "Exact observed metadata decomposition where validated; no counterfactual cue price, hidden-call coverage, or causal share. All admitted outcomes retained."}


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
    print(json.dumps({"output": str(args.output), "runs": [
        {"run_id": row["run_id"], "status": row["status"], "errors": row["errors"],
         "response_records": row.get("summary", {}).get("completed_response_records")} for row in result["runs"]]}))


if __name__ == "__main__":
    main()
