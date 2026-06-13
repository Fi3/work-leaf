use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use crate::agent::{
    AgentBackend, AgentId, AgentLaunch, AgentProfile, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, ChatMessage, PromptPolicy, ReadPermission,
};
use crate::chat_title::{chat_title_from_agent_reply, chat_title_prompt};
use crate::claude::{ClaudeBackend, ClaudeCommandConfig};
use crate::codex::{CodexBackend, CodexCommandConfig, SandboxMode};
use crate::linearize::{LinearizePlanner, LinearizeQuestion};
use crate::locks::{CommandWritePolicy, FileLockTable};
use crate::orchestrator::{
    AgentFollowUp, CommandChangeTracker, ContextBundleStore, DirectiveServices,
    DirectiveStreamInterruptDetector, FileReadTracker, OrchestratorEvent, PatchOwnershipTracker,
    handle_agent_directives_streaming, send_agent_streaming_interruptible,
};
use crate::review::{AgentCommit, has_no_findings, render_review_source_context};
use crate::review::{GitHistory, ReviewResult};
use crate::terminal_app::{RemoteTerminalApp, TerminalApp};
use crate::ui::{UiAction, chat_content_from_transcript};
use crate::{HttpControllerClient, OrchestratorHttpError, WorkLeafSnapshot};

const DEFAULT_NEW_AGENT_PROMPT: &str = "Start a new work-leaf user-agent session. Ask the user what to work on if the task is not already clear, then report the broad feature before proposing patches.";
const COMMAND_AGENT_ID: &str = "command-agent";
const TITLE_AGENT_ID: &str = "title-agent";
pub(crate) const COMMAND_AGENT_TRANSCRIPT_LIMIT: usize = 40;

fn launch_agent_streaming_interruptible<B>(
    backend: &mut B,
    launch: AgentLaunch,
    stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
) -> Result<AgentSession, crate::agent::AgentError>
where
    B: AgentBackend,
{
    let agent_id = launch.id.clone();
    let mut detector = DirectiveStreamInterruptDetector::default();
    let mut sink = |event| stream(&agent_id, event);
    let mut should_interrupt = |event: &AgentStreamEvent| detector.observe(event);
    backend.launch_streaming_interruptible(launch, &mut sink, &mut should_interrupt)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum SelectedAgent {
    #[default]
    Codex,
    Claude,
}

impl SelectedAgent {
    pub fn parse(value: &str) -> Result<Self, CliError> {
        match value {
            "codex" => Ok(Self::Codex),
            "claude" => Ok(Self::Claude),
            _ => Err(CliError::Usage(format!(
                "unsupported agent `{value}`; expected `codex` or `claude`"
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    pub fn profile(&self) -> AgentProfile {
        match self {
            Self::Codex => AgentProfile::codex(),
            Self::Claude => AgentProfile::claude(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessCommand {
    Help,
    Daemon {
        model: Option<String>,
        read_permission: ReadPermission,
        agent: SelectedAgent,
    },
    Launch {
        model: Option<String>,
        read_permission: ReadPermission,
        agent: SelectedAgent,
        cli_url: Option<String>,
    },
}

pub fn parse_process_args<I, S>(args: I) -> Result<ProcessCommand, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args
        .first()
        .is_some_and(|arg| arg.ends_with("work-leaf") || arg.ends_with("work-leaf.exe"))
    {
        args.remove(0);
    }

    if args.is_empty() {
        return Ok(ProcessCommand::Launch {
            model: None,
            read_permission: ReadPermission::Orchestrator,
            agent: SelectedAgent::default(),
            cli_url: None,
        });
    }

    let mut model = None;
    let mut read_permission = ReadPermission::Orchestrator;
    let mut agent = SelectedAgent::default();
    let mut daemon = false;
    let mut cli_url = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" | "help" => return Ok(ProcessCommand::Help),
            "-d" | "--deamon" | "--daemon" => {
                daemon = true;
                index += 1;
            }
            "-c" | "--cli" => {
                if index + 1 >= args.len() {
                    return Err(CliError::Usage(format!("{} requires a value", args[index])));
                }
                cli_url = Some(args[index + 1].clone());
                index += 2;
            }
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
            "-a" | "--agent" => {
                if index + 1 >= args.len() {
                    return Err(CliError::Usage(format!("{} requires a value", args[index])));
                }
                agent = SelectedAgent::parse(&args[index + 1])?;
                index += 2;
            }
            "new"
            | "patch"
            | "review"
            | "linearize"
            | "force-linearize"
            | "linearize-questions"
            | "locks" => {
                return Err(CliError::Usage(
                    "work-leaf does not accept top-level workflow commands; start work-leaf and use the command chat".to_string(),
                ));
            }
            other => return Err(CliError::Usage(format!("unknown option `{other}`"))),
        }
    }

    if daemon {
        if cli_url.is_some() {
            return Err(CliError::Usage(
                "-d/--deamon cannot be combined with -c/--cli".to_string(),
            ));
        }
        return Ok(ProcessCommand::Daemon {
            model,
            read_permission,
            agent,
        });
    }

    Ok(ProcessCommand::Launch {
        model,
        read_permission,
        agent,
        cli_url,
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
            agent,
            cli_url,
        } => {
            let result = if let Some(cli_url) = cli_url {
                run_http_cli_with_url(cli_url)
            } else if env::var_os("WORK_LEAF_IN_PROCESS").is_some() {
                run_in_process_cli(model, read_permission, agent)
            } else {
                run_http_cli(model, read_permission, agent)
            };
            if let Err(error) = result {
                eprintln!("{error}");
                process::exit(1);
            }
            process::exit(0);
        }
        ProcessCommand::Daemon {
            model,
            read_permission,
            agent,
        } => {
            if let Err(error) = run_daemon_cli(model, read_permission, agent) {
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
    agent: SelectedAgent,
) -> Result<(), CliError> {
    let project_dir = env::current_dir()?;
    let profile = agent.profile();
    let backend = selected_agent_backend(project_dir.clone(), model, read_permission, agent)?;
    let chat = CommandChat::new(project_dir, backend).with_agent_profile(profile);
    run_command_chat(chat)
}

fn run_http_cli(
    model: Option<String>,
    read_permission: ReadPermission,
    agent: SelectedAgent,
) -> Result<(), CliError> {
    let project_dir = env::current_dir()?;
    if let Ok(url) = env::var("WORK_LEAF_ORCHESTRATOR_URL") {
        let client = HttpControllerClient::connect(url).map_err(http_cli_error)?;
        return run_http_command_chat(client, project_dir);
    }

    let daemon =
        ManagedInProcessOrchestrator::start(project_dir.clone(), model, read_permission, agent)?;
    run_http_command_chat(daemon.client(), project_dir)
}

fn run_http_cli_with_url(url: String) -> Result<(), CliError> {
    let project_dir = env::current_dir()?;
    let client = HttpControllerClient::connect(url).map_err(http_cli_error)?;
    run_http_command_chat(client, project_dir)
}

fn run_daemon_cli(
    model: Option<String>,
    read_permission: ReadPermission,
    agent: SelectedAgent,
) -> Result<(), CliError> {
    let project_dir = env::current_dir()?;
    let profile = agent.profile();
    let backend = selected_agent_backend(project_dir.clone(), model, read_permission, agent)?;
    let chat = CommandChat::new(project_dir, backend).with_agent_profile(profile);
    let controller = crate::WorkLeafController::new(chat);
    let api_server = crate::http_controller::HttpControllerServer::bind_api_only("127.0.0.1:0")
        .map_err(http_cli_error)?;
    let web_server =
        crate::http_controller::WebUiServer::bind("127.0.0.1:0").map_err(http_cli_error)?;
    let api_url = api_server.local_url().map_err(http_cli_error)?;
    let web_url = web_server.local_url().map_err(http_cli_error)?;
    let web_ui_url = daemon_web_ui_url(&web_url, &api_url);
    let web_shutdown = Arc::new(AtomicBool::new(false));
    let web_thread_shutdown = Arc::clone(&web_shutdown);
    let web_thread = thread::spawn(move || web_server.serve_until_shutdown(web_thread_shutdown));

    print!("{}", render_split_daemon_startup(&api_url, &web_ui_url));
    io::stdout().flush()?;
    let api_result = api_server.serve(controller).map_err(http_cli_error);
    web_shutdown.store(true, Ordering::SeqCst);
    let web_result = web_thread
        .join()
        .map_err(|_| CliError::Io(io::Error::other("web UI server thread panicked")))?;
    api_result?;
    web_result.map_err(http_cli_error)
}

pub fn render_daemon_startup(api_url: &str) -> String {
    let api_url = api_url.trim_end_matches('/');
    render_split_daemon_startup(api_url, &format!("{api_url}/web-ui"))
}

fn render_split_daemon_startup(api_url: &str, web_ui_url: &str) -> String {
    let api_url = api_url.trim_end_matches('/');
    let web_ui_url = web_ui_url.trim_end_matches('/');
    format!("http api at: {api_url}\nweb-ui at: {web_ui_url}\n")
}

fn daemon_web_ui_url(web_url: &str, api_url: &str) -> String {
    let web_url = web_url.trim_end_matches('/');
    let api_url = api_url.trim_end_matches('/');
    format!("{web_url}?api={api_url}")
}

struct ManagedInProcessOrchestrator {
    client: HttpControllerClient,
    thread: Option<thread::JoinHandle<Result<(), OrchestratorHttpError>>>,
}

impl ManagedInProcessOrchestrator {
    fn start(
        project_dir: PathBuf,
        model: Option<String>,
        read_permission: ReadPermission,
        agent: SelectedAgent,
    ) -> Result<Self, CliError> {
        let profile = agent.profile();
        let backend = selected_agent_backend(project_dir.clone(), model, read_permission, agent)?;
        let chat = CommandChat::new(project_dir, backend).with_agent_profile(profile);
        let controller = crate::WorkLeafController::new(chat);
        let server = crate::HttpControllerServer::bind("127.0.0.1:0").map_err(http_cli_error)?;
        let url = server.local_url().map_err(http_cli_error)?;
        let thread = thread::spawn(move || server.serve(controller));
        let client = HttpControllerClient::connect(url).map_err(http_cli_error)?;
        Ok(Self {
            client,
            thread: Some(thread),
        })
    }

    fn client(&self) -> HttpControllerClient {
        self.client.clone()
    }
}

impl Drop for ManagedInProcessOrchestrator {
    fn drop(&mut self) {
        let mut client = self.client.clone();
        let _ = client.shutdown();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn http_cli_error(error: OrchestratorHttpError) -> CliError {
    CliError::Io(io::Error::other(error.to_string()))
}

#[derive(Clone, Debug)]
pub(crate) enum SelectedAgentBackend {
    Codex(CodexBackend),
    Claude(ClaudeBackend),
}

impl AgentBackend for SelectedAgentBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, crate::agent::AgentError> {
        match self {
            Self::Codex(backend) => backend.launch(request),
            Self::Claude(backend) => backend.launch(request),
        }
    }

    fn send(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
    ) -> Result<ChatMessage, crate::agent::AgentError> {
        match self {
            Self::Codex(backend) => backend.send(agent_id, prompt),
            Self::Claude(backend) => backend.send(agent_id, prompt),
        }
    }

    fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        match self {
            Self::Codex(backend) => backend.session(agent_id),
            Self::Claude(backend) => backend.session(agent_id),
        }
    }

    fn interrupt(&mut self, agent_id: &AgentId) -> Result<(), crate::agent::AgentError> {
        match self {
            Self::Codex(backend) => backend.interrupt(agent_id),
            Self::Claude(backend) => backend.interrupt(agent_id),
        }
    }

    fn shutdown_handle(&self) -> AgentShutdownHandle {
        match self {
            Self::Codex(backend) => backend.shutdown_handle(),
            Self::Claude(backend) => backend.shutdown_handle(),
        }
    }

    fn shutdown(&mut self) {
        match self {
            Self::Codex(backend) => backend.shutdown(),
            Self::Claude(backend) => backend.shutdown(),
        }
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, crate::agent::AgentError> {
        match self {
            Self::Codex(backend) => backend.launch_streaming(request, sink),
            Self::Claude(backend) => backend.launch_streaming(request, sink),
        }
    }

    fn launch_streaming_interruptible(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<AgentSession, crate::agent::AgentError> {
        match self {
            Self::Codex(backend) => {
                backend.launch_streaming_interruptible(request, sink, should_interrupt)
            }
            Self::Claude(backend) => {
                backend.launch_streaming_interruptible(request, sink, should_interrupt)
            }
        }
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, crate::agent::AgentError> {
        match self {
            Self::Codex(backend) => backend.send_streaming(agent_id, prompt, sink),
            Self::Claude(backend) => backend.send_streaming(agent_id, prompt, sink),
        }
    }

    fn send_streaming_interruptible(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, crate::agent::AgentError> {
        match self {
            Self::Codex(backend) => {
                backend.send_streaming_interruptible(agent_id, prompt, sink, should_interrupt)
            }
            Self::Claude(backend) => {
                backend.send_streaming_interruptible(agent_id, prompt, sink, should_interrupt)
            }
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
    context_bundles: ContextBundleStore,
    command_changes: CommandChangeTracker,
    patch_ownership: PatchOwnershipTracker,
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
    system_agents: SystemAgentRegistry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProcessedAgentReply {
    transcript: String,
    final_reply: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CommandAgentDecision {
    Execute {
        command_lines: Vec<String>,
        reply: String,
    },
    Reply(String),
}

#[derive(Clone, Debug)]
struct SystemAgentRegistry {
    state: Arc<(Mutex<SystemAgentState>, Condvar)>,
}

impl Default for SystemAgentRegistry {
    fn default() -> Self {
        Self {
            state: Arc::new((Mutex::new(SystemAgentState::default()), Condvar::new())),
        }
    }
}

#[derive(Debug, Default)]
struct SystemAgentState {
    initialized: BTreeSet<AgentId>,
    active: BTreeSet<AgentId>,
}

#[derive(Debug)]
struct SystemAgentTurn {
    registry: SystemAgentRegistry,
    agent_id: AgentId,
    initialized: bool,
}

impl SystemAgentRegistry {
    fn begin_turn(&self, agent_id: &AgentId) -> SystemAgentTurn {
        let (lock, condvar) = &*self.state;
        let mut state = lock.lock().expect("system agent registry mutex poisoned");
        while state.active.contains(agent_id) {
            state = condvar
                .wait(state)
                .expect("system agent registry mutex poisoned");
        }
        let initialized = state.initialized.contains(agent_id);
        state.active.insert(agent_id.clone());
        SystemAgentTurn {
            registry: self.clone(),
            agent_id: agent_id.clone(),
            initialized,
        }
    }

    fn mark_initialized(&self, agent_id: &AgentId) {
        let (lock, _) = &*self.state;
        lock.lock()
            .expect("system agent registry mutex poisoned")
            .initialized
            .insert(agent_id.clone());
    }

    fn finish_turn(&self, agent_id: &AgentId) {
        let (lock, condvar) = &*self.state;
        lock.lock()
            .expect("system agent registry mutex poisoned")
            .active
            .remove(agent_id);
        condvar.notify_all();
    }
}

impl SystemAgentTurn {
    fn initialized(&self) -> bool {
        self.initialized
    }

    fn mark_initialized(&mut self) {
        self.registry.mark_initialized(&self.agent_id);
        self.initialized = true;
    }
}

impl Drop for SystemAgentTurn {
    fn drop(&mut self) {
        self.registry.finish_turn(&self.agent_id);
    }
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
            context_bundles: self.context_bundles.clone(),
            command_changes: self.command_changes.clone(),
            patch_ownership: self.patch_ownership.clone(),
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
            system_agents: self.system_agents.clone(),
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
            context_bundles: ContextBundleStore::new(),
            command_changes: CommandChangeTracker::default(),
            patch_ownership: PatchOwnershipTracker::default(),
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
            system_agents: SystemAgentRegistry::default(),
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

    pub fn shutdown_agents(&mut self) {
        if let Some(backend) = self.backend.as_mut() {
            backend.shutdown();
        } else {
            self.shutdown.shutdown();
        }
    }

    pub(crate) fn project_dir(&self) -> &std::path::Path {
        &self.project_dir
    }

    pub(crate) fn register_agent_feature(&mut self, agent_id: AgentId, feature: String) {
        self.agents.insert(agent_id, feature);
    }

    pub(crate) fn agent_session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        self.backend.as_ref()?.session(agent_id)
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
        _source_agent_id: &AgentId,
        first_prompt: &str,
    ) -> Result<String, CliError> {
        let title_agent_id = AgentId::new(TITLE_AGENT_ID).map_err(CliError::Agent)?;
        let prompt = chat_title_prompt(first_prompt);
        let reply =
            self.run_system_agent_turn(title_agent_id, "chat-title", prompt.clone(), prompt)?;
        Ok(chat_title_from_agent_reply(&reply, first_prompt))
    }

    pub(crate) fn interpret_command_agent_message(
        &mut self,
        message: &str,
        command_transcript: &[String],
    ) -> Result<CommandAgentDecision, CliError> {
        let command_agent_id = AgentId::new(COMMAND_AGENT_ID).map_err(CliError::Agent)?;
        let reply = self.run_system_agent_turn(
            command_agent_id,
            "command-agent",
            command_agent_launch_prompt(message, command_transcript),
            command_agent_follow_up_prompt(message, command_transcript),
        )?;
        parse_command_agent_reply(&reply)
    }

    fn run_system_agent_turn(
        &mut self,
        agent_id: AgentId,
        feature: &str,
        launch_prompt: String,
        follow_up_prompt: String,
    ) -> Result<String, CliError> {
        let mut turn = self.system_agents.begin_turn(&agent_id);
        if turn.initialized() {
            return self
                .backend
                .as_mut()
                .expect("command chat backend is present")
                .send(&agent_id, &follow_up_prompt)
                .map(|message| message.text)
                .map_err(CliError::Agent);
        }

        let session = self
            .backend
            .as_mut()
            .expect("command chat backend is present")
            .launch(AgentLaunch::new(
                agent_id,
                self.agent_profile.kind.clone(),
                feature,
                launch_prompt,
            ))
            .map_err(CliError::Agent)?;
        turn.mark_initialized();
        Ok(session
            .messages
            .last()
            .map(|message| message.text.clone())
            .unwrap_or_default())
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
            "promote" | "escalate" => self.promote_agent(&parts[1..]),
            "review" => self.review(),
            "linearize" => Err(CliError::Usage(
                "reviewed patch chats must be classified as closed before linearize; use force-linearize to bypass this direct command-chat gate".to_string(),
            )),
            "force-linearize" => self.linearize(),
            "linearize-questions" => self.linearize_questions(),
            "patch" | "locks" => Err(CliError::Usage(format!(
                "`{command}` is automatic orchestrator machinery, not a command chat command"
            ))),
            other => Err(CliError::Usage(format!(
                "unknown command chat command `{other}`"
            ))),
        }
    }

    fn promote_agent(&mut self, args: &[String]) -> Result<CommandChatResult, CliError> {
        let Some(agent_id) = args.first() else {
            return Err(CliError::Usage("promote requires an agent id".to_string()));
        };
        let agent_id = AgentId::new(agent_id.clone()).map_err(CliError::Agent)?;
        let prompt = args[1..].join(" ");
        self.remember_agent_review_baseline(&agent_id);
        self.send_to_agent(&agent_id, &patch_promotion_prompt(&prompt))
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
        let reply = {
            let backend = self
                .backend
                .as_mut()
                .expect("command chat backend is present");
            send_agent_streaming_interruptible(backend, agent_id, message, &mut *stream)
        }
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
        let session = {
            let backend = self
                .backend
                .as_mut()
                .expect("command chat backend is present");
            launch_agent_streaming_interruptible(backend, launch, &mut *stream)
        }
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
        if self.agent_review_baselines.contains_key(agent_id) {
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
                        context_bundles: &self.context_bundles,
                        command_changes: &self.command_changes,
                        patch_ownership: &self.patch_ownership,
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
            stream_secondary_follow_ups(agent_id, &run.follow_up_replies, stream);

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
        let source_context = {
            let session = self.agent_session(&commit.agent_id);
            render_review_source_context(&commit, session.as_ref())
        };

        let review_feature = format!("review {}", commit.feature);
        let review_prompt = format!(
            "Review the full patch scope for Agent-ID {}.\nLatest commit: {}\nFeature: {}\nReason: {}\nReview scope:\n{}\n\nSource context from Work Leaf commits, logs, and chat history:\n{}\n\nReview every commit listed in the review scope and reply with NO_FINDINGS if there are no findings. Otherwise reply with FINDINGS followed by the issues.\n\nDocumentation and plain-text updates are deferred to the linearize agent. Do not treat missing docs, README, changelog, markdown, txt, or other prose-only updates as findings against this patch agent; review the code and behavior that the patch agent changed.\n\nFor agent-facing changes, missing required real-agent verification is a finding unless the source context includes the exact real-agent scenario and visible result, or the exact pre-agent blocker. If you report missing verification, state the precise evidence that would resolve it. When the patch agent responds with verification evidence or a blocker rather than code, evaluate that evidence instead of requiring another patch.",
            commit.agent_id,
            commit.hash,
            commit.feature,
            commit.reason,
            commit.context,
            source_context
        );
        let mut review_text = if reuse_reviewer {
            {
                let backend = self
                    .backend
                    .as_mut()
                    .expect("command chat backend is present");
                send_agent_streaming_interruptible(
                    backend,
                    &reviewer_id,
                    &review_prompt,
                    &mut *stream,
                )
            }
            .map_err(CliError::Agent)?
            .text
        } else {
            let reviewer_session = {
                let backend = self
                    .backend
                    .as_mut()
                    .expect("command chat backend is present");
                launch_agent_streaming_interruptible(
                    backend,
                    AgentLaunch::new(
                        reviewer_id.clone(),
                        self.agent_profile.kind.clone(),
                        review_feature.clone(),
                        review_prompt,
                    ),
                    &mut *stream,
                )
            }
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
                "The reviewer found issues in your patch for commit {}.\n{}\n\nPlease fix the patch's code or test defects through the orchestrator patch flow. If a finding is about missing verification, missing explanation, or another non-code issue, resolve it by replying with the exact evidence, command result, real-agent scenario, or blocker; do not submit a cosmetic patch for non-code evidence. Do not modify documentation or plain-text files; documentation and prose updates are deferred to the linearize agent. Emit `@work-leaf done` when the findings are resolved.",
                commit.hash, review_text
            );
            stream(
                &commit.agent_id,
                AgentStreamEvent::AgentMessage(format!("reviewer findings:\n{review_text}")),
            );
            let fix_reply = {
                let backend = self
                    .backend
                    .as_mut()
                    .expect("command chat backend is present");
                send_agent_streaming_interruptible(
                    backend,
                    &commit.agent_id,
                    &fix_prompt,
                    &mut *stream,
                )
            }
            .map_err(CliError::Agent)?
            .text;
            let fix_reply = self.process_agent_reply_streaming(
                &commit.agent_id,
                &commit.feature,
                fix_reply,
                stream,
            )?;

            let recheck_prompt = format!(
                "The original agent has responded to the findings for commit {}.\n{}\n\nPlease check the patch again and reply with NO_FINDINGS if resolved, otherwise list remaining FINDINGS. The response may include code patches, verification evidence, real-agent smoke results, or an exact blocker; evaluate that evidence directly and do not require a code patch for a non-code finding. Documentation and plain-text updates are deferred to the linearize agent and must not be reported as remaining patch-agent findings.",
                commit.hash, fix_reply
            );
            let recheck_reply = {
                let backend = self
                    .backend
                    .as_mut()
                    .expect("command chat backend is present");
                send_agent_streaming_interruptible(
                    backend,
                    &reviewer_id,
                    &recheck_prompt,
                    &mut *stream,
                )
            }
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
        let mut commits = Vec::new();
        for reviewed_commit in &self.linearize_reviewed_commits {
            if history.agent_commit(&reviewed_commit.hash)?.is_some() {
                commits.push(reviewed_commit.clone());
            } else if let Some(current_commit) =
                history.agent_review_commit(&reviewed_commit.agent_id, None)?
            {
                commits.push(current_commit);
            }
        }
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

fn stream_secondary_follow_ups(
    primary_agent_id: &AgentId,
    follow_ups: &[AgentFollowUp],
    stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
) {
    for follow_up in follow_ups {
        if follow_up.agent_id == *primary_agent_id || follow_up.text.is_empty() {
            continue;
        }
        stream(
            &follow_up.agent_id,
            AgentStreamEvent::AgentMessage(follow_up.text.clone()),
        );
    }
}

pub fn render_process_help() -> String {
    [
        "Usage: work-leaf [--agent <codex|claude>] [--model <model>] [--no-read-permission] [-d|--deamon] [-c|--cli <http-api-url>]",
        "",
        "launches the orchestrator from the current project directory.",
        "Agents are created inside the command chat. Patches, file locks, review routing, and linearization handoff are orchestrator-controlled workflows, not top-level process commands.",
        "",
        "Options:",
        "  -d, --deamon           run only the localhost HTTP API and web UI daemon",
        "      --daemon           alias for --deamon",
        "  -c, --cli <url>        connect the CLI to an existing localhost HTTP API",
        "  -a, --agent <agent>      select the default agent provider: codex or claude (default: codex)",
        "  --model <model>          select the model for the chosen agent",
        "  --no-read-permission     allow agents to read project files directly; writes still require orchestrator patches",
        "",
        "Inside command chat:",
        "  new [prompt...]",
        "  review",
        "  linearize",
        "  force-linearize",
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
        "  force-linearize",
        "  quit",
        "",
        "Patches and file locks are triggered automatically when agents interact with the orchestrator.",
    ]
    .join("\n")
}

fn command_agent_launch_prompt(message: &str, command_transcript: &[String]) -> String {
    let recent_transcript = recent_command_transcript_text(command_transcript);
    format!(
        "You are the Work Leaf command-agent, a hidden system agent used only to interpret chat sent to the Work Leaf command surface.\n\
Return exactly one of these protocols:\n\
COMMAND: <one Work Leaf command line>\n\
REPLY: <short visible reply>\n\
\n\
or:\n\
COMMAND: <first Work Leaf command line>\n\
COMMAND: <second Work Leaf command line>\n\
REPLY: <short visible reply>\n\
\n\
or:\n\
REPLY: <short visible reply>\n\
\n\
Allowed command lines are help, new [prompt...], review, linearize, force-linearize, promote <agent-id> [prompt...], quit.\n\
Use force-linearize when the user asks for forced linearization, forced linearisation, or bypassing the linearize gate.\n\
Do not use shell commands. Do not modify files. Do not include markdown fences or extra prose.\n\n\
Recent command transcript:\n{recent_transcript}\n\n\
User command-surface message:\n{message}"
    )
}

fn command_agent_follow_up_prompt(message: &str, command_transcript: &[String]) -> String {
    let recent_transcript = recent_command_transcript_text(command_transcript);
    format!(
        "Interpret this Work Leaf command-surface message using the same protocol as before.\n\
Return only COMMAND/REPLY protocol lines.\n\n\
Recent command transcript:\n{recent_transcript}\n\n\
User command-surface message:\n{message}"
    )
}

fn recent_command_transcript_text(command_transcript: &[String]) -> String {
    command_transcript
        .iter()
        .rev()
        .take(COMMAND_AGENT_TRANSCRIPT_LIMIT)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_command_agent_reply(reply: &str) -> Result<CommandAgentDecision, CliError> {
    let mut command_lines = Vec::new();
    let mut visible_reply = None;

    for line in reply.lines() {
        let trimmed = line.trim();
        if let Some(value) = strip_ascii_prefix_case_insensitive(trimmed, "COMMAND:") {
            let value = value
                .trim()
                .trim_matches('`')
                .trim_start_matches(':')
                .trim();
            if !value.is_empty() {
                command_lines.push(value.to_string());
            }
        } else if let Some(value) = strip_ascii_prefix_case_insensitive(trimmed, "REPLY:") {
            let value = value.trim();
            if !value.is_empty() {
                visible_reply = Some(value.to_string());
            }
        }
    }

    if !command_lines.is_empty() {
        let reply = visible_reply.unwrap_or_else(|| {
            if command_lines.len() == 1 {
                format!("running `{}`", command_lines[0])
            } else {
                format!("running {} commands", command_lines.len())
            }
        });
        return Ok(CommandAgentDecision::Execute {
            command_lines,
            reply,
        });
    }

    let reply = visible_reply.unwrap_or_else(|| {
        let trimmed = reply.trim();
        if trimmed.is_empty() {
            "I can run help, new [prompt...], review, linearize, force-linearize, or quit."
                .to_string()
        } else {
            trimmed.to_string()
        }
    });
    Ok(CommandAgentDecision::Reply(reply))
}

fn strip_ascii_prefix_case_insensitive<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    value
        .to_ascii_lowercase()
        .starts_with(&prefix.to_ascii_lowercase())
        .then(|| &value[prefix.len()..])
}

pub(crate) fn patch_promotion_prompt(prompt: &str) -> String {
    if prompt.is_empty() {
        "Continue this existing Work Leaf session as a patch agent. Report the broad feature before proposing patches, follow the patch-agent instructions, and use the orchestrator patch flow for file changes.".to_string()
    } else {
        format!(
            "Continue this existing Work Leaf session as a patch agent.\n\nPatch task:\n{prompt}\n\nReport the broad feature before proposing patches, follow the patch-agent instructions, and use the orchestrator patch flow for file changes."
        )
    }
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
    chat_content_from_transcript(chat_buffer, transcript)
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
            "\u{1b}[?1049h\u{1b}[?1000h\u{1b}[?1006h\u{1b}[?2004h\u{1b}[>1u\u{1b}[>4;2m\u{1b}[2J\u{1b}[H"
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
            "\u{1b}[>4;0m\u{1b}[<u\u{1b}[?2004l\u{1b}[?1006l\u{1b}[?1000l\u{1b}[?1049l\u{1b}[?25h"
        );
        let _ = stdout.flush();
    }
}

#[cfg(test)]
mod terminal_mode_tests {
    use super::*;

    #[test]
    fn alternate_screen_enables_modified_enter_reporting() {
        let mut output = Vec::new();
        let mode = AlternateScreenMode::enter(&mut output).expect("alternate screen escape writes");
        let output = String::from_utf8(output).expect("terminal escapes are utf-8");

        assert!(output.contains("\u{1b}[>1u"));
        assert!(output.contains("\u{1b}[>4;2m"));

        std::mem::forget(mode);
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
    let binary = resolve_codex_binary();
    prepend_process_path(binary.parent());
    let mut config = CodexCommandConfig::new(project_dir.clone()).with_binary(binary);
    if let Some(model) = model {
        config = config.with_model(model);
    }
    if let Some(sandbox) = codex_linearize_sandbox_from_env()? {
        config = config.with_linearize_sandbox(sandbox);
    }
    Ok(CodexBackend::new(
        config,
        PromptPolicy::for_project_with_read_permission(&project_dir, read_permission)
            .map_err(CliError::Agent)?,
    ))
}

pub(crate) fn selected_agent_backend(
    project_dir: PathBuf,
    model: Option<String>,
    read_permission: ReadPermission,
    agent: SelectedAgent,
) -> Result<SelectedAgentBackend, CliError> {
    match agent {
        SelectedAgent::Codex => {
            codex_backend(project_dir, model, read_permission).map(SelectedAgentBackend::Codex)
        }
        SelectedAgent::Claude => {
            claude_backend(project_dir, model, read_permission).map(SelectedAgentBackend::Claude)
        }
    }
}

pub(crate) fn claude_backend(
    project_dir: PathBuf,
    model: Option<String>,
    read_permission: ReadPermission,
) -> Result<ClaudeBackend, CliError> {
    let binary = resolve_binary_from_path("claude");
    prepend_process_path(binary.parent());
    let mut config = ClaudeCommandConfig::new(project_dir.clone())
        .with_binary(binary)
        .with_read_tools(read_permission == ReadPermission::DirectFilesystem);
    if let Some(model) = model {
        config = config.with_model(model);
    }
    Ok(ClaudeBackend::new(
        config,
        PromptPolicy::for_project_with_read_permission(&project_dir, read_permission)
            .map_err(CliError::Agent)?,
    ))
}

fn codex_linearize_sandbox_from_env() -> Result<Option<SandboxMode>, CliError> {
    let value = match env::var("WORK_LEAF_CODEX_LINEARIZE_SANDBOX") {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(CliError::Usage(
                "WORK_LEAF_CODEX_LINEARIZE_SANDBOX must be valid UTF-8".to_string(),
            ));
        }
    };

    match value.as_str() {
        "read-only" => Ok(Some(SandboxMode::ReadOnly)),
        "workspace-write" => Ok(Some(SandboxMode::WorkspaceWrite)),
        "danger-full-access" => Ok(Some(SandboxMode::DangerFullAccess)),
        _ => Err(CliError::Usage(format!(
            "invalid WORK_LEAF_CODEX_LINEARIZE_SANDBOX `{value}`; expected read-only, workspace-write, or danger-full-access"
        ))),
    }
}

fn resolve_codex_binary() -> PathBuf {
    let path = env::var_os("PATH");
    resolve_codex_binary_from_path(path.as_deref())
}

fn resolve_binary_from_path(name: &str) -> PathBuf {
    let Some(path) = env::var_os("PATH") else {
        return PathBuf::from(name);
    };
    for dir in env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return candidate;
        }
    }
    PathBuf::from(name)
}

fn prepend_process_path(dir: Option<&Path>) {
    let Some(dir) = dir else {
        return;
    };
    let current = env::var_os("PATH");
    let mut entries = vec![dir.to_path_buf()];
    if let Some(current) = current.as_deref() {
        entries.extend(env::split_paths(current).filter(|entry| entry.as_path() != dir));
    }
    if let Ok(path) = env::join_paths(entries) {
        // SAFETY: this runs while constructing the CLI/daemon backend, before Work Leaf starts
        // worker threads that read the process environment.
        unsafe { env::set_var("PATH", path) };
    }
}

fn resolve_codex_binary_from_path(path: Option<&OsStr>) -> PathBuf {
    let Some(path) = path else {
        return PathBuf::from("codex");
    };
    let mut fallback = None;
    for dir in env::split_paths(path) {
        let candidate = dir.join("codex");
        if !candidate.is_file() {
            continue;
        }
        if is_codex_arg0_shim(&candidate) {
            fallback.get_or_insert(candidate);
            continue;
        }
        return candidate;
    }
    fallback.unwrap_or_else(|| PathBuf::from("codex"))
}

fn is_codex_arg0_shim(path: &Path) -> bool {
    let path = path.to_string_lossy();
    path.contains("/.codex/tmp/arg0/") || path.contains("\\.codex\\tmp\\arg0\\")
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
    use std::fs;
    use std::sync::{Mutex, Once, OnceLock};

    static REGISTER_TEST_CLEANUP: Once = Once::new();
    static TEST_TEMP_ROOTS: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();
    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

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

    #[test]
    fn process_args_parse_daemon_mode() {
        assert_eq!(
            parse_process_args(["work-leaf", "-d"]).unwrap(),
            ProcessCommand::Daemon {
                model: None,
                read_permission: ReadPermission::Orchestrator,
                agent: SelectedAgent::Codex,
            }
        );
        assert_eq!(
            parse_process_args(["work-leaf.exe", "-d"]).unwrap(),
            ProcessCommand::Daemon {
                model: None,
                read_permission: ReadPermission::Orchestrator,
                agent: SelectedAgent::Codex,
            }
        );
        assert_eq!(
            parse_process_args(["work-leaf", "--deamon", "--model", "gpt-test"]).unwrap(),
            ProcessCommand::Daemon {
                model: Some("gpt-test".to_string()),
                read_permission: ReadPermission::Orchestrator,
                agent: SelectedAgent::Codex,
            }
        );
    }

    #[test]
    fn process_args_parse_remote_cli_url() {
        assert_eq!(
            parse_process_args(["work-leaf", "--cli", "http://127.0.0.1:7878"]).unwrap(),
            ProcessCommand::Launch {
                model: None,
                read_permission: ReadPermission::Orchestrator,
                agent: SelectedAgent::Codex,
                cli_url: Some("http://127.0.0.1:7878".to_string()),
            }
        );
        assert_eq!(
            parse_process_args(["work-leaf", "-c", "http://localhost:7878"]).unwrap(),
            ProcessCommand::Launch {
                model: None,
                read_permission: ReadPermission::Orchestrator,
                agent: SelectedAgent::Codex,
                cli_url: Some("http://localhost:7878".to_string()),
            }
        );
        assert_eq!(
            parse_process_args(["work-leaf.exe", "--cli", "http://127.0.0.1:7878"]).unwrap(),
            ProcessCommand::Launch {
                model: None,
                read_permission: ReadPermission::Orchestrator,
                agent: SelectedAgent::Codex,
                cli_url: Some("http://127.0.0.1:7878".to_string()),
            }
        );
    }

    #[test]
    fn process_args_reject_missing_remote_cli_url() {
        assert!(matches!(
            parse_process_args(["work-leaf", "--cli"]),
            Err(CliError::Usage(message)) if message.contains("--cli requires a value")
        ));
    }

    #[test]
    fn process_args_reject_combined_daemon_and_remote_cli_modes() {
        assert!(matches!(
            parse_process_args(["work-leaf", "-d", "-c", "http://127.0.0.1:7878"]),
            Err(CliError::Usage(message)) if message.contains("cannot be combined")
        ));
    }

    #[test]
    fn daemon_startup_text_reports_distinct_http_api_and_web_ui_urls() {
        let web_ui_url = daemon_web_ui_url("http://127.0.0.1:49153/", "http://127.0.0.1:49152/");

        assert_eq!(
            web_ui_url,
            "http://127.0.0.1:49153?api=http://127.0.0.1:49152"
        );
        assert_ne!(
            web_ui_url.split('?').next().unwrap(),
            "http://127.0.0.1:49152"
        );
        assert_eq!(
            render_split_daemon_startup("http://127.0.0.1:49152/", &web_ui_url),
            "http api at: http://127.0.0.1:49152\nweb-ui at: http://127.0.0.1:49153?api=http://127.0.0.1:49152\n"
        );
        assert_eq!(
            render_daemon_startup("http://127.0.0.1:49152/"),
            "http api at: http://127.0.0.1:49152\nweb-ui at: http://127.0.0.1:49152/web-ui\n"
        );
    }

    #[test]
    fn codex_binary_resolver_skips_temporary_arg0_shim_when_stable_binary_exists() {
        let root = test_temp_dir("codex-arg0-shim");
        let shim_dir = root.join(".codex/tmp/arg0/codex-arg0test");
        let stable_dir = root.join("bin");
        fs::create_dir_all(&shim_dir).unwrap();
        fs::create_dir_all(&stable_dir).unwrap();
        fs::write(shim_dir.join("codex"), "").unwrap();
        fs::write(stable_dir.join("codex"), "").unwrap();
        let path = env::join_paths([shim_dir, stable_dir.clone()]).unwrap();

        let binary = resolve_codex_binary_from_path(Some(path.as_os_str()));

        assert_eq!(binary, stable_dir.join("codex"));
    }

    #[test]
    fn codex_binary_resolver_respects_non_shim_path_entries() {
        let root = test_temp_dir("codex-normal-path");
        let first_dir = root.join("first");
        let second_dir = root.join("second");
        fs::create_dir_all(&first_dir).unwrap();
        fs::create_dir_all(&second_dir).unwrap();
        fs::write(first_dir.join("codex"), "").unwrap();
        fs::write(second_dir.join("codex"), "").unwrap();
        let path = env::join_paths([first_dir.clone(), second_dir]).unwrap();

        let binary = resolve_codex_binary_from_path(Some(path.as_os_str()));

        assert_eq!(binary, first_dir.join("codex"));
    }

    #[test]
    fn codex_process_binary_resolver_uses_stable_executable_directly() {
        let root = test_temp_dir("codex-direct-executable");
        let bin_dir = root.join("bin with space");
        fs::create_dir_all(&bin_dir).unwrap();
        let codex = bin_dir.join("codex");
        fs::write(&codex, "").unwrap();
        let path = env::join_paths([bin_dir]).unwrap();

        let binary = resolve_codex_binary_from_path(Some(path.as_os_str()));

        assert_eq!(binary, codex);
    }

    #[test]
    fn codex_process_binary_resolver_keeps_distinct_target_binaries() {
        let root = test_temp_dir("codex-distinct-executables");
        let first_dir = root.join("first");
        let second_dir = root.join("second");
        fs::create_dir_all(&first_dir).unwrap();
        fs::create_dir_all(&second_dir).unwrap();
        let first_codex = first_dir.join("codex");
        let second_codex = second_dir.join("codex");
        fs::write(&first_codex, "").unwrap();
        fs::write(&second_codex, "").unwrap();
        let first_path = env::join_paths([first_dir]).unwrap();
        let second_path = env::join_paths([second_dir]).unwrap();

        let first_binary = resolve_codex_binary_from_path(Some(first_path.as_os_str()));
        let second_binary = resolve_codex_binary_from_path(Some(second_path.as_os_str()));

        assert_eq!(first_binary, first_codex);
        assert_eq!(second_binary, second_codex);
        assert_ne!(first_binary, second_binary);
    }

    #[test]
    fn codex_linearize_sandbox_env_parses_known_modes_and_rejects_invalid_values() {
        let _env_lock = TEST_ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::new("WORK_LEAF_CODEX_LINEARIZE_SANDBOX");
        unsafe {
            env::set_var("WORK_LEAF_CODEX_LINEARIZE_SANDBOX", "danger-full-access");
        }

        assert_eq!(
            codex_linearize_sandbox_from_env().unwrap(),
            Some(SandboxMode::DangerFullAccess)
        );

        unsafe {
            env::set_var("WORK_LEAF_CODEX_LINEARIZE_SANDBOX", "invalid");
        }

        assert!(matches!(
            codex_linearize_sandbox_from_env(),
            Err(CliError::Usage(message))
                if message.contains("WORK_LEAF_CODEX_LINEARIZE_SANDBOX")
        ));
    }

    fn test_temp_dir(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!("work-leaf-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        register_test_temp_dir(root.clone());
        root
    }

    fn register_test_temp_dir(root: PathBuf) {
        REGISTER_TEST_CLEANUP.call_once(|| unsafe {
            let _ = atexit(cleanup_test_temp_dirs);
        });
        TEST_TEMP_ROOTS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .push(root);
    }

    unsafe extern "C" {
        fn atexit(callback: extern "C" fn()) -> i32;
    }

    extern "C" fn cleanup_test_temp_dirs() {
        let Some(roots) = TEST_TEMP_ROOTS.get() else {
            return;
        };
        let Ok(mut roots) = roots.lock() else {
            return;
        };
        for root in roots.drain(..) {
            let _ = fs::remove_dir_all(root);
        }
    }

    struct EnvGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn new(key: &'static str) -> Self {
            let previous = env::var_os(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                match self.previous.take() {
                    Some(value) => env::set_var(self.key, value),
                    None => env::remove_var(self.key),
                }
            }
        }
    }
}
