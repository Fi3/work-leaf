import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("guard_continuation", Path(__file__).with_name("batch_adapter.py"))
adapter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(adapter)


class ContinuationTests(unittest.TestCase):
    def test_retained_engine_gets_new_nonoverlapping_exact_three(self):
        runner = adapter.qualified_runner()
        rows = runner.schedule(Path("/tmp/phase"), None, Path("/tmp/runtime"))
        self.assertEqual([r["run_id"] for r in rows],
            ["author-joint-confirmation-004", "author-joint-confirmation-005", "author-joint-confirmation-006"])
        self.assertEqual(runner.PHASE, "author-joint-confirmation-qualified-20260915")
        runner.validate_schedule(rows)
        with self.assertRaises(ValueError):
            runner.validate_schedule(rows[:2])
        with self.assertRaises(ValueError):
            runner.run_batch(Path("/tmp/unadmitted"))


if __name__ == "__main__":
    unittest.main()
