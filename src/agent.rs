use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

pub use crate::agent_runtime::{AgentBackend, AgentShutdownHandle, AgentStreamEvent};
use crate::instructions::{ProjectInstructionFile, load_project_instructions};

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
    External(String),
}

impl AgentKind {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Codex => "Codex",
            Self::External(name) => name,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentProfile {
    pub kind: AgentKind,
    pub display_name: String,
    pub default_feature: String,
}

impl AgentProfile {
    pub fn new(
        kind: AgentKind,
        display_name: impl Into<String>,
        default_feature: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            display_name: display_name.into(),
            default_feature: default_feature.into(),
        }
    }

    pub fn codex() -> Self {
        Self {
            kind: AgentKind::Codex,
            display_name: "Codex".to_string(),
            default_feature: "user-agent".to_string(),
        }
    }
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
    project_instructions: Vec<ProjectInstructionFile>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadPermission {
    Orchestrator,
    DirectFilesystem,
}

impl PromptPolicy {
    pub fn for_restricted_agents() -> Self {
        Self::for_read_permission(ReadPermission::Orchestrator)
    }

    pub fn for_direct_read_agents() -> Self {
        Self::for_read_permission(ReadPermission::DirectFilesystem)
    }

    pub fn for_read_permission(read_permission: ReadPermission) -> Self {
        let mut lines = vec!["You are running under the work-leaf orchestrator."];
        match read_permission {
            ReadPermission::Orchestrator => lines.extend([
                "You are not allowed to read files directly; ask the orchestrator to provide file text.",
                "You are not allowed to write files directly; provide a unified diff patch for every file you want to change.",
                "Commands that create, update, delete, format, build, test, or otherwise write files require orchestrator mediation.",
                "Keep every patch focused on the current feature and explain the specific reason for the patch.",
                "`@work-leaf` is an orchestrator response protocol, not an executable command. Do not run `@work-leaf` in a shell or ask the user to run it.",
                "Emit every `@work-leaf ...` request as a top-level plain response line, without quotes, prose, or code fences, so the orchestrator can parse it.",
                "Never claim that you are switching to local workspace tools; keep using orchestrator directives until the orchestrator responds.",
                "Use `@work-leaf read <path>` to request file text from the orchestrator.",
                "Request related files together, because `@work-leaf read <path> <path...>` returns multiple file snapshots in one orchestrator response.",
            ]),
            ReadPermission::DirectFilesystem => lines.extend([
                "You may read repository files directly from the filesystem.",
                "Use direct filesystem reads and read-only inspection commands for repository context instead of `@work-leaf read`.",
                "You are not allowed to write files directly; provide a unified diff patch for every file you want to change.",
                "Commands that create, update, delete, format, build, test, or otherwise write files require orchestrator mediation.",
                "Keep every patch focused on the current feature and explain the specific reason for the patch.",
                "`@work-leaf` is an orchestrator response protocol, not an executable command. Do not run `@work-leaf` in a shell or ask the user to run it.",
                "Emit every `@work-leaf ...` request as a top-level plain response line, without quotes, prose, or code fences, so the orchestrator can parse it.",
            ]),
        }
        lines.extend([
            "Use `@work-leaf patch <reason>` followed by a unified diff and `@work-leaf end` to request a write.",
            "Use `@work-leaf locks classify <command>` to ask whether a command writes project files.",
            "Use `@work-leaf locks run <path> <path...> -- <command>` to run a command while the orchestrator holds write locks for every path the command may write.",
            "This command-lock rule is language- and tool-agnostic: use it for any formatter, build, test, code generator, package manager, installer, cache-producing tool, or repository-required check that may write files.",
            "Choose the command from the repository instructions and project context; choose the lock paths from the files, directories, caches, build outputs, dependency folders, or lockfiles that command may write.",
            "Do not use command locks for manual feature edits; manual code or documentation changes must still be submitted with the unified-diff patch directive.",
            "Use `@work-leaf send <agent-id> <message>` to route context to another agent.",
            "You are responsible for following the project instructions, including running the repository's required checks before you submit a patch or report work done.",
            "Use `@work-leaf done` when no more orchestrator work is required.",
        ]);
        Self {
            preamble: lines.join("\n"),
            project_instructions: Vec::new(),
        }
    }

    pub fn for_project(root: impl AsRef<Path>) -> Result<Self, AgentError> {
        Self::for_project_with_read_permission(root, ReadPermission::Orchestrator)
    }

    pub fn for_project_with_read_permission(
        root: impl AsRef<Path>,
        read_permission: ReadPermission,
    ) -> Result<Self, AgentError> {
        let mut policy = Self::for_read_permission(read_permission);
        policy.project_instructions = load_project_instructions(root.as_ref())?;
        Ok(policy)
    }

    pub fn inject(&self, agent_id: &AgentId, feature: &str, prompt: &str) -> String {
        let mut text = self.preamble.clone();
        if !self.project_instructions.is_empty() {
            text.push_str("\n\nRepository instructions from the launch project:");
            for instructions in &self.project_instructions {
                text.push_str("\n\n--- ");
                text.push_str(&instructions.path.display().to_string());
                text.push_str(" ---\n");
                text.push_str(&instructions.text);
                if !instructions.text.ends_with('\n') {
                    text.push('\n');
                }
            }
        }
        text.push_str(&format!(
            "\n\nAgent-ID: {agent_id}\nFeature: {feature}\n\nUser prompt:\n{prompt}"
        ));
        text
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
