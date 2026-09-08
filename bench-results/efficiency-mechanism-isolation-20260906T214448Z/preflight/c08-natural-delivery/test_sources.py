"""Finite temporary-source wrapper tests; no actual workflow payloads."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import patch

import audit_automatic_refresh_sources as wrapper


HERE = Path(__file__).resolve().parent
STUDY = HERE.parents[1]
FRAME = (HERE.parent / "review-evidence-native-diagnostic-001" / "postcapture" /
         "observer-frame-correction" / "audit_observer_frames.py")
OLD = (FRAME.parents[2] / "infrastructure" / "evidence" / "bench-results" /
       "efficiency-mechanism-isolation-20260906T214448Z")
PINS = {
    STUDY / "audit_review_evidence_sources.py": "d3fd8c80746bf4bce565cb5f0a2e2eab29681b3aa40f89196cf95ebf344ed9ee",
    FRAME: "ccfb4cc3fe2a24c6496f4a749e6c95f147d6b5fefa7fd4e91dedc6f40be1961f",
    OLD / "audit_review_evidence_sources.py": "d3fd8c80746bf4bce565cb5f0a2e2eab29681b3aa40f89196cf95ebf344ed9ee",
    OLD / "audit_review_evidence.py": "34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257",
}


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False,
                      separators=(",", ":"), allow_nan=False).encode()


def rows_raw(rows):
    return b"".join(canonical(row) + b"\n" for row in rows)


def code(path):
    raw = path.read_bytes()
    if digest(raw) != PINS[path]:
        raise AssertionError("static dependency pin differs")
    module = types.ModuleType("fixture_exact_" + path.stem)
    module.__file__ = str(path)
    exec(compile(raw, str(path), "exec"), module.__dict__)
    return module


class TemporarySources(unittest.TestCase):
    """Actual old source/frame functions over independent temporary bytes."""

    @classmethod
    def setUpClass(cls):
        # Only pinned static dependency code is loaded from outside the fixture.
        for path, expected in PINS.items():
            if digest(path.read_bytes()) != expected:
                raise AssertionError("static dependency pin differs")
        cls.collector = code(STUDY / "audit_review_evidence_sources.py")
        cls.frame = code(FRAME)

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="c08-source-unit-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.observation = self.root / "observation"

    def put(self, path, value, raw=False):
        path.parent.mkdir(parents=True, exist_ok=True)
        data = value if raw else canonical(value)
        path.write_bytes(data)
        return {"path": str(path), "sha256": digest(data)}

    def fixture(self):
        capture = self.observation / "app-server" / "owned-capture"
        invocation = self.observation / "invocations" / capture.name
        before = [
            {"id": "init", "method": "initialize", "params": {"capabilities": {"experimentalApi": True}}},
            {"id": "thread", "method": "thread/start", "params": {"cwd": str(self.root)}},
            {"id": 1, "method": "turn/start", "params": {"threadId": "owned-thread", "input": [{"type": "text", "text": "synthetic input"}]}}
        ]
        after = copy.deepcopy(before)
        after[0]["params"]["capabilities"]["optOutNotificationMethods"] = ["rawResponseItem/completed"]
        after[1]["params"]["experimentalRawEvents"] = True
        journal = []
        for old, new in zip(before[:2], after[:2]):
            old_raw, new_raw = rows_raw([old]), rows_raw([new])
            journal.append(dict(method=old["method"], id=old["id"], changed=True,
                original_sha256=digest(old_raw), forwarded_sha256=digest(new_raw),
                original_bytes=len(old_raw), forwarded_bytes=len(new_raw), observed_monotonic_ns=15))
        servers = [
            {"id": "thread", "result": {"thread": {"id": "owned-thread"}}},
            {"id": 1, "result": {"turn": {"id": "owned-turn"}}},
            {"method": "item/completed", "params": {"threadId": "owned-thread", "turnId": "owned-turn",
             "item": {"id": "public-user", "type": "userMessage", "content": [{"type": "text", "text": "synthetic input", "text_elements": []}]}}},
            {"method": "turn/completed", "params": {"threadId": "owned-thread", "turn": {"id": "owned-turn", "status": "completed"}}}
        ]
        cap = {"path": str(capture)}
        for name, filename, rows in (("clients", "client-to-server.raw", before),
                                     ("forwarded", "client-to-server.forwarded.raw", after),
                                     ("servers", "server-to-client.raw", servers)):
            cap[name] = self.put(capture / filename, rows_raw(rows), raw=True)
        cap["settings"] = self.put(capture / "raw-response-usage.json", self.frame.SETTINGS)
        cap["journal"] = self.put(capture / "raw-response-rewrites.jsonl", rows_raw(journal), raw=True)
        cap["grace"] = self.put(capture / "provider-usage-grace.jsonl", b"", raw=True)
        start = dict(invocation_id=capture.name, capture_kind="app-server", primary=True,
                     start_unix_ns=10, start_monotonic_ns=10, raw_response_usage=True,
                     provider_usage_grace_ms=1000, provider_usage_grace_output_resume="forward")
        cap["start"] = self.put(invocation / "start.json", start)
        cap["child"] = self.put(invocation / "child.json", {"invocation_id": capture.name, "pid": 42, "started_monotonic_ns": 12})
        end = dict(invocation_id=capture.name, end_unix_ns=20, end_monotonic_ns=20, exit_code=0, terminating_signal=None,
            stdin_sha256=cap["clients"]["sha256"], stdout_sha256=cap["servers"]["sha256"],
            raw_response_usage_start_sha256=cap["start"]["sha256"],
            raw_response_usage_sha256={Path(cap[key]["path"]).name: cap[key]["sha256"]
                                       for key in ("forwarded", "settings", "journal", "grace")})
        cap["end"] = self.put(invocation / "end.json", end)
        logged = dict({key: value for key, value in start.items() if key not in ("raw_response_usage", "project_layer_inventory_required")}, end=end)
        return dict(invocations=self.put(self.observation / "process-invocations.jsonl", rows_raw([logged]), raw=True),
                    captures=[cap], invocation_metadata=[{key: cap[key] for key in ("start", "child", "end")}])

    def collect(self, manifest):
        # Private unit seam only: the eventual CLI must load its own pinned
        # dependencies and cannot accept a JSON-supplied module/proof object.
        return wrapper.collect_closed_captures(manifest, self.collector.Sources(), self.collector, self.frame)

    def seal(self, manifest):
        """Rebind synthetic end/inventory hashes so a semantic guard is tested."""
        cap = manifest["captures"][0]
        start = json.loads(Path(cap["start"]["path"]).read_bytes())
        end = json.loads(Path(cap["end"]["path"]).read_bytes())
        end.update(stdin_sha256=cap["clients"]["sha256"], stdout_sha256=cap["servers"]["sha256"])
        for key in ("forwarded", "settings", "journal", "grace"):
            end["raw_response_usage_sha256"][Path(cap[key]["path"]).name] = cap[key]["sha256"]
        cap["end"] = self.put(Path(cap["end"]["path"]), end)
        manifest["invocation_metadata"][0]["end"] = cap["end"]
        logged = dict({key: value for key, value in start.items() if key not in ("raw_response_usage", "project_layer_inventory_required")}, end=end)
        manifest["invocations"] = self.put(Path(manifest["invocations"]["path"]), rows_raw([logged]), raw=True)

    def test_exact_old_collection_and_actual_frame_proof_both_execute(self):
        manifest = self.fixture()
        with patch.object(self.collector, "collect_captures", wraps=self.collector.collect_captures) as collect:
            with patch.object(self.frame, "prove_frames", wraps=self.frame.prove_frames) as prove:
                result = self.collect(manifest)
        self.assertEqual(result["errors"], [])
        self.assertEqual(collect.call_count, 1)
        self.assertEqual(prove.call_count, 1)
        self.assertEqual(result["frames"][0]["frame_count"], 3)

    def test_an_extra_actual_capture_directory_is_not_hidden_by_declared_rows(self):
        manifest = self.fixture()
        (self.observation / "app-server" / "undeclared").mkdir()
        self.assertTrue(self.collect(manifest)["errors"])

    def test_an_extra_nonapp_invocation_directory_is_not_hidden(self):
        manifest = self.fixture()
        (self.observation / "invocations" / "undeclared-shell").mkdir()
        self.assertTrue(self.collect(manifest)["errors"])

    def test_declared_capture_missing_from_input_is_retained_as_failure(self):
        manifest = self.fixture(); manifest["captures"] = []
        self.assertTrue(self.collect(manifest)["errors"])

    def test_saved_proof_does_not_replace_missing_exact_rewrite_witness(self):
        manifest = self.fixture()
        manifest["saved_frame_proof"] = {"valid": True, "errors": []}
        cap = manifest["captures"][0]
        cap["journal"] = self.put(Path(cap["journal"]["path"]), b"", raw=True)
        self.seal(manifest)
        with patch.object(self.frame, "prove_frames", wraps=self.frame.prove_frames) as prove:
            self.assertTrue(self.collect(manifest)["errors"])
        self.assertEqual(prove.call_count, 1)

    def test_nonmetadata_mutation_is_not_normalized(self):
        manifest = self.fixture(); cap = manifest["captures"][0]
        data = Path(cap["forwarded"]["path"]).read_bytes()
        cap["forwarded"] = self.put(Path(cap["forwarded"]["path"]), data.replace(b"synthetic input", b"mutated input"), raw=True)
        self.seal(manifest)
        with patch.object(self.frame, "prove_frames", wraps=self.frame.prove_frames) as prove:
            self.assertTrue(self.collect(manifest)["errors"])
        self.assertEqual(prove.call_count, 1)

    def test_closed_incomplete_tail_and_duplicate_json_keys_reject(self):
        for bad in (b'{"id":1}', b'{"id":1,"id":2}\n'):
            with self.subTest(case=digest(bad)):
                manifest = self.fixture(); cap = manifest["captures"][0]
                cap["servers"] = self.put(Path(cap["servers"]["path"]), bad, raw=True)
                self.seal(manifest)
                self.assertTrue(self.collect(manifest)["errors"])

    def test_sources_bound_size_alias_and_endpoints_without_clipping(self):
        ref = self.put(self.root / "owned", b"abcd", raw=True)
        with patch.object(self.collector, "MAX_FILE", 3):
            with self.assertRaises(ValueError): self.collector.Sources().read(ref)
        sources = self.collector.Sources(); sources.read(ref)
        (self.root / "owned").write_bytes(b"drift")
        with self.assertRaises(ValueError): sources.finish()
        alias = self.root / "alias"; alias.symlink_to(self.root / "owned")
        with self.assertRaises(ValueError): self.collector.Sources().read({"path": str(alias), "sha256": digest(b"drift")})

    def test_nonzero_terminal_is_not_rewritten_as_missing_or_successful(self):
        row = dict(run_id="fixture-run", id="fixture-run", condition="automatic-changed-refresh-full",
            launch_status="completed", launcher_exit_code=1,
            started_at="2026-01-01T00:00:00+00:00", finished_at="2026-01-01T00:00:01+00:00")
        self.assertEqual(self.collector.terminal(row, row["id"], row["condition"])["exit_code"], 1)

    def project_fixture(self, run="fixture-run"):
        root = self.observation / "project-layer-inventory"
        entries = [{"path": ".codex", "kind": "absent"}]
        record = dict(schema="work-leaf-project-layer-inventory-v1", study_id="fixture-study", run_id=run,
            label="pre-spawn", invocation_id="owned-capture", repository=str(self.root),
            started_monotonic_ns="10", completed_monotonic_ns="11", valid=True, errors=[],
            entries=entries, inventory_sha256=digest(canonical(entries)), matches_pre_spawn=None, config_sources=[])
        refs = dict(manifest=self.put(root / "manifest.jsonl", rows_raw([record]), raw=True),
            baseline=self.put(root / "pre-spawn-baseline.json", record), snapshots=[self.put(root / "snapshot.json", record)])
        config = dict(root=str(self.observation), study_id="fixture-study", run_id=run)
        invocations = [dict(invocation_id="owned-capture", start=dict(primary=True, capture_kind="app-server",
            project_layer_inventory_required=True, cwd=str(self.root)), child=dict(started_monotonic_ns=12))]
        return refs, config, invocations

    def test_project_snapshot_full_files_journal_baseline_and_prechild_time_match(self):
        refs, config, invocations = self.project_fixture()
        self.assertEqual(wrapper.check_project_inventory(refs, self.collector.Sources(), config, invocations)["errors"], [])
        invocations[0]["child"]["started_monotonic_ns"] = 10
        self.assertTrue(wrapper.check_project_inventory(refs, self.collector.Sources(), config, invocations)["errors"])

    def test_extra_project_file_and_false_validity_are_not_waived(self):
        refs, config, invocations = self.project_fixture()
        self.put(Path(refs["baseline"]["path"]).parent / "unlisted.json", {})
        self.assertTrue(wrapper.check_project_inventory(refs, self.collector.Sources(), config, invocations)["errors"])
        refs, config, invocations = self.project_fixture()
        Path(refs["baseline"]["path"]).parent.joinpath("unlisted.json").unlink()
        record = json.loads(Path(refs["snapshots"][0]["path"]).read_bytes()); record["valid"] = False
        refs["snapshots"][0] = self.put(Path(refs["snapshots"][0]["path"]), record)
        self.assertTrue(wrapper.check_project_inventory(refs, self.collector.Sources(), config, invocations)["errors"])

    def test_original_json_array_is_preserved_without_exporting_its_payload(self):
        ref = self.put(self.root / "original-array.json", [{"private-body": "not exported"}])
        item = {"kind": "controller-records", "format": "json", "source": ref}
        saved = wrapper.preserve_original_report(item, self.collector.Sources(), self.collector)
        self.assertEqual(saved["record_count"], 1)
        self.assertNotIn("not exported", repr(saved))
        ref = self.put(self.root / "original-flags.json", {"capture_complete": False, "errors": ["original missing-source flag"]})
        item["source"] = ref
        self.assertFalse(wrapper.preserve_original_report(item, self.collector.Sources(), self.collector)["flags"]["capture_complete"])

    def generated_profile_fixture(self, project=None, artifact=None, actual=None):
        project = project or self.root / "job" / "repo"; artifact = artifact or self.root / "artifacts"
        actual = actual or self.put(self.root / "provider" / "codex", b"synthetic-native-binary", raw=True)
        profile_path = project.parent / "codex-profile" / "codex"
        log_path = profile_path.parent / "recursive-codex-attempts.log"
        script = ("#!/usr/bin/env bash\n"
            'if [[ "${WORK_LEAF_BENCH_CODEX_ACTIVE:-}" == "1" ]]; then\n'
            "  printf 'blocked recursive Codex launch pid=%s\\n' \"$$\" >> " + str(log_path) + "\n"
            "  printf 'benchmark policy blocks recursive Codex provider launches; do not retry; continue with automated validation\\n' >&2\n"
            "  exit 86\nfi\nexport WORK_LEAF_BENCH_CODEX_ACTIVE=1\n"
            'exec ' + actual["path"] + ' -c model=\\"gpt-5.5\\" -c model_reasoning_effort=\\"xhigh\\" "$@"\n').encode()
        profile = dict(agent_model="gpt-5.5", agent_reasoning_effort="xhigh", actual_codex=actual["path"],
            actual_codex_sha256=actual["sha256"], profiled_codex=str(profile_path), profiled_codex_sha256=digest(script),
            codex_cli_version="codex-cli 0.153.4", recursive_codex_attempt_log=str(log_path))
        profile_raw = "".join(key + "=" + value + "\n" for key, value in profile.items()).encode()
        renderer = STUDY.parents[1] / "bench-agent-profile-common"
        refs = dict(renderer=dict(path=str(renderer), sha256=digest(renderer.read_bytes())),
            profile=self.put(artifact / "agent-profile.txt", profile_raw, raw=True),
            recursive_log=self.put(artifact / "recursive-codex-attempts.log", b"", raw=True))
        config = dict(real_codex=str(profile_path), real_codex_sha256=digest(script), real_codex_version="codex-cli 0.153.4")
        report = dict(profiled_codex_sha256=digest(script), actual_codex_sha256=actual["sha256"],
                      codex_cli_path=actual["path"], codex_cli_version="codex-cli 0.153.4")
        return refs, config, str(project), str(artifact), report, {actual["path"]: actual["sha256"]}, script

    def test_removed_generated_profile_is_byte_bound_without_execution(self):
        refs, config, project, artifact, report, approved, script = self.generated_profile_fixture()
        self.assertFalse(Path(config["real_codex"]).exists())
        result = wrapper.check_generated_profile(refs, self.collector.Sources(), config, project, artifact, report, approved)
        self.assertEqual(result["rendered_sha256"], digest(script))
        self.assertEqual(result["rendered_bytes"], len(script))
        self.assertEqual(result["actual_binary"]["sha256"], report["actual_codex_sha256"])
        self.assertEqual(result["recursive_attempt_bytes"], 0)

    def test_generated_profile_rejects_foreign_source_fields_quoting_and_bindings(self):
        for mutation in ("renderer", "duplicate", "quoted-path", "config", "report", "undeclared", "recursive", "path"):
            refs, config, project, artifact, report, approved, _ = self.generated_profile_fixture()
            if mutation == "renderer": refs["renderer"] = self.put(self.root / "renderer", b"foreign", raw=True)
            elif mutation in ("duplicate", "quoted-path"):
                path = Path(refs["profile"]["path"]); raw = path.read_bytes()
                raw = raw + b"agent_model=gpt-5.5\n" if mutation == "duplicate" else raw.replace(b"/provider/codex", b"/provider space/codex")
                refs["profile"] = self.put(path, raw, raw=True)
            elif mutation == "config": config["real_codex_sha256"] = "0" * 64
            elif mutation == "report": report["profiled_codex_sha256"] = "0" * 64
            elif mutation == "undeclared": approved.clear()
            elif mutation == "recursive": refs["recursive_log"] = self.put(Path(refs["recursive_log"]["path"]), b"retained attempt\n", raw=True)
            else: project = str(self.root / "foreign" / "repo")
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                wrapper.check_generated_profile(refs, self.collector.Sources(), config, project, artifact, report, approved)

    def complete_execute_fixture(self):
        """Independent closed synthetic documents; no saved workflow payload."""
        run, phase, ended, score, _ = ExecuteBoundaries().documents()
        parent = self.root / "runs" / run
        artifact = parent / (run + "-three-feature-bench-artifacts")
        self.observation = artifact / "observation"
        manifest = self.fixture(); cap = manifest["captures"][0]
        binaries = {name: self.put(self.root / "bin" / name, ("synthetic-" + name).encode(), raw=True)
                    for name in ("bench-observer", "codex", "sh")}
        generated, profile_config, _, _, profile_report, _, _ = self.generated_profile_fixture(self.root, artifact, binaries["codex"])
        start = json.loads(Path(cap["start"]["path"]).read_bytes())
        start.update(cwd=str(self.root), project_layer_inventory_required=True,
                     real_executable=profile_config["real_codex"], real_executable_sha256=profile_config["real_codex_sha256"])
        cap["start"] = self.put(Path(cap["start"]["path"]), start)
        stderr = self.put(Path(cap["path"]) / "server-stderr.raw", b"", raw=True)
        end = json.loads(Path(cap["end"]["path"]).read_bytes())
        end.update(raw_response_usage_start_sha256=cap["start"]["sha256"], stderr_sha256=stderr["sha256"])
        cap["end"] = self.put(Path(cap["end"]["path"]), end)
        metadata = manifest["invocation_metadata"][0]
        metadata.update(start=cap["start"], end=cap["end"], streams=dict(stdin=cap["clients"], stdout=cap["servers"], stderr=stderr))
        self.seal(manifest)
        end = json.loads(Path(cap["end"]["path"]).read_bytes())
        metadata["meta"] = self.put(Path(cap["path"]) / "meta.json", dict(start=start, end=end))
        trace_path = self.root / "prompt-events" / (run + ".jsonl")
        experiment_path = self.root / "experiments" / (run + ".json")
        selected = phase["schedule"][0]
        selected.update(results_dir=str(parent), artifact=str(artifact), report=str(artifact / "report.json"),
                        prompt_trace=str(trace_path), experiment_manifest=str(experiment_path))
        for rows in (ended["runs"], score["runs"]): rows[0].update(selected)
        phase.update(model="gpt-5.5", reasoning_effort="xhigh", provider_route="existing-codex-chatgpt-subscription",
                     bin_dir=str(self.root / "bin"), files=[dict(ref, role="synthetic-executable") for ref in binaries.values()])
        phase_ref = self.put(self.root / "PHASE-MANIFEST.json", phase)
        ended["manifest_sha256"] = phase_ref["sha256"]
        score.update(phase_manifest=phase_ref["path"], phase_manifest_sha256=phase_ref["sha256"])
        config = dict(root=str(parent / (".bench-artifact-publish." + "a" * 32) / "observation"),
            study_id="fixture-study", run_id=run, condition="work-leaf", model="gpt-5.5", effort="xhigh", real_codex_version="codex-cli 0.153.4",
            observer_executable=binaries["bench-observer"]["path"], observer_sha256=binaries["bench-observer"]["sha256"],
            real_codex=profile_config["real_codex"], real_codex_sha256=profile_config["real_codex_sha256"],
            real_sh=binaries["sh"]["path"], real_sh_sha256=binaries["sh"]["sha256"])
        native = [
            {"type": "session_meta", "payload": {"id": "owned-thread", "cwd": str(self.root), "cli_version": "0.153.4"}},
            {"type": "turn_context", "payload": {"turn_id": "owned-turn", "cwd": str(self.root), "model": "gpt-5.5", "effort": "xhigh"}},
            {"type": "response_item", "payload": {"type": "message", "role": "user", "id": "native-user",
                "content": [{"type": "input_text", "text": "synthetic input"}],
                "internal_chat_message_metadata_passthrough": {"turn_id": "owned-turn", "content_item_kinds": ["user.text"]}}}]
        project, _, _ = self.project_fixture(run)
        source = STUDY.parents[1]
        publication = {name: {"path": str(source / name), "sha256": digest((source / name).read_bytes())}
                       for name in ("bench-candidate-common", "bench-three-features")}
        input_path = self.root / "SOURCE-INPUT.json"; output = self.root / "SOURCE-RESULT.json"
        scope = dict(schema="work-leaf-c08-natural-source-scope-v1", run_ids=list(wrapper.RUNS), condition=wrapper.CONDITION,
            phase_manifest=phase_ref, limits=dict(wrapper.LIMITS), input_paths={key: str(self.root / (key + ".json")) for key in wrapper.RUNS},
            output_paths={key: str(self.root / (key + "-result.json")) for key in wrapper.RUNS})
        scope["input_paths"][run] = str(input_path); scope["output_paths"][run] = str(output)
        value = dict(manifest, schema="work-leaf-c08-natural-source-input-v1", helper_sha256=digest(Path(wrapper.__file__).read_bytes()),
            run_id=run, scope=self.put(self.root / "SCOPE.json", scope), phase_manifest=phase_ref,
            phase_result=self.put(self.root / "PHASE-RESULT.json", ended), score_manifest=self.put(self.root / "score-manifest.json", score),
            additional_sources=[], publication_sources=publication, generated_profile=generated,
            experiment=self.put(experiment_path, dict(schema="work-leaf-bench-experiment-v7", run_id=run, condition=wrapper.CONDITION, evidence_path=str(trace_path))),
            trace=self.put(trace_path, rows_raw([dict(event="activation", schema="work-leaf-bench-experiment-v7", condition=wrapper.CONDITION, run_id=run, process_id=17)]), raw=True),
            observer_config=self.put(self.observation / "observer-config.json", config), project_cwd=str(self.root), project_inventory=project,
            native_sessions=[dict(thread_id="owned-thread", source=self.put(self.root / "native.jsonl", rows_raw(native), raw=True))],
            original_reports=[dict(kind="benchmark-report", format="json", source=self.put(artifact / "report.json",
                dict(profile_report, run_id=run, observation=str(self.observation), capture_complete=False, errors=["retained original synthetic flag"])))])
        return self.put(input_path, value), output

    def test_complete_synthetic_execute_uses_actual_source_and_join_functions(self):
        ref, output = self.complete_execute_fixture()
        result = wrapper.execute(ref, str(output))
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["status"], "available")
        self.assertEqual(result["terminal"]["exit_code"], 1)
        self.assertEqual(result["delivery"]["inputs"][0]["status"], "joined")
        self.assertFalse(result["original_reports"][0]["flags"]["capture_complete"])
        self.assertEqual(result["witness_provider_calls"], 0)

    def test_complete_synthetic_cli_publishes_once_and_retains_full_hash(self):
        ref, output = self.complete_execute_fixture()
        args = ["--input", ref["path"], "--input-sha256", ref["sha256"], "--output", str(output)]
        with patch.object(wrapper, "finish_result", wraps=wrapper.finish_result) as finish:
            self.assertEqual(wrapper.main(args), 0)
        self.assertEqual(finish.call_args.kwargs, {"max_rows": 100000, "max_bytes": 32 * 1024**2})
        self.assertTrue(wrapper.attempt_path(output).is_file())
        published = json.loads(output.read_bytes())
        original = {key: value for key, value in published.items() if key not in ("full_result_sha256", "full_result_bytes", "metadata_list_entries")}
        self.assertEqual(published["full_result_sha256"], digest(canonical(original)))
        with self.assertRaises(ValueError): wrapper.main(args)


class NativeMembership(unittest.TestCase):
    def fixture(self):
        expected = dict(cwd="/declared/project", cli_version="fixture-cli", model="fixture-model", effort="fixture-effort")
        captures = [dict(capture_id="capture", clients=[
            (1, {"id": "a", "method": "thread/start", "params": {}}),
            (2, {"id": "b", "method": "thread/start", "params": {}}),
            (3, {"id": 1, "method": "turn/start", "params": {"threadId": "author", "input": [{"type": "text", "text": "input"}]}})],
            servers=[(1, {"id": "a", "result": {"thread": {"id": "author"}}}),
                     (2, {"id": "b", "result": {"thread": {"id": "empty-title"}}}),
                     (3, {"id": 1, "result": {"turn": {"id": "turn"}}})])]
        natives = []
        for thread in ("author", "empty-title"):
            rows = [(1, {"type": "session_meta", "payload": {"id": thread, "cwd": expected["cwd"], "cli_version": expected["cli_version"]}})]
            if thread == "author":
                rows.append((2, {"type": "turn_context", "payload": {"turn_id": "turn", "cwd": expected["cwd"], "model": expected["model"], "effort": expected["effort"]}}))
            natives.append(dict(source="source-" + thread, thread_id=thread, rows=rows))
        return captures, natives, expected

    def test_zero_turn_and_usage_less_native_threads_are_required_without_role_guess(self):
        captures, natives, expected = self.fixture()
        result = wrapper.check_native_membership(captures, natives, expected)
        self.assertEqual(result["errors"], [])
        self.assertEqual({x["thread_id"] for x in result["threads"]}, {"author", "empty-title"})
        self.assertTrue(wrapper.check_native_membership(captures, natives[:1], expected)["errors"])

    def test_native_cli_model_effort_cwd_and_explicit_turn_are_not_inferred(self):
        captures, natives, expected = self.fixture()
        for key in ("turn_id", "cwd", "model", "effort"):
            changed = copy.deepcopy(natives)
            changed[0]["rows"][1][1]["payload"].pop(key)
            self.assertTrue(wrapper.check_native_membership(captures, changed, expected)["errors"])
        changed = copy.deepcopy(natives)
        changed[0]["rows"][0][1]["payload"]["cli_version"] = "foreign-cli"
        self.assertTrue(wrapper.check_native_membership(captures, changed, expected)["errors"])

    def test_context_bool_is_not_integer_and_conflicting_replay_is_not_deduplicated(self):
        captures, natives, expected = self.fixture()
        natives[0]["rows"][1][1]["payload"]["additional_typed_field"] = 1
        duplicate = copy.deepcopy(natives[0]["rows"][1][1])
        duplicate["payload"]["additional_typed_field"] = True
        natives[0]["rows"].append((3, duplicate))
        self.assertTrue(wrapper.check_native_membership(captures, natives, expected)["errors"])

    def test_foreign_or_duplicate_native_declaration_is_explicit(self):
        captures, natives, expected = self.fixture()
        for extra in (copy.deepcopy(natives[0]), dict(source="foreign-source", thread_id="foreign", rows=[])):
            self.assertTrue(wrapper.check_native_membership(captures, natives + [extra], expected)["errors"])


class ExecuteBoundaries(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.collector = code(STUDY / "audit_review_evidence_sources.py")

    def documents(self):
        ids = [f"automatic-refresh-01-workflow-{index:03d}" for index in (1, 2, 3)]
        schedule = [dict(run_id=run, condition="automatic-changed-refresh-full", phase="automatic-refresh-01",
            block_id="automatic-refresh-01-block-01", wave=1, workflow="work-leaf",
            artifact=f"/declared/{run}/artifacts", report=f"/declared/{run}/report.json",
            prompt_trace=f"/declared/{run}/trace.jsonl", experiment_manifest=f"/declared/{run}/experiment.json") for run in ids]
        phase = dict(phase="automatic-refresh-01", schedule=schedule)
        ref = dict(path="/declared/PHASE-MANIFEST.json", sha256=digest(canonical(phase)))
        rows = [dict(row, id=row["run_id"], launch_status="completed", launcher_exit_code=1 if index == 0 else 0,
            started_at="2026-01-01T00:00:00+00:00", finished_at="2026-01-01T00:00:01+00:00") for index, row in enumerate(schedule)]
        result = dict(phase=phase["phase"], manifest_sha256=ref["sha256"], runs=rows)
        score = dict(phase=phase["phase"], phase_manifest=ref["path"], phase_manifest_sha256=ref["sha256"], runs=copy.deepcopy(rows))
        return ids[0], phase, result, score, ref

    def test_all_three_terminal_rows_match_and_failed_workflow_is_retained(self):
        args = self.documents()
        checked = wrapper.check_phase_documents(*args, self.collector)
        self.assertEqual(checked["terminal"]["exit_code"], 1)
        args[2]["runs"][2]["launch_status"] = "not_launched"
        with self.assertRaises(ValueError): wrapper.check_phase_documents(*args, self.collector)

    def test_scope_extra_or_mismatched_score_rows_reject(self):
        for field in ("extra", "identity", "score"):
            args = self.documents()
            if field == "extra": args[1]["schedule"].append(dict(args[1]["schedule"][0], run_id="replacement"))
            elif field == "identity": args[2]["runs"][0]["prompt_trace"] = "/foreign/trace"
            else: args[3]["runs"][0]["launcher_exit_code"] = 0
            with self.assertRaises(ValueError): wrapper.check_phase_documents(*args, self.collector)

    def test_full_canonical_identity_precedes_explicit_output_bounds(self):
        original = {"status": "unverifiable", "errors": ["retained"], "inputs": [{"id": "nonascii-é"}]}
        result = wrapper.finish_result(copy.deepcopy(original), max_rows=10, max_bytes=1000)
        self.assertEqual(result["full_result_sha256"], digest(canonical(original)))
        with self.assertRaises(ValueError): wrapper.finish_result(copy.deepcopy(original), max_rows=0, max_bytes=1000)
        with self.assertRaises(ValueError): wrapper.finish_result(copy.deepcopy(original), max_rows=10, max_bytes=1)
        # The published LF counts toward the actual output-byte ceiling.
        with self.assertRaises(ValueError):
            wrapper.finish_result(copy.deepcopy(original), max_rows=10, max_bytes=len(canonical(result)))

    def test_invalid_input_schema_never_reaches_source_collection(self):
        with tempfile.TemporaryDirectory(prefix="c08-execute-unit-") as directory:
            path = Path(directory).resolve() / "input.json"; path.write_bytes(b'{"schema":"foreign"}')
            ref = {"path": str(path), "sha256": digest(path.read_bytes())}
            with patch.object(wrapper, "collect_closed_captures") as capture:
                result = wrapper.execute(ref)
            self.assertTrue(result["errors"])
            capture.assert_not_called()

    def test_grace_occurrences_and_every_accepted_turn_terminal_are_required(self):
        cap = dict(capture_id="capture", clients=[
            (1, {"id": 1, "method": "turn/start", "params": {"threadId": "thread"}}),
            (2, {"id": 2, "method": "turn/interrupt", "params": {"threadId": "thread", "turnId": "turn"}})],
            servers=[(1, {"id": 1, "result": {"turn": {"id": "turn"}}}),
                (2, {"id": 2, "result": {}}),
                (3, {"method": "turn/completed", "params": {"threadId": "thread", "turn": {"id": "turn", "status": "interrupted"}}})])
        grace = [(1, dict(thread_id="thread", turn_id="turn", configured_grace_ms=1000,
            output_resume_policy="forward", waited_ms=1002, outcome="forwarded-after-timeout"))]
        self.assertEqual(wrapper.check_turn_closure(cap, grace)["errors"], [])
        self.assertTrue(wrapper.check_turn_closure(cap, [])["errors"])
        cap["servers"].pop()
        self.assertTrue(wrapper.check_turn_closure(cap, grace)["errors"])

    def test_invalid_grace_type_or_extra_occurrence_is_retained_as_error(self):
        cap = dict(capture_id="capture", clients=[], servers=[])
        grace = [(1, dict(thread_id="thread", turn_id="turn", configured_grace_ms=True,
            output_resume_policy="forward", waited_ms=0, outcome="not-eligible"))]
        self.assertTrue(wrapper.check_turn_closure(cap, grace)["errors"])

    def test_exact_publication_relocation_is_not_a_blanket_path_rewrite(self):
        run = "automatic-refresh-01-workflow-001"
        parent = Path("/declared/runs") / run
        entry = dict(run_id=run, results_dir=str(parent), artifact=str(parent / (run + "-three-feature-bench-artifacts")))
        published = Path(entry["artifact"]) / "observation"
        original = str(parent / (".bench-artifact-publish." + "a" * 32) / "observation")
        result = wrapper.check_publication_root(original, entry, published)
        self.assertEqual(result["original_observation_root"], original)
        for wrong in (original.replace("/runs/", "/foreign/"), original.replace("a" * 32, "a" * 31), original + "/other"):
            with self.assertRaises(ValueError): wrapper.check_publication_root(wrong, entry, published)

    def test_scope_requires_exact_source_and_output_limits(self):
        limits = {"max_file_bytes": 1024**3, "max_total_source_bytes": 32*1024**3, "max_sources": 20000,
                  "max_output_rows": 100000, "max_output_bytes": 32*1024**2}
        self.assertEqual(wrapper.check_limits({"limits": limits}), limits)
        for key in limits:
            wrong = dict(limits); wrong[key] = True
            with self.assertRaises(ValueError): wrapper.check_limits({"limits": wrong})

    def test_nonapp_stream_and_meta_hashes_do_not_require_project_cwd(self):
        with tempfile.TemporaryDirectory(prefix="c08-stream-unit-") as directory:
            root = Path(directory).resolve(); cap = root / "locked-commands" / "shell"; cap.mkdir(parents=True)
            streams = {}; end = {}
            for name in ("stdin", "stdout", "stderr"):
                path = cap / (name + ".raw"); path.write_bytes(name.encode())
                streams[name] = {"path": str(path), "sha256": digest(path.read_bytes())}
                end[name + "_sha256"] = streams[name]["sha256"]
            start = {"capture_kind": "locked-command", "cwd": "/project/test-created/nested", "invocation_id": "shell"}
            meta = cap / "meta.json"; meta.write_bytes(canonical({"start": start, "end": end}))
            refs = {"streams": streams, "meta": {"path": str(meta), "sha256": digest(meta.read_bytes())}}
            item = {"invocation_id": "shell", "start": start, "end": end}
            self.assertEqual(wrapper.check_invocation_streams(refs, item, root, self.collector.Sources())["errors"], [])
            streams["stdout"]["sha256"] = "0" * 64
            self.assertTrue(wrapper.check_invocation_streams(refs, item, root, self.collector.Sources())["errors"])


class PublicationReadiness(unittest.TestCase):
    def test_python_publication_and_exclusive_path_preflight_prevents_execution(self):
        with tempfile.TemporaryDirectory(prefix="c08-publication-unit-") as directory:
            root = Path(directory).resolve(); input_path = root / "input.json"; input_path.write_bytes(b"{}")
            output = root / "result.json"
            args = ["--input", str(input_path), "--input-sha256", digest(b"{}"), "--output", str(output)]
            with patch.object(wrapper, "execute") as execute:
                with patch.object(wrapper, "prepare_publication", side_effect=OSError("synthetic Python writer failure")):
                    with self.assertRaises((ValueError, OSError)): wrapper.main(args)
                execute.assert_not_called()
                self.assertFalse(wrapper.attempt_path(output).exists())
                alias = root / "input-alias.json"; alias.symlink_to(input_path)
                parent_alias = root / "parent-alias"; parent_alias.symlink_to(root, target_is_directory=True)
                invalid_paths = [
                    ["--input", str(alias), "--input-sha256", digest(b"{}"), "--output", str(output)],
                    ["--input", str(input_path), "--input-sha256", digest(b"{}"), "--output", str(root / "missing-parent" / "result.json")],
                    ["--input", str(input_path), "--input-sha256", digest(b"{}"), "--output", str(parent_alias / "result.json")],
                ]
                for invalid_args in invalid_paths:
                    with self.subTest(arguments=invalid_args):
                        with self.assertRaises((ValueError, OSError)): wrapper.main(invalid_args)
                        execute.assert_not_called()
                        self.assertFalse(output.exists() or output.is_symlink())
                        attempt = wrapper.attempt_path(output)
                        self.assertFalse(attempt.exists() or attempt.is_symlink())
                for reserved in (output, wrapper.attempt_path(output)):
                    for kind in ("file", "directory", "dangling-symlink"):
                        with self.subTest(reserved=reserved.name, kind=kind):
                            if kind == "file": reserved.write_bytes(b"retained")
                            elif kind == "directory": reserved.mkdir()
                            else: reserved.symlink_to(root / "absent-target")
                            with self.assertRaises((ValueError, OSError)): wrapper.main(args)
                            execute.assert_not_called()
                            if kind == "directory": reserved.rmdir()
                            else: reserved.unlink()

    def test_failed_publication_retains_attempt_and_cannot_repeat_execute(self):
        with tempfile.TemporaryDirectory(prefix="c08-publication-unit-") as directory:
            root = Path(directory).resolve(); input_path = root / "input.json"; input_path.write_bytes(b"{}")
            output = root / "result.json"
            args = ["--input", str(input_path), "--input-sha256", digest(b"{}"), "--output", str(output)]
            with patch.object(wrapper, "execute", return_value={"status": "unverifiable", "errors": ["synthetic-failure"]}) as execute:
                with patch.object(wrapper, "publish_result", side_effect=OSError("synthetic publication failure")):
                    with self.assertRaises(OSError): wrapper.main(args)
                self.assertTrue(wrapper.attempt_path(output).is_file())
                with self.assertRaises((ValueError, OSError)): wrapper.main(args)
                self.assertEqual(execute.call_count, 1)


if __name__ == "__main__":
    unittest.main()
