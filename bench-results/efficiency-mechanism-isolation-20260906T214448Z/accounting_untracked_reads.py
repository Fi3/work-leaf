"""Future-only, provider-free response-identity accounting for the v3 read study.

Original observer totals and diagnostics remain separate. Only explicit native
compaction identities with a complete captured lifecycle can explain omitted
responses. Nonadditive last metadata is unusable, not estimated or zero usage.
All joins are indexed; source scans are O(bytes + events), apart from sorting
identity inventories and existing indexed predecessor tail-window checks.
No configuration, provider option, wait, frozen source or old report is changed.
"""
import ast
from bisect import bisect_right
from collections import defaultdict
import copy
import hashlib
import json
from pathlib import Path
import types

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
GATE = ROOT / "bench-results/efficiency-measurement-gate-20260906/batch_analysis.py"
PINS = {str(GATE): "dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3",
        str(HERE / "analyze.py"): "2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78",
        str(HERE / "audit_compaction.py"): "dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153"}
FIELDS = ("input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens")
CAMEL = ("inputTokens", "cachedInputTokens", "outputTokens", "reasoningOutputTokens")


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def exact_module(path):
    data = Path(path).read_bytes()
    if sha_bytes(data) != PINS[str(path)]:
        raise ValueError("frozen accounting dependency changed: " + str(path))
    module = types.ModuleType("read_accounting_" + Path(path).stem)
    module.__file__ = str(path)
    exec(compile(data, str(path), "exec"), module.__dict__)
    return module


STRICT = exact_module(GATE)
NATIVE = exact_module(HERE / "audit_compaction.py")


def prospective_helpers():
    """Compile unchanged selected definitions, without executing cached loaders.

    The exact admitted predecessor bytes supply its constants and three pure
    helpers. AST selection removes unrelated module loading/CLI/analysis only;
    no selected function body or accounting predicate is transformed.
    """
    path = HERE / "analyze.py"; data = path.read_bytes()
    if sha_bytes(data) != PINS[str(path)]:
        raise ValueError("prospective predecessor source changed")
    functions = {"sha256", "sha_bytes", "capture_provenance", "tail_proof"}
    constants = {"CAPTURE_SETTINGS", "POLICY"}
    tree = ast.parse(data, str(path)); selected = []
    for node in tree.body:
        if isinstance(node, ast.FunctionDef) and node.name in functions:
            selected.append(node)
        elif isinstance(node, ast.Assign) and len(node.targets) == 1:
            target = node.targets[0]
            if isinstance(target, ast.Name) and target.id in constants:
                selected.append(node)
    if len(selected) != len(functions) + len(constants):
        raise ValueError("pinned predecessor definitions are incomplete")
    from bisect import bisect_left
    from collections import Counter
    namespace = dict(STRICT=STRICT, usage=STRICT.usage, Path=Path, hashlib=hashlib,
                     json=json, copy=copy, Counter=Counter, bisect_left=bisect_left)
    exec(compile(ast.Module(body=selected, type_ignores=[]), str(path), "exec"), namespace)
    return types.SimpleNamespace(**namespace)


BASE = prospective_helpers()
PROSPECTIVE = exact_module(GATE)
PROSPECTIVE.tail_proof = BASE.tail_proof


def identity(value):
    if type(value) is not str or not value:
        raise ValueError("identity must be a nonempty string")
    return value


def checked_usage(value, camel=True, allow_nonadditive=False):
    if not isinstance(value, dict):
        raise ValueError("usage must be an object")
    names = CAMEL if camel else FIELDS
    counts = [STRICT.integer(value.get(name)) for name in names]
    i, c, o, r = counts
    if c > i or r > o:
        raise ValueError("cached/reasoning counters exceed parent")
    cache = "cacheWriteInputTokens" if camel else "cache_write_input_tokens"
    if cache in value and (type(value[cache]) is not int or value[cache] != 0):
        raise ValueError("unsupported cache-write accounting")
    total = "totalTokens" if camel else "total_tokens"
    if total in value or camel:
        actual = STRICT.integer(value.get(total))
        if actual != i + o and not allow_nonadditive:
            raise ValueError("provider total differs from input plus output")
    return STRICT.usage(dict(zip(FIELDS, counts)))


def add(*values):
    return STRICT.usage({key: sum(value[key] for value in values) for key in FIELDS})


ZERO = STRICT.usage(dict.fromkeys(FIELDS, 0))


def subtract(left, right):
    return STRICT.usage({key: left[key] - right[key] for key in FIELDS})


def core_usage(value):
    return tuple(value[key] for key in FIELDS)


def native_ledger(rows, metadata, model, effort, source):
    """Require explicit compaction IDs, not adjacency or output item guesses."""
    report = NATIVE.audit_rollout(rows, metadata, model, effort)
    if report["errors"]:
        raise ValueError("native prefix integrity: " + "; ".join(report["errors"]))
    records = {}; compactions = {}
    for rid, entry in report["records"].items():
        records[rid] = {"thread_id": entry["thread_id"], "turn_id": entry["turn_id"],
                        "usage": checked_usage(entry["usage"], camel=False),
                        "sources": [{"path": source, "native_line": entry["line"]}]}
    for line, row in enumerate(rows, 1):
        if row.get("type") != "compacted":
            continue
        payload = row.get("payload", {}); rid = identity(payload.get("compaction_response_id"))
        latest = payload.get("latest_token_usage_record")
        if not isinstance(latest, dict) or rid not in records or rid in compactions:
            raise ValueError("missing or duplicate explicit native compaction identity")
        record = records[rid]
        if (identity(latest.get("response_id")) != rid
                or identity(latest.get("thread_id")) != record["thread_id"]
                or identity(latest.get("session_id")) != record["thread_id"]
                or identity(latest.get("turn_id")) != record["turn_id"]
                or checked_usage(latest.get("usage"), camel=False) != record["usage"]):
            raise ValueError("explicit native compaction usage/identity mismatch")
        # The entire persisted latest record must match the prior independent
        # usage record, including additive thread and turn prefix counters.
        preceding = rows[report["records"][rid]["line"] - 1]["payload"]
        if latest != preceding:
            raise ValueError("compaction latest record differs from native response record")
        compactions[rid] = {"thread_id": record["thread_id"], "turn_id": record["turn_id"],
                            "sources": [{"path": source, "native_line": line}]}
    return {"records": records, "compactions": compactions}


def late_terminal_recoveries(servers, gaps, grace, raw, fresh, excluded_turns):
    """Exact post-grace response completion, never time-based gap recovery."""
    by_turn = defaultdict(list); by_thread = defaultdict(lambda: ([], []))
    for seq, event in enumerate(servers):
        key = STRICT.key(event)
        if key is not None:
            by_turn[key].append((seq, event))
        thread = (event.get("params") or {}).get("threadId")
        if type(thread) is str:
            by_thread[thread][0].append(seq); by_thread[thread][1].append(event)
    grace_by_turn = {(g.get("thread_id"), g.get("turn_id")): g for g in grace or []}
    proofs = {}
    for gap in gaps:
        key = gap["thread_id"], gap["turn_id"]
        if key in excluded_turns or key not in grace_by_turn:
            continue
        values = by_turn[key]
        directives = [s for s, v in values if STRICT.directive(v)]
        terminals = [(s, v) for s, v in values if v.get("method") == "turn/completed"]
        if len(directives) != 1 or len(terminals) != 1:
            continue
        directive = directives[0]; terminal, end = terminals[0]
        if end.get("params", {}).get("turn", {}).get("status") != "interrupted":
            continue
        responses = [(s, v["params"]["responseId"]) for s, v in values
                     if directive < s < terminal and v.get("method") == "rawResponse/completed"]
        if len(responses) != 1:
            continue
        response_seq, rid = responses[0]
        usages = [s for s, v in values if response_seq < s < terminal and fresh.get(s) == rid]
        if not usages or rid not in raw:
            continue
        seqs, events = by_thread[key[0]]
        start, stop = bisect_right(seqs, response_seq), bisect_right(seqs, terminal)
        # No new item, generation boundary, unknown same-thread notification,
        # or compaction can be hidden behind the completed response's charge.
        if any(v.get("method") not in {"thread/tokenUsage/updated", "thread/status/changed", "turn/completed"}
               for v in events[start:stop]):
            continue
        proofs[key] = {"proof": "exact_late_terminal_response_usage", "thread_id": key[0],
            "turn_id": key[1], "response_id": rid, "directive_line": directive+1,
            "raw_response_line": response_seq+1, "fresh_usage_line": usages[0]+1,
            "terminal_line": terminal+1, "original_grace_outcome": grace_by_turn[key]["outcome"],
            "original_grace": grace_by_turn[key], "no_later_generation_or_item_boundary": True,
            "uses_elapsed_time": False, "usage_already_in_unique_response_ledger": True}
    return proofs


def reconcile_stream(clients, servers, grace, native):
    """Validate response prefixes; prove precisely which compactions are omitted.

    Unusable metadata is removed only from the predecessor's *proof view*, after
    its unchanged cumulative total and explicit compaction identity/lifecycle
    have independently passed. Original events and warning payloads are retained.
    No synthetic zero-usage response is constructed. Every affected turn's tail
    proof is withheld conservatively; valid independent fresh usage is unchanged.
    """
    result = dict(errors=[], warnings=[], raw_responses={}, omitted_compaction_ids=[],
                  thread_ledger={}, gaps=[], proof_view_server_lines=[], response_server_lines={},
                  late_terminal_usage_recoveries=[])
    errors = result["errors"]
    try:
        requests = {}; accepted = set(); replied = set()
        for value in clients:
            if value.get("method") == "turn/start":
                rpc = STRICT.rpc_id(value)
                if rpc in requests:
                    raise ValueError("duplicate client turn/start RPC identity")
                requests[rpc] = identity((value.get("params") or {}).get("threadId"))
        for value in servers:
            if "method" not in value and "id" in value:
                rpc = STRICT.rpc_id(value)
                if rpc in requests:
                    if rpc in replied or "error" in value or "result" not in value:
                        raise ValueError("ambiguous or rejected turn/start response")
                    replied.add(rpc)
                    turn = identity((value.get("result") or {}).get("turn", {}).get("id"))
                    accepted.add((requests[rpc], turn))
        raw = {}; starts = {}; ends = {}; windows = {}; raw_sequences = {}
        active_compactions = {}; lifecycle_candidates = defaultdict(list)
        for sequence, value in enumerate(servers):
            method = value.get("method"); p = value.get("params") or {}
            if method == "rawResponse/completed":
                rid = identity(p.get("responseId"))
                entry = {"thread_id": identity(p.get("threadId")), "turn_id": identity(p.get("turnId")),
                         "usage": checked_usage(p.get("usage"))}
                if (entry["thread_id"], entry["turn_id"]) not in accepted:
                    raise ValueError("raw response lacks accepted local turn")
                if rid in raw and raw[rid] != entry:
                    raise ValueError("conflicting raw response identity or usage")
                counterpart = native["records"].get(rid)
                if counterpart is None or any(counterpart.get(k) != entry[k] for k in entry):
                    raise ValueError("raw/native response identity or usage mismatch")
                raw[rid] = entry; raw_sequences.setdefault(rid, sequence)
                result["response_server_lines"].setdefault(rid, []).append(sequence+1)
                if rid in native["compactions"] and result["response_server_lines"][rid] == [sequence+1]:
                    if any(native["compactions"][rid].get(k) != entry[k] for k in ("thread_id", "turn_id")):
                        raise ValueError("native compaction scope differs from raw response")
                    scoped = (entry["thread_id"], entry["turn_id"])
                    active_key = active_compactions.get(scoped)
                    if active_key is None:
                        raise ValueError("explicit compaction response outside captured lifecycle")
                    lifecycle_candidates[active_key].append(rid)
            item = p.get("item") or {}
            if method in {"item/started", "item/completed"} and item.get("type") == "contextCompaction":
                k = identity(p.get("threadId")), identity(p.get("turnId")), identity(item.get("id"))
                target = starts if method == "item/started" else ends
                if k in target:
                    raise ValueError("duplicate context-compaction lifecycle boundary")
                target[k] = sequence
                if method == "item/started":
                    if k[:2] in active_compactions:
                        raise ValueError("overlapping context-compaction lifecycle")
                    active_compactions[k[:2]] = k
                elif active_compactions.pop(k[:2], None) != k:
                    raise ValueError("context-compaction completion does not match start")
        if starts.keys() != ends.keys():
            raise ValueError("incomplete context-compaction lifecycle")
        for k, begin in starts.items():
            end = ends[k]
            matches = lifecycle_candidates[k]
            if len(matches) != 1 or begin >= end or matches[0] in windows:
                raise ValueError("context-compaction lifecycle lacks one explicit native/raw response")
            windows[matches[0]] = (begin, end, k)
        if set(windows) != native["compactions"].keys() & raw.keys():
            raise ValueError("explicit native compaction lacks captured lifecycle")
        result["raw_responses"] = raw
        seen = set(); sums = {}; previous = {}; omitted = {}; pending = defaultdict(list)
        warning_sequences = set(); affected_turns = set()
        fresh = {}; latest_raw = {}
        for sequence, value in enumerate(servers):
            method = value.get("method"); p = value.get("params") or {}
            if method == "rawResponse/completed":
                rid = p["responseId"]; record = raw[rid]; thread = record["thread_id"]
                latest_raw[thread] = rid
                if rid not in seen:
                    seen.add(rid); sums[thread] = add(sums.get(thread, ZERO), record["usage"])
                    if rid in windows:
                        pending[thread].append(rid)
            elif method == "thread/tokenUsage/updated":
                thread = identity(p.get("threadId")); turn = identity(p.get("turnId"))
                if (thread, turn) not in accepted:
                    raise ValueError("cumulative metadata lacks accepted local turn")
                data = p.get("tokenUsage", {}); total = checked_usage(data.get("total"))
                prior = previous.get(thread, ZERO)
                if any(a < b for a, b in zip(core_usage(total), core_usage(prior))):
                    raise ValueError("cumulative provider usage regressed")
                # Parsing all component types/parents/cache scope remains strict.
                last = checked_usage(data.get("last"), allow_nonadditive=True)
                nonadditive = data["last"]["totalTokens"] != last["raw_input_plus_output"]
                expected = subtract(sums.get(thread, ZERO), omitted.get(thread, ZERO))
                newly_omitted = None
                if total != expected:
                    if len(pending[thread]) != 1:
                        raise ValueError("cumulative prefix has unexplained response omission")
                    rid = pending[thread][0]
                    if subtract(expected, raw[rid]["usage"]) != total:
                        raise ValueError("cumulative prefix differs beyond named compaction")
                    begin, end, k = windows[rid]
                    if not begin < sequence < end or k[:2] != (thread, turn):
                        raise ValueError("compaction omission lacks same-turn lifecycle evidence")
                    newly_omitted = rid
                    omitted[thread] = add(omitted.get(thread, ZERO), raw[rid]["usage"])
                    result["omitted_compaction_ids"].append(rid)
                if nonadditive:
                    if newly_omitted is None or total != prior:
                        raise ValueError("nonadditive last lacks unchanged explicit compaction boundary")
                    begin, end, k = windows[newly_omitted]
                    warning_sequences.add(sequence); affected_turns.add((thread, turn))
                    result["warnings"].append({"classification": "unusable_compaction_last_metadata",
                        "server_line": sequence+1, "thread_id": thread, "turn_id": turn,
                        "response_id": newly_omitted, "item_id": k[2],
                        "lifecycle_start_line": begin+1, "lifecycle_end_line": end+1,
                        "original_last": data["last"], "unchanged_cumulative": total,
                        "establishes_completed_response_usage": False,
                        "establishes_context_size_estimate": False,
                        "used_for_fresh_usage_or_tail_recovery": False})
                else:
                    rid = latest_raw.get(thread)
                    if (rid is not None and rid not in windows and raw[rid]["turn_id"] == turn
                            and last == raw[rid]["usage"] and any(core_usage(last))
                            and all(a-b >= c for a, b, c in zip(core_usage(total), core_usage(prior), core_usage(last)))):
                        fresh[sequence] = rid
                previous[thread] = total; pending[thread] = []
        for thread, measured in sums.items():
            original = previous.get(thread)
            if original is None or add(original, omitted.get(thread, ZERO)) != measured:
                raise ValueError("final observed/raw prefix scope mismatch")
            result["thread_ledger"][thread] = {"original": original,
                "omitted_compaction_usage": omitted.get(thread, ZERO), "corrected": measured}
        proof_view = [value for sequence, value in enumerate(servers) if sequence not in warning_sequences]
        result["proof_view_server_lines"] = [sequence+1 for sequence in range(len(servers)) if sequence not in warning_sequences]
        inv = PROSPECTIVE.inventory(clients, proof_view, grace)
        errors.extend(inv["errors"])
        excluded = affected_turns | {window[2][:2] for window in windows.values()}
        recoveries = {} if errors else late_terminal_recoveries(servers, inv["gaps"], grace, raw, fresh, excluded)
        result["late_terminal_usage_recoveries"] = list(recoveries.values())
        inv["gaps"] = [gap for gap in inv["gaps"] if (gap["thread_id"], gap["turn_id"]) not in recoveries]
        for gap in inv["gaps"]:
            if (gap["thread_id"], gap["turn_id"]) in affected_turns:
                gap.update(response_count_upper=None, proof=None,
                           reason="unusable compaction metadata in same turn; tail proof withheld")
        result["gaps"] = inv["gaps"]
        result["captured_turn_count"] = inv["captured_turn_count"]
        result["raw_response_count"] = len(raw)
        result["model_call_inventory_exhaustive"] = False
    except (ValueError, TypeError, KeyError, IndexError) as error:
        errors.append(str(error))
    return result


def audit_run(entry, frozen, sessions_root):
    """Independently replay one retained workflow; phase gates belong to caller."""
    result = {"schema": "work-leaf-read-identity-accounting-v1", "run_id": entry.get("run_id", entry.get("id")),
              "status": "unknown", "errors": [], "warnings": [], "source_sha256": {},
              "measurement": {"status": "ineligible", "bounds": None},
              "original_observer_ledger": None, "corrected_scope": None,
              "exact_response_evidence": {}, "whole_workflow_hidden_call_completeness_proven": False}
    sources = result["source_sha256"]; errors = result["errors"]

    def remember(value):
        path = Path(value); path = path if path.is_absolute() else ROOT/path
        path = path.resolve(); data = path.read_bytes(); digest = sha_bytes(data)
        if str(path) in sources and sources[str(path)] != digest:
            raise ValueError("source changed during accounting")
        sources[str(path)] = digest
        return path, data

    def read(value):
        return json.loads(remember(value)[1])

    def rows(value):
        path, data = remember(value); result_rows = []
        for physical, line in enumerate(data.splitlines(), 1):
            if not line.strip():
                # Keeping exact physical locators is required by the schema.
                result_rows.append({})
                continue
            parsed = json.loads(line)
            if not isinstance(parsed, dict):
                raise ValueError("JSONL record must be an object")
            result_rows.append(parsed)
        return path, result_rows

    try:
        identity(result["run_id"])
        self_path, self_data = remember(__file__)
        pinned = [f for f in frozen["files"] if Path(f["path"]).resolve() == self_path]
        if len(pinned) != 1 or pinned[0].get("role") != "frozen-evidence" or pinned[0].get("sha256") != sha_bytes(self_data):
            raise ValueError("accounting helper lacks unique frozen identity")
        for path, expected in PINS.items():
            if sha_bytes(remember(path)[1]) != expected:
                raise ValueError("accounting dependency identity drift")
        if not entry.get("started_at") or not entry.get("artifact"):
            raise ValueError("unlaunched or missing artifact; outcome retained")
        artifact = Path(entry["artifact"]); artifact = artifact if artifact.is_absolute() else ROOT/artifact
        observation = artifact/"observation"
        report = read(entry["report"]); observed = read(observation/"analysis.json")
        policy = BASE.POLICY
        if report.get("agent_model") != policy["model"] or report.get("agent_reasoning_effort") != policy["reasoning_effort"]:
            raise ValueError("report model/effort differs from frozen accounting policy")
        if frozen.get("model") != policy["model"] or frozen.get("reasoning_effort") != policy["reasoning_effort"]:
            raise ValueError("phase model/effort differs from accounting policy")
        original = STRICT.usage(observed["usage_scopes"]["total_workflow"])
        result["original_observer_ledger"] = {"usage": original, "errors": observed.get("errors"),
            "capture_complete": observed.get("capture_complete"),
            "interrupted_provider_turns": observed.get("interrupted_provider_turns")}
        if STRICT.usage(report["total_workflow_usage"]) != original:
            raise ValueError("report/observer original ledger mismatch")
        if observed.get("session_only_threads"):
            raise ValueError("session-only observer scope unsupported")
        observer_threads = {}
        for thread in observed["threads"]:
            tid = identity(thread.get("thread_id"))
            if tid in observer_threads:
                raise ValueError("duplicate observer thread")
            observer_threads[tid] = thread
        native = {"records": {}, "compactions": {}}; metadata_threads = set()
        _, metadata = rows(observation/"rollout-metadata.jsonl")
        sessions_root = Path(sessions_root).resolve()
        for meta in metadata:
            tid = identity(meta.get("thread_id"))
            if tid in metadata_threads or tid not in observer_threads or meta.get("descendant"):
                raise ValueError("unsupported or duplicate native thread scope")
            metadata_threads.add(tid)
            if any(meta.get(k) != observer_threads[tid].get(k) for k in ("primary", "visible")):
                raise ValueError("native/observer scope labels differ")
            relative = Path(meta["source_relative_path"]); path = (sessions_root/relative).resolve()
            if relative.is_absolute() or not path.is_relative_to(sessions_root):
                raise ValueError("native source escapes declared sessions root")
            _, native_rows = rows(path)
            if sources[str(path)] != meta["source_sha256"]:
                raise ValueError("native source digest differs from retained metadata")
            ledger = native_ledger(native_rows, meta, policy["model"], policy["reasoning_effort"], str(path))
            for kind in native:
                if native[kind].keys() & ledger[kind].keys():
                    raise ValueError("response identity repeated across native threads")
                native[kind].update(ledger[kind])
        if not observer_threads or metadata_threads != observer_threads.keys():
            raise ValueError("native source inventory does not cover observer scope")
        combined = {"errors": [], "gaps": [], "raw_responses": {}}
        thread_ledgers = {}; omitted_ids = []; captures = []
        for app in sorted((observation/"app-server").iterdir()):
            if not app.is_dir():
                continue
            provenance = BASE.capture_provenance(app)
            for path, expected in provenance["source_sha256"].items():
                if sha_bytes(remember(path)[1]) != expected:
                    raise ValueError("capture changed during provenance replay")
            if provenance["errors"]:
                raise ValueError("capture provenance: " + "; ".join(provenance["errors"]))
            _, clients = rows(app/"client-to-server.raw"); _, servers = rows(app/"server-to-client.raw")
            grace_path = app/"provider-usage-grace.jsonl"
            grace = rows(grace_path)[1] if grace_path.is_file() else None
            inv = reconcile_stream(clients, servers, grace, native)
            combined["errors"].extend(inv["errors"]); combined["gaps"].extend(inv["gaps"])
            for tid, ledger in inv["thread_ledger"].items():
                if tid in thread_ledgers:
                    raise ValueError("thread repeated across primary captures; scope ambiguous")
                thread_ledgers[tid] = ledger
            omitted_ids.extend(inv["omitted_compaction_ids"])
            result["warnings"].extend({"capture": str(app), **w} for w in inv["warnings"])
            for rid, record in inv["raw_responses"].items():
                if rid in combined["raw_responses"]:
                    raise ValueError("response repeated across primary captures")
                combined["raw_responses"][rid] = record
                result["exact_response_evidence"][rid] = {**record,
                    "sources": native["records"][rid]["sources"] + [
                        {"path": str(app/"server-to-client.raw"), "server_line": line}
                        for line in inv["response_server_lines"][rid]]}
            captures.append({"path": str(app), "proof_view_server_lines": inv["proof_view_server_lines"],
                             "warnings": inv["warnings"], "omitted_compaction_ids": inv["omitted_compaction_ids"],
                             "late_terminal_usage_recoveries": inv["late_terminal_usage_recoveries"]})
        if not captures or set(combined["raw_responses"]) != set(native["records"]):
            raise ValueError("raw/native response inventory coverage differs")
        if thread_ledgers.keys() != observer_threads.keys():
            raise ValueError("raw cumulative thread scope differs from observer")
        for tid, ledger in thread_ledgers.items():
            if ledger["original"] != STRICT.usage(observer_threads[tid]["usage"]):
                raise ValueError("original thread cumulative differs from observer")
        if add(*(v["original"] for v in thread_ledgers.values())) != original:
            raise ValueError("original whole-workflow scope differs from thread inventory")
        if len(set(omitted_ids)) != len(omitted_ids):
            raise ValueError("compaction omission would be counted twice")
        correction = add(*(native["records"][rid]["usage"] for rid in omitted_ids))
        corrected = add(original, correction)
        if corrected != add(*(v["usage"] for v in native["records"].values())):
            raise ValueError("corrected scope differs from unique response ledger")
        result["corrected_scope"] = {"usage": corrected, "named_compaction_usage": correction,
            "omitted_compaction_response_ids": omitted_ids, "threads": thread_ledgers,
            "compaction_identity_evidence": native["compactions"], "captures": captures}
        # Only the separately named scope correction changes this private copy.
        # Original observer diagnostics and original source bytes remain intact.
        adjusted = copy.deepcopy(observed)
        adjusted["usage_scopes"]["total_workflow"] = corrected
        result["measurement"] = STRICT.accounting(adjusted, combined, policy)
        if result["measurement"]["status"] == "ineligible":
            errors.extend(result["measurement"]["reasons"])
        if not errors:
            result["status"] = "validated"
    except (ValueError, KeyError, TypeError, OSError, IndexError) as error:
        errors.append(str(error))
    for path, expected in list(sources.items()):
        try:
            if sha_bytes(Path(path).read_bytes()) != expected:
                errors.append("source changed before accounting closure: " + path)
        except OSError as error:
            errors.append(str(error))
    if errors:
        result["status"] = "unknown"
        result["measurement"]["status"] = "ineligible"
        result["measurement"]["bounds"] = None
    return result
