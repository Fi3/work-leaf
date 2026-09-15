"""Qualify a same-value configuration-layer permission discriminator."""
import sys
from pathlib import Path
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parent))
import integration_probe as base
import config_probe as target

class ConfigArguments(unittest.TestCase):
    def test_identical_explicit_policy_is_present_before_exec_for_both_turns(self):
        thread = "01a0a2ce-fc9d-7003-98d1-03edaf8fbc97"
        for identity in (None, thread):
            original = base.command("/provider", "/repo", identity)
            actual = target.command("/provider", "/repo", identity)
            self.assertEqual(actual[:5], ["/provider", "-c",
                'sandbox_mode="danger-full-access"', "-c", 'approval_policy="never"'])
            self.assertEqual(actual[5:], original[1:])
            self.assertEqual(actual.count("--sandbox"), 1)
            self.assertEqual(actual.count("--ask-for-approval"), 1)
        with self.assertRaises(ValueError):
            target.command("/provider", "/repo", "foreign")

if __name__ == "__main__":
    unittest.main()
