"""The frozen host's actual exec ordering must preserve native resume."""
import unittest
import catalog_dispatch as dispatch


class DispatchTests(unittest.TestCase):
    env = {"WORK_LEAF_BENCH_P01_CATALOG": "1"}

    def test_host_resume_after_exec_color_options_bypasses_view(self):
        self.assertFalse(dispatch.needs_view(
            ["--cd", "/tmp/fixture", "--sandbox", "read-only", "--ask-for-approval", "never",
             "--model", "gpt-5.5", "exec", "--color", "never", "resume", "--json",
             "-o", "/tmp/result", "session-id", "-"], self.env))

    def test_fresh_host_order_gets_view(self):
        self.assertTrue(dispatch.needs_view(
            ["--cd", "/tmp/fixture", "exec", "--color", "never", "--json", "-o", "/tmp/result", "-"], self.env))

    def test_resume_named_option_value_is_not_a_subcommand(self):
        self.assertTrue(dispatch.needs_view(["exec", "--color", "never", "-o", "resume", "-"], self.env))

    def test_equal_sign_options_and_config_values(self):
        self.assertFalse(dispatch.needs_view(["exec", "--color=never", "-c", "model='resume'", "resume", "-"], self.env))

    def test_escaped_literal_resume_is_a_fresh_prompt(self):
        self.assertTrue(dispatch.needs_view(["exec", "--", "resume"], self.env))

    def test_unsupported_or_incomplete_command_fails_closed(self):
        for argv in (["exec", "--color"], ["exec", "--unknown"], ["exec", "fork", "-"],
                     ["--model", "exec", "review"], ["plugin", "list"]):
            with self.subTest(argv=argv), self.assertRaises(ValueError):
                dispatch.needs_view(argv, self.env)

    def test_opt_in_and_recursion_rules_are_preserved(self):
        for env in ({}, {**self.env, "WORK_LEAF_BENCH_CODEX_ACTIVE": "1"}):
            with self.assertRaises(ValueError):
                dispatch.needs_view(["exec", "-"], env)


if __name__ == "__main__":
    unittest.main()
