"""Provider-free regressions for prospective WL mechanism accounting."""

import unittest
from pathlib import Path
import tempfile
import json

import analyze as subject


def tail(messages=1):
    clients = [{"id": "start", "method": "turn/start", "params": {"threadId": "t"}},
               {"id": "stop", "method": "turn/interrupt", "params": {"threadId": "t", "turnId": "u"}}]
    servers = [{"id": "start", "result": {"turn": {"id": "u"}}},
               {"method": "turn/started", "params": {"threadId": "t", "turn": {"id": "u"}}}]
    for n in range(messages):
        item = {"type": "agentMessage", "id": f"item-{n}",
                "text": "@work-leaf read context" if n == messages-1 else "Inspecting context."}
        for method in ("item/started", "item/completed"):
            servers.append({"method": method, "params": {"threadId": "t", "turnId": "u", "item": item}})
    servers.extend([
        {"method": "item/started", "params": {"threadId": "t", "turnId": "u", "item": {"id": "unfinished", "type": "reasoning"}}},
        {"method": "turn/completed", "params": {"threadId": "t", "turn": {"id": "u", "status": "interrupted"}}}])
    grace = [{"thread_id": "t", "turn_id": "u", "outcome": "forwarded-after-output-resumed"}]
    return clients, servers, grace


def row(identity, condition, lower, upper=None, block="b", workflow="pass"):
    return {"id": identity, "condition": condition, "block_id": block,
            "started_at": "time", "launch_status": "completed", "workflow_result": workflow,
            "measurement": {"bounds": {metric: {"lower": lower, "upper": lower if upper is None else upper}
                                       for metric in subject.METRICS}}}


class TailTests(unittest.TestCase):
    def test_multiple_paired_assistant_items_still_allow_one_conditional_response(self):
        for count in (1, 2, 4):
            inv = subject.inventory(*tail(count))
            self.assertEqual(inv["gaps"][0]["response_count_upper"], 1)
            self.assertFalse(inv["model_call_inventory_exhaustive"])
        self.assertIsNone(subject.strict_inventory(*tail(2))["gaps"][0]["response_count_upper"])

    def test_tool_compaction_unknown_method_unpaired_or_second_directive_stay_unknown(self):
        for case in ("tool", "compaction", "unknown", "unpaired", "second-directive"):
            c, s, g = tail(2)
            if case == "unpaired":
                s.pop(2)
            elif case == "second-directive":
                s[2]["params"]["item"]["text"] = "@work-leaf done"
            else:
                value = {"method": "unfamiliar/boundary", "params": {"threadId": "t", "turnId": "u"}}
                if case != "unknown":
                    value = {"method": "item/completed", "params": {"threadId": "t", "turnId": "u",
                             "item": {"type": "commandExecution" if case == "tool" else "contextCompaction", "id": "boundary"}}}
                s.insert(4, value)
            self.assertIsNone(subject.inventory(c, s, g)["gaps"][0]["response_count_upper"], case)

    def test_conflicting_response_id_and_unknown_interrupt_are_errors(self):
        c, s, g = tail(2)
        c[-1]["params"]["turnId"] = "unknown"
        self.assertTrue(subject.inventory(c, s, g)["errors"])

    def test_prospective_policy_names_the_visibility_assumptions(self):
        self.assertEqual(subject.POLICY["per_response"]["input_upper"] +
                         subject.POLICY["per_response"]["output_upper"], 1178000)
        for word in ("hidden", "preemption", "not"):
            self.assertIn(word, subject.POLICY["assumptions"])


class ContrastTests(unittest.TestCase):
    def test_positive_difference_means_more_tokens_with_candidate_disabled(self):
        rows = [row("c", "control", 100, 120), row("v", "ack-validation-unbounded", 150, 170)]
        result = subject.contrast(rows, "ack-validation-unbounded")
        self.assertEqual(result["raw_input_plus_output"]["variant_minus_control"], {"lower": 30, "upper": 70})
        self.assertEqual(result["raw_input_plus_output"]["increase_percent"], {"lower": 25, "upper": 70})

    def test_failed_workflow_is_not_dropped(self):
        rows = [row("c", "control", 100), row("v1", "variant", 130), row("v2", "variant", 170, workflow="fail")]
        result = subject.contrast(rows, "variant")
        self.assertEqual(result["counts"], {"control": 1, "variant": 2})
        self.assertEqual(result["raw_input_plus_output"]["variant_mean"], {"lower": 150, "upper": 150})

    def test_missing_or_unlaunched_row_prevents_complete_contrast(self):
        rows = [row("c", "control", 100), row("v", "variant", 170)]
        rows[1]["measurement"]["bounds"] = None
        self.assertEqual(subject.contrast(rows, "variant")["status"], "incomplete")
        rows[1]["started_at"] = None
        self.assertEqual(subject.contrast(rows, "variant")["status"], "incomplete")

    def test_unknown_upper_never_becomes_finite_by_averaging(self):
        rows = [row("c", "control", 100), row("v", "variant", 170)]
        rows[1]["measurement"]["bounds"][subject.METRICS[0]]["upper"] = None
        result = subject.contrast(rows, "variant")
        self.assertIsNone(result[subject.METRICS[0]]["variant_minus_control"]["upper"])

    def test_screening_does_not_make_confirmatory_p_value(self):
        rows = [row("c", "control", 100), row("v", "variant", 200)]
        self.assertEqual(subject.permutation_test(rows, "variant", phase="screening", randomized=True)["status"], "exploratory_only")

    def test_exact_whole_workflow_permutation_with_ties(self):
        rows = [row("c1", "control", 1), row("c2", "control", 2), row("v1", "variant", 3), row("v2", "variant", 4)]
        result = subject.permutation_test(rows, "variant", phase="confirmation", randomized=True)
        self.assertEqual(result["allocations"], 6)
        self.assertAlmostEqual(result["p_one_sided_lower"], 1/6)
        self.assertAlmostEqual(result["p_one_sided_upper"], 1/6)

    def test_blocked_permutations_respect_actual_allocation(self):
        rows = [row("c1", "control", 1, block="a"), row("v1", "variant", 3, block="a"),
                row("c2", "control", 2, block="b"), row("v2", "variant", 4, block="b")]
        result = subject.permutation_test(rows, "variant", phase="confirmation", randomized=True)
        self.assertEqual(result["allocations"], 4)
        self.assertAlmostEqual(result["p_one_sided_upper"], .25)

    def test_interval_permutation_is_conservative_and_requires_randomization(self):
        rows = [row("c", "control", 1, 100), row("v", "variant", 2, 100)]
        result = subject.permutation_test(rows, "variant", phase="confirmation", randomized=True)
        self.assertEqual(result["p_one_sided_upper"], 1)
        self.assertEqual(subject.permutation_test(rows, "variant", phase="confirmation", randomized=False)["status"], "unavailable")

    def test_confirmation_mixed_waves_have_324_not_400_allocations(self):
        rows = []
        for block in ("a", "b"):
            for index, condition in enumerate(("control", "control", "variant", "control", "variant", "variant")):
                entry = row(f"{block}{index}", condition, 100 if condition == "control" else 200, block=block)
                entry["wave"] = f"{block}-{index//3}"
                rows.append(entry)
        result = subject.permutation_test(rows, "variant", phase="confirmation", randomized=True, mixed_waves=True)
        self.assertEqual(result["allocations"], 324)
        self.assertAlmostEqual(result["p_one_sided_upper"], 1/324)

    def test_unmixed_observed_confirmation_wave_is_not_validated(self):
        rows = [row(f"{c}{i}", c, i+100) for c in ("control", "variant") for i in range(3)]
        for entry in rows:
            entry["wave"] = entry["condition"]
        result = subject.permutation_test(rows, "variant", phase="confirmation", randomized=True, mixed_waves=True)
        self.assertEqual(result["status"], "unavailable")


class TraceTests(unittest.TestCase):
    def test_control_cannot_omit_captured_eligible_boundary_from_trace(self):
        c, s, _ = tail()
        c[0]["params"]["input"] = [{"type": "text", "text": "work-leaf patch applied\n" + subject.ACK}]
        trace = [{"event": "activation", "schema": "work-leaf-bench-experiment-v1", "run_id": "r", "condition": "control", "process_id": 8}]
        capture = {"path": "app", "clients": c, "servers": s, "inventory": subject.inventory(c, s)}
        result = subject.prompt_chains(trace, [{"agent_id": "user-1", "thread_id": "t"}], [capture], "r", "control")
        self.assertEqual(result["status"], "incomplete")
        self.assertEqual(result["unmatched_boundary_requests"], 1)

    def test_genuinely_unexposed_control_remains_valid(self):
        c, s, _ = tail()
        c[0]["params"]["input"] = [{"type": "text", "text": "ordinary task prompt"}]
        trace = [{"event": "activation", "schema": "work-leaf-bench-experiment-v1", "run_id": "r", "condition": "control", "process_id": 8}]
        capture = {"path": "app", "clients": c, "servers": s, "inventory": subject.inventory(c, s)}
        result = subject.prompt_chains(trace, [{"agent_id": "user-1", "thread_id": "t"}], [capture], "r", "control")
        self.assertEqual(result["status"], "available")
        self.assertEqual(result["unmatched_boundary_requests"], 0)

    def test_prompt_chain_matches_agent_full_text_and_rpc_not_timestamp(self):
        c, s, _ = tail()
        c[0]["params"]["input"] = [{"type": "text", "text": subject.ACK}]
        events = [{"event": "activation", "schema": "work-leaf-bench-experiment-v1", "run_id": "r", "condition": "control", "process_id": 8},
                  {"event": "prompt", "sequence": 1, "run_id": "r", "condition": "control", "process_id": 8,
                   "site": "patch-applied", "agent_id": "user-1", "original_prompt": subject.ACK, "forwarded_prompt": subject.ACK,
                   "original_bytes": len(subject.ACK), "forwarded_bytes": len(subject.ACK), "byte_delta": 0, "changed": False,
                   "cue_start": 0, "cue_end": len(subject.ACK)}]
        capture = {"path": "app", "clients": c, "servers": s, "inventory": subject.inventory(c, s)}
        result = subject.prompt_chains(events, [{"agent_id": "user-1", "thread_id": "t"}], [capture], "r", "control")
        self.assertFalse(result["errors"])
        self.assertEqual(result["exposures"][0]["turn_id"], "u")
        self.assertEqual(result["exposures"][0]["link_rule"], "same-agent exact-forwarded-text occurrence and typed RPC reply")
        self.assertEqual(result["sites"]["patch-applied"]["exposures"], 1)

    def test_trace_tampering_or_ambiguous_agent_mapping_is_not_accepted(self):
        c, s, _ = tail()
        result = subject.prompt_chains([], [], [{"path": "app", "clients": c, "servers": s, "inventory": subject.inventory(c, s)}], "r", "control")
        self.assertTrue(result["errors"])

    def test_prompt_replacement_is_validated_at_exact_utf8_byte_span(self):
        original = "é prefix " + subject.ACK + "\nstdout contains " + subject.ACK
        start = len("é prefix ".encode())
        event = {"site": "patch-applied", "original_prompt": original,
                 "forwarded_prompt": original.replace(subject.ACK, subject.UNLIMITED, 1),
                 "cue_start": start, "cue_end": start+len(subject.ACK.encode())}
        self.assertTrue(subject.valid_transform(event, "ack-validation-unlimited"))
        event["forwarded_prompt"] = original.replace(subject.ACK, subject.UNLIMITED)
        self.assertFalse(subject.valid_transform(event, "ack-validation-unlimited"))


class DesignTests(unittest.TestCase):
    def test_fixed_confirmation_design_rejects_dropped_row(self):
        rows = [row("c", "control", 1), row("v", "variant", 2)]
        manifest = {"phase_kind": "confirmation", "randomization": {
            "unit": "workflow", "scheme": "within_block", "mixed_waves": True,
            "block_condition_counts": {"b": {"control": 3, "variant": 3}}}}
        self.assertTrue(subject.design_errors(manifest, rows))

    def test_screening_is_not_mistaken_for_confirmation_by_phase_id(self):
        self.assertEqual(subject.phase_kind({"phase": "confirm-01", "phase_kind": "screening"}), "screening")
        self.assertEqual(subject.phase_kind({"phase": "confirm-01", "phase_kind": "confirmation"}), "confirmation")


class ProvenanceTests(unittest.TestCase):
    def test_metadata_replay_preserves_prompt_and_rejects_modified_turn_start(self):
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory)/"app-server"/"invocation"
            invocation = Path(directory)/"invocations"/"invocation"
            app.mkdir(parents=True)
            invocation.mkdir(parents=True)
            original = b'{"id":1,"method":"thread/start","params":{}}\n'
            forwarded = b'{"id":1,"method":"thread/start","params":{"experimentalRawEvents":true}}\n'
            (app/"client-to-server.raw").write_bytes(original)
            (app/"client-to-server.forwarded.raw").write_bytes(forwarded)
            (app/"server-to-client.raw").write_bytes(b'{}\n')
            (invocation/"start.json").write_text(json.dumps({"raw_response_usage": True, "provider_usage_grace_ms": 0}))
            (app/"raw-response-usage.json").write_text(json.dumps(subject.CAPTURE_SETTINGS))
            decision = {"method": "thread/start", "id": 1, "changed": True,
                        "original_sha256": subject.sha_bytes(original), "forwarded_sha256": subject.sha_bytes(forwarded),
                        "original_bytes": len(original), "forwarded_bytes": len(forwarded), "observed_monotonic_ns": 1}
            (app/"raw-response-rewrites.jsonl").write_text(json.dumps(decision)+"\n")

            def finish():
                (invocation/"end.json").write_text(json.dumps({
                    "stdin_sha256": subject.sha256(app/"client-to-server.raw"),
                    "stdout_sha256": subject.sha256(app/"server-to-client.raw"),
                    "raw_response_usage_start_sha256": subject.sha256(invocation/"start.json"),
                    "raw_response_usage_sha256": {name: subject.sha256(app/name) for name in
                        ("raw-response-usage.json", "raw-response-rewrites.jsonl", "client-to-server.forwarded.raw")}}))
            finish()
            self.assertFalse(subject.capture_provenance(app)["errors"])
            (app/"client-to-server.forwarded.raw").write_bytes(forwarded.replace(b'"params":{', b'"params":{"model":"changed",'))
            finish()
            self.assertTrue(subject.capture_provenance(app)["errors"])


if __name__ == "__main__":
    unittest.main()
