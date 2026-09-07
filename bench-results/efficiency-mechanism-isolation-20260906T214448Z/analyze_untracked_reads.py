#!/usr/bin/env python3
"""Prospective v3 read delivery and fixed-N raw-token primary, provider-free.

No v1/v2 trace conversion, substitute endpoint, inferred usage, or causal share.
Legacy reuse is limited to source-pinned pure nonfactor/source/trust validators.
Read/event joins are indexed; the fixed randomization costs 324 * 12 operations.
"""
import argparse
from collections import Counter, defaultdict, deque
from functools import lru_cache
import hashlib
from itertools import combinations, product
import json
from pathlib import Path, PurePosixPath
import stat
import sys
import tempfile
import types

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SCHEMA = "work-leaf-bench-experiment-v3"
VARIANT = "untracked-read-inline"
ALPHA = .025
PINS = {"analyze_work_units.py": "e59e6a22bf1ab09e0aabec86791cefb54cd3f3d38167a4c9822cdce1d09b50d3",
        "analyze.py": "2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78"}
HEADER = "work-leaf file text\n"
BUNDLE_INTRO = "Exact file text is in an orchestrator context bundle instead of this chat to keep the agent session compact.\nContext bundle: "
BUNDLE_PERMISSION = "\nYou may read this temporary bundle file for the exact mediated file text. Do not edit the bundle; project writes still require `@work-leaf edit`.\nBundled files:\n"


def identity(value):
    return isinstance(value, str) and bool(value)


def integer(value):
    return type(value) is int and value >= 0


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def regular(path):
    path = Path(path).absolute()
    require(path.resolve() == path and stat.S_ISREG(path.lstat().st_mode), "noncanonical or nonregular source: "+str(path))
    return path


def sha(path):
    h = hashlib.sha256()
    with regular(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024*1024), b""):
            h.update(block)
    return h.hexdigest()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON object key")
        result[key] = value
    return result


def decode(text):
    def invalid(value):
        raise ValueError("nonfinite JSON value: "+value)
    return json.loads(text, object_pairs_hook=unique_object, parse_constant=invalid)


def records(path):
    values, physical = [], []
    with regular(path).open(encoding="utf-8") as stream:
        for line, text in enumerate(stream, 1):
            if not text.strip():
                continue
            value = decode(text)
            require(isinstance(value, dict), "JSONL record is not an object")
            values.append(value); physical.append(line)
    return values, physical


def compile_bytes(path, content):
    module = types.ModuleType("untracked_read_"+Path(path).stem)
    module.__file__ = str(path)
    # Legacy imports cannot reuse repository timestamp-only .pyc files.
    previous = sys.pycache_prefix; no_write = sys.dont_write_bytecode
    with tempfile.TemporaryDirectory(prefix="wl-read-offline-import-") as cache:
        try:
            sys.pycache_prefix = cache; sys.dont_write_bytecode = True
            exec(compile(content, str(path), "exec"), module.__dict__)
        finally:
            sys.pycache_prefix = previous; sys.dont_write_bytecode = no_write
    return module


@lru_cache(maxsize=2)
def pinned(name):
    path = regular(HERE/name); content = path.read_bytes()
    require(digest(content) == PINS[name], "frozen pure helper differs: "+name)
    return compile_bytes(path, content)


def require_helper(frozen, path):
    path = regular(path)
    rows = [r for r in frozen.get("files", []) if isinstance(r, dict) and r.get("path") == str(path)]
    require(len(rows) == 1 and rows[0].get("role") == "frozen-evidence", "executing helper lacks unique exact frozen path: "+str(path))
    actual = sha(path)
    require(rows[0].get("sha256") == actual, "executing helper differs from admission")
    return actual


def fnv(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte)*0x100000001b3) & ((1 << 64)-1)
    return f"fnv64:{value:016x}; bytes:{len(data)}"


def byte_range(data, start, end):
    require(integer(start) and integer(end) and start <= end <= len(data), "invalid owned byte interval")
    # Inputs were encoded from valid Unicode. Constant-time boundary tests avoid
    # decoding every preceding prefix again for every snapshot (quadratic work).
    require(all(i == len(data) or data[i] & 0xc0 != 0x80 for i in (start, end)), "offset splits UTF-8 scalar")
    return data[start:end]


def validate_read(event, condition):
    result = {"errors": [], "snapshot_evidence": []}
    try:
        require(condition in {"control", VARIANT}, "unsupported v3 condition")
        require(event.get("event") == "read-response" and event.get("site") == "file-read" and event.get("schema") == SCHEMA, "unsupported read event")
        baseline = event["baseline_prompt"].encode(); alternative = event["inline_candidate_prompt"].encode()
        require(baseline.startswith(HEADER.encode()) and alternative.startswith(HEADER.encode()), "missing actual file-read boundary")
        snapshots = event["snapshots"]; failures = event["failures"]; paths = event["requested_paths"]
        require(isinstance(snapshots, list) and isinstance(failures, list) and isinstance(paths, list) and all(identity(p) for p in paths), "invalid read metadata inventory")
        projects = []; untracked = []; explicit = False; seen = set()
        for row in snapshots:
            require(isinstance(row, dict) and identity(row.get("path")) and integer(row.get("bytes")) and identity(row.get("digest")), "invalid snapshot metadata")
            kind = row.get("class")
            require(kind in {"untracked", "changed", "unchanged", "explicit-bundle"}, "unknown snapshot class")
            if kind == "explicit-bundle":
                explicit = True
            else:
                require(not explicit and row["path"] not in seen, "project snapshot order or duplicate identity differs")
                seen.add(row["path"]); projects.append(row["path"])
            if kind == "untracked":
                text = byte_range(alternative, row["inline_body_start"], row["inline_body_end"])
                require(len(text) == row["bytes"] and fnv(text) == row["digest"], "snapshot body length/digest differs")
                untracked.append((row, text))
                result["snapshot_evidence"].append({"path": row["path"], "class": kind, "bytes": len(text),
                    "fnv64_label": row["digest"], "sha256": digest(text), "inline_body_start": row["inline_body_start"], "inline_body_end": row["inline_body_end"]})
            else:
                require(row.get("inline_body_start") is None and row.get("inline_body_end") is None, "non-target snapshot gained a body projection")
        require(projects == sorted(projects, key=lambda p: PurePosixPath(p).parts), "normalized project snapshot order differs")
        for row in failures:
            require(isinstance(row, dict) and identity(row.get("path")) and isinstance(row.get("diagnostic"), str), "unknown read failure shape")
        bundle = event["bundle"]
        require(isinstance(bundle, dict) and type(bundle.get("threshold_eligible")) is bool and type(bundle.get("write_succeeded")) is bool, "invalid bundle success/threshold flags")
        threshold = sum(r["bytes"] for r, _ in untracked) > 24576 or any(r["bytes"] > 16384 for r, _ in untracked)
        success = bundle["write_succeeded"]
        require(bundle["threshold_eligible"] == threshold and (not success or threshold), "bundle threshold differs from exact snapshots")
        require(identity(bundle.get("path")) if success else bundle.get("path") is None, "bundle path/success identity differs")
        require(type(event.get("eligible")) is bool and event["eligible"] == success, "eligibility is not actual successful bundle write")
        reason = "bundled-untracked" if success else "bundle-write-failed" if threshold else "below-threshold" if untracked else "no-untracked-project-snapshots"
        require(event.get("eligibility_reason") == reason, "eligibility reason differs")
        component = event["component"]
        if component is None:
            require(explicit and not projects and not failures and baseline == alternative, "absent component is only valid for pure explicit-bundle reads")
        else:
            require(isinstance(component, dict), "invalid component shape")
            bs, be, ins, ine = (component[k] for k in ("baseline_start", "baseline_end", "inline_start", "inline_end"))
            old = byte_range(baseline, bs, be); new = byte_range(alternative, ins, ine)
            require(bs == ins and baseline[:bs] == alternative[:ins] and baseline[be:] == alternative[ine:], "bytes outside owned component differ")
            require((bs > 0) == explicit, "component prefix differs from explicit-bundle inventory")
            pieces = [HEADER.encode()]; cursor = ins+len(HEADER.encode())
            for row, text in untracked:
                heading = ("\n--- "+row["path"]+" ---\n").encode()
                require(row["inline_body_start"] == cursor+len(heading), "snapshot body offset is not renderer-owned")
                ending = b"" if text.endswith(b"\n") else b"\n"
                pieces.extend((heading, text, ending)); cursor += len(heading)+len(text)+len(ending)
            require(new == b"".join(pieces), "inline component differs from exact source formatter")
            if success:
                manifest = HEADER+BUNDLE_INTRO+bundle["path"]+BUNDLE_PERMISSION
                manifest += "".join("- "+r["path"]+" ("+r["digest"]+")\n" for r, _ in untracked)
                require(old == manifest.encode(), "baseline component is not actual bundle manifest formatter")
            else:
                require(old == new and baseline == alternative, "ineligible read changed representation")
            if failures:
                ending = "\nUnavailable file text\n"+"".join("- "+r["path"]+": "+r["diagnostic"]+"\n" for r in failures)
                require(baseline[be:].endswith(ending.encode()), "failure suffix differs from renderer metadata")
        selected = "inline" if condition == VARIANT and success else "baseline"
        require(event.get("selected_candidate") == selected, "selected candidate differs from condition")
        chosen = alternative if selected == "inline" else baseline
        for key, expected in (("baseline_bytes", len(baseline)), ("inline_candidate_bytes", len(alternative)),
                              ("selected_bytes", len(chosen)), ("candidate_byte_delta", len(alternative)-len(baseline)), ("byte_delta", len(chosen)-len(baseline))):
            require(type(event.get(key)) is int and event[key] == expected, "candidate byte arithmetic differs: "+key)
        require(type(event.get("changed")) is bool and event["changed"] == (chosen != baseline), "selected changed flag differs")
        result.update(selected_prompt=chosen.decode(), baseline_sha256=digest(baseline), inline_candidate_sha256=digest(alternative),
                      selected_sha256=digest(chosen), eligible=success, selected_candidate=selected,
                      component=component, bundle=bundle, selected_bytes=len(chosen), candidate_byte_delta=len(alternative)-len(baseline))
    except (ValueError, KeyError, TypeError, AttributeError, UnicodeError) as error:
        result["errors"].append(str(error))
    return result


def validate_nonread(event):
    errors = []
    try:
        require(event.get("schema") == SCHEMA and event.get("event") == "prompt", "unsupported v3 non-read event")
        # This pure validator does not inspect schemas or infer a v2 condition.
        # V3 nonfactor spans must equal the frozen baseline; no records are edited.
        errors.extend(pinned("analyze_work_units.py").validate_transform(event, "control"))
        return {"errors": errors, "selected_prompt": event["forwarded_prompt"],
                "selected_sha256": digest(event["forwarded_prompt"].encode()), "eligible": False}
    except (ValueError, KeyError, TypeError, AttributeError) as error:
        return {"errors": errors+[str(error)]}


def site_of(text):
    return "file-read" if isinstance(text, str) and text.startswith(HEADER) else pinned("analyze_work_units.py").site_of(text)


def rpc(value):
    key = value.get("id")
    require(identity(key) or type(key) is int, "unsupported typed RPC identity")
    return type(key).__name__, key


def prompt_inventory(trace, threads, captures, run_id, condition, trace_lines=None):
    errors = []; exposures = []; by_prompt = defaultdict(deque); scopes = defaultdict(set)
    thread_agent = {}; recognized = set(); matched = set(); accepted = set(); capture_ids = set()
    try:
        require(identity(run_id) and condition in {"control", VARIANT}, "invalid v3 run/condition")
        for row in threads:
            require(isinstance(row, dict) and identity(row.get("thread_id")) and identity(row.get("agent_id")), "unknown thread/agent identity")
            thread, agent = row["thread_id"], row["agent_id"]
            require(thread not in thread_agent or thread_agent[thread] == agent, "ambiguous thread ownership")
            thread_agent[thread] = agent
        for capture in captures:
            cap = capture["path"]
            require(identity(cap) and cap not in capture_ids, "invalid or duplicate capture identity")
            capture_ids.add(cap); replies = {}; request_ids = set()
            server_lines = capture.get("server_lines", list(range(1, len(capture["servers"])+1)))
            client_lines = capture.get("client_lines", list(range(1, len(capture["clients"])+1)))
            require(len(server_lines) == len(capture["servers"]) and len(client_lines) == len(capture["clients"]), "physical line inventory differs")
            for line, value in zip(server_lines, capture["servers"]):
                require(isinstance(value, dict), "unsupported server frame")
                if "method" in value or "id" not in value:
                    continue
                key = rpc(value)
                require(key not in replies, "duplicate RPC reply identity")
                replies[key] = value, line
            for line, value in zip(client_lines, capture["clients"]):
                require(isinstance(value, dict), "unsupported client frame")
                if value.get("method") != "turn/start":
                    continue
                key = rpc(value); require(key not in request_ids, "duplicate turn/start RPC identity"); request_ids.add(key)
                params = value.get("params", {}); inputs = params.get("input")
                require(isinstance(inputs, list) and len(inputs) == 1 and isinstance(inputs[0], dict) and inputs[0].get("type") == "text" and isinstance(inputs[0].get("text"), str), "unsupported turn input shape")
                text = inputs[0]["text"]; thread = params.get("threadId")
                require(identity(thread) and thread in thread_agent, "unknown captured request thread")
                request_key = (cap, *key)
                if site_of(text): recognized.add(request_key)
                reply, reply_line = replies.get(key, ({}, None))
                record = {"capture": cap, "client_line": line, "reply_line": reply_line, "rpc_id": value["id"],
                          "thread_id": thread, "agent_id": thread_agent[thread], "text": text,
                          "request_key": request_key, "status": "request_without_terminal_reply"}
                require(not ("result" in reply and "error" in reply), "RPC result and error are mutually exclusive")
                if "error" in reply:
                    rejection = reply["error"]
                    require(isinstance(rejection, dict) and type(rejection.get("code")) is int and identity(rejection.get("message")), "unsupported RPC rejection")
                    record.update(status="rejected_not_delivered", rejection_code=rejection["code"])
                elif "result" in reply:
                    turn = reply["result"].get("turn")
                    require(isinstance(turn, dict) and identity(turn.get("id")), "unknown typed accepted turn")
                    turn_key = thread, turn["id"]
                    require(turn_key not in accepted, "duplicate accepted turn identity")
                    accepted.add(turn_key); record.update(status="delivered", turn_id=turn["id"])
                prompt_key = thread_agent[thread], digest(text.encode())
                by_prompt[prompt_key].append(record); scopes[prompt_key].add((cap, thread))
        require(isinstance(trace, list) and trace and all(isinstance(e, dict) for e in trace), "unknown trace inventory")
        require(trace[0].get("event") == "activation" and sum(e.get("event") == "activation" for e in trace) == 1, "expected one initial activation")
        process_id = trace[0].get("process_id")
        require(integer(process_id) and process_id > 0, "invalid trace process identity")
        trace_lines = trace_lines if trace_lines is not None else list(range(1, len(trace)+1))
        require(len(trace_lines) == len(trace), "trace physical line inventory differs")
        for number, (line, e) in enumerate(zip(trace_lines, trace)):
            require(e.get("schema") == SCHEMA and e.get("run_id") == run_id and e.get("condition") == condition and type(e.get("process_id")) is int and e["process_id"] == process_id, "trace identity differs")
            if number == 0:
                continue
            record = {"sequence": e.get("sequence"), "trace_line": line, "site": e.get("site"),
                      "agent_id": e.get("agent_id"), "status": "invalid_trace", "errors": []}
            exposures.append(record)
            try:
                require(type(e.get("sequence")) is int and e["sequence"] == number and identity(e.get("agent_id")), "unknown/noncontiguous event identity")
                timestamp = e.get("unix_time_ns")
                require(isinstance(timestamp, str) and timestamp.isascii() and timestamp.isdecimal(), "unknown event timestamp")
                result = validate_read(e, condition) if e.get("event") == "read-response" else validate_nonread(e)
                require(not result["errors"], "; ".join(result["errors"]))
                selected = result.pop("selected_prompt"); record.update(result)
                key = e["agent_id"], digest(selected.encode()); candidates = by_prompt[key]
                if not candidates:
                    record["status"] = "prepared_without_captured_request"
                    raise ValueError("prepared trace has no exact captured request; nondelivery not independently proven")
                require(len(scopes[key]) == 1, "identical prompt across captures/threads is ambiguous")
                actual = candidates.popleft()
                require(actual.pop("text") == selected, "prompt hash collision")
                matched.add(actual.pop("request_key")); record.update(actual)
                require(record["status"] != "request_without_terminal_reply", "captured request lacks typed terminal reply")
            except (ValueError, KeyError, TypeError, AttributeError) as error:
                record["errors"].append(str(error)); errors.append(f"trace line {line}: {error}")
        require(recognized == matched, "supported captured requests and trace lack bidirectional coverage")
    except (ValueError, KeyError, TypeError, AttributeError, IndexError) as error:
        errors.append(str(error))
    return {"status": "available" if not errors else "unverifiable", "errors": errors, "exposures": exposures,
            "delivered_eligible_reads": sum(r.get("eligible") is True and r["status"] == "delivered" for r in exposures),
            "delivered_read_count": sum(r.get("site") == "file-read" and r["status"] == "delivered" for r in exposures),
            "recognized_requests": len(recognized), "matched_requests": len(matched), "accepted_provider_turns": len(accepted),
            "native_input_join": "Separate exact thread/item evidence required; accepted turn alone is not native item identity."}


def allocation_choices(rows):
    require(len(rows) == 12 and all(isinstance(r, dict) and identity(r.get("id")) for r in rows) and len({r["id"] for r in rows}) == 12, "exactly twelve unique frozen workflows required")
    blocks = defaultdict(list); observed = set(); wave_owners = defaultdict(set)
    for i, row in enumerate(rows):
        require(row.get("condition") in {"control", VARIANT} and identity(row.get("block_id")) and integer(row.get("wave")) and row["wave"] > 0, "unknown condition/block/wave")
        blocks[row["block_id"]].append(i); wave_owners[row["wave"]].add(row["block_id"])
        if row["condition"] == VARIANT: observed.add(i)
    require(len(blocks) == 2 and len(wave_owners) == 4 and all(len(v) == 1 for v in wave_owners.values()), "exactly two blocks and four distinct waves required")
    choices = []
    for indices in blocks.values():
        waves = defaultdict(set)
        for i in indices: waves[rows[i]["wave"]].add(i)
        require(len(indices) == 6 and len(observed.intersection(indices)) == 3 and len(waves) == 2 and all(len(w) == 3 and 0 < len(w & observed) < 3 for w in waves.values()), "block/wave allocation differs from frozen mixed design")
        allowed = [set(c) for c in combinations(indices, 3) if all(0 < len(set(c) & w) < 3 for w in waves.values())]
        require(len(allowed) == 18, "block lacks eighteen allowed allocations")
        choices.append(allowed)
    return choices, observed


def randomization_test(rows):
    try:
        choices, observed = allocation_choices(rows); bounds = []
        for row in rows:
            b = row["bound"]
            require(isinstance(b, dict) and integer(b.get("lower")) and integer(b.get("upper")) and b["lower"] <= b["upper"], "raw bounds must be finite ordered nonnegative integers")
            bounds.append(b)
        definitely = possibly = 0
        for pair in product(*choices):
            permuted = set.union(*pair); low = high = 0
            for i, b in enumerate(bounds):
                coefficient = int(i in permuted)-int(i in observed)
                low += coefficient*(b["lower"] if coefficient >= 0 else b["upper"])
                high += coefficient*(b["upper"] if coefficient >= 0 else b["lower"])
            definitely += low >= 0; possibly += high >= 0
        lower = sum(b["lower"] if i in observed else -b["upper"] for i, b in enumerate(bounds))
        upper = sum(b["upper"] if i in observed else -b["lower"] for i, b in enumerate(bounds))
        return {"status": "available", "allocations": 324, "definitely_greater_or_equal": definitely,
                "possibly_greater_or_equal": possibly, "p_lower": definitely/324, "p_upper": possibly/324,
                "difference": {"lower": lower/6, "upper": upper/6}, "difference_numerators": {"lower": lower, "upper": upper, "denominator": 6},
                "exact_outcomes": all(b["lower"] == b["upper"] for b in bounds),
                "method": "One-sided whole-workflow raw treatment-minus-control mean. Same-outcome coefficient cancellation; conservative interval p envelope, not necessarily attainable joint extrema.",
                "scope": "Frozen mixed-wave assignment regime; no direct-versus-spillover decomposition."}
    except (ValueError, KeyError, TypeError) as error:
        return {"status": "unavailable", "reason": str(error)}


def passes(result):
    return result.get("status") == "available" and result["difference_numerators"]["lower"] > 0 and result["possibly_greater_or_equal"]*40 <= 324


def measurement_for(frozen, entry, sessions_root):
    out = {"status": "unknown", "errors": [], "measurement": {"status": "ineligible", "bounds": None}}
    try:
        path = HERE/"accounting_untracked_reads.py"; expected = require_helper(frozen, path)
        content = path.read_bytes(); require(digest(content) == expected, "accounting adapter changed before execution")
        adapter = compile_bytes(path, content)
        result = adapter.audit_run(entry, frozen, Path(sessions_root))
        require(isinstance(result, dict) and result.get("run_id") == entry["run_id"] and isinstance(result.get("errors"), list), "unsupported accounting adapter report")
        out = result
        require(result.get("status") == "validated" and not result["errors"], "independent future-only accounting did not validate")
        sources = result.get("source_sha256")
        require(isinstance(sources, dict) and sources.get(str(path.resolve())) == expected, "accounting adapter source provenance missing")
        for source, expected_hash in sources.items():
            require(sha(source) == expected_hash, "accounting evidence source differs")
        require(sha(path) == expected, "accounting adapter changed during replay")
        evidence = result.get("exact_response_evidence")
        require(isinstance(evidence, dict), "exact response inventory missing")
        for rid, row in evidence.items():
            require(identity(rid) and isinstance(row, dict) and identity(row.get("thread_id")) and identity(row.get("turn_id")), "invalid response/thread/turn identity")
        status = result["measurement"].get("status")
        require(status in {"exact", "bounded", "unbounded_accounting_gap"}, "unsupported validated accounting status")
        metric = result["measurement"]["bounds"]["raw_input_plus_output"]
        require(integer(metric.get("lower")), "unknown raw lower accounting bound")
        if status == "unbounded_accounting_gap":
            require(metric.get("upper") is None, "unbounded status conflicts with raw upper bound")
        else:
            require(integer(metric.get("upper")) and metric["lower"] <= metric["upper"], "finite accounting interval unavailable")
    except (ValueError, KeyError, TypeError, AttributeError, OSError) as error:
        out["status"] = "unknown"; out.setdefault("errors", []).append(str(error))
    return out


def primary_for(rows, integrity):
    try:
        require(not integrity, "phase/source/configuration integrity unavailable")
        inference = []; owners = {}
        for row in rows:
            require(not row.get("errors"), "workflow integrity unavailable: "+str(row.get("id")))
            accounting = row["accounting"]
            require(accounting.get("status") == "validated" and not accounting.get("errors"), "workflow accounting unavailable: "+str(row.get("id")))
            for response_id in accounting["exact_response_evidence"]:
                require(response_id not in owners or owners[response_id] == row["id"], "response identity belongs to multiple workflows")
                owners[response_id] = row["id"]
            inference.append({**{k: row.get(k) for k in ("id", "condition", "block_id", "wave")},
                              "bound": accounting["measurement"]["bounds"]["raw_input_plus_output"]})
        return randomization_test(inference)
    except (ValueError, KeyError, TypeError) as error:
        return {"status": "unavailable", "reason": str(error)}


def capture_provenance(app):
    # Guard every exact input before delegating to the immutable pure replay.
    invocation = app.parent.parent/"invocations"/app.name
    start = decode(regular(invocation/"start.json").read_bytes())
    regular(invocation/"end.json")
    names = ["client-to-server.raw", "server-to-client.raw", "raw-response-usage.json",
             "client-to-server.forwarded.raw", "raw-response-rewrites.jsonl"]
    grace = start.get("provider_usage_grace_ms")
    if type(grace) is int and grace > 0:
        names.append("provider-usage-grace.jsonl")
    for name in names: regular(app/name)
    return pinned("analyze.py").capture_provenance(app)


def analyze_manifest(path, sessions_root):
    """Only a closed complete phase can reach the single preregistered endpoint."""
    sources = {}; integrity = []; observations = []; trust = {"valid": False, "errors": ["not_replayed"]}
    manifest = {}; frozen = {}; phase_result = {}

    def remember(value):
        candidate = Path(value); candidate = candidate if candidate.is_absolute() else ROOT/candidate
        p = regular(candidate); actual = sha(p)
        require(str(p) not in sources or sources[str(p)] == actual, "source changed during analysis: "+str(p))
        sources[str(p)] = actual
        return p

    def read(value):
        return decode(remember(value).read_bytes())

    try:
        manifest = read(path); frozen = read(manifest["phase_manifest"])
        require(isinstance(manifest, dict) and isinstance(frozen, dict), "unsupported manifest object")
    except (ValueError, KeyError, TypeError, OSError) as error:
        integrity.append(str(error))
    try:
        core = pinned("analyze_work_units.py")
        phase = remember(manifest["phase_manifest"]).parent
        phase_hash = sources[str(regular(manifest["phase_manifest"]))]
        require(phase_hash == manifest.get("phase_manifest_sha256"), "frozen phase hash differs")
        require(read(phase/"RUN-ONCE/admission.json").get("manifest_sha256") == phase_hash, "phase was not admitted under this exact identity")
        file_paths = set()
        for row in frozen["files"]:
            require(isinstance(row, dict) and identity(row.get("path")) and row["path"] not in file_paths, "duplicate/invalid frozen source path")
            file_paths.add(row["path"])
            require(sources[str(remember(row["path"]))] == row.get("sha256"), "frozen source inventory differs")
        for helper in [Path(__file__).resolve(), *(HERE/name for name in PINS), HERE/"trust_work_units.py", HERE/"accounting_untracked_reads.py"]:
            actual = require_helper(frozen, helper); remember(helper)
            if helper.name in PINS:
                require(actual == PINS[helper.name], "executed pure helper source differs from its fixed pin")
        base = pinned("analyze.py"); strict_path = base.GATE/"batch_analysis.py"
        require_helper(frozen, strict_path); remember(strict_path)
        phase_result = read(phase/"PHASE-RESULT.json")
        # Pure canonical settings/terminal-incident check: no v2 trace or endpoint.
        integrity.extend(core.phase_errors(frozen, manifest, phase_result))
        for key in ("study", "phase", "phase_kind", "base_commit", "task_list_sha256", "source_commit", "model", "reasoning_effort", "randomization"):
            require(frozen.get(key) == manifest.get(key), "score/frozen manifest differ: "+key)
        random = frozen["randomization"]
        require(random.get("scheme") == "within_block" and random.get("unit") == "workflow" and random.get("mixed_waves") is True, "unknown allocation method")
        counts = defaultdict(Counter)
        for row in frozen["schedule"]: counts[row["block_id"]][row["condition"]] += 1
        require(random.get("block_condition_counts") == {k: dict(v) for k, v in counts.items()}, "allocation metadata block counts differ")
        allocation_choices([{**r, "id": r["run_id"]} for r in frozen["schedule"]])
        launch = frozen["launch_order"]
        by_id = {r["run_id"]: r for r in frozen["schedule"]}
        require(isinstance(launch, list) and len(launch) == 4 and all(isinstance(w, list) and len(w) == 3 for w in launch), "frozen launch waves differ")
        launched_ids = [rid for wave in launch for rid in wave]
        require(all(identity(rid) for rid in launched_ids) and len(set(launched_ids)) == 12 and set(launched_ids) == set(by_id), "launch order does not cover exact frozen identities")
        for wave in launch:
            require(len({by_id[r]["wave"] for r in wave}) == 1 and len({by_id[r]["condition"] for r in wave}) == 2, "launch wave differs from frozen mixed grouping")
        config = read(phase/"config-history.json")
        require(isinstance(config.get("snapshots"), list) and config["snapshots"], "original configuration history missing")
        trust = core.audit_trust(frozen, phase, helper_path=HERE/"trust_work_units.py")
        require(trust.get("valid") is True and not trust.get("errors") and trust.get("verified_snapshot_indices") == list(range(len(config["snapshots"]))), "configuration history did not independently replay")
        for name in ("config-history.jsonl", "trust-final.json", "trust-evidence.jsonl", "trust-pending.jsonl"):
            remember(phase/name)
    except (ValueError, KeyError, TypeError, AttributeError, OSError) as error:
        integrity.append(str(error))
    schedule = frozen.get("schedule", [])
    if not isinstance(schedule, list):
        integrity.append("unsupported frozen schedule"); schedule = []
    entries = {}; unexpected = []
    actual_rows = manifest.get("runs", [])
    if not isinstance(actual_rows, list):
        integrity.append("unsupported result row inventory"); actual_rows = []
    for row in actual_rows:
        rid = row.get("run_id", row.get("id")) if isinstance(row, dict) else None
        if not identity(rid) or rid in entries:
            integrity.append("invalid or duplicate result identity"); unexpected.append(row)
        else: entries[rid] = row
    declared_ids = [r.get("run_id") if isinstance(r, dict) else None for r in schedule]
    valid_ids = {rid for rid in declared_ids if identity(rid)}
    if len(valid_ids) != len(schedule) or set(entries) != valid_ids:
        integrity.append("result identities differ from complete unique frozen schedule")
    unexpected.extend(row for rid, row in entries.items() if rid not in valid_ids)
    for declaration in schedule:
        declaration = declaration if isinstance(declaration, dict) else {}
        rid = declaration.get("run_id"); entry = entries.get(rid, {}) if identity(rid) else {}
        out = {"id": rid, **{k: declaration.get(k) for k in ("condition", "block_id", "wave")},
               **{k: entry.get(k) for k in ("launch_status", "launcher_exit_code", "started_at", "finished_at")}, "errors": []}
        observations.append(out)
        try:
            for key in ("condition", "phase", "block_id", "wave", "workflow", "artifact", "report", "prompt_trace", "experiment_manifest"):
                require(key in entry and entry[key] == declaration.get(key), "result differs from frozen row: "+key)
            require(identity(entry.get("started_at")) and identity(entry.get("finished_at")), "workflow lacks admitted terminal outcome")
            expected = {"schema": SCHEMA, "run_id": rid, "condition": entry["condition"], "evidence_path": entry["prompt_trace"]}
            require(read(entry["experiment_manifest"]) == expected, "v3 experiment manifest differs from frozen condition")
            artifact = Path(entry["artifact"]); artifact = artifact if artifact.is_absolute() else ROOT/artifact
            observed = read(artifact/"observation/analysis.json"); report = read(entry["report"])
            out.update(workflow_result=report.get("workflow_result", report.get("result", "unknown")), original_observer_errors=observed.get("errors"))
            captures = []
            for app in sorted((artifact/"observation/app-server").iterdir()):
                if not app.is_dir(): continue
                proof = capture_provenance(app)
                require(not proof["errors"], "; ".join(proof["errors"]))
                for source, expected_hash in proof["source_sha256"].items():
                    require(sources[str(remember(source))] == expected_hash, "capture changed since source replay")
                clients, cl = records(remember(app/"client-to-server.raw")); servers, sl = records(remember(app/"server-to-client.raw"))
                captures.append({"path": str(app), "clients": clients, "servers": servers, "client_lines": cl, "server_lines": sl})
            require(bool(captures), "no actual capture inventory")
            trace, lines = records(remember(entry["prompt_trace"]))
            out["delivery"] = prompt_inventory(trace, observed["threads"], captures, rid, entry["condition"], lines)
            out["errors"].extend(out["delivery"]["errors"])
            out["accounting"] = measurement_for(frozen, entry, sessions_root)
            out["errors"].extend(out["accounting"].get("errors", []))
            for source, expected_hash in out["accounting"].get("source_sha256", {}).items():
                require(sources[str(remember(source))] == expected_hash, "accounting source changed since independent replay")
        except (ValueError, KeyError, TypeError, AttributeError, OSError) as error:
            out["errors"].append(str(error))
    for entry in unexpected:
        value = entry if isinstance(entry, dict) else {}
        observations.append({"id": value.get("run_id", value.get("id")), "retained_result_row": entry,
                             "errors": ["Unexpected or duplicate result retained outside inference"]})
    for source, expected_hash in sources.items():
        try: require(sha(source) == expected_hash, "source changed at analysis endpoint: "+source)
        except (ValueError, OSError) as error: integrity.append(str(error))
    primary = primary_for(observations, integrity)
    return {"schema": "work-leaf-untracked-read-primary-v1", "phase": manifest.get("phase"),
            "observations": observations, "primary": primary, "primary_passes": passes(primary),
            "alpha": ALPHA, "endpoint": "whole-workflow raw_input_plus_output", "integrity_errors": integrity,
            "source_sha256": sources, "trust_replay": trust,
            "legacy_behavioral_config_drift_detected": phase_result.get("behavioral_config_drift_detected"),
            "unexplained_config_drift_detected": phase_result.get("unexplained_config_drift_detected"),
            "pending_config_attestation_at_finish": phase_result.get("pending_config_attestation_at_finish"),
            "secondary_test": None, "sampling_confidence_interval": False, "causal_share": None,
            "scope": "All twelve fixed whole workflows, including failures; source representation factor under mixed-wave assignment. Candidate bytes are not tokens. Native-input mechanism evidence remains separately reviewed."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--sessions-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    require(not args.output.exists(), "output exists; reports are create-new")
    result = analyze_manifest(args.manifest, args.sessions_root)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2, sort_keys=True, allow_nan=False); stream.write("\n")
    print(args.output)


if __name__ == "__main__":
    try: main()
    except (ValueError, KeyError, TypeError, OSError) as error:
        print(f"untracked-read analysis error: {error}", file=sys.stderr); sys.exit(2)
