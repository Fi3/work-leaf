#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentOrchestrator, ChatMessage,
    ClaudeBackend, ClaudeCommandConfig, CodexBackend, CodexCommandConfig, MessageRole,
    PromptPolicy, ReadPermission,
};

mod temp_cleanup;

const A: &str = "Design tests before implementation when required, but submit a cohesive patch that includes the test and the implementation needed for the shared tree to build.";
const NEW_A: &str = "Design tests before implementation when required. Prefer naturally separable, independently buildable feature increments, keeping mutually dependent test and implementation changes together. Submit each completed increment and continue any remaining requested feature work. Do not split inseparable work or create otherwise unnecessary patches.";
const B: &str = "Design the needed tests, but submit tests with the implementation needed to keep the shared worktree buildable.";
const NEW_B: &str = "Design the needed tests, and prefer naturally separable, independently buildable feature increments, keeping mutually dependent tests and implementation together. Do not split inseparable work or create otherwise unnecessary patches.";
const C: &str = "After the focused validation passes, or after you report an external blocker, emit a top-level `@work-leaf done` so review can start. Send another edit only if validation found a concrete issue in your own patch.";
const NEW_C: &str = "After the focused validation passes, or after you report an external blocker, continue with another independently buildable increment if requested feature work remains and can proceed without taking over another agent's work. Otherwise emit a top-level `@work-leaf done` so review can start. Send another edit only to implement remaining requested feature work or to repair a concrete issue that validation found in your own patch.";
const ACK: &str = "run at most one focused validation step that is relevant to files you touched or checks you added.";
const VARIANT: &str = "buildable-work-unit-incremental";

fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "work-leaf-work-unit-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    temp_cleanup::register(&path);
    path
}

fn read_rows(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn child(root: &Path, schema: Option<&str>, condition: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "work_unit_child", "--ignored", "--nocapture"])
        .env("WORK_UNIT_TEST_ROOT", root)
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT")
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST")
        .env_remove("WORK_LEAF_BENCH_RUN_ID");
    if let Some(schema) = schema {
        let path = root.join("manifest.json");
        fs::write(
            &path,
            serde_json::to_vec(&json!({
                "schema": schema, "condition": condition, "run_id": "work-unit-fixture",
                "evidence_path": root.join("evidence.jsonl")
            }))
            .unwrap(),
        )
        .unwrap();
        command
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", path)
            .env("WORK_LEAF_BENCH_RUN_ID", "work-unit-fixture");
    }
    command
}

fn exercise(schema: Option<&str>, condition: &str) -> (Vec<Value>, Vec<Value>) {
    let root = root();
    let output = child(&root, schema, condition).output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (
        serde_json::from_slice(&fs::read(root.join("delivered.json")).unwrap()).unwrap(),
        read_rows(&root.join("evidence.jsonl")),
    )
}

#[test]
fn work_unit_control_and_default_preserve_every_provider_boundary() {
    let (baseline, no_trace) = exercise(None, "control");
    let (control, trace) = exercise(Some("work-leaf-bench-experiment-v2"), "control");
    assert_eq!(baseline, control);
    assert!(no_trace.is_empty());
    for row in &control {
        assert_eq!(row["actual"], row["baseline"]);
    }
    if cfg!(feature = "bench-experiments") {
        assert_eq!(trace[0]["schema"], "work-leaf-bench-experiment-v2");
        assert_eq!(
            trace.len(),
            control.iter().filter(|row| row["site"] != "raw").count() + 1
        );
        assert!(trace[1..].iter().all(|row| row["changed"] == false));
        validate_trace(&control, &trace, false);
    } else {
        assert!(trace.is_empty());
    }
}

#[test]
fn work_unit_variant_changes_only_owned_spans_across_all_eight_injection_paths() {
    let (delivered, trace) = exercise(Some("work-leaf-bench-experiment-v2"), VARIANT);
    if cfg!(feature = "bench-experiments") {
        validate_trace(&delivered, &trace, true);
        assert!(delivered.iter().any(|row| row["actual"] != row["baseline"]));
    } else {
        assert!(trace.is_empty());
        assert!(delivered.iter().all(|row| row["actual"] == row["baseline"]));
    }
}

#[test]
fn work_unit_v1_does_not_emit_policy_rows_or_change_policy() {
    let (delivered, trace) = exercise(Some("work-leaf-bench-experiment-v1"), "control");
    assert!(delivered.iter().all(|row| row["actual"] == row["baseline"]));
    if cfg!(feature = "bench-experiments") {
        assert_eq!(trace.len(), 3);
        assert_eq!(trace[1]["site"], "patch-applied");
        assert_eq!(trace[2]["site"], "command-result");
        assert!(
            trace[1..]
                .iter()
                .all(|row| row.get("spans").is_none() && row.get("schema").is_none())
        );
    } else {
        assert!(trace.is_empty());
    }
}

#[test]
#[cfg(feature = "bench-experiments")]
fn work_unit_bad_admission_fails_all_provider_paths_before_generation() {
    for reason in [
        "unknown-schema",
        "unknown-condition",
        "wrong-run",
        "existing-evidence",
    ] {
        let root = root();
        let schema = if reason == "unknown-schema" {
            "bad-schema"
        } else {
            "work-leaf-bench-experiment-v2"
        };
        let condition = if reason == "unknown-condition" {
            "ack-validation-unlimited"
        } else {
            VARIANT
        };
        let mut command = child(&root, Some(schema), condition);
        command.env("WORK_UNIT_EXPECT_ERROR", "1");
        if reason == "wrong-run" {
            command.env("WORK_LEAF_BENCH_RUN_ID", "different");
        }
        if reason == "existing-evidence" {
            fs::write(root.join("evidence.jsonl"), "preserve").unwrap();
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{reason}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if reason == "existing-evidence" {
            assert_eq!(
                fs::read_to_string(root.join("evidence.jsonl")).unwrap(),
                "preserve"
            );
        }
    }
}

fn validate_trace(delivered: &[Value], trace: &[Value], variant: bool) {
    let mut records = trace[1..].iter();
    for row in delivered {
        if row["site"] == "raw" {
            assert_eq!(row["actual"], row["baseline"]);
            continue;
        }
        let evidence = records.next().expect("every actual injection has evidence");
        assert_eq!(evidence["schema"], "work-leaf-bench-experiment-v2");
        assert_eq!(evidence["agent_id"], row["agent_id"]);
        assert_eq!(evidence["site"], row["site"]);
        assert_eq!(evidence["original_prompt"], row["baseline"]);
        assert_eq!(evidence["forwarded_prompt"], row["actual"]);
        let original = row["baseline"].as_str().unwrap();
        let spans = evidence["spans"].as_array().unwrap();
        let mut reconstructed = String::new();
        let mut cursor = 0;
        let mut delta = 0;
        for span in spans {
            let start = span["cue_start"].as_u64().unwrap() as usize;
            let end = span["cue_end"].as_u64().unwrap() as usize;
            assert!(start >= cursor && end >= start);
            let old = span["original"].as_str().unwrap();
            let replacement = span["replacement"].as_str().unwrap();
            assert_eq!(&original[start..end], old);
            let intended = match span["id"].as_str().unwrap() {
                "policy-buildable-work-unit" => {
                    assert_eq!(old, A);
                    NEW_A
                }
                "instruction-tests-work-unit" => {
                    assert_eq!(old, B);
                    NEW_B
                }
                "patch-applied-remaining-work" => {
                    assert_eq!(old, C);
                    NEW_C
                }
                "patch-applied-validation" => {
                    assert_eq!(old, ACK);
                    old
                }
                "command-result-guidance" => old,
                id => panic!("unexpected span {id}"),
            };
            assert_eq!(replacement, if variant { intended } else { old });
            assert_eq!(span["changed"], old != replacement);
            let byte_delta = replacement.len() as i64 - old.len() as i64;
            assert_eq!(span["byte_delta"], byte_delta);
            delta += byte_delta;
            reconstructed.push_str(&original[cursor..start]);
            reconstructed.push_str(replacement);
            cursor = end;
        }
        reconstructed.push_str(&original[cursor..]);
        assert_eq!(reconstructed, row["actual"].as_str().unwrap());
        assert_eq!(evidence["byte_delta"], delta);
        assert_eq!(evidence["original_bytes"], original.len());
        assert_eq!(evidence["forwarded_bytes"], reconstructed.len());
        assert_eq!(evidence["changed"], original != reconstructed);
        if row["site"] == "policy-injection" {
            let linearizer = row["agent_id"].as_str().unwrap().starts_with("linearize");
            let count = if linearizer {
                0
            } else {
                1 + row["test_translations"].as_u64().unwrap() as usize
            };
            assert_eq!(spans.len(), count);
            // Both original instructions and user data contain exact cue duplicates and Unicode.
            assert!(reconstructed.ends_with(row["suffix"].as_str().unwrap()));
            if !linearizer && count > 1 {
                assert!(reconstructed.contains(&format!("--- AGENTS.md ---\nλ {A}\n{B}\n")));
            }
        } else if row["site"] == "patch-applied" {
            assert_eq!(spans.len(), 2);
            assert!(reconstructed.contains(ACK));
        } else {
            assert_eq!(spans.len(), 1);
        }
    }
    assert!(
        records.next().is_none(),
        "no extra model boundaries in trace"
    );
    for (index, row) in trace[1..].iter().enumerate() {
        assert_eq!(row["sequence"], index + 1);
    }
}

fn fake_provider(root: &Path) -> PathBuf {
    let path = root.join("fixture-provider");
    fs::write(&path, r#"#!/usr/bin/env python3
import json, pathlib, sys
def emit(value):
    print(json.dumps(value), flush=True)
def capture(prompt):
    with pathlib.Path('captured.jsonl').open('a') as log:
        log.write(json.dumps({'prompt':prompt})+'\n')
if 'app-server' in sys.argv:
    for line in sys.stdin:
        frame=json.loads(line); method=frame.get('method'); rpc=frame.get('id'); p=frame.get('params',{})
        if method in ('thread/start','thread/resume'):
            emit({'id':rpc,'result':{'thread':{'id':'thread-'+str(rpc),'cwd':str(pathlib.Path.cwd()),'status':'loaded'}}})
        elif method == 'turn/start':
            capture(p['input'][0]['text']); thread=p['threadId']; turn='turn-'+str(rpc)
            emit({'id':rpc,'result':{'turn':{'id':turn}}})
            emit({'method':'turn/started','params':{'threadId':thread,'turnId':turn,'turn':{'id':turn,'status':'inProgress'}}})
            emit({'method':'item/completed','params':{'threadId':thread,'turnId':turn,'item':{'id':'item-'+str(rpc),'type':'agentMessage','text':'fixture reply'}}})
            emit({'method':'turn/completed','params':{'threadId':thread,'turnId':turn,'turn':{'id':turn,'status':'completed'}}})
        elif rpc is not None: emit({'id':rpc,'result':{}})
else:
    frame=json.loads(sys.stdin.readline()); capture(frame['message']['content'])
    emit({'type':'result','subtype':'success','session_id':'fixture-session','result':'fixture reply'})
"#).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn provider_paths<B: AgentBackend>(
    mut backend: B,
    root: &Path,
    policy: &PromptPolicy,
    prefix: &str,
    translations: usize,
    delivered: &mut Vec<Value>,
    expect_error: bool,
) {
    let prompt = format!("user α\n{A}\n{B}\n{C}");
    for (index, id) in [
        format!("{prefix}-launch"),
        format!("{prefix}-launch-interruptible"),
        format!("{prefix}-fallback"),
        format!("{prefix}-fallback-interruptible"),
        format!("linearize-{prefix}"),
        format!("review-{prefix}"),
        format!("title-{prefix}"),
    ]
    .iter()
    .enumerate()
    {
        let id = AgentId::new(id).unwrap();
        let feature = if matches!(index, 2 | 3) {
            "unknown"
        } else {
            "feature"
        };
        let baseline = policy.inject(&id, feature, &prompt);
        let result = match index {
            1 => backend
                .launch_streaming_interruptible(
                    AgentLaunch::new(id.clone(), AgentKind::Codex, feature, &prompt),
                    &mut |_| {},
                    &mut |_| false,
                )
                .map(|_| ()),
            2 => backend
                .send_streaming(&id, &prompt, &mut |_| {})
                .map(|_| ()),
            3 => backend
                .send_streaming_interruptible(&id, &prompt, &mut |_| {}, &mut |_| false)
                .map(|_| ()),
            _ => backend
                .launch_streaming(
                    AgentLaunch::new(id.clone(), AgentKind::Codex, feature, &prompt),
                    &mut |_| {},
                )
                .map(|_| ()),
        };
        if expect_error {
            assert!(matches!(result, Err(AgentError::Io(_))), "{result:?}");
            assert!(!root.join("captured.jsonl").exists());
            continue;
        }
        result.unwrap();
        let captured = read_rows(&root.join("captured.jsonl"));
        delivered.push(json!({"agent_id":id,"site":"policy-injection","baseline":baseline,"actual":captured.last().unwrap()["prompt"],"suffix":prompt,"test_translations":translations}));
        if index == 0 {
            for interruptible in [false, true] {
                if interruptible {
                    backend
                        .send_streaming_interruptible(&id, &prompt, &mut |_| {}, &mut |_| false)
                        .unwrap();
                } else {
                    backend.send_streaming(&id, &prompt, &mut |_| {}).unwrap();
                }
                let captured = read_rows(&root.join("captured.jsonl"));
                delivered.push(json!({"agent_id":id,"site":"raw","baseline":prompt,"actual":captured.last().unwrap()["prompt"]}));
            }
        }
    }
    if !expect_error {
        assert_eq!(
            read_rows(&root.join("captured.jsonl")).len(),
            9,
            "seven injections and two raw follow-ups, without extra turns"
        );
    }
    backend.shutdown();
}

#[derive(Default)]
struct RecordingBackend(Vec<String>);
impl AgentBackend for RecordingBackend {
    fn launch(&mut self, _: AgentLaunch) -> Result<work_leaf::AgentSession, AgentError> {
        unreachable!()
    }
    fn send(&mut self, _: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.0.push(prompt.to_string());
        Ok(ChatMessage::new(MessageRole::Agent, "fixture reply"))
    }
}

#[test]
#[ignore = "isolated subprocess fixture; parent tests provide the environment"]
fn work_unit_child() {
    let root = PathBuf::from(std::env::var_os("WORK_UNIT_TEST_ROOT").unwrap());
    let expect_error = std::env::var_os("WORK_UNIT_EXPECT_ERROR").is_some();
    let binary = fake_provider(&root);
    let mut delivered = Vec::new();
    for (permission_index, permission) in [
        ReadPermission::Orchestrator,
        ReadPermission::DirectFilesystem,
    ]
    .into_iter()
    .enumerate()
    {
        for instructions in [false, true] {
            for provider in ["codex", "claude"] {
                let prefix = format!("p{permission_index}-{instructions}-{provider}");
                let project = root.join(&prefix);
                fs::create_dir(&project).unwrap();
                if instructions {
                    fs::write(project.join("AGENTS.md"), format!("λ {A}\n{B}\n")).unwrap();
                    fs::write(project.join("agent.md"), "Add regression coverage. Required checks: cargo test. Review only changes. Documentation and commit message rules apply. Real-agent verification is mandatory.").unwrap();
                    fs::write(project.join("AGENT.md"), "Preserve naming and ownership.").unwrap();
                }
                let policy =
                    PromptPolicy::for_project_with_read_permission(&project, permission).unwrap();
                if provider == "codex" {
                    provider_paths(
                        CodexBackend::new(
                            CodexCommandConfig::new(project.clone()).with_binary(&binary),
                            policy.clone(),
                        ),
                        &project,
                        &policy,
                        &prefix,
                        if instructions { 2 } else { 0 },
                        &mut delivered,
                        expect_error,
                    );
                } else {
                    provider_paths(
                        ClaudeBackend::new(
                            ClaudeCommandConfig::new(project.clone()).with_binary(&binary),
                            policy.clone(),
                        ),
                        &project,
                        &policy,
                        &prefix,
                        if instructions { 2 } else { 0 },
                        &mut delivered,
                        expect_error,
                    );
                }
            }
        }
    }
    if expect_error {
        return;
    }
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.email", "fixture@example.invalid"],
        vec!["config", "user.name", "Fixture"],
    ] {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&root)
                .status()
                .unwrap()
                .success()
        );
    }
    fs::write(root.join("input.rs"), "fn before() {}\n").unwrap();
    for args in [
        vec!["add", "input.rs"],
        vec!["commit", "-qm", "ADD fixture for work unit delivery"],
    ] {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&root)
                .status()
                .unwrap()
                .success()
        );
    }
    let mut orchestrator = AgentOrchestrator::new(root.clone(), RecordingBackend::default());
    let id = AgentId::new("fixture-edit").unwrap();
    for message in [
        "@work-leaf edit rename function\n*** Begin Patch\n*** Update File: input.rs\n@@\n-fn before() {}\n+fn after() {}\n*** End Patch\n@work-leaf end",
        "@work-leaf locks run target -- git status --short input.rs",
    ] {
        orchestrator
            .handle_agent_message(&id, "feature", message)
            .unwrap();
    }
    assert_eq!(
        fs::read_to_string(root.join("input.rs")).unwrap(),
        "fn after() {}\n"
    );
    for (index, actual) in orchestrator.into_backend().0.into_iter().enumerate() {
        let baseline = if index == 0 {
            actual.replace(NEW_C, C)
        } else {
            actual.clone()
        };
        delivered.push(json!({"agent_id":id,"site":if index == 0 { "patch-applied" } else { "command-result" },"baseline":baseline,"actual":actual}));
    }
    fs::write(
        root.join("delivered.json"),
        serde_json::to_vec(&delivered).unwrap(),
    )
    .unwrap();
}
