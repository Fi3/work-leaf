"""Provider-free checks of opaque snapshot identity and actual native retrieval."""
import copy
import hashlib
import shlex
import unittest

import audit_review_evidence as audit


class SnapshotTests(unittest.TestCase):
    def fixture(self, condition="review-evidence-native"):
        body = "Recorded context λ\nAgent-ID: copied\n--- source.rs ---\n"
        root = "/tmp/opaque-review-fixture"
        archive = {"kind": "review-source-context", "run_id": "run-1",
                   "source_agent_id": "author", "reviewer_id": "review-author",
                   "target_commit": "abc", "sequence": 1, "path": root + "/review-1.md",
                   "bytes": len(body.encode()), "digest": audit.fnv64(body.encode())}
        receipt = "Native read-only context: " + archive["path"]
        original = "prefix\n" + body + "\nsuffix"
        candidate = "prefix\n" + receipt + "\nsuffix"
        event = {"event": "review-context", "schema": "work-leaf-bench-experiment-v5",
                 "site": "review-source-context", "run_id": "run-1", "condition": condition,
                 "source_agent_id": "author", "reviewer_id": "review-author", "target_commit": "abc",
                 "original_prompt": original, "candidate_prompt": candidate,
                 "forwarded_prompt": candidate if condition.endswith("native") else original,
                 "context_start": 7, "context_end": 7 + len(body.encode()),
                 "candidate_start": 7, "candidate_end": 7 + len(receipt.encode()),
                 "selected_candidate": "candidate" if condition.endswith("native") else "baseline",
                 "archive": archive}
        return event, body.encode(), copy.deepcopy(archive), root

    def test_exact_opaque_bytes_and_both_selection_paths(self):
        for condition in ("review-evidence-native", "review-evidence-inline"):
            event, body, manifest, root = self.fixture(condition)
            result = audit.validate_snapshot(event, body, manifest, "run-1", condition, root)
            self.assertEqual(result["sha256"], hashlib.sha256(body).hexdigest())
            self.assertEqual(result["bytes"], len(body))

    def test_mismatched_body_owner_or_unowned_bytes_fail(self):
        for field in ("body", "owner", "prefix", "selection", "range", "archive"):
            event, body, manifest, root = self.fixture()
            if field == "body": body += b"x"
            if field == "owner": event["reviewer_id"] = "unrelated"
            if field == "prefix": event["candidate_prompt"] = "x" + event["candidate_prompt"]
            if field == "selection": event["forwarded_prompt"] = event["original_prompt"]
            if field == "range": event["context_end"] -= 1
            if field == "archive": manifest["sequence"] = 2
            with self.subTest(field=field), self.assertRaises(ValueError):
                audit.validate_snapshot(event, body, manifest, "run-1", "review-evidence-native", root)

    def test_boolean_and_integer_archive_identity_are_distinct(self):
        event, body, manifest, root = self.fixture()
        manifest["sequence"] = True
        with self.assertRaises(ValueError):
            audit.validate_snapshot(event, body, manifest, "run-1", "review-evidence-native", root)


class RetrievalTests(unittest.TestCase):
    BODY = b"first\nsecond\nthird\n"
    PATH = "/tmp/opaque-review-fixture/review-1.md"

    def wrapped(self, body):
        return "Chunk ID: fixture\nWall time: 0.1 seconds\nProcess exited with code 0\nOriginal token count: 12\nOutput:\n" + body.decode()

    def test_full_and_partial_exact_output(self):
        for command, output, wanted in [("cat " + self.PATH, self.BODY, "complete"),
                                         ("sed -n '2,3p' " + self.PATH, b"second\nthird\n", "verified_partial")]:
            self.assertEqual(audit.classify_read(command, self.wrapped(output), self.PATH, self.BODY)["status"], wanted)

    def test_path_mention_failure_truncation_and_wrong_output_are_not_retrieval(self):
        cases = [("printf '%s' " + self.PATH, self.wrapped(self.BODY)),
                 ("cat " + self.PATH, self.wrapped(b"wrong\n")),
                 ("cat " + self.PATH, self.wrapped(self.BODY).replace("code 0", "code 1")),
                 ("cat " + self.PATH, self.wrapped(self.BODY).replace("Output:\n", "Output:\nWarning: truncated output\n"))]
        for command, output in cases:
            with self.subTest(command=command, output=output):
                self.assertEqual(audit.classify_read(command, output, self.PATH, self.BODY)["status"], "unresolved")

    def test_original_token_count_header_is_not_a_truncation_marker(self):
        self.assertEqual(audit.classify_read("cat " + self.PATH, self.wrapped(self.BODY), self.PATH, self.BODY)["status"], "complete")

    def test_sed_lines_are_lf_delimited_not_python_splitlines(self):
        payload = b"first\rstill-first\nsecond\n"
        result = audit.classify_read("sed -n '2p' " + self.PATH, self.wrapped(b"second\n"), self.PATH, payload)
        self.assertEqual(result["status"], "verified_partial")

    def test_arbitrary_executable_named_cat_is_not_a_supported_alias(self):
        result = audit.classify_read("/tmp/custom/cat " + self.PATH, self.wrapped(self.BODY), self.PATH, self.BODY)
        self.assertEqual(result["status"], "unresolved")

    def test_empty_archive_requires_an_actual_supported_successful_read(self):
        result = audit.classify_read("cat " + self.PATH, self.wrapped(b""), self.PATH, b"")
        self.assertEqual(result["status"], "complete")

    def test_unquoted_shell_operators_and_expansions_cannot_prove_literal_access(self):
        for suffix in ("a;b", "$NAME", "a*", "a?", "a[bc]", "a{b,c}", "a)b", "a|b", "a\nb", "a`b`"):
            path = "/tmp/opaque/" + suffix
            with self.subTest(path=path):
                self.assertEqual(audit.classify_read("cat " + path, self.wrapped(self.BODY), path, self.BODY)["status"], "unresolved")
                self.assertEqual(audit.classify_read("cat '" + path + "'", self.wrapped(self.BODY), path, self.BODY)["status"], "complete")

    def test_literal_shell_wrapper_and_inner_expansion_are_distinct(self):
        for suffix in ("plain", "$NAME", "a;b", "a'b", "a)b"):
            path = "/tmp/opaque/" + suffix
            literal = "cat -- " + shlex.quote(path)
            wrapped = "/usr/bin/bash -lc " + shlex.quote(literal)
            with self.subTest(path=path):
                self.assertEqual(audit.classify_read(wrapped, self.wrapped(self.BODY), path, self.BODY)["status"], "complete")
                if suffix != "plain":
                    unsafe = "sh -c " + shlex.quote("cat " + path)
                    self.assertEqual(audit.classify_read(unsafe, self.wrapped(self.BODY), path, self.BODY)["status"], "unresolved")


class DeliveryTests(unittest.TestCase):
    def fixture(self):
        event, _, _, _ = SnapshotTests().fixture()
        event["sequence"] = 1
        selected = event["forwarded_prompt"]
        prompt = "owned policy\n\nUser prompt:\n" + selected
        policy = {"event": "prompt", "schema": event["schema"], "run_id": "run-1",
                  "condition": event["condition"], "sequence": 2, "site": "policy-injection",
                  "agent_id": "review-author", "original_prompt": prompt, "forwarded_prompt": prompt}
        request = {"id": "1", "method": "turn/start", "params": {
            "threadId": "thread-1", "input": [{"type": "text", "text": prompt}]}}
        capture = {"path": "/capture", "clients": [request], "forwarded": [copy.deepcopy(request)],
                   "servers": [{"id": "1", "result": {"turn": {"id": "turn-1"}}},
                       {"method": "item/completed", "params": {"threadId": "thread-1", "turnId": "turn-1",
                        "item": {"id": "public-user", "type": "userMessage", "content": [{"type": "text", "text": prompt}]}}}]}
        native = {"thread_id": "thread-1", "source": "/native", "rows": [
            (1, {"type": "session_meta", "payload": {"id": "thread-1"}}),
            (2, {"type": "response_item", "payload": {"id": "native-user", "type": "message", "role": "user",
                "internal_chat_message_metadata_passthrough": {"turn_id": "turn-1", "content_item_kinds": ["user.text"]},
                "content": [{"type": "input_text", "text": prompt}]}})]}
        return [event, policy], [capture], [native]

    def test_exact_public_native_join_preserves_distinct_item_ids(self):
        trace, captures, native = self.fixture()
        result = audit.join_review_inputs(trace, captures, native)
        self.assertEqual(result["accepted_inputs"], 1)
        self.assertEqual(result["reviews"][0]["public_item_id"], "public-user")
        self.assertEqual(result["reviews"][0]["native_item_id"], "native-user")

    def test_rpc_type_forwarding_native_turn_and_missing_delivery_fail(self):
        for field in ("rpc_type", "forwarding", "native_turn", "delivery"):
            trace, captures, native = self.fixture()
            if field == "rpc_type": captures[0]["servers"][0]["id"] = 1
            if field == "forwarding": captures[0]["forwarded"] = []
            if field == "native_turn": native[0]["rows"][1][1]["payload"]["internal_chat_message_metadata_passthrough"]["turn_id"] = "wrong"
            if field == "delivery": trace[0]["forwarded_prompt"] += "not delivered"
            with self.subTest(field=field), self.assertRaises(ValueError):
                audit.join_review_inputs(trace, captures, native)

    def test_malformed_present_native_turn_cannot_fall_back_to_valid_nested_turn(self):
        for bad in (False, 0, ""):
            trace, captures, native = self.fixture()
            native[0]["rows"][1][1]["payload"]["turn_id"] = bad
            with self.subTest(turn=bad), self.assertRaises(ValueError):
                audit.join_review_inputs(trace, captures, native)

    def test_boolean_forwarded_rpc_identity_is_not_integer_identity(self):
        trace, captures, native = self.fixture()
        captures[0]["clients"][0]["id"] = 1
        captures[0]["forwarded"][0]["id"] = True
        captures[0]["servers"][0]["id"] = 1
        with self.assertRaises(ValueError):
            audit.join_review_inputs(trace, captures, native)


if __name__ == "__main__":
    unittest.main()
