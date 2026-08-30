#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
SCRIPT = STUDY / "analyze-combined.py"


def load_module():
    specification = importlib.util.spec_from_file_location("analyze_combined", SCRIPT)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load {SCRIPT}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


class CombinedAnalysisTest(unittest.TestCase):
    def test_combined_control_passes_every_infrastructure_gate(self):
        evidence = load_module().build_evidence()

        self.assertEqual(
            evidence["status"],
            "complete_controls_with_bounded_normal_endpoint",
        )
        self.assertEqual(
            evidence["normal_endpoint_accounting"]["unresolved_provider_responses"],
            35,
        )
        self.assertEqual(evidence["rollout_integrity"]["hash_mismatches"], [])
        self.assertEqual(len(evidence["activation"]), 3)
        self.assertTrue(all(row["passed"] for row in evidence["activation"]))
        self.assertEqual(
            [row["continued_response_count"] for row in evidence["activation"]],
            [21, 24, 11],
        )
        self.assertTrue(all(row["timeout_count"] == 0 for row in evidence["activation"]))
        self.assertTrue(
            all(row["app_server_incremental_usage_reconciles"] for row in evidence["activation"])
        )
        self.assertEqual(
            sum(
                len(row["app_server_usage_reconciliation"]["derived_total_token_anomalies"])
                for row in evidence["activation"]
            ),
            1,
        )

    def test_two_factor_interaction_is_not_reported_as_additive(self):
        evidence = load_module().build_evidence()
        interaction = evidence["factorial"]["raw_input_plus_output"]

        self.assertAlmostEqual(interaction["combined_mean"], 19_399_622.0)
        self.assertAlmostEqual(interaction["combined_minus_normal"], 1_928_090.0)
        self.assertAlmostEqual(interaction["interaction"], -4_867_190.0)
        self.assertAlmostEqual(
            interaction["combined_fraction_of_endpoint_gap_percent"],
            10.341139963424038,
        )
        self.assertFalse(evidence["causal_summary"]["separate_effects_are_additive"])
        bounded = evidence["bounded_normal_comparison"]
        self.assertAlmostEqual(bounded["combined_minus_normal_raw_tokens"]["lower"], -3_905_243.333333332)
        self.assertAlmostEqual(bounded["combined_minus_normal_raw_tokens"]["upper"], 1_928_090.0)
        self.assertAlmostEqual(bounded["raw_interaction_tokens"]["lower"], -4_867_190.0)
        self.assertAlmostEqual(bounded["raw_interaction_tokens"]["upper"], 966_143.3333333321)
        self.assertFalse(bounded["direction_proven"])

    def test_quality_and_same_cli_counterchecks_are_retained(self):
        evidence = load_module().build_evidence()

        self.assertEqual(
            evidence["groups"]["combined_work_leaf"]["completed_features"], 8
        )
        self.assertEqual(
            evidence["counterchecks"]["same_cli_direct_vs_combined"]["direct_features"],
            9,
        )
        self.assertEqual(
            evidence["counterchecks"]["same_cli_direct_vs_combined"]["combined_features"],
            8,
        )
        self.assertGreater(
            evidence["counterchecks"]["review_rounds"]["combined_mean"],
            evidence["counterchecks"]["review_rounds"]["direct_mean"],
        )

    def test_residual_gap_is_grounded_in_provider_actions_and_stages(self):
        evidence = load_module().build_evidence()
        actions = evidence["counterchecks"]["provider_actions"]
        residual = evidence["residual_decomposition"]

        self.assertAlmostEqual(actions["direct_mean"]["exec_command"], 3805 / 6)
        self.assertAlmostEqual(actions["direct_mean"]["apply_patch"], 382 / 6)
        self.assertAlmostEqual(actions["combined_mean"]["exec_command"], 1287 / 3)
        self.assertAlmostEqual(actions["combined_mean"]["apply_patch"], 12 / 3)
        self.assertAlmostEqual(actions["combined_structured_edit_mean"], 41 / 3)
        self.assertAlmostEqual(actions["combined_total_write_submission_mean"], 53 / 3)
        self.assertAlmostEqual(
            residual["input_gap_factorization"]["fewer_usage_changes_share_percent"],
            76.61569251682681,
        )
        self.assertAlmostEqual(
            residual["direct_minus_combined_stage_usage"]["implementation"][
                "raw_input_plus_output"
            ],
            13_455_639.5,
        )
        ranks = evidence["counterchecks"]["raw_rank_separation"]
        self.assertAlmostEqual(ranks["direct_vs_normal"]["p_value"], 1 / 924)
        self.assertAlmostEqual(ranks["direct_vs_combined"]["p_value"], 1 / 84)
        self.assertAlmostEqual(ranks["same_cli_direct_vs_combined"]["p_value"], 1 / 20)


if __name__ == "__main__":
    unittest.main()
