#!/usr/bin/env python3

import csv
import subprocess
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
SCHEDULE = STUDY / "SCHEDULE.tsv"
LAUNCHER = STUDY / "run-attribution-control"
BATCH = STUDY / "run-attribution-batch"


class LaunchProtocolTest(unittest.TestCase):
    def test_schedule_has_two_mixed_batches_and_six_unique_attempts(self):
        with SCHEDULE.open(newline="", encoding="utf-8") as handle:
            rows = list(csv.DictReader(handle, delimiter="\t"))

        self.assertEqual(len(rows), 6)
        self.assertEqual(len({row["attempt_id"] for row in rows}), 6)
        self.assertEqual(
            {row["condition"] for row in rows},
            {"direct-compact-linearizer", "sequential-work-leaf-combined"},
        )
        self.assertEqual([sum(row["batch"] == str(batch) for row in rows) for batch in (1, 2)], [3, 3])
        for batch in (1, 2):
            conditions = [row["condition"] for row in rows if row["batch"] == str(batch)]
            self.assertEqual(len(set(conditions)), 2)

    def test_launcher_pins_profile_and_preserves_normal_sources(self):
        text = LAUNCHER.read_text(encoding="utf-8")

        self.assertIn("gpt-5.5", text)
        self.assertIn("xhigh", text)
        self.assertIn("generate_diagnostic_driver.py", text)
        self.assertIn("WORK_LEAF_DIAGNOSTIC_SOURCE_REPO", text)
        self.assertIn("WORK_LEAF_BENCH_NO_READ_PERMISSION=1", text)
        self.assertIn("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME=wait-for-usage", text)
        self.assertIn("WORK_LEAF_DIRECT_BENCH_MODEL=gpt-5.5", text)
        self.assertIn("WORK_LEAF_DIRECT_BENCH_REASONING_EFFORT=xhigh", text)
        self.assertNotIn("sed -i", text)
        self.assertNotIn("git checkout", text)

    def test_batch_launcher_admits_exactly_three_parallel_workflows(self):
        text = BATCH.read_text(encoding="utf-8")

        self.assertIn('[[ "${#attempts[@]}" == "3" ]]', text)
        self.assertIn('"$script_dir/run-attribution-control" "$attempt" &', text)
        self.assertNotIn("xargs -P", text)

    def test_provider_free_preflight_passes(self):
        result = subprocess.run(
            [str(LAUNCHER), "--check"],
            cwd=STUDY.parents[1],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("ATTRIBUTION_CONTROL_PREFLIGHT_OK", result.stdout)


if __name__ == "__main__":
    unittest.main()
