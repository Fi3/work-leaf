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

mod temp_cleanup;

static CODEX_ENV_LOCK: Mutex<()> = Mutex::new(());

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

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) {}

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
fn codex_backend_sdk_sidecar_uses_persistent_sidecar_protocol() {
    let root = temp_dir("codex-sdk-sidecar-protocol");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_python = fake_bin.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"op":"launch"'*)
      text='sdk launch reply'
      thread='sdk-thread-1'
      ;;
    *'"op":"send"'*)
      text='sdk send reply'
      thread='sdk-thread-1'
      ;;
    *'"op":"command"'*)
      text='sdk command reply'
      thread='sdk-thread-1'
      ;;
    *)
      text='sdk ok'
      thread='sdk-thread-1'
      ;;
  esac
  printf '{"id":%s,"event":{"type":"status","text":"Codex is working"}}\n' "$id"
  printf '{"id":%s,"event":{"type":"usage","usage":{"input_tokens":7,"cached_input_tokens":3,"output_tokens":2,"reasoning_output_tokens":1}}}\n' "$id"
  printf '{"id":%s,"ok":true,"thread_id":"%s","reply":"%s","usage":{"input_tokens":7,"cached_input_tokens":3,"output_tokens":2,"reasoning_output_tokens":1}}\n' "$id" "$thread" "$text"
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    let mut events = Vec::new();

    let session = backend
        .launch_streaming(
            AgentLaunch::new(
                agent_id.clone(),
                AgentKind::Codex,
                "sdk",
                "launch through sdk",
            ),
            &mut |event| events.push(event),
        )
        .unwrap();
    let reply = backend
        .send_streaming(&agent_id, "/status", &mut |event| events.push(event))
        .unwrap();

    assert_eq!(session.messages[1].text, "sdk launch reply");
    assert_eq!(reply.text, "sdk command reply");
    assert!(events.contains(&AgentStreamEvent::Status("Codex is working".to_string())));
    assert!(events.contains(&AgentStreamEvent::Usage(AgentTokenUsage {
        input_tokens: 7,
        cached_input_tokens: 3,
        output_tokens: 2,
        reasoning_output_tokens: 1
    })));
}

#[test]
fn codex_backend_uses_sdk_sidecar_without_transport_opt_in() {
    let root = temp_dir("codex-sdk-default-sidecar");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    fs::write(
        &codex,
        "#!/bin/sh\nprintf 'unexpected direct codex invocation\\n' >> \"$(dirname \"$0\")/runs.log\"\nexit 42\n",
    )
    .unwrap();
    make_executable(&codex);
    let fake_python = fake_bin.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-default","reply":"sdk default launch"}\n' "$id"
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary(&codex)
            .with_sdk_python(&fake_python),
        PromptPolicy::for_restricted_agents(),
    );

    let session = backend
        .launch(AgentLaunch::new(
            AgentId::new("chat-a").unwrap(),
            AgentKind::Codex,
            "sdk",
            "launch through default sdk",
        ))
        .unwrap();

    assert_eq!(session.messages[1].text, "sdk default launch");
    assert!(
        !fake_bin.join("runs.log").exists(),
        "resolved Codex binary must not run"
    );
}

#[test]
fn codex_backend_sdk_sidecar_returns_full_streamed_message_transcript() {
    let root = temp_dir("codex-sdk-sidecar-streamed-transcript");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_python = fake_bin.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  printf '{"id":%s,"event":{"type":"message","text":"@work-leaf read src/ui.rs"}}\n' "$id"
  printf '{"id":%s,"event":{"type":"message","text":"@work-leaf done"}}\n' "$id"
  printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-1","reply":"@work-leaf done"}\n' "$id"
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    let mut events = Vec::new();

    let session = backend
        .launch_streaming(
            AgentLaunch::new(agent_id, AgentKind::Codex, "sdk", "launch through sdk"),
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
fn codex_backend_sdk_sidecar_interrupts_after_complete_streamed_directive() {
    let root = temp_dir("codex-sdk-interrupts-streamed-directive");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_python = fake_bin.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
log="$(dirname "$0")/requests.log"
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$log"
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"op":"launch"'*)
      printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-1","reply":"ready"}\n' "$id"
      ;;
    *'"op":"send"'*)
      patch='@work-leaf patch update readme\ndiff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end'
      printf '{"id":%s,"event":{"type":"message","text":"%s"}}\n' "$id" "$patch"
      IFS= read -r interrupt_line
      printf '%s\n' "$interrupt_line" >> "$log"
      interrupt_id=$(printf '%s' "$interrupt_line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
      printf '{"id":%s,"ok":true}\n' "$interrupt_id"
      printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-1","reply":"%s"}\n' "$id" "$patch"
      ;;
    *'"op":"interrupt"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      ;;
    *)
      printf '{"id":%s,"ok":false,"error":"unexpected request"}\n' "$id"
      ;;
  esac
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    backend
        .launch_streaming(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "sdk", "launch"),
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
        if requests.contains(r#""op":"interrupt""#) {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(requests.contains(r#""op":"interrupt""#), "{requests}");
}

#[test]
fn codex_backend_sdk_sidecar_returns_after_requesting_interrupt_without_waiting_for_ack_or_turn_completion()
 {
    let root = temp_dir("codex-sdk-interrupt-returns-before-turn-completion");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_python = fake_bin.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"op":"launch"'*)
      printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-1","reply":"ready"}\n' "$id"
      ;;
    *'"op":"send"'*)
      patch='@work-leaf patch update readme\ndiff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end'
      printf '{"id":%s,"event":{"type":"message","text":"%s"}}\n' "$id" "$patch"
      IFS= read -r interrupt_line
      interrupt_id=$(printf '%s' "$interrupt_line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
      sleep 2
      printf '{"id":%s,"ok":true}\n' "$interrupt_id"
      printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-1","reply":"late completion"}\n' "$id"
      ;;
    *'"op":"interrupt"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      ;;
    *'"op":"shutdown"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      exit 0
      ;;
    *)
      printf '{"id":%s,"ok":false,"error":"unexpected request"}\n' "$id"
      ;;
  esac
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root)
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    backend
        .launch_streaming(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "sdk", "launch"),
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
fn codex_backend_sdk_sidecar_keeps_thread_id_when_launch_returns_after_interrupt_ack() {
    let root = temp_dir("codex-sdk-launch-interrupt-keeps-thread-id");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_python = fake_bin.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"op":"send"'*'"thread_id":"sdk-thread-1"'*)
      printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-1","reply":"continued with sdk-thread-1"}\n' "$id"
      ;;
    *'"op":"launch"'*)
      patch='@work-leaf patch update readme\ndiff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-before\n+after\n@work-leaf end'
      printf '{"id":%s,"event":{"type":"status","text":"Codex session sdk-thread-1"}}\n' "$id"
      printf '{"id":%s,"event":{"type":"message","text":"%s"}}\n' "$id" "$patch"
      IFS= read -r interrupt_line
      interrupt_id=$(printf '%s' "$interrupt_line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
      printf '{"id":%s,"ok":true}\n' "$interrupt_id"
      sleep 1
      printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-1","reply":"late completion"}\n' "$id"
      ;;
    *'"op":"interrupt"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      ;;
    *'"op":"shutdown"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      exit 0
      ;;
    *)
      printf '{"id":%s,"ok":false,"error":"unexpected request"}\n' "$id"
      ;;
  esac
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root)
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    let mut events = Vec::new();
    let mut should_interrupt = |event: &AgentStreamEvent| matches!(event, AgentStreamEvent::AgentMessage(text) if text.contains("@work-leaf end"));

    let session = backend
        .launch_streaming_interruptible(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "sdk", "launch"),
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
        AgentStreamEvent::Status(text) if text == "Codex session sdk-thread-1"
    )));
    let reply = backend.send(&agent_id, "continue").unwrap();
    assert_eq!(reply.text, "continued with sdk-thread-1");
    backend.shutdown();
}

#[test]
fn codex_backend_sdk_sidecar_sends_workspace_write_for_linearize() {
    let root = temp_dir("codex-sdk-linearize-sandbox");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_python = fake_bin.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"agent_id":"linearize"'*'"sandbox":"workspace-write"'*)
      printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-linearize","reply":"linearize sandbox ok"}\n' "$id"
      ;;
    *)
      printf '{"id":%s,"ok":false,"error":"unexpected request"}\n' "$id"
      ;;
  esac
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
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
fn codex_backend_sdk_sidecar_sends_configured_sandbox_for_linearize() {
    let root = temp_dir("codex-sdk-linearize-configured-sandbox");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_python = fake_bin.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"agent_id":"linearize"'*'"sandbox":"danger-full-access"'*)
      printf '{"id":%s,"ok":true,"thread_id":"sdk-thread-linearize","reply":"linearize sandbox ok"}\n' "$id"
      ;;
    *)
      printf '{"id":%s,"ok":false,"error":"unexpected request"}\n' "$id"
      ;;
  esac
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python)
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
fn codex_backend_sdk_sidecar_error_is_reported() {
    let root = temp_dir("codex-sdk-sidecar-error");
    let fake_python = root.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  printf '{"id":%s,"ok":false,"error":"sidecar launch failed"}\n' "$id"
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root)
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
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

    assert!(error.contains("sidecar launch failed"));
}

#[test]
fn codex_backend_removes_parent_codex_and_work_leaf_runtime_environment() {
    let _guard = CODEX_ENV_LOCK.lock().unwrap();
    let root = temp_dir("codex-sdk-env-sanitized");
    let fake_python = root.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
for name in CODEX_THREAD_ID CODEX_CI CODEX_MANAGED_BY_NPM CODEX_MANAGED_PACKAGE_ROOT WORK_LEAF_CODEX_TRACE WORK_LEAF_COMMAND_TMPDIR WORK_LEAF_CONTEXT_BUNDLE_DIR WORK_LEAF_CODEX_LINEARIZE_SANDBOX WORK_LEAF_CODEX_SDK_PYTHON; do
  value=$(eval "printf '%s' \"\${$name-}\"")
  if [ -n "$value" ]; then
    printf '{"id":0,"ok":false,"error":"%s leaked as %s"}\n' "$name" "$value"
    exit 0
  fi
done
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"op":"shutdown"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      exit 0
      ;;
    *)
      printf '{"id":%s,"ok":true,"thread_id":"thread-clean-env","reply":"clean env"}\n' "$id"
      ;;
  esac
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);
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
        (
            "WORK_LEAF_CODEX_SDK_PYTHON",
            std::env::var_os("WORK_LEAF_CODEX_SDK_PYTHON"),
        ),
    ];
    for (name, _) in &saved {
        unsafe { std::env::set_var(name, "parent-value") };
    }

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root)
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
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
    let root = temp_dir("codex-sdk-same-agent-single-flight");
    let fake_python = root.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
dir=$(dirname "$0")
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"op":"launch"'*)
      printf '{"id":%s,"ok":true,"thread_id":"thread-123","reply":"launch reply"}\n' "$id"
      ;;
    *'"op":"send"'*)
      if ! mkdir "$dir/inflight" 2>/dev/null; then
        printf 'overlap\n' >> "$dir/overlap.log"
      fi
      sleep 0.3
      rmdir "$dir/inflight" 2>/dev/null
      printf '{"id":%s,"ok":true,"thread_id":"thread-123","reply":"resume reply"}\n' "$id"
      ;;
    *'"op":"shutdown"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      exit 0
      ;;
  esac
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
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
        "same-agent SDK sends must not overlap"
    );
}

#[test]
fn codex_backend_reuses_one_sdk_sidecar_for_concurrent_launches() {
    let root = temp_dir("codex-sdk-single-sidecar");
    let fake_python = root.join("python");
    fs::write(
        &fake_python,
        r#"#!/bin/sh
dir=$(dirname "$0")
count_file="$dir/start-count"
count=0
if [ -f "$count_file" ]; then
  count=$(cat "$count_file")
fi
count=$((count + 1))
printf '%s\n' "$count" > "$count_file"
printf '%s\n' '{"id":0,"ok":true,"ready":true}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"op":"launch"'*)
      sleep 0.2
      printf '{"id":%s,"ok":true,"thread_id":"thread-%s","reply":"launch reply"}\n' "$id" "$id"
      ;;
    *'"op":"shutdown"'*)
      printf '{"id":%s,"ok":true}\n' "$id"
      exit 0
      ;;
  esac
done
"#,
    )
    .unwrap();
    make_executable(&fake_python);
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary("/usr/bin/codex")
            .with_sdk_python(&fake_python),
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
