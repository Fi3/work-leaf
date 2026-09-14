"""Fail-first checks for a startup-only private catalog view."""
import unittest
import private_catalog as catalog


class CatalogTests(unittest.TestCase):
    def test_requires_explicit_opt_in_and_blocks_recursion(self):
        for env in ({}, {"WORK_LEAF_BENCH_P01_CATALOG": "1", "WORK_LEAF_BENCH_CODEX_ACTIVE": "1"}):
            with self.assertRaises(ValueError):
                catalog.needs_view(["exec", "-"], env)

    def test_native_resume_is_unchanged(self):
        self.assertFalse(catalog.needs_view(["exec", "resume", "--json", "thread", "-"],
            {"WORK_LEAF_BENCH_P01_CATALOG": "1"}))

    def test_fresh_launch_needs_private_view(self):
        self.assertTrue(catalog.needs_view(["--cd", "/tmp/repo", "exec", "--json", "-"],
            {"WORK_LEAF_BENCH_P01_CATALOG": "1"}))

    def test_non_exec_is_rejected(self):
        with self.assertRaises(ValueError):
            catalog.needs_view(["app-server"], {"WORK_LEAF_BENCH_P01_CATALOG": "1"})


if __name__ == "__main__":
    unittest.main()
