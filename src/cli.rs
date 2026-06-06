use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

use crate::agent::{AgentId, AgentKind, AgentLaunch, PromptPolicy};
use crate::codex::{AgentBackend, CodexBackend, CodexCommandConfig};
use crate::linearize::LinearizePlanner;
use crate::locks::{CommandWritePolicy, FileLockTable};
use crate::patch::{GitPatcher, PatchRequest};
use crate::review::{GitHistory, ReviewCoordinator};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Help,
    NewAgent {
        agent_id: String,
        feature: String,
        prompt: String,
        model: Option<String>,
    },
    Patch {
        agent_id: String,
        feature: String,
        reason: String,
        diff_path: PathBuf,
    },
    Review {
        max_rounds: usize,
        model: Option<String>,
    },
    LinearizeQuestions,
    ClassifyCommand {
        command: Vec<String>,
    },
}

pub fn parse_cli_args<I, S>(args: I) -> Result<CliCommand, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg.ends_with("work-leaf")) {
        args.remove(0);
    }
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::Usage("missing command".to_string()));
    };

    match command {
        "--help" | "-h" | "help" => Ok(CliCommand::Help),
        "new" => parse_new(&args[1..]),
        "patch" => parse_patch(&args[1..]),
        "review" => parse_review(&args[1..]),
        "linearize-questions" => {
            expect_no_extra("linearize-questions", &args[1..])?;
            Ok(CliCommand::LinearizeQuestions)
        }
        "locks" => parse_locks(&args[1..]),
        other => Err(CliError::Usage(format!("unknown command `{other}`"))),
    }
}

pub fn run_cli_command(
    command: CliCommand,
    project_dir: PathBuf,
    stdin: &str,
) -> Result<String, CliError> {
    match command {
        CliCommand::Help => Ok(usage()),
        CliCommand::NewAgent {
            agent_id,
            feature,
            prompt,
            model,
        } => {
            let agent_id = AgentId::new(agent_id).map_err(CliError::Agent)?;
            let backend = codex_backend(project_dir, model);
            run_new_agent(backend, agent_id, feature, prompt)
        }
        CliCommand::Patch {
            agent_id,
            feature,
            reason,
            diff_path,
        } => {
            let diff = read_diff(&diff_path, stdin)?;
            let agent_id = AgentId::new(agent_id).map_err(CliError::Agent)?;
            let locks = FileLockTable::new(project_dir.clone());
            let patcher = GitPatcher::new(project_dir, locks);
            let outcome = patcher.apply(PatchRequest::new(agent_id, feature, reason, diff))?;
            Ok(format!(
                "applied patch\ncommit: {}\nfiles: {}\n",
                outcome.commit,
                outcome
                    .files
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
        CliCommand::Review { max_rounds, model } => {
            let backend = codex_backend(project_dir.clone(), model);
            let mut coordinator =
                ReviewCoordinator::new(project_dir, backend).with_max_rounds(max_rounds);
            let results = coordinator.review_latest_agent_commits()?;
            let mut output = String::new();
            for result in results {
                output.push_str(&format!(
                    "{} reviewed by {}: rounds={} resolved={}\n",
                    result.agent_id,
                    result.reviewer_id,
                    result.rounds,
                    if result.findings_resolved {
                        "yes"
                    } else {
                        "no"
                    }
                ));
            }
            if output.is_empty() {
                output.push_str("no agent commits found\n");
            }
            Ok(output)
        }
        CliCommand::LinearizeQuestions => {
            let commits = GitHistory::new(project_dir).latest_agent_commits()?;
            let questions = LinearizePlanner::<CodexBackend>::questions_for(&commits);
            let mut output = String::new();
            for question in questions {
                output.push_str(&format!(
                    "{} [{}]\n{}\n",
                    question.agent_id, question.feature, question.prompt
                ));
            }
            if output.is_empty() {
                output.push_str("no reviewed agent commits found\n");
            }
            Ok(output)
        }
        CliCommand::ClassifyCommand { command } => {
            let intent = CommandWritePolicy.classify(command.iter().map(String::as_str));
            let paths = if intent.paths.is_empty() {
                "-".to_string()
            } else {
                intent
                    .paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            Ok(format!(
                "writes: {}\npaths: {}\n",
                if intent.writes { "yes" } else { "no" },
                paths
            ))
        }
    }
}

pub fn run_cli_from_env() -> ! {
    let command = match parse_cli_args(env::args()) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}");
            process::exit(2);
        }
    };
    let mut stdin = String::new();
    if command_reads_stdin(&command)
        && let Err(error) = io::stdin().read_to_string(&mut stdin)
    {
        eprintln!("{error}");
        process::exit(1);
    }
    let project_dir = match env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
    match run_cli_command(command, project_dir, &stdin) {
        Ok(output) => {
            print!("{output}");
            process::exit(0);
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

fn parse_new(args: &[String]) -> Result<CliCommand, CliError> {
    let (model, args) = parse_optional_model(args)?;
    if args.len() < 3 {
        return Err(CliError::Usage(
            "new requires <agent-id> <feature> <prompt...>".to_string(),
        ));
    }
    Ok(CliCommand::NewAgent {
        agent_id: args[0].clone(),
        feature: args[1].clone(),
        prompt: args[2..].join(" "),
        model,
    })
}

fn parse_patch(args: &[String]) -> Result<CliCommand, CliError> {
    if args.len() != 4 {
        return Err(CliError::Usage(
            "patch requires <agent-id> <feature> <reason> <diff-file|->".to_string(),
        ));
    }
    Ok(CliCommand::Patch {
        agent_id: args[0].clone(),
        feature: args[1].clone(),
        reason: args[2].clone(),
        diff_path: PathBuf::from(&args[3]),
    })
}

fn parse_review(args: &[String]) -> Result<CliCommand, CliError> {
    let (model, mut args) = parse_optional_model(args)?;
    let mut max_rounds = 8;
    while !args.is_empty() {
        match args[0].as_str() {
            "--max-rounds" => {
                if args.len() < 2 {
                    return Err(CliError::Usage(
                        "review --max-rounds requires a value".to_string(),
                    ));
                }
                max_rounds = args[1]
                    .parse()
                    .map_err(|_| CliError::Usage("max rounds must be a number".to_string()))?;
                args.drain(0..2);
            }
            other => {
                return Err(CliError::Usage(format!("unknown review option `{other}`")));
            }
        }
    }
    Ok(CliCommand::Review { max_rounds, model })
}

fn parse_locks(args: &[String]) -> Result<CliCommand, CliError> {
    if args.first().map(String::as_str) != Some("classify") {
        return Err(CliError::Usage(
            "locks requires `classify <command...>`".to_string(),
        ));
    }
    if args.len() < 2 {
        return Err(CliError::Usage(
            "locks classify requires <command...>".to_string(),
        ));
    }
    Ok(CliCommand::ClassifyCommand {
        command: args[1..].to_vec(),
    })
}

fn parse_optional_model(args: &[String]) -> Result<(Option<String>, Vec<String>), CliError> {
    let mut model = None;
    let mut remaining = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--model" {
            if index + 1 >= args.len() {
                return Err(CliError::Usage("--model requires a value".to_string()));
            }
            model = Some(args[index + 1].clone());
            index += 2;
        } else {
            remaining.push(args[index].clone());
            index += 1;
        }
    }
    Ok((model, remaining))
}

fn expect_no_extra(command: &str, args: &[String]) -> Result<(), CliError> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(CliError::Usage(format!(
            "{command} does not accept extra arguments"
        )))
    }
}

fn codex_backend(project_dir: PathBuf, model: Option<String>) -> CodexBackend {
    let mut config = CodexCommandConfig::new(project_dir);
    if let Some(model) = model {
        config = config.with_model(model);
    }
    CodexBackend::new(config, PromptPolicy::for_restricted_agents())
}

fn run_new_agent<B>(
    mut backend: B,
    agent_id: AgentId,
    feature: String,
    prompt: String,
) -> Result<String, CliError>
where
    B: AgentBackend,
{
    let session = backend
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
        .map(|message| message.text.as_str())
        .unwrap_or("");
    Ok(format!("launched {agent_id}\n{reply}\n"))
}

fn read_diff(path: &PathBuf, stdin: &str) -> Result<String, CliError> {
    if path.as_os_str() == "-" {
        Ok(stdin.to_string())
    } else {
        fs::read_to_string(path).map_err(CliError::Io)
    }
}

fn command_reads_stdin(command: &CliCommand) -> bool {
    matches!(command, CliCommand::Patch { diff_path, .. } if diff_path.as_os_str() == "-")
}

pub fn usage() -> String {
    [
        "Usage: work-leaf <command>",
        "",
        "Commands:",
        "  new <agent-id> <feature> <prompt...> [--model <model>]",
        "  patch <agent-id> <feature> <reason> <diff-file|->",
        "  review [--model <model>] [--max-rounds <n>]",
        "  linearize-questions",
        "  locks classify <command...>",
        "  help",
        "",
    ]
    .join("\n")
}

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Agent(crate::agent::AgentError),
    Io(io::Error),
    Patch(crate::patch::PatchError),
    Review(crate::review::ReviewError),
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n\n{}", usage()),
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Patch(error) => write!(formatter, "{error}"),
            Self::Review(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Agent(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Patch(error) => Some(error),
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

impl From<crate::patch::PatchError> for CliError {
    fn from(error: crate::patch::PatchError) -> Self {
        Self::Patch(error)
    }
}

impl From<crate::review::ReviewError> for CliError {
    fn from(error: crate::review::ReviewError) -> Self {
        Self::Review(error)
    }
}
