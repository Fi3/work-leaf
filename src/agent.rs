use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AgentId(String);

impl AgentId {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AgentError::InvalidAgentId(
                "agent id cannot be empty".to_string(),
            ));
        }
        if trimmed
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace())
        {
            return Err(AgentError::InvalidAgentId(format!(
                "agent id `{trimmed}` cannot contain whitespace or control characters"
            )));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentKind {
    Codex,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageRole {
    User,
    Agent,
    Orchestrator,
    System,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub text: String,
}

impl ChatMessage {
    pub fn new(role: MessageRole, text: impl Into<String>) -> Self {
        Self {
            role,
            text: text.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentLaunch {
    pub id: AgentId,
    pub kind: AgentKind,
    pub feature: String,
    pub prompt: String,
}

impl AgentLaunch {
    pub fn new(
        id: AgentId,
        kind: AgentKind,
        feature: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            id,
            kind,
            feature: feature.into(),
            prompt: prompt.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentState {
    Running,
    Ready,
    Reviewing,
    Linearizing,
    Done,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentSession {
    pub id: AgentId,
    pub kind: AgentKind,
    pub feature: String,
    pub state: AgentState,
    pub messages: Vec<ChatMessage>,
    pub modified_files: BTreeSet<PathBuf>,
    pub depends_on: BTreeSet<AgentId>,
    pub depended_on_by: BTreeSet<AgentId>,
}

impl AgentSession {
    pub fn new(launch: AgentLaunch) -> Self {
        Self {
            id: launch.id,
            kind: launch.kind,
            feature: launch.feature,
            state: AgentState::Running,
            messages: vec![ChatMessage::new(MessageRole::User, launch.prompt)],
            modified_files: BTreeSet::new(),
            depends_on: BTreeSet::new(),
            depended_on_by: BTreeSet::new(),
        }
    }

    pub fn push_message(&mut self, role: MessageRole, text: impl Into<String>) {
        self.messages.push(ChatMessage::new(role, text));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPolicy {
    preamble: String,
}

impl PromptPolicy {
    pub fn for_restricted_agents() -> Self {
        Self {
            preamble: [
                "You are running under the work-leaf orchestrator.",
                "You are not allowed to read files directly; ask the orchestrator to provide file text.",
                "You are not allowed to write files directly; provide a unified diff patch for every file you want to change.",
                "Commands that create, update, delete, format, build, test, or otherwise write files require orchestrator mediation.",
                "Keep every patch focused on the current feature and explain the specific reason for the patch.",
                "Use `@work-leaf read <path>` to request file text from the orchestrator.",
                "Use `@work-leaf patch <reason>` followed by a unified diff and `@work-leaf end` to request a write.",
                "Use `@work-leaf locks classify <command>` to ask whether a command writes project files.",
                "Use `@work-leaf send <agent-id> <message>` to route context to another agent.",
            ]
            .join("\n"),
        }
    }

    pub fn inject(&self, agent_id: &AgentId, feature: &str, prompt: &str) -> String {
        format!(
            "{preamble}\n\nAgent-ID: {agent_id}\nFeature: {feature}\n\nUser prompt:\n{prompt}",
            preamble = self.preamble
        )
    }
}

impl Default for PromptPolicy {
    fn default() -> Self {
        Self::for_restricted_agents()
    }
}

#[derive(Debug)]
pub enum AgentError {
    InvalidAgentId(String),
    Io(std::io::Error),
    ProcessFailed {
        program: PathBuf,
        status: Option<i32>,
        stderr: String,
    },
    UnknownSession(AgentId),
}

impl fmt::Display for AgentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAgentId(message) => formatter.write_str(message),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::ProcessFailed {
                program,
                status,
                stderr,
            } => write!(
                formatter,
                "{} failed with status {:?}: {}",
                program.display(),
                status,
                stderr.trim()
            ),
            Self::UnknownSession(id) => write!(formatter, "unknown agent session `{id}`"),
        }
    }
}

impl std::error::Error for AgentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AgentError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
