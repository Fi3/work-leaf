#!/usr/bin/env python3

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent


def load_module():
    specification = importlib.util.spec_from_file_location(
        "accounting_precision", STUDY / "accounting_precision.py"
    )
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def bounds(lower, upper=None):
    return {"lower": lower, "upper": lower if upper is None else upper}


def usage(input_tokens, cached_tokens, output_tokens, reasoning_tokens):
    return {
        "inputTokens": input_tokens,
        "cachedInputTokens": cached_tokens,
        "outputTokens": output_tokens,
        "reasoningOutputTokens": reasoning_tokens,
        "totalTokens": input_tokens + output_tokens,
    }


def usage_event(total, last, thread_id="thread-a"):
    return {
        "method": "thread/tokenUsage/updated",
        "params": {
            "threadId": thread_id,
            "turnId": "turn-later",
            "tokenUsage": {"total": total, "last": last},
        },
    }


class AccountingPrecisionTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.module = load_module()

    def test_exact_counts_can_pass_with_savings_or_increases(self):
        for candidate, saving in ((80, 20), (120, -20)):
            with self.subTest(candidate=candidate):
                result = self.module.saving_interval(bounds(100), bounds(candidate))
                self.assertEqual(result["saving_percent"], bounds(saving))
                self.assertEqual(result["width_percentage_points"], 0)
                self.assertTrue(result["precision_pass"])

    def test_bounded_direct_denominator_uses_ratio_extrema(self):
        result = self.module.saving_interval(bounds(100, 200), bounds(80, 120))
        self.assertEqual(result["saving_percent"], bounds(-20, 60))
        self.assertEqual(result["width_percentage_points"], 80)
        self.assertFalse(result["precision_pass"])

    def test_five_point_threshold_is_inclusive_and_does_not_round_down(self):
        accepted = self.module.saving_interval(bounds(100), bounds(80, 85))
        rejected = self.module.saving_interval(bounds(100), bounds(80, 85.0001))
        self.assertAlmostEqual(accepted["width_percentage_points"], 5)
        self.assertTrue(accepted["precision_pass"])
        self.assertFalse(rejected["precision_pass"])

    def test_a_narrow_interval_crossing_zero_savings_remains_informative(self):
        result = self.module.saving_interval(bounds(100), bounds(99, 101))
        self.assertEqual(result["saving_percent"], bounds(-1, 1))
        self.assertTrue(result["precision_pass"])

    def test_nonpositive_direct_and_invalid_bounds_fail(self):
        invalid_direct = (
            bounds(0), bounds(-2, 3), bounds(-3, -1), bounds(2, 1),
            bounds(True), bounds(float("inf")), bounds(float("nan")),
            {"lower": 1}, None,
        )
        for direct in invalid_direct:
            with self.subTest(direct=direct), self.assertRaises(ValueError):
                self.module.saving_interval(direct, bounds(1))
        for candidate in (bounds(-1, 1), bounds(2, 1), bounds(False), bounds("2")):
            with self.subTest(candidate=candidate), self.assertRaises(ValueError):
                self.module.saving_interval(bounds(10), candidate)
        self.assertTrue(self.module.saving_interval(bounds(10), bounds(0))["precision_pass"])

    def test_precision_target_must_be_positive_and_finite(self):
        for target in (0, -1, True, float("inf"), float("nan")):
            with self.subTest(target=target), self.assertRaises(ValueError):
                self.module.saving_interval(bounds(10), bounds(5), target)

    def test_each_metric_has_its_own_gate_and_quality_is_not_inferred(self):
        metrics = {
            "raw_input_plus_output": {"direct": bounds(100), "work_leaf": bounds(80)},
            "uncached_input_plus_output": {
                "direct": bounds(20), "work_leaf": bounds(10, 15),
            },
        }
        quality = {"direct": {"passed": 3}, "work_leaf": {"passed": 2}}
        result = self.module.evaluate_precision(metrics, quality=quality)
        self.assertTrue(result["metrics"]["raw_input_plus_output"]["precision_pass"])
        self.assertFalse(result["metrics"]["uncached_input_plus_output"]["precision_pass"])
        self.assertFalse(result["precision_pass"])
        self.assertEqual(result["quality"], quality)
        self.assertFalse(result["sampling_confidence_interval"])
        self.assertEqual(result["quality_equivalence"], "not_assessed_by_precision_gate")

    def test_both_declared_metrics_are_required(self):
        with self.assertRaises(ValueError):
            self.module.evaluate_precision({
                "raw_input_plus_output": {"direct": bounds(100), "work_leaf": bounds(80)},
            })


class HistoricalStreamAuditTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.module = load_module()

    def gap(self):
        return {
            "thread_id": "thread-a", "turn_id": "turn-missing",
            "previous_usage_sequence": 0, "directive_sequence": 1,
        }

    def test_later_response_own_usage_does_not_recover_missing_response(self):
        records = [
            usage_event(usage(10, 5, 2, 1), usage(10, 5, 2, 1)),
            {"method": "item/completed", "params": {}},
            usage_event(usage(10, 5, 2, 1), usage(10, 5, 2, 1)),
            usage_event(usage(20, 10, 4, 2), usage(10, 5, 2, 1)),
        ]
        result = self.module.inspect_stream(records, [self.gap()])
        self.assertEqual(result["raw_response_completed_events"], 0)
        self.assertEqual(result["unresolved_responses"], 1)
        self.assertEqual(result["gaps"][0]["later_usage_sequence"], 3)
        self.assertEqual(result["gaps"][0]["status"], "zero_unreported_residual")
        self.assertEqual(result["gaps"][0]["residual"], usage(0, 0, 0, 0))

    def test_no_later_advance_and_other_thread_usage_remain_unresolved(self):
        records = [
            usage_event(usage(10, 5, 2, 1), usage(10, 5, 2, 1)),
            {"method": "item/completed", "params": {}},
            usage_event(usage(20, 10, 4, 2), usage(10, 5, 2, 1), "thread-b"),
            usage_event(usage(10, 5, 2, 1), usage(10, 5, 2, 1)),
        ]
        result = self.module.inspect_stream(records, [self.gap()])
        self.assertEqual(result["gaps"][0]["status"], "no_later_advance")
        self.assertIsNone(result["gaps"][0]["residual"])
        self.assertEqual(result["unresolved_responses"], 1)

    def test_residual_without_attribution_does_not_shrink_the_missing_count(self):
        records = [
            usage_event(usage(10, 5, 2, 1), usage(10, 5, 2, 1)),
            {"method": "item/completed", "params": {}},
            usage_event(usage(40, 20, 8, 4), usage(10, 5, 2, 1)),
            {"method": "rawResponse/completed", "params": {"usage": None}},
            {"method": "rawResponseItem/completed", "params": {}},
        ]
        result = self.module.inspect_stream(records, [self.gap()])
        self.assertEqual(result["gaps"][0]["status"], "unattributed_residual")
        self.assertEqual(result["unresolved_responses"], 1)
        self.assertEqual(result["raw_response_completed_events"], 1)
        self.assertEqual(result["raw_response_item_completed_events"], 1)

    def test_regressing_cumulative_usage_fails_instead_of_inventing_recovery(self):
        records = [
            usage_event(usage(10, 5, 2, 1), usage(10, 5, 2, 1)),
            usage_event(usage(9, 5, 2, 1), usage(1, 0, 0, 0)),
        ]
        with self.assertRaises(ValueError):
            self.module.inspect_stream(records, [])


class PrecisionCommandTest(unittest.TestCase):
    def test_command_distinguishes_narrow_wide_and_invalid_measurement(self):
        for direct, candidate, expected_code in (
            (bounds(100), bounds(120), 0),
            (bounds(100), bounds(80, 90), 1),
            (bounds(0, 100), bounds(80), 2),
        ):
            with self.subTest(expected_code=expected_code), tempfile.TemporaryDirectory() as directory:
                source = Path(directory) / "input.json"
                source.write_text(json.dumps({
                    "metrics": {
                        metric: {"direct": direct, "work_leaf": candidate}
                        for metric in ("raw_input_plus_output", "uncached_input_plus_output")
                    },
                    "quality": {"assessment": "separate"},
                }), encoding="utf-8")
                completed = subprocess.run(
                    [sys.executable, "-B", str(STUDY / "accounting_precision.py"), "evaluate", str(source)],
                    capture_output=True, text=True, timeout=10, check=False,
                )
                self.assertEqual(completed.returncode, expected_code, completed.stderr)
                if expected_code != 2:
                    report = json.loads(completed.stdout)
                    self.assertEqual(report["precision_pass"], expected_code == 0)
                    self.assertEqual(report["quality"], {"assessment": "separate"})
                else:
                    self.assertEqual(completed.stdout, "")

    def test_command_preserves_an_existing_output(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "input.json"
            output = Path(directory) / "existing.json"
            source.write_text(json.dumps({
                "metrics": {
                    metric: {"direct": bounds(100), "work_leaf": bounds(80)}
                    for metric in ("raw_input_plus_output", "uncached_input_plus_output")
                },
            }), encoding="utf-8")
            output.write_text("preserved evidence\n", encoding="utf-8")
            completed = subprocess.run(
                [sys.executable, "-B", str(STUDY / "accounting_precision.py"), "evaluate", str(source),
                 "--output", str(output)],
                capture_output=True, text=True, timeout=10, check=False,
            )
            self.assertEqual(completed.returncode, 2)
            self.assertEqual(output.read_text(encoding="utf-8"), "preserved evidence\n")


if __name__ == "__main__":
    unittest.main()
