from pathlib import Path
import tempfile
import unittest

import integration_probe as p


class IntegrationProbeTests(unittest.TestCase):
    def test_exact_unsandboxed_launch_and_same_thread_resume_order(self):
        base = ["/provider", "--cd", "/repo", "--sandbox", "danger-full-access",
                "--ask-for-approval", "never", "--model", "gpt-5.5", "exec", "--color", "never"]
        thread = "11111111-1111-7111-8111-111111111111"
        self.assertEqual(p.command("/provider", "/repo", None), base + ["--json", "-"])
        self.assertEqual(p.command("/provider", "/repo", thread),
                         base + ["resume", "--json", thread, "-"])
        with self.assertRaises(ValueError):
            p.command("/provider", "/repo", "foreign")

    def test_terminal_requires_owned_completed_marker_and_retains_errors(self):
        rows = [{"type":"thread.started", "thread_id":"owner"},
                {"type":"item.completed", "item":{"type":"agent_message", "text":"MARK"}},
                {"type":"turn.completed", "usage":{"input_tokens":10,"output_tokens":2}}]
        self.assertEqual(p.terminal(rows, "owner", "MARK"), ("owner",12))
        for changed in (rows[:-1], rows + [{"type":"turn.failed"}],
                        rows + [{"type":"error","message":"unclassified"}]):
            with self.assertRaises(ValueError):
                p.terminal(changed, "owner", "MARK")
        with self.assertRaises(ValueError):
            p.terminal(rows, "foreign", "MARK")
        with self.assertRaises(ValueError):
            p.terminal(rows, "owner", "wrong")


if __name__ == "__main__":
    unittest.main()
