from contextlib import redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest

import full_audit as a


class AggregateTests(unittest.TestCase):
    def fixture(self, partial=False, diagnostic=False):
        temp = tempfile.TemporaryDirectory(prefix="wl-full-audit-fixture.")
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        batch, sessions, output = root/"phase", root/"sessions", root/"output"
        sessions.mkdir()
        run = "fixture"
        artifact = batch/"runs"/run/(run+"-three-feature-sequential-bench-artifacts")
        observation = artifact/"observation"
        observation.mkdir(parents=True)
        def save(path, value, jsonl=False):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("".join(json.dumps(x)+"\n" for x in value) if jsonl else json.dumps(value))
        def usage(i=100, c=80, o=10, r=4):
            return dict(input_tokens=i, cached_input_tokens=c, output_tokens=o,
                        reasoning_output_tokens=r, total_tokens=i+o)
        specs = [
            ("sequential-feature-1-implement", "author"),
            ("sequential-feature-1-fix-1", "author"),
            ("sequential-feature-1-review-1", "review"),
            ("linearize-plan", "integration")]
        metadata, threads, native_rows, counts = [], {}, {}, {}
        for stage, group in specs:
            thread = {"author":"11111111-1111-7111-8111-111111111111",
                      "review":"22222222-2222-7222-8222-222222222222",
                      "integration":"33333333-3333-7333-8333-333333333333"}[group]
            counts[thread] = counts.get(thread, 0)+1
            number = counts[thread]
            turn, response = "turn-"+stage, "response-"+stage
            prompt = "Required work for "+stage
            if thread not in native_rows:
                native_rows[thread] = [{"type":"session_meta","payload":{
                    "id":thread,"session_id":thread,"cwd":"/fixture/repo","cli_version":"0.153.4","model_provider":"openai"}}]
            rr = native_rows[thread]
            rr.extend([
                {"type":"event_msg","payload":{"type":"task_started","turn_id":turn}},
                {"type":"turn_context","payload":{"turn_id":turn,"cwd":"/fixture/repo","model":"gpt-5.5","effort":"xhigh"}},
                {"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":prompt}]}},
                {"type":"token_usage_record","payload":{
                    "thread_id":thread,"session_id":thread,"turn_id":turn,"response_id":response,
                    "usage":usage(),"turn_token_usage":usage(),
                    "thread_token_usage":usage(100*number,80*number,10*number,4*number)}}])
            incomplete = partial and group=="integration"
            if not incomplete:
                rr.append({"type":"event_msg","payload":{"type":"task_complete","turn_id":turn}})
            host = group=="author"
            public = artifact/"runs"/(stage+".host")/"invocation-0001"/"stdout.jsonl" if host else artifact/"runs"/(stage+".jsonl")
            prompt_path = public.parent/"prompt.txt" if host else public.with_suffix(".prompt.txt")
            prompt_path.parent.mkdir(parents=True, exist_ok=True)
            prompt_path.write_text(prompt)
            events = [{"type":"thread.started","thread_id":thread}]
            if diagnostic:
                events.append({"type":"error","message":"recovered transport diagnostic"})
            if not incomplete:
                events.append({"type":"turn.completed","usage":usage()})
            save(public, events, True)
            if host:
                save(public.parent/"request.json", {"prompt_sha256":hashlib.sha256(prompt.encode()).hexdigest()})
            prior = threads.get(thread, 0)
            threads[thread] = prior + (0 if incomplete else 1)
        for thread, rr in native_rows.items():
            native = sessions/(thread+".jsonl")
            save(native, rr, True)
            metadata.append({"thread_id":thread,"model":"gpt-5.5","effort":"xhigh",
                "source_relative_path":native.name,"source_sha256":hashlib.sha256(native.read_bytes()).hexdigest()})
        save(observation/"rollout-metadata.jsonl", metadata, True)
        save(observation/"analysis.json", {"capture_complete":not partial,"session_only_threads":[],
            "threads":[{"thread_id":t,"usage":usage(100*n,80*n,10*n,4*n)} for t,n in threads.items()]})
        save(artifact/"report.json", {"workflow_result":"fail" if partial else "pass"})
        save(batch/"logs"/(run+".exit.json"), {"launcher_exit_code":1 if partial else 0})
        for name in ("PHASE-MANIFEST.json","PHASE-RESULT.json","PROTOCOL.md"):
            save(batch/name, {})
        return batch, sessions, output, run

    def audit(self, partial=False, diagnostic=False):
        batch, sessions, output, run = self.fixture(partial, diagnostic)
        source = a.aggregate_source(batch, run, "/fixture/repo", output, sessions_root=sessions)
        capture = io.StringIO()
        with redirect_stdout(capture):
            exec(compile(source, "fixture", "exec"),
                 {"NativeCache":a.NativeCache,"public_lifecycle":a.public_lifecycle})
        return json.loads(capture.getvalue()), output

    def test_actual_core_partitions_every_response_once(self):
        result, output = self.audit()
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["native_usage"]["raw_input_plus_output"], 440)
        self.assertEqual(set(result["category_totals"]), {"initial_author","author_fix","review","integration"})
        self.assertEqual(sum(x["native_usage"]["raw_input_plus_output"] for x in result["category_totals"].values()),440)
        self.assertEqual(len(list((output/"native-core").glob("*.result.json"))),3)

    def test_partial_stage_cost_is_retained_with_unknown_tail(self):
        result, _ = self.audit(partial=True)
        self.assertTrue(result["errors"])
        self.assertTrue(result["capture_qualification"]["unknown_unfinished_tail_observed"])
        self.assertEqual(result["category_totals"]["integration"]["native_usage"]["raw_input_plus_output"],110)
        turn = next(t for t in result["turns"] if t["category"]=="integration")
        self.assertIsNone(turn["public_usage"])
        self.assertIsNone(turn["native_minus_public"])

    def test_recovered_owned_diagnostic_does_not_erase_complete_usage(self):
        result, _ = self.audit(diagnostic=True)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["native_usage"]["raw_input_plus_output"],440)


if __name__ == "__main__":
    unittest.main()
