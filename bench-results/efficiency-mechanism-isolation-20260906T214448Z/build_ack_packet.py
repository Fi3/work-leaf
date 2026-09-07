#!/usr/bin/env python3
"""Derive one closed workflow's manual ACK-coding packet, never labels or contrasts.

Frozen delivery/provenance algorithms remain the authority. Streams and identities are
indexed once; each public item and turn is stored once, with ACK context index ranges.
Cost is O(B + F log F) for bounded bytes B and support files F, not ACKs times bytes.
"""
import argparse
from datetime import datetime
import hashlib
import importlib.machinery
import json
import os
from pathlib import Path
import re
import stat
import types

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
MAX_FILE = 512 * 1024 * 1024
MAX_TOTAL = 2 * 1024 * 1024 * 1024
MAX_SOURCES = 4096


def require(value, message):
    if not value:
        raise ValueError(message)


def identity(value):
    return type(value) is str and bool(value)


def rpc(value):
    require(type(value) in (str, int), "invalid typed RPC identity")
    return type(value).__name__, value


class SourceIndex:
    def __init__(self, phase):
        self.phase = phase
        self.hashes = {}
        self.total = 0

    def path(self, value, *, external=False):
        path = Path(value)
        require(path.is_absolute() and path.resolve() == path, "source path must be absolute and nonsymlinked")
        require(external or path.is_relative_to(self.phase), "source/output must be phase-local")
        return path

    def read(self, value, *, external=False):
        path = self.path(value, external=external)
        before = path.lstat()
        require(stat.S_ISREG(before.st_mode) and before.st_size <= MAX_FILE, "source file unsupported or too large")
        require(len(self.hashes) < MAX_SOURCES or str(path) in self.hashes, "source inventory limit")
        require(str(path) in self.hashes or self.total + before.st_size <= MAX_TOTAL, "total source byte limit")
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as stream:
            opened = os.fstat(stream.fileno())
            data = stream.read(MAX_FILE + 1)
            after = os.fstat(stream.fileno())
        key = lambda s: (s.st_dev, s.st_ino, s.st_mode, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
        require(key(before) == key(opened) == key(after) and len(data) <= MAX_FILE, "source changed during read")
        digest = hashlib.sha256(data).hexdigest()
        if str(path) in self.hashes:
            require(self.hashes[str(path)] == digest, "source hash changed")
        else:
            self.total += len(data)
            require(self.total <= MAX_TOTAL, "total source byte limit")
            self.hashes[str(path)] = digest
        return data

    def json(self, path):
        value = json.loads(self.read(path))
        require(isinstance(value, dict), "source JSON must be an object")
        return value

    def records(self, path):
        data = self.read(path)
        require(not data or data.endswith(b"\n"), "incomplete JSONL tail")
        rows, physical = [], []
        for line, raw in enumerate(data.splitlines(), 1):
            if not raw.strip():
                continue
            value = json.loads(raw)
            require(isinstance(value, dict), "nonobject JSONL record")
            rows.append(value)
            physical.append(line)
        return rows, physical

    def verify(self):
        for path in tuple(self.hashes):
            self.read(path, external=True)


def load_frozen(index, manifest):
    """Load admitted source bytes, including repository helper imports, without pyc."""
    pins = {row["path"]: row["sha256"] for row in manifest["files"]}
    require(len(pins) == len(manifest["files"]), "duplicate frozen source path")
    original_loader = importlib.machinery.SourceFileLoader.get_code

    def source_code(loader, name):
        path = Path(loader.path).resolve()
        if path.is_relative_to(REPO):
            return compile(index.read(path, external=True), str(path), "exec")
        return original_loader(loader, name)

    def module(name):
        path = HERE / name
        data = index.read(path, external=True)
        saved = index.phase / "infrastructure/evidence" / path.relative_to(REPO)
        saved_data = index.read(saved)
        require(pins.get(str(saved)) == hashlib.sha256(data).hexdigest()
                and saved_data == data, "frozen helper source identity differs")
        result = types.ModuleType("coding_" + path.stem)
        result.__file__ = str(path)
        exec(compile(data, str(path), "exec"), result.__dict__)
        return result

    importlib.machinery.SourceFileLoader.get_code = source_code
    try:
        return module("analyze_work_units.py"), module("analyze.py")
    finally:
        importlib.machinery.SourceFileLoader.get_code = original_loader


def reference(path, line, pointer):
    return {"path": str(path), "locator": f"JSONL line {line}; {pointer}"}


def project_packet(inventory, captures, trace_path, trace_lines, frozen):
    require(inventory.get("status") == "available" and inventory.get("errors") == [], "frozen prompt inventory is unverified")
    cues = tuple(dict.fromkeys((*frozen.A, *frozen.B, *frozen.C)))
    redactor = re.compile("|".join(re.escape(cue) for cue in cues))
    clean = lambda text: redactor.sub("[work-unit policy prose omitted]", text)
    exposures = {(e["capture"], rpc(e["rpc_id"])): e for e in inventory["exposures"]}
    thread_groups, turns, turn_requests = {}, {}, {}
    unlinked, native_incomplete = [], False
    item_fingerprints = {}
    for cap in captures:
        root = Path(cap["path"])
        replies = {rpc(value["id"]): (line, value) for line, value in enumerate(cap["servers"])
                   if "id" in value and "method" not in value}
        for offset, frame in enumerate(cap["clients"]):
            if frame.get("method") != "turn/start":
                continue
            key = rpc(frame["id"])
            reply_offset, reply = replies.get(key, (None, {}))
            if "error" in reply or not isinstance(reply.get("result"), dict):
                continue
            params = frame["params"]
            thread, turn = params["threadId"], reply["result"]["turn"]["id"]
            require(identity(thread) and identity(turn), "invalid accepted turn identity")
            scope = (str(root), thread)
            group = thread_groups.setdefault(scope, {"capture": str(root), "thread_id": thread, "turns": []})
            text = params["input"][0]["text"]
            exposure = exposures.get((str(root), key))
            site = exposure["site"] if exposure else "ordinary-followup"
            if site == "policy-injection":
                public = "[launch policy omitted; full request retained at source locator]"
            elif site == "patch-applied":
                public = "\n".join(text.splitlines()[:2])
            else:
                public = clean(text)
            record = {"turn_id": turn, "rpc_id": frame["id"], "rpc_id_type": key[0],
                      "thread_turn_index": len(group["turns"]), "request_site": site,
                      "public_user_text": public, "public_items": [],
                      "request_source": reference(root / "client-to-server.raw", cap["client_lines"][offset], "/params/input/0/text"),
                      "reply_source": reference(root / "server-to-client.raw", cap["server_lines"][reply_offset], "/result/turn/id")}
            turn_key = (str(root), thread, turn)
            require(turn_key not in turns, "duplicate accepted turn")
            turns[turn_key] = record
            turn_requests[(str(root), key)] = record
            group["turns"].append(record)
        for offset, frame in enumerate(cap["servers"]):
            if frame.get("method") != "item/completed":
                continue
            params = frame.get("params", {})
            item = params.get("item", {})
            require(isinstance(item, dict) and identity(item.get("type")), "unsupported completed item shape")
            if item["type"] == "reasoning":
                continue
            thread, turn, item_id = params.get("threadId"), params.get("turnId"), item.get("id")
            require(all(identity(value) for value in (thread, turn, item_id)), "invalid public item identity")
            item_key = (str(root), thread, item_id)
            fingerprint = hashlib.sha256(json.dumps([turn, item], sort_keys=True).encode()).hexdigest()
            if item_key in item_fingerprints:
                require(item_fingerprints[item_key] == fingerprint, "conflicting public item identity")
                continue
            item_fingerprints[item_key] = fingerprint
            record = {"item_id": item_id, "kind": item["type"], "thread_id": thread, "turn_id": turn,
                      "source": reference(root / "server-to-client.raw", cap["server_lines"][offset], "/params/item")}
            if item["type"] == "agentMessage":
                require(isinstance(item.get("text"), str), "public agent message text missing")
                record["public_text"] = clean(item["text"])
            else:
                record["body_projection"] = "not exported; consult retained public item at exact source"
                native_incomplete = True
            target = turns.get((str(root), thread, turn))
            if target is None:
                record.pop("public_text", None)
                record["association"] = "no accepted turn in this capture; no inferred link"
                unlinked.append(record)
            else:
                target["public_items"].append(record)
    acks = []
    for ack in inventory["acks"]:
        record = turn_requests[(ack["capture"], rpc(ack["rpc_id"]))]
        group = thread_groups[(ack["capture"], ack["thread_id"])]
        position = record["thread_turn_index"]
        acks.append({**{key: ack[key] for key in ("ack_id", "capture", "rpc_id", "thread_id", "turn_id", "agent_id", "files_receipt")},
                     "rpc_id_type": rpc(ack["rpc_id"])[0], "thread_turn_index": position,
                     "request_source": record["request_source"], "reply_source": record["reply_source"],
                     "trace_source": reference(trace_path, trace_lines[ack["trace_line"] - 1], "/forwarded_prompt"),
                     "preceding_context_range": [0, position], "following_context_range": [position, len(group["turns"])],
                     "nearest_prior_turn_candidate_index": position - 1 if position else None,
                     "edit_group_join": "unassigned; candidate context is not an application receipt",
                     "ambiguity": "No typed PatchApplied/commit join. Check returned directive prefix, earlier followups and prior state manually; raw suffixes may be unprocessed."})
    return {"acks": acks, "threads": list(thread_groups.values()), "unlinked_public_items": unlinked,
            "native_activity_projection_incomplete": native_incomplete, "perfect_blinding_claimed": False,
            "presentation": "Policy launch bodies and exact A/B/C prose are omitted; identity/source links may reveal assignment. Public items are captured, not proof they were processed. Ranges are half-open turn indices; no inter-stream global timestamp merge."}


def support_sources(index, artifact):
    links = []
    for name in ("final-state.json", "final-log.txt", "observation/context-bundles", "observation/git-checkpoints",
                 "observation/locked-commands", "patches"):
        root = artifact / name
        if not root.exists():
            continue
        if root.is_file():
            index.read(root)
            links.append(str(root))
            continue
        pending, visited = [root], 0
        while pending:
            directory = pending.pop()
            index.path(directory)
            with os.scandir(directory) as children:
                for child in children:
                    visited += 1
                    require(visited <= MAX_SOURCES, "support inventory limit")
                    path = Path(child.path)
                    if child.is_dir(follow_symlinks=False):
                        pending.append(path)
                    else:
                        index.read(path)
                        links.append(str(path))
    return sorted(links)


def build_packet(phase_root, run_id, output):
    phase = Path(phase_root)
    require(phase.is_absolute() and phase.resolve() == phase, "phase must be absolute and canonical")
    index = SourceIndex(phase)
    output = index.path(output)
    if output.exists():
        raise FileExistsError(output)
    manifest = index.json(phase / "PHASE-MANIFEST.json")
    admission = index.json(phase / "RUN-ONCE/admission.json")
    require(admission.get("manifest_sha256") == index.hashes[str(phase / "PHASE-MANIFEST.json")], "phase admission hash differs")
    rows = [row for row in manifest["schedule"] if row.get("run_id") == run_id]
    require(identity(run_id) and len(rows) == 1, "missing or ambiguous frozen run identity")
    row = rows[0]
    receipt = index.json(phase / "logs" / (run_id + ".exit.json"))
    require(receipt.get("id") == run_id and receipt.get("launch_status") == "completed"
            and type(receipt.get("launcher_exit_code")) is int, "workflow is not terminally published")
    require(datetime.fromisoformat(receipt["finished_at"]) >= datetime.fromisoformat(receipt["started_at"]), "terminal times invalid")
    require(all(receipt.get(key) == row.get(key) for key in ("run_id", "condition", "phase", "block_id", "wave", "workflow", "artifact", "report", "prompt_trace", "experiment_manifest")), "exit receipt differs from frozen row")
    require(row.get("workflow") == "work-leaf-concurrent", "only work-unit WL workflows supported")
    artifact = index.path(row["artifact"])
    report = index.json(row["report"])
    require(report.get("agent_model") == manifest["model"] and report.get("agent_reasoning_effort") == manifest["reasoning_effort"], "published report model differs")
    frozen, base = load_frozen(index, manifest)
    experiment = index.json(row["experiment_manifest"])
    require(experiment == {"schema": frozen.SCHEMA, "run_id": run_id, "condition": row["condition"], "evidence_path": row["prompt_trace"]}, "experiment is not exact v2 row")
    require(any(entry["path"] == row["experiment_manifest"] and entry["sha256"] == index.hashes[row["experiment_manifest"]] for entry in manifest["files"]), "experiment manifest lacks frozen identity")
    observed = index.json(artifact / "observation/analysis.json")
    require(observed.get("run_id") == run_id and observed.get("condition") == "work-leaf", "observer run identity differs")
    captures = []
    with os.scandir(artifact / "observation/app-server") as directories:
        for count, entry in enumerate(directories):
            require(count < 128 and entry.is_dir(follow_symlinks=False), "unsupported capture inventory")
            app = index.path(entry.path)
            invocation = app.parent.parent / "invocations" / app.name
            start = index.json(invocation / "start.json")
            index.read(invocation / "end.json")
            required = ["raw-response-usage.json", "raw-response-rewrites.jsonl", "client-to-server.forwarded.raw",
                        "client-to-server.raw", "server-to-client.raw"]
            if start.get("provider_usage_grace_ms"):
                required.append("provider-usage-grace.jsonl")
            for name in required:
                index.read(app / name)
            provenance = base.capture_provenance(app)
            require(provenance["errors"] == [], "closed capture provenance failed")
            for path, expected in provenance["source_sha256"].items():
                index.read(path)
                require(index.hashes[path] == expected, "capture changed after provenance")
            clients, client_lines = index.records(app / "client-to-server.raw")
            servers, server_lines = index.records(app / "server-to-client.raw")
            captures.append({"path": str(app), "clients": clients, "servers": servers,
                             "client_lines": client_lines, "server_lines": server_lines})
    require(captures, "no closed captures")
    captures.sort(key=lambda cap: cap["path"])
    trace, trace_lines = index.records(row["prompt_trace"])
    inventory = frozen.prompt_inventory(trace, observed["threads"], captures, run_id, row["condition"])
    packet = project_packet(inventory, captures, row["prompt_trace"], trace_lines, frozen)
    packet.update(schema="work-leaf-ack-coding-packet-v1", run_id=run_id,
                  phase_manifest_sha256=admission["manifest_sha256"],
                  scope="One closed workflow; manual evidence preparation only. No labels, contrasts, token totals, causal inference or inferred commit links.",
                  support_source_paths=support_sources(index, artifact))
    index.read(Path(__file__).resolve(), external=True)
    packet["source_sha256"] = dict(index.hashes)
    encoded = json.dumps(packet, indent=2, sort_keys=True, allow_nan=False) + "\n"
    index.verify()
    output.parent.mkdir(parents=True, exist_ok=True)
    index.path(output.parent)
    with output.open("x", encoding="utf-8") as stream:
        stream.write(encoded)
        stream.flush()
        os.fsync(stream.fileno())
    return packet


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--phase-root", type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    build_packet(args.phase_root, args.run_id, args.output)


if __name__ == "__main__":
    main()
