use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, CommandChat,
    MessageRole, TerminalApp,
};

fn main() -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        println!("Run this example in an interactive terminal: cargo run --example ui_harness");
        return Ok(());
    }

    let (width, height) = terminal_size();
    let _raw_mode = RawTerminalMode::enter();
    let mut stdout = io::stdout();
    let _screen_mode = AlternateScreenMode::enter(&mut stdout)?;
    let backend = HarnessBackend;
    let mut chat = CommandChat::new(PathBuf::from("."), backend);
    let mut app = TerminalApp::new(&mut chat, width, height);
    let mut stdin = io::stdin().lock();

    render_frame(&mut stdout, &app)?;

    loop {
        let mut byte = [0_u8; 1];
        if stdin.read(&mut byte)? == 0 {
            break;
        }
        if !app.handle_byte(byte[0]) {
            break;
        }
        render_frame(&mut stdout, &app)?;
    }

    write!(stdout, "\u{1b}[2J\u{1b}[H")?;
    stdout.flush()?;
    Ok(())
}

fn render_frame(output: &mut impl Write, app: &TerminalApp<'_, HarnessBackend>) -> io::Result<()> {
    write!(output, "{}", app.render_frame())?;
    output.flush()
}

#[derive(Debug, Default)]
struct HarnessBackend;

impl AgentBackend for HarnessBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let id = request.id.clone();
        let mut session = AgentSession::new(request);
        session.push_message(
            MessageRole::Agent,
            format!("{id} ready; send a chat message to test the reply path"),
        );
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(
            MessageRole::Agent,
            format!("{agent_id} received: {prompt}"),
        ))
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
