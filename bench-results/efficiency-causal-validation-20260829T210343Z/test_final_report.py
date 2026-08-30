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
        self.assertIn("ten interrupted responses", report)

    def test_report_contains_reproducible_final_numbers(self):
        report = (STUDY / "FINAL-REPORT.md").read_text(encoding="utf-8")

        for expected in (
            "49.78%-51.62%",
            "19,220,509",
            "22,517,835",
            "19,399,622",
            "35.66 million versus 19.31 million",
            "97.95%-98.02%",
        ):
            self.assertIn(expected, report)

    def test_state_marks_no_required_step_remaining(self):
        state = (STUDY / "STATE.md").read_text(encoding="utf-8")
        self.assertIn("Collection is complete.", state)
        self.assertIn("17,471,532-18,138,199", state)
        self.assertIn("decompose.py` refuses", state)


if __name__ == "__main__":
    unittest.main()
