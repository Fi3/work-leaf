#!/usr/bin/env python3

import json
import tempfile
import unittest
from pathlib import Path

import quality_audit


class QualityAuditTest(unittest.TestCase):
    def test_command_feature_requires_status_but_not_fork_continuation(self):
        self.assertTrue(
            quality_audit.feature_scores(
                {
                    "visual": "pass",
                    "slash_status": "pass",
                    "slash_fork_continuation": "fail",
                    "completion": "pass",
                }
            )["commands"]
        )
        self.assertFalse(
            quality_audit.feature_scores(
                {
                    "visual": "pass",
                    "slash_status": "fail",
                    "slash_fork_continuation": "pass",
                    "completion": "pass",
                }
            )["commands"]
        )

    def test_same_block_pairs_keep_unmatched_attempts_as_reliability_data(self):
        runs = [
            quality_audit.scored_run(
                "r1-direct-1",
                "sequential",
                "pass",
                {"visual": "pass", "slash_status": "pass", "completion": "pass"},
                block="r1",
                attempt=1,
            ),
            quality_audit.scored_run(
                "r1-work-leaf-1",
                "work-leaf",
                "pass",
                {"visual": "pass", "slash_status": "fail", "completion": "pass"},
                block="r1",
                attempt=1,
            ),
            quality_audit.scored_run(
                "r1-direct-2",
                "sequential",
                "fail",
                {"visual": "fail", "slash_status": "fail", "completion": "pass"},
                block="r1",
                attempt=2,
            ),
        ]

        comparison = quality_audit.same_block_comparison(runs)

        self.assertEqual(comparison["pairs"], 1)
        self.assertEqual(comparison["mean_completed_features"]["sequential"], 3.0)
        self.assertEqual(comparison["mean_completed_features"]["work-leaf"], 2.0)
        self.assertEqual(comparison["observed_mean_paired_difference"], -1.0)
        self.assertEqual(comparison["unmatched_run_ids"], ["r1-direct-2"])

    def test_failed_attempts_remain_in_workflow_summary(self):
        runs = [
            quality_audit.scored_run(
                "direct-pass",
                "sequential",
                "pass",
                {
                    "visual": "pass",
                    "slash_status": "pass",
                    "slash_fork_continuation": "pass",
                    "completion": "pass",
                },
            ),
            quality_audit.scored_run(
                "direct-fail",
                "sequential",
                "fail",
                {
                    "visual": "pass",
                    "slash_status": "fail",
                    "slash_fork_continuation": "fail",
                    "completion": "pass",
                },
            ),
        ]

        summary = quality_audit.summarize(runs)["sequential"]

        self.assertEqual(summary["runs"], 2)
        self.assertEqual(summary["workflow_passes"], 1)
        self.assertEqual(summary["feature_pass_counts"], {
            "visual": 2,
            "commands": 1,
            "completion": 2,
        })
        self.assertEqual(summary["mean_completed_features"], 2.5)

    def test_manifest_contains_only_normal_product_conditions(self):
        manifest = quality_audit.load_manifest(quality_audit.DEFAULT_MANIFEST)
        current = [run for run in manifest["runs"] if run["cohort"] == "current"]

        self.assertEqual(len(current), 10)
        self.assertEqual(
            {run["workflow"] for run in current}, {"sequential", "work-leaf"}
        )
        self.assertTrue(all("wl-000" in run["id"] for run in current if run["workflow"] == "work-leaf"))
        self.assertTrue(all("direct" in run["id"] for run in current if run["workflow"] == "sequential"))
        self.assertTrue(all("block" in run and "attempt" in run for run in current))

    def test_manifest_freezes_original_task_and_historical_work_leaf_sanity_set(self):
        manifest = quality_audit.load_manifest(quality_audit.DEFAULT_MANIFEST)

        self.assertEqual(
            manifest["original_task"]["source_commit"],
            "e70c933ff0313fafb771ff214d06734845537b86",
        )
        self.assertEqual(
            manifest["original_task"]["features"][1],
            "when an user prompt start with / and is followed by something without whitespace that is a command for the agent; the orchestrator must send it to the selected backend agent and show that backend response",
        )
        historical_work_leaf = [
            run
            for run in manifest["runs"]
            if run["cohort"] == "historical-work-leaf-sanity"
        ]
        self.assertEqual(len(historical_work_leaf), 3)
        self.assertTrue(all(run["workflow"] == "work-leaf" for run in historical_work_leaf))

    def test_result_writer_preserves_historical_evidence(self):
        payload = {
            "historical_status_evidence": [
                {"id": "historical-r2", "outcome": "pass", "audit": "audit.txt"}
            ],
            "runs": [],
        }
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "result.json"
            quality_audit.write_json(output, payload)
            restored = json.loads(output.read_text(encoding="utf-8"))

        self.assertEqual(
            restored["historical_status_evidence"][0]["id"], "historical-r2"
        )

    def test_historical_sanity_does_not_define_a_current_quality_floor(self):
        references = [
            {"completed_features": 2},
            {"completed_features": 3},
            {"completed_features": 2},
        ]

        sanity = quality_audit.historical_sanity(references)

        self.assertEqual(sanity["reference_runs"], 3)
        self.assertEqual(sanity["mean_completed_features"], 2.333333)
        self.assertNotIn("current_meets_floor", sanity)


if __name__ == "__main__":
    unittest.main()
