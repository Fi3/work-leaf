"""Offline tests of the thin work-unit secondary accounting bridge."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

import analyze as frozen
from test_analysis import tail
import report_work_unit_tokens as subject


def fixture(missing=0):
    usage = {"input_tokens": 100, "cached_input_tokens": 40,
             "output_tokens": 20, "reasoning_output_tokens": 5}
    observed = {"usage_scopes": {"total_workflow": usage}, "interrupted_provider_turns": missing,
                "errors": [f"interrupted provider turn has no complete usage: count={missing}"] if missing else [],
                "capture_complete": missing == 0, "invocation_count": 1, "complete_invocation_count": 1,
                "model_strata": [{"model": "gpt-5.5", "effort": "xhigh"}]}
    report = {"agent_model": "gpt-5.5", "agent_reasoning_effort": "xhigh", "total_workflow_usage": usage,
              "workflow_result": "pass"}
    capture = {"path": "capture", "provenance": {"errors": []}, "inventory": {
        "gaps": [], "errors": [], "raw_responses": {"response": {
            "thread_id": "thread", "turn_id": "turn", "usage": frozen.usage(usage)}}},
        "response_sources": {"response": [3]}}
    return observed, report, [capture]


def row(rid, condition, lower=120):
    return {"id": rid, "condition": condition, "started_at": "time", "launch_status": "completed",
            "workflow_result": "pass", "errors": [], "exact_response_evidence": {},
            "measurement": {"bounds": {m: {"lower": lower, "upper": lower} for m in frozen.METRICS}}}


def gate():
    return {"integrity_errors": [], "observations": [{"errors": []}], "primary": {"status": "available"}}


class AccountingTests(unittest.TestCase):
    def test_response_refs_use_physical_source_lines_not_filtered_record_index(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/"server.raw"
            path.write_text('\n'+json.dumps({"method": "rawResponse/completed", "params": {"responseId": "r"}})+'\n\n')
            records, refs = subject.server_records(path)
            self.assertEqual(len(records), 1)
            self.assertEqual(refs, {"r": [2]})

    def test_exact_unchanged_raw_and_uncached_arithmetic(self):
        result = subject.measure(*fixture(), frozen)
        self.assertEqual(result["measurement"]["bounds"][frozen.METRICS[0]], {"lower": 120, "upper": 120})
        self.assertEqual(result["measurement"]["bounds"][frozen.METRICS[1]], {"lower": 80, "upper": 80})
        self.assertEqual(result["exact_completed_response_id_count"], 1)
        self.assertEqual(result["exact_response_evidence"]["response"]["sources"], [{"capture": "capture", "server_line": 3}])

    def test_prospective_multi_item_tail_uses_existing_ceiling(self):
        observed, report, captures = fixture(1)
        captures[0]["inventory"] = frozen.inventory(*tail(3))
        result = subject.measure(observed, report, captures, frozen)
        self.assertEqual(result["measurement"]["status"], "bounded")
        self.assertEqual(result["measurement"]["bounds"][frozen.METRICS[0]]["upper"], 1178120)
        self.assertFalse(result["exhaustive_model_call_count"])

    def test_unknown_tail_stays_unknown(self):
        observed, report, captures = fixture(1)
        captures[0]["inventory"]["gaps"] = [{"thread_id": "thread", "turn_id": "turn", "response_count_upper": None}]
        result = subject.measure(observed, report, captures, frozen)
        self.assertIsNone(result["measurement"]["bounds"][frozen.METRICS[0]]["upper"])

    def test_exact_duplicate_response_deduplicated_conflict_rejected(self):
        observed, report, captures = fixture()
        captures.append(copy.deepcopy(captures[0])); captures[1]["path"] = "capture2"
        result = subject.measure(observed, report, captures, frozen)
        self.assertEqual(result["exact_completed_response_id_count"], 1)
        self.assertEqual(len(result["exact_response_evidence"]["response"]["sources"]), 2)
        captures[1]["inventory"]["raw_responses"]["response"]["turn_id"] = "other"
        self.assertEqual(subject.measure(observed, report, captures, frozen)["measurement"]["status"], "ineligible")

    def test_provenance_report_mismatch_and_bool_identity_fail_closed(self):
        for kind in ("provenance", "usage", "model", "response", "thread", "turn"):
            observed, report, captures = fixture()
            if kind == "provenance": captures[0]["provenance"]["errors"].append("source differs")
            elif kind == "usage": report["total_workflow_usage"] = {**report["total_workflow_usage"], "input_tokens": 101}
            elif kind == "model": report["agent_model"] = "other"
            elif kind == "response": captures[0]["inventory"]["raw_responses"][True] = captures[0]["inventory"]["raw_responses"].pop("response")
            else: captures[0]["inventory"]["raw_responses"]["response"][kind+"_id"] = True
            self.assertEqual(subject.measure(observed, report, captures, frozen)["measurement"]["status"], "ineligible", kind)

    def test_no_capture_and_observer_failure_remain_ineligible(self):
        observed, report, _ = fixture()
        self.assertEqual(subject.measure(observed, report, [], frozen)["measurement"]["status"], "ineligible")
        observed["errors"] = ["observer executable SHA-256 changed after observer initialization"]
        self.assertEqual(subject.measure(observed, report, fixture()[2], frozen)["measurement"]["status"], "ineligible")


class ComparisonTests(unittest.TestCase):
    def test_failure_retained_no_primary_significance_gate_or_token_inference(self):
        rows = [row("c", "control"), row("v", subject.VARIANT, 150)]
        rows[1]["workflow_result"] = "fail"; rows[1]["launcher_exit_code"] = 1
        result = subject.summarize(rows, gate(), frozen)
        self.assertEqual(result["status"], "available")
        self.assertEqual(result[frozen.METRICS[0]]["variant_minus_control"], {"lower": 30, "upper": 30})
        self.assertNotIn("permutation", result)

    def test_missing_row_unlaunched_duplicate_and_cross_workflow_response_disable_contrast(self):
        for kind in ("missing", "unlaunched", "duplicate", "response"):
            rows = [row("c", "control"), row("v", subject.VARIANT)]
            if kind == "missing": rows[1]["measurement"]["bounds"] = None
            elif kind == "unlaunched": rows[1]["started_at"] = None
            elif kind == "duplicate": rows.append(copy.deepcopy(rows[1]))
            else:
                for r in rows: r["exact_response_evidence"] = {"same-response": {}}
            self.assertNotEqual(subject.summarize(rows, gate(), frozen)["status"], "available", kind)
            self.assertEqual(len(rows), 3 if kind == "duplicate" else 2)

    def test_independent_source_exposure_gate_not_saved_success_alone(self):
        rows = [row("c", "control"), row("v", subject.VARIANT)]
        for kind in ("integrity", "observation", "primary"):
            check = gate()
            if kind == "integrity": check["integrity_errors"] = ["recorded phase source incident"]
            elif kind == "observation": check["observations"][0]["errors"] = ["unmatched prompt"]
            else: check["primary"]["status"] = "unavailable"
            self.assertNotEqual(subject.summarize(rows, check, frozen)["status"], "available")


if __name__ == "__main__":
    unittest.main()
