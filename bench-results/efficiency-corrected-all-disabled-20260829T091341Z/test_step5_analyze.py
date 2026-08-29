import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def load_analyzer():
    path = ROOT / "step5-analyze.py"
    spec = importlib.util.spec_from_file_location("step5_analyze", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Step5AnalyzeTests(unittest.TestCase):
    def test_keeps_independent_groups_and_all_real_workflow_outcomes(self):
        evidence = load_analyzer().build_final_evidence(bootstrap_samples=2_000)

        self.assertEqual(
            {name: group["observations"] for name, group in evidence["groups"].items()},
            {"direct": 6, "normal_work_leaf": 5, "all_disabled_work_leaf": 6},
        )
        self.assertIn(
            "step4-normal-002", evidence["groups"]["normal_work_leaf"]["run_ids"]
        )
        self.assertIn(
            "step4-control-003",
            evidence["groups"]["all_disabled_work_leaf"]["run_ids"],
        )
        self.assertEqual(
            evidence["excluded_infrastructure_attempts"][0]["run_id"],
            "step4-normal-001",
        )

    def test_separates_overall_saving_from_mechanism_attribution(self):
        evidence = load_analyzer().build_final_evidence(bootstrap_samples=2_000)
        overall = evidence["comparisons"]["direct_minus_normal_work_leaf"]
        mechanism = evidence["comparisons"][
            "all_disabled_minus_normal_work_leaf"
        ]

        self.assertGreater(overall["raw_mean_difference_interval"]["lower"], 0)
        self.assertTrue(overall["raw_average_saving_proven_under_cap"])
        self.assertTrue(
            evidence["conclusions"][
                "collected_sample_average_raw_saving_survives_conservative_cap"
            ]
        )
        self.assertFalse(
            evidence["conclusions"][
                "population_average_raw_saving_statistically_established"
            ]
        )
        self.assertLess(mechanism["raw_mean_difference_interval"]["lower"], 0)
        self.assertGreater(mechanism["raw_mean_difference_interval"]["upper"], 0)
        self.assertFalse(mechanism["combined_mechanism_effect_proven"])

    def test_quality_and_full_feature_subset_remain_visible(self):
        evidence = load_analyzer().build_final_evidence(bootstrap_samples=2_000)

        self.assertEqual(
            evidence["groups"]["direct"]["total_completed_features"], 17
        )
        self.assertEqual(
            evidence["groups"]["normal_work_leaf"]["total_completed_features"],
            13,
        )
        self.assertEqual(
            evidence["groups"]["all_disabled_work_leaf"][
                "total_completed_features"
            ],
            12,
        )
        subset = evidence["quality_matched_full_feature_subset"]
        self.assertEqual(subset["direct_observations"], 5)
        self.assertEqual(subset["normal_work_leaf_observations"], 3)
        self.assertGreater(
            subset["raw_mean_difference_interval"]["lower"], 0
        )


if __name__ == "__main__":
    unittest.main()
