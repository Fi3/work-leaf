"""Reference-state delivery must be explicit, ordered and benchmark-only."""
import copy
import unittest

import catalog_schedule as schedule


class ScheduleTests(unittest.TestCase):
    def setUp(self):
        self.plan = {"schema": 1, "states": ["full", "full", "short"],
                     "tail": "hold_last", "profiles": {
                         "full": {"view": {"uid": 1000, "mounts": []},
                                  "catalog_sha256": "a" * 64},
                         "short": {"view": {"uid": 1000, "mounts": []},
                                   "catalog_sha256": "b" * 64}}}
        self.env = {"WORK_LEAF_BENCH_P01_CATALOG": "1",
                    "WORK_LEAF_BENCH_INPUT_SCHEDULE": "1"}
        self.fresh = ["--cd", "/tmp/project", "exec", "--color", "never", "--json", "-"]
        self.resume = ["--cd", "/tmp/project", "exec", "--color", "never",
                       "resume", "--json", "-o", "/tmp/out", "thread", "-"]

    def test_reference_state_is_preserved_on_resume(self):
        for ordinal, command, expected in ((1, self.fresh, "full"),
                                            (2, self.resume, "full"),
                                            (3, self.resume, "short")):
            with self.subTest(ordinal=ordinal):
                result = schedule.select(self.plan, ordinal, command, self.env)
                self.assertEqual(result["name"], expected)
                self.assertFalse(result["extrapolated"])

    def test_extra_turn_is_declared_extrapolation_not_observed_reference(self):
        result = schedule.select(self.plan, 4, self.resume, self.env)
        self.assertEqual(result["name"], "short")
        self.assertTrue(result["extrapolated"])

    def test_invalid_or_missing_boundary_fails_closed(self):
        for value in (None, 0, -1, True, 1.5, "2"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                schedule.select(self.plan, value, self.resume, self.env)

    def test_fresh_resume_disagreement_fails_closed(self):
        for ordinal, command in ((1, self.resume), (2, self.fresh)):
            with self.assertRaises(ValueError):
                schedule.select(self.plan, ordinal, command, self.env)

    def test_opt_in_and_recursive_guard(self):
        for env in ({}, {**self.env, "WORK_LEAF_BENCH_CODEX_ACTIVE": "1"},
                    {**self.env, "WORK_LEAF_BENCH_INPUT_SCHEDULE": "0"}):
            with self.assertRaises(ValueError):
                schedule.select(self.plan, 1, self.fresh, env)

    def test_unknown_or_empty_schedule_rejected(self):
        for replacement in ({"states": []}, {"states": ["missing"]},
                            {"tail": "cycle"}, {"schema": 2}):
            with self.assertRaises(ValueError):
                schedule.select({**self.plan, **replacement}, 1, self.fresh, self.env)

    def test_missing_view_or_catalog_digest_rejected(self):
        for key in ("view", "catalog_sha256"):
            plan = copy.deepcopy(self.plan)
            del plan["profiles"]["full"][key]
            with self.assertRaises(ValueError):
                schedule.select(plan, 1, self.fresh, self.env)

    def test_host_cursor_rejects_skips_retries_and_wrong_thread(self):
        cursor = schedule.Cursor()
        cursor.enter(1, None)
        for ordinal, thread in ((1, None), (3, "session"), (2, None)):
            with self.assertRaises(ValueError):
                cursor.enter(ordinal, thread)
        cursor.enter(2, "session")
        with self.assertRaises(ValueError):
            cursor.enter(3, "different")
        cursor.enter(3, "session")

    def test_separate_workflows_have_independent_cursors(self):
        left, right = schedule.Cursor(), schedule.Cursor()
        left.enter(1, None)
        left.enter(2, "left")
        right.enter(1, None)
        right.enter(2, "right")

    def test_option_values_named_resume_do_not_change_command(self):
        result = schedule.select(self.plan, 1, ["exec", "-o", "resume", "-"], self.env)
        self.assertEqual(result["name"], "full")


if __name__ == "__main__":
    unittest.main()
