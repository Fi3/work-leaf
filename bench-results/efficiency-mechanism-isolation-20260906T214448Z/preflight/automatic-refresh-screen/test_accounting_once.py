"""Temporary-source orchestration tests; no real accounting calls.

Only synthetic fixtures are created by this test module. The fake
helper pin and invoke callback are test monkeypatches, never CLI overrides.
"""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import execute_accounting_once as subject


NEW = [f"automatic-refresh-01-workflow-{n:03}" for n in (1, 2, 3)]
OLD = [f"work-units-01-workflow-{n:03}" for n in (2, 4, 6, 9, 10, 12)]


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False).encode()


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def usage():
    return dict(input_tokens=7, cached_input_tokens=2, output_tokens=3, reasoning_output_tokens=1,
                uncached_input_tokens=5, raw_input_plus_output=10, uncached_input_plus_output=8)


def full_result(run_id, sources, status="exact"):
    response = {"thread_id": "thread-" + run_id, "turn_id": "turn-" + run_id,
                "usage": usage(), "sources": [{"path": next(iter(sources)), "native_line": 1}]}
    return dict(schema="work-leaf-read-identity-accounting-v1", run_id=run_id,
        status="unknown" if status == "ineligible" else "validated", errors=["retained failure"] if status == "ineligible" else [], warnings=[],
        source_sha256=copy.deepcopy(sources), measurement={"status": status, "bounds": None if status == "ineligible" else {
            "raw_input_plus_output": {"lower": 10, "upper": None if status == "unbounded_accounting_gap" else 10},
            "uncached_input_plus_output": {"lower": 8, "upper": None if status == "unbounded_accounting_gap" else 8}},
            "gap_inventory": {"errors": [], "gaps": [], "raw_responses": {"response-" + run_id: response}}},
        original_observer_ledger={"errors": []}, corrected_scope={"usage": usage(), "captures": []},
        exact_response_evidence={"response-" + run_id: response}, whole_workflow_hidden_call_completeness_proven=False)


class OnceTests(unittest.TestCase):
    def save(self, path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        raw = canonical(value); path.write_bytes(raw)
        return {"path": str(path), "sha256": digest(raw)}

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="synthetic-c08-accounting-")
        self.addCleanup(self.temp.cleanup); self.root = Path(self.temp.name)
        self.helper = self.root / "frozen/bench-results/study/accounting_untracked_reads.py"
        self.helper.parent.mkdir(parents=True)
        self.helper.write_text("# Synthetic non-accounting source; never imported by invoke mock.\n")
        self.helper_sha = digest(self.helper.read_bytes())
        self.source = self.root / "closed-source.txt"; self.source.write_text("synthetic closed bytes")
        self.sources = {str(self.helper): self.helper_sha, str(self.source): digest(self.source.read_bytes())}
        for code in (Path(subject.__file__).resolve(), Path(sys.executable).resolve()):
            self.sources[str(code)] = digest(code.read_bytes())
        self.manifest = {"files": [{"path": str(self.helper), "sha256": self.helper_sha, "role": "frozen-evidence"}],
                         "model": "synthetic-model", "reasoning_effort": "synthetic-effort", "unchanged_extra": {"typed": False}}
        self.manifest_ref = self.save(self.root / "PHASE-MANIFEST.json", self.manifest)
        self.entries = [{"id": run, "condition": "automatic-changed-refresh-full", "started_at": "2026-01-01T00:00:00Z",
                         "finished_at": "2026-01-01T00:01:00Z", "launch_status": "completed", "launcher_exit_code": int(i == 0),
                         "artifact": str(self.root / run), "report": str(self.root / run / "report.json")} for i, run in enumerate(NEW)]
        self.schedule = [{"run_id": row["id"], **{k: row[k] for k in ("condition", "artifact", "report")}} for row in self.entries]
        self.manifest.update(schedule=self.schedule, launch_order=[NEW[::-1]], randomization={"synthetic": True})
        self.schedule_ref = self.save(self.root / "SCHEDULE.json", {"runs": self.schedule,
            "launch_order": self.manifest["launch_order"], "randomization": self.manifest["randomization"]})
        self.manifest["schedule_sha256"] = self.schedule_ref["sha256"]
        self.manifest_ref = self.save(self.root / "PHASE-MANIFEST.json", self.manifest)
        score = {"phase_manifest": self.manifest_ref["path"], "phase_manifest_sha256": self.manifest_ref["sha256"], "runs": self.entries}
        self.score_ref = self.save(self.root / "score-manifest.json", score)
        closed = {"finished_at": "2026-01-01T00:01:01Z", "manifest_sha256": self.manifest_ref["sha256"], "runs": self.entries}
        self.closed_ref = self.save(self.root / "PHASE-RESULT.json", closed)
        self.terminals = {run: self.save(self.root / "logs" / (run + ".exit.json"), row) for run, row in zip(NEW, self.entries)}
        self.baselines = []
        for run in OLD:
            full = full_result(run, self.sources)
            projected = copy.deepcopy(full); evidence = projected["exact_response_evidence"]
            projected["exact_response_evidence"] = {"count": len(evidence), "sha256": digest(canonical(evidence)), "retention": "synthetic fixture"}
            projected["exact_helper_result_sha256"] = digest(canonical(full))
            receipt = {"kind": "run", "original_entry": {"id": run, "launcher_exit_code": int(run.endswith("010"))}, "accounting": projected}
            self.baselines.append({"run_id": run, "expected_response_count": 1,
                "receipt": self.save(self.root / "baselines" / run / "receipt.json", receipt),
                "response_supplement": self.save(self.root / "baselines" / run / "ids.json", {
                    "schema": "work-leaf-admitted-provider-ledger-execution-v1", "original_entry": receipt["original_entry"],
                    "result": {"schema": "work-leaf-provider-ledger-qualification-v1", "original": projected},
                    "response_evidence": full["exact_response_evidence"]})})
        self.output = self.root / "postcapture"; self.output.mkdir()
        self.scope = {"schema": "work-leaf-c08-original-accounting-once-v1", "new_run_ids": NEW, "baseline_run_ids": OLD,
                      "execution_authorized": True,
                      "phase_manifest": self.manifest_ref, "score_manifest": self.score_ref, "phase_result": self.closed_ref,
                      "terminals": self.terminals, "helper": {"path": str(self.helper), "sha256": self.helper_sha},
                      "source_sha256": self.sources, "baselines": self.baselines, "sessions_root": str(self.root / "sessions"),
                      "output_root": str(self.output), "per_run_timeout_seconds": 600}
        self.scope["schedule"] = self.schedule_ref
        self.scope["outputs"] = {"attempt": str(self.output / "ACCOUNTING-ORIGINAL-ATTEMPT.json"),
            "final": str(self.output / "COMMON-ACCOUNTING-ORIGINAL.json"),
            "ownership": str(self.output / "COMMON-ACCOUNTING-ORIGINAL-RESPONSE-OWNERSHIP.json")}
        for run in NEW:
            self.scope["outputs"][run + ":full"] = str(self.output / run / "COMMON-ACCOUNTING-ORIGINAL-FULL.json")
            self.scope["outputs"][run + ":projection"] = str(self.output / run / "COMMON-ACCOUNTING-ORIGINAL.json")
            for name in ("INPUT.json", "RESULT.json", "stdout", "stderr", "EXECUTION.json"):
                self.scope["outputs"][run + ":worker:" + name] = str(self.output / run / "worker" / name)
        (self.root / "sessions").mkdir()
        self.calls = []; self.results = {run: full_result(run, self.sources) for run in NEW}
        self.addCleanup(patch.stopall)
        patch.object(subject, "HELPER_SHA", self.helper_sha).start()
        patch.object(subject, "DEPENDENCY_PINS", {}).start()
        self.invoke = patch.object(subject, "invoke_accounting", side_effect=self.fake_invoke).start()

    def fake_invoke(self, path, raw, entry, frozen, sessions_root, timeout_seconds, operation_root):
        self.assertEqual(path, self.helper)
        self.assertEqual(digest(raw), self.helper_sha)
        self.assertEqual(frozen, self.manifest)
        self.assertIs(type(frozen["unchanged_extra"]["typed"]), bool)
        self.assertEqual(sessions_root, self.root / "sessions")
        self.assertEqual(timeout_seconds, 600)
        self.assertEqual(operation_root, self.output / entry["id"] / "worker")
        self.calls.append(copy.deepcopy(entry))
        return copy.deepcopy(self.results[entry["id"]])

    def execute(self):
        ref = self.save(self.root / "SCOPE.json", self.scope)
        return subject.execute(Path(ref["path"]), ref["sha256"])

    def refresh_manifest_refs(self):
        self.scope["phase_manifest"] = self.save(self.root / "PHASE-MANIFEST.json", self.manifest)
        self.scope["score_manifest"] = self.save(self.root / "score-manifest.json", {
            "phase_manifest": self.scope["phase_manifest"]["path"],
            "phase_manifest_sha256": self.scope["phase_manifest"]["sha256"], "runs": self.entries})
        self.scope["phase_result"] = self.save(self.root / "PHASE-RESULT.json", {
            "finished_at": "2026-01-01T00:01:01Z", "manifest_sha256": self.scope["phase_manifest"]["sha256"], "runs": self.entries})

    def test_exact_three_calls_and_six_reused_receipts_preserve_failed_outcomes(self):
        result = self.execute()
        self.assertEqual(self.calls, self.entries)
        self.assertEqual([x["run_id"] for x in result["runs"]], NEW + OLD)
        self.assertEqual(len(result["response_ownership"]), 9)
        self.assertEqual(result["runs"][0]["original_entry"]["launcher_exit_code"], 1)
        old = next(x for x in result["runs"] if x["run_id"].endswith("010"))
        self.assertEqual(old["original_entry"]["launcher_exit_code"], 1)

    def test_unknown_and_unbounded_original_results_are_not_zero_filled_or_rerun(self):
        self.results[NEW[0]] = full_result(NEW[0], self.sources, "ineligible")
        self.results[NEW[1]] = full_result(NEW[1], self.sources, "unbounded_accounting_gap")
        result = self.execute()
        self.assertEqual(len(self.calls), 3)
        self.assertIsNone(result["runs"][0]["accounting"]["measurement"]["bounds"])
        self.assertIsNone(result["runs"][1]["accounting"]["measurement"]["bounds"]["raw_input_plus_output"]["upper"])
        self.assertEqual(result["runs"][0]["accounting"]["errors"], ["retained failure"])

    def test_closed_worker_exception_is_terminal_without_repeat_and_preserves_nine_rows(self):
        def fail(*args):
            self.fake_invoke(*args)
            raise subject.WorkerFailure(kind="worker_exception", closed=True, exit_code=7)
        self.invoke.side_effect = fail
        result = self.execute()
        self.assertEqual(self.calls, self.entries)
        self.assertEqual(len(result["runs"]), 9)
        self.assertEqual([row["execution_status"] for row in result["runs"][:3]], ["worker_exception"] * 3)
        self.assertTrue(all(row["accounting"] is None for row in result["runs"][:3]))
        with self.assertRaises(ValueError): self.execute()
        self.assertEqual(len(self.calls), 3)

    def test_closed_worker_timeout_is_terminal_without_repeat_and_preserves_nine_rows(self):
        def timeout(*args):
            self.fake_invoke(*args)
            raise subject.WorkerFailure(kind="worker_timeout", closed=True, exit_code=-9)
        self.invoke.side_effect = timeout
        result = self.execute()
        self.assertEqual(self.calls, self.entries)
        self.assertEqual(len(result["runs"]), 9)
        self.assertEqual([row["execution_status"] for row in result["runs"][:3]], ["worker_timeout"] * 3)
        self.assertTrue(all(row["accounting"] is None for row in result["runs"][:3]))
        with self.assertRaises(ValueError): self.execute()
        self.assertEqual(len(self.calls), 3)

    def test_unproven_worker_closure_blocks_later_calls_without_dropping_rows(self):
        def uncertain(*args):
            self.fake_invoke(*args)
            raise subject.WorkerFailure(kind="worker_timeout", closed=False, exit_code=None)
        self.invoke.side_effect = uncertain
        result = self.execute()
        self.assertEqual(len(self.calls), 1)
        self.assertEqual(len(result["runs"]), 9)
        self.assertEqual([row["execution_status"] for row in result["runs"][1:3]], ["not_called_after_worker_uncertainty"] * 2)
        self.assertTrue(result["execution_errors"])
        for row in result["runs"][:3]:
            saved = json.loads(Path(row["receipt"]["path"]).read_bytes())
            self.assertEqual(saved["execution_status"], row["execution_status"])

    def test_pinned_postcapture_witness_is_not_an_output_collision(self):
        witness = self.output / NEW[0] / "closed-delivery.json"
        ref = self.save(witness, {"synthetic": "source witness"})
        self.scope["source_sha256"][ref["path"]] = ref["sha256"]
        result = self.execute()
        self.assertEqual(len(self.calls), 3)
        self.assertEqual(result["source_sha256"][ref["path"]], ref["sha256"])

    def test_worker_output_declaration_mismatch_prevents_attempt(self):
        self.scope["outputs"][NEW[1] + ":worker:stderr"] = str(self.output / "wrong-stderr")
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()
        self.assertFalse((self.output / "ACCOUNTING-ORIGINAL-ATTEMPT.json").exists())

    def test_schedule_mismatch_prevents_attempt(self):
        schedule = json.loads(Path(self.schedule_ref["path"]).read_bytes())
        schedule["runs"][0]["artifact"] = str(self.root / "other-artifact")
        self.scope["schedule"] = self.save(Path(self.schedule_ref["path"]), schedule)
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()
        self.assertFalse((self.output / "ACCOUNTING-ORIGINAL-ATTEMPT.json").exists())

    def test_prior_attempt_forbids_replay_even_when_no_result_exists(self):
        (self.output / "ACCOUNTING-ORIGINAL-ATTEMPT.json").write_text("retained attempt")
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()

    def test_disarmed_draft_is_rejected_before_attempt_or_call(self):
        self.scope["execution_authorized"] = False
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()
        self.assertFalse((self.output / "ACCOUNTING-ORIGINAL-ATTEMPT.json").exists())

    def test_existing_or_symlink_output_is_rejected_before_attempt_or_call(self):
        target = self.output / "COMMON-ACCOUNTING-ORIGINAL.json"
        target.symlink_to(self.root / "absent-target")
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()
        self.assertFalse((self.output / "ACCOUNTING-ORIGINAL-ATTEMPT.json").exists())

    def test_publication_readiness_failure_happens_before_any_call(self):
        with patch.object(subject, "publication_preflight", side_effect=OSError("synthetic no writer")):
            with self.assertRaises((OSError, ValueError)): self.execute()
        self.invoke.assert_not_called()

    def test_duplicate_helper_manifest_entry_is_not_accepted(self):
        self.manifest["files"].append(dict(self.manifest["files"][0]))
        self.refresh_manifest_refs()
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()

    def test_wrong_helper_role_is_not_accepted(self):
        self.manifest["files"][0]["role"] = "unfrozen-code"
        self.refresh_manifest_refs()
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()

    def test_compilation_keeps_actual_file_and_relative_dependency_root(self):
        raw = b"from pathlib import Path\nHERE=Path(__file__).resolve().parent\nROOT=HERE.parents[1]\n"
        self.helper.write_bytes(raw)
        self.manifest["files"][0]["sha256"] = digest(raw)
        with patch.object(subject, "HELPER_SHA", digest(raw)):
            module = subject.compile_accounting(self.helper, raw, self.manifest)
        self.assertEqual(module.__file__, str(self.helper))
        self.assertEqual(module.HERE, self.helper.parent)
        self.assertEqual(module.ROOT, self.helper.parent.parents[1])

    def test_different_helper_bytes_cannot_use_the_declared_manifest_pin(self):
        raw = self.helper.read_bytes() + b"# different\n"
        with self.assertRaises(ValueError): subject.compile_accounting(self.helper, raw, self.manifest)

    def test_helper_source_or_predeclared_input_drift_prevents_calls(self):
        self.source.write_text("changed")
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()

    def test_post_call_source_drift_retains_result_and_blocks_unsafe_next_call(self):
        def drift(*args):
            value = self.fake_invoke(*args); self.source.write_text("after first call")
            return value
        self.invoke.side_effect = drift
        result = self.execute()
        self.assertEqual(len(self.calls), 1)
        self.assertEqual(len(result["runs"]), 9)
        self.assertEqual(result["runs"][1]["execution_status"], "not_called_after_source_failure")
        self.assertEqual(result["runs"][0]["accounting"]["status"], "validated")
        self.assertTrue(result["integrity_errors"])

    def test_baseline_projection_hash_mismatch_prevents_new_calls(self):
        ref = self.scope["baselines"][0]["receipt"]; value = json.loads(Path(ref["path"]).read_bytes())
        value["accounting"]["exact_response_evidence"]["sha256"] = "0" * 64
        self.scope["baselines"][0]["receipt"] = self.save(Path(ref["path"]), value)
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()

    def test_baseline_map_count_mismatch_prevents_new_calls(self):
        self.scope["baselines"][0]["expected_response_count"] = 2
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()

    def test_a_baseline_cannot_be_omitted_or_repeated(self):
        self.scope["baselines"][-1] = copy.deepcopy(self.scope["baselines"][0])
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()

    def test_actual_saved_supplement_envelope_and_response_projection_are_bound(self):
        result = self.execute()
        self.assertEqual(len(self.calls), 3)
        self.assertEqual(len(result["response_ownership"]), 9)

    def test_supplement_original_projection_cannot_conflict_with_full_response_map(self):
        baseline = self.scope["baselines"][0]; ref = baseline["response_supplement"]
        value = json.loads(Path(ref["path"]).read_bytes())
        value["result"]["original"]["exact_response_evidence"]["sha256"] = "0" * 64
        baseline["response_supplement"] = self.save(Path(ref["path"]), value)
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()

    def test_supplement_original_entry_cannot_claim_another_workflow(self):
        baseline = self.scope["baselines"][0]; ref = baseline["response_supplement"]
        value = json.loads(Path(ref["path"]).read_bytes()); value["original_entry"]["id"] = OLD[1]
        baseline["response_supplement"] = self.save(Path(ref["path"]), value)
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()

    def test_cross_run_response_reuse_fails_identity_without_recomputing_originals(self):
        old = full_result(OLD[0], self.sources)["exact_response_evidence"]
        self.results[NEW[0]]["exact_response_evidence"] = copy.deepcopy(old)
        result = self.execute()
        self.assertEqual(len(self.calls), 3)
        self.assertTrue(result["identity_errors"])

    def test_full_result_hash_precedes_projection_and_original_bytes_remain(self):
        result = self.execute()
        for row in result["runs"][:3]:
            raw = Path(row["full_result"]["path"]).read_bytes()
            self.assertEqual(digest(raw), row["full_result"]["sha256"])
            self.assertEqual(json.loads(raw), self.results[row["run_id"]])
            self.assertEqual(row["accounting"]["exact_helper_result_sha256"], digest(canonical(self.results[row["run_id"]])))

    def test_failed_result_publication_cannot_trigger_an_extra_call_or_replay(self):
        with patch.object(subject, "publish_result", side_effect=OSError("synthetic publication failure")):
            result = self.execute()
        self.assertLessEqual(len(self.calls), 3)
        self.assertTrue(result["publication_errors"])
        previous = len(self.calls)
        with self.assertRaises(ValueError): self.execute()
        self.assertEqual(len(self.calls), previous)

    def test_unlisted_returned_source_is_not_retroactively_admitted(self):
        self.results[NEW[0]]["source_sha256"][str(self.root / "not-admitted")] = "0" * 64
        result = self.execute()
        self.assertTrue(result["integrity_errors"])
        self.assertEqual(len(result["runs"]), 9)

    def test_running_or_missing_terminal_row_prevents_all_calls(self):
        self.entries[1]["launch_status"] = "running"; self.entries[1]["finished_at"] = None
        self.scope["score_manifest"] = self.save(self.root / "score-manifest.json", {
            "phase_manifest": self.manifest_ref["path"], "phase_manifest_sha256": self.manifest_ref["sha256"], "runs": self.entries})
        with self.assertRaises(ValueError): self.execute()
        self.invoke.assert_not_called()


class SupervisionTests(unittest.TestCase):
    """OS-child qualification, never an accounting/bootstrap invocation."""

    def test_nonaccounting_child_nonzero_exit_and_output_are_preserved(self):
        with tempfile.TemporaryDirectory(prefix="synthetic-c08-supervisor-") as directory:
            root = Path(directory); stdout = root / "stdout"; stderr = root / "stderr"
            argv = [sys.executable, "-I", "-B", "-c", "import sys; print('synthetic child'); print('synthetic failure', file=sys.stderr); sys.exit(7)"]
            result = subject.supervise_worker(argv, stdout, stderr, timeout_seconds=5, grace_seconds=0.2)
            self.assertTrue(result["closed"])
            self.assertFalse(result["timed_out"])
            self.assertEqual(result["exit_code"], 7)
            self.assertEqual(stdout.read_text(), "synthetic child\n")
            self.assertEqual(stderr.read_text(), "synthetic failure\n")

    def test_nonaccounting_sleeper_is_waited_after_timeout_and_escalation(self):
        with tempfile.TemporaryDirectory(prefix="synthetic-c08-supervisor-") as directory:
            root = Path(directory); stdout = root / "stdout"; stderr = root / "stderr"
            argv = [sys.executable, "-I", "-B", "-c", "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); print('synthetic ready', flush=True); time.sleep(30)"]
            result = subject.supervise_worker(argv, stdout, stderr, timeout_seconds=1, grace_seconds=0.2)
            self.assertTrue(result["timed_out"])
            self.assertTrue(result["closed"])
            self.assertIsNotNone(result["exit_code"])
            self.assertLess(result["exit_code"], 0)
            self.assertIn("synthetic ready", stdout.read_text())


if __name__ == "__main__":
    unittest.main()
