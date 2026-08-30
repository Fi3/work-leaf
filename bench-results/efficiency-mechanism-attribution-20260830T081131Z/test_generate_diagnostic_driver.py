#!/usr/bin/env python3

import hashlib
import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path


STUDY = Path(__file__).resolve().parent
ROOT = STUDY.parents[1]
GENERATOR = STUDY / "generate_diagnostic_driver.py"
WORK_LEAF_SOURCE = ROOT / "bench-three-features"
DIRECT_SOURCE = ROOT / "bench-three-features-direct-common"


def load_module():
    specification = importlib.util.spec_from_file_location("diagnostic_driver", GENERATOR)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load {GENERATOR}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class DiagnosticDriverTest(unittest.TestCase):
    def test_sequential_work_leaf_changes_only_diagnostic_driver_behavior(self):
        module = load_module()
        source_before = WORK_LEAF_SOURCE.read_bytes()
        generated = module.generate("sequential-work-leaf", WORK_LEAF_SOURCE)

        self.assertEqual(WORK_LEAF_SOURCE.read_bytes(), source_before)
        self.assertIn('readonly feature_schedule="sequential-diagnostic"', generated)
        self.assertIn("launched_features=1", generated)
        self.assertIn("launch_next_sequential_feature", generated)
        self.assertIn(
            "feature_stage_deadline=$((now_active + timeout_secs))", generated
        )
        self.assertNotIn('readonly feature_schedule="concurrent"', generated)
        self.assertIn("WORK_LEAF_DIAGNOSTIC_SOURCE_REPO", generated)
        self.assertEqual(sha256(WORK_LEAF_SOURCE), module.WORK_LEAF_SOURCE_SHA256)
        self.assert_valid_shell(generated)

    def test_compact_direct_changes_only_linearizer_handoff(self):
        module = load_module()
        source_before = DIRECT_SOURCE.read_bytes()
        generated = module.generate("compact-direct-linearizer", DIRECT_SOURCE)

        self.assertEqual(DIRECT_SOURCE.read_bytes(), source_before)
        self.assertIn("Exact reviewed provisional targets", generated)
        self.assertIn("compact_linearize_target_rows", generated)
        self.assertIn("WORK_LEAF_DIAGNOSTIC_SOURCE_REPO", generated)
        self.assertIn("run_feature_cycle", generated)
        self.assertEqual(sha256(DIRECT_SOURCE), module.DIRECT_SOURCE_SHA256)
        self.assert_valid_shell(generated)

    def test_rejects_changed_or_unknown_sources(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as directory:
            changed = Path(directory) / "changed"
            changed.write_text("#!/usr/bin/env bash\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "source SHA-256"):
                module.generate("sequential-work-leaf", changed)
            with self.assertRaisesRegex(ValueError, "unknown diagnostic mode"):
                module.generate("unknown", WORK_LEAF_SOURCE)

    def assert_valid_shell(self, text: str):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "driver"
            path.write_text(text, encoding="utf-8")
            result = subprocess.run(
                ["bash", "-n", str(path)],
                text=True,
                capture_output=True,
                check=False,
            )
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
