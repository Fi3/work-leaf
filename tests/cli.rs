use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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
fn command_chat_new_without_prompt_opens_interactive_agent_session() {
    let backend = FakeBackend::new(["agent ready"]);
    let mut chat = CommandChat::new(PathBuf::from("/repo"), backend);

    let result = chat.handle_line("new").unwrap();

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
    assert!(
        backend.launches[0]
            .prompt
            .contains("Ask the user what to work on")
    );
}

#[test]
fn failed_agent_launch_does_not_consume_user_agent_id() {
    let backend = FlakyLaunchBackend::default();
    let mut chat = CommandChat::new(PathBuf::from("/repo"), backend);

    let error = chat.handle_line("new first try").unwrap_err().to_string();
    assert!(error.contains("first launch failed"));

    let result = chat.handle_line("new retry").unwrap();

    assert_eq!(
        result,
        CommandChatResult::AgentLaunched {
            agent_id: AgentId::new("user-1").unwrap(),
            feature: "user-agent".to_string(),
            reply: "agent ready".to_string(),
        }
    );
}

#[test]
fn scripted_command_chat_reports_agent_launch_error_without_exiting() {
    let root = temp_dir("scripted-new-error");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    fs::write(&codex, "#!/bin/sh\necho codex launch failed >&2\nexit 42\n").unwrap();
    make_executable(&codex);

    let mut child = Command::new(env!("CARGO_BIN_EXE_work-leaf"))
        .env("PATH", format!("{}:{}", fake_bin.display(), current_path()))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"new\nhelp\nquit\n")
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("work-leaf orchestrator"));
    assert!(stdout.contains("codex launch failed"));
    assert!(stdout.contains("Command chat:"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty());
}

#[test]
fn process_help_mentions_internal_actions_as_in_app_commands_only() {
    let help = render_process_help();

    assert!(help.contains("Inside command chat"));
    assert!(help.contains("new [prompt...]"));
    assert!(help.contains("review"));
    assert!(help.contains("linearize"));
    assert!(!help.contains("Usage: work-leaf <command>"));
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn current_path() -> String {
    std::env::var("PATH").unwrap_or_default()
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) {}

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

#[derive(Debug, Default)]
struct FlakyLaunchBackend {
    attempts: usize,
}

impl AgentBackend for FlakyLaunchBackend {
    fn launch(&mut self, request: work_leaf::AgentLaunch) -> Result<AgentSession, AgentError> {
        self.attempts += 1;
        if self.attempts == 1 {
            return Err(AgentError::ProcessFailed {
                program: PathBuf::from("codex"),
                status: Some(42),
                stderr: "first launch failed".to_string(),
            });
        }
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "agent ready");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "reply"))
    }
}
