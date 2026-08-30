#!/usr/bin/env python3

import importlib.util
import json
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
SCRIPT = STUDY / "score.py"


def load_module():
    specification = importlib.util.spec_from_file_location("mechanism_score", SCRIPT)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load {SCRIPT}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


class ScoreTest(unittest.TestCase):
    def test_batch_one_manifest_contains_only_admitted_runs(self):
        manifest = json.loads((STUDY / "batch-1-score-manifest.json").read_text())
        self.assertEqual(manifest["base_commit"], "c92a0b7060a36eac6db2d869b85e589a7a9480f9")
        self.assertEqual(manifest["model"], "gpt-5.5")
        self.assertEqual(manifest["reasoning_effort"], "xhigh")
        self.assertEqual(
            [(run["id"], run["condition"]) for run in manifest["runs"]],
            [
                ("compact-direct-001", "compact-direct"),
                ("sequential-work-leaf-combined-001", "sequential-work-leaf-combined"),
                ("compact-direct-002", "compact-direct"),
            ],
        )

    def test_frozen_scorer_writes_logs_to_this_study(self):
        module = load_module()
        scorer = module.load_frozen_scorer()
        self.assertEqual(scorer.STUDY_DIR, STUDY)
        self.assertEqual(
            module.sha256(module.FROZEN_SCORER),
            "c0a4e951e96d7da53a6d414a7677176183cae6e30d0bbfab92069d5082865162",
        )


if __name__ == "__main__":
    unittest.main()
