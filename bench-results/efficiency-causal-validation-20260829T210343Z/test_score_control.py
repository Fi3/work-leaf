#!/usr/bin/env python3

import importlib.util
import json
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
SCRIPT = STUDY / "score-control.py"


def load_module():
    specification = importlib.util.spec_from_file_location("score_control", SCRIPT)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load {SCRIPT}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


class ScoreControlTest(unittest.TestCase):
    def test_manifest_scores_exactly_three_direct_read_controls(self):
        manifest = json.loads((STUDY / "control-score-manifest.json").read_text())
        self.assertEqual(manifest["base_commit"], "c92a0b7060a36eac6db2d869b85e589a7a9480f9")
        self.assertEqual(manifest["model"], "gpt-5.5")
        self.assertEqual(manifest["reasoning_effort"], "xhigh")
        self.assertEqual(
            [run["id"] for run in manifest["runs"]],
            ["direct-read-001", "direct-read-002", "direct-read-003"],
        )
        self.assertTrue(all(run["condition"] == "work-leaf-direct-read" for run in manifest["runs"]))

    def test_frozen_scorer_keeps_new_logs_in_this_study(self):
        module = load_module()
        scorer = module.load_frozen_scorer()
        self.assertEqual(scorer.STUDY_DIR, STUDY)
        self.assertEqual(
            module.sha256(module.FROZEN_SCORER),
            "c0a4e951e96d7da53a6d414a7677176183cae6e30d0bbfab92069d5082865162",
        )

    def test_manifest_scores_exactly_three_continued_response_controls(self):
        manifest = json.loads(
            (STUDY / "continued-response-score-manifest.json").read_text()
        )
        self.assertEqual(manifest["base_commit"], "c92a0b7060a36eac6db2d869b85e589a7a9480f9")
        self.assertEqual(manifest["model"], "gpt-5.5")
        self.assertEqual(manifest["reasoning_effort"], "xhigh")
        self.assertEqual(
            [run["id"] for run in manifest["runs"]],
            [
                "continued-response-001",
                "continued-response-002",
                "continued-response-003",
            ],
        )
        self.assertTrue(
            all(
                run["condition"] == "work-leaf-continued-response"
                for run in manifest["runs"]
            )
        )

    def test_manifest_scores_exactly_three_combined_controls(self):
        manifest = json.loads((STUDY / "combined-score-manifest.json").read_text())
        self.assertEqual(manifest["base_commit"], "c92a0b7060a36eac6db2d869b85e589a7a9480f9")
        self.assertEqual(manifest["model"], "gpt-5.5")
        self.assertEqual(manifest["reasoning_effort"], "xhigh")
        self.assertEqual(
            [run["id"] for run in manifest["runs"]],
            [
                "combined-control-001",
                "combined-control-002",
                "combined-control-003",
            ],
        )
        self.assertTrue(
            all(
                run["condition"] == "work-leaf-direct-read-continued-response"
                for run in manifest["runs"]
            )
        )


if __name__ == "__main__":
    unittest.main()
