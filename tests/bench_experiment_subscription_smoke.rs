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

const EDIT: &str = "@work-leaf edit set fixture value\n*** Begin Patch\n*** Update File: fixture.txt\n@@\n-value=1\n+value=2\n*** End Patch\n@work-leaf end";
const VALIDATE: &str = "@work-leaf locks run fixture.txt -- test \"$(cat fixture.txt)\" = 'value=2' && printf 'WORK_LEAF_HANDOFF_VALIDATION_OK\\n'";
const DONE: &str = "@work-leaf done";
const ACK: &str = "run at most one focused validation step that is relevant to files you touched or checks you added.";
const UNLIMITED: &str = "run the required focused validation steps that are relevant to files you touched or checks you added.";
const GUIDANCE: &str = "\nnext: Reply with the next Work Leaf directive, such as `@work-leaf done`, `@work-leaf edit`, `@work-leaf read`, or another `@work-leaf locks run`. Keep any non-directive explanation brief.";

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
            1 => "work-leaf patch applied\n",
            2 => "work-leaf command result\n",
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
            && frame["result"]["turn"]["id"].is_string()
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
        validate_reply(&session.messages.last().ok_or_else(rejected)?.text, EDIT)?;
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
            if self.budget.calls == 2 {
                VALIDATE
            } else {
                DONE
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

fn prepare_fixture(root: &Path) {
    assert!(
        fs::read_dir(root).unwrap().next().is_none(),
        "handoff fixture directory must be empty and dedicated to this diagnostic"
    );
    fs::write(root.join("fixture.txt"), "value=1\n").unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Work Leaf Handoff Diagnostic"],
        vec!["config", "user.email", "handoff@example.invalid"],
        vec!["add", "fixture.txt"],
        vec![
            "commit",
            "-qm",
            "ADD fixture to verify experimental handoffs",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(output.status.success(), "cannot prepare handoff fixture");
    }
}

fn assert_trace(manifest: &Value, session: &AgentSession) {
    let trace = fs::read_to_string(manifest["evidence_path"].as_str().unwrap()).unwrap();
    let rows = trace
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        3,
        "one activation and exactly two handoffs are required"
    );
    assert_eq!(rows[0]["event"], "activation");
    let condition = manifest["condition"].as_str().unwrap();
    for (index, (site, cue)) in [("patch-applied", ACK), ("command-result", GUIDANCE)]
        .iter()
        .enumerate()
    {
        let row = &rows[index + 1];
        assert_eq!(row["event"], "prompt");
        assert_eq!(row["sequence"], index + 1);
        assert_eq!(row["site"], *site);
        assert_eq!(row["agent_id"], session.id.to_string());
        assert_eq!(row["run_id"], manifest["run_id"]);
        assert_eq!(row["condition"], condition);
        let original = row["original_prompt"].as_str().unwrap();
        let forwarded = row["forwarded_prompt"].as_str().unwrap();
        let start = row["cue_start"].as_u64().unwrap() as usize;
        let end = row["cue_end"].as_u64().unwrap() as usize;
        assert_eq!(&original[start..end], *cue);
        let replacement = match (condition, *site) {
            ("ack-validation-unlimited", "patch-applied") => UNLIMITED,
            ("command-guidance-neutral", "command-result") => "",
            _ => cue,
        };
        assert_eq!(
            forwarded,
            format!("{}{replacement}{}", &original[..start], &original[end..])
        );
        assert_eq!(row["changed"], original != forwarded);
        assert_eq!(row["original_bytes"], original.len());
        assert_eq!(row["forwarded_bytes"], forwarded.len());
        assert_eq!(
            row["byte_delta"],
            forwarded.len() as i64 - original.len() as i64
        );
        assert_eq!(session.messages[2 + index * 2].role, MessageRole::User);
        assert_eq!(session.messages[2 + index * 2].text, forwarded);
    }
    assert!(
        rows[2]["original_prompt"]
            .as_str()
            .unwrap()
            .contains("\nstatus: 0\n")
    );
    assert!(
        rows[2]["original_prompt"]
            .as_str()
            .unwrap()
            .contains("stdout:\nWORK_LEAF_HANDOFF_VALIDATION_OK\n")
    );
}

fn run_handoffs(backend: CodexBackend, root: PathBuf, manifest: &Value) -> Result<(), AgentError> {
    let id = AgentId::new("user-handoff-smoke")?;
    let prompt = format!(
        "This is an explicitly bounded handoff diagnostic, not a feature benchmark. The only file is fixture.txt, whose full contents are `value=1` followed by a newline. Do not inspect other files, invoke built-in tools, launch other agents, or run other commands. Perform exactly these three responses, with no prose or markdown fences. First emit exactly this structured edit and stop:\n{EDIT}\nAfter the orchestrator acknowledges that the patch was applied, emit exactly this focused validation directive and stop:\n{VALIDATE}\nAfter receiving its successful command result, emit exactly {DONE} and stop. Do not combine these steps in one response."
    );
    let backend = HandoffBackend {
        inner: backend,
        budget: HandoffBudget::default(),
    };
    let mut chat = CommandChat::new(root.clone(), backend)
        .with_max_review_rounds(3)
        .with_locked_command_timeout(Duration::from_secs(10));
    let mut new_sessions = 0;
    chat.launch_prepared_agent_streaming(
        AgentLaunch::new(
            id.clone(),
            AgentKind::Codex,
            "experimental handoff diagnostic",
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
    assert_eq!(new_sessions, 1, "handoffs must resume the original session");
    assert_eq!(
        fs::read_to_string(root.join("fixture.txt")).unwrap(),
        "value=2\n"
    );
    let session = backend.session(&id).ok_or_else(rejected)?;
    assert_eq!(session.messages.len(), 6);
    assert_eq!(session.messages.last().unwrap().text.trim(), DONE);
    assert_trace(manifest, &session);
    Ok(())
}

#[test]
#[ignore = "requires explicit real subscription/observer configuration; consumes exactly three provider turns"]
fn real_subscription_experimental_patch_and_command_handoffs() {
    if env::var("WORK_LEAF_REAL_BENCH_HANDOFF_SMOKE").as_deref() != Ok("1") {
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
        if finished_rx.recv_timeout(Duration::from_secs(298)).is_err() {
            shutdown.shutdown();
            eprintln!("real handoff smoke exceeded its 300-second bound");
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
            "WORK_LEAF_HANDOFF_SMOKE_OK condition={} turns=3 sessions=1 patch_applied=true validation=passed exact_prompt_spans=true",
            manifest["condition"].as_str().unwrap()
        ),
        Ok(Err(error)) => panic!("real handoff smoke failed: {error}"),
        Err(error) => std::panic::resume_unwind(error),
    }
}

#[test]
fn handoff_budget_rejects_extra_sessions_wrong_agent_wrong_stage_and_extra_turns() {
    let id = AgentId::new("fixture-agent").unwrap();
    let other = AgentId::new("other-agent").unwrap();
    let mut budget = HandoffBudget::default();
    assert!(budget.follow_up(&id, "work-leaf patch applied\n").is_err());
    assert!(budget.launch(&id).is_ok());
    assert!(budget.launch(&id).is_err());
    assert!(
        budget
            .follow_up(&other, "work-leaf patch applied\n")
            .is_err()
    );
    assert!(budget.follow_up(&id, "work-leaf command result\n").is_err());
    assert!(budget.follow_up(&id, "work-leaf patch applied\n").is_ok());
    assert!(budget.follow_up(&id, "work-leaf patch applied\n").is_err());
    assert!(budget.follow_up(&id, "work-leaf command result\n").is_ok());
    assert!(budget.follow_up(&id, "work-leaf command result\n").is_err());
    assert_eq!(budget.calls, 3);
}

#[test]
fn handoff_reply_guard_rejects_extra_or_different_directives_before_dispatch() {
    for expected in [EDIT, VALIDATE, DONE] {
        assert!(validate_reply(expected, expected).is_ok());
        assert!(
            validate_reply(
                &format!("{expected}\n@work-leaf read secrets.txt"),
                expected
            )
            .is_err()
        );
        assert!(validate_reply("@work-leaf locks run . -- unexpected-command", expected).is_err());
    }
}
