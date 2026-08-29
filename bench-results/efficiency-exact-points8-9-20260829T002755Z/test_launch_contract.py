#!/usr/bin/env python3

import subprocess
import unittest
from pathlib import Path


STUDY_DIR = Path(__file__).resolve().parent


class LaunchContractTests(unittest.TestCase):
    def test_launch_scripts_have_valid_shell_syntax(self):
        for name in ("run-condition", "run-batch"):
            completed = subprocess.run(
                ["bash", "-n", str(STUDY_DIR / name)],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(0, completed.returncode, completed.stderr)

    def test_condition_launcher_rejects_unknown_condition_without_creating_state(self):
        completed = subprocess.run(
            [str(STUDY_DIR / "run-condition"), "unknown-001", "unknown", "1"],
            cwd=STUDY_DIR,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(2, completed.returncode)
        self.assertIn("invalid attempt ID", completed.stderr)
        self.assertFalse((STUDY_DIR / "runs" / "unknown-001").exists())

    def test_batch_launcher_rejects_unknown_batch_without_creating_state(self):
        completed = subprocess.run(
            [str(STUDY_DIR / "run-batch"), "0"],
            cwd=STUDY_DIR,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(2, completed.returncode)
        self.assertIn("usage:", completed.stderr)
        self.assertFalse((STUDY_DIR / "batch-status" / "batch-0.complete").exists())


if __name__ == "__main__":
    unittest.main()
