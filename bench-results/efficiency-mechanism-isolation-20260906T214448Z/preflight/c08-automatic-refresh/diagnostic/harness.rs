// Private qualification harness. The ignored real test requires separately
// frozen exact-cut admission; automatic tests use only the scripted backend.
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;
use std::time::{Duration, Instant};

use c08_automatic_refresh_diagnostic::{
    CURRENT_SOURCE, INITIAL_LOCK, INITIAL_MANIFEST, INITIAL_SOURCE, initial_read_owned,
    recovery_chain,
};
use c15_real_diagnostic_guards::{Budget, settled_turns};
use serde_json::{Value, json};
use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, ChatMessage, CodexBackend, CodexCommandConfig, CommandChat,
    CommandChatResult, FileLockTable, GitPatcher, MessageRole, PatchRequest, PromptPolicy,
    SandboxMode,
};

#[cfg(test)]
#[path = "wrapper_tests.rs"]
mod wrapper_tests;

const FIXTURE_MUTATOR_ID: &str = "qualification-host-mutator";

fn error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

fn publish(path: &Path, value: &Value) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()
}

fn git(project: &Path, args: &[&str]) -> io::Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(project)
        .output()?;
    if !output.status.success() {
        return Err(error(format!(
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout).map_err(io::Error::other)
}

struct State {
    budget: Budget,
    calls: Vec<Value>,
    previous_reply: String,
    mutation_attempted: bool,
    accepted: Option<Value>,
}

#[derive(Clone)]
struct BoundedBackend<B> {
    inner: B,
    state: Arc<Mutex<Option<State>>>,
    project: PathBuf,
    evidence: PathBuf,
    cancelled: Arc<AtomicBool>,
}

impl<B> BoundedBackend<B> {
    fn before(&self, id: &AgentId, launch: bool, prompt: &str) -> Result<usize, AgentError> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err(error("cancelled").into());
        }
        let mut lock = self.state.lock().map_err(|_| error("state poisoned"))?;
        let state = lock.as_mut().ok_or_else(|| error("author not prepared"))?;
        state.budget.admit(id.as_str(), launch)?;
        let n = state.budget.calls();
        let input = json!({"call":n,"agent_id":id.as_str(),"launch":launch,"prompt":prompt});
        publish(
            &self.evidence.join(format!("CALL-{n:03}-INPUT.json")),
            &input,
        )?;
        state.calls.push(input);
        if !launch && !state.mutation_attempted && initial_read_owned(&state.previous_reply, prompt)
        {
            let mutator = AgentId::new(FIXTURE_MUTATOR_ID)?;
            if &mutator == id {
                return Err(error("fixture mutator must be distinct from the author").into());
            }
            state.mutation_attempted = true;
            publish(
                &self.evidence.join("MUTATION-ATTEMPT.json"),
                &json!({
                "author_id":id.as_str(),"fixture_mutator_id":mutator.as_str(),"delivery_call":n,"actual_read_reply":state.previous_reply,
                "entire_unchanged_read_prompt":prompt,"stimulus_owner":"qualification harness, not agent-authored work"}),
            )?;
            let before = fs::read_to_string(self.project.join("src/lib.rs"))?;
            if before != INITIAL_SOURCE
                || !git(
                    &self.project,
                    &["status", "--porcelain", "--untracked-files=all"],
                )?
                .is_empty()
            {
                return Err(
                    error("initial source/tree differs at controlled mutation seam").into(),
                );
            }
            let before_head = git(&self.project, &["rev-parse", "HEAD"])?;
            let patch = format!(
                "*** Begin Patch\n*** Update File: src/lib.rs\n@@\n-{INITIAL_SOURCE}+{CURRENT_SOURCE}*** End Patch\n"
            );
            let patcher = GitPatcher::new(
                self.project.clone(),
                FileLockTable::new(self.project.clone()),
            );
            let outcome = patcher
                .apply_edit(PatchRequest::new(
                    mutator.clone(),
                    "controlled equivalent snapshot stimulus",
                    "preserve behavior while changing the held source text",
                    patch.clone(),
                ))
                .map_err(|e| error(e.to_string()))?;
            let after = fs::read_to_string(self.project.join("src/lib.rs"))?;
            let accepted = json!({"author_id":id.as_str(),"fixture_mutator_id":mutator.as_str(),"delivery_call":n,
                "before_head":before_head.trim(),"commit":outcome.commit,
                "after_head":git(&self.project,&["rev-parse","HEAD"])?.trim(),
                "files":outcome.files,"source_before":before,"source_after":after,"patch_request":patch});
            // Retain acceptance even if an unexpected postcondition fails.
            publish(&self.evidence.join("MUTATION-ACCEPTED.json"), &accepted)?;
            state.accepted = Some(accepted);
            if after != CURRENT_SOURCE {
                return Err(error("accepted stimulus after-image differs").into());
            }
        }
        Ok(n)
    }

    fn after(&self, n: usize, reply: Result<&str, String>) -> Result<(), AgentError> {
        let mut lock = self.state.lock().map_err(|_| error("state poisoned"))?;
        let state = lock.as_mut().ok_or_else(|| error("author not prepared"))?;
        let row = state
            .calls
            .get_mut(n - 1)
            .ok_or_else(|| error("missing owned call"))?;
        match reply {
            Ok(text) => {
                row["reply"] = json!(text);
                row["returned"] = json!(true);
                state.previous_reply = text.to_owned();
            }
            Err(message) => {
                row["error"] = json!(message);
                row["returned"] = json!(false);
            }
        }
        publish(&self.evidence.join(format!("CALL-{n:03}-RESULT.json")), row)?;
        Ok(())
    }
}

impl<B: AgentBackend> AgentBackend for BoundedBackend<B> {
    fn launch(&mut self, _: AgentLaunch) -> Result<AgentSession, AgentError> {
        Err(error("unplanned non-streaming launch").into())
    }
    fn send(&mut self, _: &AgentId, _: &str) -> Result<ChatMessage, AgentError> {
        Err(error("unplanned non-streaming send").into())
    }
    fn session(&self, id: &AgentId) -> Option<AgentSession> {
        self.inner.session(id)
    }
    fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.inner.shutdown_handle()
    }
    fn launch_streaming_interruptible(
        &mut self,
        launch: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
        interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<AgentSession, AgentError> {
        let n = self.before(&launch.id, true, &launch.prompt)?;
        let result = self
            .inner
            .launch_streaming_interruptible(launch, sink, interrupt);
        let reply = result
            .as_ref()
            .map_err(ToString::to_string)
            .and_then(|session| {
                session
                    .messages
                    .iter()
                    .rev()
                    .find(|m| m.role == MessageRole::Agent)
                    .map(|m| m.text.as_str())
                    .ok_or_else(|| "launch has no actual agent reply".to_owned())
            });
        self.after(n, reply)?;
        result
    }
    fn send_streaming_interruptible(
        &mut self,
        id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        let n = self.before(id, false, prompt)?;
        let result = self
            .inner
            .send_streaming_interruptible(id, prompt, sink, interrupt);
        self.after(
            n,
            result
                .as_ref()
                .map(|m| m.text.as_str())
                .map_err(ToString::to_string),
        )?;
        result
    }
}

fn diagnostic_chat<B: AgentBackend>(project: PathBuf, backend: B) -> CommandChat<B> {
    CommandChat::new(project, backend).with_locked_command_timeout(Duration::from_secs(30))
}

fn processed_done(author: &str, transcript: Option<&str>) -> bool {
    transcript.is_some_and(|text| {
        text.ends_with(&format!("\n\norchestrator:\nagent {author} reported done"))
    })
}

fn path(name: &str, directory: bool) -> PathBuf {
    let p = PathBuf::from(env::var_os(name).unwrap_or_else(|| panic!("missing {name}")));
    let m = fs::symlink_metadata(&p).unwrap();
    assert!(p.is_absolute() && p.canonicalize().unwrap() == p && !m.file_type().is_symlink());
    assert!(if directory { m.is_dir() } else { m.is_file() });
    p
}
fn json_file(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
fn frames(path: &Path, closed: bool) -> io::Result<Vec<Value>> {
    let bytes = fs::read(path)?;
    if closed && !bytes.is_empty() && !bytes.ends_with(b"\n") {
        return Err(error("partial closed JSONL tail"));
    }
    bytes
        .split_inclusive(|b| *b == b'\n')
        .filter(|l| l.ends_with(b"\n"))
        .map(|l| serde_json::from_slice(l).map_err(io::Error::other))
        .collect()
}
fn settled(observation: &Path, closed: bool) -> io::Result<usize> {
    let all: Vec<_> = fs::read_dir(observation.join("app-server"))?
        .map(|e| e.map(|e| e.path()))
        .collect::<io::Result<Vec<_>>>()?;
    if all.len() != 1 || !all[0].is_dir() {
        return Err(error("expected one owned app-server capture"));
    }
    settled_turns(
        &frames(&all[0].join("client-to-server.raw"), closed)?,
        &frames(&all[0].join("client-to-server.forwarded.raw"), closed)?,
        &frames(&all[0].join("server-to-client.raw"), closed)?,
    )
}

#[test]
#[ignore = "one future frozen subscription admission only; at most eight real calls and 240-second watchdog"]
fn real_subscription_automatic_refresh() {
    for key in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "OPENAI_BASE_URL",
        "OPENAI_API_BASE",
        "CODEX_BASE_URL",
        "CODEX_ACCESS_TOKEN",
        "WORK_LEAF_CODEX_TRACE",
        "WORK_LEAF_OBSERVER_PARENT_INVOCATION",
    ] {
        assert!(env::var_os(key).is_none(), "unexpected {key}");
    }
    for (key, value) in [
        ("WORK_LEAF_REAL_AUTOMATIC_REFRESH_SMOKE", "1"),
        ("WORK_LEAF_BENCH_EXPERIMENT", "1"),
        ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "1"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "1000"),
        (
            "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
            "forward",
        ),
        ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "1"),
    ] {
        assert_eq!(env::var(key).as_deref(), Ok(value));
    }
    let manifest = json_file(&path("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", false));
    let config_path = path("WORK_LEAF_OBSERVER_CONFIG", false);
    let config = json_file(&config_path);
    assert_eq!(manifest["schema"], "work-leaf-bench-experiment-v7");
    assert_eq!(manifest["condition"], "automatic-changed-refresh-full");
    assert_eq!(config["condition"], "work-leaf");
    assert_eq!(config["model"], "gpt-5.5");
    assert_eq!(config["effort"], "xhigh");
    assert_eq!(config["run_id"], manifest["run_id"]);
    assert_eq!(
        env::var("WORK_LEAF_BENCH_RUN_ID").unwrap(),
        manifest["run_id"].as_str().unwrap()
    );
    assert_eq!(
        env::var("WORK_LEAF_OBSERVER_PRIMARY_MARKER").unwrap(),
        config["primary_invocation_marker"].as_str().unwrap()
    );
    let project = path("WORK_LEAF_REAL_OBSERVER_PROJECT_DIR", true);
    let evidence = path("WORK_LEAF_REAL_AUTOMATIC_REFRESH_ROOT", true);
    let proxy = path("WORK_LEAF_REAL_OBSERVER_CODEX_PROXY", false);
    let observer = path("WORK_LEAF_REAL_OBSERVER_BIN", false);
    let observation = PathBuf::from(config["root"].as_str().unwrap());
    let trace = PathBuf::from(manifest["evidence_path"].as_str().unwrap());
    for (name, text) in [
        ("src/lib.rs", INITIAL_SOURCE),
        ("Cargo.toml", INITIAL_MANIFEST),
        ("Cargo.lock", INITIAL_LOCK),
    ] {
        assert_eq!(fs::read_to_string(project.join(name)).unwrap(), text);
    }
    assert!(
        git(
            &project,
            &["status", "--porcelain", "--untracked-files=all"]
        )
        .unwrap()
        .is_empty()
    );
    assert!(!evidence.starts_with(&project) && !project.starts_with(&evidence));
    for name in [
        "HARNESS-RESULT.json",
        "MUTATION-ATTEMPT.json",
        "MUTATION-ACCEPTED.json",
    ] {
        assert_eq!(
            fs::symlink_metadata(evidence.join(name))
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotFound
        );
    }
    publish(&evidence.join("HARNESS-ATTEMPT.json"),&json!({"run_id":manifest["run_id"],"maximum_real_calls":8,
        "cooperative_seconds":240,"qualification_only":true,"project":project,"fixture_mutator_id":FIXTURE_MUTATOR_ID})).unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let state = Arc::new(Mutex::new(None));
    let backend = BoundedBackend {
        inner: CodexBackend::new(
            CodexCommandConfig::new(project.clone())
                .with_binary(proxy)
                .with_model("gpt-5.5")
                .with_sandbox(SandboxMode::ReadOnly),
            PromptPolicy::for_project(&project).unwrap(),
        ),
        state: Arc::clone(&state),
        project: project.clone(),
        evidence: evidence.clone(),
        cancelled: Arc::clone(&cancelled),
    };
    let mut chat = diagnostic_chat(project.clone(), backend);
    let request = "Implement capped(value, limit) so it returns the smaller input, preserving below-limit and equal-limit behavior. Write your own regression tests. First inspect src/lib.rs with the ordinary Work Leaf read directive before editing. Cargo.toml and Cargo.lock describe a dependency-free Rust library. Use normal Work Leaf changes and the focused check cargo test --offline --locked, then finish with @work-leaf done. This is a bounded recovery qualification, not a token-saving estimate.";
    let launch = chat.prepare_agent_launch(&[request.to_owned()]).unwrap();
    let author = launch.id.as_str().to_owned();
    *state.lock().unwrap() = Some(State {
        budget: Budget::new(&author, 8).unwrap(),
        calls: Vec::new(),
        previous_reply: String::new(),
        mutation_attempted: false,
        accepted: None,
    });
    let mut cancellation_chat = chat.clone();
    let watch_cancelled = Arc::clone(&cancelled);
    let (finish, wait) = mpsc::channel();
    let watchdog = thread::spawn(move || {
        if wait.recv_timeout(Duration::from_secs(240)).is_err() {
            watch_cancelled.store(true, Ordering::SeqCst);
            cancellation_chat.shutdown_agents();
            true
        } else {
            false
        }
    });
    let started = Instant::now();
    let actual = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        chat.launch_prepared_agent_streaming(launch, &mut |_| {})
    }));
    let (workflow_returned, transcript, failure) = match actual {
        Ok(Ok(CommandChatResult::AgentLaunched {
            agent_id, reply, ..
        })) if agent_id.as_str() == author => (true, Some(reply), None),
        Ok(Ok(other)) => (false, None, Some(format!("unexpected result {other:?}"))),
        Ok(Err(e)) => (false, None, Some(e.to_string())),
        Err(_) => (false, None, Some("scenario panicked".to_owned())),
    };
    let deadline = Instant::now() + Duration::from_secs(15);
    let preclosed = loop {
        if let Ok(n) = settled(&observation, false) {
            break Some(n);
        }
        if Instant::now() >= deadline {
            break None;
        }
        thread::sleep(Duration::from_millis(25));
    };
    let stop = Command::new("timeout")
        .args(["--kill-after=1s", "10s"])
        .arg(observer)
        .args(["stop-app-server", "--config"])
        .arg(config_path)
        .stdin(Stdio::null())
        .output();
    cancelled.store(true, Ordering::SeqCst);
    chat.shutdown_agents();
    let _ = finish.send(());
    let watchdog_fired = watchdog.join().unwrap_or(true);
    let closed = settled(&observation, true);
    let (stop_ok, stop_receipt) = match stop {
        Ok(o) => (
            o.status.success() && o.stdout == b"1\n",
            json!({"exit_code":o.status.code(),"stdout":String::from_utf8_lossy(&o.stdout),"stderr":String::from_utf8_lossy(&o.stderr)}),
        ),
        Err(e) => (false, json!({"error":e.to_string()})),
    };
    let state = state.lock().unwrap();
    let state = state.as_ref().unwrap();
    let local = frames(&trace, true)
        .and_then(|rows| recovery_chain(&author, CURRENT_SOURCE, &state.calls, &rows));
    let processed_done = processed_done(&author, transcript.as_deref());
    let result = json!({"schema":"work-leaf-automatic-refresh-harness-v1","run_id":manifest["run_id"],"condition":"work-leaf",
        "author_id":author,"workflow_returned":workflow_returned,"workflow_error":failure,"transcript":transcript,
        "outer_calls":state.budget.calls(),"real_roles":state.budget.real_roles(),"controlled_mutation":state.accepted,
        "processed_done":processed_done,"local_chain":local.as_ref().ok(),"local_chain_error":local.as_ref().err().map(ToString::to_string),
        "watchdog_fired":watchdog_fired,"settled_before_stop":preclosed,"closed_capture_turns":closed.as_ref().ok(),
        "closed_capture_error":closed.as_ref().err().map(ToString::to_string),"observer_stop":stop_receipt,
        "duration_seconds":started.elapsed().as_secs_f64(),"real_qualification":"pending independent source/public/native/semantic witness",
        "token_effect_estimate":false});
    publish(&evidence.join("HARNESS-RESULT.json"), &result).unwrap();
    println!("C08 harness retained; qualification remains a separate closed-source gate");
    assert!(workflow_returned && processed_done && local.is_ok() && state.accepted.is_some());
    assert!(!watchdog_fired && stop_ok && preclosed.is_some() && closed.is_ok());
    assert_eq!(closed.unwrap(), state.budget.calls());
    assert_eq!(state.budget.real_roles(), 1);
}
