use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::agent::{
    AgentBackend, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, AgentTokenUsage, ChatMessage, MessageRole,
};
use crate::chat_title::{ChatTitleAgent, fallback_chat_title_from_prompt};
use crate::cli::{
    CliError, CommandChat, CommandChatResult, command_chat_error_text, command_result_text,
    render_command_chat_help,
};
use crate::review::{AgentCommit, GitHistory, ReviewResult};

const FEATURE_DONE_QUESTION: &str = "work-leaf: is this feature done? [yes/no]";
const FEATURE_CLOSED_MESSAGE: &str = "work-leaf: feature marked closed";
const FEATURE_OPEN_MESSAGE: &str = "work-leaf: feature remains open";
const FEATURE_DONE_ANSWER_MESSAGE: &str = "work-leaf: answer yes or no to close this feature";

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
    launch_starting: BTreeSet<AgentId>,
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
            launch_starting: BTreeSet::new(),
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
            "review" => {
                if let Err(error) = self.start_review() {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            "linearize" => {
                if let Err(error) = self.start_linearize() {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            _ => self.start_command_worker(trimmed.to_string()),
        }
    }

    pub fn send_command_agent_message(&mut self, message: &str) {
        let message = message.trim();
        if message.is_empty() {
            return;
        }

        self.push_command_line(format!("user: {message}"));
        let display_name = self.agent_display_name();
        if literal_command_line(message).is_none()
            && let Some(request) = command_agent_new_request(message)
        {
            self.push_command_line(format!(
                "command-agent: {}",
                command_agent_launch_reply(&display_name, &request)
            ));
            let command_line = command_agent_new_command_line(&request.prompt);
            for _ in 0..request.count {
                self.execute_command_line(&command_line);
            }
            return;
        }

        match command_agent_response(message, &display_name) {
            CommandAgentResponse::Execute {
                command_line,
                reply,
            } => {
                self.push_command_line(format!("command-agent: {reply}"));
                self.execute_command_line(&command_line);
            }
            CommandAgentResponse::Reply(reply) => {
                self.push_command_line(format!("command-agent: {reply}"));
            }
        }
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

        let title_pending = request.args.is_empty();
        let launch = self.prepare_agent_launch(&request.args)?;
        let agent_id = launch.id.clone();
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
                self.defer_launch_until_dependency(launch, dependency);
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
        if self
            .sessions
            .get(agent_id)
            .and_then(|session| session.loading)
            .is_some()
        {
            self.append_agent_line(
                agent_id,
                format!("work-leaf: {} is still working", self.agent_display_name()),
            );
            return Ok(());
        }

        if self.session_completion(agent_id) == Some(WorkLeafCompletion::Closed) {
            self.set_session_completion(agent_id, None);
        }

        if let Some(title) = self.reserve_first_chat_title(agent_id, message) {
            self.apply_agent_title(agent_id, title);
        }
        self.set_session_loading(agent_id, Some(WorkLeafLoading::WaitingForReply));
        self.append_agent_line(agent_id, format!("user: {message}"));
        let agent_id = agent_id.clone();
        let message = message.to_string();
        self.start_worker(Some(agent_id.clone()), move |mut chat, sender| {
            let stream_sender = sender.clone();
            let display_name = chat.agent_profile().display_name.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                send_worker_stream_event(&stream_sender, event_agent_id, event, &display_name);
            };
            match chat.send_to_agent_streaming_with_ids(&agent_id, &message, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(agent_id),
                        result,
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(agent_id),
                        message: command_chat_error_text(&error),
                    });
                }
            }
        });
        Ok(())
    }

    pub fn promote_agent_to_patch(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
    ) -> Result<(), CliError> {
        self.ensure_session_exists(agent_id)?;
        let request = parse_agent_creation_request(split_command_line(prompt))?;
        let prompt = request.args.join(" ");
        let promotion_prompt = if prompt.is_empty() {
            "Continue this existing Work Leaf session as a patch agent. Report the broad feature before proposing patches, follow the patch-agent instructions, and use the orchestrator patch flow for file changes.".to_string()
        } else {
            format!(
                "Continue this existing Work Leaf session as a patch agent.\n\nPatch task:\n{prompt}\n\nReport the broad feature before proposing patches, follow the patch-agent instructions, and use the orchestrator patch flow for file changes."
            )
        };
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
        let backend_session = self
            .chat
            .as_ref()
            .and_then(|chat| chat.agent_session(source_agent_id));
        let launch_prompt =
            fork_launch_prompt(source_agent_id, backend_session.as_ref(), &source, &fork_prompt);
        let launch = self.prepare_agent_launch(&[launch_prompt])?;
        let agent_id = launch.id.clone();
        self.remember_agent_review_baseline(&agent_id);
        let title = if fork_prompt.is_empty() {
            format!("{} fork", source.title)
        } else {
            fallback_chat_title_from_prompt(&fork_prompt)
        };
        self.title_agent.mark_named(&agent_id);
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
                self.defer_launch_until_dependency(launch, dependency);
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
            self.title_agent.mark_named(&launch.id);
            Some(fallback_chat_title_from_prompt(&launch.prompt))
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

    fn reserve_first_chat_title(&mut self, agent_id: &AgentId, prompt: &str) -> Option<String> {
        if !agent_id.as_str().starts_with("user-") {
            return None;
        }
        if !self.title_agent.reserve_first_prompt_title(agent_id) {
            return None;
        }
        Some(fallback_chat_title_from_prompt(prompt))
    }

    fn remember_agent_review_baseline(&mut self, agent_id: &AgentId) {
        if !agent_id.as_str().starts_with("user-")
            || self.agent_review_baselines.contains_key(agent_id)
        {
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

    fn defer_launch_until_dependency(&mut self, launch: AgentLaunch, dependency: AgentId) {
        let agent_id = launch.id.clone();
        self.set_session_loading(&agent_id, Some(WorkLeafLoading::WaitingForDependency));
        self.append_agent_line(
            &agent_id,
            format!("work-leaf: waiting for {dependency} to be marked done"),
        );
        self.pending_dependent_launches
            .insert(agent_id, PendingDependentLaunch { launch, dependency });
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
        self.review_commits_in_progress.clear();
        self.pending_dependent_launches.clear();
        self.pending_dependent_sends.clear();

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

    fn start_launch_worker(&mut self, launch: AgentLaunch) {
        let agent_id = launch.id.clone();
        self.set_session_loading(&agent_id, Some(WorkLeafLoading::Launching));
        self.launch_starting.insert(agent_id.clone());
        self.start_worker(Some(agent_id.clone()), move |mut chat, sender| {
            let stream_sender = sender.clone();
            let display_name = chat.agent_profile().display_name.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                send_worker_stream_event(&stream_sender, event_agent_id, event, &display_name);
            };
            match chat.launch_prepared_agent_streaming_with_ids(launch, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(agent_id),
                        result,
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(agent_id),
                        message: command_chat_error_text(&error),
                    });
                }
            }
        });
    }

    fn start_send_worker(&mut self, agent_id: AgentId, message: String) {
        self.set_session_loading(&agent_id, Some(WorkLeafLoading::WaitingForReply));
        self.append_agent_line(&agent_id, format!("user: {message}"));
        self.start_worker(Some(agent_id.clone()), move |mut chat, sender| {
            let stream_sender = sender.clone();
            let display_name = chat.agent_profile().display_name.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                send_worker_stream_event(&stream_sender, event_agent_id, event, &display_name);
            };
            match chat.send_to_agent_streaming_with_ids(&agent_id, &message, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(agent_id),
                        result,
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(agent_id),
                        message: command_chat_error_text(&error),
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
            let display_name = chat.agent_profile().display_name.clone();
            let reviewed_agent_id = commit.agent_id.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                send_worker_stream_event(&stream_sender, event_agent_id, event, &display_name);
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
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::ReviewError {
                        reviewer_id,
                        reviewed_agent_id,
                        message: error.to_string(),
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
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: None,
                        message: command_chat_error_text(&error),
                    });
                }
            }
        });
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
            WorkerEvent::Stream { agent_id, text } => {
                if self.stopped_for_linearize.contains(&agent_id) {
                    return;
                }
                if text.contains("Codex is working") {
                    self.launch_starting.remove(&agent_id);
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
            WorkerEvent::Complete { agent_id, result } => {
                if let Some(agent_id) = agent_id {
                    if self.stopped_for_linearize.contains(&agent_id) {
                        self.launch_starting.remove(&agent_id);
                        self.set_session_loading(&agent_id, None);
                        self.start_next_pending_launch();
                        return;
                    }
                    let start_review = self.should_start_review(&agent_id, &result);
                    self.launch_starting.remove(&agent_id);
                    self.set_session_loading(&agent_id, None);
                    self.apply_agent_result(&agent_id, &result);
                    self.start_next_pending_launch();
                    if start_review && let Err(error) = self.start_review_for_patch_agent(&agent_id)
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
            WorkerEvent::Error { agent_id, message } => {
                if let Some(agent_id) = agent_id {
                    if self.stopped_for_linearize.contains(&agent_id) {
                        self.launch_starting.remove(&agent_id);
                        self.set_session_loading(&agent_id, None);
                        self.start_next_pending_launch();
                        return;
                    }
                    self.launch_starting.remove(&agent_id);
                    self.set_session_loading(&agent_id, None);
                    self.append_agent_line(&agent_id, message);
                    self.start_next_pending_launch();
                } else {
                    self.push_command_line(message);
                }
            }
            WorkerEvent::ReviewError {
                reviewer_id,
                reviewed_agent_id,
                message,
            } => {
                if self.stopped_for_linearize.contains(&reviewer_id)
                    || self.stopped_for_linearize.contains(&reviewed_agent_id)
                {
                    self.review_commits_in_progress.remove(&reviewed_agent_id);
                    self.set_session_loading(&reviewer_id, None);
                    return;
                }
                self.review_commits_in_progress.remove(&reviewed_agent_id);
                self.set_session_loading(&reviewer_id, None);
                self.append_agent_line(&reviewer_id, message);
            }
            WorkerEvent::WorkerPanicked { agent_id } => {
                let message = "work-leaf: worker panicked; see daemon stderr for details";
                if let Some(agent_id) = agent_id {
                    self.launch_starting.remove(&agent_id);
                    self.set_session_loading(&agent_id, None);
                    self.append_agent_line(&agent_id, message.to_string());
                    self.start_next_pending_launch();
                } else {
                    self.push_command_line(message.to_string());
                }
            }
        }
    }

    fn handle_completion_answer(&mut self, agent_id: &AgentId, message: &str) {
        self.append_agent_line(agent_id, format!("user: {message}"));
        match message.to_ascii_lowercase().as_str() {
            "yes" => {
                self.set_session_completion(agent_id, Some(WorkLeafCompletion::Closed));
                self.append_agent_line(agent_id, FEATURE_CLOSED_MESSAGE.to_string());
                self.release_dependents(agent_id);
            }
            "no" => {
                self.set_session_completion(agent_id, None);
                self.append_agent_line(agent_id, FEATURE_OPEN_MESSAGE.to_string());
            }
            _ => {
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
        self.set_session_completion(agent_id, Some(WorkLeafCompletion::NeedsDecision));
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
        agent_id.as_str().starts_with("user-")
            && match result {
                CommandChatResult::AgentLaunched { reply, .. }
                | CommandChatResult::AgentMessage { reply, .. } => {
                    contains_done_directive(reply) && self.has_unreviewed_agent_commit(agent_id)
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
        trim_streamed_reply_blocks(reply, &session.lines).to_string()
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
    text.lines()
        .any(|line| line.trim_start() == "@work-leaf done")
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
    },
    Usage {
        agent_id: AgentId,
        usage: AgentTokenUsage,
    },
    Complete {
        agent_id: Option<AgentId>,
        result: CommandChatResult,
    },
    Error {
        agent_id: Option<AgentId>,
        message: String,
    },
    ReviewError {
        reviewer_id: AgentId,
        reviewed_agent_id: AgentId,
        message: String,
    },
    WorkerPanicked {
        agent_id: Option<AgentId>,
    },
}

fn send_worker_stream_event(
    sender: &Sender<WorkerEvent>,
    agent_id: &AgentId,
    event: AgentStreamEvent,
    agent_display_name: &str,
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum CommandAgentResponse {
    Execute { command_line: String, reply: String },
    Reply(String),
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommandAgentNewRequest {
    count: usize,
    prompt: String,
}

fn command_agent_response(message: &str, agent_display_name: &str) -> CommandAgentResponse {
    if let Some(command_line) = literal_command_line(message) {
        return CommandAgentResponse::Execute {
            reply: format!("running `{command_line}`"),
            command_line,
        };
    }

    let lower = message.to_ascii_lowercase();
    if asks_for_new_agent(&lower) {
        let prompt = command_agent_new_prompt(message);
        let command_line = if prompt.is_empty() {
            "new".to_string()
        } else {
            format!("new {prompt}")
        };
        let reply = if prompt.is_empty() {
            format!("launching {agent_display_name} user agent")
        } else {
            format!("launching {agent_display_name} user agent for {prompt}")
        };
        return CommandAgentResponse::Execute {
            command_line,
            reply,
        };
    }

    for (needle, command_line) in [
        ("linearize", "linearize"),
        ("linearise", "linearize"),
        ("review", "review"),
        ("help", "help"),
        ("quit", "quit"),
        ("exit", "quit"),
    ] {
        if lower.contains(needle) {
            return CommandAgentResponse::Execute {
                command_line: command_line.to_string(),
                reply: format!("running `{command_line}`"),
            };
        }
    }

    CommandAgentResponse::Reply(
        "I can run help, new [prompt...], review, linearize, or quit.".to_string(),
    )
}

fn literal_command_line(message: &str) -> Option<String> {
    let command = split_command_line(message).into_iter().next()?;
    matches!(
        command.as_str(),
        "help" | "?" | "new" | "review" | "linearize" | "quit" | "exit" | "q"
    )
    .then(|| message.to_string())
}

fn asks_for_new_agent(lower: &str) -> bool {
    lower.contains("agent")
        && ["new", "spawn", "create", "start", "launch"]
            .iter()
            .any(|verb| lower.contains(verb))
}

fn command_agent_new_request(message: &str) -> Option<CommandAgentNewRequest> {
    let lower = message.to_ascii_lowercase();
    if !asks_for_agent_launch_request(&lower) {
        return None;
    }

    let prompt = command_agent_launch_prompt(message);
    let count = agent_launch_count(&prompt).unwrap_or(1);
    let prompt = strip_agent_launch_count_and_noun(&prompt);
    Some(CommandAgentNewRequest {
        count,
        prompt: normalize_common_agent_typos(&prompt),
    })
}

fn asks_for_agent_launch_request(lower: &str) -> bool {
    lower.contains("agent")
        && ["new", "spawn", "create", "start", "launch", "open", "make"]
            .iter()
            .any(|verb| lower.contains(verb))
}

fn command_agent_launch_prompt(message: &str) -> String {
    let trimmed = strip_polite_prefix(message.trim());
    [
        "open a new ",
        "open new ",
        "open an ",
        "open a ",
        "open ",
        "spawn a new ",
        "spawn new ",
        "spawn an ",
        "spawn a ",
        "spawn ",
        "create a new ",
        "create new ",
        "create an ",
        "create a ",
        "create ",
        "start a new ",
        "start new ",
        "start an ",
        "start a ",
        "start ",
        "launch a new ",
        "launch new ",
        "launch an ",
        "launch a ",
        "launch ",
        "make a new ",
        "make new ",
        "make an ",
        "make a ",
        "make ",
        "new an ",
        "new a ",
        "new ",
    ]
    .iter()
    .find_map(|prefix| strip_ascii_prefix_case_insensitive(trimmed, prefix))
    .unwrap_or(trimmed)
    .to_string()
}

fn command_agent_new_command_line(prompt: &str) -> String {
    if prompt.is_empty() {
        "new".to_string()
    } else {
        format!("new {prompt}")
    }
}

fn command_agent_launch_reply(
    agent_display_name: &str,
    request: &CommandAgentNewRequest,
) -> String {
    let count_prefix = if request.count > 1 {
        format!("{} ", request.count)
    } else {
        String::new()
    };
    let agent_label = if request.count == 1 {
        "user agent"
    } else {
        "user agents"
    };

    if request.prompt.is_empty() {
        format!("launching {count_prefix}{agent_display_name} {agent_label}")
    } else {
        format!(
            "launching {count_prefix}{agent_display_name} {agent_label} for {}",
            request.prompt
        )
    }
}

fn agent_launch_count(text: &str) -> Option<usize> {
    text.split_whitespace().next().and_then(agent_count_word)
}

fn strip_agent_launch_count_and_noun(prompt: &str) -> String {
    let words = prompt.split_whitespace().collect::<Vec<_>>();
    let mut start = 0;
    let mut end = words.len();
    if words
        .first()
        .and_then(|word| agent_count_word(word))
        .is_some()
    {
        start = 1;
    }
    if words.last().is_some_and(|word| is_agent_noun(word)) {
        end -= 1;
    }
    words[start..end].join(" ")
}

fn agent_count_word(word: &str) -> Option<usize> {
    let clean = clean_agent_word(word);
    if let Ok(count) = clean.parse::<usize>() {
        return (count > 0).then_some(count);
    }

    match clean.as_str() {
        "a" | "an" | "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        _ => None,
    }
}

fn is_agent_noun(word: &str) -> bool {
    matches!(clean_agent_word(word).as_str(), "agent" | "agents")
}

fn normalize_common_agent_typos(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            if clean_agent_word(word) == "pacth" {
                "patch"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn clean_agent_word(word: &str) -> String {
    word.trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
        .to_ascii_lowercase()
}

fn command_agent_new_prompt(message: &str) -> String {
    let trimmed = strip_polite_prefix(message.trim());
    [
        "spawn a new ",
        "spawn new ",
        "create a new ",
        "create new ",
        "start a new ",
        "start new ",
        "launch a new ",
        "launch new ",
        "make a new ",
        "make new ",
        "new ",
    ]
    .iter()
    .find_map(|prefix| strip_ascii_prefix_case_insensitive(trimmed, prefix))
    .unwrap_or(trimmed)
    .to_string()
}

fn strip_polite_prefix(message: &str) -> &str {
    ["please ", "can you ", "could you ", "would you "]
        .iter()
        .find_map(|prefix| strip_ascii_prefix_case_insensitive(message, prefix))
        .unwrap_or(message)
}

fn strip_ascii_prefix_case_insensitive<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    message
        .to_ascii_lowercase()
        .starts_with(prefix)
        .then(|| message[prefix.len()..].trim())
}

fn split_command_line(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}
