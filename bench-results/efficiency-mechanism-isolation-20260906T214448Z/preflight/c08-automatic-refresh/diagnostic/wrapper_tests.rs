//! Actual CommandChat/Git/shell execution with a scripted backend, never a provider.
use super::*;
use std::sync::atomic::AtomicUsize;
use std::time::{SystemTime, UNIX_EPOCH};

const FINAL_SOURCE: &str = "pub fn capped(value: u8, limit: u8) -> u8 { value.min(limit) }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn smaller_input() {\n        assert_eq!(super::capped(9, 4), 4);\n        assert_eq!(super::capped(2, 4), 2);\n        assert_eq!(super::capped(4, 4), 4);\n    }\n}\n";

fn edit(old: &str) -> String {
    let replacement: String = FINAL_SOURCE
        .lines()
        .map(|line| format!("+{line}\n"))
        .collect();
    format!(
        "@work-leaf edit implement and cover smaller input\n*** Begin Patch\n*** Update File: src/lib.rs\n@@\n-{}{replacement}*** End Patch\n@work-leaf end",
        old
    )
}

#[derive(Clone)]
struct Scripted {
    project: PathBuf,
    session: Arc<Mutex<Option<AgentSession>>>,
    inputs: Arc<Mutex<Vec<String>>>,
}

impl AgentBackend for Scripted {
    fn launch(&mut self, launch: AgentLaunch) -> Result<AgentSession, AgentError> {
        assert!(self.inputs.lock().unwrap().is_empty(), "only one launch");
        self.inputs.lock().unwrap().push(launch.prompt.clone());
        let mut session = AgentSession::new(launch);
        session.push_message(MessageRole::Agent, "@work-leaf read src/lib.rs");
        *self.session.lock().unwrap() = Some(session.clone());
        Ok(session)
    }

    fn send(&mut self, id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let mut inputs = self.inputs.lock().unwrap();
        let reply = match inputs.len() {
            1 => {
                assert_eq!(
                    prompt,
                    format!("work-leaf file text\n\n--- src/lib.rs ---\n{INITIAL_SOURCE}")
                );
                // The real host patch has committed, but the backend must still
                // receive the unmodified held read selected before that patch.
                assert_eq!(
                    fs::read_to_string(self.project.join("src/lib.rs")).unwrap(),
                    CURRENT_SOURCE
                );
                edit(INITIAL_SOURCE)
            }
            2 => {
                assert!(
                    prompt.contains("old block was not found in current file text"),
                    "{prompt}"
                );
                assert!(prompt.contains(&format!("current full text:\n{CURRENT_SOURCE}")));
                assert_eq!(
                    fs::read_to_string(self.project.join("src/lib.rs")).unwrap(),
                    CURRENT_SOURCE
                );
                edit(CURRENT_SOURCE)
            }
            3 => {
                assert!(
                    prompt.starts_with("work-leaf patch applied\nfiles: src/lib.rs\n"),
                    "{prompt}"
                );
                assert_eq!(
                    fs::read_to_string(self.project.join("src/lib.rs")).unwrap(),
                    FINAL_SOURCE
                );
                "@work-leaf locks run . -- cargo test --offline --locked".into()
            }
            4 => {
                assert!(prompt.starts_with("work-leaf command result\ncommand: cargo test --offline --locked\nstatus: 0\n"), "{prompt}");
                assert!(!prompt.contains("timed out: yes"));
                assert!(
                    prompt.contains("test tests::smaller_input ... ok"),
                    "{prompt}"
                );
                assert!(self.project.join("target/debug/deps").is_dir());
                "@work-leaf done".into()
            }
            n => panic!("unplanned scripted call {n}: {prompt}"),
        };
        inputs.push(prompt.to_owned());
        let mut session = self.session.lock().unwrap();
        let session = session.as_mut().unwrap();
        assert_eq!(&session.id, id);
        session.push_message(MessageRole::User, prompt);
        session.push_message(MessageRole::Agent, &reply);
        Ok(ChatMessage::new(MessageRole::Agent, reply))
    }

    fn session(&self, id: &AgentId) -> Option<AgentSession> {
        self.session
            .lock()
            .unwrap()
            .as_ref()
            .filter(|s| &s.id == id)
            .cloned()
    }
}

fn exercise(capped: bool) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = env::temp_dir().join(format!(
        "c08-wrapper-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    for name in ["project", "project/src", "evidence", "bundles"] {
        fs::create_dir(root.join(name)).unwrap();
    }
    let project = root.join("project");
    for (name, body) in [
        ("src/lib.rs", INITIAL_SOURCE),
        ("Cargo.toml", INITIAL_MANIFEST),
        ("Cargo.lock", INITIAL_LOCK),
        (".gitignore", "/target/\n"),
    ] {
        fs::write(project.join(name), body).unwrap();
    }
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Synthetic Fixture"],
        vec!["config", "user.email", "fixture@example.invalid"],
        vec!["add", "."],
        vec!["commit", "-qm", "initial accepted fixture"],
    ] {
        git(&project, &args).unwrap();
    }
    publish(
        &root.join("manifest.json"),
        &json!({"schema":"work-leaf-bench-experiment-v7",
        "run_id":"c08-scripted-wrapper", "condition":"automatic-changed-refresh-full",
        "evidence_path":root.join("trace.jsonl")}),
    )
    .unwrap();
    let cargo_dir = Path::new(env!("CARGO")).parent().unwrap();
    let output = Command::new("timeout")
        .args(["--kill-after=5s", "45s"])
        .arg(env::current_exe().unwrap())
        .args([
            "--exact",
            "wrapper_tests::actual_wrapper_child",
            "--ignored",
            "--nocapture",
        ])
        .env("C08_WRAPPER_ROOT", &root)
        .env("C08_WRAPPER_CAPPED", if capped { "1" } else { "0" })
        .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
        .env("WORK_LEAF_BENCH_RUN_ID", "c08-scripted-wrapper")
        .env(
            "WORK_LEAF_BENCH_EXPERIMENT_MANIFEST",
            root.join("manifest.json"),
        )
        .env("WORK_LEAF_CONTEXT_BUNDLE_DIR", root.join("bundles"))
        .env("PATH", format!("{}:/usr/bin:/bin", cargo_dir.display()))
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("WORK_LEAF_OBSERVER_CONFIG")
        .env_remove("WORK_LEAF_OBSERVER_PARENT_INVOCATION")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "retained synthetic fixture {}\n{}\n{}",
        root.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn actual_wrapper_recovery_checks_and_processes_done_with_distinct_host_owner() {
    exercise(false);
}

#[test]
fn two_round_nonconvergence_is_not_mistaken_for_qualified_return() {
    exercise(true);
}

#[test]
#[ignore = "provider-free subprocess invoked by the two actual-wrapper parent tests"]
fn actual_wrapper_child() {
    let root = PathBuf::from(env::var_os("C08_WRAPPER_ROOT").unwrap());
    let project = root.join("project");
    let state = Arc::new(Mutex::new(None));
    let inputs = Arc::new(Mutex::new(Vec::new()));
    let inner = Scripted {
        project: project.clone(),
        session: Arc::new(Mutex::new(None)),
        inputs: Arc::clone(&inputs),
    };
    let backend = BoundedBackend {
        inner,
        state: Arc::clone(&state),
        project: project.clone(),
        evidence: root.join("evidence"),
        cancelled: Arc::new(AtomicBool::new(false)),
    };
    let mut chat = diagnostic_chat(project.clone(), backend);
    let capped = env::var("C08_WRAPPER_CAPPED").unwrap() == "1";
    if capped {
        chat = chat.with_max_review_rounds(2);
    }
    let launch = chat
        .prepare_agent_launch(&["Implement smaller-input behavior and test it".into()])
        .unwrap();
    let author = launch.id.to_string();
    *state.lock().unwrap() = Some(State {
        budget: Budget::new(&author, 8).unwrap(),
        calls: Vec::new(),
        previous_reply: String::new(),
        mutation_attempted: false,
        accepted: None,
    });
    let actual = chat.launch_prepared_agent_streaming(launch, &mut |_| {});
    chat.shutdown_agents();
    let CommandChatResult::AgentLaunched {
        agent_id, reply, ..
    } = actual.unwrap()
    else {
        panic!("actual author launch result expected");
    };
    assert_eq!(agent_id.as_str(), author);
    let done = processed_done(&author, Some(&reply));
    let rows = frames(&root.join("trace.jsonl"), true).unwrap();
    let lock = state.lock().unwrap();
    let state = lock.as_ref().unwrap();
    let local = recovery_chain(&author, CURRENT_SOURCE, &state.calls, &rows);
    let accepted = state
        .accepted
        .as_ref()
        .expect("actual host mutation must be retained");
    let mutation_commit = accepted["commit"].as_str().unwrap();
    let metadata = git(&project, &["show", "-s", "--format=%B", mutation_commit]).unwrap();
    let mutation_owner = metadata
        .lines()
        .find_map(|l| l.strip_prefix("Agent-ID: "))
        .unwrap();
    assert_ne!(
        mutation_owner, author,
        "host stimulus must not use the provider author's commit identity"
    );
    assert_eq!(accepted["fixture_mutator_id"], mutation_owner);
    assert_eq!(accepted["author_id"], author);
    assert_eq!(accepted["source_before"], INITIAL_SOURCE);
    assert_eq!(accepted["source_after"], CURRENT_SOURCE);
    assert_eq!(accepted["commit"], accepted["after_head"]);
    assert_eq!(state.calls.len(), inputs.lock().unwrap().len());
    for (call, input) in state.calls.iter().zip(inputs.lock().unwrap().iter()) {
        assert_eq!(
            call["prompt"],
            input.as_str(),
            "wrapper must forward every whole input unchanged"
        );
    }
    if capped {
        assert_eq!(state.budget.calls(), 3);
        assert!(reply.contains("did not converge after 2 orchestrator rounds"));
        assert!(!done);
        assert!(local.is_err());
        assert_eq!(
            fs::read_to_string(project.join("src/lib.rs")).unwrap(),
            CURRENT_SOURCE
        );
        assert!(
            !project.join("target").exists(),
            "no ordinary check was executed"
        );
    } else {
        assert_eq!(state.budget.calls(), 5);
        assert!(done, "{reply}");
        assert!(local.is_ok(), "{local:?}");
        assert_eq!(
            fs::read_to_string(project.join("src/lib.rs")).unwrap(),
            FINAL_SOURCE
        );
        let final_metadata = git(&project, &["show", "-s", "--format=%B", "HEAD"]).unwrap();
        assert!(
            final_metadata
                .lines()
                .any(|l| l == format!("Agent-ID: {author}"))
        );
        assert!(project.join("target/debug/deps").is_dir());
        assert!(
            git(
                &project,
                &["status", "--porcelain", "--untracked-files=all"]
            )
            .unwrap()
            .is_empty()
        );
    }
    assert_eq!(
        state.budget.real_roles(),
        1,
        "one logical scripted role, zero provider calls"
    );
    publish(
        &root.join("WRAPPER-RESULT.json"),
        &json!({"provider_calls":0,"capped":capped,
        "outer_calls":state.budget.calls(),"processed_done":done,"local_chain":local.ok(),
        "mutation_commit":mutation_commit,"fixture_mutator_id":mutation_owner,"author_id":author}),
    )
    .unwrap();
}
