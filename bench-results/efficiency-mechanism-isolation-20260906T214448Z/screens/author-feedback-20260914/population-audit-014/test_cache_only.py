from pathlib import Path
import tempfile
import unittest

import population_audit as p


class CacheOnlyTests(unittest.TestCase):
    def test_unseen_thread_cannot_execute_core(self):
        with tempfile.TemporaryDirectory(prefix="wl-cache-only.") as root:
            calls = []
            with self.assertRaisesRegex(ValueError, "already audited"):
                p.CacheOnly(root, "hash").run(lambda *args:calls.append(args), [],
                    {"thread_id":"11111111-1111-7111-8111-111111111111"}, "m", "e")
            self.assertEqual(calls, [])
            self.assertEqual(list(Path(root).iterdir()), [])

    def test_cached_thread_is_reused_and_stale_input_rejected(self):
        with tempfile.TemporaryDirectory(prefix="wl-cache-only.") as root:
            metadata = {"thread_id":"11111111-1111-7111-8111-111111111111"}
            calls = []
            def once(*args):
                calls.append(args)
                return {"raw":42}
            original = p.original.NativeCache(root, "hash")
            self.assertEqual(original.run(once, [], metadata, "m", "e"), {"raw":42})
            supplement = p.CacheOnly(root, "hash")
            self.assertEqual(supplement.run(once, [], metadata, "m", "e"), {"raw":42})
            with self.assertRaisesRegex(ValueError, "identity differs"):
                supplement.run(once, ["foreign"], metadata, "m", "e")
            self.assertEqual(len(calls), 1)


if __name__ == "__main__":
    unittest.main()
