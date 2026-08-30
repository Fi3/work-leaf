#!/usr/bin/env python3

import json
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent


class FinalReportTest(unittest.TestCase):
    def test_report_matches_machine_readable_causal_result(self):
        evidence = json.loads((STUDY / "evidence.json").read_text(encoding="utf-8"))
        report = (STUDY / "FINAL-REPORT.md").read_text(encoding="utf-8")

        coverage = evidence["causal_coverage"]["share_of_endpoint_gap_percent"]
        steps = {
            step["name"]: step
            for step in evidence["ordered_attribution"]["raw"]["steps"]
        }
        endpoint_gap = evidence["ordered_attribution"]["raw"]["endpoint_gap"]
        direct = evidence["groups"]["L"]["mean_usage"]["raw_input_plus_output"]
        work_leaf = evidence["groups"]["S"]["mean_usage"]["raw_input_plus_output"]

        self.assertIn(f"{coverage:.2f}%", report)
        self.assertIn(
            f"{steps['work_leaf_orchestration']['share_of_endpoint_gap_percent']:.2f}%",
            report,
        )
        self.assertIn(
            f"{steps['mediated_reads_and_interruption']['share_of_endpoint_gap_percent']:.2f}%",
            report,
        )
        self.assertIn(f"{endpoint_gap / 1_000_000:.3f} million", report)
        self.assertIn(f"{direct:,.0f}", report)
        self.assertIn(f"{work_leaf:,.0f}", report)

    def test_report_defines_the_two_token_totals(self):
        report = (STUDY / "FINAL-REPORT.md").read_text(encoding="utf-8")
        normalized = " ".join(report.split())
        self.assertIn('"Raw tokens" means all input plus output tokens', normalized)
        self.assertIn('"Uncached tokens" means fresh input plus output', normalized)

    def test_report_includes_the_conservative_endpoint_scenario(self):
        evidence = json.loads((STUDY / "evidence.json").read_text(encoding="utf-8"))
        report = (STUDY / "FINAL-REPORT.md").read_text(encoding="utf-8")
        bounded = evidence["causal_coverage_bounded"]
        raw = evidence["ordered_attribution"]["raw_bounded"]

        self.assertEqual(
            evidence["status"],
            "complete_with_bounded_normal_endpoint",
        )
        self.assertIn(
            f"{bounded['share_of_endpoint_gap_percent']['lower']:.2f}%",
            report,
        )
        self.assertIn(
            f"{raw['endpoint_gap']['lower']:,.0f}",
            report,
        )
        self.assertIn("ten unresolved responses", report)


if __name__ == "__main__":
    unittest.main()
