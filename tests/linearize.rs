use std::collections::VecDeque;

use work_leaf::{
    AgentBackend, AgentCommit, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession,
    ChatMessage, LinearizeAction, LinearizeGroup, LinearizePlan, LinearizePlanner, MessageRole,
};

#[test]
fn linearize_questions_cover_each_reviewed_chat_id() {
    let commits = vec![
        agent_commit(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "chat-a",
            "parser",
            "parse values",
        ),
        agent_commit(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "chat-b",
            "docs",
            "document parser",
        ),
    ];

    let questions = LinearizePlanner::<FakeBackend>::questions_for(&commits);

    assert_eq!(questions.len(), 2);
    assert_eq!(questions[0].agent_id.as_str(), "chat-a");
    assert!(questions[0].prompt.contains("keep a final commit"));
    assert!(
        questions[0]
            .prompt
            .contains("integrate into another commit")
    );
    assert_eq!(questions[1].agent_id.as_str(), "chat-b");
}

#[test]
fn linearize_questions_compact_multiple_reviewed_hashes_from_one_patch_agent() {
    let commits = vec![
        agent_commit(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "chat-a",
            "parser",
            "first fix",
        ),
        agent_commit(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "chat-a",
            "parser",
            "second fix",
        ),
    ];

    let questions = LinearizePlanner::<FakeBackend>::questions_for(&commits);

    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].agent_id.as_str(), "chat-a");
    assert!(questions[0].prompt.contains("one final commit"));
}

#[test]
fn interactive_linearize_prompt_requires_user_accepted_plan_before_rewrite() {
    let commits = vec![agent_commit(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "chat-a",
        "parser",
        "parse values",
    )];

    let prompt = LinearizePlanner::<FakeBackend>::interactive_prompt(&commits);

    assert!(prompt.contains("which final commit message should be kept"));
    assert!(prompt.contains("which provisional commit message should be removed"));
    assert!(prompt.contains("Ask the user to accept the solution or request changes"));
    assert!(prompt.contains("Do not rewrite history until the user accepts"));
    assert!(prompt.contains("Only the reviewed commits listed in this prompt are in scope"));
    assert!(prompt.contains("one final commit per listed patch agent"));
    assert!(prompt.contains("AGENTS.md commit message rules"));
    assert!(prompt.contains("parent/common base of the listed reviewed commits"));
    assert!(prompt.contains("Do not retarget the final commits onto main/master"));
    assert!(prompt.contains("Run the checks required by the repository instructions"));
    assert!(prompt.contains("Update documentation and plain-text files directly"));
    assert!(
        prompt.contains(
            "do not use `@work-leaf read`, `@work-leaf edit`, `@work-leaf patch`, or `@work-leaf locks run`"
        )
    );
    assert!(prompt.contains("Agent-ID: chat-a"));
    assert!(prompt.contains("Subject: UPDATE apply parser patch from chat-a"));
}

#[test]
fn interactive_linearize_prompt_keeps_one_final_target_per_patch_agent() {
    let commits = vec![
        agent_commit(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "chat-a",
            "parser",
            "first fix",
        ),
        agent_commit(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "chat-a",
            "parser",
            "second fix",
        ),
        agent_commit(
            "cccccccccccccccccccccccccccccccccccccccc",
            "chat-b",
            "slash",
            "slash commands",
        ),
    ];

    let prompt = LinearizePlanner::<FakeBackend>::interactive_prompt(&commits);

    assert_eq!(prompt.matches("Agent-ID: chat-a").count(), 1);
    assert_eq!(prompt.matches("Agent-ID: chat-b").count(), 1);
    assert!(prompt.contains("first fix"));
    assert!(prompt.contains("second fix"));
    assert!(prompt.contains("exactly 2 final commits"));
    assert!(prompt.contains("Do not keep or create separate support, test-hygiene"));
}

#[test]
fn linearize_handoff_launches_codex_agent_with_decisions_groups_and_tests() {
    let commits = vec![
        agent_commit(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "chat-a",
            "parser",
            "parse values",
        ),
        agent_commit(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "chat-b",
            "docs",
            "document parser",
        ),
        agent_commit(
            "cccccccccccccccccccccccccccccccccccccccc",
            "chat-c",
            "cli",
            "wire command",
        ),
    ];
    let plan = LinearizePlan::new(commits)
        .decide(
            AgentId::new("chat-a").unwrap(),
            LinearizeAction::KeepFinalCommit,
        )
        .decide(
            AgentId::new("chat-b").unwrap(),
            LinearizeAction::IntegrateInto(AgentId::new("chat-a").unwrap()),
        )
        .decide(
            AgentId::new("chat-c").unwrap(),
            LinearizeAction::KeepFinalCommit,
        )
        .group(LinearizeGroup::new(
            "parser-and-cli",
            [
                AgentId::new("chat-a").unwrap(),
                AgentId::new("chat-c").unwrap(),
            ],
        ))
        .test_command(["cargo", "test"]);
    let backend = FakeBackend::new(["linearizer ready"]);
    let mut planner = LinearizePlanner::new(backend);

    let handoff = planner.launch_linearizer(plan).unwrap();
    let backend = planner.into_backend();

    assert_eq!(handoff.linearizer_id.as_str(), "linearize");
    assert_eq!(handoff.initial_reply, "linearizer ready");
    assert_eq!(backend.launches.len(), 1);
    assert_eq!(backend.launches[0].kind, AgentKind::Codex);
    assert_eq!(backend.launches[0].feature, "linearize reviewed patches");

    let prompt = &backend.launches[0].prompt;
    assert!(prompt.contains("chat-a: keep a final commit"));
    assert!(prompt.contains("chat-b: integrate into chat-a"));
    assert!(prompt.contains("Group parser-and-cli: chat-a, chat-c"));
    assert!(prompt.contains("cargo test"));
    assert!(prompt.contains("parent/common base of the listed reviewed commits"));
    assert!(prompt.contains("Do not retarget onto main/master"));
    assert!(
        prompt
            .contains("Documentation and plain-text files intentionally deferred by patch agents")
    );
    assert!(
        prompt.contains("Use direct workspace reads, writes, commands, and git history rewrites")
    );
}

#[derive(Debug)]
struct FakeBackend {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
}

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            replies: replies.into_iter().map(String::from).collect(),
            launches: Vec::new(),
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

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, self.next_reply()))
    }
}

fn agent_commit(hash: &str, agent_id: &str, feature: &str, reason: &str) -> AgentCommit {
    AgentCommit {
        hash: hash.to_string(),
        agent_id: AgentId::new(agent_id).unwrap(),
        feature: feature.to_string(),
        reason: reason.to_string(),
        context: format!("context for {feature}"),
        subject: format!("UPDATE apply {feature} patch from {agent_id}"),
        body: format!(
            "UPDATE apply {feature} patch from {agent_id}\n\nAgent-ID: {agent_id}\nFeature: {feature}\nReason: {reason}\nContext: context for {feature}"
        ),
    }
}
