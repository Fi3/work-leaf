from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "full-analysis-012"))
from test_full_aggregate import AggregateTests
import full_audit as original

try:
    import population_audit as audited
except ModuleNotFoundError:
    audited = original


class PopulationTests(AggregateTests):
    def evaluate(self, mutation=None):
        batch, sessions, output, run = self.fixture()
        artifact = batch / "runs" / run / (run + "-three-feature-sequential-bench-artifacts")
        (artifact / "unit").mkdir()
        (artifact / "unit" / "result").write_text("unchanged")
        (artifact / "unit-log").write_text("unchanged")
        source = audited.aggregate_source(batch, run, "/fixture/repo", output, sessions_root=sessions)
        if mutation:
            source = original.replace_once(source, "after={p:sha(Path(p).read_bytes()) for p in originals}",
                mutation + "\nafter={p:sha(Path(p).read_bytes()) for p in originals}")
        stream = io.StringIO()
        with redirect_stdout(stream):
            exec(compile(source, "population-fixture", "exec"),
                 {"NativeCache":original.NativeCache, "public_lifecycle":original.public_lifecycle})
        return json.loads(stream.getvalue())

    def test_unchanged_prefix_paths_do_not_report_population_mutation(self):
        self.assertEqual(self.evaluate()["errors"], [])

    def test_new_archive_file_remains_rejected(self):
        result = self.evaluate("(A/'unexpected').write_text('new')")
        self.assertIn("archive file population changed during audit", result["errors"])

    def test_changed_archive_content_remains_rejected(self):
        result = self.evaluate("(A/'unit-log').write_text('changed')")
        self.assertIn("original files changed during audit", result["errors"])


if __name__ == "__main__":
    unittest.main()
