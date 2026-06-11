use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, CodexBackend,
    CodexCommandConfig, CommandChat, MessageRole, PaneFocus, PromptPolicy, TerminalApp, UiMode,
};

mod temp_cleanup;

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
    let launches = backend.launches();
    let sends = backend.sends();
    assert_eq!(launches.len(), 1);
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].0.as_str(), "user-1");
    assert_eq!(sends[0].1, "please continue");
}

#[test]
fn terminal_app_slash_command_from_chat_view_sends_to_selected_agent() {
    let backend = FakeBackend::new(["launch reply", "backend status output"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new status command\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b/status\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(app.ui().mode(), UiMode::Insert);
    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert!(app.render_frame().contains("user: /status"));
    assert!(app.render_frame().contains("backend status output"));

    let backend = app.into_chat().into_backend();
    assert_eq!(
        backend.sends(),
        vec![(AgentId::new("user-1").unwrap(), "/status".to_string())]
    );
}

#[test]
fn terminal_app_slash_command_from_colon_prompt_sends_to_selected_agent() {
    let backend = FakeBackend::new(["launch reply", "backend colon status output"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new status command\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:/status\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: /status"));
    assert!(app.render_frame().contains("backend colon status output"));

    let backend = app.into_chat().into_backend();
    assert_eq!(
        backend.sends(),
        vec![(AgentId::new("user-1").unwrap(), "/status".to_string())]
    );
}

#[test]
fn terminal_app_codex_status_slash_command_resumes_backend_session() {
    let root = temp_dir("terminal-app-codex-slash-command");
    let (codex, python) = write_fake_sdk_sidecar(
        &root,
        r#"#!/bin/sh
dir=$(dirname "$0")
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"agent_id":"title-'*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-title","reply":"status"}\n' "$id"
      ;;
    *'"op":"command"'*'"/status"'*)
      printf '%s\n' "$line" >> "$dir/command.log"
      printf '{"id":%s,"ok":true,"thread_id":"thread-slash-command","reply":"backend status from fake sdk"}\n' "$id"
      ;;
    *'"op":"launch"'*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-slash-command","reply":"launch reply from fake sdk"}\n' "$id"
      ;;
    *'"op":"shutdown"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      exit 0
      ;;
    *)
      printf '{"id":%s,"ok":true,"thread_id":"thread-slash-command","reply":"unexpected sdk request"}\n' "$id"
      ;;
  esac
done
"#,
    );
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary(&codex)
            .with_sdk_python(&python),
        PromptPolicy::for_restricted_agents(),
    );
    let chat = CommandChat::new(root.clone(), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new status command\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    app.handle_bytes(b"\x1b/status\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    let frame = app.render_frame();
    assert!(frame.contains("user: /status"));
    assert!(frame.contains("backend status from fake sdk"), "{frame}");
    let command_log = fs::read_to_string(root.join("bin").join("command.log")).unwrap();
    assert!(command_log.contains(r#""op":"command""#), "{command_log}");
    assert!(
        command_log.contains(r#""prompt":"/status""#),
        "{command_log}"
    );
}

#[test]
fn terminal_app_does_not_run_project_required_checks_outside_agent() {
    let root = temp_dir("terminal-app-no-required-check-run");
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\n- `./check.sh`\n",
    )
    .unwrap();
    fs::write(
        root.join("check.sh"),
        "#!/bin/sh\necho compile failed\nexit 1\n",
    )
    .unwrap();
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new break compile\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    let frame = app.render_frame();
    assert!(frame.contains("launch reply"));
    assert!(frame.contains("READY"));
    assert!(!frame.contains("required check failed"));
    assert!(!frame.contains("compile failed"));
}

#[test]
fn terminal_app_command_agent_opens_multiple_patch_agents_from_chat_request() {
    let backend = FakeBackend::new([
        "launch reply",
        "launch reply",
        "launch reply",
        "launch reply",
    ]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b"iopen 4 pacth agents\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-4")
    );
    let frame = app.render_frame();
    assert!(frame.contains("user-4"));
    assert!(frame.contains("launch reply"));

    let backend = app.into_chat().into_backend();
    let launches = backend.launches();
    assert_eq!(launches.len(), 4);
    assert!(launches.iter().all(|launch| launch.prompt == "patch"));
}

#[test]
fn terminal_app_command_mode_f_forks_selected_agent_chat() {
    let backend = FakeBackend::new(["source launch", "fork launch"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new original context\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));
    app.handle_byte(27);

    assert_eq!(app.ui().mode(), UiMode::Command);
    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );

    app.handle_byte(b'f');
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-2")
    );
    let frame = app.render_frame();
    assert!(frame.contains("source launch"));
    assert!(frame.contains("work-leaf: forked from user-1"));
    assert!(frame.contains("fork launch"));

    let backend = app.into_chat().into_backend();
    let launches = backend.launches();
    assert_eq!(launches.len(), 2);
    assert!(launches[1].prompt.contains("original context"));
    assert!(launches[1].prompt.contains("source launch"));
}

#[test]
fn terminal_app_ctrl_c_never_quits_and_only_right_focus_interrupts_agent() {
    let backend = InterruptRecordingBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new interruptible task\n");
    assert!(app.is_busy());
    assert_eq!(app.ui().focus(), PaneFocus::Right);

    assert!(app.handle_byte(3));
    assert!(!app.is_quit());
    assert_eq!(backend.interrupts(), vec![AgentId::new("user-1").unwrap()]);
    assert!(
        app.render_frame()
            .contains("work-leaf: sent Ctrl-C to Codex")
    );

    app.handle_bytes(&[27, 23, b'h']);
    assert_eq!(app.ui().focus(), PaneFocus::Left);
    assert!(app.handle_byte(3));
    assert!(!app.is_quit());
    assert_eq!(backend.interrupts(), vec![AgentId::new("user-1").unwrap()]);

    assert!(app.wait_for_idle(Duration::from_secs(1)));
}

#[test]
fn terminal_app_quits_only_from_prompt_q() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    assert!(app.handle_byte(3));
    assert!(!app.is_quit());
    assert!(app.handle_byte(b'q'));
    assert!(!app.is_quit());

    assert!(!app.handle_bytes(b":q\n"));
    assert!(app.is_quit());
}

#[test]
fn terminal_app_command_mode_typing_shows_insert_mode_notice() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b"hello");

    assert_eq!(app.ui().mode(), UiMode::Command);
    assert!(
        app.render_frame()
            .contains("command mode: press i for insert mode before typing")
    );
}

#[test]
fn terminal_app_ctrl_c_shows_quit_notice() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    assert!(app.handle_byte(3));
    assert!(!app.is_quit());
    assert!(
        app.render_frame()
            .contains("to exit, press Esc then :q then Enter")
    );
}

#[derive(Clone, Debug, Default)]
struct InterruptRecordingBackend {
    state: Arc<Mutex<InterruptRecordingState>>,
}

#[derive(Debug, Default)]
struct InterruptRecordingState {
    interrupts: Vec<AgentId>,
}

impl InterruptRecordingBackend {
    fn interrupts(&self) -> Vec<AgentId> {
        self.state.lock().unwrap().interrupts.clone()
    }
}

impl AgentBackend for InterruptRecordingBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            session.push_message(MessageRole::Agent, "interruptible-task");
            return Ok(session);
        }

        thread::sleep(Duration::from_millis(250));
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "finished without interrupt");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "follow-up"))
    }

    fn interrupt(&mut self, agent_id: &AgentId) -> Result<(), AgentError> {
        self.state.lock().unwrap().interrupts.push(agent_id.clone());
        Ok(())
    }
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
fn terminal_app_command_prompt_arrows_keep_visible_cursor_at_edit_position() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 20, 10);

    app.handle_bytes(b":abcdefghijklmnopqrstuvwxyz0123\x1b[D\x1b[D\x1b[D\x1b[D\x1b[DX");

    assert_eq!(app.ui().mode(), UiMode::Prompt);
    let frame = app.render_frame();
    assert!(frame.contains(":ijklmnopqrstuvwxyXz"));
    assert!(frame.ends_with("\u{1b}[10;20H"));
}

#[test]
fn terminal_app_prompt_history_down_restores_in_progress_command() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 80, 24);

    app.handle_bytes(b":help\n");
    app.handle_bytes(b":draft command\x1b[A");

    assert_eq!(app.ui().mode(), UiMode::Prompt);
    assert!(app.render_frame().contains(":help"));

    app.handle_bytes(b"\x1b[B");

    assert_eq!(app.ui().mode(), UiMode::Prompt);
    assert!(app.render_frame().contains(":draft command"));
}

#[test]
fn terminal_app_chat_arrows_keep_visible_cursor_at_edit_position() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 80, 24);

    app.handle_bytes(b":new cursor render\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"abcdefghijklmnopqrstuvwxyz0123\x1b[D\x1b[D\x1b[D\x1b[D\x1b[DX");

    let frame = app.render_frame();
    assert!(frame.contains("chat> abcdefghijklmnopqrstuvwxyXz0123"));
    assert!(frame.ends_with("\u{1b}[3;50H"));
}

#[test]
fn terminal_app_chat_history_down_restores_in_progress_message() {
    let backend = FakeBackend::new(["launch reply", "first reply", "draft reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new chat history\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"draft message\x1b[A");
    assert!(app.render_frame().contains("chat> first"));

    app.handle_bytes(b"\x1b[B\n");
    app.wait_for_idle(Duration::from_secs(1));

    let backend = app.into_chat().into_backend();
    let sends = backend.sends();
    assert_eq!(
        sends
            .iter()
            .filter(|(_, prompt)| prompt == "draft message")
            .count(),
        1
    );
}

#[test]
fn terminal_app_bytewise_arrow_prefix_keeps_focused_chat_in_insert_mode() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new arrow editing\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"ab");
    app.handle_byte(27);

    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert_eq!(app.ui().mode(), UiMode::Insert);
    assert!(app.render_frame().contains("mode=insert focus=right"));

    app.handle_byte(b'[');
    assert_eq!(app.ui().mode(), UiMode::Insert);

    app.handle_byte(b'D');
    app.handle_byte(b'Z');
    app.handle_byte(b'\n');
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: aZb"));
}

#[test]
fn terminal_app_chat_focus_arrows_edit_buffer_while_command_mode_is_active() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new command arrow editing\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"ab\x1b");

    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert_eq!(app.ui().mode(), UiMode::Command);

    app.handle_bytes(b"\x1b[DiZ\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: aZb"));
}

#[test]
fn terminal_app_chat_focus_arrows_recall_history_while_command_mode_is_active() {
    let backend = FakeBackend::new([
        "launch reply",
        "first reply",
        "second reply",
        "repeat reply",
    ]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new command history\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"second\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b");

    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert_eq!(app.ui().mode(), UiMode::Command);

    app.handle_bytes(b"\x1b[Ai\n");
    app.wait_for_idle(Duration::from_secs(1));

    let backend = app.into_chat().into_backend();
    let second_sends = backend
        .sends()
        .iter()
        .filter(|(_, prompt)| prompt == "second")
        .count();
    assert_eq!(second_sends, 2);
}

#[test]
fn terminal_app_new_and_chat_work_through_spawned_codex_backend() {
    let root = temp_dir("terminal-app-codex-backend");
    let (codex, python) = write_fake_sdk_sidecar(
        &root,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"agent_id":"title-'*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-title","reply":"spawned-process"}\n' "$id"
      ;;
    *'"op":"send"'*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-user-1","reply":"resume reply from fake sdk"}\n' "$id"
      ;;
    *'"op":"launch"'*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-user-1","reply":"launch reply from fake sdk"}\n' "$id"
      ;;
    *'"op":"shutdown"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      exit 0
      ;;
  esac
done
"#,
    );
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary(&codex)
            .with_sdk_python(&python),
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
    assert!(app.render_frame().contains("launch reply from fake sdk"));

    app.handle_bytes(b"continue\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: continue"));
    assert!(app.render_frame().contains("resume reply from fake sdk"));
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
fn terminal_app_keeps_chat_prompt_visible_after_large_agent_output() {
    let long_reply = (0..48)
        .map(|index| format!("agent-output-line-{index:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let backend = FakeBackend::from_replies(VecDeque::from([long_reply]));
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 80, 12);

    app.handle_bytes(b":new large output\n");
    app.wait_for_idle(Duration::from_secs(1));

    let frame = app.render_frame();
    assert!(frame.contains("agent-output-line-47"));
    assert!(frame.contains("chat> "));

    app.handle_bytes(b"next question");
    assert!(app.render_frame().contains("chat> next question"));
}

#[test]
fn terminal_app_mouse_wheel_scrolls_chat_history() {
    let long_reply = (0..48)
        .map(|index| format!("agent-output-line-{index:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let backend = FakeBackend::from_replies(VecDeque::from([long_reply]));
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 80, 12);

    app.handle_bytes(b":new large scroll\n");
    app.wait_for_idle(Duration::from_secs(1));

    let bottom_frame = app.render_frame();
    assert!(bottom_frame.contains("agent-output-line-47"));
    assert!(!bottom_frame.contains("agent-output-line-00"));
    assert!(bottom_frame.contains("chat> "));

    for _ in 0..16 {
        app.handle_bytes(b"\x1b[<64;20;3M");
    }

    let scrolled_frame = app.render_frame();
    assert!(scrolled_frame.contains("agent-output-line-00"));
    assert!(scrolled_frame.contains("chat> "));

    for _ in 0..16 {
        app.handle_bytes(b"\x1b[<65;20;3M");
    }

    let bottom_again = app.render_frame();
    assert!(bottom_again.contains("agent-output-line-47"));
    assert!(!bottom_again.contains("agent-output-line-00"));
    assert!(bottom_again.contains("chat> "));
}

#[test]
fn terminal_app_visual_mode_yanks_right_pane_without_resuming_backend() {
    let backend = FakeBackend::new(["launch reply", "backend should not receive visual yank"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new copy from chat\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));
    app.handle_bytes(&[27]);
    assert_eq!(app.ui().mode(), UiMode::Command);
    assert_eq!(app.ui().focus(), PaneFocus::Right);

    app.handle_byte(b'V');
    assert!(app.ui().visual_selection_active());
    assert!(app.render_frame().contains("mode=visual-line focus=right"));
    app.handle_byte(b'Y');

    assert_eq!(app.ui().copied_text(), Some("launch reply"));
    assert!(
        app.render_frame()
            .starts_with("\u{1b}]52;c;bGF1bmNoIHJlcGx5\u{7}\u{1b}[H")
    );
    let backend = app.into_chat().into_backend();
    assert!(backend.sends().is_empty());
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
fn terminal_app_left_pane_keyboard_selection_switches_the_visible_chat() {
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

    app.handle_bytes(&[27, 23, b'h', b'k']);

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("first launch"));
    assert!(!app.render_frame().contains("second launch"));

    app.handle_bytes(b"j");

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-2")
    );
    assert!(app.render_frame().contains("second launch"));
    assert!(!app.render_frame().contains("first launch"));
}

#[test]
fn terminal_app_comma_collapses_left_pane_and_keeps_selected_chat_visible() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new keep chat visible\n");
    app.wait_for_idle(Duration::from_secs(1));

    app.handle_bytes(b"\x1b,");

    let layout = app.ui().layout();
    assert_eq!(layout.left_width, 0);
    assert_eq!(layout.right_width, 100);
    assert!(app.render_frame().contains("launch reply"));
    assert!(app.render_frame().contains("chat> "));

    app.handle_bytes(b",");

    let layout = app.ui().layout();
    assert_eq!(layout.left_width, 20);
    assert_eq!(layout.right_width, 80);
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("launch reply"));
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
fn terminal_app_sgr_mouse_release_on_left_agent_row_selects_that_chat() {
    let backend = FakeBackend::new(["first launch", "second launch"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:new second\n");
    app.wait_for_idle(Duration::from_secs(1));

    app.handle_bytes(b"\x1b[<0;4;3m");

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
    let (codex, python) = write_fake_sdk_sidecar(
        &root,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"agent_id":"title-'*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-title","reply":"stream-output"}\n' "$id"
      ;;
    *'"op":"launch"'*)
      printf '{"id":%s,"event":{"type":"status","text":"streamed progress before final"}}\n' "$id"
      sleep 1
      printf '{"id":%s,"ok":true,"thread_id":"thread-stream","reply":"streamed final reply"}\n' "$id"
      ;;
    *'"op":"shutdown"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      exit 0
      ;;
  esac
done
"#,
    );
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary(&codex)
            .with_sdk_python(&python),
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
fn terminal_app_spawned_codex_processes_directive_message_before_later_prose_message() {
    let root = temp_dir("terminal-app-codex-directive-before-prose");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/ui.rs"), "pub fn ui() {}\n").unwrap();
    fs::write(root.join("src/ui_harness.rs"), "pub fn harness() {}\n").unwrap();
    let (codex, python) = write_fake_sdk_sidecar(
        &root,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"agent_id":"title-'*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-title","reply":"patch-arrow-keys"}\n' "$id"
      ;;
    *'"op":"send"'*"work-leaf file text"*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-directive-prose","reply":"I can patch after receiving src/ui.rs and src/ui_harness.rs."}\n' "$id"
      ;;
    *'"op":"send"'*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-directive-prose","reply":"missing orchestrator file text"}\n' "$id"
      ;;
    *'"op":"launch"'*)
      printf '{"id":%s,"event":{"type":"message","text":"@work-leaf read src/ui.rs src/ui_harness.rs\\nI have requested the relevant UI and harness files from the orchestrator."}}\n' "$id"
      sleep 0.1
      printf '{"id":%s,"ok":true,"thread_id":"thread-directive-prose","reply":"@work-leaf read src/ui.rs src/ui_harness.rs\\nI have requested the relevant UI and harness files from the orchestrator."}\n' "$id"
      ;;
    *'"op":"interrupt"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      ;;
    *'"op":"shutdown"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      exit 0
      ;;
  esac
done
"#,
    );
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary(&codex)
            .with_sdk_python(&python),
        PromptPolicy::for_restricted_agents(),
    );
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new patch arrow keys\n");
    app.wait_for_idle(Duration::from_secs(1));

    let frame = app.render_frame();
    assert!(frame.contains("I have requested the relevant UI and harness files"));
    assert!(frame.contains("sent file text to user-1: src/ui.rs, src/ui_harness.rs"));
    assert!(frame.contains("I can patch after receiving src/ui.rs and src/ui_harness.rs"));
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
    let sends = backend.sends();
    assert_eq!(sends.len(), 1);
    assert!(sends[0].1.contains("Cargo.toml"));
    assert!(sends[0].1.contains("name = \"work-leaf\""));
    assert!(sends[0].1.contains("Unavailable file text"));
    assert!(sends[0].1.contains("README.md"));
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

#[test]
fn terminal_app_sends_to_one_chat_while_another_chat_is_waiting_for_codex() {
    let backend = ConcurrentChatBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new first parser task\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:new second docs task\n");
    app.wait_for_idle(Duration::from_secs(1));

    app.handle_bytes(b"slow question\n");
    assert!(app.is_busy());

    app.handle_bytes(&[27, 23, b'h', b'k', b'l']);
    app.handle_bytes(b"iquick question\n");

    assert!(app.wait_for_frame_contains("quick reply for user-1", Duration::from_millis(150)));
    assert!(
        !app.render_frame()
            .contains("work-leaf: Codex is still working")
    );
}

#[test]
fn terminal_app_review_adds_reviewer_chat_and_streams_output_immediately() {
    let root = git_repo("terminal-app-review-streams");
    fs::write(
        root.join("README.md"),
        "Agent-ID: user-1\nFeature: parser\nReason: parse configs\nContext: parser work",
    )
    .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["add", "README.md"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args([
            "commit",
            "-m",
            "UPDATE apply parser patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: parser\nReason: parse configs\nContext: parser work",
        ])
        .status()
        .unwrap();

    let chat = CommandChat::new(root, ReviewStreamingBackend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":review\n");

    assert!(app.render_frame().contains("review-user-1"));
    assert!(app.wait_for_frame_contains(
        "reviewer streamed before finishing",
        Duration::from_millis(150)
    ));
}

#[test]
fn terminal_app_marks_reviewed_patch_agent_done_and_closed_visibly() {
    let root = git_repo("terminal-app-review-feature-done");
    fs::write(root.join("README.md"), "before\n").unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["add", "README.md"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "ADD initial readme fixture"])
        .status()
        .unwrap();
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after",
        "NO_FINDINGS",
        "backend status output",
        "follow reply",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new update readme\n");
    assert!(app.wait_for_idle(Duration::from_secs(2)));

    let left_pane = app.ui().render_left_pane();
    assert!(left_pane.contains("DONE?"));
    assert!(left_pane.contains("READY"));
    app.handle_bytes(&[27, 23, b'h', b'l']);
    let frame = app.render_frame();
    assert!(frame.contains("work-leaf: is this feature done? [yes/no]"));
    let sends_after_review = backend.sends().len();

    app.handle_bytes(b"i/status\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));
    let frame = app.render_frame();
    assert!(frame.contains("user: /status"));
    assert!(frame.contains("backend status output"));
    assert!(app.ui().render_left_pane().contains("DONE?"));
    assert_eq!(backend.sends().len(), sends_after_review + 1);

    app.handle_bytes(b"maybe\n");
    assert!(app.render_frame().contains("work-leaf: answer yes or no"));
    assert_eq!(backend.sends().len(), sends_after_review + 1);

    app.handle_bytes(b"yes\n");
    let frame = app.render_frame();
    assert!(frame.contains("work-leaf: feature marked closed"));
    assert!(app.ui().render_left_pane().contains("CLOSED"));
    assert_eq!(backend.sends().len(), sends_after_review + 1);

    app.handle_bytes(b"add another tweak\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));
    let frame = app.render_frame();
    assert!(frame.contains("user: add another tweak"));
    assert!(frame.contains("follow reply"));
    assert!(!app.ui().render_left_pane().contains("CLOSED"));
    assert!(
        backend.sends().iter().any(|(target, prompt)| {
            target.as_str() == "user-1" && prompt == "add another tweak"
        })
    );
}

#[test]
fn terminal_app_delays_dependent_new_until_dependency_closes() {
    let root = git_repo("terminal-app-dependent-new");
    fs::write(root.join("README.md"), "before\n").unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["add", "README.md"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "ADD initial readme fixture"])
        .status()
        .unwrap();
    let backend = FakeBackend::new([
        "parent patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after parent\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after parent",
        "NO_FINDINGS",
        "child launch reply",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new update readme\n");
    assert!(app.wait_for_idle(Duration::from_secs(2)));
    app.handle_bytes(b"\x1b:new --depends-on user-1 update follow-up\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    let left_pane = app.ui().render_left_pane();
    assert!(left_pane.contains("depends-on: user-1"), "{left_pane}");
    assert!(left_pane.contains("depended-on-by: user-2"), "{left_pane}");
    let frame = app.render_frame();
    assert!(
        frame.contains("work-leaf: waiting for user-1 to be marked done"),
        "{frame}"
    );
    assert!(
        frame.contains("work-leaf: Waiting for dependency"),
        "{frame}"
    );
    assert!(
        !backend
            .launches()
            .iter()
            .any(|launch| launch.id.as_str() == "user-2"),
        "dependent launch must not reach the backend before the parent closes"
    );

    app.handle_bytes(&[27, 23, b'h', b'k', b'l']);
    app.handle_bytes(b"iyes\n");
    assert!(app.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    assert!(
        launches
            .iter()
            .any(|launch| launch.id.as_str() == "user-2" && launch.prompt == "update follow-up"),
        "{launches:?}"
    );
    let frame = app.render_frame();
    assert!(
        frame.contains("work-leaf: feature marked closed"),
        "{frame}"
    );
    assert!(app.ui().render_left_pane().contains("CLOSED"));
}

#[test]
fn terminal_app_names_user_session_from_first_prompt_immediately() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new implement parser combinator for config files\n");

    assert!(app.wait_for_idle(Duration::from_secs(1)));
    assert!(app.ui().render_left_pane().contains("parser-combinator"));
    let frame = app.render_frame();
    assert!(!frame.contains("working: user-agent"));
}

#[test]
fn terminal_app_names_interactive_chat_from_first_inserted_prompt_immediately() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new\n");
    app.wait_for_idle(Duration::from_secs(1));
    let initial_left_pane = app.ui().render_left_pane();
    assert!(initial_left_pane.contains("user-1"));
    assert!(!initial_left_pane.contains("oauth redirect handler"));

    app.handle_bytes(b"please fix the OAuth redirect handler\n");

    assert!(app.wait_for_idle(Duration::from_secs(1)));
    let named_left_pane = app.ui().render_left_pane();
    assert!(named_left_pane.contains(">please-fix-the-oauth-redirect-handler"));
    assert!(!named_left_pane.contains("oauth redirect handler"));
    assert!(!named_left_pane.contains("working: user-agent"));

    app.wait_for_idle(Duration::from_secs(1));
    let backend = app.into_chat().into_backend();
    assert_eq!(
        backend.sends()[0].1,
        "please fix the OAuth redirect handler"
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
    sessions: BTreeMap<AgentId, AgentSession>,
}

#[derive(Clone, Debug)]
struct SlowBackend;

#[derive(Clone, Debug)]
struct StreamingDirectiveBackend;

#[derive(Clone, Debug, Default)]
struct ConcurrentChatBackend {
    state: Arc<Mutex<ConcurrentChatState>>,
}

#[derive(Debug, Default)]
struct ConcurrentChatState {
    launches: usize,
}

#[derive(Clone, Debug)]
struct ReviewStreamingBackend;

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
    root
}

fn write_fake_sdk_sidecar(root: &Path, script: &str) -> (PathBuf, PathBuf) {
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let codex = bin.join("codex");
    fs::write(
        &codex,
        "#!/bin/sh\necho unexpected direct codex invocation >&2\nexit 97\n",
    )
    .unwrap();
    make_executable(&codex);
    let python = bin.join("python");
    fs::write(&python, script).unwrap();
    make_executable(&python);
    (codex, python)
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

fn fake_title_from_title_prompt(prompt: &str) -> String {
    let first_prompt = prompt
        .rsplit_once("First prompt:\n")
        .map(|(_, first_prompt)| first_prompt)
        .unwrap_or(prompt);
    if first_prompt.contains("parser combinator") {
        "parser-combinator".to_string()
    } else if first_prompt.contains("OAuth redirect handler") {
        "oauth-redirect-handler".to_string()
    } else if first_prompt.contains("patch") {
        "patch".to_string()
    } else if first_prompt.contains("cursor render") {
        "cursor-render".to_string()
    } else if first_prompt.contains("chat history") {
        "chat-history".to_string()
    } else {
        "chat-title".to_string()
    }
}

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self::from_replies(replies.into_iter().map(String::from).collect())
    }

    fn from_replies(replies: VecDeque<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeBackendState {
                replies,
                launches: Vec::new(),
                sends: Vec::new(),
                sessions: BTreeMap::new(),
            })),
        }
    }

    fn launches(&self) -> Vec<AgentLaunch> {
        self.state.lock().unwrap().launches.clone()
    }

    fn sends(&self) -> Vec<(AgentId, String)> {
        self.state.lock().unwrap().sends.clone()
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

        let mut state = self.state.lock().unwrap();
        state.launches.push(request.clone());
        let agent_id = request.id.clone();
        let mut session = AgentSession::new(request);
        session.push_message(
            MessageRole::Agent,
            state.replies.pop_front().expect("missing fake reply"),
        );
        state.sessions.insert(agent_id, session.clone());
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let mut state = self.state.lock().unwrap();
        state.sends.push((agent_id.clone(), prompt.to_string()));
        let reply = state.replies.pop_front().expect("missing fake reply");
        if let Some(session) = state.sessions.get_mut(agent_id) {
            session.push_message(MessageRole::User, prompt);
            session.push_message(MessageRole::Agent, reply.clone());
        }
        Ok(ChatMessage::new(MessageRole::Agent, reply))
    }

    fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        self.state.lock().unwrap().sessions.get(agent_id).cloned()
    }
}

impl AgentBackend for SlowBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

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
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

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

impl AgentBackend for ConcurrentChatBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

        self.state.lock().unwrap().launches += 1;
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str() == "user-2" {
            thread::sleep(Duration::from_millis(350));
            return Ok(ChatMessage::new(
                MessageRole::Agent,
                "slow reply for user-2",
            ));
        }
        Ok(ChatMessage::new(
            MessageRole::Agent,
            "quick reply for user-1",
        ))
    }
}

impl AgentBackend for ReviewStreamingBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "NO_FINDINGS");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "summary"))
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(work_leaf::AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        sink(work_leaf::AgentStreamEvent::AgentMessage(
            "reviewer streamed before finishing".to_string(),
        ));
        thread::sleep(Duration::from_millis(350));
        self.launch(request)
    }
}

fn git_repo(name: &str) -> PathBuf {
    let root = temp_dir(name);
    Command::new("git")
        .current_dir(&root)
        .args(["init", "-q"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["config", "user.email", "test@example.com"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["config", "user.name", "Test User"])
        .status()
        .unwrap();
    root
}
