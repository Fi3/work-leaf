#!/usr/bin/env python3

import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("score_study", ROOT / "score-study.py")
assert SPEC is not None and SPEC.loader is not None
SCORE_STUDY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SCORE_STUDY)


class ScoreStudyTests(unittest.TestCase):
    def test_build_result_preserves_independent_control_runs(self) -> None:
        scorer = SCORE_STUDY.load_frozen_scorer()
        manifest = {
            "study": "study",
            "base_commit": "base",
            "task_list_sha256": "task",
            "model": "gpt-5.5",
            "reasoning_effort": "xhigh",
        }
        runs = [{"id": "one"}, {"id": "two"}, {"id": "three"}]

        result = SCORE_STUDY.build_result(scorer, manifest, runs)

        self.assertEqual(result["runs"], runs)
        self.assertEqual(result["model"], "gpt-5.5")
        self.assertEqual(result["reasoning_effort"], "xhigh")
        self.assertEqual(
            result["fixtures"],
            {
                "completion": "4d6a19f4c6f515b9a97f184a056d08dc2882041d40d26dd82011180a949c5c87",
                "status": "5f5295c9dec6be20abb28827b68e84342110fb748aa0cc887d8b58ffa3c6e6b5",
                "visual": "5ce0782e37b8672e04d8016ca3fbec9caa12f6f9cfaf431cac847c1f6e6a26ca",
            },
        )


if __name__ == "__main__":
    unittest.main()
