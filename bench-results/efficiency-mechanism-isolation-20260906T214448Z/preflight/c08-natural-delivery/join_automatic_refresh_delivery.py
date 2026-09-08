"""Pure closed-input joins; full-frame/source and exposure proof stay upstream.

Only exact supplied code bytes are compiled. No filesystem or capture I/O, and
no policy-owner, timestamp-FIFO, native-action or accounting inference.
"""
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import PurePosixPath
import types


PRIMITIVE_SHA = "34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257"
EVENT_SHA = "c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f"
FRAME_SHA = "ccfb4cc3fe2a24c6496f4a749e6c95f147d6b5fefa7fd4e91dedc6f40be1961f"


class Invalid(ValueError):
    """A fixed error code, never captured body text."""


def need(value, code):
    if not value:
        raise Invalid(code)


def ident(value):
    need(type(value) is str and bool(value), "invalid-identity")
    value.encode("utf-8")
    return value


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":"), allow_nan=False).encode()


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def code_module(raw, expected, filename):
    need(type(raw) is bytes and sha(raw) == expected, "dependency-source-pin")
    module = types.ModuleType("pinned_" + filename.stem)
    module.__file__ = str(filename)
    exec(compile(raw, str(filename), "exec"), module.__dict__)
    return module


def records(value):
    """A bad line map is flagged without zipping/truncating its supplied rows."""
    if type(value) is not list:
        return [], False
    result = []; previous = 0; valid = True
    for entry in value:
        if type(entry) not in (tuple, list) or len(entry) != 2:
            result.append((None, None)); valid = False; continue
        line, row = entry
        okay = type(line) is int and line > previous
        if okay: previous = line
        else: valid = False
        result.append((line if okay else None, row))
        if type(row) is not dict: valid = False
    return result, valid


def text_item(content, primitive):
    need(type(content) is list and len(content) == 1 and type(content[0]) is dict, "unsupported-text-content")
    item = content[0]
    if "text_elements" in item:
        need(type(item["text_elements"]) is list and not item["text_elements"], "unsupported-text-metadata")
    text = primitive.one_text(content)
    need(type(text) is str, "unsupported-text-content")
    return text, sha(text.encode())


def safe_frame(row):
    try:
        return sha(canonical(row))
    except (TypeError, ValueError, UnicodeError):
        return None


def frame_interfaces(captures, primitive, result):
    parsed = []; publics = []; capture_names = Counter()
    for capture in captures:
        if type(capture) is dict and type(capture.get("capture_id")) is str:
            capture_names[capture["capture_id"]] += 1
    for capture_index, capture in enumerate(captures):
        if type(capture) is not dict:
            result["errors"].append("invalid-capture"); continue
        name = capture.get("capture_id")
        name_ok = type(name) is str and bool(name) and capture_names[name] == 1
        clients, client_ok = records(capture.get("clients"))
        forwarded, forward_ok = records(capture.get("forwarded"))
        servers, server_ok = records(capture.get("servers"))
        bad = not (name_ok and client_ok and forward_ok and server_ok)
        if bad: result["errors"].append("capture-shape-or-physical-map")
        different = []; frame_details = []; nonmetadata_bad = len(clients) != len(forwarded)
        for index in range(max(len(clients), len(forwarded))):
            old_line, old = clients[index] if index < len(clients) else (None, None)
            new_line, new = forwarded[index] if index < len(forwarded) else (None, None)
            same = primitive.typed_equal(old, new)
            if not same or old_line != new_line:
                different.append(old_line)
                frame_details.append({"original_line": old_line, "forwarded_line": new_line,
                                      "original_sha256": safe_frame(old), "forwarded_sha256": safe_frame(new)})
                metadata = (type(old) is dict and type(new) is dict and old.get("method") in ("initialize", "thread/start")
                            and new.get("method") == old.get("method") and old_line == new_line)
                if not metadata: nonmetadata_bad = True
        if nonmetadata_bad: result["errors"].append("nonmetadata-forwarding-mismatch")
        result["frame_relation"]["captures"].append({"capture_index": capture_index,
            "capture_id": name if type(name) is str else None, "original_frames": len(clients),
            "forwarded_frames": len(forwarded), "server_frames": len(servers),
            "different_frame_lines": different, "differences": frame_details})
        all_requests = defaultdict(list); replies = defaultdict(list); turns = []
        for index, (line, frame) in enumerate(clients):
            if type(frame) is not dict: continue
            key = None
            if "id" in frame:
                try: key = primitive.rpc_key(frame)
                except (ValueError, TypeError): result["errors"].append("invalid-request-rpc")
                if key is not None: all_requests[key].append(index)
            if frame.get("method") == "turn/start":
                turns.append((index, line, frame, key))
        for line, frame in servers:
            if type(frame) is not dict: continue
            if "id" in frame and "method" not in frame:
                try:
                    key = primitive.rpc_key(frame); replies[key].append((line, frame))
                except (ValueError, TypeError): result["errors"].append("invalid-reply-rpc")
            if frame.get("method") != "item/completed": continue
            params = frame.get("params")
            if type(params) is not dict:
                result["errors"].append("malformed-public-item"); continue
            item = params.get("item")
            if type(item) is not dict:
                result["errors"].append("malformed-public-item"); continue
            if item.get("type") != "userMessage": continue
            record = {"capture_index": capture_index, "public_line": line, "valid": False}
            try:
                record["thread"] = ident(params.get("threadId")); record["turn"] = ident(params.get("turnId"))
                record["item"] = ident(item.get("id"))
                record["text"], record["digest"] = text_item(item.get("content"), primitive)
                record["valid"] = True
            except (ValueError, TypeError, UnicodeError): result["errors"].append("invalid-public-user")
            publics.append(record)
        if any(len(v) != 1 for v in all_requests.values()): result["errors"].append("duplicate-request-rpc")
        if any(len(v) != 1 for v in replies.values()): result["errors"].append("duplicate-reply-rpc")
        for key, values in replies.items():
            if key not in all_requests:
                result["orphans"]["replies"].extend({"capture_index": capture_index, "reply_line": line,
                    "rpc_type": key[0], "rpc_id": key[1]} for line, _ in values)
        parsed.append({"index": capture_index, "name": name if type(name) is str else None,
                       "bad": bad, "forwarding_bad": nonmetadata_bad, "turns": turns,
                       "requests": all_requests, "replies": replies})
    return parsed, publics


def native_interfaces(sources, primitive, result):
    users = {}; threads = {}; unavailable = []
    declared_threads = Counter(s.get("thread_id") for s in sources
                               if type(s) is dict and type(s.get("thread_id")) is str)
    declared_sources = Counter(s.get("source") for s in sources
                               if type(s) is dict and type(s.get("source")) is str)
    for source_index, source in enumerate(sources):
        if type(source) is not dict:
            result["errors"].append("invalid-native-source"); continue
        rows, line_ok = records(source.get("rows"))
        thread = source.get("thread_id"); name = source.get("source")
        if type(thread) is str and thread: threads.setdefault(thread, None)
        seen = {}; raw_rows = []; context_count = 0; replays = 0
        try:
            ident(thread); ident(name)
            need(line_ok and declared_threads[thread] == 1 and declared_sources[name] == 1, "native-source-identity")
            for line, row in rows:
                need(type(row) is dict, "native-row-shape")
                payload = row.get("payload", {})
                need(type(payload) is dict, "native-payload-shape")
                if row.get("type") != "response_item" or payload.get("role") != "user": continue
                raw_rows.append(line)
                need(payload.get("type") == "message", "native-user-kind")
                passthrough = payload.get("internal_chat_message_metadata_passthrough")
                need(passthrough is None or type(passthrough) is dict, "native-passthrough")
                kinds = (passthrough or {}).get("content_item_kinds")
                need(kinds is None or (type(kinds) is list and all(type(k) is str for k in kinds)), "native-content-kinds")
                item = payload.get("id")
                if item is not None:
                    ident(item)
                    if item in seen:
                        need(primitive.typed_equal(seen[item], payload), "native-replay-payload")
                        replays += 1
                    else: seen[item] = payload
                if kinds is not None and kinds != ["user.text"]:
                    context_count += 1; continue
                text_item(payload.get("content"), primitive)
            # Each source is checked independently so one failure cannot erase
            # another thread. Global source/thread uniqueness was checked above.
            indexed, _ = primitive.native_user_index([source])
            for key, value in indexed.items():
                users[key] = {**value, "source_index": source_index, "thread": thread,
                              "turn": key[1], "digest": key[2]}
            result["native_sources"].append({"source_index": source_index, "source": name, "thread_id": thread,
                "status": "indexed", "context_user_rows": context_count, "identical_replays": replays})
        except (ValueError, TypeError, KeyError, UnicodeError):
            result["errors"].append("native-source-invalid")
            result["native_sources"].append({"source_index": source_index, "status": "invalid"})
            unavailable.append({"source_index": source_index, "status": "unavailable-source"})
    return users, threads, unavailable


def request_interfaces(parsed, result, primitive):
    requests = []
    for capture in parsed:
        for _, line, frame, rpc in capture["turns"]:
            record = {"capture_index": capture["index"], "capture_id": capture["name"], "request_line": line,
                      "rpc_type": rpc[0] if rpc else None, "rpc_id": rpc[1] if rpc else None,
                      "thread_id": None, "turn_id": None, "status": "ambiguous"}
            internal = {"record": record, "text": None, "digest": None}
            requests.append(internal)
            try:
                params = frame.get("params"); need(type(params) is dict, "request-params")
                record["thread_id"] = ident(params.get("threadId"))
                internal["text"], internal["digest"] = text_item(params.get("input"), primitive)
                record["input_sha256"] = internal["digest"]; record["input_bytes"] = len(internal["text"].encode())
                need(rpc is not None and len(capture["requests"][rpc]) == 1 and not capture["bad"], "request-identity")
                if capture["forwarding_bad"]:
                    record["status"] = "mismatched"; continue
                replies = capture["replies"].get(rpc, [])
                if not replies:
                    record["status"] = "missing_reply"; continue
                need(len(replies) == 1, "reply-identity")
                reply_line, reply = replies[0]; record["reply_line"] = reply_line
                need(not ("result" in reply and "error" in reply), "mixed-reply")
                if "error" in reply:
                    need(type(reply["error"]) is dict, "rejection-shape")
                    record["status"] = "rejected"
                    code = reply["error"].get("code")
                    if type(code) is int: record["rejection_code"] = code
                    continue
                value = reply.get("result"); need(type(value) is dict, "acceptance-shape")
                turn = value.get("turn"); need(type(turn) is dict, "accepted-turn-shape")
                record["turn_id"] = ident(turn.get("id")); record["status"] = "accepted"
            except (ValueError, TypeError, KeyError, UnicodeError): result["errors"].append("request-or-reply-invalid")
    return requests


def link_inputs(requests, publics, native, unavailable, result):
    by_turn = defaultdict(list); by_item = defaultdict(list)
    for index, public in enumerate(publics):
        if "thread" in public and "turn" in public: by_turn[public["thread"], public["turn"]].append(index)
        if "thread" in public and "item" in public: by_item[public["thread"], public["item"]].append(index)
    accepted_counts = Counter((q["record"]["thread_id"], q["record"]["turn_id"])
                              for q in requests if q["record"]["status"] == "accepted")
    used_public = set(); used_native = set(); native_to_input = {}
    for index, request in enumerate(requests):
        record = request["record"]
        if record["status"] != "accepted": continue
        key = record["thread_id"], record["turn_id"]
        if accepted_counts[key] != 1:
            record["status"] = "ambiguous"; result["errors"].append("duplicate-accepted-turn"); continue
        native_key = (*key, request["digest"])
        source = native.get(native_key)
        native_ok = source is not None and source["text"] == request["text"]
        if native_ok:
            used_native.add(native_key)
            record.update(native_item_id=source["native_item_id"], native_line=source["native_line"],
                          native_source=source["native_source"])
        matches = by_turn.get(key, [])
        if not matches:
            record["status"] = "public_missing"
        elif len(matches) != 1:
            record["status"] = "ambiguous"
        else:
            public = publics[matches[0]]
            public_ok = (public["valid"] and public["capture_index"] == record["capture_index"]
                         and len(by_item[public["thread"], public["item"]]) == 1 and public["text"] == request["text"])
            if not public_ok: record["status"] = "mismatched"
            else:
                used_public.add(matches[0])
                record.update(public_item_id=public["item"], public_line=public["public_line"])
                record["status"] = "joined" if native_ok else "native_missing"
                if native_ok: native_to_input[native_key] = index
        if record["status"] != "joined": result["errors"].append("accepted-input-not-joined")
    for index, public in enumerate(publics):
        if index not in used_public:
            result["orphans"]["public"].append({"capture_index": public["capture_index"], "public_line": public["public_line"],
                **{out: public[key] for out, key in (("thread_id", "thread"), ("turn_id", "turn"), ("public_item_id", "item")) if key in public}})
    for key, source in native.items():
        if key not in used_native:
            result["orphans"]["native"].append({"source_index": source["source_index"], "native_line": source["native_line"],
                "native_item_id": source["native_item_id"], "thread_id": source["thread"], "turn_id": source["turn"]})
    result["orphans"]["native"].extend(unavailable)
    if any(result["orphans"].values()): result["errors"].append("orphan-input-or-reply")
    return native_to_input


def join_occurrences(trace_rows, census, requests, native, native_threads, native_to_input, result):
    trace = []; text_groups = {}; group_by_hash = {}

    def group(digest, text):
        if digest not in group_by_hash:
            number = len(text_groups); group_by_hash[digest] = number
            text_groups[number] = {"sha256": digest, "text": text, "collision": False, "trace_indices": [], "input_indices": []}
        number = group_by_hash[digest]
        if text_groups[number]["text"] != text: text_groups[number]["collision"] = True
        return number

    for checked in census["events"]:
        record = {"row_index": checked["row_index"], "status": "invalid", "eligible": checked["eligible"]}
        row = trace_rows[checked["row_index"]]
        internal = {"record": record, "row": row, "valid": not checked["errors"]}
        trace.append(internal)
        if checked["errors"]:
            record["errors"] = list(checked["errors"]); continue
        selected = row["candidate_prompt"] if row["event"] == "automatic-refresh" else row["forwarded_prompt"]
        record.update(sequence=row["sequence"], selected_sha256=checked["selected_sha256"], status="undelivered")
        internal.update(agent=row["agent_id"], text=selected)
        number = group(checked["selected_sha256"], selected)
        record["occurrence_group"] = number; text_groups[number]["trace_indices"].append(len(trace) - 1)
    for index, request in enumerate(requests):
        if request["text"] is not None:
            number = group(request["digest"], request["text"])
            request["group"] = number; text_groups[number]["input_indices"].append(index)
    claims = defaultdict(set)
    for value in text_groups.values():
        if value["collision"] or len(value["trace_indices"]) != 1 or len(value["input_indices"]) != 1: continue
        event = trace[value["trace_indices"][0]]; request = requests[value["input_indices"][0]]
        if event["row"]["event"] == "automatic-refresh" and request["record"]["status"] == "joined":
            claims[request["record"]["thread_id"]].add(event["agent"])
    owners = {thread: next(iter(agents)) for thread, agents in claims.items() if len(agents) == 1}
    conflicted = {thread for thread, agents in claims.items() if len(agents) != 1}
    if conflicted: result["errors"].append("conflicting-thread-owner")
    all_threads = dict(native_threads)
    for request in requests:
        if request["record"]["thread_id"] is not None: all_threads.setdefault(request["record"]["thread_id"], None)
    for thread in all_threads:
        result["threads"].append({"thread_id": thread, "ownership": "conflicting" if thread in conflicted else "bound" if thread in owners else "unknown",
                                  "agent_id": owners.get(thread)})
    owner_events = defaultdict(list); owner_requests = defaultdict(list)
    for index, event in enumerate(trace):
        if event["valid"]: owner_events[event["agent"], event["record"]["occurrence_group"]].append(index)
    # Native dict insertion order comes from each unique source's physical order.
    # This supplies within-thread order even when capture arrays are reversed.
    ordered_inputs = list(native_to_input[key] for key in native if key in native_to_input)
    ordered_set = set(ordered_inputs)
    ordered_inputs.extend(index for index in range(len(requests)) if index not in ordered_set)
    for index in ordered_inputs:
        request = requests[index]; owner = owners.get(request["record"]["thread_id"])
        if owner is not None and "group" in request:
            owner_requests[owner, request["group"]].append(index)
    for (owner, number), event_indices in owner_events.items():
        value = text_groups[number]; candidates = owner_requests.get((owner, number), [])
        global_candidates = value["input_indices"]
        threads = {requests[index]["record"]["thread_id"] for index in candidates}
        ambiguous = (value["collision"] or len(candidates) != len(event_indices) or len(threads) != 1
                     or (len(candidates) > 1 and any(requests[i]["record"]["status"] != "joined" for i in candidates)))
        if ambiguous:
            for index in event_indices:
                trace[index]["record"]["status"] = "ambiguous" if global_candidates else "undelivered"
            continue
        for event_index, input_index in zip(event_indices, candidates):
            record = trace[event_index]["record"]; request = requests[input_index]["record"]
            if request["status"] == "joined":
                record.update(status="joined", input_index=input_index)
            else: record["status"] = "undelivered"
    previous = {}; order_bad = set()
    for event in trace:
        record = event["record"]
        if record["status"] != "joined": continue
        request = requests[record["input_index"]]["record"]
        thread = request["thread_id"]; line = request["native_line"]
        if thread in previous and line <= previous[thread]: order_bad.add(thread)
        previous[thread] = line
    if order_bad: result["errors"].append("trace-native-order-conflict")
    for event in trace:
        record = event["record"]
        if record["status"] == "joined" and requests[record["input_index"]]["record"]["thread_id"] in order_bad:
            record["status"] = "order_conflict"; record.pop("input_index")
        if record["status"] != "joined": result["errors"].append("trace-occurrence-not-joined")
        result["trace"].append(record)
    for number, value in text_groups.items():
        result["occurrence_groups"].append({"index": number, "sha256": value["sha256"], "collision": value["collision"],
            "trace_indices": value["trace_indices"], "input_indices": value["input_indices"]})


def join_delivery(trace_rows, captures, native_sources, expected_run_id, primitive_source, event_validator_source):
    """Join supplied identities; caller must separately execute full-frame proof."""
    result = {"errors": [], "inputs": [], "trace": [], "threads": [], "native_sources": [],
              "orphans": {"public": [], "native": [], "replies": []}, "occurrence_groups": [],
              "frame_relation": {"status": "upstream-full-frame-proof-required", "captures": [],
                                  "required_prove_frames_sha256": FRAME_SHA}, "exposure_qualified": False}
    try:
        here = PurePosixPath(__file__).parent
        primitive = code_module(primitive_source, PRIMITIVE_SHA, here.parent.parent / "audit_review_evidence.py")
        validator = code_module(event_validator_source, EVENT_SHA, here / "validate_automatic_refresh.py")
        need(type(captures) is list and type(native_sources) is list and type(trace_rows) is list, "invalid-populations")
    except (ValueError, TypeError):
        result["errors"].append("dependency-or-population-invalid")
        return result
    result["dependency_sha256"] = {"primitive": PRIMITIVE_SHA, "event_validator": EVENT_SHA}
    census = validator.census_trace(trace_rows, expected_run_id)
    result["errors"].extend(census["errors"])
    parsed, publics = frame_interfaces(captures, primitive, result)
    native, native_threads, unavailable = native_interfaces(native_sources, primitive, result)
    requests = request_interfaces(parsed, result, primitive)
    native_to_input = link_inputs(requests, publics, native, unavailable, result)
    join_occurrences(trace_rows, census, requests, native, native_threads, native_to_input, result)
    result["inputs"] = [request["record"] for request in requests]
    return result
