import copy
import os
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

import full_workflow as f


class FullWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.env = {"WORK_LEAF_BENCH_FULL_INVERSE": "1", "WORK_LEAF_BENCH_P01_CATALOG": "1"}
        self.plan = {"schema": 1, "tail": "hold_last", "sources": {},
                     "profiles": {name: {"view": {"mounts": []}, "catalog_sha256": "a" * 64}
                                  for name in ("full", "short", "off")},
                     "stages": {"author-first": {"first": "fresh", "kind": "host", "states": ["full", "off"]},
                                "author-fix": {"first": "resume", "kind": "host", "states": ["short"]},
                                "review-first": {"first": "fresh", "kind": "direct", "states": ["short"]}},
                     "fallbacks": [{"pattern": "review-again-[1-9][0-9]*", "first": "resume", "kind": "direct", "states": ["short"]}]}

    def choose(self, stage, ordinal, resume, role=None):
        argv = ["--cd", "/tmp/project", "exec", "--color", "never"]
        argv += ["resume", "--json", "thread", "-"] if resume else ["--json", "-"]
        env = dict(self.env)
        entry = self.plan["stages"].get(stage, {"kind": "direct"})
        env["WORK_LEAF_OBSERVER_ROLE"] = role or (f"{stage}-host-{ordinal:04d}" if entry["kind"] == "host" else stage)
        return f.select(self.plan, stage, ordinal, argv, env)

    def test_fresh_resume_and_held_tail(self):
        self.assertEqual(self.choose("author-first", 1, False)["name"], "full")
        self.assertEqual(self.choose("author-first", 2, True)["name"], "off")
        self.assertTrue(self.choose("author-first", 3, True)["extrapolated"])

    def test_review_fix_starts_on_existing_thread(self):
        self.assertEqual(self.choose("author-fix", 1, True)["name"], "short")
        with self.assertRaises(ValueError):
            self.choose("author-fix", 1, False)

    def test_wrong_fresh_or_observer_owner_rejected(self):
        with self.assertRaises(ValueError):
            self.choose("author-first", 1, True)
        with self.assertRaises(ValueError):
            self.choose("author-first", 1, False, "foreign-host-0001")

    def test_direct_stage_not_host_loop(self):
        self.assertEqual(self.choose("review-first", 1, False)["name"], "short")
        with self.assertRaises(ValueError):
            self.choose("review-first", 2, True)

    def test_explicit_fallback_unknown_and_ambiguity(self):
        self.assertTrue(self.choose("review-again-7", 1, True)["fallback"])
        with self.assertRaises(ValueError):
            self.choose("unlisted", 1, True)
        self.plan["fallbacks"].append(copy.deepcopy(self.plan["fallbacks"][0]))
        with self.assertRaises(ValueError):
            self.choose("review-again-7", 1, True)

    def test_opt_in_and_recursion_guard(self):
        self.env.pop("WORK_LEAF_BENCH_FULL_INVERSE")
        with self.assertRaises(ValueError):
            self.choose("author-first", 1, False)
        self.env["WORK_LEAF_BENCH_FULL_INVERSE"] = "1"
        self.env["WORK_LEAF_BENCH_CODEX_ACTIVE"] = "1"
        with self.assertRaises(ValueError):
            self.choose("author-first", 1, False)

    def test_bad_plan_and_ordinals(self):
        for ordinal in (0, -1, True, "1"):
            with self.assertRaises(ValueError):
                self.choose("review-first", ordinal, False)
        self.plan["profiles"]["full"]["catalog_sha256"] = "not-a-digest"
        with self.assertRaises(ValueError):
            self.choose("author-first", 1, False)

    def test_fresh_stage_cursor_and_foreign_thread(self):
        cursor = f.StageCursor(None)
        cursor.enter(1, None)
        cursor.enter(2, "thread-a")
        with self.assertRaises(ValueError):
            cursor.enter(2, "thread-a")
        with self.assertRaises(ValueError):
            cursor.enter(4, "thread-a")
        with self.assertRaises(ValueError):
            cursor.enter(3, "thread-b")
        cursor.enter(3, "thread-a")

    def test_resumed_stage_cursor_binds_original_thread(self):
        cursor = f.StageCursor("thread-a")
        with self.assertRaises(ValueError):
            cursor.enter(1, None)
        with self.assertRaises(ValueError):
            cursor.enter(1, "thread-b")
        cursor.enter(1, "thread-a")
        cursor.enter(2, "thread-a")

    def test_generated_driver_exact_inverse_and_shell_syntax(self):
        original = f.DRIVER.read_text()
        changed = f.driver_source()
        restored = changed
        for before, after in reversed(f.driver_replacements()):
            restored = f.inverse.replace_once(restored, after, before)
        self.assertEqual(restored, original)
        self.assertEqual(subprocess.run(["bash", "-n"], input=changed, text=True, capture_output=True).returncode, 0)
        self.assertIn('WORK_LEAF_BENCH_INPUT_NEXT="$observer_proxy_dir/codex"', changed)
        for name in ("review_prompt", "linearize_plan_prompt_sequential", "linearize_accept_prompt_sequential", "normal_validation_guidance"):
            start = original.index(name + "() {")
            end = original.index("\n}\n", start) + 3
            self.assertIn(original[start:end], changed)

    def test_source_mutation_fails_closed(self):
        with patch.object(Path, "read_bytes", return_value=b"foreign driver"):
            with self.assertRaises(ValueError):
                f.driver_source()

    def test_published_plan_allows_every_later_review_round(self):
        import json
        self.plan = json.loads((f.HERE/'REFERENCE-PLAN.json').read_text())
        for number in (2, 9, 10, 19, 20, 100):
            self.choose(f'sequential-feature-2-review-{number}', 1, True)

    def test_host_reuses_verified_b_inverse_and_diagnostic_repair(self):
        host = f.load_host()
        self.assertIn("recovered_private_benchmark_host", host.__name__)
        self.assertIsNotNone(host.main)
        self.assertIsNotNone(host.owned_final)


if __name__ == "__main__":
    unittest.main()
