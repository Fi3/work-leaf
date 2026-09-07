#!/usr/bin/env python3
"""V2-only provider-free ACK delivery inventory and fixed-N mediator tests.

No token retotal, subjective automatic classification, or v1 trace conversion.
Captured requests/replies are indexed once. Exact randomization is bounded at
12 workflows and 324 allocations; it is not a general unbounded permutation API.
"""

import argparse
from collections import Counter, defaultdict, deque
import hashlib
import importlib.util
from itertools import combinations, product
import json
from pathlib import Path
import re
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SCHEMA = "work-leaf-bench-experiment-v2"
VARIANT = "buildable-work-unit-incremental"
BASE_SHA = "2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78"
ALPHA = .025
CANONICAL = {"model": "gpt-5.5", "reasoning_effort": "xhigh", "compiled_feature": "bench-experiments",
             "provider_route": "existing-codex-chatgpt-subscription", "maximum_concurrent_workflows": 3,
             "base_commit": "c92a0b7060a36eac6db2d869b85e589a7a9480f9",
             "task_list_sha256": "45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a"}
A = (
    "Design tests before implementation when required, but submit a cohesive patch that includes the test and the implementation needed for the shared tree to build.",
    "Design tests before implementation when required. Prefer naturally separable, independently buildable feature increments, keeping mutually dependent test and implementation changes together. Submit each completed increment and continue any remaining requested feature work. Do not split inseparable work or create otherwise unnecessary patches.",
)
B = (
    "Design the needed tests, but submit tests with the implementation needed to keep the shared worktree buildable.",
    "Design the needed tests, and prefer naturally separable, independently buildable feature increments, keeping mutually dependent tests and implementation together. Do not split inseparable work or create otherwise unnecessary patches.",
)
C = (
    "After the focused validation passes, or after you report an external blocker, emit a top-level `@work-leaf done` so review can start. Send another edit only if validation found a concrete issue in your own patch.",
    "After the focused validation passes, or after you report an external blocker, continue with another independently buildable increment if requested feature work remains and can proceed without taking over another agent's work. Otherwise emit a top-level `@work-leaf done` so review can start. Send another edit only to implement remaining requested feature work or to repair a concrete issue that validation found in your own patch.",
)
ACK = "run at most one focused validation step that is relevant to files you touched or checks you added."
GUIDANCE = "\nnext: Reply with the next Work Leaf directive, such as `@work-leaf done`, `@work-leaf edit`, `@work-leaf read`, or another `@work-leaf locks run`. Keep any non-directive explanation brief."
SPANS = {"policy-buildable-work-unit": A, "instruction-tests-work-unit": B,
         "patch-applied-remaining-work": C, "patch-applied-validation": (ACK, ACK),
         "command-result-guidance": (GUIDANCE, GUIDANCE)}
LABELS = {"remaining-work", "validation-repair", "review-repair", "formatting-only",
          "no-op-replayed", "unresolved"}


def identity(value):
    return isinstance(value, str) and bool(value)


def integer(value):
    return type(value) is int and value >= 0


def digest(data):
    return hashlib.sha256(data).hexdigest()


def sha(path):
    h = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024*1024), b""):
            h.update(chunk)
    return h.hexdigest()


def legacy():
    path = HERE/"analyze.py"
    if sha(path) != BASE_SHA:
        raise ValueError("frozen legacy analyzer hash differs")
    spec = importlib.util.spec_from_file_location("wl_frozen_primary_base", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def require_frozen_helper(files, path):
    """The executing helper bytes, not only a saved copy, must be admitted."""
    path = Path(path); actual = sha(path)
    if not any(isinstance(row, dict) and Path(row.get("path", "")).name == path.name and row.get("sha256") == actual for row in files):
        raise ValueError("executing helper is absent from frozen source identity: "+path.name)
    return actual


def audit_trust(frozen, phase_root, *, helper_path=None):
    """Invoke the admitted read-only replay; never accept a saved boolean alone."""
    result = {"valid": False, "errors": []}
    try:
        path = Path(helper_path) if helper_path is not None else HERE/"trust_work_units.py"
        content = path.read_bytes(); actual = digest(content)
        if frozen.get("trust_classifier_sha256") != actual:
            raise ValueError("trust classifier differs from admitted hash")
        require_frozen_helper(frozen["files"], path)
        spec = importlib.util.spec_from_file_location("wl_frozen_work_unit_trust", path)
        module = importlib.util.module_from_spec(spec)
        # Execute the bytes just verified, not a timestamp-cached .pyc file.
        exec(compile(content, str(path), "exec"), module.__dict__)
        replay = module.audit(Path(phase_root))
        if not isinstance(replay, dict) or type(replay.get("valid")) is not bool or not isinstance(replay.get("errors"), list):
            raise ValueError("unsupported trust replay report shape")
        result.update(replay, helper_path=str(path), helper_sha256=actual)
        if replay["valid"] is not True or replay["errors"]:
            raise ValueError("independent trust replay did not pass")
        if replay.get("legacy_flags_preserved") is not True or replay.get("self_contained_global_config_replay") is not False or not isinstance(replay.get("source_identities"), list) or not isinstance(replay.get("global_snapshot_replay"), dict):
            raise ValueError("trust replay provenance/limitation contract differs")
    except (ValueError, KeyError, TypeError, AttributeError, IndexError, OSError) as error:
        result["valid"] = False
        result["errors"].append(str(error))
    return result


def site_of(text):
    if not isinstance(text, str):
        return None
    if text.startswith("work-leaf patch applied\n"):
        return "patch-applied"
    if text.startswith("work-leaf command result\n"):
        return "command-result"
    if text.startswith(("You are running under the work-leaf orchestrator.\n",
                        "You are running as the work-leaf linearize agent.\n")):
        return "policy-injection"
    return None


def expected_spans(event):
    """Independently locate spans only in renderer-owned structural sections.

    Copied instruction/user data are after the first original-instruction header
    or Agent-ID footer. They cannot supply extra policy spans. This is validation,
    not a provider prompt rewrite. Unrecognized structural shapes fail closed.
    """
    text = event["original_prompt"]
    if not isinstance(text, str) or site_of(text) != event["site"]:
        raise ValueError("unknown original renderer boundary")
    data = text.encode(); result = []

    def add(name, start):
        old = SPANS[name][0].encode()
        if data[start:start+len(old)] != old:
            raise ValueError("renderer-owned original span missing")
        result.append((name, start, start+len(old)))

    if event["site"] == "patch-applied":
        if not text.splitlines()[1].startswith("files: "):
            raise ValueError("successful ACK file receipt missing")
        start = data.index(b"\nNext step: ")+len(b"\nNext step: ")
        add("patch-applied-validation", start)
        add("patch-applied-remaining-work", len(data)-len(C[0].encode()))
    elif event["site"] == "command-result":
        # The renderer emits this cue before stdout and any pending diff.
        limit = data.find(b"\nstdout:\n")
        prefix = data if limit < 0 else data[:limit]
        start = prefix.rfind(GUIDANCE.encode())
        if start < 0:
            raise ValueError("command-result guidance missing")
        add("command-result-guidance", start)
    else:
        linearizer = text.startswith("You are running as the work-leaf linearize agent.\n")
        agent = event.get("agent_id")
        if not identity(agent) or linearizer != (agent == "linearize" or agent.startswith("linearize-")):
            raise ValueError("policy renderer role differs from agent identity")
        if linearizer:
            return []
        ends = [i for marker in (b"\n\n--- ", b"\n\nAgent-ID: ") if (i := data.find(marker)) >= 0]
        if not ends:
            raise ValueError("policy owned-section boundary missing")
        owned = data[:min(ends)]
        old = A[0].encode()
        if owned.count(old) != 1:
            raise ValueError("non-linearizer policy requires exactly one A span")
        add("policy-buildable-work-unit", owned.index(old))
        prefix = b"\n- Test requirements remain mandatory. "
        needle = prefix+B[0].encode(); pos = 0
        while (pos := owned.find(needle, pos)) >= 0:
            add("instruction-tests-work-unit", pos+len(prefix)); pos += len(needle)
    return result


def validate_transform(event, condition):
    try:
        if condition not in {"control", VARIANT}:
            raise ValueError("unsupported condition")
        expected = expected_spans(event); spans = event["spans"]
        if not isinstance(spans, list) or len(spans) != len(expected):
            raise ValueError("owned span inventory differs")
        original = event["original_prompt"].encode(); pieces = []; previous = 0
        for row, (name, start, end) in zip(spans, expected):
            if not isinstance(row, dict) or type(row.get("cue_start")) is not int or type(row.get("cue_end")) is not int:
                raise ValueError("invalid span offsets")
            if (row.get("id"), row["cue_start"], row["cue_end"]) != (name, start, end) or start < previous:
                raise ValueError("span identity, order or ownership differs")
            old, variant = SPANS[name]; new = variant if condition == VARIANT else old
            if row.get("original") != old or row.get("replacement") != new:
                raise ValueError("span replacement differs from frozen condition")
            if type(row.get("changed")) is not bool or row["changed"] != (old != new) or type(row.get("byte_delta")) is not int or row["byte_delta"] != len(new.encode())-len(old.encode()):
                raise ValueError("span metadata differs")
            pieces.extend((original[previous:start], new.encode())); previous = end
        pieces.append(original[previous:]); forwarded = b"".join(pieces)
        if event["forwarded_prompt"].encode() != forwarded:
            raise ValueError("forwarded prompt differs beyond declared spans")
        for key, value in (("original_bytes", len(original)), ("forwarded_bytes", len(forwarded)),
                           ("byte_delta", len(forwarded)-len(original))):
            if type(event.get(key)) is not int or event[key] != value:
                raise ValueError("prompt byte arithmetic differs")
        if type(event.get("changed")) is not bool or event["changed"] != (original != forwarded):
            raise ValueError("prompt changed flag differs")
        return []
    except (ValueError, KeyError, TypeError, AttributeError, IndexError) as error:
        return [str(error)]


def rpc(value):
    key = value.get("id")
    if not (identity(key) or type(key) is int):
        raise ValueError("unsupported RPC identity")
    return type(key).__name__, key


def prompt_inventory(trace, threads, captures, run_id, condition):
    errors = []; exposures = []; acks = []; thread_agent = {}; by_prompt = defaultdict(deque)
    prompt_scopes = defaultdict(set)
    eligible = set(); matched = set(); accepted_turns = set()
    try:
        for row in threads:
            if not isinstance(row, dict) or not identity(row.get("thread_id")) or not identity(row.get("agent_id")):
                raise ValueError("unknown provider thread/agent identity")
            t, a = row["thread_id"], row["agent_id"]
            if t in thread_agent and thread_agent[t] != a:
                raise ValueError("ambiguous thread agent identity")
            thread_agent[t] = a
        for capture in captures:
            cap = capture["path"]; replies = {}; request_ids = set()
            for line, value in enumerate(capture["servers"], 1):
                if not isinstance(value, dict):
                    raise ValueError("unknown server frame shape")
                if "method" in value or "id" not in value:
                    continue
                key = rpc(value)
                if key in replies:
                    raise ValueError("duplicate RPC reply identity")
                replies[key] = (value, line)
            for line, value in enumerate(capture["clients"], 1):
                if value.get("method") != "turn/start":
                    continue
                key = rpc(value)
                if key in request_ids:
                    raise ValueError("duplicate turn/start RPC identity")
                request_ids.add(key)
                params = value.get("params", {}); inputs = params.get("input")
                if not isinstance(inputs, list) or len(inputs) != 1 or not isinstance(inputs[0], dict) or inputs[0].get("type") != "text":
                    raise ValueError("unsupported turn input shape")
                text = inputs[0].get("text"); thread = params.get("threadId")
                if not isinstance(text, str) or not identity(thread) or thread not in thread_agent:
                    raise ValueError("unknown request text/thread identity")
                request_key = (cap, *key)
                if site_of(text):
                    eligible.add(request_key)
                reply, reply_line = replies.get(key, ({}, None))
                record = {"capture": cap, "client_line": line, "reply_line": reply_line,
                          "rpc_id": value["id"], "thread_id": thread, "agent_id": thread_agent[thread],
                          "text": text, "request_key": request_key, "status": "unverified"}
                if "result" in reply and "error" in reply:
                    raise ValueError("RPC result and error are mutually exclusive")
                if "error" in reply:
                    error = reply["error"]
                    if not isinstance(error, dict) or type(error.get("code")) is not int or not identity(error.get("message")):
                        raise ValueError("malformed RPC rejection is not known nondelivery")
                    record["status"] = "rejected_not_delivered"
                elif isinstance(reply.get("result"), dict):
                    turn = reply["result"].get("turn")
                    if not isinstance(turn, dict) or not identity(turn.get("id")):
                        raise ValueError("unknown typed turn reply identity")
                    scope = (thread, turn["id"])
                    if scope in accepted_turns:
                        raise ValueError("duplicate accepted turn identity")
                    accepted_turns.add(scope)
                    record.update(status="delivered", turn_id=turn["id"])
                prompt_key = (thread_agent[thread], digest(text.encode()))
                by_prompt[prompt_key].append(record)
                prompt_scopes[prompt_key].add((cap, thread))
        if not trace or trace[0].get("event") != "activation" or sum(x.get("event") == "activation" for x in trace) != 1:
            raise ValueError("expected exactly one initial activation")
        process_id = trace[0].get("process_id")
        if not integer(process_id) or process_id == 0:
            raise ValueError("invalid trace process identity")
        sequence = 0
        for line, event in enumerate(trace, 1):
            if event.get("schema") != SCHEMA or event.get("run_id") != run_id or event.get("condition") != condition or type(event.get("process_id")) is not int or event["process_id"] != process_id:
                raise ValueError("trace schema/run/condition/process differs")
            if event.get("event") == "activation":
                continue
            sequence += 1
            if event.get("event") != "prompt" or type(event.get("sequence")) is not int or event["sequence"] != sequence or not identity(event.get("agent_id")):
                raise ValueError("noncontiguous or unknown prompt record")
            timestamp = event.get("unix_time_ns")
            if not isinstance(timestamp, str) or not timestamp.isascii() or not timestamp.isdecimal():
                raise ValueError("unknown trace timestamp shape")
            problems = validate_transform(event, condition)
            if problems:
                raise ValueError("; ".join(problems))
            prompt_key = (event["agent_id"], digest(event["forwarded_prompt"].encode()))
            candidates = by_prompt[prompt_key]
            if not candidates:
                raise ValueError("trace prompt has no exact captured request")
            if len(prompt_scopes[prompt_key]) != 1:
                raise ValueError("same-prompt occurrence across captures/threads is ambiguous")
            record = candidates.popleft()
            if record.pop("text") != event["forwarded_prompt"]:
                raise ValueError("prompt digest collision")
            if record["status"] == "unverified":
                raise ValueError("request has no accepted typed reply or explicit error")
            matched.add(record.pop("request_key"))
            record.update(sequence=sequence, trace_line=line, site=event["site"],
                          changed=event["changed"], byte_delta=event["byte_delta"],
                          original_sha256=digest(event["original_prompt"].encode()),
                          forwarded_sha256=digest(event["forwarded_prompt"].encode()),
                          span_ids=[x["id"] for x in event["spans"]])
            exposures.append(record)
            if event["site"] == "patch-applied" and record["status"] == "delivered":
                ack_id = "ack:"+digest(json.dumps([run_id, record["capture"], record["rpc_id"], record["thread_id"], record["turn_id"]], separators=(",", ":")).encode())
                acks.append({**record, "ack_id": ack_id, "files_receipt": event["original_prompt"].splitlines()[1],
                             "successful_group_witness": "Frozen renderer-owned successful-ACK boundary plus accepted typed request; not an invented typed commit-event join."})
        if eligible != matched:
            raise ValueError("eligible captured requests and trace are not bidirectionally covered")
    except (KeyError, ValueError, TypeError, AttributeError, IndexError) as error:
        errors.append(str(error))
    return {"status": "available" if not errors else "unverifiable", "errors": errors,
            "delivered_ack_count": len(acks), "exact_primary_count": len(acks) if not errors else None,
            "acks": acks, "exposures": exposures, "accepted_provider_turns": len(accepted_turns),
            "recognized_requests": len(eligible), "matched_requests": len(matched),
            "completed_response_count": None, "exhaustive_model_call_count": False}


def classification_bounds(acks, labels, verify_reference):
    errors = []; lower = upper = 0; counts = Counter(); seen = set()
    try:
        expected = {x["ack_id"] for x in acks}
        if len(expected) != len(acks) or any(not identity(x) for x in expected):
            raise ValueError("ACK identity inventory invalid")
        if not isinstance(labels, list):
            raise ValueError("classification list missing")
        for row in labels:
            if not isinstance(row, dict) or not identity(row.get("ack_id")) or row["ack_id"] in seen or row["ack_id"] not in expected:
                raise ValueError("unknown, duplicate or invalid classification identity")
            seen.add(row["ack_id"]); label = row.get("label")
            if label not in LABELS or not identity(row.get("rationale")):
                raise ValueError("unsupported label or missing rationale")
            references = row.get("evidence")
            if not isinstance(references, list) or not references:
                raise ValueError("classification needs underlying evidence references")
            for ref in references:
                if not isinstance(ref, dict) or not identity(ref.get("path")) or not identity(ref.get("locator")) or not isinstance(ref.get("sha256"), str) or len(ref["sha256"]) != 64 or any(c not in "0123456789abcdef" for c in ref["sha256"]):
                    raise ValueError("invalid classification evidence reference")
                verify_reference(ref)
            counts[label] += 1; lower += label == "remaining-work"; upper += label in {"remaining-work", "unresolved"}
        if seen != expected:
            raise ValueError("classification does not cover every ACK")
    except (KeyError, ValueError, TypeError, OSError) as error:
        errors.append(str(error))
    return {"status": "available" if not errors else "unverifiable", "errors": errors,
            "counts": dict(counts), "bounds": {"lower": lower, "upper": upper} if not errors else None,
            "classification_is_automatic": False}


def randomization_test(rows):
    try:
        if len(rows) != 12 or len({r["id"] for r in rows}) != 12 or any(not identity(r["id"]) for r in rows):
            raise ValueError("exactly twelve unique frozen workflows required")
        blocks = defaultdict(list); observed = set(); bounds = []
        for i, row in enumerate(rows):
            if row["condition"] not in {"control", VARIANT} or not identity(row.get("block_id")) or not integer(row.get("wave")) or row["wave"] == 0:
                raise ValueError("unknown condition/block/wave")
            blocks[row["block_id"]].append(i)
            if row["condition"] == VARIANT:
                observed.add(i)
            bound = row["bound"]
            if not integer(bound["lower"]) or not integer(bound["upper"]) or bound["lower"] > bound["upper"]:
                raise ValueError("count bounds must be finite ordered nonnegative integers")
            bounds.append(bound)
        if len(blocks) != 2:
            raise ValueError("exactly two blocks required")
        choices = []
        for indices in blocks.values():
            waves = defaultdict(set)
            for i in indices:
                waves[rows[i]["wave"]].add(i)
            if len(indices) != 6 or len(observed.intersection(indices)) != 3 or len(waves) != 2 or any(len(w) != 3 or not 0 < len(w & observed) < 3 for w in waves.values()):
                raise ValueError("block/wave allocation violates frozen mixed design")
            allowed = [set(c) for c in combinations(indices, 3) if all(0 < len(set(c) & w) < 3 for w in waves.values())]
            if len(allowed) != 18:
                raise ValueError("block does not contain eighteen allowed allocations")
            choices.append(allowed)
        definitely = possibly = 0
        for parts in product(*choices):
            perm = set.union(*parts); small = large = 0
            for i, b in enumerate(bounds):
                coefficient = int(i in perm)-int(i in observed)
                small += coefficient*(b["lower"] if coefficient >= 0 else b["upper"])
                large += coefficient*(b["upper"] if coefficient >= 0 else b["lower"])
            definitely += small >= 0; possibly += large >= 0
        lower = sum(b["lower"] if i in observed else -b["upper"] for i, b in enumerate(bounds))/6
        upper = sum(b["upper"] if i in observed else -b["lower"] for i, b in enumerate(bounds))/6
        return {"status": "available", "allocations": 324, "p_lower": definitely/324, "p_upper": possibly/324,
                "difference": {"lower": lower, "upper": upper}, "exact_counts": all(b["lower"] == b["upper"] for b in bounds),
                "method": "One-sided treatment-minus-control mean, exact frozen allocation enumeration with ties; interval p bounds conservatively enclose every compatible exact p, not necessarily an attainable joint supremum.",
                "scope": "Frozen mixed-wave assignment regime; no direct-versus-spillover decomposition."}
    except (KeyError, ValueError, TypeError) as error:
        return {"status": "unavailable", "reason": str(error)}


def passes(result):
    return result.get("status") == "available" and result["difference"]["lower"] > 0 and result["p_upper"] <= ALPHA


def phase_errors(frozen, manifest, result):
    errors = []
    if frozen.get("phase_kind") != "confirmation" or any(frozen.get(k) != v for k, v in CANONICAL.items()):
        errors.append("phase differs from prospective fixed work-unit experiment")
    if result.get("manifest_sha256") != manifest.get("phase_manifest_sha256") or result.get("runs") != manifest.get("runs"):
        errors.append("terminal phase result and score manifest differ")
    for key, value in (("frozen_input_integrity_errors", []), ("supervisor_error", None),
                       ("signals_received", []), ("supervisor_wall_timeout", False),
                       ("phase_schedule_exhausted", True), ("all_scheduled_workflows_launched", True),
                       ("unexplained_config_drift_detected", False),
                       ("pending_config_attestation_at_finish", False)):
        if key not in result or type(result[key]) is not type(value) or result[key] != value:
            errors.append("recorded phase incident/incompletion: "+key)
    return errors


def observer_errors(observed):
    """Usage gaps are permitted; source, model and observer-integrity errors are not."""
    try:
        errors = observed["errors"]; complete = observed["capture_complete"]
        if not isinstance(errors, list) or type(complete) is not bool:
            raise ValueError("unknown observer error/completion shape")
        if any(not isinstance(e, str) or not re.fullmatch(r"interrupted provider turn has no complete usage: count=[1-9][0-9]*", e) for e in errors):
            raise ValueError("observer integrity errors beyond permitted missing usage")
        if complete != (not errors):
            raise ValueError("observer completion and error facts disagree")
        strata = observed["model_strata"]
        if not isinstance(strata, list) or not strata or any(not isinstance(s, dict) or s.get("model") != "gpt-5.5" or s.get("effort") != "xhigh" for s in strata):
            raise ValueError("observer model/effort scope differs")
        return []
    except (ValueError, KeyError, TypeError) as error:
        return [str(error)]


def analyze_manifest(path, classifications=None):
    """Retain every frozen row; inference stays gated on complete phase integrity.

    The admitted trust adapter independently replays the phase evidence. Original
    classifier flags remain visible; a new label alone cannot clear an incident.
    """
    base = legacy(); sources = {str(HERE/"analyze.py"): BASE_SHA, str(Path(__file__).resolve()): sha(__file__)}

    def resolve(value):
        p = Path(value)
        return p if p.is_absolute() else ROOT/p

    def remember(value):
        p = resolve(value)
        if str(p) not in sources:
            sources[str(p)] = sha(p)
        return p

    def read(value):
        return json.loads(remember(value).read_text())

    manifest = read(path); frozen = read(manifest["phase_manifest"]); phase_dir = resolve(manifest["phase_manifest"]).parent
    integrity = []; observations = []; trust_result = {"valid": False, "errors": ["not_replayed"]}
    phase_result = {}
    try:
        admission = read(phase_dir/"RUN-ONCE/admission.json")
        if sources[str(resolve(manifest["phase_manifest"]))] != manifest["phase_manifest_sha256"] or admission.get("manifest_sha256") != manifest["phase_manifest_sha256"]:
            raise ValueError("phase identity differs from prelaunch admission")
        for entry in frozen["files"]:
            if sources[str(remember(entry["path"]))] != entry["sha256"]:
                raise ValueError("frozen file inventory differs")
        require_frozen_helper(frozen["files"], __file__)
        phase_result = read(phase_dir/"PHASE-RESULT.json")
        integrity.extend(phase_errors(frozen, manifest, phase_result))
        for key in ("study", "phase", "phase_kind", "base_commit", "task_list_sha256", "source_commit", "model", "reasoning_effort", "randomization"):
            if frozen.get(key) != manifest.get(key):
                raise ValueError("score manifest differs from frozen phase: "+key)
        random = frozen["randomization"]
        if random.get("scheme") != "within_block" or random.get("unit") != "workflow" or random.get("mixed_waves") is not True:
            raise ValueError("missing frozen mixed-block randomization")
        expected_counts = defaultdict(Counter)
        for row in frozen["schedule"]:
            expected_counts[row["block_id"]][row["condition"]] += 1
        if random.get("block_condition_counts") != {k: dict(v) for k, v in expected_counts.items()}:
            raise ValueError("frozen block counts differ from allocation metadata")
        config = read(phase_dir/"config-history.json")
        if not isinstance(config.get("snapshots"), list) or not config["snapshots"]:
            raise ValueError("complete original configuration history missing")
        trust_result = audit_trust(frozen, phase_dir)
        if trust_result.get("valid") is not True or trust_result.get("errors") or trust_result.get("verified_snapshot_indices") != list(range(len(config["snapshots"]))):
            raise ValueError("configuration history needs complete independent trust replay")
        if "helper_path" in trust_result:
            remember(trust_result["helper_path"])
        for name in ("trust-final.json", "trust-evidence.jsonl", "trust-pending.jsonl"):
            if (phase_dir/name).is_file():
                remember(phase_dir/name)
    except (KeyError, ValueError, TypeError, OSError) as error:
        integrity.append(str(error))
    labels = None; classification_errors = []
    if classifications:
        try:
            labels = read(classifications)
            if not isinstance(labels, dict) or labels.get("schema") != "work-leaf-work-unit-classification-v1" or labels.get("phase_manifest_sha256") != manifest.get("phase_manifest_sha256") or not isinstance(labels.get("runs"), dict):
                raise ValueError("classification phase/schema/rows differ")
        except (ValueError, KeyError, TypeError, OSError) as error:
            classification_errors.append(str(error)); labels = None
    actual = {}; unexpected = []
    for row in manifest.get("runs", []):
        rid = row.get("run_id", row.get("id"))
        if not identity(rid) or rid in actual:
            integrity.append("invalid or duplicate result run identity")
            unexpected.append(row)
        else:
            actual[rid] = row
    scheduled = frozen.get("schedule", [])
    scheduled_ids = {r.get("run_id") for r in scheduled}
    if labels is not None and set(labels["runs"]) != scheduled_ids:
        classification_errors.append("classification run inventory differs from frozen schedule")
        labels = None
    if set(actual) != scheduled_ids:
        integrity.append("result rows differ from complete frozen schedule")
    unexpected.extend(r for rid, r in actual.items() if rid not in scheduled_ids)
    for declaration in scheduled:
        rid = declaration.get("run_id"); entry = actual.get(rid, {})
        out = {"id": rid, **{k: declaration.get(k) for k in ("condition", "block_id", "wave")},
               "launch_status": entry.get("launch_status"), "launcher_exit_code": entry.get("launcher_exit_code"), "errors": []}
        observations.append(out)
        try:
            for key in ("condition", "phase", "block_id", "wave", "workflow", "artifact", "report", "prompt_trace", "experiment_manifest"):
                if entry.get(key) != declaration.get(key):
                    raise ValueError("result differs from frozen workflow: "+key)
            if not entry.get("started_at") or not entry.get("finished_at"):
                raise ValueError("workflow not admitted and terminally recorded")
            em = read(entry["experiment_manifest"])
            if em != {"schema": SCHEMA, "run_id": rid, "condition": entry["condition"], "evidence_path": entry["prompt_trace"]}:
                raise ValueError("experiment manifest differs from v2 condition")
            artifact = resolve(entry["artifact"]); observed = read(artifact/"observation/analysis.json"); captures = []
            out["observer_errors"] = observed.get("errors")
            out["errors"].extend(observer_errors(observed))
            report = read(entry["report"])
            out["workflow_result"] = report.get("workflow_result", report.get("result", "unknown"))
            if report.get("agent_model") != "gpt-5.5" or report.get("agent_reasoning_effort") != "xhigh":
                out["errors"].append("published report model/effort differs")
            for app in sorted((artifact/"observation/app-server").iterdir()):
                if not app.is_dir():
                    continue
                provenance = base.capture_provenance(app); sources.update(provenance["source_sha256"])
                if provenance["errors"]:
                    raise ValueError("; ".join(provenance["errors"]))
                captures.append({"path": str(app), "clients": list(base.STRICT.records(remember(app/"client-to-server.raw"))),
                                 "servers": list(base.STRICT.records(remember(app/"server-to-client.raw")))})
            if not captures:
                raise ValueError("no raw app-server capture")
            trace = list(base.STRICT.records(remember(entry["prompt_trace"])))
            out["inventory"] = prompt_inventory(trace, observed["threads"], captures, rid, entry["condition"])
            out["errors"].extend(out["inventory"]["errors"])

            def reference(ref):
                p = resolve(ref["path"]).resolve()
                if not p.is_relative_to(phase_dir.resolve()):
                    raise ValueError("classification evidence must be retained within this phase")
                if sources[str(remember(p))] != ref["sha256"]:
                    raise ValueError("classification evidence hash differs")

            classification_rows = None if labels is None else labels.get("runs", {}).get(rid)
            out["classification"] = classification_bounds(out["inventory"]["acks"], classification_rows, reference)
        except (ValueError, KeyError, TypeError, OSError) as error:
            out["errors"].append(str(error))
    for entry in unexpected:
        observations.append({"id": entry.get("run_id", entry.get("id")), "condition": entry.get("condition"),
                             "launch_status": entry.get("launch_status"), "launcher_exit_code": entry.get("launcher_exit_code"),
                             "errors": ["Unexpected/duplicate result retained outside frozen allocation"],
                             "admitted_to_inference": False})
    for source, expected_hash in sources.items():
        if sha(source) != expected_hash:
            integrity.append("source changed during offline analysis: "+source)
    primary_rows = [{**r, "bound": {"lower": r.get("inventory", {}).get("exact_primary_count"),
                                    "upper": r.get("inventory", {}).get("exact_primary_count")}} for r in observations]
    primary = randomization_test(primary_rows) if not integrity and not any(r["errors"] for r in observations) else {"status": "unavailable", "reason": "Incomplete delivery/source/configuration integrity"}
    secondary = {"status": "not_tested", "reason": "Fixed-sequence primary gate not passed"}
    if passes(primary) and not classification_errors:
        secondary = randomization_test([{**r, "bound": r.get("classification", {}).get("bounds")} for r in observations])
    elif passes(primary):
        secondary = {"status": "unavailable", "reason": "Classification integrity failed"}
    return {"schema": "work-leaf-work-unit-mediator-analysis-v1", "phase": manifest.get("phase"), "observations": observations,
            "primary": primary, "primary_passes": passes(primary), "secondary": secondary, "secondary_passes": passes(secondary),
            "alpha_fixed_sequence": ALPHA, "integrity_errors": integrity, "classification_errors": classification_errors, "source_sha256": sources,
            "trust_replay": trust_result,
            "legacy_behavioral_config_drift_detected": phase_result.get("behavioral_config_drift_detected"),
            "unexplained_config_drift_detected": phase_result.get("unexplained_config_drift_detected"),
            "pending_config_attestation_at_finish": phase_result.get("pending_config_attestation_at_finish"),
            "token_retotal": None, "causal_share": None,
            "scope": "All admitted workflow ACK counts; secondary semantic coding is supplied separately, never inferred from reason titles. Token accounting remains the separately frozen audit."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--classifications", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError("output exists; reports are create-new")
    result = analyze_manifest(args.manifest, args.classifications)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2, sort_keys=True); stream.write("\n")
    print(args.output)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, KeyError, TypeError, OSError) as error:
        print(f"work-unit analysis error: {error}", file=sys.stderr)
        sys.exit(2)
