#!/usr/bin/env python3
"""Provider-free checks for the bounded API diagnostic's saved evidence."""

import importlib.util
import unittest
from pathlib import Path


spec = importlib.util.spec_from_file_location(
    "api_probe", Path(__file__).with_name("api_probe.py")
)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class ApiProbeTests(unittest.TestCase):
    def test_response_evidence_retains_identity_usage_and_error_but_no_content(self):
        secret = "example-credential-not-real"
        response = {
            "id": "resp_example",
            "model": "gpt-5.5-2026-04-23",
            "status": "failed",
            "usage": None,
            "error": {"code": "server_error", "message": secret},
            "output": [{"text": secret}],
        }
        evidence = probe.safe_response(response, secret)
        self.assertEqual(evidence["response_id"], "resp_example")
        self.assertEqual(evidence["model"], "gpt-5.5-2026-04-23")
        self.assertEqual(evidence["error"]["code"], "server_error")
        self.assertNotIn(secret, str(evidence))
        self.assertNotIn("output", evidence)

    def test_cancellation_requires_active_generation_and_complete_cancelled_usage(self):
        response = {
            "status": "cancelled",
            "usage": {
                "input_tokens": 10,
                "cached_input_tokens": 0,
                "output_tokens": 4,
                "reasoning_output_tokens": 1,
                "total_tokens": 14,
            },
        }
        self.assertTrue(probe.cancel_usage_gate(True, True, response))
        self.assertFalse(probe.cancel_usage_gate(False, True, response))
        self.assertFalse(probe.cancel_usage_gate(True, False, response))
        self.assertFalse(probe.cancel_usage_gate(True, True, {**response, "status": "completed"}))
        self.assertFalse(probe.cancel_usage_gate(True, True, {**response, "usage": None}))
        incomplete = {**response, "usage": {**response["usage"], "cached_input_tokens": None}}
        self.assertFalse(probe.cancel_usage_gate(True, True, incomplete))

    def test_secret_in_error_code_is_not_saved(self):
        secret = "credential"
        self.assertEqual(
            probe.safe_error({"code": secret, "type": secret}, secret),
            {"code": "redacted", "type": "redacted"},
        )

    def test_cancellation_rejects_inconsistent_or_incomplete_usage_arithmetic(self):
        usage = {
            "input_tokens": 10, "cached_input_tokens": 2,
            "output_tokens": 4, "reasoning_output_tokens": 1, "total_tokens": 14,
        }
        for invalid in (
            {"cached_input_tokens": 11}, {"total_tokens": 15},
            {"total_tokens": None}, {"total_tokens": "14"},
            {"reasoning_output_tokens": 5}, {"reasoning_output_tokens": -1},
        ):
            with self.subTest(invalid=invalid):
                self.assertFalse(probe.cancel_usage_gate(
                    True, True, {"status": "cancelled", "usage": {**usage, **invalid}}
                ))


if __name__ == "__main__":
    unittest.main()
