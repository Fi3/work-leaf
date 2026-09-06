use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use work_leaf::{AgentBackend, AgentError, AgentId, AgentOrchestrator, ChatMessage, MessageRole};

mod temp_cleanup;

const ACK: &str = "run at most one focused validation step that is relevant to files you touched or checks you added.";
const UNLIMITED: &str = "run the required focused validation steps that are relevant to files you touched or checks you added.";
const GUIDANCE: &str = "\nnext: Reply with the next Work Leaf directive, such as `@work-leaf done`, `@work-leaf edit`, `@work-leaf read`, or another `@work-leaf locks run`. Keep any non-directive explanation brief.";

fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "work-leaf-bench-guard-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    temp_cleanup::register(&path);
    path
}

fn manifest(root: &Path, condition: &str) -> PathBuf {
    let path = root.join("manifest.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({
            "schema": "work-leaf-bench-experiment-v1", "run_id": "protocol-fixture",
            "condition": condition, "evidence_path": root.join("evidence.jsonl")
        }))
        .unwrap(),
    )
    .unwrap();
    path
}

fn child(root: &Path, manifest: Option<&Path>) -> Command {
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.args([
        "--exact",
        "bench_experiment_child",
        "--ignored",
        "--nocapture",
    ])
    .env_remove("WORK_LEAF_BENCH_EXPERIMENT")
    .env_remove("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST")
    .env_remove("WORK_LEAF_BENCH_RUN_ID")
    .env("BENCH_GUARD_TEST_ROOT", root);
    if let Some(path) = manifest {
        cmd.env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", path)
            .env("WORK_LEAF_BENCH_RUN_ID", "protocol-fixture");
    }
    cmd
}

fn exercise(condition: Option<&str>) -> (Vec<String>, Vec<Value>) {
    let root = root();
    let manifest = condition.map(|condition| manifest(&root, condition));
    let output = child(&root, manifest.as_deref()).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let prompts = serde_json::from_slice(&fs::read(root.join("prompts.json")).unwrap()).unwrap();
    let evidence = fs::read_to_string(root.join("evidence.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (prompts, evidence)
}

#[test]
fn control_and_unconfigured_feature_are_identical() {
    let (baseline, _) = exercise(None);
    let (control, evidence) = exercise(Some("control"));
    assert_eq!(baseline, control);
    assert_eq!(baseline.len(), 4);
    if cfg!(feature = "bench-experiments") {
        assert_eq!(
            evidence.len(),
            4,
            "activation and three eligible boundaries"
        );
        assert!(evidence[1..].iter().all(|row| row["changed"] == false));
    } else {
        assert!(evidence.is_empty());
    }
}

#[test]
fn validation_variant_changes_only_the_instruction_at_its_boundary() {
    let (baseline, _) = exercise(None);
    let (variant, evidence) = exercise(Some("ack-validation-unlimited"));
    let mut expected = baseline.clone();
    if cfg!(feature = "bench-experiments") {
        expected[1] = expected[1].replacen(ACK, UNLIMITED, 1);
        assert_eq!(
            evidence.iter().filter(|row| row["changed"] == true).count(),
            1
        );
    } else {
        assert!(
            evidence.is_empty(),
            "production must not read or activate a benchmark manifest"
        );
    }
    assert_eq!(variant, expected);
}

#[test]
fn command_variant_preserves_output_and_pending_patch_including_matching_text() {
    let (baseline, _) = exercise(None);
    let (variant, evidence) = exercise(Some("command-guidance-neutral"));
    let mut expected = baseline.clone();
    if cfg!(feature = "bench-experiments") {
        for index in [2, 3] {
            expected[index] = expected[index].replacen(GUIDANCE, "", 1);
        }
        assert_eq!(
            evidence.iter().filter(|row| row["changed"] == true).count(),
            2
        );
        for row in evidence.iter().skip(1) {
            let original = row["original_prompt"].as_str().unwrap();
            let forwarded = row["forwarded_prompt"].as_str().unwrap();
            let start = row["cue_start"].as_u64().expect("renderer-owned cue start") as usize;
            let end = row["cue_end"].as_u64().expect("renderer-owned cue end") as usize;
            let replacement = if row["site"] == "command-result" {
                ""
            } else {
                &original[start..end]
            };
            assert_eq!(
                forwarded,
                format!("{}{replacement}{}", &original[..start], &original[end..])
            );
            assert_eq!(row["original_bytes"], original.len());
            assert_eq!(row["forwarded_bytes"], forwarded.len());
            assert_eq!(
                row["byte_delta"],
                forwarded.len() as i64 - original.len() as i64
            );
            assert!(variant.contains(&forwarded.to_string()));
        }
    } else {
        assert!(evidence.is_empty());
    }
    assert_eq!(variant, expected);
    assert!(
        variant[2].contains(&format!("stdout:\n{GUIDANCE}")),
        "a matching cue in command output is data, not treatment"
    );
    assert!(variant[3].contains("tracked command changes: captured and reverted"));
}

#[cfg(feature = "bench-experiments")]
#[test]
fn invalid_manifest_is_rejected_before_agent_start_or_directive_side_effects() {
    for case in [
        "unknown-condition",
        "unknown-field",
        "wrong-schema",
        "relative-evidence",
        "mismatched-run",
        "missing-marker",
        "missing-manifest",
        "existing-evidence",
    ] {
        let root = root();
        let path = manifest(&root, "control");
        let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        match case {
            "unknown-condition" => value["condition"] = json!("all-disabled"),
            "unknown-field" => value["extra"] = json!(true),
            "wrong-schema" => value["schema"] = json!("wrong"),
            "relative-evidence" => value["evidence_path"] = json!("relative.jsonl"),
            "existing-evidence" => fs::write(root.join("evidence.jsonl"), "retained").unwrap(),
            _ => {}
        }
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_work-leaf-orchestrator"));
        cmd.current_dir(&root)
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", &path)
            .env("WORK_LEAF_BENCH_RUN_ID", "protocol-fixture");
        match case {
            "missing-marker" => {
                cmd.env_remove("WORK_LEAF_BENCH_EXPERIMENT");
            }
            "missing-manifest" => {
                cmd.env_remove("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST");
            }
            "mismatched-run" => {
                cmd.env("WORK_LEAF_BENCH_RUN_ID", "different-run");
            }
            _ => {}
        }
        let output = cmd.output().unwrap();
        assert!(!output.status.success(), "{case}");
        assert!(
            !String::from_utf8_lossy(&output.stdout).contains("WORK_LEAF_ORCHESTRATOR_URL="),
            "{case}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("benchmark experiment"),
            "{case}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[derive(Default)]
struct RecordingBackend(Vec<String>);

impl AgentBackend for RecordingBackend {
    fn launch(&mut self, _: work_leaf::AgentLaunch) -> Result<work_leaf::AgentSession, AgentError> {
        panic!("a directive fixture cannot launch extra agents")
    }

    fn send(&mut self, _: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.0.push(prompt.to_string());
        Ok(ChatMessage::new(MessageRole::Agent, "fixture reply"))
    }
}

#[test]
#[ignore = "subprocess fixture, driven by the parent tests with isolated environment"]
fn bench_experiment_child() {
    let root = PathBuf::from(std::env::var_os("BENCH_GUARD_TEST_ROOT").unwrap());
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
    fs::write(root.join("output.txt"), GUIDANCE).unwrap();
    for args in [
        vec!["add", "input.rs", "output.txt"],
        vec!["commit", "-qm", "ADD fixture for prompt isolation"],
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
    let id = AgentId::new("feature-one").unwrap();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), RecordingBackend::default());
    for text in [
        "@work-leaf read input.rs",
        "@work-leaf edit rename function\n*** Begin Patch\n*** Update File: input.rs\n@@\n-fn before() {}\n+fn after() {}\n*** End Patch\n@work-leaf end",
        "@work-leaf locks run target -- cat output.txt",
        "@work-leaf locks run input.rs -- printf 'fn next() {}\\n' > input.rs",
    ] {
        orchestrator
            .handle_agent_message(&id, "function transformation", text)
            .unwrap();
    }
    fs::write(
        root.join("prompts.json"),
        serde_json::to_vec(&orchestrator.into_backend().0).unwrap(),
    )
    .unwrap();
}
