"""Synthetic prospective v3 delivery/raw-primary tests; no provider calls."""
import copy
import itertools
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import analyze_untracked_reads as subject


def fnv(text):
    value = 0xcbf29ce484222325
    for byte in text.encode():
        value = ((value ^ byte) * 0x100000001b3) % (2**64)
    return f"fnv64:{value:016x}; bytes:{len(text.encode())}"


def event(condition="control", *, body=None, success=True, prefix="", suffix="", sequence=1):
    body = "λ"*9000 if body is None else body
    path = "src/example.rs"; size = len(body.encode()); digest = fnv(body)
    threshold = size > 16384
    success = threshold and success
    bundle_path = "/tmp/context/bundle-0.md" if success else None
    opening = "work-leaf file text\n\n--- "+path+" ---\n"
    inline = opening+body+("" if body.endswith("\n") else "\n")
    baseline = ("work-leaf file text\nExact file text is in an orchestrator context bundle instead of this chat to keep the agent session compact.\n"
                +"Context bundle: "+bundle_path+"\nYou may read this temporary bundle file for the exact mediated file text. Do not edit the bundle; project writes still require `@work-leaf edit`.\n"
                +"Bundled files:\n- "+path+" ("+digest+")\n") if success else inline
    selected = "inline" if condition == "untracked-read-inline" and success else "baseline"
    b = prefix+baseline+suffix; a = prefix+inline+suffix
    n = len(prefix.encode()); changed = selected == "inline" and b != a
    return {"schema": "work-leaf-bench-experiment-v3", "event": "read-response", "site": "file-read",
            "run_id": "run", "condition": condition, "process_id": 12, "sequence": sequence,
            "unix_time_ns": "123", "agent_id": "agent", "baseline_prompt": b, "inline_candidate_prompt": a,
            "baseline_bytes": len(b.encode()), "inline_candidate_bytes": len(a.encode()),
            "selected_candidate": selected, "selected_bytes": len((a if changed else b).encode()),
            "changed": changed, "candidate_byte_delta": len(a.encode())-len(b.encode()),
            "byte_delta": len(a.encode())-len(b.encode()) if changed else 0,
            "component": {"baseline_start": n, "baseline_end": n+len(baseline.encode()),
                          "inline_start": n, "inline_end": n+len(inline.encode())},
            "bundle": {"threshold_eligible": threshold, "write_succeeded": success, "path": bundle_path},
            "eligible": success, "eligibility_reason": "bundled-untracked" if success else "bundle-write-failed" if threshold else "below-threshold",
            "requested_paths": [path], "snapshots": [{"path": path, "class": "untracked", "bytes": size,
               "digest": digest, "inline_body_start": n+len(opening.encode()), "inline_body_end": n+len(opening.encode())+size}],
            "failures": []}


def nonread(condition="control", sequence=1):
    core = subject.pinned("analyze_work_units.py")
    original = "work-leaf patch applied\nfiles: src/a.rs\nNext step: "+core.ACK+"\n"+core.C[0]
    spans = []
    for name, text in (("patch-applied-validation", core.ACK), ("patch-applied-remaining-work", core.C[0])):
        start = original.encode().index(text.encode())
        spans.append({"id": name, "cue_start": start, "cue_end": start+len(text.encode()),
                      "original": text, "replacement": text, "changed": False, "byte_delta": 0})
    return {"event": "prompt", "schema": subject.SCHEMA, "run_id": "run", "condition": condition,
            "process_id": 12, "sequence": sequence, "unix_time_ns": "123", "agent_id": "agent",
            "site": "patch-applied", "spans": spans, "original_prompt": original,
            "forwarded_prompt": original, "original_bytes": len(original.encode()),
            "forwarded_bytes": len(original.encode()), "changed": False, "byte_delta": 0}


def fixture(events=None, condition="control"):
    events = [event(condition)] if events is None else events
    trace = [{"event": "activation", "schema": subject.SCHEMA, "run_id": "run", "condition": condition, "process_id": 12}]+events
    clients = []; servers = []
    for i, row in enumerate(events):
        text = row["forwarded_prompt"] if row["event"] == "prompt" else row["inline_candidate_prompt"] if row["selected_candidate"] == "inline" else row["baseline_prompt"]
        clients.append({"id": str(i), "method": "turn/start", "params": {"threadId": "thread", "input": [{"type": "text", "text": text}]}})
        servers.append({"id": str(i), "result": {"turn": {"id": "turn-"+str(i)}}})
    return trace, [{"agent_id": "agent", "thread_id": "thread"}], [{"path": "capture", "clients": clients, "servers": servers, "client_lines": [10+i*2 for i in range(len(clients))], "server_lines": [20+i*3 for i in range(len(servers))]}]


def rows():
    return [{"id": f"{b}-{i}", "block_id": b, "wave": 1+i//3+(0 if b == "a" else 2),
             "condition": c, "bound": {"lower": n, "upper": n}}
            for b in ("a", "b") for i, c in enumerate(("control", "control", subject.VARIANT, "control", subject.VARIANT, subject.VARIANT))
            for n in [100 if c == "control" else 400]]


class CandidateTests(unittest.TestCase):
    def test_project_order_matches_rust_path_components_not_string_order(self):
        e = event(body="x")
        tracked = [{"path": p, "class": "unchanged", "bytes": 1, "digest": fnv("x"), "inline_body_start": None, "inline_body_end": None} for p in ("a/z", "a-b")]
        e["snapshots"] = tracked+e["snapshots"]
        self.assertFalse(subject.validate_read(e, "control")["errors"])
        e["snapshots"][:2] = reversed(e["snapshots"][:2])
        self.assertTrue(subject.validate_read(e, "control")["errors"])

    def test_actual_v3_shapes_control_and_inline(self):
        for condition in ("control", subject.VARIANT):
            result = subject.validate_read(event(condition), condition)
            self.assertEqual(result["errors"], [])
            self.assertEqual(result["snapshot_evidence"][0]["sha256"], subject.digest(("λ"*9000).encode()))

    def test_body_range_excludes_added_newline_and_marker_data_are_not_spans(self):
        e = event(subject.VARIANT, body="work-leaf file text\n--- marker ---\n"+"λ"*9000)
        self.assertFalse(subject.validate_read(e, subject.VARIANT)["errors"])
        e["snapshots"][0]["inline_body_end"] += 1
        self.assertTrue(subject.validate_read(e, subject.VARIANT)["errors"])

    def test_below_threshold_and_failed_bundle_are_identical_candidates(self):
        for e in (event(subject.VARIANT, body="x"*16384), event(subject.VARIANT, success=False)):
            self.assertFalse(subject.validate_read(e, subject.VARIANT)["errors"])
            self.assertFalse(e["changed"])
        self.assertFalse(subject.validate_read(event(body="x"*16385), "control")["errors"])

    def test_nonfactor_prefix_and_suffix_must_be_byte_identical(self):
        e = event(subject.VARIANT, prefix="work-leaf file text\n\n--- /tmp/old ---\nold\n\n", suffix="\nRepeated file reads unchanged\nunchanged")
        e["snapshots"].append({"path": "/tmp/old", "class": "explicit-bundle", "bytes": 4, "digest": fnv("old\n"), "inline_body_start": None, "inline_body_end": None})
        self.assertFalse(subject.validate_read(e, subject.VARIANT)["errors"])
        e["inline_candidate_prompt"] = e["inline_candidate_prompt"][:-1]+"!"
        self.assertTrue(subject.validate_read(e, subject.VARIANT)["errors"])

    def test_bad_boolean_offsets_digest_selection_and_bundle_metadata_fail(self):
        for mutate in (lambda e: e["component"].update(inline_start=True),
                       lambda e: e["snapshots"][0].update(digest="fnv64:wrong"),
                       lambda e: e.update(selected_candidate="baseline"),
                       lambda e: e["bundle"].update(write_succeeded=1),
                       lambda e: e["bundle"].update(threshold_eligible=False)):
            e = event(subject.VARIANT); mutate(e)
            self.assertTrue(subject.validate_read(e, subject.VARIANT)["errors"])

    def test_v3_nonread_is_baseline_not_work_unit_treatment(self):
        e = nonread(subject.VARIANT)
        self.assertFalse(subject.validate_nonread(e)["errors"])
        e["forwarded_prompt"] += "other factor"
        self.assertTrue(subject.validate_nonread(e)["errors"])


class DeliveryTests(unittest.TestCase):
    def test_exact_typed_delivery_physical_lines_and_nonread_coverage(self):
        t, th, c = fixture([event(), nonread(sequence=2)])
        result = subject.prompt_inventory(t, th, c, "run", "control")
        self.assertFalse(result["errors"])
        self.assertEqual(result["delivered_eligible_reads"], 1)
        self.assertEqual(result["exposures"][0]["client_line"], 10)
        self.assertEqual(result["exposures"][0]["reply_line"], 20)

    def test_reverse_coverage_and_unsent_rows_remain_explicit(self):
        t, th, c = fixture()
        self.assertTrue(subject.prompt_inventory(t[:1], th, c, "run", "control")["errors"])
        c[0]["clients"] = []; c[0]["client_lines"] = []
        r = subject.prompt_inventory(t, th, c, "run", "control")
        self.assertTrue(r["errors"])
        self.assertEqual(r["exposures"][0]["status"], "prepared_without_captured_request")

    def test_rejection_is_not_exposure_and_unknown_reply_is_retained(self):
        t, th, c = fixture()
        c[0]["servers"] = [{"id": "0", "error": {"code": -1, "message": "too large"}}]
        r = subject.prompt_inventory(t, th, c, "run", "control")
        self.assertFalse(r["errors"]); self.assertEqual(r["delivered_eligible_reads"], 0)
        self.assertEqual(r["exposures"][0]["status"], "rejected_not_delivered")
        c[0]["servers"] = []; c[0]["server_lines"] = []
        r = subject.prompt_inventory(t, th, c, "run", "control")
        self.assertTrue(r["errors"]); self.assertEqual(len(r["exposures"]), 1)

    def test_mixed_reply_bool_turn_duplicate_rpc_and_additional_input_fail(self):
        for kind in ("mixed", "bool", "duplicate", "input"):
            t, th, c = fixture()
            if kind == "mixed": c[0]["servers"][0]["error"] = {"code": -1, "message": "bad"}
            if kind == "bool": c[0]["servers"][0]["result"]["turn"]["id"] = True
            if kind == "duplicate": c[0]["servers"].append(copy.deepcopy(c[0]["servers"][0])); c[0]["server_lines"].append(30)
            if kind == "input": c[0]["clients"][0]["params"]["input"].append({"type": "text", "text": "extra"})
            self.assertTrue(subject.prompt_inventory(t, th, c, "run", "control")["errors"], kind)


class PrimaryTests(unittest.TestCase):
    def test_four_global_waves_cannot_alias_across_blocks(self):
        data = rows()
        for row in data:
            if row["block_id"] == "b": row["wave"] -= 2
        self.assertEqual(subject.randomization_test(data)["status"], "unavailable")

    def test_exact_324_ties_and_positive_net_gate(self):
        r = subject.randomization_test(rows())
        self.assertEqual(r["allocations"], 324); self.assertEqual(r["p_upper"], 1/324)
        self.assertTrue(subject.passes(r))
        tied = rows()
        for row in tied: row["bound"] = {"lower": 1, "upper": 1}
        self.assertEqual(subject.randomization_test(tied)["p_upper"], 1)
        self.assertFalse(subject.passes(subject.randomization_test(tied)))

    def test_interval_p_encloses_every_compatible_exact_small_oracle(self):
        uncertain = rows()
        for i in (0, 2, 6, 8): uncertain[i]["bound"] = {"lower": 80, "upper": 420}
        interval = subject.randomization_test(uncertain)
        for values in itertools.product((80, 250, 420), repeat=4):
            exact = copy.deepcopy(uncertain)
            for i, value in zip((0, 2, 6, 8), values): exact[i]["bound"] = {"lower": value, "upper": value}
            p = subject.randomization_test(exact)["p_upper"]
            self.assertLessEqual(interval["p_lower"], p); self.assertGreaterEqual(interval["p_upper"], p)

    def test_all_fixed_rows_required_and_unknown_tail_never_zero(self):
        for bad in (None, True, -1):
            r = rows(); r[0]["bound"]["upper"] = bad
            self.assertEqual(subject.randomization_test(r)["status"], "unavailable")
        self.assertEqual(subject.randomization_test(rows()[:-1])["status"], "unavailable")
        r = rows(); r[0]["wave"] = True
        self.assertEqual(subject.randomization_test(r)["status"], "unavailable")

    def test_failed_retained_workflow_is_not_quality_filtered(self):
        r = rows(); r[0].update(launcher_exit_code=1, workflow_result="failed")
        self.assertEqual(subject.randomization_test(r)["status"], "available")


class FileTests(unittest.TestCase):
    def test_validated_scope_with_unbounded_tail_is_retained_but_not_finite(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); helper = root/"accounting_untracked_reads.py"
            helper.write_text("import hashlib\nfrom pathlib import Path\ndef audit_run(entry,frozen,sessions_root):\n    return {'run_id':entry['run_id'],'status':'validated','errors':[],'measurement':{'status':'unbounded_accounting_gap','bounds':{'raw_input_plus_output':{'lower':10,'upper':None}}},'exact_response_evidence':{},'source_sha256':{str(Path(__file__).resolve()):hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}}\n")
            frozen = {"files": [{"path": str(helper), "role": "frozen-evidence", "sha256": subject.sha(helper)}]}
            with patch.object(subject, "HERE", root):
                result = subject.measurement_for(frozen, {"run_id": "r"}, root)
            self.assertEqual(result["status"], "validated"); self.assertFalse(result["errors"])
            self.assertIsNone(result["measurement"]["bounds"]["raw_input_plus_output"]["upper"])

    def test_exact_helper_path_not_duplicate_basename_or_saved_boolean(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); helper = root/"accounting_untracked_reads.py"
            helper.write_text("def audit_run(entry, frozen, sessions_root):\n    return {'run_id':entry['run_id'],'status':'validated','errors':[],'measurement':{'bounds':{'raw_input_plus_output':{'lower':1,'upper':1}}},'exact_response_evidence':{},'source_sha256':{}}\n")
            frozen = {"files": [{"path": str(helper), "role": "frozen-evidence", "sha256": subject.sha(helper)}]}
            with patch.object(subject, "HERE", root):
                self.assertEqual(subject.measurement_for(frozen, {"run_id": "r"}, root)["status"], "unknown")
                self.assertEqual(subject.require_helper(frozen, helper), subject.sha(helper))
                wrong = copy.deepcopy(frozen); wrong["files"][0]["path"] = str(root/"other"/helper.name)
                with self.assertRaises(ValueError): subject.require_helper(wrong, helper)
                with self.assertRaises(ValueError): subject.require_helper({"files": frozen["files"]*2}, helper)

    def test_duplicate_json_keys_and_physical_lines(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/"stream"
            path.write_text('\n{"x":1}\n\n{"x":2}\n')
            values, lines = subject.records(path)
            self.assertEqual(lines, [2, 4]); self.assertEqual(len(values), 2)
            path.write_text('{"id":"a","id":"b"}\n')
            with self.assertRaises(ValueError): subject.records(path)

    def test_measurement_is_not_trusted_without_independent_adapter(self):
        result = subject.measurement_for({}, {"run_id": "r"}, Path("/not-used"))
        self.assertEqual(result["status"], "unknown")
        self.assertTrue(result["errors"])


class ManifestTests(unittest.TestCase):
    def test_complete_actual_row_schema_reaches_only_raw_primary_and_preserves_failed_run(self):
        core = subject.pinned("analyze_work_units.py"); base = subject.pinned("analyze.py")
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); helpers = root/"helpers"; helpers.mkdir()
            for name in subject.PINS: (helpers/name).write_bytes((subject.HERE/name).read_bytes())
            trust = helpers/"trust_work_units.py"
            trust.write_text("def audit(phase):\n    return {'valid':True,'errors':[],'legacy_flags_preserved':True,'self_contained_global_config_replay':False,'source_identities':[],'verified_snapshot_indices':[0],'global_snapshot_replay':{}}\n")
            accounting = helpers/"accounting_untracked_reads.py"
            accounting.write_text("import hashlib\nfrom pathlib import Path\ndef audit_run(entry, frozen, sessions_root):\n    n=100 if entry['condition']=='control' else 400\n    return {'run_id':entry['run_id'],'status':'validated','errors':[],'measurement':{'status':'exact','bounds':{'raw_input_plus_output':{'lower':n,'upper':n}}},'exact_response_evidence':{},'source_sha256':{str(Path(__file__).resolve()):hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}}\n")
            schedule = []
            for index, r in enumerate(rows()):
                rid = r["id"]; artifact = root/rid; app = artifact/"observation/app-server/app"; app.mkdir(parents=True)
                trace, threads, captures = fixture(condition=r["condition"])
                for e in trace: e["run_id"] = rid
                for filename, key in (("client-to-server.raw", "clients"), ("server-to-client.raw", "servers")):
                    (app/filename).write_text("\n"+"".join(json.dumps(e)+"\n" for e in captures[0][key]))
                prompt = artifact/"trace.jsonl"; prompt.write_text("".join(json.dumps(e)+"\n" for e in trace))
                (artifact/"observation/analysis.json").write_text(json.dumps({"threads": threads, "errors": []}))
                report = artifact/"report.json"; report.write_text(json.dumps({"workflow_result": "failed" if index == 0 else "pass"}))
                experiment = artifact/"experiment.json"; experiment.write_text(json.dumps({"schema": subject.SCHEMA, "run_id": rid, "condition": r["condition"], "evidence_path": str(prompt)}))
                schedule.append({"run_id": rid, **{k: r[k] for k in ("condition", "block_id", "wave")}, "phase": "fresh", "workflow": "work-leaf-concurrent", "artifact": str(artifact), "report": str(report), "prompt_trace": str(prompt), "experiment_manifest": str(experiment), "started_at": "started", "finished_at": "finished", "launcher_exit_code": 1 if index == 0 else 0, "launch_status": "finished"})
            files = [Path(subject.__file__).resolve(), *(helpers/n for n in subject.PINS), trust, accounting, base.GATE/"batch_analysis.py"]
            frozen = {**core.CANONICAL, "phase": "fresh", "phase_kind": "confirmation", "schedule": schedule,
                "files": [{"path": str(p), "role": "frozen-evidence", "sha256": subject.sha(p)} for p in files], "trust_classifier_sha256": subject.sha(trust),
                "launch_order": [[r["run_id"] for r in schedule if r["wave"] == w] for w in range(1, 5)],
                "randomization": {"scheme": "within_block", "unit": "workflow", "mixed_waves": True, "block_condition_counts": {b: {"control": 3, subject.VARIANT: 3} for b in ("a", "b")}}}
            phase = root/"PHASE-MANIFEST.json"; phase.write_text(json.dumps(frozen)); h = subject.sha(phase)
            admission = root/"RUN-ONCE"; admission.mkdir(); (admission/"admission.json").write_text(json.dumps({"manifest_sha256": h}))
            (root/"config-history.json").write_text(json.dumps({"snapshots": [{}]}))
            for name in ("config-history.jsonl", "trust-final.json", "trust-evidence.jsonl", "trust-pending.jsonl"): (root/name).write_text("{}\n")
            manifest = {**{k: v for k,v in frozen.items() if k not in {"files", "schedule"}}, "phase_manifest": str(phase), "phase_manifest_sha256": h, "runs": schedule}
            manifest_path = root/"score-manifest.json"; manifest_path.write_text(json.dumps(manifest))
            result = {"manifest_sha256": h, "runs": schedule, "frozen_input_integrity_errors": [], "supervisor_error": None, "signals_received": [], "supervisor_wall_timeout": False, "phase_schedule_exhausted": True, "all_scheduled_workflows_launched": True, "unexplained_config_drift_detected": False, "pending_config_attestation_at_finish": False}
            (root/"PHASE-RESULT.json").write_text(json.dumps(result))
            with patch.object(subject, "HERE", helpers), patch.object(subject, "capture_provenance", return_value={"errors": [], "source_sha256": {}}):
                checked = subject.analyze_manifest(manifest_path, root/"sessions")
            self.assertEqual(checked["integrity_errors"], [])
            self.assertEqual([r["errors"] for r in checked["observations"]], [[]]*12)
            self.assertTrue(checked["primary_passes"]); self.assertIsNone(checked["secondary_test"])
            self.assertEqual(checked["observations"][0]["workflow_result"], "failed")

    def test_every_withheld_slot_and_duplicate_unexpected_row_is_retained(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            schedule = [{"run_id": r["id"], **{k: r[k] for k in ("condition", "wave", "block_id")}} for r in rows()]
            frozen = {"schedule": schedule, "files": []}
            phase = root/"PHASE-MANIFEST.json"; phase.write_text(json.dumps(frozen))
            actual = [{**r, "launch_status": "withheld"} for r in schedule]
            actual += [dict(actual[0]), {"run_id": "unexpected", "launch_status": "failed"}]
            path = root/"score-manifest.json"
            path.write_text(json.dumps({"phase_manifest": str(phase), "phase_manifest_sha256": subject.sha(phase), "runs": actual}))
            result = subject.analyze_manifest(path, root/"sessions")
            self.assertEqual(len(result["observations"]), 14)
            self.assertEqual(result["primary"]["status"], "unavailable")
            self.assertTrue(result["integrity_errors"])

    def test_cross_workflow_response_duplicate_disables_primary(self):
        data = [{**r, "errors": [], "accounting": {"status": "validated", "errors": [],
                 "measurement": {"bounds": {"raw_input_plus_output": r["bound"]}},
                 "exact_response_evidence": {"same": {"thread_id": "thread", "turn_id": "turn"}}}} for r in rows()]
        result = subject.primary_for(data, [])
        self.assertEqual(result["status"], "unavailable")
        self.assertIn("multiple workflows", result["reason"])

    def test_phase_incident_blocks_even_with_known_raw_bounds(self):
        data = [{**r, "errors": [], "accounting": {"status": "validated", "errors": [],
                 "measurement": {"bounds": {"raw_input_plus_output": r["bound"]}},
                 "exact_response_evidence": {r["id"]: {"thread_id": "thread", "turn_id": "turn"}}}} for r in rows()]
        self.assertEqual(subject.primary_for(data, ["unexplained drift"])["status"], "unavailable")
        self.assertTrue(subject.passes(subject.primary_for(data, [])))


if __name__ == "__main__":
    unittest.main()
