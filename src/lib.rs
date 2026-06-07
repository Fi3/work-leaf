pub mod agent;
pub mod agent_runtime;
mod chat_title;
pub mod cli;
pub mod codex;
mod instructions;
pub mod linearize;
pub mod locks;
pub mod orchestrator;
pub mod patch;
pub mod review;
pub mod terminal_app;
pub mod ui;
pub mod ui_harness;
pub mod workspace;

pub use agent::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentProfile, AgentSession,
    AgentShutdownHandle, AgentStreamEvent, ChatMessage, MessageRole, PromptPolicy, ReadPermission,
};
pub use cli::{
    CliError, CommandChat, CommandChatResult, ProcessCommand, parse_process_args,
    render_command_chat_help, render_process_help, run_cli_from_env,
};
pub use codex::{CodexBackend, CodexCommandConfig, CodexInvocation, SandboxMode};
pub use linearize::{
    LinearizeAction, LinearizeError, LinearizeGroup, LinearizeHandoff, LinearizePlan,
    LinearizePlanner, LinearizeQuestion,
};
pub use locks::{
    CommandWriteIntent, CommandWritePolicy, FileAccessError, FileLockTable, FileSnapshot,
};
pub use orchestrator::{AgentOrchestrator, OrchestratorError, OrchestratorEvent};
pub use patch::{GitPatcher, PatchCoordinator, PatchError, PatchOutcome, PatchRequest};
pub use review::{AgentCommit, GitHistory, ReviewCoordinator, ReviewError, ReviewResult};
pub use terminal_app::TerminalApp;
pub use ui::{
    AgentListEntry, PaneFocus, TerminalLayout, TerminalUi, UiAction, UiKey, UiMode, UiSurface,
};
pub use ui_harness::UiHarness;
pub use workspace::{
    WorkLeafController, WorkLeafEvent, WorkLeafLoading, WorkLeafSession, WorkLeafSnapshot,
};
