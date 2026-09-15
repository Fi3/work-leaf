import subprocess
import unittest

import repair_driver as r
import verify_guard as v


class SourceInverseTests(unittest.TestCase):
    def test_only_observer_setup_differs_and_shell_parses(self):
        original = r.original.driver_source()
        changed = r.driver_source()
        before = v.shell_function(original, "setup_observer")
        after = v.shell_function(changed, "setup_observer")
        self.assertEqual(changed.replace(after, before, 1), original)
        self.assertEqual(subprocess.run(["bash", "-n"], input=changed, text=True,
                                       capture_output=True).returncode, 0)
        self.assertIn('observer_root="$(cd -- "$observer_root" && pwd -P)"', after)
        self.assertIn('== "$observer_identity"', after)


if __name__ == "__main__":
    unittest.main()
