use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

use work_leaf::{
    AgentBackend, AgentId, AgentKind, AgentLaunch, AgentStreamEvent, AgentTokenUsage, CodexBackend,
    CodexCommandConfig, MessageRole, PromptPolicy, SandboxMode,
};

mod support;
mod temp_cleanup;

static CODEX_ENV_LOCK: Mutex<()> = Mutex::new(());
use support::fake_codex::write_app_server_script;

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
    assert!(wrapped.contains("temporary context bundle files"));
    assert!(wrapped.contains("orchestrator-provided file text"));
    assert!(wrapped.contains("not allowed to write files directly"));
    assert!(wrapped.contains("submit a structured edit patch"));
    assert!(wrapped.contains("Do not modify documentation or plain-text files"));
    assert!(wrapped.contains("leave those updates for the linearize agent after review"));
    assert!(wrapped.contains("Do not run `@work-leaf` in a shell"));
    assert!(wrapped.contains("@work-leaf read <path>"));
    assert!(wrapped.contains("@work-leaf read --force <path>"));
    assert!(wrapped.contains("@work-leaf read <path> <path...>"));
    assert!(wrapped.contains("@work-leaf edit <reason>"));
    assert!(wrapped.contains("exact unchanged context lines"));
    assert!(wrapped.contains("legacy `@work-leaf patch <reason>`"));
    assert!(wrapped.contains("@work-leaf locks classify <command>"));
    assert!(wrapped.contains("@work-leaf locks run <path> <path...> -- <command>"));
    assert!(wrapped.contains("language- and tool-agnostic"));
    assert!(wrapped.contains("formatter, build, test, code generator, package manager"));
    assert!(wrapped.contains("checks that existed before your patch"));
    assert!(wrapped.contains("Do not run another patch agent's focused tests"));
    assert!(wrapped.contains("do not edit that other agent's tests"));
    assert!(wrapped.contains("do not submit known-red"));
    assert!(wrapped.contains("submit a cohesive patch"));
    assert!(wrapped.contains("Choose the command from the repository instructions"));
    assert!(wrapped.contains("Do not use command locks for manual feature edits"));
    assert!(wrapped.contains("@work-leaf send <agent-id> <message>"));
    assert!(wrapped.contains("implement the flag parser"));
}

#[test]
fn prompt_policy_can_allow_direct_filesystem_reads() {
    let policy = PromptPolicy::for_direct_read_agents();
    let wrapped = policy.inject(
        &AgentId::new("chat-1").unwrap(),
        "feature flags",
        "implement the flag parser",
    );

    assert!(wrapped.contains("may read repository files directly from the filesystem"));
    assert!(!wrapped.contains("not allowed to read files directly"));
    assert!(!wrapped.contains("ask the orchestrator to provide file text"));
    assert!(!wrapped.contains("@work-leaf read <path>"));
    assert!(wrapped.contains("not allowed to write files directly"));
    assert!(wrapped.contains("Do not modify documentation or plain-text files"));
    assert!(wrapped.contains("@work-leaf edit <reason>"));
    assert!(wrapped.contains("@work-leaf locks run <path> <path...> -- <command>"));
    assert!(wrapped.contains("language- and tool-agnostic"));
    assert!(wrapped.contains("checks that existed before your patch"));
    assert!(wrapped.contains("Do not run another patch agent's focused tests"));
    assert!(wrapped.contains("do not edit that other agent's tests"));
    assert!(wrapped.contains("do not submit known-red"));
    assert!(wrapped.contains("submit a cohesive patch"));
    assert!(wrapped.contains("implement the flag parser"));
}

#[test]
fn prompt_policy_gives_linearize_agent_direct_workspace_access() {
    let policy = PromptPolicy::for_restricted_agents();
    let wrapped = policy.inject(
        &AgentId::new("linearize").unwrap(),
        "linearize reviewed patches",
        "rewrite history",
    );

    assert!(wrapped.contains("work-leaf linearize agent"));
    assert!(wrapped.contains("allowed to read repository files directly"));
    assert!(wrapped.contains("allowed to write repository files"));
    assert!(wrapped.contains(
        "without using `@work-leaf read`, `@work-leaf edit`, `@work-leaf patch`, or `@work-leaf locks run`"
    ));
    assert!(wrapped.contains("Documentation and plain-text updates deferred by patch agents"));
    assert!(!wrapped.contains("not allowed to write files directly"));
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
fn prompt_policy_translates_project_instructions_for_concurrent_patch_agents() {
    let root = temp_dir("prompt-policy-concurrent-project-instructions");
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\n1. `project-wide check`\n\nFollow project-specific APIs.\n",
    )
    .unwrap();
    let policy = PromptPolicy::for_project(&root).unwrap();

    let wrapped = policy.inject(
        &AgentId::new("user-1").unwrap(),
        "feature flags",
        "implement the flag parser",
    );

    assert!(wrapped.contains("Repository instructions from the launch project"));
    assert!(wrapped.contains("Follow project-specific APIs."));
    assert!(wrapped.contains("Concurrent Work Leaf interpretation"));
    assert!(wrapped.contains("preserve the repository-specific intent"));
    assert!(wrapped.contains("Do not repeatedly rerun the same broad check"));
    assert!(wrapped.contains("If a broad required check is blocked only by another patch agent"));
    assert!(wrapped.contains("If a failing test or assertion belongs to another feature"));
    assert!(
        wrapped
            .to_ascii_lowercase()
            .contains("report the blocker once")
    );
    assert!(wrapped.contains("@work-leaf done"));
}

#[test]
fn prompt_policy_builds_concurrent_translation_from_project_instruction_rules() {
    let root = temp_dir("prompt-policy-project-instruction-translation");
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\nRun `cargo fmt` and `cargo test` before submitting.\n\n\
## Documentation\nUpdate README.md when public behavior changes.\n\n\
## Commit message rules\nEvery commit must start with ADD or FIX.\n\n\
## Real Agent Verification\nRun a real agent smoke test for agent-facing behavior.\n",
    )
    .unwrap();
    let policy = PromptPolicy::for_project(&root).unwrap();

    let wrapped = policy.inject(
        &AgentId::new("user-1").unwrap(),
        "feature flags",
        "implement the flag parser",
    );

    assert!(wrapped.contains("Concurrent Work Leaf translation for AGENTS.md"));
    assert!(wrapped.contains("Required checks remain mandatory"));
    assert!(wrapped.contains("checks you added or changed"));
    assert!(wrapped.contains("Documentation rules remain mandatory"));
    assert!(wrapped.contains("handled by the linearize agent"));
    assert!(wrapped.contains("Commit-message rules remain mandatory"));
    assert!(wrapped.contains("patch reason and final linearized commits"));
    assert!(wrapped.contains("Real-agent verification rules remain mandatory"));
    assert!(wrapped.contains("bounded real-agent scenario"));
    assert!(wrapped.contains("Run `cargo fmt` and `cargo test` before submitting."));
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
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
fn codex_backend_app_server_uses_persistent_json_rpc_protocol() {
    let root = temp_dir("codex-app-server-protocol");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_codex = fake_bin.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-1"
      ;;
    *'"method":"thread/resume"'*)
      thread_result "$id" "thread-1"
      ;;
    *'"method":"turn/start"'*)
      turn_message_with_usage "$id" "thread-1" "app-server launch reply"
      ;;
    *'"method":"thread/read"'*)
      thread_result "$id" "thread-1"
      ;;
    *'"method":"config/read"'*)
      printf '{"id":"%s","result":{"config":{"model":"fake-model","modelContextWindow":8192}}}\n' "$id"
      ;;
    *'"method":"account/read"'*)
      printf '{"id":"%s","result":{"account":{"type":"chatgpt"}}}\n' "$id"
      ;;
  esac
done
"#,
    );

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    let mut events = Vec::new();

    let session = backend
        .launch_streaming(
            AgentLaunch::new(
                agent_id.clone(),
                AgentKind::Codex,
                "app-server",
                "launch through app-server",
            ),
            &mut |event| events.push(event),
        )
        .unwrap();
    let reply = backend
        .send_streaming(&agent_id, "/status", &mut |event| events.push(event))
        .unwrap();

    assert_eq!(session.messages[1].text, "app-server launch reply");
    assert!(reply.text.contains("OpenAI Codex app-server status"));
    assert!(reply.text.contains("Model: fake-model"));
    assert!(events.contains(&AgentStreamEvent::Status("Codex is working".to_string())));
    assert!(events.contains(&AgentStreamEvent::Usage(AgentTokenUsage {
        input_tokens: 7,
        cached_input_tokens: 3,
        output_tokens: 2,
        reasoning_output_tokens: 1
    })));
}

#[test]
fn codex_backend_uses_app_server_without_transport_opt_in() {
    let root = temp_dir("codex-app-server-default");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_codex = fake_bin.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-default"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-default" "app-server default launch"
      ;;
  esac
done
"#,
    );

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );

    let session = backend
        .launch(AgentLaunch::new(
            AgentId::new("chat-a").unwrap(),
            AgentKind::Codex,
            "app-server",
            "launch through default app-server",
        ))
        .unwrap();

    assert_eq!(session.messages[1].text, "app-server default launch");
}

#[test]
fn codex_backend_app_server_returns_full_streamed_message_transcript() {
    let root = temp_dir("codex-app-server-streamed-transcript");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_codex = fake_bin.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-1"
      ;;
    *'"method":"turn/start"'*)
      turn_started "$id" "thread-1"
      agent_message_item "$id" "thread-1" "@work-leaf read src/ui.rs"
      agent_message_item "$id" "thread-1" "@work-leaf done"
      turn_completed "$id" "thread-1"
      ;;
  esac
done
"#,
    );

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    let mut events = Vec::new();

    let session = backend
        .launch_streaming(
            AgentLaunch::new(
                agent_id,
                AgentKind::Codex,
                "app-server",
                "launch through app-server",
            ),
            &mut |event| events.push(event),
        )
        .unwrap();

    assert_eq!(
        session.messages[1].text,
        "@work-leaf read src/ui.rs\n\n@work-leaf done"
    );
    assert!(events.contains(&AgentStreamEvent::AgentMessage(
        "@work-leaf read src/ui.rs".to_string()
    )));
    assert!(events.contains(&AgentStreamEvent::AgentMessage(
        "@work-leaf done".to_string()
    )));
}

#[test]
fn codex_backend_app_server_interrupts_after_complete_streamed_directive() {
    let root = temp_dir("codex-app-server-interrupts-streamed-directive");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_codex = fake_bin.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
log="$(dirname "$0")/requests.log"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$log"
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-1"
      ;;
    *'"method":"turn/start"'*'"continue"'*)
      patch='@work-leaf patch update readme\ndiff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end'
      turn_started "$id" "thread-1"
      agent_message_item "$id" "thread-1" "$patch"
      IFS= read -r interrupt_line
      printf '%s\n' "$interrupt_line" >> "$log"
      interrupt_id=$(request_id "$interrupt_line")
      rpc_ok "$interrupt_id"
      turn_completed "$id" "thread-1"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-1" "ready"
      ;;
  esac
done
"#,
    );

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    backend
        .launch_streaming(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "app-server", "launch"),
            &mut |_| {},
        )
        .unwrap();
    let mut events = Vec::new();
    let mut should_interrupt = |event: &AgentStreamEvent| matches!(event, AgentStreamEvent::AgentMessage(text) if text.contains("@work-leaf end"));

    let reply = backend
        .send_streaming_interruptible(
            &agent_id,
            "continue",
            &mut |event| events.push(event),
            &mut should_interrupt,
        )
        .unwrap();

    assert!(reply.text.contains("@work-leaf patch update readme"));
    assert!(events.iter().any(|event| matches!(
        event,
        AgentStreamEvent::AgentMessage(text) if text.contains("@work-leaf end")
    )));
    let requests_path = fake_bin.join("requests.log");
    let mut requests = String::new();
    for _ in 0..20 {
        requests = fs::read_to_string(&requests_path).unwrap();
        if requests.contains(r#""method":"turn/interrupt""#) {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        requests.contains(r#""method":"turn/interrupt""#),
        "{requests}"
    );
}

#[test]
fn codex_backend_app_server_returns_after_requesting_interrupt_without_waiting_for_ack_or_turn_completion()
 {
    let root = temp_dir("codex-app-server-interrupt-returns-before-turn-completion");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_codex = fake_bin.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-1"
      ;;
    *'"method":"turn/start"'*'"continue"'*)
      patch='@work-leaf patch update readme\ndiff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end'
      turn_started "$id" "thread-1"
      agent_message_item "$id" "thread-1" "$patch"
      IFS= read -r interrupt_line
      interrupt_id=$(request_id "$interrupt_line")
      sleep 2
      rpc_ok "$interrupt_id"
      turn_message "$id" "thread-1" "late completion"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-1" "ready"
      ;;
  esac
done
"#,
    );

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    backend
        .launch_streaming(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "app-server", "launch"),
            &mut |_| {},
        )
        .unwrap();

    let (sender, receiver) = mpsc::channel();
    let worker = thread::spawn(move || {
        let mut events = Vec::new();
        let mut should_interrupt = |event: &AgentStreamEvent| matches!(event, AgentStreamEvent::AgentMessage(text) if text.contains("@work-leaf end"));
        let result = backend
            .send_streaming_interruptible(
                &agent_id,
                "continue",
                &mut |event| events.push(event),
                &mut should_interrupt,
            )
            .map(|reply| (reply.text, events))
            .map_err(|error| error.to_string());
        sender.send(result).unwrap();
        backend.shutdown();
    });

    let immediate = receiver.recv_timeout(Duration::from_millis(500));
    let _ = worker.join();
    let (reply, events) = immediate
        .expect("streaming send should return as soon as the directive interrupt is requested")
        .expect("streaming send should succeed");
    assert!(reply.contains("@work-leaf patch update readme"));
    assert!(events.iter().any(|event| matches!(
        event,
        AgentStreamEvent::AgentMessage(text) if text.contains("@work-leaf end")
    )));
}

#[test]
fn codex_backend_app_server_keeps_thread_id_when_launch_returns_after_interrupt_ack() {
    let root = temp_dir("codex-app-server-launch-interrupt-keeps-thread-id");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_codex = fake_bin.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-1"
      ;;
    *'"method":"turn/start"'*'"continue"'*)
      turn_message "$id" "thread-1" "continued with thread-1"
      ;;
    *'"method":"turn/start"'*)
      patch='@work-leaf patch update readme\ndiff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end'
      turn_started "$id" "thread-1"
      agent_message_item "$id" "thread-1" "$patch"
      IFS= read -r interrupt_line
      interrupt_id=$(request_id "$interrupt_line")
      rpc_ok "$interrupt_id"
      sleep 1
      turn_completed "$id" "thread-1"
      ;;
  esac
done
"#,
    );

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    let mut events = Vec::new();
    let mut should_interrupt = |event: &AgentStreamEvent| matches!(event, AgentStreamEvent::AgentMessage(text) if text.contains("@work-leaf end"));

    let session = backend
        .launch_streaming_interruptible(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "app-server", "launch"),
            &mut |event| events.push(event),
            &mut should_interrupt,
        )
        .unwrap();

    assert!(
        session.messages[1]
            .text
            .contains("@work-leaf patch update readme")
    );
    assert!(events.iter().any(|event| matches!(
        event,
        AgentStreamEvent::Status(text) if text == "Codex session thread-1"
    )));
    let reply = backend.send(&agent_id, "continue").unwrap();
    assert_eq!(reply.text, "continued with thread-1");
    backend.shutdown();
}

#[test]
fn codex_backend_app_server_sends_workspace_write_for_linearize() {
    let root = temp_dir("codex-app-server-linearize-sandbox");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_codex = fake_bin.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*'"sandbox":"workspace-write"'*)
      thread_result "$id" "thread-linearize"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-linearize" "linearize sandbox ok"
      ;;
    *'"method":"thread/start"'*)
      rpc_error "$id" "unexpected sandbox"
      ;;
  esac
done
"#,
    );

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );

    let session = backend
        .launch_streaming(
            AgentLaunch::new(
                AgentId::new("linearize").unwrap(),
                AgentKind::Codex,
                "linearize reviewed patches",
                "rewrite history",
            ),
            &mut |_| {},
        )
        .unwrap();

    assert_eq!(session.messages[1].text, "linearize sandbox ok");
}

#[test]
fn codex_backend_app_server_sends_configured_sandbox_for_linearize() {
    let root = temp_dir("codex-app-server-linearize-configured-sandbox");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_codex = fake_bin.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*'"sandbox":"danger-full-access"'*)
      thread_result "$id" "thread-linearize"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-linearize" "linearize sandbox ok"
      ;;
    *'"method":"thread/start"'*)
      rpc_error "$id" "unexpected sandbox"
      ;;
  esac
done
"#,
    );

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary(&fake_codex)
            .with_linearize_sandbox(SandboxMode::DangerFullAccess),
        PromptPolicy::for_restricted_agents(),
    );

    let session = backend
        .launch_streaming(
            AgentLaunch::new(
                AgentId::new("linearize").unwrap(),
                AgentKind::Codex,
                "linearize reviewed patches",
                "rewrite history",
            ),
            &mut |_| {},
        )
        .unwrap();

    assert_eq!(session.messages[1].text, "linearize sandbox ok");
}

#[test]
fn codex_backend_app_server_error_is_reported() {
    let root = temp_dir("codex-app-server-error");
    let fake_codex = root.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      rpc_error "$id" "app-server launch failed"
      ;;
  esac
done
"#,
    );
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );

    let error = backend
        .launch(AgentLaunch::new(
            AgentId::new("chat-a").unwrap(),
            AgentKind::Codex,
            "parser",
            "implement parser",
        ))
        .unwrap_err()
        .to_string();

    assert!(error.contains("app-server launch failed"));
}

#[test]
fn codex_backend_removes_parent_codex_and_work_leaf_runtime_environment() {
    let _guard = CODEX_ENV_LOCK.lock().unwrap();
    let root = temp_dir("codex-app-server-env-sanitized");
    let fake_codex = root.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
for name in CODEX_THREAD_ID CODEX_CI CODEX_MANAGED_BY_NPM CODEX_MANAGED_PACKAGE_ROOT WORK_LEAF_CODEX_TRACE WORK_LEAF_COMMAND_TMPDIR WORK_LEAF_CONTEXT_BUNDLE_DIR WORK_LEAF_CODEX_LINEARIZE_SANDBOX; do
  value=$(eval "printf '%s' \"\${$name-}\"")
  if [ -n "$value" ]; then
    printf '{"id":"0","error":{"code":-32000,"message":"%s leaked as %s"}}\n' "$name" "$value"
    exit 0
  fi
done
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-clean-env"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-clean-env" "clean env"
      ;;
  esac
done
"#,
    );
    let saved = [
        ("CODEX_THREAD_ID", std::env::var_os("CODEX_THREAD_ID")),
        ("CODEX_CI", std::env::var_os("CODEX_CI")),
        (
            "CODEX_MANAGED_BY_NPM",
            std::env::var_os("CODEX_MANAGED_BY_NPM"),
        ),
        (
            "CODEX_MANAGED_PACKAGE_ROOT",
            std::env::var_os("CODEX_MANAGED_PACKAGE_ROOT"),
        ),
        (
            "WORK_LEAF_CODEX_TRACE",
            std::env::var_os("WORK_LEAF_CODEX_TRACE"),
        ),
        (
            "WORK_LEAF_COMMAND_TMPDIR",
            std::env::var_os("WORK_LEAF_COMMAND_TMPDIR"),
        ),
        (
            "WORK_LEAF_CONTEXT_BUNDLE_DIR",
            std::env::var_os("WORK_LEAF_CONTEXT_BUNDLE_DIR"),
        ),
        (
            "WORK_LEAF_CODEX_LINEARIZE_SANDBOX",
            std::env::var_os("WORK_LEAF_CODEX_LINEARIZE_SANDBOX"),
        ),
    ];
    for (name, _) in &saved {
        unsafe { std::env::set_var(name, "parent-value") };
    }

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    let result = backend.launch(AgentLaunch::new(
        AgentId::new("user-1").unwrap(),
        AgentKind::Codex,
        "env",
        "check env",
    ));

    for (name, value) in saved {
        match value {
            Some(value) => unsafe { std::env::set_var(name, value) },
            None => unsafe { std::env::remove_var(name) },
        }
    }

    let session = result.unwrap();
    assert_eq!(session.messages[1].text, "clean env");
}

#[test]
fn codex_backend_serializes_concurrent_sends_to_the_same_agent() {
    let root = temp_dir("codex-app-server-same-agent-single-flight");
    let fake_codex = root.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
dir=$(dirname "$0")
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-123"
      ;;
    *'"method":"turn/start"'*'"first"'*|*'"method":"turn/start"'*'"second"'*)
      if ! mkdir "$dir/inflight" 2>/dev/null; then
        printf 'overlap\n' >> "$dir/overlap.log"
      fi
      sleep 0.3
      rmdir "$dir/inflight" 2>/dev/null
      turn_message "$id" "thread-123" "resume reply"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-123" "launch reply"
      ;;
  esac
done
"#,
    );
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("user-1").unwrap();
    backend
        .launch(AgentLaunch::new(
            agent_id.clone(),
            AgentKind::Codex,
            "feature",
            "launch",
        ))
        .unwrap();

    let barrier = Arc::new(Barrier::new(3));
    let mut first_backend = backend.clone();
    let mut second_backend = backend.clone();
    let first_id = agent_id.clone();
    let second_id = agent_id.clone();
    let first_barrier = Arc::clone(&barrier);
    let second_barrier = Arc::clone(&barrier);
    let first = thread::spawn(move || {
        first_barrier.wait();
        first_backend.send(&first_id, "first").unwrap();
    });
    let second = thread::spawn(move || {
        second_barrier.wait();
        second_backend.send(&second_id, "second").unwrap();
    });
    barrier.wait();
    first.join().unwrap();
    second.join().unwrap();

    assert!(
        !root.join("overlap.log").exists(),
        "same-agent app-server sends must not overlap"
    );
}

#[test]
fn codex_backend_reuses_one_app_server_for_concurrent_launches() {
    let root = temp_dir("codex-app-server-single-process");
    let fake_codex = root.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
dir=$(dirname "$0")
count_file="$dir/start-count"
count=0
if [ -f "$count_file" ]; then
  count=$(cat "$count_file")
fi
count=$((count + 1))
printf '%s\n' "$count" > "$count_file"
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-$id"
      ;;
    *'"method":"turn/start"'*)
      sleep 0.2
      turn_message "$id" "thread-$id" "launch reply"
      ;;
  esac
done
"#,
    );
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );

    let barrier = Arc::new(Barrier::new(3));
    let mut first_backend = backend.clone();
    let mut second_backend = backend.clone();
    let first_barrier = Arc::clone(&barrier);
    let second_barrier = Arc::clone(&barrier);
    let first = thread::spawn(move || {
        first_barrier.wait();
        first_backend
            .launch(AgentLaunch::new(
                AgentId::new("user-1").unwrap(),
                AgentKind::Codex,
                "feature one",
                "launch one",
            ))
            .unwrap();
    });
    let second = thread::spawn(move || {
        second_barrier.wait();
        second_backend
            .launch(AgentLaunch::new(
                AgentId::new("user-2").unwrap(),
                AgentKind::Codex,
                "feature two",
                "launch two",
            ))
            .unwrap();
    });
    barrier.wait();
    first.join().unwrap();
    second.join().unwrap();

    assert_eq!(fs::read_to_string(root.join("start-count")).unwrap(), "1\n");
}
