"""Regression tests for the user-approved finite task counter; no provider calls."""

import copy
import json
import pathlib
import unittest

from progress_counter import (
    ROOT, STUDY, load_contract, migration_checkpoint, read_counts,
    validate_ledger, validate_published_progress, validate_visible_progress,
    validate_evidence_files,
)


class ProgressCounterTests(unittest.TestCase):
    def setUp(self):
        self.scope, self.legacy, self.old_publications = load_contract()
        self.definitions = {task["id"]: task for task in self.scope["tasks"]}

    def fixture(self):
        ledger = {
            "schema": 3, "scope_id": self.scope["id"], "fixed_total": 66,
            "completed": list(self.legacy["completed"]),
            "pending": list(self.definitions),
            "tasks": {identity: {"definition": copy.deepcopy(definition),
                                "status": "pending", "started_at": None, "result": None}
                      for identity, definition in self.definitions.items()},
            "history": copy.deepcopy(self.legacy["history"]) + [
                migration_checkpoint(self.scope, self.legacy)],
            "research": {"status": "paused", "zero_decision": None},
            "scope_issues": [],
        }
        return ledger

    def publications(self, ledger):
        return {
            "schema": 2, "scope_id": self.scope["id"], "fixed_total": 66,
            "checkpoints": copy.deepcopy(self.old_publications["checkpoints"])
            + copy.deepcopy(ledger["history"][len(self.legacy["history"]):]),
        }

    def complete(self, ledger, identity, outcome="inconclusive"):
        item = ledger["tasks"][identity]
        item.update(status="done", started_at="2026-09-14T13:00:00+00:00",
                    result={"completed_at": "2026-09-14T13:01:00+00:00",
                            "outcome": outcome, "checked": "Finite declared check.",
                            "evidence": ["test-only-evidence.md"],
                            "finding": "Test fixture, not a scientific result.",
                            "limits": "Test fixture, not a causal claim."})
        ledger["pending"].remove(identity)
        ledger["completed"].append(identity)
        ledger["history"].append({
            "event": "task_completed", "scope_id": self.scope["id"],
            "at": item["result"]["completed_at"], "done": len(ledger["completed"]),
            "todo": len(ledger["pending"]), "total": 66,
            "completed": list(ledger["completed"]),
            "completion": {"id": identity, "record": copy.deepcopy(item)},
        })

    def zero(self):
        ledger = self.fixture()
        for identity in self.definitions:
            self.complete(ledger, identity)
        return ledger

    def permission(self, ledger, cases):
        ledger["research"] = {
            "status": "awaiting_permission",
            "zero_decision": {
                "kind": "permission_required", "cases": cases,
                "reason": "The listed checks did not identify the historical share.",
                "evidence": [self.definitions["R12"]["result"]],
                "additional_tasks": [
                    {"id": "proposed-extra", "question": "A specific missing check?",
                     "scope": "One bounded check, not yet authorized."}],
                "requested_todo_increase": 1,
                "permission_question": "May I add this one specified task?",
            },
        }

    def visible(self, ledger):
        done, todo = len(ledger["completed"]), len(ledger["pending"])
        header = f"# DONE: {done} | TODO: {todo} | TOTAL: 66"
        rows = ["| ID | Status | Remaining check |", "| --- | --- | --- |"]
        for identity, definition in self.definitions.items():
            status = {"pending": "TODO", "checking": "CHECKING", "done": "DONE"}[
                ledger["tasks"][identity]["status"]]
            rows.append(f'| {identity} | {status} | {definition["question"]} |')
        return header + "\n\n" + "\n".join(rows), (
            "# Provisional investigation ledger\n\nLive analysis counter: **"
            + header.removeprefix("# ") + "**"
        )

    def assert_invalid(self, ledger):
        with self.assertRaises(ValueError):
            validate_published_progress(ledger, self.publications(ledger))

    def test_actual_ledger_header_tasks_note_and_evidence(self):
        ledger = json.loads((STUDY / "PROGRESS-CHECKLIST.json").read_text())
        publications = json.loads((STUDY / "PROGRESS-PUBLICATION-HISTORY.json").read_text())
        validate_published_progress(ledger, publications)
        validate_visible_progress(ledger, (ROOT / "hypotesis.md").read_text(),
                                  (ROOT / "ephemeral-note.md").read_text())
        validate_evidence_files(ledger)

    def test_approved_correction_retains_all_54_and_old_checkpoints(self):
        ledger = self.fixture()
        self.assertEqual(validate_published_progress(ledger, self.publications(ledger)), (54, 12))
        self.assertEqual(ledger["history"][:2], self.legacy["history"])
        self.assertEqual(len(ledger["completed"]), 54)

    def test_old_aggregate_and_superseded_zero_are_not_live_ledgers(self):
        for name in ("progress-archive/PROGRESS-CHECKLIST-v2.json",
                     "PROGRESS-CHECKLIST-SUPERSEDED-49.json"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                validate_ledger(json.loads((STUDY / name).read_text()))

    def test_header_total_floor_and_first_line_are_strict(self):
        self.assertEqual(read_counts("# DONE: 54 | TODO: 12 | TOTAL: 66"), (54, 12))
        for header in ("# DONE: 54 | TODO: 1 | TOTAL: 55",
                       "# DONE: 53 | TODO: 13 | TOTAL: 66",
                       "# DONE: 54 | TODO: 13 | TOTAL: 66",
                       "# DONE: 67 | TODO: 0 | TOTAL: 67",
                       "text\n# DONE: 54 | TODO: 12 | TOTAL: 66"):
            with self.subTest(header=header), self.assertRaises(ValueError):
                read_counts(header)

    def test_no_extra_duplicate_replacement_or_removed_task(self):
        for mutation in ("extra", "duplicate", "replacement", "removed"):
            ledger = self.fixture()
            if mutation == "extra":
                ledger["pending"].append("R13")
                ledger["fixed_total"] += 1
            elif mutation == "duplicate":
                ledger["pending"][-1] = "R01"
            elif mutation == "replacement":
                ledger["pending"][-1] = "R13"
                ledger["tasks"]["R13"] = ledger["tasks"].pop("R12")
            else:
                ledger["pending"].pop()
                ledger["tasks"].pop("R12")
            with self.subTest(mutation=mutation):
                self.assert_invalid(ledger)

    def test_same_id_cannot_hide_changed_question_scope_or_result_location(self):
        for field in ("question", "scope", "result"):
            ledger = self.fixture()
            ledger["tasks"]["R04"]["definition"][field] += " and extra work"
            with self.subTest(field=field):
                self.assert_invalid(ledger)

    def test_hidden_supplementary_work_or_task_fields_are_rejected(self):
        for location in ("ledger", "task"):
            ledger = self.fixture()
            target = ledger if location == "ledger" else ledger["tasks"]["R01"]
            target["supplementary"] = [{"id": "extra", "status": "checking"}]
            with self.subTest(location=location):
                self.assert_invalid(ledger)

    def test_historical_records_and_history_cannot_be_erased(self):
        for mutation in ("record", "history", "checkpoint", "approval"):
            ledger = self.fixture()
            if mutation == "record":
                ledger["completed"].remove("G01")
                ledger["pending"].append("G01")
            elif mutation == "history":
                ledger["history"] = ledger["history"][1:]
            elif mutation == "checkpoint":
                ledger["history"][0]["done"] = 51
            else:
                ledger["history"][2]["approval_id"] = "agent-approved"
            with self.subTest(mutation=mutation):
                self.assert_invalid(ledger)

    def test_starting_a_task_does_not_advance_count(self):
        ledger = self.fixture()
        ledger["tasks"]["R01"].update(status="checking",
                                    started_at="2026-09-14T13:00:00+00:00")
        self.assertEqual(validate_ledger(ledger), (54, 12))
        ledger["tasks"]["R01"]["started_at"] = None
        self.assert_invalid(ledger)

    def test_every_terminal_outcome_needs_actual_dated_result(self):
        for outcome in ("supported", "unsupported", "inconclusive",
                        "failed_setup", "justified_not_needed"):
            ledger = self.fixture()
            self.complete(ledger, "R01", outcome)
            self.assertEqual(validate_ledger(ledger), (55, 11))
        for field in ("completed_at", "evidence", "checked", "finding", "limits"):
            ledger = self.fixture()
            self.complete(ledger, "R01")
            ledger["tasks"]["R01"]["result"][field] = None
            self.assert_invalid(ledger)

    def test_protocol_or_timeout_alone_is_not_completion(self):
        for outcome in ("protocol_written", "timed_out", "pending", "pass"):
            ledger = self.fixture()
            self.complete(ledger, "R01", outcome)
            self.assert_invalid(ledger)

    def test_completed_result_is_preserved_by_publication(self):
        ledger = self.fixture()
        self.complete(ledger, "R01")
        ledger["tasks"]["R01"]["result"]["finding"] = "Rewritten conclusion."
        self.assert_invalid(ledger)

    def test_each_completion_has_one_matching_publication(self):
        ledger = self.fixture()
        publications = self.publications(ledger)
        self.complete(ledger, "R01")
        with self.assertRaises(ValueError):
            validate_published_progress(ledger, publications)
        self.assertEqual(validate_published_progress(ledger, self.publications(ledger)), (55, 11))
        self.complete(ledger, "R02")
        ledger["history"].pop(-2)
        self.assert_invalid(ledger)

    def test_published_advance_cannot_be_rolled_back_even_with_truncated_ledger(self):
        baseline = self.fixture()
        advanced = self.fixture()
        self.complete(advanced, "R01")
        with self.assertRaises(ValueError):
            validate_published_progress(baseline, self.publications(advanced))

    def test_reopening_a_task_or_swapping_completed_ids_is_rejected(self):
        ledger = self.fixture()
        self.complete(ledger, "R01")
        ledger["completed"].remove("R01")
        ledger["pending"].append("R01")
        ledger["tasks"]["R01"] = self.fixture()["tasks"]["R01"]
        self.assert_invalid(ledger)

    def test_malformed_publication_archives_are_rejected(self):
        ledger = self.fixture()
        good = self.publications(ledger)
        for bad in (None, {}, dict(good, schema=1), dict(good, fixed_total=55),
                    dict(good, checkpoints=[]), dict(good, checkpoints="invalid")):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                validate_published_progress(ledger, bad)

    def test_scope_issues_are_unapproved_notes_not_untracked_execution(self):
        ledger = self.fixture()
        ledger["scope_issues"] = [{
            "case": 2, "status": "unapproved", "reason": "A missing finite check.",
            "proposed_tasks": [{"id": "extra", "question": "An omitted check?",
                                "scope": "One finite observation."}],
        }]
        self.assertEqual(validate_ledger(ledger), (54, 12))
        ledger["scope_issues"][0]["status"] = "checking"
        self.assert_invalid(ledger)

    def test_zero_without_explicit_decision_is_rejected(self):
        ledger = self.zero()
        for status in ("active", "paused", "complete", "awaiting_permission"):
            ledger["research"]["status"] = status
            self.assert_invalid(ledger)

    def test_zero_may_stop_for_case_one_case_two_or_both(self):
        for cases in ([1], [2], [1, 2]):
            ledger = self.zero()
            self.permission(ledger, cases)
            self.assertEqual(validate_ledger(ledger), (66, 0))

    def test_zero_permission_hold_cannot_run_or_claim_success(self):
        ledger = self.zero()
        self.permission(ledger, [2])
        for status in ("active", "complete"):
            ledger["research"]["status"] = status
            self.assert_invalid(ledger)

    def test_zero_requires_exact_additional_tasks_count_reason_and_question(self):
        for field in ("cases", "reason", "additional_tasks", "permission_question", "evidence"):
            ledger = self.zero()
            self.permission(ledger, [2])
            ledger["research"]["zero_decision"][field] = []
            self.assert_invalid(ledger)
        ledger = self.zero()
        self.permission(ledger, [2])
        ledger["research"]["zero_decision"]["requested_todo_increase"] = 2
        self.assert_invalid(ledger)

    def test_zero_supported_answer_requires_evidence_and_no_unapproved_work(self):
        ledger = self.zero()
        ledger["research"] = {
            "status": "complete",
            "zero_decision": {"kind": "supported", "evidence": [
                self.definitions["R12"]["result"]], "conclusion": "Test-only supported answer."},
        }
        self.assertEqual(validate_ledger(ledger), (66, 0))
        ledger["scope_issues"] = [{"case": 1, "status": "unapproved", "reason": "More work",
                                 "proposed_tasks": [{"id": "extra", "question": "Why?",
                                                     "scope": "A finite extra check."}]}]
        self.assert_invalid(ledger)

    def test_nonzero_cannot_claim_finished_analysis(self):
        ledger = self.fixture()
        ledger["research"]["status"] = "complete"
        self.assert_invalid(ledger)

    def test_final_coverage_cannot_close_before_its_eleven_input_tasks(self):
        ledger = self.fixture()
        self.complete(ledger, "R12")
        self.assert_invalid(ledger)

    def test_visible_header_note_and_each_of_twelve_rows_are_checked(self):
        ledger = self.fixture()
        hypothesis, note = self.visible(ledger)
        validate_visible_progress(ledger, hypothesis, note)
        for bad_hypothesis, bad_note in (
                (hypothesis.replace("R02 | TODO", "R02 | DONE"), note),
                (hypothesis.replace("R12 | TODO", "R13 | TODO"), note),
                (hypothesis.replace(self.definitions["R04"]["question"], "Do unlimited work."), note),
                (hypothesis, note.replace("TODO: 12", "TODO: 1")),
                ("\n" + hypothesis, note)):
            with self.assertRaises(ValueError):
                validate_visible_progress(ledger, bad_hypothesis, bad_note)

    def test_missing_result_file_is_not_live_evidence(self):
        ledger = self.fixture()
        self.complete(ledger, "R01")
        with self.assertRaises(ValueError):
            validate_evidence_files(ledger)


if __name__ == "__main__":
    unittest.main()
