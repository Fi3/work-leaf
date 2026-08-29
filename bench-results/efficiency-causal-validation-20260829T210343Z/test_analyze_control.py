#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
SCRIPT = STUDY / "analyze-control.py"


def load_module():
    specification = importlib.util.spec_from_file_location("analyze_control", SCRIPT)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load {SCRIPT}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


class AnalyzeControlTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.evidence = load_module().build_evidence()

    def test_every_control_activated_and_reconciled(self):
        self.assertEqual(self.evidence["status"], "complete")
        self.assertEqual(self.evidence["rollout_integrity"]["hash_mismatches"], [])
        for run in self.evidence["activation"]:
            self.assertEqual(run["turn_start_threads"], 8)
            self.assertEqual(run["direct_prompt_threads"], 7)
            self.assertEqual(run["mediated_prompt_threads"], 0)
            self.assertEqual(run["mediated_read_directives"], 0)
            self.assertGreater(run["direct_read_commands"], 0)
            self.assertTrue(run["exact_accounting"])
            self.assertTrue(run["original_and_cumulative_usage_match"])
            self.assertEqual(run["completed_features"], 3)

    def test_control_group_result_is_exact(self):
        control = self.evidence["groups"]["direct_read_work_leaf"]
        self.assertEqual(control["runs"], 3)
        self.assertEqual(control["completed_features"], 9)
        self.assertAlmostEqual(control["mean_usage"]["raw_input_plus_output"], 19220508.666666668)
        self.assertAlmostEqual(control["mean_usage"]["uncached_input_plus_output"], 1607367.3333333333)
        self.assertAlmostEqual(control["mean_usage_changes"], 202.66666666666666)

    def test_read_route_effect_and_remaining_gap_are_kept_separate(self):
        comparison = self.evidence["comparisons"]["direct_read_minus_normal_work_leaf"]
        self.assertAlmostEqual(comparison["raw_tokens"], 1748976.666666668)
        self.assertAlmostEqual(comparison["uncached_tokens"], 263963.33333333326)
        self.assertAlmostEqual(comparison["raw_fraction_of_direct_gap_percent"], 9.380481462360596)
        self.assertAlmostEqual(
            comparison["uncached_fraction_of_direct_gap_percent"], 99.49304268618275
        )
        self.assertAlmostEqual(comparison["usage_changes"], -9.666666666666686)
        self.assertGreater(comparison["input_context_per_usage_change"], 0)
        full_quality = self.evidence["comparisons"][
            "full_quality_direct_read_minus_normal_work_leaf"
        ]
        self.assertAlmostEqual(full_quality["raw_fraction_of_direct_gap_percent"], 23.039765042807378)


if __name__ == "__main__":
    unittest.main()
