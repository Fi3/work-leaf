use std::collections::VecDeque;
use std::path::PathBuf;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, CommandChat,
    MessageRole, PaneFocus, TerminalApp, UiMode,
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
