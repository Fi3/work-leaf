# Work Leaf Context Bundle

This file contains orchestrator-mediated read output. Use it as read-only context; submit project changes through `@work-leaf edit`.

----- BEGIN FILE src/terminal_app.rs -----
digest: fnv64:7ac50b1f40bc5531; bytes:43603

use std::thread;
use std::time::{Duration, Instant};

use rustyline::line_buffer::{ChangeListener, DeleteListener, Direction, LineBuffer};

use crate::agent::{AgentBackend, AgentId};
use crate::cli::{CommandChat, terminal_right_content, ui_action_text};
use crate::http_controller::HttpControllerClient;
use crate::ui::{AgentListEntry, PaneFocus, TerminalUi, UiKey, UiMode};
use crate::workspace::{
    WorkLeafController, WorkLeafEvent, WorkLeafLoading, WorkLeafSession, WorkLeafSessionStatus,
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
            self.apply_controller_events();
            if !self.controller.is_busy() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.apply_controller_events();
        !self.controller.is_busy()
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
        if is_agent_slash_command(line)
            && let Some(agent_id) = self.ui.selected_agent().cloned()
        {
            self.controller.send_message(&agent_id, line);
            self.apply_controller_events();
            return;
        }
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
                    status,
                } => {
                    let session = self
                        .upsert_cached_session_status(agent_id, kind, title, feature, loading, status);
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
        status: WorkLeafSessionStatus,
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
            session.status = status;
            return session.clone();
        }

        let session = WorkLeafSession {
            id: agent_id,
            kind,
            title,
            feature,
            lines: Vec::new(),
            loading,
            status,
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
            .set_agent_ready_state(&session.id, session.loading.is_none() && session.status != WorkLeafSessionStatus::Done);
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
            23 => Some(Self::Key(UiKey::CtrlW)),
            8 | 127 => Some(Self::Backspace),
            byte if byte.is_ascii_graphic() || byte == b' ' => Some(Self::Char(byte as char)),
            _ => None,
        }
    }
}

fn is_agent_slash_command(message: &str) -> bool {
    let mut chars = message.chars();
    matches!((chars.next(), chars.next()), (Some('/'), Some(next)) if !next.is_whitespace())
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
                    status: WorkLeafSessionStatus::Active,
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
----- END FILE src/terminal_app.rs -----

----- BEGIN FILE src/workspace.rs -----
digest: fnv64:cf68f2fb92946e4d; bytes:43937

use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::agent::{
    AgentBackend, AgentId, AgentKind, AgentLaunch, AgentShutdownHandle, AgentStreamEvent,
};
use crate::chat_title::{ChatTitleAgent, fallback_chat_title_from_prompt};
use crate::cli::{
    CliError, CommandChat, CommandChatResult, command_chat_error_text, command_result_text,
    render_command_chat_help,
};
use crate::review::{AgentCommit, GitHistory, ReviewResult};

#[derive(Debug)]
pub struct WorkLeafController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    chat: Option<CommandChat<B>>,
    shutdown: AgentShutdownHandle,
    shutdown_on_drop: bool,
    workers: Vec<Worker>,
    command_transcript: Vec<String>,
    sessions: BTreeMap<AgentId, WorkLeafSession>,
    title_agent: ChatTitleAgent,
    pending_events: Vec<WorkLeafEvent>,
    reviewers: BTreeSet<AgentId>,
    review_commits_in_progress: BTreeMap<AgentId, String>,
    reviewed_agent_commits: BTreeMap<AgentId, String>,
    agent_review_baselines: BTreeMap<AgentId, String>,
}

impl<B> WorkLeafController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    pub fn new(chat: CommandChat<B>) -> Self {
        let shutdown = chat.shutdown_handle();
        Self {
            chat: Some(chat),
            shutdown,
            shutdown_on_drop: true,
            workers: Vec::new(),
            command_transcript: vec![render_command_chat_help()],
            sessions: BTreeMap::new(),
            title_agent: ChatTitleAgent::new(),
            pending_events: Vec::new(),
            reviewers: BTreeSet::new(),
            review_commits_in_progress: BTreeMap::new(),
            reviewed_agent_commits: BTreeMap::new(),
            agent_review_baselines: BTreeMap::new(),
        }
    }

    pub fn into_chat(mut self) -> CommandChat<B> {
        self.wait_for_idle(Duration::from_secs(5));
        self.shutdown_on_drop = false;
        self.chat
            .take()
            .expect("work-leaf controller command chat is present")
    }

    pub fn transcript(&self) -> &[String] {
        &self.command_transcript
    }

    pub fn push_transcript_line(&mut self, line: impl Into<String>) {
        self.push_command_line(line.into());
    }

    pub fn snapshot(&self) -> WorkLeafSnapshot {
        WorkLeafSnapshot {
            command_transcript: self.command_transcript.clone(),
            sessions: self.sessions.values().cloned().collect(),
        }
    }

    pub fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
        if self.pending_events.is_empty() {
            self.poll_worker();
        }
        self.pending_events.drain(..).collect()
    }

    pub fn is_busy(&mut self) -> bool {
        self.poll_worker();
        !self.workers.is_empty()
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.poll_worker();
            if self.workers.is_empty() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.poll_worker();
        self.workers.is_empty()
    }

    pub fn wait_for_session_line(
        &mut self,
        agent_id: &AgentId,
        needle: &str,
        timeout: Duration,
    ) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.poll_worker();
            if self.session_contains(agent_id, needle) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.poll_worker();
        self.session_contains(agent_id, needle)
    }

    pub fn execute_command_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        self.push_command_line(format!("work-leaf> {trimmed}"));
        let parts = split_command_line(trimmed);
        let Some(command) = parts.first().map(String::as_str) else {
            return;
        };

        match command {
            "quit" | "exit" | "q" => self.request_quit(),
            "new" => {
                let prompt = parts[1..].join(" ");
                if let Err(error) = self.create_agent(prompt) {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            "review" => {
                if let Err(error) = self.start_review() {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            "linearize" => {
                if let Err(error) = self.start_linearize() {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            _ => self.start_command_worker(trimmed.to_string()),
        }
    }

    pub fn send_command_agent_message(&mut self, message: &str) {
        let message = message.trim();
        if message.is_empty() {
            return;
        }

        self.push_command_line(format!("user: {message}"));
        let display_name = self.agent_display_name();
        if literal_command_line(message).is_none()
            && let Some(request) = command_agent_new_request(message)
        {
            self.push_command_line(format!(
                "command-agent: {}",
                command_agent_launch_reply(&display_name, &request)
            ));
            let command_line = command_agent_new_command_line(&request.prompt);
            for _ in 0..request.count {
                self.execute_command_line(&command_line);
            }
            return;
        }

        match command_agent_response(message, &display_name) {
            CommandAgentResponse::Execute {
                command_line,
                reply,
            } => {
                self.push_command_line(format!("command-agent: {reply}"));
                self.execute_command_line(&command_line);
            }
            CommandAgentResponse::Reply(reply) => {
                self.push_command_line(format!("command-agent: {reply}"));
            }
        }
    }

    pub fn interrupt_agent(&mut self, agent_id: &AgentId) {
        let display_name = self.agent_display_name();
        let result = self
            .chat
            .as_mut()
            .expect("work-leaf controller command chat is present")
            .interrupt_agent(agent_id);
        match result {
            Ok(()) => self.append_agent_line(
                agent_id,
                format!("work-leaf: sent Ctrl-C to {display_name}"),
            ),
            Err(error) => self.append_agent_line(agent_id, command_chat_error_text(&error)),
        }
    }

    pub fn create_agent(&mut self, prompt: impl Into<String>) -> Result<AgentId, CliError> {
        let prompt = prompt.into();
        let args = split_command_line(&prompt);
        let title_pending = args.is_empty();
        let launch = {
            let chat = self
                .chat
                .as_mut()
                .expect("work-leaf controller command chat is present");
            chat.prepare_agent_launch(&args)?
        };
        let agent_id = launch.id.clone();
        self.remember_agent_review_baseline(&agent_id);
        let title_prompt = self.reserve_launch_title_prompt(&launch, title_pending);
        let title = launch.feature.clone();
        self.register_agent_feature(agent_id.clone(), title.clone());
        self.add_session(WorkLeafSession {
            id: agent_id.clone(),
            kind: launch.kind.clone(),
            title,
            feature: launch.feature.clone(),
            lines: Vec::new(),
            loading: Some(WorkLeafLoading::Launching),
            status: WorkLeafSessionStatus::Active,
        });
        self.pending_events.push(WorkLeafEvent::AgentSelected {
            agent_id: agent_id.clone(),
        });
        if let Some(first_prompt) = title_prompt {
            self.start_title_worker(agent_id.clone(), first_prompt);
        }
        self.start_launch_worker(launch);
        Ok(agent_id)
    }

    pub fn send_message(&mut self, agent_id: &AgentId, message: &str) -> Result<(), CliError> {
        let message = message.trim();
        if message.is_empty() {
            return Ok(());
        }
        if self
            .sessions
            .get(agent_id)
            .and_then(|session| session.loading)
            .is_some()
        {
            self.append_agent_line(
                agent_id,
                format!("work-leaf: {} is still working", self.agent_display_name()),
            );
            return Ok(());
        }

        let mut append_user_message = true;
        match self.session_status(agent_id) {
            WorkLeafSessionStatus::AwaitingFeatureDoneConfirmation => {
                self.append_agent_line(agent_id, format!("user: {message}"));
                append_user_message = false;
                if is_yes_response(message) {
                    self.append_agent_line(agent_id, "work-leaf: feature closed".to_string());
                    self.set_session_status(agent_id, WorkLeafSessionStatus::Done);
                    return Ok(());
                }
                self.set_session_status(agent_id, WorkLeafSessionStatus::Active);
                if is_no_response(message) {
                    self.append_agent_line(agent_id, "work-leaf: feature kept open".to_string());
                    return Ok(());
                }
            }
            WorkLeafSessionStatus::Done => {
                self.set_session_status(agent_id, WorkLeafSessionStatus::Active);
                self.append_agent_line(agent_id, "work-leaf: feature reopened".to_string());
            }
            WorkLeafSessionStatus::Active => {}
        }

        let title_prompt = self.reserve_first_chat_title_prompt(agent_id, message);
        self.set_session_loading(agent_id, Some(WorkLeafLoading::WaitingForReply));
        if append_user_message {
            self.append_agent_line(agent_id, format!("user: {message}"));
        }
        let agent_id = agent_id.clone();
        let message = message.to_string();
        if let Some(first_prompt) = title_prompt {
            self.start_title_worker(agent_id.clone(), first_prompt);
        }
        self.start_worker(move |mut chat, sender| {
            let stream_sender = sender.clone();
            let display_name = chat.agent_profile().display_name.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let _ = stream_sender.send(WorkerEvent::Stream {
                    agent_id: event_agent_id.clone(),
                    text: stream_event_text(event, &display_name),
                });
            };
            match chat.send_to_agent_streaming_with_ids(&agent_id, &message, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(agent_id),
                        result,
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(agent_id),
                        message: command_chat_error_text(&error),
                    });
                }
            }
        });
        Ok(())
    }

    pub fn start_review(&mut self) -> Result<Vec<AgentId>, CliError> {
        self.start_review_for_agent(None)
    }

    fn start_review_for_patch_agent(
        &mut self,
        agent_id: &AgentId,
    ) -> Result<Vec<AgentId>, CliError> {
        self.start_review_for_agent(Some(agent_id))
    }

    fn start_review_for_agent(
        &mut self,
        target_agent_id: Option<&AgentId>,
    ) -> Result<Vec<AgentId>, CliError> {
        let (project_dir, agent_profile) = {
            let chat = self
                .chat
                .as_ref()
                .expect("work-leaf controller command chat is present");
            (
                chat.project_dir().to_path_buf(),
                chat.agent_profile().clone(),
            )
        };
        let empty_baselines = BTreeMap::new();
        let agent_baselines = if target_agent_id.is_some() {
            &self.agent_review_baselines
        } else {
            &empty_baselines
        };
        let commits = GitHistory::new(project_dir)
            .latest_agent_review_commits(&self.reviewed_agent_commits, agent_baselines)?;
        if commits.is_empty() {
            self.push_command_line("no agent commits found".to_string());
            return Ok(Vec::new());
        }

        let mut reviewer_ids = Vec::new();
        for commit in commits {
            if target_agent_id.is_some_and(|agent_id| &commit.agent_id != agent_id) {
                continue;
            }
            if self
                .reviewed_agent_commits
                .get(&commit.agent_id)
                .is_some_and(|hash| hash == &commit.hash)
                || self
                    .review_commits_in_progress
                    .get(&commit.agent_id)
                    .is_some_and(|hash| hash == &commit.hash)
            {
                continue;
            }
            let reviewer_id = AgentId::new(format!("review-{}", commit.agent_id.as_str()))
                .map_err(CliError::Agent)?;
            let reviewer_busy = self
                .sessions
                .get(&reviewer_id)
                .and_then(|session| session.loading)
                .is_some();
            if reviewer_busy {
                continue;
            }
            let session_exists = self.sessions.contains_key(&reviewer_id);
            let reuse_reviewer = self.reviewers.contains(&reviewer_id);
            if session_exists {
                self.set_session_loading(&reviewer_id, Some(WorkLeafLoading::WaitingForReply));
            } else {
                self.add_session(WorkLeafSession {
                    id: reviewer_id.clone(),
                    kind: agent_profile.kind.clone(),
                    title: format!("review {}", commit.feature),
                    feature: format!("review {}", commit.feature),
                    lines: Vec::new(),
                    loading: Some(WorkLeafLoading::WaitingForReply),
                    status: WorkLeafSessionStatus::Active,
                });
            }
            if reviewer_ids.is_empty() {
                self.pending_events.push(WorkLeafEvent::AgentSelected {
                    agent_id: reviewer_id.clone(),
                });
            }
            self.review_commits_in_progress
                .insert(commit.agent_id.clone(), commit.hash.clone());
            self.start_review_worker(commit, reviewer_id.clone(), reuse_reviewer);
            reviewer_ids.push(reviewer_id);
        }
        Ok(reviewer_ids)
    }

    pub fn start_linearize(&mut self) -> Result<Option<AgentId>, CliError> {
        let launch = {
            let chat = self
                .chat
                .as_mut()
                .expect("work-leaf controller command chat is present");
            chat.prepare_linearize_launch()?
        };
        let Some(launch) = launch else {
            self.push_command_line("no reviewed agent commits found".to_string());
            return Ok(None);
        };

        let agent_id = launch.id.clone();
        let title = launch.feature.clone();
        self.register_agent_feature(agent_id.clone(), title.clone());
        self.add_session(WorkLeafSession {
            id: agent_id.clone(),
            kind: launch.kind.clone(),
            title: title.clone(),
            feature: title,
            lines: Vec::new(),
            loading: Some(WorkLeafLoading::Launching),
            status: WorkLeafSessionStatus::Active,
        });
        self.pending_events.push(WorkLeafEvent::AgentSelected {
            agent_id: agent_id.clone(),
        });
        self.start_launch_worker(launch);
        Ok(Some(agent_id))
    }

    pub fn loading_text(&self, loading: WorkLeafLoading) -> String {
        match loading {
            WorkLeafLoading::Launching => {
                format!("Starting {} session", self.agent_display_name())
            }
            WorkLeafLoading::WaitingForReply => {
                format!("Waiting for {}", self.agent_display_name())
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.shutdown.shutdown();
    }

    fn poll_worker(&mut self) {
        let mut events = Vec::new();
        for worker in &self.workers {
            while let Ok(event) = worker.receiver.try_recv() {
                events.push(event);
            }
        }
        for event in events {
            self.apply_worker_event(event);
        }

        let mut index = 0;
        while index < self.workers.len() {
            if self.workers[index].handle.is_finished() {
                let worker = self.workers.swap_remove(index);
                while let Ok(event) = worker.receiver.try_recv() {
                    self.apply_worker_event(event);
                }
                worker
                    .handle
                    .join()
                    .expect("work-leaf worker did not panic");
            } else {
                index += 1;
            }
        }
    }

    fn session_contains(&self, agent_id: &AgentId, needle: &str) -> bool {
        self.sessions
            .get(agent_id)
            .is_some_and(|session| session.lines.iter().any(|line| line.contains(needle)))
    }

    fn reserve_launch_title_prompt(
        &mut self,
        launch: &AgentLaunch,
        title_pending: bool,
    ) -> Option<String> {
        if title_pending {
            None
        } else {
            self.title_agent.mark_named(&launch.id);
            Some(launch.prompt.clone())
        }
    }

    fn register_agent_feature(&mut self, agent_id: AgentId, feature: String) {
        if let Some(chat) = self.chat.as_mut() {
            chat.register_agent_feature(agent_id, feature);
        }
    }

    fn reserve_first_chat_title_prompt(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
    ) -> Option<String> {
        if !agent_id.as_str().starts_with("user-") {
            return None;
        }
        if !self.title_agent.reserve_first_prompt_title(agent_id) {
            return None;
        }
        Some(prompt.to_string())
    }

    fn remember_agent_review_baseline(&mut self, agent_id: &AgentId) {
        if !agent_id.as_str().starts_with("user-")
            || self.agent_review_baselines.contains_key(agent_id)
        {
            return;
        }
        let Some(root) = self
            .chat
            .as_ref()
            .map(|chat| chat.project_dir().to_path_buf())
        else {
            return;
        };
        if let Ok(Some(hash)) = GitHistory::new(root).head_hash() {
            self.agent_review_baselines.insert(agent_id.clone(), hash);
        }
    }

    fn add_session(&mut self, session: WorkLeafSession) {
        self.sessions.insert(session.id.clone(), session.clone());
        self.pending_events
            .push(WorkLeafEvent::AgentAdded { session });
    }

    fn start_launch_worker(&mut self, launch: AgentLaunch) {
        let agent_id = launch.id.clone();
        self.set_session_loading(&agent_id, Some(WorkLeafLoading::Launching));
        self.start_worker(move |mut chat, sender| {
            let stream_sender = sender.clone();
            let display_name = chat.agent_profile().display_name.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let _ = stream_sender.send(WorkerEvent::Stream {
                    agent_id: event_agent_id.clone(),
                    text: stream_event_text(event, &display_name),
                });
            };
            match chat.launch_prepared_agent_streaming_with_ids(launch, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(agent_id),
                        result,
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(agent_id),
                        message: command_chat_error_text(&error),
                    });
                }
            }
        });
    }

    fn start_title_worker(&mut self, agent_id: AgentId, first_prompt: String) {
        self.start_worker(move |mut chat, sender| {
            let title = chat
                .generate_chat_title(&agent_id, &first_prompt)
                .unwrap_or_else(|_| fallback_chat_title_from_prompt(&first_prompt));
            let _ = sender.send(WorkerEvent::TitleGenerated { agent_id, title });
        });
    }

    fn start_review_worker(
        &mut self,
        commit: crate::review::AgentCommit,
        reviewer_id: AgentId,
        reuse_reviewer: bool,
    ) {
        self.start_worker(move |mut chat, sender| {
            let stream_sender = sender.clone();
            let display_name = chat.agent_profile().display_name.clone();
            let reviewed_agent_id = commit.agent_id.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let _ = stream_sender.send(WorkerEvent::Stream {
                    agent_id: event_agent_id.clone(),
                    text: stream_event_text(event, &display_name),
                });
            };
            match chat.review_commit_streaming_with_ids(
                commit,
                reviewer_id.clone(),
                reuse_reviewer,
                &mut stream,
            ) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(reviewer_id),
                        result: CommandChatResult::ReviewComplete(vec![result]),
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::ReviewError {
                        reviewer_id,
                        reviewed_agent_id,
                        message: error.to_string(),
                    });
                }
            }
        });
    }

    fn start_command_worker(&mut self, line: String) {
        self.start_worker(move |mut chat, sender| match chat.handle_line(&line) {
            Ok(result) => {
                let _ = sender.send(WorkerEvent::Complete {
                    agent_id: None,
                    result,
                });
            }
            Err(error) => {
                let _ = sender.send(WorkerEvent::Error {
                    agent_id: None,
                    message: command_chat_error_text(&error),
                });
            }
        });
    }

    fn start_worker<F>(&mut self, operation: F)
    where
        F: FnOnce(CommandChat<B>, Sender<WorkerEvent>) + Send + 'static,
    {
        let Some(chat) = self.chat.as_ref().cloned() else {
            return;
        };
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || operation(chat, sender));
        self.workers.push(Worker { receiver, handle });
    }

    fn apply_worker_event(&mut self, event: WorkerEvent) {
        match event {
            WorkerEvent::Stream { agent_id, text } => {
                self.append_agent_line(&agent_id, text);
            }
            WorkerEvent::TitleGenerated { agent_id, title } => {
                self.apply_agent_title(&agent_id, title);
            }
            WorkerEvent::Complete { agent_id, result } => {
                if let Some(agent_id) = agent_id {
                    let start_review = should_start_review(&agent_id, &result);
                    self.set_session_loading(&agent_id, None);
                    self.apply_agent_result(&agent_id, &result);
                    if start_review && let Err(error) = self.start_review_for_patch_agent(&agent_id)
                    {
                        self.push_command_line(command_chat_error_text(&error));
                    }
                } else {
                    self.push_command_line(command_result_text(&result));
                    if matches!(result, CommandChatResult::Quit) {
                        self.request_quit();
                    }
                }
            }
            WorkerEvent::Error { agent_id, message } => {
                if let Some(agent_id) = agent_id {
                    self.set_session_loading(&agent_id, None);
                    self.append_agent_line(&agent_id, message);
                } else {
                    self.push_command_line(message);
                }
            }
            WorkerEvent::ReviewError {
                reviewer_id,
                reviewed_agent_id,
                message,
            } => {
                self.review_commits_in_progress.remove(&reviewed_agent_id);
                self.set_session_loading(&reviewer_id, None);
                self.append_agent_line(&reviewer_id, message);
            }
        }
    }

    fn apply_agent_result(&mut self, agent_id: &AgentId, result: &CommandChatResult) {
        match result {
            CommandChatResult::AgentLaunched { reply, .. }
            | CommandChatResult::AgentMessage { reply, .. } => {
                if !reply.is_empty() {
                    self.append_agent_line(agent_id, reply.clone());
                }
            }
            CommandChatResult::ReviewComplete(results) => {
                let text = command_result_text(result);
                self.push_command_line(text.clone());
                for review in results {
                    self.record_review_result(review);
                    self.append_agent_line(&review.commit.agent_id, format!("review: {text}"));
                    self.request_feature_done_confirmation(&review.commit.agent_id);
                }
            }
            other => {
                self.push_command_line(command_result_text(other));
            }
        }
    }

    fn apply_agent_title(&mut self, agent_id: &AgentId, title: String) {
        if let Some(session) = self.sessions.get_mut(agent_id) {
            session.title = title.clone();
            session.feature = title.clone();
            self.pending_events.push(WorkLeafEvent::AgentStatusUpdated {
                agent_id: session.id.clone(),
                kind: session.kind.clone(),
                title: session.title.clone(),
                feature: session.feature.clone(),
                loading: session.loading,
                status: session.status,
            });
        }
        self.register_agent_feature(agent_id.clone(), title);
    }

    fn append_agent_line(&mut self, agent_id: &AgentId, line: String) {
        if line.is_empty() {
            return;
        }
        let fallback_kind = self.agent_kind();
        if !self.sessions.contains_key(agent_id) {
            let session = WorkLeafSession::unknown(agent_id.clone(), fallback_kind);
            self.sessions.insert(agent_id.clone(), session.clone());
            self.pending_events
                .push(WorkLeafEvent::AgentAdded { session });
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .expect("session was inserted before appending a line");
        if session.lines.iter().any(|existing| existing == &line) {
            return;
        }
        session.lines.push(line.clone());
        self.pending_events.push(WorkLeafEvent::AgentLineAppended {
            agent_id: agent_id.clone(),
            line,
        });
    }

    fn record_review_result(&mut self, review: &ReviewResult) {
        self.review_commits_in_progress.remove(&review.agent_id);
        let latest_commit = self
            .latest_agent_review_commit(&review.agent_id)
            .unwrap_or_else(|| review.commit.clone());
        self.reviewed_agent_commits
            .insert(review.agent_id.clone(), latest_commit.hash.clone());
        self.agent_review_baselines
            .insert(review.agent_id.clone(), latest_commit.hash.clone());
        if let Some(chat) = self.chat.as_mut() {
            chat.mark_reviewed_agent_commit(latest_commit);
        }
        self.reviewers.insert(review.reviewer_id.clone());
    }

    fn request_feature_done_confirmation(&mut self, agent_id: &AgentId) {
        self.append_agent_line(
            agent_id,
            "work-leaf: is this feature done? yes/no".to_string(),
        );
        self.set_session_status(
            agent_id,
            WorkLeafSessionStatus::AwaitingFeatureDoneConfirmation,
        );
    }

    fn latest_agent_review_commit(&self, agent_id: &AgentId) -> Option<AgentCommit> {
        let root = self
            .chat
            .as_ref()
            .map(|chat| chat.project_dir().to_path_buf())?;
        let boundary = self
            .reviewed_agent_commits
            .get(agent_id)
            .or_else(|| self.agent_review_baselines.get(agent_id))
            .map(String::as_str);
        GitHistory::new(root)
            .agent_review_commit(agent_id, boundary)
            .ok()?
    }

    fn set_session_loading(&mut self, agent_id: &AgentId, loading: Option<WorkLeafLoading>) {
        let fallback_kind = self.agent_kind();
        if !self.sessions.contains_key(agent_id) {
            let session = WorkLeafSession::unknown(agent_id.clone(), fallback_kind);
            self.sessions.insert(agent_id.clone(), session.clone());
            self.pending_events
                .push(WorkLeafEvent::AgentAdded { session });
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .expect("session was inserted before updating loading");
        session.loading = loading;
        self.pending_events.push(WorkLeafEvent::AgentStatusUpdated {
            agent_id: session.id.clone(),
            kind: session.kind.clone(),
            title: session.title.clone(),
            feature: session.feature.clone(),
            loading: session.loading,
            status: session.status,
        });
    }

    fn set_session_status(&mut self, agent_id: &AgentId, status: WorkLeafSessionStatus) {
        let fallback_kind = self.agent_kind();
        if !self.sessions.contains_key(agent_id) {
            let session = WorkLeafSession::unknown(agent_id.clone(), fallback_kind);
            self.sessions.insert(agent_id.clone(), session.clone());
            self.pending_events
                .push(WorkLeafEvent::AgentAdded { session });
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .expect("session was inserted before updating status");
        if session.status == status {
            return;
        }
        session.status = status;
        self.pending_events.push(WorkLeafEvent::AgentStatusUpdated {
            agent_id: session.id.clone(),
            kind: session.kind.clone(),
            title: session.title.clone(),
            feature: session.feature.clone(),
            loading: session.loading,
            status: session.status,
        });
    }

    fn session_status(&self, agent_id: &AgentId) -> WorkLeafSessionStatus {
        self.sessions
            .get(agent_id)
            .map(|session| session.status)
            .unwrap_or_default()
    }

    fn push_command_line(&mut self, line: String) {
        if line.is_empty() {
            return;
        }
        self.command_transcript.push(line.clone());
        self.pending_events
            .push(WorkLeafEvent::CommandTranscriptLine { line });
    }

    fn request_quit(&mut self) {
        self.shutdown.shutdown();
        self.pending_events.push(WorkLeafEvent::QuitRequested);
    }

    fn agent_display_name(&self) -> String {
        self.chat
            .as_ref()
            .map(|chat| chat.agent_profile().display_name.clone())
            .unwrap_or_else(|| "agent".to_string())
    }

    fn agent_kind(&self) -> AgentKind {
        self.chat
            .as_ref()
            .map(|chat| chat.agent_profile().kind.clone())
            .unwrap_or_else(|| AgentKind::External("agent".to_string()))
    }
}

impl<B> Drop for WorkLeafController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    fn drop(&mut self) {
        if self.shutdown_on_drop {
            self.shutdown.shutdown();
        }
    }
}

fn should_start_review(agent_id: &AgentId, result: &CommandChatResult) -> bool {
    agent_id.as_str().starts_with("user-")
        && match result {
            CommandChatResult::AgentLaunched { reply, .. }
            | CommandChatResult::AgentMessage { reply, .. } => {
                contains_applied_patch(agent_id, reply) && contains_done_directive(reply)
            }
            _ => false,
        }
}

fn contains_applied_patch(agent_id: &AgentId, text: &str) -> bool {
    let prefix = format!("applied patch from {agent_id}:");
    text.lines()
        .any(|line| line.trim_start().starts_with(&prefix))
}

fn contains_done_directive(text: &str) -> bool {
    text.lines()
        .any(|line| line.trim_start() == "@work-leaf done")
}

fn is_yes_response(message: &str) -> bool {
    matches!(message.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn is_no_response(message: &str) -> bool {
    matches!(message.trim().to_ascii_lowercase().as_str(), "n" | "no")
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkLeafSnapshot {
    pub command_transcript: Vec<String>,
    pub sessions: Vec<WorkLeafSession>,
}

impl WorkLeafSnapshot {
    pub fn session(&self, agent_id: &AgentId) -> Option<&WorkLeafSession> {
        self.sessions.iter().find(|session| &session.id == agent_id)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkLeafSession {
    pub id: AgentId,
    pub kind: AgentKind,
    pub title: String,
    pub feature: String,
    pub lines: Vec<String>,
    pub loading: Option<WorkLeafLoading>,
    #[serde(default)]
    pub status: WorkLeafSessionStatus,
}

impl WorkLeafSession {
    fn unknown(agent_id: AgentId, kind: AgentKind) -> Self {
        Self {
            id: agent_id,
            kind,
            title: "agent".to_string(),
            feature: "agent".to_string(),
            lines: Vec::new(),
            loading: None,
            status: WorkLeafSessionStatus::Active,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkLeafLoading {
    Launching,
    WaitingForReply,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkLeafSessionStatus {
    #[default]
    Active,
    AwaitingFeatureDoneConfirmation,
    Done,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkLeafEvent {
    AgentAdded {
        session: WorkLeafSession,
    },
    AgentUpdated {
        session: WorkLeafSession,
    },
    AgentStatusUpdated {
        agent_id: AgentId,
        kind: AgentKind,
        title: String,
        feature: String,
        loading: Option<WorkLeafLoading>,
        status: WorkLeafSessionStatus,
    },
    AgentLineAppended {
        agent_id: AgentId,
        line: String,
    },
    AgentSelected {
        agent_id: AgentId,
    },
    CommandTranscriptLine {
        line: String,
    },
    QuitRequested,
}

#[derive(Debug)]
struct Worker {
    receiver: Receiver<WorkerEvent>,
    handle: JoinHandle<()>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WorkerEvent {
    Stream {
        agent_id: AgentId,
        text: String,
    },
    TitleGenerated {
        agent_id: AgentId,
        title: String,
    },
    Complete {
        agent_id: Option<AgentId>,
        result: CommandChatResult,
    },
    Error {
        agent_id: Option<AgentId>,
        message: String,
    },
    ReviewError {
        reviewer_id: AgentId,
        reviewed_agent_id: AgentId,
        message: String,
    },
}

fn stream_event_text(event: AgentStreamEvent, agent_display_name: &str) -> String {
    let label = agent_display_name.to_ascii_lowercase();
    match event {
        AgentStreamEvent::Status(text) => format!("{label}: {text}"),
        AgentStreamEvent::AgentMessage(text) => text,
        AgentStreamEvent::Error(text) => format!("{label} error: {text}"),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CommandAgentResponse {
    Execute { command_line: String, reply: String },
    Reply(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommandAgentNewRequest {
    count: usize,
    prompt: String,
}

fn command_agent_response(message: &str, agent_display_name: &str) -> CommandAgentResponse {
    if let Some(command_line) = literal_command_line(message) {
        return CommandAgentResponse::Execute {
            reply: format!("running `{command_line}`"),
            command_line,
        };
    }

    let lower = message.to_ascii_lowercase();
    if asks_for_new_agent(&lower) {
        let prompt = command_agent_new_prompt(message);
        let command_line = if prompt.is_empty() {
            "new".to_string()
        } else {
            format!("new {prompt}")
        };
        let reply = if prompt.is_empty() {
            format!("launching {agent_display_name} user agent")
        } else {
            format!("launching {agent_display_name} user agent for {prompt}")
        };
        return CommandAgentResponse::Execute {
            command_line,
            reply,
        };
    }

    for (needle, command_line) in [
        ("linearize", "linearize"),
        ("linearise", "linearize"),
        ("review", "review"),
        ("help", "help"),
        ("quit", "quit"),
        ("exit", "quit"),
    ] {
        if lower.contains(needle) {
            return CommandAgentResponse::Execute {
                command_line: command_line.to_string(),
                reply: format!("running `{command_line}`"),
            };
        }
    }

    CommandAgentResponse::Reply(
        "I can run help, new [prompt...], review, linearize, or quit.".to_string(),
    )
}

fn literal_command_line(message: &str) -> Option<String> {
    let command = split_command_line(message).into_iter().next()?;
    matches!(
        command.as_str(),
        "help" | "?" | "new" | "review" | "linearize" | "quit" | "exit" | "q"
    )
    .then(|| message.to_string())
}

fn asks_for_new_agent(lower: &str) -> bool {
    lower.contains("agent")
        && ["new", "spawn", "create", "start", "launch"]
            .iter()
            .any(|verb| lower.contains(verb))
}

fn command_agent_new_request(message: &str) -> Option<CommandAgentNewRequest> {
    let lower = message.to_ascii_lowercase();
    if !asks_for_agent_launch_request(&lower) {
        return None;
    }

    let prompt = command_agent_launch_prompt(message);
    let count = agent_launch_count(&prompt).unwrap_or(1);
    let prompt = strip_agent_launch_count_and_noun(&prompt);
    Some(CommandAgentNewRequest {
        count,
        prompt: normalize_common_agent_typos(&prompt),
    })
}

fn asks_for_agent_launch_request(lower: &str) -> bool {
    lower.contains("agent")
        && ["new", "spawn", "create", "start", "launch", "open", "make"]
            .iter()
            .any(|verb| lower.contains(verb))
}

fn command_agent_launch_prompt(message: &str) -> String {
    let trimmed = strip_polite_prefix(message.trim());
    [
        "open a new ",
        "open new ",
        "open an ",
        "open a ",
        "open ",
        "spawn a new ",
        "spawn new ",
        "spawn an ",
        "spawn a ",
        "spawn ",
        "create a new ",
        "create new ",
        "create an ",
        "create a ",
        "create ",
        "start a new ",
        "start new ",
        "start an ",
        "start a ",
        "start ",
        "launch a new ",
        "launch new ",
        "launch an ",
        "launch a ",
        "launch ",
        "make a new ",
        "make new ",
        "make an ",
        "make a ",
        "make ",
        "new an ",
        "new a ",
        "new ",
    ]
    .iter()
    .find_map(|prefix| strip_ascii_prefix_case_insensitive(trimmed, prefix))
    .unwrap_or(trimmed)
    .to_string()
}

fn command_agent_new_command_line(prompt: &str) -> String {
    if prompt.is_empty() {
        "new".to_string()
    } else {
        format!("new {prompt}")
    }
}

fn command_agent_launch_reply(
    agent_display_name: &str,
    request: &CommandAgentNewRequest,
) -> String {
    let count_prefix = if request.count > 1 {
        format!("{} ", request.count)
    } else {
        String::new()
    };
    let agent_label = if request.count == 1 {
        "user agent"
    } else {
        "user agents"
    };

    if request.prompt.is_empty() {
        format!("launching {count_prefix}{agent_display_name} {agent_label}")
    } else {
        format!(
            "launching {count_prefix}{agent_display_name} {agent_label} for {}",
            request.prompt
        )
    }
}

fn agent_launch_count(text: &str) -> Option<usize> {
    text.split_whitespace().next().and_then(agent_count_word)
}

fn strip_agent_launch_count_and_noun(prompt: &str) -> String {
    let words = prompt.split_whitespace().collect::<Vec<_>>();
    let mut start = 0;
    let mut end = words.len();
    if words
        .first()
        .and_then(|word| agent_count_word(word))
        .is_some()
    {
        start = 1;
    }
    if words.last().is_some_and(|word| is_agent_noun(word)) {
        end -= 1;
    }
    words[start..end].join(" ")
}

fn agent_count_word(word: &str) -> Option<usize> {
    let clean = clean_agent_word(word);
    if let Ok(count) = clean.parse::<usize>() {
        return (count > 0).then_some(count);
    }

    match clean.as_str() {
        "a" | "an" | "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        _ => None,
    }
}

fn is_agent_noun(word: &str) -> bool {
    matches!(clean_agent_word(word).as_str(), "agent" | "agents")
}

fn normalize_common_agent_typos(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            if clean_agent_word(word) == "pacth" {
                "patch"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn clean_agent_word(word: &str) -> String {
    word.trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
        .to_ascii_lowercase()
}

fn command_agent_new_prompt(message: &str) -> String {
    let trimmed = strip_polite_prefix(message.trim());
    [
        "spawn a new ",
        "spawn new ",
        "create a new ",
        "create new ",
        "start a new ",
        "start new ",
        "launch a new ",
        "launch new ",
        "make a new ",
        "make new ",
        "new ",
    ]
    .iter()
    .find_map(|prefix| strip_ascii_prefix_case_insensitive(trimmed, prefix))
    .unwrap_or(trimmed)
    .to_string()
}

fn strip_polite_prefix(message: &str) -> &str {
    ["please ", "can you ", "could you ", "would you "]
        .iter()
        .find_map(|prefix| strip_ascii_prefix_case_insensitive(message, prefix))
        .unwrap_or(message)
}

fn strip_ascii_prefix_case_insensitive<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    message
        .to_ascii_lowercase()
        .starts_with(prefix)
        .then(|| message[prefix.len()..].trim())
}

fn split_command_line(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}
----- END FILE src/workspace.rs -----

----- BEGIN FILE tests/workspace.rs -----
digest: fnv64:8db23e3c84350d4b; bytes:31718

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentProfile, AgentSession,
    ChatMessage, CommandChat, MessageRole, WorkLeafController, WorkLeafEvent, WorkLeafLoading,
    WorkLeafSessionStatus,
};

#[test]
fn controller_exposes_ui_neutral_events_and_snapshot_without_terminal_ui() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("implement parser combinator")
        .unwrap();

    assert_eq!(agent_id, AgentId::new("user-1").unwrap());
    assert!(controller.drain_events().iter().any(|event| {
        matches!(event, WorkLeafEvent::AgentAdded { session } if session.id == agent_id)
    }));
    let starting = controller.snapshot();
    let session = starting.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "user-agent");
    assert_eq!(session.loading, Some(WorkLeafLoading::Launching));

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let ready = controller.snapshot();
    let session = ready.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "parser-combinator");
    assert_eq!(session.loading, None);
    assert!(session.lines.iter().any(|line| line == "launch reply"));

    controller.send_message(&agent_id, "continue").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let replied = controller.snapshot();
    let session = replied.session(&agent_id).expect("session exists");
    assert!(session.lines.iter().any(|line| line == "user: continue"));
    assert!(session.lines.iter().any(|line| line == "follow reply"));
}

#[test]
fn controller_line_events_do_not_resend_full_session_transcripts() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("stream compactly").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let events = controller.drain_events();
    assert!(events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentLineAppended { agent_id: id, line }
                if id == &agent_id && line == "launch reply"
        )
    }));
    assert!(
        !events.iter().any(|event| {
            matches!(
                event,
                WorkLeafEvent::AgentUpdated { session }
                    if session.id == agent_id
                        && session.lines.iter().any(|line| line == "launch reply")
            )
        }),
        "line append events should not be paired with full-session transcript updates"
    );
}

#[test]
fn controller_status_events_do_not_resend_existing_large_transcripts() {
    let large_reply = "large transcript line\n".repeat(8192);
    let backend = FakeBackend::new([large_reply.as_str(), "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("keep status compact").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.drain_events();

    controller.send_message(&agent_id, "continue").unwrap();
    let waiting_events = controller.drain_events();
    assert!(waiting_events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentStatusUpdated {
                agent_id: id,
                loading: Some(WorkLeafLoading::WaitingForReply),
                ..
            } if id == &agent_id
        )
    }));
    assert!(
        !waiting_events.iter().any(|event| {
            matches!(
                event,
                WorkLeafEvent::AgentUpdated { session }
                    if session.id == agent_id
                        && session.lines.iter().any(|line| line == &large_reply)
            )
        }),
        "status changes should not serialize existing transcript text"
    );

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let ready_events = controller.drain_events();
    assert!(ready_events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentStatusUpdated {
                agent_id: id,
                loading: None,
                ..
            } if id == &agent_id
        )
    }));
    assert!(ready_events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentLineAppended { agent_id: id, line }
                if id == &agent_id && line == "follow reply"
        )
    }));
    assert!(
        !ready_events.iter().any(|event| {
            matches!(
                event,
                WorkLeafEvent::AgentUpdated { session }
                    if session.id == agent_id
                        && session.lines.iter().any(|line| line == &large_reply)
            )
        }),
        "ready status changes should not serialize existing transcript text"
    );
}

#[test]
fn controller_uses_backend_agent_to_name_chat_from_first_prompt() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("please fix login callback")
        .unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "oauth-redirect-handler");
    assert!(session.lines.iter().any(|line| line == "launch reply"));

    let launches = backend.launches();
    assert!(launches.iter().any(|launch| {
        launch.id.as_str() == "title-user-1"
            && launch.feature == "chat-title"
            && launch.prompt.contains("please fix login callback")
    }));
}

#[test]
fn controller_uses_agent_profile_for_non_codex_launches_and_reviews() {
    let root = git_repo("workspace-custom-profile-review");
    fs::write(root.join("README.md"), "fixture\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply parser patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: parser\nReason: parse configs\nContext: parser context",
        ],
    );
    let backend = FakeBackend::new(["launch reply", "summary", "NO_FINDINGS"]);
    let profile = AgentProfile::new(
        AgentKind::External("local-test-agent".to_string()),
        "Local Test Agent",
        "local-agent",
    );
    let chat = CommandChat::new(root, backend.clone()).with_agent_profile(profile.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("build custom provider path")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.start_review().unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let launches = backend.launches();
    assert!(launches.iter().any(|launch| {
        launch.id == agent_id
            && launch.kind == profile.kind
            && launch.feature == profile.default_feature
    }));
    assert!(
        launches
            .iter()
            .any(|launch| { launch.id.as_str() == "review-user-1" && launch.kind == profile.kind })
    );
}

#[test]
fn controller_starts_review_after_patch_agent_done_and_loops_until_clean() {
    let root = git_repo("workspace-automatic-review-loop");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after",
        "FINDINGS\n- missing reviewed wording",
        "fixed review finding\n@work-leaf patch address review\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after\n+after review\n@work-leaf end\n@work-leaf done",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after review\n"
    );

    let reviewer_id = AgentId::new("review-user-1").unwrap();
    let snapshot = controller.snapshot();
    let reviewer = snapshot
        .session(&reviewer_id)
        .expect("reviewer session exists");
    assert_eq!(reviewer.title, "review user-agent");
    assert_eq!(reviewer.loading, None);
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(
        patch_agent.status,
        WorkLeafSessionStatus::AwaitingFeatureDoneConfirmation
    );
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line.contains("missing reviewed wording"))
    );
    assert!(
        patch_agent.lines.iter().any(|line| {
            line.contains("user-1 reviewed by review-user-1: rounds=2 resolved=yes")
        })
    );
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line == "work-leaf: is this feature done? yes/no")
    );

    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &agent_id
            && prompt.contains("missing reviewed wording")
            && prompt.contains("Please fix the patch")
    }));
    assert!(sends.iter().any(|(target, prompt)| {
        target == &reviewer_id
            && prompt.contains("The original agent has responded to the findings")
            && prompt.contains("Please check the patch again")
    }));
}

#[test]
fn controller_yes_closes_reviewed_feature_and_later_chat_reopens_it() {
    let root = git_repo("workspace-feature-done-confirmation");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: README changes",
        "NO_FINDINGS",
        "follow-up reply",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let reviewed = controller.snapshot();
    assert_eq!(
        reviewed.session(&agent_id).expect("patch agent").status,
        WorkLeafSessionStatus::AwaitingFeatureDoneConfirmation
    );

    let sends_before_close = backend.sends().len();
    controller.send_message(&agent_id, "yes").unwrap();
    let closed = controller.snapshot();
    let closed_session = closed.session(&agent_id).expect("patch agent");
    assert_eq!(closed_session.status, WorkLeafSessionStatus::Done);
    assert!(closed_session.lines.iter().any(|line| line == "user: yes"));
    assert!(
        closed_session
            .lines
            .iter()
            .any(|line| line == "work-leaf: feature closed")
    );
    assert_eq!(
        backend.sends().len(),
        sends_before_close,
        "confirmation replies are handled locally"
    );

    controller
        .send_message(&agent_id, "please reopen this feature")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let reopened = controller.snapshot();
    let reopened_session = reopened.session(&agent_id).expect("patch agent");
    assert_eq!(reopened_session.status, WorkLeafSessionStatus::Active);
    assert!(
        reopened_session
            .lines
            .iter()
            .any(|line| line == "work-leaf: feature reopened")
    );
    assert!(
        reopened_session
            .lines
            .iter()
            .any(|line| line == "follow-up reply")
    );
    assert!(backend.sends().iter().any(|(target, prompt)| {
        target == &agent_id && prompt == "please reopen this feature"
    }));
}

#[test]
fn controller_does_not_start_review_until_patch_agent_reports_done() {
    let root = git_repo("workspace-review-waits-for-done");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end",
        "summary that should not be requested",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let snapshot = controller.snapshot();
    assert!(
        snapshot
            .session(&AgentId::new("review-user-1").unwrap())
            .is_none(),
        "review must wait for the patch agent to report done"
    );
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(patch_agent.loading, None);
    let launches = backend.launches();
    assert!(
        !launches
            .iter()
            .any(|launch| launch.id.as_str() == "review-user-1")
    );
    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &agent_id
            && prompt.contains("work-leaf patch applied")
            && prompt.contains("@work-leaf done")
    }));
}

#[test]
fn controller_review_prompt_covers_all_agent_commits_since_launch() {
    let root = git_repo("workspace-review-full-agent-scope");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch first step\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end",
        "second patch\n@work-leaf patch second step\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: full change",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("update readme in two steps")
        .unwrap();

    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let launches = backend.launches();
    let review_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "review-user-1")
        .expect("reviewer launched");
    assert!(
        review_launch.prompt.contains("first step"),
        "{}",
        review_launch.prompt
    );
    assert!(
        review_launch.prompt.contains("second step"),
        "{}",
        review_launch.prompt
    );
}

#[test]
fn controller_reuses_one_reviewer_for_repeated_patch_agent_iterations() {
    let root = git_repo("workspace-reuses-reviewer");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end\n@work-leaf done",
        "summary: README changes to after first",
        "NO_FINDINGS",
        "second patch\n@work-leaf patch update readme again\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: README changes to after second",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    controller
        .send_message(&agent_id, "make the second update")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after second\n"
    );
    let reviewer_id = AgentId::new("review-user-1").unwrap();
    let launches = backend.launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id == reviewer_id)
            .count(),
        1
    );
    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &reviewer_id
            && prompt.contains("Review the full patch scope")
            && prompt.contains("after second")
    }));
}

#[test]
fn controller_reviews_only_unreviewed_patch_agent_commits() {
    let root = git_repo("workspace-reviews-only-unreviewed");
    fs::write(root.join("README.md"), "readme before\n").unwrap();
    fs::write(root.join("CHANGELOG.md"), "changelog before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial docs fixture"]);
    let backend = FakeBackend::new([
        "readme patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-readme before\n+readme after\n@work-leaf end\n@work-leaf done",
        "summary: README changes",
        "NO_FINDINGS",
        "changelog patch\n@work-leaf patch update changelog\n--- a/CHANGELOG.md\n+++ b/CHANGELOG.md\n@@ -1 +1 @@\n-changelog before\n+changelog after\n@work-leaf end\n@work-leaf done",
        "summary: changelog changes",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let first = controller.create_agent("update readme").unwrap();
    assert_eq!(first.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let second = controller.create_agent("update changelog").unwrap();
    assert_eq!(second.as_str(), "user-2");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str() == "review-user-1")
            .count(),
        1
    );
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str() == "review-user-2")
            .count(),
        1
    );
}

#[test]
fn controller_auto_review_ignores_historical_agents_outside_current_patch_agent() {
    let root = git_repo("workspace-auto-review-current-agent-only");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    for old_agent in ["user-2", "user-3"] {
        fs::write(
            root.join("README.md"),
            format!("historical commit from {old_agent}\n"),
        )
        .unwrap();
        git(&root, ["add", "README.md"]);
        git(
            &root,
            [
                "commit",
                "-m",
                "UPDATE apply historical patch",
                "-m",
                &format!(
                    "Agent-ID: {old_agent}\nFeature: historical\nReason: previous work\nContext: old patch"
                ),
            ],
        );
    }
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "UPDATE reset live fixture"]);
    let backend = FakeBackend::new([
        "live patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: live README change",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str().starts_with("review-"))
            .map(|launch| launch.id.as_str().to_string())
            .collect::<Vec<_>>(),
        vec!["review-user-1".to_string()]
    );
}

#[test]
fn controller_linearize_uses_only_commits_reviewed_in_this_session() {
    let root = git_repo("workspace-linearize-current-reviewed-only");
    fs::write(root.join("README.md"), "before\n").unwrap();
    fs::write(root.join("legacy.txt"), "legacy before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial linearize fixture"]);
    fs::write(root.join("legacy.txt"), "legacy after\n").unwrap();
    git(&root, ["add", "legacy.txt"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply legacy patch from user-2",
            "-m",
            "Agent-ID: user-2\nFeature: legacy\nReason: old run\nContext: old session",
        ],
    );
    let backend = FakeBackend::new([
        "live patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: live README change",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert!(controller.start_linearize().unwrap().is_some());
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    let linearize_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "linearize")
        .expect("linearize agent launched");
    assert!(linearize_launch.prompt.contains("Agent-ID: user-1"));
    assert!(linearize_launch.prompt.contains("Commit:"));
    assert!(!linearize_launch.prompt.contains("Agent-ID: user-2"));
    assert!(!linearize_launch.prompt.contains("old session"));
}

#[test]
fn controller_linearize_keeps_multiple_reviewed_commits_from_same_agent() {
    let root = git_repo("workspace-linearize-same-agent-multiple-reviewed");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch update readme once\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end\n@work-leaf done",
        "summary: first reviewed change",
        "NO_FINDINGS",
        "second patch\n@work-leaf patch update readme twice\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: second reviewed change",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    controller
        .send_message(&agent_id, "make a second reviewed update")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert!(controller.start_linearize().unwrap().is_some());
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    let linearize_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "linearize")
        .expect("linearize agent launched");
    assert!(
        linearize_launch
            .prompt
            .contains("Reason: update readme once"),
        "{}",
        linearize_launch.prompt
    );
    assert!(
        linearize_launch
            .prompt
            .contains("Reason: update readme twice"),
        "{}",
        linearize_launch.prompt
    );
    assert_eq!(
        linearize_launch.prompt.matches("Agent-ID: user-1").count(),
        2
    );
}

#[test]
fn controller_does_not_run_project_required_checks_after_agent_reply() {
    let root = git_repo("workspace-no-required-check-run");
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\n- `sh check.sh`\n",
    )
    .unwrap();
    fs::write(
        root.join("check.sh"),
        "#!/bin/sh\necho state is bad\nexit 1\n",
    )
    .unwrap();
    fs::write(root.join("state.txt"), "bad\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD project instruction fixture"]);
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(root.clone(), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("inspect required checks").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert_eq!(fs::read_to_string(root.join("state.txt")).unwrap(), "bad\n");
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.loading, None);
    assert!(
        !session
            .lines
            .iter()
            .any(|line| line.contains("required check failed"))
    );
    assert!(backend.sends().is_empty());
}

#[test]
fn controller_keeps_agent_loading_scoped_to_the_active_session() {
    let backend = ConcurrentBackend;
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let first = controller.create_agent("first task").unwrap();
    let second = controller.create_agent("second task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    controller.send_message(&second, "slow question").unwrap();
    assert_eq!(
        controller
            .snapshot()
            .session(&second)
            .expect("second session")
            .loading,
        Some(WorkLeafLoading::WaitingForReply)
    );

    controller.send_message(&first, "quick question").unwrap();

    assert!(controller.wait_for_session_line(&first, "quick reply", Duration::from_millis(150)));
    let snapshot = controller.snapshot();
    let first_session = snapshot.session(&first).expect("first session");
    assert!(first_session.lines.iter().any(|line| line == "quick reply"));
    assert!(
        !first_session
            .lines
            .iter()
            .any(|line| line.contains("still working"))
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
}

#[derive(Clone, Debug)]
struct ConcurrentBackend;

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeBackendState {
                replies: replies.into_iter().map(String::from).collect(),
                launches: Vec::new(),
                sends: Vec::new(),
            })),
        }
    }

    fn launches(&self) -> Vec<AgentLaunch> {
        self.state.lock().unwrap().launches.clone()
    }

    fn sends(&self) -> Vec<(AgentId, String)> {
        self.state.lock().unwrap().sends.clone()
    }

    fn next_reply(&self) -> String {
        self.state
            .lock()
            .unwrap()
            .replies
            .pop_front()
            .expect("missing fake reply")
    }

    fn title_reply(&self, prompt: &str) -> String {
        fake_title_from_title_prompt(prompt)
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.state.lock().unwrap().launches.push(request.clone());
        let mut session = AgentSession::new(request);
        let reply = if session.id.as_str().starts_with("title-") {
            self.title_reply(&session.messages[0].text)
        } else {
            self.next_reply()
        };
        session.push_message(MessageRole::Agent, reply);
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.state
            .lock()
            .unwrap()
            .sends
            .push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(MessageRole::Agent, self.next_reply()))
    }
}

impl AgentBackend for ConcurrentBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str() == "user-2" {
            thread::sleep(Duration::from_millis(350));
            return Ok(ChatMessage::new(MessageRole::Agent, "slow reply"));
        }
        Ok(ChatMessage::new(MessageRole::Agent, "quick reply"))
    }
}

fn git_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    git(&root, ["init", "-q"]);
    git(&root, ["config", "user.email", "test@example.com"]);
    git(&root, ["config", "user.name", "Test User"]);
    root
}

fn git<const N: usize>(root: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn fake_title_from_title_prompt(prompt: &str) -> String {
    let first_prompt = prompt
        .rsplit_once("First prompt:\n")
        .map(|(_, first_prompt)| first_prompt)
        .unwrap_or(prompt);
    if first_prompt.contains("parser combinator") {
        "parser-combinator".to_string()
    } else if first_prompt.contains("login callback")
        || first_prompt.contains("OAuth redirect handler")
    {
        "oauth-redirect-handler".to_string()
    } else {
        "chat-title".to_string()
    }
}
----- END FILE tests/workspace.rs -----
