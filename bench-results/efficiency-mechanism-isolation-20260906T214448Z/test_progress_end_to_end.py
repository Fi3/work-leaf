"""Additive tests for the approved complete-known-work ledger; no provider calls."""
import copy
import json
import unittest
from unittest.mock import patch

import progress_counter as core
import progress_end_to_end as current


class CompletePlanTests(unittest.TestCase):
    def setUp(self):
        self.scope, self.parent, self.parent_pubs = current.load_contract()
        self.ledger = copy.deepcopy(self.parent)
        self.ledger.update(schema=5, scope_id=self.scope["id"], fixed_total=90,
                           pending=[item["id"] for item in self.scope["tasks"]],
                           research={"status": "paused", "zero_decision": None}, scope_issues=[])
        self.ledger["tasks"].update({item["id"]: {
            "definition": item, "status": "pending", "started_at": None,
            "result": None, "blocker": None, "preparation_seconds": 0,
        } for item in self.scope["tasks"]})
        self.ledger["history"].append(current.checkpoint(self.scope, self.parent))

    def pubs(self):
        return {"schema": 4, "scope_id": self.scope["id"], "fixed_total": 90,
                "checkpoints": self.parent_pubs["checkpoints"] +
                self.ledger["history"][len(self.parent["history"]):]}

    def reject(self):
        with self.assertRaises(ValueError):
            core.validate_published_progress(self.ledger, self.pubs())

    def close(self, identity, outcome="supported", mode="performed", answer=None):
        if (self.ledger["tasks"][identity]["definition"]["kind"] in {"screen", "confirmation"}
                and mode == "performed" and self.ledger["tasks"]["P01"]["status"] == "pending"):
            self.close("P01")
        at = "2026-09-15T12:00:00+00:00"
        item = self.ledger["tasks"][identity]
        item.update(status="done", started_at=at, result={
            "completed_at": at, "outcome": outcome, "checked": "Test fixture only",
            "evidence": [item["definition"]["result"]], "finding": "Fixture finding",
            "limits": "Declarations are not scientific proof", "gate": {
                "mode": mode, "setup_valid": True, "performed": mode == "performed",
                "deliverable_complete": True, "answer_kind": answer}})
        self.ledger["pending"].remove(identity)
        self.ledger["completed"].append(identity)
        self.publish_completion(identity)

    def publish_completion(self, identity):
        self.ledger["history"].append({
            "event": "task_completed", "scope_id": self.scope["id"],
            "at": self.ledger["tasks"][identity]["result"]["completed_at"],
            "done": len(self.ledger["completed"]), "todo": len(self.ledger["pending"]),
            "total": 90, "completed": list(self.ledger["completed"]),
            "completion": {"id": identity, "record": copy.deepcopy(self.ledger["tasks"][identity])}})

    def refresh_last(self, identity):
        self.ledger["history"][-1]["completion"]["record"] = copy.deepcopy(self.ledger["tasks"][identity])

    def visible(self):
        header = f'# DONE: {len(self.ledger["completed"])} | TODO: {len(self.ledger["pending"])} | TOTAL: 90'
        rows = ["| ID | Status | Remaining check |", "| --- | --- | --- |"]
        for definition in self.scope["tasks"]:
            identity = definition["id"]
            item = self.ledger["tasks"][identity]
            status = {"pending": "TODO", "checking": "CHECKING", "blocked": "BLOCKED", "done": "DONE"}[item["status"]]
            rows.append(f'| {identity} | {status} | {definition["question"]} |')
        return header + "\n\n" + "\n".join(rows), "# Note\n\n" + header[2:]

    def test_publication_preserves_every_old_record_and_checkpoint(self):
        self.assertEqual(core.validate_published_progress(self.ledger, self.pubs()), (74, 16))

    def test_exact_ninety_header(self):
        self.assertEqual(core.read_counts("# DONE: 74 | TODO: 16 | TOTAL: 90"), (74, 16))
        for value in ("# DONE: 73 | TODO: 17 | TOTAL: 90", "# DONE: 74 | TODO: 17 | TOTAL: 91"):
            with self.assertRaises(ValueError):
                core.read_counts(value)

    def test_visible_pending_and_blocked_rows(self):
        self.block()
        core.validate_visible_progress(self.ledger, *self.visible())

    def test_visible_row_cannot_hide_an_unrun_task(self):
        text, note = self.visible()
        with self.assertRaises(ValueError):
            core.validate_visible_progress(self.ledger, text.replace("| P03 | TODO |", "| P03 | DONE |"), note)

    def block(self):
        self.ledger["tasks"]["P01"].update(status="blocked", started_at=self.scope["approved_on"],
            blocker={"detected_at": self.scope["approved_on"], "reason": "Fixture setup failure",
                     "evidence": [self.scope["authority"]["path"]]})

    def test_blocked_setup_and_unrun_dependents_do_not_advance(self):
        self.block()
        self.assertEqual(core.validate_ledger(self.ledger), (74, 16))
        self.assertIsNone(self.ledger["tasks"]["P03"]["result"])

    def test_generated_check_cannot_start_behind_blocked_setup(self):
        self.block()
        self.ledger["tasks"]["P03"].update(status="checking", started_at=self.scope["approved_on"])
        self.reject()

    def test_generated_check_cannot_complete_behind_blocked_setup(self):
        self.block()
        self.close("P03")
        self.reject()

    def test_actual_preparation_overrun_can_be_preserved_as_blocked(self):
        self.block()
        self.ledger["tasks"]["P01"]["preparation_seconds"] = 3601
        self.assertEqual(core.validate_ledger(self.ledger), (74, 16))

    def test_blocker_requires_evidence(self):
        self.block()
        self.ledger["tasks"]["P01"]["blocker"]["evidence"] = []
        self.reject()

    def test_setup_failure_cannot_complete(self):
        self.close("P01", outcome="failed_setup")
        self.reject()

    def test_unrun_or_unqualified_screen_cannot_complete(self):
        for field in ("performed", "setup_valid", "deliverable_complete"):
            with self.subTest(field=field):
                self.setUp()
                self.close("P03", outcome="inconclusive")
                self.ledger["tasks"]["P03"]["result"]["gate"][field] = False
                self.refresh_last("P03")
                self.reject()

    def test_valid_negative_screen_closes_its_bounded_check(self):
        self.close("P03", outcome="unsupported")
        self.assertEqual(core.validate_ledger(self.ledger), (76, 14))

    def test_unavailable_offset_analysis_does_not_complete(self):
        self.close("P11", outcome="inconclusive")
        self.ledger["tasks"]["P11"]["result"]["gate"]["deliverable_complete"] = False
        self.refresh_last("P11")
        self.reject()

    def test_qualified_setup_requires_supported_outcome(self):
        self.close("P01", outcome="inconclusive")
        self.reject()

    def test_conditional_path_exclusion_is_not_a_setup_failure(self):
        self.close("P06", outcome="justified_not_needed", mode="absent_or_shared")
        self.assertEqual(core.validate_ledger(self.ledger), (75, 15))

    def test_nonconditional_screen_cannot_use_absent_path_shortcut(self):
        self.close("P03", outcome="justified_not_needed", mode="absent_or_shared")
        self.reject()

    def test_joint_coverage_discharge_needs_verified_attribution(self):
        self.close("P07", outcome="justified_not_needed", mode="covered_by_verified_joint")
        self.reject()

    def test_error_discharge_requires_proven_error_result(self):
        self.close("P14", outcome="justified_not_needed", mode="verified_benchmark_error")
        self.reject()

    def test_final_answer_cannot_close_early_or_without_answer_kind(self):
        self.close("P16", answer="causal_explanation")
        self.reject()

    def test_complete_causal_answer_requires_explicit_zero_decision(self):
        for definition in self.scope["tasks"][:-1]:
            self.close(definition["id"])
        self.close("P16", answer="causal_explanation")
        self.ledger["tasks"]["P16"]["result"]["evidence"] += [
            self.ledger["tasks"][key]["definition"]["result"] for key in ("P13", "P14")]
        self.refresh_last("P16")
        self.reject()
        self.ledger["research"] = {"status": "complete", "zero_decision": {
            "kind": "supported", "evidence": [self.ledger["tasks"]["P16"]["definition"]["result"]],
            "conclusion": "Fixture causal answer, not a scientific claim"}}
        self.assertEqual(core.validate_published_progress(self.ledger, self.pubs()), (90, 0))

    def test_verified_error_can_discharge_recipe_with_exact_evidence(self):
        for definition in self.scope["tasks"][:-1]:
            if definition["id"] != "P14":
                self.close(definition["id"], answer="benchmark_error" if definition["id"] == "P15" else None)
        self.close("P14", outcome="justified_not_needed", mode="verified_benchmark_error")
        self.ledger["tasks"]["P14"]["result"]["evidence"].append(
            self.ledger["tasks"]["P15"]["definition"]["result"])
        self.refresh_last("P14")
        self.assertEqual(core.validate_ledger(self.ledger), (89, 1))

    def test_no_old_completion_edits(self):
        self.ledger["tasks"]["R13"]["result"]["finding"] = "Reinterpreted"
        self.reject()

    def test_no_truncated_history_or_unapproved_task(self):
        for mutation in ("history", "task", "total", "scope"):
            with self.subTest(mutation=mutation):
                self.setUp()
                if mutation == "history": self.ledger["history"].pop(0)
                if mutation == "task": self.ledger["pending"].append("P17")
                if mutation == "total": self.ledger["fixed_total"] = 91
                if mutation == "scope": self.ledger["tasks"]["P08"]["definition"]["scope"] += " plus other experiments"
                self.reject()

    def test_publications_cannot_be_truncated(self):
        publications = self.pubs()
        publications["checkpoints"].pop(0)
        with self.assertRaises(ValueError):
            core.validate_published_progress(self.ledger, publications)

    def test_completed_result_cannot_be_rewritten(self):
        self.close("P03")
        self.ledger["tasks"]["P03"]["result"]["finding"] = "Rewritten"
        self.reject()

    def test_prep_spending_cannot_be_hidden_or_exceed_cap(self):
        self.ledger["tasks"]["P01"]["preparation_seconds"] = 3601
        self.reject()

    def test_new_discovery_requires_new_fact_and_reason_for_promise(self):
        issue = {
            "case": 1, "status": "unapproved", "classification": "new_promising_mechanism",
            "discovered_at": self.scope["approved_on"], "reason": "Fixture only",
            "new_fact": "New observation", "why_not_known": "Distinct fact not in frozen inventory",
            "why_promising": "Observed upstream work path with relevant token exposure",
            "evidence": [self.scope["authority"]["path"]],
            "proposed_tasks": [{"id": "P17", "question": "New question", "scope": "One bounded screen"}]}
        self.ledger["scope_issues"].append(issue)
        self.assertEqual(core.validate_ledger(self.ledger), (74, 16))
        issue["new_fact"] = ""
        self.reject()

    def test_known_omission_cannot_be_recorded_as_completed_or_autoapproved(self):
        self.ledger["scope_issues"] = [{"case": 2, "status": "approved", "classification": "known_omission"}]
        self.reject()

    def test_contract_files_are_hash_pinned(self):
        original = current.SCOPE_SHA256
        with patch.object(current, "SCOPE_SHA256", "0" * 64), self.assertRaises(ValueError):
            current.load_contract()
        self.assertNotEqual(original, "0" * 64)

    def test_actual_publication(self):
        ledger = json.loads((core.STUDY / "PROGRESS-CHECKLIST.json").read_text())
        pubs = json.loads((core.STUDY / "PROGRESS-PUBLICATION-HISTORY.json").read_text())
        self.assertEqual(core.validate_published_progress(ledger, pubs), (74, 16))
        core.validate_visible_progress(ledger, (core.ROOT / "hypotesis.md").read_text(),
                                      (core.ROOT / "ephemeral-note.md").read_text())
        core.validate_evidence_files(ledger)


if __name__ == "__main__":
    unittest.main()
