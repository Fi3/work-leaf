use std::thread;
use std::time::{Duration, Instant};

use rustyline::line_buffer::{ChangeListener, DeleteListener, Direction, LineBuffer};

use crate::agent::{AgentBackend, AgentId};
use crate::cli::{CommandChat, terminal_right_content, ui_action_text};
use crate::http_controller::HttpControllerClient;
use crate::ui::{AgentListEntry, PaneFocus, TerminalUi, UiKey, UiMode};
use crate::workspace::{
    WorkLeafCompletion, WorkLeafController, WorkLeafEvent, WorkLeafLoading, WorkLeafSession,
    WorkLeafSnapshot,
};

#[derive(Debug)]
pub struct TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    inner: TerminalAppCore<LocalTerminalController<B>>,
}

impl<B> TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    pub fn new(chat: CommandChat<B>, width: u16, height: u16) -> Self {
        Self {
            inner: TerminalAppCore::new(
                LocalTerminalController {
                    controller: WorkLeafController::new(chat),
                },
                width,
                height,
            ),
        }
    }

    pub fn into_chat(mut self) -> CommandChat<B> {
        self.wait_for_idle(Duration::from_secs(5));
        self.inner.controller.controller.into_chat()
    }

    pub fn ui(&self) -> &TerminalUi {
        self.inner.ui()
    }

    pub fn transcript(&self) -> &[String] {
        self.inner.controller.controller.transcript()
    }

    pub fn is_quit(&self) -> bool {
        self.inner.is_quit()
    }

    pub fn is_busy(&mut self) -> bool {
        self.inner.is_busy()
    }

    pub fn needs_render(&self) -> bool {
        self.inner.needs_render()
    }

    pub fn mark_rendered(&mut self) {
        self.inner.mark_rendered();
    }

    pub fn tick(&mut self) {
        self.inner.tick();
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        self.inner.wait_for_idle(timeout)
    }

    pub fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        self.inner.wait_for_frame_contains(needle, timeout)
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_bytes(bytes)
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.inner.handle_byte(byte)
    }

    pub(crate) fn handle_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_terminal_bytes(bytes)
    }

    pub(crate) fn finish_pending_terminal_input(&mut self) {
        self.inner.finish_pending_terminal_input();
    }

    pub fn render_frame(&self) -> String {
        self.inner.render_frame()
    }

    pub fn poll_worker(&mut self) {
        self.inner.poll_worker();
    }

    #[cfg(test)]
    fn clear_agent_loading(&mut self, agent_id: &AgentId) {
        self.inner.clear_agent_loading(agent_id);
    }

    #[cfg(test)]
    fn set_agent_loading(&mut self, agent_id: &AgentId, loading: Option<LoadingKind>) {
        self.inner.set_agent_loading(agent_id, loading);
    }
}

#[derive(Debug)]
pub struct RemoteTerminalApp {
    inner: TerminalAppCore<HttpControllerClient>,
}

impl RemoteTerminalApp {
    pub fn new(client: HttpControllerClient, width: u16, height: u16) -> Self {
        Self {
            inner: TerminalAppCore::new(client, width, height),
        }
    }

    pub fn ui(&self) -> &TerminalUi {
        self.inner.ui()
    }

    pub fn is_quit(&self) -> bool {
        self.inner.is_quit()
    }

    pub fn is_busy(&mut self) -> bool {
        self.inner.is_busy()
    }

    pub fn needs_render(&self) -> bool {
        self.inner.needs_render()
    }

    pub fn mark_rendered(&mut self) {
        self.inner.mark_rendered();
    }

    pub fn tick(&mut self) {
        self.inner.tick();
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        self.inner.wait_for_idle(timeout)
    }

    pub fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        self.inner.wait_for_frame_contains(needle, timeout)
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_bytes(bytes)
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.inner.handle_byte(byte)
    }

    pub(crate) fn handle_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_terminal_bytes(bytes)
    }

    pub(crate) fn finish_pending_terminal_input(&mut self) {
        self.inner.finish_pending_terminal_input();
    }

    pub fn render_frame(&self) -> String {
        self.inner.render_frame()
    }

    pub fn poll_worker(&mut self) {
        self.inner.poll_worker();
    }
}

#[derive(Debug)]
struct LocalTerminalController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    controller: WorkLeafController<B>,
}

trait TerminalController {
    fn snapshot(&self) -> crate::WorkLeafSnapshot;
    fn drain_events(&mut self) -> Vec<WorkLeafEvent>;
    fn execute_command_line(&mut self, line: &str);
    fn send_command_agent_message(&mut self, message: &str);
    fn send_message(&mut self, agent_id: &AgentId, message: &str);
    fn interrupt_agent(&mut self, agent_id: &AgentId);
    fn push_transcript_line(&mut self, line: String);
    fn is_busy(&mut self) -> bool;
    fn loading_text(&self, loading: WorkLeafLoading) -> String;
    fn shutdown(&mut self);
}

impl<B> TerminalController for LocalTerminalController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    fn snapshot(&self) -> crate::WorkLeafSnapshot {
        self.controller.snapshot()
    }

    fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
        self.controller.drain_events()
    }

    fn execute_command_line(&mut self, line: &str) {
        self.controller.execute_command_line(line);
    }

    fn send_command_agent_message(&mut self, message: &str) {
        self.controller.send_command_agent_message(message);
    }

    fn send_message(&mut self, agent_id: &AgentId, message: &str) {
        let _ = self.controller.send_message(agent_id, message);
    }

    fn interrupt_agent(&mut self, agent_id: &AgentId) {
        self.controller.interrupt_agent(agent_id);
    }

    fn push_transcript_line(&mut self, line: String) {
        self.controller.push_transcript_line(line);
    }

    fn is_busy(&mut self) -> bool {
        self.controller.is_busy()
    }

    fn loading_text(&self, loading: WorkLeafLoading) -> String {
        self.controller.loading_text(loading)
    }

    fn shutdown(&mut self) {
        self.controller.shutdown();
    }
}

impl TerminalController for HttpControllerClient {
    fn snapshot(&self) -> crate::WorkLeafSnapshot {
        self.snapshot()
            .unwrap_or_else(|error| crate::WorkLeafSnapshot {
                command_transcript: vec![format!("error: {error}")],
                sessions: Vec::new(),
            })
    }

    fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
        HttpControllerClient::drain_events(self).unwrap_or_default()
    }

    fn execute_command_line(&mut self, line: &str) {
        let _ = HttpControllerClient::execute_command_line(self, line);
    }

    fn send_command_agent_message(&mut self, message: &str) {
        let _ = HttpControllerClient::send_command_agent_message(self, message);
    }

    fn send_message(&mut self, agent_id: &AgentId, message: &str) {
        let _ = HttpControllerClient::send_message(self, agent_id, message);
    }

    fn interrupt_agent(&mut self, agent_id: &AgentId) {
        let _ = HttpControllerClient::interrupt_agent(self, agent_id);
    }

    fn push_transcript_line(&mut self, line: String) {
        let _ = HttpControllerClient::push_transcript_line(self, line);
    }

    fn is_busy(&mut self) -> bool {
        HttpControllerClient::is_busy(self).unwrap_or(false)
    }

    fn loading_text(&self, loading: WorkLeafLoading) -> String {
        HttpControllerClient::loading_text(self, loading)
            .unwrap_or_else(|_| "Waiting for agent".to_string())
    }

    fn shutdown(&mut self) {
        let _ = HttpControllerClient::shutdown(self);
    }
}

#[derive(Debug)]
struct TerminalAppCore<C>
where
    C: TerminalController,
{
    controller: C,
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
    snapshot: WorkLeafSnapshot,
    loading_text: [(WorkLeafLoading, String); 2],
    dirty: bool,
    quit: bool,
}

impl<C> TerminalAppCore<C>
where
    C: TerminalController,
{
    fn new(controller: C, width: u16, height: u16) -> Self {
        let snapshot = controller.snapshot();
        let loading_text = [
            (
                WorkLeafLoading::Launching,
                controller.loading_text(WorkLeafLoading::Launching),
            ),
            (
                WorkLeafLoading::WaitingForReply,
                controller.loading_text(WorkLeafLoading::WaitingForReply),
            ),
        ];
        let mut app = Self {
            controller,
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
            snapshot,
            loading_text,
            dirty: true,
            quit: false,
        };
        let sessions = app.snapshot.sessions.clone();
        for session in sessions {
            app.apply_session_to_ui(&session);
        }
        app
    }

    fn ui(&self) -> &TerminalUi {
        &self.ui
    }

    fn is_quit(&self) -> bool {
        self.quit
    }

    fn is_busy(&mut self) -> bool {
        let busy = self.controller.is_busy();
        self.apply_controller_events();
        busy
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.ui.has_status_notice()
    }

    fn mark_rendered(&mut self) {
        self.dirty = false;
        self.ui.clear_expired_status_notice();
    }

    fn tick(&mut self) {
        let busy = self.controller.is_busy();
        self.apply_controller_events();
        if busy {
            self.spinner = (self.spinner + 1) % SPINNER.len();
            self.dirty = true;
        }
    }

    fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let busy = self.controller.is_busy();
            self.apply_controller_events();
            if !busy {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let busy = self.controller.is_busy();
        self.apply_controller_events();
        !busy
    }

    fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
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

    fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        if !self.handle_terminal_bytes(bytes) {
            return false;
        }
        self.finish_pending_terminal_input();
        !self.quit
    }

    fn handle_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        self.apply_controller_events();
        for byte in bytes {
            if !self.handle_byte_without_poll(*byte) {
                self.apply_controller_events();
                return false;
            }
        }
        self.apply_controller_events();
        !self.quit
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.apply_controller_events();
        let keep_running = self.handle_byte_without_poll(byte);
        self.apply_controller_events();
        keep_running && !self.quit
    }

    fn finish_pending_terminal_input(&mut self) {
        self.finish_pending_escape_sequence();
        self.apply_controller_events();
    }

    fn handle_byte_without_poll(&mut self, byte: u8) -> bool {
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
            TerminalAppInput::Interrupt => {
                self.ui.show_ctrl_c_exit_notice();
                if self.ui.focus() == PaneFocus::Right
                    && let Some(agent_id) = self.ui.selected_agent().cloned()
                {
                    self.controller.interrupt_agent(&agent_id);
                    self.apply_controller_events();
                }
                self.dirty = true;
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
                self.handle_ui_key(UiKey::Esc);
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
            TerminalAppInput::Char('/') if self.should_start_agent_slash_command() => {
                self.start_agent_slash_command();
                self.dirty = true;
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
                let actions = self.handle_ui_key(UiKey::Esc);
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Key(key) => {
                let actions = self.handle_ui_key(key);
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Char(ch) => {
                let actions = self.handle_ui_key(UiKey::Char(ch));
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

    fn handle_ui_key(&mut self, key: UiKey) -> Vec<crate::UiAction> {
        let right_content = self.right_content();
        let right_cursor_column = (self.ui.focus() == PaneFocus::Right
            && self.ui.mode() != UiMode::Prompt)
            .then_some(6 + self.chat_buffer.cursor_char_count());
        self.ui
            .handle_key_with_context(key, &right_content, right_cursor_column)
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
            self.controller.send_message(&agent_id, &message);
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
                    self.upsert_cached_session(session.clone());
                    self.apply_session_to_ui(&session);
                }
                WorkLeafEvent::AgentStatusUpdated {
                    agent_id,
                    kind,
                    title,
                    feature,
                    loading,
                    completion,
                } => {
                    let session = self.upsert_cached_session_status(
                        agent_id, kind, title, feature, loading, completion,
                    );
                    self.apply_session_to_ui(&session);
                }
                WorkLeafEvent::AgentLineAppended { agent_id, line } => {
                    self.append_cached_agent_line(&agent_id, line);
                }
                WorkLeafEvent::AgentSelected { agent_id } => {
                    let _ = self.ui.activate_agent_chat(&agent_id);
                }
                WorkLeafEvent::CommandTranscriptLine { line } => {
                    self.snapshot.command_transcript.push(line);
                }
                WorkLeafEvent::QuitRequested => {
                    self.quit = true;
                }
            }
        }
        self.dirty = true;
    }

    fn upsert_cached_session(&mut self, session: WorkLeafSession) {
        if let Some(existing) = self
            .snapshot
            .sessions
            .iter_mut()
            .find(|existing| existing.id == session.id)
        {
            *existing = session;
        } else {
            self.snapshot.sessions.push(session);
            self.snapshot
                .sessions
                .sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        }
    }

    fn upsert_cached_session_status(
        &mut self,
        agent_id: AgentId,
        kind: crate::agent::AgentKind,
        title: String,
        feature: String,
        loading: Option<WorkLeafLoading>,
        completion: Option<WorkLeafCompletion>,
    ) -> WorkLeafSession {
        if let Some(session) = self
            .snapshot
            .sessions
            .iter_mut()
            .find(|session| session.id == agent_id)
        {
            session.kind = kind;
            session.title = title;
            session.feature = feature;
            session.loading = loading;
            session.completion = completion;
            return session.clone();
        }

        let session = WorkLeafSession {
            id: agent_id,
            kind,
            title,
            feature,
            lines: Vec::new(),
            loading,
            completion,
        };
        self.snapshot.sessions.push(session.clone());
        self.snapshot
            .sessions
            .sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        session
    }

    fn append_cached_agent_line(&mut self, agent_id: &AgentId, line: String) {
        if line.is_empty() {
            return;
        }
        let Some(session) = self
            .snapshot
            .sessions
            .iter_mut()
            .find(|session| &session.id == agent_id)
        else {
            return;
        };
        if !session.lines.iter().any(|existing| existing == &line) {
            session.lines.push(line);
        }
    }

    fn apply_session_to_ui(&mut self, session: &WorkLeafSession) {
        let display_title = session_display_title(session);
        if self
            .ui
            .set_agent_feature(&session.id, display_title.clone())
            .is_err()
        {
            self.ui
                .add_agent(AgentListEntry::new(session.id.clone(), display_title));
        }
        let _ = self
            .ui
            .set_agent_ready_state(&session.id, session.loading.is_none());
    }

    fn should_start_agent_slash_command(&self) -> bool {
        self.ui.mode() == UiMode::Command && self.ui.selected_agent().is_some()
    }

    fn start_agent_slash_command(&mut self) {
        let Some(agent_id) = self.ui.selected_agent().cloned() else {
            return;
        };
        if self.ui.activate_agent_chat(&agent_id).is_ok() {
            self.chat_buffer.push('/');
            self.chat_history_index = None;
            self.chat_history_draft = None;
        }
    }

    fn should_route_chat_arrow(&self) -> bool {
        !self.ui.visual_selection_active()
            && (self.ui.mode() == UiMode::Insert
                || (self.ui.mode() == UiMode::Command && self.ui.focus() == PaneFocus::Right))
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
        if let Some(agent_id) = self.ui.selected_agent() {
            let session = self.snapshot.session(agent_id);
            let mut lines = session
                .map(|session| session.lines.clone())
                .unwrap_or_default();
            if let Some(loading) = session.and_then(|session| session.loading) {
                lines.push(format!(
                    "work-leaf: {} {}",
                    self.cached_loading_text(loading),
                    SPINNER[self.spinner]
                ));
            }
            return terminal_right_content(self.chat_buffer.as_str(), &lines);
        }
        terminal_right_content(self.chat_buffer.as_str(), &self.snapshot.command_transcript)
    }

    fn cached_loading_text(&self, loading: WorkLeafLoading) -> &str {
        self.loading_text
            .iter()
            .find(|(kind, _)| *kind == loading)
            .map(|(_, text)| text.as_str())
            .unwrap_or("Waiting for agent")
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
                let actions = self.handle_ui_key(UiKey::Char('i'));
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

fn session_display_title(session: &WorkLeafSession) -> String {
    match session.completion {
        Some(WorkLeafCompletion::NeedsDecision) => format!("{} DONE?", session.title),
        Some(WorkLeafCompletion::Closed) => format!("{} CLOSED", session.title),
        None => session.title.clone(),
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
        _ => parse_sgr_mouse_event(sequence).map(TerminalAppInput::Key),
    }
}

fn parse_sgr_mouse_event(sequence: &[u8]) -> Option<UiKey> {
    let final_byte = *sequence.last()?;
    if !sequence.starts_with(b"[<") || !matches!(final_byte, b'M' | b'm') {
        return None;
    }

    let body = std::str::from_utf8(&sequence[2..sequence.len() - 1]).ok()?;
    let mut parts = body.split(';');
    let button = parts.next()?.parse::<u16>().ok()?;
    let column = parts.next()?.parse::<u16>().ok()?;
    let row = parts.next()?.parse::<u16>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    let button_kind = button & !0b0001_1100_u16;
    match (button_kind, final_byte) {
        (64, b'M') => Some(UiKey::MouseScrollUp { column, row }),
        (65, b'M') => Some(UiKey::MouseScrollDown { column, row }),
        (_, b'M' | b'm') if button_kind < 64 && button & 0b11 == 0 => {
            Some(UiKey::MouseClick { column, row })
        }
        _ => None,
    }
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
    Interrupt,
    Quit,
}

impl TerminalAppInput {
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            3 => Some(Self::Interrupt),
            4 => Some(Self::Quit),
            13 | 10 => Some(Self::Enter),
            27 => Some(Self::Key(UiKey::Esc)),
            22 => Some(Self::Char('\u{16}')),
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
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use crate::agent::{
        AgentError, AgentKind, AgentLaunch, AgentSession, ChatMessage, MessageRole,
    };

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

    #[derive(Debug)]
    struct CountingController {
        snapshot: crate::WorkLeafSnapshot,
        snapshot_calls: Arc<AtomicUsize>,
        drain_calls: Arc<AtomicUsize>,
    }

    impl CountingController {
        fn new(
            snapshot: crate::WorkLeafSnapshot,
            snapshot_calls: Arc<AtomicUsize>,
            drain_calls: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                snapshot,
                snapshot_calls,
                drain_calls,
            }
        }
    }

    impl TerminalController for CountingController {
        fn snapshot(&self) -> crate::WorkLeafSnapshot {
            self.snapshot_calls.fetch_add(1, Ordering::Relaxed);
            self.snapshot.clone()
        }

        fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
            self.drain_calls.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        }

        fn execute_command_line(&mut self, _line: &str) {}

        fn send_command_agent_message(&mut self, _message: &str) {}

        fn send_message(&mut self, _agent_id: &AgentId, _message: &str) {}

        fn interrupt_agent(&mut self, _agent_id: &AgentId) {}

        fn push_transcript_line(&mut self, _line: String) {}

        fn is_busy(&mut self) -> bool {
            false
        }

        fn loading_text(&self, _loading: WorkLeafLoading) -> String {
            "Waiting for Codex".to_string()
        }

        fn shutdown(&mut self) {}
    }

    #[test]
    fn pasted_command_prompt_input_polls_controller_once_per_chunk() {
        let snapshot_calls = Arc::new(AtomicUsize::new(0));
        let drain_calls = Arc::new(AtomicUsize::new(0));
        let controller = CountingController::new(
            crate::WorkLeafSnapshot {
                command_transcript: Vec::new(),
                sessions: Vec::new(),
            },
            Arc::clone(&snapshot_calls),
            Arc::clone(&drain_calls),
        );
        let mut app = TerminalAppCore::new(controller, 80, 24);
        app.handle_bytes(b":");
        drain_calls.store(0, Ordering::Relaxed);

        let paste = "a".repeat(4096);
        assert!(app.handle_bytes(paste.as_bytes()));

        assert_eq!(app.prompt_buffer.as_str(), paste);
        assert!(
            drain_calls.load(Ordering::Relaxed) <= 3,
            "large input chunks should not drain events once per byte"
        );
    }

    #[test]
    fn rendering_uses_cached_snapshot_instead_of_refetching_full_transcripts() {
        let snapshot_calls = Arc::new(AtomicUsize::new(0));
        let drain_calls = Arc::new(AtomicUsize::new(0));
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        let controller = CountingController::new(
            crate::WorkLeafSnapshot {
                command_transcript: vec!["help".to_string()],
                sessions: vec![WorkLeafSession {
                    id: agent_id,
                    kind: AgentKind::Codex,
                    title: "feature".to_string(),
                    feature: "feature".to_string(),
                    lines: vec!["large transcript line".repeat(256)],
                    loading: None,
                    completion: None,
                }],
            },
            Arc::clone(&snapshot_calls),
            Arc::clone(&drain_calls),
        );
        let app = TerminalAppCore::new(controller, 80, 24);
        snapshot_calls.store(0, Ordering::Relaxed);

        assert!(app.render_frame().contains("help"));
        assert!(app.render_frame().contains("help"));

        assert_eq!(
            snapshot_calls.load(Ordering::Relaxed),
            0,
            "rendering and scrolling should use the local snapshot cache"
        );
    }

    #[test]
    fn clearing_agent_loading_marks_chat_ready_in_left_pane() {
        let chat = CommandChat::new(PathBuf::from("."), NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");

        app.inner
            .ui
            .add_agent(AgentListEntry::new(agent_id.clone(), "feature"));
        app.inner
            .ui
            .activate_agent_chat(&agent_id)
            .expect("test agent is registered");
        app.set_agent_loading(&agent_id, Some(LoadingKind::WaitingForReply));

        assert!(!app.render_frame().contains('\u{7}'));
        assert!(!app.ui().render_left_pane().contains("READY"));

        app.clear_agent_loading(&agent_id);

        assert!(app.render_frame().starts_with('\u{7}'));
        assert!(!app.render_frame().contains('\u{7}'));
        assert!(
            app.ui()
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

        assert!(app.ui().selected_agent().is_none());

        app.handle_bytes(b"ispawn a new patch agent that uses codex\n");

        assert!(app.wait_for_idle(Duration::from_secs(1)));
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        assert_eq!(app.ui().selected_agent(), Some(&agent_id));
        assert!(app.transcript().iter().any(|line| line
            == "command-agent: launching Codex user agent for patch agent that uses codex"));
        assert!(
            app.transcript()
                .iter()
                .any(|line| line == "work-leaf> new patch agent that uses codex")
        );
    }
}
