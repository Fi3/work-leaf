"""Provider-free regressions for the descriptive one-pair analysis."""

import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import batch_analysis as subject


def usage(input_tokens=100, cached=40, output=10, reasoning=3):
    return {"input_tokens": input_tokens, "cached_input_tokens": cached,
            "output_tokens": output, "reasoning_output_tokens": reasoning,
            "uncached_input_tokens": input_tokens-cached,
            "raw_input_plus_output": input_tokens+output,
            "uncached_input_plus_output": input_tokens-cached+output}


def analysis(missing=0, errors=None):
    return {"capture_complete": not missing, "interrupted_provider_turns": missing,
            "errors": errors if errors is not None else ([
                f"interrupted provider turn has no complete usage: count={missing}"
            ] if missing else []), "invocation_count": 1, "complete_invocation_count": 1,
            "passthrough_invocation_count": 0,
            "usage_scopes": {"total_workflow": usage()},
            "model_strata": [{"model": "test-model", "effort": "test-effort"}]}


def policy():
    return {"model": "test-model", "reasoning_effort": "test-effort",
            "per_response": {"input_upper": 1000, "output_upper": 100,
                             "source_url": "https://example.invalid/model",
                             "scope": "model_limit_not_response_count_proof"}}


def rpc_stream():
    clients = [{"id": "1", "method": "turn/start", "params": {"threadId": "thread"}},
               {"id": "2", "method": "turn/interrupt", "params": {
                   "threadId": "thread", "turnId": "turn"}}]
    servers = [{"id": "1", "result": {"turn": {"id": "turn"}}},
               {"method": "turn/started", "params": {"threadId": "thread",
                   "turn": {"id": "turn", "status": "inProgress"}}},
               {"method": "turn/completed", "params": {"threadId": "thread",
                   "turn": {"id": "turn", "status": "interrupted"}}}]
    return clients, servers


def isolated_tail():
    c, s = rpc_stream()
    items = [{"type": "userMessage", "id": "user"},
             {"type": "reasoning", "id": "thought"},
             {"type": "agentMessage", "id": "answer", "text": "@work-leaf read file.rs"}]
    events = []
    for item in items:
        for method in ["item/started", "item/completed"]:
            events.append({"method": method, "params": {
                "threadId": "thread", "turnId": "turn", "item": item}})
    s[2:2] = events
    s.insert(-1, {"method": "item/started", "params": {"threadId": "thread",
                 "turnId": "turn", "item": {"type": "reasoning", "id": "unfinished"}}})
    grace = [{"thread_id": "thread", "turn_id": "turn", "outcome": "forwarded-after-output-resumed"}]
    return c, s, grace


class AccountingTests(unittest.TestCase):
    def test_raw_primary_and_secondary_are_separate_without_precision_gate(self):
        d = {"raw_input_plus_output": {"lower": 200, "upper": 200},
             "uncached_input_plus_output": {"lower": 80, "upper": 80}}
        w = {"raw_input_plus_output": {"lower": 100, "upper": 180},
             "uncached_input_plus_output": {"lower": 90, "upper": 170}}
        result = subject.compare_bounds(d, w)
        self.assertEqual(result["primary_metric"], "raw_input_plus_output")
        self.assertEqual(result["raw_input_plus_output"]["saving_percent"],
                         {"lower": 10.0, "upper": 50.0})
        self.assertEqual(result["raw_input_plus_output"]["width_percentage_points"], 40)
        self.assertNotIn("precision_pass", json.dumps(result))
        self.assertLess(result["uncached_input_plus_output"]["saving_percent"]["upper"], 0)

    def test_uncertain_direct_denominator_uses_both_endpoints(self):
        self.assertEqual(subject.saving_interval({"lower": 100, "upper": 200},
                         {"lower": 40, "upper": 80})["saving_percent"],
                         {"lower": 20.0, "upper": 80.0})

    def test_invalid_or_nonpositive_denominators_are_rejected(self):
        for bounds in [{"lower": 0, "upper": 10}, {"lower": -1, "upper": 10},
                       {"lower": 3, "upper": 2}, {"lower": True, "upper": 10},
                       {"lower": 1, "upper": float("inf")}]:
            with self.subTest(bounds=bounds), self.assertRaises(ValueError):
                subject.saving_interval(bounds, {"lower": 1, "upper": 2})

    def test_missing_workflow_upper_is_not_a_finite_saving_interval(self):
        result = subject.saving_interval({"lower": 200, "upper": 200},
                                        {"lower": 100, "upper": None})
        self.assertIsNone(result["saving_percent"]["lower"])
        self.assertEqual(result["saving_percent"]["upper"], 50)
        self.assertEqual(result["status"], "unbounded")

    def test_exact_measurement_needs_no_missing_response_assumption(self):
        result = subject.accounting(analysis(), {"gaps": [], "errors": []}, policy())
        self.assertEqual(result["status"], "exact")
        self.assertEqual(result["bounds"]["raw_input_plus_output"], {"lower": 110, "upper": 110})

    def test_missing_turn_never_becomes_one_missing_response_by_assumption(self):
        inv = {"gaps": [{"thread_id": "thread", "turn_id": "turn",
                         "response_count_upper": None}], "errors": []}
        result = subject.accounting(analysis(1), inv, policy())
        self.assertEqual(result["status"], "unbounded_accounting_gap")
        self.assertIsNone(result["bounds"]["raw_input_plus_output"]["upper"])
        self.assertEqual(result["recorded_usage"], usage())

    def test_nonaccounting_error_is_not_waived(self):
        for extra in ["provider model mismatch", "raw stream SHA-256 mismatch",
                      "unclassified process", "response identity conflict"]:
            a = analysis(1); a["errors"].append(extra)
            result = subject.accounting(a, {"gaps": [{}], "errors": []}, policy())
            self.assertEqual(result["status"], "ineligible")

    def test_profile_arithmetic_and_gap_inventory_mismatches_are_ineligible(self):
        cases = [analysis(), analysis(), analysis(1)]
        cases[0]["model_strata"][0]["effort"] = "different"
        cases[1]["usage_scopes"]["total_workflow"]["raw_input_plus_output"] = 999
        for a in cases:
            self.assertEqual(subject.accounting(a, {"gaps": [], "errors": []}, policy())["status"],
                             "ineligible")


class InventoryTests(unittest.TestCase):
    def test_foreign_completion_cannot_clear_the_active_turn(self):
        c, s = rpc_stream()
        s.insert(2, {"method": "turn/completed", "params": {"threadId": "thread",
                    "turn": {"id": "foreign", "status": "completed"}}})
        self.assertTrue(subject.inventory(c, s)["errors"])

    def test_one_response_bound_requires_the_isolated_tail_predicates(self):
        c, s, g = isolated_tail(); inv = subject.inventory(c, s, g)
        self.assertEqual(inv["gaps"][0]["response_count_upper"], 1)
        result = subject.accounting(analysis(1), inv, policy())
        self.assertEqual(result["status"], "bounded")
        self.assertEqual(result["bounds"]["raw_input_plus_output"], {"lower": 110, "upper": 1210})

    def test_tool_boundary_additional_item_or_unpaired_item_prevents_tail_proof(self):
        for mutation in ["tool", "extra_post_item", "unpaired"]:
            c, s, g = isolated_tail()
            if mutation == "tool": s[2]["params"]["item"]["type"] = "commandExecution"
            elif mutation == "extra_post_item": s.insert(-1, copy.deepcopy(s[-2]))
            else: s.pop(2)
            self.assertIsNone(subject.inventory(c, s, g)["gaps"][0]["response_count_upper"], mutation)

    def test_forged_exact_grace_with_stale_or_foreign_usage_is_rejected(self):
        c, s, g = isolated_tail(); g[0]["outcome"] = "forwarded-after-exact-usage"
        self.assertTrue(subject.inventory(c, s, g)["errors"])

    def test_interrupted_gap_is_tied_to_local_turn_and_has_unknown_cardinality(self):
        c, s = rpc_stream(); result = subject.inventory(c, s)
        self.assertEqual(len(result["gaps"]), 1)
        self.assertEqual(result["gaps"][0]["thread_id"], "thread")
        self.assertIsNone(result["gaps"][0]["response_count_upper"])

    def test_exact_raw_record_from_other_turn_does_not_clear_gap(self):
        c, s = rpc_stream()
        c.append({"id": "3", "method": "turn/start", "params": {"threadId": "thread"}})
        s.extend([{"id": "3", "result": {"turn": {"id": "second"}}},
                  {"method": "rawResponse/completed", "params": {"threadId": "thread",
                    "turnId": "second", "responseId": "response", "usage": {
                    "inputTokens": 20, "cachedInputTokens": 10, "outputTokens": 2,
                    "reasoningOutputTokens": 0, "totalTokens": 22}}}])
        result = subject.inventory(c, s)
        self.assertEqual(len(result["gaps"]), 1)
        self.assertEqual(result["raw_response_count"], 1)

    def test_stale_cumulative_usage_cannot_clear_interruption(self):
        c, s = rpc_stream()
        event = {"method": "thread/tokenUsage/updated", "params": {"threadId": "thread",
            "turnId": "turn", "tokenUsage": {"total": {"inputTokens": 20,
            "cachedInputTokens": 10, "outputTokens": 2, "reasoningOutputTokens": 0,
            "totalTokens": 22}, "last": {"inputTokens": 20, "cachedInputTokens": 10,
            "outputTokens": 2, "reasoningOutputTokens": 0, "totalTokens": 22}}}}
        s.insert(0, event); s.append(copy.deepcopy(event))
        self.assertEqual(len(subject.inventory(c, s)["gaps"]), 1)

    def test_unknown_turn_and_conflicting_rpc_id_are_errors(self):
        c, s = rpc_stream(); c[-1]["params"]["turnId"] = "foreign"
        self.assertTrue(subject.inventory(c, s)["errors"])
        c, s = rpc_stream(); c.append(copy.deepcopy(c[0]))
        self.assertTrue(subject.inventory(c, s)["errors"])

    def test_typed_rpc_ids_do_not_collide_and_server_requests_are_not_replies(self):
        c, s = rpc_stream()
        s.insert(0, {"id": "1", "method": "request/permission", "params": {}})
        c.append({"id": 1, "method": "turn/start", "params": {"threadId": "other"}})
        s.append({"id": 1, "result": {"turn": {"id": "other-turn"}}})
        self.assertFalse(subject.inventory(c, s)["errors"])


class ScoringTests(unittest.TestCase):
    def test_unlaunched_rows_do_not_count_as_benchmark_observations(self):
        rows = [{"condition": "direct", "launch_status": "completed", "started_at": "time"},
                {"condition": "work-leaf", "launch_status": "not_launched_after_signal", "started_at": None}]
        self.assertEqual(subject.launched_counts(rows), {"direct": 1, "work-leaf": 0})

    def test_generation_activity_reuses_frozen_hash_checked_rollout_helper(self):
        class FakeHelper:
            def rollout_activity(self, path, captured, condition):
                return {"usage_changes": 7, "usage_changes_by_stage": {"review": 7},
                        "actions_by_stage": {"review": {"tool": 2}}}
        result = subject.stage_activity(FakeHelper(), Path("analysis.json"), {}, "direct")
        self.assertEqual(result["reported_usage_advances"], 7)
        self.assertEqual(result["reported_usage_advances_by_stage"], {"review": 7})
        self.assertFalse(result["exact_model_call_count"])

    def test_generation_activity_failure_is_unavailable_not_zero(self):
        class BrokenHelper:
            def rollout_activity(self, *args): raise OSError("rollout missing")
        result = subject.stage_activity(BrokenHelper(), Path("analysis.json"), {}, "direct")
        self.assertEqual(result["status"], "unavailable")
        self.assertIsNone(result["reported_usage_advances"])

    def test_scorer_configuration_is_hash_pinned(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d)/"score.py"; path.write_text("FIXTURES = {}\n")
            with self.assertRaises(ValueError):
                subject.load_scorer({"scorer": {"path": str(path), "sha256": "wrong"},
                                     "fixtures": []}, Path(d))

    def test_all_quality_outcomes_are_retained_without_provider_execution(self):
        class FakeScorer:
            FIXTURES = {"feature": ("fixture", "test")}
            def score_entry(self, entry, manifest, work_root, timeout):
                if entry["id"] == "broken": raise OSError("report missing")
                return {"id": entry["id"], "checks": {"feature": "fail"}, "completed_features": 0}
        manifest = {"runs": [{"id": "saved"}, {"id": "broken"}]}
        with tempfile.TemporaryDirectory() as d, patch.object(subject, "load_scorer", return_value=FakeScorer()):
            rows = subject.score_all(manifest, {}, Path(d), Path(d), 1)
        self.assertEqual([r["id"] for r in rows], ["saved", "broken"])
        self.assertEqual(rows[0]["checks"], {"feature": "fail"})
        self.assertIn("scoring_error", rows[1])


if __name__ == "__main__":
    unittest.main()
