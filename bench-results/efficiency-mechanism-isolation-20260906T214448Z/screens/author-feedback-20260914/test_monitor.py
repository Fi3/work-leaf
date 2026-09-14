"""R13 fail-first operator-monitor checks; no model provider is started."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock

import monitor


class MonitorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="wl-monitor-qualification-")
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_missing_resources_created_and_first_receipt_precedes_launch(self):
        monitor.prepare(self.root)
        self.assertTrue((self.root / "resources/start.json").is_file())
        self.assertEqual(monitor.step(self.root, 1, lambda: {"raw": 4}, Mock()), {"raw": 4})
        self.assertEqual(json.loads((self.root / "resources/sample-0001.json").read_text()), {"raw": 4})

    def test_first_publication_failure_stops_and_uses_independent_error_path(self):
        monitor.prepare(self.root)
        stop = Mock(return_value={"stopped": "owned"})
        writer = Mock(side_effect=OSError("simulated first-sample failure"))
        with self.assertRaises(monitor.MonitorFailure):
            monitor.step(self.root, 1, lambda: {"raw": 4}, stop, writer)
        stop.assert_called_once_with()
        record = json.loads((self.root / "monitor-failure-0001.json").read_text())
        self.assertEqual(record["stop"], {"stopped": "owned"})
        self.assertIn("first-sample failure", record["error"])

    def test_missing_directory_during_run_still_stops_and_retains_reason(self):
        stop = Mock(return_value={"stopped": True})
        with self.assertRaises(monitor.MonitorFailure):
            monitor.step(self.root, 1, lambda: {}, stop)
        stop.assert_called_once_with()
        self.assertTrue((self.root / "monitor-failure-0001.json").is_file())

    def test_sampler_failure_and_stop_failure_are_both_retained(self):
        monitor.prepare(self.root)
        with self.assertRaises(monitor.MonitorFailure):
            monitor.step(self.root, 1, Mock(side_effect=ValueError("bad metadata")),
                         Mock(side_effect=ValueError("no exact owner")))
        record = json.loads((self.root / "monitor-failure-0001.json").read_text())
        self.assertIn("bad metadata", record["error"])
        self.assertIn("no exact owner", record["stop_error"])

    def test_reusing_preflight_or_sample_is_rejected(self):
        monitor.prepare(self.root)
        with self.assertRaises(FileExistsError):
            monitor.prepare(self.root)
        monitor.step(self.root, 1, lambda: {"original": True}, Mock())
        stop = Mock(return_value={})
        with self.assertRaises(monitor.MonitorFailure):
            monitor.step(self.root, 1, lambda: {"replacement": True}, stop)
        self.assertEqual(json.loads((self.root / "resources/sample-0001.json").read_text()), {"original": True})
        stop.assert_called_once_with()

    def test_exact_cancellation_leaves_other_supervisor_running(self):
        script = Path(__file__).with_name("idle_supervisor.py").resolve()
        processes = []
        try:
            for arm in ("R14", "R15"):
                child = subprocess.Popen([sys.executable, str(script), "--arm", arm,
                    "--artifact-dir", str(self.root / arm)], stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE, text=True)
                processes.append(child)
                self.assertEqual(child.stdout.readline().strip(), "ready")
            receipt = monitor.resources.stop_owned(script, "R14", self.root / "R14")
            self.assertEqual(receipt["signalled_pid"], processes[0].pid)
            self.assertEqual(processes[0].wait(timeout=3), 130)
            self.assertIsNone(processes[1].poll())
            with self.assertRaises(ValueError):
                monitor.resources.stop_owned(script, "R14", self.root / "R14")
        finally:
            for child in processes:
                if child.poll() is None:
                    child.terminate()
                child.communicate(timeout=3)


if __name__ == "__main__":
    unittest.main()
