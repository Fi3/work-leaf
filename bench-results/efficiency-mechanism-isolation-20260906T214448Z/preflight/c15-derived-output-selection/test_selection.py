"""Actual-Git tests for explicit cold-build output exclusion; no provider/executor."""
import hashlib
import json
from pathlib import Path
import tempfile
import types
import unittest
from unittest import mock


HERE = Path(__file__).resolve().parent
ORIGINAL = HERE.parent / "c15-live-selection/live_selection.py"
ORIGINAL_SHA = "0614db9a5c874cd5f623997ce3433edbd2bc0ff3585c23913dd9ede9705d3a55"


def module(path, expected=None):
    body = path.read_bytes()
    if expected is not None:
        assert hashlib.sha256(body).hexdigest() == expected
    value = types.ModuleType("derived_output_test_" + path.parent.name)
    value.__file__ = str(path)
    exec(compile(body, str(path), "exec"), value.__dict__)
    return value


OLD = module(ORIGINAL, ORIGINAL_SHA)


class DerivedOutputSelection(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="c15-derived-output-")
        self.root = Path(self.temp.name)
        self.repo = self.root / "project"
        self.repo.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "Fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        (self.repo / ".gitignore").write_text("/build-products/\n/other-ignored/\n")
        (self.repo / "source.txt").write_text("accepted source\n")
        self.commit("fixture")
        self.owned = self.root / "selection"
        self.owned.mkdir()

    def tearDown(self):
        self.temp.cleanup()

    def git(self, *args):
        return OLD.git(self.repo, list(args))

    def commit(self, message):
        self.git("add", "--", ".")
        self.git("commit", "-qm", message)
        self.head = self.git("rev-parse", "HEAD").decode().strip()

    def output(self, name="build-products/deep/object.bin", body=b"generated only"):
        path = self.repo / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(body)
        return path

    def candidate(self):
        return module(HERE / "live_selection.py")

    def census(self, roots=("build-products",)):
        return self.candidate().census(self.repo, self.head, [], derived_output_roots=list(roots))

    def test_original_selector_rejects_actual_ignored_build_output(self):
        before = OLD.census(self.repo, self.head, [])
        output = self.output()
        self.assertEqual(self.git("ls-files", "--others", "--exclude-standard", "-z"), b"")
        self.assertIn(b"build-products/deep/object.bin\0", self.git("ls-files", "--others", "-z"))
        with self.assertRaisesRegex(ValueError, "untracked or ignored input unsupported"):
            OLD.census(self.repo, self.head, [])
        self.assertEqual(output.read_bytes(), b"generated only")
        self.assertEqual(before["commit"], self.head)

    def test_only_explicit_ignored_outputs_are_excluded_without_source_changes(self):
        before = OLD.census(self.repo, self.head, [])
        output = self.output()
        index = (self.repo / ".git/index").read_bytes()
        actual = self.census()
        self.assertEqual(actual["files"], before["files"])
        self.assertEqual(actual["derived_outputs"]["roots"], ["build-products"])
        self.assertEqual(actual["derived_outputs"]["ignored_paths"], ["build-products/deep/object.bin"])
        self.assertEqual(output.read_bytes(), b"generated only")
        self.assertEqual((self.repo / ".git/index").read_bytes(), index)
        self.assertEqual(self.git("rev-parse", "HEAD").decode().strip(), self.head)

    def test_default_census_remains_exact_and_unknown_outputs_still_fail(self):
        self.assertEqual(self.candidate().census(self.repo, self.head, []), OLD.census(self.repo, self.head, []))
        self.output()
        with self.assertRaises(ValueError):
            self.candidate().census(self.repo, self.head, [])
        self.output("other-ignored/unknown.bin")
        with self.assertRaises(ValueError):
            self.census()

    def test_nonignored_file_under_declared_root_is_not_an_exclusion(self):
        (self.repo / ".gitignore").write_text("/build-products/*\n!/build-products/input.txt\n")
        self.commit("declare nonignored input")
        self.output("build-products/input.txt")
        with self.assertRaises(ValueError):
            self.census()

    def test_tracked_descendant_cannot_be_declared_derived(self):
        self.output("build-products/required.txt", b"tracked source")
        self.git("add", "-f", "--", "build-products/required.txt")
        self.git("commit", "-qm", "tracked child")
        self.head = self.git("rev-parse", "HEAD").decode().strip()
        with self.assertRaises(ValueError):
            self.census()

    def test_unsafe_or_overlapping_declarations_fail(self):
        for roots in (["."], [""], ["/tmp"], ["../outside"], [".git"], [".git/cache"],
                      ["build-products", "build-products/deep"], ["build-products", "build-products"],
                      ["source.txt"], ["build-products/../other"]):
            with self.subTest(roots=roots), self.assertRaises(ValueError):
                self.census(roots)

    def test_symlinked_root_or_ancestor_is_rejected(self):
        outside = self.root / "outside"
        outside.mkdir()
        (self.repo / "build-products").symlink_to(outside, target_is_directory=True)
        for roots in (["build-products"], ["build-products/nested"]):
            with self.subTest(roots=roots), self.assertRaises(ValueError):
                self.census(roots)

    def test_generated_growth_does_not_hide_accepted_source_drift(self):
        candidate = self.candidate()
        self.output()
        original_git = candidate.git

        def grow(root, args, **kwargs):
            result = original_git(root, args, **kwargs)
            if args[:2] == ["bundle", "create"]:
                self.output("build-products/later.bin", b"later generated")
            return result

        with mock.patch.object(candidate, "git", side_effect=grow):
            selected = candidate.capture_selection(self.repo, self.owned, self.head, [],
                                                   derived_output_roots=["build-products"])
        endpoints = selected["derived_output_endpoints"]
        self.assertNotEqual(endpoints["before"]["ignored_paths"], endpoints["after"]["ignored_paths"])
        self.assertEqual(selected["commit"], self.head)
        self.assertEqual((self.repo / "build-products/later.bin").read_bytes(), b"later generated")

    def test_private_materialization_contains_only_selected_source(self):
        candidate = self.candidate()
        output = self.output()
        selected = candidate.capture_selection(self.repo, self.owned, self.head, [],
                                               derived_output_roots=["build-products"])
        destination = self.root / "materialized"
        destination.mkdir()
        snapshot = candidate.materialize_selected(selected, destination)
        private = Path(snapshot["repo"])
        self.assertEqual((private / "source.txt").read_text(), "accepted source\n")
        self.assertFalse((private / "build-products").exists())
        self.assertEqual(output.read_bytes(), b"generated only")

    def test_tracked_drift_during_bundle_is_a_retained_failure(self):
        candidate = self.candidate()
        self.output()
        original_git = candidate.git

        def drift(root, args, **kwargs):
            result = original_git(root, args, **kwargs)
            if args[:2] == ["bundle", "create"]:
                (self.repo / "source.txt").write_text("unaccepted mutation\n")
            return result

        with mock.patch.object(candidate, "git", side_effect=drift), self.assertRaises(ValueError):
            candidate.capture_selection(self.repo, self.owned, self.head, [],
                                        derived_output_roots=["build-products"])
        self.assertTrue((self.owned / "failure.json").is_file())
        self.assertFalse((self.owned / "SELECTED.json").exists())
        self.assertEqual((self.repo / "source.txt").read_text(), "unaccepted mutation\n")

    def test_untracked_ignore_rule_cannot_authorize_derived_input(self):
        (self.repo / ".gitignore").write_text("/build-products/*\n")
        self.commit("tracked parent rule")
        self.output("build-products/.gitignore", b"*.bin\n")
        self.output("build-products/object.bin")
        proof = OLD.git(self.repo, ["check-ignore", "-v", "-z", "--stdin"],
                        data=b"build-products/object.bin\0")
        self.assertIn(b"build-products/.gitignore\0", proof)
        with self.assertRaises(ValueError):
            self.census()

    def test_external_ignore_file_is_not_implicit_authority(self):
        external = self.root / "ignore-policy"
        external.write_text("*.bin\n")
        self.git("config", "core.excludesFile", str(external))
        self.output()
        with self.assertRaises(ValueError):
            self.census()

    def test_ignore_authority_and_declaration_are_retained(self):
        self.output()
        actual = self.census()["derived_outputs"]
        self.assertEqual(actual["rules"]["build-products/deep/object.bin"]["source"], ".gitignore")
        self.assertEqual(actual["rules"]["build-products/deep/object.bin"]["pattern"], "/build-products/")

    def test_ignored_symlink_or_hardlink_is_not_derived_source(self):
        directory = self.repo / "build-products"
        directory.mkdir()
        link = directory / "alias"
        link.symlink_to(self.repo / "source.txt")
        with self.assertRaises(ValueError):
            self.census()
        link.unlink()
        import os
        os.link(self.repo / "source.txt", link)
        with self.assertRaises(ValueError):
            self.census()

    def test_nested_administration_and_non_list_declarations_fail(self):
        for roots in (["build-products/.git/objects"], "build-products", [False]):
            with self.subTest(roots=roots), self.assertRaises(ValueError):
                self.candidate().census(self.repo, self.head, [], derived_output_roots=roots)

    def test_output_body_is_never_read_and_rule_changes_fail(self):
        candidate = self.candidate()
        path = self.output()
        original_read = Path.read_bytes

        def read(instance):
            self.assertNotEqual(instance, path, "excluded output body read")
            return original_read(instance)

        with mock.patch.object(Path, "read_bytes", read):
            candidate.census(self.repo, self.head, [], derived_output_roots=["build-products"])
        original_git = candidate.git

        def drift(root, args, **kwargs):
            result = original_git(root, args, **kwargs)
            if args[:2] == ["bundle", "create"]:
                (self.repo / ".git/info/exclude").write_text("new-rule\n")
            return result

        with mock.patch.object(candidate, "git", side_effect=drift), self.assertRaises(ValueError):
            candidate.capture_selection(self.repo, self.owned, self.head, [], derived_output_roots=["build-products"])
        self.assertTrue((self.owned / "failure.json").is_file())

    def test_rule_administration_drift_inside_one_census_is_rejected(self):
        candidate = self.candidate()
        self.output()
        original_git = candidate.git

        def drift(root, args, **kwargs):
            result = original_git(root, args, **kwargs)
            if args[:2] == ["check-ignore", "-v"]:
                (self.repo / ".git/info/exclude").write_text("changed while classifying\n")
            return result

        with mock.patch.object(candidate, "git", side_effect=drift), self.assertRaisesRegex(ValueError, "authority.*drift"):
            candidate.census(self.repo, self.head, [], derived_output_roots=["build-products"])

    def test_even_unused_untracked_ignore_administration_is_not_output(self):
        self.output("build-products/.gitignore", b"an-unused-pattern\n")
        with self.assertRaisesRegex(ValueError, "untracked ignore administration"):
            self.census()


if __name__ == "__main__":
    unittest.main()
