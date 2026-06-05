pub mod agent;
pub mod codex;
pub mod linearize;
pub mod locks;
pub mod patch;
pub mod review;
pub mod ui;

pub use agent::{
    AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, ChatMessage, MessageRole,
    PromptPolicy,
};
pub use codex::{AgentBackend, CodexBackend, CodexCommandConfig, CodexInvocation, SandboxMode};
pub use linearize::{
    LinearizeAction, LinearizeError, LinearizeGroup, LinearizeHandoff, LinearizePlan,
    LinearizePlanner, LinearizeQuestion,
};
pub use locks::{
    CommandWriteIntent, CommandWritePolicy, FileAccessError, FileLockTable, FileSnapshot,
};
pub use patch::{GitPatcher, PatchError, PatchOutcome, PatchRequest};
pub use review::{AgentCommit, GitHistory, ReviewCoordinator, ReviewError, ReviewResult};
pub use ui::{
    AgentListEntry, PaneFocus, TerminalLayout, TerminalUi, UiAction, UiKey, UiMode, UiSurface,
};

pub fn greeting() -> &'static str {
    "work-leaf"
}
