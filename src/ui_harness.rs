use crate::{
    AgentId, AgentListEntry, PaneFocus, TerminalUi, UiKey, UiMode, chat_title::ChatTitleAgent,
};

#[derive(Debug)]
pub struct UiHarness {
    ui: TerminalUi,
    prompt_buffer: String,
    prompt_cursor: usize,
    prompt_history: Vec<String>,
    prompt_history_index: Option<usize>,
    chat_buffer: String,
    chat_cursor: usize,
    chat_history: Vec<String>,
    chat_history_index: Option<usize>,
    escape_sequence: Option<PendingEscapeSequence>,
    transcript: Vec<String>,
    chat_title_agent: ChatTitleAgent,
    next_agent: usize,
    quit: bool,
}

impl UiHarness {
    pub fn new(width: u16, height: u16) -> Self {
        let parser = AgentId::new("user-1").expect("fixture agent id is valid");
        let tests = AgentId::new("user-2").expect("fixture agent id is valid");
        let mut chat_title_agent = ChatTitleAgent::new();
        chat_title_agent.mark_named(&parser);
        chat_title_agent.mark_named(&tests);

        Self {
            ui: fixture_ui(width, height, parser, tests),
            prompt_buffer: String::new(),
            prompt_cursor: 0,
            prompt_history: Vec::new(),
            prompt_history_index: None,
            chat_buffer: String::new(),
            chat_cursor: 0,
            chat_history: Vec::new(),
            chat_history_index: None,
            escape_sequence: None,
            transcript: vec![
                "UI harness".to_string(),
                "Esc command, i insert, : prompt, Ctrl-W h/j/k/l focus, , toggle right, q quit"
                    .to_string(),
            ],
            chat_title_agent,
            next_agent: 3,
            quit: false,
        }
    }

    pub fn ui(&self) -> &TerminalUi {
        &self.ui
    }

    pub fn transcript(&self) -> &[String] {
        &self.transcript
    }

    pub fn mark_agent_ready(&mut self, agent_id: &str) -> Result<(), String> {
        let agent_id = AgentId::new(agent_id).map_err(|error| error.to_string())?;
        self.ui.set_agent_ready(&agent_id, true)
    }

    pub fn is_quit(&self) -> bool {
        self.quit
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        let mut index = 0;
        while index < bytes.len() {
            if let Some((key, len)) = parse_key_sequence(&bytes[index..]) {
                self.handle_input(HarnessInput::Key(key));
                index += len;
            } else if !self.handle_byte(bytes[index]) {
                return false;
            } else {
                index += 1;
            }
        }
        self.finish_pending_escape_sequence();
        !self.quit
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        if self.quit {
            return false;
        }

        if self.continue_escape_sequence(byte) {
            return !self.quit;
        }

        if byte == 27 {
            let defer_escape = self.defer_escape_key();
            self.escape_sequence = Some(PendingEscapeSequence {
                bytes: vec![27],
                mode_before: self.ui.mode(),
                escape_dispatched: !defer_escape,
            });
            if !defer_escape {
                self.handle_input(HarnessInput::Key(UiKey::Esc));
            }
            return !self.quit;
        }

        let Some(input) = HarnessInput::from_byte(byte) else {
            return true;
        };
        self.handle_input(input);
        !self.quit
    }

    pub fn render_frame(&self) -> String {
        self.ui.render_screen_with_cursors(
            &self.right_content(),
            &self.prompt_buffer,
            self.prompt_cursor,
            Some(self.chat_cursor_column()),
        )
    }

    fn handle_input(&mut self, input: HarnessInput) {
        match input {
            HarnessInput::Quit => self.quit = true,
            HarnessInput::Backspace if self.ui.mode() == UiMode::Prompt => {
                self.backspace_prompt_char();
            }
            HarnessInput::Backspace if self.ui.mode() == UiMode::Insert => {
                self.backspace_chat_char();
            }
            HarnessInput::Enter if self.ui.mode() == UiMode::Prompt => {
                let line = self.prompt_buffer.trim().to_string();
                self.prompt_buffer.clear();
                self.prompt_cursor = 0;
                self.prompt_history_index = None;
                self.ui.handle_key(UiKey::Esc);
                if !line.is_empty() {
                    self.prompt_history.push(line.clone());
                    self.transcript.push(format!("work-leaf> {line}"));
                    self.execute_prompt(&line);
                }
            }
            HarnessInput::Enter if self.ui.mode() == UiMode::Insert => {
                let message = self.chat_buffer.trim().to_string();
                self.chat_buffer.clear();
                self.chat_cursor = 0;
                self.chat_history_index = None;
                if !message.is_empty() {
                    self.chat_history.push(message.clone());
                    let target_agent = self.ui.selected_agent().cloned();
                    if let Some(agent_id) = target_agent.as_ref() {
                        self.name_chat_from_first_prompt(agent_id, &message);
                    }
                    let target = target_agent
                        .as_ref()
                        .map(AgentId::as_str)
                        .unwrap_or("work-leaf");
                    self.transcript.push(format!("{target}> {message}"));
                    self.transcript
                        .push("fixture reply: message recorded".to_string());
                }
            }
            HarnessInput::Char(ch) if self.ui.mode() == UiMode::Prompt => {
                self.insert_prompt_char(ch);
            }
            HarnessInput::Char(ch) if self.ui.mode() == UiMode::Insert => {
                self.insert_chat_char(ch);
            }
            HarnessInput::Key(UiKey::Left) if self.ui.mode() == UiMode::Prompt => {
                self.move_prompt_cursor_left();
            }
            HarnessInput::Key(UiKey::Right) if self.ui.mode() == UiMode::Prompt => {
                self.move_prompt_cursor_right();
            }
            HarnessInput::Key(UiKey::Up) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(-1);
            }
            HarnessInput::Key(UiKey::Down) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(1);
            }
            HarnessInput::Key(UiKey::Left) if self.should_route_chat_arrow() => {
                self.move_chat_cursor_left();
            }
            HarnessInput::Key(UiKey::Right) if self.should_route_chat_arrow() => {
                self.move_chat_cursor_right();
            }
            HarnessInput::Key(UiKey::Up) if self.should_route_chat_arrow() => {
                self.recall_chat_history(-1);
            }
            HarnessInput::Key(UiKey::Down) if self.should_route_chat_arrow() => {
                self.recall_chat_history(1);
            }
            HarnessInput::Key(UiKey::Esc) => {
                self.prompt_buffer.clear();
                self.prompt_cursor = 0;
                self.prompt_history_index = None;
                let actions = self.ui.handle_key(UiKey::Esc);
                self.record_actions(actions);
            }
            HarnessInput::Char('q') if self.ui.mode() == UiMode::Command => {
                self.quit = true;
            }
            HarnessInput::Key(key) => {
                let actions = self.ui.handle_key(key);
                self.record_actions(actions);
            }
            HarnessInput::Char(ch) => {
                let actions = self.ui.handle_key(UiKey::Char(ch));
                self.record_actions(actions);
            }
            HarnessInput::Backspace | HarnessInput::Enter => {}
        }
    }

    fn execute_prompt(&mut self, line: &str) {
        if matches!(line, "quit" | "exit" | "q") {
            self.quit = true;
            return;
        }

        let new_prompt = if line == "new" {
            Some("interactive task discovery")
        } else {
            line.strip_prefix("new ")
        };

        if let Some(prompt) = new_prompt {
            let agent_id = AgentId::new(format!("user-{}", self.next_agent))
                .expect("generated fixture id is valid");
            self.next_agent += 1;
            self.ui
                .add_agent(AgentListEntry::new(agent_id.clone(), "harness-agent"));
            self.ui
                .activate_agent_chat(&agent_id)
                .expect("generated fixture agent is registered");
            self.transcript
                .push(format!("agent {agent_id} launched for: {prompt}"));
            return;
        }

        match line {
            "help" | "?" => {
                self.transcript
                    .push("commands: new [prompt...], review, linearize, quit".into());
            }
            "review" => self.transcript.push("fixture review: no findings".into()),
            "linearize" => self
                .transcript
                .push("fixture linearize: keep user-1, keep user-2".into()),
            other => self
                .transcript
                .push(format!("unknown fixture command: {other}")),
        }
    }

    fn name_chat_from_first_prompt(&mut self, agent_id: &AgentId, prompt: &str) {
        let Some(title) = self
            .chat_title_agent
            .title_for_first_prompt(agent_id, prompt)
        else {
            return;
        };
        let _ = self.ui.update_agent_feature(agent_id, title);
    }

    fn insert_prompt_char(&mut self, ch: char) {
        self.prompt_buffer.insert(self.prompt_cursor, ch);
        self.prompt_cursor += ch.len_utf8();
        self.prompt_history_index = None;
    }
    fn backspace_prompt_char(&mut self) {
        let Some((previous, _)) = self.prompt_buffer[..self.prompt_cursor]
            .char_indices()
            .next_back()
        else {
            return;
        };
        self.prompt_buffer.drain(previous..self.prompt_cursor);
        self.prompt_cursor = previous;
        self.prompt_history_index = None;
    }
    fn move_prompt_cursor_left(&mut self) {
        if let Some((previous, _)) = self.prompt_buffer[..self.prompt_cursor]
            .char_indices()
            .next_back()
        {
            self.prompt_cursor = previous;
        }
    }
    fn move_prompt_cursor_right(&mut self) {
        if self.prompt_cursor >= self.prompt_buffer.len() {
            return;
        }
        let next = self.prompt_buffer[self.prompt_cursor..]
            .chars()
            .next()
            .map(|ch| self.prompt_cursor + ch.len_utf8())
            .unwrap_or(self.prompt_buffer.len());
        self.prompt_cursor = next;
    }
    fn recall_prompt_history(&mut self, delta: isize) {
        if self.prompt_history.is_empty() {
            return;
        }
        let current = self
            .prompt_history_index
            .unwrap_or(self.prompt_history.len()) as isize;
        let max = self.prompt_history.len().saturating_sub(1) as isize;
        let next = (current + delta).clamp(0, max) as usize;
        self.prompt_history_index = Some(next);
        self.prompt_buffer = self.prompt_history[next].clone();
        self.prompt_cursor = self.prompt_buffer.len();
    }
    fn insert_chat_char(&mut self, ch: char) {
        self.chat_buffer.insert(self.chat_cursor, ch);
        self.chat_cursor += ch.len_utf8();
        self.chat_history_index = None;
    }

    fn backspace_chat_char(&mut self) {
        let Some((previous, _)) = self.chat_buffer[..self.chat_cursor]
            .char_indices()
            .next_back()
        else {
            return;
        };
        self.chat_buffer.drain(previous..self.chat_cursor);
        self.chat_cursor = previous;
        self.chat_history_index = None;
    }

    fn move_chat_cursor_left(&mut self) {
        if let Some((previous, _)) = self.chat_buffer[..self.chat_cursor]
            .char_indices()
            .next_back()
        {
            self.chat_cursor = previous;
        }
    }

    fn move_chat_cursor_right(&mut self) {
        if self.chat_cursor >= self.chat_buffer.len() {
            return;
        }
        let next = self.chat_buffer[self.chat_cursor..]
            .chars()
            .next()
            .map(|ch| self.chat_cursor + ch.len_utf8())
            .unwrap_or(self.chat_buffer.len());
        self.chat_cursor = next;
    }

    fn recall_chat_history(&mut self, delta: isize) {
        if self.chat_history.is_empty() {
            return;
        }

        let current = self.chat_history_index.unwrap_or(self.chat_history.len()) as isize;
        let max = self.chat_history.len().saturating_sub(1) as isize;
        let next = (current + delta).clamp(0, max) as usize;
        self.chat_history_index = Some(next);
        self.chat_buffer = self.chat_history[next].clone();
        self.chat_cursor = self.chat_buffer.len();
    }

    fn chat_cursor_column(&self) -> usize {
        CHAT_PROMPT.chars().count() + cursor_char_count(&self.chat_buffer, self.chat_cursor)
    }
    fn record_actions(&mut self, actions: Vec<crate::UiAction>) {
        self.transcript
            .extend(actions.into_iter().map(|action| format!("{action:?}")));
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
            .is_some_and(|sequence| sequence.bytes.len() == 1);
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
            self.handle_input(HarnessInput::Key(UiKey::Esc));
        }
    }

    fn continue_escape_sequence(&mut self, byte: u8) -> bool {
        let Some(sequence) = self.escape_sequence.as_mut() else {
            return false;
        };

        if sequence.bytes.len() == 1 && byte != b'[' {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
            return false;
        }

        sequence.bytes.push(byte);
        if let Some((key, len)) = parse_key_sequence(&sequence.bytes) {
            if len == sequence.bytes.len() {
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
                self.handle_input(HarnessInput::Key(key));
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

    fn right_content(&self) -> String {
        let mut content = self.transcript.join("\n");
        if !content.is_empty() {
            content.push('\n');
        }
        content.push_str(CHAT_PROMPT);
        content.push_str(&self.chat_buffer);
        content
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingEscapeSequence {
    bytes: Vec<u8>,
    mode_before: UiMode,
    escape_dispatched: bool,
}

const CHAT_PROMPT: &str = "chat> ";
const MAX_ESCAPE_SEQUENCE: usize = 8;

fn parse_key_sequence(bytes: &[u8]) -> Option<(UiKey, usize)> {
    match bytes {
        [27, b'[', b'A', ..] => Some((UiKey::Up, 3)),
        [27, b'[', b'B', ..] => Some((UiKey::Down, 3)),
        [27, b'[', b'C', ..] => Some((UiKey::Right, 3)),
        [27, b'[', b'D', ..] => Some((UiKey::Left, 3)),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HarnessInput {
    Key(UiKey),
    Char(char),
    Enter,
    Backspace,
    Quit,
}

impl HarnessInput {
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

fn cursor_char_count(text: &str, cursor: usize) -> usize {
    text.char_indices()
        .take_while(|(index, _)| *index < cursor)
        .count()
}
fn fixture_ui(width: u16, height: u16, parser: AgentId, tests: AgentId) -> TerminalUi {
    let mut ui = TerminalUi::new(width, height);
    ui.add_agent(
        AgentListEntry::new(parser.clone(), "parser")
            .with_ready(true)
            .with_modified_file("src/parser.rs")
            .with_conflicting_agent(tests.clone())
            .with_dependent(tests.clone()),
    );
    ui.add_agent(
        AgentListEntry::new(tests.clone(), "tests")
            .with_modified_file("tests/parser.rs")
            .with_dependency(parser.clone()),
    );
    ui.select_agent(&parser)
        .expect("fixture parser agent is registered");
    ui
}
