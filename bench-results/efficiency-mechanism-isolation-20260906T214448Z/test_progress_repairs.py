"""Additive budget/bug-counter checks; no provider calls and no historical-test edits."""
import copy
import json
import unittest

import progress_counter as core
import progress_end_to_end as current
import progress_repairs as repairs
import test_progress_end_to_end as original


class RepairProgressTests(unittest.TestCase):
    def fixture(self):
        fixture = original.CompletePlanTests()
        fixture.setUp()
        return fixture

    def test_approved_extra_budget_keeps_fixed_research_count(self):
        fixture = self.fixture()
        fixture.ledger["budget_exceptions"] = ["approved-p01-extra-20260914"]
        fixture.ledger["tasks"]["P01"].update(status="checking",
            started_at="2026-09-14T19:20:55+00:00", preparation_seconds=5399)
        self.assertEqual(current.validate_ledger(fixture.ledger), (74, 16))

    def test_no_exception_does_not_inherit_more_budget(self):
        fixture = self.fixture()
        fixture.ledger["tasks"]["P01"].update(status="checking",
            started_at="2026-09-14T19:20:55+00:00", preparation_seconds=3601)
        with self.assertRaises(ValueError):
            current.validate_ledger(fixture.ledger)

    def test_exception_cannot_repeat_or_expand(self):
        for ids in (["approved-p01-extra-20260914"] * 2, ["unapproved"]):
            fixture = self.fixture()
            fixture.ledger["budget_exceptions"] = ids
            with self.assertRaises(ValueError):
                current.validate_ledger(fixture.ledger)

    def test_exception_exhaustion_still_stops(self):
        fixture = self.fixture()
        fixture.ledger["budget_exceptions"] = ["approved-p01-extra-20260914"]
        fixture.ledger["tasks"]["P01"].update(status="checking",
            started_at="2026-09-14T19:20:55+00:00", preparation_seconds=5401)
        with self.assertRaises(ValueError):
            current.validate_ledger(fixture.ledger)

    def bug_ledger(self):
        return {"schema": 1, "authority": repairs.AUTHORITY, "events": []}

    def discovery(self):
        return {"event": "discovered", "id": "BUG001",
            "at": "2026-09-14T19:25:00+00:00", "title": "Fixture unexpected defect",
            "related_task": "P02", "already_accounted": False,
            "evidence": [repairs.AUTHORITY]}

    def test_empty_bug_counter_is_separate(self):
        self.assertEqual(repairs.validate_bugs(self.bug_ledger()), (0, 0, 0))

    def test_new_bug_then_verified_fix_is_monotone(self):
        ledger = self.bug_ledger()
        ledger["events"].append(self.discovery())
        self.assertEqual(repairs.validate_bugs(ledger), (0, 1, 1))
        ledger["events"].append({"event": "fixed", "id": "BUG001",
            "at": "2026-09-14T19:26:00+00:00", "reproduction": [repairs.AUTHORITY],
            "verification": [repairs.AUTHORITY], "result": repairs.AUTHORITY})
        self.assertEqual(repairs.validate_bugs(ledger), (1, 0, 1))
        ledger["events"].append(copy.deepcopy(ledger["events"][-1]))
        with self.assertRaises(ValueError):
            repairs.validate_bugs(ledger)

    def test_already_accounted_or_duplicate_bug_is_rejected(self):
        for duplicate in (False, True):
            ledger = self.bug_ledger()
            event = self.discovery()
            if duplicate:
                ledger["events"] = [event, copy.deepcopy(event)]
            else:
                event["already_accounted"] = True
                ledger["events"] = [event]
            with self.assertRaises(ValueError):
                repairs.validate_bugs(ledger)

    def test_unverified_fix_is_rejected(self):
        ledger = self.bug_ledger()
        ledger["events"] = [self.discovery(), {"event": "fixed", "id": "BUG001",
            "at": "2026-09-14T19:26:00+00:00", "reproduction": [],
            "verification": [], "result": repairs.AUTHORITY}]
        with self.assertRaises(ValueError):
            repairs.validate_bugs(ledger)

    def test_actual_visible_bug_counter(self):
        bugs = json.loads((core.STUDY / "BUG-FIX-CHECKLIST.json").read_text())
        counts = repairs.validate_bugs(bugs)
        line = f"## BUG FIXES — DONE: {counts[0]} | TODO: {counts[1]} | TOTAL: {counts[2]}"
        self.assertEqual((core.ROOT / "hypotesis.md").read_text().splitlines()[1], line)


if __name__ == "__main__":
    unittest.main()
