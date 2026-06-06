use crate::cli::{
    CommandChat, CommandChatResult, apply_command_result_to_ui, command_chat_error_text,
    command_result_text, terminal_right_content, ui_action_text,
};
use crate::codex::AgentBackend;
use crate::ui::{TerminalUi, UiKey, UiMode};

#[derive(Debug)]
pub struct TerminalApp<'a, B>
where
    B: AgentBackend,
{
    chat: &'a mut CommandChat<B>,
    ui: TerminalUi,
    prompt_buffer: String,
    chat_buffer: String,
    transcript: Vec<String>,
    quit: bool,
}

impl<'a, B> TerminalApp<'a, B>
where
    B: AgentBackend,
{
    pub fn new(chat: &'a mut CommandChat<B>, width: u16, height: u16) -> Self {
        Self {
            chat,
            ui: TerminalUi::new(width, height),
            prompt_buffer: String::new(),
            chat_buffer: String::new(),
            transcript: vec![crate::cli::render_command_chat_help()],
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
        for byte in bytes {
            if !self.handle_byte(*byte) {
                return false;
            }
        }
        true
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        if self.quit {
            return false;
        }

        let Some(input) = TerminalAppInput::from_byte(byte) else {
            return true;
        };
        self.handle_input(input);
        !self.quit
    }

    pub fn render_frame(&self) -> String {
        let right_content = terminal_right_content(&self.chat_buffer, &self.transcript);
        self.ui
            .render_screen_with_prompt(&right_content, &self.prompt_buffer)
    }

    fn handle_input(&mut self, input: TerminalAppInput) {
        match input {
            TerminalAppInput::Quit => self.quit = true,
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.pop();
            }
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.pop();
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Prompt => {
                let line = self.prompt_buffer.trim().to_string();
                self.prompt_buffer.clear();
                self.ui.handle_key(UiKey::Esc);
                if !line.is_empty() {
                    self.transcript.push(format!("work-leaf> {line}"));
                    self.handle_command_line(&line);
                }
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Insert => {
                if let Some(agent_id) = self.ui.selected_agent().cloned() {
                    let message = self.chat_buffer.trim().to_string();
                    self.chat_buffer.clear();
                    if !message.is_empty() {
                        self.transcript.push(format!("{agent_id}> {message}"));
                        match self.chat.send_to_agent(&agent_id, &message) {
                            Ok(result) => self.transcript.push(command_result_text(&result)),
                            Err(error) => self.transcript.push(command_chat_error_text(&error)),
                        }
                    }
                }
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.push(ch);
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.push(ch);
            }
            TerminalAppInput::Key(UiKey::Esc) => {
                self.prompt_buffer.clear();
                let actions = self.ui.handle_key(UiKey::Esc);
                self.record_actions(actions);
            }
            TerminalAppInput::Key(key) => {
                let actions = self.ui.handle_key(key);
                self.record_actions(actions);
            }
            TerminalAppInput::Char(ch) => {
                let actions = self.ui.handle_key(UiKey::Char(ch));
                self.record_actions(actions);
            }
            TerminalAppInput::Backspace | TerminalAppInput::Enter => {}
        }
    }

    fn handle_command_line(&mut self, line: &str) {
        match self.chat.handle_line(line) {
            Ok(result) => {
                let should_quit = matches!(result, CommandChatResult::Quit);
                apply_command_result_to_ui(&mut self.ui, &result);
                self.transcript.push(command_result_text(&result));
                if should_quit {
                    self.quit = true;
                }
            }
            Err(error) => self.transcript.push(command_chat_error_text(&error)),
        }
    }

    fn record_actions(&mut self, actions: Vec<crate::UiAction>) {
        self.transcript
            .extend(actions.into_iter().map(ui_action_text));
    }
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
