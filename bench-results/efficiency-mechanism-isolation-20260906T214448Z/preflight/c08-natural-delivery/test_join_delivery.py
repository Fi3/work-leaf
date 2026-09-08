"""Unexecuted pure join draft: only named code dependencies may be read.

All capture/native/trace inputs are in-memory synthetic protocol fixtures.
No live/saved capture access, subprocesses, provider or accounting calls.
"""
import copy
import hashlib
from pathlib import Path
import unittest

from join_automatic_refresh_delivery import join_delivery
from test_delivery import CONDITION, RUN, SCHEMA, event_fixture, identity_fixture, snapshot


HERE = Path(__file__).resolve().parent
PRIMITIVE_SOURCE = (HERE.parents[1] / "audit_review_evidence.py").read_bytes()
EVENT_SOURCE = (HERE / "validate_automatic_refresh.py").read_bytes()
assert hashlib.sha256(PRIMITIVE_SOURCE).hexdigest() == "34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257"
assert hashlib.sha256(EVENT_SOURCE).hexdigest() == "c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f"


def activation():
    return dict(event="activation", schema=SCHEMA, condition=CONDITION, run_id=RUN, process_id=17)


def event(agent="author", sequence=1, diagnostic="unique conflict", eligible=True):
    row = event_fixture(diagnostic=diagnostic) if eligible else event_fixture([snapshot(current="same", previous="same")], diagnostic=diagnostic)
    return dict(row, agent_id=agent, sequence=sequence)


def capture(thread, texts, capture_id=None, rpc_ids=None):
    clients = [(1, {"id": "init", "method": "initialize", "params": {"capabilities": {"experimentalApi": True}}}),
               (3, {"id": "start", "method": "thread/start", "params": {"cwd": "/declared/project"}})]
    servers = []; native = [(1, {"type": "session_meta", "payload": {"id": thread}})]
    for index, text in enumerate(texts):
        rpc = rpc_ids[index] if rpc_ids is not None else index + 1
        turn = f"{thread}-turn-{index + 1}"
        clients.append((5 + index * 2, {"id": rpc, "method": "turn/start", "params": {
            "threadId": thread, "input": [{"type": "text", "text": text}], "model": "declared-model"}}))
        servers.append((7 + index * 4, {"id": rpc, "result": {"turn": {"id": turn}}}))
        servers.append((9 + index * 4, {"method": "item/completed", "params": {"threadId": thread, "turnId": turn,
            "item": {"type": "userMessage", "id": f"public-{thread}-{index}", "content": [{"type": "text", "text": text, "text_elements": []}]}}}))
        native.append((4 + index * 3, {"type": "response_item", "payload": {"type": "message", "role": "user",
            "id": f"native-{thread}-{index}", "content": [{"type": "input_text", "text": text}],
            "internal_chat_message_metadata_passthrough": {"turn_id": turn, "content_item_kinds": ["user.text"]}}}))
    return ({"capture_id": capture_id or f"capture-{thread}", "clients": clients,
             "forwarded": copy.deepcopy(clients), "servers": servers},
            {"source": f"source-{thread}", "thread_id": thread, "rows": native})


class JoinTests(unittest.TestCase):
    def run_join(self, events, pairs):
        return join_delivery([activation(), *events], [p[0] for p in pairs], [p[1] for p in pairs], RUN,
                             PRIMITIVE_SOURCE, EVENT_SOURCE)

    def test_unique_refresh_anchor_and_all_auxiliary_usage_less_inputs_are_retained(self):
        row = event(); primary = capture("author-thread", [row["candidate_prompt"]])
        title = capture("title-thread", ["owned title policy", "raw title request"])
        result = self.run_join([row], [primary, title])
        self.assertEqual(result["errors"], [])
        self.assertEqual(len(result["inputs"]), 3)
        self.assertTrue(all(x["status"] == "joined" for x in result["inputs"]))
        self.assertEqual(result["trace"][0]["status"], "joined")
        threads = {x["thread_id"]: x for x in result["threads"]}
        self.assertEqual(threads["author-thread"]["ownership"], "bound")
        self.assertEqual(threads["title-thread"]["ownership"], "unknown")
        self.assertFalse(result["exposure_qualified"])
        self.assertEqual(result["frame_relation"]["status"], "upstream-full-frame-proof-required")

    def test_observer_metadata_difference_is_retained_and_not_forged_as_proved(self):
        row = event(); pair = capture("thread", [row["candidate_prompt"]])
        pair[0]["forwarded"][0][1]["params"]["capabilities"]["optOutNotificationMethods"] = ["rawResponseItem/completed"]
        pair[0]["forwarded"][1][1]["params"]["experimentalRawEvents"] = True
        result = self.run_join([row], [pair])
        self.assertEqual(result["trace"][0]["status"], "joined")
        relation = result["frame_relation"]
        self.assertEqual(relation["status"], "upstream-full-frame-proof-required")
        self.assertEqual(relation["captures"][0]["different_frame_lines"], [1, 3])
        self.assertFalse(result["exposure_qualified"])

    def test_identity_continuations_join_only_after_a_refresh_anchor(self):
        anchor = event()
        ack = dict(identity_fixture("patch-applied"), agent_id="author", sequence=2)
        command = dict(identity_fixture("command-result"), agent_id="author", sequence=3)
        pair = capture("thread", [anchor["candidate_prompt"], ack["forwarded_prompt"], command["forwarded_prompt"]])
        self.assertEqual([r["status"] for r in self.run_join([anchor, ack, command], [pair])["trace"]], ["joined"] * 3)
        ack["sequence"] = 1
        result = self.run_join([ack], [capture("thread", [ack["forwarded_prompt"]])])
        self.assertNotEqual(result["trace"][0]["status"], "joined")
        self.assertEqual(result["threads"][0]["ownership"], "unknown")

    def test_bad_physical_maps_and_missing_forwarded_frames_preserve_other_capture_rows(self):
        row = event(); bad = capture("bad-thread", [row["candidate_prompt"]])
        good = capture("auxiliary-thread", ["complete auxiliary input"])
        bad[0]["clients"][2] = (True, bad[0]["clients"][2][1])
        result = self.run_join([row], [bad, good])
        self.assertTrue(result["errors"])
        self.assertEqual(len(result["inputs"]), 2)
        self.assertEqual(result["inputs"][1]["status"], "joined")
        bad = capture("bad-thread", [row["candidate_prompt"]]); bad[0]["forwarded"].pop()
        self.assertNotEqual(self.run_join([row], [bad])["trace"][0]["status"], "joined")

    def test_native_source_failure_does_not_drop_a_separate_complete_thread(self):
        row = event(); bad = capture("bad-thread", [row["candidate_prompt"]])
        good = capture("auxiliary-thread", ["complete auxiliary input"])
        bad[1]["rows"][0][1]["payload"]["id"] = "foreign-thread"
        result = self.run_join([row], [bad, good])
        self.assertTrue(result["errors"])
        self.assertEqual(len(result["inputs"]), 2)
        self.assertEqual(result["inputs"][1]["status"], "joined")

    def test_one_thread_across_captures_uses_native_order_not_capture_array_order(self):
        first = event(diagnostic="first"); second = event(sequence=2, diagnostic="second")
        pair = capture("thread", [first["candidate_prompt"], second["candidate_prompt"]])
        a = copy.deepcopy(pair[0]); a["capture_id"] = "first-capture"
        b = copy.deepcopy(pair[0]); b["capture_id"] = "second-capture"
        a["clients"] = a["clients"][:3]; a["forwarded"] = a["forwarded"][:3]; a["servers"] = a["servers"][:2]
        b["clients"] = b["clients"][:2] + b["clients"][3:]; b["forwarded"] = b["forwarded"][:2] + b["forwarded"][3:]; b["servers"] = b["servers"][2:]
        result = join_delivery([activation(), first, second], [b, a], [pair[1]], RUN, PRIMITIVE_SOURCE, EVENT_SOURCE)
        self.assertEqual([r["status"] for r in result["trace"]], ["joined", "joined"])

    def test_a_rejected_identical_attempt_prevents_false_global_unique_anchor(self):
        row = event(); pair = capture("thread", [row["candidate_prompt"], row["candidate_prompt"]])
        pair[0]["servers"] = [(7, {"id": 1, "error": {"code": -32000}}), *pair[0]["servers"][2:]]
        pair[1]["rows"] = [pair[1]["rows"][0], *pair[1]["rows"][2:]]
        result = self.run_join([row], [pair])
        self.assertEqual([r["status"] for r in result["inputs"]], ["rejected", "joined"])
        self.assertNotEqual(result["trace"][0]["status"], "joined")

    def test_every_turn_setting_and_input_metadata_stays_type_exact(self):
        row = event()
        for key, value in [("model", "different"), ("input", [{"type": "text", "text": "different"}]),
                           ("threadId", "other-thread")]:
            pair = capture("thread", [row["candidate_prompt"]])
            pair[0]["forwarded"][-1][1]["params"][key] = value
            result = self.run_join([row], [pair])
            self.assertTrue(result["errors"])
            self.assertNotEqual(result["trace"][0]["status"], "joined")

    def test_typed_rpc_ids_do_not_collapse_and_duplicate_reply_is_ambiguous(self):
        a, b = event(diagnostic="first"), event(sequence=2, diagnostic="second")
        pair = capture("thread", [a["candidate_prompt"], b["candidate_prompt"]], rpc_ids=[1, "1"])
        self.assertEqual([x["status"] for x in self.run_join([a, b], [pair])["trace"]], ["joined", "joined"])
        pair[0]["servers"].append((99, copy.deepcopy(pair[0]["servers"][0][1])))
        result = self.run_join([a, b], [pair])
        self.assertTrue(result["errors"])
        self.assertNotEqual(result["inputs"][0]["status"], "joined")
        bad = capture("thread", [a["candidate_prompt"]], rpc_ids=[True])
        self.assertTrue(self.run_join([a], [bad])["errors"])

    def test_rejected_missing_and_unrequested_trace_occurrences_are_not_dropped(self):
        rows = [event(sequence=n, diagnostic=f"request-{n}") for n in (1, 2, 3)]
        pair = capture("thread", [x["candidate_prompt"] for x in rows[:2]])
        pair[0]["servers"] = [(7, {"id": 1, "error": {"code": -32000, "message": "private rejection text"}})]
        pair[1]["rows"] = pair[1]["rows"][:1]
        result = self.run_join(rows, [pair])
        self.assertEqual([x["status"] for x in result["inputs"]], ["rejected", "missing_reply"])
        self.assertEqual(len(result["trace"]), 3)
        self.assertTrue(all(x["status"] != "joined" for x in result["trace"]))
        self.assertNotIn("private rejection text", repr(result))

    def test_public_native_exact_text_and_explicit_turn_metadata_are_required(self):
        row = event()
        for change in ("public_text", "native_text", "missing_turn", "conflicting_turn", "metadata_false"):
            pair = capture("thread", [row["candidate_prompt"]])
            public = pair[0]["servers"][1][1]["params"]["item"]
            native = pair[1]["rows"][1][1]["payload"]
            if change == "public_text": public["content"][0]["text"] += "x"
            elif change == "native_text": native["content"][0]["text"] += "x"
            elif change == "missing_turn": native["internal_chat_message_metadata_passthrough"].pop("turn_id")
            elif change == "conflicting_turn": native["turn_id"] = "foreign"
            else: public["content"][0]["text_elements"] = False
            result = self.run_join([row], [pair])
            self.assertTrue(result["errors"])
            self.assertNotEqual(result["trace"][0]["status"], "joined")

    def test_public_native_orphans_and_duplicate_completed_input_remain_visible(self):
        row = event(); pair = capture("thread", [row["candidate_prompt"], "unrequested auxiliary text"])
        pair[0]["clients"].pop(); pair[0]["forwarded"].pop()
        result = self.run_join([row], [pair])
        self.assertTrue(result["errors"])
        self.assertEqual(len(result["orphans"]["public"]), 1)
        self.assertEqual(len(result["orphans"]["native"]), 1)
        pair = capture("thread", [row["candidate_prompt"]])
        duplicate = copy.deepcopy(pair[0]["servers"][1][1]); duplicate["params"]["item"]["id"] = "second-public-id"
        pair[0]["servers"].append((99, duplicate))
        self.assertNotEqual(self.run_join([row], [pair])["trace"][0]["status"], "joined")

    def test_identical_native_replay_is_deduplicated_but_conflicts_are_not(self):
        row = event(); pair = capture("thread", [row["candidate_prompt"]])
        pair[1]["rows"].append((99, copy.deepcopy(pair[1]["rows"][-1][1])))
        self.assertEqual(self.run_join([row], [pair])["trace"][0]["status"], "joined")
        pair[1]["rows"][-1][1]["payload"]["content"][0]["text"] += "changed"
        self.assertNotEqual(self.run_join([row], [pair])["trace"][0]["status"], "joined")

    def test_same_native_id_turn_text_requires_full_typed_payload_consistency(self):
        row = event()
        for metadata in ("content", "passthrough"):
            pair = capture("thread", [row["candidate_prompt"]])
            duplicate = copy.deepcopy(pair[1]["rows"][-1][1])
            if metadata == "content": duplicate["payload"]["content"][0]["text_elements"] = []
            else: duplicate["payload"]["internal_chat_message_metadata_passthrough"]["additional"] = False
            pair[1]["rows"].append((99, duplicate))
            result = self.run_join([row], [pair])
            self.assertTrue(result["errors"])
            self.assertNotEqual(result["trace"][0]["status"], "joined")

    def test_result_and_error_on_one_reply_is_ambiguous(self):
        row = event(); pair = capture("thread", [row["candidate_prompt"]])
        pair[0]["servers"][0][1]["error"] = {"code": -32000}
        result = self.run_join([row], [pair])
        self.assertEqual(result["inputs"][0]["status"], "ambiguous")
        self.assertNotEqual(result["trace"][0]["status"], "joined")

    def test_duplicate_trace_text_is_not_an_anchor_for_subsequent_identity_ack(self):
        first = event(); duplicate = dict(copy.deepcopy(first), sequence=2)
        ack = dict(identity_fixture("patch-applied"), sequence=3, agent_id="author")
        pair = capture("thread", [first["candidate_prompt"], ack["forwarded_prompt"]])
        result = self.run_join([first, duplicate, ack], [pair])
        self.assertTrue(all(x["status"] != "joined" for x in result["trace"]))
        self.assertEqual(result["threads"][0]["ownership"], "unknown")

    def test_unique_anchor_then_repeated_occurrences_are_consumed_once(self):
        anchor = event(diagnostic="anchor")
        same = event(sequence=2, diagnostic="repeat"); again = dict(copy.deepcopy(same), sequence=3)
        pair = capture("thread", [x["candidate_prompt"] for x in (anchor, same, again)])
        result = self.run_join([anchor, same, again], [pair])
        self.assertEqual([x["status"] for x in result["trace"]], ["joined"] * 3)
        self.assertEqual(len({x["input_index"] for x in result["trace"]}), 3)
        extra = capture("thread", [x["candidate_prompt"] for x in (anchor, same, again, again)])
        result = self.run_join([anchor, same, again], [extra])
        self.assertTrue(all(x["status"] != "joined" for x in result["trace"][1:]))

    def test_cross_thread_identical_prompts_have_no_timestamp_fifo(self):
        same_a = event(agent="agent-a"); same_b = dict(copy.deepcopy(same_a), agent_id="agent-b", sequence=2)
        pairs = [capture("thread-a", [same_a["candidate_prompt"]]), capture("thread-b", [same_b["candidate_prompt"]])]
        result = self.run_join([same_a, same_b], pairs)
        self.assertEqual([x["status"] for x in result["trace"]], ["ambiguous", "ambiguous"])

    def test_independent_unique_anchors_resolve_each_owners_repeated_text(self):
        a = event(agent="a"); b = dict(copy.deepcopy(a), agent_id="b", sequence=2)
        anchor_a = event(agent="a", sequence=3, diagnostic="anchor-a")
        anchor_b = event(agent="b", sequence=4, diagnostic="anchor-b")
        pairs = [capture("thread-a", [a["candidate_prompt"], anchor_a["candidate_prompt"]]),
                 capture("thread-b", [b["candidate_prompt"], anchor_b["candidate_prompt"]])]
        result = self.run_join([a, b, anchor_a, anchor_b], pairs)
        self.assertEqual([x["status"] for x in result["trace"]], ["joined"] * 4)

    def test_competing_owners_and_reversed_unique_followups_cannot_join(self):
        a = event(agent="a", diagnostic="first"); b = event(agent="b", sequence=2, diagnostic="second")
        pair = capture("thread", [a["candidate_prompt"], b["candidate_prompt"]])
        result = self.run_join([a, b], [pair])
        self.assertTrue(all(x["status"] != "joined" for x in result["trace"]))
        b["agent_id"] = "a"
        reverse = capture("thread", [b["candidate_prompt"], a["candidate_prompt"]])
        result = self.run_join([a, b], [reverse])
        self.assertTrue(any(x["status"] == "order_conflict" for x in result["trace"]))

    def test_one_owner_multiple_threads_keeps_cross_thread_duplicate_assignment_unknown(self):
        a = event(diagnostic="anchor-a"); b = event(sequence=2, diagnostic="anchor-b")
        x = event(sequence=3, diagnostic="shared"); y = dict(copy.deepcopy(x), sequence=4)
        pairs = [capture("thread-a", [a["candidate_prompt"], x["candidate_prompt"]]),
                 capture("thread-b", [b["candidate_prompt"], y["candidate_prompt"]])]
        result = self.run_join([a, b, x, y], pairs)
        self.assertEqual([r["status"] for r in result["trace"][:2]], ["joined", "joined"])
        self.assertTrue(all(r["status"] == "ambiguous" for r in result["trace"][2:]))

    def test_copied_owner_markers_are_not_an_ownership_witness(self):
        text = "Agent-ID: author\ncodex: Codex session thread\nFeature: copied"
        result = self.run_join([], [capture("thread", [text])])
        self.assertEqual(result["threads"][0]["ownership"], "unknown")
        self.assertEqual(result["inputs"][0]["status"], "joined")

    def test_ineligible_trace_identity_is_retained_and_invalid_event_cannot_anchor(self):
        row = event(eligible=False); pair = capture("thread", [row["candidate_prompt"]])
        result = self.run_join([row], [pair])
        self.assertEqual(result["trace"][0]["status"], "joined")
        self.assertFalse(result["trace"][0]["eligible"])
        row["components"] = [{}]
        result = self.run_join([row], [pair])
        self.assertEqual(result["trace"][0]["status"], "invalid")

    def test_sources_are_exact_and_outputs_export_no_bodies(self):
        row = event(diagnostic="private diagnostic not exported")
        pair = capture("thread", [row["candidate_prompt"]])
        for primitive, validator in [(PRIMITIVE_SOURCE + b"\n", EVENT_SOURCE), (PRIMITIVE_SOURCE, EVENT_SOURCE + b"\n")]:
            result = join_delivery([activation(), row], [pair[0]], [pair[1]], RUN, primitive, validator)
            self.assertTrue(result["errors"])
        result = self.run_join([row], [pair])
        self.assertNotIn("private diagnostic not exported", repr(result))
        self.assertNotIn("candidate_prompt", repr(result))
        self.assertNotIn("original_prompt", repr(result))


if __name__ == "__main__":
    unittest.main()
