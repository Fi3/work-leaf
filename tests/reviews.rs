use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, ChatMessage,
    GitHistory, MessageRole, ReviewCoordinator,
};

mod temp_cleanup;

#[test]
fn git_history_finds_latest_commit_for_each_agent_id() {
    let root = git_repo("review-history");
    fs::write(root.join("one.txt"), "one\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply parser patch from chat-a",
            "-m",
            "Agent-ID: chat-a\nFeature: parser\nReason: first",
        ],
    );
    fs::write(root.join("two.txt"), "two\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply docs patch from chat-b",
            "-m",
            "Agent-ID: chat-b\nFeature: docs\nReason: docs",
        ],
    );
    fs::write(root.join("one.txt"), "one again\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply parser patch from chat-a",
            "-m",
            "Agent-ID: chat-a\nFeature: parser\nReason: second",
        ],
    );

    let commits = GitHistory::new(root).latest_agent_commits().unwrap();

    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].agent_id.as_str(), "chat-a");
    assert_eq!(commits[0].reason, "second");
    assert_eq!(commits[1].agent_id.as_str(), "chat-b");
    assert_eq!(commits[1].feature, "docs");
}

#[test]
fn review_coordinator_loops_until_reviewer_reports_no_findings() {
    let root = git_repo("review-loop");
    fs::write(root.join("lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply parser patch from chat-a",
            "-m",
            "Agent-ID: chat-a\nFeature: parser\nReason: return value",
        ],
    );

    let backend = FakeBackend::new([
        "summary: returns a parser value",
        "FINDINGS\n- missing edge case",
        "fixed missing edge case",
        "NO_FINDINGS",
    ]);
    let mut coordinator = ReviewCoordinator::new(root, backend).with_max_rounds(4);

    let results = coordinator.review_latest_agent_commits().unwrap();
    let backend = coordinator.into_backend();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].agent_id.as_str(), "chat-a");
    assert_eq!(results[0].reviewer_id.as_str(), "review-chat-a");
    assert_eq!(results[0].rounds, 2);
    assert!(results[0].findings_resolved);

    assert_eq!(backend.launches.len(), 1);
    assert_eq!(backend.launches[0].id.as_str(), "review-chat-a");
    assert_eq!(backend.launches[0].kind, AgentKind::Codex);
    assert!(
        backend.launches[0]
            .prompt
            .contains("summary: returns a parser value")
    );
    assert!(backend.launches[0].prompt.contains("chat-a"));
    assert!(
        backend.launches[0]
            .prompt
            .contains("Documentation and plain-text updates are deferred to the linearize agent")
    );

    assert_eq!(backend.sends.len(), 3);
    assert_eq!(backend.sends[0].0.as_str(), "chat-a");
    assert!(backend.sends[0].1.contains("summarize"));
    assert_eq!(backend.sends[1].0.as_str(), "chat-a");
    assert!(backend.sends[1].1.contains("missing edge case"));
    assert!(
        backend.sends[1]
            .1
            .contains("Do not modify documentation or plain-text files")
    );
    assert_eq!(backend.sends[2].0.as_str(), "review-chat-a");
    assert!(backend.sends[2].1.contains("check the patch again"));
    assert!(
        backend.sends[2]
            .1
            .contains("must not be reported as remaining patch-agent findings")
    );
}

#[test]
fn git_history_builds_agent_review_scope_since_baseline() {
    let root = git_repo("review-history-scope");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial review scope fixture"]);
    let baseline = GitHistory::new(root.clone()).head_hash().unwrap().unwrap();
    fs::write(root.join("README.md"), "after first\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply first patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: readme\nReason: first step\nContext: first context",
        ],
    );
    fs::write(root.join("README.md"), "after second\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply second patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: readme\nReason: second step\nContext: second context",
        ],
    );

    let target = GitHistory::new(root)
        .agent_review_commit(&AgentId::new("user-1").unwrap(), Some(&baseline))
        .unwrap()
        .expect("review target");

    assert!(target.reason.contains("2 provisional commits"));
    assert!(target.context.contains("first step"));
    assert!(target.context.contains("second step"));
    assert_eq!(target.feature, "readme");
}

#[derive(Debug)]
struct FakeBackend {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
}

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            replies: replies.into_iter().map(String::from).collect(),
            launches: Vec::new(),
            sends: Vec::new(),
        }
    }

    fn next_reply(&mut self) -> String {
        self.replies.pop_front().expect("missing fake reply")
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let reply = self.next_reply();
        self.launches.push(request.clone());
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, reply);
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.sends.push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(MessageRole::Agent, self.next_reply()))
    }
}

fn git_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
    git(&root, ["init"]);
    git(&root, ["config", "user.name", "Work Leaf Test"]);
    git(&root, ["config", "user.email", "work-leaf@example.test"]);
    root
}

fn git<const N: usize>(root: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
