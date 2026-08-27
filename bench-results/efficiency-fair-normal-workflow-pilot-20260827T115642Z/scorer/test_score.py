#!/usr/bin/env python3

import importlib.util
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


SCORER_PATH = Path(__file__).with_name("score.py")
SPEC = importlib.util.spec_from_file_location("fair_pilot_score", SCORER_PATH)
assert SPEC is not None and SPEC.loader is not None
SCORER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SCORER)


def scored_run(
    workflow: str,
    features: int,
    raw: int,
    uncached: int,
    usable: bool = True,
    checks=None,
):
    if checks is None:
        checks = {
            "visual": "pass" if features >= 1 else "fail",
            "status": "pass" if features >= 2 else "fail",
            "completion": "pass" if features >= 3 else "fail",
        }
    return {
        "workflow": workflow,
        "workflow_result": "pass",
        "completed_features": features,
        "checks": checks,
        "measurement": {
            "usable": usable,
            "usage": {
                "raw_input_plus_output": raw,
                "uncached_input_plus_output": uncached,
            },
        },
    }


class ScoreTests(unittest.TestCase):
    def test_original_quality_surface_has_three_features_and_no_fork(self):
        self.assertEqual(set(SCORER.FIXTURES), {"visual", "status", "completion"})
        for fixture, _ in SCORER.FIXTURES.values():
            text = (SCORER.SCRIPT_DIR / "fixtures" / fixture).read_text(encoding="utf-8")
            self.assertNotIn("/fork", text)

    def test_equal_quality_pair_reports_token_reduction(self):
        comparison = SCORER.build_comparison(
            [
                scored_run("direct-sequential", 3, 1_000, 800),
                scored_run("work-leaf-concurrent", 3, 400, 300),
            ]
        )
        self.assertTrue(comparison["usable"])
        self.assertEqual(comparison["raw_tokens"]["work_leaf_reduction_percent"], 60.0)
        self.assertEqual(
            comparison["uncached_tokens"]["work_leaf_reduction_percent"], 62.5
        )

    def test_unequal_quality_pair_does_not_support_efficiency_claim(self):
        comparison = SCORER.build_comparison(
            [
                scored_run("direct-sequential", 3, 1_000, 800),
                scored_run("work-leaf-concurrent", 2, 400, 300),
            ]
        )
        self.assertFalse(comparison["usable"])
        self.assertIn("different numbers", " ".join(comparison["reasons"]))

    def test_same_count_but_different_features_does_not_support_efficiency_claim(self):
        comparison = SCORER.build_comparison(
            [
                scored_run(
                    "direct-sequential",
                    2,
                    1_000,
                    800,
                    checks={"visual": "pass", "status": "pass", "completion": "fail"},
                ),
                scored_run(
                    "work-leaf-concurrent",
                    2,
                    400,
                    300,
                    checks={"visual": "pass", "status": "fail", "completion": "pass"},
                ),
            ]
        )
        self.assertFalse(comparison["usable"])
        self.assertIn("different requested features", " ".join(comparison["reasons"]))

    def test_quality_mismatch_report_keeps_the_observed_token_arithmetic(self):
        runs = [
            scored_run("direct-sequential", 3, 1_000, 800),
            scored_run("work-leaf-concurrent", 2, 400, 300),
        ]
        report = SCORER.markdown_report(
            {"runs": runs, "comparison": SCORER.build_comparison(runs)}
        )
        self.assertIn("1,000", report)
        self.assertIn("400", report)
        self.assertIn("not a comparable-output efficiency claim", report)

    def test_incomplete_measurement_does_not_support_efficiency_claim(self):
        comparison = SCORER.build_comparison(
            [
                scored_run("direct-sequential", 3, 1_000, 800),
                scored_run("work-leaf-concurrent", 3, 400, 300, usable=False),
            ]
        )
        self.assertFalse(comparison["usable"])
        self.assertIn("Work Leaf token measurement is not usable", comparison["reasons"])

    def test_measurement_uses_and_cross_checks_observer_total_workflow_usage(self):
        usage = {
            "input_tokens": 900,
            "cached_input_tokens": 400,
            "uncached_input_tokens": 500,
            "output_tokens": 100,
            "reasoning_output_tokens": 50,
            "raw_input_plus_output": 1_000,
            "uncached_input_plus_output": 600,
        }
        report = {
            "measurement_status": "complete",
            "agent_model": "gpt-5.5",
            "agent_reasoning_effort": "xhigh",
            "total_workflow_usage": usage,
        }
        with tempfile.TemporaryDirectory() as temporary:
            artifact = Path(temporary)
            observation = artifact / "observation"
            observation.mkdir()
            (observation / "analysis.json").write_text(
                json.dumps(
                    {
                        "capture_complete": True,
                        "model_strata": [
                            {
                                "model": "gpt-5.5",
                                "effort": "xhigh",
                                "thread_count": 1,
                            }
                        ],
                        "usage_scopes": {"total_workflow": usage},
                    }
                ),
                encoding="utf-8",
            )
            observed = SCORER.measurement(report, artifact, "gpt-5.5", "xhigh")
            self.assertTrue(observed["usable"])
            self.assertEqual(observed["usage"], usage)

            report["total_workflow_usage"] = {**usage, "input_tokens": 901}
            observed = SCORER.measurement(report, artifact, "gpt-5.5", "xhigh")
            self.assertFalse(observed["usable"])
            self.assertIn("does not match", " ".join(observed["reasons"]))

    def test_saved_bundle_and_untracked_diff_reconstruct_the_candidate(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.mkdir()

            def git(*args, environment=None):
                return subprocess.run(
                    ["git", *args],
                    cwd=source,
                    env=environment,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=True,
                )

            git("init", "-q")
            git("config", "user.email", "pilot@example.com")
            git("config", "user.name", "Pilot Scorer")
            (source / "base.txt").write_text("base\n", encoding="utf-8")
            git("add", "base.txt")
            git("commit", "-q", "-m", "ADD base fixture")
            base_commit = git("rev-parse", "HEAD").stdout.strip()

            (source / "committed.txt").write_text("committed\n", encoding="utf-8")
            git("add", "committed.txt")
            git("commit", "-q", "-m", "ADD committed candidate file")

            artifact = root / "artifact"
            snapshot = artifact / "patches" / "pass"
            snapshot.mkdir(parents=True)
            git("bundle", "create", str(snapshot / "commits.bundle"), f"{base_commit}..HEAD")
            (snapshot / "index.diff").write_text("", encoding="utf-8")

            (source / "untracked.txt").write_text("untracked\n", encoding="utf-8")
            snapshot_index = root / "snapshot.index"
            shutil.copyfile(source / ".git" / "index", snapshot_index)
            environment = dict(os.environ)
            environment["GIT_INDEX_FILE"] = str(snapshot_index)
            git("add", "-N", "--", ".", environment=environment)
            diff = git("diff", "--binary", environment=environment).stdout
            (snapshot / "worktree.diff").write_text(diff, encoding="utf-8")

            report = artifact / "report.json"
            report.write_text('{"workflow_result":"pass"}\n', encoding="utf-8")
            checkout, final_commit, notes = SCORER.prepare_checkout(
                {
                    "id": "candidate",
                    "artifact": str(artifact),
                    "report": str(report),
                },
                source,
                base_commit,
                root / "score-work",
                30,
            )
            self.assertNotEqual(final_commit, base_commit)
            self.assertEqual(notes, [])
            self.assertEqual(
                (checkout / "committed.txt").read_text(encoding="utf-8"), "committed\n"
            )
            self.assertEqual(
                (checkout / "untracked.txt").read_text(encoding="utf-8"), "untracked\n"
            )


if __name__ == "__main__":
    unittest.main()
