#!/usr/bin/env python3
"""Provider-free WL-only contrasts and prospective conditional tail accounting.

Raw tokens are primary. Missing responses, failed workflows, unlaunched rows and
unexposed interventions remain visible. Screening selects hypotheses, never the
confirmation evidence. Event joins use indexed identities, not time proximity.
"""

import argparse
from bisect import bisect_left
from collections import Counter, defaultdict, deque
import copy
import hashlib
import importlib.util
from itertools import combinations, product
import json
import math
import os
from pathlib import Path
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
GATE = ROOT / "bench-results/efficiency-measurement-gate-20260906"
GATE_SHA = "dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3"
METRICS = ("raw_input_plus_output", "uncached_input_plus_output")
ACK = "run at most one focused validation step that is relevant to files you touched or checks you added."
UNLIMITED = "run the required focused validation steps that are relevant to files you touched or checks you added."
GUIDANCE = "\nnext: Reply with the next Work Leaf directive, such as `@work-leaf done`, `@work-leaf edit`, `@work-leaf read`, or another `@work-leaf locks run`. Keep any non-directive explanation brief."
TARGETS = {"ack-validation-unlimited": "patch-applied", "command-guidance-neutral": "command-result"}
CAPTURE_SETTINGS = {
    "schema_version": 1, "enabled": True, "environment_variable": "WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE",
    "request_metadata_overrides": {"initialize.params.capabilities.experimentalApi": True,
        "initialize.params.capabilities.optOutNotificationMethods.append": "rawResponseItem/completed",
        "thread/start.params.experimentalRawEvents": True},
    "original_client_stream": "client-to-server.raw", "forwarded_client_stream": "client-to-server.forwarded.raw",
    "rewrite_decisions": "raw-response-rewrites.jsonl", "interrupt_policy_unchanged": True,
}
POLICY = {
    "schema_version": 1, "model": "gpt-5.5", "reasoning_effort": "xhigh",
    "per_response": {"input_upper": 1050000, "output_upper": 128000,
                     "source_url": "https://developers.openai.com/api/docs/models/gpt-5.5"},
    "tail_rule": "isolated visible response segment; one or more paired assistant items",
    "assumptions": "Conditional on complete visible normal response/tool boundaries, no hidden retries or opaque provider responses, and no unobserved preemption. Item count is not response count. A model ceiling is not a proof of call cardinality.",
    "provider_source": "https://github.com/openai/codex/blob/rust-v0.153.4/codex-rs/core/src/session/turn.rs",
    "sampling_confidence_interval": False,
}


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sha_bytes(value):
    return hashlib.sha256(value).hexdigest()


def capture_provenance(app):
    """Revalidate digests and every allowed metadata rewrite from saved bytes.

    Changed metadata frames must be semantically identical except for the three
    declared capabilities. All other frames, including prompts and interrupts,
    must be byte-identical. The strict observer independently validates its exact
    serializer output; this replay does not rely solely on its saved success flag.
    """
    app = Path(app)
    invocation = app.parent.parent/"invocations"/app.name
    result = {"errors": [], "source_sha256": {}, "checked_frames": 0}

    def remember(path):
        result["source_sha256"][str(path)] = sha256(path)
        return Path(path).read_bytes()

    try:
        start_bytes = remember(invocation/"start.json")
        start = json.loads(start_bytes)
        end = json.loads(remember(invocation/"end.json"))
        if start.get("raw_response_usage") is not True:
            raise ValueError("prospective study requires explicit raw metadata opt-in")
        if end.get("raw_response_usage_start_sha256") != sha_bytes(start_bytes):
            raise ValueError("invocation start provenance mismatch")
        names = {"raw-response-usage.json", "raw-response-rewrites.jsonl", "client-to-server.forwarded.raw"}
        if STRICT.integer(start.get("provider_usage_grace_ms")):
            names.add("provider-usage-grace.jsonl")
        hashes = end.get("raw_response_usage_sha256", {})
        if set(hashes) != names:
            raise ValueError("unexpected or incomplete raw metadata artifact inventory")
        contents = {}
        for name in names:
            contents[name] = remember(app/name)
            if hashes[name] != sha_bytes(contents[name]):
                raise ValueError(f"capture artifact digest differs: {name}")
        if json.loads(contents["raw-response-usage.json"]) != CAPTURE_SETTINGS:
            raise ValueError("capture settings differ from frozen metadata-only policy")
        original = remember(app/"client-to-server.raw")
        server = remember(app/"server-to-client.raw")
        if end.get("stdin_sha256") != sha_bytes(original) or end.get("stdout_sha256") != sha_bytes(server):
            raise ValueError("original client/server stream digest mismatch")
        before = original.splitlines(keepends=True)
        after = contents["client-to-server.forwarded.raw"].splitlines(keepends=True)
        decisions = iter(json.loads(line) for line in contents["raw-response-rewrites.jsonl"].splitlines() if line.strip())
        if len(before) != len(after):
            raise ValueError("forwarded request frame inventory differs")
        for old, new in zip(before, after):
            value = json.loads(old)
            expected = copy.deepcopy(value)
            method = value.get("method")
            metadata_site = method in {"initialize", "thread/start"}
            if metadata_site and "id" in value:
                params = expected["params"]
                if method == "thread/start":
                    params["experimentalRawEvents"] = True
                else:
                    if params.get("capabilities") is None:
                        params["capabilities"] = {}
                    capabilities = params["capabilities"]
                    if capabilities.get("optOutNotificationMethods") is None:
                        capabilities["optOutNotificationMethods"] = []
                    opt_out = capabilities["optOutNotificationMethods"]
                    if not isinstance(opt_out, list) or any(not isinstance(v, str) for v in opt_out):
                        raise ValueError("unknown capability shape")
                    if "rawResponseItem/completed" not in opt_out:
                        opt_out.append("rawResponseItem/completed")
                    capabilities["experimentalApi"] = True
            if json.loads(new) != expected or (expected == value and old != new):
                raise ValueError("forwarded bytes differ from allowed metadata-only transformation")
            if metadata_site:
                decision = next(decisions, None)
                timestamp = (decision or {}).get("observed_monotonic_ns")
                if type(timestamp) is not int or timestamp < 0:
                    raise ValueError("missing rewrite decision timestamp")
                expected_decision = {"method": method, "id": value.get("id"), "changed": expected != value,
                                     "original_sha256": sha_bytes(old), "forwarded_sha256": sha_bytes(new),
                                     "original_bytes": len(old), "forwarded_bytes": len(new),
                                     "observed_monotonic_ns": timestamp}
                if decision != expected_decision:
                    raise ValueError("rewrite decision differs from captured frame identities")
            result["checked_frames"] += 1
        if next(decisions, None) is not None:
            raise ValueError("extra rewrite decisions")
    except (OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        result["errors"].append(str(error))
    return result


def checked_module(path, expected, name):
    path = Path(path)
    if sha256(path) != expected:
        raise ValueError(f"frozen helper digest mismatch: {path}")
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


STRICT = checked_module(GATE / "batch_analysis.py", GATE_SHA, "wl_mechanism_strict")
PROSPECTIVE = checked_module(GATE / "batch_analysis.py", GATE_SHA, "wl_mechanism_prospective")
strict_inventory = STRICT.inventory
usage = STRICT.usage


def tail_proof(turn_key, events, thread_events, usage_events, interrupt_count, grace):
    """One conditional unfinished response, never one response per message item.

    This retains the predecessor's typed identity, paired-item, unchanged usage,
    lifecycle, single directive, no-tool and allowed-method requirements. Only
    its exactly-one assistant-item predicate becomes one-or-more. Unknown event
    boundaries and unpaired items are not waived. The source assumptions above
    are required even when this visible-stream predicate passes.
    """
    result = {"thread_id": turn_key[0], "turn_id": turn_key[1], "response_count_upper": None,
              "proof": None, "reason": "isolated response tail not established"}
    starts = [n for n, v in events if v.get("method") == "turn/started"]
    directives = [n for n, v in events if STRICT.directive(v)]
    completions = [(n, v) for n, v in events if v.get("method") == "turn/completed"]
    if interrupt_count != 1 or len(starts) != 1 or len(directives) != 1 or len(completions) != 1:
        return result
    dseq = directives[0]
    endseq, completed = completions[0]
    if not starts[0] <= dseq < endseq or completed["params"]["turn"].get("status") != "interrupted":
        return result
    seqs, totals = usage_events
    previous_index = bisect_left(seqs, dseq)-1
    previous_seq, prior = (seqs[previous_index], totals[previous_index]) if previous_index >= 0 else (-1, (0,)*4)
    startseq = max(starts[0], previous_seq+1)
    thread_seqs, values = thread_events
    pre, post = [], []
    for index in range(bisect_left(thread_seqs, startseq), bisect_left(thread_seqs, endseq+1)):
        sequence, value = thread_seqs[index], values[index]
        scoped = STRICT.key(value)
        if scoped is not None and scoped != turn_key:
            return result
        (pre if sequence <= dseq else post).append(value)
    if any(v.get("method") not in STRICT.PRE_METHODS for v in pre):
        return result
    if any(v.get("method") not in STRICT.POST_METHODS for v in post):
        return result
    started, ended = Counter(), Counter()
    assistant_items = 0
    for value in pre:
        if value.get("method") not in {"item/started", "item/completed"}:
            continue
        item = value["params"].get("item") or {}
        if item.get("type") not in {"userMessage", "reasoning", "agentMessage"} or not isinstance(item.get("id"), str):
            return result
        identity = item["type"], item["id"]
        (started if value["method"] == "item/started" else ended)[identity] += 1
        assistant_items += value["method"] == "item/completed" and item["type"] == "agentMessage"
    if started != ended or any(count != 1 for count in started.values()) or assistant_items < 1:
        return result
    post_started = 0
    for value in post:
        item = (value.get("params") or {}).get("item")
        if item is not None and (item.get("type") not in {"reasoning", "agentMessage"} or not isinstance(item.get("id"), str)):
            return result
        post_started += value.get("method") == "item/started"
        if value.get("method") == "thread/tokenUsage/updated":
            current = usage(value["params"]["tokenUsage"]["total"], camel=True)
            if tuple(current[name] for name in STRICT.FIELDS) != prior:
                return result
    if post_started > 1 or (grace["outcome"] == "forwarded-after-output-resumed" and post_started != 1):
        return result
    result.update(response_count_upper=1, proof="isolated_normal_response_tail", reason=None,
                  previous_usage_sequence=previous_seq, response_start_sequence=startseq,
                  directive_sequence=dseq, completion_sequence=endseq, paired_items=len(started),
                  paired_assistant_items=assistant_items, post_directive_unfinished_items=post_started,
                  no_tool_boundary=True, conditional_assumptions=POLICY["assumptions"])
    return result


# A private imported module receives the prospective policy. No historical file,
# module used by another process, capture, or previously admitted result is edited.
PROSPECTIVE.tail_proof = tail_proof
inventory = PROSPECTIVE.inventory


def launched(row):
    return bool(row.get("started_at")) and row.get("launch_status") in {
        "running", "completed", "supervisor_stopped"}


def mean_bound(rows, metric):
    bounds = [row["measurement"]["bounds"][metric] for row in rows]
    if not bounds:
        raise ValueError("empty condition group")
    for bound in bounds:
        low, high = bound["lower"], bound["upper"]
        if type(low) not in (int, float) or not math.isfinite(low) or low < 0:
            raise ValueError("invalid lower bound")
        if high is not None and (type(high) not in (int, float) or not math.isfinite(high) or high < low):
            raise ValueError("invalid upper bound")
    return {"lower": sum(b["lower"] for b in bounds)/len(bounds),
            "upper": None if any(b["upper"] is None for b in bounds) else sum(b["upper"] for b in bounds)/len(bounds)}


def contrast(rows, variant):
    selected = [r for r in rows if r["condition"] in {"control", variant}]
    groups = {name: [r for r in selected if r["condition"] == name] for name in ("control", variant)}
    result = {"variant": variant, "status": "incomplete", "counts": {"control": len(groups["control"]), "variant": len(groups[variant])},
              "row_ids": [r["id"] for r in selected], "primary_metric": METRICS[0],
              "interpretation": "Independent group means; positive variant-minus-control means more tokens with the intervention. Accounting envelopes are not sampling confidence intervals."}
    if not all(groups.values()) or any(not launched(r) or not r.get("measurement", {}).get("bounds") for r in selected):
        result["reason"] = "A planned row is unlaunched or lacks defensible accounting; no outcome is dropped."
        return result
    try:
        for metric in METRICS:
            control, changed = (mean_bound(groups[name], metric) for name in ("control", variant))
            lower = None if control["upper"] is None else changed["lower"]-control["upper"]
            upper = None if changed["upper"] is None else changed["upper"]-control["lower"]
            pct_low = None if control["upper"] is None or control["upper"] <= 0 else 100*(changed["lower"]-control["upper"])/control["upper"]
            pct_high = None if changed["upper"] is None or control["lower"] <= 0 else 100*(changed["upper"]-control["lower"])/control["lower"]
            result[metric] = {"control_mean": control, "variant_mean": changed,
                              "variant_minus_control": {"lower": lower, "upper": upper},
                              "increase_percent": {"lower": pct_low, "upper": pct_high}}
        result["status"] = "available"
    except (KeyError, TypeError, ValueError) as error:
        result["reason"] = str(error)
    return result


def permutation_test(rows, variant, *, phase, randomized, mixed_waves=False, max_allocations=200000):
    """Exact blocked randomization enumeration for a one-sided increase test.

    Interval p bounds maximize/minimize T(permutation)-T(observed) over each
    workflow's interval, before counting assignments. They conservatively enclose
    every compatible exact p value, not a p value of convenient interval midpoints.
    Runtime is O(A*n) for A allowed assignments and n workflows; A is combinatorial
    and explicitly capped. Raw event processing elsewhere is indexed/linear.
    """
    if phase != "confirmation":
        return {"status": "exploratory_only", "reason": "Fresh fixed-N confirmation is separate from candidate selection."}
    result = {"status": "unavailable", "alternative": "variant uses more raw tokens", "unit": "whole workflow"}
    if not randomized:
        return {**result, "reason": "No verified frozen random allocation scheme."}
    selected = [r for r in rows if r["condition"] in {"control", variant}]
    if contrast(selected, variant)["status"] != "available":
        return {**result, "reason": "Incomplete planned observations."}
    bounds = [r["measurement"]["bounds"][METRICS[0]] for r in selected]
    if any(b["upper"] is None for b in bounds):
        return {**result, "reason": "Unknown accounting upper bound."}
    n_variant = sum(r["condition"] == variant for r in selected)
    n_control = len(selected)-n_variant
    # Common-denominator integer weights preserve exact ties for integer token
    # totals, avoiding floating-point tie tolerances at tens of millions of tokens.
    observed = [n_control if r["condition"] == variant else -n_variant for r in selected]
    blocks = defaultdict(list)
    for index, entry in enumerate(selected):
        blocks[entry.get("block_id", "all")].append(index)
    allocations = 1
    choices = []
    for indices in blocks.values():
        k = sum(selected[index]["condition"] == variant for index in indices)
        possible_count = math.comb(len(indices), k)
        if possible_count > max_allocations:
            return {**result, "reason": "Exact block enumeration exceeds predeclared computational cap.", "allocations": possible_count}
        waves = defaultdict(set)
        if mixed_waves:
            for index in indices:
                wave = selected[index].get("wave")
                if wave is None:
                    return {**result, "reason": "Missing frozen wave identity."}
                waves[wave].add(index)
            if any(len({selected[i]["condition"] for i in wave}) != 2 for wave in waves.values()):
                return {**result, "reason": "Observed assignment violates the frozen mixed-wave restriction."}
        allowed = []
        for assignment in combinations(indices, k):
            assigned = set(assignment)
            if mixed_waves and any(not wave.intersection(assigned) or wave <= assigned for wave in waves.values()):
                continue
            allowed.append(assignment)
        allocations *= len(allowed)
        if not allocations or allocations > max_allocations:
            return {**result, "reason": "Exact randomization enumeration exceeds cap or has no allowed assignment.", "allocations": allocations}
        choices.append(allowed)
    definitely, possibly = 0, 0
    for assignment in product(*choices):
        variant_indices = {index for part in assignment for index in part}
        smallest = largest = 0
        for index, bound in enumerate(bounds):
            delta = (n_control if index in variant_indices else -n_variant)-observed[index]
            smallest += delta*(bound["lower"] if delta >= 0 else bound["upper"])
            largest += delta*(bound["upper"] if delta >= 0 else bound["lower"])
        definitely += smallest >= 0
        possibly += largest >= 0
    return {**result, "status": "available", "allocations": allocations,
            "p_one_sided_lower": definitely/allocations, "p_one_sided_upper": possibly/allocations,
            "exact_accounting": all(b["lower"] == b["upper"] for b in bounds),
            "method": "Exact label permutations preserving condition counts within frozen blocks and the declared mixed-wave restriction; ties included.",
            "mixed_waves": mixed_waves,
            "assumptions": "Frozen randomized allocation and sharp-null exchangeability; no interference between whole-workflow assignments; fixed-N fresh confirmation; bounded usage satisfies the separately stated visibility assumptions."}


def text_digest(value):
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def valid_transform(event, condition):
    try:
        expected = {"patch-applied": ACK, "command-result": GUIDANCE}[event["site"]].encode()
        original, forwarded = event["original_prompt"].encode(), event["forwarded_prompt"].encode()
        start, end = event["cue_start"], event["cue_end"]
        if type(start) is not int or type(end) is not int or not 0 <= start <= end <= len(original):
            return False
        if original[start:end] != expected:
            return False
        replacement = expected
        if TARGETS.get(condition) == event["site"]:
            replacement = UNLIMITED.encode() if condition == "ack-validation-unlimited" else b""
        return forwarded == original[:start]+replacement+original[end:]
    except (AttributeError, KeyError, TypeError):
        return False


def phase_kind(manifest):
    return manifest.get("phase_kind", "screening")


def design_errors(manifest, rows):
    errors = []
    randomization = manifest.get("randomization", {})
    if phase_kind(manifest) != "confirmation":
        return errors
    expected = randomization.get("block_condition_counts")
    if not isinstance(expected, dict) or not expected:
        return ["missing frozen fixed-N block counts"]
    actual = defaultdict(Counter)
    waves = defaultdict(list)
    for row in rows:
        actual[row.get("block_id")][row["condition"]] += 1
        waves[(row.get("block_id"), row.get("wave"))].append(row)
    if dict(actual) != expected:
        errors.append("observed manifest rows differ from frozen fixed-N block counts")
    if randomization.get("mixed_waves"):
        for (_, wave), entries in waves.items():
            if wave is None or len(entries) != 3 or len({r["condition"] for r in entries}) != 2:
                errors.append("confirmation requires exactly three workflows and both conditions in every frozen wave")
                break
    if any(r.get("phase") != manifest.get("phase") for r in rows):
        errors.append("mixed phase rows cannot enter fresh confirmation")
    return errors


def prompt_chains(trace, threads, captures, run_id, condition):
    errors, exposures = [], []
    activations = [e for e in trace if e.get("event") == "activation"]
    if len(activations) != 1:
        errors.append("expected exactly one experiment activation")
    thread_agent = {}
    agent_threads = defaultdict(set)
    for thread in threads:
        identity, agent = thread.get("thread_id"), thread.get("agent_id")
        if identity in thread_agent and thread_agent[identity] != agent:
            errors.append("ambiguous provider-thread agent identity")
        thread_agent[identity] = agent
        agent_threads[agent].add(identity)
    if any(len(identities) != 1 for agent, identities in agent_threads.items() if agent is not None):
        errors.append("multiple provider threads for one agent; trace occurrence linking is ambiguous")
    requests_by_prompt = defaultdict(deque)
    eligible_requests, matched_requests = set(), set()
    turns = {}
    for capture in captures:
        capture_path = capture["path"]
        replies = {}
        for sequence, value in enumerate(capture["servers"]):
            if "method" not in value and "id" in value and isinstance(value.get("result"), dict):
                turn = (value["result"].get("turn") or {}).get("id")
                if isinstance(turn, str):
                    identity = STRICT.rpc_id(value)
                    if identity in replies:
                        errors.append("ambiguous turn/start reply identity")
                    replies[identity] = (turn, sequence)
            k = STRICT.key(value)
            if k is None:
                continue
            turn_data = turns.setdefault((capture_path, *k), {"actions": [], "raw_response_ids": [], "responses": []})
            if value.get("method") == "item/completed":
                item = value["params"].get("item") or {}
                kind = item.get("type")
                if kind not in {"userMessage", "reasoning"}:
                    action = {"sequence": sequence, "item_id": item.get("id"), "type": kind}
                    if kind == "agentMessage":
                        action["directives"] = [line for line in item.get("text", "").splitlines() if line.startswith("@work-leaf ")]
                    elif kind == "commandExecution":
                        action.update(command=item.get("command"), exit_code=item.get("exitCode"))
                    turn_data["actions"].append(action)
            if value.get("method") == "turn/completed":
                turn_data["turn_status"] = value["params"]["turn"].get("status")
        for identity, response in capture["inventory"].get("raw_responses", {}).items():
            k = (capture_path, response["thread_id"], response["turn_id"])
            turn_data = turns.setdefault(k, {"actions": [], "raw_response_ids": [], "responses": []})
            turn_data["raw_response_ids"].append(identity)
            turn_data["responses"].append({"response_id": identity, "usage": response["usage"]})
        for gap in capture["inventory"].get("gaps", []):
            k = (capture_path, gap["thread_id"], gap["turn_id"])
            turns.setdefault(k, {"actions": [], "raw_response_ids": [], "responses": []})["missing_tail"] = gap
        for sequence, value in enumerate(capture["clients"]):
            if value.get("method") != "turn/start":
                continue
            params = value.get("params") or {}
            identity = STRICT.rpc_id(value)
            thread = params.get("threadId")
            inputs = params.get("input")
            if not isinstance(inputs, list) or len(inputs) != 1 or inputs[0].get("type") != "text":
                continue
            text = inputs[0].get("text")
            if isinstance(text, str) and text.startswith(("work-leaf patch applied\n", "work-leaf command result\n")):
                eligible_requests.add((capture_path, identity))
            if not isinstance(text, str) or identity not in replies or thread not in thread_agent:
                continue
            turn, reply_sequence = replies[identity]
            record = {"capture": capture_path, "thread_id": thread, "turn_id": turn,
                      "client_sequence": sequence, "reply_sequence": reply_sequence,
                      "rpc_id": value["id"], "forwarded_text": text}
            requests_by_prompt[(thread_agent[thread], text_digest(text))].append(record)
    sites = defaultdict(lambda: {"exposures": 0, "changed": 0, "byte_delta": 0, "linked": 0})
    expected_sequence = 1
    for event in trace:
        if event.get("schema", "work-leaf-bench-experiment-v1") != "work-leaf-bench-experiment-v1":
            errors.append("unknown experiment trace schema")
        if event.get("run_id") != run_id or event.get("condition") != condition:
            errors.append("trace run or condition identity mismatch")
        if event.get("event") == "activation":
            continue
        if event.get("event") != "prompt":
            errors.append("unknown experiment trace event")
            continue
        if event.get("sequence") != expected_sequence:
            errors.append("noncontiguous experiment trace sequence")
        expected_sequence += 1
        original, forwarded = event.get("original_prompt"), event.get("forwarded_prompt")
        if not isinstance(original, str) or not isinstance(forwarded, str):
            errors.append("trace prompt text missing")
            continue
        original_bytes, forwarded_bytes = len(original.encode()), len(forwarded.encode())
        if not valid_transform(event, condition):
            errors.append("forwarded prompt differs from the frozen exact-span intervention")
        if (event.get("original_bytes"), event.get("forwarded_bytes"), event.get("byte_delta"), event.get("changed")) != (
                original_bytes, forwarded_bytes, forwarded_bytes-original_bytes, original != forwarded):
            errors.append("trace byte arithmetic or changed flag mismatch")
        site = event.get("site")
        if site not in {"patch-applied", "command-result"}:
            errors.append("unknown prompt site")
        sites[site]["exposures"] += 1
        sites[site]["changed"] += original != forwarded
        sites[site]["byte_delta"] += forwarded_bytes-original_bytes
        record = {key: value for key, value in event.items() if key not in {"original_prompt", "forwarded_prompt"}}
        record.update(original_sha256=text_digest(original), forwarded_sha256=text_digest(forwarded),
                      link_rule="same-agent exact-forwarded-text occurrence and typed RPC reply")
        candidates = requests_by_prompt[(event.get("agent_id"), text_digest(forwarded))]
        if not candidates:
            record["link_status"] = "unmatched"
            errors.append(f"unmatched forwarded prompt at trace sequence {event.get('sequence')}")
        else:
            request = candidates.popleft()
            if request.pop("forwarded_text") != forwarded:
                raise ValueError("prompt hash collision")
            record.update(request, link_status="linked")
            matched_requests.add((request["capture"], STRICT.rpc_id({"id": request["rpc_id"]})))
            sites[site]["linked"] += 1
            turn_data = turns.get((request["capture"], request["thread_id"], request["turn_id"]), {})
            record["following_turn"] = turn_data
            record["usage_scope"] = "Unique completed response-ID usage in the immediately following provider turn; never added to workflow totals. Missing tail stays separate."
        exposures.append(record)
    unmatched_requests = eligible_requests-matched_requests
    if unmatched_requests:
        errors.append(f"captured eligible prompt requests absent from trace: count={len(unmatched_requests)}")
    return {"status": "available" if not errors else "incomplete", "errors": errors,
            "activation_count": len(activations), "sites": dict(sites), "exposures": exposures,
            "recognized_boundary_requests": len(eligible_requests), "unmatched_boundary_requests": len(unmatched_requests),
            "response_count_is_exhaustive_model_call_count": False,
            "causal_claim": "Observed intermediate behavior, not independent causal attribution or proof that every model continuation was captured."}


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def resolve(path):
    path = Path(path)
    return path if path.is_absolute() else ROOT / path


def analyze_manifest(manifest, config, quality=None):
    helper = STRICT.checked_module(config["stage_helper"], "wl_mechanism_stages")
    helper.SESSIONS = Path(os.environ.get("CODEX_HOME", str(Path.home()/".codex"))) / "sessions"
    sources = {str(Path(__file__).resolve()): sha256(__file__),
               str(GATE/"batch_analysis.py"): GATE_SHA,
               str(resolve(config["stage_helper"]["path"])): config["stage_helper"]["sha256"]}

    def remember(path):
        path = resolve(path)
        sources[str(path)] = sha256(path)
        return path

    integrity_errors = []
    try:
        phase_path = remember(manifest["phase_manifest"])
        frozen = read_json(phase_path)
        admitted = read_json(remember(phase_path.parent/"RUN-ONCE/admission.json"))
        if sha256(phase_path) != manifest["phase_manifest_sha256"] or sha256(phase_path) != admitted.get("manifest_sha256"):
            raise ValueError("phase manifest differs from prelaunch admission identity")
        for entry in frozen["files"]:
            if sha256(remember(entry["path"])) != entry["sha256"]:
                raise ValueError(f"frozen source identity differs: {entry['path']}")
        for key in ("study", "phase", "phase_kind", "base_commit", "task_list_sha256", "model", "reasoning_effort", "source_commit", "randomization"):
            if manifest.get(key) != frozen.get(key):
                raise ValueError(f"score manifest differs from frozen phase: {key}")
        expected = {r["run_id"]: r for r in frozen["schedule"]}
        if len(expected) != len(manifest["runs"]):
            raise ValueError("analysis row inventory differs from admitted schedule")
        for row in manifest["runs"]:
            declaration = expected[row.get("run_id", row.get("id"))]
            for key in ("condition", "phase", "block_id", "wave", "workflow", "artifact", "report", "prompt_trace", "experiment_manifest"):
                if row.get(key) != declaration.get(key):
                    raise ValueError(f"admitted workflow identity differs: {key}")
        config_history = read_json(remember(phase_path.parent/"config-history.json"))
        if any(snapshot.get("drift_from_baseline") not in {"unchanged", "formatting_only", "known_bookkeeping_only"}
               for snapshot in config_history["snapshots"]):
            raise ValueError("behavioral or unclassified configuration drift prevents causal inference")
    except (OSError, ValueError, TypeError, KeyError) as error:
        integrity_errors.append(str(error))

    observations = []
    identifiers = set()
    for entry in manifest["runs"]:
        identity = entry.get("id", entry.get("run_id"))
        if not isinstance(identity, str) or identity in identifiers:
            raise ValueError("missing or duplicate run identity")
        identifiers.add(identity)
        if entry.get("workflow", "work-leaf-concurrent") != "work-leaf-concurrent":
            raise ValueError("this study analyzes only concurrent WL workflows")
        row = {key: entry.get(key) for key in ("condition", "phase", "block_id", "wave", "launcher_exit_code", "launch_status", "started_at", "finished_at")}
        row.update(id=identity, workflow_result="unknown")
        try:
            if not launched(entry):
                raise ValueError("planned workflow was not launched; row remains visible")
            artifact = resolve(entry["artifact"])
            report = read_json(remember(entry["report"]))
            observed_path = remember(artifact/"observation/analysis.json")
            observed = read_json(observed_path)
            row["workflow_result"] = report.get("workflow_result", report.get("result", "unknown"))
            row["duration_seconds"] = report.get("duration_seconds")
            combined = {"gaps": [], "errors": [], "raw_responses": {}, "captures": []}
            strict_combined = {"gaps": [], "errors": [], "raw_responses": {}}
            captures = []
            for app in sorted((artifact/"observation/app-server").glob("*")):
                if not app.is_dir():
                    continue
                clients = list(STRICT.records(remember(app/"client-to-server.raw")))
                servers = list(STRICT.records(remember(app/"server-to-client.raw")))
                grace_path = app/"provider-usage-grace.jsonl"
                grace = list(STRICT.records(remember(grace_path))) if grace_path.is_file() else None
                inv = inventory(clients, servers, grace)
                strict = strict_inventory(clients, servers, grace)
                provenance = capture_provenance(app)
                sources.update(provenance["source_sha256"])
                inv["errors"].extend(provenance["errors"])
                strict["errors"].extend(provenance["errors"])
                for target, capture_inventory in ((combined, inv), (strict_combined, strict)):
                    target["gaps"].extend(capture_inventory["gaps"])
                    target["errors"].extend(capture_inventory["errors"])
                    for response_id, response in capture_inventory["raw_responses"].items():
                        if response_id in target["raw_responses"] and target["raw_responses"][response_id] != response:
                            target["errors"].append("conflicting response identity across captures")
                        target["raw_responses"][response_id] = response
                combined["captures"].append({"path": str(app), "provenance": provenance,
                                             **{k: v for k, v in inv.items() if k not in {"gaps", "raw_responses"}}})
                captures.append({"path": str(app), "clients": clients, "servers": servers, "inventory": inv})
                for filename in ("invocation.json", "response-usage.json", "raw-response-rewrites.jsonl", "client-to-server.forwarded.raw", "server-to-client.forwarded.raw"):
                    if (app/filename).is_file():
                        remember(app/filename)
            if not captures:
                combined["errors"].append("no primary app-server capture")
            if report.get("agent_model") != POLICY["model"] or report.get("agent_reasoning_effort") != POLICY["reasoning_effort"]:
                combined["errors"].append("report model or effort differs from frozen ceiling policy")
            if usage(report.get("total_workflow_usage", {})) != usage(observed["usage_scopes"]["total_workflow"]):
                combined["errors"].append("report and strict observer usage disagree")
            row["measurement"] = STRICT.accounting(observed, combined, POLICY)
            row["predecessor_strict_measurement"] = STRICT.accounting(observed, strict_combined, POLICY)
            row["stage_recorded_usage"] = helper.stage_usage(observed, "normal-work-leaf")
            row["stage_usage_scope"] = "Recorded lower bounds; uncertain tails not allocated to stages."
            row["generation_activity"] = STRICT.stage_activity(helper, observed_path, observed, "work-leaf")
            row["mechanism_observations"] = observed.get("mechanisms", {})
            row["exact_completed_response_id_count"] = len(combined["raw_responses"])
            row["exhaustive_model_call_count"] = False
            try:
                trace_path = remember(entry["prompt_trace"])
                row["prompt_activity"] = prompt_chains(list(STRICT.records(trace_path)), observed.get("threads", []), captures, identity, entry["condition"])
            except (KeyError, ValueError, OSError) as error:
                row["prompt_activity"] = {"status": "unavailable", "errors": [str(error)]}
            # Capture provenance digests and matched rollout digests remain in the
            # strict observer/stage evidence as well as this direct source inventory.
        except (OSError, ValueError, TypeError, KeyError) as error:
            row["measurement"] = {"status": "ineligible", "reasons": [str(error)], "bounds": None}
        observations.append(row)
    comparisons = []
    phase = manifest.get("phase", "screening")
    kind = phase_kind(manifest)
    randomization = manifest.get("randomization", {})
    design_findings = design_errors(manifest, observations)
    randomized = randomization.get("scheme") == "within_block" and randomization.get("unit") == "workflow"
    if design_findings or integrity_errors or any(not row.get("block_id") for row in observations):
        randomized = False
    for variant in sorted({r["condition"] for r in observations} - {"control"}):
        compared = contrast(observations, variant)
        exposure_ok = all(r.get("prompt_activity", {}).get("status") == "available" and
                          (r["condition"] == "control" or r["prompt_activity"].get("sites", {}).get(TARGETS.get(r["condition"]), {}).get("changed", 0) > 0)
                          for r in observations if r["condition"] in {"control", variant})
        compared["intervention_exposure_verified"] = exposure_ok
        compared["permutation"] = permutation_test(observations, variant, phase=kind, randomized=randomized and exposure_ok,
                                                   mixed_waves=randomization.get("mixed_waves", False))
        comparisons.append(compared)
    return {"schema_version": 1, "study": manifest.get("study"), "phase": phase, "phase_kind": kind,
            "primary_metric": METRICS[0], "secondary_metric": METRICS[1], "policy": POLICY,
            "observations": observations, "comparisons": comparisons, "quality": quality,
            "design_errors": design_findings, "integrity_errors": integrity_errors,
            "source_sha256": sources, "planned_count": len(observations),
            "launched_count": sum(launched(r) for r in observations),
            "sampling_confidence_interval": False,
            "selection_warning": "Screening is exploratory. Fresh confirmation is analyzed separately. Failed or missing outcomes cannot be replaced or dropped to produce significance."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("analyze", "score"))
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--scorer-config", type=Path, default=GATE/"BATCH-SCORER.json")
    parser.add_argument("--quality", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout-seconds", type=int, default=900)
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError("output already exists; admitted evidence is immutable")
    manifest, config = read_json(args.manifest), read_json(args.scorer_config)
    if args.mode == "score":
        with tempfile.TemporaryDirectory(prefix="work-leaf-mechanism-score-") as directory:
            result = {"runs": STRICT.score_all(manifest, config, args.output.parent, Path(directory), args.timeout_seconds),
                      "frozen_scorer": config, "scope": "Offline final feature checks; no feedback to generation and no quality-based row exclusion."}
    else:
        result = analyze_manifest(manifest, config, read_json(args.quality) if args.quality else None)
    result.setdefault("source_sha256", {}).update({str(args.manifest.resolve()): sha256(args.manifest),
                                                  str(args.scorer_config.resolve()): sha256(args.scorer_config)})
    if args.quality:
        result["source_sha256"][str(args.quality.resolve())] = sha256(args.quality)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2, sort_keys=True)
        stream.write("\n")
    print(args.output)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, KeyError, OSError) as error:
        print(f"analysis error: {error}", file=sys.stderr)
        sys.exit(2)
