import json
from pathlib import Path
import tempfile
import unittest

import full_audit as a


class FullAuditTests(unittest.TestCase):
    def test_cached_native_core_is_executed_once(self):
        with tempfile.TemporaryDirectory(prefix="wl-audit-cache.") as folder:
            calls = []
            cache = a.NativeCache(Path(folder), "a"*64)
            def core(rows, metadata, model, effort):
                calls.append(1)
                return {"records": {}, "errors": []}
            metadata = {"thread_id": "11111111-1111-7111-8111-111111111111"}
            expected = cache.run(core, [], metadata, "model", "effort")
            self.assertEqual(cache.run(core, [], metadata, "model", "effort"), expected)
            self.assertEqual(len(calls), 1)
            with self.assertRaises(ValueError):
                cache.run(core, [{"different": 1}], metadata, "model", "effort")

    def test_failed_core_claim_cannot_trigger_reexecution(self):
        with tempfile.TemporaryDirectory(prefix="wl-audit-cache.") as folder:
            cache = a.NativeCache(Path(folder), "a"*64)
            metadata = {"thread_id": "11111111-1111-7111-8111-111111111111"}
            calls = []
            def fail(*args):
                calls.append(1)
                raise RuntimeError("fixture failure")
            with self.assertRaises(RuntimeError):
                cache.run(fail, [], metadata, "model", "effort")
            with self.assertRaises(ValueError):
                cache.run(fail, [], metadata, "model", "effort")
            self.assertEqual(calls, [1])

    def test_missing_or_mixed_terminal_population_is_rejected(self):
        with tempfile.TemporaryDirectory(prefix="wl-audit-closed.") as folder:
            root = Path(folder)
            manifest = {"schedule": [{"run_id": "one"}, {"run_id": "two"}]}
            with self.assertRaises(ValueError):
                a.require_terminal(root, manifest)
            (root/"PHASE-RESULT.json").write_text(json.dumps({
                "collection_paused": True,
                "runs": [{"id": "one", "launcher_exit_code": 0, "finished_at": "dated"}]}))
            with self.assertRaises(ValueError):
                a.require_terminal(root, manifest)

    def test_only_ordered_owned_diagnostics_are_recoverable(self):
        start = {"type": "thread.started", "thread_id": "owned"}
        end = {"type": "turn.completed", "usage": {}}
        error = {"type": "error"}
        self.assertEqual(a.public_lifecycle([start, error, end]), [])
        self.assertTrue(a.public_lifecycle([error, start, end]))
        self.assertTrue(a.public_lifecycle([start, end, error]))
        self.assertTrue(a.public_lifecycle([start, {"type": "turn.failed"}, end]))
        self.assertTrue(a.public_lifecycle([start]))
        self.assertTrue(a.public_lifecycle([start, start, end]))


if __name__ == "__main__":
    unittest.main()
