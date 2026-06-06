use std::collections::BTreeMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use rustyline::line_buffer::{ChangeListener, DeleteListener, Direction, LineBuffer};

use crate::agent::{AgentId, AgentLaunch};
use crate::cli::{
    CommandChat, CommandChatResult, command_chat_error_text, command_result_text,
    terminal_right_content, ui_action_text,
};
use crate::codex::{AgentBackend, AgentShutdownHandle, AgentStreamEvent};
use crate::ui::{AgentListEntry, TerminalUi, UiKey, UiMode};

#[derive(Debug)]
pub struct TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    chat: Option<CommandChat<B>>,
    shutdown: AgentShutdownHandle,
    shutdown_on_drop: bool,
    workers: Vec<Worker>,
    ui: TerminalUi,
    prompt_buffer: PromptLine,
    chat_buffer: PromptLine,
    command_transcript: Vec<String>,
    agent_chats: BTreeMap<AgentId, AgentChat>,
    escape_sequence: Option<Vec<u8>>,
    next_user_agent: usize,
    spinner: usize,
    dirty: bool,
    quit: bool,
}

impl<B> TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    pub fn new(chat: CommandChat<B>, width: u16, height: u16) -> Self {
        let shutdown = chat.shutdown_handle();
        let next_user_agent = chat.next_user_agent_index();
        Self {
            chat: Some(chat),
            shutdown,
            shutdown_on_drop: true,
            workers: Vec::new(),
            ui: TerminalUi::new(width, height),
            prompt_buffer: PromptLine::new(),
            chat_buffer: PromptLine::new(),
            command_transcript: vec![crate::cli::render_command_chat_help()],
            agent_chats: BTreeMap::new(),
            escape_sequence: None,
            next_user_agent,
            spinner: 0,
            dirty: true,
            quit: false,
        }
    }

    pub fn into_chat(mut self) -> CommandChat<B> {
        self.wait_for_idle(Duration::from_secs(5));
        self.shutdown_on_drop = false;
        self.chat
            .take()
            .expect("terminal app command chat is present")
    }

    pub fn ui(&self) -> &TerminalUi {
        &self.ui
    }

    pub fn transcript(&self) -> &[String] {
        &self.command_transcript
    }

    pub fn is_quit(&self) -> bool {
        self.quit
    }

    pub fn is_busy(&mut self) -> bool {
        self.poll_worker();
        !self.workers.is_empty()
    }

    pub fn needs_render(&self) -> bool {
        self.dirty
    }

    pub fn mark_rendered(&mut self) {
        self.dirty = false;
    }

    pub fn tick(&mut self) {
        self.poll_worker();
        if !self.workers.is_empty() {
            self.spinner = (self.spinner + 1) % SPINNER.len();
            self.dirty = true;
        }
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

    pub fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.poll_worker();
            if self.render_frame().contains(needle) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.poll_worker();
        self.render_frame().contains(needle)
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        for byte in bytes {
            if !self.handle_byte(*byte) {
                return false;
            }
        }
        true
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.poll_worker();
        if self.quit {
            return false;
        }

        if self.continue_escape_sequence(byte) {
            return !self.quit;
        }

        if byte == 27 {
            self.escape_sequence = Some(Vec::new());
            self.handle_input(TerminalAppInput::Key(UiKey::Esc));
            return !self.quit;
        }

        let Some(input) = TerminalAppInput::from_byte(byte) else {
            return true;
        };
        self.handle_input(input);
        !self.quit
    }

    pub fn render_frame(&self) -> String {
        let right_content = self.right_content();
        self.ui
            .render_screen_with_prompt(&right_content, self.prompt_buffer.as_str())
    }

    pub fn poll_worker(&mut self) {
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
                worker.handle.join().expect("terminal worker did not panic");
                self.dirty = true;
            } else {
                index += 1;
            }
        }
    }

    fn handle_input(&mut self, input: TerminalAppInput) {
        match input {
            TerminalAppInput::Quit => {
                self.request_quit();
            }
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.backspace();
                self.dirty = true;
            }
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.backspace();
                self.dirty = true;
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Prompt => {
                let line = self.prompt_buffer.trimmed_string();
                self.prompt_buffer.clear();
                self.ui.handle_key(UiKey::Esc);
                if !line.is_empty() {
                    self.command_transcript.push(format!("work-leaf> {line}"));
                    self.handle_command_line(&line);
                }
                self.dirty = true;
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Insert => {
                self.send_chat_buffer();
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.push(ch);
                self.dirty = true;
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.push(ch);
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
            TerminalAppInput::Backspace | TerminalAppInput::Enter => {}
        }
    }

    fn handle_command_line(&mut self, line: &str) {
        let parts = split_command_line(line);
        let Some(command) = parts.first().map(String::as_str) else {
            return;
        };

        match command {
            "quit" | "exit" => self.request_quit(),
            "new" => self.start_new_agent(parts[1..].to_vec()),
            "review" => self.start_review_workers(),
            _ => self.start_command_worker(line.to_string()),
        }
    }

    fn start_new_agent(&mut self, args: Vec<String>) {
        let Some(chat) = self.chat.as_mut() else {
            return;
        };
        let launch = match chat.prepare_agent_launch(&args) {
            Ok(launch) => {
                self.next_user_agent = chat.next_user_agent_index();
                launch
            }
            Err(error) => {
                self.command_transcript
                    .push(command_chat_error_text(&error));
                return;
            }
        };
        chat.register_agent_feature(launch.id.clone(), launch.feature.clone());

        self.add_launching_agent(&launch);
        self.start_launch_worker(launch);
    }

    fn display_feature_for_prompt(prompt: &str, fallback: &str) -> String {
        let title = session_title_from_prompt(prompt);
        if title.is_empty() {
            fallback.to_string()
        } else {
            title
        }
    }

    fn start_review_workers(&mut self) {
        let Some(chat) = self.chat.as_ref() else {
            return;
        };
        let commits = match crate::review::GitHistory::new(chat.project_dir().to_path_buf())
            .latest_agent_commits()
        {
            Ok(commits) => commits,
            Err(error) => {
                self.command_transcript.push(error.to_string());
                self.dirty = true;
                return;
            }
        };
        if commits.is_empty() {
            self.command_transcript
                .push("no agent commits found".to_string());
            self.dirty = true;
            return;
        }

        for (index, commit) in commits.into_iter().enumerate() {
            let reviewer_id = match AgentId::new(format!("review-{}", commit.agent_id.as_str())) {
                Ok(launch) => launch,
                Err(error) => {
                    self.command_transcript.push(error.to_string());
                    return;
                }
            };
            self.ui.add_agent(AgentListEntry::new(
                reviewer_id.clone(),
                format!("review {}", commit.feature),
            ));
            if index == 0 {
                let _ = self.ui.activate_agent_chat(&reviewer_id);
            }
            self.agent_chats
                .entry(reviewer_id.clone())
                .or_default()
                .loading = Some(LoadingKind::WaitingForReply);
            self.start_review_worker(commit, reviewer_id);
        }
        self.dirty = true;
    }

    fn add_launching_agent(&mut self, launch: &AgentLaunch) {
        self.ui.add_agent(AgentListEntry::new(
            launch.id.clone(),
            Self::display_feature_for_prompt(&launch.prompt, &launch.feature),
        ));
        let _ = self.ui.activate_agent_chat(&launch.id);
        self.agent_chats
            .entry(launch.id.clone())
            .or_default()
            .loading = Some(LoadingKind::Launching);
    }

    fn start_launch_worker(&mut self, launch: AgentLaunch) {
        let agent_id = launch.id.clone();
        self.agent_chats
            .entry(agent_id.clone())
            .or_default()
            .loading = Some(LoadingKind::Launching);
        self.start_worker(move |mut chat, sender| {
            let stream_sender = sender.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let _ = stream_sender.send(WorkerEvent::Stream {
                    agent_id: event_agent_id.clone(),
                    text: stream_event_text(event),
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

    fn start_review_worker(&mut self, commit: crate::review::AgentCommit, reviewer_id: AgentId) {
        self.start_worker(move |mut chat, sender| {
            let stream_sender = sender.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let _ = stream_sender.send(WorkerEvent::Stream {
                    agent_id: event_agent_id.clone(),
                    text: stream_event_text(event),
                });
            };
            match chat.review_commit_streaming_with_ids(commit, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(reviewer_id),
                        result: CommandChatResult::ReviewComplete(vec![result]),
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(reviewer_id),
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

    fn send_chat_buffer(&mut self) {
        let Some(agent_id) = self.ui.selected_agent().cloned() else {
            return;
        };
        if self
            .agent_chats
            .get(&agent_id)
            .and_then(|chat| chat.loading)
            .is_some()
        {
            self.agent_chats
                .entry(agent_id)
                .or_default()
                .lines
                .push("work-leaf: Codex is still working".to_string());
            self.dirty = true;
            return;
        }

        let message = self.chat_buffer.trimmed_string();
        self.chat_buffer.clear();
        if message.is_empty() {
            self.dirty = true;
            return;
        }

        self.agent_chats
            .entry(agent_id.clone())
            .or_default()
            .lines
            .push(format!("user: {message}"));
        self.agent_chats
            .entry(agent_id.clone())
            .or_default()
            .loading = Some(LoadingKind::WaitingForReply);

        self.start_worker(move |mut chat, sender| {
            let stream_sender = sender.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let _ = stream_sender.send(WorkerEvent::Stream {
                    agent_id: event_agent_id.clone(),
                    text: stream_event_text(event),
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
        self.dirty = true;
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
        self.dirty = true;
    }

    fn apply_worker_event(&mut self, event: WorkerEvent) {
        match event {
            WorkerEvent::Stream { agent_id, text } => {
                self.append_agent_line(&agent_id, text);
            }
            WorkerEvent::Complete { agent_id, result } => {
                if let Some(agent_id) = agent_id {
                    self.apply_agent_result(&agent_id, &result);
                    self.clear_agent_loading(&agent_id);
                } else {
                    self.command_transcript.push(command_result_text(&result));
                    if matches!(result, CommandChatResult::Quit) {
                        self.request_quit();
                    }
                }
            }
            WorkerEvent::Error { agent_id, message } => {
                if let Some(agent_id) = agent_id {
                    self.append_agent_line(&agent_id, message);
                    self.clear_agent_loading(&agent_id);
                } else {
                    self.command_transcript.push(message);
                }
            }
        }
        self.dirty = true;
    }

    fn apply_agent_result(&mut self, agent_id: &AgentId, result: &CommandChatResult) {
        match result {
            CommandChatResult::AgentLaunched { reply, .. }
            | CommandChatResult::AgentMessage { reply, .. } => {
                if !reply.is_empty() {
                    self.append_agent_line(agent_id, reply.clone());
                }
            }
            other => self.command_transcript.push(command_result_text(other)),
        }
    }

    fn append_agent_line(&mut self, agent_id: &AgentId, line: String) {
        if line.is_empty() {
            return;
        }
        let chat = self.agent_chats.entry(agent_id.clone()).or_default();
        if !chat.lines.iter().any(|existing| existing == &line) {
            chat.lines.push(line);
        }
    }

    fn clear_agent_loading(&mut self, agent_id: &AgentId) {
        if let Some(chat) = self.agent_chats.get_mut(agent_id) {
            chat.loading = None;
        }
    }

    fn record_actions(&mut self, actions: Vec<crate::UiAction>) {
        self.command_transcript
            .extend(actions.into_iter().map(ui_action_text));
    }

    fn request_quit(&mut self) {
        self.shutdown.shutdown();
        self.quit = true;
        self.dirty = true;
    }

    fn right_content(&self) -> String {
        if let Some(agent_id) = self.ui.selected_agent() {
            let chat = self.agent_chats.get(agent_id);
            let mut lines = chat.map(|chat| chat.lines.clone()).unwrap_or_default();
            if let Some(loading) = chat.and_then(|chat| chat.loading) {
                lines.push(format!(
                    "work-leaf: {} {}",
                    loading.as_str(),
                    SPINNER[self.spinner]
                ));
            }
            return terminal_right_content(self.chat_buffer.as_str(), &lines);
        }
        terminal_right_content("", &self.command_transcript)
    }

    fn continue_escape_sequence(&mut self, byte: u8) -> bool {
        let Some(sequence) = self.escape_sequence.as_mut() else {
            return false;
        };

        if sequence.is_empty() && byte != b'[' {
            self.escape_sequence = None;
            return false;
        }

        sequence.push(byte);
        if is_complete_control_sequence(sequence) {
            let complete = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            if let Some(key) = parse_control_sequence(&complete) {
                self.handle_input(TerminalAppInput::Key(key));
            }
        } else if sequence.len() > MAX_ESCAPE_SEQUENCE {
            self.escape_sequence = None;
        }

        true
    }
}

impl<B> Drop for TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    fn drop(&mut self) {
        if self.shutdown_on_drop {
            self.shutdown.shutdown();
        }
    }
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
    Complete {
        agent_id: Option<AgentId>,
        result: CommandChatResult,
    },
    Error {
        agent_id: Option<AgentId>,
        message: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LoadingKind {
    Launching,
    WaitingForReply,
}

impl LoadingKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Launching => "Starting Codex session",
            Self::WaitingForReply => "Waiting for Codex",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AgentChat {
    lines: Vec<String>,
    loading: Option<LoadingKind>,
}

const SPINNER: [&str; 4] = ["|", "/", "-", "\\"];

fn stream_event_text(event: AgentStreamEvent) -> String {
    match event {
        AgentStreamEvent::Status(text) => format!("codex: {text}"),
        AgentStreamEvent::AgentMessage(text) => text,
        AgentStreamEvent::Error(text) => format!("codex error: {text}"),
    }
}

fn split_command_line(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}

fn session_title_from_prompt(prompt: &str) -> String {
    const STOP_WORDS: &[&str] = &[
        "a",
        "an",
        "and",
        "for",
        "the",
        "to",
        "with",
        "please",
        "implement",
        "add",
        "fix",
        "update",
        "create",
        "build",
    ];

    let mut title = String::new();
    for word in prompt
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_')
                .to_ascii_lowercase()
        })
        .filter(|word| !word.is_empty())
        .filter(|word| !STOP_WORDS.contains(&word.as_str()))
    {
        let next_len = if title.is_empty() {
            word.len()
        } else {
            title.len() + 1 + word.len()
        };
        if next_len > 16 {
            continue;
        }
        if !title.is_empty() {
            title.push(' ');
        }
        title.push_str(&word);
        if !title.is_empty() {
            break;
        }
    }

    if title.is_empty() {
        prompt
            .split_whitespace()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        title
    }
}

const MAX_ESCAPE_SEQUENCE: usize = 64;

fn is_complete_control_sequence(sequence: &[u8]) -> bool {
    sequence.len() > 1
        && sequence
            .last()
            .is_some_and(|byte| (0x40..=0x7e).contains(byte))
}

fn parse_control_sequence(sequence: &[u8]) -> Option<UiKey> {
    parse_sgr_mouse_click(sequence)
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

    fn trimmed_string(&self) -> String {
        self.as_str().trim().to_string()
    }

    fn push(&mut self, ch: char) {
        let mut listener = NoopLineListener;
        let _ = self.buffer.insert(ch, 1, &mut listener);
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
