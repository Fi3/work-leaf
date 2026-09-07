"""Provider-free source/identity/retrieval checks for the opaque-review collector."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import audit_review_evidence_sources as audit


def digest(data):
    return hashlib.sha256(data).hexdigest()


class CollectorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def ref(self, path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        data = value if isinstance(value, bytes) else json.dumps(value).encode()
        path.write_bytes(data)
        return {"path": str(path), "sha256": digest(data)}

    def records(self, path, rows):
        return self.ref(path, b"\n".join(json.dumps(row).encode() for row in rows) + b"\n")

    def fixture(self, command="cat", condition="review-evidence-native"):
        primitive = audit.primitives()
        run = "generic-run"; reviewer = "review-author"; thread = "thread-review"; turn = "turn-review"
        root = self.root / "opaque"; root.mkdir()
        path = root / "review-0000000000000001.txt"
        manifest_path = root / "review-0000000000000001.json"
        payload = b"Public recorded evidence\nsecond line\n"
        archive = {"kind": "review-source-context", "path": str(path), "manifest_path": str(manifest_path),
                   "sequence": 1, "bytes": len(payload), "digest": primitive.fnv64(payload), "run_id": run,
                   "source_agent_id": "author", "reviewer_id": reviewer, "target_commit": "abc"}
        # Independent literal of the pinned Rust receipt, not the collector's formatter.
        receipt = ("Work Leaf opaque review evidence (temporary read-only context):\n"
                   f"Path: {json.dumps(str(path), ensure_ascii=False)}\nBytes: {len(payload)}\nEvidence identity: {archive['digest']}\n"
                   "This artifact contains the same complete commit/log/recorded-chat evidence for this review. "
                   "Native read-only inspection of this exact supplied path is permitted; it is not served by `@work-leaf read`. "
                   "Consult relevant archived evidence before declaring required evidence missing.\n")
        prefix = "Review the full patch scope for Agent-ID author.\nLatest commit: abc\nFeature: generic\nReason: change\nReview scope:\ncommit abc\n\nSource context from Work Leaf commits, logs, and chat history:\n"
        original = prefix + payload.decode() + audit.REVIEW_SUFFIX
        candidate = prefix + receipt + audit.REVIEW_SUFFIX
        selected = candidate if condition.endswith("native") else original
        a = len(prefix.encode()); b = a + len(payload)
        event = {"event": "review-context", "schema": audit.SCHEMA, "site": "review-source-context",
                 "sequence": 1, "run_id": run, "condition": condition, "process_id": 8, "unix_time_ns": "12",
                 "source_agent_id": "author", "reviewer_id": reviewer, "target_commit": "abc",
                 "original_prompt": original, "candidate_prompt": candidate, "forwarded_prompt": selected,
                 "context_start": a, "context_end": b, "candidate_start": a,
                 "candidate_end": a + len(receipt.encode()), "selected_candidate": "candidate" if condition.endswith("native") else "baseline",
                 "original_bytes": len(original.encode()), "candidate_bytes": len(candidate.encode()),
                 "forwarded_bytes": len(selected.encode()), "byte_delta": len(selected.encode()) - len(original.encode()),
                 "changed": selected != original, "archive": archive}
        prompt = "You are running under the work-leaf orchestrator.\n\nAgent-ID: review-author\nFeature: generic\n\nUser prompt:\n" + selected
        policy = {"event": "prompt", "schema": audit.SCHEMA, "site": "policy-injection", "sequence": 2,
                  "run_id": run, "condition": condition, "agent_id": reviewer, "original_prompt": prompt,
                  "forwarded_prompt": prompt, "original_bytes": len(prompt.encode()), "forwarded_bytes": len(prompt.encode()),
                  "byte_delta": 0, "changed": False}
        trace_path = self.root / "trace.jsonl"
        activation = {"schema": audit.SCHEMA, "run_id": run, "condition": condition,
                      "evidence_path": str(trace_path), "review_evidence_root": str(root)}
        trace = [{"event": "activation", "schema": audit.SCHEMA, "run_id": run, "condition": condition,
                  "review_evidence_root": str(root), "process_id": 8}, event, policy]
        request = {"id": "1", "method": "turn/start", "params": {"threadId": thread, "input": [{"type": "text", "text": prompt}]}}
        server = [{"id": "1", "result": {"turn": {"id": turn}}},
                  {"method": "item/completed", "params": {"threadId": thread, "turnId": turn,
                   "item": {"id": "public-user", "type": "userMessage", "content": [{"type": "text", "text": prompt, "text_elements": []}]}}},
                  {"method": "turn/completed", "params": {"threadId": thread, "turn": {"id": turn, "status": "completed"}}}]
        obs = self.root / "observation"; cap = obs / "app-server" / "inv-1"
        clients = self.records(cap / "client-to-server.raw", [request])
        forwarded = self.records(cap / "client-to-server.forwarded.raw", [request])
        servers = self.records(cap / "server-to-client.raw", server)
        start = {"invocation_id": "inv-1", "capture_kind": "app-server", "primary": True, "start_unix_ns": 1}
        start_ref = self.ref(obs / "invocations" / "inv-1" / "start.json", start)
        end = {"invocation_id": "inv-1", "end_unix_ns": 20, "exit_code": 0, "terminating_signal": None,
               "raw_response_usage_start_sha256": start_ref["sha256"], "stdin_sha256": clients["sha256"],
               "stdout_sha256": servers["sha256"], "raw_response_usage_sha256": {"client-to-server.forwarded.raw": forwarded["sha256"]}}
        end_ref = self.ref(obs / "invocations" / "inv-1" / "end.json", end)
        native = [{"type": "session_meta", "payload": {"id": thread}},
                  {"type": "turn_context", "payload": {"turn_id": turn}},
                  {"type": "response_item", "payload": {"type": "message", "role": "user", "id": "native-user",
                   "internal_chat_message_metadata_passthrough": {"turn_id": turn, "content_item_kinds": ["user.text"]},
                   "content": [{"type": "input_text", "text": prompt}]}}]
        if command is not None:
            native += [{"type": "response_item", "payload": {"type": "function_call", "name": "exec_command", "call_id": "call-1", "id": "native-call",
                        "arguments": json.dumps({"cmd": command + " '" + str(path) + "'"})}},
                       {"type": "response_item", "payload": {"type": "function_call_output", "call_id": "call-1", "id": "native-output",
                        "output": "Chunk ID: x\nWall time: 0.1 seconds\nProcess exited with code 0\nOutput:\n" + payload.decode()}}]
        runtime = {name: {"path": str(audit.REPO / name), "sha256": sha} for name, sha in audit.RUNTIME_PINS.items()}
        build = {"schema": "work-leaf-review-evidence-build-attestation-v5", "source_repo": str(audit.REPO),
                 "source_files": [{"path": name, "sha256": ref["sha256"]} for name, ref in runtime.items()],
                 "binaries": [{"path": "target/release/work-leaf", "sha256": "a" * 64}],
                 "runtime_build_exit": 0, "runtime_build": "cargo build --release --features bench-experiments --bins",
                 "observer_reused_unchanged": True, "smoke_build_exit": 0, "smoke_build": "cargo test --no-run --features bench-experiments --test generic_smoke",
                 "smoke_test_binary": {"path": "/retained/build/smoke", "sha256": "b" * 64}}
        result = {"schema": audit.INPUT_SCHEMA, "run_id": run, "condition": condition,
                  "helper_sha256": digest(Path(audit.__file__).read_bytes()), "runtime_sources": runtime,
                  "build_attestation": self.ref(self.root / "build.json", build),
                  "activation": self.ref(self.root / "activation.json", activation),
                  "trace": self.records(trace_path, trace),
                  "terminal": self.ref(self.root / "terminal.json", {"schema": "work-leaf-review-evidence-terminal-v1", "run_id": run, "condition": condition,
                      "started_at": "2026-09-07T01:00:00+00:00", "finished_at": "2026-09-07T01:01:00+00:00", "exit_code": 0}),
                  "invocations": self.records(obs / "process-invocations.jsonl", [{**start, "end": end}]),
                  "captures": [{"path": str(cap), "clients": clients, "forwarded": forwarded, "servers": servers, "start": start_ref, "end": end_ref}],
                  "native_sessions": [{"thread_id": thread, "source": self.records(self.root / "native.jsonl", native)}],
                  "archives": [{"payload": self.ref(path, payload), "manifest": self.ref(manifest_path,
                      {"schema": "work-leaf-review-evidence-v1", "archive": archive, "context_start": a, "context_end": b, "project_snapshots_applicable": False})}]}
        return result

    def mutate(self, ref, change, records=False):
        path = Path(ref["path"])
        value = [json.loads(line) for line in path.read_text().splitlines()] if records else json.loads(path.read_text())
        change(value)
        ref.update(self.records(path, value) if records else self.ref(path, value))

    def rebind_capture(self, manifest):
        cap = manifest["captures"][0]
        for key in ("clients", "forwarded", "servers"):
            cap[key]["sha256"] = digest(Path(cap[key]["path"]).read_bytes())
        self.mutate(cap["end"], lambda value: value.update(stdin_sha256=cap["clients"]["sha256"], stdout_sha256=cap["servers"]["sha256"],
                    raw_response_usage_sha256={"client-to-server.forwarded.raw": cap["forwarded"]["sha256"]}))
        end = json.loads(Path(cap["end"]["path"]).read_text())
        self.mutate(manifest["invocations"], lambda values: values[0].update(end=end), records=True)

    def test_full_source_bound_closed_workflow_without_usage_records(self):
        result = audit.audit_sources(self.fixture())
        self.assertEqual(result["status"], "available", result["errors"])
        self.assertEqual(result["reviews"][0]["retrieval"]["status"], "complete")
        self.assertEqual(result["reviews"][0]["delivery"]["native_line"], 3)
        self.assertEqual(result["tool_calls"][0]["call_line"], 4)
        self.assertEqual(result["tool_calls"][0]["output_line"], 5)
        self.assertNotIn("Public recorded evidence", json.dumps(result))
        self.assertNotIn("response_id", json.dumps(result))

    def test_no_native_tool_activity_is_observed_absence_not_inference(self):
        result = audit.audit_sources(self.fixture(command=None))
        self.assertEqual(result["reviews"][0]["retrieval"]["status"], "absent")

    def test_unsupported_command_remains_unresolved(self):
        result = audit.audit_sources(self.fixture(command="/custom/cat"))
        self.assertEqual(result["reviews"][0]["retrieval"]["status"], "unresolved")

    def test_manifest_project_snapshot_or_byte_field_forgery_is_retained(self):
        manifest = self.fixture()
        self.mutate(manifest["archives"][0]["manifest"], lambda value: value.update(project_snapshots_applicable=True))
        result = audit.audit_sources(manifest)
        self.assertEqual(result["status"], "unverifiable")
        self.assertEqual(len(result["reviews"]), 1)
        self.assertEqual(result["reviews"][0]["retrieval"]["status"], "unresolved")

    def test_nonzero_terminal_is_retained_without_exclusion(self):
        manifest = self.fixture()
        self.mutate(manifest["terminal"], lambda value: value.update(exit_code=101))
        self.assertEqual(audit.audit_sources(manifest)["terminal"]["exit_code"], 101)

    def test_source_mismatch_symlink_and_null_digest_fail_closed(self):
        manifest = self.fixture()
        for mode in ("hash", "symlink", "null"):
            changed = copy.deepcopy(manifest)
            if mode == "hash": changed["trace"]["sha256"] = "0" * 64
            if mode == "null": changed["trace"]["sha256"] = None
            if mode == "symlink":
                link = self.root / "trace-link"; link.symlink_to(manifest["trace"]["path"]); changed["trace"]["path"] = str(link)
            with self.subTest(mode=mode):
                self.assertEqual(audit.audit_sources(changed)["status"], "unverifiable")

    def test_create_new_output_never_overwrites_or_follows_parent_symlink(self):
        path = self.root / "result.json"
        audit.write_new(path, {"status": "test"})
        with self.assertRaises((ValueError, FileExistsError)):
            audit.write_new(path, {"status": "overwrite"})
        directory = self.root / "real"; directory.mkdir()
        alias = self.root / "alias"; alias.symlink_to(directory, target_is_directory=True)
        with self.assertRaises(ValueError):
            audit.write_new(alias / "result.json", {})

    def test_duplicate_call_output_identity_cannot_prove_retrieval(self):
        manifest = self.fixture()
        def duplicate(rows):
            other = copy.deepcopy(rows[-1]); other["payload"]["id"] = "different-output-item"
            rows.append(other)
        self.mutate(manifest["native_sessions"][0]["source"], duplicate, records=True)
        result = audit.audit_sources(manifest)
        self.assertEqual(result["reviews"][0]["retrieval"]["status"], "unresolved")

    def test_partial_read_and_explicit_nested_turn(self):
        manifest = self.fixture(command="sed -n '2p'")
        def partial(rows):
            for row in rows[-2:]:
                row["payload"]["internal_chat_message_metadata_passthrough"] = {"turn_id": "turn-review"}
            rows[-1]["payload"]["output"] = rows[-1]["payload"]["output"].replace("Public recorded evidence\n", "")
        self.mutate(manifest["native_sessions"][0]["source"], partial, records=True)
        result = audit.audit_sources(manifest)
        self.assertEqual(result["status"], "available", result["errors"])
        self.assertEqual(result["reviews"][0]["retrieval"]["status"], "verified_partial")

    def test_missing_native_usage_is_allowed_but_missing_native_thread_is_not(self):
        manifest = self.fixture(); manifest["native_sessions"] = []
        result = audit.audit_sources(manifest)
        self.assertEqual(result["status"], "unverifiable")
        self.assertEqual(len(result["reviews"]), 1)

    def test_all_prompt_byte_fields_and_literal_receipt_are_verified(self):
        manifest = self.fixture()
        for field in ("original_bytes", "candidate_bytes", "forwarded_bytes", "byte_delta", "changed", "receipt"):
            rows = [json.loads(line) for line in Path(manifest["trace"]["path"]).read_text().splitlines()]
            modified = copy.deepcopy(rows)
            if field == "changed": modified[1][field] = 1
            elif field == "receipt": modified[1]["candidate_prompt"] = modified[1]["candidate_prompt"].replace("Native read-only", "Native write-enabled")
            else: modified[1][field] += 1
            changed = copy.deepcopy(manifest); changed["trace"] = self.records(self.root / "trace-alternative.jsonl", modified)
            self.mutate(changed["activation"], lambda value: value.update(evidence_path=changed["trace"]["path"]))
            with self.subTest(field=field):
                self.assertEqual(audit.audit_sources(changed)["status"], "unverifiable")

    def test_public_physical_lines_include_blank_lines(self):
        manifest = self.fixture(); cap = manifest["captures"][0]
        for name in ("clients", "forwarded", "servers"):
            path = Path(cap[name]["path"]); path.write_bytes(b"\n" + path.read_bytes())
        self.rebind_capture(manifest)
        result = audit.audit_sources(manifest)
        self.assertEqual(result["status"], "available", result["errors"])
        self.assertEqual(result["reviews"][0]["delivery"]["client_line"], 2)
        self.assertEqual(result["reviews"][0]["delivery"]["public_line"], 3)

    def test_owned_title_launch_then_raw_title_reuse_are_retained(self):
        manifest = self.fixture(); cap = manifest["captures"][0]
        raw = "Name this Work Leaf chat from the user's first prompt.\nRules:\n- Return only the chat name, no prose and no quotes.\n- Derive the name from the first prompt only.\n- Use at most 40 characters.\n- Use lowercase words separated by hyphens.\n- Do not use spaces.\n\nFirst prompt:\na different task"
        wrapped = "You are running under the work-leaf orchestrator.\n\nAgent-ID: title-agent\nFeature: titles\n\nUser prompt:\n" + raw
        self.mutate(manifest["trace"], lambda rows: rows.append({"event": "prompt", "schema": audit.SCHEMA, "site": "policy-injection", "sequence": 3,
            "run_id": manifest["run_id"], "condition": manifest["condition"], "agent_id": "title-agent", "original_prompt": wrapped, "forwarded_prompt": wrapped,
            "original_bytes": len(wrapped.encode()), "forwarded_bytes": len(wrapped.encode()), "byte_delta": 0, "changed": False}), records=True)
        native = [{"type": "session_meta", "payload": {"id": "thread-title"}}]
        for number, prompt in ((2, wrapped), (3, raw)):
            request = {"id": str(number), "method": "turn/start", "params": {"threadId": "thread-title", "input": [{"type": "text", "text": prompt}]}}
            for key in ("clients", "forwarded"):
                self.mutate(cap[key], lambda rows: rows.append(request), records=True)
            self.mutate(cap["servers"], lambda rows: rows.extend([
                {"id": str(number), "result": {"turn": {"id": f"title-turn-{number}"}}},
                {"method": "item/completed", "params": {"threadId": "thread-title", "turnId": f"title-turn-{number}",
                    "item": {"id": f"public-title-{number}", "type": "userMessage", "content": [{"type": "text", "text": prompt}]}}}]), records=True)
            native.append({"type": "response_item", "payload": {"type": "message", "role": "user", "id": f"native-title-{number}",
                "internal_chat_message_metadata_passthrough": {"turn_id": f"title-turn-{number}", "content_item_kinds": ["user.text"]},
                "content": [{"type": "input_text", "text": prompt}]}})
        self.rebind_capture(manifest)
        manifest["native_sessions"].append({"thread_id": "thread-title", "source": self.records(self.root / "native-title.jsonl", native)})
        result = audit.audit_sources(manifest)
        self.assertEqual(result["status"], "available", result["errors"])
        self.assertEqual(result["accepted_inputs"], 3)
        self.assertEqual(result["threads"]["thread-title"], "title-agent")

    def test_unlisted_retained_partial_publication_is_not_silently_ignored(self):
        manifest = self.fixture()
        root = Path(manifest["archives"][0]["payload"]["path"]).parent
        (root / "review-0000000000000002.txt").write_bytes(b"retained partial publication")
        result = audit.audit_sources(manifest)
        self.assertEqual(result["status"], "unverifiable")
        self.assertIn(str(root / "review-0000000000000002.txt"), result["unlisted_archive_files"])

    def test_supplied_untraced_archive_is_retained_with_unknown_delivery(self):
        manifest = self.fixture()
        root = Path(manifest["archives"][0]["payload"]["path"]).parent
        path = root / "review-0000000000000002.txt"; meta = root / "review-0000000000000002.json"
        manifest["archives"].append({"payload": self.ref(path, b"not delivered"), "manifest": self.ref(meta, {"schema": "incomplete"})})
        result = audit.audit_sources(manifest)
        self.assertEqual(result["status"], "unverifiable")
        self.assertEqual(len(result["untraced_archives"]), 1)
        self.assertIn(str(path), result["source_sha256"])

    def test_nonempty_public_text_metadata_is_not_exact_plain_text_delivery(self):
        manifest = self.fixture(); cap = manifest["captures"][0]
        self.mutate(cap["servers"], lambda rows: rows[1]["params"]["item"]["content"][0].update(text_elements=[{"type": "unsupported"}]), records=True)
        self.rebind_capture(manifest)
        self.assertEqual(audit.audit_sources(manifest)["status"], "unverifiable")

    def test_unknown_build_metadata_is_not_exported(self):
        manifest = self.fixture()
        self.mutate(manifest["build_attestation"], lambda value: value["binaries"][0].update(body="SECRET_BUILD_BODY"))
        result = audit.audit_sources(manifest)
        self.assertNotIn("SECRET_BUILD_BODY", json.dumps(result))

    def test_public_native_action_missing_from_native_file_prevents_absence(self):
        manifest = self.fixture(command=None); cap = manifest["captures"][0]
        path = manifest["archives"][0]["payload"]["path"]
        self.mutate(cap["servers"], lambda rows: rows.append({"method": "item/completed", "params": {
            "threadId": "thread-review", "turnId": "turn-review", "item": {"type": "commandExecution", "id": "public-read",
            "command": "cat -- '" + path + "'", "aggregatedOutput": "Public recorded evidence\nsecond line\n",
            "commandActions": [{"type": "read", "path": path, "command": "cat"}]}}}), records=True)
        self.rebind_capture(manifest)
        result = audit.audit_sources(manifest)
        self.assertEqual(result["reviews"][0]["retrieval"]["status"], "unresolved")
        self.assertEqual(len(result["public_native_activity"]), 1)

    def test_custom_tool_cannot_masquerade_as_supported_executor(self):
        manifest = self.fixture()
        def custom(rows):
            rows[-2]["payload"]["type"] = "custom_tool_call"
            rows[-1]["payload"]["type"] = "custom_tool_call_output"
        self.mutate(manifest["native_sessions"][0]["source"], custom, records=True)
        result = audit.audit_sources(manifest)
        self.assertEqual(result["reviews"][0]["retrieval"]["status"], "unresolved")

    def test_conflicting_native_item_id_invalidates_both_call_id_claims(self):
        manifest = self.fixture()
        def conflict(rows):
            other = copy.deepcopy(rows[-2]); other["payload"]["call_id"] = "different-call"
            rows.append(other)
        self.mutate(manifest["native_sessions"][0]["source"], conflict, records=True)
        result = audit.audit_sources(manifest)
        self.assertEqual(result["reviews"][0]["retrieval"]["status"], "unresolved")

    def test_source_bound_cli_create_new_output_includes_input_hash(self):
        manifest = self.fixture()
        ref = self.ref(self.root / "collector-input.json", manifest)
        output = self.root / "derived.json"
        command = [sys.executable, str(Path(audit.__file__).absolute()), "--input", ref["path"], "--input-sha256", ref["sha256"], "--output", str(output)]
        first = subprocess.run(command, capture_output=True, text=True, timeout=10)
        self.assertEqual(first.returncode, 0, first.stderr)
        value = json.loads(output.read_text())
        self.assertEqual(value["source_sha256"][ref["path"]], ref["sha256"])
        prior = output.read_bytes()
        second = subprocess.run(command, capture_output=True, text=True, timeout=10)
        self.assertEqual(second.returncode, 2)
        self.assertEqual(output.read_bytes(), prior)


if __name__ == "__main__":
    unittest.main()
