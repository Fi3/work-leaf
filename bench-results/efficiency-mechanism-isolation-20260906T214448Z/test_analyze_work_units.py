"""Provider-free prospective work-unit mediator regressions; no model calls."""

import copy
import itertools
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import analyze_work_units as subject
import allocate_work_units


def event(site="patch-applied", condition="control", sequence=1, agent="user"):
    if site == "patch-applied":
        original = "work-leaf patch applied\nfiles: src/a.rs\nNext step: " + subject.ACK + "\n" + subject.C[0]
        parts = [("patch-applied-validation", subject.ACK), ("patch-applied-remaining-work", subject.C[0])]
    elif site == "command-result":
        original = "work-leaf command result\ncommand: cargo test\nstatus: 0" + subject.GUIDANCE
        parts = [("command-result-guidance", subject.GUIDANCE)]
    elif agent == "linearize":
        original = "You are running as the work-leaf linearize agent.\n\nAgent-ID: linearize\nFeature: x\n\nUser prompt:\nx"
        parts = []
    else:
        original = ("You are running under the work-leaf orchestrator.\n" + subject.A[0]
                    + "\n\nRepository instructions from the launch project:\n\nConcurrent Work Leaf interpretation:\n"
                    + "\n\nConcurrent Work Leaf translation for AGENTS.md:\n- Test requirements remain mandatory. " + subject.B[0]
                    + "\n\n--- AGENTS.md ---\nCopied cue: " + subject.B[0]
                    + "\n\nAgent-ID: user\nFeature: x\n\nUser prompt:\n" + subject.A[0])
        parts = [("policy-buildable-work-unit", subject.A[0]), ("instruction-tests-work-unit", subject.B[0])]
    spans = []
    data = original.encode(); forwarded = []; previous = 0
    for identity, old in parts:
        start = data.index(old.encode(), previous); end = start + len(old.encode())
        new = subject.SPANS[identity][1] if condition == subject.VARIANT else old
        spans.append({"id": identity, "cue_start": start, "cue_end": end, "original": old,
                      "replacement": new, "changed": new != old, "byte_delta": len(new.encode())-len(old.encode())})
        forwarded.extend([data[previous:start], new.encode()]); previous = end
    forwarded.append(data[previous:]); text = b"".join(forwarded).decode()
    return {"event": "prompt", "schema": subject.SCHEMA, "run_id": "run", "condition": condition,
            "sequence": sequence, "process_id": 12, "unix_time_ns": "123", "site": site,
            "agent_id": agent, "original_prompt": original, "forwarded_prompt": text,
            "original_bytes": len(data), "forwarded_bytes": len(text.encode()),
            "byte_delta": len(text.encode())-len(data), "changed": text != original, "spans": spans}


def fixture(events=None, condition="control"):
    events = events if events is not None else [event(condition=condition)]
    trace = [{"event": "activation", "schema": subject.SCHEMA, "run_id": "run",
              "condition": condition, "process_id": 12}] + events
    clients, servers = [], []
    for i, e in enumerate(events):
        clients.append({"id": str(i), "method": "turn/start", "params": {
            "threadId": "thread", "input": [{"type": "text", "text": e["forwarded_prompt"]}]}})
        servers.append({"id": str(i), "result": {"turn": {"id": "turn-"+str(i)}}})
    threads = [{"agent_id": events[0]["agent_id"] if events else "user", "thread_id": "thread"}]
    captures = [{"path": "capture", "clients": clients, "servers": servers}]
    return trace, threads, captures


def phase_rows():
    return [{"id": f"{b}-{i}", "condition": c, "block_id": b, "wave": (1 if b == "a" else 3)+i//3,
             "bound": {"lower": 2 if c == "control" else 6, "upper": 2 if c == "control" else 6}}
            for b in ("a", "b") for i, c in enumerate(("control", "control", subject.VARIANT,
                                                        "control", subject.VARIANT, subject.VARIANT))]


class TransformTests(unittest.TestCase):
    def test_next_step_words_in_files_receipt_do_not_shadow_owned_paragraph(self):
        e = event(); extra = "Next step: "
        e["original_prompt"] = e["original_prompt"].replace("files: src/a.rs", "files: "+extra+"src/a.rs")
        e["forwarded_prompt"] = e["original_prompt"]
        e["original_bytes"] += len(extra); e["forwarded_bytes"] += len(extra)
        for span in e["spans"]:
            span["cue_start"] += len(extra); span["cue_end"] += len(extra)
        self.assertFalse(subject.validate_transform(e, "control"))

    def test_linearizer_namespace_must_match_the_actual_renderer_role(self):
        e = event("policy-injection", agent="linearize"); e["agent_id"] = "user"
        self.assertTrue(subject.validate_transform(e, "control"))

    def test_all_sites_and_linearizer_exact_identity_and_treatment(self):
        for condition in ("control", subject.VARIANT):
            for site, agent in (("patch-applied", "user"), ("command-result", "user"),
                                ("policy-injection", "user"), ("policy-injection", "linearize")):
                self.assertEqual(subject.validate_transform(event(site, condition, agent=agent), condition), [])

    def test_omitted_policy_copy_and_undeclared_replacement_fail(self):
        e = event("policy-injection", subject.VARIANT)
        e["spans"].pop()
        self.assertTrue(subject.validate_transform(e, subject.VARIANT))
        e = event("policy-injection")
        e["spans"].pop()
        self.assertTrue(subject.validate_transform(e, "control"))

    def test_changed_data_cue_overlap_bool_offsets_or_metadata_fail(self):
        for mutate in (lambda e: e.update(forwarded_prompt=e["forwarded_prompt"]+"!"),
                       lambda e: e["spans"][0].update(cue_start=True),
                       lambda e: e["spans"][1].update(cue_start=e["spans"][0]["cue_start"]),
                       lambda e: e.update(changed=1)):
            e = event(condition=subject.VARIANT); mutate(e)
            self.assertTrue(subject.validate_transform(e, subject.VARIANT))

    def test_second_test_translation_is_required_but_copied_original_is_not(self):
        e = event("policy-injection")
        self.assertEqual(len(subject.expected_spans(e)), 2)
        e["original_prompt"] = e["original_prompt"].replace("\n\n--- AGENTS.md ---", "\n- Test requirements remain mandatory. " + subject.B[0] + "\n\n--- AGENTS.md ---")
        self.assertEqual(len(subject.expected_spans(e)), 3)


class DeliveryTests(unittest.TestCase):
    def test_mixed_success_error_and_malformed_rejection_are_unverifiable(self):
        for error in ({"code": -1, "message": "rejected"}, {}, {"code": True, "message": "x"}):
            t, th, c = fixture(); c[0]["servers"][0]["error"] = error
            result = subject.prompt_inventory(t, th, c, "run", "control")
            self.assertIsNone(result["exact_primary_count"])
        t, th, c = fixture(); c[0]["servers"] = [{"id": "0", "error": {}}]
        self.assertIsNone(subject.prompt_inventory(t, th, c, "run", "control")["exact_primary_count"])

    def test_same_prompt_distinct_turns_counts_twice_not_notification_count(self):
        t, th, c = fixture([event(sequence=1), event(sequence=2)])
        c[0]["servers"].append({"method": "turn/started", "params": {"threadId": "thread", "turn": {"id": "turn-0"}}})
        r = subject.prompt_inventory(t, th, c, "run", "control")
        self.assertFalse(r["errors"]); self.assertEqual(r["delivered_ack_count"], 2)
        self.assertEqual(len({x["ack_id"] for x in r["acks"]}), 2)

    def test_explicit_rejection_is_known_nondelivery_but_missing_reply_unknown(self):
        t, th, c = fixture(); c[0]["servers"] = [{"id": "0", "error": {"code": -1, "message": "rejected"}}]
        r = subject.prompt_inventory(t, th, c, "run", "control")
        self.assertFalse(r["errors"]); self.assertEqual(r["delivered_ack_count"], 0)
        c[0]["servers"] = []
        self.assertTrue(subject.prompt_inventory(t, th, c, "run", "control")["errors"])

    def test_accepted_ack_then_generation_failure_still_counts(self):
        t, th, c = fixture(); c[0]["servers"].append({"method": "turn/completed", "params": {"threadId": "thread", "turn": {"id": "turn-0", "status": "failed"}}})
        self.assertEqual(subject.prompt_inventory(t, th, c, "run", "control")["delivered_ack_count"], 1)

    def test_orphan_trace_and_reverse_missing_ack_or_policy_trace_fail(self):
        t, th, c = fixture(); c[0]["clients"] = []
        self.assertTrue(subject.prompt_inventory(t, th, c, "run", "control")["errors"])
        for site in ("patch-applied", "policy-injection", "command-result"):
            t, th, c = fixture([event(site)])
            self.assertTrue(subject.prompt_inventory(t[:1], th, c, "run", "control")["errors"])

    def test_bool_identity_duplicate_rpc_or_ambiguous_thread_fail(self):
        for case in ("bool", "duplicate", "ambiguous"):
            t, th, c = fixture()
            if case == "bool": c[0]["servers"][0]["result"]["turn"]["id"] = True
            elif case == "duplicate": c[0]["servers"].append(copy.deepcopy(c[0]["servers"][0]))
            else: th.append({"agent_id": "other", "thread_id": "thread"})
            self.assertTrue(subject.prompt_inventory(t, th, c, "run", "control")["errors"], case)

    def test_distinct_already_applied_prompt_not_ack_true_renderer_ack_still_counts(self):
        t, th, c = fixture(); c[0]["clients"][0]["params"]["input"][0]["text"] = "work-leaf patch already applied\nfiles: src/a.rs"
        r = subject.prompt_inventory(t[:1], th, c, "run", "control")
        self.assertFalse(r["errors"]); self.assertEqual(r["delivered_ack_count"], 0)


class ClassificationTests(unittest.TestCase):
    def test_reference_verification_failure_cannot_become_known_substantive(self):
        rows = [{"ack_id": "a", "label": "remaining-work", "rationale": "x",
                 "evidence": [{"path": "a", "sha256": "0"*64, "locator": "commit"}]}]
        def reject(ref):
            raise ValueError("hash mismatch")
        self.assertIsNone(subject.classification_bounds([{"ack_id": "a"}], rows, reject)["bounds"])

    def test_complete_disjoint_labels_and_finite_unknown_bounds(self):
        acks = [{"ack_id": "a"}, {"ack_id": "b"}, {"ack_id": "c"}]
        rows = [{"ack_id": a["ack_id"], "label": l, "rationale": "saved patch evidence",
                 "evidence": [{"path": "evidence", "sha256": "a"*64, "locator": "line 1"}]}
                for a, l in zip(acks, ("remaining-work", "no-op-replayed", "unresolved"))]
        r = subject.classification_bounds(acks, rows, lambda ref: None)
        self.assertEqual(r["bounds"], {"lower": 1, "upper": 2}); self.assertFalse(r["errors"])
        for changed in (rows[:-1], rows + [rows[0]], [dict(rows[0], ack_id=True)] + rows[1:]):
            self.assertTrue(subject.classification_bounds(acks, changed, lambda ref: None)["errors"])

    def test_missing_evidence_bad_hash_or_unsupported_label_stays_unknown(self):
        for entry in ({"ack_id": "a", "label": "remaining-work", "rationale": "x", "evidence": []},
                      {"ack_id": "a", "label": "invented", "rationale": "x", "evidence": [{}]}):
            self.assertTrue(subject.classification_bounds([{"ack_id": "a"}], [entry], lambda ref: None)["errors"])


class RandomizationTests(unittest.TestCase):
    def test_actual_allocator_integer_wave_schema_enters_exact_test(self):
        plan = allocate_work_units.make_plan("phase", "new", lambda choices: choices[0])
        rows = [{**r, "id": r["run_id"], "bound": {"lower": 1 if r["condition"] == "control" else 3, "upper": 1 if r["condition"] == "control" else 3}} for r in plan["runs"]]
        result = subject.randomization_test(rows)
        self.assertEqual(result["status"], "available"); self.assertEqual(result["allocations"], 324)
        rows[0]["wave"] = True
        self.assertEqual(subject.randomization_test(rows)["status"], "unavailable")

    def test_wrong_observed_block_counts_and_single_condition_wave_fail(self):
        rows = phase_rows(); rows[0]["condition"] = subject.VARIANT
        self.assertEqual(subject.randomization_test(rows)["status"], "unavailable")
        rows = phase_rows()
        for row in rows: row["wave"] = row["block_id"]+row["condition"]
        self.assertEqual(subject.randomization_test(rows)["status"], "unavailable")

    def test_exact_324_ties_and_invalid_partial_design(self):
        rows = phase_rows(); r = subject.randomization_test(rows)
        self.assertEqual(r["allocations"], 324); self.assertEqual(r["p_upper"], 1/324)
        for x in rows: x["bound"] = {"lower": 1, "upper": 1}
        self.assertEqual(subject.randomization_test(rows)["p_upper"], 1)
        self.assertEqual(subject.randomization_test(rows[:-1])["status"], "unavailable")

    def test_interval_p_bounds_enclose_every_compatible_exact_assignment(self):
        rows = phase_rows()
        for i in (0, 2, 6, 8): rows[i]["bound"] = {"lower": 1, "upper": 7}
        interval = subject.randomization_test(rows)
        for values in itertools.product((1, 4, 7), repeat=4):
            exact = copy.deepcopy(rows)
            for i, value in zip((0, 2, 6, 8), values): exact[i]["bound"] = {"lower": value, "upper": value}
            p = subject.randomization_test(exact)["p_upper"]
            self.assertLessEqual(interval["p_lower"], p); self.assertGreaterEqual(interval["p_upper"], p)

    def test_unknown_and_bool_bounds_cannot_enter_inference(self):
        for value in (None, True, -1):
            rows = phase_rows(); rows[0]["bound"]["upper"] = value
            self.assertEqual(subject.randomization_test(rows)["status"], "unavailable")

    def test_secondary_is_gated_not_promoted_when_primary_fails(self):
        self.assertFalse(subject.passes({"status": "available", "difference": {"lower": 1, "upper": 2}, "p_upper": .03}))
        self.assertTrue(subject.passes({"status": "available", "difference": {"lower": 1, "upper": 2}, "p_upper": 1/324}))


class ManifestTests(unittest.TestCase):
    def test_trust_replay_requires_frozen_executing_code_and_independent_invocation(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); helper = root/"trust_work_units.py"
            helper.write_text("def audit(phase):\n    return {'valid': True, 'errors': [], 'legacy_flags_preserved': True, 'self_contained_global_config_replay': False, 'source_identities': [], 'verified_snapshot_indices': [0], 'global_snapshot_replay': {'path': 'external'}}\n")
            h = subject.sha(helper); frozen = {"trust_classifier_sha256": h, "files": [{"path": str(helper), "sha256": h}]}
            result = subject.audit_trust(frozen, root, helper_path=helper)
            self.assertTrue(result["valid"]); self.assertEqual(result["helper_sha256"], h)
            self.assertFalse(subject.audit_trust({**frozen, "trust_classifier_sha256": "0"*64}, root, helper_path=helper)["valid"])
            helper.write_text("def audit(phase):\n    return {'valid': False, 'errors': ['external-final-global-snapshot-unavailable']}\n")
            h = subject.sha(helper); frozen = {"trust_classifier_sha256": h, "files": [{"path": str(helper), "sha256": h}]}
            self.assertFalse(subject.audit_trust(frozen, root, helper_path=helper)["valid"])

    def test_missing_usage_does_not_waive_observer_executable_or_other_integrity_flags(self):
        observed = {"capture_complete": False, "errors": ["interrupted provider turn has no complete usage: count=3"], "model_strata": [{"model": "gpt-5.5", "effort": "xhigh"}]}
        self.assertFalse(subject.observer_errors(observed))
        changed = copy.deepcopy(observed); changed["errors"].append("observer executable SHA-256 changed after observer initialization")
        self.assertTrue(subject.observer_errors(changed))
        changed = copy.deepcopy(observed); changed["capture_complete"] = True
        self.assertTrue(subject.observer_errors(changed))
        changed = copy.deepcopy(observed); changed["model_strata"][0]["model"] = "other"
        self.assertTrue(subject.observer_errors(changed))

    def test_recorded_phase_integrity_incidents_block_even_if_sources_match(self):
        frozen = {**subject.CANONICAL, "phase_kind": "confirmation"}
        manifest = {"phase_manifest_sha256": "a"*64, "runs": [{"run_id": "x"}]}
        result = {"manifest_sha256": "a"*64, "runs": manifest["runs"], "frozen_input_integrity_errors": [],
                  "supervisor_error": None, "signals_received": [], "supervisor_wall_timeout": False,
                  "phase_schedule_exhausted": True, "all_scheduled_workflows_launched": True, "unexplained_config_drift_detected": False, "pending_config_attestation_at_finish": False}
        self.assertFalse(subject.phase_errors(frozen, manifest, result))
        for key, bad in (("frozen_input_integrity_errors", [{"reason": "changed then restored"}]),
                         ("supervisor_error", "interrupted"), ("runs", []),
                         ("all_scheduled_workflows_launched", False), ("supervisor_wall_timeout", True),
                         ("pending_config_attestation_at_finish", True)):
            self.assertTrue(subject.phase_errors(frozen, manifest, {**result, key: bad}), key)
        self.assertTrue(subject.phase_errors({**frozen, "phase_kind": "screening"}, manifest, result))

    def test_executing_helper_must_match_a_frozen_source_identity(self):
        path = Path(subject.__file__)
        with self.assertRaises(ValueError): subject.require_frozen_helper([], path)
        with self.assertRaises(ValueError): subject.require_frozen_helper([{"path": str(path), "sha256": "0"*64}], path)
        self.assertEqual(subject.require_frozen_helper([{"path": str(path), "sha256": subject.sha(path)}], path), subject.sha(path))

    def test_bad_optional_classification_does_not_change_automatic_primary(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); phase = root/"PHASE-MANIFEST.json"; admission = root/"RUN-ONCE"; admission.mkdir()
            schedule = []
            for row in phase_rows():
                rid = row["id"]; art = root/rid; app = art/"observation/app-server/app"; app.mkdir(parents=True)
                count = 1 if row["condition"] == "control" else 4
                es = [event(sequence=i+1, condition=row["condition"]) for i in range(count)]
                trace, threads, caps = fixture(es, row["condition"])
                for e in trace: e["run_id"] = rid
                trace_path = art/"trace.jsonl"; trace_path.write_text("".join(json.dumps(e)+"\n" for e in trace))
                for filename, key in (("client-to-server.raw", "clients"), ("server-to-client.raw", "servers")):
                    (app/filename).write_text("".join(json.dumps(e)+"\n" for e in caps[0][key]))
                (art/"observation/analysis.json").write_text(json.dumps({"threads": threads, "errors": [], "capture_complete": True, "model_strata": [{"model": "gpt-5.5", "effort": "xhigh"}]}))
                report = art/"report.json"; report.write_text(json.dumps({"workflow_result": "pass", "agent_model": "gpt-5.5", "agent_reasoning_effort": "xhigh"}))
                experiment = art/"experiment.json"; experiment.write_text(json.dumps({"schema": subject.SCHEMA, "run_id": rid, "condition": row["condition"], "evidence_path": str(trace_path)}))
                schedule.append({"run_id": rid, "phase": "fresh", "condition": row["condition"], "wave": row["wave"], "block_id": row["block_id"], "workflow": "work-leaf-concurrent", "artifact": str(art), "report": str(report), "prompt_trace": str(trace_path), "experiment_manifest": str(experiment), "started_at": "start", "finished_at": "finish"})
            random = {"scheme": "within_block", "unit": "workflow", "mixed_waves": True,
                      "block_condition_counts": {b: {"control": 3, subject.VARIANT: 3} for b in ("a", "b")}}
            frozen = {**subject.CANONICAL, "files": [{"path": str(Path(subject.__file__)), "sha256": subject.sha(subject.__file__)}], "schedule": schedule, "randomization": random, "phase_kind": "confirmation"}
            phase.write_text(json.dumps(frozen)); h = subject.sha(phase)
            (admission/"admission.json").write_text(json.dumps({"manifest_sha256": h}))
            (root/"config-history.json").write_text(json.dumps({"snapshots": [{"drift_from_baseline": "unchanged"}]}))
            manifest = {**{k:v for k,v in frozen.items() if k not in {"files", "schedule"}}, "phase_manifest": str(phase), "phase_manifest_sha256": h, "runs": schedule}
            path = root/"score-manifest.json"; path.write_text(json.dumps(manifest)); labels = root/"labels.json"; labels.write_text("[]")
            (root/"PHASE-RESULT.json").write_text(json.dumps({"manifest_sha256": h, "runs": schedule,
                "frozen_input_integrity_errors": [], "supervisor_error": None, "signals_received": [],
                "supervisor_wall_timeout": False, "phase_schedule_exhausted": True, "all_scheduled_workflows_launched": True, "unexplained_config_drift_detected": False, "pending_config_attestation_at_finish": False}))
            helper = SimpleNamespace(STRICT=SimpleNamespace(records=lambda p: [json.loads(l) for l in Path(p).read_text().splitlines()]), capture_provenance=lambda p: {"source_sha256": {}, "errors": []})
            with patch.object(subject, "legacy", return_value=helper), patch.object(subject, "audit_trust", return_value={"valid": True, "errors": [], "verified_snapshot_indices": [0]}):
                result = subject.analyze_manifest(path, labels)
            self.assertTrue(result["primary_passes"])
            self.assertFalse(result["secondary_passes"])
            self.assertTrue(result["classification_errors"])
            labels.write_text(json.dumps({"schema": "work-leaf-work-unit-classification-v1", "phase_manifest_sha256": h, "runs": {"foreign-run": []}}))
            with patch.object(subject, "legacy", return_value=helper), patch.object(subject, "audit_trust", return_value={"valid": True, "errors": [], "verified_snapshot_indices": [0]}):
                result = subject.analyze_manifest(path, labels)
            self.assertTrue(result["primary_passes"])
            self.assertTrue(result["classification_errors"])

    def test_every_withheld_frozen_slot_and_unexpected_result_are_retained(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); phase = root/"PHASE-MANIFEST.json"; admission = root/"RUN-ONCE"
            admission.mkdir()
            schedule = [{"run_id": r["id"], "condition": r["condition"], "wave": r["wave"],
                         "block_id": r["block_id"]} for r in phase_rows()]
            random = {"scheme": "within_block", "unit": "workflow", "mixed_waves": True}
            frozen = {"files": [], "schedule": schedule, "randomization": random}
            phase.write_text(json.dumps(frozen)); h = subject.sha(phase)
            (admission/"admission.json").write_text(json.dumps({"manifest_sha256": h}))
            (root/"config-history.json").write_text(json.dumps({"snapshots": [{"drift_from_baseline": "unchanged"}]}))
            manifest = {"phase_manifest": str(phase), "phase_manifest_sha256": h, "randomization": random,
                        "runs": [{**r, "launch_status": "withheld"} for r in schedule]+[{"run_id": "extra", "launch_status": "failed"}]}
            path = root/"score-manifest.json"; path.write_text(json.dumps(manifest))
            with patch.object(subject, "legacy", return_value=object()):
                result = subject.analyze_manifest(path)
            self.assertEqual(len(result["observations"]), 13)
            self.assertEqual({r["id"] for r in result["observations"]}, {r["run_id"] for r in schedule}|{"extra"})
            self.assertEqual(result["primary"]["status"], "unavailable")
            self.assertIn("token_retotal", result)


if __name__ == "__main__":
    unittest.main()
