import json
import hashlib
from pathlib import Path
import tempfile
import unittest
from types import SimpleNamespace

import recover_host


class OwnedFinalTests(unittest.TestCase):
    def setUp(self):
        self.module = recover_host.load_host("R03")
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.raw = self.root / "stdout.jsonl"
        self.final = self.root / "final.txt"
        self.final.write_text("complete")
        self.events = [
            {"type": "thread.started", "thread_id": "owned-thread"},
            {"type": "turn.started"},
            {"type": "error", "message": "arbitrary recoverable diagnostic"},
            {"type": "item.completed", "item": {
                "id": "final-item", "type": "agent_message", "text": "complete"}},
            {"type": "turn.completed", "usage": {"input_tokens": 1, "output_tokens": 1}},
        ]

    def accept(self, expected="owned-thread"):
        self.raw.write_text("".join(json.dumps(row) + "\n" for row in self.events))
        return self.module.owned_final(self.raw, self.final, expected)

    def test_recovered_diagnostic_keeps_owned_final_and_evidence(self):
        thread, final, public = self.accept()
        self.assertEqual((thread, final), ("owned-thread", "complete"))
        self.assertIn("arbitrary recoverable diagnostic", public)

    def test_real_saved_completed_turn_is_accepted_without_generation(self):
        folder = Path(__file__).resolve().parent.parent / "P03-qualified002/host/invocation-0001"
        status = json.loads((folder / "exit.json").read_text())
        self.assertEqual(status["exit_code"], 0)
        self.assertFalse(status["timed_out"])
        self.assertIsNone(status["cancelled_signal"])
        thread, final, public = self.module.owned_final(folder / "stdout.jsonl", folder / "final.txt", None)
        self.assertEqual(thread, "01a0a1f2-54aa-7983-b2d5-c80b87ce2ab9")
        self.assertEqual(final, (folder / "final.txt").read_text())
        self.assertGreater(len(public), 0)

    def test_terminal_failure_cannot_be_recovered_by_later_completion(self):
        self.events.insert(3, {"type": "turn.failed"})
        with self.assertRaises(self.module.Fatal):
            self.accept()

    def test_diagnostic_outside_active_turn_is_rejected(self):
        diagnostic = self.events.pop(2)
        for index in (0, len(self.events)):
            with self.subTest(index=index):
                self.events.insert(index, diagnostic)
                with self.assertRaises(self.module.Fatal):
                    self.accept()
                self.events.pop(index)

    def test_missing_or_duplicate_completion_rejected(self):
        good = list(self.events)
        for events in (good[:-1], good + [good[-1]]):
            with self.subTest(events=events):
                self.events = events
                with self.assertRaises(self.module.Fatal):
                    self.accept()

    def test_wrong_thread_mismatched_final_and_active_item_rejected(self):
        with self.assertRaises(self.module.Fatal):
            self.accept("foreign-thread")
        self.final.write_text("different")
        with self.assertRaises(self.module.Fatal):
            self.accept()
        self.final.write_text("complete")
        self.events.insert(2, {"type": "item.started", "item": {"id": "unfinished"}})
        with self.assertRaises(self.module.Fatal):
            self.accept()

    def test_duplicate_identity_and_thread_rejected(self):
        good = list(self.events)
        for events in (good[:1] + good, good[:4] + [good[3]] + good[4:]):
            self.events = events
            with self.assertRaises(self.module.Fatal):
                self.accept()

    def seed(self):
        self.accept()
        request = {"expected_thread": None, "prompt_sha256": hashlib.sha256(b"launch").hexdigest()}
        result = {"initial_head": "base", "final_head": "base", "clean": True,
                  "completed": False, "pending_changes": [], "accepted_commits": [],
                  "unprocessed_replies": [], "invocations": [request],
                  "repo": str(self.root), "feature": "f", "stage": "author",
                  "serialized_feedback": True}
        values = {
            "request": request, "result": result,
            "exit": {"exit_code": 0, "timed_out": False, "cancelled_signal": None},
        }
        files = {"raw": str(self.raw), "final": str(self.final)}
        for name, value in values.items():
            path = self.root / (name + ".json")
            path.write_text(json.dumps(value))
            files[name] = str(path)
        events = self.root / "events.jsonl"
        events.write_text("")
        files["events"] = str(events)
        native = self.root / "native.jsonl"
        native.write_bytes(b"retained native prefix\n")
        seed = {"schema": 1, "files": files,
                "sha256": {name: hashlib.sha256(Path(path).read_bytes()).hexdigest()
                           for name, path in files.items()},
                "native_path": str(native), "native_prefix_bytes": native.stat().st_size,
                "native_prefix_sha256": hashlib.sha256(native.read_bytes()).hexdigest(),
                "claim_path": str(self.root / "claim.json")}
        evidence, events = [], []
        host = SimpleNamespace(
            repo=self.root, artifacts=self.root, initial_head="base", invocations=[],
            thread=None, feature="f", stage="author", serialized_feedback=True,
            unchanged=lambda: None, evidence=lambda *args: evidence.append(args),
            event=lambda *args, **kwargs: events.append((args, kwargs)))
        return seed, host, evidence, events

    def test_seed_reuses_exact_final_and_boundary_once(self):
        seed, host, evidence, events = self.seed()
        final = recover_host.recover_first(self.module, host, "launch", seed)
        self.assertEqual(final, "complete")
        self.assertEqual(host.thread, "owned-thread")
        self.assertEqual(len(host.invocations), 1)
        self.module._reference_cursor.enter(2, host.thread)
        self.assertTrue(evidence)
        self.assertEqual(events[-1][1]["invocation"], 1)
        with self.assertRaises((ValueError, self.module.Fatal)):
            recover_host.recover_first(self.module, host, "launch", seed)

    def test_seed_rejects_nonzero_exit_before_claim(self):
        seed, host, _, _ = self.seed()
        path = Path(seed["files"]["exit"])
        value = json.loads(path.read_text())
        value["exit_code"] = 1
        path.write_text(json.dumps(value))
        seed["sha256"]["exit"] = hashlib.sha256(path.read_bytes()).hexdigest()
        with self.assertRaises((ValueError, self.module.Fatal)):
            recover_host.recover_first(self.module, host, "launch", seed)
        self.assertFalse(Path(seed["claim_path"]).exists())

    def test_seed_rejects_changed_prompt_native_or_consumed_state(self):
        seed, host, _, _ = self.seed()
        for prompt in ("different",):
            with self.assertRaises((ValueError, self.module.Fatal)):
                recover_host.recover_first(self.module, host, prompt, seed)
        Path(seed["native_path"]).write_bytes(b"other native state")
        with self.assertRaises((ValueError, self.module.Fatal)):
            recover_host.recover_first(self.module, host, "launch", seed)
        self.assertFalse(Path(seed["claim_path"]).exists())

    def test_seed_rejects_recorded_operation_or_hash_drift(self):
        seed, host, _, _ = self.seed()
        path = Path(seed["files"]["events"])
        path.write_text('{"kind":"edit_accepted"}\n')
        with self.assertRaises((ValueError, self.module.Fatal)):
            recover_host.recover_first(self.module, host, "launch", seed)
        seed["sha256"]["events"] = hashlib.sha256(path.read_bytes()).hexdigest()
        with self.assertRaises((ValueError, self.module.Fatal)):
            recover_host.recover_first(self.module, host, "launch", seed)
        self.assertFalse(Path(seed["claim_path"]).exists())


if __name__ == "__main__":
    unittest.main()
