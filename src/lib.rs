pub mod agent;
pub mod agent_runtime;
mod chat_title;
pub mod claude;
pub mod cli;
pub mod codex;
pub mod http_controller;
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
    AgentShutdownHandle, AgentStreamEvent, AgentTokenUsage, ChatMessage, MessageRole, PromptPolicy,
    ReadPermission,
};
pub use claude::{ClaudeBackend, ClaudeCommandConfig};
pub use cli::{
    CliError, CommandChat, CommandChatResult, ProcessCommand, SelectedAgent, parse_process_args,
    render_command_chat_help, render_daemon_startup, render_process_help, run_cli_from_env,
};
pub use codex::{CodexBackend, CodexCommandConfig, SandboxMode};
pub use http_controller::{
    HttpControllerClient, HttpControllerServer, OrchestratorHttpError, WorkLeafControllerState,
    run_orchestrator_from_env,
};
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
pub use terminal_app::{RemoteTerminalApp, TerminalApp};
pub use ui::{
    AgentListEntry, PaneFocus, TerminalLayout, TerminalUi, UiAction, UiKey, UiMode, UiSurface,
};
pub use ui_harness::UiHarness;
pub use workspace::{
    WorkLeafCompletion, WorkLeafController, WorkLeafEvent, WorkLeafLoading, WorkLeafSession,
    WorkLeafSnapshot,
};
