#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
MODULE_PATH = STUDY / "endpoint_audit.py"


def load_module():
    spec = importlib.util.spec_from_file_location("endpoint_audit", MODULE_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class EndpointAuditTest(unittest.TestCase):
    def test_frozen_endpoint_result(self):
        evidence = load_module().build_evidence()

        self.assertEqual(evidence["status"], "complete")
        self.assertEqual(evidence["fairness"]["failed_checks"], [])
        self.assertEqual(evidence["accounting"]["inexact_runs"], [])
        self.assertEqual(evidence["groups"]["direct"]["run_count"], 3)
        self.assertEqual(evidence["groups"]["work_leaf"]["run_count"], 3)
        self.assertEqual(evidence["groups"]["direct"]["completed_features"], 8)
        self.assertEqual(evidence["groups"]["work_leaf"]["completed_features"], 8)
        self.assertAlmostEqual(
            evidence["groups"]["direct"]["mean_raw_tokens"],
            35_196_786.333333336,
        )
        self.assertAlmostEqual(
            evidence["groups"]["work_leaf"]["mean_raw_tokens"],
            13_989_717.666666666,
        )
        self.assertAlmostEqual(
            evidence["comparison"]["raw_reduction_percent"],
            60.252855092575274,
        )
        self.assertAlmostEqual(
            evidence["comparison"]["uncached_reduction_percent"],
            37.58259268636313,
        )
        self.assertTrue(evidence["comparison"]["complete_raw_separation"])
        self.assertTrue(evidence["comparison"]["complete_uncached_separation"])


if __name__ == "__main__":
    unittest.main()
