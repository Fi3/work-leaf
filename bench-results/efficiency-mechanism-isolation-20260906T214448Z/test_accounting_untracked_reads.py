"""Offline prospective accounting regressions; no provider invocation."""
import copy
import importlib.util
from pathlib import Path
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("read_accounting", HERE / "accounting_untracked_reads.py")
M = importlib.util.module_from_spec(spec)
spec.loader.exec_module(M)


def usage(i=10, c=2, o=3, r=1):
    return dict(inputTokens=i, cachedInputTokens=c, outputTokens=o,
                reasoningOutputTokens=r, totalTokens=i+o, cacheWriteInputTokens=0)


def frame(method, **extra):
    return {"method": method, "params": {"threadId": "thread", "turnId": "turn", **extra}}


def raw(identity, measured=None):
    return frame("rawResponse/completed", responseId=identity,
                 usage=usage() if measured is None else measured)


def cumulative(total=None, last=None):
    return frame("thread/tokenUsage/updated", tokenUsage={
        "total": usage() if total is None else total,
        "last": usage() if last is None else last})


def scenario(compaction=False, included=False, invalid_last=True):
    clients = [{"id": "start", "method": "turn/start", "params": {"threadId": "thread"}}]
    servers = [{"id": "start", "result": {"turn": {"id": "turn"}}},
               frame("turn/started", turn={"id": "turn"}), raw("ordinary"), cumulative()]
    native = {"records": {"ordinary": {"thread_id": "thread", "turn_id": "turn",
              "usage": M.checked_usage(usage()), "sources": []}}, "compactions": {}}
    if compaction:
        servers += [frame("item/started", item={"type": "contextCompaction", "id": "compact-item"}),
                    raw("compact", usage(20, 4, 5, 0))]
        native["records"]["compact"] = {"thread_id": "thread", "turn_id": "turn",
            "usage": M.checked_usage(usage(20, 4, 5, 0)), "sources": []}
        native["compactions"]["compact"] = {"thread_id": "thread", "turn_id": "turn", "sources": []}
        last = usage(20, 4, 5, 0) if included else usage(0, 0, 0, 0)
        if invalid_last:
            last["totalTokens"] = 7
        servers += [cumulative(usage(30, 6, 8, 1) if included else usage(), last),
                    frame("item/completed", item={"type": "contextCompaction", "id": "compact-item"})]
    servers += [frame("turn/completed", turn={"id": "turn", "status": "completed"})]
    return clients, servers, [], native


class ReconciliationTests(unittest.TestCase):
    def run_case(self, **kw):
        return M.reconcile_stream(*scenario(**kw))

    def test_ordinary_identity_unchanged(self):
        result = self.run_case()
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["omitted_compaction_ids"], [])
        self.assertEqual(result["raw_responses"]["ordinary"]["usage"]["raw_input_plus_output"], 13)

    def test_explicit_omitted_compaction_and_nonadditive_last_are_separate(self):
        result = self.run_case(compaction=True)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["omitted_compaction_ids"], ["compact"])
        self.assertEqual(result["warnings"][0]["classification"], "unusable_compaction_last_metadata")
        self.assertFalse(result["warnings"][0]["establishes_completed_response_usage"])
        self.assertEqual(result["thread_ledger"]["thread"]["corrected"]["raw_input_plus_output"], 38)

    def test_compaction_already_in_cumulative_is_not_added_twice(self):
        result = self.run_case(compaction=True, included=True, invalid_last=False)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["omitted_compaction_ids"], [])
        self.assertEqual(result["thread_ledger"]["thread"]["corrected"]["raw_input_plus_output"], 38)

    def test_shape_without_explicit_compaction_identity_is_rejected(self):
        args = list(scenario(compaction=True)); args[3]["compactions"] = {}
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_compaction_without_complete_lifecycle_is_rejected(self):
        args = list(scenario(compaction=True)); args[1].pop(-2)
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_invalid_last_with_cumulative_advance_is_rejected(self):
        self.assertTrue(self.run_case(compaction=True, included=True)["errors"])

    def test_malformed_completed_usage_never_becomes_warning(self):
        args = list(scenario(compaction=True)); args[1][5]["params"]["usage"]["totalTokens"] = 999
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_nonadditive_total_is_always_rejected(self):
        args = list(scenario()); args[1][3]["params"]["tokenUsage"]["total"]["totalTokens"] = 999
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_cumulative_regression_is_rejected(self):
        args = list(scenario()); args[1].insert(-1, cumulative(usage(1, 0, 1, 0)))
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_ordinary_response_cannot_be_omitted(self):
        args = list(scenario()); args[1][3] = cumulative(usage(0, 0, 0, 0))
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_exact_duplicate_response_counts_once(self):
        args = list(scenario()); args[1].insert(3, copy.deepcopy(args[1][2]))
        result = M.reconcile_stream(*args)
        self.assertEqual(result["errors"], [])
        self.assertEqual(len(result["raw_responses"]), 1)

    def test_conflicting_duplicate_response_is_rejected(self):
        args = list(scenario()); args[1].insert(3, raw("ordinary", usage(11)))
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_bool_and_empty_identifiers_fail_closed(self):
        for identity in (True, 1, ""):
            args = list(scenario()); args[1][2]["params"]["responseId"] = identity
            self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_raw_native_usage_mismatch_fails(self):
        args = list(scenario()); args[3]["records"]["ordinary"]["usage"] = M.checked_usage(usage(99))
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_missing_native_response_is_not_zero(self):
        args = list(scenario()); args[3]["records"] = {}
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_wrong_typed_reply_id_is_rejected(self):
        args = list(scenario()); args[0][0]["id"] = 1; args[1][0]["id"] = True
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_physical_response_locator_index_preserves_duplicates(self):
        args = list(scenario()); args[1].insert(3, copy.deepcopy(args[1][2]))
        self.assertEqual(M.reconcile_stream(*args)["response_server_lines"]["ordinary"], [3, 4])

    def test_two_compactions_in_one_lifecycle_are_ambiguous(self):
        args = list(scenario(compaction=True)); args[1].insert(6, raw("compact-2", usage(20, 4, 5, 0)))
        args[3]["records"]["compact-2"] = copy.deepcopy(args[3]["records"]["compact"])
        args[3]["compactions"]["compact-2"] = copy.deepcopy(args[3]["compactions"]["compact"])
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_invalid_last_never_recovers_an_interrupted_tail(self):
        args = list(scenario(compaction=True)); args[1][-1]["params"]["turn"]["status"] = "interrupted"
        args[0].append({"id": "interrupt", **frame("turn/interrupt")})
        args[2].append({"thread_id": "thread", "turn_id": "turn", "outcome": "forwarded-after-timeout"})
        result = M.reconcile_stream(*args)
        self.assertEqual(result["errors"], [])
        self.assertIsNone(result["gaps"][0]["response_count_upper"])
        self.assertIn("withheld", result["gaps"][0]["reason"])

    def test_cumulative_metadata_requires_accepted_local_turn(self):
        args = list(scenario()); args[1][3]["params"]["turnId"] = "foreign"
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_wrong_native_compaction_turn_fails(self):
        args = list(scenario(compaction=True)); args[3]["compactions"]["compact"]["turn_id"] = "foreign"
        self.assertTrue(M.reconcile_stream(*args)["errors"])

    def test_invalid_last_component_type_is_not_a_warning(self):
        args = list(scenario(compaction=True)); args[1][6]["params"]["tokenUsage"]["last"]["inputTokens"] = True
        result = M.reconcile_stream(*args)
        self.assertTrue(result["errors"])
        self.assertEqual(result["warnings"], [])

    def test_unlaunched_outcome_retained_without_zero_fill(self):
        result = M.audit_run({"run_id": "retained"}, {"files": []}, "/unused")
        self.assertEqual(result["run_id"], "retained")
        self.assertEqual(result["status"], "unknown")
        self.assertIsNone(result["measurement"]["bounds"])

    def test_late_terminal_response_recovery_preserves_grace(self):
        args = late_response()
        result = M.reconcile_stream(*args)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["gaps"], [])
        proof = result["late_terminal_usage_recoveries"][0]
        self.assertEqual(proof["response_id"], "ordinary")
        self.assertEqual(proof["original_grace_outcome"], "forwarded-after-output-resumed")
        self.assertTrue(proof["no_later_generation_or_item_boundary"])
        self.assertEqual(args[2][0]["outcome"], "forwarded-after-output-resumed")

    def test_late_response_does_not_hide_a_new_item_before_terminal(self):
        args = late_response()
        args[1].insert(-1, frame("item/started", item={"type": "reasoning", "id": "new"}))
        result = M.reconcile_stream(*args)
        self.assertEqual(result["late_terminal_usage_recoveries"], [])
        self.assertEqual(len(result["gaps"]), 1)

    def test_stale_last_is_not_late_exact_usage(self):
        args = late_response(); args[1][-2]["params"]["tokenUsage"]["last"] = usage(0, 0, 0, 0)
        result = M.reconcile_stream(*args)
        self.assertEqual(result["late_terminal_usage_recoveries"], [])
        self.assertEqual(len(result["gaps"]), 1)

    def test_late_usage_requires_exact_matching_response_usage(self):
        args = late_response(); args[1][-2]["params"]["tokenUsage"]["last"] = usage(1, 0, 1, 0)
        result = M.reconcile_stream(*args)
        self.assertEqual(result["late_terminal_usage_recoveries"], [])
        self.assertEqual(len(result["gaps"]), 1)


def late_response():
    clients, servers, grace, native = scenario()
    servers.insert(2, frame("item/completed", item={"type": "agentMessage", "id": "directive", "text": "@work-leaf read source.rs"}))
    servers[-1]["params"]["turn"]["status"] = "interrupted"
    clients.append({"id": "interrupt", **frame("turn/interrupt")})
    grace.append({"thread_id": "thread", "turn_id": "turn", "outcome": "forwarded-after-output-resumed"})
    return clients, servers, grace, native


def native_rows():
    u = dict(input_tokens=10, cached_input_tokens=2, output_tokens=3,
             reasoning_output_tokens=1, total_tokens=13, cache_write_input_tokens=0)
    record = dict(thread_id="thread", session_id="thread", turn_id="turn", response_id="compact",
                  usage=u, thread_token_usage=u, turn_token_usage=u)
    rows = [{"type": "session_meta", "payload": {"id": "thread"}},
            {"type": "turn_context", "payload": {"turn_id": "turn", "model": "gpt-5.5", "effort": "xhigh"}},
            {"type": "token_usage_record", "payload": record},
            {"type": "compacted", "payload": {"compaction_response_id": "compact",
                "latest_token_usage_record": copy.deepcopy(record)}}]
    metadata = {"thread_id": "thread", "model": "gpt-5.5", "effort": "xhigh"}
    return rows, metadata


class NativeIdentityTests(unittest.TestCase):
    def run_rows(self, rows, metadata):
        return M.native_ledger(rows, metadata, "gpt-5.5", "xhigh", "/safe/native.jsonl")

    def test_explicit_native_compaction_identity_is_required(self):
        rows, meta = native_rows()
        self.assertIn("compact", self.run_rows(rows, meta)["compactions"])
        del rows[-1]["payload"]["compaction_response_id"]
        with self.assertRaises(ValueError): self.run_rows(rows, meta)

    def test_latest_compaction_usage_must_equal_independent_record(self):
        rows, meta = native_rows(); rows[-1]["payload"]["latest_token_usage_record"]["usage"]["input_tokens"] = 9
        with self.assertRaises(ValueError): self.run_rows(rows, meta)

    def test_all_native_cumulative_fields_are_strict(self):
        rows, meta = native_rows(); rows[2]["payload"]["thread_token_usage"] = dict(rows[2]["payload"]["usage"], total_tokens=99)
        with self.assertRaises(ValueError): self.run_rows(rows, meta)

    def test_model_scope_and_boolean_id_rejected(self):
        rows, meta = native_rows(); rows[1]["payload"]["model"] = "wrong"
        with self.assertRaises(ValueError): self.run_rows(rows, meta)
        rows, meta = native_rows(); rows[0]["payload"]["id"] = True
        with self.assertRaises(ValueError): self.run_rows(rows, meta)

    def test_duplicate_compaction_marker_is_not_an_extra_charge(self):
        rows, meta = native_rows(); rows += copy.deepcopy(rows[2:])
        with self.assertRaises(ValueError): self.run_rows(rows, meta)


if __name__ == "__main__":
    unittest.main()
