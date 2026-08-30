#!/usr/bin/env python3

import re
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent


class FinalReportTest(unittest.TestCase):
    def test_report_defines_metrics_conditions_and_limits(self):
        report = (STUDY / "FINAL-REPORT.md").read_text(encoding="utf-8")

        self.assertIn("## Abstract", report)
        self.assertIn('"Raw tokens" means input plus output', report)
        self.assertIn("Direct sequential Codex", report)
        self.assertIn("Concurrent Work Leaf", report)
        self.assertIn("The exact percentage should not be generalized", report)
        self.assertNotIn("wl-000", report)

    def test_report_contains_reproducible_final_numbers(self):
        report = (STUDY / "FINAL-REPORT.md").read_text(encoding="utf-8")

        for expected in (
            "51.62%",
            "60.25%",
            "10.34%",
            "89.66%",
            "76.62%",
            "23.38%",
            "16.72 million",
        ):
            self.assertIn(expected, report)
        self.assertRegex(report, re.compile(r"17\.17\s+million"))

    def test_state_marks_no_required_step_remaining(self):
        state = (STUDY / "STATE.md").read_text(encoding="utf-8")
        self.assertIn("The causal study is complete.", state)
        self.assertIn("No required study step remains.", state)


if __name__ == "__main__":
    unittest.main()
