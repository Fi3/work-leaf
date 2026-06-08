use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::agent::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, ChatMessage, MessageRole, PromptPolicy,
};
use crate::agent_runtime::configure_agent_child_process;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl SandboxMode {
    fn as_codex_arg(&self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::DangerFullAccess => "danger-full-access",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodexCommandConfig {
    pub binary: PathBuf,
    pub project_dir: PathBuf,
    pub model: Option<String>,
    pub sandbox: SandboxMode,
}

impl CodexCommandConfig {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            binary: PathBuf::from("codex"),
            project_dir,
            model: None,
            sandbox: SandboxMode::ReadOnly,
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

    pub fn with_sandbox(mut self, sandbox: SandboxMode) -> Self {
        self.sandbox = sandbox;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodexInvocation {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub stdin: String,
}

#[derive(Debug)]
pub struct CodexBackend {
    config: CodexCommandConfig,
    policy: PromptPolicy,
    state: Arc<Mutex<CodexBackendState>>,
    shutdown: AgentShutdownHandle,
    lifecycle: Arc<()>,
}

impl Clone for CodexBackend {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            policy: self.policy.clone(),
            state: self.state.clone(),
            shutdown: self.shutdown.clone(),
            lifecycle: self.lifecycle.clone(),
        }
    }
}

#[derive(Debug, Default)]
struct CodexBackendState {
    sessions: BTreeMap<AgentId, AgentSession>,
    thread_ids: BTreeMap<AgentId, String>,
    active_processes: BTreeMap<AgentId, u32>,
}

impl CodexBackend {
    pub fn new(config: CodexCommandConfig, policy: PromptPolicy) -> Self {
        Self {
            config,
            policy,
            state: Arc::new(Mutex::new(CodexBackendState::default())),
            shutdown: AgentShutdownHandle::default(),
            lifecycle: Arc::new(()),
        }
    }

    pub fn build_launch_invocation(&self, request: &AgentLaunch) -> CodexInvocation {
        let stdin = self
            .policy
            .inject(&request.id, &request.feature, &request.prompt);
        self.exec_invocation(stdin)
    }

    pub fn build_send_invocation(
        &self,
        agent_id: &AgentId,
        prompt: &str,
    ) -> Result<CodexInvocation, AgentError> {
        let (feature, resume_id) = {
            let state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            let feature = state
                .sessions
                .get(agent_id)
                .map(|session| session.feature.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let resume_id = state
                .thread_ids
                .get(agent_id)
                .cloned()
                .unwrap_or_else(|| agent_id.as_str().to_string());
            (feature, resume_id)
        };
        let stdin = self.policy.inject(agent_id, &feature, prompt);
        self.resume_invocation(&resume_id, stdin)
    }

    pub fn record_launch_reply(
        &mut self,
        request: AgentLaunch,
        reply: String,
    ) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, reply);
        self.state
            .lock()
            .expect("codex backend state mutex poisoned")
            .sessions
            .insert(session.id.clone(), session.clone());
        Ok(session)
    }

    pub fn record_launch_output(
        &mut self,
        request: AgentLaunch,
        output: String,
    ) -> Result<AgentSession, AgentError> {
        let parsed = parse_codex_output(&output);
        if let Some(thread_id) = parsed.thread_id {
            self.state
                .lock()
                .expect("codex backend state mutex poisoned")
                .thread_ids
                .insert(request.id.clone(), thread_id);
        }
        self.record_launch_reply(request, parsed.agent_reply.unwrap_or(output))
    }

    pub fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        self.state
            .lock()
            .expect("codex backend state mutex poisoned")
            .sessions
            .get(agent_id)
            .cloned()
    }

    fn exec_invocation(&self, stdin: String) -> CodexInvocation {
        let mut args = vec![
            "--cd".to_string(),
            self.config.project_dir.display().to_string(),
            "--sandbox".to_string(),
            self.config.sandbox.as_codex_arg().to_string(),
            "--ask-for-approval".to_string(),
            "never".to_string(),
        ];
        if let Some(model) = &self.config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        args.push("exec".to_string());
        args.push("--color".to_string());
        args.push("never".to_string());
        args.push("--json".to_string());
        args.push("-".to_string());
        CodexInvocation {
            program: self.config.binary.clone(),
            args,
            stdin,
        }
    }

    fn resume_invocation(
        &self,
        resume_id: &str,
        stdin: String,
    ) -> Result<CodexInvocation, AgentError> {
        let mut args = vec![
            "--cd".to_string(),
            self.config.project_dir.display().to_string(),
            "--sandbox".to_string(),
            self.config.sandbox.as_codex_arg().to_string(),
            "--ask-for-approval".to_string(),
            "never".to_string(),
        ];
        if let Some(model) = &self.config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        args.extend([
            "exec".to_string(),
            "resume".to_string(),
            "--json".to_string(),
            resume_id.to_string(),
            "-".to_string(),
        ]);
        Ok(CodexInvocation {
            program: self.config.binary.clone(),
            args,
            stdin,
        })
    }

    fn run_invocation_streaming(
        &self,
        invocation: &CodexInvocation,
        agent_id: Option<&AgentId>,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<String, AgentError> {
        let mut command = Command::new(&invocation.program);
        command
            .args(&invocation.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        configure_agent_child_process(&mut command);
        let mut child = command.spawn()?;
        let child_pid = child.id();
        let _process_guard = self.shutdown.register(child.id());
        if let Some(agent_id) = agent_id {
            self.state
                .lock()
                .expect("codex backend state mutex poisoned")
                .active_processes
                .insert(agent_id.clone(), child_pid);
        }

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(invocation.stdin.as_bytes())?;
        }
        drop(child.stdin.take());

        let stderr_reader = child.stderr.take().map(|mut stderr| {
            thread::spawn(move || {
                let mut text = String::new();
                let _ = stderr.read_to_string(&mut text);
                text
            })
        });

        let mut stdout_text = String::new();
        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines() {
                let line = line?;
                if let Some(event) = codex_stream_event(&line) {
                    sink(event);
                }
                stdout_text.push_str(&line);
                stdout_text.push('\n');
            }
        }

        let status = child.wait()?;
        if let Some(agent_id) = agent_id {
            let mut state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            if state
                .active_processes
                .get(agent_id)
                .is_some_and(|pid| *pid == child_pid)
            {
                state.active_processes.remove(agent_id);
            }
        }
        let stderr = stderr_reader
            .and_then(|reader| reader.join().ok())
            .unwrap_or_default();

        if status.success() {
            Ok(stdout_text.trim().to_string())
        } else {
            Err(AgentError::ProcessFailed {
                program: invocation.program.clone(),
                status: status.code(),
                stderr,
            })
        }
    }
}

impl Drop for CodexBackend {
    fn drop(&mut self) {
        if Arc::strong_count(&self.lifecycle) == 1 {
            self.shutdown.shutdown();
        }
    }
}

impl AgentBackend for CodexBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let invocation = self.build_launch_invocation(&request);
        let output = self.run_invocation_streaming(&invocation, Some(&request.id), &mut |_| {})?;
        self.record_launch_output(request, output)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let invocation = self.build_send_invocation(agent_id, prompt)?;
        let output = self.run_invocation_streaming(&invocation, Some(agent_id), &mut |_| {})?;
        let reply = parse_codex_output(&output).agent_reply.unwrap_or(output);
        let message = ChatMessage::new(MessageRole::Agent, reply);
        let mut state = self
            .state
            .lock()
            .expect("codex backend state mutex poisoned");
        if let Some(session) = state.sessions.get_mut(agent_id) {
            session.push_message(MessageRole::User, prompt);
        } else {
            state.sessions.insert(
                agent_id.clone(),
                AgentSession::new(AgentLaunch::new(
                    agent_id.clone(),
                    AgentKind::Codex,
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

    fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.shutdown.clone()
    }

    fn interrupt(&mut self, agent_id: &AgentId) -> Result<(), AgentError> {
        let pid = self
            .state
            .lock()
            .expect("codex backend state mutex poisoned")
            .active_processes
            .get(agent_id)
            .copied();
        if let Some(pid) = pid {
            let _ = self.shutdown.terminate_process(pid);
        }
        Ok(())
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        let invocation = self.build_launch_invocation(&request);
        let output = self.run_invocation_streaming(&invocation, Some(&request.id), sink)?;
        self.record_launch_output(request, output)
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        let invocation = self.build_send_invocation(agent_id, prompt)?;
        let output = self.run_invocation_streaming(&invocation, Some(agent_id), sink)?;
        let reply = parse_codex_output(&output).agent_reply.unwrap_or(output);
        let message = ChatMessage::new(MessageRole::Agent, reply);
        let mut state = self
            .state
            .lock()
            .expect("codex backend state mutex poisoned");
        if let Some(session) = state.sessions.get_mut(agent_id) {
            session.push_message(MessageRole::User, prompt);
        } else {
            state.sessions.insert(
                agent_id.clone(),
                AgentSession::new(AgentLaunch::new(
                    agent_id.clone(),
                    AgentKind::Codex,
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
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ParsedCodexOutput {
    thread_id: Option<String>,
    agent_reply: Option<String>,
}

fn parse_codex_output(output: &str) -> ParsedCodexOutput {
    let mut parsed = ParsedCodexOutput::default();
    for line in output.lines() {
        let compact = compact_json_line(line);
        if compact.contains(r#""type":"thread.started""#) {
            parsed.thread_id = json_string_field(&compact, "thread_id").or(parsed.thread_id);
        }
        if compact.contains(r#""type":"item.completed""#)
            && compact.contains(r#""type":"agent_message""#)
            && let Some(text) = json_string_field(&compact, "text")
        {
            append_agent_reply(&mut parsed.agent_reply, text);
        }
    }
    parsed
}

fn append_agent_reply(reply: &mut Option<String>, text: String) {
    match reply {
        Some(existing) if !existing.is_empty() && !text.is_empty() => {
            existing.push_str("\n\n");
            existing.push_str(&text);
        }
        Some(existing) => existing.push_str(&text),
        None => *reply = Some(text),
    }
}

fn codex_stream_event(line: &str) -> Option<AgentStreamEvent> {
    let compact = compact_json_line(line);
    if compact.contains(r#""type":"item.completed""#)
        && compact.contains(r#""type":"agent_message""#)
    {
        return json_string_field(&compact, "text").map(AgentStreamEvent::AgentMessage);
    }
    if compact.contains(r#""type":"error""#) {
        return json_string_field(&compact, "message").map(AgentStreamEvent::Error);
    }
    if compact.contains(r#""type":"thread.started""#) {
        return json_string_field(&compact, "thread_id")
            .map(|thread_id| AgentStreamEvent::Status(format!("Codex session {thread_id}")));
    }
    if compact.contains(r#""type":"turn.started""#) {
        return Some(AgentStreamEvent::Status("Codex is working".to_string()));
    }
    None
}

fn compact_json_line(line: &str) -> String {
    let mut compact = String::with_capacity(line.len());
    let mut in_string = false;
    let mut escaped = false;

    for ch in line.chars() {
        if in_string {
            compact.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            compact.push(ch);
        } else if !ch.is_whitespace() {
            compact.push(ch);
        }
    }

    compact
}

fn json_string_field(line: &str, field: &str) -> Option<String> {
    let needle = format!(r#""{field}":"#);
    let start = line.find(&needle)? + needle.len();
    let mut chars = line[start..].chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            match ch {
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                '/' => value.push('/'),
                'b' => value.push('\u{0008}'),
                'f' => value.push('\u{000c}'),
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                other => value.push(other),
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(value);
        } else {
            value.push(ch);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_parsing_accepts_json_field_whitespace() {
        let output = [
            r#"{ "type": "thread.started", "thread_id": "thread-spaced" }"#,
            r#"{ "type": "item.completed", "item": { "type": "agent_message", "text": "first reply" } }"#,
            r#"{ "type": "item.completed", "item": { "type": "agent_message", "text": "second reply" } }"#,
        ]
        .join("\n");

        let parsed = parse_codex_output(&output);

        assert_eq!(parsed.thread_id.as_deref(), Some("thread-spaced"));
        assert_eq!(
            parsed.agent_reply.as_deref(),
            Some("first reply\n\nsecond reply")
        );
        assert_eq!(
            codex_stream_event(
                r#"{ "type": "item.completed", "item": { "type": "agent_message", "text": "streamed reply" } }"#
            ),
            Some(AgentStreamEvent::AgentMessage("streamed reply".to_string()))
        );
        assert_eq!(
            codex_stream_event(r#"{ "type": "error", "message": "streamed error" }"#),
            Some(AgentStreamEvent::Error("streamed error".to_string()))
        );
        assert_eq!(
            codex_stream_event(r#"{ "type": "turn.started" }"#),
            Some(AgentStreamEvent::Status("Codex is working".to_string()))
        );
    }
}
