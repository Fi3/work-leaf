import json
from pathlib import Path
import tempfile
import unittest

import output_monitor

THREAD = "12345678-1234-1234-1234-123456789abc"

class OutputMonitorTests(unittest.TestCase):
    def test_arbitrary_public_layout_and_rollout_day_are_counted(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            first, second = root / "old-day", root / "new-day"
            first.mkdir()
            second.mkdir()
            public = root / "any-name.log"
            public.write_text(json.dumps({"type": "thread.started", "thread_id": THREAD}) + "\n"
                              + json.dumps({"type": "turn.completed", "usage": {
                                  "input_tokens": 11, "output_tokens": 3}}) + "\n")
            native = second / ("rollout-" + THREAD + ".jsonl")
            native.write_text(json.dumps({"type": "token_usage_record", "payload": {
                "thread_id": THREAD, "response_id": "response", "usage": {
                    "input_tokens": 11, "output_tokens": 3}}}) + "\n")
            result = output_monitor.sample(root, [public], [first, second])
            self.assertEqual(result["threads"], [THREAD])
            self.assertEqual(result["public_completed_raw"], 14)
            self.assertEqual(result["native_recorded_raw"], 14)
            self.assertEqual(result["distinct_responses"], 1)

    def test_repeated_public_path_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "public"
            path.write_text("")
            with self.assertRaises(ValueError):
                output_monitor.sample(Path(temp), [path, path], [])

    def test_missing_native_is_visible_not_zero_completeness(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "public"
            path.write_text(json.dumps({"type": "thread.started", "thread_id": THREAD}) + "\n")
            result = output_monitor.sample(Path(temp), [path], [Path(temp)])
            self.assertEqual(result["native_pending_threads"], [THREAD])

    def test_ambiguous_rollout_identity_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            path = root / "public"
            path.write_text(json.dumps({"type": "thread.started", "thread_id": THREAD}) + "\n")
            (root / ("a-" + THREAD + ".jsonl")).write_text("")
            (root / ("b-" + THREAD + ".jsonl")).write_text("")
            with self.assertRaises(ValueError):
                output_monitor.sample(root, [path], [root])

    def test_no_provider_started_yet_is_an_empty_snapshot(self):
        result = output_monitor.sample(Path("/tmp/unused"), [], [])
        self.assertEqual(result["threads"], [])
        self.assertFalse(result["stop_threshold_reached"])


if __name__ == "__main__":
    unittest.main()
