use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use work_leaf::{AgentBackend, AgentError, AgentId, AgentOrchestrator, ChatMessage, MessageRole};

mod temp_cleanup;

fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "work-leaf-candidate-activation-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    temp_cleanup::register(&root);
    root
}

#[derive(Default)]
struct RecordingBackend(Vec<String>);

impl AgentBackend for RecordingBackend {
    fn launch(&mut self, _: work_leaf::AgentLaunch) -> Result<work_leaf::AgentSession, AgentError> {
        panic!("activation fixture must not launch a provider")
    }

    fn send(&mut self, _: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.0.push(prompt.to_string());
        Ok(ChatMessage::new(MessageRole::Agent, "fixture reply"))
    }
}

#[test]
fn candidate_schemas_activate_only_the_three_single_factor_conditions() {
    for condition in [
        "requested-repeat-full",
        "unified-diff-preferred",
        "review-fix-request-resupply",
    ] {
        let root = root();
        let manifest = root.join("manifest.json");
        fs::write(
            &manifest,
            serde_json::to_vec(&json!({
                "schema": "work-leaf-bench-experiment-v4", "run_id": "activation-fixture",
                "condition": condition, "evidence_path": root.join("evidence.jsonl")
            }))
            .unwrap(),
        )
        .unwrap();
        let result = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "candidate_activation_child",
                "--ignored",
                "--nocapture",
            ])
            .env("CANDIDATE_ACTIVATION_ROOT", &root)
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", &manifest)
            .env("WORK_LEAF_BENCH_RUN_ID", "activation-fixture")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{condition}: {} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        let prompts: Vec<String> =
            serde_json::from_slice(&fs::read(root.join("prompts.json")).unwrap()).unwrap();
        assert_eq!(prompts.len(), 1);
        assert!(prompts[0].contains("immutable fixture content"));
        if cfg!(feature = "bench-experiments") {
            let rows: Vec<Value> = fs::read_to_string(root.join("evidence.jsonl"))
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(rows[0]["schema"], "work-leaf-bench-experiment-v4");
            assert_eq!(rows[0]["condition"], condition);
            assert!(rows[1..].iter().all(|row| row["changed"] == false));
        } else {
            assert!(!root.join("evidence.jsonl").exists());
        }
    }
}

#[test]
#[ignore = "isolated activation subprocess with a recording backend; not a provider run"]
fn candidate_activation_child() {
    let root = PathBuf::from(std::env::var_os("CANDIDATE_ACTIVATION_ROOT").unwrap());
    fs::write(root.join("source.rs"), "immutable fixture content\n").unwrap();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), RecordingBackend::default());
    orchestrator
        .handle_agent_message(
            &AgentId::new("fixture").unwrap(),
            "fixture",
            "@work-leaf read source.rs",
        )
        .unwrap();
    fs::write(
        root.join("prompts.json"),
        serde_json::to_vec(&orchestrator.into_backend().0).unwrap(),
    )
    .unwrap();
}

#[test]
#[cfg(feature = "bench-experiments")]
fn candidate_activation_rejects_controls_cross_schema_conditions_and_reused_evidence() {
    for (schema, condition, reused) in [
        ("work-leaf-bench-experiment-v4", "control", false),
        (
            "work-leaf-bench-experiment-v4",
            "untracked-read-inline",
            false,
        ),
        (
            "work-leaf-bench-experiment-v3",
            "requested-repeat-full",
            false,
        ),
        (
            "work-leaf-bench-experiment-v4",
            "requested-repeat-full",
            true,
        ),
    ] {
        let root = root();
        let manifest = root.join("manifest.json");
        if reused {
            fs::write(root.join("evidence.jsonl"), "retained evidence\n").unwrap();
        }
        fs::write(
            &manifest,
            serde_json::to_vec(&json!({
                "schema": schema, "run_id": "activation-fixture", "condition": condition,
                "evidence_path": root.join("evidence.jsonl")
            }))
            .unwrap(),
        )
        .unwrap();
        let result = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "candidate_activation_child",
                "--ignored",
                "--nocapture",
            ])
            .env("CANDIDATE_ACTIVATION_ROOT", &root)
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", &manifest)
            .env("WORK_LEAF_BENCH_RUN_ID", "activation-fixture")
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("benchmark experiment"));
        assert!(!root.join("prompts.json").exists());
        if reused {
            assert_eq!(
                fs::read_to_string(root.join("evidence.jsonl")).unwrap(),
                "retained evidence\n"
            );
        } else {
            assert!(!root.join("evidence.jsonl").exists());
        }
    }
}
