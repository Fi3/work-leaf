#!/usr/bin/env python3

import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from argparse import Namespace
from unittest.mock import Mock, patch


def load_module():
    path = Path(__file__).with_name("first_batch.py")
    spec = importlib.util.spec_from_file_location("first_batch", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class FirstBatchTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.module = load_module()

    def manifest(self):
        return {"study": "pair-study", "bin_dir": "/frozen/bin", "provider_dir": "/frozen/provider"}

    def test_schedule_contains_one_direct_and_one_normal_work_leaf(self):
        rows = self.module.schedule(Path("/batch"), Path("/source"), Path("/runtime"))
        self.assertEqual([row["condition"] for row in rows], ["direct", "work-leaf"])
        self.assertEqual([Path(row["driver"]).name for row in rows], ["bench-three-features-sequential", "bench-three-features"])
        self.assertEqual(len({row["run_id"] for row in rows}), 2)

    def test_only_the_declared_pair_is_admissible(self):
        for rows in ([], [{"condition": "direct"}], [{"condition": "direct"}, {"condition": "direct"}],
                     [{"condition": "direct"}, {"condition": "work-leaf"}, {"condition": "work-leaf"}]):
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                self.module.validate_schedule(rows)

    def test_environment_clears_experiments_and_credentials_but_keeps_auth_home(self):
        inherited = {"PATH": "/usr/bin", "CODEX_HOME": "/existing/auth-home", "OPENAI_API_KEY": "not-a-real-key",
                     "CODEX_API_KEY": "not-a-real-key", "OPENAI_BASE_URL": "http://invalid",
                     "WORK_LEAF_EXPERIMENT_ANYTHING": "on", "WORK_LEAF_BENCH_TIMEOUT_SECS": "1",
                     "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME": "wait-for-usage"}
        row = {"condition": "work-leaf", "run_id": "work-leaf-001", "runtime_dir": "/runtime/wl", "results_dir": "/batch/wl"}
        env = self.module.run_environment(self.manifest(), row, inherited)
        for key in ("OPENAI_API_KEY", "CODEX_API_KEY", "OPENAI_BASE_URL", "WORK_LEAF_EXPERIMENT_ANYTHING"):
            self.assertNotIn(key, env)
        self.assertEqual(env["CODEX_HOME"], inherited["CODEX_HOME"])
        self.assertEqual(env["PATH"], "/frozen/provider:/usr/bin")
        self.assertEqual(env["WORK_LEAF_BENCH_TIMEOUT_SECS"], "7200")
        self.assertEqual(env["WORK_LEAF_BENCH_BUSY_STALL_SECS"], "1800")
        self.assertEqual(env["WORK_LEAF_BENCH_IDLE_STALL_SECS"], "300")
        self.assertEqual(env["WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS"], "1000")
        self.assertEqual(env["WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME"], "forward")
        self.assertEqual(env["WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE"], "1")
        self.assertEqual(env["WORK_LEAF_BENCH_DISABLE_TMUX_SUPERVISOR"], "1")

    def test_direct_uses_normal_stage_limits_and_no_work_leaf_interruption_options(self):
        row = {"condition": "direct", "run_id": "direct-001", "runtime_dir": "/runtime/direct", "results_dir": "/batch/direct"}
        env = self.module.run_environment(self.manifest(), row, {"PATH": "/usr/bin", "WORK_LEAF_DIRECT_BENCH_REVIEW_ROUNDS": "1"})
        self.assertEqual(env["WORK_LEAF_DIRECT_BENCH_TIMEOUT_SECS"], "7200")
        self.assertEqual(env["WORK_LEAF_DIRECT_BENCH_REVIEW_ROUNDS"], "0")
        self.assertEqual(env["WORK_LEAF_DIRECT_BENCH_MODEL"], "gpt-5.5")
        self.assertEqual(env["WORK_LEAF_DIRECT_BENCH_REASONING_EFFORT"], "xhigh")
        self.assertEqual(env["WORK_LEAF_DIRECT_BENCH_SANDBOX"], "workspace-write")
        self.assertEqual(env["WORK_LEAF_DIRECT_BENCH_REVIEW_SANDBOX"], "read-only")
        self.assertEqual(env["WORK_LEAF_DIRECT_BENCH_LINEARIZE_SANDBOX"], "danger-full-access")
        self.assertNotIn("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", env)
        self.assertNotIn("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", env)

    def test_one_shot_claim_cannot_be_reused_even_without_a_result(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.module.claim_batch(root)
            with self.assertRaises(FileExistsError):
                self.module.claim_batch(root)
            self.assertFalse((root / "FIRST-BATCH-RESULT.json").exists())

    def mock_git(self):
        return patch.object(self.module, "git", side_effect=lambda _source, *args: "1" * 40 if args[0] == "rev-parse" else "")

    def prepared_fixture(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        source = root / "source"
        binaries = root / "binaries"
        source.mkdir()
        binaries.mkdir()
        for name in self.module.DRIVERS:
            (source / name).write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            (source / name).chmod(0o700)
        for name in ("work-leaf", "work-leaf-orchestrator", "bench-observer"):
            (binaries / name).write_bytes(b"fixture executable\n")
            (binaries / name).chmod(0o700)
        args = Namespace(batch_root=root / "batch", source_repo=source, bin_dir=binaries,
                         observer_bin=binaries / "bench-observer", runtime_root=root / "runtime",
                         subscription_wrapper=Path(__file__).with_name("subscription-codex"),
                         evidence=[], identity_file=[], task_list_sha256="a" * 64)
        with self.mock_git():
            manifest = self.module.prepare(args)
        return args.batch_root, manifest

    def test_launch_order_uses_the_supplied_random_shuffle(self):
        rows = self.module.schedule(Path("/batch"), Path("/source"), Path("/runtime"))
        rng = Mock()
        rng.shuffle.side_effect = lambda values: values.reverse()
        self.assertEqual(self.module.choose_launch_order(rows, rng), ["work-leaf-001", "direct-001"])
        rng.shuffle.assert_called_once()

    def test_prepare_freezes_launch_order_before_any_provider_work(self):
        batch, manifest = self.prepared_fixture()
        frozen = json.loads((batch / "SCHEDULE.json").read_text())
        self.assertEqual(sorted(manifest["launch_order"]), ["direct-001", "work-leaf-001"])
        self.assertEqual(frozen["launch_order"], manifest["launch_order"])
        self.assertFalse((batch / "RUN-ONCE").exists())
        with self.mock_git():
            self.module.verify_manifest(batch)

    def test_spawn_failure_retains_both_rows_and_stops_new_launches(self):
        batch, _manifest = self.prepared_fixture()
        launcher = Mock(side_effect=OSError("fixture setup failure"))
        with self.mock_git():
            report = self.module.run_batch(batch, popen=launcher)
        self.assertEqual(launcher.call_count, 1)
        self.assertEqual(len(report["runs"]), 2)
        self.assertEqual(sorted(row["launch_status"] for row in report["runs"]),
                         ["failed_to_start", "not_launched_after_infrastructure_failure"])
        self.assertTrue(report["collection_paused"])
        self.assertEqual(len(json.loads((batch / "score-manifest.json").read_text())["runs"]), 2)

    def test_observed_nonzero_exit_stops_new_launches_even_with_a_saved_report(self):
        batch, manifest = self.prepared_fixture()
        first = next(row for row in manifest["schedule"] if row["run_id"] == manifest["launch_order"][0])
        process = Mock(pid=999999, poll=Mock(return_value=1), wait=Mock(return_value=1))

        def completed_failure(*_args, **_kwargs):
            report_path = Path(first["report"])
            report_path.parent.mkdir(parents=True, exist_ok=True)
            report_path.write_text("{}", encoding="utf-8")
            return process

        launcher = Mock(side_effect=completed_failure)
        with self.mock_git():
            report = self.module.run_batch(batch, popen=launcher)
        self.assertEqual(launcher.call_count, 1)
        self.assertEqual(sorted(row["launch_status"] for row in report["runs"]),
                         ["completed", "not_launched_after_infrastructure_failure"])
        self.assertEqual(next(row for row in report["runs"] if row["id"] == first["run_id"])["launcher_exit_code"], 1)

    def test_verify_rejects_missing_required_hash_entries(self):
        batch, manifest = self.prepared_fixture()
        manifest["files"] = [row for row in manifest["files"] if row["role"] != "observer"]
        (batch / "FIRST-BATCH-MANIFEST.json").write_text(json.dumps(manifest))
        with self.mock_git(), self.assertRaises(ValueError):
            self.module.verify_manifest(batch)

    def test_verify_rejects_changed_runner_identity(self):
        batch, manifest = self.prepared_fixture()
        manifest["runner_sha256"] = "0" * 64
        (batch / "FIRST-BATCH-MANIFEST.json").write_text(json.dumps(manifest))
        with self.mock_git(), self.assertRaises(ValueError):
            self.module.verify_manifest(batch)

    def test_verify_rejects_changed_frozen_binary(self):
        batch, manifest = self.prepared_fixture()
        observer = Path(manifest["bin_dir"]) / "bench-observer"
        observer.chmod(0o700)
        observer.write_bytes(b"changed fixture")
        with self.mock_git(), self.assertRaises(ValueError):
            self.module.verify_manifest(batch)

    def test_manifest_declares_a_generous_whole_batch_wall_bound(self):
        _batch, manifest = self.prepared_fixture()
        self.assertEqual(manifest["supervisor_wall_timeout_seconds"], 86400)


if __name__ == "__main__":
    unittest.main()
