"""Regression: equal input objects must not fail because of mixed representations."""
import json
from pathlib import Path
import unittest


class InputComparisonTests(unittest.TestCase):
    def test_corrected_actual_report_uses_matching_representations(self):
        root = Path(__file__).resolve().parent
        proof = json.loads((root / "INPUT-VERIFIER-CORRECTION-002.json").read_text())
        report = json.loads((root / "ACTUAL-INPUT-COMPARISON-002.json").read_text())
        self.assertTrue(proof["base_objects_equal"])
        self.assertTrue(proof["developer_content_objects_equal"])
        self.assertTrue(report["base_equal"])
        self.assertTrue(report["developer_equal"])
        for key in ("base_sha256", "developer_sha256", "developer_bytes"):
            self.assertEqual(report["reference"][key], report["diagnostic"][key])
        self.assertEqual(report["diagnostic"]["base_sha256"], proof["diagnostic_base_canonical_sha"])
        self.assertEqual(report["diagnostic"]["developer_sha256"], proof["joined_developer_sha"][1])
        self.assertEqual(report["diagnostic"]["developer_bytes"], proof["joined_developer_bytes"][1])

    def test_preliminary_representation_error_is_retained(self):
        root = Path(__file__).resolve().parent
        prior = json.loads((root / "ACTUAL-INPUT-COMPARISON-002-PRELIMINARY.json").read_text())
        self.assertFalse(prior["base_equal"])
        self.assertFalse(prior["developer_equal"])


if __name__ == "__main__":
    unittest.main()
