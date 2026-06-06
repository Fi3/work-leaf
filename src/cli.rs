use std::env;
use std::fmt;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process;

use crate::agent::{AgentId, AgentKind, AgentLaunch, PromptPolicy};
use crate::codex::{AgentBackend, CodexBackend, CodexCommandConfig};
use crate::linearize::{LinearizePlanner, LinearizeQuestion};
use crate::review::{GitHistory, ReviewCoordinator, ReviewResult};

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
            let mut chat = CommandChat::new(project_dir, backend);
            if let Err(error) = run_command_chat(&mut chat) {
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
    max_review_rounds: usize,
}

impl<B> CommandChat<B>
where
    B: AgentBackend,
{
    pub fn new(project_dir: PathBuf, backend: B) -> Self {
        Self {
            project_dir,
            backend: Some(backend),
            max_review_rounds: 8,
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

    fn launch_agent(&mut self, args: &[String]) -> Result<CommandChatResult, CliError> {
        if args.len() < 3 {
            return Err(CliError::Usage(
                "command chat `new` requires <agent-id> <feature> <prompt...>".to_string(),
            ));
        }
        let agent_id = AgentId::new(args[0].clone()).map_err(CliError::Agent)?;
        let feature = args[1].clone();
        let prompt = args[2..].join(" ");
        let session = self
            .backend
            .as_mut()
            .expect("command chat backend is present")
            .launch(AgentLaunch::new(
                agent_id.clone(),
                AgentKind::Codex,
                feature,
                prompt,
            ))
            .map_err(CliError::Agent)?;
        let reply = session
            .messages
            .last()
            .map(|message| message.text.clone())
            .unwrap_or_default();
        Ok(CommandChatResult::AgentLaunched { agent_id, reply })
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
    AgentLaunched { agent_id: AgentId, reply: String },
    ReviewComplete(Vec<ReviewResult>),
    LinearizeQuestions(Vec<LinearizeQuestion>),
    Quit,
}

pub fn render_process_help() -> String {
    [
        "Usage: work-leaf [--model <model>]",
        "",
        "launches the orchestrator from the current project directory.",
        "Agents are created inside the command chat. Patches, file locks, review routing, and linearization handoff are orchestrator-controlled workflows, not top-level process commands.",
        "",
        "Inside command chat:",
        "  new <agent-id> <feature> <prompt...>",
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
        "  new <agent-id> <feature> <prompt...>",
        "  review",
        "  linearize",
        "  quit",
        "",
        "Patches and file locks are triggered automatically when agents interact with the orchestrator.",
    ]
    .join("\n")
}

fn run_command_chat<B>(chat: &mut CommandChat<B>) -> Result<(), CliError>
where
    B: AgentBackend,
{
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    writeln!(stdout, "work-leaf orchestrator")?;
    writeln!(stdout, "project: {}", chat.project_dir.display())?;
    writeln!(stdout, "{}", render_command_chat_help())?;

    if stdin.is_terminal() {
        loop {
            write!(stdout, "work-leaf> ")?;
            stdout.flush()?;
            let mut line = String::new();
            if stdin.read_line(&mut line)? == 0 {
                break;
            }
            if render_command_result(chat.handle_line(line.trim())?, &mut stdout)? {
                break;
            }
        }
    } else {
        for line in stdin.lock().lines() {
            if render_command_result(chat.handle_line(&line?)?, &mut stdout)? {
                break;
            }
        }
    }
    Ok(())
}

fn render_command_result(
    result: CommandChatResult,
    output: &mut impl Write,
) -> Result<bool, CliError> {
    match result {
        CommandChatResult::Noop => {}
        CommandChatResult::Help(help) => writeln!(output, "{help}")?,
        CommandChatResult::AgentLaunched { agent_id, reply } => {
            writeln!(output, "agent {agent_id} launched")?;
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
    Review(crate::review::ReviewError),
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n\n{}", render_process_help()),
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Review(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Agent(error) => Some(error),
            Self::Io(error) => Some(error),
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

impl From<crate::review::ReviewError> for CliError {
    fn from(error: crate::review::ReviewError) -> Self {
        Self::Review(error)
    }
}
