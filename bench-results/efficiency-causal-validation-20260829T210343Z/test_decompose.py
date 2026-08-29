#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
MODULE_PATH = STUDY / "decompose.py"


def load_module():
    spec = importlib.util.spec_from_file_location("decompose", MODULE_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class DecompositionTest(unittest.TestCase):
    def test_two_cohorts_reproduce_the_same_mechanism_direction(self):
        evidence = load_module().build_evidence()

        self.assertEqual(evidence["status"], "complete")
        self.assertEqual(evidence["rollout_integrity"]["hash_mismatches"], [])
        current = evidence["cohorts"]["current_detailed_6_by_6"]
        historical = evidence["cohorts"]["historical_quality_balanced_3_by_3"]
        self.assertAlmostEqual(current["token_gap"]["raw_tokens"], 18_644_849.666666668)
        self.assertGreater(current["token_gap"]["cached_input_share_of_raw_gap_percent"], 98.5)
        self.assertAlmostEqual(current["usage_changes"]["direct_mean"], 320.1666666666667)
        self.assertAlmostEqual(current["usage_changes"]["work_leaf_mean"], 212.33333333333334)
        self.assertGreater(current["context_per_change"]["direct_mean_input_tokens"], 110_000)
        self.assertLess(current["context_per_change"]["work_leaf_mean_input_tokens"], 85_000)
        self.assertAlmostEqual(
            current["input_gap_factorization"]["sum_tokens"],
            current["token_gap"]["input_tokens"],
        )
        self.assertLess(
            historical["usage_changes"]["work_leaf_mean"],
            historical["usage_changes"]["direct_mean"],
        )
        self.assertLess(
            historical["context_per_change"]["work_leaf_mean_input_tokens"],
            historical["context_per_change"]["direct_mean_input_tokens"],
        )


if __name__ == "__main__":
    unittest.main()
