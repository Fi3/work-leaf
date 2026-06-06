use crate::{AgentId, AgentListEntry, TerminalUi, UiKey, UiMode};

#[derive(Debug)]
pub struct UiHarness {
    ui: TerminalUi,
    prompt_buffer: String,
    chat_buffer: String,
    chat_cursor: usize,
    chat_history: Vec<String>,
    chat_history_index: Option<usize>,
    transcript: Vec<String>,
    next_agent: usize,
    quit: bool,
}

impl UiHarness {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            ui: fixture_ui(width, height),
            prompt_buffer: String::new(),
            chat_buffer: String::new(),
            chat_cursor: 0,
            chat_history: Vec::new(),
            chat_history_index: None,
            transcript: vec![
                "UI harness".to_string(),
                "Esc command, i insert, : prompt, Ctrl-W h/j/k/l focus, , toggle right, q quit"
                    .to_string(),
            ],
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
        true
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        if self.quit {
            return false;
        }

        let Some(input) = HarnessInput::from_byte(byte) else {
            return true;
        };
        self.handle_input(input);
        !self.quit
    }

    pub fn render_frame(&self) -> String {
        self.ui
            .render_screen_with_prompt(&self.right_content(), &self.prompt_buffer)
    }

    fn handle_input(&mut self, input: HarnessInput) {
        match input {
            HarnessInput::Quit => self.quit = true,
            HarnessInput::Backspace if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.pop();
            }
            HarnessInput::Backspace if self.ui.mode() == UiMode::Insert => {
                self.backspace_chat_char();
            }
            HarnessInput::Enter if self.ui.mode() == UiMode::Prompt => {
                let line = self.prompt_buffer.trim().to_string();
                self.prompt_buffer.clear();
                self.ui.handle_key(UiKey::Esc);
                if !line.is_empty() {
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
                    let target = self
                        .ui
                        .selected_agent()
                        .map(AgentId::as_str)
                        .unwrap_or("work-leaf");
                    self.transcript.push(format!("{target}> {message}"));
                    self.transcript
                        .push("fixture reply: message recorded".to_string());
                }
            }
            HarnessInput::Char(ch) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.push(ch);
            }
            HarnessInput::Char(ch) if self.ui.mode() == UiMode::Insert => {
                self.insert_chat_char(ch);
            }
            HarnessInput::Key(UiKey::Left) if self.ui.mode() == UiMode::Insert => {
                self.move_chat_cursor_left();
            }
            HarnessInput::Key(UiKey::Right) if self.ui.mode() == UiMode::Insert => {
                self.move_chat_cursor_right();
            }
            HarnessInput::Key(UiKey::Up) if self.ui.mode() == UiMode::Insert => {
                self.recall_chat_history(-1);
            }
            HarnessInput::Key(UiKey::Down) if self.ui.mode() == UiMode::Insert => {
                self.recall_chat_history(1);
            }
            HarnessInput::Key(UiKey::Esc) => {
                self.prompt_buffer.clear();
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

    fn record_actions(&mut self, actions: Vec<crate::UiAction>) {
        self.transcript
            .extend(actions.into_iter().map(|action| format!("{action:?}")));
    }

    fn right_content(&self) -> String {
        let mut content = self.transcript.join("\n");
        if !content.is_empty() {
            content.push('\n');
        }
        content.push_str("chat> ");
        content.push_str(&self.chat_buffer);
        content
    }
}

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

fn fixture_ui(width: u16, height: u16) -> TerminalUi {
    let parser = AgentId::new("user-1").expect("fixture agent id is valid");
    let tests = AgentId::new("user-2").expect("fixture agent id is valid");
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
