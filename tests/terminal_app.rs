use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, CodexBackend,
    CodexCommandConfig, CommandChat, MessageRole, PaneFocus, PromptPolicy, TerminalApp, UiMode,
};

#[test]
fn terminal_app_new_and_chat_message_use_real_command_chat_backend() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new implement parser\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert_eq!(app.ui().mode(), UiMode::Insert);
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("launch reply"));

    app.handle_bytes(b"please continue\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: please continue"));
    assert!(app.render_frame().contains("follow reply"));
    let backend = app.into_chat().into_backend();
    assert_eq!(backend.launches.len(), 1);
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0.as_str(), "user-1");
    assert_eq!(backend.sends[0].1, "please continue");
}

#[test]
fn terminal_app_keeps_visible_cursor_on_chat_input() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new implement parser\nhello");

    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert!(app.render_frame().ends_with("\u{1b}[3;33H"));
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
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new spawned process\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("launch reply from fake codex"));

    app.handle_bytes(b"continue\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: continue"));
    assert!(app.render_frame().contains("resume reply from fake codex"));
}

#[test]
fn terminal_app_does_not_clear_screen_on_each_render_or_drop_fast_input() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new fast input\nabcdef");
    app.wait_for_idle(Duration::from_secs(1));

    let frame = app.render_frame();
    assert!(!frame.contains("\u{1b}[2J"));
    assert!(frame.contains("chat> abcdef"));
}

#[test]
fn terminal_app_new_adds_agent_immediately_while_backend_is_loading() {
    let backend = SlowBackend;
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);
    let start = Instant::now();

    app.handle_bytes(b":new slow launch\n");

    assert!(start.elapsed() < Duration::from_millis(100));
    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("Starting Codex session"));

    app.wait_for_idle(Duration::from_secs(2));
    assert!(app.render_frame().contains("slow launch reply"));
}

#[test]
fn terminal_app_chat_pane_shows_only_the_selected_agent_session() {
    let backend = FakeBackend::new([
        "first launch",
        "second launch",
        "second reply",
        "first reply",
    ]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:new second\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"message for second\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("message for second"));
    app.handle_bytes(&[27, 23, b'h', b'k', b'l']);

    let frame = app.render_frame();
    assert!(frame.contains("first launch"));
    assert!(!frame.contains("message for second"));
    assert!(!frame.contains("second reply"));

    app.handle_bytes(b"imessage for first\n");
    app.wait_for_idle(Duration::from_secs(1));
    let frame = app.render_frame();
    assert!(frame.contains("message for first"));
    assert!(frame.contains("first reply"));
    assert!(!frame.contains("message for second"));
}

#[test]
fn terminal_app_sgr_mouse_click_on_left_agent_row_selects_that_chat() {
    let backend = FakeBackend::new(["first launch", "second launch"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:new second\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-2")
    );

    app.handle_bytes(b"\x1b[<0;4;3M");

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("first launch"));
    assert!(!app.render_frame().contains("second launch"));
}

#[test]
fn terminal_app_streams_spawned_codex_output_before_process_finishes() {
    let root = temp_dir("terminal-app-codex-streaming");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    fs::write(
        &codex,
        "\
#!/bin/sh
printf '%s\\n' '{\"type\":\"thread.started\",\"thread_id\":\"thread-stream\"}'
printf '%s\\n' '{\"type\":\"error\",\"message\":\"streamed progress before final\"}'
sleep 1
printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-1\",\"type\":\"agent_message\",\"text\":\"streamed final reply\"}}'
",
    )
    .unwrap();
    make_executable(&codex);
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new stream output\n");

    assert!(app.wait_for_frame_contains("streamed progress before final", Duration::from_secs(1)));
    assert!(app.is_busy());
    app.wait_for_idle(Duration::from_secs(2));
    assert!(app.render_frame().contains("streamed final reply"));
}

#[test]
fn terminal_app_answers_agent_file_requests_even_when_one_requested_path_is_missing() {
    let root = temp_dir("terminal-app-missing-agent-read");
    fs::write(root.join("Readme.md"), "work-leaf readme\n").unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname = \"work-leaf\"\n").unwrap();
    let backend = FakeBackend::new([
        "@work-leaf read README.md Cargo.toml",
        "I can answer after receiving the available file text",
    ]);
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new explain repo\n");
    app.wait_for_idle(Duration::from_secs(1));

    let frame = app.render_frame();
    assert!(frame.contains("reported unavailable file text to user-1"));
    assert!(frame.contains("sent file text to user-1: Cargo.toml"));
    assert!(frame.contains("I can answer after receiving the available file text"));

    let backend = app.into_chat().into_backend();
    assert_eq!(backend.sends.len(), 1);
    assert!(backend.sends[0].1.contains("Cargo.toml"));
    assert!(backend.sends[0].1.contains("name = \"work-leaf\""));
    assert!(backend.sends[0].1.contains("Unavailable file text"));
    assert!(backend.sends[0].1.contains("README.md"));
}

#[test]
fn terminal_app_streams_automatic_agent_follow_up_output() {
    let root = temp_dir("terminal-app-streaming-agent-read");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    let chat = CommandChat::new(root, StreamingDirectiveBackend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new inspect source\n");

    assert!(app.wait_for_frame_contains(
        "streamed answer from directive follow-up",
        Duration::from_secs(1)
    ));
    assert!(app.is_busy());
    app.wait_for_idle(Duration::from_secs(2));
    assert!(
        app.render_frame()
            .contains("final answer from directive follow-up")
    );
}

#[derive(Debug)]
struct FakeBackend {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
}

#[derive(Debug)]
struct SlowBackend;

#[derive(Debug)]
struct StreamingDirectiveBackend;

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

impl AgentBackend for SlowBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        thread::sleep(Duration::from_millis(250));
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "slow launch reply");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        thread::sleep(Duration::from_millis(250));
        Ok(ChatMessage::new(MessageRole::Agent, "slow send reply"))
    }
}

impl AgentBackend for StreamingDirectiveBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "@work-leaf read src/lib.rs");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(
            MessageRole::Agent,
            "final answer from directive follow-up",
        ))
    }

    fn send_streaming(
        &mut self,
        _agent_id: &AgentId,
        _prompt: &str,
        sink: &mut dyn FnMut(work_leaf::AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        sink(work_leaf::AgentStreamEvent::AgentMessage(
            "streamed answer from directive follow-up".to_string(),
        ));
        thread::sleep(Duration::from_millis(250));
        Ok(ChatMessage::new(
            MessageRole::Agent,
            "final answer from directive follow-up",
        ))
    }
}
