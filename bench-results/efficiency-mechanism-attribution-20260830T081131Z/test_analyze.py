#!/usr/bin/env python3

import importlib.util
import math
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
SCRIPT = STUDY / "analyze.py"


def load_module():
    specification = importlib.util.spec_from_file_location("mechanism_analyze", SCRIPT)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load {SCRIPT}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


class AnalyzeTest(unittest.TestCase):
    def test_ordered_bridge_allocates_the_complete_endpoint_gap(self):
        module = load_module()
        result = module.ordered_bridge(
            {"D": 100.0, "L": 90.0, "S": 60.0, "C": 55.0, "W": 50.0}
        )
        self.assertEqual(
            [step["tokens"] for step in result["steps"]],
            [10.0, 30.0, 5.0, 5.0],
        )
        self.assertEqual(
            [step["share_of_endpoint_gap_percent"] for step in result["steps"]],
            [20.0, 60.0, 10.0, 10.0],
        )
        self.assertEqual(result["endpoint_gap"], 50.0)
        self.assertEqual(result["allocated_tokens"], 50.0)
        self.assertEqual(result["unallocated_tokens"], 0.0)

    def test_ordered_bridge_retains_negative_effects(self):
        module = load_module()
        result = module.ordered_bridge(
            {"D": 100.0, "L": 80.0, "S": 60.0, "C": 70.0, "W": 50.0}
        )
        self.assertEqual(
            [step["tokens"] for step in result["steps"]],
            [20.0, 20.0, -10.0, 20.0],
        )
        self.assertEqual(result["unallocated_tokens"], 0.0)

    def test_selected_causal_coverage_combines_only_named_bridge_steps(self):
        module = load_module()
        bridge = module.ordered_bridge(
            {"D": 100.0, "L": 90.0, "S": 60.0, "C": 55.0, "W": 50.0}
        )
        result = module.selected_causal_coverage(
            bridge,
            ("work_leaf_orchestration", "mediated_reads_and_interruption"),
        )
        self.assertEqual(result["tokens"], 35.0)
        self.assertEqual(result["share_of_endpoint_gap_percent"], 70.0)
        self.assertEqual(
            result["mechanisms"],
            ["work_leaf_orchestration", "mediated_reads_and_interruption"],
        )

    def test_selected_causal_coverage_rejects_unknown_bridge_step(self):
        module = load_module()
        bridge = module.ordered_bridge(
            {"D": 100.0, "L": 90.0, "S": 60.0, "C": 55.0, "W": 50.0}
        )
        with self.assertRaisesRegex(ValueError, "unknown bridge mechanism"):
            module.selected_causal_coverage(bridge, ("not-a-step",))

    def test_bounded_endpoint_bridge_keeps_fixed_steps_exact(self):
        module = load_module()
        result = module.bounded_endpoint_bridge(
            {"D": 100.0, "L": 90.0, "S": 60.0, "C": 55.0},
            {"lower": 50.0, "upper": 52.0},
        )
        steps = {step["name"]: step for step in result["steps"]}

        self.assertEqual(result["endpoint_gap"], {"lower": 48.0, "upper": 50.0})
        self.assertEqual(
            steps["work_leaf_orchestration"]["tokens"],
            {"lower": 30.0, "upper": 30.0},
        )
        self.assertEqual(
            steps["mediated_reads_and_interruption"]["tokens"],
            {"lower": 3.0, "upper": 5.0},
        )

        coverage = module.bounded_selected_causal_coverage(
            result,
            ("work_leaf_orchestration", "mediated_reads_and_interruption"),
        )
        self.assertEqual(coverage["tokens"], {"lower": 33.0, "upper": 35.0})
        self.assertTrue(
            math.isclose(
                coverage["share_of_endpoint_gap_percent"]["lower"],
                68.75,
            )
        )
        self.assertTrue(
            math.isclose(
                coverage["share_of_endpoint_gap_percent"]["upper"],
                70.0,
            )
        )

    def test_stage_difference_sums_to_the_group_transition(self):
        module = load_module()
        left = {
            "implementation": 70.0,
            "review": 20.0,
            "linearization": 10.0,
        }
        right = {
            "implementation": 30.0,
            "review": 15.0,
            "linearization": 12.0,
            "title": 3.0,
        }
        result = module.stage_difference(left, right)
        self.assertEqual(
            result,
            {
                "implementation": 40.0,
                "linearization": -2.0,
                "review": 5.0,
                "title": -3.0,
            },
        )
        self.assertEqual(sum(result.values()), sum(left.values()) - sum(right.values()))

    def test_exact_rank_test_reports_complete_three_by_three_separation(self):
        module = load_module()
        result = module.exact_permutation_greater(
            lower=[10.0, 11.0, 12.0], higher=[20.0, 21.0, 22.0]
        )
        self.assertEqual(result["partitions"], 20)
        self.assertTrue(result["complete_separation"])
        self.assertTrue(math.isclose(result["one_sided_p"], 0.05))


if __name__ == "__main__":
    unittest.main()
