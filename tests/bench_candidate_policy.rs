#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use work_leaf::{
    AgentBackend, AgentId, AgentKind, AgentLaunch, ClaudeBackend, ClaudeCommandConfig,
    CodexBackend, CodexCommandConfig, PromptPolicy, ReadPermission,
};

mod temp_cleanup;

const REPEAT: &str = "If you request a file you already received, Work Leaf compares digests and returns either unchanged status or a diff from your last snapshot; do not use repeated reads to reload whole files.";
const FULL: &str = "If you request a file you already received, Work Leaf returns its current full text; avoid unnecessary repeated reads.";
const AUTHORITY: &str = "Treat compact file refreshes and repeated-read digests as authoritative.";
const FULL_AUTHORITY: &str = "Treat compact automatic file refreshes and current full-text requested reads as authoritative.";
const WRITE: &str = "You are not allowed to write files directly; submit a structured edit patch for every file you want to change.";
const PATCH_WRITE: &str = "You are not allowed to write files directly; submit an orchestrator patch for every file you want to change.";
const EDIT: &str = "Use `@work-leaf edit <reason>` followed by an apply-patch-style exact edit body and `@work-leaf end` to request a write.";
const PATCH: &str = "Use `@work-leaf patch <reason>` followed by a complete valid unified diff with real hunk ranges and `@work-leaf end` to request a write.";
const PREFERENCE: &str = "The legacy `@work-leaf patch <reason>` unified-diff directive is still accepted only when you already have a complete valid unified diff with real hunk ranges; prefer `@work-leaf edit` for manual code, configuration, and test changes.";
const PATCH_PREFERENCE: &str = "The `@work-leaf edit <reason>` directive followed by an apply-patch-style exact edit body and `@work-leaf end` remains accepted; prefer `@work-leaf patch` for manual code, configuration, and test changes.";
const MANUAL: &str = "Do not use command locks for manual feature edits; manual code, configuration, and test changes must still be submitted with the structured edit directive.";
const PATCH_MANUAL: &str = "Do not use command locks for manual feature edits; manual code, configuration, and test changes must still be submitted with a patch directive.";
const COMMIT: &str = "- Commit-message rules remain mandatory. Patch agents express intent through the `@work-leaf edit <reason>` reason; final commit-message compliance is enforced through patch reason and final linearized commits.";
const PATCH_COMMIT: &str = "- Commit-message rules remain mandatory. Patch agents express intent through the `@work-leaf patch <reason>` or `@work-leaf edit <reason>` reason; final commit-message compliance is enforced through patch reason and final linearized commits.";

fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "work-leaf-candidate-policy-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    temp_cleanup::register(&root);
    root
}

fn rows(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn exercise(schema: Option<&str>, condition: &str) -> (Vec<Value>, Vec<Value>) {
    let root = root();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "candidate_policy_child",
            "--ignored",
            "--nocapture",
        ])
        .env("CANDIDATE_POLICY_TEST_ROOT", &root)
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT")
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST")
        .env_remove("WORK_LEAF_BENCH_RUN_ID");
    if let Some(schema) = schema {
        let manifest = root.join("manifest.json");
        fs::write(
            &manifest,
            serde_json::to_vec(&json!({
                "schema":schema,"condition":condition,"run_id":"candidate-policy-fixture",
                "evidence_path":root.join("evidence.jsonl")
            }))
            .unwrap(),
        )
        .unwrap();
        command
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", manifest)
            .env("WORK_LEAF_BENCH_RUN_ID", "candidate-policy-fixture");
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (
        rows(&root.join("delivered.jsonl")),
        rows(&root.join("evidence.jsonl")),
    )
}

#[test]
fn candidate_policy_preserves_default_and_all_legacy_schema_contracts() {
    let (baseline, empty) = exercise(None, "control");
    assert!(empty.is_empty());
    for schema in [
        "work-leaf-bench-experiment-v1",
        "work-leaf-bench-experiment-v2",
        "work-leaf-bench-experiment-v3",
    ] {
        let (delivered, trace) = exercise(Some(schema), "control");
        assert_eq!(baseline, delivered);
        if cfg!(feature = "bench-experiments") && schema != "work-leaf-bench-experiment-v1" {
            assert_eq!(trace.len(), 33);
            for row in &trace[1..] {
                assert_eq!(row["schema"], schema);
                for span in row["spans"].as_array().unwrap() {
                    assert!(matches!(
                        span["id"].as_str().unwrap(),
                        "policy-buildable-work-unit" | "instruction-tests-work-unit"
                    ));
                }
            }
        } else {
            assert_eq!(
                trace.len(),
                usize::from(cfg!(feature = "bench-experiments"))
            );
        }
    }
}

#[test]
fn candidate_policy_v4_only_changes_renderer_owned_factor_spans() {
    for condition in [
        "requested-repeat-full",
        "unified-diff-preferred",
        "review-fix-request-resupply",
    ] {
        let (delivered, trace) = exercise(Some("work-leaf-bench-experiment-v4"), condition);
        if !cfg!(feature = "bench-experiments") {
            assert!(trace.is_empty());
            assert!(delivered.iter().all(|row| row["baseline"] == row["actual"]));
            continue;
        }
        assert_eq!(trace.len(), 33);
        let mut events = trace[1..].iter();
        let mut changed = 0;
        for row in &delivered {
            if row["raw"] == true {
                assert_eq!(row["actual"], row["baseline"]);
                continue;
            }
            let event = events.next().unwrap();
            assert_eq!(event["schema"], "work-leaf-bench-experiment-v4");
            assert_eq!(event["site"], "candidate-policy");
            assert_eq!(event["original_prompt"], row["baseline"]);
            assert_eq!(event["candidate_prompt"], row["actual"]);
            let original = row["baseline"].as_str().unwrap();
            let mut reconstructed = String::new();
            let mut cursor = 0;
            let spans = event["metadata"]["spans"].as_array().unwrap();
            let components = event["components"].as_array().unwrap();
            assert_eq!(spans.len(), components.len());
            for (span, component) in spans.iter().zip(components) {
                let (old, replacement, factor) = match span["id"].as_str().unwrap() {
                    "policy-requested-repeat-contract" => (REPEAT, FULL, "requested-repeat-full"),
                    "policy-repeat-authority" => {
                        (AUTHORITY, FULL_AUTHORITY, "requested-repeat-full")
                    }
                    "policy-write-format" => (WRITE, PATCH_WRITE, "unified-diff-preferred"),
                    "policy-edit-request" => (EDIT, PATCH, "unified-diff-preferred"),
                    "policy-edit-preference" => {
                        (PREFERENCE, PATCH_PREFERENCE, "unified-diff-preferred")
                    }
                    "policy-manual-write-format" => {
                        (MANUAL, PATCH_MANUAL, "unified-diff-preferred")
                    }
                    "instruction-commit-format" => (COMMIT, PATCH_COMMIT, "unified-diff-preferred"),
                    id => panic!("unexpected candidate policy span {id}"),
                };
                let start = component["baseline_start"].as_u64().unwrap() as usize;
                let end = component["baseline_end"].as_u64().unwrap() as usize;
                assert!(start >= cursor);
                assert_eq!(&original[start..end], old);
                let expected = if factor == condition {
                    replacement
                } else {
                    old
                };
                assert_eq!(span["original"], old);
                assert_eq!(span["replacement"], replacement);
                assert_eq!(span["condition"], factor);
                let actual = row["actual"].as_str().unwrap();
                if condition == "unified-diff-preferred" && span["id"] == "policy-edit-request" {
                    assert!(
                        actual[component["inline_start"].as_u64().unwrap() as usize
                            ..component["inline_end"].as_u64().unwrap() as usize]
                            .contains("and `@work-leaf end` to request a write."),
                        "preferred patch guidance must preserve the explicit termination marker"
                    );
                }
                assert_eq!(
                    &actual[component["inline_start"].as_u64().unwrap() as usize
                        ..component["inline_end"].as_u64().unwrap() as usize],
                    expected
                );
                reconstructed.push_str(&original[cursor..start]);
                reconstructed.push_str(expected);
                cursor = end;
            }
            reconstructed.push_str(&original[cursor..]);
            assert_eq!(reconstructed, row["actual"].as_str().unwrap());
            assert!(reconstructed.ends_with(row["suffix"].as_str().unwrap()));
            assert!(reconstructed.contains(&format!(
                "--- AGENTS.md ---\n{}",
                row["instructions"].as_str().unwrap()
            )));
            let linearize = row["agent_id"].as_str().unwrap().starts_with("linearize");
            assert_eq!(
                spans.len(),
                if linearize {
                    0
                } else if row["restricted"] == true {
                    7
                } else {
                    6
                }
            );
            changed += usize::from(original != reconstructed);
        }
        assert!(events.next().is_none());
        assert_eq!(
            changed,
            if matches!(
                condition,
                "requested-repeat-full" | "unified-diff-preferred"
            ) {
                28
            } else {
                0
            }
        );
    }
}

fn fake_provider(root: &Path) -> PathBuf {
    let binary = root.join("fixture-provider");
    fs::write(&binary, r#"#!/usr/bin/env python3
import json,pathlib,sys
def emit(v): print(json.dumps(v),flush=True)
def capture(p):
    with pathlib.Path('capture.jsonl').open('a') as f: f.write(json.dumps({'prompt':p})+'\n')
if 'app-server' in sys.argv:
    for line in sys.stdin:
        f=json.loads(line); m=f.get('method'); i=f.get('id'); p=f.get('params',{})
        if m in ('thread/start','thread/resume'): emit({'id':i,'result':{'thread':{'id':'thread-'+str(i),'cwd':str(pathlib.Path.cwd()),'status':'loaded'}}})
        elif m=='turn/start':
            capture(p['input'][0]['text']); t=p['threadId']; v='turn-'+str(i)
            emit({'id':i,'result':{'turn':{'id':v}}})
            emit({'method':'item/completed','params':{'threadId':t,'turnId':v,'item':{'id':'item-'+str(i),'type':'agentMessage','text':'fixture reply'}}})
            emit({'method':'turn/completed','params':{'threadId':t,'turnId':v,'turn':{'id':v,'status':'completed'}}})
        elif i is not None: emit({'id':i,'result':{}})
else:
    f=json.loads(sys.stdin.readline()); capture(f['message']['content'])
    emit({'type':'result','subtype':'success','session_id':'fixture-session','result':'fixture reply'})
"#).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    binary
}

fn provider_paths<B: AgentBackend>(
    mut backend: B,
    project: &Path,
    policy: &PromptPolicy,
    restricted: bool,
    instructions: &str,
    delivered: &mut Vec<Value>,
) {
    let prompt = format!("λ user input\n{instructions}");
    for (index, name) in [
        "launch",
        "launch-interrupt",
        "fallback",
        "fallback-interrupt",
        "review-fixture",
        "title-fixture",
        "linearize-fixture",
        "ordinary-fixture",
    ]
    .iter()
    .enumerate()
    {
        let id = AgentId::new(*name).unwrap();
        let feature = if matches!(index, 2 | 3) {
            "unknown"
        } else {
            "feature"
        };
        let baseline = policy.inject(&id, feature, &prompt);
        match index {
            1 => {
                backend
                    .launch_streaming_interruptible(
                        AgentLaunch::new(id.clone(), AgentKind::Codex, feature, &prompt),
                        &mut |_| {},
                        &mut |_| false,
                    )
                    .unwrap();
            }
            2 => {
                backend.send_streaming(&id, &prompt, &mut |_| {}).unwrap();
            }
            3 => {
                backend
                    .send_streaming_interruptible(&id, &prompt, &mut |_| {}, &mut |_| false)
                    .unwrap();
            }
            _ => {
                backend
                    .launch_streaming(
                        AgentLaunch::new(id.clone(), AgentKind::Codex, feature, &prompt),
                        &mut |_| {},
                    )
                    .unwrap();
            }
        }
        delivered.push(json!({"agent_id":id,"baseline":baseline,"actual":rows(&project.join("capture.jsonl")).last().unwrap()["prompt"],"suffix":prompt,"instructions":instructions,"restricted":restricted}));
        if index == 0 {
            backend.send_streaming(&id, &prompt, &mut |_| {}).unwrap();
            delivered.push(json!({"raw":true,"baseline":prompt,"actual":rows(&project.join("capture.jsonl")).last().unwrap()["prompt"]}));
        }
    }
    assert_eq!(rows(&project.join("capture.jsonl")).len(), 9);
    backend.shutdown();
}

#[test]
#[ignore = "isolated subprocess fixture; parent tests provide environment"]
fn candidate_policy_child() {
    let root = PathBuf::from(std::env::var_os("CANDIDATE_POLICY_TEST_ROOT").unwrap());
    let binary = fake_provider(&root);
    let instructions = format!(
        "λ commit messages and regression tests\n{REPEAT}\n{AUTHORITY}\n{WRITE}\n{EDIT}\n{PREFERENCE}\n{MANUAL}\n{COMMIT}\n"
    );
    let mut delivered = Vec::new();
    for (index, permission) in [
        ReadPermission::Orchestrator,
        ReadPermission::DirectFilesystem,
    ]
    .into_iter()
    .enumerate()
    {
        for provider in ["codex", "claude"] {
            let project = root.join(format!("{index}-{provider}"));
            fs::create_dir(&project).unwrap();
            fs::write(project.join("AGENTS.md"), &instructions).unwrap();
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
                    index == 0,
                    &instructions,
                    &mut delivered,
                );
            } else {
                provider_paths(
                    ClaudeBackend::new(
                        ClaudeCommandConfig::new(project.clone()).with_binary(&binary),
                        policy.clone(),
                    ),
                    &project,
                    &policy,
                    index == 0,
                    &instructions,
                    &mut delivered,
                );
            }
        }
    }
    fs::write(
        root.join("delivered.jsonl"),
        delivered
            .iter()
            .map(|row| format!("{row}\n"))
            .collect::<String>(),
    )
    .unwrap();
}
