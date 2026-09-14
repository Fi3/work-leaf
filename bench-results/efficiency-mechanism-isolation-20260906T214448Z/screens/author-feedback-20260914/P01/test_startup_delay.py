"""Local invariants for the opt-in, text-free startup-delay qualification."""
import subprocess
import unittest

import startup_delay


class StartupDelayTests(unittest.TestCase):
    def test_opt_in_required(self):
        with self.assertRaises(ValueError):
            startup_delay.build_argv(["exec", "--json", "-"], {})

    def test_recursive_launch_rejected(self):
        with self.assertRaises(ValueError):
            startup_delay.build_argv(["exec", "-"], {
                "WORK_LEAF_BENCH_P01_DELAY": "1", "WORK_LEAF_BENCH_CODEX_ACTIVE": "1"})

    def test_only_startup_no_prompt_output_or_tool_change(self):
        argv = ["--cd", "/arbitrary/repository", "--sandbox", "read-only",
                "--ask-for-approval", "never", "--model", "gpt-5.5",
                "exec", "--color", "never", "--json", "-"]
        result = startup_delay.build_argv(argv, {"WORK_LEAF_BENCH_P01_DELAY": "1"})
        self.assertEqual(result[3:], argv)
        self.assertEqual(result[:3], ["--dangerously-bypass-hook-trust", "-c",
            'hooks.SessionStart=[{matcher="^startup$",hooks=[{type="command",command="/usr/bin/sleep 2",timeout=5}]}]'])

    def test_resume_retains_reference_behavior(self):
        argv = ["exec", "resume", "--json", "arbitrary-thread", "-"]
        self.assertEqual(startup_delay.build_argv(argv, {"WORK_LEAF_BENCH_P01_DELAY": "1"}), argv)

    def test_non_exec_commands_rejected(self):
        with self.assertRaises(ValueError):
            startup_delay.build_argv(["plugin", "list"], {"WORK_LEAF_BENCH_P01_DELAY": "1"})

    def test_exact_delay_command_has_no_output(self):
        result = subprocess.run(["/usr/bin/sleep", "2"], capture_output=True, timeout=5)
        self.assertEqual((result.returncode, result.stdout, result.stderr), (0, b"", b""))


if __name__ == "__main__":
    unittest.main()
