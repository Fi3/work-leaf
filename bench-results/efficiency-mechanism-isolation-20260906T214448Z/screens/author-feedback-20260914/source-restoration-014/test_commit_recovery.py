"""Exact object identity, not merely equivalent file contents, is the gate."""
import hashlib
import unittest

import commit_recovery


class CommitRecoveryTests(unittest.TestCase):
    def setUp(self):
        self.tree = "a" * 40
        self.parent = "b" * 40
        self.message = "UPDATE 1: retained reason\n"
        self.identity = "Work Leaf Bench <bench@example.com>"
        self.raw = (f"tree {self.tree}\nparent {self.parent}\n"
                    f"author {self.identity} 1789435000 +0200\n"
                    f"committer {self.identity} 1789435000 +0200\n\n"
                    f"{self.message}").encode()
        self.target = hashlib.sha1(
            f"commit {len(self.raw)}\0".encode() + self.raw).hexdigest()

    def recover(self, **overrides):
        args = dict(tree=self.tree, parent=self.parent, message=self.message,
                    identity=self.identity, start=1789434999, end=1789435001,
                    timezone="+0200", target=self.target)
        args.update(overrides)
        return commit_recovery.recover(**args)

    def test_exact_retained_object_identity(self):
        self.assertEqual(self.recover(), (self.raw, 1789435000))

    def test_wrong_tree_parent_or_message_is_not_equivalent(self):
        for override in ({"tree": "c" * 40}, {"parent": "d" * 40},
                         {"message": "different\n"}):
            with self.subTest(override=override), self.assertRaises(ValueError):
                self.recover(**override)

    def test_wrong_timezone_or_outside_interval_fails(self):
        for override in ({"timezone": "+0000"}, {"end": 1789434999}):
            with self.subTest(override=override), self.assertRaises(ValueError):
                self.recover(**override)

    def test_rejects_unbounded_or_malformed_search(self):
        for override in ({"end": 1789635000}, {"tree": "bad"},
                         {"identity": "name\nextra header"},
                         {"start": 1789435002}):
            with self.subTest(override=override), self.assertRaises(ValueError):
                self.recover(**override)


if __name__ == "__main__":
    unittest.main()
