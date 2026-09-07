"""Synthetic complete-census tests; no providers or study outcome selection."""
import copy
import hashlib
import json
import os
from pathlib import Path
import py_compile
import tempfile
import types
import unittest
from unittest.mock import patch

import audit_read_mechanism as subject
from test_analyze_untracked_reads import event, fixture


def sample(condition="control"):
    e = event(condition)
    trace, threads, captures = fixture([e], condition)
    text = e["inline_candidate_prompt"] if condition != "control" else e["baseline_prompt"]
    usage = {"inputTokens": 18, "cachedInputTokens": 0, "outputTokens": 2,
             "reasoningOutputTokens": 0, "cacheWriteInputTokens": 0, "totalTokens": 20}
    native_usage = {"input_tokens": 18, "cached_input_tokens": 0, "output_tokens": 2,
                    "reasoning_output_tokens": 0}
    def counts(i=0, o=0):
        return {"input_tokens": i, "cached_tokens": 0, "output_tokens": o, "cache_write_tokens": 0}
    attribution = {"items": {"user": counts(10), "result": counts(5), "reply": counts(o=2)},
                   "request_fields": {"tools": counts(3)}}
    raw = {"method": "rawResponse/completed", "params": {"threadId": "thread", "turnId": "turn-0", "responseId": "response-1",
        "usage": usage, "usageMetadata": {"metadata": {"attribution": attribution}}}}
    captures[0]["servers"].append(raw); captures[0]["server_lines"].append(22)
    native = [{"source": "native.jsonl", "thread_id": "thread", "rows": [
        (2, {"type": "turn_context", "payload": {"turn_id": "turn-0"}}),
        (3, {"type": "response_item", "payload": {"id": "user", "type": "message", "role": "user", "content": [{"type": "input_text", "text": text}]}}),
        (5, {"type": "response_item", "payload": {"id": "call-item", "type": "function_call", "name": "functions.exec_command", "call_id": "call", "arguments": json.dumps({"cmd": "sed -n '1,20p' /tmp/context/bundle-0.md"})}}),
        (7, {"type": "response_item", "payload": {"id": "result", "type": "function_call_output", "call_id": "call", "output": "PRIVATE_PUBLIC_RESULT_BODY"}}),
        (9, {"type": "response_item", "payload": {"id": "reply", "type": "message", "role": "assistant", "content": [{"type": "output_text", "text": "PRIVATE_PUBLIC_MESSAGE_BODY"}]}}),
        (10, {"type": "token_usage_record", "payload": {"response_id": "response-1", "thread_id": "thread", "turn_id": "turn-0", "usage": native_usage}}),
    ]}]
    accounting = {"status": "validated", "errors": [], "measurement": {"status": "bounded", "bounds": {"raw_input_plus_output": {"lower": 20, "upper": 1178020}}},
                  "exact_response_evidence": {"response-1": {"thread_id": "thread", "turn_id": "turn-0", "usage": native_usage}},
                  "gaps": [{"thread_id": "thread", "turn_id": "tail", "status": "conditional"}]}
    return {"run_id": "run", "condition": condition, "trace": trace, "threads": threads,
            "captures": captures, "native_sources": native, "accounting": accounting}


def repeated_sample(count, multiple_users=False):
    s = sample()
    for i in range(1, count):
        event = copy.deepcopy(s["trace"][1]); event["sequence"] = i+1
        s["trace"].append(event)
        client = copy.deepcopy(s["captures"][0]["clients"][0]); client["id"] = "next-"+str(i)
        s["captures"][0]["clients"].append(client); s["captures"][0]["client_lines"].append(20+i)
        s["captures"][0]["servers"].append({"id": client["id"], "result": {"turn": {"id": "turn-"+str(i)}}})
        s["captures"][0]["server_lines"].append(30+i)
        if multiple_users:
            item = copy.deepcopy(s["native_sources"][0]["rows"][1][1]); item["payload"]["id"] = "native-user-"+str(i)
            s["native_sources"][0]["rows"].append((20+i, item))
    return s


class CensusTests(unittest.TestCase):
    def test_repeated_text_and_reissued_path_use_shared_ambiguity_groups(self):
        s = repeated_sample(8, multiple_users=True)
        r = subject.derive_run(**s)
        self.assertEqual(len(r["native_user_groups"]), 1)
        self.assertEqual(len(r["native_user_groups"][0]["candidates"]), 8)
        self.assertTrue(all(row["native_user"]["candidate_group_index"] == 0 for row in r["reads"]))
        self.assertTrue(all("candidates" not in row["native_user"] for row in r["reads"]))
        self.assertEqual(len(r["issued_path_groups"]), 1)
        self.assertEqual(r["issued_path_groups"][0]["read_indices"], list(range(8)))
        ref = r["tool_calls"][0]["bundle_references"][0]
        self.assertEqual(ref["association"], "ambiguous_issued_input")
        self.assertEqual(ref["issued_path_group_index"], 0)
        self.assertNotIn("read_indices", ref)

    def test_repeated_native_owner_conflict_has_one_shared_owner_list(self):
        r = subject.derive_run(**repeated_sample(8))
        self.assertEqual(len(r["native_user_ownership_conflicts"]), 1)
        self.assertEqual(r["native_user_ownership_conflicts"][0]["read_indices"], list(range(8)))
        self.assertTrue(all(row["native_user"]["ownership_conflict_index"] == 0 for row in r["reads"]))
        self.assertTrue(all("read_indices" not in row["native_user"] for row in r["reads"]))

    def test_unknown_nested_trace_fields_never_export_bodies(self):
        s = sample(); e = s["trace"][1]
        e["snapshots"][0]["body"] = "SECRET_SNAPSHOT_BODY"
        e["bundle"]["extra_payload"] = "SECRET_BUNDLE_BODY"
        e["component"]["body"] = "SECRET_COMPONENT_BODY"
        s["accounting"]["body"] = "SECRET_ACCOUNTING_BODY"
        s["accounting"]["exact_response_evidence"]["response-1"]["body"] = "SECRET_RESPONSE_BODY"
        text = json.dumps(subject.derive_run(**s))
        for secret in ("SECRET_SNAPSHOT_BODY", "SECRET_BUNDLE_BODY", "SECRET_COMPONENT_BODY", "SECRET_ACCOUNTING_BODY", "SECRET_RESPONSE_BODY"):
            self.assertNotIn(secret, text)

    def test_global_trace_failure_retains_every_physical_read_even_duplicate_sequences(self):
        s = sample(); bad = copy.deepcopy(s["trace"][1]); bad["run_id"] = "different-run"
        s["trace"] = [s["trace"][0], bad, s["trace"][1]]
        s["trace_lines"] = [1, 3, 7]
        r = subject.derive_run(**s)
        self.assertTrue(r["errors"])
        self.assertEqual([r["trace_line"] for r in r["reads"]], [3, 7])
        self.assertTrue(all(r["status"] == "invalid_trace" for r in r["reads"]))
        self.assertTrue(all(not r["charge_refs"] for r in r["reads"]))

    def test_one_native_user_cannot_serve_two_distinct_accepted_inputs(self):
        s = sample(); e = copy.deepcopy(s["trace"][1]); e["sequence"] = 2
        s["trace"].append(e)
        client = copy.deepcopy(s["captures"][0]["clients"][0]); client["id"] = "next"
        s["captures"][0]["clients"].append(client); s["captures"][0]["client_lines"].append(12)
        s["captures"][0]["servers"].append({"id": "next", "result": {"turn": {"id": "turn-next"}}})
        s["captures"][0]["server_lines"].append(26)
        r = subject.derive_run(**s)
        self.assertEqual(len(r["reads"]), 2)
        self.assertTrue(all(row["native_user"]["status"] == "ambiguous" for row in r["reads"]))
        self.assertTrue(all(row["charge_refs"] == [] for row in r["reads"]))

    def test_explicit_passthrough_user_turn_is_checked_without_context_guess(self):
        s = sample(); user = s["native_sources"][0]["rows"][1][1]["payload"]
        user["internal_chat_message_metadata_passthrough"] = {"turn_id": "wrong-turn"}
        self.assertEqual(subject.derive_run(**s)["reads"][0]["native_user"]["status"], "unlinked")
        user["turn_id"] = "turn-0"
        self.assertTrue(subject.derive_run(**s)["errors"])
        user["internal_chat_message_metadata_passthrough"]["turn_id"] = "turn-0"
        self.assertEqual(subject.derive_run(**s)["reads"][0]["native_user"]["status"], "linked")

    def test_distinct_explicit_turns_disambiguate_identical_native_user_text(self):
        s = sample(); user = s["native_sources"][0]["rows"][1][1]["payload"]
        user["internal_chat_message_metadata_passthrough"] = {"turn_id": "turn-0"}
        duplicate = copy.deepcopy(s["native_sources"][0]["rows"][1][1])
        duplicate["payload"]["id"] = "other-user"
        duplicate["payload"]["internal_chat_message_metadata_passthrough"]["turn_id"] = "other-turn"
        s["native_sources"][0]["rows"].append((15, duplicate))
        r = subject.derive_run(**s)
        self.assertEqual(r["reads"][0]["native_user"]["status"], "linked")
        self.assertEqual(r["reads"][0]["native_user"]["item_id"], "user")
        self.assertEqual(r["reads"][0]["native_user"]["join_method"], "explicit_thread_turn_text")

    def test_exact_selected_read_native_item_and_all_response_charge_links(self):
        r = subject.derive_run(**sample())
        self.assertFalse(r["errors"])
        self.assertEqual(r["reads"][0]["native_user"]["status"], "linked")
        self.assertEqual(r["reads"][0]["native_user"]["item_id"], "user")
        self.assertEqual(r["reads"][0]["native_user"]["line"], 3)
        self.assertEqual(r["reads"][0]["client_line"], 10)
        self.assertEqual(r["reads"][0]["charge_refs"], [{"response_index": 0, "input_item_index": 0}])
        self.assertEqual(r["responses"][0]["request_field_usage"]["input_tokens"], 3)
        self.assertTrue(all(v == 0 for v in r["responses"][0]["residual"].values()))
        self.assertEqual(r["reads"][0]["same_turn_response_indices"], [0])
        self.assertEqual(r["responses"][0]["relative_to_first_eligible_read"], "same_turn")

    def test_repeated_charge_keeps_one_item_and_distinct_responses(self):
        s = sample(); raw = copy.deepcopy(s["captures"][0]["servers"][-1])
        raw["params"]["responseId"] = "response-2"
        raw["params"]["usageMetadata"]["metadata"]["attribution"]["items"]["reply-2"] = raw["params"]["usageMetadata"]["metadata"]["attribution"]["items"].pop("reply")
        s["captures"][0]["servers"].append(raw); s["captures"][0]["server_lines"].append(24)
        s["native_sources"][0]["rows"].extend([
            (11, {"type": "response_item", "payload": {"id": "reply-2", "type": "message", "role": "assistant", "content": [{"type": "output_text", "text": "second"}]}}),
            (12, {"type": "token_usage_record", "payload": {**s["native_sources"][0]["rows"][-1][1]["payload"], "response_id": "response-2"}})])
        s["accounting"]["exact_response_evidence"]["response-2"] = s["accounting"]["exact_response_evidence"]["response-1"]
        r = subject.derive_run(**s)
        self.assertFalse(r["errors"])
        self.assertEqual(len(r["reads"][0]["charge_refs"]), 2)
        self.assertEqual(len(r["tool_calls"]), 1)
        self.assertEqual(len(r["tool_outputs"]), 1)
        self.assertEqual(len(r["tool_outputs"][0]["charge_refs"]), 2)

    def test_identical_native_text_ids_stay_ambiguous_without_guessing_turn_context(self):
        s = sample(); duplicate = copy.deepcopy(s["native_sources"][0]["rows"][1][1])
        duplicate["payload"]["id"] = "other-user"
        s["native_sources"][0]["rows"].append((15, duplicate))
        r = subject.derive_run(**s)
        self.assertEqual(r["reads"][0]["native_user"]["status"], "ambiguous")
        self.assertEqual(r["reads"][0]["charge_refs"], [])
        group = r["reads"][0]["native_user"]["candidate_group_index"]
        self.assertEqual(len(r["native_user_groups"][group]["candidates"]), 2)

    def test_cross_thread_same_text_never_joins(self):
        s = sample(); s["native_sources"][0]["thread_id"] = "different-thread"
        r = subject.derive_run(**s)
        self.assertEqual(r["reads"][0]["native_user"]["status"], "unlinked")
        self.assertTrue(r["errors"])

    def test_rejected_read_retained_not_a_native_or_tool_exposure(self):
        s = sample(); s["captures"][0]["servers"][0] = {"id": "0", "error": {"code": -1, "message": "too large"}}
        r = subject.derive_run(**s)
        self.assertEqual(r["reads"][0]["status"], "rejected_not_delivered")
        self.assertEqual(r["reads"][0]["charge_refs"], [])
        self.assertEqual(r["tool_calls"][0]["bundle_references"][0]["association"], "not_issued_to_this_thread")

    def test_unselected_candidate_path_does_not_make_a_delivered_bundle(self):
        r = subject.derive_run(**sample("untracked-read-inline"))
        self.assertFalse(r["errors"])
        self.assertEqual(r["tool_calls"][0]["bundle_references"][0]["association"], "not_issued_to_this_thread")

    def test_actual_call_result_identity_not_an_automatic_file_open_claim(self):
        r = subject.derive_run(**sample())
        tool = r["tool_calls"][0]
        self.assertEqual(tool["output_indices"], [0])
        self.assertEqual(tool["bundle_references"][0]["association"], "after_unique_issued_input")
        self.assertFalse(tool["bundle_references"][0]["actual_read_proven"])
        self.assertEqual(r["tool_outputs"][0]["item_id"], "result")

    def test_tool_reference_before_input_remains_temporally_unresolved(self):
        s = sample(); line, row = s["native_sources"][0]["rows"][2]
        s["native_sources"][0]["rows"][2] = (1, row)
        r = subject.derive_run(**s)
        self.assertEqual(r["tool_calls"][0]["bundle_references"][0]["association"], "not_after_linked_input")

    def test_unknown_tool_shape_and_unmatched_output_remain_visible(self):
        s = sample()
        s["native_sources"][0]["rows"].append((14, {"type": "response_item", "payload": {"id": "unknown", "type": "future_tool_call", "secret": "DO_NOT_EXPORT"}}))
        s["native_sources"][0]["rows"][3][1]["payload"]["call_id"] = "unmatched"
        r = subject.derive_run(**s)
        self.assertEqual(r["tool_calls"][0]["output_indices"], [])
        self.assertEqual(r["tool_outputs"][0]["call_link_status"], "unlinked")
        self.assertEqual(r["unknown_native_items"][0]["kind"], "future_tool_call")

    def test_nested_counts_never_double_count_and_signed_residual_stays_unknown(self):
        s = sample(); item = s["captures"][0]["servers"][-1]["params"]["usageMetadata"]["metadata"]["attribution"]["items"]["user"]
        item["content"] = {"part": {"input_tokens": 99999}}
        self.assertFalse(subject.derive_run(**s)["errors"])
        item["input_tokens"] = 11
        r = subject.derive_run(**s)
        self.assertEqual(r["responses"][0]["residual"]["input_tokens"], -1)
        self.assertEqual(r["responses"][0]["status"], "unknown")
        self.assertTrue(r["errors"])

    def test_bodies_credentials_and_reasoning_are_not_exported(self):
        s = sample()
        s["native_sources"][0]["rows"].append((18, {"type": "response_item", "payload": {"id": "reason", "type": "reasoning", "encrypted_content": "DO_NOT_EXPORT_REASON"}}))
        text = json.dumps(subject.derive_run(**s))
        for secret in ("PRIVATE_PUBLIC_RESULT_BODY", "PRIVATE_PUBLIC_MESSAGE_BODY", "DO_NOT_EXPORT_REASON", "sed -n"):
            self.assertNotIn(secret, text)

    def test_accounting_unknown_and_gaps_never_disappear(self):
        s = sample(); s["accounting"]["measurement"]["status"] = "unbounded_accounting_gap"
        s["accounting"]["measurement"]["bounds"]["raw_input_plus_output"]["upper"] = None
        r = subject.derive_run(**s)
        self.assertIsNone(r["accounting"]["measurement"]["bounds"]["raw_input_plus_output"]["upper"])
        self.assertEqual(r["accounting"]["gaps"], s["accounting"]["gaps"])


class IndexTests(unittest.TestCase):
    def test_diagnostic_reproducer_executes_exact_source_not_stale_bytecode(self):
        path = subject.HERE/"preflight/replay_read_mechanism_diagnostics.py"
        module = types.ModuleType("diagnostic_replay_test"); module.__file__ = str(path)
        exec(compile(path.read_bytes(), str(path), "exec"), module.__dict__)
        with tempfile.TemporaryDirectory() as directory:
            helper = Path(directory)/"fixture.py"; helper.write_text("value = 1\n")
            original = helper.stat(); py_compile.compile(str(helper), doraise=True)
            helper.write_text("value = 2\n"); os.utime(helper, ns=(original.st_atime_ns, original.st_mtime_ns))
            loaded, digest = module.compile_source(helper)
            self.assertEqual(loaded.value, 2)
            self.assertEqual(digest, hashlib.sha256(helper.read_bytes()).hexdigest())

    def test_all_overlapping_exact_patterns_and_unicode_byte_offsets(self):
        patterns = ["a", "aa", "λ/a"]
        index = subject.PatternIndex(patterns)
        text = "λ/aaa".encode()
        got = sorted(index.matches(text))
        want = sorted((start, start+len(p.encode()), p) for p in patterns for start in range(len(text)) if text.startswith(p.encode(), start))
        self.assertEqual(got, want)

    def test_source_guard_rejects_symlink_mutation_and_incomplete_jsonl(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); path = root/"source"
            path.write_text('{"a":1}\n'); (root/"link").symlink_to(path)
            index = subject.SourceIndex()
            with self.assertRaises(ValueError): index.read(root/"link")
            index.read(path); path.write_text('{"a":2}\n')
            with self.assertRaises(ValueError): index.verify()
            partial = root/"partial"; partial.write_text('{"a":1}')
            with self.assertRaises(ValueError): subject.SourceIndex().records(partial)

    def test_publication_is_create_new_and_rechecks_sources(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); source = root/"source"; source.write_text("first")
            index = subject.SourceIndex(); index.read(source)
            output = root/"output.json"; subject.publish(output, {"runs": []}, index)
            saved = output.read_bytes()
            with self.assertRaises(FileExistsError): subject.publish(output, {}, index)
            self.assertEqual(output.read_bytes(), saved)
            source.write_text("changed")
            with self.assertRaises(ValueError): subject.publish(root/"never.json", {}, index)
            self.assertFalse((root/"never.json").exists())


def phase_fixture(root, *, running=False, extra=False, launched=False, invalid_extra_pin=False):
    def save(name, value):
        path = root/name; path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value)+"\n"); return path
    sources = {}
    pins = []
    for path in [Path(subject.__file__).resolve(), *(subject.HERE/name for name in subject.PINS)]:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        pins.append({"path": str(path), "sha256": digest, "role": "frozen-evidence"})
        sources[str(path)] = digest
    if invalid_extra_pin:
        extra_path = save("ordinary-input.json", {"value": "retained"})
        sources[str(extra_path)] = hashlib.sha256(extra_path.read_bytes()).hexdigest()
        pins.append({"path": str(extra_path), "sha256": None, "role": "frozen-evidence"})
    rows = [{"run_id": f"sample-{i:03}", "id": f"sample-{i:03}", "condition": "control" if i % 2 == 0 else "untracked-read-inline",
             "launch_status": "not_launched_after_supervisor_error", "started_at": None, "finished_at": None, "launcher_exit_code": None}
            for i in range(12)]
    if running: rows[0].update(launch_status="running", started_at="time")
    sample_data = None
    if launched:
        sample_data = sample(); rid = rows[0]["run_id"]
        artifact = root/rid/"artifact"; observation = artifact/"observation"; app = observation/"app-server"/"invocation"
        def recorded_save(name, value):
            p = save(name, value); sources[str(p)] = hashlib.sha256(p.read_bytes()).hexdigest(); return p
        def lines(name, values, record=True):
            p = root/name; p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text("".join(json.dumps(v)+"\n" for v in values))
            if record: sources[str(p)] = hashlib.sha256(p.read_bytes()).hexdigest()
            return p
        for e in sample_data["trace"]: e["run_id"] = rid
        trace = lines(Path("prompt-events")/(rid+".jsonl"), sample_data["trace"])
        native = lines(Path("sessions")/"rollout.jsonl", [v for _, v in sample_data["native_sources"][0]["rows"]])
        lines(observation.relative_to(root)/"rollout-metadata.jsonl", [{"thread_id": "thread", "source_relative_path": "rollout.jsonl", "source_sha256": sources[str(native)]}])
        recorded_save(observation.relative_to(root)/"analysis.json", {"threads": sample_data["threads"]})
        lines(app.relative_to(root)/"client-to-server.raw", sample_data["captures"][0]["clients"])
        lines(app.relative_to(root)/"client-to-server.forwarded.raw", sample_data["captures"][0]["clients"])
        lines(app.relative_to(root)/"server-to-client.raw", sample_data["captures"][0]["servers"])
        for name in ("raw-response-rewrites.jsonl", "provider-usage-grace.jsonl"): lines(app.relative_to(root)/name, [])
        recorded_save(app.relative_to(root)/"raw-response-usage.json", {})
        recorded_save(observation.relative_to(root)/"invocations/invocation/start.json", {"provider_usage_grace_ms": 1000})
        recorded_save(observation.relative_to(root)/"invocations/invocation/end.json", {"finished": True})
        rows[0].update(artifact=str(artifact), prompt_trace=str(trace), launch_status="completed", started_at="start-time", finished_at="end-time", launcher_exit_code=0)
        # The primary analyzer does not consume this independently retained receipt.
        save(Path("logs")/(rid+".exit.json"), rows[0])
    frozen = save("PHASE-MANIFEST.json", {"phase": "sample-phase", "schedule": rows, "files": pins})
    frozen_hash = hashlib.sha256(frozen.read_bytes()).hexdigest()
    manifest = save("score-manifest.json", {"phase": "sample-phase", "phase_manifest": str(frozen), "phase_manifest_sha256": frozen_hash, "runs": rows})
    terminal = save("PHASE-RESULT.json", {"phase": "sample-phase", "manifest_sha256": frozen_hash, "finished_at": "closed-time", "runs": rows})
    admission = save("RUN-ONCE/admission.json", {"manifest_sha256": frozen_hash})
    for path in [manifest, frozen, terminal, admission]: sources[str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    observations = [{"id": row["id"], "errors": ["not launched"], "launch_status": row["launch_status"]} for row in rows]
    if launched: observations[0].update(errors=[], accounting=sample_data["accounting"], delivery={"status": "available"})
    if extra: observations.append({"id": "unexpected", "body": "PRIVATE_UNEXPECTED_BODY"})
    report = save("PRIMARY.json", {"schema": "work-leaf-untracked-read-primary-v1", "phase": "sample-phase", "observations": observations,
        "source_sha256": sources, "integrity_errors": ["phase stopped before admission completed"], "primary": {"do_not_read": "PRIVATE_PRIMARY_ENDPOINT"}})
    return manifest, report


class WrapperTests(unittest.TestCase):
    def test_launched_wrapper_uses_shared_closed_capture_replay(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); manifest, report = phase_fixture(root, launched=True)
            delivery, _ = subject.dependencies()
            with patch.object(subject, "replay_launched_sources", wraps=subject.replay_launched_sources) as replay, patch.object(delivery, "capture_provenance", return_value={"errors": [], "source_sha256": {}}), patch.object(delivery, "measurement_for", return_value=sample()["accounting"]):
                result, _ = subject.build_census(manifest, report, root/"sessions")
            self.assertEqual(replay.call_count, 1)
            self.assertEqual(result["runs"][0]["status"], "complete_descriptive", result["runs"][0]["errors"])
            self.assertEqual(result["runs"][0]["reads"][0]["native_user"]["line"], 2)

    def test_actual_launched_row_receipt_is_independently_hashed_not_required_in_primary_sources(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); manifest, report = phase_fixture(root, launched=True)
            receipt = root/"logs/sample-000.exit.json"
            self.assertNotIn(str(receipt), json.loads(report.read_text())["source_sha256"])
            delivery, _ = subject.dependencies()
            with patch.object(delivery, "capture_provenance", return_value={"errors": [], "source_sha256": {}}), patch.object(delivery, "measurement_for", return_value=sample()["accounting"]):
                result, index = subject.build_census(manifest, report, root/"sessions")
            self.assertEqual(result["runs"][0]["status"], "complete_descriptive", result["runs"][0]["errors"])
            self.assertEqual(len(result["runs"][0]["reads"]), 1)
            self.assertEqual(result["runs"][0]["reads"][0]["native_user"]["status"], "linked")
            self.assertIn(str(receipt), index.hashes)

    def test_recorded_digest_must_be_exact_lowercase_sha256(self):
        for bad in (None, True, 1, "not-a-hash", "A"*64):
            with self.subTest(bad=bad), tempfile.TemporaryDirectory() as directory:
                root = Path(directory); manifest, report = phase_fixture(root)
                value = json.loads(report.read_text()); value["source_sha256"][str(manifest)] = bad
                report.write_text(json.dumps(value))
                with self.assertRaises(ValueError): subject.build_census(manifest, report, root/"sessions")

    def test_null_digest_in_frozen_inventory_is_not_a_fresh_source_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); manifest, report = phase_fixture(root, invalid_extra_pin=True)
            with self.assertRaises(ValueError): subject.build_census(manifest, report, root/"sessions")

    def test_complete_frozen_population_retained_without_zero_fill_or_primary_read(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); manifest, report = phase_fixture(root)
            result, index = subject.build_census(manifest, report, root/"sessions")
            self.assertEqual(len(result["runs"]), 12)
            self.assertTrue(all(row["status"] == "unavailable" for row in result["runs"]))
            self.assertEqual([row["run_id"] for row in result["runs"]], sorted(row["run_id"] for row in result["runs"]))
            self.assertNotIn("PRIVATE_PRIMARY_ENDPOINT", json.dumps(result))
            self.assertIn(str(manifest), index.hashes)

    def test_running_workflow_is_not_a_closed_phase(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); manifest, report = phase_fixture(root, running=True)
            with self.assertRaises(ValueError): subject.build_census(manifest, report, root/"sessions")

    def test_source_hash_change_blocks_publication_before_any_outcome_replay(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); manifest, report = phase_fixture(root)
            with manifest.open("a") as stream: stream.write(" ")
            with self.assertRaises(ValueError): subject.build_census(manifest, report, root/"sessions")

    def test_unexpected_result_identity_is_retained_without_copying_body(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); manifest, report = phase_fixture(root, extra=True)
            result, _ = subject.build_census(manifest, report, root/"sessions")
            self.assertEqual(len(result["runs"]), 13)
            self.assertTrue(result["errors"])
            self.assertNotIn("PRIVATE_UNEXPECTED_BODY", json.dumps(result))


if __name__ == "__main__":
    unittest.main()
