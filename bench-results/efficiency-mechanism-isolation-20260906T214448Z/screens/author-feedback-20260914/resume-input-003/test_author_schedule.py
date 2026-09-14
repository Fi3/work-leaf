"""The scheduler changes only native invocation environment, never host policy."""
import hashlib
import os
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import run_author


class AuthorScheduleTests(unittest.TestCase):
    def test_only_exact_environment_span_changes_for_each_arm(self):
        for arm in ("R03", "R04"):
            original = run_author.author_inverse.host_source(arm)
            changed = run_author.scheduled_source(arm)
            self.assertEqual(changed.replace(run_author.REPLACEMENT, run_author.SPAN), original)
            self.assertEqual(changed.count(run_author.REPLACEMENT), 1)

    def test_rejected_or_missing_source_span_does_not_fall_back(self):
        with patch.object(run_author.author_inverse, "host_source", return_value="unrelated"):
            with self.assertRaises(ValueError):
                run_author.scheduled_source("R03")

    def test_declared_wall_bound_reaches_the_unchanged_host(self):
        fake = SimpleNamespace(run_stage=lambda args: args)
        with patch.object(run_author, "load_host", return_value=fake), \
                patch.dict(os.environ, {"WORK_LEAF_BENCH_INVERSE": "1"}, clear=True):
            args = run_author.main(["--arm", "R03", "--repo", "/tmp/project", "--artifact-dir",
                                    "/tmp/artifacts", "--prompt-file", "/tmp/prompt",
                                    "--codex-bin", "/provider"])
        self.assertGreater(args.deadline - run_author.time.monotonic(), 1199)
        self.assertLessEqual(args.deadline - run_author.time.monotonic(), 1200)
        self.assertTrue(args.serialized_feedback)
        self.assertIsNone(args.resume_thread)

    def test_delivery_plan_env_is_separate_from_the_prompt_and_argv(self):
        module = run_author.load_host("R03")
        folder = Path("/tmp/nonexistent-generic-scheduler-fixture")
        host = SimpleNamespace(invocations=[], artifacts=folder, repo=folder,
                               thread=None, unchanged=lambda: None,
                               evidence=lambda *_a, **_k: None,
                               event=lambda *_a, **_k: None)
        args = SimpleNamespace(codex_bin="/provider", model="arbitrary-model",
                               deadline=999999999999, stage="generic-stage")
        results = []
        def child(argv, cwd, stdin, stdout, stderr, seconds, env):
            results.append((list(argv), dict(env)))
            return {"exit_code": 0, "timed_out": False, "cancelled_signal": None}
        with patch.object(Path, "mkdir"), patch.object(Path, "write_text") as write, \
                patch.object(module, "save_json"), patch.object(module, "execute_child", side_effect=child), \
                patch.object(module, "owned_final", return_value=("thread-one", "reply", [])), \
                patch.dict(os.environ, {"WORK_LEAF_OBSERVER_ROLE": "actor"}, clear=True):
            module.invoke(host, args, "unchanged first prompt")
            module.invoke(host, args, "unchanged feedback")
            self.assertEqual([call.args[0] for call in write.call_args_list],
                             ["unchanged first prompt", "unchanged feedback"])
        self.assertNotIn("resume", results[0][0])
        self.assertIn("resume", results[1][0])
        self.assertIn("thread-one", results[1][0])
        self.assertEqual([env["WORK_LEAF_BENCH_INPUT_BOUNDARY"] for _, env in results], ["1", "2"])
        for _, env in results:
            self.assertEqual(set(env), {"WORK_LEAF_OBSERVER_ROLE", "WORK_LEAF_BENCH_INPUT_BOUNDARY"})


if __name__ == "__main__":
    unittest.main()
