use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use serde_json::{Value, json};

use crate::agent::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, AgentTokenUsage, ChatMessage, MessageRole, PromptPolicy,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaudeCommandConfig {
    pub binary: PathBuf,
    pub project_dir: PathBuf,
    pub model: Option<String>,
    pub read_tools: bool,
}

impl ClaudeCommandConfig {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            binary: PathBuf::from("claude"),
            project_dir,
            model: None,
            read_tools: false,
        }
    }

    pub fn with_binary(mut self, binary: impl Into<PathBuf>) -> Self {
        self.binary = binary.into();
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_read_tools(mut self, read_tools: bool) -> Self {
        self.read_tools = read_tools;
        self
    }
}

#[derive(Debug)]
pub struct ClaudeBackend {
    config: ClaudeCommandConfig,
    policy: PromptPolicy,
    state: Arc<Mutex<ClaudeBackendState>>,
    operation_condvar: Arc<Condvar>,
    shutdown: AgentShutdownHandle,
}

impl Clone for ClaudeBackend {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            policy: self.policy.clone(),
            state: self.state.clone(),
            operation_condvar: self.operation_condvar.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

#[derive(Debug, Default)]
struct ClaudeBackendState {
    sessions: BTreeMap<AgentId, AgentSession>,
    session_ids: BTreeMap<AgentId, String>,
    usage: BTreeMap<AgentId, AgentTokenUsage>,
    active_agent_operations: BTreeSet<AgentId>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ClaudeTurnOutput {
    session_id: String,
    reply: String,
    usage: Option<AgentTokenUsage>,
}

impl ClaudeBackend {
    pub fn new(config: ClaudeCommandConfig, policy: PromptPolicy) -> Self {
        Self {
            config,
            policy,
            state: Arc::new(Mutex::new(ClaudeBackendState::default())),
            operation_condvar: Arc::new(Condvar::new()),
            shutdown: AgentShutdownHandle::default(),
        }
    }

    pub fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        self.state
            .lock()
            .expect("claude backend state mutex poisoned")
            .sessions
            .get(agent_id)
            .cloned()
    }

    fn run_turn_streaming(
        &self,
        prompt: &str,
        resume_session_id: Option<&str>,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ClaudeTurnOutput, AgentError> {
        let mut command = Command::new(&self.config.binary);
        command
            .current_dir(&self.config.project_dir)
            .arg("--print")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--include-partial-messages")
            .arg("--permission-mode")
            .arg("dontAsk")
            .arg("--tools")
            .arg(if self.config.read_tools {
                "Read,Glob,Grep"
            } else {
                ""
            })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(model) = &self.config.model {
            command.arg("--model").arg(model);
        }
        if let Some(session_id) = resume_session_id {
            command.arg("--resume").arg(session_id);
        }
        remove_parent_claude_environment(&mut command);

        let mut child = command.spawn().map_err(|error| {
            backend_error(format!(
                "failed to start Claude executable `{}`: {error}",
                self.config.binary.display()
            ))
        })?;
        let process_guard = self.shutdown.register_single_process(child.id());
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| backend_error("Claude stdin was not piped".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| backend_error("Claude stdout was not piped".to_string()))?;
        let stderr = child.stderr.take();
        let stderr_reader = stderr.map(|stderr| thread::spawn(move || read_to_string(stderr)));

        let input = json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": prompt,
            },
            "parent_tool_use_id": null,
        });
        writeln!(stdin, "{input}").map_err(AgentError::Io)?;
        drop(stdin);

        let mut output = ClaudeTurnOutput::default();
        if let Some(session_id) = resume_session_id {
            output.session_id = session_id.to_string();
        }
        let mut streamed_reply = String::new();
        let mut assistant_reply = String::new();
        let mut result_reply = None;
        let mut result_error = None;

        for line in BufReader::new(stdout).lines() {
            let line = line.map_err(AgentError::Io)?;
            if line.trim().is_empty() {
                continue;
            }
            let value = serde_json::from_str::<Value>(&line).map_err(|error| {
                backend_error(format!("Claude emitted invalid JSON: {error}: {line}"))
            })?;
            handle_claude_event(
                &value,
                &mut output,
                &mut streamed_reply,
                &mut assistant_reply,
                &mut result_reply,
                &mut result_error,
                sink,
            )?;
        }

        let status = child.wait().map_err(AgentError::Io)?;
        drop(process_guard);
        let stderr = stderr_reader
            .map(|reader| {
                reader
                    .join()
                    .unwrap_or_else(|_| "Claude stderr reader panicked".to_string())
            })
            .unwrap_or_default();
        if !status.success() {
            return Err(AgentError::ProcessFailed {
                program: self.config.binary.clone(),
                status: status.code(),
                stderr,
            });
        }
        if let Some(error) = result_error {
            return Err(backend_error(error));
        }
        output.reply = result_reply
            .or_else(|| (!assistant_reply.is_empty()).then_some(assistant_reply))
            .unwrap_or(streamed_reply);
        Ok(output)
    }

    fn record_launch_reply(
        &mut self,
        request: AgentLaunch,
        output: ClaudeTurnOutput,
    ) -> AgentSession {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, output.reply);
        let mut state = self
            .state
            .lock()
            .expect("claude backend state mutex poisoned");
        if !output.session_id.is_empty() {
            state
                .session_ids
                .insert(session.id.clone(), output.session_id);
        }
        if let Some(usage) = output.usage {
            record_usage(&mut state, &session.id, usage);
        }
        state.sessions.insert(session.id.clone(), session.clone());
        session
    }

    fn record_send_reply(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        output: ClaudeTurnOutput,
    ) -> Result<ChatMessage, AgentError> {
        let message = ChatMessage::new(MessageRole::Agent, output.reply);
        let mut state = self
            .state
            .lock()
            .expect("claude backend state mutex poisoned");
        if !output.session_id.is_empty() {
            state
                .session_ids
                .insert(agent_id.clone(), output.session_id);
        }
        if let Some(usage) = output.usage {
            record_usage(&mut state, agent_id, usage);
        }
        if let Some(session) = state.sessions.get_mut(agent_id) {
            session.push_message(MessageRole::User, prompt);
        } else {
            state.sessions.insert(
                agent_id.clone(),
                AgentSession::new(AgentLaunch::new(
                    agent_id.clone(),
                    AgentKind::External("claude".to_string()),
                    "unknown",
                    prompt,
                )),
            );
        }
        let session = state
            .sessions
            .get_mut(agent_id)
            .ok_or_else(|| AgentError::UnknownSession(agent_id.clone()))?;
        session.messages.push(message.clone());
        Ok(message)
    }

    fn acquire_agent_operation(&self, agent_id: &AgentId) -> AgentOperationGuard {
        let mut state = self
            .state
            .lock()
            .expect("claude backend state mutex poisoned");
        while state.active_agent_operations.contains(agent_id) {
            state = self
                .operation_condvar
                .wait(state)
                .expect("claude backend state mutex poisoned");
        }
        state.active_agent_operations.insert(agent_id.clone());
        AgentOperationGuard {
            agent_id: agent_id.clone(),
            state: self.state.clone(),
            condvar: self.operation_condvar.clone(),
        }
    }
}

impl AgentBackend for ClaudeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.launch_streaming(request, &mut |_| {})
    }

    fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        ClaudeBackend::session(self, agent_id)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.send_streaming(agent_id, prompt, &mut |_| {})
    }

    fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.shutdown.clone()
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        let _operation_guard = self.acquire_agent_operation(&request.id);
        let prompt = self
            .policy
            .inject(&request.id, &request.feature, &request.prompt);
        let output = self.run_turn_streaming(&prompt, None, sink)?;
        Ok(self.record_launch_reply(request, output))
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        let _operation_guard = self.acquire_agent_operation(agent_id);
        let (has_session, feature, claude_session_id) = {
            let state = self
                .state
                .lock()
                .expect("claude backend state mutex poisoned");
            let feature = state
                .sessions
                .get(agent_id)
                .map(|session| session.feature.clone())
                .unwrap_or_else(|| "unknown".to_string());
            (
                state.sessions.contains_key(agent_id),
                feature,
                state.session_ids.get(agent_id).cloned(),
            )
        };
        let prompt = if has_session {
            prompt.to_string()
        } else {
            self.policy.inject(agent_id, &feature, prompt)
        };
        let output = self.run_turn_streaming(&prompt, claude_session_id.as_deref(), sink)?;
        self.record_send_reply(agent_id, &prompt, output)
    }
}

#[derive(Debug)]
struct AgentOperationGuard {
    agent_id: AgentId,
    state: Arc<Mutex<ClaudeBackendState>>,
    condvar: Arc<Condvar>,
}

impl Drop for AgentOperationGuard {
    fn drop(&mut self) {
        self.state
            .lock()
            .expect("claude backend state mutex poisoned")
            .active_agent_operations
            .remove(&self.agent_id);
        self.condvar.notify_all();
    }
}

fn handle_claude_event(
    value: &Value,
    output: &mut ClaudeTurnOutput,
    streamed_reply: &mut String,
    assistant_reply: &mut String,
    result_reply: &mut Option<String>,
    result_error: &mut Option<String>,
    sink: &mut dyn FnMut(AgentStreamEvent),
) -> Result<(), AgentError> {
    let event_type = json_str(value, &["type"]).unwrap_or_default();
    match event_type {
        "system" => handle_system_event(value, output, sink),
        "stream_event" => handle_stream_event(value, streamed_reply, sink),
        "assistant" => {
            if assistant_reply.is_empty() {
                *assistant_reply = assistant_message_text(value);
            }
        }
        "result" => {
            if let Some(session_id) = json_str(value, &["session_id"]) {
                output.session_id = session_id.to_string();
            }
            output.usage = usage_from_value(value);
            if value
                .get("is_error")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                *result_error = Some(
                    json_str(value, &["result"])
                        .unwrap_or("Claude returned an error result")
                        .to_string(),
                );
            } else if let Some(result) = json_str(value, &["result"]) {
                *result_reply = Some(result.to_string());
            }
        }
        "rate_limit_event" | "user" => {}
        _ => {}
    }
    Ok(())
}

fn handle_system_event(
    value: &Value,
    output: &mut ClaudeTurnOutput,
    sink: &mut dyn FnMut(AgentStreamEvent),
) {
    match json_str(value, &["subtype"]) {
        Some("init") => {
            if let Some(session_id) = json_str(value, &["session_id"]) {
                let first_session_id = output.session_id.is_empty();
                output.session_id = session_id.to_string();
                if first_session_id {
                    sink(AgentStreamEvent::Status(format!(
                        "Claude session {session_id}"
                    )));
                }
            }
        }
        Some("status") => {
            let status = json_str(value, &["status"]).unwrap_or_default();
            if status == "requesting" {
                sink(AgentStreamEvent::Status("Claude is working".to_string()));
            } else if !status.is_empty() {
                sink(AgentStreamEvent::Status(format!("Claude status: {status}")));
            }
        }
        Some("api_retry") => {
            let attempt = json_u64(value, &["attempt"]).unwrap_or_default();
            let max_retries = json_u64(value, &["max_retries"]).unwrap_or_default();
            if attempt > 0 && max_retries > 0 {
                sink(AgentStreamEvent::Status(format!(
                    "Claude API retry {attempt}/{max_retries}"
                )));
            } else {
                sink(AgentStreamEvent::Status("Claude API retry".to_string()));
            }
        }
        _ => {}
    }
}

fn handle_stream_event(
    value: &Value,
    streamed_reply: &mut String,
    sink: &mut dyn FnMut(AgentStreamEvent),
) {
    let event = value.get("event").unwrap_or(&Value::Null);
    match json_str(event, &["type"]) {
        Some("content_block_delta") => {
            if json_str(event, &["delta", "type"]) == Some("text_delta")
                && let Some(text) = json_str(event, &["delta", "text"])
            {
                streamed_reply.push_str(text);
                sink(AgentStreamEvent::AgentMessage(text.to_string()));
            }
        }
        Some("content_block_start") => {
            let block = event.get("content_block").unwrap_or(&Value::Null);
            if json_str(block, &["type"]) == Some("tool_use")
                && let Some(name) = json_str(block, &["name"])
            {
                sink(AgentStreamEvent::Status(format!("Claude started {name}")));
            }
        }
        Some("content_block_stop") => {}
        _ => {}
    }
}

fn assistant_message_text(value: &Value) -> String {
    value
        .pointer("/message/content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|block| {
            (json_str(block, &["type"]) == Some("text"))
                .then(|| json_str(block, &["text"]))
                .flatten()
        })
        .collect::<Vec<_>>()
        .join("")
}

fn usage_from_value(value: &Value) -> Option<AgentTokenUsage> {
    let usage = value.get("usage")?;
    Some(AgentTokenUsage {
        input_tokens: json_u64(usage, &["input_tokens"]).unwrap_or_default(),
        cached_input_tokens: json_u64(usage, &["cache_read_input_tokens"]).unwrap_or_default(),
        output_tokens: json_u64(usage, &["output_tokens"]).unwrap_or_default(),
        reasoning_output_tokens: json_u64(usage, &["output_tokens_details", "thinking_tokens"])
            .unwrap_or_default(),
    })
}

fn record_usage(state: &mut ClaudeBackendState, agent_id: &AgentId, usage: AgentTokenUsage) {
    state
        .usage
        .entry(agent_id.clone())
        .and_modify(|current| *current = current.combine(usage))
        .or_insert(usage);
}

fn remove_parent_claude_environment(command: &mut Command) {
    for name in [
        "CLAUDE_CODE_SESSION_ID",
        "CLAUDE_CODE_ENTRYPOINT",
        "CLAUDE_CODE_SESSION_KIND",
        "CLAUDE_CODE_AGENT",
        "CLAUDE_CODE_AGENT_ID",
        "CLAUDE_CODE_AGENT_NAME",
        "CLAUDE_CODE_PARENT_SESSION_ID",
        "CLAUDE_CODE_TEAM_NAME",
        "CLAUDE_CODE_CHILD_SESSION",
        "CLAUDE_CODE_FORCE_SESSION_PERSISTENCE",
    ] {
        command.env_remove(name);
    }
}

fn read_to_string(mut reader: impl Read) -> String {
    let mut text = String::new();
    let _ = reader.read_to_string(&mut text);
    text
}

fn backend_error(message: String) -> AgentError {
    AgentError::Io(io::Error::other(message))
}

fn json_str<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn json_u64(value: &Value, path: &[&str]) -> Option<u64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_u64()
}
