"""Provider-free supplemental native-accounting audit regressions."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("audit_compaction", Path(__file__).with_name("audit_compaction.py"))
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)


def usage(i=100, c=80, o=10, r=4):
    return dict(input_tokens=i, cached_input_tokens=c, output_tokens=o,
                reasoning_output_tokens=r, total_tokens=i + o, cache_write_input_tokens=0)


def fixture():
    first, compact = usage(), usage(200, 190, 20, 0)
    total = usage(300, 270, 30, 4)
    def record(identity, value, cumulative):
        return {"type": "token_usage_record", "payload": {
            "thread_id": "thread", "session_id": "thread", "turn_id": "turn",
            "root_turn_id": "turn", "response_id": identity, "usage": value,
            "turn_token_usage": cumulative, "thread_token_usage": cumulative}}
    rows = [{"type": "session_meta", "payload": {"id": "thread", "session_id": "thread"}},
            {"type": "turn_context", "payload": {"turn_id": "turn", "model": "gpt-5.5", "effort": "xhigh"}},
            record("ordinary", first, first), record("compact", compact, total),
            {"type": "compacted", "payload": {"message": "irrelevant text"}}]
    meta = {"thread_id": "thread", "model": "gpt-5.5", "effort": "xhigh",
            "primary": True, "visible": True, "descendant": False}
    return rows, meta


class NativeAuditTests(unittest.TestCase):
    def audit(self, rows=None):
        original, meta = fixture()
        return AUDIT.audit_rollout(original if rows is None else rows, meta, "gpt-5.5", "xhigh")

    def test_counts_unique_responses_and_links_only_adjacent_marker(self):
        result = self.audit()
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["native_usage"]["input_tokens"], 300)
        self.assertEqual(result["native_usage"]["raw_input_plus_output"], 330)
        self.assertEqual(result["compaction_markers"][0]["adjacent_native_response_id"], "compact")
        self.assertEqual(result["response_count"], 2)

    def test_duplicate_identical_identity_is_not_an_extra_call(self):
        rows, _ = fixture()
        rows.insert(3, copy.deepcopy(rows[2]))
        result = self.audit(rows)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["response_count"], 2)
        self.assertEqual(result["duplicate_response_records"], 1)

    def test_conflicting_identity_is_unknown(self):
        rows, _ = fixture()
        bad = copy.deepcopy(rows[2])
        bad["payload"]["usage"] = usage(101)
        rows.insert(3, bad)
        self.assertTrue(any("conflicting response" in e for e in self.audit(rows)["errors"]))

    def test_marker_without_adjacent_usage_does_not_guess(self):
        rows, _ = fixture()
        rows.insert(4, {"type": "event_msg", "payload": {"type": "task_complete"}})
        result = self.audit(rows)
        self.assertIsNone(result["compaction_markers"][0]["adjacent_native_response_id"])
        self.assertTrue(any("unlinked compaction" in e for e in result["errors"]))

    def test_bad_thread_model_or_usage_rejected(self):
        for field, value in (("thread_id", "foreign"), ("session_id", "foreign"), ("turn_id", "unknown")):
            rows, _ = fixture()
            rows[2]["payload"][field] = value
            with self.subTest(field=field):
                self.assertTrue(self.audit(rows)["errors"])
        rows, _ = fixture()
        rows[1]["payload"]["model"] = "different-model"
        self.assertTrue(self.audit(rows)["errors"])
        for field, value in (("input_tokens", True), ("cached_input_tokens", 101),
                             ("output_tokens", -1), ("total_tokens", 999), ("cache_write_input_tokens", 1)):
            rows, _ = fixture()
            rows[2]["payload"]["usage"][field] = value
            with self.subTest(field=field):
                self.assertTrue(self.audit(rows)["errors"])

    def test_native_cumulative_must_match_unique_prefix(self):
        rows, _ = fixture()
        rows[3]["payload"]["thread_token_usage"] = usage()
        self.assertTrue(any("native cumulative" in e for e in self.audit(rows)["errors"]))

    def test_unknown_native_payload_shape_is_reported_not_crashed(self):
        rows, _ = fixture()
        rows[2]["payload"] = "unsupported payload"
        self.assertTrue(self.audit(rows)["errors"])

    def test_native_identity_values_must_be_strings_without_bool_integer_aliasing(self):
        rows, _ = fixture()
        rows[1]["payload"]["turn_id"] = 1
        for row in rows[2:4]:
            row["payload"]["turn_id"] = True
        self.assertTrue(self.audit(rows)["errors"])
        for field in ("response_id", "thread_id", "session_id", "turn_id"):
            rows, _ = fixture()
            rows[2]["payload"][field] = 1
            with self.subTest(field=field):
                self.assertTrue(self.audit(rows)["errors"])
        rows, meta = fixture()
        meta["thread_id"] = 1
        rows[0]["payload"].update(id=True, session_id=True)
        for row in rows[2:4]:
            row["payload"].update(thread_id=True, session_id=True)
        self.assertTrue(AUDIT.audit_rollout(rows, meta, "gpt-5.5", "xhigh")["errors"])


class ArtifactAuditTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.sessions = self.root / "sessions"
        self.sessions.mkdir()
        self.artifact = self.root / "artifact"
        self.observation = self.artifact / "observation"
        self.observation.mkdir(parents=True)
        self.rows, self.meta = fixture()
        self.native = self.sessions / "rollout.jsonl"
        self.native.write_text("".join(json.dumps(row) + "\n" for row in self.rows))
        self.meta.update(source_relative_path="rollout.jsonl", source_sha256=hashlib.sha256(self.native.read_bytes()).hexdigest())
        self.write_meta()
        self.analysis = {"capture_complete": True, "threads": [{"thread_id": "thread", "usage": usage(), "primary": True, "visible": True}], "session_only_threads": []}
        self.write_analysis()
        self.run = {"id": "run", "condition": "any-arm", "artifact": str(self.artifact), "launch_status": "completed"}

    def write_meta(self):
        (self.observation / "rollout-metadata.jsonl").write_text(json.dumps(self.meta) + "\n")

    def write_analysis(self):
        (self.observation / "analysis.json").write_text(json.dumps(self.analysis))

    def raw(self, identities):
        path = self.observation / "app-server" / "invocation" / "server-to-client.raw"
        path.parent.mkdir(parents=True, exist_ok=True)
        mapping = dict(input_tokens="inputTokens", cached_input_tokens="cachedInputTokens", output_tokens="outputTokens", reasoning_output_tokens="reasoningOutputTokens", total_tokens="totalTokens", cache_write_input_tokens="cacheWriteInputTokens")
        payloads = [r["payload"] for r in self.rows if r["type"] == "token_usage_record" and r["payload"]["response_id"] in identities]
        path.write_text("".join(json.dumps({"method": "rawResponse/completed", "params": {"threadId": p["thread_id"], "turnId": p["turn_id"], "responseId": p["response_id"], "usage": {mapping[k]: v for k, v in p["usage"].items()}}}) + "\n" for p in payloads))

    def audit(self):
        return AUDIT.audit_run(self.run, "gpt-5.5", "xhigh", self.sessions)

    def test_unmatched_native_usage_is_separate_not_automatic_retotal(self):
        self.raw({"ordinary"})
        result = self.audit()
        self.assertEqual(result["threads"][0]["native_only_response_ids"], ["compact"])
        self.assertEqual(result["threads"][0]["native_only_usage"]["raw_input_plus_output"], 220)
        self.assertEqual(result["threads"][0]["native_minus_observer"]["input_tokens"], 200)
        self.assertIsNone(result["adjusted_primary_total"])
        self.assertFalse(result["whole_workflow_coverage_established"])

    def test_compaction_raw_coverage_requires_same_identity_and_usage(self):
        self.raw({"ordinary", "compact"})
        result = self.audit()
        self.assertEqual(result["threads"][0]["compaction_markers"][0]["raw_identity_match"], True)
        self.assertEqual(result["threads"][0]["native_only_response_ids"], [])
        self.assertFalse(result["whole_workflow_coverage_established"])

    def test_no_app_capture_is_not_evidence_of_zero_usage(self):
        result = self.audit()
        self.assertEqual(result["app_raw_status"], "unavailable")
        self.assertEqual(result["native_response_count"], 2)
        self.assertFalse(result["whole_workflow_coverage_established"])

    def test_reports_counter_gap_and_unexercised_compaction_separately(self):
        self.raw({"ordinary", "compact"})
        result = self.audit()
        self.assertEqual(result["native_minus_observer"]["raw_input_plus_output"], 220)
        self.assertEqual(result["compaction_raw_coverage"], "adjacent_candidates_identity_matched")
        self.assertEqual(result["observed_counter_scope"], "different")

    def test_empty_inventory_stays_unknown(self):
        self.analysis["threads"] = []
        self.write_analysis()
        (self.observation / "rollout-metadata.jsonl").write_text("")
        self.assertEqual(self.audit()["status"], "unknown")

    def test_unknown_observer_thread_row_retains_unknown_run(self):
        self.analysis["threads"] = ["unsupported-thread-row"]
        self.write_analysis()
        result = self.audit()
        self.assertEqual(result["status"], "unknown")
        self.assertEqual(result["run_id"], "run")
        self.assertTrue(result["errors"])

    def test_raw_response_identity_types_cannot_join_as_equal_values(self):
        for field in ("responseId", "threadId", "turnId"):
            self.raw({"ordinary", "compact"})
            path = self.observation / "app-server" / "invocation" / "server-to-client.raw"
            rows = [json.loads(line) for line in path.read_text().splitlines()]
            rows[0]["params"][field] = 1
            path.write_text("".join(json.dumps(row) + "\n" for row in rows))
            with self.subTest(field=field):
                self.assertEqual(self.audit()["status"], "unknown")

    def test_invalid_source_hash_or_path_never_trusted(self):
        for key, value in (("source_sha256", "0" * 64), ("source_relative_path", "../outside.jsonl")):
            original = self.meta[key]
            self.meta[key] = value
            self.write_meta()
            with self.subTest(key=key):
                self.assertTrue(self.audit()["errors"])
            self.meta[key] = original

    def test_missing_observer_thread_is_retained_unknown(self):
        self.analysis["threads"].append({"thread_id": "missing", "usage": usage()})
        self.write_analysis()
        result = self.audit()
        self.assertIn("missing", result["observer_threads_without_native_source"])
        self.assertFalse(result["whole_workflow_coverage_established"])

    def test_all_manifest_outcomes_retained_and_sources_hashed(self):
        manifest = self.root / "manifest.json"
        manifest.write_text(json.dumps({"model": "gpt-5.5", "reasoning_effort": "xhigh", "runs": [self.run, {"id": "unlaunched", "launch_status": "not_launched", "condition": "variant"}]}))
        result = AUDIT.audit_manifest(manifest, self.sessions)
        self.assertEqual([r["run_id"] for r in result["runs"]], ["run", "unlaunched"])
        self.assertEqual(result["runs"][1]["status"], "unknown")
        self.assertTrue(all(len(row["sha256"]) == 64 for row in result["sources"]))


if __name__ == "__main__":
    unittest.main()
