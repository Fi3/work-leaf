use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentProfile, AgentSession,
    AgentStreamEvent, ChatMessage, CommandChat, MessageRole, WorkLeafCompletion,
    WorkLeafController, WorkLeafEvent, WorkLeafLoading,
};

mod temp_cleanup;

#[test]
fn controller_exposes_ui_neutral_events_and_snapshot_without_terminal_ui() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("implement parser combinator")
        .unwrap();

    assert_eq!(agent_id, AgentId::new("user-1").unwrap());
    assert!(controller.drain_events().iter().any(|event| {
        matches!(event, WorkLeafEvent::AgentAdded { session } if session.id == agent_id)
    }));
    let starting = controller.snapshot();
    let session = starting.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "implement-parser-combinator");
    assert_eq!(session.loading, Some(WorkLeafLoading::Launching));

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let ready = controller.snapshot();
    let session = ready.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "implement-parser-combinator");
    assert_eq!(session.loading, None);
    assert!(session.lines.iter().any(|line| line == "launch reply"));

    controller.send_message(&agent_id, "continue").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let replied = controller.snapshot();
    let session = replied.session(&agent_id).expect("session exists");
    assert!(session.lines.iter().any(|line| line == "user: continue"));
    assert!(session.lines.iter().any(|line| line == "follow reply"));
}

#[test]
fn controller_line_events_do_not_resend_full_session_transcripts() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("stream compactly").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let events = controller.drain_events();
    assert!(events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentLineAppended { agent_id: id, line }
                if id == &agent_id && line == "launch reply"
        )
    }));
    assert!(
        !events.iter().any(|event| {
            matches!(
                event,
                WorkLeafEvent::AgentUpdated { session }
                    if session.id == agent_id
                        && session.lines.iter().any(|line| line == "launch reply")
            )
        }),
        "line append events should not be paired with full-session transcript updates"
    );
}

#[test]
fn controller_does_not_append_streamed_agent_messages_again_on_completion() {
    let backend = StreamingTranscriptBackend;
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("stream directives").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(
        session
            .lines
            .iter()
            .filter(|line| line.as_str() == "@work-leaf read src/ui.rs")
            .count(),
        1,
        "{session:?}"
    );
    assert_eq!(
        session
            .lines
            .iter()
            .filter(|line| line.as_str() == "@work-leaf done")
            .count(),
        1,
        "{session:?}"
    );
    assert!(
        !session
            .lines
            .iter()
            .any(|line| line.contains("agent user-1 reported done")),
        "done in the same streamed turn as a read waits for the read follow-up"
    );
}

#[test]
fn controller_keeps_repeated_streamed_status_activity_lines() {
    let backend = RepeatedStatusBackend;
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("rerun checks").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(
        session
            .lines
            .iter()
            .filter(|line| line.as_str() == "codex: command started: cargo test")
            .count(),
        2,
        "{session:?}"
    );
}

#[test]
fn controller_does_not_append_orchestrator_follow_up_blocks_on_completion() {
    let root = std::env::temp_dir().join(format!(
        "work-leaf-workspace-trims-orchestrator-follow-up-blocks-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
    fs::write(root.join("README.md"), "fixture context\n").unwrap();
    let backend = FakeBackend::new(["@work-leaf read README.md", "final reply after read"]);
    let chat = CommandChat::new(root, backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("read context").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert!(
        session
            .lines
            .iter()
            .any(|line| line.contains("final reply after read")),
        "{session:?}"
    );
    assert!(
        !session
            .lines
            .iter()
            .any(|line| line.contains("orchestrator:") || line.contains("agent follow-up from")),
        "{session:?}"
    );
}

#[test]
fn controller_status_events_do_not_resend_existing_large_transcripts() {
    let large_reply = "large transcript line\n".repeat(8192);
    let backend = FakeBackend::new([large_reply.as_str(), "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("keep status compact").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.drain_events();

    controller.send_message(&agent_id, "continue").unwrap();
    let waiting_events = controller.drain_events();
    assert!(waiting_events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentStatusUpdated {
                agent_id: id,
                loading: Some(WorkLeafLoading::WaitingForReply),
                ..
            } if id == &agent_id
        )
    }));
    assert!(
        !waiting_events.iter().any(|event| {
            matches!(
                event,
                WorkLeafEvent::AgentUpdated { session }
                    if session.id == agent_id
                        && session.lines.iter().any(|line| line == &large_reply)
            )
        }),
        "status changes should not serialize existing transcript text"
    );

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let ready_events = controller.drain_events();
    assert!(ready_events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentStatusUpdated {
                agent_id: id,
                loading: None,
                ..
            } if id == &agent_id
        )
    }));
    assert!(ready_events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentLineAppended { agent_id: id, line }
                if id == &agent_id && line == "follow reply"
        )
    }));
    assert!(
        !ready_events.iter().any(|event| {
            matches!(
                event,
                WorkLeafEvent::AgentUpdated { session }
                    if session.id == agent_id
                        && session.lines.iter().any(|line| line == &large_reply)
            )
        }),
        "ready status changes should not serialize existing transcript text"
    );
}

#[test]
fn controller_names_chat_from_backend_title_system_agent() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("please fix login callback")
        .unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "oauth-redirect-handler");
    assert!(session.lines.iter().any(|line| line == "launch reply"));

    let launches = backend.all_launches();
    assert!(
        launches
            .iter()
            .any(|launch| launch.id.as_str() == "title-agent"
                && launch.feature == "chat-title"
                && launch
                    .prompt
                    .contains("First prompt:\nplease fix login callback")),
        "{launches:?}"
    );
    assert!(
        launches.iter().any(|launch| launch.id.as_str() == "user-1"),
        "{launches:?}"
    );
}

#[test]
fn controller_reuses_one_backend_title_system_agent_for_multiple_chats() {
    let backend = FakeBackend::new(["first launch reply", "second launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let first_agent_id = controller
        .create_agent("please fix login callback")
        .unwrap();
    let second_agent_id = controller
        .create_agent("implement parser combinator")
        .unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    assert_eq!(
        snapshot
            .session(&first_agent_id)
            .expect("first session exists")
            .title,
        "oauth-redirect-handler"
    );
    assert_eq!(
        snapshot
            .session(&second_agent_id)
            .expect("second session exists")
            .title,
        "implement-parser-combinator"
    );

    let launches = backend.all_launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str() == "title-agent")
            .count(),
        1,
        "{launches:?}"
    );
    let sends = backend.all_sends();
    assert_eq!(
        sends
            .iter()
            .filter(|(agent_id, _)| agent_id.as_str() == "title-agent")
            .count(),
        1,
        "{sends:?}"
    );
}

#[test]
fn command_surface_chat_uses_backend_command_system_agent_to_parse_requests() {
    let backend = FakeBackend::new(["COMMAND: force-linearize\nREPLY: running `force-linearize`"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    controller.send_command_agent_message("start a forced linearization");
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let launches = backend.all_launches();
    assert!(
        launches
            .iter()
            .any(|launch| launch.id.as_str() == "command-agent"
                && launch.feature == "command-agent"
                && launch.prompt.contains("start a forced linearization")),
        "{launches:?}"
    );
    assert!(
        controller
            .transcript()
            .iter()
            .any(|line| line == "work-leaf> force-linearize"),
        "{:?}",
        controller.transcript()
    );
}

#[test]
fn command_agent_follow_up_receives_recent_command_transcript() {
    let backend = FakeBackend::new([
        "REPLY: ready",
        "launch reply",
        "COMMAND: review\nREPLY: starting review",
    ]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    controller.send_command_agent_message("hello");
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.execute_command_line("new implement parser");
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    controller.send_command_agent_message("review the agent I just opened");
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let sends = backend.all_sends();
    assert!(
        sends.iter().any(|(agent_id, prompt)| {
            agent_id.as_str() == "command-agent"
                && prompt.contains("work-leaf> new implement parser")
                && prompt.contains("review the agent I just opened")
        }),
        "{sends:?}"
    );
}

#[test]
fn command_agent_persistence_does_not_depend_on_backend_session_lookup() {
    let backend = SessionlessSystemBackend::new(["REPLY: ready", "COMMAND: help\nREPLY: helping"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    controller.send_command_agent_message("hello");
    controller.send_command_agent_message("show help");

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    assert_eq!(
        backend.command_agent_launches(),
        1,
        "command-agent should be launched once even when session() uses the trait default"
    );
    assert_eq!(
        backend.command_agent_sends(),
        1,
        "second command-agent turn should be a send follow-up"
    );
    assert!(
        controller
            .transcript()
            .iter()
            .any(|line| line == "work-leaf> help"),
        "{:?}",
        controller.transcript()
    );
}

#[test]
fn controller_queues_rapid_agent_launches_until_startup_stream() {
    let backend = LaunchTimingBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    controller.create_agent("first feature").unwrap();
    controller.create_agent("second feature").unwrap();
    controller.create_agent("third feature").unwrap();

    thread::sleep(Duration::from_millis(50));
    assert_eq!(
        backend.starts().len(),
        0,
        "creating agents should queue launches without entering the backend"
    );

    assert!(controller.is_busy());
    thread::sleep(Duration::from_millis(50));
    controller.drain_events();
    assert_eq!(
        backend.starts().len(),
        1,
        "rapid launches should not all enter the backend before the first startup stream"
    );

    assert!(controller.wait_for_idle(Duration::from_secs(4)));
    let starts = backend.starts();
    assert_eq!(starts.len(), 3);
    assert!(
        starts[1].duration_since(starts[0]) >= Duration::from_millis(80),
        "second launch should wait for first startup stream: {starts:?}"
    );
    assert!(
        starts[2].duration_since(starts[1]) >= Duration::from_millis(80),
        "third launch should wait for second startup stream: {starts:?}"
    );
}

#[test]
fn controller_marks_launch_as_waiting_after_first_backend_stream() {
    let backend = LaunchTimingBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("streaming launch").unwrap();

    assert!(controller.wait_for_session_line(&agent_id, "backend startup", Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.loading, Some(WorkLeafLoading::WaitingForReply));

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
}

#[test]
fn controller_reports_worker_panic_without_panicking() {
    let backend = PanicLaunchBackend;
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("panic please").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.loading, None);
    assert!(
        session
            .lines
            .iter()
            .any(|line| line.contains("worker panicked"))
    );
}

#[derive(Clone, Debug)]
struct SessionlessSystemBackend {
    state: Arc<Mutex<SessionlessSystemBackendState>>,
}

#[derive(Debug)]
struct SessionlessSystemBackendState {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
}

impl SessionlessSystemBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            state: Arc::new(Mutex::new(SessionlessSystemBackendState {
                replies: replies.into_iter().map(String::from).collect(),
                launches: Vec::new(),
                sends: Vec::new(),
            })),
        }
    }

    fn command_agent_launches(&self) -> usize {
        self.state
            .lock()
            .unwrap()
            .launches
            .iter()
            .filter(|launch| launch.id.as_str() == "command-agent")
            .count()
    }

    fn command_agent_sends(&self) -> usize {
        self.state
            .lock()
            .unwrap()
            .sends
            .iter()
            .filter(|(agent_id, _)| agent_id.as_str() == "command-agent")
            .count()
    }
}

impl AgentBackend for SessionlessSystemBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut state = self.state.lock().unwrap();
        state.launches.push(request.clone());
        let mut session = AgentSession::new(request);
        let reply = state.replies.pop_front().expect("missing fake reply");
        session.push_message(MessageRole::Agent, reply);
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let mut state = self.state.lock().unwrap();
        state.sends.push((agent_id.clone(), prompt.to_string()));
        let reply = state.replies.pop_front().expect("missing fake reply");
        Ok(ChatMessage::new(MessageRole::Agent, reply))
    }
}

#[derive(Clone, Debug)]
struct PanicLaunchBackend;

impl AgentBackend for PanicLaunchBackend {
    fn launch(&mut self, _request: AgentLaunch) -> Result<AgentSession, AgentError> {
        panic!("intentional backend panic")
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        panic!("intentional backend panic")
    }
}

#[derive(Clone, Debug)]
struct RepeatedStatusBackend;

impl AgentBackend for RepeatedStatusBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.launch_streaming(request, &mut |_| {})
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        sink(AgentStreamEvent::Status(
            "command started: cargo test".to_string(),
        ));
        sink(AgentStreamEvent::Status(
            "command started: cargo test".to_string(),
        ));
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "checks finished");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "unused"))
    }
}

#[derive(Clone, Debug, Default)]
struct LaunchTimingBackend {
    starts: Arc<Mutex<Vec<Instant>>>,
}

impl LaunchTimingBackend {
    fn starts(&self) -> Vec<Instant> {
        self.starts.lock().unwrap().clone()
    }
}

impl AgentBackend for LaunchTimingBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.launch_streaming(request, &mut |_| {})
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        if is_system_agent_launch(&request) {
            let mut session = AgentSession::new(request);
            session.push_message(MessageRole::Agent, "timed-title");
            return Ok(session);
        }
        self.starts.lock().unwrap().push(Instant::now());
        thread::sleep(Duration::from_millis(100));
        sink(AgentStreamEvent::Status("backend startup".to_string()));
        thread::sleep(Duration::from_millis(100));
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "launch reply");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "reply"))
    }
}

#[test]
fn controller_uses_agent_profile_for_non_codex_launches_and_reviews() {
    let root = git_repo("workspace-custom-profile-review");
    fs::write(root.join("README.md"), "fixture\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply parser patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: parser\nReason: parse configs\nContext: parser context",
        ],
    );
    let backend = FakeBackend::new(["launch reply", "NO_FINDINGS"]);
    let profile = AgentProfile::new(
        AgentKind::External("local-test-agent".to_string()),
        "Local Test Agent",
        "local-agent",
    );
    let chat = CommandChat::new(root, backend.clone()).with_agent_profile(profile.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("build custom provider path")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.start_review().unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let launches = backend.launches();
    assert!(launches.iter().any(|launch| {
        launch.id == agent_id
            && launch.kind == profile.kind
            && launch.feature == profile.default_feature
    }));
    assert!(
        launches
            .iter()
            .any(|launch| { launch.id.as_str() == "review-user-1" && launch.kind == profile.kind })
    );
}

#[test]
fn controller_starts_review_after_patch_agent_done_and_loops_until_clean() {
    let root = git_repo("workspace-automatic-review-loop");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after",
        "FINDINGS\n- missing reviewed wording",
        "fixed review finding\n@work-leaf patch address review\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after\n+after review\n@work-leaf end\n@work-leaf done",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after review\n"
    );

    let reviewer_id = AgentId::new("review-user-1").unwrap();
    let snapshot = controller.snapshot();
    let reviewer = snapshot
        .session(&reviewer_id)
        .expect("reviewer session exists");
    assert_eq!(reviewer.title, "review user-agent");
    assert_eq!(reviewer.loading, None);
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line.contains("missing reviewed wording"))
    );
    assert!(
        patch_agent.lines.iter().any(|line| {
            line.contains("user-1 reviewed by review-user-1: rounds=2 resolved=yes")
        })
    );

    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &agent_id
            && prompt.contains("missing reviewed wording")
            && prompt.contains("Please fix the patch")
            && prompt.contains("If a finding is about missing verification")
    }));
    assert!(sends.iter().any(|(target, prompt)| {
        target == &reviewer_id
            && prompt.contains("The original agent has responded to the findings")
            && prompt.contains("Please check the patch again")
            && prompt.contains("verification evidence")
    }));
    assert!(
        !sends
            .iter()
            .any(|(target, prompt)| target == &agent_id && prompt.contains("Please summarize"))
    );
}

#[test]
fn controller_asks_patch_agent_if_feature_is_done_after_clean_review() {
    let root = git_repo("workspace-review-feature-done-question");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after",
        "NO_FINDINGS",
        "backend status reply",
        "follow reply",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(
        patch_agent.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line == "work-leaf: is this feature done? [yes/no]"),
        "{patch_agent:?}"
    );
    let sends_after_review = backend.sends().len();

    controller.send_message(&agent_id, "/status").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(
        patch_agent.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert!(patch_agent.lines.iter().any(|line| line == "user: /status"));
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line == "backend status reply")
    );
    assert_eq!(backend.sends().len(), sends_after_review + 1);
    assert!(
        backend
            .sends()
            .iter()
            .any(|(target, prompt)| target == &agent_id && prompt == "/status")
    );

    controller.send_message(&agent_id, "maybe").unwrap();
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(
        patch_agent.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line == "work-leaf: answer yes or no to close this feature")
    );
    assert_eq!(backend.sends().len(), sends_after_review + 1);

    controller.send_message(&agent_id, "no thanks").unwrap();
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(
        patch_agent.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line == "user: no thanks")
    );
    assert_eq!(backend.sends().len(), sends_after_review + 1);

    controller.send_message(&agent_id, "no /status").unwrap();
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(
        patch_agent.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line == "user: no /status")
    );
    assert_eq!(backend.sends().len(), sends_after_review + 1);

    controller.send_message(&agent_id, "yes").unwrap();
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(patch_agent.completion, Some(WorkLeafCompletion::Closed));
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line == "work-leaf: feature marked closed")
    );
    assert_eq!(backend.sends().len(), sends_after_review + 1);

    controller
        .send_message(&agent_id, "add another tweak")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(patch_agent.completion, None);
    assert!(patch_agent.lines.iter().any(|line| line == "follow reply"));
    assert!(
        backend
            .sends()
            .iter()
            .any(|(target, prompt)| { target == &agent_id && prompt == "add another tweak" })
    );
}

#[test]
fn controller_delays_dependent_agent_launch_until_dependency_closes() {
    let root = git_repo("workspace-dependent-agent-launch");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "parent patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after parent\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after parent",
        "NO_FINDINGS",
        "child launch reply",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let parent = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let child = controller
        .create_agent(format!("--depends-on {parent} update follow-up"))
        .unwrap();

    let snapshot = controller.snapshot();
    let parent_session = snapshot.session(&parent).expect("parent session exists");
    let child_session = snapshot.session(&child).expect("child session exists");
    assert_eq!(
        parent_session.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert_eq!(
        child_session.loading,
        Some(WorkLeafLoading::WaitingForDependency)
    );
    assert_eq!(child_session.title, "update-follow-up");
    assert_eq!(child_session.depends_on, vec![parent.clone()]);
    assert_eq!(parent_session.depended_on_by, vec![child.clone()]);
    assert!(
        child_session
            .lines
            .iter()
            .any(|line| { line == &format!("work-leaf: waiting for {parent} to be marked done") }),
        "{child_session:?}"
    );
    assert!(
        !backend
            .launches()
            .iter()
            .any(|launch| launch.id == child || launch.prompt == "update follow-up"),
        "dependent agent prompt must not be sent before the dependency closes"
    );

    controller.send_message(&parent, "yes").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    assert!(
        launches
            .iter()
            .any(|launch| launch.id == child && launch.prompt == "update follow-up"),
        "{launches:?}"
    );
    let snapshot = controller.snapshot();
    let parent_session = snapshot.session(&parent).expect("parent session exists");
    let child_session = snapshot.session(&child).expect("child session exists");
    assert_eq!(parent_session.completion, Some(WorkLeafCompletion::Closed));
    assert_eq!(child_session.loading, None);
    assert!(
        child_session
            .lines
            .iter()
            .any(|line| line == "child launch reply"),
        "{child_session:?}"
    );
    assert!(
        child_session.lines.iter().any(|line| {
            line == &format!("work-leaf: dependency {parent} marked done; launching")
        }),
        "{child_session:?}"
    );
}

#[test]
fn controller_uses_first_waiting_prompt_as_dependent_launch_task() {
    let root = git_repo("workspace-dependent-agent-first-prompt");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "parent patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after parent\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after parent",
        "NO_FINDINGS",
        "child launch reply",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let parent = controller.create_agent("update readme").unwrap();
    let child = controller
        .create_agent(format!("--depends-on {parent}"))
        .unwrap();

    controller
        .send_message(&child, "fix dependent follow-up")
        .unwrap();

    let waiting = controller.snapshot();
    let child_session = waiting.session(&child).expect("child session exists");
    assert_eq!(child_session.title, "fix-dependent-follow-up");
    assert_eq!(
        child_session.loading,
        Some(WorkLeafLoading::WaitingForDependency)
    );
    assert!(
        child_session
            .lines
            .iter()
            .any(|line| line == "user: fix dependent follow-up"),
        "{child_session:?}"
    );
    assert!(
        !backend.launches().iter().any(|launch| launch.id == child),
        "dependent launch must wait for the parent to close"
    );

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let reviewed = controller.snapshot();
    let parent_session = reviewed.session(&parent).expect("parent session exists");
    let child_session = reviewed.session(&child).expect("child session exists");
    assert_eq!(
        parent_session.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert_eq!(
        child_session.loading,
        Some(WorkLeafLoading::WaitingForDependency)
    );
    assert!(
        reviewed
            .session(&AgentId::new("review-user-1").unwrap())
            .is_some(),
        "parent review must start even while a dependent launch is waiting"
    );
    assert!(
        !backend.launches().iter().any(|launch| launch.id == child),
        "review completion alone must not release the dependent launch"
    );

    controller.send_message(&parent, "yes").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    assert!(
        launches.iter().any(|launch| {
            launch.id == child
                && launch.prompt == "fix dependent follow-up"
                && launch.feature == "fix-dependent-follow-up"
        }),
        "{launches:?}"
    );
    assert!(
        backend.sends().iter().all(|(target, _)| target != &child),
        "the waiting task should be the child launch prompt, not a post-launch send"
    );
    let released = controller.snapshot();
    let child_session = released.session(&child).expect("child session exists");
    assert_eq!(child_session.loading, None);
    assert!(
        child_session
            .lines
            .iter()
            .any(|line| line == "child launch reply"),
        "{child_session:?}"
    );
}

#[test]
fn controller_defers_dependent_patch_promotion_until_dependency_closes() {
    let root = git_repo("workspace-dependent-patch-promotion");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "parent patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after parent\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after parent",
        "NO_FINDINGS",
        "reader ready",
        "patch ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let parent = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let reader = controller.create_agent("inspect dependent patch").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    controller
        .promote_agent_to_patch(
            &reader,
            &format!("--depends-on {parent} implement dependent patch"),
        )
        .unwrap();

    let waiting = controller.snapshot();
    let parent_session = waiting.session(&parent).expect("parent session exists");
    let reader_session = waiting.session(&reader).expect("reader session exists");
    assert_eq!(
        parent_session.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert_eq!(
        reader_session.loading,
        Some(WorkLeafLoading::WaitingForDependency)
    );
    assert_eq!(reader_session.depends_on, vec![parent.clone()]);
    assert_eq!(parent_session.depended_on_by, vec![reader.clone()]);
    assert!(
        backend.sends().iter().all(|(target, _)| target != &reader),
        "patch promotion must not send before the dependency closes"
    );

    controller.send_message(&parent, "yes").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &reader
            && prompt.contains("Continue this existing Work Leaf session as a patch agent")
            && prompt.contains("implement dependent patch")
    }));
    let released = controller.snapshot();
    let reader_session = released.session(&reader).expect("reader session exists");
    assert_eq!(reader_session.loading, None);
    assert!(
        reader_session.lines.iter().any(|line| {
            line == &format!("work-leaf: dependency {parent} marked done; sending patch task")
        }),
        "{reader_session:?}"
    );
    assert!(
        reader_session
            .lines
            .iter()
            .any(|line| line == "patch ready"),
        "{reader_session:?}"
    );
}

#[test]
fn controller_rejects_unknown_dependency_without_creating_agent() {
    let backend = FakeBackend::new(["parent launch"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let parent = controller.create_agent("parent task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    assert_eq!(parent, AgentId::new("user-1").unwrap());

    let result = controller.create_agent("--depends-on user-99 follow-up");

    assert!(result.is_err());
    let snapshot = controller.snapshot();
    assert!(snapshot.session(&AgentId::new("user-2").unwrap()).is_none());
    let launches = backend.launches();
    assert_eq!(launches.len(), 1);
    assert!(
        launches
            .iter()
            .all(|launch| launch.id != AgentId::new("user-2").unwrap())
    );
}

#[test]
fn controller_linearize_cancels_pending_dependent_launches_visibly() {
    let root = git_repo("workspace-linearize-cancels-dependent-launch");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "parent patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after parent\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after parent",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let parent = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let child = controller
        .create_agent(format!("--depends-on {parent} update follow-up"))
        .unwrap();

    let snapshot = controller.snapshot();
    let child_session = snapshot.session(&child).expect("child session exists");
    assert_eq!(
        child_session.loading,
        Some(WorkLeafLoading::WaitingForDependency)
    );
    assert_eq!(child_session.depends_on, vec![parent.clone()]);

    assert!(controller.start_linearize().unwrap().is_some());
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let snapshot = controller.snapshot();
    let parent_session = snapshot.session(&parent).expect("parent session exists");
    let child_session = snapshot.session(&child).expect("child session exists");
    assert_eq!(child_session.loading, None);
    assert!(child_session.depends_on.is_empty(), "{child_session:?}");
    assert!(
        parent_session.depended_on_by.is_empty(),
        "{parent_session:?}"
    );
    assert!(
        child_session.lines.iter().any(|line| {
            line == &format!("work-leaf: cancelled dependency wait for {parent} before linearize")
        }),
        "{child_session:?}"
    );
    assert!(
        !backend.launches().iter().any(|launch| launch.id == child),
        "dependent launch must be cancelled rather than left queued"
    );

    controller.send_message(&parent, "yes").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    assert!(
        !backend.launches().iter().any(|launch| launch.id == child),
        "closing the former dependency after linearize must not launch cancelled work"
    );
}

#[test]
fn controller_does_not_start_review_until_patch_agent_reports_done() {
    let root = git_repo("workspace-review-waits-for-done");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end",
        "summary that should not be requested",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let snapshot = controller.snapshot();
    assert!(
        snapshot
            .session(&AgentId::new("review-user-1").unwrap())
            .is_none(),
        "review must wait for the patch agent to report done"
    );
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(patch_agent.loading, None);
    let launches = backend.launches();
    assert!(
        !launches
            .iter()
            .any(|launch| launch.id.as_str() == "review-user-1")
    );
    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &agent_id
            && prompt.contains("work-leaf patch applied")
            && prompt.contains("@work-leaf done")
    }));
}

#[test]
fn controller_starts_review_when_agent_reports_done_after_prior_patch_turn() {
    let root = git_repo("workspace-review-later-done");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end",
        "waiting for review signal",
        "@work-leaf done",
        "summary: README changes from before to after",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert!(
        controller
            .snapshot()
            .session(&AgentId::new("review-user-1").unwrap())
            .is_none(),
        "review should wait until a later done message"
    );

    controller
        .send_message(&agent_id, "finish when ready")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let reviewer_id = AgentId::new("review-user-1").unwrap();
    let snapshot = controller.snapshot();
    let reviewer = snapshot
        .session(&reviewer_id)
        .expect("reviewer session starts after later done");
    assert_eq!(reviewer.loading, None);
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert!(
        patch_agent.lines.iter().any(|line| {
            line.contains("user-1 reviewed by review-user-1: rounds=1 resolved=yes")
        }),
        "{patch_agent:?}"
    );
}

#[test]
fn controller_starts_review_when_done_directive_has_trailing_whitespace() {
    let root = git_repo("workspace-review-done-trailing-whitespace");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end \t\n@work-leaf done \t",
        "summary: README changes from before to after",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let reviewer_id = AgentId::new("review-user-1").unwrap();
    let snapshot = controller.snapshot();
    let reviewer = snapshot
        .session(&reviewer_id)
        .expect("reviewer session starts from whitespace-tolerant done");
    assert_eq!(reviewer.loading, None);
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(
        patch_agent.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line.contains("user-1 reviewed by review-user-1: rounds=1 resolved=yes")),
        "{patch_agent:?}"
    );
    assert!(
        !patch_agent
            .lines
            .iter()
            .any(|line| line.contains("unknown work-leaf directive `done"))
    );
}

#[test]
fn controller_command_linearize_requires_closed_patch_chats_unless_forced() {
    let root = git_repo("workspace-linearize-force-command");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(
        patch_agent.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );

    controller.execute_command_line("linearize");
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let launches = backend.launches();
    assert!(
        !launches
            .iter()
            .any(|launch| launch.id.as_str() == "linearize"),
        "{launches:?}"
    );
    assert!(controller.transcript().iter().any(|line| {
        line == "work-leaf: reviewed patch chats must be classified as closed before linearize: user-1. Use force-linearize to bypass."
    }));

    controller.execute_command_line("force-linearize");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let launches = backend.launches();
    assert!(
        launches
            .iter()
            .any(|launch| launch.id.as_str() == "linearize"),
        "{launches:?}"
    );
}

#[test]
fn controller_linearize_preserves_cumulative_review_scope_for_one_done() {
    let root = git_repo("workspace-linearize-cumulative-review-scope");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch first step\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end",
        "second patch\n@work-leaf patch second step\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: full change",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("update readme in two steps")
        .unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert!(controller.start_linearize().unwrap().is_some());
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    let linearize_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "linearize")
        .expect("linearize agent launched");
    assert!(
        linearize_launch
            .prompt
            .contains("Review scope includes 2 provisional commits"),
        "{}",
        linearize_launch.prompt
    );
    assert!(linearize_launch.prompt.contains("first step"));
    assert!(linearize_launch.prompt.contains("second step"));
}

#[test]
fn controller_sends_agent_slash_commands_to_the_backend_unchanged() {
    let backend = FakeBackend::new(["launch reply", "backend status output"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("status command").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.drain_events();

    controller.send_message(&agent_id, "/status").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.loading, None);
    assert!(session.lines.iter().any(|line| line == "user: /status"));
    assert!(
        session
            .lines
            .iter()
            .any(|line| line == "backend status output")
    );
    assert_eq!(
        backend.sends(),
        vec![(agent_id.clone(), "/status".to_string())],
        "slash commands must be sent to the selected agent unchanged"
    );
}

#[test]
fn controller_sends_fork_slash_command_to_the_same_agent_unchanged() {
    let backend = FakeBackend::new(["launch reply", "backend fork output"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let source_agent = controller.create_agent("original task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.drain_events();

    controller
        .send_message(&source_agent, "/fork try an alternate implementation")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let snapshot = controller.snapshot();
    let source = snapshot
        .session(&source_agent)
        .expect("source agent exists");
    assert!(
        source
            .lines
            .iter()
            .any(|line| line == "user: /fork try an alternate implementation")
    );
    assert!(
        source
            .lines
            .iter()
            .any(|line| line == "backend fork output")
    );
    assert!(snapshot.session(&AgentId::new("user-2").unwrap()).is_none());
    assert_eq!(
        backend.sends(),
        vec![(
            source_agent.clone(),
            "/fork try an alternate implementation".to_string()
        )],
        "slash commands must not be converted into Work Leaf actions"
    );
}

#[test]
fn controller_promotes_existing_agent_to_patch_agent_without_relaunching() {
    let backend = FakeBackend::new(["reader ready", "patch ready"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("read the code first").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.drain_events();

    controller
        .promote_agent_to_patch(&agent_id, "implement the selected fix")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let launches = backend.launches();
    assert_eq!(
        launches.len(),
        1,
        "promotion must reuse the existing backend session instead of launching a replacement"
    );
    let sends = backend.sends();
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].0, agent_id);
    assert!(sends[0].1.contains("patch agent"));
    assert!(sends[0].1.contains("implement the selected fix"));

    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert!(session.lines.iter().any(|line| line == "reader ready"));
    assert!(
        session
            .lines
            .iter()
            .any(|line| line == "work-leaf: escalated this chat to a patch agent")
    );
    assert!(session.lines.iter().any(|line| line == "patch ready"));
}

#[test]
fn controller_command_promotes_existing_agent_to_patch_agent() {
    let backend = FakeBackend::new(["reader reply", "promotion reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("inspect the regression").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.drain_events();

    controller.execute_command_line(&format!("promote {agent_id} implement the fix"));
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.loading, None);
    assert!(
        session
            .lines
            .iter()
            .any(|line| line == "work-leaf: escalated this chat to a patch agent")
    );
    assert!(
        session
            .lines
            .iter()
            .any(|line| line.contains("Patch task:") && line.contains("implement the fix"))
    );
    assert!(session.lines.iter().any(|line| line == "promotion reply"));
    assert!(snapshot.session(&AgentId::new("user-2").unwrap()).is_none());

    assert_eq!(backend.launches().len(), 1);
    let sends = backend.sends();
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].0, agent_id);
    assert!(
        sends[0]
            .1
            .contains("Continue this existing Work Leaf session as a patch agent")
    );
    assert!(sends[0].1.contains("implement the fix"));
}

#[test]
fn controller_forks_patch_agent_with_copied_history_and_independent_backend_session() {
    let backend = FakeBackend::new(["source launch", "source follow-up", "fork launch"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let source_id = controller.create_agent("original task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller
        .send_message(&source_id, "capture this context")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.drain_events();

    let fork_id = controller
        .fork_agent(&source_id, "try an alternate implementation")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    assert_eq!(fork_id, AgentId::new("user-2").unwrap());
    let snapshot = controller.snapshot();
    let fork = snapshot.session(&fork_id).expect("fork session exists");
    assert!(fork.lines.iter().any(|line| line == "source launch"));
    assert!(
        fork.lines
            .iter()
            .any(|line| line == "user: capture this context")
    );
    assert!(fork.lines.iter().any(|line| line == "source follow-up"));
    assert!(
        fork.lines
            .iter()
            .any(|line| line == "work-leaf: forked from user-1")
    );
    assert!(fork.lines.iter().any(|line| line == "fork launch"));

    let launches = backend.launches();
    assert_eq!(launches.len(), 2);
    assert_eq!(launches[1].id, fork_id);
    assert!(
        launches[1]
            .prompt
            .contains("Conversation history from user-1")
    );
    assert!(launches[1].prompt.contains("original task"));
    assert!(launches[1].prompt.contains("capture this context"));
    assert!(launches[1].prompt.contains("source follow-up"));
    assert!(
        launches[1]
            .prompt
            .contains("try an alternate implementation")
    );
}

#[test]
fn controller_review_prompt_covers_all_agent_commits_since_launch() {
    let root = git_repo("workspace-review-full-agent-scope");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch first step\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end",
        "second patch\n@work-leaf patch second step\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: full change",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("update readme in two steps")
        .unwrap();

    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let launches = backend.launches();
    let review_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "review-user-1")
        .expect("reviewer launched");
    assert!(
        review_launch.prompt.contains("first step"),
        "{}",
        review_launch.prompt
    );
    assert!(
        review_launch.prompt.contains("second step"),
        "{}",
        review_launch.prompt
    );
}

#[test]
fn controller_reuses_one_reviewer_for_repeated_patch_agent_iterations() {
    let root = git_repo("workspace-reuses-reviewer");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end\n@work-leaf done",
        "summary: README changes to after first",
        "NO_FINDINGS",
        "second patch\n@work-leaf patch update readme again\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: README changes to after second",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    controller.send_message(&agent_id, "no").unwrap();
    controller
        .send_message(&agent_id, "make the second update")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after second\n"
    );
    let reviewer_id = AgentId::new("review-user-1").unwrap();
    let launches = backend.launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id == reviewer_id)
            .count(),
        1
    );
    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &reviewer_id
            && prompt.contains("Review the full patch scope")
            && prompt.contains("after second")
    }));
}

#[test]
fn controller_routes_no_follow_up_fix_to_same_reviewer_and_asks_again() {
    let root = git_repo("workspace-no-follow-up-review");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end\n@work-leaf done",
        "summary: README changes to after first",
        "NO_FINDINGS",
        "second patch\n@work-leaf patch update readme again\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: README changes to after second",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    controller.drain_events();

    controller
        .send_message(&agent_id, "no, make the second update")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let events = controller.drain_events();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after second\n"
    );
    let reviewer_id = AgentId::new("review-user-1").unwrap();
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(
        patch_agent.completion,
        Some(WorkLeafCompletion::NeedsDecision)
    );
    assert_eq!(
        patch_agent
            .lines
            .iter()
            .filter(|line| line.as_str() == "work-leaf: is this feature done? [yes/no]")
            .count(),
        2
    );
    assert!(events.iter().any(|event| {
        matches!(event, WorkLeafEvent::AgentSelected { agent_id: selected } if selected == &agent_id)
    }));
    assert_eq!(
        backend
            .launches()
            .iter()
            .filter(|launch| launch.id == reviewer_id)
            .count(),
        1
    );
    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &agent_id
            && prompt.contains("feature is not done")
            && prompt.contains("make the second update")
            && prompt.contains("emit `@work-leaf done` again")
            && prompt.contains("another review round")
    }));
    assert!(sends.iter().any(|(target, prompt)| {
        target == &reviewer_id
            && prompt.contains("Review the full patch scope")
            && prompt.contains("after second")
    }));

    controller.send_message(&agent_id, "yes").unwrap();
    let snapshot = controller.snapshot();
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(patch_agent.completion, Some(WorkLeafCompletion::Closed));
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line == "work-leaf: feature marked closed")
    );
}

#[test]
fn controller_reviews_only_unreviewed_patch_agent_commits() {
    let root = git_repo("workspace-reviews-only-unreviewed");
    fs::write(root.join("README.md"), "readme before\n").unwrap();
    fs::write(root.join("CHANGELOG.md"), "changelog before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial docs fixture"]);
    let backend = FakeBackend::new([
        "readme patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-readme before\n+readme after\n@work-leaf end\n@work-leaf done",
        "summary: README changes",
        "NO_FINDINGS",
        "changelog patch\n@work-leaf patch update changelog\n--- a/CHANGELOG.md\n+++ b/CHANGELOG.md\n@@ -1 +1 @@\n-changelog before\n+changelog after\n@work-leaf end\n@work-leaf done",
        "summary: changelog changes",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let first = controller.create_agent("update readme").unwrap();
    assert_eq!(first.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let second = controller.create_agent("update changelog").unwrap();
    assert_eq!(second.as_str(), "user-2");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str() == "review-user-1")
            .count(),
        1
    );
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str() == "review-user-2")
            .count(),
        1
    );
}

#[test]
fn controller_auto_review_ignores_historical_agents_outside_current_patch_agent() {
    let root = git_repo("workspace-auto-review-current-agent-only");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    for old_agent in ["user-2", "user-3"] {
        fs::write(
            root.join("README.md"),
            format!("historical commit from {old_agent}\n"),
        )
        .unwrap();
        git(&root, ["add", "README.md"]);
        git(
            &root,
            [
                "commit",
                "-m",
                "UPDATE apply historical patch",
                "-m",
                &format!(
                    "Agent-ID: {old_agent}\nFeature: historical\nReason: previous work\nContext: old patch"
                ),
            ],
        );
    }
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "UPDATE reset live fixture"]);
    let backend = FakeBackend::new([
        "live patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: live README change",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str().starts_with("review-"))
            .map(|launch| launch.id.as_str().to_string())
            .collect::<Vec<_>>(),
        vec!["review-user-1".to_string()]
    );
}

#[test]
fn controller_linearize_uses_only_commits_reviewed_in_this_session() {
    let root = git_repo("workspace-linearize-current-reviewed-only");
    fs::write(root.join("README.md"), "before\n").unwrap();
    fs::write(root.join("legacy.txt"), "legacy before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial linearize fixture"]);
    fs::write(root.join("legacy.txt"), "legacy after\n").unwrap();
    git(&root, ["add", "legacy.txt"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply legacy patch from user-2",
            "-m",
            "Agent-ID: user-2\nFeature: legacy\nReason: old run\nContext: old session",
        ],
    );
    let backend = FakeBackend::new([
        "live patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: live README change",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert!(controller.start_linearize().unwrap().is_some());
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    let linearize_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "linearize")
        .expect("linearize agent launched");
    assert!(linearize_launch.prompt.contains("Agent-ID: user-1"));
    assert!(linearize_launch.prompt.contains("Commit:"));
    assert!(!linearize_launch.prompt.contains("Agent-ID: user-2"));
    assert!(!linearize_launch.prompt.contains("old session"));
}

#[test]
fn controller_linearize_compacts_multiple_reviewed_commits_from_same_agent() {
    let root = git_repo("workspace-linearize-same-agent-multiple-reviewed");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch update readme once\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end\n@work-leaf done",
        "summary: first reviewed change",
        "NO_FINDINGS",
        "second patch\n@work-leaf patch update readme twice\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: second reviewed change",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    controller.send_message(&agent_id, "no").unwrap();
    controller
        .send_message(&agent_id, "make a second reviewed update")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert!(controller.start_linearize().unwrap().is_some());
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    let linearize_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "linearize")
        .expect("linearize agent launched");
    assert!(
        linearize_launch
            .prompt
            .contains("Reason: update readme once"),
        "{}",
        linearize_launch.prompt
    );
    assert!(
        linearize_launch
            .prompt
            .contains("Reason: update readme twice"),
        "{}",
        linearize_launch.prompt
    );
    assert_eq!(
        linearize_launch.prompt.matches("Agent-ID: user-1").count(),
        1
    );
    assert!(linearize_launch.prompt.contains("exactly 1 final commit"));
}

#[test]
fn controller_can_create_review_and_linearize_three_features_from_missing_feature_commit() {
    let root = git_repo("workspace-three-feature-orchestration");
    fs::create_dir_all(root.join("features")).unwrap();
    fs::write(root.join("features/visual_mode.txt"), "missing\n").unwrap();
    fs::write(root.join("features/slash_commands.txt"), "missing\n").unwrap();
    fs::write(root.join("features/review_completion.txt"), "missing\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD missing feature fixtures"]);
    assert_eq!(
        fs::read_to_string(root.join("features/visual_mode.txt")).unwrap(),
        "missing\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("features/slash_commands.txt")).unwrap(),
        "missing\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("features/review_completion.txt")).unwrap(),
        "missing\n"
    );

    let backend = ThreeFeatureBackend::default();
    let chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let visual_id = controller
        .create_agent("add vim like visual mode for both panes")
        .unwrap();
    let slash_id = controller
        .create_agent("send slash commands unchanged to the selected backend agent")
        .unwrap();
    let completion_id = controller
        .create_agent("ask yes or no when reviewed patch work is done")
        .unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(4)));
    let snapshot = controller.snapshot();
    for agent_id in [&visual_id, &slash_id, &completion_id] {
        let session = snapshot.session(agent_id).expect("patch session exists");
        assert_eq!(
            session.completion,
            Some(WorkLeafCompletion::NeedsDecision),
            "{session:?}"
        );
        assert!(
            session
                .lines
                .iter()
                .any(|line| { line == "work-leaf: is this feature done? [yes/no]" })
        );
    }
    assert_eq!(
        fs::read_to_string(root.join("features/visual_mode.txt")).unwrap(),
        "implemented visual mode\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("features/slash_commands.txt")).unwrap(),
        "implemented slash commands\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("features/review_completion.txt")).unwrap(),
        "implemented review completion\n"
    );

    assert!(controller.start_linearize().unwrap().is_some());
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    let patch_launches = launches
        .iter()
        .filter(|launch| launch.id.as_str().starts_with("user-"))
        .count();
    let review_launches = launches
        .iter()
        .filter(|launch| launch.id.as_str().starts_with("review-user-"))
        .count();
    assert_eq!(patch_launches, 3, "{launches:?}");
    assert_eq!(review_launches, 3, "{launches:?}");
    let interrupts = backend.interrupts();
    for agent_id in [
        "user-1",
        "user-2",
        "user-3",
        "review-user-1",
        "review-user-2",
        "review-user-3",
    ] {
        assert!(
            interrupts
                .iter()
                .any(|interrupted| interrupted.as_str() == agent_id),
            "{interrupts:?}"
        );
    }
    let snapshot = controller.snapshot();
    for agent_id in [&visual_id, &slash_id, &completion_id] {
        let session = snapshot.session(agent_id).expect("patch session exists");
        assert!(
            session
                .lines
                .iter()
                .any(|line| line.contains("stopped Codex before linearize")),
            "{session:?}"
        );
    }
    let linearize_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "linearize")
        .expect("linearize agent launched");
    assert!(linearize_launch.prompt.contains("Agent-ID: user-1"));
    assert!(linearize_launch.prompt.contains("Agent-ID: user-2"));
    assert!(linearize_launch.prompt.contains("Agent-ID: user-3"));
}

#[test]
fn controller_does_not_run_project_required_checks_after_agent_reply() {
    let root = git_repo("workspace-no-required-check-run");
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\n- `sh check.sh`\n",
    )
    .unwrap();
    fs::write(
        root.join("check.sh"),
        "#!/bin/sh\necho state is bad\nexit 1\n",
    )
    .unwrap();
    fs::write(root.join("state.txt"), "bad\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD project instruction fixture"]);
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(root.clone(), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("inspect required checks").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert_eq!(fs::read_to_string(root.join("state.txt")).unwrap(), "bad\n");
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.loading, None);
    assert!(
        !session
            .lines
            .iter()
            .any(|line| line.contains("required check failed"))
    );
    assert!(backend.sends().is_empty());
}

#[test]
fn controller_keeps_agent_loading_scoped_to_the_active_session() {
    let backend = ConcurrentBackend;
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let first = controller.create_agent("first task").unwrap();
    let second = controller.create_agent("second task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    controller.send_message(&second, "slow question").unwrap();
    assert_eq!(
        controller
            .snapshot()
            .session(&second)
            .expect("second session")
            .loading,
        Some(WorkLeafLoading::WaitingForReply)
    );

    controller.send_message(&first, "quick question").unwrap();

    assert!(controller.wait_for_session_line(&first, "quick reply", Duration::from_millis(150)));
    let snapshot = controller.snapshot();
    let first_session = snapshot.session(&first).expect("first session");
    assert!(first_session.lines.iter().any(|line| line == "quick reply"));
    assert!(
        !first_session
            .lines
            .iter()
            .any(|line| line.contains("still working"))
    );
}

#[test]
fn controller_queues_same_agent_prompts_while_the_chat_is_working() {
    let backend = QueuedPromptBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("queued prompt task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.drain_events();

    controller
        .send_message(&agent_id, "first slow prompt")
        .unwrap();
    assert!(backend.wait_for_send_count(1, Duration::from_secs(1)));

    controller
        .send_message(&agent_id, "second queued prompt")
        .unwrap();

    let queued = controller.snapshot();
    let session = queued.session(&agent_id).expect("session exists");
    assert!(
        session
            .lines
            .iter()
            .any(|line| line == "user: second queued prompt")
    );
    assert!(
        !session
            .lines
            .iter()
            .any(|line| line.contains("still working"))
    );
    assert_eq!(
        backend.sends(),
        vec![(agent_id.clone(), "first slow prompt".to_string())]
    );

    thread::sleep(Duration::from_millis(50));
    assert_eq!(
        backend.sends(),
        vec![(agent_id.clone(), "first slow prompt".to_string())]
    );

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert_eq!(
        backend.sends(),
        vec![
            (agent_id.clone(), "first slow prompt".to_string()),
            (agent_id.clone(), "second queued prompt".to_string()),
        ]
    );
    let replied = controller.snapshot();
    let session = replied.session(&agent_id).expect("session exists");
    assert!(
        session
            .lines
            .iter()
            .any(|line| line == "reply to first slow prompt")
    );
    assert!(
        session
            .lines
            .iter()
            .any(|line| line == "reply to second queued prompt")
    );
}

#[test]
fn controller_interrupt_clears_visible_agent_loading_immediately() {
    let backend = InterruptibleBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("interruptible task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    controller.send_message(&agent_id, "keep working").unwrap();
    assert_eq!(
        controller
            .snapshot()
            .session(&agent_id)
            .expect("session exists")
            .loading,
        Some(WorkLeafLoading::WaitingForReply)
    );

    controller.interrupt_agent(&agent_id);

    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.loading, None);
    assert!(
        session
            .lines
            .iter()
            .any(|line| line.contains("work-leaf: sent Ctrl-C to "))
    );
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
}

#[test]
fn controller_marks_secondary_follow_up_streams_as_waiting_until_worker_finishes() {
    let backend = SecondaryFollowUpBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let first = controller.create_agent("first task").unwrap();
    let second = controller.create_agent("second task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    controller.send_message(&first, "route follow-up").unwrap();

    assert!(controller.wait_for_session_line(
        &second,
        "secondary follow-up started",
        Duration::from_secs(1)
    ));
    let active = controller.snapshot();
    assert_eq!(
        active.session(&second).expect("second session").loading,
        Some(WorkLeafLoading::WaitingForReply)
    );

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let idle = controller.snapshot();
    assert_eq!(idle.session(&second).expect("second session").loading, None);
    assert!(
        idle.session(&second)
            .expect("second session")
            .lines
            .iter()
            .any(|line| line == "secondary follow-up complete")
    );
}

#[test]
fn controller_drains_queued_prompts_for_secondary_streamed_sessions() {
    let backend = SecondaryFollowUpBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let first = controller.create_agent("first task").unwrap();
    let second = controller.create_agent("second task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    controller.send_message(&first, "route follow-up").unwrap();
    assert!(controller.wait_for_session_line(
        &second,
        "secondary follow-up started",
        Duration::from_secs(1)
    ));

    controller
        .send_message(&second, "queued secondary prompt")
        .unwrap();

    let queued = controller.snapshot();
    let second_session = queued.session(&second).expect("second session");
    assert!(
        second_session
            .lines
            .iter()
            .any(|line| line == "user: queued secondary prompt")
    );
    let sends = backend.sends();
    assert_eq!(sends.len(), 2);
    assert_eq!(sends[0], (first.clone(), "route follow-up".to_string()));
    assert_eq!(sends[1].0, second);
    assert!(sends[1].1.contains("follow-up work"));
    assert!(!sends[1].1.contains("queued secondary prompt"));

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let sends = backend.sends();
    assert_eq!(sends.len(), 3);
    assert_eq!(sends[0], (first, "route follow-up".to_string()));
    assert_eq!(sends[1].0, second);
    assert!(sends[1].1.contains("follow-up work"));
    assert_eq!(
        sends[2],
        (second.clone(), "queued secondary prompt".to_string())
    );
    let idle = controller.snapshot();
    let second_session = idle.session(&second).expect("second session");
    assert_eq!(second_session.loading, None);
    assert!(
        second_session
            .lines
            .iter()
            .any(|line| line == "ordinary reply")
    );
}

#[derive(Clone, Debug)]
struct FakeBackend {
    state: Arc<Mutex<FakeBackendState>>,
}

#[derive(Debug)]
struct FakeBackendState {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
    sessions: BTreeMap<AgentId, AgentSession>,
}

#[derive(Clone, Debug)]
struct ConcurrentBackend;

#[derive(Clone, Debug, Default)]
struct QueuedPromptBackend {
    sends: Arc<Mutex<Vec<(AgentId, String)>>>,
}

#[derive(Clone, Debug, Default)]
struct InterruptibleBackend {
    interrupted: Arc<Mutex<bool>>,
}

#[derive(Clone, Debug, Default)]
struct SecondaryFollowUpBackend {
    sessions: Arc<Mutex<BTreeMap<AgentId, AgentSession>>>,
    sends: Arc<Mutex<Vec<(AgentId, String)>>>,
}

#[derive(Clone, Debug)]
struct StreamingTranscriptBackend;

#[derive(Clone, Debug, Default)]
struct ThreeFeatureBackend {
    state: Arc<Mutex<ThreeFeatureState>>,
}

#[derive(Debug, Default)]
struct ThreeFeatureState {
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
    interrupts: Vec<AgentId>,
}

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeBackendState {
                replies: replies.into_iter().map(String::from).collect(),
                launches: Vec::new(),
                sends: Vec::new(),
                sessions: BTreeMap::new(),
            })),
        }
    }

    fn launches(&self) -> Vec<AgentLaunch> {
        self.state
            .lock()
            .unwrap()
            .launches
            .iter()
            .filter(|launch| !is_system_agent_launch(launch))
            .cloned()
            .collect()
    }

    fn all_launches(&self) -> Vec<AgentLaunch> {
        self.state.lock().unwrap().launches.clone()
    }

    fn sends(&self) -> Vec<(AgentId, String)> {
        self.state
            .lock()
            .unwrap()
            .sends
            .iter()
            .filter(|(agent_id, _)| {
                agent_id.as_str() != "command-agent" && !agent_id.as_str().starts_with("title-")
            })
            .cloned()
            .collect()
    }

    fn all_sends(&self) -> Vec<(AgentId, String)> {
        self.state.lock().unwrap().sends.clone()
    }

    fn next_reply(&self) -> String {
        self.state
            .lock()
            .unwrap()
            .replies
            .pop_front()
            .expect("missing fake reply")
    }

    fn next_reviewer_reply(&self) -> String {
        let mut state = self.state.lock().unwrap();
        let reply = state.replies.pop_front().expect("missing fake reply");
        if reply.starts_with("summary:") && !state.replies.is_empty() {
            state.replies.pop_front().expect("missing fake reply")
        } else {
            reply
        }
    }

    fn title_reply(&self, prompt: &str) -> String {
        fake_title_from_title_prompt(prompt)
    }
}

fn is_system_agent_launch(launch: &AgentLaunch) -> bool {
    launch.id.as_str() == "command-agent" || launch.id.as_str().starts_with("title-")
}

impl ThreeFeatureBackend {
    fn launches(&self) -> Vec<AgentLaunch> {
        self.state.lock().unwrap().launches.clone()
    }

    fn interrupts(&self) -> Vec<AgentId> {
        self.state.lock().unwrap().interrupts.clone()
    }

    fn launch_reply(request: &AgentLaunch) -> String {
        if request.id.as_str().starts_with("title-") {
            return fake_title_from_title_prompt(&request.prompt);
        }
        if request.id.as_str().starts_with("review-") {
            return "NO_FINDINGS".to_string();
        }
        if request.id.as_str() == "linearize" {
            return "linearizer ready".to_string();
        }
        if request.prompt.contains("visual mode") {
            return feature_patch(
                "vim visual mode",
                "features/visual_mode.txt",
                "implemented visual mode",
            );
        }
        if request.prompt.contains("slash") {
            return feature_patch(
                "agent slash command pass-through",
                "features/slash_commands.txt",
                "implemented slash commands",
            );
        }
        if request.prompt.contains("yes or no") {
            return feature_patch(
                "review completion question",
                "features/review_completion.txt",
                "implemented review completion",
            );
        }
        panic!("unexpected launch prompt: {}", request.prompt);
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.state.lock().unwrap().launches.push(request.clone());
        let mut session = AgentSession::new(request);
        let agent_id = session.id.clone();
        let reply = if session.id.as_str().starts_with("title-") {
            self.title_reply(&session.messages[0].text)
        } else if session.id.as_str().starts_with("review-") {
            self.next_reviewer_reply()
        } else {
            self.next_reply()
        };
        session.push_message(MessageRole::Agent, reply);
        self.state
            .lock()
            .unwrap()
            .sessions
            .insert(agent_id, session.clone());
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str().starts_with("title-") {
            let reply = self.title_reply(prompt);
            let mut state = self.state.lock().unwrap();
            state.sends.push((agent_id.clone(), prompt.to_string()));
            if let Some(session) = state.sessions.get_mut(agent_id) {
                session.push_message(MessageRole::User, prompt);
                session.push_message(MessageRole::Agent, reply.clone());
            }
            return Ok(ChatMessage::new(MessageRole::Agent, reply));
        }

        let mut state = self.state.lock().unwrap();
        state.sends.push((agent_id.clone(), prompt.to_string()));
        let reply = state.replies.pop_front().expect("missing fake reply");
        let reply = if agent_id.as_str().starts_with("review-")
            && reply.starts_with("summary:")
            && !state.replies.is_empty()
        {
            state.replies.pop_front().expect("missing fake reply")
        } else {
            reply
        };
        if let Some(session) = state.sessions.get_mut(agent_id) {
            session.push_message(MessageRole::User, prompt);
            session.push_message(MessageRole::Agent, reply.clone());
        }
        Ok(ChatMessage::new(MessageRole::Agent, reply))
    }

    fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        self.state.lock().unwrap().sessions.get(agent_id).cloned()
    }
}

impl AgentBackend for StreamingTranscriptBackend {
    fn session(&self, _agent_id: &AgentId) -> Option<AgentSession> {
        None
    }

    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(
            MessageRole::Agent,
            "@work-leaf read src/ui.rs\n\n@work-leaf done",
        );
        Ok(session)
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        sink(AgentStreamEvent::AgentMessage(
            "@work-leaf read src/ui.rs".to_string(),
        ));
        sink(AgentStreamEvent::AgentMessage(
            "@work-leaf done".to_string(),
        ));
        self.launch(request)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "unused"))
    }
}

impl AgentBackend for ThreeFeatureBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let reply = Self::launch_reply(&request);
        self.state.lock().unwrap().launches.push(request.clone());
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, reply);
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str().starts_with("title-") {
            return Ok(ChatMessage::new(
                MessageRole::Agent,
                fake_title_from_title_prompt(prompt),
            ));
        }
        self.state
            .lock()
            .unwrap()
            .sends
            .push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(
            MessageRole::Agent,
            "summary: implemented the requested fixture feature",
        ))
    }

    fn interrupt(&mut self, agent_id: &AgentId) -> Result<(), AgentError> {
        self.state.lock().unwrap().interrupts.push(agent_id.clone());
        Ok(())
    }
}

impl AgentBackend for ConcurrentBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str().starts_with("title-") {
            return Ok(ChatMessage::new(
                MessageRole::Agent,
                fake_title_from_title_prompt(prompt),
            ));
        }
        if agent_id.as_str() == "user-2" {
            thread::sleep(Duration::from_millis(350));
            return Ok(ChatMessage::new(MessageRole::Agent, "slow reply"));
        }
        Ok(ChatMessage::new(MessageRole::Agent, "quick reply"))
    }
}

impl QueuedPromptBackend {
    fn sends(&self) -> Vec<(AgentId, String)> {
        self.sends.lock().unwrap().clone()
    }

    fn wait_for_send_count(&self, expected: usize, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if self.sends.lock().unwrap().len() >= expected {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.sends.lock().unwrap().len() >= expected
    }
}

impl AgentBackend for QueuedPromptBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str().starts_with("title-") {
            return Ok(ChatMessage::new(
                MessageRole::Agent,
                fake_title_from_title_prompt(prompt),
            ));
        }
        self.sends
            .lock()
            .unwrap()
            .push((agent_id.clone(), prompt.to_string()));
        if prompt == "first slow prompt" {
            thread::sleep(Duration::from_millis(250));
        }
        Ok(ChatMessage::new(
            MessageRole::Agent,
            format!("reply to {prompt}"),
        ))
    }
}

impl AgentBackend for SecondaryFollowUpBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        self.sessions
            .lock()
            .unwrap()
            .insert(session.id.clone(), session.clone());
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str().starts_with("title-") {
            return Ok(ChatMessage::new(
                MessageRole::Agent,
                fake_title_from_title_prompt(prompt),
            ));
        }
        self.record_send(agent_id, prompt);
        let reply = if agent_id.as_str() == "user-1" {
            "@work-leaf send user-2 follow-up work"
        } else if prompt.contains("follow-up work") {
            "secondary follow-up complete"
        } else {
            "ordinary reply"
        };
        Ok(ChatMessage::new(MessageRole::Agent, reply))
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str() == "user-2" && prompt.contains("follow-up work") {
            self.record_send(agent_id, prompt);
            sink(AgentStreamEvent::AgentMessage(
                "secondary follow-up started".to_string(),
            ));
            thread::sleep(Duration::from_millis(150));
            return Ok(ChatMessage::new(
                MessageRole::Agent,
                "secondary follow-up complete",
            ));
        }
        self.send(agent_id, prompt)
    }

    fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        self.sessions.lock().unwrap().get(agent_id).cloned()
    }
}

impl SecondaryFollowUpBackend {
    fn sends(&self) -> Vec<(AgentId, String)> {
        self.sends.lock().unwrap().clone()
    }

    fn record_send(&self, agent_id: &AgentId, prompt: &str) {
        self.sends
            .lock()
            .unwrap()
            .push((agent_id.clone(), prompt.to_string()));
    }
}

impl AgentBackend for InterruptibleBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str().starts_with("title-") {
            return Ok(ChatMessage::new(
                MessageRole::Agent,
                fake_title_from_title_prompt(prompt),
            ));
        }
        loop {
            if *self.interrupted.lock().unwrap() {
                return Ok(ChatMessage::new(MessageRole::Agent, "interrupted"));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn interrupt(&mut self, _agent_id: &AgentId) -> Result<(), AgentError> {
        *self.interrupted.lock().unwrap() = true;
        Ok(())
    }
}

fn feature_patch(feature: &str, path: &str, implemented: &str) -> String {
    format!(
        "implemented {feature}\n@work-leaf patch {feature}\n--- a/{path}\n+++ b/{path}\n@@ -1 +1 @@\n-missing\n+{implemented}\n@work-leaf end\n@work-leaf done"
    )
}

fn git_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
    git(&root, ["init", "-q"]);
    git(&root, ["config", "user.email", "test@example.com"]);
    git(&root, ["config", "user.name", "Test User"]);
    root
}

fn git<const N: usize>(root: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn fake_title_from_title_prompt(prompt: &str) -> String {
    let first_prompt = prompt
        .rsplit_once("First prompt:\n")
        .map(|(_, first_prompt)| first_prompt)
        .unwrap_or(prompt);
    if first_prompt.contains("parser combinator") {
        "implement-parser-combinator".to_string()
    } else if first_prompt.contains("login callback")
        || first_prompt.contains("OAuth redirect handler")
    {
        "oauth-redirect-handler".to_string()
    } else {
        "chat-title".to_string()
    }
}
