import csv
import unittest
from collections import Counter
from pathlib import Path


class ScheduleTest(unittest.TestCase):
    def test_schedule_has_the_frozen_independent_design(self) -> None:
        path = Path(__file__).with_name("SCHEDULE.tsv")
        with path.open(newline="") as stream:
            rows = list(csv.DictReader(stream, delimiter="\t"))

        self.assertEqual(len(rows), 12)
        self.assertEqual(len({row["attempt_id"] for row in rows}), 12)
        self.assertEqual(
            Counter(row["batch"] for row in rows),
            Counter({str(i): 2 for i in range(1, 7)}),
        )
        self.assertEqual(
            Counter(row["condition"] for row in rows),
            Counter(
                {
                    "direct": 2,
                    "wl-000": 2,
                    "wl-001": 1,
                    "wl-010": 1,
                    "wl-011": 1,
                    "wl-100": 1,
                    "wl-101": 1,
                    "wl-110": 1,
                    "wl-111": 2,
                }
            ),
        )


if __name__ == "__main__":
    unittest.main()
