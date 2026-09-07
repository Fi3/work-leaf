#!/usr/bin/env python3
"""Complete descriptive v3 read/item/tool census; never a primary or cost ranking."""
import argparse
import ast
from collections import defaultdict, deque
from functools import lru_cache
import hashlib
import json
import os
from pathlib import Path
import stat
import sys
import types

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PINS = {"analyze_untracked_reads.py": "30a58a9312e2f9c641643698f53e0592c392fa8ed5fa3c0d01d9cc61ff427c3e",
        "audit_input_attribution.py": "ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740"}
MAX_FILE = 1024**3
MAX_TOTAL = 32*1024**3
MAX_SOURCES = 20000
MAX_PATTERNS = 20000
MAX_PATTERN_BYTES = 4*1024**2
NO_EXPECTED_HASH = object()
CALLS = {"function_call", "custom_tool_call", "web_search_call"}
OUTPUTS = {"function_call_output", "custom_tool_call_output"}


def require(value, message):
    if not value:
        raise ValueError(message)


def identity(value):
    return isinstance(value, str) and bool(value)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def valid_sha(value):
    return isinstance(value, str) and len(value) == 64 and all(c in "0123456789abcdef" for c in value)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def fields(value, keys):
    return {key: value[key] for key in keys if isinstance(value, dict) and key in value
            and (value[key] is None or type(value[key]) in (str, int, bool))}


def safe_read(event, exposure, line):
    result = fields(exposure, ("sequence", "site", "agent_id", "status", "baseline_sha256", "inline_candidate_sha256", "selected_sha256",
        "eligible", "selected_candidate", "selected_bytes", "candidate_byte_delta", "capture", "client_line", "reply_line", "rpc_id", "thread_id", "turn_id", "rejection_code"))
    result.update(trace_line=line, sequence=event.get("sequence") if type(event.get("sequence")) is int else None,
                  site="file-read", errors=list(exposure.get("errors", [])),
                  status=exposure.get("status", "invalid_trace"))
    result["component"] = fields(event.get("component"), ("baseline_start", "baseline_end", "inline_start", "inline_end")) if isinstance(event.get("component"), dict) else None
    result["bundle"] = fields(event.get("bundle"), ("threshold_eligible", "write_succeeded", "path"))
    result["snapshot_evidence"] = [fields(v, ("path", "class", "bytes", "fnv64_label", "sha256", "inline_body_start", "inline_body_end")) for v in exposure.get("snapshot_evidence", [])]
    result["snapshot_metadata"] = [fields(v, ("path", "class", "bytes", "digest", "inline_body_start", "inline_body_end")) for v in event.get("snapshots", []) if isinstance(v, dict)] if isinstance(event.get("snapshots"), list) else []
    result["requested_paths"] = [v for v in event.get("requested_paths", []) if isinstance(v, str)] if isinstance(event.get("requested_paths"), list) else []
    result["failure_metadata"] = [{**fields(v, ("path",)), "diagnostic_sha256": sha(v["diagnostic"].encode()) if isinstance(v.get("diagnostic"), str) else None}
                                  for v in event.get("failures", []) if isinstance(v, dict)] if isinstance(event.get("failures"), list) else []
    result.update(native_user={"status": "not_delivered"}, charge_refs=[], same_turn_response_indices=[])
    return result


USAGE_KEYS = ("input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens", "cache_write_input_tokens",
              "raw_input_plus_output", "uncached_input_plus_output", "total_tokens")


def safe_response_evidence(record):
    if not isinstance(record, dict): return None
    return {**fields(record, ("thread_id", "turn_id")), "usage": fields(record.get("usage"), USAGE_KEYS),
            "sources": [fields(v, ("path", "native_line", "server_line")) for v in record.get("sources", []) if isinstance(v, dict)]}


def safe_accounting(record):
    measurement = record.get("measurement", {})
    gaps = record.get("gaps", measurement.get("gap_inventory", {}).get("gaps", []))
    return {"status": record.get("status"), "errors": record.get("errors", []),
        "measurement": {"status": measurement.get("status"), "recorded_usage": fields(measurement.get("recorded_usage"), USAGE_KEYS),
            "bounds": {k: fields(v, ("lower", "upper")) for k, v in (measurement.get("bounds") or {}).items() if k in USAGE_KEYS},
            "reasons": measurement.get("reasons", [])},
        "gaps": [fields(v, ("thread_id", "turn_id", "status", "response_count_upper", "proof", "reason", "previous_usage_sequence",
            "response_start_sequence", "directive_sequence", "completion_sequence", "paired_items", "post_directive_unfinished_items", "no_tool_boundary")) for v in gaps],
        "full_accounting_evidence": "Retained in the source accounting report; no response scope correction or gap estimate is recomputed here."}


def object_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON object key")
        result[key] = value
    return result


def decode(data):
    def invalid(_):
        raise ValueError("nonfinite JSON value")
    return json.loads(data, object_pairs_hook=object_pairs, parse_constant=invalid)


@lru_cache(maxsize=1)
def dependencies():
    modules = []
    for name, expected in PINS.items():
        data = SourceIndex().read(HERE/name)
        require(sha(data) == expected, "census pure dependency source differs: "+name)
        module = types.ModuleType("census_"+Path(name).stem); module.__file__ = str(HERE/name)
        if name == "audit_input_attribution.py":
            functions = {"valid_id", "digest", "canonical", "observed_usage", "item_counts", "add_counts", "native_input_link", "extract_records"}
            constants = {"FIELDS", "COUNTS", "CAMEL", "NONREASONING", "ACTION_FIELDS", "INPUT_FIELDS"}
            nodes = [node for node in ast.parse(data, str(HERE/name)).body
                     if isinstance(node, ast.FunctionDef) and node.name in functions
                     or isinstance(node, ast.Assign) and len(node.targets) == 1
                     and isinstance(node.targets[0], ast.Name) and node.targets[0].id in constants]
            require({n.name for n in nodes if isinstance(n, ast.FunctionDef)} == functions, "pure attribution definition inventory differs")
            module.__dict__.update(hashlib=hashlib, json=json)
            exec(compile(ast.Module(body=nodes, type_ignores=[]), str(HERE/name), "exec"), module.__dict__)
        else:
            exec(compile(data, str(HERE/name), "exec"), module.__dict__)
        modules.append(module)
    return tuple(modules)


def native_inventory(sources):
    items, users, ledger, calls, outputs, unknown, errors = {}, {}, {}, [], [], [], []
    item_payloads = {}; call_inputs = {}; user_groups = []; user_texts = []; scope_seen = set()
    for source in sources:
        path, thread = source["source"], source["thread_id"]
        require(identity(path) and identity(thread) and thread not in scope_seen, "duplicate/invalid native scope")
        scope_seen.add(thread); turn = None
        for line, row in source["rows"]:
            require(type(line) is int and line > 0 and isinstance(row, dict), "unsupported native physical record")
            payload = row.get("payload")
            if row.get("type") == "turn_context" and isinstance(payload, dict):
                turn = payload.get("turn_id")
                require(identity(turn), "invalid native turn context")
            if row.get("type") == "token_usage_record":
                require(isinstance(payload, dict), "unsupported native usage record")
                rid = payload.get("response_id")
                require(identity(rid) and payload.get("thread_id") == thread and identity(payload.get("turn_id")), "invalid native response identity/scope")
                record = {"thread_id": thread, "turn_id": payload["turn_id"], "usage": payload.get("usage")}
                require(rid not in ledger or ledger[rid] == record, "conflicting native response identity")
                ledger[rid] = record
            if row.get("type") != "response_item":
                continue
            require(isinstance(payload, dict), "unsupported native response item")
            kind, item_id = payload.get("type"), payload.get("id")
            require(identity(kind), "invalid native item kind")
            if item_id is not None:
                require(identity(item_id), "invalid native item identity")
            explicit_turn = payload.get("turn_id")
            require(explicit_turn is None or identity(explicit_turn), "invalid explicit native item turn")
            metadata = payload.get("internal_chat_message_metadata_passthrough")
            require(metadata is None or isinstance(metadata, dict), "invalid native item passthrough metadata")
            nested_turn = metadata.get("turn_id") if isinstance(metadata, dict) else None
            require(nested_turn is None or identity(nested_turn), "invalid explicit native passthrough turn")
            require(not (explicit_turn is not None and nested_turn is not None and explicit_turn != nested_turn), "contradictory explicit native item turns")
            explicit_turn = explicit_turn or nested_turn
            item = {"item_id": item_id, "thread_id": thread, "kind": kind, "source": path,
                    "line": line, "payload_sha256": sha(canonical(payload))}
            for key in ("role", "name", "call_id"):
                if key in payload:
                    require(identity(payload[key]), "invalid native public identity: "+key)
                    item[key] = payload[key]
            # User-item ownership is never inferred from the preceding context.
            item["turn_id"] = explicit_turn if item.get("role") == "user" else explicit_turn or turn
            if item_id is not None:
                key = thread, item_id
                require(key not in item_payloads or item_payloads[key] == item["payload_sha256"], "conflicting native item identity")
                if key in items:
                    continue
                item_payloads[key] = item["payload_sha256"]; items[key] = item
            if kind == "message" and item.get("role") == "user":
                content = payload.get("content")
                if (item_id is not None and isinstance(content, list) and len(content) == 1
                        and isinstance(content[0], dict) and content[0].get("type") in {"input_text", "text"}
                        and isinstance(content[0].get("text"), str)):
                    text = content[0]["text"]; digest = sha(text.encode())
                    key = thread, item["turn_id"], digest
                    if key not in users:
                        users[key] = len(user_groups); user_texts.append(text)
                        user_groups.append({"thread_id": thread, "explicit_turn_id": item["turn_id"], "text_sha256": digest, "candidates": []})
                    group = users[key]
                    require(user_texts[group] == text, "native text digest collision")
                    user_groups[group]["candidates"].append(item)
                else:
                    unknown.append({**item, "projection": "unsupported native user text shape"})
            elif kind in CALLS:
                number = len(calls); calls.append({**item, "output_indices": [], "bundle_references": []})
                raw = payload.get("arguments", payload.get("input"))
                if isinstance(raw, str):
                    data = raw.encode(); call_inputs[number] = data
                    calls[-1].update(argument_bytes=len(data), argument_sha256=sha(data))
                else:
                    calls[-1]["argument_projection"] = "unknown/non-string public arguments"
            elif kind in OUTPUTS:
                text = payload.get("output")
                outputs.append({**item, "body_bytes": len(text.encode()) if isinstance(text, str) else None,
                                "body_sha256": sha(text.encode()) if isinstance(text, str) else None,
                                "body_projection": "not exported; truncation/content semantics require source review"})
            elif kind not in {"message", "reasoning"}:
                unknown.append({**item, "projection": "unknown native activity; body not exported"})
    return {"items": items, "users": users, "user_groups": user_groups, "user_texts": user_texts, "ledger": ledger,
            "calls": calls, "outputs": outputs, "unknown": unknown, "call_inputs": call_inputs, "errors": errors}


def derive_run(run_id, condition, trace, threads, captures, native_sources, accounting, trace_lines=None):
    delivery, arithmetic = dependencies()
    result = {"run_id": run_id, "condition": condition, "errors": [], "reads": [], "responses": [],
              "tool_calls": [], "tool_outputs": [], "unknown_native_items": [],
              "accounting": safe_accounting(accounting), "causal_share": None, "counterfactual_price": None,
              "retrieval_coverage": "Explicit issued-path references only; indirect retrieval and actual file opens require public source review."}
    errors = result["errors"]
    inventory = delivery.prompt_inventory(trace, threads, captures, run_id, condition, trace_lines)
    result["delivery_status"] = inventory["status"]; errors.extend(inventory["errors"])
    physical = trace_lines if trace_lines is not None else list(range(1, len(trace)+1))
    require(len(physical) == len(trace) and len(set(physical)) == len(physical), "invalid trace physical line map")
    exposures = {v["trace_line"]: v for v in inventory["exposures"]}; events = {}
    for line, event in zip(physical, trace):
        if isinstance(event, dict) and event.get("event") == "read-response":
            events[line] = event
            result["reads"].append(safe_read(event, exposures.get(line, {}), line))
    try:
        native = native_inventory(native_sources)
        errors.extend(native["errors"])
        require({s["thread_id"] for s in native_sources} == {t["thread_id"] for t in threads}, "native/captured thread inventories differ")
    except (ValueError, KeyError, TypeError) as error:
        errors.append(str(error))
        for read in result["reads"]:
            if read.get("status") == "delivered": read["native_user"] = {"status": "unlinked", "reason": "native inventory invalid"}
        return result
    expected = accounting.get("exact_response_evidence", {})
    if accounting.get("status") != "validated" or accounting.get("errors"):
        errors.append("independent whole-workflow accounting is not validated")
    if set(expected) != set(native["ledger"]): errors.append("native response census differs from independent accounting")
    for rid, record in native["ledger"].items():
        other = expected.get(rid, {})
        if any(record.get(k) != other.get(k) for k in ("thread_id", "turn_id")) or any(
                type(record.get("usage", {}).get(k)) is not int or record["usage"][k] != other.get("usage", {}).get(k) for k in arithmetic.FIELDS[:4]):
            errors.append("native response identity/usage differs from independent accounting: "+rid)
    def raw_frames():
        for capture in captures:
            lines = capture.get("server_lines", range(1, len(capture["servers"])+1))
            for line, row in zip(lines, capture["servers"]):
                if row.get("method") == "rawResponse/completed":
                    yield {**row, "_audit_source": capture["path"]+"/server-to-client.raw", "_audit_line": line}
    attribution = arithmetic.extract_records(raw_frames(), native["items"], native["ledger"])
    errors.extend(attribution["errors"]); result["responses"] = attribution["responses"]
    result["attribution_status"] = attribution["status"]
    result["unidentified_response_records"] = attribution["unidentified_response_records"]
    result["native_response_ids_without_raw"] = attribution["native_response_ids_without_raw"]
    charges = defaultdict(list); by_turn = defaultdict(list)
    for i, response in enumerate(result["responses"]):
        by_turn[(response["thread_id"], response["turn_id"])].append(i)
        response["independent_response_evidence"] = safe_response_evidence(expected.get(response["response_id"]))
        for j, item in enumerate(response["input_items"]):
            charges[(response["thread_id"], item["item_id"])].append({"response_index": i, "input_item_index": j})
    result["input_items"] = []
    for key, refs in charges.items():
        result["input_items"].append({"thread_id": key[0], "item_id": key[1], "charge_refs": refs,
            "native_link": arithmetic.native_input_link(native["items"].get(key))})
    result["native_user_groups"] = native["user_groups"]
    result["native_user_ownership_conflicts"] = []
    result["issued_path_groups"] = []
    issued = {}; paths = set(); user_owners = defaultdict(list)
    for number, read in enumerate(result["reads"]):
        e = events[read["trace_line"]]; bundle = read["bundle"].get("path")
        if identity(bundle): paths.add(bundle)
        if read.get("status") != "delivered": continue
        selected = e["inline_candidate_prompt"] if e["selected_candidate"] == "inline" else e["baseline_prompt"]
        thread = read["thread_id"]; text_hash = sha(selected.encode())
        group = native["users"].get((thread, read["turn_id"], text_hash))
        explicit = group is not None
        if group is None: group = native["users"].get((thread, None, text_hash))
        if group is not None: require(native["user_texts"][group] == selected, "selected/native text digest collision")
        choices = native["user_groups"][group]["candidates"] if group is not None else []
        method = "explicit_thread_turn_text" if explicit else "unique_thread_text_without_explicit_turn"
        if len(choices) == 1:
            item = choices[0]
            read["native_user"] = {"status": "linked", "join_method": method, "candidate_group_index": group, **item}
            read["charge_refs"] = charges.get((thread, item["item_id"]), [])
            user_owners[(thread, item["item_id"])].append(number)
        else:
            read["native_user"] = {"status": "ambiguous" if choices else "unlinked", "candidate_group_index": group, "candidate_count": len(choices)}
        read["same_turn_response_indices"] = by_turn.get((thread, read["turn_id"]), [])
        if identity(bundle) and e["selected_candidate"] == "baseline" and e.get("eligible") is True:
            key = thread, bundle
            if key not in issued:
                issued[key] = len(result["issued_path_groups"])
                result["issued_path_groups"].append({"thread_id": thread, "path": bundle, "read_indices": []})
            result["issued_path_groups"][issued[key]]["read_indices"].append(number)
    for key, owners in user_owners.items():
        if len(owners) > 1:
            conflict = len(result["native_user_ownership_conflicts"])
            result["native_user_ownership_conflicts"].append({"thread_id": key[0], "item_id": key[1], "read_indices": owners})
            for number in owners:
                read = result["reads"][number]
                read["native_user"] = {"status": "ambiguous", "reason": "one native item matches distinct accepted inputs",
                    "candidate_group_index": read["native_user"]["candidate_group_index"], "ownership_conflict_index": conflict}
                read["charge_refs"] = []
    # The relative phase comes from exact accepted request ordering, never wall time.
    turn_order = {}
    for capture in captures:
        replies = {delivery.rpc(v): v for v in capture["servers"] if "method" not in v and "id" in v}
        for order, frame in enumerate(capture["clients"]):
            if frame.get("method") != "turn/start": continue
            reply = replies.get(delivery.rpc(frame), {})
            turn = reply.get("result", {}).get("turn", {}).get("id")
            if identity(turn) and "error" not in reply:
                turn_order[(frame["params"]["threadId"], turn)] = (capture["path"], order)
    first = {}
    for read in result["reads"]:
        if read.get("status") == "delivered" and read.get("eligible") is True:
            key = read["thread_id"]; order = turn_order.get((key, read["turn_id"]))
            if order is not None and (key not in first or order < first[key]): first[key] = order
    for response in result["responses"]:
        thread = response["thread_id"]; order = turn_order.get((thread, response["turn_id"]))
        response["relative_to_first_eligible_read"] = ("no_eligible_read" if thread not in first else "unlinked_turn" if order is None
            else "pre" if order < first[thread] else "same_turn" if order == first[thread] else "post")
    calls, outputs = native["calls"], native["outputs"]
    call_ids = defaultdict(list); output_ids = defaultdict(list)
    for i, call in enumerate(calls):
        if identity(call.get("call_id")): call_ids[(call["thread_id"], call["call_id"])].append(i)
    for i, output in enumerate(outputs):
        key = output["thread_id"], output.get("call_id")
        if identity(key[1]): output_ids[key].append(i)
        output["charge_refs"] = charges.get((output["thread_id"], output.get("item_id")), [])
        output["call_link_status"] = "linked" if len(call_ids.get(key, [])) == 1 else "ambiguous" if call_ids.get(key) else "unlinked"
    matcher = PatternIndex(paths)
    for i, call in enumerate(calls):
        key = call["thread_id"], call.get("call_id")
        if len(call_ids.get(key, [])) == 1 and len(output_ids.get(key, [])) == 1:
            call["output_indices"] = output_ids[key]
        call["output_link_status"] = "linked" if call["output_indices"] else "ambiguous" if output_ids.get(key) else "unlinked"
        data = native["call_inputs"].get(i, b"")
        for start, end, path in matcher.matches(data):
            group = issued.get((call["thread_id"], path))
            candidates = result["issued_path_groups"][group]["read_indices"] if group is not None else []
            association = "not_issued_to_this_thread" if not candidates else "ambiguous_issued_input" if len(candidates) > 1 else "not_after_linked_input"
            if len(candidates) == 1:
                user = result["reads"][candidates[0]]["native_user"]
                if user.get("status") == "linked" and user["source"] == call["source"] and user["line"] < call["line"]:
                    association = "after_unique_issued_input"
            call["bundle_references"].append({"path": path, "argument_byte_start": start, "argument_byte_end": end,
                "association": association, "issued_path_group_index": group, "actual_read_proven": False,
                "interpretation": "Exact argument substring; command/result semantics require source review, not a file-open count."})
    result.update(tool_calls=calls, tool_outputs=outputs, unknown_native_items=native["unknown"])
    return result


class PatternIndex:
    def __init__(self, patterns):
        patterns = sorted(set(patterns))
        require(len(patterns) <= MAX_PATTERNS and all(identity(p) for p in patterns), "invalid/excessive path patterns")
        require(sum(len(p.encode()) for p in patterns) <= MAX_PATTERN_BYTES, "path pattern byte limit")
        self.lengths = {p: len(p.encode()) for p in patterns}
        self.edges = [{}]; self.failure = [0]; self.terminal = [None]; self.output = [0]; self.cache = {}
        for pattern in patterns:
            state = 0
            for byte in pattern.encode():
                if byte not in self.edges[state]:
                    self.edges[state][byte] = len(self.edges)
                    self.edges.append({}); self.failure.append(0); self.terminal.append(None); self.output.append(0)
                state = self.edges[state][byte]
            self.terminal[state] = pattern
        queue = deque(self.edges[0].values())
        while queue:
            state = queue.popleft()
            for byte, child in self.edges[state].items():
                fallback = self.step(self.failure[state], byte)
                self.failure[child] = fallback
                self.output[child] = fallback if self.terminal[fallback] is not None else self.output[fallback]
                queue.append(child)

    def step(self, state, byte):
        trail = []
        while byte not in self.edges[state] and state and (state, byte) not in self.cache:
            trail.append(state); state = self.failure[state]
        result = self.edges[state].get(byte, self.cache.get((state, byte), 0))
        for prior in trail: self.cache[(prior, byte)] = result
        return result

    def matches(self, data):
        state = 0
        for offset, byte in enumerate(data):
            state = self.step(state, byte); hit = state
            while hit:
                pattern = self.terminal[hit]
                if pattern is not None: yield offset+1-self.lengths[pattern], offset+1, pattern
                hit = self.output[hit]


class SourceIndex:
    def __init__(self):
        self.hashes = {}; self.total = 0

    def read(self, value, expected=NO_EXPECTED_HASH):
        require(expected is NO_EXPECTED_HASH or valid_sha(expected), "expected source digest must be lowercase SHA-256")
        path = Path(value)
        require(path.is_absolute() and path.resolve() == path, "source must be absolute and nonsymlinked")
        before = path.lstat()
        require(stat.S_ISREG(before.st_mode) and before.st_size <= MAX_FILE, "source type/size limit")
        require(str(path) in self.hashes or len(self.hashes) < MAX_SOURCES and self.total+before.st_size <= MAX_TOTAL, "source count/total byte limit")
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as stream:
            opened = os.fstat(stream.fileno()); data = stream.read(MAX_FILE+1); after = os.fstat(stream.fileno())
        key = lambda s: (s.st_dev, s.st_ino, s.st_mode, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
        require(key(before) == key(opened) == key(after) and len(data) <= MAX_FILE, "source changed while read")
        digest = sha(data)
        require(expected is NO_EXPECTED_HASH or digest == expected, "source differs from recorded hash")
        require(str(path) not in self.hashes or self.hashes[str(path)] == digest, "source changed since indexed")
        if str(path) not in self.hashes:
            self.total += len(data); self.hashes[str(path)] = digest
        return data

    def records(self, path, expected=NO_EXPECTED_HASH):
        data = self.read(path, expected); rows = []
        require(not data or data.endswith(b"\n"), "incomplete source JSONL tail")
        for line, raw in enumerate(data.splitlines(), 1):
            if raw.strip():
                value = decode(raw); require(isinstance(value, dict), "nonobject JSONL record")
                rows.append((line, value))
        return rows

    def verify(self):
        for path in tuple(self.hashes): self.read(path)


def publish(output, result, index):
    index.verify()
    with Path(output).open("x", encoding="utf-8") as stream:
        json.dump({**result, "source_sha256": index.hashes}, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write("\n")


def recorded_access(index, recorded):
    def path(value):
        p = Path(value); return p if p.is_absolute() else ROOT/p
    def read(value):
        p = path(value); require(str(p) in recorded, "source is absent from final analysis provenance: "+str(p))
        return index.read(p, recorded[str(p)])
    def read_json(value):
        value = decode(read(value)); require(isinstance(value, dict), "source JSON must be an object"); return value
    def records(value):
        p = path(value); require(str(p) in recorded, "JSONL source lacks final provenance")
        return index.records(p, recorded[str(p)])
    return path, read, read_json, records


def replay_launched_sources(entry, frozen, sessions_root, index, recorded):
    """Shared source branch; caller must prove terminal phase/workflow admission."""
    path, read, read_json, records = recorded_access(index, recorded)
    delivery, _ = dependencies()
    pairs = records(entry["prompt_trace"])
    artifact = path(entry["artifact"]); observed = read_json(artifact/"observation/analysis.json")
    native_sources = []
    for _, meta in records(artifact/"observation/rollout-metadata.jsonl"):
        relative = Path(meta["source_relative_path"]); native_path = Path(sessions_root)/relative
        require(not relative.is_absolute() and ".." not in relative.parts and native_path.resolve().is_relative_to(Path(sessions_root).resolve()), "native source escapes admitted sessions root")
        native_rows = records(native_path)
        require(index.hashes[str(native_path)] == meta.get("source_sha256"), "native source differs from retained metadata")
        native_sources.append({"source": str(native_path), "thread_id": meta["thread_id"], "rows": native_rows})
    captures = []
    for app in sorted((artifact/"observation/app-server").iterdir()):
        if not app.is_dir(): continue
        start = read_json(app.parent.parent/"invocations"/app.name/"start.json")
        read(app.parent.parent/"invocations"/app.name/"end.json")
        for name in ("client-to-server.raw", "server-to-client.raw", "client-to-server.forwarded.raw", "raw-response-usage.json", "raw-response-rewrites.jsonl"):
            read(app/name)
        if type(start.get("provider_usage_grace_ms")) is int and start["provider_usage_grace_ms"] > 0:
            read(app/"provider-usage-grace.jsonl")
        proof = delivery.capture_provenance(app)
        require(not proof["errors"], "capture provenance is incomplete")
        for source, expected in proof["source_sha256"].items(): index.read(source, expected)
        clients = records(app/"client-to-server.raw"); servers = records(app/"server-to-client.raw")
        captures.append({"path": str(app), "clients": [v for _, v in clients], "client_lines": [line for line, _ in clients],
                         "servers": [v for _, v in servers], "server_lines": [line for line, _ in servers]})
    accounting = delivery.measurement_for(frozen, entry, sessions_root)
    for source, expected in accounting.get("source_sha256", {}).items(): index.read(source, expected)
    derived = derive_run(entry["run_id"], entry["condition"], [v for _, v in pairs], observed["threads"], captures, native_sources, accounting, [line for line, _ in pairs])
    return derived, accounting


def build_census(manifest_path, report_path, sessions_root):
    """Replay one closed final census; never call a primary/permutation function."""
    index = SourceIndex(); report = decode(index.read(report_path))
    require(isinstance(report, dict) and report.get("schema") == "work-leaf-untracked-read-primary-v1", "unsupported source analysis report")
    recorded = report.get("source_sha256")
    require(isinstance(recorded, dict) and recorded, "source analysis lacks endpoint hashes")
    path, read, read_json, records = recorded_access(index, recorded)
    # Prevalidate every delegated source before a dependency can open it.
    for source, expected in recorded.items(): index.read(source, expected)
    manifest = read_json(manifest_path); frozen = read_json(manifest["phase_manifest"])
    phase = path(manifest["phase_manifest"]).parent
    require(report.get("phase") == manifest.get("phase") == frozen.get("phase"), "phase identity differs")
    require(index.hashes[str(path(manifest["phase_manifest"]))] == manifest.get("phase_manifest_sha256"), "phase hash differs")
    terminal = read_json(phase/"PHASE-RESULT.json")
    require(identity(terminal.get("finished_at")) and terminal.get("manifest_sha256") == manifest["phase_manifest_sha256"]
            and terminal.get("runs") == manifest.get("runs"), "phase is not terminally published under this manifest")
    admission = read_json(phase/"RUN-ONCE/admission.json")
    require(admission.get("manifest_sha256") == manifest["phase_manifest_sha256"], "manifest lacks exact admission receipt")
    pins = {}
    for row in frozen.get("files", []):
        require(isinstance(row, dict) and identity(row.get("path")) and row["path"] not in pins, "invalid/duplicate frozen source")
        pins[row["path"]] = row
        index.read(row["path"], row.get("sha256"))
    for helper in [Path(__file__).resolve(), *(HERE/name for name in PINS)]:
        require(str(helper) in pins and pins[str(helper)].get("role") == "frozen-evidence"
                and pins[str(helper)].get("sha256") == index.hashes[str(helper)], "census/helper lacks exact admitted source identity")
    schedule = frozen.get("schedule")
    require(isinstance(schedule, list) and len(schedule) == 12 and all(isinstance(r, dict) and identity(r.get("run_id")) for r in schedule)
            and len({r["run_id"] for r in schedule}) == 12, "complete twelve-workflow frozen schedule required")
    entries = {}; observations = defaultdict(list)
    for row in manifest["runs"]:
        require(isinstance(row, dict) and identity(row.get("run_id")) and row["run_id"] not in entries, "duplicate/invalid terminal workflow identity")
        require(row.get("launch_status") != "running", "workflow is still running")
        require(not row.get("started_at") or identity(row.get("finished_at")), "launched workflow lacks a terminal time")
        entries[row["run_id"]] = row
    require(set(entries) == {r["run_id"] for r in schedule}, "terminal workflow census differs from schedule")
    for row in report.get("observations", []):
        require(isinstance(row, dict), "unsupported source result row")
        observations[row.get("id")].append(row)
    result = {"schema": "work-leaf-read-mechanism-census-v1", "phase": manifest["phase"], "phase_root": str(phase),
              "errors": [], "source_analysis_integrity_errors": report.get("integrity_errors"), "runs": [],
              "primary_recomputed": False, "condition_contrast": None, "cost_ranking": None, "causal_share": None,
              "scope": "Every frozen workflow/read/tool/response; exact observed whole-item charges, never file-open or counterfactual prices."}
    for declaration in sorted(schedule, key=lambda row: row["run_id"]):
        rid = declaration["run_id"]; entry = entries[rid]
        out = {"run_id": rid, **fields(entry, ("condition", "launch_status", "launcher_exit_code", "started_at", "finished_at")),
               "status": "unavailable", "errors": [], "reads": [], "responses": [], "tool_calls": [], "tool_outputs": []}
        result["runs"].append(out)
        try:
            for key in ("condition", "artifact", "report", "prompt_trace", "experiment_manifest", "workflow", "wave", "block_id"):
                require(entry.get(key) == declaration.get(key), "terminal row differs from frozen declaration: "+key)
            saved = observations.pop(rid, [])
            require(len(saved) == 1, "missing/duplicate source analysis row")
            require(entry.get("started_at") and entry.get("artifact"), "workflow was not launched; outcome retained without zero-fill")
            pairs = records(entry["prompt_trace"])
            # Even a later provenance failure retains every physical prepared read.
            out["reads"] = [safe_read(v, {}, line) for line, v in pairs if v.get("event") == "read-response"]
            # This receipt is not an input to the primary analyzer. Its exact
            # phase-local identity is validated and independently hashed here.
            receipt = decode(index.read(phase/"logs"/(rid+".exit.json")))
            require(isinstance(receipt, dict) and receipt.get("run_id") == rid and type(receipt.get("launcher_exit_code")) is int
                    and all(receipt.get(k) == entry.get(k) for k in ("launcher_exit_code", "started_at", "finished_at", "launch_status")), "workflow terminal receipt differs")
            derived, accounting = replay_launched_sources(entry, frozen, sessions_root, index, recorded)
            require(saved[0].get("accounting") == accounting, "independent accounting replay differs from source report")
            require(derived["delivery_status"] == saved[0].get("delivery", {}).get("status"), "replayed delivery status differs from source report")
            out.update(derived); out["status"] = "complete_descriptive" if not out["errors"] else "partial_unknown"
        except (ValueError, KeyError, TypeError, OSError, AttributeError) as error:
            out["errors"].append(str(error))
    for rid, rows in observations.items():
        for _ in rows:
            result["runs"].append({"run_id": rid if identity(rid) else None, "status": "unavailable", "errors": ["unexpected source result row retained without body projection"]})
            result["errors"].append("unexpected/duplicate source result identity")
    index.verify()
    return result, index


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--analysis", type=Path, required=True)
    parser.add_argument("--sessions-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    require(not args.output.exists(), "output already exists")
    result, index = build_census(args.manifest, args.analysis, args.sessions_root)
    require(args.output.is_absolute() and args.output.parent.resolve() == args.output.parent
            and args.output.is_relative_to(Path(result["phase_root"])), "output must be canonical and phase-local")
    publish(args.output, result, index)
    print(args.output)


if __name__ == "__main__":
    try: main()
    except (ValueError, KeyError, TypeError, OSError, AttributeError) as error:
        print(f"read census error: {error}", file=sys.stderr); sys.exit(2)
