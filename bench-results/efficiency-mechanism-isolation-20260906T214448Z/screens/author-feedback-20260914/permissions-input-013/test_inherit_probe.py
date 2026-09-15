import unittest
import integration_probe as original
try:
    import inherit_probe as p
except ModuleNotFoundError:
    p = original


class InheritedPermissionTests(unittest.TestCase):
    def test_fresh_is_unchanged_and_resume_inherits_only_permission_settings(self):
        self.assertEqual(p.command("/provider", "/repo", None),
                         original.command("/provider", "/repo", None))
        thread = "11111111-1111-7111-8111-111111111111"
        argv = p.command("/provider", "/repo", thread)
        self.assertNotIn("--sandbox", argv)
        self.assertNotIn("--ask-for-approval", argv)
        self.assertEqual(argv, ["/provider", "--cd", "/repo", "--model", "gpt-5.5",
                               "exec", "--color", "never", "resume", "--json", thread, "-"])
        with self.assertRaises(ValueError):
            p.command("/provider", "/repo", "foreign")


if __name__ == "__main__":
    unittest.main()
