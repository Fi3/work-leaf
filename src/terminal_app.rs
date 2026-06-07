use std::thread;
use std::time::{Duration, Instant};

use rustyline::line_buffer::{ChangeListener, DeleteListener, Direction, LineBuffer};

use crate::agent::AgentBackend;
#[cfg(test)]
use crate::agent::AgentId;
use crate::cli::{CommandChat, terminal_right_content, ui_action_text};
use crate::ui::{AgentListEntry, PaneFocus, TerminalUi, UiKey, UiMode};
#[cfg(test)]
use crate::workspace::WorkLeafLoading;
use crate::workspace::{WorkLeafController, WorkLeafEvent, WorkLeafSession};

#[derive(Debug)]
pub struct TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    controller: WorkLeafController<B>,
    ui: TerminalUi,
    prompt_buffer: PromptLine,
    prompt_history: Vec<String>,
    prompt_history_index: Option<usize>,
    prompt_history_draft: Option<String>,
    chat_buffer: PromptLine,
    chat_history: Vec<String>,
    chat_history_index: Option<usize>,
    chat_history_draft: Option<String>,
    escape_sequence: Option<PendingEscapeSequence>,
    paste_mode: bool,
    skip_next_paste_lf: bool,
    spinner: usize,
    dirty: bool,
    quit: bool,
}

impl<B> TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    pub fn new(chat: CommandChat<B>, width: u16, height: u16) -> Self {
        Self {
            controller: WorkLeafController::new(chat),
            ui: TerminalUi::new(width, height),
            prompt_buffer: PromptLine::new(),
            prompt_history: Vec::new(),
            prompt_history_index: None,
            prompt_history_draft: None,
            chat_buffer: PromptLine::new(),
            chat_history: Vec::new(),
            chat_history_index: None,
            chat_history_draft: None,
            escape_sequence: None,
            paste_mode: false,
            skip_next_paste_lf: false,
            spinner: 0,
            dirty: true,
            quit: false,
        }
    }

    pub fn into_chat(mut self) -> CommandChat<B> {
        self.wait_for_idle(Duration::from_secs(5));
        self.controller.into_chat()
    }

    pub fn ui(&self) -> &TerminalUi {
        &self.ui
    }

    pub fn transcript(&self) -> &[String] {
        self.controller.transcript()
    }

    pub fn is_quit(&self) -> bool {
        self.quit
    }

    pub fn is_busy(&mut self) -> bool {
        let busy = self.controller.is_busy();
        self.apply_controller_events();
        busy
    }

    pub fn needs_render(&self) -> bool {
        self.dirty
    }

    pub fn mark_rendered(&mut self) {
        self.dirty = false;
    }

    pub fn tick(&mut self) {
        let busy = self.controller.is_busy();
        self.apply_controller_events();
        if busy {
            self.spinner = (self.spinner + 1) % SPINNER.len();
            self.dirty = true;
        }
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.apply_controller_events();
            if !self.controller.is_busy() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.apply_controller_events();
        !self.controller.is_busy()
    }

    pub fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.apply_controller_events();
            if self.render_frame().contains(needle) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.apply_controller_events();
        self.render_frame().contains(needle)
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        for byte in bytes {
            if !self.handle_byte(*byte) {
                return false;
            }
        }
        self.finish_pending_escape_sequence();
        self.apply_controller_events();
        !self.quit
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.apply_controller_events();
        if self.quit {
            return false;
        }

        if self.continue_escape_sequence(byte) {
            return !self.quit;
        }

        if byte == 27 {
            let defer_escape = self.defer_escape_key();
            self.escape_sequence = Some(PendingEscapeSequence {
                bytes: Vec::new(),
                mode_before: self.ui.mode(),
                escape_dispatched: !defer_escape,
            });
            if !defer_escape {
                self.handle_input(TerminalAppInput::Key(UiKey::Esc));
            }
            return !self.quit;
        }

        let Some(input) = self.input_from_byte(byte) else {
            return true;
        };
        self.handle_input(input);
        self.apply_controller_events();
        !self.quit
    }

    pub fn render_frame(&self) -> String {
        let right_content = self.right_content();
        let right_cursor_column = (self.ui.focus() == PaneFocus::Right
            && self.ui.mode() != UiMode::Prompt)
            .then_some(6 + self.chat_buffer.cursor_char_count());
        self.ui.render_screen_with_cursors(
            &right_content,
            self.prompt_buffer.as_str(),
            self.prompt_buffer.cursor(),
            right_cursor_column,
        )
    }

    pub fn poll_worker(&mut self) {
        self.apply_controller_events();
    }

    fn handle_input(&mut self, input: TerminalAppInput) {
        match input {
            TerminalAppInput::Quit => {
                self.request_quit();
            }
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.backspace();
                self.prompt_history_index = None;
                self.prompt_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.backspace();
                self.chat_history_index = None;
                self.chat_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Prompt => {
                let line = self.prompt_buffer.trimmed_string();
                self.prompt_buffer.clear();
                self.ui.handle_key(UiKey::Esc);
                if !line.is_empty() {
                    self.prompt_history.push(line.clone());
                    self.prompt_history_index = None;
                    self.prompt_history_draft = None;
                    self.handle_command_line(&line);
                } else {
                    self.prompt_history_index = None;
                    self.prompt_history_draft = None;
                }
                self.dirty = true;
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Insert => {
                self.send_chat_buffer();
            }
            TerminalAppInput::LineBreak if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.push('\n');
                self.chat_history_index = None;
                self.chat_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::PasteStart => {
                self.paste_mode = true;
                self.skip_next_paste_lf = false;
            }
            TerminalAppInput::PasteEnd => {
                self.paste_mode = false;
                self.skip_next_paste_lf = false;
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.push(ch);
                self.prompt_history_index = None;
                self.prompt_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.push(ch);
                self.chat_history_index = None;
                self.chat_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Left) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.move_left();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Right) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.move_right();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Up) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(-1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Down) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Left) if self.should_route_chat_arrow() => {
                self.chat_buffer.move_left();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Right) if self.should_route_chat_arrow() => {
                self.chat_buffer.move_right();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Up) if self.should_route_chat_arrow() => {
                self.recall_chat_history(-1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Down) if self.should_route_chat_arrow() => {
                self.recall_chat_history(1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Esc) => {
                self.prompt_buffer.clear();
                let actions = self.ui.handle_key(UiKey::Esc);
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Key(key) => {
                let actions = self.ui.handle_key(key);
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Char(ch) => {
                let actions = self.ui.handle_key(UiKey::Char(ch));
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Backspace | TerminalAppInput::Enter | TerminalAppInput::LineBreak => {
            }
        }
    }

    fn handle_command_line(&mut self, line: &str) {
        self.controller.execute_command_line(line);
        self.apply_controller_events();
    }

    fn send_chat_buffer(&mut self) {
        let message = self.chat_buffer.trimmed_string();
        self.chat_buffer.clear();
        self.chat_history_index = None;
        self.chat_history_draft = None;
        if message.is_empty() {
            self.dirty = true;
            return;
        }

        self.chat_history.push(message.clone());
        if let Some(agent_id) = self.ui.selected_agent().cloned() {
            let _ = self.controller.send_message(&agent_id, &message);
        } else {
            self.controller.send_command_agent_message(&message);
        }
        self.apply_controller_events();
        self.dirty = true;
    }

    #[cfg(test)]
    fn clear_agent_loading(&mut self, agent_id: &AgentId) {
        self.set_agent_loading(agent_id, None);
    }

    #[cfg(test)]
    fn set_agent_loading(&mut self, agent_id: &AgentId, loading: Option<LoadingKind>) {
        let _ = self.ui.set_agent_ready_state(agent_id, loading.is_none());
    }

    fn record_actions(&mut self, actions: Vec<crate::UiAction>) {
        for action in actions {
            self.controller.push_transcript_line(ui_action_text(action));
        }
        self.apply_controller_events();
    }

    fn apply_controller_events(&mut self) {
        let events = self.controller.drain_events();
        if events.is_empty() {
            return;
        }
        for event in events {
            match event {
                WorkLeafEvent::AgentAdded { session } | WorkLeafEvent::AgentUpdated { session } => {
                    self.apply_session_to_ui(&session);
                }
                WorkLeafEvent::AgentSelected { agent_id } => {
                    let _ = self.ui.activate_agent_chat(&agent_id);
                }
                WorkLeafEvent::QuitRequested => {
                    self.quit = true;
                }
                WorkLeafEvent::AgentLineAppended { .. }
                | WorkLeafEvent::CommandTranscriptLine { .. } => {}
            }
        }
        self.dirty = true;
    }

    fn apply_session_to_ui(&mut self, session: &WorkLeafSession) {
        if self
            .ui
            .set_agent_feature(&session.id, session.title.clone())
            .is_err()
        {
            self.ui.add_agent(AgentListEntry::new(
                session.id.clone(),
                session.title.clone(),
            ));
        }
        let _ = self
            .ui
            .set_agent_ready_state(&session.id, session.loading.is_none());
    }

    fn should_route_chat_arrow(&self) -> bool {
        self.ui.mode() == UiMode::Insert
            || (self.ui.mode() == UiMode::Command && self.ui.focus() == PaneFocus::Right)
    }

    fn defer_escape_key(&self) -> bool {
        self.ui.mode() == UiMode::Prompt
            || (self.ui.mode() == UiMode::Insert && self.ui.focus() == PaneFocus::Right)
    }

    fn finish_pending_escape_sequence(&mut self) {
        let should_finish = self
            .escape_sequence
            .as_ref()
            .is_some_and(|sequence| sequence.bytes.is_empty());
        if should_finish {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
        }
    }

    fn dispatch_pending_escape_if_needed(&mut self, sequence: &PendingEscapeSequence) {
        if !sequence.escape_dispatched {
            self.handle_input(TerminalAppInput::Key(UiKey::Esc));
        }
    }

    fn input_from_byte(&mut self, byte: u8) -> Option<TerminalAppInput> {
        if self.paste_mode {
            match byte {
                13 => {
                    self.skip_next_paste_lf = true;
                    return Some(TerminalAppInput::LineBreak);
                }
                10 if self.skip_next_paste_lf => {
                    self.skip_next_paste_lf = false;
                    return None;
                }
                10 => return Some(TerminalAppInput::LineBreak),
                _ => {
                    self.skip_next_paste_lf = false;
                }
            }
        }
        TerminalAppInput::from_byte(byte)
    }

    fn recall_prompt_history(&mut self, delta: isize) {
        if self.prompt_history.is_empty() {
            return;
        }

        if self.prompt_history_index.is_none() {
            self.prompt_history_draft = Some(self.prompt_buffer.as_str().to_string());
        }

        let current = self
            .prompt_history_index
            .unwrap_or(self.prompt_history.len()) as isize;
        let next = current + delta;
        if next < 0 {
            self.prompt_history_index = Some(0);
            self.prompt_buffer.replace(&self.prompt_history[0]);
        } else if next >= self.prompt_history.len() as isize {
            self.prompt_history_index = None;
            let draft = self.prompt_history_draft.take().unwrap_or_default();
            self.prompt_buffer.replace(&draft);
        } else {
            let next = next as usize;
            self.prompt_history_index = Some(next);
            self.prompt_buffer.replace(&self.prompt_history[next]);
        }
    }

    fn recall_chat_history(&mut self, delta: isize) {
        if self.chat_history.is_empty() {
            return;
        }

        if self.chat_history_index.is_none() {
            self.chat_history_draft = Some(self.chat_buffer.as_str().to_string());
        }

        let current = self.chat_history_index.unwrap_or(self.chat_history.len()) as isize;
        let next = current + delta;
        if next < 0 {
            self.chat_history_index = Some(0);
            self.chat_buffer.replace(&self.chat_history[0]);
        } else if next >= self.chat_history.len() as isize {
            self.chat_history_index = None;
            let draft = self.chat_history_draft.take().unwrap_or_default();
            self.chat_buffer.replace(&draft);
        } else {
            let next = next as usize;
            self.chat_history_index = Some(next);
            self.chat_buffer.replace(&self.chat_history[next]);
        }
    }

    fn request_quit(&mut self) {
        self.controller.shutdown();
        self.quit = true;
        self.dirty = true;
    }

    fn right_content(&self) -> String {
        let snapshot = self.controller.snapshot();
        if let Some(agent_id) = self.ui.selected_agent() {
            let session = snapshot.session(agent_id);
            let mut lines = session
                .map(|session| session.lines.clone())
                .unwrap_or_default();
            if let Some(loading) = session.and_then(|session| session.loading) {
                lines.push(format!(
                    "work-leaf: {} {}",
                    self.controller.loading_text(loading),
                    SPINNER[self.spinner]
                ));
            }
            return terminal_right_content(self.chat_buffer.as_str(), &lines);
        }
        terminal_right_content(self.chat_buffer.as_str(), &snapshot.command_transcript)
    }

    fn continue_escape_sequence(&mut self, byte: u8) -> bool {
        let Some(sequence) = self.escape_sequence.as_mut() else {
            return false;
        };

        if sequence.bytes.is_empty() && byte != b'[' {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
            return false;
        }

        sequence.bytes.push(byte);
        if is_complete_control_sequence(&sequence.bytes) {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            if sequence.escape_dispatched
                && sequence.mode_before == UiMode::Insert
                && self.ui.mode() != UiMode::Insert
            {
                let actions = self.ui.handle_key(UiKey::Char('i'));
                self.record_actions(actions);
            }
            if let Some(input) = parse_control_sequence(&sequence.bytes) {
                self.handle_input(input);
            }
        } else if sequence.bytes.len() > MAX_ESCAPE_SEQUENCE {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
        }

        true
    }
}

#[cfg(test)]
type LoadingKind = WorkLeafLoading;

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingEscapeSequence {
    bytes: Vec<u8>,
    mode_before: UiMode,
    escape_dispatched: bool,
}

const SPINNER: [&str; 4] = ["|", "/", "-", "\\"];

const MAX_ESCAPE_SEQUENCE: usize = 64;

fn is_complete_control_sequence(sequence: &[u8]) -> bool {
    sequence.len() > 1
        && sequence
            .last()
            .is_some_and(|byte| (0x40..=0x7e).contains(byte))
}

fn parse_control_sequence(sequence: &[u8]) -> Option<TerminalAppInput> {
    match sequence {
        [b'[', b'A'] => Some(TerminalAppInput::Key(UiKey::Up)),
        [b'[', b'B'] => Some(TerminalAppInput::Key(UiKey::Down)),
        [b'[', b'C'] => Some(TerminalAppInput::Key(UiKey::Right)),
        [b'[', b'D'] => Some(TerminalAppInput::Key(UiKey::Left)),
        [b'[', b'2', b'0', b'0', b'~'] => Some(TerminalAppInput::PasteStart),
        [b'[', b'2', b'0', b'1', b'~'] => Some(TerminalAppInput::PasteEnd),
        [b'[', b'1', b'3', b';', b'2', b'u']
        | [b'[', b'1', b'3', b';', b'2', b'~']
        | [b'[', b'2', b'7', b';', b'2', b';', b'1', b'3', b'~'] => {
            Some(TerminalAppInput::LineBreak)
        }
        _ => parse_sgr_mouse_click(sequence).map(TerminalAppInput::Key),
    }
}

fn parse_sgr_mouse_click(sequence: &[u8]) -> Option<UiKey> {
    let final_byte = *sequence.last()?;
    if !sequence.starts_with(b"[<") || !matches!(final_byte, b'M' | b'm') {
        return None;
    }

    let body = std::str::from_utf8(&sequence[2..sequence.len() - 1]).ok()?;
    let mut parts = body.split(';');
    let button = parts.next()?.parse::<u16>().ok()?;
    let column = parts.next()?.parse::<u16>().ok()?;
    let row = parts.next()?.parse::<u16>().ok()?;
    if parts.next().is_some() || button & 0b11 != 0 {
        return None;
    }

    Some(UiKey::MouseClick { column, row })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalAppInput {
    Key(UiKey),
    Char(char),
    Enter,
    Backspace,
    LineBreak,
    PasteStart,
    PasteEnd,
    Quit,
}

impl TerminalAppInput {
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            3 | 4 => Some(Self::Quit),
            13 | 10 => Some(Self::Enter),
            27 => Some(Self::Key(UiKey::Esc)),
            23 => Some(Self::Key(UiKey::CtrlW)),
            8 | 127 => Some(Self::Backspace),
            byte if byte.is_ascii_graphic() || byte == b' ' => Some(Self::Char(byte as char)),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct PromptLine {
    buffer: LineBuffer,
}

impl PromptLine {
    const CAPACITY: usize = 64 * 1024;

    fn new() -> Self {
        Self {
            buffer: LineBuffer::with_capacity(Self::CAPACITY),
        }
    }

    fn as_str(&self) -> &str {
        self.buffer.as_str()
    }

    fn cursor(&self) -> usize {
        self.buffer.pos()
    }

    fn cursor_char_count(&self) -> usize {
        self.as_str()[..self.cursor()].chars().count()
    }

    fn trimmed_string(&self) -> String {
        self.as_str().trim().to_string()
    }

    fn push(&mut self, ch: char) {
        let mut listener = NoopLineListener;
        let _ = self.buffer.insert(ch, 1, &mut listener);
    }

    fn move_left(&mut self) {
        self.buffer.move_backward(1);
    }

    fn move_right(&mut self) {
        self.buffer.move_forward(1);
    }

    fn backspace(&mut self) {
        let mut listener = NoopLineListener;
        self.buffer.backspace(1, &mut listener);
    }

    fn clear(&mut self) {
        let mut listener = NoopLineListener;
        let len = self.buffer.as_str().len();
        self.buffer.replace(0..len, "", &mut listener);
    }

    fn replace(&mut self, text: &str) {
        let mut listener = NoopLineListener;
        let len = self.buffer.as_str().len();
        self.buffer.replace(0..len, text, &mut listener);
        self.buffer.move_end();
    }
}

#[derive(Debug)]
struct NoopLineListener;

impl DeleteListener for NoopLineListener {
    fn delete(&mut self, _idx: usize, _string: &str, _dir: Direction) {}
}

impl ChangeListener for NoopLineListener {
    fn insert_char(&mut self, _idx: usize, _c: char) {}

    fn insert_str(&mut self, _idx: usize, _string: &str) {}

    fn replace(&mut self, _idx: usize, _old: &str, _new: &str) {}
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::agent::{AgentError, AgentLaunch, AgentSession, ChatMessage, MessageRole};

    #[derive(Clone, Debug)]
    struct NoopBackend;

    impl AgentBackend for NoopBackend {
        fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
            Ok(AgentSession::new(request))
        }

        fn send(&mut self, _agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
            Ok(ChatMessage::new(MessageRole::Agent, prompt))
        }
    }

    #[test]
    fn clearing_agent_loading_marks_chat_ready_in_left_pane() {
        let chat = CommandChat::new(PathBuf::from("."), NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");

        app.ui
            .add_agent(AgentListEntry::new(agent_id.clone(), "feature"));
        app.ui
            .activate_agent_chat(&agent_id)
            .expect("test agent is registered");
        app.set_agent_loading(&agent_id, Some(LoadingKind::WaitingForReply));

        assert!(!app.render_frame().contains('\u{7}'));
        assert!(!app.ui.render_left_pane().contains("READY"));

        app.clear_agent_loading(&agent_id);

        assert!(app.render_frame().starts_with('\u{7}'));
        assert!(!app.render_frame().contains('\u{7}'));
        assert!(
            app.ui
                .render_left_pane()
                .contains("\u{1b}[7m>feature user-1  working: feature  READY\u{1b}[0m")
        );
    }

    #[test]
    fn command_surface_insert_mode_renders_chat_buffer() {
        let chat = CommandChat::new(PathBuf::from("."), NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);

        app.handle_bytes(b"itype in command agent");

        assert!(app.render_frame().contains("type in command agent"));
    }

    #[test]
    fn command_surface_chat_uses_command_agent_to_spawn_codex_agent() {
        let root =
            std::env::temp_dir().join(format!("work-leaf-command-surface-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let chat = CommandChat::new(root, NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);

        assert!(app.ui.selected_agent().is_none());

        app.handle_bytes(b"ispawn a new patch agent that uses codex\n");

        assert!(app.wait_for_idle(Duration::from_secs(1)));
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        assert_eq!(app.ui.selected_agent(), Some(&agent_id));
        assert!(app.transcript().iter().any(|line| line
            == "command-agent: launching Codex user agent for patch agent that uses codex"));
        assert!(
            app.transcript()
                .iter()
                .any(|line| line == "work-leaf> new patch agent that uses codex")
        );
    }
}
