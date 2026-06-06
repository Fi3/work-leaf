use std::path::PathBuf;
use std::{fmt, fs};

use crate::agent::{AgentError, AgentId, ChatMessage};
use crate::codex::{AgentBackend, AgentStreamEvent};
use crate::locks::{CommandWriteIntent, CommandWritePolicy, FileAccessError, FileLockTable};
use crate::patch::{GitPatcher, PatchError, PatchRequest};

#[derive(Debug)]
pub struct AgentOrchestrator<B> {
    locks: FileLockTable,
    command_policy: CommandWritePolicy,
    backend: B,
}

impl<B> AgentOrchestrator<B>
where
    B: AgentBackend,
{
    pub fn new(root: PathBuf, backend: B) -> Self {
        Self {
            locks: FileLockTable::new(root),
            command_policy: CommandWritePolicy,
            backend,
        }
    }

    pub fn handle_agent_message(
        &mut self,
        agent_id: &AgentId,
        feature: &str,
        text: &str,
    ) -> Result<Vec<OrchestratorEvent>, OrchestratorError> {
        handle_agent_directives(
            &mut self.backend,
            &self.locks,
            &self.command_policy,
            agent_id,
            feature,
            text,
        )
        .map(|run| run.events)
    }

    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrchestratorEvent {
    AgentDone {
        agent_id: AgentId,
    },
    FileTextSent {
        agent_id: AgentId,
        paths: Vec<PathBuf>,
    },
    FileTextUnavailable {
        agent_id: AgentId,
        paths: Vec<PathBuf>,
        diagnostic: String,
    },
    CommandClassified {
        agent_id: AgentId,
        writes: bool,
        paths: Vec<PathBuf>,
    },
    PatchApplied {
        agent_id: AgentId,
        feature: String,
        reason: String,
        commit: String,
        files: Vec<PathBuf>,
    },
    PatchRejected {
        agent_id: AgentId,
        files: Vec<PathBuf>,
        diagnostic: String,
    },
    MessageRouted {
        from: AgentId,
        to: AgentId,
    },
}

impl OrchestratorEvent {
    pub fn summary(&self) -> String {
        match self {
            Self::AgentDone { agent_id } => {
                format!("agent {agent_id} reported done")
            }
            Self::FileTextSent { agent_id, paths } => {
                format!("sent file text to {agent_id}: {}", display_paths(paths))
            }
            Self::FileTextUnavailable {
                agent_id, paths, ..
            } => {
                format!(
                    "reported unavailable file text to {agent_id}: {}",
                    display_paths(paths)
                )
            }
            Self::CommandClassified {
                agent_id,
                writes,
                paths,
            } => format!(
                "classified command for {agent_id}: writes={} paths={}",
                if *writes { "yes" } else { "no" },
                display_paths(paths)
            ),
            Self::PatchApplied {
                agent_id,
                reason,
                commit,
                files,
                ..
            } => format!(
                "applied patch from {agent_id}: {reason}; commit={commit}; files={}",
                display_paths(files)
            ),
            Self::PatchRejected {
                agent_id, files, ..
            } => format!(
                "sent patch conflict diagnostics to {agent_id}: {}",
                display_paths(files)
            ),
            Self::MessageRouted { from, to } => {
                format!("routed message from {from} to {to}")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AgentFollowUp {
    pub agent_id: AgentId,
    pub text: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DirectiveRun {
    pub events: Vec<OrchestratorEvent>,
    pub follow_up_replies: Vec<AgentFollowUp>,
    pub completed: bool,
}

pub(crate) fn handle_agent_directives<B>(
    backend: &mut B,
    locks: &FileLockTable,
    command_policy: &CommandWritePolicy,
    agent_id: &AgentId,
    feature: &str,
    text: &str,
) -> Result<DirectiveRun, OrchestratorError>
where
    B: AgentBackend,
{
    handle_agent_directives_streaming(
        backend,
        locks,
        command_policy,
        agent_id,
        feature,
        text,
        &mut |_, _| {},
    )
}

pub(crate) fn handle_agent_directives_streaming<B>(
    backend: &mut B,
    locks: &FileLockTable,
    command_policy: &CommandWritePolicy,
    agent_id: &AgentId,
    feature: &str,
    text: &str,
    stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
) -> Result<DirectiveRun, OrchestratorError>
where
    B: AgentBackend,
{
    let directives = parse_agent_directives(text)?;
    let mut run = DirectiveRun::default();

    for directive in directives {
        match directive {
            AgentDirective::Read(paths) => {
                let response = read_requested_files(locks, &paths)?;
                let normalized_paths = response
                    .snapshots
                    .iter()
                    .map(|snapshot| snapshot.path.clone())
                    .collect::<Vec<_>>();
                let unavailable_paths = response
                    .failures
                    .iter()
                    .map(|failure| failure.path.clone())
                    .collect::<Vec<_>>();
                let prompt = render_file_read_response(&response.snapshots, &response.failures);
                let mut sink = |event| stream(agent_id, event);
                let reply = backend.send_streaming(agent_id, &prompt, &mut sink)?;
                run.follow_up_replies
                    .push(follow_up(agent_id.clone(), reply));
                if !normalized_paths.is_empty() {
                    run.events.push(OrchestratorEvent::FileTextSent {
                        agent_id: agent_id.clone(),
                        paths: normalized_paths,
                    });
                }
                if !unavailable_paths.is_empty() {
                    run.events.push(OrchestratorEvent::FileTextUnavailable {
                        agent_id: agent_id.clone(),
                        paths: unavailable_paths,
                        diagnostic: render_file_read_failures(&response.failures),
                    });
                }
            }
            AgentDirective::Classify(command) => {
                let intent = command_policy.classify(command.iter().map(String::as_str));
                let mut sink = |event| stream(agent_id, event);
                let reply = backend.send_streaming(
                    agent_id,
                    &render_command_classification(&command, &intent),
                    &mut sink,
                )?;
                run.follow_up_replies
                    .push(follow_up(agent_id.clone(), reply));
                run.events.push(OrchestratorEvent::CommandClassified {
                    agent_id: agent_id.clone(),
                    writes: intent.writes,
                    paths: intent.paths,
                });
            }
            AgentDirective::Patch { reason, diff } => {
                let patcher = GitPatcher::new(locks.root().to_path_buf(), locks.clone());
                let request =
                    PatchRequest::new(agent_id.clone(), feature.to_string(), reason.clone(), diff);
                match patcher.apply(request) {
                    Ok(outcome) => run.events.push(OrchestratorEvent::PatchApplied {
                        agent_id: agent_id.clone(),
                        feature: feature.to_string(),
                        reason,
                        commit: outcome.commit,
                        files: outcome.files,
                    }),
                    Err(PatchError::Conflict { files, diagnostic }) => {
                        let mut sink = |event| stream(agent_id, event);
                        let reply = backend.send_streaming(
                            agent_id,
                            &render_patch_conflict_prompt(&files, &diagnostic),
                            &mut sink,
                        )?;
                        run.follow_up_replies
                            .push(follow_up(agent_id.clone(), reply));
                        run.events.push(OrchestratorEvent::PatchRejected {
                            agent_id: agent_id.clone(),
                            files,
                            diagnostic,
                        });
                    }
                    Err(PatchError::ValidationFailed {
                        files,
                        command,
                        diagnostic,
                    }) => {
                        let diagnostic =
                            format!("Validation command `{command}` failed:\n{diagnostic}");
                        let mut sink = |event| stream(agent_id, event);
                        let reply = backend.send_streaming(
                            agent_id,
                            &render_patch_validation_prompt(&files, &command, &diagnostic),
                            &mut sink,
                        )?;
                        run.follow_up_replies
                            .push(follow_up(agent_id.clone(), reply));
                        run.events.push(OrchestratorEvent::PatchRejected {
                            agent_id: agent_id.clone(),
                            files,
                            diagnostic,
                        });
                    }
                    Err(error) => return Err(OrchestratorError::Patch(error)),
                }
            }
            AgentDirective::Send { target, message } => {
                let mut sink = |event| stream(&target, event);
                let reply = backend.send_streaming(
                    &target,
                    &format!("Message from {agent_id} about {feature}:\n{message}"),
                    &mut sink,
                )?;
                run.follow_up_replies.push(follow_up(target.clone(), reply));
                run.events.push(OrchestratorEvent::MessageRouted {
                    from: agent_id.clone(),
                    to: target,
                });
            }
            AgentDirective::Done => {
                run.completed = true;
                run.events.push(OrchestratorEvent::AgentDone {
                    agent_id: agent_id.clone(),
                });
                break;
            }
        }
    }

    Ok(run)
}

fn follow_up(agent_id: AgentId, message: ChatMessage) -> AgentFollowUp {
    AgentFollowUp {
        agent_id,
        text: message.text,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileReadResponse {
    snapshots: Vec<crate::locks::FileSnapshot>,
    failures: Vec<FileReadFailure>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileReadFailure {
    path: PathBuf,
    diagnostic: String,
}

fn read_requested_files(
    locks: &FileLockTable,
    paths: &[PathBuf],
) -> Result<FileReadResponse, FileAccessError> {
    let mut snapshots = Vec::new();
    let mut failures = Vec::new();

    for path in paths {
        let normalized = match locks.normalize_path(path) {
            Ok(path) => path,
            Err(error) => {
                failures.push(FileReadFailure {
                    path: path.clone(),
                    diagnostic: error.to_string(),
                });
                continue;
            }
        };

        let read = locks.with_read_locks(std::slice::from_ref(&normalized), || {
            fs::read_to_string(locks.root().join(&normalized))
                .map(|text| crate::locks::FileSnapshot {
                    path: normalized.clone(),
                    text,
                })
                .map_err(FileAccessError::Io)
        });

        match read {
            Ok(snapshot) => snapshots.push(snapshot),
            Err(FileAccessError::Io(error)) => failures.push(FileReadFailure {
                path: normalized,
                diagnostic: error.to_string(),
            }),
            Err(FileAccessError::PathEscapesRoot(path)) => failures.push(FileReadFailure {
                path,
                diagnostic: "path escapes project root".to_string(),
            }),
            Err(FileAccessError::Poisoned) => return Err(FileAccessError::Poisoned),
        }
    }

    Ok(FileReadResponse {
        snapshots,
        failures,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AgentDirective {
    Read(Vec<PathBuf>),
    Classify(Vec<String>),
    Patch { reason: String, diff: String },
    Send { target: AgentId, message: String },
    Done,
}

fn parse_agent_directives(text: &str) -> Result<Vec<AgentDirective>, OrchestratorError> {
    let mut directives = Vec::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        let Some(body) = directive_body(line) else {
            continue;
        };

        if body == "end" {
            continue;
        }

        if body == "done" {
            directives.push(AgentDirective::Done);
        } else if let Some(rest) = directive_rest(body, "read") {
            let paths = split_required(rest, "read requires at least one path")?
                .into_iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>();
            directives.push(AgentDirective::Read(paths));
        } else if let Some(rest) = directive_rest(body, "locks classify") {
            directives.push(AgentDirective::Classify(split_required(
                rest,
                "locks classify requires a command",
            )?));
        } else if let Some(rest) = directive_rest(body, "patch") {
            let reason = rest.trim();
            if reason.is_empty() {
                return Err(OrchestratorError::Usage(
                    "patch requires a reason".to_string(),
                ));
            }
            let mut diff = String::new();
            while let Some(next) = lines.peek().copied() {
                if directive_body(next).is_some_and(|body| body == "end") {
                    lines.next();
                    break;
                }
                diff.push_str(next);
                diff.push('\n');
                lines.next();
            }
            if diff.trim().is_empty() {
                return Err(OrchestratorError::Usage(
                    "patch requires a unified diff body".to_string(),
                ));
            }
            directives.push(AgentDirective::Patch {
                reason: reason.to_string(),
                diff,
            });
        } else if let Some(rest) = directive_rest(body, "send") {
            let mut parts = rest.trim().splitn(2, char::is_whitespace);
            let target = parts
                .next()
                .filter(|part| !part.is_empty())
                .ok_or_else(|| OrchestratorError::Usage("send requires an agent id".to_string()))?;
            let message = parts.next().map(str::trim).filter(|part| !part.is_empty());
            let Some(message) = message else {
                return Err(OrchestratorError::Usage(
                    "send requires a message".to_string(),
                ));
            };
            directives.push(AgentDirective::Send {
                target: AgentId::new(target)?,
                message: message.to_string(),
            });
        } else {
            return Err(OrchestratorError::Usage(format!(
                "unknown work-leaf directive `{body}`"
            )));
        }
    }

    Ok(directives)
}

fn directive_body(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let rest = line.strip_prefix("@work-leaf")?;
    let mut chars = rest.chars();
    if !chars.next()?.is_whitespace() {
        return None;
    }
    Some(chars.as_str().trim_start())
}

fn directive_rest<'a>(body: &'a str, command: &str) -> Option<&'a str> {
    let rest = body.strip_prefix(command)?;
    if rest.is_empty() {
        return Some("");
    }
    let mut chars = rest.chars();
    if chars.next()?.is_whitespace() {
        Some(chars.as_str().trim_start())
    } else {
        None
    }
}

fn split_required(rest: &str, error: &str) -> Result<Vec<String>, OrchestratorError> {
    let parts = rest
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if parts.is_empty() {
        Err(OrchestratorError::Usage(error.to_string()))
    } else {
        Ok(parts)
    }
}

fn render_file_read_response(
    snapshots: &[crate::locks::FileSnapshot],
    failures: &[FileReadFailure],
) -> String {
    let mut text = String::from("work-leaf file text\n");
    for snapshot in snapshots {
        text.push_str("\n--- ");
        text.push_str(&snapshot.path.display().to_string());
        text.push_str(" ---\n");
        text.push_str(&snapshot.text);
        if !snapshot.text.ends_with('\n') {
            text.push('\n');
        }
    }
    if !failures.is_empty() {
        text.push_str("\nUnavailable file text\n");
        text.push_str(&render_file_read_failures(failures));
    }
    text
}

fn render_file_read_failures(failures: &[FileReadFailure]) -> String {
    let mut text = String::new();
    for failure in failures {
        text.push_str("- ");
        text.push_str(&failure.path.display().to_string());
        text.push_str(": ");
        text.push_str(&failure.diagnostic);
        text.push('\n');
    }
    text
}

fn render_command_classification(command: &[String], intent: &CommandWriteIntent) -> String {
    format!(
        "work-leaf command classification\ncommand: {}\nwrites: {}\npaths: {}",
        command.join(" "),
        if intent.writes { "yes" } else { "no" },
        display_paths(&intent.paths)
    )
}

fn render_patch_conflict_prompt(files: &[PathBuf], diagnostic: &str) -> String {
    format!(
        "The orchestrator could not apply your patch.\nFiles: {}\n\nGit diagnostic:\n{}\n\nPlease provide a corrected unified diff patch.",
        display_paths(files),
        diagnostic
    )
}

fn render_patch_validation_prompt(files: &[PathBuf], command: &str, diagnostic: &str) -> String {
    format!(
        "The orchestrator rejected your patch because repository validation failed.\nFiles: {}\nCommand: {}\n\nDiagnostic:\n{}\n\nPlease provide a corrected unified diff patch.",
        display_paths(files),
        command,
        diagnostic
    )
}

fn display_paths(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        return "-".to_string();
    }
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug)]
pub enum OrchestratorError {
    Usage(String),
    Agent(AgentError),
    FileAccess(FileAccessError),
    Patch(PatchError),
}

impl fmt::Display for OrchestratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => formatter.write_str(message),
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::FileAccess(error) => write!(formatter, "{error}"),
            Self::Patch(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for OrchestratorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Agent(error) => Some(error),
            Self::FileAccess(error) => Some(error),
            Self::Patch(error) => Some(error),
            Self::Usage(_) => None,
        }
    }
}

impl From<AgentError> for OrchestratorError {
    fn from(error: AgentError) -> Self {
        Self::Agent(error)
    }
}

impl From<FileAccessError> for OrchestratorError {
    fn from(error: FileAccessError) -> Self {
        Self::FileAccess(error)
    }
}
