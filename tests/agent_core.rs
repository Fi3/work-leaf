use std::fs;
use std::path::PathBuf;

use work_leaf::{
    AgentId, AgentKind, AgentLaunch, CodexBackend, CodexCommandConfig, MessageRole, PromptPolicy,
    SandboxMode,
};

#[test]
fn prompt_policy_wraps_every_agent_prompt_with_file_access_rules() {
    let policy = PromptPolicy::for_restricted_agents();
    let wrapped = policy.inject(
        &AgentId::new("chat-1").unwrap(),
        "feature flags",
        "implement the flag parser",
    );

    assert!(wrapped.contains("Agent-ID: chat-1"));
    assert!(wrapped.contains("Feature: feature flags"));
    assert!(wrapped.contains("not allowed to read files directly"));
    assert!(wrapped.contains("ask the orchestrator to provide file text"));
    assert!(wrapped.contains("not allowed to write files directly"));
    assert!(wrapped.contains("provide a unified diff patch"));
    assert!(wrapped.contains("@work-leaf read <path>"));
    assert!(wrapped.contains("@work-leaf patch <reason>"));
    assert!(wrapped.contains("@work-leaf locks classify <command>"));
    assert!(wrapped.contains("@work-leaf send <agent-id> <message>"));
    assert!(wrapped.contains("implement the flag parser"));
}

#[test]
fn prompt_policy_injects_launch_project_agent_instructions() {
    let root = temp_dir("prompt-policy-project-instructions");
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\n1. `cargo check`\n\nProject-specific rule from fixture.\n",
    )
    .unwrap();
    let policy = PromptPolicy::for_project(&root).unwrap();

    let wrapped = policy.inject(
        &AgentId::new("chat-1").unwrap(),
        "feature flags",
        "implement the flag parser",
    );

    assert!(wrapped.contains("Repository instructions from the launch project"));
    assert!(wrapped.contains("--- AGENTS.md ---"));
    assert!(wrapped.contains("Project-specific rule from fixture."));
}

#[test]
fn codex_backend_builds_exec_invocation_for_project_directory() {
    let config = CodexCommandConfig::new(PathBuf::from("/repo"))
        .with_binary("codex")
        .with_model("gpt-5")
        .with_sandbox(SandboxMode::WorkspaceWrite);
    let backend = CodexBackend::new(config, PromptPolicy::for_restricted_agents());

    let invocation = backend.build_launch_invocation(&AgentLaunch::new(
        AgentId::new("chat-a").unwrap(),
        AgentKind::Codex,
        "search",
        "add ripgrep support",
    ));

    assert_eq!(invocation.program, PathBuf::from("codex"));
    assert_eq!(
        invocation.args,
        vec![
            "--cd",
            "/repo",
            "--sandbox",
            "workspace-write",
            "--ask-for-approval",
            "never",
            "--model",
            "gpt-5",
            "exec",
            "--color",
            "never",
            "--json",
            "-"
        ]
    );
    assert!(invocation.stdin.contains("Agent-ID: chat-a"));
    assert!(invocation.stdin.contains("add ripgrep support"));
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn codex_backend_records_agent_replies_in_session_history() {
    let config = CodexCommandConfig::new(PathBuf::from("/repo")).with_binary("printf");
    let mut backend = CodexBackend::new(config, PromptPolicy::for_restricted_agents());

    let session = backend
        .record_launch_reply(
            AgentLaunch::new(
                AgentId::new("chat-a").unwrap(),
                AgentKind::Codex,
                "parser",
                "implement parser",
            ),
            "ready to patch".to_string(),
        )
        .unwrap();

    assert_eq!(session.id.as_str(), "chat-a");
    assert_eq!(session.feature, "parser");
    assert_eq!(session.kind, AgentKind::Codex);
    assert_eq!(session.messages.len(), 2);
    assert_eq!(session.messages[0].role, MessageRole::User);
    assert_eq!(session.messages[1].role, MessageRole::Agent);
    assert_eq!(session.messages[1].text, "ready to patch");
}

#[test]
fn codex_backend_records_json_thread_id_for_follow_up_messages() {
    let config = CodexCommandConfig::new(PathBuf::from("/repo")).with_binary("codex");
    let mut backend = CodexBackend::new(config, PromptPolicy::for_restricted_agents());
    let agent_id = AgentId::new("chat-a").unwrap();

    let session = backend
        .record_launch_output(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "parser", "implement parser"),
            r#"{"type":"thread.started","thread_id":"thread-123"}"#
                .to_string()
                + "\n"
                + r#"{"type":"item.completed","item":{"id":"item-1","type":"agent_message","text":"ready to patch"}}"#,
        )
        .unwrap();

    assert_eq!(session.messages[1].text, "ready to patch");

    let invocation = backend
        .build_send_invocation(&agent_id, "continue")
        .unwrap();

    assert_eq!(
        invocation.args,
        vec![
            "--cd",
            "/repo",
            "--sandbox",
            "workspace-write",
            "--ask-for-approval",
            "never",
            "exec",
            "resume",
            "--json",
            "thread-123",
            "-"
        ]
    );
    assert!(invocation.stdin.contains("continue"));
}

#[test]
fn codex_backend_preserves_multiple_agent_messages_from_one_jsonl_turn() {
    let config = CodexCommandConfig::new(PathBuf::from("/repo")).with_binary("codex");
    let mut backend = CodexBackend::new(config, PromptPolicy::for_restricted_agents());
    let agent_id = AgentId::new("chat-a").unwrap();

    let session = backend
        .record_launch_output(
            AgentLaunch::new(agent_id, AgentKind::Codex, "parser", "implement parser"),
            r#"{"type":"thread.started","thread_id":"thread-123"}"#
                .to_string()
                + "\n"
                + r#"{"type":"item.completed","item":{"id":"item-1","type":"agent_message","text":"@work-leaf read src/ui.rs"}}"#
                + "\n"
                + r#"{"type":"item.completed","item":{"id":"item-2","type":"agent_message","text":"I have requested the relevant files."}}"#,
        )
        .unwrap();

    assert_eq!(
        session.messages[1].text,
        "@work-leaf read src/ui.rs\n\nI have requested the relevant files."
    );
}

#[test]
fn codex_backend_can_build_resume_invocation_without_in_memory_session() {
    let config = CodexCommandConfig::new(PathBuf::from("/repo")).with_binary("codex");
    let backend = CodexBackend::new(config, PromptPolicy::for_restricted_agents());

    let invocation = backend
        .build_send_invocation(&AgentId::new("chat-a").unwrap(), "review the patch")
        .unwrap();

    assert_eq!(invocation.program, PathBuf::from("codex"));
    assert_eq!(
        invocation.args,
        vec![
            "--cd",
            "/repo",
            "--sandbox",
            "workspace-write",
            "--ask-for-approval",
            "never",
            "exec",
            "resume",
            "--json",
            "chat-a",
            "-"
        ]
    );
    assert!(invocation.stdin.contains("Agent-ID: chat-a"));
    assert!(invocation.stdin.contains("Feature: unknown"));
    assert!(invocation.stdin.contains("review the patch"));
}
