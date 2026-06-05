use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::agent::{
    AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, MessageRole, PromptPolicy,
};

pub trait AgentBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError>;
    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError>;
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
}

impl CodexBackend {
    pub fn new(config: CodexCommandConfig, policy: PromptPolicy) -> Self {
        Self {
            config,
            policy,
            sessions: BTreeMap::new(),
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
        let session = self
            .sessions
            .get(agent_id)
            .ok_or_else(|| AgentError::UnknownSession(agent_id.clone()))?;
        let stdin = self.policy.inject(agent_id, &session.feature, prompt);
        self.resume_invocation(agent_id, stdin)
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

    pub fn session(&self, agent_id: &AgentId) -> Option<&AgentSession> {
        self.sessions.get(agent_id)
    }

    fn exec_invocation(&self, stdin: String) -> CodexInvocation {
        let mut args = vec![
            "exec".to_string(),
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
        args.push("--color".to_string());
        args.push("never".to_string());
        args.push("-".to_string());
        CodexInvocation {
            program: self.config.binary.clone(),
            args,
            stdin,
        }
    }

    fn resume_invocation(
        &self,
        agent_id: &AgentId,
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
            agent_id.as_str().to_string(),
            "-".to_string(),
        ]);
        Ok(CodexInvocation {
            program: self.config.binary.clone(),
            args,
            stdin,
        })
    }

    fn run_invocation(&self, invocation: &CodexInvocation) -> Result<String, AgentError> {
        let mut child = Command::new(&invocation.program)
            .args(&invocation.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(invocation.stdin.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(AgentError::ProcessFailed {
                program: invocation.program.clone(),
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }
}

impl AgentBackend for CodexBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let invocation = self.build_launch_invocation(&request);
        let reply = self.run_invocation(&invocation)?;
        self.record_launch_reply(request, reply)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let invocation = self.build_send_invocation(agent_id, prompt)?;
        let reply = self.run_invocation(&invocation)?;
        let message = ChatMessage::new(MessageRole::Agent, reply);
        let session = self
            .sessions
            .get_mut(agent_id)
            .ok_or_else(|| AgentError::UnknownSession(agent_id.clone()))?;
        session.push_message(MessageRole::User, prompt);
        session.messages.push(message.clone());
        Ok(message)
    }
}
