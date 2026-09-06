#!/usr/bin/env python3

import importlib.util
import json
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent


def load_module():
    specification = importlib.util.spec_from_file_location(
        "attribution_bounds_analyze", STUDY / "analyze.py"
    )
    if specification is None or specification.loader is None:
        raise RuntimeError("cannot load the study analyzer")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


class AttributionBoundsTest(unittest.TestCase):
    def setUp(self):
        self.module = load_module()
        self.values = {"D": 100.0, "L": 90.0, "S": 60.0, "C": 55.0}
        self.mechanisms = (
            "work_leaf_orchestration",
            "mediated_reads_and_interruption",
        )

    def test_zero_crossing_keeps_token_ranges_and_marks_percentage_undefined(self):
        for lower, upper in ((90.0, 110.0), (100.0, 110.0), (90.0, 100.0), (100.0, 100.0)):
            with self.subTest(lower=lower, upper=upper):
                result = self.module.bounded_endpoint_bridge(
                    self.values, {"lower": lower, "upper": upper}
                )
                self.assertEqual(
                    result["endpoint_gap"],
                    {"lower": 100.0 - upper, "upper": 100.0 - lower},
                )
                self.assertEqual(result["share_of_endpoint_gap_status"], "undefined")
                self.assertEqual(
                    result["share_of_endpoint_gap_reason"],
                    "endpoint_gap_interval_contains_zero",
                )
                steps = {step["name"]: step for step in result["steps"]}
                self.assertEqual(
                    steps["work_leaf_orchestration"]["tokens"],
                    {"lower": 30.0, "upper": 30.0},
                )
                self.assertEqual(
                    steps["mediated_reads_and_interruption"]["tokens"],
                    {"lower": 55.0 - upper, "upper": 55.0 - lower},
                )
                for step in result["steps"]:
                    self.assertIsNone(step["share_of_endpoint_gap_percent"])
                for scenario in result["scenarios"].values():
                    if scenario["endpoint_gap"] == 0:
                        for step in scenario["steps"]:
                            self.assertIsNone(step["share_of_endpoint_gap_percent"])

    def test_selected_coverage_is_undefined_when_gap_includes_zero(self):
        for lower, upper in ((90.0, 110.0), (100.0, 110.0), (90.0, 100.0), (100.0, 100.0)):
            with self.subTest(lower=lower, upper=upper):
                bridge = self.module.bounded_endpoint_bridge(
                    self.values, {"lower": lower, "upper": upper}
                )
                result = self.module.bounded_selected_causal_coverage(
                    bridge, self.mechanisms
                )
                self.assertEqual(
                    result["tokens"],
                    {"lower": 85.0 - upper, "upper": 85.0 - lower},
                )
                self.assertIsNone(result["share_of_endpoint_gap_percent"])
                self.assertEqual(result["share_of_endpoint_gap_status"], "undefined")
                self.assertEqual(
                    result["share_of_endpoint_gap_reason"],
                    "endpoint_gap_interval_contains_zero",
                )
                for name, scenario in result["scenarios"].items():
                    if bridge["scenarios"][name]["endpoint_gap"] == 0:
                        self.assertIsNone(scenario["share_of_endpoint_gap_percent"])

    def test_positive_and_negative_gap_intervals_keep_valid_percentage_bounds(self):
        for lower, upper in ((50.0, 52.0), (150.0, 160.0)):
            with self.subTest(lower=lower, upper=upper):
                bridge = self.module.bounded_endpoint_bridge(
                    self.values, {"lower": lower, "upper": upper}
                )
                self.assertEqual(bridge["share_of_endpoint_gap_status"], "bounded")
                self.assertIsNone(bridge["share_of_endpoint_gap_reason"])
                steps = {step["name"]: step for step in bridge["steps"]}
                shares = [30.0 / (100.0 - value) * 100 for value in (lower, upper)]
                self.assertEqual(
                    steps["work_leaf_orchestration"]["share_of_endpoint_gap_percent"],
                    {"lower": min(shares), "upper": max(shares)},
                )
                result = self.module.bounded_selected_causal_coverage(
                    bridge, self.mechanisms
                )
                shares = [
                    (85.0 - value) / (100.0 - value) * 100
                    for value in (lower, upper)
                ]
                self.assertEqual(
                    result["share_of_endpoint_gap_percent"],
                    {"lower": min(shares), "upper": max(shares)},
                )
                self.assertEqual(result["share_of_endpoint_gap_status"], "bounded")
                self.assertIsNone(result["share_of_endpoint_gap_reason"])

    def test_point_bridge_still_rejects_an_undefined_percentage(self):
        with self.assertRaisesRegex(ValueError, "endpoint gap is zero"):
            self.module.ordered_bridge({**self.values, "W": self.values["D"]})


class EvidenceInterpretationTest(unittest.TestCase):
    def test_saved_evidence_distinguishes_accounting_from_statistical_certainty(self):
        evidence = json.loads((STUDY / "evidence.json").read_text(encoding="utf-8"))
        self.assertEqual(evidence["status"], "incomplete_normal_endpoint_measurement")
        interpretation = evidence["interpretation"]
        self.assertEqual(interpretation["bounds"]["kind"], "descriptive_accounting_bounds")
        self.assertFalse(interpretation["bounds"]["sampling_confidence_interval"])
        self.assertEqual(
            interpretation["bounds"]["coverage"],
            "missing_response_token_allowance_only",
        )
        self.assertEqual(
            interpretation["causal_attribution"]["kind"],
            "ordered_group_mean_differences",
        )
        self.assertFalse(
            interpretation["causal_attribution"]["sampling_uncertainty_quantified"]
        )
        self.assertFalse(
            interpretation["causal_attribution"]["causal_coverage_target_established"]
        )
        self.assertEqual(interpretation["quality_equivalence"], "not_established")


if __name__ == "__main__":
    unittest.main()
