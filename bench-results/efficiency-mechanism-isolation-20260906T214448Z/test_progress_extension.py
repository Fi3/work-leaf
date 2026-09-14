"""Additive regression checks for the explicitly approved eight-task extension."""
import copy
import unittest

import progress_counter as counter
import progress_extension as extension


class ExtensionTests(unittest.TestCase):
    def setUp(self):
        self.scope, self.parent, self.publications = extension.load_contract()
        self.ledger = copy.deepcopy(self.parent)
        self.ledger.update(schema=4, scope_id=self.scope["id"], fixed_total=74,
                           pending=[item["id"] for item in self.scope["tasks"]],
                           research={"status": "active", "zero_decision": None}, scope_issues=[])
        self.ledger["tasks"].update({item["id"]: {
            "definition": item, "status": "pending", "started_at": None, "result": None
        } for item in self.scope["tasks"]})
        self.ledger["history"].append(extension.checkpoint(self.scope, self.parent))

    def reject(self):
        with self.assertRaises(ValueError):
            counter.validate_ledger(self.ledger)

    def test_approved_extension_preserves_zero(self):
        self.assertEqual(counter.validate_ledger(self.ledger), (66, 8))
        self.assertEqual(self.ledger["history"][:-1], self.parent["history"])

    def test_header(self):
        self.assertEqual(counter.read_counts("# DONE: 66 | TODO: 8 | TOTAL: 74"), (66, 8))

    def test_no_unapproved_total(self):
        self.ledger["fixed_total"] = 75
        self.reject()

    def test_no_old_completion_edit(self):
        self.ledger["tasks"]["R04"]["result"]["finding"] = "different"
        self.reject()

    def test_no_old_history_truncation(self):
        self.ledger["history"].pop(0)
        self.reject()

    def test_no_parent_rollback(self):
        self.ledger["completed"].pop()
        self.ledger["pending"].append("R12")
        self.reject()

    def test_no_scope_substitution(self):
        self.ledger["tasks"]["R14"]["definition"] = dict(
            self.ledger["tasks"]["R14"]["definition"], scope="Another experiment")
        self.reject()

    def test_no_extra_task(self):
        self.ledger["pending"].append("R21")
        self.reject()

    def test_no_unrecorded_completion(self):
        self.ledger["completed"].append(self.ledger["pending"].pop(0))
        self.reject()

    def test_no_missing_extension_checkpoint(self):
        self.ledger["history"].pop()
        self.reject()

    def test_publication_preserves_parent(self):
        pubs = {"schema": 3, "scope_id": self.scope["id"], "fixed_total": 74,
                "checkpoints": self.publications["checkpoints"] + [self.ledger["history"][-1]]}
        self.assertEqual(counter.validate_published_progress(self.ledger, pubs), (66, 8))
        pubs["checkpoints"].pop(0)
        with self.assertRaises(ValueError):
            counter.validate_published_progress(self.ledger, pubs)

    def test_no_fabricated_zero(self):
        self.ledger["pending"] = []
        self.reject()

    def close(self, identity):
        item = self.ledger["tasks"][identity]
        at = "2026-09-14T16:00:00+00:00"
        item.update(status="done", started_at=at, result={
            "completed_at": at, "outcome": "inconclusive", "checked": "bounded fixture",
            "evidence": [item["definition"]["result"]], "finding": "fixture result",
            "limits": "test data, not a scientific claim"})
        self.ledger["pending"].remove(identity)
        self.ledger["completed"].append(identity)
        self.ledger["history"].append({"event": "task_completed", "scope_id": self.scope["id"],
            "at": at, "done": len(self.ledger["completed"]), "todo": len(self.ledger["pending"]),
            "total": 74, "completed": list(self.ledger["completed"]),
            "completion": {"id": identity, "record": copy.deepcopy(item)}})

    def test_one_completion_advances_exactly_one(self):
        self.close("R13")
        self.assertEqual(counter.validate_ledger(self.ledger), (67, 7))

    def test_completed_result_cannot_be_rewritten(self):
        self.close("R13")
        self.ledger["tasks"]["R13"]["result"]["finding"] = "replacement"
        self.reject()

    def test_final_review_cannot_close_before_inputs(self):
        self.close("R20")
        self.reject()

    def test_zero_requires_new_scope_decision(self):
        for identity in list(self.ledger["pending"]):
            self.close(identity)
        self.reject()
        self.ledger["research"] = {"status": "complete", "zero_decision": {
            "kind": "supported", "evidence": [self.ledger["tasks"]["R20"]["definition"]["result"]],
            "conclusion": "fixture only"}}
        self.assertEqual(counter.validate_ledger(self.ledger), (74, 0))
        self.ledger["research"]["zero_decision"]["evidence"] = [
            self.ledger["tasks"]["R12"]["definition"]["result"]]
        self.reject()

    def test_zero_requires_exact_extension_permission_request(self):
        for identity in list(self.ledger["pending"]):
            self.close(identity)
        self.ledger["research"] = {"status": "awaiting_permission", "zero_decision": {
            "kind": "permission_required", "cases": [1], "reason": "fixture missing work",
            "evidence": [self.ledger["tasks"]["R20"]["definition"]["result"]],
            "additional_tasks": [{"id": "R21", "question": "fixture question", "scope": "one bounded task"}],
            "requested_todo_increase": 1, "permission_question": "Approve this one task?"}}
        self.assertEqual(counter.validate_ledger(self.ledger), (74, 0))
        self.ledger["research"]["zero_decision"]["requested_todo_increase"] = 2
        self.reject()


if __name__ == "__main__":
    unittest.main()
