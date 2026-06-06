use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, FileLockTable,
    GitHistory, GitPatcher, LinearizeAction, LinearizePlan, LinearizePlanner, MessageRole,
    PatchRequest, ReviewCoordinator,
};

#[test]
fn non_ui_agent_patch_review_and_linearize_flow_works_end_to_end() {
    let root = git_repo("non-ui-flow");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let patch = patcher
        .apply(PatchRequest::new(
            AgentId::new("user-1").unwrap(),
            "parser",
            "return parsed value",
            "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1 +1 @@
-pub fn value() -> u8 { 1 }
+pub fn value() -> u8 { 2 }
",
        ))
        .unwrap();

    assert_eq!(patch.files, vec![PathBuf::from("src/lib.rs")]);
    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "pub fn value() -> u8 { 2 }\n"
    );

    let commits = GitHistory::new(root.clone())
        .latest_agent_commits()
        .unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].agent_id.as_str(), "user-1");
    assert_eq!(commits[0].feature, "parser");
    assert_eq!(commits[0].reason, "return parsed value");

    let backend = FakeBackend::new([
        "summary: returns parsed value",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let mut reviewer = ReviewCoordinator::new(root.clone(), backend);
    let review_results = reviewer.review_latest_agent_commits().unwrap();
    let backend = reviewer.into_backend();

    assert_eq!(review_results.len(), 1);
    assert!(review_results[0].findings_resolved);
    assert_eq!(backend.sends[0].0.as_str(), "user-1");
    assert!(backend.sends[0].1.contains("summarize"));
    assert_eq!(backend.launches[0].id.as_str(), "review-user-1");
    assert!(backend.launches[0].prompt.contains("NO_FINDINGS"));

    let questions = LinearizePlanner::<FakeBackend>::questions_for(&commits);
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].agent_id.as_str(), "user-1");

    let plan = LinearizePlan::new(commits).decide(
        AgentId::new("user-1").unwrap(),
        LinearizeAction::KeepFinalCommit,
    );
    let mut linearizer = LinearizePlanner::new(backend);
    let handoff = linearizer.launch_linearizer(plan).unwrap();
    let backend = linearizer.into_backend();

    assert_eq!(handoff.linearizer_id.as_str(), "linearize");
    assert_eq!(handoff.initial_reply, "linearizer ready");
    assert_eq!(backend.launches[1].id.as_str(), "linearize");
    assert!(
        backend.launches[1]
            .prompt
            .contains("user-1: keep a final commit")
    );
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
