#!/usr/bin/env python3

import csv
import unittest
from collections import Counter, defaultdict
from pathlib import Path


STUDY_DIR = Path(__file__).resolve().parent


class ScheduleTests(unittest.TestCase):
    def test_schedule_has_three_independent_runs_per_condition(self):
        with (STUDY_DIR / "SCHEDULE.tsv").open(newline="", encoding="ascii") as stream:
            rows = list(csv.DictReader(stream, delimiter="\t"))

        self.assertEqual(9, len(rows))
        self.assertEqual(
            Counter({"direct": 3, "wl-000": 3, "wl-111": 3}),
            Counter(row["condition"] for row in rows),
        )
        self.assertEqual(len(rows), len({row["attempt_id"] for row in rows}))
        self.assertTrue(all(row["analytical_pair"] == "false" for row in rows))

        batches = defaultdict(list)
        for row in rows:
            batches[int(row["batch"])].append(row)
        self.assertEqual([1, 2, 3, 4, 5], sorted(batches))
        self.assertTrue(all(1 <= len(batch) <= 2 for batch in batches.values()))

    def test_attempt_ids_match_conditions(self):
        with (STUDY_DIR / "SCHEDULE.tsv").open(newline="", encoding="ascii") as stream:
            rows = list(csv.DictReader(stream, delimiter="\t"))

        for row in rows:
            self.assertRegex(row["attempt_id"], rf"^{row['condition']}-[0-9]{{3}}$")


if __name__ == "__main__":
    unittest.main()
