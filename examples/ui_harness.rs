use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process::{Command, Stdio};

use work_leaf::{AgentId, AgentListEntry, TerminalUi, UiKey, UiMode};

fn main() -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        println!("Run this example in an interactive terminal: cargo run --example ui_harness");
        return Ok(());
    }

    let (width, height) = terminal_size();
    let _raw_mode = RawTerminalMode::enter();
    let mut stdout = io::stdout();
    let _screen_mode = AlternateScreenMode::enter(&mut stdout)?;
    let mut ui = fixture_ui(width, height);
    let mut prompt_buffer = String::new();
    let mut chat_buffer = String::new();
    let mut transcript = vec![
        "UI harness".to_string(),
        "Esc command, i insert, : prompt, Ctrl-W h/j/k/l focus, , toggle right, q quit".to_string(),
    ];
    let mut next_agent = 3;
    let mut stdin = io::stdin().lock();

    render_frame(&mut stdout, &ui, &prompt_buffer, &chat_buffer, &transcript)?;

    loop {
        let mut byte = [0_u8; 1];
        if stdin.read(&mut byte)? == 0 {
            break;
        }
        let Some(input) = Input::from_byte(byte[0]) else {
            render_frame(&mut stdout, &ui, &prompt_buffer, &chat_buffer, &transcript)?;
            continue;
        };

        match input {
            Input::Quit => break,
            Input::Backspace if ui.mode() == UiMode::Prompt => {
                prompt_buffer.pop();
            }
            Input::Backspace if ui.mode() == UiMode::Insert => {
                chat_buffer.pop();
            }
            Input::Enter if ui.mode() == UiMode::Prompt => {
                let line = prompt_buffer.trim().to_string();
                prompt_buffer.clear();
                ui.handle_key(UiKey::Esc);
                if !line.is_empty() {
                    transcript.push(format!("work-leaf> {line}"));
                    if !execute_prompt(&mut ui, &mut transcript, &mut next_agent, &line) {
                        break;
                    }
                }
            }
            Input::Enter if ui.mode() == UiMode::Insert => {
                let message = chat_buffer.trim().to_string();
                chat_buffer.clear();
                if !message.is_empty() {
                    let target = ui
                        .selected_agent()
                        .map(AgentId::as_str)
                        .unwrap_or("work-leaf");
                    transcript.push(format!("{target}> {message}"));
                    transcript.push("fixture reply: message recorded".to_string());
                }
            }
            Input::Char(ch) if ui.mode() == UiMode::Prompt => {
                prompt_buffer.push(ch);
            }
            Input::Char(ch) if ui.mode() == UiMode::Insert => {
                chat_buffer.push(ch);
            }
            Input::Key(UiKey::Esc) => {
                prompt_buffer.clear();
                ui.handle_key(UiKey::Esc);
            }
            Input::Char('q') if ui.mode() == UiMode::Command => break,
            Input::Key(key) => {
                for action in ui.handle_key(key) {
                    transcript.push(format!("{action:?}"));
                }
            }
            Input::Char(ch) => {
                for action in ui.handle_key(UiKey::Char(ch)) {
                    transcript.push(format!("{action:?}"));
                }
            }
            Input::Backspace | Input::Enter => {}
        }

        render_frame(&mut stdout, &ui, &prompt_buffer, &chat_buffer, &transcript)?;
    }

    write!(stdout, "\u{1b}[2J\u{1b}[H")?;
    stdout.flush()?;
    Ok(())
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

fn execute_prompt(
    ui: &mut TerminalUi,
    transcript: &mut Vec<String>,
    next_agent: &mut usize,
    line: &str,
) -> bool {
    if matches!(line, "quit" | "exit" | "q") {
        return false;
    }

    let new_prompt = if line == "new" {
        Some("interactive task discovery")
    } else {
        line.strip_prefix("new ")
    };

    if let Some(prompt) = new_prompt {
        let agent_id =
            AgentId::new(format!("user-{next_agent}")).expect("generated fixture id is valid");
        *next_agent += 1;
        ui.add_agent(AgentListEntry::new(agent_id.clone(), "harness-agent"));
        ui.activate_agent_chat(&agent_id)
            .expect("generated fixture agent is registered");
        transcript.push(format!("agent {agent_id} launched for: {prompt}"));
        return true;
    }

    match line {
        "help" | "?" => {
            transcript.push("commands: new [prompt...], review, linearize, quit".into());
        }
        "review" => transcript.push("fixture review: no findings".into()),
        "linearize" => transcript.push("fixture linearize: keep user-1, keep user-2".into()),
        other => transcript.push(format!("unknown fixture command: {other}")),
    }
    true
}

fn render_frame(
    output: &mut impl Write,
    ui: &TerminalUi,
    prompt_buffer: &str,
    chat_buffer: &str,
    transcript: &[String],
) -> io::Result<()> {
    let mut right_content = transcript.join("\n");
    if !right_content.is_empty() {
        right_content.push('\n');
    }
    right_content.push_str("chat> ");
    right_content.push_str(chat_buffer);
    write!(
        output,
        "{}",
        ui.render_screen_with_prompt(&right_content, prompt_buffer)
    )?;
    output.flush()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Input {
    Key(UiKey),
    Char(char),
    Enter,
    Backspace,
    Quit,
}

impl Input {
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

struct RawTerminalMode {
    saved_state: Option<String>,
}

impl RawTerminalMode {
    fn enter() -> Self {
        let saved_state = stty_output(&["-g"]);
        if saved_state.is_some() {
            let _ = stty_status(&["raw", "-echo", "min", "1", "time", "0"]);
        }
        Self { saved_state }
    }
}

impl Drop for RawTerminalMode {
    fn drop(&mut self) {
        if let Some(saved_state) = &self.saved_state {
            let _ = stty_status(&[saved_state.as_str()]);
        }
    }
}

struct AlternateScreenMode;

impl AlternateScreenMode {
    fn enter(output: &mut impl Write) -> io::Result<Self> {
        write!(output, "\u{1b}[?1049h\u{1b}[2J\u{1b}[H")?;
        output.flush()?;
        Ok(Self)
    }
}

impl Drop for AlternateScreenMode {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = write!(stdout, "\u{1b}[?1049l\u{1b}[?25h");
        let _ = stdout.flush();
    }
}

fn terminal_size() -> (u16, u16) {
    if let Some(size) = terminal_size_from_stty() {
        return size;
    }
    let width = env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(100);
    let height = env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(30);
    (width.max(20), height.max(5))
}

fn terminal_size_from_stty() -> Option<(u16, u16)> {
    let text = stty_output(&["size"])?;
    let mut parts = text.split_whitespace();
    let rows = parts.next()?.parse::<u16>().ok()?;
    let columns = parts.next()?.parse::<u16>().ok()?;
    Some((columns.max(20), rows.max(5)))
}

fn stty_output(args: &[&str]) -> Option<String> {
    let output = Command::new("stty")
        .args(args)
        .stdin(Stdio::inherit())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn stty_status(args: &[&str]) -> Option<()> {
    let status = Command::new("stty")
        .args(args)
        .stdin(Stdio::inherit())
        .status()
        .ok()?;
    status.success().then_some(())
}
