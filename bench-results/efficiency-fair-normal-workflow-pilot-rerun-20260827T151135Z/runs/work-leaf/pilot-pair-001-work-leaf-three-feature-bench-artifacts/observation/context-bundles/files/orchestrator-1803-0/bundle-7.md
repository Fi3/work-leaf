# Work Leaf Context Bundle

This file contains orchestrator-mediated read output. Use it as read-only context; submit project changes through `@work-leaf edit`.

----- BEGIN FILE src/cli.rs -----
digest: fnv64:22710fff93cc0275; bytes:50711

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::fmt;
use std::io::{self, BufRead, BufReader, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::{self, Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::agent::{
    AgentBackend, AgentId, AgentLaunch, AgentProfile, AgentShutdownHandle, AgentStreamEvent,
    PromptPolicy, ReadPermission,
};
use crate::chat_title::{chat_title_from_llm_reply, chat_title_prompt};
use crate::codex::{CodexBackend, CodexCommandConfig};
use crate::linearize::{LinearizePlanner, LinearizeQuestion};
use crate::locks::{CommandWritePolicy, FileLockTable};
use crate::orchestrator::{
    AgentFollowUp, CommandChangeTracker, DirectiveServices, FileReadTracker, OrchestratorEvent,
    handle_agent_directives_streaming,
};
use crate::review::{AgentCommit, has_no_findings};
use crate::review::{GitHistory, ReviewResult};
use crate::terminal_app::{RemoteTerminalApp, TerminalApp};
use crate::ui::UiAction;
use crate::{HttpControllerClient, OrchestratorHttpError, WorkLeafSnapshot};

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
            let result = if env::var_os("WORK_LEAF_IN_PROCESS").is_some() {
                run_in_process_cli(model, read_permission)
            } else {
                run_http_cli(model, read_permission)
            };
            if let Err(error) = result {
                eprintln!("{error}");
                process::exit(1);
            }
            process::exit(0);
        }
    }
}

fn run_in_process_cli(
    model: Option<String>,
    read_permission: ReadPermission,
) -> Result<(), CliError> {
    let project_dir = env::current_dir()?;
    let backend = codex_backend(project_dir.clone(), model, read_permission)?;
    let chat = CommandChat::new(project_dir, backend);
    run_command_chat(chat)
}

fn run_http_cli(model: Option<String>, read_permission: ReadPermission) -> Result<(), CliError> {
    let project_dir = env::current_dir()?;
    let (client, _daemon) = http_client_from_env_or_spawn(model, read_permission)?;
    run_http_command_chat(client, project_dir)
}

fn http_client_from_env_or_spawn(
    model: Option<String>,
    read_permission: ReadPermission,
) -> Result<(HttpControllerClient, Option<ManagedOrchestrator>), CliError> {
    if let Ok(url) = env::var("WORK_LEAF_ORCHESTRATOR_URL") {
        let client = HttpControllerClient::connect(url).map_err(http_cli_error)?;
        return Ok((client, None));
    }

    let mut daemon = ManagedOrchestrator::spawn(model, read_permission)?;
    let client = HttpControllerClient::connect(daemon.url.clone()).map_err(http_cli_error)?;
    daemon.client_url = Some(client.base_url().to_string());
    Ok((client, Some(daemon)))
}

#[derive(Debug)]
struct ManagedOrchestrator {
    child: Child,
    url: String,
    client_url: Option<String>,
}

impl ManagedOrchestrator {
    fn spawn(model: Option<String>, read_permission: ReadPermission) -> Result<Self, CliError> {
        let mut command = Command::new(orchestrator_binary_path()?);
        command.arg("--listen").arg("127.0.0.1:0");
        if let Some(model) = model {
            command.arg("--model").arg(model);
        }
        if read_permission == ReadPermission::DirectFilesystem {
            command.arg("--no-read-permission");
        }
        command
            .current_dir(env::current_dir()?)
            .env("WORK_LEAF_PARENT_PID", process::id().to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let mut child = command.spawn()?;
        let stdout = child.stdout.take().ok_or_else(|| {
            CliError::Io(io::Error::other("orchestrator stdout was not captured"))
        })?;
        let mut lines = BufReader::new(stdout).lines();
        let line = lines.next().ok_or_else(|| {
            CliError::Io(io::Error::other("orchestrator exited before startup"))
        })??;
        let url = line
            .strip_prefix("WORK_LEAF_ORCHESTRATOR_URL=")
            .ok_or_else(|| {
                CliError::Io(io::Error::other("orchestrator did not print a startup URL"))
            })?
            .to_string();
        thread::spawn(move || for _ in lines {});
        Ok(Self {
            child,
            url,
            client_url: None,
        })
    }
}

impl Drop for ManagedOrchestrator {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_some() {
            return;
        }
        if let Some(url) = &self.client_url
            && let Ok(mut client) = HttpControllerClient::connect(url.clone())
        {
            let _ = client.shutdown();
        }
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn orchestrator_binary_path() -> Result<PathBuf, CliError> {
    let exe = env::current_exe()?;
    let name = if cfg!(windows) {
        "work-leaf-orchestrator.exe"
    } else {
        "work-leaf-orchestrator"
    };
    Ok(exe.with_file_name(name))
}

fn http_cli_error(error: OrchestratorHttpError) -> CliError {
    CliError::Io(io::Error::other(error.to_string()))
}

#[derive(Debug)]
pub struct CommandChat<B> {
    project_dir: PathBuf,
    backend: Option<B>,
    shutdown: AgentShutdownHandle,
    locks: FileLockTable,
    file_reads: FileReadTracker,
    command_changes: CommandChangeTracker,
    command_policy: CommandWritePolicy,
    agents: BTreeMap<AgentId, String>,
    reviewers: BTreeSet<AgentId>,
    reviewed_agent_commits: BTreeMap<AgentId, String>,
    linearize_reviewed_commits: Vec<AgentCommit>,
    agent_review_baselines: BTreeMap<AgentId, String>,
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
            command_changes: self.command_changes.clone(),
            command_policy: self.command_policy.clone(),
            agents: self.agents.clone(),
            reviewers: self.reviewers.clone(),
            reviewed_agent_commits: self.reviewed_agent_commits.clone(),
            linearize_reviewed_commits: self.linearize_reviewed_commits.clone(),
            agent_review_baselines: self.agent_review_baselines.clone(),
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
            command_changes: CommandChangeTracker::default(),
            project_dir,
            backend: Some(backend),
            shutdown,
            command_policy: CommandWritePolicy,
            agents: BTreeMap::new(),
            reviewers: BTreeSet::new(),
            reviewed_agent_commits: BTreeMap::new(),
            linearize_reviewed_commits: Vec::new(),
            agent_review_baselines: BTreeMap::new(),
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

    pub(crate) fn mark_reviewed_agent_commit(&mut self, commit: AgentCommit) {
        let agent_id = commit.agent_id.clone();
        let hash = commit.hash.clone();
        self.reviewed_agent_commits
            .insert(agent_id.clone(), hash.clone());
        self.agent_review_baselines
            .insert(agent_id.clone(), hash.clone());
        if self
            .linearize_reviewed_commits
            .iter()
            .any(|commit| commit.hash == hash)
        {
            return;
        }
        self.linearize_reviewed_commits.push(commit);
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
        let commits = self.linearize_commits()?;
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
        self.remember_agent_review_baseline(&agent_id);
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

    fn remember_agent_review_baseline(&mut self, agent_id: &AgentId) {
        if user_agent_number(agent_id).is_none()
            || self.agent_review_baselines.contains_key(agent_id)
        {
            return;
        }
        if let Ok(Some(hash)) = GitHistory::new(self.project_dir.clone()).head_hash() {
            self.agent_review_baselines.insert(agent_id.clone(), hash);
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
                        command_changes: &self.command_changes,
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
        let commits = self.review_commits()?;
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

    fn review_commits(&self) -> Result<Vec<AgentCommit>, CliError> {
        Ok(
            GitHistory::new(self.project_dir.clone()).latest_agent_review_commits(
                &self.reviewed_agent_commits,
                &self.agent_review_baselines,
            )?,
        )
    }

    pub(crate) fn review_commit_streaming_with_ids(
        &mut self,
        commit: AgentCommit,
        reviewer_id: AgentId,
        reuse_reviewer: bool,
        stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    ) -> Result<ReviewResult, CliError> {
        let summary_prompt = format!(
            "Please summarize the full reviewed patch scope for Agent-ID {}.\nLatest commit: {}\nFeature: {}\nReason: {}\nReview scope:\n{}\n\nFocus on what behavior the cumulative patch changes.",
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
            "Review the full patch scope for Agent-ID {}.\nLatest commit: {}\nFeature: {}\nReason: {}\nReview scope:\n{}\nSummary from original agent:\n{}\n\nReview every commit listed in the review scope and reply with NO_FINDINGS if there are no findings. Otherwise reply with FINDINGS followed by the issues.",
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
        let latest_commit = self
            .latest_agent_review_commit(&result.agent_id)
            .unwrap_or_else(|| result.commit.clone());
        self.mark_reviewed_agent_commit(latest_commit);
        self.reviewers.insert(result.reviewer_id.clone());
    }

    fn latest_agent_review_commit(&self, agent_id: &AgentId) -> Option<AgentCommit> {
        let boundary = self
            .reviewed_agent_commits
            .get(agent_id)
            .or_else(|| self.agent_review_baselines.get(agent_id))
            .map(String::as_str);
        GitHistory::new(self.project_dir.clone())
            .agent_review_commit(agent_id, boundary)
            .ok()?
    }

    fn linearize(&mut self) -> Result<CommandChatResult, CliError> {
        let Some(launch) = self.prepare_linearize_launch()? else {
            return Ok(CommandChatResult::LinearizeQuestions(Vec::new()));
        };
        self.launch_prepared_agent_streaming(launch, &mut |_| {})
    }

    fn linearize_questions(&self) -> Result<CommandChatResult, CliError> {
        let commits = self.linearize_commits()?;
        Ok(CommandChatResult::LinearizeQuestions(
            LinearizePlanner::<B>::questions_for(&commits),
        ))
    }

    fn linearize_commits(&self) -> Result<Vec<AgentCommit>, CliError> {
        if self.linearize_reviewed_commits.is_empty() {
            return Ok(Vec::new());
        }
        let history = GitHistory::new(self.project_dir.clone());
        let commits = self
            .linearize_reviewed_commits
            .iter()
            .map(|commit| history.agent_commit(&commit.hash))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(commits)
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

fn run_http_command_chat(
    client: HttpControllerClient,
    project_dir: PathBuf,
) -> Result<(), CliError> {
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        run_remote_terminal_ui(client)
    } else {
        run_remote_scripted_command_chat(client, project_dir)
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

    let mut input = [0_u8; 4096];
    loop {
        app.tick();
        match stdin.read(&mut input)? {
            0 => {
                app.finish_pending_terminal_input();
                thread::sleep(Duration::from_millis(10));
            }
            count => {
                if !app.handle_terminal_bytes(&input[..count]) {
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

fn run_remote_terminal_ui(client: HttpControllerClient) -> Result<(), CliError> {
    let (width, height) = terminal_size();
    let _raw_mode = RawTerminalMode::enter()?;
    let mut app = RemoteTerminalApp::new(client, width, height);
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    let _screen_mode = AlternateScreenMode::enter(&mut stdout)?;

    write!(stdout, "{}", app.render_frame())?;
    stdout.flush()?;

    let mut input = [0_u8; 4096];
    loop {
        app.tick();
        match stdin.read(&mut input)? {
            0 => {
                app.finish_pending_terminal_input();
                thread::sleep(Duration::from_millis(10));
            }
            count => {
                if !app.handle_terminal_bytes(&input[..count]) {
                    break;
                }
            }
        }
        if app.needs_render() {
            write!(stdout, "{}", app.render_frame())?;
            stdout.flush()?;
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

fn run_remote_scripted_command_chat(
    mut client: HttpControllerClient,
    project_dir: PathBuf,
) -> Result<(), CliError> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    writeln!(stdout, "work-leaf orchestrator")?;
    writeln!(stdout, "project: {}", project_dir.display())?;
    writeln!(stdout, "{}", render_command_chat_help())?;

    let mut printed = PrintedRemoteState::new(
        client
            .snapshot()
            .map_err(http_cli_error)
            .unwrap_or_else(|_| WorkLeafSnapshot {
                command_transcript: Vec::new(),
                sessions: Vec::new(),
            }),
    );
    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim().to_string();
        client.execute_command_line(&line).map_err(http_cli_error)?;
        wait_and_print_remote_updates(&mut client, &mut printed, &mut stdout)?;
        if matches!(trimmed.as_str(), "quit" | "exit" | "q") {
            break;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct PrintedRemoteState {
    command_lines: usize,
    session_lines: BTreeMap<AgentId, usize>,
}

impl PrintedRemoteState {
    fn new(snapshot: WorkLeafSnapshot) -> Self {
        Self {
            command_lines: snapshot.command_transcript.len(),
            session_lines: snapshot
                .sessions
                .into_iter()
                .map(|session| (session.id, session.lines.len()))
                .collect(),
        }
    }

    fn print_new_lines(
        &mut self,
        snapshot: WorkLeafSnapshot,
        output: &mut impl Write,
    ) -> Result<(), CliError> {
        for line in snapshot.command_transcript.iter().skip(self.command_lines) {
            writeln!(output, "{line}")?;
        }
        self.command_lines = snapshot.command_transcript.len();

        for session in snapshot.sessions {
            let printed = self.session_lines.entry(session.id.clone()).or_insert(0);
            for line in session.lines.iter().skip(*printed) {
                writeln!(output, "{line}")?;
            }
            *printed = session.lines.len();
        }
        output.flush()?;
        Ok(())
    }
}

fn wait_and_print_remote_updates(
    client: &mut HttpControllerClient,
    printed: &mut PrintedRemoteState,
    output: &mut impl Write,
) -> Result<(), CliError> {
    loop {
        let busy = client.is_busy().map_err(http_cli_error)?;
        let snapshot = client.snapshot().map_err(http_cli_error)?;
        printed.print_new_lines(snapshot, output)?;
        if !busy {
            break;
        }
        thread::sleep(Duration::from_millis(20));
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

pub(crate) fn codex_backend(
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
----- END FILE src/cli.rs -----

----- BEGIN FILE src/http_controller.rs -----
digest: fnv64:dd2ce01ac0fab571; bytes:22497

use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::CommandChat;
use crate::agent::{AgentBackend, AgentId, ReadPermission};
use crate::cli::{ProcessCommand, codex_backend, parse_process_args, render_process_help};
use crate::workspace::{WorkLeafController, WorkLeafEvent, WorkLeafLoading, WorkLeafSnapshot};

#[derive(Clone, Debug)]
pub struct HttpControllerClient {
    base_url: String,
    address: String,
}

impl HttpControllerClient {
    pub fn connect(base_url: impl Into<String>) -> Result<Self, OrchestratorHttpError> {
        let base_url = base_url.into();
        let address = parse_http_address(&base_url)?;
        let client = Self {
            base_url: format!("http://{address}"),
            address,
        };
        client.health()?;
        Ok(client)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn health(&self) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.get("/health")?;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<WorkLeafSnapshot, OrchestratorHttpError> {
        self.get("/snapshot")
    }

    pub fn drain_events(&self) -> Result<Vec<WorkLeafEvent>, OrchestratorHttpError> {
        self.post("/events/drain", &EmptyRequest)
    }

    pub fn is_busy(&self) -> Result<bool, OrchestratorHttpError> {
        let response: BusyResponse = self.get("/busy")?;
        Ok(response.busy)
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> Result<bool, OrchestratorHttpError> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if !self.is_busy()? {
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(!self.is_busy()?)
    }

    pub fn execute_command_line(&mut self, line: &str) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post(
            "/command",
            &LineRequest {
                line: line.to_string(),
            },
        )?;
        Ok(())
    }

    pub fn send_command_agent_message(
        &mut self,
        message: &str,
    ) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post(
            "/command-agent",
            &MessageRequest {
                message: message.to_string(),
            },
        )?;
        Ok(())
    }

    pub fn send_message(
        &mut self,
        agent_id: &AgentId,
        message: &str,
    ) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post(
            "/agent/message",
            &AgentMessageRequest {
                agent_id: agent_id.clone(),
                message: message.to_string(),
            },
        )?;
        Ok(())
    }

    pub fn interrupt_agent(&mut self, agent_id: &AgentId) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post(
            "/agent/interrupt",
            &AgentRequest {
                agent_id: agent_id.clone(),
            },
        )?;
        Ok(())
    }

    pub fn push_transcript_line(&mut self, line: String) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post("/transcript", &LineRequest { line })?;
        Ok(())
    }

    pub fn loading_text(&self, loading: WorkLeafLoading) -> Result<String, OrchestratorHttpError> {
        let response: LoadingTextResponse =
            self.post("/loading-text", &LoadingTextRequest { loading })?;
        Ok(response.text)
    }

    pub fn shutdown(&mut self) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post("/shutdown", &EmptyRequest)?;
        Ok(())
    }

    fn get<T>(&self, path: &str) -> Result<T, OrchestratorHttpError>
    where
        T: DeserializeOwned,
    {
        self.request::<EmptyRequest, T>("GET", path, None)
    }

    fn post<T, R>(&self, path: &str, body: &T) -> Result<R, OrchestratorHttpError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.request("POST", path, Some(body))
    }

    fn request<T, R>(
        &self,
        method: &str,
        path: &str,
        body: Option<&T>,
    ) -> Result<R, OrchestratorHttpError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let body = match body {
            Some(body) => serde_json::to_vec(body)?,
            None => Vec::new(),
        };
        let mut stream = TcpStream::connect(&self.address)?;
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.address,
            body.len()
        )?;
        stream.write_all(&body)?;
        stream.flush()?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
        let (status, body) = parse_http_response(&response)?;
        if !(200..300).contains(&status) {
            let error = serde_json::from_slice::<ErrorResponse>(body)
                .map(|response| response.error)
                .unwrap_or_else(|_| String::from_utf8_lossy(body).to_string());
            return Err(OrchestratorHttpError::Api(error));
        }
        Ok(serde_json::from_slice(body)?)
    }
}

#[derive(Debug)]
pub enum OrchestratorHttpError {
    Io(io::Error),
    Json(serde_json::Error),
    Protocol(String),
    Api(String),
}

impl fmt::Display for OrchestratorHttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
            Self::Protocol(message) => write!(formatter, "{message}"),
            Self::Api(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for OrchestratorHttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Protocol(_) | Self::Api(_) => None,
        }
    }
}

impl From<io::Error> for OrchestratorHttpError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for OrchestratorHttpError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug)]
pub struct HttpControllerServer {
    listener: TcpListener,
}

impl HttpControllerServer {
    pub fn bind(address: &str) -> Result<Self, OrchestratorHttpError> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener })
    }

    pub fn local_url(&self) -> Result<String, OrchestratorHttpError> {
        Ok(format!("http://{}", self.listener.local_addr()?))
    }

    pub fn serve<B>(self, controller: WorkLeafController<B>) -> Result<(), OrchestratorHttpError>
    where
        B: AgentBackend + Clone + Send + 'static,
    {
        self.serve_with_parent(controller, None)
    }

    pub fn serve_with_parent<B>(
        self,
        controller: WorkLeafController<B>,
        parent_pid: Option<u32>,
    ) -> Result<(), OrchestratorHttpError>
    where
        B: AgentBackend + Clone + Send + 'static,
    {
        let controller = Arc::new(Mutex::new(controller));
        let shutdown = Arc::new(AtomicBool::new(false));
        while !shutdown.load(Ordering::SeqCst) {
            if parent_pid.is_some_and(|pid| !process_is_alive(pid)) {
                break;
            }
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let controller = Arc::clone(&controller);
                    let shutdown = Arc::clone(&shutdown);
                    thread::spawn(move || {
                        let _ = handle_connection(stream, controller, shutdown);
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(OrchestratorHttpError::Io(error)),
            }
        }
        if let Ok(mut controller) = controller.lock() {
            controller.shutdown();
        }
        Ok(())
    }
}

pub fn run_orchestrator_from_env() -> ! {
    let command = match parse_orchestrator_args(std::env::args()) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}");
            process::exit(2);
        }
    };

    match command {
        OrchestratorProcessCommand::Help => {
            print!("{}", render_orchestrator_help());
            process::exit(0);
        }
        OrchestratorProcessCommand::Launch(config) => {
            if let Err(error) = run_orchestrator(config) {
                eprintln!("{error}");
                process::exit(1);
            }
            process::exit(0);
        }
    }
}

fn run_orchestrator(config: OrchestratorProcessConfig) -> Result<(), String> {
    let project_dir = std::env::current_dir().map_err(|error| error.to_string())?;
    let backend = codex_backend(project_dir.clone(), config.model, config.read_permission)
        .map_err(|error| error.to_string())?;
    let chat = CommandChat::new(project_dir, backend);
    let controller = WorkLeafController::new(chat);
    let server = HttpControllerServer::bind(&config.listen).map_err(|error| error.to_string())?;
    println!(
        "WORK_LEAF_ORCHESTRATOR_URL={}",
        server.local_url().map_err(|error| error.to_string())?
    );
    io::stdout().flush().map_err(|error| error.to_string())?;
    let parent_pid = std::env::var("WORK_LEAF_PARENT_PID")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());
    server
        .serve_with_parent(controller, parent_pid)
        .map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OrchestratorProcessConfig {
    listen: String,
    model: Option<String>,
    read_permission: ReadPermission,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum OrchestratorProcessCommand {
    Help,
    Launch(OrchestratorProcessConfig),
}

fn parse_orchestrator_args<I, S>(args: I) -> Result<OrchestratorProcessCommand, crate::CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| {
        arg.ends_with("work-leaf-orchestrator") || arg.ends_with("work-leaf-orchestrator.exe")
    }) {
        args.remove(0);
    }

    let mut listen = "127.0.0.1:7878".to_string();
    let mut process_args = vec!["work-leaf".to_string()];
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--listen" => {
                if index + 1 >= args.len() {
                    return Err(crate::CliError::Usage(
                        "--listen requires a value".to_string(),
                    ));
                }
                listen = args[index + 1].clone();
                index += 2;
            }
            other => {
                process_args.push(other.to_string());
                index += 1;
            }
        }
    }

    match parse_process_args(process_args)? {
        ProcessCommand::Help => Ok(OrchestratorProcessCommand::Help),
        ProcessCommand::Launch {
            model,
            read_permission,
        } => Ok(OrchestratorProcessCommand::Launch(
            OrchestratorProcessConfig {
                listen,
                model,
                read_permission,
            },
        )),
    }
}

fn render_orchestrator_help() -> String {
    let mut help = render_process_help();
    help.push_str("Daemon options:\n");
    help.push_str("  --listen <addr>         bind the localhost HTTP API address\n");
    help
}

fn handle_connection<B>(
    mut stream: TcpStream,
    controller: Arc<Mutex<WorkLeafController<B>>>,
    shutdown: Arc<AtomicBool>,
) -> Result<(), OrchestratorHttpError>
where
    B: AgentBackend + Clone + Send + 'static,
{
    let Some(request) = read_http_request(&mut stream)? else {
        return Ok(());
    };
    let reply = route_request(request, controller, shutdown);
    write_http_reply(&mut stream, reply)?;
    Ok(())
}

fn route_request<B>(
    request: HttpRequest,
    controller: Arc<Mutex<WorkLeafController<B>>>,
    shutdown: Arc<AtomicBool>,
) -> HttpReply
where
    B: AgentBackend + Clone + Send + 'static,
{
    let path = request
        .path
        .split('?')
        .next()
        .unwrap_or(request.path.as_str());
    match (request.method.as_str(), path) {
        ("GET", "/health") => json_reply(200, &OkResponse { ok: true }),
        ("GET", "/snapshot") => with_controller(&controller, |controller| controller.snapshot()),
        ("POST", "/events/drain") => with_controller(&controller, WorkLeafController::drain_events),
        ("GET", "/busy") => with_controller(&controller, |controller| BusyResponse {
            busy: controller.is_busy(),
        }),
        ("POST", "/command") => {
            let body = match decode_body::<LineRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| {
                controller.execute_command_line(&body.line);
                OkResponse { ok: true }
            })
        }
        ("POST", "/command-agent") => {
            let body = match decode_body::<MessageRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| {
                controller.send_command_agent_message(&body.message);
                OkResponse { ok: true }
            })
        }
        ("POST", "/agent/message") => {
            let body = match decode_body::<AgentMessageRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller_result(&controller, |controller| {
                controller
                    .send_message(&body.agent_id, &body.message)
                    .map(|()| OkResponse { ok: true })
            })
        }
        ("POST", "/agent/interrupt") => {
            let body = match decode_body::<AgentRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| {
                controller.interrupt_agent(&body.agent_id);
                OkResponse { ok: true }
            })
        }
        ("POST", "/transcript") => {
            let body = match decode_body::<LineRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| {
                controller.push_transcript_line(body.line);
                OkResponse { ok: true }
            })
        }
        ("POST", "/loading-text") => {
            let body = match decode_body::<LoadingTextRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| LoadingTextResponse {
                text: controller.loading_text(body.loading),
            })
        }
        ("POST", "/shutdown") => {
            with_controller(&controller, WorkLeafController::shutdown);
            shutdown.store(true, Ordering::SeqCst);
            json_reply(200, &OkResponse { ok: true })
        }
        _ => error_reply(404, "not found"),
    }
}

fn with_controller<B, T, F>(
    controller: &Arc<Mutex<WorkLeafController<B>>>,
    operation: F,
) -> HttpReply
where
    B: AgentBackend + Clone + Send + 'static,
    T: Serialize,
    F: FnOnce(&mut WorkLeafController<B>) -> T,
{
    match controller.lock() {
        Ok(mut controller) => json_reply(200, &operation(&mut controller)),
        Err(_) => error_reply(500, "controller mutex poisoned"),
    }
}

fn with_controller_result<B, T, F>(
    controller: &Arc<Mutex<WorkLeafController<B>>>,
    operation: F,
) -> HttpReply
where
    B: AgentBackend + Clone + Send + 'static,
    T: Serialize,
    F: FnOnce(&mut WorkLeafController<B>) -> Result<T, crate::CliError>,
{
    match controller.lock() {
        Ok(mut controller) => match operation(&mut controller) {
            Ok(value) => json_reply(200, &value),
            Err(error) => error_reply(400, &error.to_string()),
        },
        Err(_) => error_reply(500, "controller mutex poisoned"),
    }
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

#[derive(Debug)]
struct HttpReply {
    status: u16,
    body: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
struct EmptyRequest;

#[derive(Deserialize, Serialize)]
struct OkResponse {
    ok: bool,
}

#[derive(Deserialize, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize, Serialize)]
struct BusyResponse {
    busy: bool,
}

#[derive(Deserialize, Serialize)]
struct LineRequest {
    line: String,
}

#[derive(Deserialize, Serialize)]
struct MessageRequest {
    message: String,
}

#[derive(Deserialize, Serialize)]
struct AgentRequest {
    agent_id: AgentId,
}

#[derive(Deserialize, Serialize)]
struct AgentMessageRequest {
    agent_id: AgentId,
    message: String,
}

#[derive(Deserialize, Serialize)]
struct LoadingTextRequest {
    loading: WorkLeafLoading,
}

#[derive(Deserialize, Serialize)]
struct LoadingTextResponse {
    text: String,
}

fn read_http_request(stream: &mut TcpStream) -> Result<Option<HttpRequest>, OrchestratorHttpError> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(None);
    }
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP method".to_string()))?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP path".to_string()))?
        .to_string();

    let mut content_length = 0_usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        let header = header.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse::<usize>().map_err(|_| {
                OrchestratorHttpError::Protocol("invalid Content-Length".to_string())
            })?;
        }
    }

    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    Ok(Some(HttpRequest { method, path, body }))
}

fn decode_body<T>(request: &HttpRequest) -> Result<T, HttpReply>
where
    T: DeserializeOwned,
{
    serde_json::from_slice(&request.body).map_err(|error| error_reply(400, &error.to_string()))
}

fn write_http_reply(stream: &mut TcpStream, reply: HttpReply) -> Result<(), OrchestratorHttpError> {
    let status_text = status_text(reply.status);
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reply.status,
        status_text,
        reply.body.len()
    )?;
    stream.write_all(&reply.body)?;
    stream.flush()?;
    Ok(())
}

fn json_reply<T>(status: u16, body: &T) -> HttpReply
where
    T: Serialize,
{
    match serde_json::to_vec(body) {
        Ok(body) => HttpReply { status, body },
        Err(error) => error_reply(500, &error.to_string()),
    }
}

fn error_reply(status: u16, error: &str) -> HttpReply {
    json_reply(
        status,
        &ErrorResponse {
            error: error.to_string(),
        },
    )
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn parse_http_response(response: &[u8]) -> Result<(u16, &[u8]), OrchestratorHttpError> {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP response headers".into()))?;
    let headers = std::str::from_utf8(&response[..header_end])
        .map_err(|_| OrchestratorHttpError::Protocol("HTTP headers are not UTF-8".into()))?;
    let status_line = headers
        .lines()
        .next()
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP status line".into()))?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP status code".into()))?
        .parse::<u16>()
        .map_err(|_| OrchestratorHttpError::Protocol("invalid HTTP status code".into()))?;
    Ok((status, &response[header_end + 4..]))
}

fn parse_http_address(base_url: &str) -> Result<String, OrchestratorHttpError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    let address = trimmed.strip_prefix("http://").ok_or_else(|| {
        OrchestratorHttpError::Protocol(
            "orchestrator URL must start with http:// and point at localhost".to_string(),
        )
    })?;
    if address.is_empty() || address.contains('/') {
        return Err(OrchestratorHttpError::Protocol(
            "orchestrator URL must not include a path".to_string(),
        ));
    }
    if !address.starts_with("127.0.0.1:") && !address.starts_with("localhost:") {
        return Err(OrchestratorHttpError::Protocol(
            "orchestrator URL must use localhost".to_string(),
        ));
    }
    Ok(address.to_string())
}

#[cfg(target_os = "linux")]
fn process_is_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(not(target_os = "linux"))]
fn process_is_alive(_pid: u32) -> bool {
    true
}
----- END FILE src/http_controller.rs -----

----- BEGIN FILE src/terminal_app.rs -----
digest: fnv64:37e6fb9d2b3a4bd1; bytes:43323

use std::thread;
use std::time::{Duration, Instant};

use rustyline::line_buffer::{ChangeListener, DeleteListener, Direction, LineBuffer};

use crate::agent::{AgentBackend, AgentId};
use crate::cli::{CommandChat, terminal_right_content, ui_action_text};
use crate::http_controller::HttpControllerClient;
use crate::ui::{AgentListEntry, PaneFocus, TerminalUi, UiKey, UiMode};
use crate::workspace::{
    WorkLeafController, WorkLeafEvent, WorkLeafLoading, WorkLeafSession, WorkLeafSnapshot,
};

#[derive(Debug)]
pub struct TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    inner: TerminalAppCore<LocalTerminalController<B>>,
}

impl<B> TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    pub fn new(chat: CommandChat<B>, width: u16, height: u16) -> Self {
        Self {
            inner: TerminalAppCore::new(
                LocalTerminalController {
                    controller: WorkLeafController::new(chat),
                },
                width,
                height,
            ),
        }
    }

    pub fn into_chat(mut self) -> CommandChat<B> {
        self.wait_for_idle(Duration::from_secs(5));
        self.inner.controller.controller.into_chat()
    }

    pub fn ui(&self) -> &TerminalUi {
        self.inner.ui()
    }

    pub fn transcript(&self) -> &[String] {
        self.inner.controller.controller.transcript()
    }

    pub fn is_quit(&self) -> bool {
        self.inner.is_quit()
    }

    pub fn is_busy(&mut self) -> bool {
        self.inner.is_busy()
    }

    pub fn needs_render(&self) -> bool {
        self.inner.needs_render()
    }

    pub fn mark_rendered(&mut self) {
        self.inner.mark_rendered();
    }

    pub fn tick(&mut self) {
        self.inner.tick();
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        self.inner.wait_for_idle(timeout)
    }

    pub fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        self.inner.wait_for_frame_contains(needle, timeout)
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_bytes(bytes)
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.inner.handle_byte(byte)
    }

    pub(crate) fn handle_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_terminal_bytes(bytes)
    }

    pub(crate) fn finish_pending_terminal_input(&mut self) {
        self.inner.finish_pending_terminal_input();
    }

    pub fn render_frame(&self) -> String {
        self.inner.render_frame()
    }

    pub fn poll_worker(&mut self) {
        self.inner.poll_worker();
    }

    #[cfg(test)]
    fn clear_agent_loading(&mut self, agent_id: &AgentId) {
        self.inner.clear_agent_loading(agent_id);
    }

    #[cfg(test)]
    fn set_agent_loading(&mut self, agent_id: &AgentId, loading: Option<LoadingKind>) {
        self.inner.set_agent_loading(agent_id, loading);
    }
}

#[derive(Debug)]
pub struct RemoteTerminalApp {
    inner: TerminalAppCore<HttpControllerClient>,
}

impl RemoteTerminalApp {
    pub fn new(client: HttpControllerClient, width: u16, height: u16) -> Self {
        Self {
            inner: TerminalAppCore::new(client, width, height),
        }
    }

    pub fn ui(&self) -> &TerminalUi {
        self.inner.ui()
    }

    pub fn is_quit(&self) -> bool {
        self.inner.is_quit()
    }

    pub fn is_busy(&mut self) -> bool {
        self.inner.is_busy()
    }

    pub fn needs_render(&self) -> bool {
        self.inner.needs_render()
    }

    pub fn mark_rendered(&mut self) {
        self.inner.mark_rendered();
    }

    pub fn tick(&mut self) {
        self.inner.tick();
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        self.inner.wait_for_idle(timeout)
    }

    pub fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        self.inner.wait_for_frame_contains(needle, timeout)
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_bytes(bytes)
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.inner.handle_byte(byte)
    }

    pub(crate) fn handle_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_terminal_bytes(bytes)
    }

    pub(crate) fn finish_pending_terminal_input(&mut self) {
        self.inner.finish_pending_terminal_input();
    }

    pub fn render_frame(&self) -> String {
        self.inner.render_frame()
    }

    pub fn poll_worker(&mut self) {
        self.inner.poll_worker();
    }
}

#[derive(Debug)]
struct LocalTerminalController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    controller: WorkLeafController<B>,
}

trait TerminalController {
    fn snapshot(&self) -> crate::WorkLeafSnapshot;
    fn drain_events(&mut self) -> Vec<WorkLeafEvent>;
    fn execute_command_line(&mut self, line: &str);
    fn send_command_agent_message(&mut self, message: &str);
    fn send_message(&mut self, agent_id: &AgentId, message: &str);
    fn interrupt_agent(&mut self, agent_id: &AgentId);
    fn push_transcript_line(&mut self, line: String);
    fn is_busy(&mut self) -> bool;
    fn loading_text(&self, loading: WorkLeafLoading) -> String;
    fn shutdown(&mut self);
}

impl<B> TerminalController for LocalTerminalController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    fn snapshot(&self) -> crate::WorkLeafSnapshot {
        self.controller.snapshot()
    }

    fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
        self.controller.drain_events()
    }

    fn execute_command_line(&mut self, line: &str) {
        self.controller.execute_command_line(line);
    }

    fn send_command_agent_message(&mut self, message: &str) {
        self.controller.send_command_agent_message(message);
    }

    fn send_message(&mut self, agent_id: &AgentId, message: &str) {
        let _ = self.controller.send_message(agent_id, message);
    }

    fn interrupt_agent(&mut self, agent_id: &AgentId) {
        self.controller.interrupt_agent(agent_id);
    }

    fn push_transcript_line(&mut self, line: String) {
        self.controller.push_transcript_line(line);
    }

    fn is_busy(&mut self) -> bool {
        self.controller.is_busy()
    }

    fn loading_text(&self, loading: WorkLeafLoading) -> String {
        self.controller.loading_text(loading)
    }

    fn shutdown(&mut self) {
        self.controller.shutdown();
    }
}

impl TerminalController for HttpControllerClient {
    fn snapshot(&self) -> crate::WorkLeafSnapshot {
        self.snapshot()
            .unwrap_or_else(|error| crate::WorkLeafSnapshot {
                command_transcript: vec![format!("error: {error}")],
                sessions: Vec::new(),
            })
    }

    fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
        HttpControllerClient::drain_events(self).unwrap_or_default()
    }

    fn execute_command_line(&mut self, line: &str) {
        let _ = HttpControllerClient::execute_command_line(self, line);
    }

    fn send_command_agent_message(&mut self, message: &str) {
        let _ = HttpControllerClient::send_command_agent_message(self, message);
    }

    fn send_message(&mut self, agent_id: &AgentId, message: &str) {
        let _ = HttpControllerClient::send_message(self, agent_id, message);
    }

    fn interrupt_agent(&mut self, agent_id: &AgentId) {
        let _ = HttpControllerClient::interrupt_agent(self, agent_id);
    }

    fn push_transcript_line(&mut self, line: String) {
        let _ = HttpControllerClient::push_transcript_line(self, line);
    }

    fn is_busy(&mut self) -> bool {
        HttpControllerClient::is_busy(self).unwrap_or(false)
    }

    fn loading_text(&self, loading: WorkLeafLoading) -> String {
        HttpControllerClient::loading_text(self, loading)
            .unwrap_or_else(|_| "Waiting for agent".to_string())
    }

    fn shutdown(&mut self) {
        let _ = HttpControllerClient::shutdown(self);
    }
}

#[derive(Debug)]
struct TerminalAppCore<C>
where
    C: TerminalController,
{
    controller: C,
    ui: TerminalUi,
    prompt_buffer: PromptLine,
    prompt_history: Vec<String>,
    prompt_history_index: Option<usize>,
    prompt_history_draft: Option<String>,
    chat_buffer: PromptLine,
    chat_history: Vec<String>,
    chat_history_index: Option<usize>,
    chat_history_draft: Option<String>,
    escape_sequence: Option<PendingEscapeSequence>,
    paste_mode: bool,
    skip_next_paste_lf: bool,
    spinner: usize,
    snapshot: WorkLeafSnapshot,
    loading_text: [(WorkLeafLoading, String); 2],
    dirty: bool,
    quit: bool,
}

impl<C> TerminalAppCore<C>
where
    C: TerminalController,
{
    fn new(controller: C, width: u16, height: u16) -> Self {
        let snapshot = controller.snapshot();
        let loading_text = [
            (
                WorkLeafLoading::Launching,
                controller.loading_text(WorkLeafLoading::Launching),
            ),
            (
                WorkLeafLoading::WaitingForReply,
                controller.loading_text(WorkLeafLoading::WaitingForReply),
            ),
        ];
        let mut app = Self {
            controller,
            ui: TerminalUi::new(width, height),
            prompt_buffer: PromptLine::new(),
            prompt_history: Vec::new(),
            prompt_history_index: None,
            prompt_history_draft: None,
            chat_buffer: PromptLine::new(),
            chat_history: Vec::new(),
            chat_history_index: None,
            chat_history_draft: None,
            escape_sequence: None,
            paste_mode: false,
            skip_next_paste_lf: false,
            spinner: 0,
            snapshot,
            loading_text,
            dirty: true,
            quit: false,
        };
        let sessions = app.snapshot.sessions.clone();
        for session in sessions {
            app.apply_session_to_ui(&session);
        }
        app
    }

    fn ui(&self) -> &TerminalUi {
        &self.ui
    }

    fn is_quit(&self) -> bool {
        self.quit
    }

    fn is_busy(&mut self) -> bool {
        let busy = self.controller.is_busy();
        self.apply_controller_events();
        busy
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.ui.has_status_notice()
    }

    fn mark_rendered(&mut self) {
        self.dirty = false;
        self.ui.clear_expired_status_notice();
    }

    fn tick(&mut self) {
        let busy = self.controller.is_busy();
        self.apply_controller_events();
        if busy {
            self.spinner = (self.spinner + 1) % SPINNER.len();
            self.dirty = true;
        }
    }

    fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.apply_controller_events();
            if !self.controller.is_busy() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.apply_controller_events();
        !self.controller.is_busy()
    }

    fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.apply_controller_events();
            if self.render_frame().contains(needle) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.apply_controller_events();
        self.render_frame().contains(needle)
    }

    fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        if !self.handle_terminal_bytes(bytes) {
            return false;
        }
        self.finish_pending_terminal_input();
        !self.quit
    }

    fn handle_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        self.apply_controller_events();
        for byte in bytes {
            if !self.handle_byte_without_poll(*byte) {
                self.apply_controller_events();
                return false;
            }
        }
        self.apply_controller_events();
        !self.quit
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.apply_controller_events();
        let keep_running = self.handle_byte_without_poll(byte);
        self.apply_controller_events();
        keep_running && !self.quit
    }

    fn finish_pending_terminal_input(&mut self) {
        self.finish_pending_escape_sequence();
        self.apply_controller_events();
    }

    fn handle_byte_without_poll(&mut self, byte: u8) -> bool {
        if self.quit {
            return false;
        }

        if self.continue_escape_sequence(byte) {
            return !self.quit;
        }

        if byte == 27 {
            let defer_escape = self.defer_escape_key();
            self.escape_sequence = Some(PendingEscapeSequence {
                bytes: Vec::new(),
                mode_before: self.ui.mode(),
                escape_dispatched: !defer_escape,
            });
            if !defer_escape {
                self.handle_input(TerminalAppInput::Key(UiKey::Esc));
            }
            return !self.quit;
        }

        let Some(input) = self.input_from_byte(byte) else {
            return true;
        };
        self.handle_input(input);
        !self.quit
    }

    pub fn render_frame(&self) -> String {
        let right_content = self.right_content();
        let right_cursor_column = (self.ui.focus() == PaneFocus::Right
            && self.ui.mode() != UiMode::Prompt)
            .then_some(6 + self.chat_buffer.cursor_char_count());
        self.ui.render_screen_with_cursors(
            &right_content,
            self.prompt_buffer.as_str(),
            self.prompt_buffer.cursor(),
            right_cursor_column,
        )
    }

    pub fn poll_worker(&mut self) {
        self.apply_controller_events();
    }

    fn handle_input(&mut self, input: TerminalAppInput) {
        match input {
            TerminalAppInput::Quit => {
                self.request_quit();
            }
            TerminalAppInput::Interrupt => {
                self.ui.show_ctrl_c_exit_notice();
                if self.ui.focus() == PaneFocus::Right
                    && let Some(agent_id) = self.ui.selected_agent().cloned()
                {
                    self.controller.interrupt_agent(&agent_id);
                    self.apply_controller_events();
                }
                self.dirty = true;
            }
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.backspace();
                self.prompt_history_index = None;
                self.prompt_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.backspace();
                self.chat_history_index = None;
                self.chat_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Prompt => {
                let line = self.prompt_buffer.trimmed_string();
                self.prompt_buffer.clear();
                self.ui.handle_key(UiKey::Esc);
                if !line.is_empty() {
                    self.prompt_history.push(line.clone());
                    self.prompt_history_index = None;
                    self.prompt_history_draft = None;
                    self.handle_command_line(&line);
                } else {
                    self.prompt_history_index = None;
                    self.prompt_history_draft = None;
                }
                self.dirty = true;
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Insert => {
                self.send_chat_buffer();
            }
            TerminalAppInput::Char('/') if self.should_start_agent_slash_command() => {
                self.start_agent_slash_command();
                self.dirty = true;
            }
            TerminalAppInput::LineBreak if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.push('\n');
                self.chat_history_index = None;
                self.chat_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::PasteStart => {
                self.paste_mode = true;
                self.skip_next_paste_lf = false;
            }
            TerminalAppInput::PasteEnd => {
                self.paste_mode = false;
                self.skip_next_paste_lf = false;
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.push(ch);
                self.prompt_history_index = None;
                self.prompt_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.push(ch);
                self.chat_history_index = None;
                self.chat_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Left) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.move_left();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Right) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.move_right();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Up) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(-1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Down) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Left) if self.should_route_chat_arrow() => {
                self.chat_buffer.move_left();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Right) if self.should_route_chat_arrow() => {
                self.chat_buffer.move_right();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Up) if self.should_route_chat_arrow() => {
                self.recall_chat_history(-1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Down) if self.should_route_chat_arrow() => {
                self.recall_chat_history(1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Esc) => {
                self.prompt_buffer.clear();
                let actions = self.ui.handle_key(UiKey::Esc);
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Key(key) => {
                let actions = self.ui.handle_key(key);
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Char(ch) => {
                let actions = self.ui.handle_key(UiKey::Char(ch));
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Backspace | TerminalAppInput::Enter | TerminalAppInput::LineBreak => {
            }
        }
    }

    fn handle_command_line(&mut self, line: &str) {
        if is_agent_slash_command(line)
            && let Some(agent_id) = self.ui.selected_agent().cloned()
        {
            self.controller.send_message(&agent_id, line);
            self.apply_controller_events();
            return;
        }

        self.controller.execute_command_line(line);
        self.apply_controller_events();
    }

    fn send_chat_buffer(&mut self) {
        let message = self.chat_buffer.trimmed_string();
        self.chat_buffer.clear();
        self.chat_history_index = None;
        self.chat_history_draft = None;
        if message.is_empty() {
            self.dirty = true;
            return;
        }

        self.chat_history.push(message.clone());
        if let Some(agent_id) = self.ui.selected_agent().cloned() {
            self.controller.send_message(&agent_id, &message);
        } else {
            self.controller.send_command_agent_message(&message);
        }
        self.apply_controller_events();
        self.dirty = true;
    }

    #[cfg(test)]
    fn clear_agent_loading(&mut self, agent_id: &AgentId) {
        self.set_agent_loading(agent_id, None);
    }

    #[cfg(test)]
    fn set_agent_loading(&mut self, agent_id: &AgentId, loading: Option<LoadingKind>) {
        let _ = self.ui.set_agent_ready_state(agent_id, loading.is_none());
    }

    fn record_actions(&mut self, actions: Vec<crate::UiAction>) {
        for action in actions {
            self.controller.push_transcript_line(ui_action_text(action));
        }
        self.apply_controller_events();
    }

    fn apply_controller_events(&mut self) {
        let events = self.controller.drain_events();
        if events.is_empty() {
            return;
        }
        for event in events {
            match event {
                WorkLeafEvent::AgentAdded { session } | WorkLeafEvent::AgentUpdated { session } => {
                    self.upsert_cached_session(session.clone());
                    self.apply_session_to_ui(&session);
                }
                WorkLeafEvent::AgentStatusUpdated {
                    agent_id,
                    kind,
                    title,
                    feature,
                    loading,
                } => {
                    let session =
                        self.upsert_cached_session_status(agent_id, kind, title, feature, loading);
                    self.apply_session_to_ui(&session);
                }
                WorkLeafEvent::AgentLineAppended { agent_id, line } => {
                    self.append_cached_agent_line(&agent_id, line);
                }
                WorkLeafEvent::AgentSelected { agent_id } => {
                    let _ = self.ui.activate_agent_chat(&agent_id);
                }
                WorkLeafEvent::CommandTranscriptLine { line } => {
                    self.snapshot.command_transcript.push(line);
                }
                WorkLeafEvent::QuitRequested => {
                    self.quit = true;
                }
            }
        }
        self.dirty = true;
    }

    fn upsert_cached_session(&mut self, session: WorkLeafSession) {
        if let Some(existing) = self
            .snapshot
            .sessions
            .iter_mut()
            .find(|existing| existing.id == session.id)
        {
            *existing = session;
        } else {
            self.snapshot.sessions.push(session);
            self.snapshot
                .sessions
                .sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        }
    }

    fn upsert_cached_session_status(
        &mut self,
        agent_id: AgentId,
        kind: crate::agent::AgentKind,
        title: String,
        feature: String,
        loading: Option<WorkLeafLoading>,
    ) -> WorkLeafSession {
        if let Some(session) = self
            .snapshot
            .sessions
            .iter_mut()
            .find(|session| session.id == agent_id)
        {
            session.kind = kind;
            session.title = title;
            session.feature = feature;
            session.loading = loading;
            return session.clone();
        }

        let session = WorkLeafSession {
            id: agent_id,
            kind,
            title,
            feature,
            lines: Vec::new(),
            loading,
        };
        self.snapshot.sessions.push(session.clone());
        self.snapshot
            .sessions
            .sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        session
    }

    fn append_cached_agent_line(&mut self, agent_id: &AgentId, line: String) {
        if line.is_empty() {
            return;
        }
        let Some(session) = self
            .snapshot
            .sessions
            .iter_mut()
            .find(|session| &session.id == agent_id)
        else {
            return;
        };
        if !session.lines.iter().any(|existing| existing == &line) {
            session.lines.push(line);
        }
    }

    fn apply_session_to_ui(&mut self, session: &WorkLeafSession) {
        if self
            .ui
            .set_agent_feature(&session.id, session.title.clone())
            .is_err()
        {
            self.ui.add_agent(AgentListEntry::new(
                session.id.clone(),
                session.title.clone(),
            ));
        }
        let _ = self
            .ui
            .set_agent_ready_state(&session.id, session.loading.is_none());
    }

    fn should_start_agent_slash_command(&self) -> bool {
        self.ui.mode() == UiMode::Command && self.ui.selected_agent().is_some()
    }

    fn start_agent_slash_command(&mut self) {
        let Some(agent_id) = self.ui.selected_agent().cloned() else {
            return;
        };
        if self.ui.activate_agent_chat(&agent_id).is_ok() {
            self.chat_buffer.push('/');
            self.chat_history_index = None;
            self.chat_history_draft = None;
        }
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
            .is_some_and(|sequence| sequence.bytes.is_empty());
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
            self.handle_input(TerminalAppInput::Key(UiKey::Esc));
        }
    }

    fn input_from_byte(&mut self, byte: u8) -> Option<TerminalAppInput> {
        if self.paste_mode {
            match byte {
                13 => {
                    self.skip_next_paste_lf = true;
                    return Some(TerminalAppInput::LineBreak);
                }
                10 if self.skip_next_paste_lf => {
                    self.skip_next_paste_lf = false;
                    return None;
                }
                10 => return Some(TerminalAppInput::LineBreak),
                _ => {
                    self.skip_next_paste_lf = false;
                }
            }
        }
        TerminalAppInput::from_byte(byte)
    }

    fn recall_prompt_history(&mut self, delta: isize) {
        if self.prompt_history.is_empty() {
            return;
        }

        if self.prompt_history_index.is_none() {
            self.prompt_history_draft = Some(self.prompt_buffer.as_str().to_string());
        }

        let current = self
            .prompt_history_index
            .unwrap_or(self.prompt_history.len()) as isize;
        let next = current + delta;
        if next < 0 {
            self.prompt_history_index = Some(0);
            self.prompt_buffer.replace(&self.prompt_history[0]);
        } else if next >= self.prompt_history.len() as isize {
            self.prompt_history_index = None;
            let draft = self.prompt_history_draft.take().unwrap_or_default();
            self.prompt_buffer.replace(&draft);
        } else {
            let next = next as usize;
            self.prompt_history_index = Some(next);
            self.prompt_buffer.replace(&self.prompt_history[next]);
        }
    }

    fn recall_chat_history(&mut self, delta: isize) {
        if self.chat_history.is_empty() {
            return;
        }

        if self.chat_history_index.is_none() {
            self.chat_history_draft = Some(self.chat_buffer.as_str().to_string());
        }

        let current = self.chat_history_index.unwrap_or(self.chat_history.len()) as isize;
        let next = current + delta;
        if next < 0 {
            self.chat_history_index = Some(0);
            self.chat_buffer.replace(&self.chat_history[0]);
        } else if next >= self.chat_history.len() as isize {
            self.chat_history_index = None;
            let draft = self.chat_history_draft.take().unwrap_or_default();
            self.chat_buffer.replace(&draft);
        } else {
            let next = next as usize;
            self.chat_history_index = Some(next);
            self.chat_buffer.replace(&self.chat_history[next]);
        }
    }

    fn request_quit(&mut self) {
        self.controller.shutdown();
        self.quit = true;
        self.dirty = true;
    }

    fn right_content(&self) -> String {
        if let Some(agent_id) = self.ui.selected_agent() {
            let session = self.snapshot.session(agent_id);
            let mut lines = session
                .map(|session| session.lines.clone())
                .unwrap_or_default();
            if let Some(loading) = session.and_then(|session| session.loading) {
                lines.push(format!(
                    "work-leaf: {} {}",
                    self.cached_loading_text(loading),
                    SPINNER[self.spinner]
                ));
            }
            return terminal_right_content(self.chat_buffer.as_str(), &lines);
        }
        terminal_right_content(self.chat_buffer.as_str(), &self.snapshot.command_transcript)
    }

    fn cached_loading_text(&self, loading: WorkLeafLoading) -> &str {
        self.loading_text
            .iter()
            .find(|(kind, _)| *kind == loading)
            .map(|(_, text)| text.as_str())
            .unwrap_or("Waiting for agent")
    }

    fn continue_escape_sequence(&mut self, byte: u8) -> bool {
        let Some(sequence) = self.escape_sequence.as_mut() else {
            return false;
        };

        if sequence.bytes.is_empty() && byte != b'[' {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
            return false;
        }

        sequence.bytes.push(byte);
        if is_complete_control_sequence(&sequence.bytes) {
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
            if let Some(input) = parse_control_sequence(&sequence.bytes) {
                self.handle_input(input);
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
}

#[cfg(test)]
type LoadingKind = WorkLeafLoading;

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingEscapeSequence {
    bytes: Vec<u8>,
    mode_before: UiMode,
    escape_dispatched: bool,
}

const SPINNER: [&str; 4] = ["|", "/", "-", "\\"];

const MAX_ESCAPE_SEQUENCE: usize = 64;

fn is_agent_slash_command(line: &str) -> bool {
    line.strip_prefix('/')
        .and_then(|rest| rest.chars().next())
        .is_some_and(|ch| !ch.is_whitespace())
}

fn is_complete_control_sequence(sequence: &[u8]) -> bool {
    sequence.len() > 1
        && sequence
            .last()
            .is_some_and(|byte| (0x40..=0x7e).contains(byte))
}

fn parse_control_sequence(sequence: &[u8]) -> Option<TerminalAppInput> {
    match sequence {
        [b'[', b'A'] => Some(TerminalAppInput::Key(UiKey::Up)),
        [b'[', b'B'] => Some(TerminalAppInput::Key(UiKey::Down)),
        [b'[', b'C'] => Some(TerminalAppInput::Key(UiKey::Right)),
        [b'[', b'D'] => Some(TerminalAppInput::Key(UiKey::Left)),
        [b'[', b'2', b'0', b'0', b'~'] => Some(TerminalAppInput::PasteStart),
        [b'[', b'2', b'0', b'1', b'~'] => Some(TerminalAppInput::PasteEnd),
        [b'[', b'1', b'3', b';', b'2', b'u']
        | [b'[', b'1', b'3', b';', b'2', b'~']
        | [b'[', b'2', b'7', b';', b'2', b';', b'1', b'3', b'~'] => {
            Some(TerminalAppInput::LineBreak)
        }
        _ => parse_sgr_mouse_event(sequence).map(TerminalAppInput::Key),
    }
}

fn parse_sgr_mouse_event(sequence: &[u8]) -> Option<UiKey> {
    let final_byte = *sequence.last()?;
    if !sequence.starts_with(b"[<") || !matches!(final_byte, b'M' | b'm') {
        return None;
    }

    let body = std::str::from_utf8(&sequence[2..sequence.len() - 1]).ok()?;
    let mut parts = body.split(';');
    let button = parts.next()?.parse::<u16>().ok()?;
    let column = parts.next()?.parse::<u16>().ok()?;
    let row = parts.next()?.parse::<u16>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    let button_kind = button & !0b0001_1100_u16;
    match (button_kind, final_byte) {
        (64, b'M') => Some(UiKey::MouseScrollUp { column, row }),
        (65, b'M') => Some(UiKey::MouseScrollDown { column, row }),
        (_, b'M' | b'm') if button_kind < 64 && button & 0b11 == 0 => {
            Some(UiKey::MouseClick { column, row })
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalAppInput {
    Key(UiKey),
    Char(char),
    Enter,
    Backspace,
    LineBreak,
    PasteStart,
    PasteEnd,
    Interrupt,
    Quit,
}

impl TerminalAppInput {
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            3 => Some(Self::Interrupt),
            4 => Some(Self::Quit),
            13 | 10 => Some(Self::Enter),
            27 => Some(Self::Key(UiKey::Esc)),
            23 => Some(Self::Key(UiKey::CtrlW)),
            8 | 127 => Some(Self::Backspace),
            byte if byte.is_ascii_graphic() || byte == b' ' => Some(Self::Char(byte as char)),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct PromptLine {
    buffer: LineBuffer,
}

impl PromptLine {
    const CAPACITY: usize = 64 * 1024;

    fn new() -> Self {
        Self {
            buffer: LineBuffer::with_capacity(Self::CAPACITY),
        }
    }

    fn as_str(&self) -> &str {
        self.buffer.as_str()
    }

    fn cursor(&self) -> usize {
        self.buffer.pos()
    }

    fn cursor_char_count(&self) -> usize {
        self.as_str()[..self.cursor()].chars().count()
    }

    fn trimmed_string(&self) -> String {
        self.as_str().trim().to_string()
    }

    fn push(&mut self, ch: char) {
        let mut listener = NoopLineListener;
        let _ = self.buffer.insert(ch, 1, &mut listener);
    }

    fn move_left(&mut self) {
        self.buffer.move_backward(1);
    }

    fn move_right(&mut self) {
        self.buffer.move_forward(1);
    }

    fn backspace(&mut self) {
        let mut listener = NoopLineListener;
        self.buffer.backspace(1, &mut listener);
    }

    fn clear(&mut self) {
        let mut listener = NoopLineListener;
        let len = self.buffer.as_str().len();
        self.buffer.replace(0..len, "", &mut listener);
    }

    fn replace(&mut self, text: &str) {
        let mut listener = NoopLineListener;
        let len = self.buffer.as_str().len();
        self.buffer.replace(0..len, text, &mut listener);
        self.buffer.move_end();
    }
}

#[derive(Debug)]
struct NoopLineListener;

impl DeleteListener for NoopLineListener {
    fn delete(&mut self, _idx: usize, _string: &str, _dir: Direction) {}
}

impl ChangeListener for NoopLineListener {
    fn insert_char(&mut self, _idx: usize, _c: char) {}

    fn insert_str(&mut self, _idx: usize, _string: &str) {}

    fn replace(&mut self, _idx: usize, _old: &str, _new: &str) {}
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use crate::agent::{
        AgentError, AgentKind, AgentLaunch, AgentSession, ChatMessage, MessageRole,
    };

    #[derive(Clone, Debug)]
    struct NoopBackend;

    impl AgentBackend for NoopBackend {
        fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
            Ok(AgentSession::new(request))
        }

        fn send(&mut self, _agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
            Ok(ChatMessage::new(MessageRole::Agent, prompt))
        }
    }

    #[derive(Debug)]
    struct CountingController {
        snapshot: crate::WorkLeafSnapshot,
        snapshot_calls: Arc<AtomicUsize>,
        drain_calls: Arc<AtomicUsize>,
    }

    impl CountingController {
        fn new(
            snapshot: crate::WorkLeafSnapshot,
            snapshot_calls: Arc<AtomicUsize>,
            drain_calls: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                snapshot,
                snapshot_calls,
                drain_calls,
            }
        }
    }

    impl TerminalController for CountingController {
        fn snapshot(&self) -> crate::WorkLeafSnapshot {
            self.snapshot_calls.fetch_add(1, Ordering::Relaxed);
            self.snapshot.clone()
        }

        fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
            self.drain_calls.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        }

        fn execute_command_line(&mut self, _line: &str) {}

        fn send_command_agent_message(&mut self, _message: &str) {}

        fn send_message(&mut self, _agent_id: &AgentId, _message: &str) {}

        fn interrupt_agent(&mut self, _agent_id: &AgentId) {}

        fn push_transcript_line(&mut self, _line: String) {}

        fn is_busy(&mut self) -> bool {
            false
        }

        fn loading_text(&self, _loading: WorkLeafLoading) -> String {
            "Waiting for Codex".to_string()
        }

        fn shutdown(&mut self) {}
    }

    #[test]
    fn pasted_command_prompt_input_polls_controller_once_per_chunk() {
        let snapshot_calls = Arc::new(AtomicUsize::new(0));
        let drain_calls = Arc::new(AtomicUsize::new(0));
        let controller = CountingController::new(
            crate::WorkLeafSnapshot {
                command_transcript: Vec::new(),
                sessions: Vec::new(),
            },
            Arc::clone(&snapshot_calls),
            Arc::clone(&drain_calls),
        );
        let mut app = TerminalAppCore::new(controller, 80, 24);
        app.handle_bytes(b":");
        drain_calls.store(0, Ordering::Relaxed);

        let paste = "a".repeat(4096);
        assert!(app.handle_bytes(paste.as_bytes()));

        assert_eq!(app.prompt_buffer.as_str(), paste);
        assert!(
            drain_calls.load(Ordering::Relaxed) <= 3,
            "large input chunks should not drain events once per byte"
        );
    }

    #[test]
    fn rendering_uses_cached_snapshot_instead_of_refetching_full_transcripts() {
        let snapshot_calls = Arc::new(AtomicUsize::new(0));
        let drain_calls = Arc::new(AtomicUsize::new(0));
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        let controller = CountingController::new(
            crate::WorkLeafSnapshot {
                command_transcript: vec!["help".to_string()],
                sessions: vec![WorkLeafSession {
                    id: agent_id,
                    kind: AgentKind::Codex,
                    title: "feature".to_string(),
                    feature: "feature".to_string(),
                    lines: vec!["large transcript line".repeat(256)],
                    loading: None,
                }],
            },
            Arc::clone(&snapshot_calls),
            Arc::clone(&drain_calls),
        );
        let app = TerminalAppCore::new(controller, 80, 24);
        snapshot_calls.store(0, Ordering::Relaxed);

        assert!(app.render_frame().contains("help"));
        assert!(app.render_frame().contains("help"));

        assert_eq!(
            snapshot_calls.load(Ordering::Relaxed),
            0,
            "rendering and scrolling should use the local snapshot cache"
        );
    }

    #[test]
    fn clearing_agent_loading_marks_chat_ready_in_left_pane() {
        let chat = CommandChat::new(PathBuf::from("."), NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");

        app.inner
            .ui
            .add_agent(AgentListEntry::new(agent_id.clone(), "feature"));
        app.inner
            .ui
            .activate_agent_chat(&agent_id)
            .expect("test agent is registered");
        app.set_agent_loading(&agent_id, Some(LoadingKind::WaitingForReply));

        assert!(!app.render_frame().contains('\u{7}'));
        assert!(!app.ui().render_left_pane().contains("READY"));

        app.clear_agent_loading(&agent_id);

        assert!(app.render_frame().starts_with('\u{7}'));
        assert!(!app.render_frame().contains('\u{7}'));
        assert!(
            app.ui()
                .render_left_pane()
                .contains("\u{1b}[7m>feature user-1  working: feature  READY\u{1b}[0m")
        );
    }

    #[test]
    fn command_surface_insert_mode_renders_chat_buffer() {
        let chat = CommandChat::new(PathBuf::from("."), NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);

        app.handle_bytes(b"itype in command agent");

        assert!(app.render_frame().contains("type in command agent"));
    }

    #[test]
    fn command_surface_chat_uses_command_agent_to_spawn_codex_agent() {
        let root =
            std::env::temp_dir().join(format!("work-leaf-command-surface-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let chat = CommandChat::new(root, NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);

        assert!(app.ui().selected_agent().is_none());

        app.handle_bytes(b"ispawn a new patch agent that uses codex\n");

        assert!(app.wait_for_idle(Duration::from_secs(1)));
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        assert_eq!(app.ui().selected_agent(), Some(&agent_id));
        assert!(app.transcript().iter().any(|line| line
            == "command-agent: launching Codex user agent for patch agent that uses codex"));
        assert!(
            app.transcript()
                .iter()
                .any(|line| line == "work-leaf> new patch agent that uses codex")
        );
    }
}
----- END FILE src/terminal_app.rs -----

----- BEGIN FILE tests/reviews.rs -----
digest: fnv64:ea641f4f0e2f1719; bytes:6738

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, ChatMessage,
    GitHistory, MessageRole, ReviewCoordinator,
};

#[test]
fn git_history_finds_latest_commit_for_each_agent_id() {
    let root = git_repo("review-history");
    fs::write(root.join("one.txt"), "one\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply parser patch from chat-a",
            "-m",
            "Agent-ID: chat-a\nFeature: parser\nReason: first",
        ],
    );
    fs::write(root.join("two.txt"), "two\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply docs patch from chat-b",
            "-m",
            "Agent-ID: chat-b\nFeature: docs\nReason: docs",
        ],
    );
    fs::write(root.join("one.txt"), "one again\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply parser patch from chat-a",
            "-m",
            "Agent-ID: chat-a\nFeature: parser\nReason: second",
        ],
    );

    let commits = GitHistory::new(root).latest_agent_commits().unwrap();

    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].agent_id.as_str(), "chat-a");
    assert_eq!(commits[0].reason, "second");
    assert_eq!(commits[1].agent_id.as_str(), "chat-b");
    assert_eq!(commits[1].feature, "docs");
}

#[test]
fn review_coordinator_loops_until_reviewer_reports_no_findings() {
    let root = git_repo("review-loop");
    fs::write(root.join("lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply parser patch from chat-a",
            "-m",
            "Agent-ID: chat-a\nFeature: parser\nReason: return value",
        ],
    );

    let backend = FakeBackend::new([
        "summary: returns a parser value",
        "FINDINGS\n- missing edge case",
        "fixed missing edge case",
        "NO_FINDINGS",
    ]);
    let mut coordinator = ReviewCoordinator::new(root, backend).with_max_rounds(4);

    let results = coordinator.review_latest_agent_commits().unwrap();
    let backend = coordinator.into_backend();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].agent_id.as_str(), "chat-a");
    assert_eq!(results[0].reviewer_id.as_str(), "review-chat-a");
    assert_eq!(results[0].rounds, 2);
    assert!(results[0].findings_resolved);

    assert_eq!(backend.launches.len(), 1);
    assert_eq!(backend.launches[0].id.as_str(), "review-chat-a");
    assert_eq!(backend.launches[0].kind, AgentKind::Codex);
    assert!(
        backend.launches[0]
            .prompt
            .contains("summary: returns a parser value")
    );
    assert!(backend.launches[0].prompt.contains("chat-a"));

    assert_eq!(backend.sends.len(), 3);
    assert_eq!(backend.sends[0].0.as_str(), "chat-a");
    assert!(backend.sends[0].1.contains("summarize"));
    assert_eq!(backend.sends[1].0.as_str(), "chat-a");
    assert!(backend.sends[1].1.contains("missing edge case"));
    assert_eq!(backend.sends[2].0.as_str(), "review-chat-a");
    assert!(backend.sends[2].1.contains("check the patch again"));
}

#[test]
fn git_history_builds_agent_review_scope_since_baseline() {
    let root = git_repo("review-history-scope");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial review scope fixture"]);
    let baseline = GitHistory::new(root.clone()).head_hash().unwrap().unwrap();
    fs::write(root.join("README.md"), "after first\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply first patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: readme\nReason: first step\nContext: first context",
        ],
    );
    fs::write(root.join("README.md"), "after second\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply second patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: readme\nReason: second step\nContext: second context",
        ],
    );

    let target = GitHistory::new(root)
        .agent_review_commit(&AgentId::new("user-1").unwrap(), Some(&baseline))
        .unwrap()
        .expect("review target");

    assert!(target.reason.contains("2 provisional commits"));
    assert!(target.context.contains("first step"));
    assert!(target.context.contains("second step"));
    assert_eq!(target.feature, "readme");
}

#[derive(Debug)]
struct FakeBackend {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
}

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            replies: replies.into_iter().map(String::from).collect(),
            launches: Vec::new(),
            sends: Vec::new(),
        }
    }

    fn next_reply(&mut self) -> String {
        self.replies.pop_front().expect("missing fake reply")
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let reply = self.next_reply();
        self.launches.push(request.clone());
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, reply);
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.sends.push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(MessageRole::Agent, self.next_reply()))
    }
}

fn git_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    git(&root, ["init"]);
    git(&root, ["config", "user.name", "Work Leaf Test"]);
    git(&root, ["config", "user.email", "work-leaf@example.test"]);
    root
}

fn git<const N: usize>(root: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
----- END FILE tests/reviews.rs -----

----- BEGIN FILE tests/workspace.rs -----
digest: fnv64:60e65a59fbc85f3a; bytes:28957

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentProfile, AgentSession,
    ChatMessage, CommandChat, MessageRole, WorkLeafController, WorkLeafEvent, WorkLeafLoading,
};

#[test]
fn controller_exposes_ui_neutral_events_and_snapshot_without_terminal_ui() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("implement parser combinator")
        .unwrap();

    assert_eq!(agent_id, AgentId::new("user-1").unwrap());
    assert!(controller.drain_events().iter().any(|event| {
        matches!(event, WorkLeafEvent::AgentAdded { session } if session.id == agent_id)
    }));
    let starting = controller.snapshot();
    let session = starting.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "user-agent");
    assert_eq!(session.loading, Some(WorkLeafLoading::Launching));

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let ready = controller.snapshot();
    let session = ready.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "parser-combinator");
    assert_eq!(session.loading, None);
    assert!(session.lines.iter().any(|line| line == "launch reply"));

    controller.send_message(&agent_id, "continue").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let replied = controller.snapshot();
    let session = replied.session(&agent_id).expect("session exists");
    assert!(session.lines.iter().any(|line| line == "user: continue"));
    assert!(session.lines.iter().any(|line| line == "follow reply"));
}

#[test]
fn controller_line_events_do_not_resend_full_session_transcripts() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("stream compactly").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let events = controller.drain_events();
    assert!(events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentLineAppended { agent_id: id, line }
                if id == &agent_id && line == "launch reply"
        )
    }));
    assert!(
        !events.iter().any(|event| {
            matches!(
                event,
                WorkLeafEvent::AgentUpdated { session }
                    if session.id == agent_id
                        && session.lines.iter().any(|line| line == "launch reply")
            )
        }),
        "line append events should not be paired with full-session transcript updates"
    );
}

#[test]
fn controller_status_events_do_not_resend_existing_large_transcripts() {
    let large_reply = "large transcript line\n".repeat(8192);
    let backend = FakeBackend::new([large_reply.as_str(), "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("keep status compact").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.drain_events();

    controller.send_message(&agent_id, "continue").unwrap();
    let waiting_events = controller.drain_events();
    assert!(waiting_events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentStatusUpdated {
                agent_id: id,
                loading: Some(WorkLeafLoading::WaitingForReply),
                ..
            } if id == &agent_id
        )
    }));
    assert!(
        !waiting_events.iter().any(|event| {
            matches!(
                event,
                WorkLeafEvent::AgentUpdated { session }
                    if session.id == agent_id
                        && session.lines.iter().any(|line| line == &large_reply)
            )
        }),
        "status changes should not serialize existing transcript text"
    );

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let ready_events = controller.drain_events();
    assert!(ready_events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentStatusUpdated {
                agent_id: id,
                loading: None,
                ..
            } if id == &agent_id
        )
    }));
    assert!(ready_events.iter().any(|event| {
        matches!(
            event,
            WorkLeafEvent::AgentLineAppended { agent_id: id, line }
                if id == &agent_id && line == "follow reply"
        )
    }));
    assert!(
        !ready_events.iter().any(|event| {
            matches!(
                event,
                WorkLeafEvent::AgentUpdated { session }
                    if session.id == agent_id
                        && session.lines.iter().any(|line| line == &large_reply)
            )
        }),
        "ready status changes should not serialize existing transcript text"
    );
}

#[test]
fn controller_uses_backend_agent_to_name_chat_from_first_prompt() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("please fix login callback")
        .unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.title, "oauth-redirect-handler");
    assert!(session.lines.iter().any(|line| line == "launch reply"));

    let launches = backend.launches();
    assert!(launches.iter().any(|launch| {
        launch.id.as_str() == "title-user-1"
            && launch.feature == "chat-title"
            && launch.prompt.contains("please fix login callback")
    }));
}

#[test]
fn controller_uses_agent_profile_for_non_codex_launches_and_reviews() {
    let root = git_repo("workspace-custom-profile-review");
    fs::write(root.join("README.md"), "fixture\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply parser patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: parser\nReason: parse configs\nContext: parser context",
        ],
    );
    let backend = FakeBackend::new(["launch reply", "summary", "NO_FINDINGS"]);
    let profile = AgentProfile::new(
        AgentKind::External("local-test-agent".to_string()),
        "Local Test Agent",
        "local-agent",
    );
    let chat = CommandChat::new(root, backend.clone()).with_agent_profile(profile.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("build custom provider path")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));
    controller.start_review().unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    let launches = backend.launches();
    assert!(launches.iter().any(|launch| {
        launch.id == agent_id
            && launch.kind == profile.kind
            && launch.feature == profile.default_feature
    }));
    assert!(
        launches
            .iter()
            .any(|launch| { launch.id.as_str() == "review-user-1" && launch.kind == profile.kind })
    );
}

#[test]
fn controller_starts_review_after_patch_agent_done_and_loops_until_clean() {
    let root = git_repo("workspace-automatic-review-loop");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: README changes from before to after",
        "FINDINGS\n- missing reviewed wording",
        "fixed review finding\n@work-leaf patch address review\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after\n+after review\n@work-leaf end\n@work-leaf done",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after review\n"
    );

    let reviewer_id = AgentId::new("review-user-1").unwrap();
    let snapshot = controller.snapshot();
    let reviewer = snapshot
        .session(&reviewer_id)
        .expect("reviewer session exists");
    assert_eq!(reviewer.title, "review user-agent");
    assert_eq!(reviewer.loading, None);
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert!(
        patch_agent
            .lines
            .iter()
            .any(|line| line.contains("missing reviewed wording"))
    );
    assert!(
        patch_agent.lines.iter().any(|line| {
            line.contains("user-1 reviewed by review-user-1: rounds=2 resolved=yes")
        })
    );

    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &agent_id
            && prompt.contains("missing reviewed wording")
            && prompt.contains("Please fix the patch")
    }));
    assert!(sends.iter().any(|(target, prompt)| {
        target == &reviewer_id
            && prompt.contains("The original agent has responded to the findings")
            && prompt.contains("Please check the patch again")
    }));
}

#[test]
fn controller_does_not_start_review_until_patch_agent_reports_done() {
    let root = git_repo("workspace-review-waits-for-done");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end",
        "summary that should not be requested",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let snapshot = controller.snapshot();
    assert!(
        snapshot
            .session(&AgentId::new("review-user-1").unwrap())
            .is_none(),
        "review must wait for the patch agent to report done"
    );
    let patch_agent = snapshot.session(&agent_id).expect("patch agent exists");
    assert_eq!(patch_agent.loading, None);
    let launches = backend.launches();
    assert!(
        !launches
            .iter()
            .any(|launch| launch.id.as_str() == "review-user-1")
    );
    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &agent_id
            && prompt.contains("work-leaf patch applied")
            && prompt.contains("@work-leaf done")
    }));
}

#[test]
fn controller_review_prompt_covers_all_agent_commits_since_launch() {
    let root = git_repo("workspace-review-full-agent-scope");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch first step\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end",
        "second patch\n@work-leaf patch second step\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: full change",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller
        .create_agent("update readme in two steps")
        .unwrap();

    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    let launches = backend.launches();
    let review_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "review-user-1")
        .expect("reviewer launched");
    assert!(
        review_launch.prompt.contains("first step"),
        "{}",
        review_launch.prompt
    );
    assert!(
        review_launch.prompt.contains("second step"),
        "{}",
        review_launch.prompt
    );
}

#[test]
fn controller_reuses_one_reviewer_for_repeated_patch_agent_iterations() {
    let root = git_repo("workspace-reuses-reviewer");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end\n@work-leaf done",
        "summary: README changes to after first",
        "NO_FINDINGS",
        "second patch\n@work-leaf patch update readme again\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: README changes to after second",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    controller
        .send_message(&agent_id, "make the second update")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after second\n"
    );
    let reviewer_id = AgentId::new("review-user-1").unwrap();
    let launches = backend.launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id == reviewer_id)
            .count(),
        1
    );
    let sends = backend.sends();
    assert!(sends.iter().any(|(target, prompt)| {
        target == &reviewer_id
            && prompt.contains("Review the full patch scope")
            && prompt.contains("after second")
    }));
}

#[test]
fn controller_reviews_only_unreviewed_patch_agent_commits() {
    let root = git_repo("workspace-reviews-only-unreviewed");
    fs::write(root.join("README.md"), "readme before\n").unwrap();
    fs::write(root.join("CHANGELOG.md"), "changelog before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial docs fixture"]);
    let backend = FakeBackend::new([
        "readme patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-readme before\n+readme after\n@work-leaf end\n@work-leaf done",
        "summary: README changes",
        "NO_FINDINGS",
        "changelog patch\n@work-leaf patch update changelog\n--- a/CHANGELOG.md\n+++ b/CHANGELOG.md\n@@ -1 +1 @@\n-changelog before\n+changelog after\n@work-leaf end\n@work-leaf done",
        "summary: changelog changes",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let first = controller.create_agent("update readme").unwrap();
    assert_eq!(first.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let second = controller.create_agent("update changelog").unwrap();
    assert_eq!(second.as_str(), "user-2");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str() == "review-user-1")
            .count(),
        1
    );
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str() == "review-user-2")
            .count(),
        1
    );
}

#[test]
fn controller_auto_review_ignores_historical_agents_outside_current_patch_agent() {
    let root = git_repo("workspace-auto-review-current-agent-only");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    for old_agent in ["user-2", "user-3"] {
        fs::write(
            root.join("README.md"),
            format!("historical commit from {old_agent}\n"),
        )
        .unwrap();
        git(&root, ["add", "README.md"]);
        git(
            &root,
            [
                "commit",
                "-m",
                "UPDATE apply historical patch",
                "-m",
                &format!(
                    "Agent-ID: {old_agent}\nFeature: historical\nReason: previous work\nContext: old patch"
                ),
            ],
        );
    }
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "UPDATE reset live fixture"]);
    let backend = FakeBackend::new([
        "live patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: live README change",
        "NO_FINDINGS",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    assert_eq!(
        launches
            .iter()
            .filter(|launch| launch.id.as_str().starts_with("review-"))
            .map(|launch| launch.id.as_str().to_string())
            .collect::<Vec<_>>(),
        vec!["review-user-1".to_string()]
    );
}

#[test]
fn controller_linearize_uses_only_commits_reviewed_in_this_session() {
    let root = git_repo("workspace-linearize-current-reviewed-only");
    fs::write(root.join("README.md"), "before\n").unwrap();
    fs::write(root.join("legacy.txt"), "legacy before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial linearize fixture"]);
    fs::write(root.join("legacy.txt"), "legacy after\n").unwrap();
    git(&root, ["add", "legacy.txt"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply legacy patch from user-2",
            "-m",
            "Agent-ID: user-2\nFeature: legacy\nReason: old run\nContext: old session",
        ],
    );
    let backend = FakeBackend::new([
        "live patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
        "summary: live README change",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert!(controller.start_linearize().unwrap().is_some());
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    let linearize_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "linearize")
        .expect("linearize agent launched");
    assert!(linearize_launch.prompt.contains("Agent-ID: user-1"));
    assert!(linearize_launch.prompt.contains("Commit:"));
    assert!(!linearize_launch.prompt.contains("Agent-ID: user-2"));
    assert!(!linearize_launch.prompt.contains("old session"));
}

#[test]
fn controller_linearize_keeps_multiple_reviewed_commits_from_same_agent() {
    let root = git_repo("workspace-linearize-same-agent-multiple-reviewed");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "first patch\n@work-leaf patch update readme once\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after first\n@work-leaf end\n@work-leaf done",
        "summary: first reviewed change",
        "NO_FINDINGS",
        "second patch\n@work-leaf patch update readme twice\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-after first\n+after second\n@work-leaf end\n@work-leaf done",
        "summary: second reviewed change",
        "NO_FINDINGS",
        "linearizer ready",
    ]);
    let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("update readme").unwrap();
    assert_eq!(agent_id.as_str(), "user-1");
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    controller
        .send_message(&agent_id, "make a second reviewed update")
        .unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert!(controller.start_linearize().unwrap().is_some());
    assert!(controller.wait_for_idle(Duration::from_secs(2)));

    let launches = backend.launches();
    let linearize_launch = launches
        .iter()
        .find(|launch| launch.id.as_str() == "linearize")
        .expect("linearize agent launched");
    assert!(
        linearize_launch
            .prompt
            .contains("Reason: update readme once"),
        "{}",
        linearize_launch.prompt
    );
    assert!(
        linearize_launch
            .prompt
            .contains("Reason: update readme twice"),
        "{}",
        linearize_launch.prompt
    );
    assert_eq!(
        linearize_launch.prompt.matches("Agent-ID: user-1").count(),
        2
    );
}

#[test]
fn controller_does_not_run_project_required_checks_after_agent_reply() {
    let root = git_repo("workspace-no-required-check-run");
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\n- `sh check.sh`\n",
    )
    .unwrap();
    fs::write(
        root.join("check.sh"),
        "#!/bin/sh\necho state is bad\nexit 1\n",
    )
    .unwrap();
    fs::write(root.join("state.txt"), "bad\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD project instruction fixture"]);
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(root.clone(), backend.clone());
    let mut controller = WorkLeafController::new(chat);

    let agent_id = controller.create_agent("inspect required checks").unwrap();

    assert!(controller.wait_for_idle(Duration::from_secs(2)));
    assert_eq!(fs::read_to_string(root.join("state.txt")).unwrap(), "bad\n");
    let snapshot = controller.snapshot();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.loading, None);
    assert!(
        !session
            .lines
            .iter()
            .any(|line| line.contains("required check failed"))
    );
    assert!(backend.sends().is_empty());
}

#[test]
fn controller_keeps_agent_loading_scoped_to_the_active_session() {
    let backend = ConcurrentBackend;
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut controller = WorkLeafController::new(chat);

    let first = controller.create_agent("first task").unwrap();
    let second = controller.create_agent("second task").unwrap();
    assert!(controller.wait_for_idle(Duration::from_secs(1)));

    controller.send_message(&second, "slow question").unwrap();
    assert_eq!(
        controller
            .snapshot()
            .session(&second)
            .expect("second session")
            .loading,
        Some(WorkLeafLoading::WaitingForReply)
    );

    controller.send_message(&first, "quick question").unwrap();

    assert!(controller.wait_for_session_line(&first, "quick reply", Duration::from_millis(150)));
    let snapshot = controller.snapshot();
    let first_session = snapshot.session(&first).expect("first session");
    assert!(first_session.lines.iter().any(|line| line == "quick reply"));
    assert!(
        !first_session
            .lines
            .iter()
            .any(|line| line.contains("still working"))
    );
}

#[derive(Clone, Debug)]
struct FakeBackend {
    state: Arc<Mutex<FakeBackendState>>,
}

#[derive(Debug)]
struct FakeBackendState {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
}

#[derive(Clone, Debug)]
struct ConcurrentBackend;

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeBackendState {
                replies: replies.into_iter().map(String::from).collect(),
                launches: Vec::new(),
                sends: Vec::new(),
            })),
        }
    }

    fn launches(&self) -> Vec<AgentLaunch> {
        self.state.lock().unwrap().launches.clone()
    }

    fn sends(&self) -> Vec<(AgentId, String)> {
        self.state.lock().unwrap().sends.clone()
    }

    fn next_reply(&self) -> String {
        self.state
            .lock()
            .unwrap()
            .replies
            .pop_front()
            .expect("missing fake reply")
    }

    fn title_reply(&self, prompt: &str) -> String {
        fake_title_from_title_prompt(prompt)
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.state.lock().unwrap().launches.push(request.clone());
        let mut session = AgentSession::new(request);
        let reply = if session.id.as_str().starts_with("title-") {
            self.title_reply(&session.messages[0].text)
        } else {
            self.next_reply()
        };
        session.push_message(MessageRole::Agent, reply);
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.state
            .lock()
            .unwrap()
            .sends
            .push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(MessageRole::Agent, self.next_reply()))
    }
}

impl AgentBackend for ConcurrentBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str() == "user-2" {
            thread::sleep(Duration::from_millis(350));
            return Ok(ChatMessage::new(MessageRole::Agent, "slow reply"));
        }
        Ok(ChatMessage::new(MessageRole::Agent, "quick reply"))
    }
}

fn git_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    git(&root, ["init", "-q"]);
    git(&root, ["config", "user.email", "test@example.com"]);
    git(&root, ["config", "user.name", "Test User"]);
    root
}

fn git<const N: usize>(root: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn fake_title_from_title_prompt(prompt: &str) -> String {
    let first_prompt = prompt
        .rsplit_once("First prompt:\n")
        .map(|(_, first_prompt)| first_prompt)
        .unwrap_or(prompt);
    if first_prompt.contains("parser combinator") {
        "parser-combinator".to_string()
    } else if first_prompt.contains("login callback")
        || first_prompt.contains("OAuth redirect handler")
    {
        "oauth-redirect-handler".to_string()
    } else {
        "chat-title".to_string()
    }
}
----- END FILE tests/workspace.rs -----
