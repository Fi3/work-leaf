#!/usr/bin/env python3
"""Offline checks for the separately frozen work-unit phase supervisor."""

import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parent
LEGACY_SHA256 = "3927f097fbb1efdf195c21fc22c69d2abf5766f2bfc99bee9916a05271f34dfd"


def load():
    spec = importlib.util.spec_from_file_location("work_unit_runner", ROOT / "runner_work_units.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class WorkUnitRunnerTests(unittest.TestCase):
    def setUp(self):
        self.runner = load()

    def plan(self):
        conditions = ["control", "buildable-work-unit-incremental", "control"]
        return {"phase": "work-units", "phase_kind": "confirmation", "randomization": {
            "unit": "workflow", "scheme": "within_block", "method": "synthetic allocation",
            "mixed_waves": True, "block_condition_counts": {
                "block-1": {"control": 2, "buildable-work-unit-incremental": 1}}},
            "runs": [{"run_id": f"unit-{index}", "condition": condition, "wave": 1,
                      "block_id": "block-1"} for index, condition in enumerate(conditions)]}

    def test_old_frozen_supervisor_is_not_modified(self):
        self.assertEqual(hashlib.sha256((ROOT / "runner.py").read_bytes()).hexdigest(), LEGACY_SHA256)

    def test_only_new_work_unit_conditions_and_v2_schema_are_supported(self):
        self.assertEqual(self.runner.EXPERIMENT_SCHEMA, "work-leaf-bench-experiment-v2")
        self.assertEqual(self.runner.CONDITIONS, ("control", "buildable-work-unit-incremental"))
        self.runner.validate_plan(self.plan())
        for rejected in ("ack-validation-unlimited", "command-guidance-neutral", "direct"):
            plan = self.plan()
            plan["runs"][1]["condition"] = rejected
            with self.subTest(rejected=rejected), self.assertRaises(ValueError):
                self.runner.validate_plan(plan)

    def test_environment_keeps_all_nonfactor_settings_and_existing_subscription_home(self):
        manifest = {"provider_dir": "/frozen/provider", "bin_dir": "/frozen/bin", "study": "study"}
        row = {**self.plan()["runs"][1], "runtime_dir": "/runtime/run", "results_dir": "/results/run",
               "experiment_manifest": "/frozen/experiment.json"}
        inherited = {"PATH": "/usr/bin", "HOME": "/existing-home", "CODEX_HOME": "/existing-home/.codex",
                     "OPENAI_API_KEY": "synthetic-key", "WORK_LEAF_UNKNOWN": "remove",
                     "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS": "100000"}
        env = self.runner.run_environment(manifest, row, inherited)
        self.assertEqual(env["HOME"], inherited["HOME"])
        self.assertEqual(env["CODEX_HOME"], inherited["CODEX_HOME"])
        self.assertNotIn("OPENAI_API_KEY", env)
        self.assertNotIn("WORK_LEAF_UNKNOWN", env)
        expected = {"WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS": "1000",
                    "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME": "forward",
                    "WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE": "1",
                    "WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY": "1",
                    "WORK_LEAF_BENCH_MODEL": "gpt-5.5",
                    "WORK_LEAF_BENCH_REASONING_EFFORT": "xhigh",
                    "WORK_LEAF_BENCH_TIMEOUT_SECS": "7200",
                    "WORK_LEAF_BENCH_BUSY_STALL_SECS": "1800",
                    "WORK_LEAF_BENCH_IDLE_STALL_SECS": "300"}
        for key, value in expected.items():
            self.assertEqual(env[key], value)
        control = self.runner.run_environment(manifest, {**row, "condition": "control"}, inherited)
        self.assertEqual(env, control)

    def test_prepare_freezes_v2_before_admission_and_refuses_reuse(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, binaries = root / "source", root / "bin"
            source.mkdir()
            binaries.mkdir()
            for name in self.runner.DRIVERS:
                (source / name).write_text("#!/bin/sh\nexit 0\n")
                (source / name).chmod(0o700)
            for name in ("work-leaf", "work-leaf-orchestrator", "bench-observer"):
                (binaries / name).write_text("#!/bin/sh\nexit 0\n")
                (binaries / name).chmod(0o700)
            for name, content in (("PROTOCOL.md", "# Synthetic\n"), ("SCORER.json", "{}\n"),
                                  ("config.toml", 'model = "gpt-5.5"\n'),
                                  ("schedule.json", json.dumps(self.plan()))):
                (root / name).write_text(content)
            from argparse import Namespace
            args = Namespace(phase_root=root / "phase", source_repo=source, bin_dir=binaries,
                             observer_bin=binaries / "bench-observer", runtime_root=root / "runtime",
                             subscription_wrapper=self.runner.CANONICAL_WRAPPER, evidence=[],
                             identity_file=[], task_list_sha256="a" * 64, protocol=root / "PROTOCOL.md",
                             scorer_config=root / "SCORER.json", schedule=root / "schedule.json",
                             global_config=root / "config.toml", evidence_root=self.runner.REPO)
            with patch.object(self.runner, "git", side_effect=lambda _root, *arguments:
                              "1" * 40 if arguments[0] == "rev-parse" else ""):
                manifest = self.runner.prepare(args)
                self.runner.verify_manifest(args.phase_root)
                for row in manifest["schedule"]:
                    experiment = json.loads(Path(row["experiment_manifest"]).read_text())
                    self.assertEqual(experiment["schema"], "work-leaf-bench-experiment-v2")
                    self.assertEqual(experiment["condition"], row["condition"])
                self.assertFalse((args.phase_root / "RUN-ONCE").exists())
                with self.assertRaises(FileExistsError):
                    self.runner.prepare(args)


if __name__ == "__main__":
    unittest.main()
