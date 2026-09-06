"""Offline exact retained-item attribution regressions."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("audit_input_attribution", Path(__file__).with_name("audit_input_attribution.py"))
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)


def counts(i=0, c=0, o=0):
    return {"input_tokens": i, "cached_tokens": c, "output_tokens": o, "cache_write_tokens": 0}


def fixture():
    def response(identity):
        return {"method": "rawResponse/completed", "params": {
            "threadId": "thread", "turnId": "turn", "responseId": identity,
            "usage": {"inputTokens": 15, "cachedInputTokens": 10, "outputTokens": 5,
                      "reasoningOutputTokens": 3, "cacheWriteInputTokens": 0, "totalTokens": 20},
            "usageMetadata": {"metadata": {"attribution": {
                "items": {"context": {**counts(10, 10), "content": [counts(10, 10)]},
                          "reason": counts(o=3), "action": counts(o=2)},
                "request_fields": {"tools": counts(5)}}}}}}
    native = {
        ("thread", "context"): {"kind": "message", "role": "user", "source": "native.jsonl", "line": 1},
        ("thread", "reason"): {"kind": "reasoning", "turn_id": "turn", "source": "native.jsonl", "line": 2},
        ("thread", "action"): {"kind": "message", "role": "assistant", "turn_id": "turn", "directive_names": ["done"], "source": "native.jsonl", "line": 3},
    }
    ledger = {"r1": {"thread_id": "thread", "turn_id": "turn", "usage": {
        "input_tokens": 15, "cached_input_tokens": 10, "output_tokens": 5, "reasoning_output_tokens": 3}}}
    return [response("r1")], native, ledger


class AttributionTests(unittest.TestCase):
    def extract(self, rows=None, native=None, ledger=None):
        values, items, records = fixture()
        return AUDIT.extract_records(values if rows is None else rows,
                                     items if native is None else native,
                                     records if ledger is None else ledger)

    def test_exact_reconciliation_does_not_double_count_nested_content(self):
        result = self.extract()
        self.assertEqual(result["errors"], [])
        record = result["responses"][0]
        self.assertEqual(record["residual"], dict.fromkeys(AUDIT.FIELDS, 0))
        self.assertEqual(record["request_field_usage"]["input_tokens"], 5)
        self.assertEqual(record["input_items"][0]["input_tokens"], 10)
        self.assertEqual(record["output_items"][1]["item_id"], "action")
        self.assertEqual(record["output_items"][1]["action"]["directive_names"], ["done"])

    def test_repeated_input_charge_is_keyed_by_thread_and_item(self):
        rows, native, ledger = fixture()
        second = copy.deepcopy(rows[0]); second["params"]["responseId"] = "r2"
        for original in ("reason", "action"):
            item = second["params"]["usageMetadata"]["metadata"]["attribution"]["items"].pop(original)
            second["params"]["usageMetadata"]["metadata"]["attribution"]["items"][original + "2"] = item
            native[("thread", original + "2")] = copy.deepcopy(native[("thread", original)])
        rows.append(second); ledger["r2"] = copy.deepcopy(ledger["r1"])
        result = self.extract(rows, native, ledger)
        self.assertEqual(result["errors"], [])
        repeated = result["repeated_input_items"][0]
        self.assertEqual((repeated["thread_id"], repeated["item_id"]), ("thread", "context"))
        self.assertEqual(repeated["response_charge_count"], 2)
        self.assertEqual(repeated["input_tokens"], 20)
        self.assertEqual(repeated["repeated_after_first_input_tokens"], 10)

    def test_identical_response_duplicate_is_not_recharged(self):
        rows, native, ledger = fixture()
        result = self.extract(rows + copy.deepcopy(rows), native, ledger)
        self.assertEqual(result["errors"], [])
        self.assertEqual(len(result["responses"]), 1)
        self.assertEqual(result["duplicate_responses"], 1)

    def test_conflicting_response_identity_is_unknown(self):
        rows, native, ledger = fixture()
        duplicate = copy.deepcopy(rows[0])
        duplicate["params"]["usageMetadata"]["metadata"]["attribution"]["items"]["context"]["input_tokens"] = 11
        result = self.extract(rows + [duplicate], native, ledger)
        self.assertTrue(result["errors"])

    def test_signed_residual_reports_overcount_and_undercount(self):
        for amount, residual in ((11, -1), (9, 1)):
            rows, native, ledger = fixture()
            rows[0]["params"]["usageMetadata"]["metadata"]["attribution"]["request_fields"]["tools"]["input_tokens"] = amount - 5
            result = self.extract(rows, native, ledger)
            with self.subTest(amount=amount):
                self.assertEqual(result["responses"][0]["residual"]["input_tokens"], residual)
                self.assertTrue(result["errors"])

    def test_partial_metadata_is_unknown_not_zero(self):
        for missing in ("items", "request_fields"):
            rows, native, ledger = fixture()
            del rows[0]["params"]["usageMetadata"]["metadata"]["attribution"][missing]
            result = self.extract(rows, native, ledger)
            with self.subTest(missing=missing):
                self.assertTrue(result["errors"])
                self.assertEqual(len(result["responses"]), 1)
                self.assertEqual(result["responses"][0]["status"], "unknown")

    def test_boolean_ids_and_usage_are_rejected(self):
        for key in ("threadId", "turnId", "responseId"):
            rows, native, ledger = fixture(); rows[0]["params"][key] = True
            with self.subTest(key=key):
                self.assertTrue(self.extract(rows, native, ledger)["errors"])
        rows, native, ledger = fixture()
        rows[0]["params"]["usageMetadata"]["metadata"]["attribution"]["items"]["context"]["input_tokens"] = True
        self.assertTrue(self.extract(rows, native, ledger)["errors"])

    def test_unmatched_output_identity_does_not_guess_reasoning(self):
        rows, native, ledger = fixture(); del native[("thread", "reason")]
        result = self.extract(rows, native, ledger)
        self.assertTrue(result["errors"])
        self.assertIsNone(result["responses"][0]["residual"]["reasoning_output_tokens"])

    def test_scope_and_native_usage_conflict_rejected(self):
        for field, value in (("thread_id", "other"), ("turn_id", "other"), ("usage", {})):
            rows, native, ledger = fixture(); ledger["r1"][field] = value
            with self.subTest(field=field):
                self.assertTrue(self.extract(rows, native, ledger)["errors"])

    def test_unknown_output_kind_is_not_assumed_nonreasoning(self):
        rows, native, ledger = fixture(); native[("thread", "reason")]["kind"] = "future_kind"
        result = self.extract(rows, native, ledger)
        self.assertTrue(result["errors"])
        self.assertIsNone(result["responses"][0]["residual"]["reasoning_output_tokens"])

    def test_generated_message_output_requires_assistant_role(self):
        for role in (None, "user", "system", "tool", "unknown"):
            rows, native, ledger = fixture()
            if role is None:
                del native[("thread", "action")]["role"]
            else:
                native[("thread", "action")]["role"] = role
            with self.subTest(role=role):
                self.assertTrue(self.extract(rows, native, ledger)["errors"])

    def test_no_content_bodies_in_export(self):
        rows, native, ledger = fixture()
        native[("thread", "action")]["content"] = "SECRET_CONTENT_MUST_NOT_EXPORT"
        self.assertNotIn("SECRET_CONTENT_MUST_NOT_EXPORT", json.dumps(self.extract(rows, native, ledger)))

    def test_same_item_text_id_in_different_threads_is_not_combined(self):
        rows, native, ledger = fixture()
        other = copy.deepcopy(rows[0]); other["params"].update(threadId="other", responseId="r2")
        for (thread, item_id), item in list(native.items()):
            native[("other", item_id)] = copy.deepcopy(item)
        ledger["r2"] = {**copy.deepcopy(ledger["r1"]), "thread_id": "other"}
        result = self.extract(rows + [other], native, ledger)
        self.assertEqual(result["errors"], [])
        self.assertEqual(len(result["repeated_input_items"]), 2)
        self.assertTrue(all(row["response_charge_count"] == 1 for row in result["repeated_input_items"]))

    def test_output_item_cannot_be_assigned_to_two_response_ids(self):
        rows, native, ledger = fixture()
        other = copy.deepcopy(rows[0]); other["params"]["responseId"] = "r2"
        ledger["r2"] = copy.deepcopy(ledger["r1"])
        result = self.extract(rows + [other], native, ledger)
        self.assertTrue(result["errors"])

    def test_native_boolean_usage_cannot_equal_recorded_integer(self):
        rows, native, ledger = fixture()
        rows[0]["params"]["usage"]["reasoningOutputTokens"] = 1
        rows[0]["params"]["usage"]["outputTokens"] = 3
        rows[0]["params"]["usage"]["totalTokens"] = 18
        rows[0]["params"]["usageMetadata"]["metadata"]["attribution"]["items"]["reason"]["output_tokens"] = 1
        ledger["r1"]["usage"].update(reasoning_output_tokens=True, output_tokens=3)
        self.assertTrue(self.extract(rows, native, ledger)["errors"])

    def test_unknown_metadata_and_negative_counts_remain_unknown(self):
        for value in (None, [], "unknown"):
            rows, native, ledger = fixture(); rows[0]["params"]["usageMetadata"] = value
            with self.subTest(value=value):
                self.assertTrue(self.extract(rows, native, ledger)["errors"])
        rows, native, ledger = fixture()
        rows[0]["params"]["usageMetadata"]["metadata"]["attribution"]["items"]["action"]["output_tokens"] = -2
        self.assertTrue(self.extract(rows, native, ledger)["errors"])

    def test_native_item_export_preserves_identity_without_message_body(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "native.jsonl"
            rows = [{"type": "turn_context", "payload": {"turn_id": "turn"}},
                    {"type": "response_item", "payload": {"id": "message", "type": "message", "role": "assistant",
                     "content": [{"text": "PRIVATE_BODY\n@work-leaf done"}]}}]
            data = "".join(json.dumps(row) + "\n" for row in rows).encode(); path.write_bytes(data)
            sources = {str(path): AUDIT.digest(data)}
            base = {"threads": [{"thread_id": "thread", "source": str(path), "errors": [], "records": {}}]}
            items, _, errors = AUDIT.native_item_inventory(base, sources)
            self.assertEqual(errors, [])
            self.assertEqual(items[("thread", "message")]["directive_names"], ["done"])
            self.assertNotIn("PRIVATE_BODY", json.dumps(list(items.values())))
            sources[str(path)] = "0" * 64
            _, _, errors = AUDIT.native_item_inventory(base, sources)
            self.assertTrue(errors)

    def test_exact_native_input_projection_is_safe_and_optional(self):
        rows, native, ledger = fixture()
        native[("thread", "context")].update(kind="function_call_output", call_id="call-exact",
                                              output="PRIVATE_INPUT_BODY", arguments="PRIVATE_ARGUMENTS")
        result = self.extract(rows, native, ledger)
        item = result["repeated_input_items"][0]
        self.assertEqual(item["native_link"]["status"], "linked")
        self.assertEqual(item["native_link"]["metadata"]["call_id"], "call-exact")
        self.assertEqual(item["native_link"]["metadata"]["kind"], "function_call_output")
        self.assertNotIn("PRIVATE_INPUT_BODY", json.dumps(result))
        self.assertNotIn("PRIVATE_ARGUMENTS", json.dumps(result))
        self.assertEqual(result["responses"][0]["input_items"][0]["native_link_status"], "linked")
        del native[("thread", "context")]
        result = self.extract(rows, native, ledger)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["status"], "exact_observed_attribution")
        self.assertEqual(result["repeated_input_items"][0]["native_link"]["status"], "unlinked")

    def test_native_input_link_never_uses_another_thread_or_item_prefix(self):
        rows, native, ledger = fixture()
        native[("other", "context")] = native.pop(("thread", "context"))
        native[("thread", "context-same-prefix")] = copy.deepcopy(native[("other", "context")])
        native[("thread", "context-same-prefix")].update(call_id="context", content="context")
        result = self.extract(rows, native, ledger)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["repeated_input_items"][0]["native_link"]["status"], "unlinked")

    def test_missing_capture_manifest_row_remains_visible(self):
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            manifest.write_text(json.dumps({"model": "gpt-5.5", "reasoning_effort": "xhigh", "runs": [
                {"id": "failed", "condition": "control", "launch_status": "failed"},
                {"id": "unlaunched", "condition": "variant", "launch_status": "not_launched"}]}))
            result = AUDIT.audit_manifest(manifest, Path(directory))
        self.assertEqual([row["run_id"] for row in result["runs"]], ["failed", "unlaunched"])
        self.assertTrue(all(row["status"] == "unknown" for row in result["runs"]))
        self.assertIsNone(result["adjusted_primary_total"])


if __name__ == "__main__":
    unittest.main()
