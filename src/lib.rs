pub mod agent;
pub mod codex;

pub use agent::{
    AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, ChatMessage, MessageRole,
    PromptPolicy,
};
pub use codex::{AgentBackend, CodexBackend, CodexCommandConfig, CodexInvocation, SandboxMode};

pub fn greeting() -> &'static str {
    "work-leaf"
}
