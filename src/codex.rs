use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

use crate::agent::{
    AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, ChatMessage, MessageRole,
    PromptPolicy,
};

pub trait AgentBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError>;
    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError>;

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        _sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        self.launch(request)
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        _sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        self.send(agent_id, prompt)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentStreamEvent {
    Status(String),
    AgentMessage(String),
    Error(String),
}

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
            sandbox: SandboxMode::WorkspaceWrite,
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
    sessions: BTreeMap<AgentId, AgentSession>,
    thread_ids: BTreeMap<AgentId, String>,
}

impl CodexBackend {
    pub fn new(config: CodexCommandConfig, policy: PromptPolicy) -> Self {
        Self {
            config,
            policy,
            sessions: BTreeMap::new(),
            thread_ids: BTreeMap::new(),
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
        let feature = self
            .sessions
            .get(agent_id)
            .map(|session| session.feature.as_str())
            .unwrap_or("unknown");
        let stdin = self.policy.inject(agent_id, feature, prompt);
        let resume_id = self
            .thread_ids
            .get(agent_id)
            .map(String::as_str)
            .unwrap_or_else(|| agent_id.as_str());
        self.resume_invocation(resume_id, stdin)
    }

    pub fn record_launch_reply(
        &mut self,
        request: AgentLaunch,
        reply: String,
    ) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, reply);
        self.sessions.insert(session.id.clone(), session.clone());
        Ok(session)
    }

    pub fn record_launch_output(
        &mut self,
        request: AgentLaunch,
        output: String,
    ) -> Result<AgentSession, AgentError> {
        let parsed = parse_codex_output(&output);
        if let Some(thread_id) = parsed.thread_id {
            self.thread_ids.insert(request.id.clone(), thread_id);
        }
        self.record_launch_reply(request, parsed.agent_reply.unwrap_or(output))
    }

    pub fn session(&self, agent_id: &AgentId) -> Option<&AgentSession> {
        self.sessions.get(agent_id)
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

    fn run_invocation(&self, invocation: &CodexInvocation) -> Result<String, AgentError> {
        self.run_invocation_streaming(invocation, &mut |_| {})
    }

    fn run_invocation_streaming(
        &self,
        invocation: &CodexInvocation,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<String, AgentError> {
        let mut child = Command::new(&invocation.program)
            .args(&invocation.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

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

impl AgentBackend for CodexBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let invocation = self.build_launch_invocation(&request);
        let output = self.run_invocation(&invocation)?;
        self.record_launch_output(request, output)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let invocation = self.build_send_invocation(agent_id, prompt)?;
        let output = self.run_invocation(&invocation)?;
        let reply = parse_codex_output(&output).agent_reply.unwrap_or(output);
        let message = ChatMessage::new(MessageRole::Agent, reply);
        if let Some(session) = self.sessions.get_mut(agent_id) {
            session.push_message(MessageRole::User, prompt);
        } else {
            self.sessions.insert(
                agent_id.clone(),
                AgentSession::new(AgentLaunch::new(
                    agent_id.clone(),
                    AgentKind::Codex,
                    "unknown",
                    prompt,
                )),
            );
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .ok_or_else(|| AgentError::UnknownSession(agent_id.clone()))?;
        session.messages.push(message.clone());
        Ok(message)
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        let invocation = self.build_launch_invocation(&request);
        let output = self.run_invocation_streaming(&invocation, sink)?;
        self.record_launch_output(request, output)
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        let invocation = self.build_send_invocation(agent_id, prompt)?;
        let output = self.run_invocation_streaming(&invocation, sink)?;
        let reply = parse_codex_output(&output).agent_reply.unwrap_or(output);
        let message = ChatMessage::new(MessageRole::Agent, reply);
        if let Some(session) = self.sessions.get_mut(agent_id) {
            session.push_message(MessageRole::User, prompt);
        } else {
            self.sessions.insert(
                agent_id.clone(),
                AgentSession::new(AgentLaunch::new(
                    agent_id.clone(),
                    AgentKind::Codex,
                    "unknown",
                    prompt,
                )),
            );
        }
        let session = self
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
        if line.contains(r#""type":"thread.started""#) {
            parsed.thread_id = json_string_field(line, "thread_id").or(parsed.thread_id);
        }
        if line.contains(r#""type":"item.completed""#) && line.contains(r#""type":"agent_message""#)
        {
            parsed.agent_reply = json_string_field(line, "text").or(parsed.agent_reply);
        }
    }
    parsed
}

fn codex_stream_event(line: &str) -> Option<AgentStreamEvent> {
    if line.contains(r#""type":"item.completed""#) && line.contains(r#""type":"agent_message""#) {
        return json_string_field(line, "text").map(AgentStreamEvent::AgentMessage);
    }
    if line.contains(r#""type":"error""#) {
        return json_string_field(line, "message").map(AgentStreamEvent::Error);
    }
    if line.contains(r#""type":"thread.started""#) {
        return json_string_field(line, "thread_id")
            .map(|thread_id| AgentStreamEvent::Status(format!("Codex session {thread_id}")));
    }
    if line.contains(r#""type":"turn.started""#) {
        return Some(AgentStreamEvent::Status("Codex is working".to_string()));
    }
    None
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
