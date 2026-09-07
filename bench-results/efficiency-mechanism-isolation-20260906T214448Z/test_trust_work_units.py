#!/usr/bin/env python3
"""Synthetic, offline-only tests of the prospective own-workflow trust gate."""

from copy import deepcopy
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import py_compile
import tempfile
import time
import unittest
from unittest.mock import Mock, patch


HERE = Path(__file__).resolve().parent


def load(name):
    spec = importlib.util.spec_from_file_location(name, HERE / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value) + "\n")


def write_lines(path, values):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(json.dumps(value) + "\n" for value in values))


class TrustFixture:
    def __init__(self, root, helper, runner):
        self.root, self.helper, self.runner = root, helper, runner
        self.phase = root / "phase"
        self.phase.mkdir()
        self.home = root / "settings"
        self.home.mkdir()
        self.global_config = self.home / "config.toml"
        self.global_config.write_text('model = "gpt-5.5"\n[projects."/existing/project"]\ntrust_level = "trusted"\n')
        self.baseline = runner.config_snapshot(self.global_config)
        self.row = {"run_id": "run-001", "runtime_dir": str(root / "runtime" / "run-001"),
                    "results_dir": str(self.phase / "runs" / "run-001"),
                    "artifact": str(self.phase / "runs" / "run-001" / "published")}
        self.manifest = {"study": "synthetic-study", "phase": "synthetic-phase",
                         "global_config_baseline": self.baseline, "schedule": [self.row],
                         "model": "gpt-5.5", "reasoning_effort": "xhigh"}
        write_json(self.phase / "PHASE-MANIFEST.json", self.manifest)
        self.repo = Path(self.row["runtime_dir"]) / "fresh-checkout" / "repo"
        (self.repo / ".git").mkdir(parents=True)
        (self.repo / ".codex" / "rules").mkdir(parents=True)
        (self.repo / ".codex" / "config.toml").write_text('sandbox_mode = "workspace-write"\n')
        (self.repo / ".codex" / "rules" / "local.rules").write_text("stable rules\n")
        self.observation = Path(self.row["results_dir"]) / ".staging" / "observation"
        self.invocation = "invocation-001"
        self.capture = self.observation / "app-server" / self.invocation
        self.capture.mkdir(parents=True)
        self.base_ns = time.time_ns() - 10_000_000_000
        write_json(self.phase / "PHASE-RESULT.json", {"runs": [{"run_id": self.row["run_id"],
            "started_at": datetime.fromtimestamp((self.base_ns - 1_000_000_000) / 1e9, timezone.utc).isoformat()}]})
        self.thread = "linearizer-thread"
        self.native = self.home / "sessions" / "native.jsonl"
        write_json(self.observation / "observer-config.json", {"schema_version": 1,
                   "study_id": self.manifest["study"], "run_id": self.row["run_id"],
                   "condition": "work-leaf", "root": str(self.observation)})
        write_json(self.observation / "invocations" / self.invocation / "start.json", {
            "invocation_id": self.invocation, "primary": True, "capture_kind": "app-server",
            "parent_invocation_id": None, "cwd": str(self.repo),
            "project_layer_inventory_required": True, "start_unix_ns": self.base_ns})
        write_json(self.observation / "invocations" / self.invocation / "child.json", {
            "invocation_id": self.invocation, "started_monotonic_ns": 20})
        self.requests = [{"id": "1", "method": "thread/start", "params": {
            "cwd": str(self.repo), "sandbox": "read-only", "approvalPolicy": "never", "model": "gpt-5.5"}},
            {"id": "2", "method": "thread/start", "params": {
            "cwd": str(self.repo), "sandbox": "danger-full-access", "approvalPolicy": "never", "model": "gpt-5.5"}}]
        self.reply = {"id": "2", "result": {"thread": {"id": self.thread, "path": str(self.native), "cwd": str(self.repo)},
            "cwd": str(self.repo), "sandbox": {"type": "dangerFullAccess"},
            "approvalPolicy": "never", "model": "gpt-5.5", "reasoningEffort": "xhigh"}}
        self.started = {"method": "thread/started", "emittedAtMs": (self.base_ns + 5_000_000_000) // 1_000_000,
                        "params": {"thread": {"id": self.thread, "path": str(self.native), "cwd": str(self.repo)}}}
        self.save_streams()
        native_timestamp = datetime.fromtimestamp((self.base_ns + 6_000_000_000) / 1e9, timezone.utc).isoformat()
        write_lines(self.native, [
            {"timestamp": native_timestamp, "type": "session_meta", "payload": {
                "id": self.thread, "cwd": str(self.repo), "cli_version": "0.153.4"}},
            {"timestamp": native_timestamp, "type": "response_item", "payload": {"type": "message", "role": "developer", "content": [{"type": "input_text", "text":
                "<permissions instructions>\nFilesystem sandboxing defines which files can be read or written. `sandbox_mode` is `danger-full-access`: No filesystem sandboxing - all commands are permitted. Network access is enabled.\nApproval policy is currently never. Do not provide the `sandbox_permissions` for any reason, commands will be rejected.\n</permissions instructions>"}]}},
            {"timestamp": native_timestamp, "type": "turn_context", "payload": {
                "cwd": str(self.repo), "sandbox_policy": {"type": "danger-full-access"},
                "approval_policy": "never", "model": "gpt-5.5", "effort": "xhigh"}},
        ])
        entries = helper.project_entries(self.repo)
        digest = helper.inventory_digest(entries)
        self.inventories = [{"schema": "work-leaf-project-layer-inventory-v1", "run_id": self.row["run_id"],
            "study_id": self.manifest["study"], "repository": str(self.repo), "valid": True, "errors": [],
            "root_policy": "canonical-cwd-is-git-root; default-markers-only; no-profile-or-resource-reference",
            "entries": entries, "inventory_sha256": digest,
            "label": label, "invocation_id": self.invocation if label == "pre-spawn" else None,
            "completed_unix_ns": str(self.base_ns + offset), "completed_monotonic_ns": str(monotonic),
            "matches_pre_spawn": None if label == "pre-spawn" else True}
            for label, offset, monotonic in [("pre-spawn", 1_000_000_000, 10), ("checkpoint:pre-linearize", 4_000_000_000, 30)]]
        self.save_inventory()
        self.checker = helper.TrustClassifier(self.manifest, self.phase)

    def save_streams(self):
        write_lines(self.capture / "client-to-server.raw", self.requests)
        forwarded = deepcopy(self.requests)
        for request in forwarded:
            request["params"]["experimentalRawEvents"] = True
        write_lines(self.capture / "client-to-server.forwarded.raw", forwarded)
        write_lines(self.capture / "server-to-client.raw", [self.reply, self.started])

    def save_inventory(self):
        write_lines(self.observation / "project-layer-inventory" / "manifest.jsonl", self.inventories)

    def add_trust(self, path=None, entry='trust_level = "trusted"'):
        with self.global_config.open("a") as handle:
            handle.write(f'\n[projects.{json.dumps(str(path or self.repo))}]\n{entry}\n')

    def classify(self, launched=None):
        snapshot = self.runner.config_snapshot(self.global_config)
        legacy = self.runner.config_drift(self.baseline, snapshot)
        verdict = self.checker.classify(snapshot, legacy, {self.row["run_id"]} if launched is None else launched)
        return {**snapshot, "drift_from_baseline": legacy, "trust_classification": verdict}

    def finish(self, records):
        write_json(self.phase / "config-history.json", {"baseline": self.baseline, "snapshots": records})
        write_lines(self.phase / "config-history.jsonl", records)
        self.checker.finish(records)


class TrustGateTests(unittest.TestCase):
    def setUp(self):
        self.helper, self.runner = load("trust_work_units"), load("runner_work_units")
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.fixture = TrustFixture(Path(self.temp.name), self.helper, self.runner)

    def assert_rejected(self):
        self.assertFalse(self.fixture.classify()["trust_classification"]["allows_admission"])

    def test_exact_owned_transition_is_verified_without_clearing_legacy_hashes(self):
        self.fixture.add_trust()
        before = self.fixture.global_config.read_bytes()
        record = self.fixture.classify()
        self.assertEqual(record["drift_from_baseline"], "behavioral_or_unknown")
        self.assertEqual(record["trust_classification"]["verdict"], "verified_own_workflow_trust_transition")
        self.assertTrue(record["trust_classification"]["allows_admission"])
        self.assertEqual(self.fixture.global_config.read_bytes(), before)
        self.assertEqual(record["sha256"], hashlib.sha256(before).hexdigest())
        self.assertEqual(record["trust_classification"]["normalized_parsed_sha256"], self.fixture.baseline["parsed_sha256"])

    def test_unrelated_checkout_is_not_allowed_by_runtime_prefix(self):
        self.fixture.add_trust(Path(self.fixture.row["runtime_dir"]) / "different-checkout" / "repo")
        self.assert_rejected()

    def test_unlaunched_identity_is_not_an_admitted_workflow(self):
        self.fixture.add_trust()
        self.assertFalse(self.fixture.classify(set())["trust_classification"]["allows_admission"])

    def test_preexisting_entry_mutation_is_not_a_new_trust_addition(self):
        self.fixture.add_trust()
        self.fixture.global_config.write_text(self.fixture.global_config.read_text().replace('"/existing/project"', '"/different/project"'))
        self.assert_rejected()

    def test_extra_field_or_untrusted_value_is_rejected(self):
        self.fixture.add_trust(entry='trust_level = "trusted"\nextra = true')
        self.assert_rejected()

    def test_wrong_sandbox_is_rejected(self):
        self.fixture.requests[1]["params"]["sandbox"] = "workspace-write"
        self.fixture.save_streams()
        self.fixture.add_trust()
        self.assert_rejected()

    def test_numeric_reply_id_cannot_match_string_request_id(self):
        self.fixture.reply["id"] = 2
        self.fixture.save_streams()
        self.fixture.add_trust()
        self.assert_rejected()

    def test_thread_and_native_session_identities_are_nonempty_strings(self):
        self.fixture.reply["result"]["thread"]["id"] = True
        self.fixture.started["params"]["thread"]["id"] = True
        native = [json.loads(line) for line in self.fixture.native.read_text().splitlines()]
        native[0]["payload"]["id"] = 1
        write_lines(self.fixture.native, native)
        self.fixture.save_streams()
        self.fixture.add_trust()
        self.assert_rejected()

    def test_missing_forwarding_or_inventory_is_rejected(self):
        (self.fixture.capture / "client-to-server.forwarded.raw").unlink()
        self.fixture.add_trust()
        self.assert_rejected()

    def test_missing_prelinearize_inventory_is_rejected(self):
        self.fixture.inventories.pop()
        self.fixture.save_inventory()
        self.fixture.add_trust()
        self.assert_rejected()

    def test_actual_changed_rules_are_rejected_even_with_unchanged_saved_inventory(self):
        (self.fixture.repo / ".codex" / "rules" / "local.rules").write_text("changed after snapshot\n")
        self.fixture.add_trust()
        self.assert_rejected()

    def test_future_thread_start_does_not_justify_prior_configuration(self):
        self.fixture.started["emittedAtMs"] = int(time.time() * 1000) + 60000
        self.fixture.save_streams()
        self.fixture.add_trust()
        self.assert_rejected()

    def test_native_permissions_must_match_not_merely_model(self):
        self.fixture.native.write_text(self.fixture.native.read_text().replace("danger-full-access", "read-only"))
        self.fixture.add_trust()
        self.assert_rejected()

    def test_offline_replay_preserves_evidence_after_artifact_publication(self):
        self.fixture.add_trust()
        record = self.fixture.classify()
        self.fixture.finish([record])
        self.append_final_inventory()
        self.fixture.observation.parent.rename(Path(self.fixture.row["artifact"]))
        result = self.helper.audit(self.fixture.phase)
        self.assertTrue(result["valid"], result)
        self.assertFalse(result["self_contained_global_config_replay"])

    def test_offline_final_global_change_and_capture_prefix_tampering_are_rejected(self):
        self.fixture.add_trust()
        record = self.fixture.classify()
        self.fixture.finish([record])
        self.append_final_inventory()
        original = self.fixture.global_config.read_text()
        self.fixture.global_config.write_text(original + "\n# later edit\n")
        self.assertFalse(self.helper.audit(self.fixture.phase)["valid"])
        self.fixture.global_config.write_text(original)
        path = self.fixture.capture / "server-to-client.raw"
        path.write_text(path.read_text().replace("dangerFullAccess", "workspaceWrite"))
        self.assertFalse(self.helper.audit(self.fixture.phase)["valid"])

    def append_final_inventory(self, *, changed=False):
        final = deepcopy(self.fixture.inventories[-1])
        final["label"] = "checkpoint:final"
        if changed:
            final["entries"][0]["mode"] ^= 1
            final["inventory_sha256"] = self.helper.inventory_digest(final["entries"])
        self.fixture.inventories.append(final)
        self.fixture.save_inventory()

    def test_offline_replay_rejects_later_final_inventory_drift_outside_live_prefix(self):
        self.fixture.add_trust()
        record = self.fixture.classify()
        self.fixture.finish([record])
        self.append_final_inventory(changed=True)
        self.assertFalse(self.helper.audit(self.fixture.phase)["valid"])

    def test_baseline_change_before_supervisor_construction_is_a_retained_rejection(self):
        self.fixture.add_trust()
        checker = self.helper.TrustClassifier(self.fixture.manifest, self.fixture.phase)
        snapshot = self.runner.config_snapshot(self.fixture.global_config)
        result = checker.classify(snapshot, "behavioral_or_unknown", {self.fixture.row["run_id"]})
        self.assertFalse(result["allows_admission"])

    def test_native_publication_lag_is_pending_then_resolves_with_later_evidence(self):
        native = self.fixture.native.read_bytes()
        self.fixture.native.unlink()
        self.fixture.add_trust()
        pending = self.fixture.classify()
        self.assertEqual(pending["trust_classification"]["verdict"], "pending_own_workflow_native_attestation")
        self.assertFalse(pending["trust_classification"]["allows_admission"])
        self.assertTrue(pending["trust_classification"]["pending_transition"])
        self.fixture.native.write_bytes(native)
        resolved = self.fixture.classify()
        self.assertEqual(resolved["trust_classification"]["verdict"], "verified_own_workflow_trust_transition")
        self.fixture.finish([pending, resolved])
        self.append_final_inventory()
        audit = self.helper.audit(self.fixture.phase)
        self.assertTrue(audit["valid"], audit)
        result_path = self.fixture.phase / "PHASE-RESULT.json"
        result = json.loads(result_path.read_text())
        result["runs"][0]["started_at"] = pending["captured_at"]
        write_json(result_path, result)
        self.assertFalse(self.helper.audit(self.fixture.phase)["valid"])

    def test_live_partial_native_tail_is_pending_not_proven_or_permanently_failed(self):
        with self.fixture.native.open("wb") as handle:
            handle.write(b'{"timestamp":')
        self.fixture.add_trust()
        result = self.fixture.classify()["trust_classification"]
        self.assertTrue(result.get("pending_transition", False))
        self.assertFalse(result["allows_admission"])

    def test_native_published_after_snapshot_cutoff_pends_without_retroactive_proof(self):
        future = datetime.fromtimestamp(time.time() + 1, timezone.utc).isoformat()
        frames = [json.loads(line) for line in self.fixture.native.read_text().splitlines()]
        for frame in frames:
            frame["timestamp"] = future
        write_lines(self.fixture.native, frames)
        self.fixture.add_trust()
        first = self.fixture.classify()
        self.assertTrue(first["trust_classification"].get("pending_transition", False))
        pending = next(iter(self.fixture.checker.pending.values()))
        self.assertFalse(any(source["location"] == "external-native" for source in pending["source_identities"]))
        later = self.runner.config_snapshot(self.fixture.global_config)
        later["captured_at"] = datetime.fromtimestamp(time.time() + 2, timezone.utc).isoformat()
        decision = self.fixture.checker.classify(later, "behavioral_or_unknown", {self.fixture.row["run_id"]})
        self.assertTrue(decision["allows_admission"])
        self.assertFalse(first["trust_classification"]["allows_admission"])

    def test_native_pending_has_a_fixed_deadline_and_missing_rpc_is_never_pending(self):
        self.fixture.native.unlink()
        self.fixture.add_trust()
        pending = self.fixture.classify()
        snapshot = self.runner.config_snapshot(self.fixture.global_config)
        snapshot["captured_at"] = datetime.fromtimestamp(
            datetime.fromisoformat(pending["captured_at"]).timestamp() + 31, timezone.utc).isoformat()
        expired = self.fixture.checker.classify(snapshot, "behavioral_or_unknown", {self.fixture.row["run_id"]})
        self.assertFalse(expired["allows_admission"])
        self.assertFalse(expired.get("pending_transition", False))
        self.assertIn("native-attestation-pending-deadline", expired["errors"])
        (self.fixture.capture / "server-to-client.raw").write_text("")
        self.fixture.checker.pending.clear()
        refused = self.fixture.classify()["trust_classification"]
        self.assertFalse(refused["allows_admission"])
        self.assertFalse(refused.get("pending_transition", False))

    def test_config_secret_values_are_never_serialized_into_evidence(self):
        self.fixture.add_trust(entry='trust_level = "trusted"\nsecret = "NEVER_SERIALIZE_THIS_VALUE"')
        record = self.fixture.classify()
        self.assertNotIn("NEVER_SERIALIZE_THIS_VALUE", json.dumps(record))
        for path in self.fixture.phase.glob("trust-*"):
            self.assertNotIn("NEVER_SERIALIZE_THIS_VALUE", path.read_text())

    def test_known_bookkeeping_parsed_history_is_rejected_when_not_reconstructable(self):
        with self.fixture.global_config.open("a") as handle:
            handle.write("\n[notice]\nhide_full_access_warning = true\n")
        record = self.fixture.classify()
        self.assertEqual(record["drift_from_baseline"], "known_bookkeeping_only")
        self.assertFalse(record["trust_classification"]["allows_admission"])
        self.assertEqual(record["trust_classification"]["errors"], ["parsed-bookkeeping-history-not-replayable"])

    def test_supervisor_loads_exact_helper_source_not_timestamp_matched_stale_bytecode(self):
        directory = Path(self.temp.name) / "bench-results" / "isolated"
        directory.mkdir(parents=True)
        target = directory / "runner_work_units.py"
        target.write_bytes((HERE / "runner_work_units.py").read_bytes())
        helper = directory / "trust_work_units.py"
        helper.write_text('class TrustClassifier:\n    VERSION = "old"\n')
        stamp = helper.stat()
        py_compile.compile(str(helper), doraise=True)
        helper.write_text('class TrustClassifier:\n    VERSION = "new"\n')
        os.utime(helper, ns=(stamp.st_atime_ns, stamp.st_mtime_ns))
        spec = importlib.util.spec_from_file_location("isolated_runner", target)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertEqual(module.TrustClassifier.VERSION, "new")
        self.assertEqual(module.TRUST_LOADED_SHA256, hashlib.sha256(helper.read_bytes()).hexdigest())
        helper.write_text('class TrustClassifier:\n    VERSION = "bad"\n')
        with self.assertRaisesRegex(ValueError, "differs from executing code"):
            module.prepare(None)

    def test_supervisor_preserves_legacy_drift_but_uses_distinct_prospective_gate(self):
        for accepted, expected_launches in ((True, 6), (False, 1), ("pending-once", 6)):
            with self.subTest(accepted=accepted), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                global_config = root / "config.toml"
                global_config.write_text('model = "gpt-5.5"\n')
                rows = [{"run_id": f"run-{i}", "condition": "control", "phase": "test", "block_id": "block",
                         "wave": i // 3 + 1, "workflow": "work-leaf", "artifact": str(root / f"results-{i}" / "artifact"),
                         "report": str(root / f"results-{i}" / "report.json"), "prompt_trace": str(root / f"trace-{i}"),
                         "experiment_manifest": str(root / f"manifest-{i}"), "runtime_dir": str(root / f"runtime-{i}"),
                         "results_dir": str(root / f"results-{i}"), "driver": "/unused"} for i in range(6)]
                manifest = {"study": "test", "phase": "test", "phase_kind": "confirmation", "schedule": rows,
                            "launch_order": [[row["run_id"] for row in rows[:3]], [row["run_id"] for row in rows[3:]]],
                            "source_repo": str(root), "global_config_baseline": self.runner.config_snapshot(global_config),
                            "schedule_sha256": "a" * 64, "supervisor_wall_timeout_seconds": 86400,
                            **{key: "synthetic" for key in ("base_commit", "task_list_sha256", "model", "reasoning_effort", "source_commit", "randomization")}}
                write_json(root / "PHASE-MANIFEST.json", manifest)
                classifier = Mock()
                calls = 0
                def classify(_snapshot, legacy, _launched):
                    nonlocal calls
                    calls += 1
                    is_pending = accepted == "pending-once" and calls == 2
                    return {"allows_admission": not is_pending and (legacy in self.helper.SAFE_LEGACY or bool(accepted)),
                            "pending_transition": is_pending, "legacy_verdict": legacy,
                            "verdict": "synthetic", "proof_ids": [], "errors": []}
                classifier.classify.side_effect = classify

                def launch(*_args, **_kwargs):
                    global_config.write_text('model = "gpt-5.5"\nsynthetic_changed = true\n')
                    return Mock(pid=123, poll=Mock(return_value=0), wait=Mock(return_value=0))

                launcher = Mock(side_effect=launch)
                with patch.object(self.runner, "verify_manifest", return_value=manifest), \
                     patch.object(self.runner, "TrustClassifier", return_value=classifier, create=True), \
                     patch.object(self.runner, "run_environment", return_value={}), \
                     patch.object(self.runner.time, "sleep"), \
                     patch.object(self.runner, "classify_outcome", return_value="workflow_completed"):
                    result = self.runner.run_phase(root, popen=launcher)
                self.assertEqual(launcher.call_count, expected_launches)
                self.assertTrue(result["behavioral_config_drift_detected"])
                self.assertEqual(result["unexplained_config_drift_detected"], not bool(accepted))


if __name__ == "__main__":
    unittest.main()
