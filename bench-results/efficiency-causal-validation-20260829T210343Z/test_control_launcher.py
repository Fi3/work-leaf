#!/usr/bin/env python3

import subprocess
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
REFERENCE_LAUNCHER = (
    STUDY.parent / "efficiency-exact-normal-work-leaf-20260829T181318Z" / "run-condition"
)


def benchmark_environment(script: str) -> tuple[dict[str, str], set[str]]:
    assignments: dict[str, str] = {}
    unset: set[str] = set()
    in_environment = False
    for raw_line in script.splitlines():
        line = raw_line.strip().removesuffix("\\").strip()
        if line == "env":
            in_environment = True
            continue
        if not in_environment:
            continue
        if line.startswith('"$source_repo/bench-three-features"'):
            break
        if line.startswith("-u WORK_LEAF_"):
            unset.add(line.removeprefix("-u "))
            continue
        if line.startswith("WORK_LEAF_"):
            name, value = line.split("=", 1)
            assignments[name] = value
    return assignments, unset


class ControlLauncherTest(unittest.TestCase):
    def test_direct_read_control_changes_only_the_declared_read_mode(self):
        launcher = (STUDY / "run-direct-read-control").read_text(encoding="utf-8")
        reference = REFERENCE_LAUNCHER.read_text(encoding="utf-8")
        schedule = (STUDY / "CONTROL-SCHEDULE.tsv").read_text(encoding="utf-8").splitlines()

        self.assertEqual(
            schedule,
            [
                "batch\tattempt_id\tcondition",
                "1\tdirect-read-001\twork-leaf-direct-read",
                "1\tdirect-read-002\twork-leaf-direct-read",
                "1\tdirect-read-003\twork-leaf-direct-read",
            ],
        )
        self.assertIn('expected_commit="5b1d1ef9590850faed26052f909ddff7ff8f127d"', launcher)
        self.assertIn("WORK_LEAF_BENCH_MODEL=gpt-5.5", launcher)
        self.assertIn("WORK_LEAF_BENCH_REASONING_EFFORT=xhigh", launcher)
        self.assertIn("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS=1000", launcher)
        self.assertIn("WORK_LEAF_BENCH_NO_READ_PERMISSION=1", launcher)
        self.assertIn("WORK_LEAF_BENCH_TIMEOUT_SECS=7200", launcher)
        self.assertIn("WORK_LEAF_BENCH_WEB_UI=0", launcher)
        self.assertIn("-u WORK_LEAF_BENCH_FEATURE_SCHEDULE", launcher)
        self.assertIn("-u WORK_LEAF_EXPERIMENT_CHANGED_REPEAT_DELIVERY", launcher)
        self.assertIn("-u WORK_LEAF_EXPERIMENT_UNCHANGED_REPEAT_DELIVERY", launcher)
        self.assertIn("-u WORK_LEAF_EXPERIMENT_REVIEW_PROVENANCE", launcher)
        self.assertNotIn("/fork", launcher)
        self.assertNotIn("validation-budget", launcher)

        control_environment, control_unset = benchmark_environment(launcher)
        reference_environment, reference_unset = benchmark_environment(reference)
        self.assertEqual(control_unset, reference_unset)
        self.assertEqual(control_environment.keys(), reference_environment.keys())

        intentional_differences = {
            "WORK_LEAF_BENCH_TMPDIR",
            "WORK_LEAF_BENCH_NO_READ_PERMISSION",
            "WORK_LEAF_BENCH_STUDY_ID",
            "WORK_LEAF_BENCH_OPERATOR_NOTES",
        }
        for name in control_environment.keys() - intentional_differences:
            self.assertEqual(
                control_environment[name],
                reference_environment[name],
                f"unexpected environment difference for {name}",
            )
        self.assertEqual(reference_environment["WORK_LEAF_BENCH_NO_READ_PERMISSION"], "0")
        self.assertEqual(control_environment["WORK_LEAF_BENCH_NO_READ_PERMISSION"], "1")
        self.assertIn(
            'source_repo="/home/user/.codex/work-leaf-exact-normal-source-20260829T181318Z"',
            launcher,
        )
        self.assertIn(
            'reference_study="$repo_root/bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z"',
            launcher,
        )

    def test_preflight_uses_no_provider(self):
        completed = subprocess.run(
            [str(STUDY / "run-direct-read-control"), "--check"],
            cwd=STUDY.parents[1],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("CONTROL_PREFLIGHT_OK", completed.stdout)


if __name__ == "__main__":
    unittest.main()
