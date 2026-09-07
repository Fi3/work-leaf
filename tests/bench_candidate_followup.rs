use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use serde_json::{Value, json};
use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, AgentStreamEvent,
    ChatMessage, CommandChat, MessageRole,
};

mod temp_cleanup;

const ORIGINAL: &str = "Implement λ parsing. Preserve literal Original feature request: text.";
const REPLACEMENT: &str = "Implement the second exact feature request; preserve Ω.";
const RESUPPLY: &str = "\n\nOriginal feature request (unchanged from launch):\n";

fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "work-leaf-candidate-followup-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    temp_cleanup::register(&path);
    path
}

fn exercise(schema: Option<&str>, condition: &str, mode: &str) -> (Value, Vec<Value>) {
    let root = root();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "candidate_followup_child",
            "--ignored",
            "--nocapture",
        ])
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT")
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST")
        .env_remove("WORK_LEAF_BENCH_RUN_ID")
        .env("CANDIDATE_FOLLOWUP_ROOT", &root)
        .env("CANDIDATE_FOLLOWUP_MODE", mode);
    if let Some(schema) = schema {
        let manifest = root.join("manifest.json");
        fs::write(
            &manifest,
            serde_json::to_vec(&json!({
                "schema": schema, "condition": condition, "run_id": "followup-fixture",
                "evidence_path": root.join("evidence.jsonl"),
            }))
            .unwrap(),
        )
        .unwrap();
        command
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", manifest)
            .env("WORK_LEAF_BENCH_RUN_ID", "followup-fixture");
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "child failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let result = serde_json::from_slice(&fs::read(root.join("result.json")).unwrap()).unwrap();
    let evidence = fs::read_to_string(root.join("evidence.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (result, evidence)
}

fn fix_prompt(result: &Value) -> &str {
    result["sends"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row[1].as_str())
        .find(|prompt| prompt.starts_with("The reviewer found issues"))
        .expect("actual author fix delivery")
}

#[test]
fn review_fix_only_resupplies_the_exact_owned_launch_request() {
    for mode in ["streaming", "nonstreaming", "clone"] {
        let (baseline, _) = exercise(None, "", mode);
        let (variant, evidence) = exercise(
            Some("work-leaf-bench-experiment-v4"),
            "review-fix-request-resupply",
            mode,
        );
        if cfg!(feature = "bench-experiments") {
            let original = variant["original"].as_str().unwrap();
            assert_eq!(
                fix_prompt(&variant),
                format!("{}{RESUPPLY}{original}", fix_prompt(&baseline)),
                "{mode}",
            );
            let rows: Vec<_> = evidence
                .iter()
                .filter(|row| row["site"] == "review-fix-request")
                .collect();
            assert_eq!(rows.len(), 1, "{mode}");
            let row = rows[0];
            assert_eq!(row["original_prompt"], fix_prompt(&baseline));
            assert_eq!(row["candidate_prompt"], fix_prompt(&variant));
            assert_eq!(row["changed"], true);
            let component = &row["components"][0];
            let base_end = fix_prompt(&baseline).len();
            assert_eq!(component["baseline_start"], base_end);
            assert_eq!(component["baseline_end"], base_end);
            assert_eq!(component["inline_start"], base_end);
            assert_eq!(component["inline_end"], fix_prompt(&variant).len());
            assert_eq!(
                row["metadata"]["original_request_source"],
                "prepared-agent-launch.prompt"
            );
            assert_eq!(row["metadata"]["original_request_bytes"], original.len());
            let start = row["metadata"]["candidate_request_start"].as_u64().unwrap() as usize;
            let end = row["metadata"]["candidate_request_end"].as_u64().unwrap() as usize;
            assert_eq!(&fix_prompt(&variant)[start..end], original);
            let mut expected = baseline.clone();
            let sends = expected["sends"].as_array_mut().unwrap();
            let target = sends
                .iter_mut()
                .find(|row| {
                    row[1]
                        .as_str()
                        .unwrap()
                        .starts_with("The reviewer found issues")
                })
                .unwrap();
            target[1] = json!(fix_prompt(&variant));
            assert_eq!(
                variant, expected,
                "all other provider deliveries stay identical"
            );
        } else {
            assert_eq!(variant, baseline);
            assert!(evidence.is_empty());
        }
    }
}

#[test]
fn other_candidates_and_legacy_schemas_preserve_all_deliveries() {
    let (baseline, _) = exercise(None, "", "clone");
    for (schema, condition) in [
        ("work-leaf-bench-experiment-v1", "control"),
        ("work-leaf-bench-experiment-v2", "control"),
        ("work-leaf-bench-experiment-v3", "control"),
        ("work-leaf-bench-experiment-v4", "requested-repeat-full"),
        ("work-leaf-bench-experiment-v4", "unified-diff-preferred"),
    ] {
        let (result, evidence) = exercise(Some(schema), condition, "clone");
        assert_eq!(result, baseline, "{schema}/{condition}");
        let rows: Vec<_> = evidence
            .iter()
            .filter(|row| row["site"] == "review-fix-request")
            .collect();
        if cfg!(feature = "bench-experiments") && schema.ends_with("v4") {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0]["changed"], false);
            assert_eq!(
                rows[0]["candidate_prompt"],
                format!("{}{RESUPPLY}{ORIGINAL}", fix_prompt(&baseline)),
            );
        } else {
            assert!(rows.is_empty(), "old schemas retain their evidence sites");
        }
    }
}

#[test]
fn successful_relaunch_replaces_provenance_and_failed_relaunch_retains_it() {
    for (mode, wanted) in [("relaunch", REPLACEMENT), ("failed-relaunch", ORIGINAL)] {
        let (result, evidence) = exercise(
            Some("work-leaf-bench-experiment-v4"),
            "review-fix-request-resupply",
            mode,
        );
        if cfg!(feature = "bench-experiments") {
            assert!(fix_prompt(&result).ends_with(&format!("{RESUPPLY}{wanted}")));
            assert_eq!(
                evidence
                    .iter()
                    .filter(|row| row["site"] == "review-fix-request")
                    .count(),
                1,
            );
        } else {
            assert!(!fix_prompt(&result).contains(RESUPPLY));
        }
    }
}

#[test]
fn absent_or_failed_launch_provenance_never_uses_backend_transcript_as_fallback() {
    for mode in ["missing", "failed-first"] {
        let (result, _) = exercise(
            Some("work-leaf-bench-experiment-v4"),
            "review-fix-request-resupply",
            mode,
        );
        if cfg!(feature = "bench-experiments") {
            assert!(!result["ok"].as_bool().unwrap());
            assert!(
                result["error"]
                    .as_str()
                    .unwrap()
                    .contains("original launch request")
            );
            assert!(result["sends"].as_array().unwrap().is_empty());
        } else {
            assert!(result["ok"].as_bool().unwrap());
        }
    }
}

#[derive(Default)]
struct BackendState {
    sessions: BTreeMap<AgentId, AgentSession>,
    launches: Vec<(String, String)>,
    sends: Vec<(String, String)>,
    interruptible_launches: usize,
    fail_next_launch: bool,
}

#[derive(Clone, Default)]
struct Backend(Arc<Mutex<BackendState>>);

impl AgentBackend for Backend {
    fn launch(&mut self, launch: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut state = self.0.lock().unwrap();
        if std::mem::take(&mut state.fail_next_launch) {
            return Err(AgentError::Io(std::io::Error::other(
                "synthetic launch failure",
            )));
        }
        state
            .launches
            .push((launch.id.to_string(), launch.prompt.clone()));
        let reply = if launch.id.as_str().starts_with("review-") {
            "FINDINGS\n- Preserve the literal Original feature request: marker correctly."
        } else {
            "@work-leaf done"
        };
        let mut session = AgentSession::new(launch);
        session.push_message(MessageRole::Agent, reply);
        state.sessions.insert(session.id.clone(), session.clone());
        Ok(session)
    }

    fn launch_streaming_interruptible(
        &mut self,
        launch: AgentLaunch,
        _sink: &mut dyn FnMut(AgentStreamEvent),
        _should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<AgentSession, AgentError> {
        self.0.lock().unwrap().interruptible_launches += 1;
        self.launch(launch)
    }

    fn send(&mut self, id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let mut state = self.0.lock().unwrap();
        state.sends.push((id.to_string(), prompt.to_string()));
        let reply = if id.as_str().starts_with("review-") {
            "NO_FINDINGS"
        } else {
            "@work-leaf done"
        };
        if let Some(session) = state.sessions.get_mut(id) {
            session.push_message(MessageRole::User, prompt);
            session.push_message(MessageRole::Agent, reply);
        }
        Ok(ChatMessage::new(MessageRole::Agent, reply))
    }

    fn session(&self, id: &AgentId) -> Option<AgentSession> {
        self.0.lock().unwrap().sessions.get(id).cloned()
    }
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .env("GIT_AUTHOR_DATE", "2020-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore]
fn candidate_followup_child() {
    let root = PathBuf::from(std::env::var_os("CANDIDATE_FOLLOWUP_ROOT").unwrap());
    let mode = std::env::var("CANDIDATE_FOLLOWUP_MODE").unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.name", "Fixture"]);
    git(&root, &["config", "user.email", "fixture@example.invalid"]);
    fs::write(root.join("value.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, &["add", "value.rs"]);
    git(&root, &["commit", "-qm", "ADD fixture base"]);
    let backend = Backend::default();
    let mut chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(3);
    let id = AgentId::new("user-1").unwrap();
    let launch =
        |prompt: &str| AgentLaunch::new(id.clone(), AgentKind::Codex, "feature-label", prompt);
    let mut original = ORIGINAL.to_string();
    let missing = mode == "missing" || mode == "failed-first";
    if missing {
        backend
            .0
            .lock()
            .unwrap()
            .sessions
            .insert(id.clone(), AgentSession::new(launch("unowned transcript")));
        if mode == "failed-first" {
            backend.0.lock().unwrap().fail_next_launch = true;
            assert!(
                chat.launch_prepared_agent_streaming(launch(ORIGINAL), &mut |_| {})
                    .is_err()
            );
        }
    } else if mode == "nonstreaming" {
        chat.handle_line(&format!("new {ORIGINAL}")).unwrap();
        original = backend.0.lock().unwrap().launches[0].1.clone();
    } else if mode == "clone" {
        let mut launch_worker = chat.clone();
        let review_worker = chat.clone();
        launch_worker
            .launch_prepared_agent_streaming_with_ids(launch(ORIGINAL), &mut |_, _| {})
            .unwrap();
        chat = review_worker;
    } else {
        chat.launch_prepared_agent_streaming_with_ids(launch(ORIGINAL), &mut |_, _| {})
            .unwrap();
        if mode == "relaunch" || mode == "failed-relaunch" {
            backend.0.lock().unwrap().fail_next_launch = mode == "failed-relaunch";
            let result =
                chat.launch_prepared_agent_streaming_with_ids(launch(REPLACEMENT), &mut |_, _| {});
            assert_eq!(result.is_ok(), mode == "relaunch");
        }
    }
    if !missing {
        // The owner record must survive a provider's transcript projection changing.
        backend
            .0
            .lock()
            .unwrap()
            .sessions
            .get_mut(&id)
            .unwrap()
            .messages[0]
            .text = "untrusted transcript replacement".to_string();
        chat.send_to_agent(&id, "ordinary follow-up; do not resupply request")
            .unwrap();
    }
    fs::write(root.join("value.rs"), "pub fn value() -> u8 { 2 }\n").unwrap();
    git(&root, &["add", "value.rs"]);
    git(
        &root,
        &[
            "commit",
            "-qm",
            "UPDATE fixture behavior",
            "-m",
            "Agent-ID: user-1\nFeature: feature-label\nReason: behavior\nContext: exact fixture scope",
        ],
    );
    let result = chat.handle_line("review");
    let state = backend.0.lock().unwrap();
    fs::write(
        root.join("result.json"),
        serde_json::to_vec(&json!({
            "ok": result.is_ok(), "error": result.err().map(|e| e.to_string()),
            "original": original, "launches": state.launches, "sends": state.sends,
            "interruptible_launches": state.interruptible_launches,
        }))
        .unwrap(),
    )
    .unwrap();
}
