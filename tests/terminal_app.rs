use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, CodexBackend,
    CodexCommandConfig, CommandChat, MessageRole, PaneFocus, PromptPolicy, TerminalApp, UiMode,
};

#[test]
fn terminal_app_new_and_chat_message_use_real_command_chat_backend() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let mut chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(&mut chat, 100, 24);

    app.handle_bytes(b":new implement parser\n");

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert_eq!(app.ui().mode(), UiMode::Insert);
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("launch reply"));

    app.handle_bytes(b"please continue\n");

    assert!(app.render_frame().contains("user-1> please continue"));
    assert!(app.render_frame().contains("follow reply"));
    let backend = chat.into_backend();
    assert_eq!(backend.launches.len(), 1);
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0.as_str(), "user-1");
    assert_eq!(backend.sends[0].1, "please continue");
}

#[test]
fn terminal_app_keeps_visible_cursor_on_chat_input() {
    let backend = FakeBackend::new(["launch reply"]);
    let mut chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(&mut chat, 100, 24);

    app.handle_bytes(b":new implement parser\nhello");

    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert!(app.render_frame().ends_with("\u{1b}[13;33H"));
}

#[test]
fn terminal_app_new_and_chat_work_through_spawned_codex_backend() {
    let root = temp_dir("terminal-app-codex-backend");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    fs::write(
        &codex,
        "\
#!/bin/sh
seen_exec=0
seen_resume=0
for arg in \"$@\"; do
  if [ \"$arg\" = \"exec\" ]; then
    seen_exec=1
  fi
  if [ \"$arg\" = \"resume\" ]; then
    seen_resume=1
  fi
  if [ \"$seen_exec\" = \"1\" ] && [ \"$arg\" = \"--ask-for-approval\" ]; then
    echo \"--ask-for-approval must be passed before exec\" >&2
    exit 42
  fi
done

if [ \"$seen_exec\" != \"1\" ]; then
  echo \"missing exec subcommand\" >&2
  exit 43
fi

if [ \"$seen_resume\" = \"1\" ]; then
  printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-2\",\"type\":\"agent_message\",\"text\":\"resume reply from fake codex\"}}'
else
  printf '%s\\n' '{\"type\":\"thread.started\",\"thread_id\":\"thread-user-1\"}'
  printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-1\",\"type\":\"agent_message\",\"text\":\"launch reply from fake codex\"}}'
fi
",
    )
    .unwrap();
    make_executable(&codex);
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let mut chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(&mut chat, 100, 24);

    app.handle_bytes(b":new spawned process\n");

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("launch reply from fake codex"));

    app.handle_bytes(b"continue\n");

    assert!(app.render_frame().contains("user-1> continue"));
    assert!(app.render_frame().contains("resume reply from fake codex"));
}

#[derive(Debug)]
struct FakeBackend {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
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

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            replies: replies.into_iter().map(String::from).collect(),
            launches: Vec::new(),
            sends: Vec::new(),
        }
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.launches.push(request.clone());
        let mut session = AgentSession::new(request);
        session.push_message(
            MessageRole::Agent,
            self.replies.pop_front().expect("missing fake reply"),
        );
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.sends.push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(
            MessageRole::Agent,
            self.replies.pop_front().expect("missing fake reply"),
        ))
    }
}
