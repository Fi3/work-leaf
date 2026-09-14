"""Private inverse seams: real local Git/command tests, never a provider launch."""
import pathlib
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch

import author_inverse as inverse


class InverseTests(unittest.TestCase):
    def test_identity_has_exact_saved_prompt_and_host_bytes(self):
        prompt = inverse.REFERENCE_PROMPT.read_text()
        self.assertEqual(inverse.transform_prompt(prompt, "identity"), prompt)
        self.assertEqual(inverse.host_source("identity"), inverse.BASE_HOST.read_text())

    def test_a_removes_only_its_two_owned_spans(self):
        original = inverse.REFERENCE_PROMPT.read_text()
        changed = inverse.transform_prompt(original, "R02")
        for before, after in inverse.A_SPANS:
            self.assertNotIn(before, changed)
            changed = changed.replace(after, before)
        self.assertEqual(changed, original)
        self.assertEqual(inverse.host_source("R02"), inverse.BASE_HOST.read_text())

    def test_b_removes_launch_cue_but_preserves_a_and_protocol(self):
        original = inverse.REFERENCE_PROMPT.read_text()
        changed = inverse.transform_prompt(original, "R03")
        before, after = inverse.B_LAUNCH
        self.assertEqual(changed.replace(after, before), original)
        self.assertIn(inverse.A_SPANS[0][0], changed)
        self.assertIn("Do not combine edit, run, discard or done", changed)

    def test_ab_is_exact_union_of_declared_spans(self):
        original = inverse.REFERENCE_PROMPT.read_text()
        changed = inverse.transform_prompt(original, "R04")
        for before, after in inverse.A_SPANS + [inverse.B_LAUNCH]:
            changed = changed.replace(after, before)
        self.assertEqual(changed, original)

    def test_feedback_changes_are_exact_and_invertible(self):
        original = inverse.BASE_HOST.read_text()
        changed = inverse.host_source("R04")
        for before, after in inverse.B_SOURCE:
            self.assertEqual(changed.count(after), 1)
            changed = changed.replace(after, before)
        self.assertEqual(changed, original)
        self.assertEqual(inverse.host_source("R03"), inverse.host_source("R04"))

    def test_full_inverse_matches_original_native_prompt_without_old_cohesion_suffix(self):
        original_native = (inverse.STUDY / "screens/native-production-cohesion-20260912/PROMPT.txt"
                           ).read_text().split("\n\nProduction edit organization:", 1)[0]
        changed = inverse.transform_prompt(inverse.REFERENCE_PROMPT.read_text(), "R06")
        self.assertEqual(changed.strip(), original_native.strip())
        self.assertNotIn("@standalone", changed)

    def test_unknown_or_unfaithful_arm_is_not_supported(self):
        for arm in ("R05", "R13", "", "normal-control"):
            with self.subTest(arm=arm), self.assertRaises(ValueError):
                inverse.transform_prompt(inverse.REFERENCE_PROMPT.read_text(), arm)

    def test_missing_or_duplicate_owned_span_fails_closed(self):
        prompt = inverse.REFERENCE_PROMPT.read_text()
        owned = inverse.A_SPANS[0][0]
        for changed in (prompt.replace(owned, ""), prompt + owned):
            with self.assertRaises(ValueError):
                inverse.transform_prompt(changed, "R02")


class HostPathTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="wl-inverse-unit-")
        self.root = pathlib.Path(self.temp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "Bounded Test")
        self.git("config", "user.email", "test@example.invalid")
        (self.repo / "value.py").write_text("VALUE = 0\n")
        self.git("add", "value.py")
        self.git("commit", "-qm", "ADD initial local fixture")
        self.module = inverse.load_host("R04")
        self.host = self.module.Host(self.repo, self.root / "artifacts",
                                     "fixture", "initial", time.monotonic() + 30, True)

    def tearDown(self):
        self.temp.cleanup()

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.repo), *args],
                                       stderr=subprocess.STDOUT)

    def test_test_only_publication_and_real_red_remain_allowed(self):
        accepted = self.host.consume("@standalone edit test missing behavior\n"
            "*** Begin Patch\n*** Add File: contract_test.py\n"
            "+from value import VALUE\n+assert VALUE == 1\n*** End Patch\n@standalone end")
        self.assertIn("Host accepted files: contract_test.py.", accepted)
        self.assertNotIn("Run at most one", accepted)
        self.assertEqual(len(self.host.accepted_commits), 1)
        red = self.host.consume("@standalone run -- python3 -B contract_test.py")
        self.assertIn("status: 1", red)
        self.assertIn("AssertionError", red)
        self.assertFalse(self.host.completed)
        self.assertNotIn("When the required work and focused validation", red)

    def test_b_does_not_transform_the_same_words_inside_real_stdout(self):
        output = self.host.command([], "printf '%s' 'When the required work and focused validation are complete, send @standalone done.'")
        self.assertIn("stdout:\nWhen the required work and focused validation are complete, send @standalone done.", output)
        self.assertTrue(output.endswith("stderr:\n<empty>\n"))

    def test_malformed_stale_and_grouped_replies_still_reject_without_effects(self):
        before = self.git("rev-parse", "HEAD")
        for reply in ("Some prose\n@standalone done",
                      "@standalone run -- true\n@standalone done",
                      "@standalone edit stale\n*** Begin Patch\n*** Update File: value.py\n@@\n-NOT PRESENT\n+VALUE = 1\n*** End Patch\n@standalone end"):
            with self.subTest(reply=reply):
                feedback = self.host.consume(reply)
                self.assertIn("rejected", feedback)
                self.assertEqual(self.git("rev-parse", "HEAD"), before)
                self.assertFalse(self.host.completed)
                self.assertEqual(self.host.accepted_commits, [])

    def test_pending_command_edits_still_block_done_and_allow_explicit_discard(self):
        result = self.host.consume("@standalone run value.py -- printf 'VALUE = 2\\n' > value.py")
        self.assertIn("pending", result)
        self.assertEqual((self.repo / "value.py").read_text(), "VALUE = 0\n")
        self.assertIn("DONE rejected", self.host.consume("@standalone done"))
        self.assertFalse(self.host.completed)
        self.assertEqual(self.host.consume("@standalone discard not needed"),
                         "Pending output explicitly discarded.")
        self.assertIsNone(self.host.consume("@standalone done"))
        self.assertTrue(self.host.completed)

    def test_native_write_only_toggle_would_violate_existing_custody(self):
        (self.repo / "value.py").write_text("VALUE = 3\n")
        with self.assertRaises(self.module.Fatal):
            self.host.unchanged()

    def test_native_success_keeps_original_post_author_commit_boundary(self):
        root = self.root / "native"
        prompt = self.root / "prompt.txt"
        prompt.write_text("fixture")
        def child(*args):
            (self.repo / "value.py").write_text("VALUE = 4\n")
            return {"exit_code": 0, "timed_out": False, "cancelled_signal": None}
        argv = ["inverse", "--arm", "R06", "--repo", str(self.repo),
                "--artifact-dir", str(root), "--prompt-file", str(prompt),
                "--codex-bin", "/unused/fixture"]
        with patch.object(inverse, "load_host", return_value=self.module), \
             patch.object(self.module, "execute_child", side_effect=child) as execute, \
             patch.object(self.module, "owned_final", return_value=("fixture-thread", "done", ["done"])), \
             patch("sys.argv", argv), patch.dict("os.environ", {"WORK_LEAF_BENCH_INVERSE": "1"}):
            self.assertEqual(inverse.main(), 0)
        self.assertEqual(execute.call_count, 1)
        self.assertEqual(self.git("status", "--porcelain"), b"")
        self.assertEqual(self.git("show", "HEAD:value.py"), b"VALUE = 4\n")


if __name__ == "__main__":
    unittest.main()
