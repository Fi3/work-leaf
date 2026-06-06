use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentProfile, AgentSession,
    ChatMessage, CommandChat, MessageRole, WorkLeafController, WorkLeafEvent, WorkLeafLoading,
};

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
    assert_eq!(session.title, "parser combinator");
    assert_eq!(session.loading, Some(WorkLeafLoading::Launching));

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let ready = controller.snapshot();
    let session = ready.session(&agent_id).expect("session exists");
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
    let backend = FakeBackend::new(["launch reply", "summary", "NO_FINDINGS"]);
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

#[derive(Clone, Debug)]
struct FakeBackend {
    state: Arc<Mutex<FakeBackendState>>,
}

#[derive(Debug)]
struct FakeBackendState {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
}

#[derive(Clone, Debug)]
struct ConcurrentBackend;

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeBackendState {
                replies: replies.into_iter().map(String::from).collect(),
                launches: Vec::new(),
                sends: Vec::new(),
            })),
        }
    }

    fn launches(&self) -> Vec<AgentLaunch> {
        self.state.lock().unwrap().launches.clone()
    }

    fn next_reply(&self) -> String {
        self.state
            .lock()
            .unwrap()
            .replies
            .pop_front()
            .expect("missing fake reply")
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.state.lock().unwrap().launches.push(request.clone());
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, self.next_reply());
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.state
            .lock()
            .unwrap()
            .sends
            .push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(MessageRole::Agent, self.next_reply()))
    }
}

impl AgentBackend for ConcurrentBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str() == "user-2" {
            thread::sleep(Duration::from_millis(350));
            return Ok(ChatMessage::new(MessageRole::Agent, "slow reply"));
        }
        Ok(ChatMessage::new(MessageRole::Agent, "quick reply"))
    }
}

fn git_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
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
