use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{
    Arc, Mutex,
    mpsc::{self, Receiver, Sender},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::agent::{
    AgentBackend, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, AgentTokenUsage, ChatMessage, MessageRole,
};
use crate::chat_title::ChatTitleAgent;
use crate::cli::{
    COMMAND_AGENT_TRANSCRIPT_LIMIT, CliError, CommandAgentDecision, CommandChat, CommandChatResult,
    command_chat_error_text, command_result_text, patch_promotion_prompt, render_command_chat_help,
};
use crate::review::{AgentCommit, GitHistory, ReviewResult};

const FEATURE_DONE_QUESTION: &str = "work-leaf: is this feature done? [yes/no]";
const FEATURE_CLOSED_MESSAGE: &str = "work-leaf: feature marked closed";
const FEATURE_OPEN_MESSAGE: &str = "work-leaf: feature remains open";
const FEATURE_DONE_ANSWER_MESSAGE: &str = "work-leaf: answer yes or no to close this feature";
pub(crate) const REVIEW_FINISHED_NO_FINDINGS_MESSAGE: &str =
    "work-leaf: review finished with no findings";
pub(crate) const REVIEW_FINISHED_WITH_FINDINGS_MESSAGE: &str =
    "work-leaf: review finished with findings remaining";
pub(crate) const REVIEW_FAILED_MESSAGE: &str =
    "work-leaf: review failed before findings were resolved";

#[derive(Debug)]
pub struct WorkLeafController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    chat: Option<CommandChat<B>>,
    shutdown: AgentShutdownHandle,
    shutdown_on_drop: bool,
    workers: Vec<Worker>,
    pending_launches: VecDeque<AgentLaunch>,
    pending_dependent_launches: BTreeMap<AgentId, PendingDependentLaunch>,
    pending_dependent_sends: BTreeMap<AgentId, Vec<PendingDependentSend>>,
    pending_agent_prompts: BTreeMap<AgentId, VecDeque<String>>,
    launch_starting: BTreeSet<AgentId>,
    implicit_loading_agents: BTreeMap<AgentId, usize>,
    pending_title_prompts: BTreeMap<AgentId, String>,
    command_transcript: Vec<String>,
    sessions: BTreeMap<AgentId, WorkLeafSession>,
    title_agent: ChatTitleAgent,
    pending_events: Vec<WorkLeafEvent>,
    reviewers: BTreeSet<AgentId>,
    review_commits_in_progress: BTreeMap<AgentId, String>,
    reviewed_agent_commits: BTreeMap<AgentId, String>,
    agent_review_baselines: BTreeMap<AgentId, String>,
    stopped_for_linearize: BTreeSet<AgentId>,
}

impl<B> WorkLeafController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    pub fn new(chat: CommandChat<B>) -> Self {
        let shutdown = chat.shutdown_handle();
        Self {
            chat: Some(chat),
            shutdown,
            shutdown_on_drop: true,
            workers: Vec::new(),
            pending_launches: VecDeque::new(),
            pending_dependent_launches: BTreeMap::new(),
            pending_dependent_sends: BTreeMap::new(),
            pending_agent_prompts: BTreeMap::new(),
            launch_starting: BTreeSet::new(),
            implicit_loading_agents: BTreeMap::new(),
            pending_title_prompts: BTreeMap::new(),
            command_transcript: vec![render_command_chat_help()],
            sessions: BTreeMap::new(),
            title_agent: ChatTitleAgent::new(),
            pending_events: Vec::new(),
            reviewers: BTreeSet::new(),
            review_commits_in_progress: BTreeMap::new(),
            reviewed_agent_commits: BTreeMap::new(),
            agent_review_baselines: BTreeMap::new(),
            stopped_for_linearize: BTreeSet::new(),
        }
    }

    pub fn into_chat(mut self) -> CommandChat<B> {
        self.wait_for_idle(Duration::from_secs(5));
        self.shutdown_on_drop = false;
        self.chat
            .take()
            .expect("work-leaf controller command chat is present")
    }

    pub fn transcript(&self) -> &[String] {
        &self.command_transcript
    }

    pub fn push_transcript_line(&mut self, line: impl Into<String>) {
        self.push_command_line(line.into());
    }

    pub fn snapshot(&self) -> WorkLeafSnapshot {
        WorkLeafSnapshot {
            command_transcript: self.command_transcript.clone(),
            sessions: self.sessions.values().cloned().collect(),
        }
    }

    pub fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
        if self.pending_events.is_empty() {
            self.poll_worker();
        }
        self.pending_events.drain(..).collect()
    }

    pub fn is_busy(&mut self) -> bool {
        self.poll_worker();
        !self.workers.is_empty() || !self.pending_launches.is_empty()
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.poll_worker();
            if self.workers.is_empty() && self.pending_launches.is_empty() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.poll_worker();
        self.workers.is_empty() && self.pending_launches.is_empty()
    }

    pub fn wait_for_session_line(
        &mut self,
        agent_id: &AgentId,
        needle: &str,
        timeout: Duration,
    ) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.poll_worker();
            if self.session_contains(agent_id, needle) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.poll_worker();
        self.session_contains(agent_id, needle)
    }

    pub fn execute_command_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        self.push_command_line(format!("work-leaf> {trimmed}"));
        let parts = split_command_line(trimmed);
        let Some(command) = parts.first().map(String::as_str) else {
            return;
        };

        match command {
            "quit" | "exit" | "q" => self.request_quit(),
            "new" => {
                let prompt = parts[1..].join(" ");
                if let Err(error) = self.create_agent(prompt) {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            "promote" | "escalate" => {
                if let Err(error) = self.promote_agent_from_command(command, &parts[1..]) {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            "review" => {
                if let Err(error) = self.start_review() {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            "linearize" => self.execute_linearize_command(true),
            "force-linearize" => self.execute_linearize_command(false),
            _ => self.start_command_worker(trimmed.to_string()),
        }
    }

    pub fn send_command_agent_message(&mut self, message: &str) {
        let message = message.trim();
        if message.is_empty() {
            return;
        }

        self.push_command_line(format!("user: {message}"));
        self.start_command_agent_worker(message.to_string());
    }

    pub fn interrupt_agent(&mut self, agent_id: &AgentId) {
        let display_name = self.agent_display_name();
        let result = self
            .chat
            .as_mut()
            .expect("work-leaf controller command chat is present")
            .interrupt_agent(agent_id);
        match result {
            Ok(()) => {
                self.set_session_loading(agent_id, None);
                self.append_agent_line(
                    agent_id,
                    format!("work-leaf: sent Ctrl-C to {display_name}"),
                );
            }
            Err(error) => self.append_agent_line(agent_id, command_chat_error_text(&error)),
        }
    }

    pub fn create_agent(&mut self, prompt: impl Into<String>) -> Result<AgentId, CliError> {
        let prompt = prompt.into();
        let request = parse_agent_creation_request(split_command_line(&prompt))?;
        if let Some(source_agent_id) = request.fork_from {
            return self.fork_agent_with_options(
                &source_agent_id,
                &request.args.join(" "),
                request.depends_on,
            );
        }
        if let Some(dependency) = &request.depends_on {
            self.ensure_session_exists(dependency)?;
        }

        let title_pending = request.args.is_empty();
        let launch = self.prepare_agent_launch(&request.args)?;
        let agent_id = launch.id.clone();
        self.validate_dependency_target(&agent_id, request.depends_on.as_ref())?;
        self.remember_agent_review_baseline(&agent_id);
        let title = self
            .reserve_launch_title(&launch, title_pending)
            .unwrap_or_else(|| launch.feature.clone());
        self.register_agent_feature(agent_id.clone(), title.clone());
        self.add_session(WorkLeafSession {
            id: agent_id.clone(),
            kind: launch.kind.clone(),
            title: title.clone(),
            feature: title,
            lines: Vec::new(),
            loading: Some(WorkLeafLoading::Launching),
            completion: None,
            token_usage: None,
            depends_on: Vec::new(),
            depended_on_by: Vec::new(),
        });
        self.pending_events.push(WorkLeafEvent::AgentSelected {
            agent_id: agent_id.clone(),
        });
        if let Some(dependency) = request.depends_on {
            self.attach_dependency(&agent_id, &dependency)?;
            if self.dependency_is_closed(&dependency) {
                self.append_agent_line(
                    &agent_id,
                    format!("work-leaf: dependency {dependency} marked done; launching"),
                );
                self.queue_or_start_launch_worker(launch);
            } else {
                self.defer_launch_until_dependency(launch, dependency, title_pending);
            }
        } else {
            self.queue_or_start_launch_worker(launch);
        }
        Ok(agent_id)
    }

    pub fn send_message(&mut self, agent_id: &AgentId, message: &str) -> Result<(), CliError> {
        let message = message.trim();
        if message.is_empty() {
            return Ok(());
        }
        self.stopped_for_linearize.remove(agent_id);
        let is_agent_slash_command = is_agent_slash_command_message(message);
        if !is_agent_slash_command
            && self.session_completion(agent_id) == Some(WorkLeafCompletion::NeedsDecision)
        {
            self.handle_completion_answer(agent_id, message);
            return Ok(());
        }
        if self.session_completion(agent_id) == Some(WorkLeafCompletion::Closed) {
            self.set_session_completion(agent_id, None);
        }

        let first_chat_title = self.reserve_first_chat_title(agent_id, message);
        if let Some(title) = &first_chat_title {
            self.apply_agent_title(agent_id, title.clone());
        }
        if !is_agent_slash_command && self.set_pending_dependent_launch_prompt(agent_id, message) {
            self.append_agent_line(agent_id, format!("user: {message}"));
            return Ok(());
        }
        if self
            .sessions
            .get(agent_id)
            .and_then(|session| session.loading)
            .is_some()
        {
            self.queue_agent_prompt(agent_id, message);
            return Ok(());
        }

        self.start_send_worker(agent_id.clone(), message.to_string());
        Ok(())
    }

    pub fn promote_agent_to_patch(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
    ) -> Result<(), CliError> {
        self.ensure_session_exists(agent_id)?;
        let request = parse_agent_creation_request(split_command_line(prompt))?;
        self.validate_dependency_target(agent_id, request.depends_on.as_ref())?;
        self.remember_agent_review_baseline(agent_id);
        let prompt = request.args.join(" ");
        let promotion_prompt = patch_promotion_prompt(&prompt);
        self.append_agent_line(
            agent_id,
            "work-leaf: escalated this chat to a patch agent".to_string(),
        );
        if let Some(title) = self.reserve_first_chat_title(agent_id, &prompt) {
            self.apply_agent_title(agent_id, title);
        }
        if let Some(dependency) = request.depends_on {
            self.attach_dependency(agent_id, &dependency)?;
            if self.dependency_is_closed(&dependency) {
                self.append_agent_line(
                    agent_id,
                    format!("work-leaf: dependency {dependency} marked done; sending patch task"),
                );
                self.start_send_worker(agent_id.clone(), promotion_prompt);
            } else {
                self.defer_send_until_dependency(agent_id.clone(), promotion_prompt, dependency);
            }
        } else {
            self.start_send_worker(agent_id.clone(), promotion_prompt);
        }
        Ok(())
    }

    pub fn fork_agent(
        &mut self,
        source_agent_id: &AgentId,
        prompt: &str,
    ) -> Result<AgentId, CliError> {
        self.fork_agent_with_options(source_agent_id, prompt, None)
    }

    fn fork_agent_with_options(
        &mut self,
        source_agent_id: &AgentId,
        prompt: &str,
        inherited_dependency: Option<AgentId>,
    ) -> Result<AgentId, CliError> {
        let request = parse_agent_creation_request(split_command_line(prompt))?;
        let depends_on = request.depends_on.or(inherited_dependency);
        let fork_prompt = request.args.join(" ");
        let source = self.sessions.get(source_agent_id).cloned().ok_or_else(|| {
            CliError::Agent(crate::agent::AgentError::UnknownSession(
                source_agent_id.clone(),
            ))
        })?;
        if let Some(dependency) = &depends_on {
            self.ensure_session_exists(dependency)?;
        }
        let backend_session = self
            .chat
            .as_ref()
            .and_then(|chat| chat.agent_session(source_agent_id));
        let launch_prompt = fork_launch_prompt(
            source_agent_id,
            backend_session.as_ref(),
            &source,
            &fork_prompt,
        );
        let launch = self.prepare_agent_launch(&[launch_prompt])?;
        let agent_id = launch.id.clone();
        self.validate_dependency_target(&agent_id, depends_on.as_ref())?;
        self.remember_agent_review_baseline(&agent_id);
        let title = if fork_prompt.is_empty() {
            self.title_agent.mark_named(&agent_id);
            format!("{} fork", source.title)
        } else {
            self.pending_title_prompts
                .insert(agent_id.clone(), fork_prompt.clone());
            self.title_agent
                .assign_title_from_prompt(&agent_id, &fork_prompt)
        };
        self.register_agent_feature(agent_id.clone(), title.clone());
        let mut lines = source.lines.clone();
        lines.push(format!("work-leaf: forked from {source_agent_id}"));
        self.add_session(WorkLeafSession {
            id: agent_id.clone(),
            kind: launch.kind.clone(),
            title: title.clone(),
            feature: title,
            lines,
            loading: Some(WorkLeafLoading::Launching),
            completion: None,
            token_usage: None,
            depends_on: Vec::new(),
            depended_on_by: Vec::new(),
        });
        self.pending_events.push(WorkLeafEvent::AgentSelected {
            agent_id: agent_id.clone(),
        });
        if let Some(dependency) = depends_on {
            self.attach_dependency(&agent_id, &dependency)?;
            if self.dependency_is_closed(&dependency) {
                self.append_agent_line(
                    &agent_id,
                    format!("work-leaf: dependency {dependency} marked done; launching"),
                );
                self.queue_or_start_launch_worker(launch);
            } else {
                self.defer_launch_until_dependency(launch, dependency, false);
            }
        } else {
            self.queue_or_start_launch_worker(launch);
        }
        Ok(agent_id)
    }

    pub fn start_review(&mut self) -> Result<Vec<AgentId>, CliError> {
        self.start_review_for_agent(None)
    }

    fn start_review_for_patch_agent(
        &mut self,
        agent_id: &AgentId,
    ) -> Result<Vec<AgentId>, CliError> {
        self.start_review_for_agent(Some(agent_id))
    }

    fn start_review_for_agent(
        &mut self,
        target_agent_id: Option<&AgentId>,
    ) -> Result<Vec<AgentId>, CliError> {
        let (project_dir, agent_profile) = {
            let chat = self
                .chat
                .as_ref()
                .expect("work-leaf controller command chat is present");
            (
                chat.project_dir().to_path_buf(),
                chat.agent_profile().clone(),
            )
        };
        let empty_baselines = BTreeMap::new();
        let agent_baselines = if target_agent_id.is_some() {
            &self.agent_review_baselines
        } else {
            &empty_baselines
        };
        let commits = GitHistory::new(project_dir)
            .latest_agent_review_commits(&self.reviewed_agent_commits, agent_baselines)?;
        if commits.is_empty() {
            self.push_command_line("no agent commits found".to_string());
            return Ok(Vec::new());
        }

        let mut reviewer_ids = Vec::new();
        for commit in commits {
            if target_agent_id.is_some_and(|agent_id| &commit.agent_id != agent_id) {
                continue;
            }
            if self
                .reviewed_agent_commits
                .get(&commit.agent_id)
                .is_some_and(|hash| hash == &commit.hash)
                || self
                    .review_commits_in_progress
                    .get(&commit.agent_id)
                    .is_some_and(|hash| hash == &commit.hash)
            {
                continue;
            }
            let reviewer_id = AgentId::new(format!("review-{}", commit.agent_id.as_str()))
                .map_err(CliError::Agent)?;
            let reviewer_busy = self
                .sessions
                .get(&reviewer_id)
                .and_then(|session| session.loading)
                .is_some();
            if reviewer_busy {
                continue;
            }
            let session_exists = self.sessions.contains_key(&reviewer_id);
            let reuse_reviewer = self.reviewers.contains(&reviewer_id);
            if session_exists {
                self.set_session_loading(&reviewer_id, Some(WorkLeafLoading::WaitingForReply));
            } else {
                self.add_session(WorkLeafSession {
                    id: reviewer_id.clone(),
                    kind: agent_profile.kind.clone(),
                    title: format!("review {}", commit.feature),
                    feature: format!("review {}", commit.feature),
                    lines: Vec::new(),
                    loading: Some(WorkLeafLoading::WaitingForReply),
                    completion: None,
                    token_usage: None,
                    depends_on: Vec::new(),
                    depended_on_by: Vec::new(),
                });
            }
            if reviewer_ids.is_empty() {
                self.pending_events.push(WorkLeafEvent::AgentSelected {
                    agent_id: reviewer_id.clone(),
                });
            }
            self.review_commits_in_progress
                .insert(commit.agent_id.clone(), commit.hash.clone());
            self.start_review_worker(commit, reviewer_id.clone(), reuse_reviewer);
            reviewer_ids.push(reviewer_id);
        }
        Ok(reviewer_ids)
    }

    pub fn start_linearize(&mut self) -> Result<Option<AgentId>, CliError> {
        let launch = {
            let chat = self
                .chat
                .as_mut()
                .expect("work-leaf controller command chat is present");
            chat.prepare_linearize_launch()?
        };
        let Some(launch) = launch else {
            self.push_command_line("no reviewed agent commits found".to_string());
            return Ok(None);
        };

        self.stop_non_linearize_agents_for_linearize();
        let agent_id = launch.id.clone();
        let title = launch.feature.clone();
        self.register_agent_feature(agent_id.clone(), title.clone());
        self.add_session(WorkLeafSession {
            id: agent_id.clone(),
            kind: launch.kind.clone(),
            title: title.clone(),
            feature: title,
            lines: Vec::new(),
            loading: Some(WorkLeafLoading::Launching),
            completion: None,
            token_usage: None,
            depends_on: Vec::new(),
            depended_on_by: Vec::new(),
        });
        self.pending_events.push(WorkLeafEvent::AgentSelected {
            agent_id: agent_id.clone(),
        });
        self.queue_or_start_launch_worker(launch);
        Ok(Some(agent_id))
    }

    fn execute_linearize_command(&mut self, require_closed_patch_chats: bool) {
        if require_closed_patch_chats && !self.reviewed_patch_chats_are_closed_for_linearize() {
            return;
        }
        if let Err(error) = self.start_linearize() {
            self.push_command_line(command_chat_error_text(&error));
        }
    }

    fn reviewed_patch_chats_are_closed_for_linearize(&mut self) -> bool {
        let unclosed_agent_ids = self
            .reviewed_agent_commits
            .keys()
            .filter(|agent_id| {
                self.session_completion(agent_id) != Some(WorkLeafCompletion::Closed)
            })
            .map(|agent_id| agent_id.as_str().to_string())
            .collect::<Vec<_>>();
        if unclosed_agent_ids.is_empty() {
            return true;
        }

        self.push_command_line(format!(
            "work-leaf: reviewed patch chats must be classified as closed before linearize: {}. Use force-linearize to bypass.",
            unclosed_agent_ids.join(", ")
        ));
        false
    }

    pub fn loading_text(&self, loading: WorkLeafLoading) -> String {
        match loading {
            WorkLeafLoading::Launching => {
                format!("Starting {} session", self.agent_display_name())
            }
            WorkLeafLoading::WaitingForReply => {
                format!("Waiting for {}", self.agent_display_name())
            }
            WorkLeafLoading::WaitingForDependency => "Waiting for dependency".to_string(),
        }
    }

    pub fn shutdown(&mut self) {
        self.shutdown_agents();
    }

    fn poll_worker(&mut self) {
        let mut events = Vec::new();
        for worker in &self.workers {
            while let Ok(event) = worker.receiver.try_recv() {
                events.push(event);
            }
        }
        for event in events {
            self.apply_worker_event(event);
        }

        let mut index = 0;
        while index < self.workers.len() {
            if self.workers[index].handle.is_finished() {
                let worker = self.workers.swap_remove(index);
                while let Ok(event) = worker.receiver.try_recv() {
                    self.apply_worker_event(event);
                }
                worker
                    .handle
                    .join()
                    .expect("work-leaf worker did not panic");
            } else {
                index += 1;
            }
        }
        self.start_next_pending_launch();
    }

    fn session_contains(&self, agent_id: &AgentId, needle: &str) -> bool {
        self.sessions
            .get(agent_id)
            .is_some_and(|session| session.lines.iter().any(|line| line.contains(needle)))
    }

    fn reserve_launch_title(
        &mut self,
        launch: &AgentLaunch,
        title_pending: bool,
    ) -> Option<String> {
        if title_pending {
            None
        } else {
            self.pending_title_prompts
                .insert(launch.id.clone(), launch.prompt.clone());
            Some(
                self.title_agent
                    .assign_title_from_prompt(&launch.id, &launch.prompt),
            )
        }
    }

    fn register_agent_feature(&mut self, agent_id: AgentId, feature: String) {
        if let Some(chat) = self.chat.as_mut() {
            chat.register_agent_feature(agent_id, feature);
        }
    }

    fn prepare_agent_launch(&mut self, args: &[String]) -> Result<AgentLaunch, CliError> {
        let chat = self
            .chat
            .as_mut()
            .expect("work-leaf controller command chat is present");
        chat.prepare_agent_launch(args)
    }

    fn promote_agent_from_command(
        &mut self,
        command: &str,
        args: &[String],
    ) -> Result<(), CliError> {
        let Some(agent_id) = args.first() else {
            return Err(CliError::Usage(format!("{command} requires an agent id")));
        };
        let agent_id = AgentId::new(agent_id.clone()).map_err(CliError::Agent)?;
        self.promote_agent_to_patch(&agent_id, &args[1..].join(" "))
    }

    fn reserve_first_chat_title(&mut self, agent_id: &AgentId, prompt: &str) -> Option<String> {
        if !agent_id.as_str().starts_with("user-") {
            return None;
        }
        let title = self
            .title_agent
            .assign_first_prompt_title(agent_id, prompt)?;
        self.pending_title_prompts
            .insert(agent_id.clone(), prompt.to_string());
        Some(title)
    }

    fn remember_agent_review_baseline(&mut self, agent_id: &AgentId) {
        if self.agent_review_baselines.contains_key(agent_id) {
            return;
        }
        let Some(root) = self
            .chat
            .as_ref()
            .map(|chat| chat.project_dir().to_path_buf())
        else {
            return;
        };
        if let Ok(Some(hash)) = GitHistory::new(root).head_hash() {
            self.agent_review_baselines.insert(agent_id.clone(), hash);
        }
    }

    fn add_session(&mut self, session: WorkLeafSession) {
        self.sessions.insert(session.id.clone(), session.clone());
        self.pending_events
            .push(WorkLeafEvent::AgentAdded { session });
    }

    fn ensure_session_exists(&self, agent_id: &AgentId) -> Result<(), CliError> {
        if self.sessions.contains_key(agent_id) {
            Ok(())
        } else {
            Err(CliError::Agent(crate::agent::AgentError::UnknownSession(
                agent_id.clone(),
            )))
        }
    }

    fn validate_dependency_target(
        &self,
        agent_id: &AgentId,
        dependency: Option<&AgentId>,
    ) -> Result<(), CliError> {
        if let Some(dependency) = dependency {
            if agent_id == dependency {
                return Err(CliError::Usage(
                    "an agent cannot depend on itself".to_string(),
                ));
            }
            self.ensure_session_exists(dependency)?;
        }
        Ok(())
    }

    fn attach_dependency(
        &mut self,
        agent_id: &AgentId,
        dependency: &AgentId,
    ) -> Result<(), CliError> {
        if agent_id == dependency {
            return Err(CliError::Usage(
                "an agent cannot depend on itself".to_string(),
            ));
        }
        self.ensure_session_exists(agent_id)?;
        self.ensure_session_exists(dependency)?;
        if let Some(session) = self.sessions.get_mut(agent_id)
            && !session
                .depends_on
                .iter()
                .any(|existing| existing == dependency)
        {
            session.depends_on.push(dependency.clone());
        }
        if let Some(session) = self.sessions.get_mut(dependency)
            && !session
                .depended_on_by
                .iter()
                .any(|existing| existing == agent_id)
        {
            session.depended_on_by.push(agent_id.clone());
        }
        self.publish_full_session(agent_id);
        self.publish_full_session(dependency);
        Ok(())
    }

    fn publish_full_session(&mut self, agent_id: &AgentId) {
        if let Some(session) = self.sessions.get(agent_id).cloned() {
            self.pending_events
                .push(WorkLeafEvent::AgentUpdated { session });
        }
    }

    fn dependency_is_closed(&self, dependency: &AgentId) -> bool {
        self.session_completion(dependency) == Some(WorkLeafCompletion::Closed)
    }

    fn detach_dependency(&mut self, agent_id: &AgentId, dependency: &AgentId) {
        if let Some(session) = self.sessions.get_mut(agent_id) {
            session.depends_on.retain(|existing| existing != dependency);
        }
        if let Some(session) = self.sessions.get_mut(dependency) {
            session
                .depended_on_by
                .retain(|existing| existing != agent_id);
        }
        self.publish_full_session(agent_id);
        self.publish_full_session(dependency);
    }

    fn defer_launch_until_dependency(
        &mut self,
        launch: AgentLaunch,
        dependency: AgentId,
        prompt_pending: bool,
    ) {
        let agent_id = launch.id.clone();
        self.set_session_loading(&agent_id, Some(WorkLeafLoading::WaitingForDependency));
        self.append_agent_line(
            &agent_id,
            format!("work-leaf: waiting for {dependency} to be marked done"),
        );
        self.pending_dependent_launches.insert(
            agent_id,
            PendingDependentLaunch {
                launch,
                dependency,
                prompt_pending,
            },
        );
    }

    fn set_pending_dependent_launch_prompt(&mut self, agent_id: &AgentId, prompt: &str) -> bool {
        let Some(pending) = self.pending_dependent_launches.get_mut(agent_id) else {
            return false;
        };
        if !pending.prompt_pending {
            return false;
        }
        pending.launch.prompt = prompt.to_string();
        pending.prompt_pending = false;
        true
    }

    fn defer_send_until_dependency(
        &mut self,
        agent_id: AgentId,
        message: String,
        dependency: AgentId,
    ) {
        self.set_session_loading(&agent_id, Some(WorkLeafLoading::WaitingForDependency));
        self.append_agent_line(
            &agent_id,
            format!("work-leaf: waiting for {dependency} to be marked done"),
        );
        self.pending_dependent_sends
            .entry(dependency)
            .or_default()
            .push(PendingDependentSend { agent_id, message });
    }

    fn cancel_pending_dependents_for_linearize(&mut self) {
        let mut pending = self
            .pending_dependent_launches
            .values()
            .map(|pending| (pending.launch.id.clone(), pending.dependency.clone()))
            .collect::<Vec<_>>();
        for (dependency, sends) in &self.pending_dependent_sends {
            pending.extend(
                sends
                    .iter()
                    .map(|send| (send.agent_id.clone(), dependency.clone())),
            );
        }
        self.pending_dependent_launches.clear();
        self.pending_dependent_sends.clear();
        for (agent_id, dependency) in pending {
            self.detach_dependency(&agent_id, &dependency);
            self.set_session_loading(&agent_id, None);
            self.append_agent_line(
                &agent_id,
                format!("work-leaf: cancelled dependency wait for {dependency} before linearize"),
            );
        }
    }

    fn stop_non_linearize_agents_for_linearize(&mut self) {
        let display_name = self.agent_display_name();
        let agent_ids = self
            .sessions
            .keys()
            .filter(|agent_id| !is_linearize_agent_id(agent_id))
            .cloned()
            .collect::<Vec<_>>();

        self.pending_launches
            .retain(|launch| is_linearize_agent_id(&launch.id));
        self.launch_starting.retain(is_linearize_agent_id);
        self.pending_agent_prompts
            .retain(|agent_id, _| is_linearize_agent_id(agent_id));
        self.review_commits_in_progress.clear();
        self.cancel_pending_dependents_for_linearize();

        for agent_id in agent_ids {
            let _ = self
                .chat
                .as_mut()
                .expect("work-leaf controller command chat is present")
                .interrupt_agent(&agent_id);
            self.stopped_for_linearize.insert(agent_id.clone());
            self.set_session_loading(&agent_id, None);
            self.append_agent_line(
                &agent_id,
                format!("work-leaf: stopped {display_name} before linearize"),
            );
        }
    }

    fn queue_or_start_launch_worker(&mut self, launch: AgentLaunch) {
        self.pending_launches.push_back(launch);
    }

    fn start_next_pending_launch(&mut self) {
        if !self.launch_starting.is_empty() {
            return;
        }
        if let Some(launch) = self.pending_launches.pop_front() {
            self.start_launch_worker(launch);
        }
    }

    fn queue_agent_prompt(&mut self, agent_id: &AgentId, message: &str) {
        self.append_agent_line(agent_id, format!("user: {message}"));
        self.pending_agent_prompts
            .entry(agent_id.clone())
            .or_default()
            .push_back(message.to_string());
    }

    fn has_pending_agent_prompt(&self, agent_id: &AgentId) -> bool {
        self.pending_agent_prompts
            .get(agent_id)
            .is_some_and(|prompts| !prompts.is_empty())
    }

    fn start_next_queued_agent_prompt(&mut self, agent_id: &AgentId) -> bool {
        if self
            .sessions
            .get(agent_id)
            .and_then(|session| session.loading)
            .is_some()
        {
            return false;
        }
        let message = self
            .pending_agent_prompts
            .get_mut(agent_id)
            .and_then(VecDeque::pop_front);
        if self
            .pending_agent_prompts
            .get(agent_id)
            .is_some_and(VecDeque::is_empty)
        {
            self.pending_agent_prompts.remove(agent_id);
        }
        let Some(message) = message else {
            return false;
        };
        self.start_send_worker_without_user_line(agent_id.clone(), message);
        true
    }

    fn start_next_queued_agent_prompts(&mut self, agent_ids: &BTreeSet<AgentId>) {
        for agent_id in agent_ids {
            self.start_next_queued_agent_prompt(agent_id);
        }
    }

    fn start_launch_worker(&mut self, launch: AgentLaunch) {
        let agent_id = launch.id.clone();
        self.set_session_loading(&agent_id, Some(WorkLeafLoading::Launching));
        self.launch_starting.insert(agent_id.clone());
        self.start_worker(Some(agent_id.clone()), move |mut chat, sender| {
            let stream_sender = sender.clone();
            let streamed_agent_ids = Arc::new(Mutex::new(BTreeSet::new()));
            let stream_seen = Arc::clone(&streamed_agent_ids);
            let display_name = chat.agent_profile().display_name.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let first_for_worker = stream_seen.lock().unwrap().insert(event_agent_id.clone());
                send_worker_stream_event(
                    &stream_sender,
                    event_agent_id,
                    event,
                    &display_name,
                    first_for_worker,
                );
            };
            match chat.launch_prepared_agent_streaming_with_ids(launch, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(agent_id),
                        result,
                        streamed_agent_ids: tracked_streamed_agent_ids(&streamed_agent_ids),
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(agent_id),
                        message: command_chat_error_text(&error),
                        streamed_agent_ids: tracked_streamed_agent_ids(&streamed_agent_ids),
                    });
                }
            }
        });
    }

    fn start_send_worker(&mut self, agent_id: AgentId, message: String) {
        self.start_send_worker_with_user_line(agent_id, message, true);
    }

    fn start_send_worker_without_user_line(&mut self, agent_id: AgentId, message: String) {
        self.start_send_worker_with_user_line(agent_id, message, false);
    }

    fn start_send_worker_with_user_line(
        &mut self,
        agent_id: AgentId,
        message: String,
        append_user_line: bool,
    ) {
        self.set_session_loading(&agent_id, Some(WorkLeafLoading::WaitingForReply));
        if append_user_line {
            self.append_agent_line(&agent_id, format!("user: {message}"));
        }
        self.start_worker(Some(agent_id.clone()), move |mut chat, sender| {
            let stream_sender = sender.clone();
            let streamed_agent_ids = Arc::new(Mutex::new(BTreeSet::new()));
            let stream_seen = Arc::clone(&streamed_agent_ids);
            let display_name = chat.agent_profile().display_name.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let first_for_worker = stream_seen.lock().unwrap().insert(event_agent_id.clone());
                send_worker_stream_event(
                    &stream_sender,
                    event_agent_id,
                    event,
                    &display_name,
                    first_for_worker,
                );
            };
            match chat.send_to_agent_streaming_with_ids(&agent_id, &message, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(agent_id),
                        result,
                        streamed_agent_ids: tracked_streamed_agent_ids(&streamed_agent_ids),
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(agent_id),
                        message: command_chat_error_text(&error),
                        streamed_agent_ids: tracked_streamed_agent_ids(&streamed_agent_ids),
                    });
                }
            }
        });
    }

    fn start_review_worker(
        &mut self,
        commit: crate::review::AgentCommit,
        reviewer_id: AgentId,
        reuse_reviewer: bool,
    ) {
        self.start_worker(Some(reviewer_id.clone()), move |mut chat, sender| {
            let stream_sender = sender.clone();
            let streamed_agent_ids = Arc::new(Mutex::new(BTreeSet::new()));
            let stream_seen = Arc::clone(&streamed_agent_ids);
            let display_name = chat.agent_profile().display_name.clone();
            let reviewed_agent_id = commit.agent_id.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let first_for_worker = stream_seen.lock().unwrap().insert(event_agent_id.clone());
                send_worker_stream_event(
                    &stream_sender,
                    event_agent_id,
                    event,
                    &display_name,
                    first_for_worker,
                );
            };
            match chat.review_commit_streaming_with_ids(
                commit,
                reviewer_id.clone(),
                reuse_reviewer,
                &mut stream,
            ) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(reviewer_id),
                        result: CommandChatResult::ReviewComplete(vec![result]),
                        streamed_agent_ids: tracked_streamed_agent_ids(&streamed_agent_ids),
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::ReviewError {
                        reviewer_id,
                        reviewed_agent_id,
                        message: error.to_string(),
                        streamed_agent_ids: tracked_streamed_agent_ids(&streamed_agent_ids),
                    });
                }
            }
        });
    }

    fn start_command_worker(&mut self, line: String) {
        self.start_worker(None, move |mut chat, sender| {
            match chat.handle_line(&line) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: None,
                        result,
                        streamed_agent_ids: BTreeSet::new(),
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: None,
                        message: command_chat_error_text(&error),
                        streamed_agent_ids: BTreeSet::new(),
                    });
                }
            }
        });
    }

    fn start_title_worker(&mut self, agent_id: AgentId, first_prompt: String) {
        self.start_worker(Some(agent_id.clone()), move |mut chat, sender| {
            if let Ok(title) = chat.generate_chat_title(&agent_id, &first_prompt) {
                let _ = sender.send(WorkerEvent::TitleGenerated { agent_id, title });
            }
        });
    }

    fn start_pending_title_worker(&mut self, agent_id: &AgentId) {
        if let Some(first_prompt) = self.pending_title_prompts.remove(agent_id) {
            self.start_title_worker(agent_id.clone(), first_prompt);
        }
    }

    fn start_command_agent_worker(&mut self, message: String) {
        let command_transcript = self.recent_command_transcript();
        self.start_worker(None, move |mut chat, sender| {
            match chat.interpret_command_agent_message(&message, &command_transcript) {
                Ok(decision) => {
                    let _ = sender.send(WorkerEvent::CommandAgentDecision { decision });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: None,
                        message: command_chat_error_text(&error),
                        streamed_agent_ids: BTreeSet::new(),
                    });
                }
            }
        });
    }

    fn recent_command_transcript(&self) -> Vec<String> {
        self.command_transcript
            .iter()
            .rev()
            .take(COMMAND_AGENT_TRANSCRIPT_LIMIT)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    fn start_worker<F>(&mut self, agent_id: Option<AgentId>, operation: F)
    where
        F: FnOnce(CommandChat<B>, Sender<WorkerEvent>) + Send + 'static,
    {
        let Some(chat) = self.chat.as_ref().cloned() else {
            return;
        };
        let (sender, receiver) = mpsc::channel();
        let panic_sender = sender.clone();
        let handle = thread::spawn(move || {
            if catch_unwind(AssertUnwindSafe(|| operation(chat, sender))).is_err() {
                let _ = panic_sender.send(WorkerEvent::WorkerPanicked { agent_id });
            }
        });
        self.workers.push(Worker { receiver, handle });
    }

    fn apply_worker_event(&mut self, event: WorkerEvent) {
        match event {
            WorkerEvent::Stream {
                agent_id,
                text,
                first_for_worker,
            } => {
                if self.stopped_for_linearize.contains(&agent_id) {
                    return;
                }
                let loading = self
                    .sessions
                    .get(&agent_id)
                    .and_then(|session| session.loading);
                if loading == Some(WorkLeafLoading::Launching) {
                    self.set_session_loading(&agent_id, Some(WorkLeafLoading::WaitingForReply));
                } else if first_for_worker
                    && let Some(count) = self.implicit_loading_agents.get_mut(&agent_id)
                {
                    *count += 1;
                } else if first_for_worker && loading.is_none() {
                    self.set_session_loading(&agent_id, Some(WorkLeafLoading::WaitingForReply));
                    self.implicit_loading_agents.insert(agent_id.clone(), 1);
                }
                if self.launch_starting.remove(&agent_id) {
                    self.start_next_pending_launch();
                }
                self.append_agent_line(&agent_id, text);
            }
            WorkerEvent::Usage { agent_id, usage } => {
                if self.stopped_for_linearize.contains(&agent_id) {
                    return;
                }
                self.record_agent_usage(&agent_id, usage);
            }
            WorkerEvent::TitleGenerated { agent_id, title } => {
                self.apply_agent_title(&agent_id, title);
            }
            WorkerEvent::CommandAgentDecision { decision } => {
                self.apply_command_agent_decision(decision);
            }
            WorkerEvent::Complete {
                agent_id,
                result,
                streamed_agent_ids,
            } => {
                let cleared_implicit_agent_ids = self.clear_implicit_loading(&streamed_agent_ids);
                if let Some(agent_id) = agent_id {
                    if self.stopped_for_linearize.contains(&agent_id) {
                        self.launch_starting.remove(&agent_id);
                        self.pending_agent_prompts.remove(&agent_id);
                        self.set_session_loading(&agent_id, None);
                        self.start_next_pending_launch();
                        return;
                    }
                    let start_review = self.should_start_review(&agent_id, &result)
                        && !self.has_pending_agent_prompt(&agent_id);
                    self.launch_starting.remove(&agent_id);
                    self.set_session_loading(&agent_id, None);
                    self.apply_agent_result(&agent_id, &result);
                    self.start_pending_title_worker(&agent_id);
                    let queued_prompt_started = self.start_next_queued_agent_prompt(&agent_id);
                    self.start_next_queued_agent_prompts(&cleared_implicit_agent_ids);
                    self.start_next_pending_launch();
                    if start_review
                        && !queued_prompt_started
                        && let Err(error) = self.start_review_for_patch_agent(&agent_id)
                    {
                        self.push_command_line(command_chat_error_text(&error));
                    }
                } else {
                    self.push_command_line(command_result_text(&result));
                    if matches!(result, CommandChatResult::Quit) {
                        self.request_quit();
                    }
                }
            }
            WorkerEvent::Error {
                agent_id,
                message,
                streamed_agent_ids,
            } => {
                let cleared_implicit_agent_ids = self.clear_implicit_loading(&streamed_agent_ids);
                if let Some(agent_id) = agent_id {
                    if self.stopped_for_linearize.contains(&agent_id) {
                        self.launch_starting.remove(&agent_id);
                        self.pending_agent_prompts.remove(&agent_id);
                        self.set_session_loading(&agent_id, None);
                        self.start_next_pending_launch();
                        return;
                    }
                    self.launch_starting.remove(&agent_id);
                    self.set_session_loading(&agent_id, None);
                    self.append_agent_line(&agent_id, message);
                    self.start_next_queued_agent_prompt(&agent_id);
                    self.start_next_queued_agent_prompts(&cleared_implicit_agent_ids);
                    self.start_next_pending_launch();
                } else {
                    self.push_command_line(message);
                    self.start_next_queued_agent_prompts(&cleared_implicit_agent_ids);
                }
            }
            WorkerEvent::ReviewError {
                reviewer_id,
                reviewed_agent_id,
                message,
                streamed_agent_ids,
            } => {
                let cleared_implicit_agent_ids = self.clear_implicit_loading(&streamed_agent_ids);
                if self.stopped_for_linearize.contains(&reviewer_id)
                    || self.stopped_for_linearize.contains(&reviewed_agent_id)
                {
                    self.review_commits_in_progress.remove(&reviewed_agent_id);
                    self.pending_agent_prompts.remove(&reviewer_id);
                    self.set_session_loading(&reviewer_id, None);
                    return;
                }
                self.review_commits_in_progress.remove(&reviewed_agent_id);
                self.set_session_loading(&reviewer_id, None);
                self.append_agent_line(&reviewer_id, message);
                self.append_agent_line_allow_duplicate(
                    &reviewer_id,
                    REVIEW_FAILED_MESSAGE.to_string(),
                );
                self.start_next_queued_agent_prompt(&reviewer_id);
                self.start_next_queued_agent_prompts(&cleared_implicit_agent_ids);
            }
            WorkerEvent::WorkerPanicked { agent_id } => {
                let message = "work-leaf: worker panicked; see daemon stderr for details";
                if let Some(agent_id) = agent_id {
                    self.launch_starting.remove(&agent_id);
                    self.set_session_loading(&agent_id, None);
                    self.append_agent_line(&agent_id, message.to_string());
                    self.start_next_queued_agent_prompt(&agent_id);
                    self.start_next_pending_launch();
                } else {
                    self.push_command_line(message.to_string());
                }
            }
        }
    }

    fn clear_implicit_loading(
        &mut self,
        streamed_agent_ids: &BTreeSet<AgentId>,
    ) -> BTreeSet<AgentId> {
        let mut cleared = BTreeSet::new();
        for agent_id in streamed_agent_ids {
            let Some(count) = self.implicit_loading_agents.get_mut(agent_id) else {
                continue;
            };
            if *count > 1 {
                *count -= 1;
            } else {
                self.implicit_loading_agents.remove(agent_id);
                self.set_session_loading(agent_id, None);
                cleared.insert(agent_id.clone());
            }
        }
        cleared
    }

    fn handle_completion_answer(&mut self, agent_id: &AgentId, message: &str) {
        self.append_agent_line(agent_id, format!("user: {message}"));
        match parse_feature_done_answer(message) {
            FeatureDoneAnswer::Yes => {
                self.set_session_completion(agent_id, Some(WorkLeafCompletion::Closed));
                self.append_agent_line(agent_id, FEATURE_CLOSED_MESSAGE.to_string());
                self.release_dependents(agent_id);
            }
            FeatureDoneAnswer::No { follow_up } => {
                self.set_session_completion(agent_id, None);
                self.append_agent_line(agent_id, FEATURE_OPEN_MESSAGE.to_string());
                if let Some(follow_up) = follow_up {
                    self.start_send_worker_without_user_line(
                        agent_id.clone(),
                        user_follow_up_fix_prompt(follow_up),
                    );
                }
            }
            FeatureDoneAnswer::Unknown => {
                self.append_agent_line(agent_id, FEATURE_DONE_ANSWER_MESSAGE.to_string());
            }
        }
    }

    fn release_dependents(&mut self, dependency: &AgentId) {
        let ready_launches = self
            .pending_dependent_launches
            .iter()
            .filter(|(_, pending)| &pending.dependency == dependency)
            .map(|(agent_id, _)| agent_id.clone())
            .collect::<Vec<_>>();
        for agent_id in ready_launches {
            if let Some(pending) = self.pending_dependent_launches.remove(&agent_id) {
                self.append_agent_line(
                    &agent_id,
                    format!("work-leaf: dependency {dependency} marked done; launching"),
                );
                self.queue_or_start_launch_worker(pending.launch);
            }
        }

        for pending in self
            .pending_dependent_sends
            .remove(dependency)
            .unwrap_or_default()
        {
            self.append_agent_line(
                &pending.agent_id,
                format!("work-leaf: dependency {dependency} marked done; sending patch task"),
            );
            self.start_send_worker(pending.agent_id, pending.message);
        }
    }

    fn apply_agent_result(&mut self, agent_id: &AgentId, result: &CommandChatResult) {
        match result {
            CommandChatResult::AgentLaunched { reply, .. }
            | CommandChatResult::AgentMessage { reply, .. } => {
                let visible_reply = self.unstreamed_agent_reply(agent_id, reply);
                if !visible_reply.is_empty() {
                    self.append_agent_line(agent_id, visible_reply);
                }
            }
            CommandChatResult::ReviewComplete(results) => {
                let text = command_result_text(result);
                self.push_command_line(text.clone());
                for review in results {
                    self.record_review_result(review);
                    let review_status = if review.findings_resolved {
                        REVIEW_FINISHED_NO_FINDINGS_MESSAGE
                    } else {
                        REVIEW_FINISHED_WITH_FINDINGS_MESSAGE
                    };
                    self.append_agent_line_allow_duplicate(
                        &review.reviewer_id,
                        review_status.to_string(),
                    );
                    self.append_agent_line(&review.commit.agent_id, format!("review: {text}"));
                    if review.findings_resolved {
                        self.ask_feature_done(&review.commit.agent_id);
                    }
                }
            }
            other => {
                self.push_command_line(command_result_text(other));
            }
        }
    }

    fn apply_agent_title(&mut self, agent_id: &AgentId, title: String) {
        for launch in &mut self.pending_launches {
            if &launch.id == agent_id {
                launch.feature = title.clone();
            }
        }
        if let Some(pending) = self.pending_dependent_launches.get_mut(agent_id) {
            pending.launch.feature = title.clone();
        }
        if let Some(session) = self.sessions.get_mut(agent_id) {
            session.title = title.clone();
            session.feature = title.clone();
            self.pending_events.push(WorkLeafEvent::AgentStatusUpdated {
                agent_id: session.id.clone(),
                kind: session.kind.clone(),
                title: session.title.clone(),
                feature: session.feature.clone(),
                loading: session.loading,
                completion: session.completion,
            });
        }
        self.register_agent_feature(agent_id.clone(), title);
    }

    fn apply_command_agent_decision(&mut self, decision: CommandAgentDecision) {
        match decision {
            CommandAgentDecision::Execute {
                command_lines,
                reply,
            } => {
                self.push_command_line(format!("command-agent: {reply}"));
                for command_line in command_lines {
                    self.execute_command_line(&command_line);
                }
            }
            CommandAgentDecision::Reply(reply) => {
                self.push_command_line(format!("command-agent: {reply}"));
            }
        }
    }

    fn append_agent_line(&mut self, agent_id: &AgentId, line: String) {
        self.append_agent_line_with_dedupe(agent_id, line, true);
    }

    fn append_agent_line_allow_duplicate(&mut self, agent_id: &AgentId, line: String) {
        self.append_agent_line_with_dedupe(agent_id, line, false);
    }

    fn record_agent_usage(&mut self, agent_id: &AgentId, usage: AgentTokenUsage) {
        let fallback_kind = self.agent_kind();
        if !self.sessions.contains_key(agent_id) {
            let session = WorkLeafSession::unknown(agent_id.clone(), fallback_kind);
            self.sessions.insert(agent_id.clone(), session.clone());
            self.pending_events
                .push(WorkLeafEvent::AgentAdded { session });
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .expect("session was inserted before recording usage");
        let token_usage = session.token_usage.unwrap_or_default().combine(usage);
        session.token_usage = Some(token_usage);
        self.pending_events.push(WorkLeafEvent::AgentUsageUpdated {
            agent_id: agent_id.clone(),
            token_usage,
        });
    }

    fn append_agent_line_with_dedupe(&mut self, agent_id: &AgentId, line: String, dedupe: bool) {
        if line.is_empty() {
            return;
        }
        let fallback_kind = self.agent_kind();
        if !self.sessions.contains_key(agent_id) {
            let session = WorkLeafSession::unknown(agent_id.clone(), fallback_kind);
            self.sessions.insert(agent_id.clone(), session.clone());
            self.pending_events
                .push(WorkLeafEvent::AgentAdded { session });
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .expect("session was inserted before appending a line");
        if dedupe && session.lines.iter().any(|existing| existing == &line) {
            return;
        }
        session.lines.push(line.clone());
        self.pending_events.push(WorkLeafEvent::AgentLineAppended {
            agent_id: agent_id.clone(),
            line,
        });
    }

    fn ask_feature_done(&mut self, agent_id: &AgentId) {
        self.implicit_loading_agents.remove(agent_id);
        self.set_session_loading(agent_id, None);
        self.set_session_completion(agent_id, Some(WorkLeafCompletion::NeedsDecision));
        self.pending_events.push(WorkLeafEvent::AgentSelected {
            agent_id: agent_id.clone(),
        });
        self.append_agent_line_allow_duplicate(agent_id, FEATURE_DONE_QUESTION.to_string());
    }

    fn record_review_result(&mut self, review: &ReviewResult) {
        self.review_commits_in_progress.remove(&review.agent_id);
        let latest_commit = self
            .latest_agent_review_commit(&review.agent_id)
            .unwrap_or_else(|| review.commit.clone());
        self.reviewed_agent_commits
            .insert(review.agent_id.clone(), latest_commit.hash.clone());
        self.agent_review_baselines
            .insert(review.agent_id.clone(), latest_commit.hash.clone());
        if let Some(chat) = self.chat.as_mut() {
            chat.mark_reviewed_agent_commit(latest_commit);
        }
        self.reviewers.insert(review.reviewer_id.clone());
    }

    fn latest_agent_review_commit(&self, agent_id: &AgentId) -> Option<AgentCommit> {
        let root = self
            .chat
            .as_ref()
            .map(|chat| chat.project_dir().to_path_buf())?;
        let boundary = self
            .reviewed_agent_commits
            .get(agent_id)
            .or_else(|| self.agent_review_baselines.get(agent_id))
            .map(String::as_str);
        GitHistory::new(root)
            .agent_review_commit(agent_id, boundary)
            .ok()?
    }

    fn should_start_review(&self, agent_id: &AgentId, result: &CommandChatResult) -> bool {
        match result {
            CommandChatResult::AgentLaunched { reply, .. }
            | CommandChatResult::AgentMessage { reply, .. } => {
                (contains_done_directive(reply) || contains_done_summary(reply, agent_id))
                    && self.has_unreviewed_agent_commit(agent_id)
            }
            _ => false,
        }
    }

    fn has_unreviewed_agent_commit(&self, agent_id: &AgentId) -> bool {
        let Some(commit) = self.latest_agent_review_commit(agent_id) else {
            return false;
        };
        self.reviewed_agent_commits
            .get(agent_id)
            .is_none_or(|hash| hash != &commit.hash)
            && self
                .review_commits_in_progress
                .get(agent_id)
                .is_none_or(|hash| hash != &commit.hash)
    }

    fn set_session_loading(&mut self, agent_id: &AgentId, loading: Option<WorkLeafLoading>) {
        let fallback_kind = self.agent_kind();
        if !self.sessions.contains_key(agent_id) {
            let session = WorkLeafSession::unknown(agent_id.clone(), fallback_kind);
            self.sessions.insert(agent_id.clone(), session.clone());
            self.pending_events
                .push(WorkLeafEvent::AgentAdded { session });
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .expect("session was inserted before updating loading");
        session.loading = loading;
        self.pending_events.push(WorkLeafEvent::AgentStatusUpdated {
            agent_id: session.id.clone(),
            kind: session.kind.clone(),
            title: session.title.clone(),
            feature: session.feature.clone(),
            loading: session.loading,
            completion: session.completion,
        });
    }

    fn set_session_completion(
        &mut self,
        agent_id: &AgentId,
        completion: Option<WorkLeafCompletion>,
    ) {
        let fallback_kind = self.agent_kind();
        if !self.sessions.contains_key(agent_id) {
            let session = WorkLeafSession::unknown(agent_id.clone(), fallback_kind);
            self.sessions.insert(agent_id.clone(), session.clone());
            self.pending_events
                .push(WorkLeafEvent::AgentAdded { session });
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .expect("session was inserted before updating completion");
        if session.completion == completion {
            return;
        }
        session.completion = completion;
        self.pending_events.push(WorkLeafEvent::AgentStatusUpdated {
            agent_id: session.id.clone(),
            kind: session.kind.clone(),
            title: session.title.clone(),
            feature: session.feature.clone(),
            loading: session.loading,
            completion: session.completion,
        });
    }

    fn session_completion(&self, agent_id: &AgentId) -> Option<WorkLeafCompletion> {
        self.sessions
            .get(agent_id)
            .and_then(|session| session.completion)
    }

    fn unstreamed_agent_reply(&self, agent_id: &AgentId, reply: &str) -> String {
        let Some(session) = self.sessions.get(agent_id) else {
            return reply.to_string();
        };
        trim_processed_agent_reply(reply, &session.lines)
    }

    fn push_command_line(&mut self, line: String) {
        if line.is_empty() {
            return;
        }
        self.command_transcript.push(line.clone());
        self.pending_events
            .push(WorkLeafEvent::CommandTranscriptLine { line });
    }

    fn request_quit(&mut self) {
        self.shutdown_agents();
        self.pending_events.push(WorkLeafEvent::QuitRequested);
    }

    fn shutdown_agents(&mut self) {
        if let Some(chat) = self.chat.as_mut() {
            chat.shutdown_agents();
        } else {
            self.shutdown.shutdown();
        }
    }

    fn agent_display_name(&self) -> String {
        self.chat
            .as_ref()
            .map(|chat| chat.agent_profile().display_name.clone())
            .unwrap_or_else(|| "agent".to_string())
    }

    fn agent_kind(&self) -> AgentKind {
        self.chat
            .as_ref()
            .map(|chat| chat.agent_profile().kind.clone())
            .unwrap_or_else(|| AgentKind::External("agent".to_string()))
    }
}

impl<B> Drop for WorkLeafController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    fn drop(&mut self) {
        if self.shutdown_on_drop {
            self.shutdown_agents();
        }
    }
}

fn contains_done_directive(text: &str) -> bool {
    text.lines().any(|line| line.trim() == "@work-leaf done")
}

fn contains_done_summary(text: &str, agent_id: &AgentId) -> bool {
    let summary = format!("agent {agent_id} reported done");
    text.lines().any(|line| line.trim() == summary)
}

fn trim_streamed_reply_blocks<'a>(reply: &'a str, streamed_lines: &[String]) -> &'a str {
    let mut remaining = reply;
    loop {
        remaining = remaining.trim_start_matches('\n');
        let block_end = remaining.find("\n\n").unwrap_or(remaining.len());
        let block = &remaining[..block_end];
        if block.is_empty() || !streamed_lines.iter().any(|line| line == block) {
            return remaining;
        }
        remaining = &remaining[block_end..];
    }
}

fn trim_processed_agent_reply(reply: &str, streamed_lines: &[String]) -> String {
    let remaining = trim_streamed_reply_blocks(reply, streamed_lines).trim_start_matches('\n');
    if remaining.is_empty() {
        return String::new();
    }
    let retained_status = retained_orchestrator_status(remaining);
    if let Some(payload) = last_agent_follow_up_payload(remaining) {
        let compacted = trim_processed_agent_reply(payload, streamed_lines);
        if compacted.is_empty() || streamed_lines.iter().any(|line| line == &compacted) {
            return retained_status;
        }
        return join_visible_reply_blocks(&retained_status, &compacted);
    }
    if let Some(index) = orchestrator_block_start(remaining) {
        let leading = remaining[..index].trim_end_matches('\n');
        if leading.is_empty() || streamed_lines.iter().any(|line| line == leading) {
            return retained_status;
        }
        return join_visible_reply_blocks(&retained_status, leading);
    }
    remaining.to_string()
}

fn retained_orchestrator_status(text: &str) -> String {
    text.split("\n\n")
        .filter_map(|block| block.strip_prefix("orchestrator:\n"))
        .flat_map(str::lines)
        .filter(|line| {
            line.starts_with("agent ") && line.contains(" reported done")
                || line.starts_with("sent file text to ")
                || line.starts_with("reported unavailable file text to ")
                || line.starts_with("sent file update to ")
                || line.starts_with("classified command for ")
                || line.starts_with("applied patch from ")
                || line.starts_with("sent patch diagnostics to ")
                || line.starts_with("ran command for ")
                || line.starts_with("routed message from ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn join_visible_reply_blocks(first: &str, second: &str) -> String {
    match (first.is_empty(), second.is_empty()) {
        (true, true) => String::new(),
        (true, false) => second.to_string(),
        (false, true) => first.to_string(),
        (false, false) => format!("{first}\n\n{second}"),
    }
}

fn last_agent_follow_up_payload(text: &str) -> Option<&str> {
    let marker = "agent follow-up from ";
    let marker_start = text.rfind(marker)?;
    let payload_start = text[marker_start..].find(":\n")? + marker_start + 2;
    Some(&text[payload_start..])
}

fn orchestrator_block_start(text: &str) -> Option<usize> {
    if text.starts_with("orchestrator:") {
        return Some(0);
    }
    text.find("\n\norchestrator:")
}

fn is_linearize_agent_id(agent_id: &AgentId) -> bool {
    let value = agent_id.as_str();
    value == "linearize" || value.starts_with("linearize-")
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkLeafSnapshot {
    pub command_transcript: Vec<String>,
    pub sessions: Vec<WorkLeafSession>,
}

impl WorkLeafSnapshot {
    pub fn session(&self, agent_id: &AgentId) -> Option<&WorkLeafSession> {
        self.sessions.iter().find(|session| &session.id == agent_id)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkLeafSession {
    pub id: AgentId,
    pub kind: AgentKind,
    pub title: String,
    pub feature: String,
    pub lines: Vec<String>,
    pub loading: Option<WorkLeafLoading>,
    pub completion: Option<WorkLeafCompletion>,
    pub token_usage: Option<AgentTokenUsage>,
    #[serde(default)]
    pub depends_on: Vec<AgentId>,
    #[serde(default)]
    pub depended_on_by: Vec<AgentId>,
}

impl WorkLeafSession {
    fn unknown(agent_id: AgentId, kind: AgentKind) -> Self {
        Self {
            id: agent_id,
            kind,
            title: "agent".to_string(),
            feature: "agent".to_string(),
            lines: Vec::new(),
            loading: None,
            completion: None,
            token_usage: None,
            depends_on: Vec::new(),
            depended_on_by: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkLeafLoading {
    Launching,
    WaitingForReply,
    WaitingForDependency,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkLeafCompletion {
    NeedsDecision,
    Closed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkLeafEvent {
    AgentAdded {
        session: WorkLeafSession,
    },
    AgentUpdated {
        session: WorkLeafSession,
    },
    AgentStatusUpdated {
        agent_id: AgentId,
        kind: AgentKind,
        title: String,
        feature: String,
        loading: Option<WorkLeafLoading>,
        completion: Option<WorkLeafCompletion>,
    },
    AgentUsageUpdated {
        agent_id: AgentId,
        token_usage: AgentTokenUsage,
    },
    AgentLineAppended {
        agent_id: AgentId,
        line: String,
    },
    AgentSelected {
        agent_id: AgentId,
    },
    CommandTranscriptLine {
        line: String,
    },
    QuitRequested,
}

#[derive(Debug)]
struct Worker {
    receiver: Receiver<WorkerEvent>,
    handle: JoinHandle<()>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingDependentLaunch {
    launch: AgentLaunch,
    dependency: AgentId,
    prompt_pending: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingDependentSend {
    agent_id: AgentId,
    message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WorkerEvent {
    Stream {
        agent_id: AgentId,
        text: String,
        first_for_worker: bool,
    },
    Usage {
        agent_id: AgentId,
        usage: AgentTokenUsage,
    },
    TitleGenerated {
        agent_id: AgentId,
        title: String,
    },
    CommandAgentDecision {
        decision: CommandAgentDecision,
    },
    Complete {
        agent_id: Option<AgentId>,
        result: CommandChatResult,
        streamed_agent_ids: BTreeSet<AgentId>,
    },
    Error {
        agent_id: Option<AgentId>,
        message: String,
        streamed_agent_ids: BTreeSet<AgentId>,
    },
    ReviewError {
        reviewer_id: AgentId,
        reviewed_agent_id: AgentId,
        message: String,
        streamed_agent_ids: BTreeSet<AgentId>,
    },
    WorkerPanicked {
        agent_id: Option<AgentId>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FeatureDoneAnswer<'a> {
    Yes,
    No { follow_up: Option<&'a str> },
    Unknown,
}

fn parse_feature_done_answer(message: &str) -> FeatureDoneAnswer<'_> {
    let message = message.trim();
    if message.eq_ignore_ascii_case("yes") {
        return FeatureDoneAnswer::Yes;
    }

    let Some(rest) = strip_no_answer_prefix(message) else {
        return FeatureDoneAnswer::Unknown;
    };
    if rest.is_empty() {
        return FeatureDoneAnswer::No { follow_up: None };
    }
    let rest = rest.trim_start();
    if rest
        .chars()
        .next()
        .is_none_or(|ch| !is_no_follow_up_punctuation(ch))
    {
        return FeatureDoneAnswer::Unknown;
    }

    let follow_up = rest.trim_start_matches(no_follow_up_separator).trim();
    FeatureDoneAnswer::No {
        follow_up: (!follow_up.is_empty()).then_some(follow_up),
    }
}

fn strip_no_answer_prefix(message: &str) -> Option<&str> {
    let bytes = message.as_bytes();
    if bytes.len() < 2
        || !bytes[0].eq_ignore_ascii_case(&b'n')
        || !bytes[1].eq_ignore_ascii_case(&b'o')
    {
        return None;
    }
    Some(&message[2..])
}

fn no_follow_up_separator(ch: char) -> bool {
    ch.is_ascii_whitespace() || is_no_follow_up_punctuation(ch)
}

fn is_no_follow_up_punctuation(ch: char) -> bool {
    matches!(ch, ',' | '.' | ':' | ';' | '-' | '!' | '?')
}

fn user_follow_up_fix_prompt(follow_up: &str) -> String {
    format!(
        "The user answered that the feature is not done and asked for follow-up fixes:\n{follow_up}\n\nMake the requested fixes through the orchestrator patch flow. When no more orchestrator work is required for this follow-up, emit `@work-leaf done` again so Work Leaf can start another review round before asking the user whether the feature is done."
    )
}

fn tracked_streamed_agent_ids(
    streamed_agent_ids: &Arc<Mutex<BTreeSet<AgentId>>>,
) -> BTreeSet<AgentId> {
    streamed_agent_ids.lock().unwrap().clone()
}

fn send_worker_stream_event(
    sender: &Sender<WorkerEvent>,
    agent_id: &AgentId,
    event: AgentStreamEvent,
    agent_display_name: &str,
    first_for_worker: bool,
) {
    match event {
        AgentStreamEvent::Usage(usage) => {
            let _ = sender.send(WorkerEvent::Usage {
                agent_id: agent_id.clone(),
                usage,
            });
        }
        event => {
            let _ = sender.send(WorkerEvent::Stream {
                agent_id: agent_id.clone(),
                text: stream_event_text(event, agent_display_name),
                first_for_worker,
            });
        }
    }
}

fn stream_event_text(event: AgentStreamEvent, agent_display_name: &str) -> String {
    let label = agent_display_name.to_ascii_lowercase();
    match event {
        AgentStreamEvent::Status(text) => format!("{label}: {text}"),
        AgentStreamEvent::AgentMessage(text) => text,
        AgentStreamEvent::Error(text) => format!("{label} error: {text}"),
        AgentStreamEvent::Usage(_) => String::new(),
    }
}

fn is_agent_slash_command_message(message: &str) -> bool {
    message.strip_prefix('/').is_some_and(|rest| {
        rest.chars()
            .next()
            .is_some_and(|first| !first.is_whitespace())
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AgentCreationRequest {
    args: Vec<String>,
    depends_on: Option<AgentId>,
    fork_from: Option<AgentId>,
}

fn parse_agent_creation_request(args: Vec<String>) -> Result<AgentCreationRequest, CliError> {
    let mut launch_args = Vec::new();
    let mut depends_on = None;
    let mut fork_from = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--depends-on" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(CliError::Usage(
                        "--depends-on requires an agent id".to_string(),
                    ));
                };
                depends_on = Some(AgentId::new(value.clone()).map_err(CliError::Agent)?);
                index += 2;
            }
            "--fork-from" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(CliError::Usage(
                        "--fork-from requires an agent id".to_string(),
                    ));
                };
                fork_from = Some(AgentId::new(value.clone()).map_err(CliError::Agent)?);
                index += 2;
            }
            other => {
                launch_args.push(other.to_string());
                index += 1;
            }
        }
    }

    Ok(AgentCreationRequest {
        args: launch_args,
        depends_on,
        fork_from,
    })
}

fn fork_launch_prompt(
    source_agent_id: &AgentId,
    backend_session: Option<&AgentSession>,
    source: &WorkLeafSession,
    prompt: &str,
) -> String {
    let source_title_words = source.title.replace('-', " ");
    let mut text = format!(
        "Fork this Work Leaf patch-agent session from {source_agent_id}.\n\nSource feature: {source_title_words}\nConversation history from {source_agent_id} [{}]:",
        source.title
    );
    if let Some(session) = backend_session {
        for message in &session.messages {
            append_history_message(&mut text, message);
        }
    } else {
        if source.lines.is_empty() {
            text.push_str("\n(no visible transcript)");
        } else {
            for line in &source.lines {
                text.push('\n');
                text.push_str(line);
            }
        }
    }
    if prompt.is_empty() {
        text.push_str("\n\nContinue with an alternate implementation path.");
    } else {
        text.push_str("\n\nFork task:\n");
        text.push_str(prompt);
    }
    text
}

fn append_history_message(text: &mut String, message: &ChatMessage) {
    let role = match message.role {
        MessageRole::User => "user",
        MessageRole::Agent => "agent",
        MessageRole::Orchestrator => "orchestrator",
        MessageRole::System => "system",
    };
    text.push('\n');
    text.push_str(role);
    text.push_str(": ");
    text.push_str(&message.text);
}

fn split_command_line(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, VecDeque};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::{Arc, Mutex};

    use super::*;

    #[derive(Clone, Debug)]
    struct EmptyBackend;

    impl AgentBackend for EmptyBackend {
        fn launch(
            &mut self,
            request: AgentLaunch,
        ) -> Result<AgentSession, crate::agent::AgentError> {
            Ok(AgentSession::new(request))
        }

        fn send(
            &mut self,
            _agent_id: &AgentId,
            _prompt: &str,
        ) -> Result<ChatMessage, crate::agent::AgentError> {
            Ok(ChatMessage::new(MessageRole::Agent, String::new()))
        }
    }

    #[derive(Clone, Debug)]
    struct ScriptedBackend {
        state: Arc<Mutex<ScriptedBackendState>>,
    }

    #[derive(Debug)]
    struct ScriptedBackendState {
        replies: VecDeque<String>,
        launches: Vec<AgentLaunch>,
        sends: Vec<(AgentId, String)>,
        sessions: BTreeMap<AgentId, AgentSession>,
    }

    impl ScriptedBackend {
        fn new<const N: usize>(replies: [&str; N]) -> Self {
            Self {
                state: Arc::new(Mutex::new(ScriptedBackendState {
                    replies: replies.into_iter().map(String::from).collect(),
                    launches: Vec::new(),
                    sends: Vec::new(),
                    sessions: BTreeMap::new(),
                })),
            }
        }

        fn launches(&self) -> Vec<AgentLaunch> {
            self.state.lock().unwrap().launches.clone()
        }
    }

    impl AgentBackend for ScriptedBackend {
        fn launch(
            &mut self,
            request: AgentLaunch,
        ) -> Result<AgentSession, crate::agent::AgentError> {
            let mut state = self.state.lock().unwrap();
            state.launches.push(request.clone());
            let agent_id = request.id.clone();
            let mut session = AgentSession::new(request);
            let reply = state.replies.pop_front().expect("missing fake reply");
            session.push_message(MessageRole::Agent, reply);
            state.sessions.insert(agent_id, session.clone());
            Ok(session)
        }

        fn send(
            &mut self,
            agent_id: &AgentId,
            prompt: &str,
        ) -> Result<ChatMessage, crate::agent::AgentError> {
            let mut state = self.state.lock().unwrap();
            state.sends.push((agent_id.clone(), prompt.to_string()));
            let reply = state.replies.pop_front().expect("missing fake reply");
            if let Some(session) = state.sessions.get_mut(agent_id) {
                session.push_message(MessageRole::User, prompt);
                session.push_message(MessageRole::Agent, reply.clone());
            }
            Ok(ChatMessage::new(MessageRole::Agent, reply))
        }

        fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
            self.state.lock().unwrap().sessions.get(agent_id).cloned()
        }
    }

    #[test]
    fn feature_done_decision_clears_patch_agent_loading() {
        let chat = CommandChat::new(PathBuf::from("/repo"), EmptyBackend);
        let mut controller = WorkLeafController::new(chat);
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");

        controller.register_agent_feature(agent_id.clone(), "visual mode".to_string());
        controller.add_session(WorkLeafSession {
            id: agent_id.clone(),
            kind: AgentKind::Codex,
            title: "user-1".to_string(),
            feature: "visual mode".to_string(),
            lines: Vec::new(),
            loading: Some(WorkLeafLoading::WaitingForReply),
            completion: None,
            token_usage: None,
            depends_on: Vec::new(),
            depended_on_by: Vec::new(),
        });
        controller
            .implicit_loading_agents
            .insert(agent_id.clone(), 1);

        controller.ask_feature_done(&agent_id);

        let snapshot = controller.snapshot();
        let session = snapshot
            .session(&agent_id)
            .expect("patch agent session exists");
        assert_eq!(session.completion, Some(WorkLeafCompletion::NeedsDecision));
        assert_eq!(session.loading, None);
        assert!(!controller.implicit_loading_agents.contains_key(&agent_id));
    }

    #[test]
    fn non_user_patch_agent_done_starts_review_and_completion_question() {
        let root = git_repo("workspace-non-user-review-completion");
        fs::write(root.join("README.md"), "before\n").unwrap();
        git(&root, ["add", "README.md"]);
        git(&root, ["commit", "-m", "ADD initial readme fixture"]);
        let backend = ScriptedBackend::new([
            "implemented patch\n@work-leaf patch update readme\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end\n@work-leaf done",
            "NO_FINDINGS",
        ]);
        let chat = CommandChat::new(root, backend.clone()).with_max_review_rounds(4);
        let mut controller = WorkLeafController::new(chat);
        let agent_id = AgentId::new("chat-9").expect("test agent id is valid");

        controller.register_agent_feature(agent_id.clone(), "review completion".to_string());
        controller.add_session(WorkLeafSession {
            id: agent_id.clone(),
            kind: AgentKind::Codex,
            title: "chat-9".to_string(),
            feature: "review completion".to_string(),
            lines: Vec::new(),
            loading: None,
            completion: None,
            token_usage: None,
            depends_on: Vec::new(),
            depended_on_by: Vec::new(),
        });

        controller
            .send_message(&agent_id, "finish review completion")
            .expect("patch-agent message is accepted");
        assert!(controller.wait_for_idle(Duration::from_secs(2)));

        let reviewer_id = AgentId::new("review-chat-9").expect("test reviewer id is valid");
        assert!(
            backend
                .launches()
                .iter()
                .any(|launch| launch.id == reviewer_id),
            "clean review should launch for non-user patch-agent ids"
        );
        let snapshot = controller.snapshot();
        let patch_agent = snapshot
            .session(&agent_id)
            .expect("patch-agent session exists");
        assert_eq!(
            patch_agent.completion,
            Some(WorkLeafCompletion::NeedsDecision)
        );
        assert!(
            patch_agent
                .lines
                .iter()
                .any(|line| line == "work-leaf: is this feature done? [yes/no]"),
            "{patch_agent:?}"
        );
        assert!(
            patch_agent.lines.iter().any(|line| {
                line.contains("chat-9 reviewed by review-chat-9: rounds=1 resolved=yes")
            }),
            "{patch_agent:?}"
        );
        let reviewer = snapshot
            .session(&reviewer_id)
            .expect("reviewer session exists");
        assert!(
            reviewer
                .lines
                .iter()
                .any(|line| line == REVIEW_FINISHED_NO_FINDINGS_MESSAGE),
            "{reviewer:?}"
        );
    }

    #[test]
    fn inline_patch_agent_title_comes_from_title_agent() {
        let chat = CommandChat::new(PathBuf::from("/repo"), EmptyBackend);
        let mut controller = WorkLeafController::new(chat);

        let agent_id = controller
            .create_agent("it looks like that we there have been a bad regression chat name for patch agents is not created by the system agent but it has to summarize it")
            .expect("agent launch is prepared");

        let snapshot = controller.snapshot();
        let session = snapshot
            .session(&agent_id)
            .expect("created patch agent session exists");
        assert_eq!(session.title, "bad-regression-chat-name-patch-agents");
        assert_eq!(session.feature, "bad-regression-chat-name-patch-agents");
    }

    #[test]
    fn empty_new_session_uses_title_agent_for_first_task() {
        let chat = CommandChat::new(PathBuf::from("/repo"), EmptyBackend);
        let mut controller = WorkLeafController::new(chat);
        let agent_id = controller
            .create_agent("")
            .expect("empty agent launch is prepared");

        controller.wait_for_idle(Duration::from_secs(1));
        controller
            .send_message(
                &agent_id,
                "it looks like that we there have been a bad regression chat name for patch agents is not created by the system agent but it has to summarize it",
            )
            .expect("first task message is accepted");

        let snapshot = controller.snapshot();
        let session = snapshot
            .session(&agent_id)
            .expect("created patch agent session exists");
        assert_eq!(session.title, "bad-regression-chat-name-patch-agents");
        assert_eq!(session.feature, "bad-regression-chat-name-patch-agents");
    }

    fn git_repo(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        git(&root, ["init"]);
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
            "git failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
