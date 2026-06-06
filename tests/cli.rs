use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Command;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentSession, ChatMessage, CommandChat, CommandChatResult,
    MessageRole, ProcessCommand, parse_process_args, render_process_help,
};

#[test]
fn binary_help_describes_launching_orchestrator_not_internal_operations() {
    let output = Command::new(env!("CARGO_BIN_EXE_work-leaf"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: work-leaf [--model <model>]"));
    assert!(stdout.contains("launches the orchestrator"));
    assert!(stdout.contains("command chat"));
    assert!(!stdout.contains("patch <agent-id>"));
    assert!(!stdout.contains("locks classify"));
    assert!(!stdout.contains("linearize-questions"));
}

#[test]
fn no_args_launches_orchestrator_from_current_project_directory() {
    let command = parse_process_args(["work-leaf"]).unwrap();

    assert_eq!(command, ProcessCommand::Launch { model: None });
}

#[test]
fn top_level_internal_commands_are_rejected() {
    for args in [
        vec!["work-leaf", "new", "chat-a", "parser", "implement parser"],
        vec![
            "work-leaf",
            "patch",
            "chat-a",
            "parser",
            "reason",
            "diff.patch",
        ],
        vec!["work-leaf", "review"],
        vec!["work-leaf", "linearize-questions"],
        vec!["work-leaf", "locks", "classify", "cargo", "test"],
    ] {
        let error = parse_process_args(args).unwrap_err().to_string();

        assert!(error.contains("work-leaf does not accept top-level workflow commands"));
        assert!(error.contains("command chat"));
    }
}

#[test]
fn command_chat_launches_agents_inside_the_orchestrator() {
    let backend = FakeBackend::new(["agent ready"]);
    let mut chat = CommandChat::new(PathBuf::from("/repo"), backend);

    let result = chat.handle_line("new implement the parser").unwrap();

    assert_eq!(
        result,
        CommandChatResult::AgentLaunched {
            agent_id: AgentId::new("user-1").unwrap(),
            feature: "user-agent".to_string(),
            reply: "agent ready".to_string(),
        }
    );
    let backend = chat.into_backend();
    assert_eq!(backend.launches.len(), 1);
    assert_eq!(backend.launches[0].id.as_str(), "user-1");
    assert_eq!(backend.launches[0].feature, "user-agent");
    assert_eq!(backend.launches[0].prompt, "implement the parser");
}

#[test]
fn process_help_mentions_internal_actions_as_in_app_commands_only() {
    let help = render_process_help();

    assert!(help.contains("Inside command chat"));
    assert!(help.contains("new <prompt...>"));
    assert!(help.contains("review"));
    assert!(help.contains("linearize"));
    assert!(!help.contains("Usage: work-leaf <command>"));
}

#[derive(Debug)]
struct FakeBackend {
    replies: VecDeque<String>,
    launches: Vec<work_leaf::AgentLaunch>,
}

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            replies: replies.into_iter().map(String::from).collect(),
            launches: Vec::new(),
        }
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: work_leaf::AgentLaunch) -> Result<AgentSession, AgentError> {
        self.launches.push(request.clone());
        let mut session = AgentSession::new(request);
        session.push_message(
            MessageRole::Agent,
            self.replies.pop_front().expect("missing fake reply"),
        );
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(
            MessageRole::Agent,
            self.replies.pop_front().expect("missing fake reply"),
        ))
    }
}
