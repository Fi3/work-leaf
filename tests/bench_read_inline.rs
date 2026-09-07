use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use work_leaf::{AgentBackend, AgentError, AgentId, AgentOrchestrator, ChatMessage, MessageRole};

mod temp_cleanup;

const SCHEMA: &str = "work-leaf-bench-experiment-v3";
const VARIANT: &str = "untracked-read-inline";

fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "work-leaf-read-inline-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    temp_cleanup::register(&root);
    root
}

fn child(root: &Path, schema: Option<&str>, condition: &str, scenario: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "read_inline_child", "--ignored", "--nocapture"])
        .env("READ_INLINE_TEST_ROOT", root)
        .env("READ_INLINE_SCENARIO", scenario)
        .env("WORK_LEAF_CONTEXT_BUNDLE_DIR", root.join("bundles"))
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT")
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST")
        .env_remove("WORK_LEAF_BENCH_RUN_ID");
    if let Some(schema) = schema {
        let path = root.join("manifest.json");
        fs::write(
            &path,
            serde_json::to_vec(&json!({
                "schema": schema, "condition": condition, "run_id": "read-inline-fixture",
                "evidence_path": root.join("evidence.jsonl")
            }))
            .unwrap(),
        )
        .unwrap();
        command
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", path)
            .env("WORK_LEAF_BENCH_RUN_ID", "read-inline-fixture");
    }
    command
}

fn exercise(schema: Option<&str>, condition: &str, scenario: &str) -> (Value, Vec<Value>) {
    let root = root();
    let output = child(&root, schema, condition, scenario).output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut fixture: Value =
        serde_json::from_slice(&fs::read(root.join("result.json")).unwrap()).unwrap();
    fixture["root"] = json!(root);
    let trace = fs::read_to_string(root.join("evidence.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (fixture, trace)
}

fn normalized(fixture: &Value) -> Vec<String> {
    fixture["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|step| {
            let text = step["prompt"].as_str().unwrap();
            let bundle_dir = fixture["bundle_dir"].as_str().unwrap();
            let text = if bundle_dir.is_empty() {
                text.to_string()
            } else {
                text.replace(bundle_dir, "<BUNDLES>")
            };
            text.replace(fixture["root"].as_str().unwrap(), "<ROOT>")
        })
        .collect()
}

#[test]
fn read_inline_control_preserves_baseline_and_legacy_reads() {
    let (baseline, trace) = exercise(None, "control", "matrix");
    assert!(trace.is_empty());
    for schema in [
        "work-leaf-bench-experiment-v1",
        "work-leaf-bench-experiment-v2",
        SCHEMA,
    ] {
        let (control, trace) = exercise(Some(schema), "control", "matrix");
        assert_eq!(normalized(&baseline), normalized(&control), "{schema}");
        assert_eq!(baseline["bundles"], control["bundles"]);
        if cfg!(feature = "bench-experiments") && schema == SCHEMA {
            validate_trace(&control, &trace, false);
        } else {
            assert!(trace.iter().all(|row| row["event"] != "read-response"));
        }
    }
}

#[test]
fn read_inline_variant_changes_only_successfully_bundled_untracked_component() {
    let (control, _) = exercise(Some(SCHEMA), "control", "matrix");
    let (variant, trace) = exercise(Some(SCHEMA), VARIANT, "matrix");
    assert_eq!(
        control["bundles"], variant["bundles"],
        "same allocations and bytes"
    );
    if cfg!(feature = "bench-experiments") {
        validate_trace(&variant, &trace, true);
        let eligible: Vec<_> = trace.iter().filter(|row| row["eligible"] == true).collect();
        assert_eq!(eligible.len(), 4);
        for (index, (normal, varied)) in normalized(&control)
            .iter()
            .zip(normalized(&variant))
            .enumerate()
        {
            assert_eq!(
                normal != &varied,
                [2, 4, 7, 9].contains(&index),
                "step {index}"
            );
        }
    } else {
        assert!(
            trace.is_empty(),
            "default builds cannot activate any manifest"
        );
        assert_eq!(normalized(&control), normalized(&variant));
    }
}

#[test]
fn read_inline_failed_bundle_attempt_keeps_fallback_and_next_allocation() {
    let (control, _) = exercise(Some(SCHEMA), "control", "bundle-failure");
    let (variant, trace) = exercise(Some(SCHEMA), VARIANT, "bundle-failure");
    assert_eq!(control["bundles"], variant["bundles"]);
    assert!(control["bundles"].get("bundle-1.md").is_some());
    assert!(control["bundles"].get("bundle-0.md").is_none());
    assert_eq!(normalized(&control)[0], normalized(&variant)[0]);
    if cfg!(feature = "bench-experiments") {
        validate_trace(&variant, &trace, true);
        assert_eq!(trace[1]["eligibility_reason"], "bundle-write-failed");
        assert_eq!(trace[1]["bundle"]["threshold_eligible"], true);
        assert_eq!(trace[1]["bundle"]["write_succeeded"], false);
    }
}

#[test]
fn read_inline_failed_send_does_not_advance_snapshot_tracking() {
    let (fixture, trace) = exercise(Some(SCHEMA), VARIANT, "send-failure");
    assert!(fixture["steps"][0]["failed"].as_bool().unwrap());
    assert!(!fixture["steps"][1]["failed"].as_bool().unwrap());
    assert!(fixture["bundles"].get("bundle-0.md").is_some());
    assert!(fixture["bundles"].get("bundle-1.md").is_some());
    if cfg!(feature = "bench-experiments") {
        validate_trace(&fixture, &trace, true);
        assert!(trace[1..].iter().all(|row| row["eligible"] == true));
    }
}

#[test]
fn read_inline_authored_edit_clears_tracker_but_refresh_ack_and_command_stay_normal() {
    let (control, _) = exercise(Some(SCHEMA), "control", "state");
    let (variant, trace) = exercise(Some(SCHEMA), VARIANT, "state");
    assert_eq!(control["bundles"], variant["bundles"]);
    assert_eq!(control["refresh"], variant["refresh"]);
    assert!(
        variant["final_text"]
            .as_str()
            .unwrap()
            .starts_with("fn after() {}\n")
    );
    if cfg!(feature = "bench-experiments") {
        validate_trace(&variant, &trace, true);
        let reads: Vec<_> = trace
            .iter()
            .filter(|row| row["event"] == "read-response")
            .collect();
        assert_eq!(
            reads
                .iter()
                .map(|row| row["eligible"].as_bool().unwrap())
                .collect::<Vec<_>>(),
            [true, true, false]
        );
        let nonreads: Vec<_> = trace
            .iter()
            .filter(|row| row["event"] == "prompt")
            .collect();
        assert_eq!(nonreads.len(), 2, "normal command and successful ACK only");
        for row in nonreads {
            assert_eq!(row["changed"], false);
            assert_eq!(row["original_prompt"], row["forwarded_prompt"]);
        }
    }
}

#[test]
#[cfg(feature = "bench-experiments")]
fn read_inline_v3_rejects_nonfactor_conditions_before_send_or_bundle_allocation() {
    for condition in [
        "ack-validation-unlimited",
        "command-guidance-neutral",
        "buildable-work-unit-incremental",
    ] {
        let root = root();
        let output = child(&root, Some(SCHEMA), condition, "invalid")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!root.join("evidence.jsonl").exists());
        assert!(!root.join("bundles").exists());
    }
}

fn validate_trace(fixture: &Value, trace: &[Value], treatment: bool) {
    assert_eq!(trace[0]["schema"], SCHEMA);
    let reads: Vec<_> = trace
        .iter()
        .filter(|row| row["event"] == "read-response")
        .collect();
    let steps = fixture["steps"].as_array().unwrap();
    assert_eq!(
        reads.len(),
        steps.len(),
        "every read delivery has one trace row"
    );
    for (step, row) in steps.iter().zip(reads) {
        assert_eq!(row["schema"], SCHEMA);
        assert_eq!(row["site"], "file-read");
        let baseline = row["baseline_prompt"].as_str().unwrap();
        let inline = row["inline_candidate_prompt"].as_str().unwrap();
        let eligible = row["eligible"].as_bool().unwrap();
        assert_eq!(row["bundle"]["write_succeeded"], eligible);
        let selected = if treatment && eligible {
            inline
        } else {
            baseline
        };
        assert_eq!(
            row["selected_candidate"],
            if treatment && eligible {
                "inline"
            } else {
                "baseline"
            }
        );
        assert_eq!(step["prompt"], selected);
        assert_eq!(row["selected_bytes"], selected.len());
        assert_eq!(row["baseline_bytes"], baseline.len());
        assert_eq!(row["inline_candidate_bytes"], inline.len());
        assert_eq!(
            row["candidate_byte_delta"],
            inline.len() as i64 - baseline.len() as i64
        );
        assert_eq!(
            row["byte_delta"],
            selected.len() as i64 - baseline.len() as i64
        );
        assert_eq!(row["changed"], selected != baseline);
        assert!(
            row.get("forwarded_prompt").is_none(),
            "no arm-specific third body"
        );
        if let Some(span) = row["component"].as_object() {
            let start = span["baseline_start"].as_u64().unwrap() as usize;
            let end = span["baseline_end"].as_u64().unwrap() as usize;
            let inline_start = span["inline_start"].as_u64().unwrap() as usize;
            let inline_end = span["inline_end"].as_u64().unwrap() as usize;
            assert_eq!(start, inline_start);
            assert_eq!(&baseline[..start], &inline[..inline_start]);
            assert_eq!(&baseline[end..], &inline[inline_end..]);
        } else {
            assert_eq!(baseline, inline);
            assert!(!eligible);
        }
        if !eligible {
            assert_eq!(baseline, inline);
        }
        for snapshot in row["snapshots"].as_array().unwrap() {
            if snapshot["class"] == "untracked" {
                let start = snapshot["inline_body_start"].as_u64().unwrap() as usize;
                let end = snapshot["inline_body_end"].as_u64().unwrap() as usize;
                let path = snapshot["path"].as_str().unwrap();
                assert_eq!(&inline[start..end], step["sources"][path].as_str().unwrap());
                assert_eq!(snapshot["bytes"], end - start);
                assert!(snapshot["digest"].as_str().unwrap().starts_with("fnv64:"));
            } else {
                assert!(snapshot["inline_body_start"].is_null());
                assert!(snapshot["inline_body_end"].is_null());
            }
        }
    }
}

#[derive(Clone, Default)]
struct Recording(Arc<Mutex<Vec<String>>>, Arc<AtomicBool>);

impl AgentBackend for Recording {
    fn launch(&mut self, _: work_leaf::AgentLaunch) -> Result<work_leaf::AgentSession, AgentError> {
        panic!("read fixtures cannot launch another agent")
    }

    fn send(&mut self, _: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.0.lock().unwrap().push(prompt.to_string());
        if self.1.swap(false, Ordering::Relaxed) {
            return Err(AgentError::Io(std::io::Error::other(
                "fixture send failure",
            )));
        }
        Ok(ChatMessage::new(MessageRole::Agent, "fixture reply"))
    }
}

#[test]
#[ignore = "isolated subprocess fixture driven by ordinary tests"]
fn read_inline_child() {
    let root = PathBuf::from(std::env::var_os("READ_INLINE_TEST_ROOT").unwrap());
    let scenario = std::env::var("READ_INLINE_SCENARIO").unwrap();
    if scenario == "state" {
        state_fixture(&root);
        return;
    }
    let files = BTreeMap::from([
        ("empty", String::new()),
        ("small", "λ no final newline".to_string()),
        ("single-limit", "x".repeat(16 * 1024)),
        ("single-over", "λ".repeat(8192) + "z"),
        ("half-a", "a".repeat(12 * 1024)),
        ("half-b", "b".repeat(12 * 1024)),
        ("half-over", "c".repeat(12 * 1024 + 1)),
        (
            "fresh",
            "work-leaf file text\nContext bundle: adversarial\n--- empty ---\n".repeat(400),
        ),
        ("remembered", "before\n".to_string()),
    ]);
    for (path, text) in &files {
        fs::write(root.join(path), text).unwrap();
    }
    if scenario == "bundle-failure" {
        fs::write(root.join("bundles"), "obstruction").unwrap();
    }
    let backend = Recording::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend.clone());
    if scenario == "invalid" {
        let error = orchestrator
            .handle_agent_message(
                &AgentId::new("a").unwrap(),
                "read fixture",
                "@work-leaf read single-over",
            )
            .unwrap_err();
        assert!(error.to_string().contains("benchmark experiment"));
        assert!(backend.0.lock().unwrap().is_empty());
        return;
    }
    let mut steps = Vec::new();
    let mut bundles = BTreeMap::new();
    let mut bundle_dir = String::new();
    let requests = if scenario == "send-failure" {
        vec![
            ("a", "@work-leaf read single-over"),
            ("a", "@work-leaf read single-over"),
        ]
    } else if scenario == "bundle-failure" {
        vec![
            ("a", "@work-leaf read single-over"),
            ("b", "@work-leaf read single-over"),
        ]
    } else {
        vec![
            ("a", "@work-leaf read empty small"),
            ("a", "@work-leaf read single-limit"),
            ("a", "@work-leaf read single-over"),
            ("b", "@work-leaf read half-b half-a"),
            ("c", "@work-leaf read half-over half-a"),
            ("a", "@work-leaf read remembered"),
            ("a", "EXPLICIT"),
            ("a", "MIXED"),
            ("a", "@work-leaf read --force fresh remembered empty"),
            ("d", "@work-leaf read fresh"),
        ]
    };
    for (index, (id, request)) in requests.into_iter().enumerate() {
        if scenario == "bundle-failure" && index == 1 {
            fs::remove_file(root.join("bundles")).unwrap();
        }
        if request == "MIXED" {
            fs::write(root.join("remembered"), "after\n").unwrap();
        }
        let request = match request {
            "EXPLICIT" => format!("@work-leaf read {bundle_dir}/bundle-0.md"),
            "MIXED" => format!(
                "@work-leaf read --force fresh remembered empty absent {bundle_dir}/bundle-0.md\n@work-leaf read ./fresh"
            ),
            other => other.to_string(),
        };
        let expected_failure = scenario == "send-failure" && index == 0;
        backend.1.store(expected_failure, Ordering::Relaxed);
        let result =
            orchestrator.handle_agent_message(&AgentId::new(id).unwrap(), "read fixture", &request);
        assert_eq!(result.is_err(), expected_failure, "{result:?}");
        assert_eq!(
            backend.0.lock().unwrap().len(),
            index + 1,
            "one send per coalesced read"
        );
        let sources: BTreeMap<_, _> = files
            .keys()
            .map(|path| (*path, fs::read_to_string(root.join(path)).unwrap()))
            .collect();
        steps.push(json!({"prompt": backend.0.lock().unwrap().last().unwrap(), "sources": sources, "failed": result.is_err()}));
        if let Ok(dirs) = fs::read_dir(root.join("bundles")) {
            for dir in dirs.flatten() {
                bundle_dir = dir.path().display().to_string();
                for file in fs::read_dir(dir.path()).unwrap().flatten() {
                    bundles.insert(
                        file.file_name().to_string_lossy().to_string(),
                        fs::read_to_string(file.path()).unwrap(),
                    );
                }
            }
        }
    }
    drop(orchestrator);
    if !bundle_dir.is_empty() {
        assert!(!Path::new(&bundle_dir).exists(), "ordinary owner cleanup");
    }
    fs::write(
        root.join("result.json"),
        serde_json::to_vec(&json!({"steps": steps, "bundles": bundles, "bundle_dir": bundle_dir}))
            .unwrap(),
    )
    .unwrap();
}

fn state_fixture(root: &Path) {
    let mut text = "fn before() {}\n".to_string() + &"// λ retained context\n".repeat(1000);
    fs::write(root.join("state.rs"), &text).unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Read Fixture"],
        vec!["config", "user.email", "read@example.invalid"],
        vec!["add", "state.rs"],
        vec![
            "commit",
            "-qm",
            "ADD read fixture to verify tracker boundaries",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let backend = Recording::default();
    let mut orchestrator = AgentOrchestrator::new(root.to_path_buf(), backend.clone());
    let id = AgentId::new("state-owner").unwrap();
    let mut steps = Vec::new();
    let mut refresh = String::new();
    let mut bundles = BTreeMap::new();
    let mut bundle_dir = String::new();
    for (index, request) in [
        "@work-leaf read state.rs",
        "@work-leaf edit rejected exact block\n*** Begin Patch\n*** Update File: state.rs\n@@\n-fn absent() {}\n+fn rejected() {}\n*** End Patch\n@work-leaf end",
        "@work-leaf locks run cache -- printf 'validation output\\n'",
        "@work-leaf edit accepted owned change\n*** Begin Patch\n*** Update File: state.rs\n@@\n-fn before() {}\n+fn after() {}\n*** End Patch\n@work-leaf end",
        "@work-leaf read --force state.rs",
        "@work-leaf read state.rs",
    ].iter().enumerate() {
        orchestrator.handle_agent_message(&id, "tracker boundary fixture", request).unwrap();
        let prompts = backend.0.lock().unwrap();
        assert_eq!(prompts.len(), index + 1);
        if request.starts_with("@work-leaf read") {
            text = fs::read_to_string(root.join("state.rs")).unwrap();
            steps.push(json!({"prompt": prompts.last().unwrap(), "sources": {"state.rs": text}}));
        } else if index == 1 {
            refresh = prompts.last().unwrap().replace(&root.display().to_string(), "<ROOT>");
        }
        for dir in fs::read_dir(root.join("bundles")).unwrap().flatten() {
            bundle_dir = dir.path().display().to_string();
            for file in fs::read_dir(dir.path()).unwrap().flatten() {
                bundles.insert(file.file_name().to_string_lossy().to_string(), fs::read_to_string(file.path()).unwrap());
            }
        }
    }
    drop(orchestrator);
    assert!(!Path::new(&bundle_dir).exists());
    fs::write(root.join("result.json"), serde_json::to_vec(&json!({"steps": steps, "bundles": bundles, "bundle_dir": bundle_dir, "refresh": refresh, "final_text": text})).unwrap()).unwrap();
}
