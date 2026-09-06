#!/usr/bin/env python3
"""Offline regression checks for the mechanism-study phase supervisor."""

from argparse import Namespace
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch


def load_runner():
    spec = importlib.util.spec_from_file_location("mechanism_runner", Path(__file__).with_name("runner.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class RunnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.runner = load_runner()

    def plan(self, waves=1):
        return {"phase": "screen", "phase_kind": "screening", "randomization": {
                "unit": "workflow", "scheme": "within_block", "method": "synthetic-explicit-allocation",
                "mixed_waves": True, "block_condition_counts": {
                    f"block-{wave}": {condition: 1 for condition in self.runner.CONDITIONS}
                    for wave in range(1, waves + 1)}}, "runs": [
            {"run_id": f"screen-{wave}-{index}", "condition": condition, "wave": wave,
             "block_id": f"block-{wave}"}
            for wave in range(1, waves + 1)
            for index, condition in enumerate(self.runner.CONDITIONS)]}

    def fixture(self, waves=1):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        root = Path(tmp.name)
        source = root / "source"
        source.mkdir()
        binaries = root / "binaries"
        binaries.mkdir()
        for name in self.runner.DRIVERS:
            (source / name).write_text("#!/bin/sh\nexit 0\n")
            (source / name).chmod(0o700)
        for name in ("work-leaf", "work-leaf-orchestrator", "bench-observer"):
            (binaries / name).write_bytes(b"synthetic executable\n")
            (binaries / name).chmod(0o700)
        protocol = root / "PROTOCOL.md"
        protocol.write_text("# Frozen synthetic phase\n")
        scorer = root / "SCORER.json"
        scorer.write_text("{}\n")
        config = root / "config.toml"
        config.write_text('model = "gpt-5.5"\nmodel_reasoning_effort = "xhigh"\n')
        schedule = root / "schedule.json"
        schedule.write_text(json.dumps(self.plan(waves)))
        args = Namespace(phase_root=root / "phase", source_repo=source, bin_dir=binaries,
                         observer_bin=binaries / "bench-observer", runtime_root=root / "runtime",
                         subscription_wrapper=self.runner.CANONICAL_WRAPPER, evidence=[],
                         identity_file=[], task_list_sha256="a" * 64, protocol=protocol,
                         scorer_config=scorer, schedule=schedule, global_config=config,
                         evidence_root=self.runner.REPO)
        with self.mock_git():
            manifest = self.runner.prepare(args)
        return args, manifest

    def mock_git(self):
        return patch.object(self.runner, "git", side_effect=lambda _root, *args:
                            "1" * 40 if args[0] == "rev-parse" else "")

    def publish(self, row, failed=False, errors=None):
        usage = {"input_tokens": 100, "cached_input_tokens": 90, "uncached_input_tokens": 10,
                 "output_tokens": 20, "reasoning_output_tokens": 5, "raw_input_plus_output": 120,
                 "uncached_input_plus_output": 30}
        errors = [] if errors is None else errors
        artifact = Path(row["artifact"])
        (artifact / "observation").mkdir(parents=True, exist_ok=True)
        report = {"run_id": row["run_id"], "workflow_result": "fail" if failed else "pass",
                  "result": "fail" if failed else "pass", "bench_mode": "work-leaf",
                  "feature_schedule": "concurrent", "agent_backend": "codex", "agent_transport": "app-server",
                  "agent_model": "gpt-5.5", "agent_reasoning_effort": "xhigh",
                  "codex_cli_version": "codex-cli 0.153.4", "base_commit": self.runner.BASE_COMMIT,
                  "measurement_status": "incomplete" if errors else "complete", "total_workflow_usage": usage}
        Path(row["report"]).write_text(json.dumps(report))
        (artifact / "observation/analysis.json").write_text(json.dumps({
            "capture_complete": not errors, "errors": errors, "usage_scopes": {"total_workflow": usage}}))
        Path(row["prompt_trace"]).write_text(json.dumps({"event": "activation",
            "schema": "work-leaf-bench-experiment-v1", "run_id": row["run_id"], "condition": row["condition"]}) + "\n")

    def test_explicit_schedule_rejects_direct_duplicates_traversal_and_overfull_waves(self):
        for mutate in (lambda p: p["runs"][0].update(condition="direct"),
                       lambda p: p["runs"][1].update(run_id=p["runs"][0]["run_id"]),
                       lambda p: p["runs"][0].update(run_id="../escape"),
                       lambda p: p["runs"].append({"run_id": "extra", "condition": "control", "wave": 1}),
                       lambda p: p["runs"][0].update(wave=0)):
            plan = self.plan()
            mutate(plan)
            with self.subTest(plan=plan), self.assertRaises(ValueError):
                self.runner.validate_plan(plan)

    def test_smaller_final_wave_is_allowed_but_earlier_wave_cannot_waste_slots(self):
        plan = self.plan(2)
        plan["runs"] = plan["runs"][:4]
        plan["randomization"]["mixed_waves"] = False
        plan["randomization"]["block_condition_counts"]["block-2"] = {"control": 1}
        self.runner.validate_plan(plan)
        plan["runs"][1]["wave"] = 2
        with self.assertRaises(ValueError):
            self.runner.validate_plan(plan)

    def test_randomization_counts_and_mixed_wave_requirements_are_verified(self):
        plan = self.plan()
        plan["randomization"]["block_condition_counts"]["block-1"]["control"] = 2
        with self.assertRaises(ValueError):
            self.runner.validate_plan(plan)

    def test_phase_kind_and_randomization_unit_cannot_be_implicit(self):
        for location, key in ((None, "phase_kind"), ("randomization", "unit")):
            plan = self.plan()
            del (plan if location is None else plan[location])[key]
            with self.assertRaises(ValueError):
                self.runner.validate_plan(plan)
        plan = self.plan()
        for row in plan["runs"]:
            row["condition"] = "control"
        plan["randomization"]["block_condition_counts"] = {"block-1": {"control": 3}}
        with self.assertRaises(ValueError):
            self.runner.validate_plan(plan)

    def test_environment_keeps_subscription_home_and_only_condition_manifest_differs(self):
        args, manifest = self.fixture()
        inherited = {"PATH": "/usr/bin", "HOME": "/existing-home", "CODEX_HOME": "/existing-home/.codex",
                     "OPENAI_API_KEY": "fake", "OPENAI_BASE_URL": "https://invalid",
                     "WORK_LEAF_BENCH_TIMEOUT_SECS": "1", "WORK_LEAF_UNKNOWN_EXPERIMENT": "yes"}
        for row in manifest["schedule"]:
            env = self.runner.run_environment(manifest, row, inherited)
            self.assertEqual(env["CODEX_HOME"], inherited["CODEX_HOME"])
            self.assertNotIn("OPENAI_API_KEY", env)
            self.assertNotIn("OPENAI_BASE_URL", env)
            self.assertNotIn("WORK_LEAF_UNKNOWN_EXPERIMENT", env)
            for key, value in {"TIMEOUT_SECS": "7200", "BUSY_STALL_SECS": "1800",
                               "IDLE_STALL_SECS": "300", "MODEL": "gpt-5.5",
                               "REASONING_EFFORT": "xhigh", "DISABLE_TMUX_SUPERVISOR": "1"}.items():
                self.assertEqual(env["WORK_LEAF_BENCH_" + key], value)
            self.assertEqual(env["WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS"], "1000")
            self.assertEqual(env["WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME"], "forward")
            self.assertEqual(env["WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE"], "1")
            self.assertEqual(env["WORK_LEAF_BENCH_EXPERIMENT"], "1")
            self.assertEqual(env["WORK_LEAF_BENCH_EXPERIMENT_MANIFEST"], row["experiment_manifest"])
            self.assertEqual(env["WORK_LEAF_BENCH_BIN_DIR"], manifest["bin_dir"])
            self.assertEqual(row["driver"], str(args.source_repo / "bench-three-features"))

    def test_prepare_pins_all_rows_before_admission(self):
        args, manifest = self.fixture(2)
        self.assertFalse((args.phase_root / "RUN-ONCE").exists())
        self.assertEqual([len(wave) for wave in manifest["launch_order"]], [3, 3])
        self.assertEqual(len({row["runtime_dir"] for row in manifest["schedule"]}), 6)
        for row in manifest["schedule"]:
            experiment = json.loads(Path(row["experiment_manifest"]).read_text())
            self.assertEqual(experiment, {"schema": "work-leaf-bench-experiment-v1",
                                          "run_id": row["run_id"], "condition": row["condition"],
                                          "evidence_path": row["prompt_trace"]})
        with self.mock_git():
            self.runner.verify_manifest(args.phase_root)

    def test_claim_cannot_be_reused(self):
        with tempfile.TemporaryDirectory() as root:
            self.runner.claim_phase(Path(root))
            with self.assertRaises(FileExistsError):
                self.runner.claim_phase(Path(root))

    def test_frozen_file_mutation_is_rejected(self):
        for role in ("observer", "runtime", "experiment-manifest", "protocol", "scorer-config"):
            args, manifest = self.fixture()
            path = Path(next(item["path"] for item in manifest["files"] if item["role"] == role))
            path.chmod(0o600)
            path.write_bytes(b"changed\n")
            with self.subTest(role=role), self.mock_git(), self.assertRaises(ValueError):
                self.runner.verify_manifest(args.phase_root)

    def test_missing_required_identity_and_changed_runner_are_rejected(self):
        for mutate in (lambda m: m.update(runner_sha256="0" * 64),
                       lambda m: m.update(files=[f for f in m["files"] if f["role"] != "observer"])):
            args, manifest = self.fixture()
            mutate(manifest)
            (args.phase_root / "PHASE-MANIFEST.json").write_text(json.dumps(manifest))
            with self.mock_git(), self.assertRaises(ValueError):
                self.runner.verify_manifest(args.phase_root)

    def test_config_snapshot_never_serializes_secret_values(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / "config.toml"
            path.write_text('model = "gpt-5.5"\n[model_providers.private]\napi_key = "SECRET_VALUE"\n')
            record = self.runner.config_snapshot(path)
            self.assertNotIn("SECRET_VALUE", json.dumps(record))
            self.assertEqual(record["safe_global_settings"], {"model": "gpt-5.5"})
            self.assertTrue(record["parse_valid"])

    def test_config_classifies_only_explicit_notice_fields_as_bookkeeping(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / "config.toml"
            path.write_text('model = "gpt-5.5"\n')
            baseline = self.runner.config_snapshot(path)
            path.write_text('model = "gpt-5.5"\n[notice]\nhide_full_access_warning = true\n')
            notices = self.runner.config_snapshot(path)
            self.assertEqual(self.runner.config_drift(baseline, notices), "known_bookkeeping_only")
            path.write_text('model = "gpt-5.5"\n[notice]\nunknown_setting = true\n')
            self.assertEqual(self.runner.config_drift(baseline, self.runner.config_snapshot(path)), "behavioral_or_unknown")
            path.write_text('model = "gpt-5.5"\n[features]\nunknown_feature = true\n')
            self.assertEqual(self.runner.config_drift(baseline, self.runner.config_snapshot(path)), "behavioral_or_unknown")

    def test_config_invalid_input_is_not_reported_as_unchanged(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / "config.toml"
            path.write_text('model = "gpt-5.5"\n')
            baseline = self.runner.config_snapshot(path)
            path.write_text("not valid [ TOML")
            invalid = self.runner.config_snapshot(path)
            self.assertFalse(invalid["parse_valid"])
            self.assertEqual(self.runner.config_drift(baseline, invalid), "unreadable_or_invalid")

    def test_launch_failure_retains_schedule_and_never_replaces(self):
        args, manifest = self.fixture(2)
        launcher = Mock(side_effect=OSError("fixture spawn failure"))
        with self.mock_git():
            result = self.runner.run_phase(args.phase_root, popen=launcher)
        self.assertEqual(launcher.call_count, 1)
        self.assertEqual(len(result["runs"]), 6)
        self.assertEqual(sum(row["launch_status"] == "failed_to_start" for row in result["runs"]), 1)
        self.assertEqual(sum(row["launch_status"] == "not_launched_after_infrastructure_failure"
                             for row in result["runs"]), 5)
        self.assertEqual(len(json.loads((args.phase_root / "score-manifest.json").read_text())["runs"]), 6)
        journal = [json.loads(line) for line in (args.phase_root / "config-history.jsonl").read_text().splitlines()]
        self.assertTrue(any(item["reason"].startswith("before-launch:") for item in journal))
        self.assertEqual(journal, json.loads((args.phase_root / "config-history.json").read_text())["snapshots"])

    def test_three_are_launched_before_waiting_and_next_wave_waits(self):
        args, manifest = self.fixture(2)
        launched = []
        reaped = []

        def launch(*_args, **kwargs):
            ordinal = len(launched)
            if ordinal == 3:
                self.assertEqual(len(reaped), 3)
            wave_end = ((ordinal // 3) + 1) * 3
            process = Mock(pid=990000 + ordinal)
            process.poll.side_effect = lambda: 0 if len(launched) >= wave_end else None
            process.wait.side_effect = lambda **_kw: reaped.append(ordinal) or 0
            launched.append(kwargs["env"]["WORK_LEAF_BENCH_RUN_ID"])
            self.publish(next(row for row in manifest["schedule"] if row["run_id"] == launched[-1]))
            return process

        with self.mock_git():
            result = self.runner.run_phase(args.phase_root, popen=launch)
        self.assertEqual(launched, sum(manifest["launch_order"], []))
        self.assertEqual(len(reaped), 6)
        self.assertTrue(all(row["launcher_exit_code"] == 0 for row in result["runs"]))

    def test_genuine_workflow_failure_retained_and_next_wave_still_runs(self):
        args, manifest = self.fixture(2)
        launcher = Mock()

        def launch(*_args, **kwargs):
            row = next(row for row in manifest["schedule"] if row["run_id"] == kwargs["env"]["WORK_LEAF_BENCH_RUN_ID"])
            self.publish(row, failed=True)
            return Mock(pid=990000 + launcher.call_count, poll=Mock(return_value=1), wait=Mock(return_value=1))

        launcher.side_effect = launch
        with self.mock_git():
            result = self.runner.run_phase(args.phase_root, popen=launcher)
        self.assertEqual(launcher.call_count, 6)
        self.assertTrue(all(row["outcome_classification"] == "recorded_workflow_failure" for row in result["runs"]))

    def test_missing_report_stops_future_admissions_even_with_success_exit(self):
        args, _manifest = self.fixture(2)
        launcher = Mock(return_value=Mock(pid=999999, poll=Mock(return_value=0), wait=Mock(return_value=0)))
        with self.mock_git():
            result = self.runner.run_phase(args.phase_root, popen=launcher)
        self.assertEqual(launcher.call_count, 1)
        self.assertEqual(result["runs"][0]["launch_status"] in {"completed", "not_launched_after_infrastructure_failure"}, True)
        self.assertEqual(sum(row["launch_status"] == "not_launched_after_infrastructure_failure" for row in result["runs"]), 5)

    def test_accounting_tail_gap_is_retained_but_integrity_error_blocks(self):
        _args, manifest = self.fixture()
        row = manifest["schedule"][0]
        self.publish(row, errors=["interrupted provider turn has no complete usage: count=2"])
        self.assertEqual(self.runner.classify_outcome(row, 0), "recorded_workflow_success")
        self.publish(row, errors=["response identity mismatch"])
        self.assertEqual(self.runner.classify_outcome(row, 0), "infrastructure_or_measurement_integrity_failure")

    def test_unexpected_model_version_and_trace_identity_are_integrity_failures(self):
        _args, manifest = self.fixture()
        row = manifest["schedule"][0]
        for field, value in (("agent_model", "other"), ("codex_cli_version", "codex-cli changed")):
            self.publish(row)
            report = json.loads(Path(row["report"]).read_text())
            report[field] = value
            Path(row["report"]).write_text(json.dumps(report))
            self.assertEqual(self.runner.classify_outcome(row, 0), "infrastructure_or_measurement_integrity_failure")
        self.publish(row)
        Path(row["prompt_trace"]).write_text('{"event":"activation","condition":"unrelated"}\n')
        self.assertEqual(self.runner.classify_outcome(row, 0), "infrastructure_or_measurement_integrity_failure")

    def test_source_drift_between_waves_stops_new_admissions(self):
        args, manifest = self.fixture(2)
        launcher = Mock()
        waits = []

        def launch(*_args, **kwargs):
            row = next(row for row in manifest["schedule"] if row["run_id"] == kwargs["env"]["WORK_LEAF_BENCH_RUN_ID"])
            self.publish(row)

            def finish(**_kw):
                waits.append(row["run_id"])
                if len(waits) == 3:
                    (args.source_repo / "bench-three-features").write_text("changed driver")
                return 0

            return Mock(pid=990000 + launcher.call_count, poll=Mock(return_value=0), wait=Mock(side_effect=finish))

        launcher.side_effect = launch
        with self.mock_git():
            result = self.runner.run_phase(args.phase_root, popen=launcher)
        self.assertEqual(launcher.call_count, 3)
        self.assertTrue(result["frozen_input_integrity_errors"])

    def test_behavioral_config_drift_stops_new_launches_without_erasing_rows(self):
        args, _manifest = self.fixture()
        args.global_config.write_text('model = "gpt-5.5"\nmodel_reasoning_effort = "low"\n')
        launcher = Mock()
        with self.mock_git():
            result = self.runner.run_phase(args.phase_root, popen=launcher)
        launcher.assert_not_called()
        self.assertTrue(all(row["launch_status"] == "not_launched_after_config_drift" for row in result["runs"]))
        self.assertTrue(result["behavioral_config_drift_detected"])

    def test_exception_does_not_claim_the_schedule_loop_finished(self):
        args, _manifest = self.fixture()
        final_snapshot = self.runner.config_snapshot(args.global_config)
        with self.mock_git(), patch.object(self.runner, "config_snapshot", side_effect=[
                RuntimeError("synthetic supervisor exception"), final_snapshot]), self.assertRaises(RuntimeError):
            self.runner.run_phase(args.phase_root, popen=Mock())
        result = json.loads((args.phase_root / "PHASE-RESULT.json").read_text())
        self.assertFalse(result["phase_schedule_exhausted"])
        self.assertFalse(result["all_scheduled_workflows_launched"])
        self.assertEqual(result["supervisor_error"], "RuntimeError")

    def test_natural_schedule_exhaustion_does_not_imply_every_workflow_launched(self):
        args, _manifest = self.fixture()
        with self.mock_git():
            result = self.runner.run_phase(args.phase_root, popen=Mock(side_effect=OSError("synthetic spawn failure")))
        self.assertTrue(result["phase_schedule_exhausted"])
        self.assertFalse(result["all_scheduled_workflows_launched"])

    def test_all_scheduled_workflows_launched_is_true_after_complete_admission(self):
        args, manifest = self.fixture()

        def launch(*_args, **kwargs):
            row = next(row for row in manifest["schedule"] if row["run_id"] == kwargs["env"]["WORK_LEAF_BENCH_RUN_ID"])
            self.publish(row)
            return Mock(pid=990000, poll=Mock(return_value=0), wait=Mock(return_value=0))

        with self.mock_git():
            result = self.runner.run_phase(args.phase_root, popen=launch)
        self.assertTrue(result["phase_schedule_exhausted"])
        self.assertTrue(result["all_scheduled_workflows_launched"])


if __name__ == "__main__":
    unittest.main()
