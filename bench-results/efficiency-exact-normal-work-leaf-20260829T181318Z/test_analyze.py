#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path


STUDY_DIR = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("exact_normal_analyze", STUDY_DIR / "analyze.py")
assert SPEC is not None and SPEC.loader is not None
ANALYZE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ANALYZE)


class ExactNormalAnalysisTests(unittest.TestCase):
    def test_final_evidence_keeps_quality_and_missing_usage_visible(self) -> None:
        evidence = ANALYZE.build_evidence()

        self.assertEqual(
            evidence["status"],
            "complete_with_bounded_work_leaf_usage",
        )

        direct = evidence["groups"]["direct"]
        work_leaf = evidence["groups"]["normal_work_leaf"]
        comparison = evidence["comparisons"]["direct_minus_normal_work_leaf"]

        self.assertEqual(direct["observations"], 6)
        self.assertEqual(direct["total_completed_features"], 17)
        self.assertEqual(work_leaf["observations"], 6)
        self.assertEqual(work_leaf["total_completed_features"], 13)
        self.assertEqual(work_leaf["exact_token_observations"], 1)
        self.assertEqual(work_leaf["bounded_token_observations"], 5)
        self.assertEqual(work_leaf["missing_provider_responses"], 35)
        self.assertEqual(
            work_leaf["raw_token_mean_interval"],
            {"lower": 17_471_532.0, "upper": 23_304_865.333333332},
        )
        self.assertEqual(
            evidence["accounting"]["maximum_raw_tokens_per_unresolved_response"],
            1_000_000,
        )
        self.assertEqual(
            evidence["accounting"]["effective_context_window_tokens"],
            258_400,
        )
        self.assertEqual(
            evidence["accounting"]["maximum_output_tokens"],
            128_000,
        )
        self.assertEqual(
            evidence["accounting"]["maximum_observed_last_response_raw_tokens"],
            180_949,
        )
        self.assertEqual(
            evidence["accounting"]["maximum_single_response_raw_tokens"],
            386_400,
        )
        self.assertEqual(
            evidence["accounting"]["response_cap_headroom_tokens"],
            613_600,
        )
        self.assertGreater(comparison["raw_tokens"]["lower"], 0)
        self.assertLess(comparison["uncached_tokens"]["lower"], 0)
        self.assertTrue(
            evidence["conclusions"]["raw_saving_proven_under_conservative_bound"]
        )
        self.assertFalse(
            evidence["conclusions"]["uncached_saving_proven_under_conservative_bound"]
        )
        self.assertFalse(evidence["conclusions"]["equal_quality_average_claim_supported"])

    def test_only_full_feature_runs_enter_quality_matched_subset(self) -> None:
        evidence = ANALYZE.build_evidence()
        subset = evidence["quality_matched_full_feature_subset"]

        self.assertEqual(subset["direct"]["observations"], 5)
        self.assertEqual(subset["normal_work_leaf"]["observations"], 2)
        self.assertEqual(
            subset["normal_work_leaf"]["run_ids"],
            ["exact-normal-002", "exact-normal-003"],
        )
        self.assertEqual(subset["normal_work_leaf"]["missing_provider_responses"], 27)
        self.assertGreater(
            subset["direct_minus_normal_work_leaf"]["raw_tokens"]["lower"], 0
        )


if __name__ == "__main__":
    unittest.main()
