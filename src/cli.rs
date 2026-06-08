use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::fmt;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::agent::{
    AgentBackend, AgentId, AgentLaunch, AgentProfile, AgentShutdownHandle, AgentStreamEvent,
    PromptPolicy, ReadPermission,
};
use crate::chat_title::{chat_title_from_llm_reply, chat_title_prompt};
use crate::codex::{CodexBackend, CodexCommandConfig};
use crate::linearize::{LinearizePlanner, LinearizeQuestion};
use crate::locks::{CommandWritePolicy, FileLockTable};
use crate::orchestrator::{
    AgentFollowUp, DirectiveServices, FileReadTracker, OrchestratorEvent,
    handle_agent_directives_streaming,
};
use crate::review::{AgentCommit, has_no_findings};
use crate::review::{GitHistory, ReviewResult};
use crate::terminal_app::TerminalApp;
use crate::ui::UiAction;

const DEFAULT_NEW_AGENT_PROMPT: &str = "Start a new work-leaf user-agent session. Ask the user what to work on if the task is not already clear, then report the broad feature before proposing patches.";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessCommand {
    Help,
    Launch {
        model: Option<String>,
        read_permission: ReadPermission,
    },
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
        return Ok(ProcessCommand::Launch {
            model: None,
            read_permission: ReadPermission::Orchestrator,
        });
    }

    let mut model = None;
    let mut read_permission = ReadPermission::Orchestrator;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" | "help" => return Ok(ProcessCommand::Help),
            "--no-read-permission" => {
                read_permission = ReadPermission::DirectFilesystem;
                index += 1;
            }
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

    Ok(ProcessCommand::Launch {
        model,
        read_permission,
    })
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
        ProcessCommand::Launch {
            model,
            read_permission,
        } => {
            let project_dir = match env::current_dir() {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("{error}");
                    process::exit(1);
                }
            };
            let backend = match codex_backend(project_dir.clone(), model, read_permission) {
                Ok(backend) => backend,
                Err(error) => {
                    eprintln!("{error}");
                    process::exit(1);
                }
            };
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
    shutdown: AgentShutdownHandle,
    locks: FileLockTable,
    file_reads: FileReadTracker,
    command_policy: CommandWritePolicy,
    agents: BTreeMap<AgentId, String>,
    reviewers: BTreeSet<AgentId>,
    reviewed_agent_commits: BTreeMap<AgentId, String>,
    agent_profile: AgentProfile,
    max_review_rounds: usize,
    locked_command_timeout: Duration,
    next_user_agent: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProcessedAgentReply {
    transcript: String,
    final_reply: String,
}

impl<B> Clone for CommandChat<B>
where
    B: AgentBackend + Clone,
{
    fn clone(&self) -> Self {
        Self {
            project_dir: self.project_dir.clone(),
            backend: self.backend.clone(),
            shutdown: self.shutdown.clone(),
            locks: self.locks.clone(),
            file_reads: self.file_reads.clone(),
            command_policy: self.command_policy.clone(),
            agents: self.agents.clone(),
            reviewers: self.reviewers.clone(),
            reviewed_agent_commits: self.reviewed_agent_commits.clone(),
            agent_profile: self.agent_profile.clone(),
            max_review_rounds: self.max_review_rounds,
            locked_command_timeout: self.locked_command_timeout,
            next_user_agent: self.next_user_agent,
        }
    }
}

impl<B> CommandChat<B>
where
    B: AgentBackend,
{
    pub fn new(project_dir: PathBuf, backend: B) -> Self {
        let shutdown = backend.shutdown_handle();
        Self {
            locks: FileLockTable::new(project_dir.clone()),
            file_reads: FileReadTracker::default(),
            project_dir,
            backend: Some(backend),
            shutdown,
            command_policy: CommandWritePolicy,
            agents: BTreeMap::new(),
            reviewers: BTreeSet::new(),
            reviewed_agent_commits: BTreeMap::new(),
            agent_profile: AgentProfile::codex(),
            max_review_rounds: 80_000_000,
            locked_command_timeout: Duration::from_secs(5 * 60),
            next_user_agent: 1,
        }
    }

    pub fn with_agent_profile(mut self, agent_profile: AgentProfile) -> Self {
        self.agent_profile = agent_profile;
        self
    }

    pub fn agent_profile(&self) -> &AgentProfile {
        &self.agent_profile
    }

    pub fn with_max_review_rounds(mut self, max_review_rounds: usize) -> Self {
        self.max_review_rounds = max_review_rounds.max(1);
        self
    }

    pub fn with_locked_command_timeout(mut self, timeout: Duration) -> Self {
        self.locked_command_timeout = timeout;
        self
    }

    pub fn into_backend(self) -> B {
        self.backend.expect("command chat backend is present")
    }

    pub fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.shutdown.clone()
    }

    pub fn shutdown_agents(&self) {
        self.shutdown.shutdown();
    }

    pub(crate) fn project_dir(&self) -> &std::path::Path {
        &self.project_dir
    }

    pub(crate) fn register_agent_feature(&mut self, agent_id: AgentId, feature: String) {
        self.agents.insert(agent_id, feature);
    }

    pub(crate) fn interrupt_agent(&mut self, agent_id: &AgentId) -> Result<(), CliError> {
        self.backend
            .as_mut()
            .expect("command chat backend is present")
            .interrupt(agent_id)
            .map_err(CliError::Agent)
    }

    pub(crate) fn generate_chat_title(
        &mut self,
        source_agent_id: &AgentId,
        first_prompt: &str,
    ) -> Result<String, CliError> {
        let title_agent_id =
            AgentId::new(format!("title-{}", source_agent_id.as_str())).map_err(CliError::Agent)?;
        let session = self
            .backend
            .as_mut()
            .expect("command chat backend is present")
            .launch(AgentLaunch::new(
                title_agent_id,
                self.agent_profile.kind.clone(),
                "chat-title",
                chat_title_prompt(first_prompt),
            ))
            .map_err(CliError::Agent)?;
        let reply = session
            .messages
            .last()
            .map(|message| message.text.as_str())
            .unwrap_or_default();
        Ok(chat_title_from_llm_reply(reply, first_prompt))
    }

    pub fn handle_line(&mut self, line: &str) -> Result<CommandChatResult, CliError> {
        let parts = split_command_line(line);
        let Some(command) = parts.first().map(String::as_str) else {
            return Ok(CommandChatResult::Noop);
        };

        match command {
            "help" | "?" => Ok(CommandChatResult::Help(render_command_chat_help())),
            "quit" | "exit" | "q" => Ok(CommandChatResult::Quit),
            "new" => self.launch_agent(&parts[1..]),
            "review" => self.review(),
            "linearize" => self.linearize(),
            "linearize-questions" => self.linearize_questions(),
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
        let launch = build_user_agent_launch(self.next_user_agent, args, &self.agent_profile)?;
        self.next_user_agent += 1;
        Ok(launch)
    }

    pub fn prepare_linearize_launch(&mut self) -> Result<Option<AgentLaunch>, CliError> {
        let commits = GitHistory::new(self.project_dir.clone()).latest_agent_commits()?;
        if commits.is_empty() {
            return Ok(None);
        }

        let agent_id = self.next_linearizer_id()?;
        Ok(Some(AgentLaunch::new(
            agent_id,
            self.agent_profile.kind.clone(),
            "linearize reviewed patches",
            LinearizePlanner::<B>::interactive_prompt(&commits),
        )))
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
        self.reserve_prepared_agent_id(&agent_id);
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

    fn reserve_prepared_agent_id(&mut self, agent_id: &AgentId) {
        if let Some(number) = user_agent_number(agent_id) {
            self.next_user_agent = self.next_user_agent.max(number.saturating_add(1));
        }
    }

    fn process_agent_reply_streaming(
        &mut self,
        agent_id: &AgentId,
        feature: &str,
        reply: String,
        stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    ) -> Result<String, CliError> {
        Ok(self
            .process_agent_reply_streaming_result(agent_id, feature, reply, stream)?
            .transcript)
    }

    fn process_agent_reply_streaming_result(
        &mut self,
        agent_id: &AgentId,
        feature: &str,
        reply: String,
        stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    ) -> Result<ProcessedAgentReply, CliError> {
        let mut text = reply.clone();
        let mut final_reply = reply.clone();
        let mut pending = VecDeque::from([AgentFollowUp {
            agent_id: agent_id.clone(),
            text: reply,
        }]);
        let mut rounds = 0;

        while let Some(current) = pending.pop_front() {
            if current.agent_id == *agent_id {
                final_reply = current.text.clone();
            }
            if rounds >= self.max_review_rounds {
                let message = format!(
                    "agent did not converge after {} orchestrator rounds",
                    self.max_review_rounds
                );
                text.push_str("\n\norchestrator:\n");
                text.push_str(&message);
                final_reply = message;
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
                    DirectiveServices {
                        locks: &self.locks,
                        file_reads: &self.file_reads,
                        command_policy: &self.command_policy,
                        locked_command_timeout: self.locked_command_timeout,
                    },
                    &current.agent_id,
                    &current_feature,
                    &current.text,
                    stream,
                )?
            };

            append_orchestrator_events(&mut text, &run.events);
            append_follow_ups(&mut text, &run.follow_up_replies);

            if run.completed && current.agent_id == *agent_id {
                break;
            }

            for follow_up in run.follow_up_replies {
                if !follow_up.text.is_empty() {
                    pending.push_back(follow_up);
                }
            }
        }

        Ok(ProcessedAgentReply {
            transcript: text,
            final_reply,
        })
    }

    fn review(&mut self) -> Result<CommandChatResult, CliError> {
        let commits = GitHistory::new(self.project_dir.clone()).latest_agent_commits()?;
        let mut results = Vec::new();
        for commit in commits {
            if self
                .reviewed_agent_commits
                .get(&commit.agent_id)
                .is_some_and(|hash| hash == &commit.hash)
            {
                continue;
            }
            let reviewer_id = reviewer_id_for(&commit.agent_id)?;
            let reuse_reviewer = self.reviewers.contains(&reviewer_id);
            let result = self.review_commit_streaming_with_ids(
                commit,
                reviewer_id,
                reuse_reviewer,
                &mut |_, _| {},
            )?;
            self.record_review_result(&result);
            results.push(result);
        }
        Ok(CommandChatResult::ReviewComplete(results))
    }

    pub(crate) fn review_commit_streaming_with_ids(
        &mut self,
        commit: AgentCommit,
        reviewer_id: AgentId,
        reuse_reviewer: bool,
        stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    ) -> Result<ReviewResult, CliError> {
        let summary_prompt = format!(
            "Please summarize the final patch for Agent-ID {}.\nCommit: {}\nFeature: {}\nReason: {}\nContext: {}\n\nFocus on what behavior the patch changes.",
            commit.agent_id, commit.hash, commit.feature, commit.reason, commit.context
        );
        let mut summary_stream = |event| stream(&commit.agent_id, event);
        let summary = self
            .backend
            .as_mut()
            .expect("command chat backend is present")
            .send_streaming(&commit.agent_id, &summary_prompt, &mut summary_stream)
            .map_err(CliError::Agent)?
            .text;

        let review_feature = format!("review {}", commit.feature);
        let review_prompt = format!(
            "Review the final patch for Agent-ID {}.\nCommit: {}\nFeature: {}\nReason: {}\nContext: {}\nSummary from original agent:\n{}\n\nReply with NO_FINDINGS if there are no findings. Otherwise reply with FINDINGS followed by the issues.",
            commit.agent_id, commit.hash, commit.feature, commit.reason, commit.context, summary
        );
        let mut review_stream = |event| stream(&reviewer_id, event);
        let mut review_text = if reuse_reviewer {
            self.backend
                .as_mut()
                .expect("command chat backend is present")
                .send_streaming(&reviewer_id, &review_prompt, &mut review_stream)
                .map_err(CliError::Agent)?
                .text
        } else {
            let reviewer_session = self
                .backend
                .as_mut()
                .expect("command chat backend is present")
                .launch_streaming(
                    AgentLaunch::new(
                        reviewer_id.clone(),
                        self.agent_profile.kind.clone(),
                        review_feature.clone(),
                        review_prompt,
                    ),
                    &mut review_stream,
                )
                .map_err(CliError::Agent)?;
            reviewer_session
                .messages
                .last()
                .map(|message| message.text.clone())
                .unwrap_or_default()
        };
        review_text = self
            .process_agent_reply_streaming_result(
                &reviewer_id,
                &review_feature,
                review_text,
                stream,
            )?
            .final_reply;
        self.reviewers.insert(reviewer_id.clone());
        let mut rounds = 1;

        while !has_no_findings(&review_text) && rounds < self.max_review_rounds {
            stream(
                &commit.agent_id,
                AgentStreamEvent::Status("reviewer findings routed back for fixes".to_string()),
            );
            let fix_prompt = format!(
                "The reviewer found issues in your patch for commit {}.\n{}\n\nPlease fix the patch through the orchestrator patch flow.",
                commit.hash, review_text
            );
            stream(
                &commit.agent_id,
                AgentStreamEvent::AgentMessage(format!("reviewer findings:\n{review_text}")),
            );
            let mut fix_stream = |event| stream(&commit.agent_id, event);
            let fix_reply = self
                .backend
                .as_mut()
                .expect("command chat backend is present")
                .send_streaming(&commit.agent_id, &fix_prompt, &mut fix_stream)
                .map_err(CliError::Agent)?
                .text;
            let fix_reply = self.process_agent_reply_streaming(
                &commit.agent_id,
                &commit.feature,
                fix_reply,
                stream,
            )?;

            let recheck_prompt = format!(
                "The original agent has responded to the findings for commit {}.\n{}\n\nPlease check the patch again and reply with NO_FINDINGS if resolved, otherwise list remaining FINDINGS.",
                commit.hash, fix_reply
            );
            let mut recheck_stream = |event| stream(&reviewer_id, event);
            let recheck_reply = self
                .backend
                .as_mut()
                .expect("command chat backend is present")
                .send_streaming(&reviewer_id, &recheck_prompt, &mut recheck_stream)
                .map_err(CliError::Agent)?
                .text;
            review_text = self
                .process_agent_reply_streaming_result(
                    &reviewer_id,
                    &review_feature,
                    recheck_reply,
                    stream,
                )?
                .final_reply;
            rounds += 1;
        }

        Ok(ReviewResult {
            agent_id: commit.agent_id.clone(),
            reviewer_id,
            findings_resolved: has_no_findings(&review_text),
            rounds,
            commit,
        })
    }

    fn record_review_result(&mut self, result: &ReviewResult) {
        let latest_hash = self
            .latest_agent_commit_hash(&result.agent_id)
            .unwrap_or_else(|| result.commit.hash.clone());
        self.reviewed_agent_commits
            .insert(result.agent_id.clone(), latest_hash);
        self.reviewers.insert(result.reviewer_id.clone());
    }

    fn latest_agent_commit_hash(&self, agent_id: &AgentId) -> Option<String> {
        GitHistory::new(self.project_dir.clone())
            .latest_agent_commits()
            .ok()?
            .into_iter()
            .find(|commit| &commit.agent_id == agent_id)
            .map(|commit| commit.hash)
    }

    fn linearize(&mut self) -> Result<CommandChatResult, CliError> {
        let Some(launch) = self.prepare_linearize_launch()? else {
            return Ok(CommandChatResult::LinearizeQuestions(Vec::new()));
        };
        self.launch_prepared_agent_streaming(launch, &mut |_| {})
    }

    fn linearize_questions(&self) -> Result<CommandChatResult, CliError> {
        let commits = GitHistory::new(self.project_dir.clone()).latest_agent_commits()?;
        Ok(CommandChatResult::LinearizeQuestions(
            LinearizePlanner::<B>::questions_for(&commits),
        ))
    }

    fn next_linearizer_id(&self) -> Result<AgentId, CliError> {
        let base = AgentId::new("linearize").map_err(CliError::Agent)?;
        if !self.agents.contains_key(&base) {
            return Ok(base);
        }

        let mut number = 2;
        loop {
            let candidate = AgentId::new(format!("linearize-{number}")).map_err(CliError::Agent)?;
            if !self.agents.contains_key(&candidate) {
                return Ok(candidate);
            }
            number += 1;
        }
    }
}

pub(crate) fn build_user_agent_launch(
    agent_number: usize,
    args: &[String],
    agent_profile: &AgentProfile,
) -> Result<AgentLaunch, CliError> {
    let agent_id = AgentId::new(format!("user-{agent_number}")).map_err(CliError::Agent)?;
    let feature = agent_profile.default_feature.clone();
    let prompt = if args.is_empty() {
        DEFAULT_NEW_AGENT_PROMPT.to_string()
    } else {
        args.join(" ")
    };
    Ok(AgentLaunch::new(
        agent_id,
        agent_profile.kind.clone(),
        feature,
        prompt,
    ))
}

fn user_agent_number(agent_id: &AgentId) -> Option<usize> {
    agent_id.as_str().strip_prefix("user-")?.parse().ok()
}

fn reviewer_id_for(agent_id: &AgentId) -> Result<AgentId, CliError> {
    AgentId::new(format!("review-{}", agent_id.as_str())).map_err(CliError::Agent)
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
        "Usage: work-leaf [--model <model>] [--no-read-permission]",
        "",
        "launches the orchestrator from the current project directory.",
        "Agents are created inside the command chat. Patches, file locks, review routing, and linearization handoff are orchestrator-controlled workflows, not top-level process commands.",
        "",
        "Options:",
        "  --model <model>          select the Codex model",
        "  --no-read-permission     allow agents to read project files directly; writes still require orchestrator patches",
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
    B: AgentBackend + Clone + Send + 'static,
{
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        run_terminal_ui(chat)
    } else {
        run_scripted_command_chat(chat)
    }
}

fn run_terminal_ui<B>(chat: CommandChat<B>) -> Result<(), CliError>
where
    B: AgentBackend + Clone + Send + 'static,
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
    B: AgentBackend + Clone + Send + 'static,
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
            "\u{1b}[?1049h\u{1b}[?1000h\u{1b}[?1006h\u{1b}[?2004h\u{1b}[2J\u{1b}[H"
        )?;
        output.flush()?;
        Ok(Self)
    }
}

impl Drop for AlternateScreenMode {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = write!(
            stdout,
            "\u{1b}[?2004l\u{1b}[?1006l\u{1b}[?1000l\u{1b}[?1049l\u{1b}[?25h"
        );
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

fn codex_backend(
    project_dir: PathBuf,
    model: Option<String>,
    read_permission: ReadPermission,
) -> Result<CodexBackend, CliError> {
    let mut config = CodexCommandConfig::new(project_dir.clone());
    if let Some(model) = model {
        config = config.with_model(model);
    }
    Ok(CodexBackend::new(
        config,
        PromptPolicy::for_project_with_read_permission(&project_dir, read_permission)
            .map_err(CliError::Agent)?,
    ))
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
