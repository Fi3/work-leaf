import json
from pathlib import Path
import tempfile
import unittest
import resource_sample as sample


class ResourceTests(unittest.TestCase):
    def test_response_identity_not_cumulative_or_cached_addition(self):
        usage = {"input_tokens": 100, "output_tokens": 7, "cached_input_tokens": 80}
        e = {"type": "token_usage_record", "payload": {
            "thread_id": "t", "response_id": "r", "usage": usage,
            "thread_token_usage": {"input_tokens": 999}}}
        self.assertEqual(sample.native_totals([e, e], {"t"}), (107, 1))
        self.assertEqual(sample.native_totals([e], {"other"}), (0, 0))

    def test_conflicting_duplicate_or_invalid_usage_fails(self):
        e = {"type": "token_usage_record", "payload": {
            "thread_id": "t", "response_id": "r",
            "usage": {"input_tokens": 1, "output_tokens": 1}}}
        changed = json.loads(json.dumps(e))
        changed["payload"]["usage"]["input_tokens"] = 2
        with self.assertRaises(ValueError):
            sample.native_totals([e, changed], {"t"})
        e["payload"]["usage"]["output_tokens"] = -1
        with self.assertRaises(ValueError):
            sample.native_totals([e], {"t"})

    def test_only_unfinished_last_line_is_ignored(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "events.jsonl"
            p.write_bytes(b'{"type":"turn.started"}\n{"par')
            self.assertEqual(len(sample.events(p)), 1)
            p.write_bytes(b'{"bad"}\n')
            with self.assertRaises(json.JSONDecodeError):
                sample.events(p)


if __name__ == "__main__":
    unittest.main()
