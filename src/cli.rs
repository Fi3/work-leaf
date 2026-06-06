use std::collections::{BTreeMap, VecDeque};
use std::env;
use std::fmt;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::agent::{AgentId, AgentKind, AgentLaunch, PromptPolicy};
use crate::codex::{AgentBackend, AgentStreamEvent, CodexBackend, CodexCommandConfig};
use crate::linearize::{LinearizePlanner, LinearizeQuestion};
use crate::locks::{CommandWritePolicy, FileLockTable};
use crate::orchestrator::{AgentFollowUp, OrchestratorEvent, handle_agent_directives_streaming};
use crate::review::{GitHistory, ReviewCoordinator, ReviewResult};
use crate::terminal_app::TerminalApp;
use crate::ui::UiAction;

const DEFAULT_NEW_AGENT_PROMPT: &str = "Start a new work-leaf user-agent session. Ask the user what to work on if the task is not already clear, then report the broad feature before proposing patches.";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessCommand {
    Help,
    Launch { model: Option<String> },
}

pub fn parse_process_args<I, S>(args: I) -> Result<ProcessCommand, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg.ends_with("work-leaf")) {
        args.remove(0);
    }

    if args.is_empty() {
        return Ok(ProcessCommand::Launch { model: None });
    }

    let mut model = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" | "help" => return Ok(ProcessCommand::Help),
            "--model" => {
                if index + 1 >= args.len() {
                    return Err(CliError::Usage("--model requires a value".to_string()));
                }
                model = Some(args[index + 1].clone());
                index += 2;
            }
            "new" | "patch" | "review" | "linearize" | "linearize-questions" | "locks" => {
                return Err(CliError::Usage(
                    "work-leaf does not accept top-level workflow commands; start work-leaf and use the command chat".to_string(),
                ));
            }
            other => return Err(CliError::Usage(format!("unknown option `{other}`"))),
        }
    }

    Ok(ProcessCommand::Launch { model })
}

pub fn run_cli_from_env() -> ! {
    let command = match parse_process_args(env::args()) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}");
            process::exit(2);
        }
    };

    match command {
        ProcessCommand::Help => {
            print!("{}", render_process_help());
            process::exit(0);
        }
        ProcessCommand::Launch { model } => {
            let project_dir = match env::current_dir() {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("{error}");
                    process::exit(1);
                }
            };
            let backend = codex_backend(project_dir.clone(), model);
            let chat = CommandChat::new(project_dir, backend);
            if let Err(error) = run_command_chat(chat) {
                eprintln!("{error}");
                process::exit(1);
            }
            process::exit(0);
        }
    }
}

#[derive(Debug)]
pub struct CommandChat<B> {
    project_dir: PathBuf,
    backend: Option<B>,
    locks: FileLockTable,
    command_policy: CommandWritePolicy,
    agents: BTreeMap<AgentId, String>,
    max_review_rounds: usize,
    next_user_agent: usize,
}

impl<B> CommandChat<B>
where
    B: AgentBackend,
{
    pub fn new(project_dir: PathBuf, backend: B) -> Self {
        Self {
            locks: FileLockTable::new(project_dir.clone()),
            project_dir,
            backend: Some(backend),
            command_policy: CommandWritePolicy,
            agents: BTreeMap::new(),
            max_review_rounds: 8,
            next_user_agent: 1,
        }
    }

    pub fn with_max_review_rounds(mut self, max_review_rounds: usize) -> Self {
        self.max_review_rounds = max_review_rounds.max(1);
        self
    }

    pub fn into_backend(self) -> B {
        self.backend.expect("command chat backend is present")
    }

    pub fn handle_line(&mut self, line: &str) -> Result<CommandChatResult, CliError> {
        let parts = split_command_line(line);
        let Some(command) = parts.first().map(String::as_str) else {
            return Ok(CommandChatResult::Noop);
        };

        match command {
            "help" | "?" => Ok(CommandChatResult::Help(render_command_chat_help())),
            "quit" | "exit" => Ok(CommandChatResult::Quit),
            "new" => self.launch_agent(&parts[1..]),
            "review" => self.review(),
            "linearize" => self.linearize_questions(),
            "patch" | "locks" => Err(CliError::Usage(format!(
                "`{command}` is automatic orchestrator machinery, not a command chat command"
            ))),
            other => Err(CliError::Usage(format!(
                "unknown command chat command `{other}`"
            ))),
        }
    }

    pub fn send_to_agent(
        &mut self,
        agent_id: &AgentId,
        message: &str,
    ) -> Result<CommandChatResult, CliError> {
        self.send_to_agent_streaming(agent_id, message, &mut |_| {})
    }

    pub fn send_to_agent_streaming(
        &mut self,
        agent_id: &AgentId,
        message: &str,
        stream: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<CommandChatResult, CliError> {
        let mut stream_with_agent = |_: &AgentId, event| stream(event);
        self.send_to_agent_streaming_with_ids(agent_id, message, &mut stream_with_agent)
    }

    pub fn send_to_agent_streaming_with_ids(
        &mut self,
        agent_id: &AgentId,
        message: &str,
        stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    ) -> Result<CommandChatResult, CliError> {
        let feature = self
            .agents
            .get(agent_id)
            .cloned()
            .unwrap_or_else(|| "user-agent".to_string());
        let mut send_stream = |event| stream(agent_id, event);
        let reply = self
            .backend
            .as_mut()
            .expect("command chat backend is present")
            .send_streaming(agent_id, message, &mut send_stream)
            .map_err(CliError::Agent)?
            .text;
        let reply = self.process_agent_reply_streaming(agent_id, &feature, reply, stream)?;
        Ok(CommandChatResult::AgentMessage {
            agent_id: agent_id.clone(),
            reply,
        })
    }

    fn launch_agent(&mut self, args: &[String]) -> Result<CommandChatResult, CliError> {
        let original_next_user_agent = self.next_user_agent;
        let launch = self.prepare_agent_launch(args)?;
        match self.launch_prepared_agent_streaming(launch, &mut |_| {}) {
            Ok(result) => Ok(result),
            Err(error) => {
                self.next_user_agent = original_next_user_agent;
                Err(error)
            }
        }
    }

    pub fn prepare_agent_launch(&mut self, args: &[String]) -> Result<AgentLaunch, CliError> {
        let agent_id =
            AgentId::new(format!("user-{}", self.next_user_agent)).map_err(CliError::Agent)?;
        self.next_user_agent += 1;
        let feature = "user-agent".to_string();
        let prompt = if args.is_empty() {
            DEFAULT_NEW_AGENT_PROMPT.to_string()
        } else {
            args.join(" ")
        };
        Ok(AgentLaunch::new(
            agent_id,
            AgentKind::Codex,
            feature,
            prompt,
        ))
    }

    pub fn launch_prepared_agent_streaming(
        &mut self,
        launch: AgentLaunch,
        stream: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<CommandChatResult, CliError> {
        let mut stream_with_agent = |_: &AgentId, event| stream(event);
        self.launch_prepared_agent_streaming_with_ids(launch, &mut stream_with_agent)
    }

    pub fn launch_prepared_agent_streaming_with_ids(
        &mut self,
        launch: AgentLaunch,
        stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    ) -> Result<CommandChatResult, CliError> {
        let agent_id = launch.id.clone();
        let feature = launch.feature.clone();
        let mut launch_stream = |event| stream(&agent_id, event);
        let session = self
            .backend
            .as_mut()
            .expect("command chat backend is present")
            .launch_streaming(launch, &mut launch_stream)
            .map_err(CliError::Agent)?;
        let reply = session
            .messages
            .last()
            .map(|message| message.text.clone())
            .unwrap_or_default();
        self.agents.insert(agent_id.clone(), feature.clone());
        let reply = self.process_agent_reply_streaming(&agent_id, &feature, reply, stream)?;
        Ok(CommandChatResult::AgentLaunched {
            agent_id,
            feature,
            reply,
        })
    }

    fn process_agent_reply_streaming(
        &mut self,
        agent_id: &AgentId,
        feature: &str,
        reply: String,
        stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    ) -> Result<String, CliError> {
        let mut text = reply.clone();
        let mut pending = VecDeque::from([AgentFollowUp {
            agent_id: agent_id.clone(),
            text: reply,
        }]);
        let mut rounds = 0;

        while let Some(current) = pending.pop_front() {
            if rounds >= self.max_review_rounds {
                text.push_str(
                    "\n\norchestrator:\nstopped processing agent directives after the configured round limit",
                );
                break;
            }
            rounds += 1;

            let current_feature =
                self.agents
                    .get(&current.agent_id)
                    .cloned()
                    .unwrap_or_else(|| {
                        if current.agent_id == *agent_id {
                            feature.to_string()
                        } else {
                            "user-agent".to_string()
                        }
                    });
            let run = {
                let backend = self
                    .backend
                    .as_mut()
                    .expect("command chat backend is present");
                handle_agent_directives_streaming(
                    backend,
                    &self.locks,
                    &self.command_policy,
                    &current.agent_id,
                    &current_feature,
                    &current.text,
                    stream,
                )?
            };

            append_orchestrator_events(&mut text, &run.events);
            append_follow_ups(&mut text, &run.follow_up_replies);

            for follow_up in run.follow_up_replies {
                if !follow_up.text.is_empty() {
                    pending.push_back(follow_up);
                }
            }
        }

        Ok(text)
    }

    fn review(&mut self) -> Result<CommandChatResult, CliError> {
        let backend = self
            .backend
            .take()
            .expect("command chat backend is present");
        let mut coordinator = ReviewCoordinator::new(self.project_dir.clone(), backend)
            .with_max_rounds(self.max_review_rounds);
        let results = coordinator.review_latest_agent_commits()?;
        self.backend = Some(coordinator.into_backend());
        Ok(CommandChatResult::ReviewComplete(results))
    }

    fn linearize_questions(&self) -> Result<CommandChatResult, CliError> {
        let commits = GitHistory::new(self.project_dir.clone()).latest_agent_commits()?;
        Ok(CommandChatResult::LinearizeQuestions(
            LinearizePlanner::<B>::questions_for(&commits),
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandChatResult {
    Noop,
    Help(String),
    AgentLaunched {
        agent_id: AgentId,
        feature: String,
        reply: String,
    },
    AgentMessage {
        agent_id: AgentId,
        reply: String,
    },
    ReviewComplete(Vec<ReviewResult>),
    LinearizeQuestions(Vec<LinearizeQuestion>),
    Quit,
}

fn append_orchestrator_events(text: &mut String, events: &[OrchestratorEvent]) {
    if events.is_empty() {
        return;
    }

    text.push_str("\n\norchestrator:");
    for event in events {
        text.push('\n');
        text.push_str(&event.summary());
    }
}

fn append_follow_ups(text: &mut String, follow_ups: &[AgentFollowUp]) {
    for follow_up in follow_ups {
        if follow_up.text.is_empty() {
            continue;
        }
        text.push_str("\n\nagent follow-up from ");
        text.push_str(follow_up.agent_id.as_str());
        text.push_str(":\n");
        text.push_str(&follow_up.text);
    }
}

pub fn render_process_help() -> String {
    [
        "Usage: work-leaf [--model <model>]",
        "",
        "launches the orchestrator from the current project directory.",
        "Agents are created inside the command chat. Patches, file locks, review routing, and linearization handoff are orchestrator-controlled workflows, not top-level process commands.",
        "",
        "Inside command chat:",
        "  new [prompt...]",
        "  review",
        "  linearize",
        "  quit",
        "",
    ]
    .join("\n")
}

pub fn render_command_chat_help() -> String {
    [
        "Command chat:",
        "  new [prompt...]",
        "  review",
        "  linearize",
        "  quit",
        "",
        "Patches and file locks are triggered automatically when agents interact with the orchestrator.",
    ]
    .join("\n")
}

fn run_command_chat<B>(chat: CommandChat<B>) -> Result<(), CliError>
where
    B: AgentBackend + Send + 'static,
{
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        run_terminal_ui(chat)
    } else {
        run_scripted_command_chat(chat)
    }
}

fn run_terminal_ui<B>(chat: CommandChat<B>) -> Result<(), CliError>
where
    B: AgentBackend + Send + 'static,
{
    let (width, height) = terminal_size();
    let _raw_mode = RawTerminalMode::enter()?;
    let mut app = TerminalApp::new(chat, width, height);
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    let _screen_mode = AlternateScreenMode::enter(&mut stdout)?;

    render_terminal_frame(&mut stdout, &app)?;

    loop {
        app.tick();
        let mut byte = [0_u8; 1];
        match stdin.read(&mut byte)? {
            0 => thread::sleep(Duration::from_millis(10)),
            _ => {
                if !app.handle_byte(byte[0]) {
                    break;
                }
            }
        }
        if app.needs_render() {
            render_terminal_frame(&mut stdout, &app)?;
            app.mark_rendered();
        }
    }

    write!(stdout, "\u{1b}[2J\u{1b}[H")?;
    stdout.flush()?;
    Ok(())
}

fn run_scripted_command_chat<B>(mut chat: CommandChat<B>) -> Result<(), CliError>
where
    B: AgentBackend,
{
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    writeln!(stdout, "work-leaf orchestrator")?;
    writeln!(stdout, "project: {}", chat.project_dir.display())?;
    writeln!(stdout, "{}", render_command_chat_help())?;

    for line in stdin.lock().lines() {
        let line = line?;
        match chat.handle_line(&line) {
            Ok(result) => {
                if render_command_result(result, &mut stdout)? {
                    break;
                }
            }
            Err(error) => writeln!(stdout, "{}", command_chat_error_text(&error))?,
        }
    }
    Ok(())
}

fn render_terminal_frame<B>(output: &mut impl Write, app: &TerminalApp<B>) -> Result<(), CliError>
where
    B: AgentBackend + Send + 'static,
{
    write!(output, "{}", app.render_frame())?;
    output.flush()?;
    Ok(())
}

pub(crate) fn terminal_right_content(chat_buffer: &str, transcript: &[String]) -> String {
    let mut content = transcript.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    content.push_str("chat> ");
    content.push_str(chat_buffer);
    content
}

pub(crate) fn command_result_text(result: &CommandChatResult) -> String {
    match result {
        CommandChatResult::Noop => String::new(),
        CommandChatResult::Help(help) => help.clone(),
        CommandChatResult::AgentLaunched {
            agent_id, reply, ..
        } => {
            if reply.is_empty() {
                format!("agent {agent_id} launched")
            } else {
                format!("agent {agent_id} launched\n{reply}")
            }
        }
        CommandChatResult::AgentMessage { agent_id, reply } => {
            format!("{agent_id} replied\n{reply}")
        }
        CommandChatResult::ReviewComplete(results) => {
            if results.is_empty() {
                return "no agent commits found".to_string();
            }
            results
                .iter()
                .map(|result| {
                    format!(
                        "{} reviewed by {}: rounds={} resolved={}",
                        result.agent_id,
                        result.reviewer_id,
                        result.rounds,
                        if result.findings_resolved {
                            "yes"
                        } else {
                            "no"
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        CommandChatResult::LinearizeQuestions(questions) => {
            if questions.is_empty() {
                return "no reviewed agent commits found".to_string();
            }
            questions
                .iter()
                .map(|question| {
                    format!(
                        "{} [{}]\n{}",
                        question.agent_id, question.feature, question.prompt
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        CommandChatResult::Quit => "quit".to_string(),
    }
}

pub(crate) fn command_chat_error_text(error: &CliError) -> String {
    let message = match error {
        CliError::Usage(message) => message.clone(),
        CliError::Agent(error) => error.to_string(),
        CliError::Io(error) => error.to_string(),
        CliError::Orchestrator(error) => error.to_string(),
        CliError::Review(error) => error.to_string(),
    };
    format!("error: {message}")
}

#[cfg(test)]
pub(crate) fn apply_command_result_to_ui(
    ui: &mut crate::ui::TerminalUi,
    result: &CommandChatResult,
) {
    if let CommandChatResult::AgentLaunched {
        agent_id, feature, ..
    } = result
    {
        ui.add_agent(crate::ui::AgentListEntry::new(
            agent_id.clone(),
            feature.clone(),
        ));
        let _ = ui.activate_agent_chat(agent_id);
    }
}

pub(crate) fn ui_action_text(action: UiAction) -> String {
    match action {
        UiAction::OpenChatSamePane(agent_id) => format!("opened {agent_id} in split pane"),
        UiAction::OpenChatNewWindow(agent_id) => format!("opened {agent_id} in new window"),
        UiAction::ForkAgent(agent_id) => format!("fork requested for {agent_id}"),
    }
}

struct RawTerminalMode {
    saved_state: Option<String>,
}

impl RawTerminalMode {
    fn enter() -> Result<Self, CliError> {
        let saved_state = stty_output(&["-g"]);

        if saved_state.is_some() {
            let _ = stty_status(&["raw", "-echo", "min", "0", "time", "1"]);
        }

        Ok(Self { saved_state })
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
    fn enter(output: &mut impl Write) -> Result<Self, CliError> {
        write!(
            output,
            "\u{1b}[?1049h\u{1b}[?1000h\u{1b}[?1006h\u{1b}[2J\u{1b}[H"
        )?;
        output.flush()?;
        Ok(Self)
    }
}

impl Drop for AlternateScreenMode {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = write!(stdout, "\u{1b}[?1006l\u{1b}[?1000l\u{1b}[?1049l\u{1b}[?25h");
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

fn render_command_result(
    result: CommandChatResult,
    output: &mut impl Write,
) -> Result<bool, CliError> {
    match result {
        CommandChatResult::Noop => {}
        CommandChatResult::Help(help) => writeln!(output, "{help}")?,
        CommandChatResult::AgentLaunched {
            agent_id, reply, ..
        } => {
            writeln!(output, "agent {agent_id} launched")?;
            if !reply.is_empty() {
                writeln!(output, "{reply}")?;
            }
        }
        CommandChatResult::AgentMessage { agent_id, reply } => {
            writeln!(output, "{agent_id} replied")?;
            if !reply.is_empty() {
                writeln!(output, "{reply}")?;
            }
        }
        CommandChatResult::ReviewComplete(results) => {
            if results.is_empty() {
                writeln!(output, "no agent commits found")?;
            }
            for result in results {
                writeln!(
                    output,
                    "{} reviewed by {}: rounds={} resolved={}",
                    result.agent_id,
                    result.reviewer_id,
                    result.rounds,
                    if result.findings_resolved {
                        "yes"
                    } else {
                        "no"
                    }
                )?;
            }
        }
        CommandChatResult::LinearizeQuestions(questions) => {
            if questions.is_empty() {
                writeln!(output, "no reviewed agent commits found")?;
            }
            for question in questions {
                writeln!(output, "{} [{}]", question.agent_id, question.feature)?;
                writeln!(output, "{}", question.prompt)?;
            }
        }
        CommandChatResult::Quit => return Ok(true),
    }
    Ok(false)
}

fn codex_backend(project_dir: PathBuf, model: Option<String>) -> CodexBackend {
    let mut config = CodexCommandConfig::new(project_dir);
    if let Some(model) = model {
        config = config.with_model(model);
    }
    CodexBackend::new(config, PromptPolicy::for_restricted_agents())
}

fn split_command_line(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Agent(crate::agent::AgentError),
    Io(io::Error),
    Orchestrator(crate::orchestrator::OrchestratorError),
    Review(crate::review::ReviewError),
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n\n{}", render_process_help()),
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Orchestrator(error) => write!(formatter, "{error}"),
            Self::Review(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Agent(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Orchestrator(error) => Some(error),
            Self::Review(error) => Some(error),
            Self::Usage(_) => None,
        }
    }
}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<crate::orchestrator::OrchestratorError> for CliError {
    fn from(error: crate::orchestrator::OrchestratorError) -> Self {
        Self::Orchestrator(error)
    }
}

impl From<crate::review::ReviewError> for CliError {
    fn from(error: crate::review::ReviewError) -> Self {
        Self::Review(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{PaneFocus, TerminalUi, UiMode};

    #[test]
    fn launched_agent_result_selects_chat_and_enters_insert_mode() {
        let mut ui = TerminalUi::new(100, 30);
        let agent_id = AgentId::new("user-1").unwrap();
        let result = CommandChatResult::AgentLaunched {
            agent_id: agent_id.clone(),
            feature: "user-agent".to_string(),
            reply: String::new(),
        };

        apply_command_result_to_ui(&mut ui, &result);

        assert_eq!(ui.selected_agent(), Some(&agent_id));
        assert_eq!(ui.focus(), PaneFocus::Right);
        assert_eq!(ui.mode(), UiMode::Insert);
    }
}
