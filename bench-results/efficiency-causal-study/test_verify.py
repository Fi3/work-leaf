#!/usr/bin/env python3

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import verify


STUDY_DIR = Path(__file__).resolve().parent


class EfficiencyStudyVerificationTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.recomputed = verify.verify_study(STUDY_DIR)

    def test_current_quality_rows_and_means_are_retained(self):
        quality = self.recomputed["quality"]

        self.assertEqual(quality["retained_current_rows"], 10)
        self.assertEqual(quality["sequential"]["runs"], 6)
        self.assertEqual(quality["sequential"]["score_sum"], 12)
        self.assertEqual(quality["sequential"]["mean"], 2.0)
        self.assertEqual(quality["sequential"]["status_passes"], 4)
        self.assertEqual(quality["sequential"]["distribution"], [0, 2, 2, 2])
        self.assertEqual(quality["work_leaf"]["runs"], 4)
        self.assertEqual(quality["work_leaf"]["score_sum"], 9)
        self.assertEqual(quality["work_leaf"]["mean"], 2.25)
        self.assertEqual(quality["work_leaf"]["status_passes"], 4)
        self.assertEqual(quality["work_leaf"]["distribution"], [0, 1, 1, 2])

    def test_matched_pairs_and_historical_sanity_are_recomputed(self):
        quality = self.recomputed["quality"]

        self.assertEqual(quality["pairs"]["count"], 4)
        self.assertEqual(quality["pairs"]["differences"], [-1, 2, 1, -1])
        self.assertEqual(quality["pairs"]["sequential_mean"], 2.0)
        self.assertEqual(quality["pairs"]["work_leaf_mean"], 2.25)
        self.assertEqual(
            quality["pairs"]["unmatched_ids"],
            ["r15-direct-attempt-02", "r16-direct-attempt-01"],
        )
        self.assertEqual(quality["historical_work_leaf"]["scores"], [3, 2, 2])
        self.assertEqual(quality["historical_work_leaf"]["status_passes"], 3)

    def test_zero_score_is_retained_without_inventing_a_study_row(self):
        scorer = verify.load_frozen_scorer(STUDY_DIR)
        zero = scorer.scored_run(
            "synthetic-zero-retention-check",
            "sequential",
            "fail",
            {
                "visual": "fail",
                "slash_status": "fail",
                "slash_fork_continuation": "fail",
                "completion": "fail",
            },
        )

        summary = scorer.summarize([zero])["sequential"]

        self.assertEqual(zero["completed_features"], 0)
        self.assertEqual(summary["runs"], 1)
        self.assertEqual(summary["completed_feature_distribution"], {
            "0": 1,
            "1": 0,
            "2": 0,
            "3": 0,
        })

    def test_exact_r19_pair_and_reductions_are_recomputed(self):
        pair = self.recomputed["exact_token_pair"]

        self.assertEqual(pair["scope"], "non_title_workflow")
        self.assertEqual(pair["sequential_raw"], 43_009_498)
        self.assertEqual(pair["sequential_uncached"], 2_105_178)
        self.assertEqual(pair["work_leaf_raw"], 12_018_293)
        self.assertEqual(pair["work_leaf_uncached"], 1_072_757)
        self.assertEqual(pair["raw_reduction_percent_4dp"], 72.0567)
        self.assertEqual(pair["uncached_reduction_percent_4dp"], 49.0420)
        self.assertEqual(pair["scores"], {"sequential": 3, "work_leaf": 2})

    def test_isolated_mechanism_percentages_are_recomputed(self):
        mechanisms = self.recomputed["isolated_mechanisms"]

        self.assertEqual(
            mechanisms["changed_repeated_read"]["reductions_4dp"],
            {"raw": 14.8824, "uncached": 85.0587},
        )
        self.assertEqual(
            mechanisms["unchanged_repeated_read"]["reductions_4dp"],
            {"raw": 15.2067, "uncached": 87.2081},
        )
        self.assertEqual(
            mechanisms["inline_review_provenance"]["reductions_4dp"],
            {"raw": 75.3157, "uncached": 36.8717},
        )

    def test_inactive_and_mixed_screens_are_checked(self):
        screens = self.recomputed["screens"]

        self.assertEqual(screens["large_read_bundle"]["raw_change_percent_3dp"], 14.638)
        self.assertEqual(
            screens["large_read_bundle"]["uncached_reduction_percent_3dp"],
            59.242,
        )
        self.assertEqual(
            screens["large_read_bundle"]["requested_feature_off_direction"],
            {"raw_reduction_percent": 14.638, "uncached_increase_percent": 59.242},
        )
        self.assertFalse(screens["patch_acknowledgement"]["raw_benefit_observed"])
        self.assertFalse(screens["patch_acknowledgement"]["behavior_benefit_observed"])
        self.assertFalse(screens["linearization_compaction"]["activated"])
        self.assertEqual(screens["command_output_compaction"]["opportunity_bytes"], 0)
        self.assertEqual(screens["directive_interruption"]["post_directive_generation"], 0)
        self.assertFalse(screens["directive_interruption"]["usage_available"])

    def test_factorial_missing_cells_block_exact_allocation(self):
        factorial = self.recomputed["factorial"]

        self.assertEqual(factorial["required_cells"], 9)
        self.assertEqual(factorial["exact_cells"], 6)
        self.assertEqual(
            factorial["missing_exact_cells"],
            ["wl-011", "wl-101", "wl-111"],
        )
        self.assertFalse(factorial["exact_allocation_available"])
        self.assertFalse(factorial["mixed_block_substitution_valid"])
        self.assertEqual(factorial["completed_attempts"], 15)
        self.assertEqual(factorial["workflow_passes"], 13)
        self.assertEqual(factorial["exact_accounted_attempts"], 11)
        self.assertEqual(factorial["exact_accounted_workflow_passes"], 9)
        self.assertFalse(factorial["uniform_predeclared_retry_cap"])
        self.assertFalse(factorial["current_paid_authorization"])

    def test_scopes_and_claim_limits_are_explicit(self):
        mechanisms = self.recomputed["isolated_mechanisms"]
        scopes = [item["scope"] for item in mechanisms.values()]

        self.assertEqual(len(scopes), len(set(scopes)))
        self.assertFalse(self.recomputed["limits"]["percentages_can_be_added"])
        self.assertFalse(self.recomputed["limits"]["percentages_are_whole_gap_shares"])
        self.assertFalse(
            self.recomputed["limits"]["formal_quality_equivalence_available"]
        )
        self.assertFalse(
            self.recomputed["limits"]["exact_whole_gap_allocation_available"]
        )
        self.assertEqual(
            self.recomputed["limits"]["cross_project_generalization"], "deferred"
        )

    def test_success_partial_failure_and_zero_retention_rules_are_audited(self):
        outcomes = self.recomputed["quality"]["outcomes"]

        self.assertEqual(outcomes["saved_workflow_results"], ["fail", "pass"])
        self.assertEqual(outcomes["observed_feature_scores"], [1, 2, 3])
        self.assertTrue(outcomes["zero_feature_rows_are_eligible"])
        self.assertEqual(outcomes["zero_feature_rows_observed"], 0)
        self.assertEqual(outcomes["observed_attempt_numbers"], [1, 2])
        self.assertFalse(outcomes["uniform_predeclared_retry_cap"])

    def test_driver_bytes_and_scorer_contract_stay_distinct(self):
        prompts = self.recomputed["prompts"]

        self.assertEqual(prompts["driver"]["lengths"], [189, 609, 149])
        self.assertEqual(
            prompts["driver"]["json_lf_sha256"],
            "c27bb64412deeee646d8d25753b599bf1650050e7b761c9dc119611abac58d1a",
        )
        self.assertEqual(prompts["analysis_scorer"]["lengths"], [189, 204, 149])
        self.assertEqual(
            prompts["analysis_scorer"]["json_lf_sha256"],
            "45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a",
        )
        self.assertNotEqual(
            prompts["driver"]["json_lf_sha256"],
            prompts["analysis_scorer"]["json_lf_sha256"],
        )
        self.assertTrue(prompts["driver"]["origin_commit_matches"])
        self.assertTrue(prompts["driver"]["committed_drivers_match"])
        self.assertTrue(prompts["analysis_scorer"]["fork_is_supplemental"])

    def test_frozen_scorer_and_fixtures_match_archive_hashes(self):
        checked = self.recomputed["frozen_scorer_files"]

        self.assertEqual(len(checked), 7)
        self.assertTrue(all(item["matches"] for item in checked))

    def test_frozen_scorer_logs_are_complete_hash_bound_and_compact(self):
        logs = self.recomputed["frozen_scorer_logs"]

        self.assertEqual(logs["files"], 64)
        self.assertEqual(logs["bytes"], 141_015)
        self.assertEqual(
            logs["sha256_manifest_sha256"],
            "1297885d7ea44d5691bec56765753735626f7b47a008e4595e274d7cad236247",
        )
        self.assertTrue(logs["all_result_hashes_match"])

    def test_archive_and_replay_provenance_is_explicit(self):
        provenance = json.loads(
            (STUDY_DIR / "provenance.json").read_text(encoding="utf-8")
        )

        self.assertEqual(provenance["archive"]["verification_result"], "pass")
        self.assertEqual(provenance["archive"]["source_checksum_mismatches"], 0)
        self.assertEqual(provenance["replay"]["candidate_count"], 66)
        self.assertEqual(provenance["replay"]["passing_candidates"], 66)
        self.assertFalse(provenance["replay"]["real_agent_or_model_execution_permitted"])
        self.assertFalse(provenance["step_4_consolidation"]["candidate_execution"])
        self.assertFalse(
            provenance["step_4_consolidation"]["paid_benchmark_or_model_runs"]
        )

        references = self.recomputed["archive_references"]
        reference_ids = {reference["id"] for reference in references}
        self.assertIn("recovery_checksums", reference_ids)
        self.assertIn("recovery_keep_remove_manifest", reference_ids)
        self.assertIn("recovery_candidate_assets", reference_ids)
        self.assertIn("step3_replay_ledger", reference_ids)
        self.assertIn("step3_checksums", reference_ids)
        self.assertTrue(all(Path(item["path"]).is_absolute() for item in references))
        self.assertTrue(
            all(len(item["sha256"]) == 64 for item in references)
        )

    def test_normalized_evidence_is_bound_to_the_frozen_result(self):
        with tempfile.TemporaryDirectory() as directory:
            copy = Path(directory) / "study"
            shutil.copytree(STUDY_DIR, copy)
            evidence_path = copy / "evidence.json"
            evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
            evidence["quality"]["current_expected_ids"].pop()
            evidence_path.write_text(
                json.dumps(evidence, indent=2) + "\n", encoding="utf-8"
            )

            with self.assertRaisesRegex(
                verify.AuditError, "current quality row ids"
            ):
                verify.verify_study(copy)

    def test_reports_use_current_study_authority_and_required_limits(self):
        readme = (STUDY_DIR / "README.md").read_text(encoding="utf-8")
        report = (STUDY_DIR / "FINAL-REPORT.md").read_text(encoding="utf-8")
        combined = readme + "\n" + report

        self.assertIn("sequential direct Codex", combined)
        self.assertIn("concurrent Work Leaf", combined)
        self.assertIn("/fork", combined)
        self.assertIn("supplemental", combined)
        self.assertIn("Formal quality equivalence is unavailable", report)
        self.assertIn("Cross-project generalization is deferred", report)
        self.assertNotIn("step" + "227", combined.lower())
        self.assertNotIn("later-contract " + "evaluator", combined.lower())

    def test_cli_is_deterministic_and_reports_success(self):
        first = subprocess.run(
            [sys.executable, str(STUDY_DIR / "verify.py")],
            cwd=STUDY_DIR,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        second = subprocess.run(
            [sys.executable, str(STUDY_DIR / "verify.py")],
            cwd=STUDY_DIR,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(first.stdout, second.stdout)
        self.assertIn("efficiency causal study: PASS", first.stdout)
        self.assertIn("quality: sequential 2.00/3 (n=6); Work Leaf 2.25/3 (n=4)", first.stdout)


if __name__ == "__main__":
    unittest.main()
