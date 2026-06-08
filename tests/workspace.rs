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
    assert_eq!(session.title, "user-agent");
    assert_eq!(session.loading, Some(WorkLeafLoading::Launching));

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let ready = controller.snapshot();
    let session = ready.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "parser-combinator");
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
fn controller_uses_backend_agent_to_name_chat_from_first_prompt() {
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

    let launches = backend.launches();
    assert!(launches.iter().any(|launch| {
        launch.id.as_str() == "title-user-1"
            && launch.feature == "chat-title"
            && launch.prompt.contains("please fix login callback")
    }));
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
    }));
    assert!(sends.iter().any(|(target, prompt)| {
        target == &reviewer_id
            && prompt.contains("The original agent has responded to the findings")
            && prompt.contains("Please check the patch again")
    }));
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
            && prompt.contains("Review the final patch")
            && prompt.contains("after second")
    }));
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
fn controller_linearize_keeps_multiple_reviewed_commits_from_same_agent() {
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
        2
    );
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

    fn sends(&self) -> Vec<(AgentId, String)> {
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

    fn title_reply(&self, prompt: &str) -> String {
        fake_title_from_title_prompt(prompt)
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.state.lock().unwrap().launches.push(request.clone());
        let mut session = AgentSession::new(request);
        let reply = if session.id.as_str().starts_with("title-") {
            self.title_reply(&session.messages[0].text)
        } else {
            self.next_reply()
        };
        session.push_message(MessageRole::Agent, reply);
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

fn fake_title_from_title_prompt(prompt: &str) -> String {
    let first_prompt = prompt
        .rsplit_once("First prompt:\n")
        .map(|(_, first_prompt)| first_prompt)
        .unwrap_or(prompt);
    if first_prompt.contains("parser combinator") {
        "parser-combinator".to_string()
    } else if first_prompt.contains("login callback")
        || first_prompt.contains("OAuth redirect handler")
    {
        "oauth-redirect-handler".to_string()
    } else {
        "chat-title".to_string()
    }
}
