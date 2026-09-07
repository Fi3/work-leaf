#![cfg(feature = "bench-experiments")]

use std::env;
use std::fs;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, ChatMessage, CodexBackend, CodexCommandConfig, CommandChat, MessageRole,
    PromptPolicy, SandboxMode,
};

mod temp_cleanup;

const READ: &str = "@work-leaf read fixture.txt";
const REPEAT: &str = "@work-leaf read --force fixture.txt";
const DONE: &str = "@work-leaf done";
const HANDOFF_ROUNDS: usize = 3;

#[derive(Default)]
struct HandoffBudget {
    agent_id: Option<AgentId>,
    calls: usize,
}

fn rejected() -> AgentError {
    AgentError::Io(io::Error::other(
        "handoff smoke rejected an unplanned provider call or directive",
    ))
}

impl HandoffBudget {
    fn launch(&mut self, id: &AgentId) -> Result<(), AgentError> {
        if self.calls != 0 {
            return Err(rejected());
        }
        self.agent_id = Some(id.clone());
        self.calls += 1;
        Ok(())
    }

    fn follow_up(&mut self, id: &AgentId, prompt: &str) -> Result<(), AgentError> {
        let expected = match self.calls {
            1 | 2 => "work-leaf file text\n",
            _ => return Err(rejected()),
        };
        if self.agent_id.as_ref() != Some(id) || !prompt.starts_with(expected) {
            return Err(rejected());
        }
        self.calls += 1;
        Ok(())
    }
}

fn validate_reply(actual: &str, expected: &str) -> Result<(), AgentError> {
    if actual.trim() == expected {
        Ok(())
    } else {
        Err(rejected())
    }
}

fn final_interrupt_settled(original: &[Value], forwarded: &[Value], server: &[Value]) -> bool {
    let Some(start) = original
        .iter()
        .rfind(|frame| frame["method"] == "turn/start")
    else {
        return false;
    };
    let Some(thread) = start["params"]["threadId"].as_str() else {
        return false;
    };
    let Some(reply) = server.iter().find(|frame| {
        frame.get("method").is_none()
            && frame.get("id") == start.get("id")
            && frame.get("error").is_none()
            && frame["result"]["turn"]["id"]
                .as_str()
                .is_some_and(|id| !id.is_empty())
    }) else {
        return false;
    };
    let turn = &reply["result"]["turn"]["id"];
    let mut matching = original.iter().filter(|frame| {
        frame["method"] == "turn/interrupt"
            && frame["params"]["threadId"] == thread
            && &frame["params"]["turnId"] == turn
    });
    let Some(interrupt) = matching.next() else {
        return false;
    };
    if matching.next().is_some()
        || !matches!(
            interrupt.get("id"),
            Some(Value::String(_) | Value::Number(_))
        )
    {
        return false;
    }
    forwarded.iter().any(|frame| frame == interrupt)
        && server.iter().any(|frame| {
            frame.get("method").is_none()
                && frame.get("id") == interrupt.get("id")
                && frame.get("result").is_some()
                && frame.get("error").is_none()
        })
        && server.iter().any(|frame| {
            frame["method"] == "turn/completed"
                && frame["params"]["threadId"] == thread
                && &frame["params"]["turn"]["id"] == turn
                && matches!(
                    frame["params"]["turn"]["status"].as_str(),
                    Some("completed" | "interrupted")
                )
        })
}

fn complete_capture_frames(path: &Path) -> io::Result<Vec<Value>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    bytes
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|frame| frame.ends_with(b"\n"))
        .map(|frame| serde_json::from_slice(frame).map_err(io::Error::other))
        .collect()
}

fn wait_for_terminal_interrupt(observer_config: &Path) -> io::Result<()> {
    let config: Value =
        serde_json::from_slice(&fs::read(observer_config)?).map_err(io::Error::other)?;
    let root = config["root"]
        .as_str()
        .ok_or_else(|| io::Error::other("observer root is absent"))?;
    let captures = fs::read_dir(Path::new(root).join("app-server"))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();
    if captures.len() != 1 {
        return Err(io::Error::other(
            "handoff diagnostic requires exactly one captured app server",
        ));
    }
    let capture = captures[0].path();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        // In the observer, forwarded capture bytes are flushed only after the
        // successful child write. The earlier grace decision alone is not proof
        // of forwarding. This is a bounded local teardown receipt, not a new turn
        // or a change to the observer's ordinary interruption policy.
        let original = complete_capture_frames(&capture.join("client-to-server.raw"))?;
        let forwarded = complete_capture_frames(&capture.join("client-to-server.forwarded.raw"))?;
        let server = complete_capture_frames(&capture.join("server-to-client.raw"))?;
        if final_interrupt_settled(&original, &forwarded, &server) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "final interrupt lacks actual forwarding, typed RPC acknowledgement, or matching terminal turn",
            ));
        }
        thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn teardown_waits_for_actual_forward_and_matching_terminal_turn() {
    let start =
        serde_json::json!({"id":"7", "method":"turn/start", "params":{"threadId":"thread"}});
    let interrupt = serde_json::json!({"id":"8", "method":"turn/interrupt", "params":{"threadId":"thread", "turnId":"final-turn"}});
    let reply = serde_json::json!({"id":"7", "result":{"turn":{"id":"final-turn"}}});
    let completed = serde_json::json!({"method":"turn/completed", "params":{"threadId":"thread", "turn":{"id":"final-turn", "status":"interrupted"}}});
    let acknowledgement = serde_json::json!({"id":"8", "result":{}});
    let original = [start.clone(), interrupt.clone()];
    assert!(
        !final_interrupt_settled(
            &original,
            std::slice::from_ref(&start),
            std::slice::from_ref(&reply)
        ),
        "a locally queued final interrupt is not a teardown receipt"
    );
    assert!(
        !final_interrupt_settled(&original, &original, std::slice::from_ref(&reply)),
        "forwarding alone does not prove terminal completion"
    );
    assert!(
        !final_interrupt_settled(&original, &original, &[reply.clone(), completed.clone()]),
        "wait for the interrupt RPC acknowledgement as well"
    );
    let server = [reply.clone(), completed.clone(), acknowledgement.clone()];
    assert!(final_interrupt_settled(&original, &original, &server));
    let mut wrong = interrupt.clone();
    wrong["id"] = serde_json::json!(8);
    assert!(
        !final_interrupt_settled(&original, &[start.clone(), wrong], &server),
        "string and integer RPC IDs are different"
    );
    let mut wrong = completed;
    wrong["params"]["turn"]["id"] = serde_json::json!("earlier-turn");
    assert!(!final_interrupt_settled(
        &original,
        &original,
        &[reply, wrong, acknowledgement]
    ));
}

#[test]
fn teardown_rejects_mixed_success_and_error_start_reply() {
    let original = [
        serde_json::json!({"id":"7", "method":"turn/start", "params":{"threadId":"thread"}}),
        serde_json::json!({"id":"8", "method":"turn/interrupt", "params":{"threadId":"thread", "turnId":"final-turn"}}),
    ];
    let server = [
        serde_json::json!({"id":"7", "result":{"turn":{"id":"final-turn"}}, "error":{"code":-1,"message":"rejected"}}),
        serde_json::json!({"id":"8", "result":{}}),
        serde_json::json!({"method":"turn/completed", "params":{"threadId":"thread", "turn":{"id":"final-turn", "status":"interrupted"}}}),
    ];
    assert!(
        !final_interrupt_settled(&original, &original, &server),
        "a reply with both result and error cannot establish an accepted turn"
    );
}

#[test]
fn teardown_rejects_empty_turn_identity() {
    let original = [
        serde_json::json!({"id":"7", "method":"turn/start", "params":{"threadId":"thread"}}),
        serde_json::json!({"id":"8", "method":"turn/interrupt", "params":{"threadId":"thread", "turnId":""}}),
    ];
    let server = [
        serde_json::json!({"id":"7", "result":{"turn":{"id":""}}}),
        serde_json::json!({"id":"8", "result":{}}),
        serde_json::json!({"method":"turn/completed", "params":{"threadId":"thread", "turn":{"id":"", "status":"interrupted"}}}),
    ];
    assert!(
        !final_interrupt_settled(&original, &original, &server),
        "matching empty strings are not a provider turn identity"
    );
}

struct HandoffBackend {
    inner: CodexBackend,
    budget: HandoffBudget,
}

impl AgentBackend for HandoffBackend {
    fn launch(&mut self, _: AgentLaunch) -> Result<AgentSession, AgentError> {
        Err(rejected())
    }

    fn send(&mut self, _: &AgentId, _: &str) -> Result<ChatMessage, AgentError> {
        Err(rejected())
    }

    fn session(&self, id: &AgentId) -> Option<AgentSession> {
        self.inner.session(id)
    }

    fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.inner.shutdown_handle()
    }

    fn launch_streaming_interruptible(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
        interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<AgentSession, AgentError> {
        self.budget.launch(&request.id)?;
        let session = self
            .inner
            .launch_streaming_interruptible(request, sink, interrupt)?;
        validate_reply(&session.messages.last().ok_or_else(rejected)?.text, READ)?;
        Ok(session)
    }

    fn send_streaming_interruptible(
        &mut self,
        id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        self.budget.follow_up(id, prompt)?;
        let reply = self
            .inner
            .send_streaming_interruptible(id, prompt, sink, interrupt)?;
        validate_reply(
            &reply.text,
            match self.budget.calls {
                2 => REPEAT,
                3 => DONE,
                _ => return Err(rejected()),
            },
        )?;
        Ok(reply)
    }
}

fn required_path(name: &str, directory: bool) -> PathBuf {
    let path = PathBuf::from(
        env::var_os(name).unwrap_or_else(|| panic!("required smoke path is missing: {name}")),
    );
    assert!(path.is_absolute(), "smoke path must be absolute: {name}");
    assert!(
        if directory {
            path.is_dir()
        } else {
            path.is_file()
        },
        "smoke path has the wrong file type: {name}"
    );
    path
}

fn fixture_text() -> String {
    (0..500)
        .map(|index| format!("entry-{index:04}: label=diagnostic; enabled=true; value={index}\n"))
        .collect()
}

fn validate_observer_preconditions(
    config: &Value,
    lookup: impl Fn(&str) -> Option<String>,
) -> io::Result<()> {
    let marker = config["primary_invocation_marker"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            io::Error::other("observer configuration requires a nonempty primary marker")
        })?;
    if config["condition"] != "work-leaf"
        || lookup("WORK_LEAF_OBSERVER_PRIMARY_MARKER").as_deref() != Some(marker)
        || lookup("WORK_LEAF_OBSERVER_PARENT_INVOCATION").is_some()
    {
        return Err(io::Error::other(
            "observer diagnostic must be an admitted primary work-leaf invocation",
        ));
    }
    for (name, expected) in [
        ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "1"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "1000"),
        (
            "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
            "forward",
        ),
        ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "1"),
    ] {
        if lookup(name).as_deref() != Some(expected) {
            return Err(io::Error::other(format!(
                "observer diagnostic requires {name}={expected}"
            )));
        }
    }
    Ok(())
}

#[test]
fn observer_preconditions_reject_missing_or_wrong_primary_admission() {
    let config = serde_json::json!({"condition":"work-leaf", "primary_invocation_marker":"diagnostic-marker"});
    let environment = [
        ("WORK_LEAF_OBSERVER_PRIMARY_MARKER", "diagnostic-marker"),
        ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "1"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "1000"),
        (
            "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
            "forward",
        ),
        ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "1"),
    ];
    let lookup = |name: &str| {
        environment
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| (*value).to_owned())
    };
    assert!(validate_observer_preconditions(&config, lookup).is_ok());
    for omitted in environment.map(|(key, _)| key) {
        assert!(
            validate_observer_preconditions(&config, |name| if name == omitted {
                None
            } else {
                lookup(name)
            })
            .is_err(),
            "missing {omitted} must fail before fixture or provider launch"
        );
    }
    for (name, wrong) in [
        ("WORK_LEAF_OBSERVER_PRIMARY_MARKER", "different-marker"),
        ("WORK_LEAF_OBSERVER_PRIMARY_MARKER", ""),
        ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "0"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "0"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "999"),
        (
            "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
            "wait-for-usage",
        ),
        ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "0"),
        ("WORK_LEAF_OBSERVER_PARENT_INVOCATION", "parent"),
    ] {
        assert!(
            validate_observer_preconditions(&config, |key| if key == name {
                Some(wrong.to_owned())
            } else {
                lookup(key)
            })
            .is_err(),
            "wrong {name} must fail before fixture or provider launch"
        );
    }
}

#[test]
fn observer_preconditions_reject_incompatible_config_without_env_mutation() {
    let lookup = |name: &str| {
        Some(
            match name {
                "WORK_LEAF_OBSERVER_PRIMARY_MARKER" => "diagnostic-marker",
                "WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE"
                | "WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY" => "1",
                "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS" => "1000",
                "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME" => "forward",
                _ => return None,
            }
            .to_owned(),
        )
    };
    for config in [
        serde_json::json!({"condition":"direct", "primary_invocation_marker":"diagnostic-marker"}),
        serde_json::json!({"condition":"work-leaf"}),
        serde_json::json!({"condition":"work-leaf", "primary_invocation_marker":""}),
        serde_json::json!({"condition":"work-leaf", "primary_invocation_marker":true}),
    ] {
        assert!(validate_observer_preconditions(&config, lookup).is_err());
    }
}

fn prepare_fixture(root: &Path) {
    assert!(
        fs::read_dir(root).unwrap().next().is_none(),
        "dedicated diagnostic directory must be empty"
    );
    let text = fixture_text();
    assert!(text.len() > 16 * 1024);
    fs::write(root.join("fixture.txt"), text).unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Read Delivery Diagnostic"],
        vec!["config", "user.email", "read@example.invalid"],
        vec!["add", "fixture.txt"],
        vec![
            "commit",
            "-qm",
            "ADD diagnostic data to verify real read delivery",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(output.status.success(), "cannot prepare read fixture");
    }
}

fn assert_trace(manifest: &Value, session: &AgentSession) {
    let trace = fs::read_to_string(manifest["evidence_path"].as_str().unwrap()).unwrap();
    let rows: Vec<Value> = trace
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        rows.len(),
        4,
        "activation, normal launch policy, and two read handoffs"
    );
    assert_eq!(rows[0]["schema"], "work-leaf-bench-experiment-v3");
    assert_eq!(rows[1]["site"], "policy-injection");
    assert_eq!(rows[1]["changed"], false);
    assert_eq!(rows[1]["original_prompt"], rows[1]["forwarded_prompt"]);
    let treatment = manifest["condition"] == "untracked-read-inline";
    assert!(treatment || manifest["condition"] == "control");
    for (index, row) in rows[2..].iter().enumerate() {
        assert_eq!(row["schema"], "work-leaf-bench-experiment-v3");
        assert_eq!(row["event"], "read-response");
        assert_eq!(row["site"], "file-read");
        assert_eq!(row["agent_id"], session.id.to_string());
        assert_eq!(row["run_id"], manifest["run_id"]);
        assert_eq!(row["condition"], manifest["condition"]);
        assert_eq!(row["eligible"], index == 0);
        let baseline = row["baseline_prompt"].as_str().unwrap();
        let inline = row["inline_candidate_prompt"].as_str().unwrap();
        let selected = if treatment && index == 0 {
            inline
        } else {
            baseline
        };
        assert_eq!(
            row["selected_candidate"],
            if treatment && index == 0 {
                "inline"
            } else {
                "baseline"
            }
        );
        assert_eq!(row["selected_bytes"], selected.len());
        assert_eq!(session.messages[2 + index * 2].role, MessageRole::User);
        assert_eq!(session.messages[2 + index * 2].text, selected);
        if index == 0 {
            assert!(baseline.contains("\nContext bundle: "));
            let snapshot = &row["snapshots"][0];
            assert_eq!(snapshot["path"], "fixture.txt");
            assert_eq!(snapshot["class"], "untracked");
            let start = snapshot["inline_body_start"].as_u64().unwrap() as usize;
            let end = snapshot["inline_body_end"].as_u64().unwrap() as usize;
            assert_eq!(&inline[start..end], fixture_text());
        } else {
            assert_eq!(baseline, inline, "ordinary repeated read is unchanged");
            assert!(selected.contains("Repeated file reads unchanged"));
            assert!(!selected.contains("entry-0499:"));
        }
    }
}

fn run_handoffs(backend: CodexBackend, root: PathBuf, manifest: &Value) -> Result<(), AgentError> {
    let id = AgentId::new("user-read-smoke")?;
    let prompt = format!(
        "This is an explicitly bounded read-delivery diagnostic, not a feature benchmark. Do not modify files, use built-in tools, inspect other files, or launch another agent. Perform exactly three responses without prose or markdown fences. First emit exactly {READ} and stop. After receiving the orchestrator file response, do not inspect or open any bundle: emit exactly {REPEAT} and stop. After receiving the repeated-file response, emit exactly {DONE} and stop. The fixture is generic diagnostic data, not a workload or token-saving assertion."
    );
    let backend = HandoffBackend {
        inner: backend,
        budget: HandoffBudget::default(),
    };
    let mut chat = CommandChat::new(root.clone(), backend).with_max_review_rounds(HANDOFF_ROUNDS);
    let mut new_sessions = 0;
    chat.launch_prepared_agent_streaming(
        AgentLaunch::new(
            id.clone(),
            AgentKind::Codex,
            "experimental read diagnostic",
            prompt,
        ),
        &mut |event| {
            if matches!(event, AgentStreamEvent::Status(text) if text.starts_with("Codex session "))
            {
                new_sessions += 1;
            }
        },
    )
    .map_err(|error| AgentError::Io(io::Error::other(error.to_string())))?;
    let backend = chat.into_backend();
    assert_eq!(backend.budget.calls, 3);
    assert_eq!(new_sessions, 1);
    assert_eq!(
        fs::read_to_string(root.join("fixture.txt")).unwrap(),
        fixture_text()
    );
    let session = backend.session(&id).ok_or_else(rejected)?;
    assert_eq!(session.messages.len(), 6);
    assert_eq!(session.messages.last().unwrap().text.trim(), DONE);
    assert_trace(manifest, &session);
    Ok(())
}

#[test]
#[ignore = "requires explicit real subscription/observer configuration; consumes exactly three provider turns"]
fn real_subscription_untracked_read_inline_handoffs() {
    if env::var("WORK_LEAF_REAL_BENCH_READ_INLINE_SMOKE").as_deref() != Ok("1") {
        eprintln!("real handoff smoke skipped: explicit enable flag is absent");
        return;
    }
    assert_eq!(env::var("WORK_LEAF_BENCH_EXPERIMENT").as_deref(), Ok("1"));
    let manifest_path = required_path("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", false);
    let manifest: Value = serde_json::from_slice(&fs::read(manifest_path).unwrap()).unwrap();
    assert_eq!(
        manifest["run_id"].as_str(),
        env::var("WORK_LEAF_BENCH_RUN_ID").ok().as_deref()
    );
    let root = required_path("WORK_LEAF_REAL_OBSERVER_PROJECT_DIR", true);
    let proxy = required_path("WORK_LEAF_REAL_OBSERVER_CODEX_PROXY", false);
    let observer = required_path("WORK_LEAF_REAL_OBSERVER_BIN", false);
    let observer_config = required_path("WORK_LEAF_OBSERVER_CONFIG", false);
    assert!(
        env::var_os("WORK_LEAF_CODEX_TRACE").is_none(),
        "verbose provider tracing must be disabled for the diagnostic"
    );
    let config: Value = serde_json::from_slice(&fs::read(&observer_config).unwrap()).unwrap();
    validate_observer_preconditions(&config, |name| env::var(name).ok())
        .expect("observer diagnostic preconditions must pass before fixture or provider launch");
    prepare_fixture(&root);
    let policy = PromptPolicy::for_project(&root).expect("cannot load handoff fixture policy");
    let mut retained_backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary(proxy)
            .with_model("gpt-5.5")
            .with_sandbox(SandboxMode::ReadOnly),
        policy,
    );
    let shutdown = retained_backend.shutdown_handle();
    let (finished_tx, finished_rx) = mpsc::channel();
    let watchdog = thread::spawn(move || {
        if finished_rx.recv_timeout(Duration::from_secs(118)).is_err() {
            shutdown.shutdown();
            eprintln!("real handoff smoke exceeded its 120-second bound");
            process::exit(1);
        }
    });
    let result = catch_unwind(AssertUnwindSafe(|| {
        run_handoffs(retained_backend.clone(), root, &manifest)
    }));
    let settled = wait_for_terminal_interrupt(&observer_config);
    if let Err(error) = &settled {
        eprintln!("handoff teardown receipt failed: {error}");
    }
    let observer_stop = Command::new("timeout")
        .args(["--kill-after=1s", "10s"])
        .arg(observer)
        .args(["stop-app-server", "--config"])
        .arg(observer_config)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    retained_backend.shutdown();
    let _ = finished_tx.send(());
    watchdog.join().expect("handoff watchdog failed");
    if let Ok(Err(error)) = &result {
        eprintln!("real handoff smoke failed before completion: {error}");
    }
    let observer_stop = observer_stop.expect("cannot run observer stop/flush");
    assert!(
        observer_stop.status.success() && observer_stop.stdout == b"1\n",
        "observer did not stop and flush its captured app server: {}",
        String::from_utf8_lossy(&observer_stop.stderr)
    );
    settled.expect("handoff teardown receipt was incomplete");
    match result {
        Ok(Ok(())) => println!(
            "WORK_LEAF_READ_INLINE_SMOKE_OK condition={} turns=3 sessions=1 first_read=verified repeated_read=unchanged",
            manifest["condition"].as_str().unwrap()
        ),
        Ok(Err(error)) => panic!("real handoff smoke failed: {error}"),
        Err(error) => std::panic::resume_unwind(error),
    }
}

#[test]
fn read_budget_accepts_exactly_two_followups_and_rejects_other_sites() {
    let id = AgentId::new("fixture-agent").unwrap();
    let other = AgentId::new("other-agent").unwrap();
    let mut budget = HandoffBudget::default();
    assert!(budget.follow_up(&id, "work-leaf file text\n").is_err());
    assert!(budget.launch(&id).is_ok());
    assert!(budget.launch(&id).is_err());
    for _ in 0..2 {
        assert!(budget.follow_up(&other, "work-leaf file text\n").is_err());
        assert!(budget.follow_up(&id, "work-leaf patch applied\n").is_err());
        assert!(budget.follow_up(&id, "work-leaf file text\n").is_ok());
    }
    assert!(budget.follow_up(&id, "work-leaf file text\n").is_err());
    assert_eq!(budget.calls, 3);
}

#[test]
fn read_reply_guard_rejects_unplanned_actions() {
    for expected in [READ, REPEAT, DONE] {
        assert!(validate_reply(expected, expected).is_ok());
        assert!(
            validate_reply(&format!("{expected}\n@work-leaf read other.txt"), expected).is_err()
        );
    }
}

#[derive(Default)]
struct ScriptedReads {
    session: Option<AgentSession>,
    followups: usize,
}

impl AgentBackend for ScriptedReads {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session
            .messages
            .push(ChatMessage::new(MessageRole::Agent, READ));
        self.session = Some(session.clone());
        Ok(session)
    }
    fn send(&mut self, _: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let text = [REPEAT, DONE].get(self.followups).ok_or_else(rejected)?;
        self.followups += 1;
        let reply = ChatMessage::new(MessageRole::Agent, *text);
        let session = self.session.as_mut().ok_or_else(rejected)?;
        session
            .messages
            .push(ChatMessage::new(MessageRole::User, prompt));
        session.messages.push(reply.clone());
        Ok(reply)
    }
}

#[test]
fn diagnostic_round_budget_delivers_both_reads_and_processes_done() {
    let root = env::temp_dir().join(format!(
        "work-leaf-read-rounds-{}-{}",
        process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    temp_cleanup::register(&root);
    prepare_fixture(&root);
    let mut chat =
        CommandChat::new(root, ScriptedReads::default()).with_max_review_rounds(HANDOFF_ROUNDS);
    let output = chat
        .launch_prepared_agent_streaming(
            AgentLaunch::new(
                AgentId::new("fixture-agent").unwrap(),
                AgentKind::Codex,
                "fixture",
                "fixture",
            ),
            &mut |_| {},
        )
        .unwrap();
    assert!(!format!("{output:?}").contains("did not converge"));
    let backend = chat.into_backend();
    assert_eq!(backend.followups, 2);
    assert_eq!(backend.session.unwrap().messages.last().unwrap().text, DONE);
}
