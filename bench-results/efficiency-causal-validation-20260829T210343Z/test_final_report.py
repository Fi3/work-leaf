#!/usr/bin/env python3

import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent


class FinalReportTest(unittest.TestCase):
    def test_report_defines_metrics_conditions_and_limits(self):
        report = (STUDY / "FINAL-REPORT.md").read_text(encoding="utf-8")

        self.assertIn("## Abstract", report)
        self.assertIn("Raw tokens mean input plus output", report)
        self.assertIn("normal Work Leaf endpoint", report)
        self.assertIn("not a formal equal-quality comparison", report)
        self.assertIn("35 interrupted responses", report)

    def test_report_contains_reproducible_final_numbers(self):
        report = (STUDY / "FINAL-REPORT.md").read_text(encoding="utf-8")

        for expected in (
            "45.38%-51.62%",
            "19,220,509",
            "22,517,835",
            "19,399,622",
            "35.66 million versus 19.31 million",
            "97.75%-98.02%",
        ):
            self.assertIn(expected, report)

    def test_state_marks_no_required_step_remaining(self):
        state = (STUDY / "STATE.md").read_text(encoding="utf-8")
        self.assertIn("Collection is complete.", state)
        self.assertIn("17,471,532-19,725,532", state)
        self.assertIn("decompose.py` refuses", state)

    def test_control_reports_do_not_treat_the_normal_lower_bound_as_exact(self):
        expected_reports = {
            "04-PILOT-RESULT.md": ("19,220,509", "sign changes"),
            "06-CONTINUED-RESPONSE-RESULT.md": (
                "22,517,835",
                "2,792,303-5,046,303 more",
            ),
            "08-COMBINED-CONTROL-RESULT.md": ("19,399,622", "sign changes"),
        }
        for name, (exact_mean, interpretation) in expected_reports.items():
            with self.subTest(name=name):
                report = (STUDY / name).read_text(encoding="utf-8")
                self.assertIn("17,471,532-19,725,532", report)
                self.assertIn(exact_mean, report)
                self.assertIn(interpretation, report)


if __name__ == "__main__":
    unittest.main()
