use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use serde_json::{Value, json};
use work_leaf::review::AgentCommit;
use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, ChatMessage,
    CommandChat, GitHistory, MessageRole,
};

mod temp_cleanup;

const SCHEMA: &str = "work-leaf-bench-experiment-v5";
const NATIVE: &str = "review-evidence-native";
const INLINE: &str = "review-evidence-inline";
const SOURCE_ID: &str = "context-author";
const REVIEWER_ID: &str = "review-context-author";

fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "work-leaf-review-evidence-integration-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    temp_cleanup::register(&root);
    root
}

fn exercise(schema: Option<&str>, condition: &str, mode: &str) -> (Value, Vec<Value>) {
    let root = root();
    fs::create_dir(root.join("repo")).unwrap();
    fs::create_dir(root.join("opaque")).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap());
    child
        .args([
            "--exact",
            "review_evidence_child",
            "--ignored",
            "--nocapture",
        ])
        .env("REVIEW_EVIDENCE_TEST_ROOT", &root)
        .env("REVIEW_EVIDENCE_TEST_MODE", mode)
        .env("WORK_LEAF_CONTEXT_BUNDLE_DIR", root.join("bundles"))
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT")
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST")
        .env_remove("WORK_LEAF_BENCH_RUN_ID");
    if let Some(schema) = schema {
        let mut manifest = json!({
            "schema": schema, "condition": condition, "run_id": "review-evidence-fixture",
            "evidence_path": root.join("trace.jsonl"),
        });
        if schema == SCHEMA {
            manifest["review_evidence_root"] = json!(root.join("opaque"));
        }
        let path = root.join("manifest.json");
        fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        child
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", path)
            .env("WORK_LEAF_BENCH_RUN_ID", "review-evidence-fixture");
    }
    let output = child.output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let mut result: Value =
        serde_json::from_slice(&fs::read(root.join("result.json")).unwrap()).unwrap();
    result["root"] = json!(root);
    let trace = fs::read_to_string(root.join("trace.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (result, trace)
}

fn review_deliveries(result: &Value) -> Vec<&Value> {
    result["deliveries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["prompt"]
                .as_str()
                .unwrap()
                .starts_with("Review the full patch scope")
        })
        .collect()
}

#[test]
fn normal_review_prompts_match_the_prepatch_formatter_for_all_session_shapes() {
    for mode in ["missing", "empty", "populated", "loop"] {
        let (result, trace) = exercise(None, "", mode);
        assert!(result["ok"].as_bool().unwrap(), "{}", result["error"]);
        assert!(trace.is_empty());
        let reviews = review_deliveries(&result);
        assert_eq!(reviews.len(), 2);
        assert_eq!(reviews[0]["route"], "launch");
        assert_eq!(reviews[1]["route"], "send");
        for (review, expected) in reviews.iter().zip(result["expected"].as_array().unwrap()) {
            assert_eq!(review["prompt"], expected["prompt"], "{mode}");
            assert_eq!(review["agent_id"], REVIEWER_ID);
        }
        assert_eq!(result["source_snapshot_reads"], 2);
        assert_eq!(result["opaque_after_drop"], json!({}));
    }
}

#[test]
fn selected_review_context_is_archived_and_only_its_owned_interpolation_changes() {
    for mode in ["missing", "empty", "populated", "loop"] {
        let (result, trace) = exercise(Some(SCHEMA), NATIVE, mode);
        assert!(result["ok"].as_bool().unwrap(), "{}", result["error"]);
        let reviews = review_deliveries(&result);
        assert_eq!(reviews.len(), 2);
        for (review, expected) in reviews.iter().zip(result["expected"].as_array().unwrap()) {
            let prompt = review["prompt"].as_str().unwrap();
            let original = expected["prompt"].as_str().unwrap();
            let prefix = expected["prefix"].as_str().unwrap();
            let suffix = expected["suffix"].as_str().unwrap();
            if cfg!(feature = "bench-experiments") {
                assert_ne!(prompt, original);
                assert!(prompt.starts_with(prefix));
                assert!(prompt.ends_with(suffix));
                let context = expected["context"].as_str().unwrap();
                assert!(
                    result["opaque_after_drop"]
                        .as_object()
                        .unwrap()
                        .values()
                        .any(|v| v == context)
                );
                assert!(trace.iter().any(|row| row["original_prompt"] == original));
            } else {
                assert_eq!(prompt, original);
                assert!(trace.is_empty());
                assert_eq!(result["opaque_after_drop"], json!({}));
            }
        }
        assert_eq!(result["source_snapshot_reads"], 2);
        assert_eq!(result["first_archive_unchanged"], true);
        if cfg!(feature = "bench-experiments") {
            let rows: Vec<_> = trace
                .iter()
                .filter(|row| row["event"] == "review-context")
                .collect();
            assert_eq!(rows.len(), 2);
            for (index, row) in rows.iter().enumerate() {
                let expected = &result["expected"][index];
                let original = row["original_prompt"].as_str().unwrap();
                let candidate = row["candidate_prompt"].as_str().unwrap();
                let start = row["context_start"].as_u64().unwrap() as usize;
                let end = row["context_end"].as_u64().unwrap() as usize;
                let new_start = row["candidate_start"].as_u64().unwrap() as usize;
                let new_end = row["candidate_end"].as_u64().unwrap() as usize;
                assert_eq!(start, expected["prefix"].as_str().unwrap().len());
                assert_eq!(&original[start..end], expected["context"].as_str().unwrap());
                assert_eq!(&original[..start], &candidate[..new_start]);
                assert_eq!(&original[end..], &candidate[new_end..]);
                assert_eq!(row["forwarded_prompt"], candidate);
                assert_eq!(row["selected_candidate"], "candidate");
                assert_eq!(row["source_agent_id"], SOURCE_ID);
                assert_eq!(row["reviewer_id"], REVIEWER_ID);
                assert_eq!(row["archive"]["kind"], "review-source-context");
                assert_eq!(row["archive"]["bytes"], end - start);
                let archived =
                    fs::read_to_string(row["archive"]["path"].as_str().unwrap()).unwrap();
                assert_eq!(archived, &original[start..end]);
                assert_eq!(row["archive"]["target_commit"], row["target_commit"]);
            }
            assert_ne!(rows[0]["archive"]["path"], rows[1]["archive"]["path"]);
        }
    }
}

fn normalized_read_prompts(result: &Value) -> Vec<String> {
    result["deliveries"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["prompt"].as_str())
        .filter(|prompt| prompt.starts_with("work-leaf file text"))
        .map(|prompt| {
            prompt
                .replace(result["bundle_dir"].as_str().unwrap(), "<BUNDLES>")
                .replace(result["opaque_probe"].as_str().unwrap(), "<OPAQUE>")
                .replace(result["root"].as_str().unwrap(), "<ROOT>")
        })
        .collect()
}

#[test]
fn review_archives_do_not_change_ordinary_bundles_repeats_or_outside_root_reads() {
    let (baseline, _) = exercise(None, "", "reads");
    let expected = normalized_read_prompts(&baseline);
    assert_eq!(expected.len(), 7);
    assert!(expected[0].contains("bundle-0.md"));
    assert!(expected[1].contains("bundle-1.md"));
    assert!(expected[1].contains("Repeated file reads with changes"));
    assert!(expected[2].contains("Unavailable file text"));
    assert!(expected[3].contains("small source λ"));
    assert!(expected[4].contains("Repeated file reads unchanged"));
    assert!(expected[5].contains("# Work Leaf Context Bundle"));
    assert!(expected[6].contains("bundle-2.md"));
    for condition in [INLINE, NATIVE] {
        let (result, _) = exercise(Some(SCHEMA), condition, "reads");
        assert!(result["ok"].as_bool().unwrap(), "{}", result["error"]);
        assert_eq!(normalized_read_prompts(&result), expected);
        assert_eq!(result["bundles"], baseline["bundles"]);
        assert_eq!(result["bundles_after_drop"], json!({}));
        assert_eq!(result["first_archive_unchanged"], true);
        let reads = normalized_read_prompts(&result);
        assert!(!reads[2].contains("public evidence"));
        assert!(!reads[3].contains("public evidence"));
    }
}

#[test]
#[cfg(feature = "bench-experiments")]
fn review_archive_publication_error_prevents_reused_reviewer_delivery() {
    for condition in [INLINE, NATIVE] {
        let (result, _) = exercise(Some(SCHEMA), condition, "failed-second");
        assert_eq!(result["ok"], false);
        assert_eq!(review_deliveries(&result).len(), 1);
        assert_eq!(result["first_archive_unchanged"], true);
        assert!(!result["opaque_after_drop"].as_object().unwrap().is_empty());
    }
}

#[test]
fn inline_and_legacy_conditions_preserve_whole_review_fix_and_recheck_deliveries() {
    let (baseline, _) = exercise(None, "", "loop");
    for (schema, condition) in [
        (SCHEMA, INLINE),
        ("work-leaf-bench-experiment-v1", "control"),
        ("work-leaf-bench-experiment-v2", "control"),
        ("work-leaf-bench-experiment-v3", "control"),
        ("work-leaf-bench-experiment-v4", "unified-diff-preferred"),
    ] {
        let (result, trace) = exercise(Some(schema), condition, "loop");
        assert!(result["ok"].as_bool().unwrap(), "{}", result["error"]);
        assert_eq!(
            result["deliveries"], baseline["deliveries"],
            "{schema}/{condition}"
        );
        assert_eq!(result["source_snapshot_reads"], 2);
        if cfg!(feature = "bench-experiments") && schema == SCHEMA {
            let rows: Vec<_> = trace
                .iter()
                .filter(|row| row["event"] == "review-context")
                .collect();
            assert_eq!(rows.len(), 2);
            for (row, expected) in rows.iter().zip(result["expected"].as_array().unwrap()) {
                assert_eq!(row["selected_candidate"], "baseline");
                assert_eq!(row["original_prompt"], expected["prompt"]);
                assert_eq!(row["forwarded_prompt"], expected["prompt"]);
                assert_ne!(row["candidate_prompt"], row["original_prompt"]);
                assert_eq!(
                    fs::read_to_string(row["archive"]["path"].as_str().unwrap()).unwrap(),
                    expected["context"].as_str().unwrap()
                );
            }
        } else {
            assert_eq!(result["opaque_after_drop"], json!({}));
            assert!(
                trace
                    .iter()
                    .all(|row| row["site"] != "review-source-context")
            );
        }
    }
    let (selected, _) = exercise(Some(SCHEMA), NATIVE, "loop");
    assert!(selected["ok"].as_bool().unwrap(), "{}", selected["error"]);
    let nonreviews = |value: &Value| {
        value["deliveries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| {
                !row["prompt"]
                    .as_str()
                    .unwrap()
                    .starts_with("Review the full patch scope")
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(nonreviews(&selected), nonreviews(&baseline));
}

#[derive(Default)]
struct State {
    sessions: BTreeMap<AgentId, AgentSession>,
    replies: VecDeque<String>,
    deliveries: Vec<Value>,
    source_snapshot_reads: usize,
}

#[derive(Clone, Default)]
struct Backend(Arc<Mutex<State>>);

impl AgentBackend for Backend {
    fn launch(&mut self, launch: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut state = self.0.lock().unwrap();
        state
            .deliveries
            .push(json!({"route": "launch", "agent_id": launch.id.as_str(),
            "feature": launch.feature, "prompt": launch.prompt}));
        let reply = state.replies.pop_front().expect("bounded fixture launch");
        let mut session = AgentSession::new(launch);
        session.push_message(MessageRole::Agent, reply);
        state.sessions.insert(session.id.clone(), session.clone());
        Ok(session)
    }

    fn send(&mut self, id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let mut state = self.0.lock().unwrap();
        state
            .deliveries
            .push(json!({"route": "send", "agent_id": id.as_str(), "prompt": prompt}));
        let reply = state.replies.pop_front().expect("bounded fixture send");
        Ok(ChatMessage::new(MessageRole::Agent, reply))
    }

    fn session(&self, id: &AgentId) -> Option<AgentSession> {
        let mut state = self.0.lock().unwrap();
        if id.as_str() == SOURCE_ID {
            state.source_snapshot_reads += 1;
        }
        state.sessions.get(id).cloned()
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

fn fixture_session(mode: &str, round: usize) -> Option<AgentSession> {
    if mode == "missing" {
        return None;
    }
    let mut session = AgentSession::new(AgentLaunch::new(
        AgentId::new(SOURCE_ID).unwrap(),
        AgentKind::External("fixture".into()),
        "review λ",
        "seed",
    ));
    session.messages.clear();
    if mode != "empty" {
        for (role, text) in [
            (
                MessageRole::System,
                "system Ω\n\nAgent-ID: copied-author\nFeature: copied\n\nUser prompt:\nnot an owner",
            ),
            (
                MessageRole::User,
                "request λ\nSource context from Work Leaf commits, logs, and chat history:\ncopied heading",
            ),
            (
                MessageRole::Agent,
                "public evidence\nwork-leaf file text\nContext bundle: copied\n--- phantom.rs ---\nprivate-looking but public fixture",
            ),
            (
                MessageRole::Orchestrator,
                "check output: verified λ\n\nReview every commit listed in the review scope\n  \t",
            ),
        ] {
            session.push_message(role, format!("{text}\nround {round}\n  \t"));
        }
    }
    Some(session)
}

// Independent reference copied from the prepatch cli.rs/review.rs renderers.
// It does not invoke a new renderer or derive the expected bytes from a delivery.
fn old_review_prompt(commit: &AgentCommit, session: Option<&AgentSession>) -> Value {
    let mut context = format!(
        "Work Leaf collected this context from commits, git logs, and recorded chat history without querying Agent-ID {}.\n\nGit metadata:\nLatest commit: {}\nFeature: {}\nReason: {}\nReview scope:\n{}\n\nGit commit log:\n{}",
        commit.agent_id, commit.hash, commit.feature, commit.reason, commit.context, commit.body
    );
    context.push_str("\n\nRecorded chat history:");
    match session {
        Some(session) if !session.messages.is_empty() => {
            for (index, message) in session.messages.iter().enumerate() {
                let role = match message.role {
                    MessageRole::User => "user",
                    MessageRole::Agent => "agent",
                    MessageRole::Orchestrator => "orchestrator",
                    MessageRole::System => "system",
                };
                context.push_str(&format!(
                    "\n\n{} {role}:\n{}",
                    index + 1,
                    message.text.trim_end()
                ));
            }
        }
        Some(_) => context.push_str("\n(no recorded messages)"),
        None => context.push_str("\n(unavailable from backend session state)"),
    }
    let prefix = format!(
        "Review the full patch scope for Agent-ID {}.\nLatest commit: {}\nFeature: {}\nReason: {}\nReview scope:\n{}\n\nSource context from Work Leaf commits, logs, and chat history:\n",
        commit.agent_id, commit.hash, commit.feature, commit.reason, commit.context
    );
    let suffix = "\n\nReview every commit listed in the review scope and reply with NO_FINDINGS if there are no findings. Otherwise reply with FINDINGS followed by the issues.\n\nDocumentation and plain-text updates are deferred to the linearize agent. Do not treat missing docs, README, changelog, markdown, txt, or other prose-only updates as findings against this patch agent; review the code and behavior that the patch agent changed.\n\nFor agent-facing changes, missing required real-agent verification is a finding unless the source context includes the exact real-agent scenario and visible result, or the exact pre-agent blocker. If you report missing verification, state the precise evidence that would resolve it. When the patch agent responds with verification evidence or a blocker rather than code, evaluate that evidence instead of requiring another patch.";
    json!({"prompt": format!("{prefix}{context}{suffix}"), "context": context, "prefix": prefix, "suffix": suffix})
}

fn files(root: &Path) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    if !root.is_dir() {
        return result;
    }
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            result.insert(
                entry.file_name().to_str().unwrap().to_string(),
                fs::read_to_string(entry.path()).unwrap(),
            );
        } else if entry.file_type().unwrap().is_dir() {
            for (name, body) in files(&entry.path()) {
                result.insert(
                    format!("{}/{name}", entry.file_name().to_str().unwrap()),
                    body,
                );
            }
        }
    }
    result
}

#[test]
#[ignore = "isolated provider-free subprocess fixture"]
fn review_evidence_child() {
    let root = PathBuf::from(std::env::var_os("REVIEW_EVIDENCE_TEST_ROOT").unwrap());
    let mode = std::env::var("REVIEW_EVIDENCE_TEST_MODE").unwrap();
    let repo = root.join("repo");
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.name", "Fixture"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    if mode == "reads" {
        fs::write(
            repo.join("large-a.txt"),
            "first source line λ\n".repeat(1000),
        )
        .unwrap();
        fs::write(
            repo.join("large-b.txt"),
            "second source line Ω\n".repeat(1000),
        )
        .unwrap();
        fs::write(repo.join("small.txt"), "small source λ\n").unwrap();
    }
    let backend = Backend::default();
    let mut chat = CommandChat::new(repo.clone(), backend.clone()).with_max_review_rounds(3);
    if mode == "loop" {
        // The older v4 fix adapter requires a genuinely owned successful launch,
        // even when its selected factor leaves the fix prompt unchanged.
        backend.0.lock().unwrap().replies = VecDeque::from(["@work-leaf done".into()]);
        chat.launch_prepared_agent_streaming(
            AgentLaunch::new(
                AgentId::new(SOURCE_ID).unwrap(),
                AgentKind::Codex,
                "review λ",
                "original request λ",
            ),
            &mut |_| {},
        )
        .unwrap();
    }
    let mut expected = Vec::new();
    let mut previous = None;
    let mut first_archive = BTreeMap::new();
    let mut error = None;
    for round in 0..2 {
        if mode == "reads" && round == 1 {
            fs::write(
                repo.join("large-a.txt"),
                "first source line λ\n".repeat(1000) + "new tail Ω\n",
            )
            .unwrap();
        }
        fs::write(
            repo.join("value.rs"),
            format!("pub const VALUE: u8 = {};\n", round + 1),
        )
        .unwrap();
        git(&repo, &["add", "value.rs"]);
        git(
            &repo,
            &[
                "commit",
                "-qm",
                "UPDATE fixture value",
                "-m",
                "Agent-ID: context-author\nFeature: review λ\nReason: preserve Ω\nContext: exact source scope",
            ],
        );
        let id = AgentId::new(SOURCE_ID).unwrap();
        let session = fixture_session(&mode, round);
        let commit = GitHistory::new(repo.clone())
            .agent_review_commit(&id, previous.as_deref())
            .unwrap()
            .unwrap();
        expected.push(old_review_prompt(&commit, session.as_ref()));
        previous = Some(commit.hash);
        {
            let mut state = backend.0.lock().unwrap();
            if let Some(session) = session {
                state.sessions.insert(id, session);
            }
            state.replies = if mode == "reads" {
                let request = if round == 0 {
                    "@work-leaf read large-a.txt"
                } else {
                    "@work-leaf read large-a.txt large-b.txt"
                };
                VecDeque::from([request.into(), "NO_FINDINGS".into()])
            } else if mode == "loop" && round == 0 {
                VecDeque::from([
                    "FINDINGS\n- Missing precise verification evidence.".into(),
                    "@work-leaf done".into(),
                    "NO_FINDINGS".into(),
                ])
            } else {
                VecDeque::from(["NO_FINDINGS".into()])
            };
        }
        if let Err(failure) = chat.handle_line("review") {
            error = Some(failure.to_string());
            break;
        }
        if round == 0 {
            first_archive = files(&root.join("opaque"));
            if mode == "failed-second" {
                fs::rename(root.join("opaque"), root.join("opaque-retained")).unwrap();
                fs::write(root.join("opaque"), "fixture obstruction").unwrap();
            }
        }
    }
    let mut opaque_probe = root.join("unused-probe");
    if mode == "reads" && error.is_none() {
        opaque_probe = first_archive
            .iter()
            .find(|(_, body)| body.as_str() == expected[0]["context"].as_str().unwrap())
            .map(|(name, _)| root.join("opaque").join(name))
            .unwrap_or_else(|| {
                let path = root.join("outside-review-fixture");
                fs::write(&path, expected[0]["context"].as_str().unwrap()).unwrap();
                path
            });
        for request in [
            format!("@work-leaf read {}", opaque_probe.display()),
            format!("@work-leaf read small.txt {}", opaque_probe.display()),
        ] {
            backend.0.lock().unwrap().replies = VecDeque::from([request, "NO_FINDINGS".into()]);
            chat.send_to_agent(
                &AgentId::new(REVIEWER_ID).unwrap(),
                "fixture ordinary read routing",
            )
            .unwrap();
        }
        let bundle = backend
            .0
            .lock()
            .unwrap()
            .deliveries
            .iter()
            .filter_map(|row| row["prompt"].as_str())
            .filter(|prompt| prompt.starts_with("work-leaf file text\n"))
            .find_map(|prompt| {
                prompt
                    .lines()
                    .find_map(|line| line.strip_prefix("Context bundle: "))
            })
            .unwrap()
            .to_string();
        for reply in [
            "@work-leaf read large-a.txt".to_string(),
            format!("@work-leaf read {bundle}"),
            "@work-leaf done".to_string(),
            "@work-leaf read large-a.txt".to_string(),
        ] {
            backend.0.lock().unwrap().replies = if reply == "@work-leaf done" {
                VecDeque::from([reply])
            } else {
                VecDeque::from([reply, "NO_FINDINGS".into()])
            };
            chat.send_to_agent(
                &AgentId::new(REVIEWER_ID).unwrap(),
                "fixture tracker and bundle routing",
            )
            .unwrap();
        }
    }
    let bundles_root = root.join("bundles");
    let bundle_dir = fs::read_dir(&bundles_root)
        .ok()
        .and_then(|entries| {
            entries
                .flatten()
                .find(|entry| entry.file_type().unwrap().is_dir())
        })
        .map(|entry| entry.path());
    let bundles = bundle_dir
        .as_ref()
        .map(|path| files(path))
        .unwrap_or_default();
    drop(chat);
    let archives = files(&root.join(if mode == "failed-second" {
        "opaque-retained"
    } else {
        "opaque"
    }));
    let unchanged = first_archive
        .iter()
        .all(|(path, bytes)| archives.get(path) == Some(bytes));
    let state = backend.0.lock().unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec(&json!({
        "ok": error.is_none(), "error": error, "deliveries": state.deliveries,
        "expected": expected, "source_snapshot_reads": state.source_snapshot_reads,
        "first_archive_unchanged": unchanged, "opaque_after_drop": archives,
        "bundle_dir": bundle_dir.unwrap_or_else(|| root.join("unused-bundle-dir")),
        "bundles": bundles, "bundles_after_drop": files(&bundles_root), "opaque_probe": opaque_probe,
    })).unwrap()).unwrap();
}
