#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
SCRIPT = STUDY / "analyze-continued-response.py"


def load_module():
    specification = importlib.util.spec_from_file_location(
        "analyze_continued_response", SCRIPT
    )
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load {SCRIPT}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


class AnalyzeContinuedResponseTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.evidence = load_module().build_evidence()

    def test_real_control_activation_and_exact_accounting_are_preserved(self):
        self.assertEqual(self.evidence["status"], "complete-with-partial-activation")
        self.assertEqual(self.evidence["rollout_integrity"]["hash_mismatches"], [])
        self.assertEqual(
            [row["continued_response_count"] for row in self.evidence["activation"]],
            [2, 2, 4],
        )
        self.assertEqual(
            [row["timeout_count"] for row in self.evidence["activation"]],
            [0, 1, 0],
        )
        for row in self.evidence["activation"]:
            self.assertTrue(row["passed"])
            self.assertTrue(row["exact_accounting"])
            self.assertTrue(row["interrupt_bytes_preserved"])
            self.assertEqual(row["model"], "gpt-5.5")
            self.assertEqual(row["reasoning_effort"], "xhigh")

    def test_control_keeps_every_quality_outcome_and_exact_total(self):
        control = self.evidence["groups"]["continued_response_work_leaf"]
        self.assertEqual(control["runs"], 3)
        self.assertEqual(control["completed_features"], 6)
        self.assertEqual(control["full_quality_runs"], 1)
        self.assertAlmostEqual(
            control["mean_usage"]["raw_input_plus_output"], 22517835.333333332
        )
        self.assertAlmostEqual(
            control["mean_usage"]["uncached_input_plus_output"], 1632075.3333333333
        )
        self.assertAlmostEqual(control["mean_usage_changes"], 259.3333333333333)

    def test_interruption_effect_and_read_effect_are_not_added(self):
        comparison = self.evidence["comparisons"][
            "continued_response_minus_normal_work_leaf"
        ]
        self.assertAlmostEqual(comparison["raw_tokens"], 5046303.333333332)
        self.assertAlmostEqual(comparison["uncached_tokens"], 288671.33333333326)
        self.assertAlmostEqual(
            comparison["raw_fraction_of_direct_gap_percent"], 27.06540102790495
        )
        self.assertAlmostEqual(comparison["usage_changes"], 47.0)
        self.assertGreater(comparison["input_context_per_usage_change"], 0)
        self.assertFalse(self.evidence["causal_summary"]["fractions_are_additive"])


if __name__ == "__main__":
    unittest.main()
