use std::env;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, ChatMessage, CodexBackend, CodexCommandConfig, CommandChat, MessageRole,
    PromptPolicy, SandboxMode,
};

const INITIAL_PROMPT: &str = "This is a read-only transport diagnostic, with no repository task. \
Do not call tools, inspect files, or modify files. First emit exactly @work-leaf done as a \
top-level line in a separate commentary message. Immediately afterward, begin a final message \
listing every integer from 1 to 1000, one per line, without abbreviating. Finish that message \
with WORK_LEAF_OBSERVER_UNINTERRUPTED_END. The orchestrator is expected to interrupt you before \
that final message finishes.";
const FOLLOW_UP: &str = "The transport diagnostic continues in this same conversation. \
Do not call tools, inspect files, or modify files. Reply in one final message with exactly these \
two lines and then stop:\nWORK_LEAF_OBSERVER_RESUME_OK\n@work-leaf done";
const FINAL_REPLY: &str = "WORK_LEAF_OBSERVER_RESUME_OK\n@work-leaf done";

#[derive(Default)]
struct TurnBudget {
    calls: usize,
}

impl TurnBudget {
    fn launch(&mut self) -> Result<(), AgentError> {
        if self.calls != 0 {
            return Err(disallowed_call());
        }
        self.calls = 1;
        Ok(())
    }

    fn follow_up(&mut self, prompt: &str) -> Result<(), AgentError> {
        if self.calls != 1 || prompt != FOLLOW_UP {
            return Err(disallowed_call());
        }
        self.calls = 2;
        Ok(())
    }
}

fn disallowed_call() -> AgentError {
    AgentError::Io(io::Error::other(
        "smoke provider-call budget rejected the request",
    ))
}

struct SmokeBackend {
    inner: CodexBackend,
    budget: TurnBudget,
    directive_detected: bool,
}

impl AgentBackend for SmokeBackend {
    fn launch(&mut self, _request: AgentLaunch) -> Result<AgentSession, AgentError> {
        Err(disallowed_call())
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Err(disallowed_call())
    }

    fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        self.inner.session(agent_id)
    }

    fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.inner.shutdown_handle()
    }

    fn launch_streaming_interruptible(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<AgentSession, AgentError> {
        self.budget.launch()?;
        let detected = &mut self.directive_detected;
        let mut detector = |event: &AgentStreamEvent| {
            let decision = should_interrupt(event);
            *detected |= decision;
            decision
        };
        self.inner
            .launch_streaming_interruptible(request, sink, &mut detector)
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        self.budget.follow_up(prompt)?;
        self.inner.send_streaming(agent_id, prompt, sink)
    }

    fn send_streaming_interruptible(
        &mut self,
        _agent_id: &AgentId,
        _prompt: &str,
        _sink: &mut dyn FnMut(AgentStreamEvent),
        _should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        Err(disallowed_call())
    }
}

fn required_path(name: &str, directory: bool) -> PathBuf {
    let path = env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("required smoke path variable is missing: {name}"));
    assert!(path.is_absolute(), "smoke paths must be absolute: {name}");
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

#[derive(Default)]
struct DirectiveGuard {
    prefix: String,
    ordinary_line: bool,
    done_line: bool,
}

impl DirectiveGuard {
    fn observe(&mut self, event: &AgentStreamEvent) {
        let AgentStreamEvent::AgentMessage(text) = event else {
            return;
        };
        // Whole assistant items can arrive without a trailing newline on the preceding item.
        // Treat a new directive prefix conservatively while preserving a split directive prefix.
        if self.ordinary_line && text.trim_start().starts_with('@') {
            self.ordinary_line = false;
        }
        for character in text.chars() {
            if character == '\n' {
                self.prefix.clear();
                self.ordinary_line = false;
                self.done_line = false;
            } else if self.ordinary_line {
                continue;
            } else if self.done_line {
                assert!(
                    character.is_whitespace(),
                    "smoke received an unexpected orchestrator request"
                );
            } else if self.prefix.is_empty() && character.is_whitespace() {
                continue;
            } else {
                self.prefix.push(character);
                if "@work-leaf done".starts_with(&self.prefix) {
                    if self.prefix == "@work-leaf done" {
                        self.done_line = true;
                        self.prefix.clear();
                    }
                } else {
                    assert!(
                        !self.prefix.starts_with("@work-leaf"),
                        "smoke received an unexpected orchestrator request"
                    );
                    self.ordinary_line = true;
                    self.prefix.clear();
                }
            }
        }
    }
}

fn run_two_turns(backend: CodexBackend, project_dir: PathBuf) -> Result<(), &'static str> {
    let agent_id = AgentId::new("user-observer-smoke").map_err(|_| "invalid smoke agent id")?;
    let backend = SmokeBackend {
        inner: backend,
        budget: TurnBudget::default(),
        directive_detected: false,
    };
    let mut chat = CommandChat::new(project_dir, backend).with_max_review_rounds(1);
    let mut launch_sessions = 0;
    let mut launch_guard = DirectiveGuard::default();
    chat.launch_prepared_agent_streaming(
        AgentLaunch::new(
            agent_id.clone(),
            AgentKind::Codex,
            "observer subscription diagnostic",
            INITIAL_PROMPT,
        ),
        &mut |event| {
            launch_guard.observe(&event);
            if matches!(event, AgentStreamEvent::Status(text) if text.starts_with("Codex session "))
            {
                launch_sessions += 1;
            }
        },
    )
    .map_err(|_| "initial launch failed")?;
    let mut backend = chat.into_backend();
    if !backend.directive_detected || launch_sessions != 1 {
        return Err("initial launch did not detect a directive in one session");
    }
    let session = backend
        .session(&agent_id)
        .ok_or("initial session is absent")?;
    let initial_reply = session.messages.last().ok_or("initial reply is absent")?;
    if !initial_reply
        .text
        .lines()
        .any(|line| line.trim() == "@work-leaf done")
        || initial_reply
            .text
            .contains("WORK_LEAF_OBSERVER_UNINTERRUPTED_END")
    {
        return Err("initial directive did not stop the requested continued output");
    }

    let mut new_sessions = 0;
    let mut follow_up_guard = DirectiveGuard::default();
    let reply = backend
        .send_streaming(&agent_id, FOLLOW_UP, &mut |event| {
            follow_up_guard.observe(&event);
            if matches!(event, AgentStreamEvent::Status(text) if text.starts_with("Codex session "))
            {
                new_sessions += 1;
            }
        })
        .map_err(|_| "raw same-session follow-up failed")?;
    if reply.text.trim() != FINAL_REPLY || new_sessions != 0 || backend.budget.calls != 2 {
        return Err("raw follow-up did not complete with the expected two-turn result");
    }
    let session = backend
        .session(&agent_id)
        .ok_or("follow-up session is absent")?;
    if session.messages.len() != 4
        || session.messages[2].role != MessageRole::User
        || session.messages[2].text != FOLLOW_UP
    {
        return Err("follow-up did not retain raw input in the original session");
    }
    Ok(())
}

#[test]
#[ignore = "requires explicit real subscription observer configuration; consumes two provider turns"]
fn real_subscription_directive_interrupt_and_raw_follow_up() {
    if env::var("WORK_LEAF_REAL_OBSERVER_SMOKE").as_deref() != Ok("1") {
        eprintln!("real observer smoke skipped: explicit enable flag is absent");
        return;
    }
    let proxy = required_path("WORK_LEAF_REAL_OBSERVER_CODEX_PROXY", false);
    let project_dir = required_path("WORK_LEAF_REAL_OBSERVER_PROJECT_DIR", true);
    let observer = required_path("WORK_LEAF_REAL_OBSERVER_BIN", false);
    let observer_config = required_path("WORK_LEAF_OBSERVER_CONFIG", false);
    assert!(
        env::var_os("WORK_LEAF_CODEX_TRACE").is_none(),
        "real smoke requires verbose provider tracing to be disabled"
    );
    let policy = PromptPolicy::for_project(&project_dir)
        .unwrap_or_else(|_| panic!("cannot load smoke project policy"));
    let mut retained_backend = CodexBackend::new(
        CodexCommandConfig::new(project_dir.clone())
            .with_binary(proxy)
            .with_model("gpt-5.5")
            .with_sandbox(SandboxMode::ReadOnly),
        policy,
    );
    let shutdown = retained_backend.shutdown_handle();
    let (finished_tx, finished_rx) = mpsc::channel();
    let watchdog = thread::spawn(move || {
        if finished_rx.recv_timeout(Duration::from_secs(178)).is_err() {
            shutdown.shutdown();
            eprintln!("real observer smoke exceeded its 180-second bound");
            process::exit(1);
        }
    });

    // Retaining this clone prevents CodexBackend::drop from killing the observer before it flushes.
    let result = catch_unwind(AssertUnwindSafe(|| {
        run_two_turns(retained_backend.clone(), project_dir)
    }));
    let observer_stop = Command::new("timeout")
        .args(["--kill-after=1s", "10s"])
        .arg(observer)
        .args(["stop-app-server", "--config"])
        .arg(observer_config)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    retained_backend.shutdown();
    let _ = finished_tx.send(());
    watchdog.join().expect("smoke watchdog failed");
    assert!(
        observer_stop.is_ok_and(|output| output.status.success() && output.stdout == b"1\n"),
        "observer did not stop and flush its captured app server"
    );
    match result {
        Ok(Ok(())) => println!(
            "WORK_LEAF_OBSERVER_SMOKE_OK turns=2 directive_detected=true raw_follow_up=completed"
        ),
        Ok(Err(stage)) => panic!("real observer smoke failed: {stage}"),
        Err(_) => panic!("real observer smoke rejected an unexpected event"),
    }
}

#[test]
fn smoke_budget_rejects_extra_calls_and_non_raw_follow_up() {
    let mut budget = TurnBudget::default();
    assert!(budget.follow_up(FOLLOW_UP).is_err());
    assert!(budget.launch().is_ok());
    assert!(budget.launch().is_err());
    assert!(budget.follow_up("unplanned correction").is_err());
    assert!(budget.follow_up(FOLLOW_UP).is_ok());
    assert!(budget.follow_up(FOLLOW_UP).is_err());
    assert_eq!(budget.calls, 2);
}

#[test]
fn smoke_rejects_mediated_file_or_command_requests_before_dispatch() {
    DirectiveGuard::default().observe(&AgentStreamEvent::AgentMessage("@work-leaf done".into()));
    for text in [
        "@work-leaf read src/lib.rs",
        "@work-leaf locks run . -- cargo test",
        "@work-leaf edit change",
    ] {
        assert!(
            catch_unwind(|| {
                DirectiveGuard::default().observe(&AgentStreamEvent::AgentMessage(text.into()));
            })
            .is_err()
        );
    }
}

fn guard_message_chunks(chunks: &[&str]) {
    let mut guard = DirectiveGuard::default();
    for chunk in chunks {
        guard.observe(&AgentStreamEvent::AgentMessage((*chunk).into()));
    }
}

#[test]
fn smoke_guard_accepts_a_done_directive_split_across_chunks() {
    guard_message_chunks(&["@work-", "leaf", " done\n"]);
    guard_message_chunks(&["@work-leaf", " done\n"]);
}

#[test]
fn smoke_guard_rejects_a_file_read_split_across_chunks() {
    assert!(catch_unwind(|| guard_message_chunks(&["@work-", "leaf read input.rs\n"])).is_err());
}

#[test]
fn smoke_guard_rejects_a_new_directive_item_after_unterminated_prose() {
    assert!(
        catch_unwind(|| guard_message_chunks(&[
            "A separate commentary item",
            "@work-leaf read input.rs"
        ]))
        .is_err()
    );
}
