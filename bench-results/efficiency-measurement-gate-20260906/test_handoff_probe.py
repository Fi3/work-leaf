#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path
import time


STUDY = Path(__file__).resolve().parent
USAGE_FIELDS = (
    "inputTokens",
    "cachedInputTokens",
    "outputTokens",
    "reasoningOutputTokens",
    "totalTokens",
)


def load_module():
    specification = importlib.util.spec_from_file_location(
        "handoff_probe", STUDY / "handoff_probe.py"
    )
    if specification is None or specification.loader is None:
        raise RuntimeError("cannot load the subscription handoff probe")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def usage(input_tokens=12, cached_tokens=7, output_tokens=5, reasoning_tokens=3):
    return {
        "inputTokens": input_tokens,
        "cachedInputTokens": cached_tokens,
        "outputTokens": output_tokens,
        "reasoningOutputTokens": reasoning_tokens,
        "totalTokens": input_tokens + output_tokens,
    }


class HandoffProbeTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.probe = load_module()

    def test_subscription_account_accepts_only_explicit_chatgpt_account_type(self):
        self.assertTrue(self.probe.subscription_account({"account": {"type": "chatgpt"}}))
        for response in (
            None,
            {},
            {"account": None},
            {"account": {}},
            {"account": "chatgpt"},
            {"account": {"type": "apiKey"}},
            {"account": {"type": "chatgptAuthTokens"}},
            {"account": {"type": "ChatGPT"}},
            {"account": {"type": True}},
            {"type": "chatgpt"},
            {"result": {"account": {"type": "chatgpt"}}},
        ):
            with self.subTest(response=response):
                self.assertFalse(self.probe.subscription_account(response))

    def test_child_environment_removes_api_and_parent_agent_overrides(self):
        removed_names = (
            "OPENAI_API_KEY",
            "CODEX_API_KEY",
            "OPENAI_BASE_URL",
            "OPENAI_API_BASE",
            "CODEX_BASE_URL",
            "CODEX_ACCESS_TOKEN",
            "CODEX_THREAD_ID",
            "CODEX_CI",
            "CODEX_MANAGED_BY_NPM",
            "CODEX_MANAGED_PACKAGE_ROOT",
            "WORK_LEAF_CODEX_TRACE",
            "WORK_LEAF_COMMAND_TMPDIR",
            "WORK_LEAF_CONTEXT_BUNDLE_DIR",
            "WORK_LEAF_CODEX_LINEARIZE_SANDBOX",
        )
        preserved = {
            "HOME": "/example/user",
            "CODEX_HOME": "/example/subscription-state",
            "PATH": "/example/bin:/usr/bin",
            "LANG": "C.UTF-8",
        }
        environment = {**preserved, **dict.fromkeys(removed_names, "test-placeholder")}
        original = dict(environment)
        child = self.probe.child_environment(environment)
        self.assertEqual(child, preserved)
        self.assertEqual(environment, original)

    def test_child_environment_accepts_absent_optional_overrides(self):
        self.assertEqual(self.probe.child_environment({}), {})
        self.assertEqual(
            self.probe.child_environment({"PATH": "/usr/bin"}),
            {"PATH": "/usr/bin"},
        )

    def test_valid_usage_accepts_complete_consistent_counts(self):
        self.assertTrue(self.probe.valid_usage(usage()))
        self.assertTrue(self.probe.valid_usage(usage(0, 0, 0, 0)))
        self.assertTrue(self.probe.valid_usage(usage(10, 10, 4, 4)))

    def test_valid_usage_requires_every_field_to_be_a_nonnegative_nonbool_integer(self):
        for field in USAGE_FIELDS:
            missing = usage()
            del missing[field]
            with self.subTest(field=field, case="missing"):
                self.assertFalse(self.probe.valid_usage(missing))
            for invalid in (None, True, False, -1, 1.0, "1", [], {}):
                with self.subTest(field=field, invalid=invalid):
                    self.assertFalse(self.probe.valid_usage({**usage(), field: invalid}))
        for invalid in (None, [], "usage", 0):
            with self.subTest(invalid=invalid):
                self.assertFalse(self.probe.valid_usage(invalid))

    def test_valid_usage_rejects_inconsistent_subsets_and_totals(self):
        for invalid in (
            usage(4, 5, 2, 1),
            usage(4, 2, 2, 3),
            {**usage(), "totalTokens": 18},
            {**usage(), "totalTokens": 16},
        ):
            with self.subTest(invalid=invalid):
                self.assertFalse(self.probe.valid_usage(invalid))

    def test_reconcile_usage_sums_each_field_once_per_response(self):
        records = [
            {"responseId": "response-a", "usage": usage()},
            {"responseId": "response-b", "usage": usage(6, 2, 3, 1)},
        ]
        self.assertTrue(self.probe.reconcile_usage(records, usage(18, 9, 8, 4)))
        self.assertTrue(
            self.probe.reconcile_usage(
                [{"responseId": "response-zero", "usage": usage(0, 0, 0, 0)}],
                usage(0, 0, 0, 0),
            )
        )

    def test_reconcile_usage_rejects_missing_or_duplicate_response_identity(self):
        valid = {"responseId": "response-a", "usage": usage()}
        for records in (
            [],
            None,
            {},
            [None],
            [{}],
            [{"usage": usage()}],
            [{"responseId": None, "usage": usage()}],
            [{"responseId": "", "usage": usage()}],
            [{"responseId": [], "usage": usage()}],
            [valid, valid],
        ):
            with self.subTest(records=records):
                self.assertFalse(self.probe.reconcile_usage(records, usage()))
        self.assertFalse(self.probe.reconcile_usage([valid, valid], usage(24, 14, 10, 6)))

    def test_reconcile_usage_rejects_missing_invalid_or_mismatched_counts(self):
        valid = {"responseId": "response-a", "usage": usage()}
        for records in (
            [{"responseId": "response-a"}],
            [{"responseId": "response-a", "usage": None}],
            [{"responseId": "response-a", "usage": {**usage(), "inputTokens": True}}],
            [{"responseId": "response-a", "usage": usage(4, 5, 2, 1)}],
        ):
            with self.subTest(records=records):
                self.assertFalse(self.probe.reconcile_usage(records, usage()))
        for total in (
            None,
            {},
            {**usage(), "totalTokens": 0},
            usage(13, 7, 5, 3),
            usage(12, 8, 5, 3),
            usage(12, 7, 6, 3),
            usage(12, 7, 5, 4),
        ):
            with self.subTest(total=total):
                self.assertFalse(self.probe.reconcile_usage([valid], total))

    def test_thread_start_uses_read_only_dynamic_tool_configuration(self):
        params = self.probe.thread_start_params("/example/scratch")
        self.assertEqual(params["cwd"], "/example/scratch")
        self.assertEqual(params["sandbox"], "read-only")
        self.assertIs(params["experimentalRawEvents"], True)
        self.assertEqual(params["model"], "gpt-5.5")
        self.assertEqual(params["modelProvider"], "openai")
        self.assertEqual(params["approvalPolicy"], "never")
        self.assertEqual(len(params["dynamicTools"]), 1)
        self.assertEqual(params["dynamicTools"][0]["type"], "function")
        self.assertEqual(params["dynamicTools"][0]["name"], self.probe.TOOL)


class HandoffSummaryTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.probe = load_module()

    def evidence(self):
        return {
            "account_type": "chatgpt",
            "resolved_model": "gpt-5.5",
            "resolved_effort": "xhigh",
            "model_turns_requested": 1,
            "thread_id": "thread-current",
            "turn_id": "turn-current",
            "tool_request_count": 1,
            "tool_arguments_valid": True,
            "tool_result_sent": True,
            "tool_completed_successfully": True,
            "tool_completed_count": 1,
            "usage_before_tool_result": True,
            "pre_result_response_count": 1,
            "final_answer_matches": True,
            "turn_status": "completed",
            "interrupt_sent": False,
            "responses": [
                {"responseId": "response-before", "usage": usage()},
                {"responseId": "response-after", "usage": usage(6, 2, 3, 1)},
            ],
            "thread_total": usage(18, 9, 8, 4),
        }

    def test_completed_handoff_with_usage_on_both_sides_passes(self):
        self.assertEqual(
            self.probe.summary(self.evidence()),
            {"single_handoff_pass": True, "response_totals_reconcile": True},
        )

    def test_interrupted_incomplete_or_unexpected_workflow_cannot_pass(self):
        for key, invalid in (
            ("account_type", "apiKey"),
            ("resolved_model", "another-model"),
            ("resolved_effort", "low"),
            ("tool_request_count", 0),
            ("tool_request_count", 2),
            ("tool_arguments_valid", False),
            ("tool_completed_successfully", False),
            ("tool_completed_count", 0),
            ("tool_completed_count", 2),
            ("usage_before_tool_result", False),
            ("final_answer_matches", False),
            ("turn_status", "interrupted"),
            ("interrupt_sent", True),
            ("failure", "timeout"),
            ("provider_error_seen", True),
            ("unexpected_activity", True),
        ):
            with self.subTest(key=key, invalid=invalid):
                result = self.probe.summary({**self.evidence(), key: invalid})
                self.assertFalse(result["single_handoff_pass"])

    def test_missing_usage_or_mismatched_thread_totals_cannot_pass(self):
        for key, invalid in (
            ("responses", [{"responseId": "response-before", "usage": None}]),
            ("responses", [{"responseId": "response-before", "usage": usage()}]),
            ("responses", []),
            ("thread_total", None),
            ("thread_total", usage(18, 10, 8, 4)),
        ):
            with self.subTest(key=key, invalid=invalid):
                result = self.probe.summary({**self.evidence(), key: invalid})
                self.assertFalse(result["single_handoff_pass"])
                self.assertFalse(result["response_totals_reconcile"])

    def test_a_tool_result_must_actually_be_sent(self):
        for flag in (None, False):
            evidence = self.evidence()
            if flag is None:
                del evidence["tool_result_sent"]
            else:
                evidence["tool_result_sent"] = flag
            with self.subTest(flag=flag):
                self.assertFalse(self.probe.summary(evidence)["single_handoff_pass"])

    def test_usage_must_include_a_response_after_the_tool_result(self):
        for count in (None, False, True, -1, 0, 2, 3, "1"):
            evidence = self.evidence()
            if count is None:
                del evidence["pre_result_response_count"]
            else:
                evidence["pre_result_response_count"] = count
            with self.subTest(count=count):
                result = self.probe.summary(evidence)
                self.assertTrue(result["response_totals_reconcile"])
                self.assertFalse(result["single_handoff_pass"])

    def test_zero_usage_cannot_prove_either_side_of_a_generated_handoff(self):
        for before, after in (
            (usage(0, 0, 0, 0), usage(0, 0, 0, 0)),
            (usage(0, 0, 0, 0), usage()),
            (usage(), usage(0, 0, 0, 0)),
        ):
            with self.subTest(before=before, after=after):
                evidence = self.evidence()
                evidence["responses"] = [
                    {"responseId": "response-before", "usage": before},
                    {"responseId": "response-after", "usage": after},
                ]
                evidence["thread_total"] = {
                    field: before[field] + after[field] for field in USAGE_FIELDS
                }
                result = self.probe.summary(evidence)
                self.assertTrue(result["response_totals_reconcile"])
                self.assertFalse(result["single_handoff_pass"])


class HandoffNotificationScopeTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.module = load_module()

    def setUp(self):
        self.probe = self.module.Probe("/unused/codex", "/unused/scratch")
        self.probe.evidence.update(thread_id="thread-current", turn_id="turn-current")
        self.probe.progress = lambda *args, **kwargs: None

    def feed(self, method, params):
        self.probe.events.put((time.monotonic(), {"method": method, "params": params}))
        self.probe.receive(0.05)

    def test_other_thread_or_turn_usage_cannot_enter_the_inventory(self):
        for thread_id, turn_id in (
            ("thread-other", "turn-current"),
            ("thread-current", "turn-other"),
        ):
            with self.subTest(thread_id=thread_id, turn_id=turn_id):
                self.feed("rawResponse/completed", {
                    "threadId": thread_id, "turnId": turn_id,
                    "responseId": "unrelated-response", "usage": usage(),
                })
                self.feed("thread/tokenUsage/updated", {
                    "threadId": thread_id, "turnId": turn_id,
                    "tokenUsage": {"last": usage(), "total": usage()},
                })
                self.assertEqual(self.probe.evidence["responses"], [])
                self.assertNotIn("thread_total", self.probe.evidence)

    def test_other_turn_cannot_supply_completion_answer_or_tool_success(self):
        self.feed("turn/completed", {
            "threadId": "thread-current",
            "turn": {"id": "turn-other", "status": "completed"},
        })
        self.feed("item/completed", {
            "threadId": "thread-current", "turnId": "turn-other",
            "item": {"type": "agentMessage", "text": self.module.ANSWER},
        })
        self.feed("item/completed", {
            "threadId": "thread-current", "turnId": "turn-other",
            "item": {"type": "dynamicToolCall", "status": "completed", "success": True},
        })
        self.assertNotIn("turn_status", self.probe.evidence)
        self.assertNotIn("final_answer_matches", self.probe.evidence)
        self.assertNotIn("tool_completed_successfully", self.probe.evidence)

    def test_usage_before_start_response_is_replayed_after_turn_identity_is_known(self):
        self.probe.evidence.pop("turn_id")
        self.probe.awaiting_turn_id = True
        self.feed("rawResponse/completed", {
            "threadId": "thread-current", "turnId": "turn-current",
            "responseId": "early-response", "usage": usage(),
        })
        self.assertEqual(self.probe.evidence["responses"], [])
        self.probe.evidence["turn_id"] = "turn-current"
        self.probe.awaiting_turn_id = False
        self.probe.receive(0.05)
        self.assertEqual(
            self.probe.evidence["responses"],
            [{"responseId": "early-response", "usage": usage()}],
        )

    def test_active_item_types_outside_the_synthetic_protocol_are_rejected(self):
        for item_type in (
            "commandExecution", "fileChange", "mcpToolCall", "webSearch",
            "collabToolCall", "imageGeneration", "futureUnrecognizedItem",
        ):
            with self.subTest(item_type=item_type):
                with self.assertRaises(RuntimeError):
                    self.feed("item/started", {
                        "threadId": "thread-current", "turnId": "turn-current",
                        "item": {"type": item_type},
                    })
                self.assertIs(self.probe.evidence.get("unexpected_activity"), True)

    def completed_tool(self):
        return {
            "id": "tool-item-id", "type": "dynamicToolCall", "tool": self.module.TOOL,
            "namespace": None, "arguments": {"request": "ping"},
            "status": "completed", "success": True,
        }

    def prepare_tool_result(self):
        self.probe.evidence.update(
            tool_request_count=1, tool_arguments_valid=True, tool_result_sent=True,
            call_id="callback-call-id",
        )
        self.probe.tool_received_at = time.monotonic()
        self.probe.result_sent_at = time.monotonic()

    def test_completed_dynamic_tool_matches_the_requested_tool_and_arguments(self):
        self.prepare_tool_result()
        self.feed("item/completed", {
            "threadId": "thread-current", "turnId": "turn-current",
            "item": self.completed_tool(),
        })
        self.assertIs(self.probe.evidence.get("tool_completed_successfully"), True)
        self.assertEqual(self.probe.evidence.get("tool_completed_count"), 1)

    def test_mismatched_or_duplicate_dynamic_tool_completion_is_rejected(self):
        for key, invalid in (
            ("tool", "unrequested_tool"),
            ("namespace", "unrequested_namespace"),
            ("arguments", {"request": "pong"}),
        ):
            with self.subTest(key=key):
                self.setUp()
                self.prepare_tool_result()
                with self.assertRaises(RuntimeError):
                    self.feed("item/completed", {
                        "threadId": "thread-current", "turnId": "turn-current",
                        "item": {**self.completed_tool(), key: invalid},
                    })
                self.assertIsNot(self.probe.evidence.get("tool_completed_successfully"), True)
        self.setUp()
        self.prepare_tool_result()
        params = {
            "threadId": "thread-current", "turnId": "turn-current",
            "item": self.completed_tool(),
        }
        self.feed("item/completed", params)
        with self.assertRaises(RuntimeError):
            self.feed("item/completed", params)

    def test_final_answer_before_tool_result_does_not_prove_continuation(self):
        self.feed("item/completed", {
            "threadId": "thread-current", "turnId": "turn-current",
            "item": {"type": "agentMessage", "text": self.module.ANSWER},
        })
        self.assertIsNot(self.probe.evidence.get("final_answer_matches"), True)


if __name__ == "__main__":
    unittest.main()
