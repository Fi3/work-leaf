"""Synthetic provider-free packet regressions; no running workflow inspection."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import analyze_work_units as frozen
import runner_work_units as runner
from test_analyze_work_units import event, fixture
import build_ack_packet as subject

HERE = Path(__file__).resolve().parent


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value), encoding="utf-8")


def lines(path, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class PacketTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.phase = Path(self.temp.name)
        self.artifact = self.phase / "runs/run/run-three-feature-bench-artifacts"
        self.capture = self.artifact / "observation/app-server/invocation"
        self.trace = self.phase / "prompt-events/run.jsonl"
        self.output = self.phase / "coding/run.json"
        events = [event("policy-injection", sequence=1), event(sequence=2),
                  event("command-result", sequence=3)]
        t, threads, captures = fixture(events)
        self.clients, self.servers = captures[0]["clients"], captures[0]["servers"]
        self.trace_rows, self.threads = t, threads
        for index, text in ((3, "work-leaf file read\nsrc/a.rs\nPRIOR_STATE"),
                            (4, "The reviewer found issues in your patch for commit abc.\nREVIEW_EVIDENCE")):
            self.clients.append({"id": str(index), "method": "turn/start", "params": {
                "threadId": "thread", "input": [{"type": "text", "text": text}]}})
            self.servers.append({"id": str(index), "result": {"turn": {"id": "turn-" + str(index)}}})
        for index, kind, text in ((0, "agentMessage", "@work-leaf edit feature\nPUBLIC_EDIT"),
                                  (0, "reasoning", "SECRET_REASONING"),
                                  (1, "agentMessage", "@work-leaf locks run target -- cargo test"),
                                  (2, "commandExecution", "NATIVE_PRIVATE_PROJECTION_NOT_REQUESTED")):
            self.servers.append({"method": "item/completed", "params": {
                "threadId": "thread", "turnId": "turn-" + str(index),
                "item": {"type": kind, "id": "item-" + str(len(self.servers)), "text": text}}})
        lines(self.trace, self.trace_rows)
        write(self.artifact / "observation/analysis.json", {"run_id": "run", "condition": "work-leaf", "threads": threads})
        plan = {"phase": "test", "phase_kind": "confirmation", "runs": [
            {"run_id": "run", "condition": "control", "wave": 1, "block_id": "b"}],
            "randomization": {"method": "synthetic", "mixed_waves": False, "unit": "workflow",
                              "scheme": "within_block", "block_condition_counts": {"b": {"control": 1}}}}
        self.row = runner.schedule(self.phase, self.phase / "source", self.phase / "runtime", plan)[0]
        write(Path(self.row["experiment_manifest"]), {"schema": frozen.SCHEMA, "run_id": "run", "condition": "control", "evidence_path": str(self.trace)})
        write(Path(self.row["report"]), {"agent_model": "gpt-5.5", "agent_reasoning_effort": "xhigh"})
        files = []
        for name in ("analyze_work_units.py", "analyze.py"):
            saved = self.phase / "infrastructure/evidence" / (HERE / name).relative_to(subject.REPO)
            saved.parent.mkdir(parents=True, exist_ok=True)
            saved.write_bytes((HERE / name).read_bytes())
            files.append({"path": str(saved), "sha256": sha(saved)})
        files.append({"path": self.row["experiment_manifest"], "sha256": sha(Path(self.row["experiment_manifest"]))})
        self.manifest = {"schema_version": 1, "study": "synthetic", "phase": "test", "schedule": [self.row],
                         "files": files, "model": "gpt-5.5", "reasoning_effort": "xhigh"}
        write(self.phase / "PHASE-MANIFEST.json", self.manifest)
        write(self.phase / "RUN-ONCE/admission.json", {"manifest_sha256": sha(self.phase / "PHASE-MANIFEST.json")})
        write(self.phase / "logs/run.exit.json", {**self.row, "id": "run", "launch_status": "completed",
              "started_at": "2026-01-01T00:00:00+00:00", "finished_at": "2026-01-01T00:00:01+00:00", "launcher_exit_code": 0})
        self.sync_capture()

    def sync_capture(self):
        lines(self.capture / "client-to-server.raw", self.clients)
        lines(self.capture / "client-to-server.forwarded.raw", self.clients)
        lines(self.capture / "server-to-client.raw", self.servers)
        lines(self.capture / "raw-response-rewrites.jsonl", [])
        base = frozen.legacy()
        write(self.capture / "raw-response-usage.json", base.CAPTURE_SETTINGS)
        inv = self.artifact / "observation/invocations/invocation"
        write(inv / "start.json", {"raw_response_usage": True, "invocation_id": "invocation", "provider_usage_grace_ms": 0})
        write(inv / "end.json", {"invocation_id": "invocation", "raw_response_usage_start_sha256": sha(inv / "start.json"),
              "stdin_sha256": sha(self.capture / "client-to-server.raw"), "stdout_sha256": sha(self.capture / "server-to-client.raw"),
              "raw_response_usage_sha256": {name: sha(self.capture / name) for name in
                  ("raw-response-usage.json", "raw-response-rewrites.jsonl", "client-to-server.forwarded.raw")}})

    def build(self):
        return subject.build_packet(self.phase, "run", self.output)

    def test_exact_frozen_ack_identity_and_public_context_without_reasoning_or_policy(self):
        self.assertEqual(self.row["workflow"], "work-leaf-concurrent")
        result = self.build()
        expected = frozen.prompt_inventory(self.trace_rows, self.threads,
            [{"path": str(self.capture), "clients": self.clients, "servers": self.servers}], "run", "control")
        self.assertEqual(result["acks"][0]["ack_id"], expected["acks"][0]["ack_id"])
        self.assertEqual(result["acks"][0]["rpc_id"], "1")
        serialized = json.dumps(result)
        for value in ("PUBLIC_EDIT", "PRIOR_STATE", "REVIEW_EVIDENCE", "cargo test"):
            self.assertIn(value, serialized)
        for value in ("SECRET_REASONING", "NATIVE_PRIVATE_PROJECTION_NOT_REQUESTED", *frozen.A, *frozen.B, *frozen.C):
            self.assertNotIn(value, serialized)
        self.assertNotIn('"condition"', serialized)
        self.assertFalse(result["perfect_blinding_claimed"])
        self.assertTrue(result["native_activity_projection_incomplete"])
        self.assertNotIn("label", result["acks"][0])
        self.assertEqual(result["acks"][0]["edit_group_join"], "unassigned; candidate context is not an application receipt")

    def test_retained_full_command_evidence_is_linked_not_guessed_to_an_agent(self):
        path = self.artifact / "observation/locked-commands/command/meta.json"
        write(path, {"start": {"invocation_id": "command"}, "end": {"invocation_id": "command"}})
        result = self.build()
        self.assertIn(str(path), result["support_source_paths"])
        self.assertEqual(result["source_sha256"][str(path)], sha(path))

    def test_exact_frozen_helper_paths_allow_other_admitted_files_with_same_basename(self):
        other = self.phase / "infrastructure/evidence/other-study/analyze.py"
        other.parent.mkdir(parents=True)
        other.write_text("raise AssertionError('unrelated helper must never execute')\n")
        self.manifest["files"].append({"path": str(other), "sha256": sha(other)})
        write(self.phase / "PHASE-MANIFEST.json", self.manifest)
        write(self.phase / "RUN-ONCE/admission.json", {"manifest_sha256": sha(self.phase / "PHASE-MANIFEST.json")})
        self.assertTrue(self.build()["acks"])

    def test_physical_trace_lines_include_blank_lines(self):
        self.trace.write_text("\n" + self.trace.read_text().replace("\n", "\n\n"))
        result = self.build()
        self.assertIn("JSONL line 6;", result["acks"][0]["trace_source"]["locator"])

    def test_create_new_output_and_no_source_writes(self):
        before = {str(p): sha(p) for p in self.phase.rglob("*") if p.is_file()}
        self.build()
        with self.assertRaises(FileExistsError):
            self.build()
        self.assertTrue(all(sha(Path(p)) == value for p, value in before.items()))

    def test_running_missing_exit_or_missing_end_never_creates_packet(self):
        exit_path = self.phase / "logs/run.exit.json"
        receipt = json.loads(exit_path.read_text())
        receipt["launch_status"] = "running"
        write(exit_path, receipt)
        with self.assertRaises(ValueError): self.build()
        self.assertFalse(self.output.exists())
        exit_path.unlink()
        with self.assertRaises((ValueError, OSError)): self.build()

    def test_closed_capture_provenance_failure_never_creates_packet(self):
        (self.capture / "server-to-client.raw").write_text("{}\n")
        with self.assertRaises(ValueError): self.build()
        self.assertFalse(self.output.exists())

    def test_unsafe_capture_is_rejected_before_delegated_provenance_reads_it(self):
        path = self.capture / "server-to-client.raw"
        original = path.read_bytes()
        base = frozen.legacy()
        with tempfile.TemporaryDirectory() as elsewhere:
            target = Path(elsewhere) / "source.raw"
            target.write_bytes(original)
            path.unlink()
            path.symlink_to(target)
            with patch.object(subject, "load_frozen", return_value=(frozen, base)), \
                 patch.object(base, "capture_provenance", wraps=base.capture_provenance) as delegated:
                with self.assertRaises(ValueError): self.build()
                delegated.assert_not_called()
        path.unlink()
        path.write_bytes(original + b" " * 50001)
        with patch.object(subject, "load_frozen", return_value=(frozen, base)), \
             patch.object(subject, "MAX_FILE", 50000), \
             patch.object(base, "capture_provenance", wraps=base.capture_provenance) as delegated:
            with self.assertRaises(ValueError): self.build()
            delegated.assert_not_called()

    def test_rejected_ack_is_not_reinvented_as_delivered(self):
        self.servers[1] = {"id": "1", "error": {"code": -1, "message": "rejected"}}
        self.sync_capture()
        result = self.build()
        self.assertEqual(result["acks"], [])

    def test_typed_identity_conflict_and_missing_trace_fail(self):
        self.servers[1]["id"] = 1
        self.sync_capture()
        with self.assertRaises(ValueError): self.build()
        self.servers[1]["id"] = "1"
        self.sync_capture()
        lines(self.trace, self.trace_rows[:1])
        with self.assertRaises(ValueError): self.build()

    def test_unknown_native_activity_and_unlinked_public_items_are_explicit(self):
        self.servers.append({"method": "item/completed", "params": {"threadId": "thread", "turnId": "unlinked",
            "item": {"id": "orphan", "type": "newNativeTool", "text": "DO_NOT_EXPORT"}}})
        self.sync_capture()
        result = self.build()
        self.assertTrue(result["unlinked_public_items"])
        self.assertNotIn("DO_NOT_EXPORT", json.dumps(result))

    def test_conflicting_public_item_identity_fails(self):
        duplicate = copy.deepcopy(self.servers[-1])
        duplicate["params"]["item"]["text"] += " altered"
        self.servers.append(duplicate)
        self.sync_capture()
        with self.assertRaises(ValueError): self.build()

    def test_outside_phase_symlink_and_helper_drift_fail(self):
        with tempfile.TemporaryDirectory() as elsewhere:
            with self.assertRaises(ValueError): subject.build_packet(self.phase, "run", Path(elsewhere) / "out.json")
        self.manifest["files"][0]["sha256"] = "0" * 64
        write(self.phase / "PHASE-MANIFEST.json", self.manifest)
        write(self.phase / "RUN-ONCE/admission.json", {"manifest_sha256": sha(self.phase / "PHASE-MANIFEST.json")})
        with self.assertRaises(ValueError): self.build()

    def test_end_hash_recheck_detects_changed_source_before_output(self):
        original = subject.SourceIndex.verify
        def mutate_then_verify(index):
            self.trace.write_text(self.trace.read_text() + "\n")
            return original(index)
        with patch.object(subject.SourceIndex, "verify", mutate_then_verify):
            with self.assertRaises(ValueError): self.build()
        self.assertFalse(self.output.exists())


if __name__ == "__main__":
    unittest.main()
