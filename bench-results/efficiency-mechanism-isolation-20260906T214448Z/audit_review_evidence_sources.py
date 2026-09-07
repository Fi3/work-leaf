#!/usr/bin/env python3
"""Source-bound opaque-review delivery/retrieval census, not a token estimator.

One closed workflow, all issued review archives and all captured/native threads.
Public and native item namespaces remain distinct. Archive bytes are never exported.
Unsupported/indirect native access is unresolved, never proof of no retrieval.
Joins are indexed; work is linear in source and emitted evidence bytes, with sorted
per-thread unknown-line lookup. The pinned primitive can scan B archive bytes for
each of R actual read calls: O(R*B), potentially quadratic as both grow, even for
short partial outputs. This is explicitly flagged; no archive-by-native-stream
cross product is used. Endpoint hashes do not prove hostile-writer containment.
"""
import argparse
from bisect import bisect_right
from collections import defaultdict
from datetime import datetime
from functools import lru_cache
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys
import types

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
SCHEMA = "work-leaf-bench-experiment-v5"
INPUT_SCHEMA = "work-leaf-review-evidence-input-v1"
PRIMITIVE_PIN = "34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257"
RUNTIME_PINS = {
    "src/cli.rs": "a87be2707fcb873d1cdfba319d9e6d1b07ab6aa7aacdf8e5aa6ff25f98ab1025",
    "src/agent.rs": "a9e6065a450d05bf299202a2d6f44dd2dae33a3324e8c70e9f91d36a22b242fd",
    "src/review.rs": "12d70548086d053583e1fc2fc6b4f191dd262cf61b45637a1ee5d04931c919ee",
    "src/bench_experiment.rs": "d23fe2475f645fb6d91db29d54048be87125f3e1ec17cee04dee393aed33e7e6",
    "src/bench_review_evidence.rs": "06101576abc6166d57497d8806b48357e38c0611b6d8d46f15a5e6285078b725",
    "src/chat_title.rs": "ba67ff00c43231791f42ffa964c94162e415b2e4b4ee4c71ac1c29ac3d81057f",
    "src/orchestrator.rs": "027c35a9eaf24e99335001e68f7d0612a262831af80f8677874525db95e9332b",
    "src/codex.rs": "334eb3008f3b0a8db0d98bed2d3191f5cbf37052fb342dcf26a91ccd4ee502d8",
}
MAX_FILE = 1024**3
MAX_TOTAL = 32 * 1024**3
MAX_SOURCES = 20000
REVIEW_SUFFIX = (
    "\n\nReview every commit listed in the review scope and reply with NO_FINDINGS if there are no findings. Otherwise reply with FINDINGS followed by the issues."
    "\n\nDocumentation and plain-text updates are deferred to the linearize agent. Do not treat missing docs, README, changelog, markdown, txt, or other prose-only updates as findings against this patch agent; review the code and behavior that the patch agent changed."
    "\n\nFor agent-facing changes, missing required real-agent verification is a finding unless the source context includes the exact real-agent scenario and visible result, or the exact pre-agent blocker. If you report missing verification, state the precise evidence that would resolve it. When the patch agent responds with verification evidence or a blocker rather than code, evaluate that evidence instead of requiring another patch."
)


def require(value, message):
    if not value:
        raise ValueError(message)


def identity(value):
    return isinstance(value, str) and bool(value)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def strict_equal(left, right):
    return json.dumps(left, sort_keys=True, allow_nan=False) == json.dumps(right, sort_keys=True, allow_nan=False)


def decode(data):
    def pairs(values):
        result = {}
        for key, value in values:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result
    def invalid(_):
        raise ValueError("nonfinite JSON value")
    return json.loads(data, object_pairs_hook=pairs, parse_constant=invalid)


class Sources:
    """Exact source identities before parsing, no symlink/hardlink path aliases."""
    def __init__(self):
        self.hashes = {}; self.inodes = {}; self.total = 0

    def read(self, ref):
        require(isinstance(ref, dict) and set(ref) == {"path", "sha256"}
                and identity(ref["path"]) and isinstance(ref["sha256"], str)
                and re.fullmatch("[0-9a-f]{64}", ref["sha256"]), "invalid expected source identity")
        path = Path(ref["path"])
        require(path.is_absolute() and str(path) == ref["path"] and path.resolve() == path,
                "source path must be exact absolute and nonsymlinked")
        before = path.lstat(); key = (before.st_dev, before.st_ino)
        require(stat.S_ISREG(before.st_mode) and before.st_size <= MAX_FILE, "source type/size limit")
        require(key not in self.inodes or self.inodes[key] == str(path), "source hardlink alias")
        known = str(path) in self.hashes
        require(known or (len(self.hashes) < MAX_SOURCES and self.total + before.st_size <= MAX_TOTAL), "source census budget exceeded; no clipping")
        descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(descriptor, "rb") as stream:
            opened = os.fstat(stream.fileno()); data = stream.read(before.st_size + 1); after = os.fstat(stream.fileno())
        signature = lambda value: (value.st_dev, value.st_ino, value.st_mode, value.st_size, value.st_mtime_ns, value.st_ctime_ns)
        require(signature(before) == signature(opened) == signature(after) and len(data) == before.st_size,
                "source changed during read")
        require(sha(data) == ref["sha256"] and (not known or self.hashes[str(path)] == ref["sha256"]), "source SHA-256 differs")
        if not known: self.total += len(data)
        self.hashes[str(path)] = ref["sha256"]; self.inodes[key] = str(path)
        return data

    def json(self, ref):
        value = decode(self.read(ref)); require(isinstance(value, dict), "source JSON must be an object")
        return value

    def records(self, ref):
        data = self.read(ref); require(not data or data.endswith(b"\n"), "incomplete physical JSONL tail")
        result = []
        for number, raw in enumerate(data.split(b"\n"), 1):
            if not raw.strip(): continue
            row = decode(raw); require(isinstance(row, dict), "nonobject JSONL record")
            result.append((number, row))
        return result

    def finish(self):
        for path, expected in tuple(self.hashes.items()): self.read({"path": path, "sha256": expected})


@lru_cache(maxsize=1)
def primitives():
    path = HERE / "audit_review_evidence.py"
    data = Sources().read({"path": str(path), "sha256": PRIMITIVE_PIN})
    module = types.ModuleType("review_evidence_exact_primitives"); module.__file__ = str(path)
    exec(compile(data, str(path), "exec"), module.__dict__)
    return module


def receipt(archive):
    return ("Work Leaf opaque review evidence (temporary read-only context):\n"
            f"Path: {json.dumps(archive['path'], ensure_ascii=False)}\nBytes: {archive['bytes']}\nEvidence identity: {archive['digest']}\n"
            "This artifact contains the same complete commit/log/recorded-chat evidence for this review. "
            "Native read-only inspection of this exact supplied path is permitted; it is not served by `@work-leaf read`. "
            "Consult relevant archived evidence before declaring required evidence missing.\n")


def validate_event(event, payload, manifest, run, condition, root):
    primitive = primitives(); archive = event.get("archive", {})
    require(manifest.get("schema") == "work-leaf-review-evidence-v1"
            and manifest.get("project_snapshots_applicable") is False
            and type(manifest.get("context_start")) is int and type(manifest.get("context_end")) is int
            and manifest["context_start"] == event.get("context_start") and manifest["context_end"] == event.get("context_end"),
            "typed manifest scope/context bounds differ")
    require(strict_equal(manifest.get("archive"), archive), "typed manifest archive differs")
    verified = primitive.validate_snapshot(event, payload, manifest["archive"], run, condition, root)
    sequence = archive["sequence"]
    require(archive["path"] == str(Path(root) / f"review-{sequence:016d}.txt")
            and archive.get("manifest_path") == str(Path(root) / f"review-{sequence:016d}.json"), "archive renderer path differs")
    original = event["original_prompt"].encode(); candidate = event["candidate_prompt"].encode(); forwarded = event["forwarded_prompt"].encode()
    start, end = event["context_start"], event["context_end"]
    require(event["candidate_start"] == start and candidate[start:event["candidate_end"]] == receipt(archive).encode(),
            "candidate receipt is not the pinned renderer receipt")
    require(original[:start].startswith(f"Review the full patch scope for Agent-ID {event['source_agent_id']}.\nLatest commit: {event['target_commit']}\nFeature: ".encode())
            and original[:start].endswith(b"\n\nSource context from Work Leaf commits, logs, and chat history:\n")
            and original[end:] == REVIEW_SUFFIX.encode(), "review interpolation is not the source-owned renderer boundary")
    for field, value in (("original_bytes", len(original)), ("candidate_bytes", len(candidate)),
                         ("forwarded_bytes", len(forwarded)), ("byte_delta", len(forwarded) - len(original))):
        require(type(event.get(field)) is int and event[field] == value, "prompt byte field differs: " + field)
    require(type(event.get("changed")) is bool and event["changed"] == (forwarded != original), "changed field differs")
    return {**verified, "original_sha256": sha(original), "candidate_sha256": sha(candidate), "forwarded_sha256": sha(forwarded),
            "context_start": start, "context_end": end, "candidate_start": start, "candidate_end": event["candidate_end"]}


def terminal(value, run, condition):
    require(value.get("run_id") == run and value.get("condition") == condition, "terminal run/condition differs")
    if value.get("schema") == "work-leaf-review-evidence-terminal-v1":
        start, end, code = value.get("started_at"), value.get("finished_at"), value.get("exit_code")
    else:
        require(value.get("id") == run and value.get("launch_status") == "completed", "workflow is not terminally published")
        start, end, code = value.get("started_at"), value.get("finished_at"), value.get("launcher_exit_code")
    before, after = datetime.fromisoformat(start), datetime.fromisoformat(end)
    require(before.tzinfo is not None and after.tzinfo is not None and after >= before and type(code) is int,
            "invalid terminal timing/exit")
    return {"started_at": start, "finished_at": end, "exit_code": code}


def collect_captures(manifest, sources):
    invocations = sources.records(manifest["invocations"]); expected = {}
    observation = Path(manifest["invocations"]["path"]).parent
    require(Path(manifest["invocations"]["path"]).name == "process-invocations.jsonl", "unknown invocation inventory path")
    for _, row in invocations:
        if row.get("capture_kind") != "app-server": continue
        key = row.get("invocation_id")
        require(identity(key) and key not in expected, "duplicate/invalid app-server inventory")
        expected[key] = row
    result = []; seen = set()
    for cap in manifest["captures"]:
        path = Path(cap["path"]); key = path.name
        require(path == observation / "app-server" / key and key in expected and key not in seen, "capture inventory/path differs")
        seen.add(key)
        required = {"clients": path / "client-to-server.raw", "forwarded": path / "client-to-server.forwarded.raw", "servers": path / "server-to-client.raw",
                    "start": observation / "invocations" / key / "start.json", "end": observation / "invocations" / key / "end.json"}
        for name, target in required.items(): require(cap[name]["path"] == str(target), "capture source is not canonical")
        start, end = sources.json(cap["start"]), sources.json(cap["end"]); logged = expected[key]
        require(start.get("invocation_id") == end.get("invocation_id") == key and start.get("capture_kind") == "app-server" and start.get("primary") is True,
                "capture is not primary app-server")
        require(type(start.get("start_unix_ns")) is int and type(end.get("end_unix_ns")) is int
                and end["end_unix_ns"] >= start["start_unix_ns"]
                and (type(end.get("exit_code")) is int or type(end.get("terminating_signal")) is int), "capture lacks exact terminal metadata")
        for name in ("invocation_id", "capture_kind", "primary", "start_unix_ns"):
            require(strict_equal(logged.get(name), start.get(name)), "invocation start inventory differs")
        for name in ("invocation_id", "end_unix_ns", "exit_code", "terminating_signal", "stdin_sha256", "stdout_sha256"):
            require(strict_equal(logged.get("end", {}).get(name), end.get(name)), "invocation end inventory differs")
        require(end.get("raw_response_usage_start_sha256") == cap["start"]["sha256"] and end.get("stdin_sha256") == cap["clients"]["sha256"]
                and end.get("stdout_sha256") == cap["servers"]["sha256"]
                and end.get("raw_response_usage_sha256", {}).get("client-to-server.forwarded.raw") == cap["forwarded"]["sha256"], "terminal capture digests differ")
        rows = {name: sources.records(cap[name]) for name in ("clients", "forwarded", "servers")}
        for _, frame in rows["servers"]:
            item = frame.get("params", {}).get("item", {})
            if frame.get("method") == "item/completed" and item.get("type") == "userMessage":
                for content in item.get("content", []):
                    require(isinstance(content, dict) and content.get("text_elements", []) == [], "nonempty public user text metadata")
        result.append({"path": str(path), **{name: [value for _, value in values] for name, values in rows.items()},
                       "client_lines": [line for line, _ in rows["clients"]], "server_lines": [line for line, _ in rows["servers"]]})
    require(seen and seen == set(expected), "capture inventory incomplete")
    return result


def validate_build(value, runtime):
    require(value.get("schema") == "work-leaf-review-evidence-build-attestation-v5"
            and type(value.get("runtime_build_exit")) is int and value["runtime_build_exit"] == 0
            and value.get("runtime_build") == "cargo build --release --features bench-experiments --bins"
            and value.get("observer_reused_unchanged") is True
            and type(value.get("smoke_build_exit")) is int and value["smoke_build_exit"] == 0
            and identity(value.get("smoke_build")) and Path(value.get("source_repo", "")).is_absolute(),
            "unsupported/unsuccessful build attestation")
    inventory = {}
    for row in value["source_files"]:
        name = row.get("path"); digest = row.get("sha256")
        require(identity(name) and not Path(name).is_absolute() and ".." not in Path(name).parts and name not in inventory
                and isinstance(digest, str) and re.fullmatch("[0-9a-f]{64}", digest), "invalid build source inventory")
        inventory[name] = digest
    for name, ref in runtime.items():
        require(inventory.get(name) == ref["sha256"], "attested runtime source differs: " + name)
    binaries = value.get("binaries")
    require(isinstance(binaries, list) and binaries, "missing attested binaries")
    seen = set()
    for row in binaries + [value.get("smoke_test_binary")]:
        require(isinstance(row, dict) and identity(row.get("path")) and row["path"] not in seen
                and isinstance(row.get("sha256"), str) and re.fullmatch("[0-9a-f]{64}", row["sha256"]), "invalid attested binary identity")
        seen.add(row["path"])
    project = lambda row: {"path": row["path"], "sha256": row["sha256"]}
    return {"schema": value["schema"], "source_repo": value["source_repo"], "binaries": [project(row) for row in binaries],
            "smoke_test_binary": project(value["smoke_test_binary"]),
            "scope": "Recorded successful source/build attestation; actual admitted executable identity is separately checked by the operator."}


def tool_turn(payload, current):
    metadata = payload.get("internal_chat_message_metadata_passthrough")
    require(metadata is None or isinstance(metadata, dict), "malformed tool passthrough metadata")
    explicit = payload.get("turn_id"); nested = (metadata or {}).get("turn_id")
    for field in (explicit, nested): require(field is None or identity(field), "malformed explicit tool turn")
    require(not (explicit and nested and explicit != nested), "contradictory explicit tool turns")
    selected = explicit or nested
    require(not (selected and current and selected != current), "tool turn differs from current native turn context")
    require(identity(selected or current), "native tool lacks same-turn evidence")
    return selected or current, "explicit" if selected else "inherited-turn-context"


def native_tools(native_sources, accepted):
    """Single pass, safe projections only. Unknown action bodies are never exported."""
    calls = []; outputs = {}; unknown = defaultdict(list); errors = []; known = {}; conflicts = set()
    accepted_turns = {(row["thread_id"], row["turn_id"]) for row in accepted}
    for source in native_sources:
        thread = source["thread_id"]; current = None; item_ids = {}
        for line, row in source["rows"]:
            payload = row.get("payload")
            if row.get("type") == "turn_context":
                current = payload.get("turn_id") if isinstance(payload, dict) else None
                if not identity(current): errors.append("malformed native turn context"); current = None
            if row.get("type") != "response_item": continue
            if not isinstance(payload, dict):
                unknown[thread].append({"line": line, "reason": "nonobject native item"}); continue
            kind = payload.get("type")
            if kind in {"message", "reasoning"}: continue
            safe = {"source": source["source"], "thread_id": thread, "line": line,
                    "kind": kind if isinstance(kind, str) else None}
            if kind not in {"function_call", "function_call_output", "custom_tool_call", "custom_tool_call_output"}:
                unknown[thread].append({**safe, "reason": "unsupported native activity"}); continue
            try:
                turn, basis = tool_turn(payload, current)
                require((thread, turn) in accepted_turns, "tool turn has no accepted public/native input")
                call_id = payload.get("call_id"); require(identity(call_id), "native call/output lacks typed call ID")
                item_id = payload.get("id")
                require(item_id is None or identity(item_id), "malformed native item ID")
                if item_id:
                    encoded = json.dumps(payload, sort_keys=True, allow_nan=False).encode(); fingerprint = sha(encoded)
                    if item_id in item_ids and item_ids[item_id][0] != fingerprint:
                        conflicts.add((thread, item_ids[item_id][1]))
                        raise ValueError("conflicting native item ID")
                    if item_id in item_ids: continue
                    item_ids[item_id] = fingerprint, call_id
                safe.update(turn_id=turn, turn_basis=basis, call_id=call_id, item_id=item_id)
                key = thread, call_id
                if kind.endswith("_output"):
                    require(key not in outputs, "duplicate native call output")
                    outputs[key] = safe, payload.get("output")
                else:
                    require(key not in known, "duplicate native call identity")
                    known[key] = len(calls)
                    calls.append((safe, payload.get("name"), payload.get("arguments", payload.get("input"))))
            except (ValueError, TypeError) as error:
                if identity(payload.get("call_id")):
                    conflicts.add((thread, payload["call_id"]))
                unknown[thread].append({**safe, "reason": str(error)})
    result = []
    for safe, name, arguments in calls:
        key = safe["thread_id"], safe["call_id"]
        output = outputs.pop(key, None)
        row = {**safe, "call_line": safe["line"], "name": name if isinstance(name, str) else None,
               "status": "unresolved", "argument_sha256": sha(arguments.encode()) if isinstance(arguments, str) else None}
        command = None; text = None
        try:
            require(key not in conflicts, "native call/output identity is conflicting or malformed")
            require(safe["kind"] == "function_call" and name in {"exec_command", "functions.exec_command"}
                    and isinstance(arguments, str), "unsupported native tool/program or arguments")
            args = decode(arguments); require(isinstance(args, dict) and isinstance(args.get("cmd"), str), "unsupported execution arguments")
            command = args["cmd"]
            require(output is not None, "native call output missing")
            out, text = output
            require(out["kind"] == "function_call_output" and out["turn_id"] == safe["turn_id"]
                    and out["line"] > safe["line"] and isinstance(text, str), "output is not supported later same-turn text")
            row.update(output_line=out["line"], output_item_id=out["item_id"], output_sha256=sha(text.encode()), output_bytes=len(text.encode()),
                       output_turn_basis=out["turn_basis"])
        except (ValueError, TypeError) as error:
            row["reason"] = str(error)
        result.append((row, command, text))
    for (thread, _), (safe, _) in outputs.items():
        unknown[thread].append({**safe, "reason": "native output without one captured call"})
    return result, unknown, errors


def derive_retrieval(reviews, events, payloads, deliveries, native_sources, accepted):
    calls, unknown, errors = native_tools(native_sources, accepted)
    issued = {event["archive"]["path"]: index for index, event in events.items() if index in deliveries and reviews[index]["status"] == "verified"}
    primitive = primitives(); projected = []; matches = defaultdict(list)
    for row, command, output in calls:
        index = None
        try:
            require(command is not None, row.get("reason", "unsupported native command"))
            words = primitive.command_words(command)
            index = issued.get(words[-1]) if words else None
            require(index is not None, "no exact issued-path operand; indirect/other access unresolved")
            delivery = deliveries[index]
            require(row["thread_id"] == delivery["thread_id"] and row["source"] == delivery["native_source"]
                    and row["call_line"] > delivery["native_line"], "issued path not read after delivery by its reviewer")
            require(output is not None and "output_line" in row, row.get("reason", "native output missing"))
            read = primitive.classify_read(command, output, words[-1], payloads[index])
            row.update(read, review_index=index, issued_path=words[-1])
            matches[index].append(len(projected))
        except (ValueError, TypeError, IndexError) as error:
            row["reason"] = str(error)
        if row["status"] == "unresolved":
            unknown[row["thread_id"]].append({"source": row["source"], "line": row["call_line"], "call_index": len(projected), "reason": row.get("reason", "unresolved native read")})
        projected.append(row)
    groups = []; group_indices = {}; lines = {}
    for thread, records in unknown.items():
        records.sort(key=lambda row: row["line"])
        group_indices[thread] = len(groups); groups.append({"thread_id": thread, "items": records})
        lines[thread] = [row["line"] for row in records]
    for index, review in enumerate(reviews):
        if review["status"] != "verified" or index not in deliveries:
            review["retrieval"] = {"status": "unresolved", "reason": "source/delivery verification unavailable"}; continue
        delivery = deliveries[index]; thread = delivery["thread_id"]
        start = bisect_right(lines.get(thread, []), delivery["native_line"])
        unresolved = start < len(lines.get(thread, []))
        refs = matches[index]
        status = "complete" if any(projected[ref]["status"] == "complete" for ref in refs) else (
            "verified_partial" if any(projected[ref]["status"] == "verified_partial" for ref in refs)
            else "unresolved" if unresolved else "absent")
        review["retrieval"] = {"status": status, "call_indices": refs,
                               "unknown_group": group_indices.get(thread), "unknown_items_from": start,
                               "additional_access_unresolved": unresolved,
                               "absence_scope": "No supported read and no unresolved later native activity in the complete supplied reviewer session; not proof of OS-level nonaccess."}
    return projected, groups, errors


def archive_listing(root):
    names = set()
    with os.scandir(root) as entries:
        for entry in entries:
            require(len(names) < MAX_SOURCES, "archive directory entry budget exceeded")
            require(entry.is_file(follow_symlinks=False), "archive directory contains a nonregular entry")
            names.add(entry.path)
    return names


def public_activity(captures, accepted, calls, reviews):
    """An input join alone cannot establish native tool census completeness."""
    positions = {(row["thread_id"], row["turn_id"]): index for index, row in enumerate(accepted)}
    native = defaultdict(set)
    for index, row in enumerate(calls):
        if row["kind"] != "function_call": continue
        for value in (row.get("call_id"), row.get("item_id")):
            if identity(value): native[row["thread_id"], row["turn_id"], value].add(index)
    items = {}; errors = []
    for cap in captures:
        for physical, frame in zip(cap["server_lines"], cap["servers"]):
            if frame.get("method") not in {"item/started", "item/completed"}: continue
            params = frame.get("params", {}); item = params.get("item", {})
            if item.get("type") in {"userMessage", "agentMessage", "reasoning"}: continue
            thread, turn, item_id = params.get("threadId"), params.get("turnId"), item.get("id")
            if not all(identity(value) for value in (thread, turn, item_id)) or (thread, turn) not in positions:
                errors.append("public native activity lacks accepted typed turn/item ownership"); continue
            key = thread, turn, item_id
            row = items.setdefault(key, {"capture": cap["path"], "thread_id": thread, "turn_id": turn, "public_item_id": item_id,
                    "kind": item.get("type") if isinstance(item.get("type"), str) else None,
                    "accepted_position": positions[thread, turn], "first_public_line": physical, "completed_public_line": None})
            if frame["method"] == "item/completed": row["completed_public_line"] = physical
    projected = []; unknown = defaultdict(list)
    for key, row in items.items():
        choices = native.get(key, set()) if row["kind"] == "commandExecution" else set()
        row["native_call_index"] = next(iter(choices)) if len(choices) == 1 else None
        row["status"] = "exact_typed_identity" if row["native_call_index"] is not None else "unresolved_native_coverage"
        row["scope"] = "Public/native identity witness only; command/output byte semantics are checked separately by the archive read classifier."
        if row["status"] != "exact_typed_identity": unknown[row["thread_id"]].append(len(projected))
        projected.append(row)
    groups = []; group_of = {}; ordered_positions = {}
    for thread, indices in unknown.items():
        indices.sort(key=lambda index: projected[index]["accepted_position"])
        group_of[thread] = len(groups); groups.append({"thread_id": thread, "public_activity_indices": indices})
        ordered_positions[thread] = [projected[index]["accepted_position"] for index in indices]
    for review in reviews:
        delivery = review.get("delivery")
        if not delivery: continue
        thread = delivery["thread_id"]; position = positions[thread, delivery["turn_id"]]
        start = bisect_right(ordered_positions.get(thread, []), position - 1)
        gap = bool(errors) or start < len(ordered_positions.get(thread, []))
        review["retrieval"].update(unknown_public_group=group_of.get(thread), unknown_public_items_from=start,
                                    public_native_coverage_unresolved=gap)
        if gap and review["retrieval"]["status"] == "absent":
            review["retrieval"].update(status="unresolved", reason="public native activity is not completely represented by exact native call identities")
    return projected, groups, errors


def audit_sources(manifest, sources=None):
    sources = sources or Sources()
    result = {"schema": "work-leaf-review-evidence-audit-v1", "status": "unverifiable", "errors": [], "reviews": [],
              "tool_calls": [], "unknown_native_groups": [], "public_native_activity": [], "unknown_public_groups": [],
              "untraced_archives": [], "unlisted_archive_files": [], "provider_work_started": False,
              "scope": "Exact held/archive and accepted public/native review input identities plus narrowly supported native archive reads. No usage arithmetic, response-ID invention, outcome exclusion, quality or causal claim. Unrecognized access is unresolved, not absent."}
    errors = result["errors"]; events = {}; payloads = {}; deliveries = {}; root_inventory = None
    try:
        require(isinstance(manifest, dict) and manifest.get("schema") == INPUT_SCHEMA, "unsupported collector input")
        run, condition = manifest.get("run_id"), manifest.get("condition")
        require(identity(run) and condition in primitives().CONDITIONS, "invalid run/condition")
        result.update(run_id=run, condition=condition)
        sources.read({"path": str(Path(__file__).resolve()), "sha256": manifest["helper_sha256"]})
        sources.read({"path": str(HERE / "audit_review_evidence.py"), "sha256": PRIMITIVE_PIN})
        runtime = manifest["runtime_sources"]
        require(isinstance(runtime, dict) and set(runtime) == set(RUNTIME_PINS), "runtime source inventory differs")
        for name, expected in RUNTIME_PINS.items():
            require(runtime[name].get("sha256") == expected, "unsupported runtime source bytes: " + name)
            sources.read(runtime[name])
        result["build"] = validate_build(sources.json(manifest["build_attestation"]), runtime)
        result["terminal"] = terminal(sources.json(manifest["terminal"]), run, condition)
        activation = sources.json(manifest["activation"])
        require(set(activation) == {"schema", "run_id", "condition", "evidence_path", "review_evidence_root"}
                and activation.get("schema") == SCHEMA and activation.get("run_id") == run and activation.get("condition") == condition
                and activation.get("evidence_path") == manifest["trace"]["path"], "activation manifest differs")
        root = Path(activation["review_evidence_root"])
        require(root.is_absolute() and str(root) == activation["review_evidence_root"] and root.resolve() == root and root.is_dir(), "archive root is not exact canonical directory")
        traced = sources.records(manifest["trace"]); trace = [row for _, row in traced]
        # Retain every physical review row even when a later global identity gate fails.
        for line, event in traced:
            if event.get("event") == "review-context":
                index = len(result["reviews"]); events[index] = event
                result["reviews"].append({"trace_line": line, "sequence": event.get("sequence") if type(event.get("sequence")) is int else None,
                    "status": "unverified", "errors": [], "retrieval": {"status": "unresolved", "reason": "not yet verified"}})
        require(trace and trace[0].get("event") == "activation" and sum(row.get("event") == "activation" for row in trace) == 1, "activation event inventory differs")
        for key in ("schema", "run_id", "condition", "review_evidence_root"):
            require(trace[0].get(key) == activation[key], "activation trace identity differs")
        archive_refs = {}
        declared_files = set()
        for item in manifest["archives"]:
            path = item["payload"]["path"]
            require(path not in archive_refs, "duplicate payload reference")
            for ref in (item["payload"], item["manifest"]):
                require(Path(ref["path"]).parent == root and ref["path"] not in declared_files, "archive reference outside root or duplicated")
                declared_files.add(ref["path"])
            archive_refs[path] = item
        root_inventory = root, archive_listing(root)
        result["unlisted_archive_files"] = sorted(root_inventory[1] - declared_files)
        if root_inventory[1] != declared_files: errors.append("retained archive directory inventory differs from supplied references")
        seen_paths = set(); seen_sequences = set()
        for index, event in events.items():
            review = result["reviews"][index]
            try:
                archive = event["archive"]; path = archive["path"]
                require(path not in seen_paths and archive["sequence"] not in seen_sequences, "duplicate issued archive identity")
                seen_paths.add(path); seen_sequences.add(archive["sequence"])
                item = archive_refs[path]; require(item["manifest"]["path"] == archive["manifest_path"], "typed archive source differs")
                payloads[index] = sources.read(item["payload"]); typed = sources.json(item["manifest"])
                review["archive"] = validate_event(event, payloads[index], typed, run, condition, str(root))
                review.update(status="verified", source_agent_id=event["source_agent_id"], reviewer_id=event["reviewer_id"], target_commit=event["target_commit"])
            except (OSError, ValueError, KeyError, TypeError, AttributeError) as error:
                review["errors"].append(str(error)); errors.append("review source verification failed at physical trace line " + str(review["trace_line"]) + ": " + str(error))
        for path in sorted(set(archive_refs) - seen_paths):
            item = archive_refs[path]
            entry = {"payload_path": path, "manifest_path": item["manifest"]["path"], "status": "unresolved", "errors": []}
            for kind, ref in item.items():
                try:
                    data = sources.read(ref); entry[kind + "_sha256"] = sha(data)
                except (OSError, ValueError, TypeError) as error:
                    entry["errors"].append(str(error))
            result["untraced_archives"].append(entry)
        if seen_paths != set(archive_refs): errors.append("archive source inventory differs from issued trace; untraced publications retained")
        previous = 0
        for event in trace[1:]:
            require(event.get("schema") == SCHEMA and event.get("run_id") == run and event.get("condition") == condition,
                    "trace row identity differs")
            require(type(event.get("sequence")) is int and event["sequence"] == previous + 1, "trace sequence is not contiguous")
            previous = event["sequence"]
            if event.get("event") != "review-context":
                require(event.get("event") == "prompt" and event.get("site") in {"policy-injection", "patch-applied", "command-result"}
                        and identity(event.get("agent_id")) and isinstance(event.get("original_prompt"), str)
                        and event["original_prompt"] == event.get("forwarded_prompt"), "nonfactor prompt mutated or unknown boundary")
                size = len(event["original_prompt"].encode())
                require(type(event.get("changed")) is bool and event["changed"] is False
                        and all(type(event.get(key)) is int and event[key] == value for key, value in (("original_bytes", size), ("forwarded_bytes", size), ("byte_delta", 0))),
                        "nonfactor byte fields differ")
        captures = collect_captures(manifest, sources)
        native_sources = [{"thread_id": item["thread_id"], "source": item["source"]["path"], "rows": sources.records(item["source"])} for item in manifest["native_sessions"]]
        joined = primitives().join_review_inputs(trace, captures, native_sources)
        physical = {cap["path"]: cap for cap in captures}
        by_sequence = {event["sequence"]: index for index, event in events.items()}
        for row in joined["reviews"]:
            cap = physical[row["capture"]]; row = dict(row)
            row["client_line"] = cap["client_lines"][row["client_line"] - 1]
            row["public_line"] = cap["server_lines"][row["public_line"] - 1]
            index = by_sequence[row["trace_sequence"]]; require(index not in deliveries, "duplicate review delivery")
            deliveries[index] = row; result["reviews"][index]["delivery"] = row
        result["accepted_inputs"] = joined["accepted_inputs"]; result["threads"] = joined["threads"]
        result["tool_calls"], result["unknown_native_groups"], tool_errors = derive_retrieval(result["reviews"], events, payloads, deliveries, native_sources, joined["inputs"])
        errors.extend(tool_errors)
        result["public_native_activity"], result["unknown_public_groups"], public_errors = public_activity(captures, joined["inputs"], result["tool_calls"], result["reviews"])
        errors.extend(public_errors)
    except (OSError, ValueError, KeyError, TypeError, AttributeError, IndexError) as error:
        errors.append(str(error))
    try:
        sources.finish()
        if root_inventory is not None:
            require(archive_listing(root_inventory[0]) == root_inventory[1], "archive directory inventory changed during audit")
    except (OSError, ValueError, TypeError) as error:
        errors.append("endpoint source check: " + str(error))
    result["source_sha256"] = dict(sources.hashes)
    result["status"] = "available" if not errors else "unverifiable"
    return result


def write_new(path, result):
    path = Path(path)
    require(path.is_absolute() and str(path) == str(path.absolute()) and path.parent.resolve() == path.parent, "output parent is aliased or relative")
    with path.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, sort_keys=True, indent=2, allow_nan=False); stream.write("\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True); parser.add_argument("--input-sha256", required=True)
    parser.add_argument("--output", type=Path, required=True); args = parser.parse_args()
    try:
        require(not args.output.exists() and not args.output.is_symlink(), "derived output already exists")
        sources = Sources(); manifest = sources.json({"path": str(args.input.absolute()), "sha256": args.input_sha256})
        result = audit_sources(manifest, sources); write_new(args.output.absolute(), result)
        print(json.dumps({"status": result["status"], "provider_work_started": False, "output": str(args.output)}))
        return 0 if result["status"] == "available" else 1
    except (OSError, ValueError, KeyError, TypeError) as error:
        print("review evidence census: " + str(error), file=sys.stderr); return 2


if __name__ == "__main__":
    raise SystemExit(main())
