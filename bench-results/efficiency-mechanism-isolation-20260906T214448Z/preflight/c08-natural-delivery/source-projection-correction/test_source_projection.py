"""Separate projection regressions using independent temporary metadata only."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

import audit_automatic_refresh_sources_corrected as corrected


BASE = Path(__file__).resolve().parent.parent
BASE_SHA = "d05e7e7daa62601782fbcca2058f8c31a81d29ac5301407643474a09f4889f80"
TEST_SHA = "25c532f240bc49f070f318029a967e531d15e6ce1a055923e6fa94ec62470502"
EXTRAS = {"raw_response_usage_start_sha256", "raw_response_usage_sha256"}


def load_static(name, path, expected):
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected:
        raise AssertionError("original source pin differs")
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    exec(compile(raw, str(path), "exec"), module.__dict__)
    return module


class EndProjection(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.original = load_static("audit_automatic_refresh_sources", BASE / "audit_automatic_refresh_sources.py", BASE_SHA)
        with patch.dict(sys.modules, {"audit_automatic_refresh_sources": cls.original}):
            cls.fixtures = load_static("original_source_tests", BASE / "test_sources.py", TEST_SHA)
        cls.fixtures.TemporarySources.setUpClass()

    def setUp(self):
        self.fixture = self.fixtures.TemporarySources(methodName="runTest")
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)

    def metadata(self):
        manifest = self.fixture.fixture()
        cap = manifest["captures"][0]
        end = json.loads(Path(cap["end"]["path"]).read_bytes())
        # The smallest original fixture omitted this typed InvocationEnd field.
        end["stderr_sha256"] = hashlib.sha256(b"").hexdigest()
        self.write_end(manifest, end)
        return manifest, end

    def write_end(self, manifest, end, ledger_mutation=None):
        cap = manifest["captures"][0]
        cap["end"] = self.fixture.put(Path(cap["end"]["path"]), end)
        manifest["invocation_metadata"][0]["end"] = cap["end"]
        start = json.loads(Path(cap["start"]["path"]).read_bytes())
        typed_end = {key: value for key, value in end.items() if key not in EXTRAS}
        if ledger_mutation is not None: ledger_mutation(typed_end)
        logged = dict({key: value for key, value in start.items()
                       if key not in ("raw_response_usage", "project_layer_inventory_required")}, end=typed_end)
        manifest["invocations"] = self.fixture.put(Path(manifest["invocations"]["path"]), self.fixtures.rows_raw([logged]), raw=True)

    def collect(self, module, manifest):
        return module.collect_closed_captures(manifest, self.fixture.collector.Sources(),
                                              self.fixture.collector, self.fixture.frame)

    def test_typed_ledger_extended_raw_end_original_fails_corrected_passes(self):
        manifest, _ = self.metadata()
        original = self.collect(self.original, manifest)
        self.assertIn({"code": "capture-population-or-source-invalid"}, original["errors"])
        self.assertEqual(original["invocations"], [])
        with patch.object(self.fixture.frame, "prove_frames", wraps=self.fixture.frame.prove_frames) as prove:
            result = self.collect(corrected, manifest)
        self.assertEqual(result["errors"], [])
        self.assertEqual(len(result["invocations"]), 1)
        self.assertEqual(len(result["frames"]), 1)
        self.assertEqual(prove.call_count, 1)
        self.assertTrue(EXTRAS.issubset(result["invocations"][0]["end"]))

    def test_changed_shared_end_fields_still_reject(self):
        for key, bad in (("exit_code", 7), ("stdin_sha256", "0" * 64), ("end_monotonic_ns", True)):
            with self.subTest(key=key):
                manifest, end = self.metadata()
                self.write_end(manifest, end, lambda typed: typed.__setitem__(key, bad))
                self.assertTrue(self.collect(corrected, manifest)["errors"])

    def test_nonraw_end_retains_full_typed_identity_without_supplements(self):
        manifest, raw = self.metadata()
        end = {key: value for key, value in raw.items() if key not in EXTRAS}
        start = {"capture_kind": "locked-command"}
        for marked in (start, dict(start, raw_response_usage=False)):
            self.assertEqual(corrected.typed_end(marked, end, manifest["captures"][0]["start"], None), end)
            with self.assertRaises(ValueError):
                corrected.typed_end(marked, raw, manifest["captures"][0]["start"], None)

    def test_missing_and_wrong_supplement_hashes_still_reject(self):
        for mutation in ("missing-start", "wrong-start", "missing-map", "missing-forwarded", "wrong-forwarded", "wrong-grace"):
            with self.subTest(mutation=mutation):
                manifest, end = self.metadata()
                if mutation == "missing-start": end.pop("raw_response_usage_start_sha256")
                elif mutation == "wrong-start": end["raw_response_usage_start_sha256"] = "0" * 64
                elif mutation == "missing-map": end.pop("raw_response_usage_sha256")
                elif mutation == "missing-forwarded": end["raw_response_usage_sha256"].pop("client-to-server.forwarded.raw")
                else:
                    name = "client-to-server.forwarded.raw" if mutation == "wrong-forwarded" else "provider-usage-grace.jsonl"
                    end["raw_response_usage_sha256"][name] = "0" * 64
                self.write_end(manifest, end)
                self.assertTrue(self.collect(corrected, manifest)["errors"])

    def test_unknown_extra_and_unmarked_raw_supplements_reject(self):
        for mutation in ("raw-only", "both-unknown", "extra-digest", "unmarked"):
            with self.subTest(mutation=mutation):
                manifest, end = self.metadata()
                if mutation in ("raw-only", "both-unknown"): end["foreign_end_field"] = "not-owned"
                elif mutation == "extra-digest": end["raw_response_usage_sha256"]["foreign.raw"] = "0" * 64
                else:
                    cap = manifest["captures"][0]
                    start = json.loads(Path(cap["start"]["path"]).read_bytes())
                    start.pop("raw_response_usage")
                    cap["start"] = self.fixture.put(Path(cap["start"]["path"]), start)
                    manifest["invocation_metadata"][0]["start"] = cap["start"]
                    end["raw_response_usage_start_sha256"] = cap["start"]["sha256"]
                drop = (lambda typed: typed.pop("foreign_end_field")) if mutation == "raw-only" else None
                self.write_end(manifest, end, drop)
                self.assertTrue(self.collect(corrected, manifest)["errors"])

    def test_complete_corrected_cli_uses_typed_end_and_own_source_pin_once(self):
        ref, output = self.fixture.complete_execute_fixture()
        value = json.loads(Path(ref["path"]).read_bytes())
        value["helper_sha256"] = hashlib.sha256(Path(corrected.__file__).read_bytes()).hexdigest()
        raw_end = json.loads(Path(value["captures"][0]["end"]["path"]).read_bytes())
        self.write_end(value, raw_end)
        ref = self.fixture.put(Path(ref["path"]), value)
        args = ["--input", ref["path"], "--input-sha256", ref["sha256"], "--output", str(output)]
        self.assertEqual(corrected.main(args), 0)
        result = json.loads(output.read_bytes())
        self.assertEqual(result["status"], "available")
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["source_sha256"][str(Path(corrected.__file__).resolve())], value["helper_sha256"])
        self.assertEqual(result["source_sha256"][str(BASE / "audit_automatic_refresh_sources.py")], BASE_SHA)
        self.assertEqual(result["terminal"]["exit_code"], 1)
        self.assertFalse(result["original_reports"][0]["flags"]["capture_complete"])
        self.assertEqual(result["delivery"]["inputs"][0]["status"], "joined")
        full = {key: item for key, item in result.items() if key not in ("full_result_sha256", "full_result_bytes", "metadata_list_entries")}
        self.assertEqual(result["full_result_sha256"], hashlib.sha256(self.fixtures.canonical(full)).hexdigest())
        with patch.object(corrected, "execute") as execute:
            with self.assertRaises(ValueError): corrected.main(args)
            execute.assert_not_called()


if __name__ == "__main__":
    unittest.main()
