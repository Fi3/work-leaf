#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
MODULE_PATH = STUDY / "decompose.py"


def load_module():
    spec = importlib.util.spec_from_file_location("decompose", MODULE_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class DecompositionTest(unittest.TestCase):
    def test_incomplete_normal_endpoint_rejects_exact_decomposition(self):
        evidence = load_module().build_evidence()

        self.assertEqual(
            evidence["status"],
            "superseded_by_bounded_endpoint_analysis",
        )
        self.assertEqual(evidence["accounting"]["unresolved_provider_responses"], 35)
        self.assertEqual(
            evidence["accounting"]["normal_work_leaf_raw_mean_interval"],
            {"lower": 17_471_532.0, "upper": 23_304_865.333333332},
        )
        current = evidence["cohorts"]["current_detailed_6_by_6"]
        work_leaf = [
            run for run in current["runs"] if run["group"] == "work_leaf"
        ]
        self.assertEqual(len(work_leaf), 6)
        self.assertTrue(
            all(run["measurement"] in {"exact", "recorded_lower_bound"} for run in work_leaf)
        )
        self.assertEqual(
            sum(run["unresolved_provider_responses"] for run in work_leaf),
            35,
        )


if __name__ == "__main__":
    unittest.main()
