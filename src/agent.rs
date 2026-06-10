use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

pub use crate::agent_runtime::{
    AgentBackend, AgentShutdownHandle, AgentStreamEvent, AgentTokenUsage,
};
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

impl Serialize for AgentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AgentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MessageRole {
    User,
    Agent,
    Orchestrator,
    System,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AgentState {
    Running,
    Ready,
    Reviewing,
    Linearizing,
    Done,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
                "You may read temporary context bundle files only when the orchestrator gives you their exact paths in a Work Leaf response; those bundles are orchestrator-provided file text, not project files.",
                "You are not allowed to write files directly; submit a structured edit patch for every file you want to change.",
                "Commands that create, update, delete, format, build, test, or otherwise write files require orchestrator mediation.",
                "Keep every patch focused on the current feature and explain the specific reason for the patch.",
                "Do not modify documentation or plain-text files in patch-agent work. Do not touch `docs/**`, `README*`, `CHANGELOG*`, `*.md`, `*.txt`, or other prose-only files; leave those updates for the linearize agent after review.",
                "`@work-leaf` is an orchestrator response protocol, not an executable command. Do not run `@work-leaf` in a shell or ask the user to run it.",
                "Emit every `@work-leaf ...` request as a top-level plain response line, without quotes, prose, or code fences, so the orchestrator can parse it.",
                "Never claim that you are switching to local workspace tools; keep using orchestrator directives until the orchestrator responds.",
                "Use `@work-leaf read <path>` to request file text from the orchestrator.",
                "If you request a file you already received, Work Leaf compares digests and returns either unchanged status or a diff from your last snapshot; do not use repeated reads to reload whole files.",
                "Use `@work-leaf read --force <path>` only when you need a fresh full file snapshot after the unchanged or diff response is insufficient.",
                "Request related files together, because `@work-leaf read <path> <path...>` returns multiple file snapshots in one orchestrator response.",
            ]),
            ReadPermission::DirectFilesystem => lines.extend([
                "You may read repository files directly from the filesystem.",
                "Use direct filesystem reads and read-only inspection commands for repository context instead of `@work-leaf read`.",
                "You are not allowed to write files directly; submit a structured edit patch for every file you want to change.",
                "Commands that create, update, delete, format, build, test, or otherwise write files require orchestrator mediation.",
                "Keep every patch focused on the current feature and explain the specific reason for the patch.",
                "Do not modify documentation or plain-text files in patch-agent work. Do not touch `docs/**`, `README*`, `CHANGELOG*`, `*.md`, `*.txt`, or other prose-only files; leave those updates for the linearize agent after review.",
                "`@work-leaf` is an orchestrator response protocol, not an executable command. Do not run `@work-leaf` in a shell or ask the user to run it.",
                "Emit every `@work-leaf ...` request as a top-level plain response line, without quotes, prose, or code fences, so the orchestrator can parse it.",
            ]),
        }
        lines.extend([
            "Use `@work-leaf edit <reason>` followed by an apply-patch-style exact edit body and `@work-leaf end` to request a write.",
            "Structured edit bodies use `*** Begin Patch`, `*** Update File: path`, `@@` separators without line numbers, exact unchanged context lines prefixed with a space, old lines prefixed with `-`, new lines prefixed with `+`, and `*** End Patch`.",
            "Do not invent unified-diff line numbers for manual edits. Include enough unchanged context around each old block so it matches exactly one place in the current file.",
            "The legacy `@work-leaf patch <reason>` unified-diff directive is still accepted only when you already have a complete valid unified diff with real hunk ranges; prefer `@work-leaf edit` for manual code, configuration, and test changes.",
            "Use `@work-leaf locks classify <command>` only when you are unsure whether a command writes project files.",
            "Use `@work-leaf locks run <path> <path...> -- <command>` to run a command while the orchestrator holds write locks for every path the command may write.",
            "This command-lock rule is language- and tool-agnostic: use it for any formatter, build, test, code generator, package manager, installer, cache-producing tool, or repository-required check that may write files.",
            "Choose the command from the repository instructions and project context; choose the lock paths from the files, directories, caches, build outputs, dependency folders, or lockfiles that command may write.",
            "Run checks that existed before your patch or checks you added yourself. Do not run another patch agent's focused tests as local validation; report those as integration conflicts unless your own source change clearly caused them.",
            "Keep the shared worktree usable for the other patch agents: do not submit known-red, compile-breaking, or deliberately failing intermediate patches. Design tests before implementation when required, but submit a cohesive patch that includes the test and the implementation needed for the shared tree to build.",
            "Locked command runs are limited to five minutes; user authorization is required for longer lock-holding commands.",
            "Do not use command locks for manual feature edits; manual code, configuration, and test changes must still be submitted with the structured edit directive.",
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
        let linearize_agent = is_linearize_agent(agent_id);
        let mut text = if linearize_agent {
            linearize_preamble()
        } else {
            self.preamble.clone()
        };
        if !self.project_instructions.is_empty() {
            text.push_str("\n\nRepository instructions from the launch project:");
            if !linearize_agent {
                text.push_str("\n\n");
                text.push_str(&concurrent_work_leaf_interpretation(
                    &self.project_instructions,
                ));
            }
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

fn concurrent_work_leaf_interpretation(instruction_files: &[ProjectInstructionFile]) -> String {
    let mut text = "Concurrent Work Leaf interpretation:\n\
- preserve the repository-specific intent of the instructions below; use the repo's architecture, APIs, naming, style, and checks.\n\
- Apply broad repository check requirements in a shared-worktree way. Prefer focused checks for files you touched, checks that existed before your patch, and checks you added yourself.\n\
- Avoid write-producing broad formatters over the whole repository while other patch agents are active. Prefer check-only formatter commands or formatter commands scoped to files you touched.\n\
- If a broad required check is blocked only by another patch agent's owned files or focused tests, do not take over that agent's work. Report the blocker once with the concrete failing file/test and stop retrying the same broad check.\n\
- Do not repeatedly rerun the same broad check after it fails for the same external integration blocker. After your focused checks pass and any external blocker is reported, use `@work-leaf done` and leave cross-agent reconciliation to review or linearize.\n\
- Treat compact file refreshes and repeated-read digests as authoritative. Use `@work-leaf read --force` only when the diff or digest response is insufficient for a specific patch."
        .to_string();

    for instructions in instruction_files {
        text.push_str("\n\n");
        text.push_str(&concurrent_instruction_translation(instructions));
    }

    text
}

fn concurrent_instruction_translation(instructions: &ProjectInstructionFile) -> String {
    let topics = InstructionTopics::detect(&instructions.text);
    let mut lines = vec![
        format!(
            "Concurrent Work Leaf translation for {}:",
            instructions.path.display()
        ),
        "- Treat the instruction file below as authoritative for repository-specific architecture, APIs, style, naming, safety rules, and quality bars.".to_string(),
        "- Translate only ownership, timing, and tool-access details that assume one agent owns the whole workspace.".to_string(),
        "- Patch agents own the current feature patch; review and linearize agents own cross-agent reconciliation and final history.".to_string(),
    ];

    if topics.checks {
        lines.extend([
            "- Required checks remain mandatory. As a patch agent, run focused checks for files you touched and checks you added or changed; leave broad cross-agent failures to review or linearization after reporting the exact blocker.".to_string(),
            "- If an instruction requires a repository-wide formatter, prefer check-only mode or a file-scoped formatter for your touched files while other patch agents are active.".to_string(),
        ]);
    }
    if topics.tests {
        lines.push(
            "- Test requirements remain mandatory. Design the needed tests, but submit tests with the implementation needed to keep the shared worktree buildable.".to_string(),
        );
    }
    if topics.docs {
        lines.push(
            "- Documentation rules remain mandatory. Patch agents do not edit docs or prose-only files; required docs updates are handled by the linearize agent after reviewed behavior is accepted.".to_string(),
        );
    }
    if topics.commits {
        lines.push(
            "- Commit-message rules remain mandatory. Patch agents express intent through the `@work-leaf edit <reason>` reason; final commit-message compliance is enforced through patch reason and final linearized commits.".to_string(),
        );
    }
    if topics.reviews {
        lines.push(
            "- Review rules remain mandatory. Reviewers inspect only the reviewed patch scope; integration cleanup and broad history shaping are linearization responsibilities.".to_string(),
        );
    }
    if topics.real_agent_verification {
        lines.push(
            "- Real-agent verification rules remain mandatory. For agent-facing behavior, run or report a bounded real-agent scenario and exact result before `@work-leaf done`; if local setup blocks it, report the exact pre-agent blocker.".to_string(),
        );
    }

    lines.join("\n")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InstructionTopics {
    checks: bool,
    tests: bool,
    docs: bool,
    commits: bool,
    reviews: bool,
    real_agent_verification: bool,
}

impl InstructionTopics {
    fn detect(text: &str) -> Self {
        let text = text.to_ascii_lowercase();
        Self {
            checks: mentions_any(
                &text,
                &[
                    "required check",
                    "required checks",
                    "cargo check",
                    "cargo fmt",
                    "cargo clippy",
                    "cargo test",
                    "format",
                    "formatter",
                    "lint",
                    "build",
                ],
            ),
            tests: mentions_any(
                &text,
                &[
                    "test",
                    "tests",
                    "coverage",
                    "regression",
                    "failing test",
                    "test-first",
                ],
            ),
            docs: mentions_any(
                &text,
                &[
                    "documentation",
                    "docs",
                    "readme",
                    "changelog",
                    "markdown",
                    ".md",
                    ".txt",
                    "plain text",
                    "prose",
                ],
            ),
            commits: mentions_any(
                &text,
                &[
                    "commit message",
                    "commit-message",
                    "commit messages",
                    "git commit",
                    "commit must",
                    "commits must",
                ],
            ),
            reviews: mentions_any(&text, &["review", "reviews", "reviewer", "findings"]),
            real_agent_verification: mentions_any(
                &text,
                &[
                    "real agent",
                    "real-agent",
                    "agent-facing",
                    "smoke check",
                    "smoke test",
                    "verification",
                ],
            ),
        }
    }
}

fn mentions_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn is_linearize_agent(agent_id: &AgentId) -> bool {
    let value = agent_id.as_str();
    value == "linearize" || value.starts_with("linearize-")
}

fn linearize_preamble() -> String {
    [
        "You are running as the work-leaf linearize agent.",
        "You are allowed to read repository files directly.",
        "You are allowed to write repository files, run commands, and rewrite git history directly inside the workspace without using `@work-leaf read`, `@work-leaf edit`, `@work-leaf patch`, or `@work-leaf locks run`.",
        "Use direct workspace tools for code, documentation, plain-text files, checks, and git operations.",
        "Documentation and plain-text updates deferred by patch agents are part of your responsibility when they are required by the final reviewed behavior.",
        "Keep the final history minimal, preserve reviewed behavior, follow repository commit-message and verification instructions, and report the final commits and checks.",
    ]
    .join("\n")
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
