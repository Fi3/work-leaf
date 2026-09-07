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

use c15_real_diagnostic_guards::{
    Budget, INITIAL_LOCK, INITIAL_MANIFEST, INITIAL_SOURCE, author_completion,
    await_author_completion, diagnostic_chat, fixture_request, settled_turns, validate_settings,
};
use serde_json::{Value, json};
use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, ChatMessage, CodexBackend, CodexCommandConfig, CommandChatResult,
    PromptPolicy, SandboxMode,
};

type SharedBudget = Arc<Mutex<Option<Budget>>>;

#[derive(Clone)]
struct BoundedBackend {
    inner: CodexBackend,
    budget: SharedBudget,
}

fn failure(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

impl BoundedBackend {
    fn admit(&self, id: &AgentId, launch: bool) -> Result<(), AgentError> {
        self.budget
            .lock()
            .map_err(|_| failure("budget poisoned"))?
            .as_mut()
            .ok_or_else(|| failure("author not prepared"))?
            .admit(id.as_str(), launch)
            .map_err(AgentError::Io)
    }
}

impl AgentBackend for BoundedBackend {
    fn launch(&mut self, _: AgentLaunch) -> Result<AgentSession, AgentError> {
        Err(AgentError::Io(failure("unplanned non-streaming launch")))
    }
    fn send(&mut self, _: &AgentId, _: &str) -> Result<ChatMessage, AgentError> {
        Err(AgentError::Io(failure("unplanned non-streaming send")))
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
        self.admit(&launch.id, true)?;
        self.inner
            .launch_streaming_interruptible(launch, sink, interrupt)
    }
    fn send_streaming_interruptible(
        &mut self,
        id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        self.admit(id, false)?;
        self.inner
            .send_streaming_interruptible(id, prompt, sink, interrupt)
    }
}

fn path(name: &str, directory: bool) -> PathBuf {
    let path = PathBuf::from(env::var_os(name).unwrap_or_else(|| panic!("missing {name}")));
    let metadata = fs::symlink_metadata(&path).unwrap();
    assert!(path.is_absolute() && path.canonicalize().unwrap() == path);
    assert!(if directory {
        metadata.is_dir()
    } else {
        metadata.is_file()
    });
    path
}

fn json_file(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn frames(path: &Path, closed: bool) -> io::Result<Vec<Value>> {
    let bytes = fs::read(path)?;
    if closed && !bytes.is_empty() && !bytes.ends_with(b"\n") {
        return Err(failure("closed capture has partial JSONL tail"));
    }
    bytes
        .split_inclusive(|b| *b == b'\n')
        .filter(|line| line.ends_with(b"\n"))
        .map(|line| serde_json::from_slice(line).map_err(io::Error::other))
        .collect()
}

fn settled_capture(observation: &Path, closed: bool) -> io::Result<usize> {
    let captures: Vec<_> = fs::read_dir(observation.join("app-server"))?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .filter(|entry| entry.is_dir())
        .collect();
    if captures.len() != 1 {
        return Err(failure("expected one owned app-server capture"));
    }
    let root = &captures[0];
    settled_turns(
        &frames(&root.join("client-to-server.raw"), closed)?,
        &frames(&root.join("client-to-server.forwarded.raw"), closed)?,
        &frames(&root.join("server-to-client.raw"), closed)?,
    )
}

fn author_capture(observation: &Path, trace: &Path, author: &str) -> io::Result<Value> {
    let captures: Vec<_> = fs::read_dir(observation.join("app-server"))?
        .map(|e| e.map(|e| e.path()))
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .filter(|p| p.is_dir())
        .collect();
    if captures.len() != 1 {
        return Err(failure("expected one owned app-server capture"));
    }
    let root = &captures[0];
    author_completion(
        author,
        "cargo test --offline --locked",
        &frames(trace, false)?,
        &frames(&root.join("client-to-server.raw"), false)?,
        &frames(&root.join("client-to-server.forwarded.raw"), false)?,
        &frames(&root.join("server-to-client.raw"), false)?,
    )
}

#[test]
#[ignore = "requires one frozen subscription admission; max eight outer calls and 240-second cancellation watchdog"]
fn real_subscription_private_test_first() {
    let manifest = json_file(&path("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", false));
    let config_path = path("WORK_LEAF_OBSERVER_CONFIG", false);
    let config = json_file(&config_path);
    validate_settings(&config, &manifest, |k| env::var(k).ok()).unwrap();
    let root = path("WORK_LEAF_REAL_OBSERVER_PROJECT_DIR", true);
    let proxy = path("WORK_LEAF_REAL_OBSERVER_CODEX_PROXY", false);
    let observer = path("WORK_LEAF_REAL_OBSERVER_BIN", false);
    let evidence = path("WORK_LEAF_REAL_PRIVATE_DIAGNOSTIC_ROOT", true);
    let result_path = evidence.join("HARNESS-RESULT.json");
    assert!(fs::symlink_metadata(&result_path).is_err());
    for (file, expected) in [
        ("src/lib.rs", INITIAL_SOURCE),
        ("Cargo.toml", INITIAL_MANIFEST),
        ("Cargo.lock", INITIAL_LOCK),
    ] {
        assert_eq!(fs::read_to_string(root.join(file)).unwrap(), expected);
    }
    let status = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(status.status.success() && status.stdout.is_empty());
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary(proxy)
            .with_model("gpt-5.5")
            .with_sandbox(SandboxMode::ReadOnly),
        PromptPolicy::for_project(&root).unwrap(),
    );
    let budget: SharedBudget = Arc::new(Mutex::new(None));
    let mut chat = diagnostic_chat(
        root,
        BoundedBackend {
            inner: backend,
            budget: Arc::clone(&budget),
        },
    );
    let launch = chat.prepare_agent_launch(&[fixture_request()]).unwrap();
    *budget.lock().unwrap() = Some(Budget::new(launch.id.as_str(), 8).unwrap());
    let author = launch.id.as_str().to_owned();
    let mut cancellation_chat = chat.clone();
    let (finish, wait) = mpsc::channel();
    let cancellation = Arc::new(AtomicBool::new(false));
    let watchdog_cancellation = Arc::clone(&cancellation);
    let watchdog = thread::spawn(move || {
        if wait.recv_timeout(Duration::from_secs(240)).is_err() {
            watchdog_cancellation.store(true, Ordering::SeqCst);
            cancellation_chat.shutdown_agents();
            eprintln!("C15 diagnostic watchdog requested provider and private-job cancellation");
            true
        } else {
            false
        }
    });
    let started = Instant::now();
    let scenario = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> Result<Value, String> {
            let author_result = chat
                .launch_prepared_agent_streaming(launch, &mut |_| {})
                .map_err(|e| e.to_string())?;
            let observation =
                PathBuf::from(config["root"].as_str().ok_or("missing observation root")?);
            let trace = PathBuf::from(
                manifest["evidence_path"]
                    .as_str()
                    .ok_or("missing trace path")?,
            );
            let readiness = Duration::from_secs(15)
                .min(Duration::from_secs(240).saturating_sub(started.elapsed()));
            let author_qualification = await_author_completion(readiness, || {
                if cancellation.load(Ordering::SeqCst) {
                    return Err(failure("diagnostic cancelled before review"));
                }
                author_capture(&observation, &trace, &author)
            })
            .map_err(|e| e.to_string())?;
            if cancellation.load(Ordering::SeqCst) {
                return Err("diagnostic cancelled before review".into());
            }
            budget
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .begin_review()
                .map_err(|e| e.to_string())?;
            let review = chat.handle_line("review").map_err(|e| e.to_string())?;
            let CommandChatResult::ReviewComplete(reviews) = review else {
                return Err("ordinary review did not return ReviewComplete".to_owned());
            };
            if reviews.len() != 1 || !reviews[0].findings_resolved {
                return Err("ordinary review is unresolved or has unexpected scope".to_owned());
            }
            Ok(
                json!({"author_result":format!("{author_result:?}"),"author_qualification":author_qualification,"review_rounds":reviews[0].rounds,
            "findings_resolved":reviews[0].findings_resolved}),
            )
        },
    ));
    let scenario = match scenario {
        Ok(Ok(value)) => json!({"workflow_returned":true,"details":value}),
        Ok(Err(error)) => json!({"workflow_returned":false,"error":error}),
        Err(_) => json!({"workflow_returned":false,"panic":true}),
    };
    println!("C15_SCENARIO_RETURNED {scenario}");
    let observation = PathBuf::from(config["root"].as_str().unwrap());
    let settlement_deadline = Instant::now() + Duration::from_secs(15);
    let settled = loop {
        if let Ok(count) = settled_capture(&observation, false) {
            break Some(count);
        }
        if Instant::now() >= settlement_deadline {
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
    chat.shutdown_agents();
    let _ = finish.send(());
    let watchdog_fired = watchdog.join().unwrap_or(true);
    let closed = settled_capture(&observation, true);
    let (stop_ok, stop_receipt) = match stop {
        Ok(output) => (
            output.status.success() && output.stdout == b"1\n",
            json!({
            "exit_code":output.status.code(),"stdout":String::from_utf8_lossy(&output.stdout),
            "stderr":String::from_utf8_lossy(&output.stderr)}),
        ),
        Err(error) => (false, json!({"error":error.to_string()})),
    };
    let budget = budget.lock().unwrap();
    let budget = budget.as_ref().unwrap();
    let result = json!({"schema":"work-leaf-private-test-first-harness-v1", "run_id":manifest["run_id"],
        "condition":"work-leaf","author_id":author,"scenario":scenario,"outer_calls":budget.calls(),
        "real_roles":budget.real_roles(),"watchdog_fired":watchdog_fired,"settled_before_stop":settled,
        "closed_capture_turns":closed.as_ref().ok(),"closed_capture_error":closed.as_ref().err().map(ToString::to_string),
        "observer_stop":stop_receipt,"duration_seconds":started.elapsed().as_secs_f64(),
        "private_result_and_native_source_qualification":"requires separate closed-source review",
        "token_effect_estimate":false});
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(result_path)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, &result).unwrap();
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
    println!("C15_HARNESS_RESULT {result}");
    assert_eq!(scenario["workflow_returned"], true);
    assert!(!watchdog_fired && stop_ok && settled.is_some() && closed.is_ok());
    assert_eq!(closed.unwrap(), budget.calls());
    assert_eq!(budget.real_roles(), 2);
}
