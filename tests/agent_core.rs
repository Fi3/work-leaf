use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

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
    assert!(wrapped.contains("provide a unified diff patch"));
    assert!(wrapped.contains("Do not modify documentation or plain-text files"));
    assert!(wrapped.contains("leave those updates for the linearize agent after review"));
    assert!(wrapped.contains("Do not run `@work-leaf` in a shell"));
    assert!(wrapped.contains("@work-leaf read <path>"));
    assert!(wrapped.contains("@work-leaf read --force <path>"));
    assert!(wrapped.contains("@work-leaf read <path> <path...>"));
    assert!(wrapped.contains("@work-leaf patch <reason>"));
    assert!(wrapped.contains("@work-leaf locks classify <command>"));
    assert!(wrapped.contains("@work-leaf locks run <path> <path...> -- <command>"));
    assert!(wrapped.contains("language- and tool-agnostic"));
    assert!(wrapped.contains("formatter, build, test, code generator, package manager"));
    assert!(wrapped.contains("checks that existed before your patch"));
    assert!(wrapped.contains("Do not run another patch agent's focused tests"));
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
    assert!(wrapped.contains("@work-leaf patch <reason>"));
    assert!(wrapped.contains("@work-leaf locks run <path> <path...> -- <command>"));
    assert!(wrapped.contains("language- and tool-agnostic"));
    assert!(wrapped.contains("checks that existed before your patch"));
    assert!(wrapped.contains("Do not run another patch agent's focused tests"));
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
        "without using `@work-leaf read`, `@work-leaf patch`, or `@work-leaf locks run`"
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
    assert!(
        wrapped
            .to_ascii_lowercase()
            .contains("report the blocker once")
    );
    assert!(wrapped.contains("@work-leaf done"));
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
            "--disable",
            "apps",
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

#[test]
fn codex_backend_launches_linearize_with_workspace_write_sandbox() {
    let config = CodexCommandConfig::new(PathBuf::from("/repo")).with_binary("codex");
    let backend = CodexBackend::new(config, PromptPolicy::for_restricted_agents());

    let invocation = backend.build_launch_invocation(&AgentLaunch::new(
        AgentId::new("linearize").unwrap(),
        AgentKind::Codex,
        "linearize reviewed patches",
        "rewrite reviewed commits",
    ));

    let sandbox_position = invocation
        .args
        .iter()
        .position(|arg| arg == "--sandbox")
        .expect("sandbox argument");
    assert_eq!(invocation.args[sandbox_position + 1], "workspace-write");
    assert!(invocation.stdin.contains("work-leaf linearize agent"));
    assert!(
        invocation
            .stdin
            .contains("allowed to write repository files")
    );
}

#[test]
fn codex_backend_disables_codex_apps_for_daemon_exec_invocations() {
    let config = CodexCommandConfig::new(PathBuf::from("/repo")).with_binary("codex");
    let mut backend = CodexBackend::new(config, PromptPolicy::for_restricted_agents());
    let agent_id = AgentId::new("chat-a").unwrap();

    let launch = backend.build_launch_invocation(&AgentLaunch::new(
        agent_id.clone(),
        AgentKind::Codex,
        "search",
        "add ripgrep support",
    ));
    backend
        .record_launch_output(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "search", "launch"),
            r#"{"type":"thread.started","thread_id":"thread-123"}"#.to_string(),
        )
        .unwrap();
    let resume = backend
        .build_send_invocation(&agent_id, "continue")
        .unwrap();

    for invocation in [launch, resume] {
        assert!(
            invocation
                .args
                .windows(2)
                .any(|args| args == ["--disable", "apps"]),
            "daemon Codex invocations should not depend on the Codex apps/app-server path: {:?}",
            invocation.args
        );
    }
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
            "--disable",
            "apps",
            "--cd",
            "/repo",
            "--sandbox",
            "read-only",
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
fn codex_backend_resume_invocation_uses_raw_follow_up_after_launch_context() {
    let root = temp_dir("codex-resume-raw-follow-up");
    fs::write(
        root.join("AGENTS.md"),
        "Project-specific rule that must appear only in launch context.\n",
    )
    .unwrap();
    let config = CodexCommandConfig::new(root.clone()).with_binary("codex");
    let mut backend = CodexBackend::new(config, PromptPolicy::for_project(&root).unwrap());
    let agent_id = AgentId::new("chat-a").unwrap();
    backend
        .record_launch_output(
            AgentLaunch::new(
                agent_id.clone(),
                AgentKind::Codex,
                "parser",
                "implement parser",
            ),
            r#"{"type":"thread.started","thread_id":"thread-123"}"#.to_string(),
        )
        .unwrap();

    let prompt = "work-leaf patch applied\nfiles: tests/agent_core.rs\nContinue from the repository instructions.";
    let invocation = backend.build_send_invocation(&agent_id, prompt).unwrap();

    assert_eq!(invocation.stdin, prompt);
    assert!(!invocation.stdin.contains("Agent-ID: chat-a"));
    assert!(!invocation.stdin.contains("Project-specific rule"));
}

#[test]
fn codex_backend_status_slash_command_resumes_backend_session_unchanged() {
    let root = temp_dir("codex-status-slash-command");
    let codex = root.join("codex");
    fs::write(
        &codex,
        r#"#!/bin/sh
seen_resume=0
for arg in "$@"; do
  if [ "$arg" = "resume" ]; then
    seen_resume=1
  fi
done
input=$(cat)
if [ "$seen_resume" = "1" ]; then
  printf '%s\n' "$input" >> "$(dirname "$0")/resume.log"
  printf '%s\n' '{"type":"item.completed","item":{"id":"resume","type":"agent_message","text":"backend status output"}}'
else
  printf '%s\n' '{"type":"thread.started","thread_id":"thread-123"}'
  printf '%s\n' '{"type":"item.completed","item":{"id":"launch","type":"agent_message","text":"launch ok"}}'
fi
"#,
    )
    .unwrap();
    make_executable(&codex);
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    backend
        .launch(AgentLaunch::new(
            agent_id.clone(),
            AgentKind::Codex,
            "parser",
            "implement parser",
        ))
        .unwrap();

    let reply = backend.send(&agent_id, "/status").unwrap();

    assert_eq!(reply.text, "backend status output");
    assert_eq!(
        fs::read_to_string(root.join("resume.log")).unwrap(),
        "/status\n"
    );
    let session = backend.session(&agent_id).unwrap();
    assert_eq!(session.messages[2].role, MessageRole::User);
    assert_eq!(session.messages[2].text, "/status");
    assert_eq!(session.messages[3].role, MessageRole::Agent);
    assert_eq!(session.messages[3].text, "backend status output");
}

#[test]
fn codex_backend_streams_recorded_token_usage() {
    let root = temp_dir("codex-token-usage-stream");
    let codex = root.join("codex");
    fs::write(
        &codex,
        "#!/bin/sh\ncat >/dev/null\nprintf '%s\\n' '{\"type\":\"thread.started\",\"thread_id\":\"thread-usage\"}' '{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",\"text\":\"launch ok\"}}' '{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":100,\"cached_input_tokens\":40,\"output_tokens\":9,\"reasoning_output_tokens\":3}}'\n",
    )
    .unwrap();
    make_executable(&codex);
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();

    let mut events = Vec::new();
    backend
        .launch_streaming(
            AgentLaunch::new(
                agent_id.clone(),
                AgentKind::Codex,
                "parser",
                "implement parser",
            ),
            &mut |event| events.push(event),
        )
        .unwrap();

    assert!(
        events.iter().any(|event| matches!(
            event,
            AgentStreamEvent::Usage(AgentTokenUsage {
                input_tokens: 100,
                cached_input_tokens: 40,
                output_tokens: 9,
                reasoning_output_tokens: 3
            })
        )),
        "{events:?}"
    );
}

#[test]
fn codex_backend_fork_slash_command_resumes_backend_session_unchanged() {
    let root = temp_dir("codex-unsupported-slash-command");
    let codex = root.join("codex");
    fs::write(
        &codex,
        r#"#!/bin/sh
seen_resume=0
for arg in "$@"; do
  if [ "$arg" = "resume" ]; then
    seen_resume=1
  fi
done
input=$(cat)
if [ "$seen_resume" = "1" ]; then
  printf '%s\n' "$input" >> "$(dirname "$0")/resume.log"
  printf '%s\n' '{"type":"item.completed","item":{"id":"resume","type":"agent_message","text":"backend fork output"}}'
else
  printf '%s\n' '{"type":"thread.started","thread_id":"thread-123"}'
  printf '%s\n' '{"type":"item.completed","item":{"id":"launch","type":"agent_message","text":"launch ok"}}'
fi
"#,
    )
    .unwrap();
    make_executable(&codex);
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("chat-a").unwrap();
    backend
        .launch(AgentLaunch::new(
            agent_id.clone(),
            AgentKind::Codex,
            "parser",
            "implement parser",
        ))
        .unwrap();

    let reply = backend.send(&agent_id, "/fork try another path").unwrap();

    assert_eq!(reply.text, "backend fork output");
    assert_eq!(
        fs::read_to_string(root.join("resume.log")).unwrap(),
        "/fork try another path\n"
    );
}

#[test]
fn codex_backend_sdk_transport_uses_persistent_sidecar_protocol() {
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
            .with_sdk_transport()
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
fn codex_backend_sdk_transport_returns_full_streamed_message_transcript() {
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
            .with_sdk_transport()
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
fn codex_backend_sdk_transport_sends_workspace_write_for_linearize() {
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
            .with_sdk_transport()
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
fn codex_backend_process_failure_reports_stdout_when_stderr_is_empty() {
    let _guard = CODEX_ENV_LOCK.lock().unwrap();
    let root = temp_dir("codex-failure-stdout");
    let codex = root.join("codex");
    fs::write(
        &codex,
        "#!/bin/sh\nprintf '%s\\n' '{\"type\":\"error\",\"message\":\"Codex ran out of room in the model context window\"}'\nexit 1\n",
    )
    .unwrap();
    make_executable(&codex);
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root).with_binary(&codex),
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

    assert!(error.contains("status Some(1)"));
    assert!(error.contains("stdout:"));
    assert!(error.contains("Codex ran out of room"));
}

#[test]
fn codex_backend_retries_pre_session_app_server_startup_failures() {
    let root = temp_dir("codex-startup-retry");
    let codex = root.join("codex");
    fs::write(
        &codex,
        r#"#!/bin/sh
dir=$(dirname "$0")
count_file="$dir/start-count"
count=0
if [ -f "$count_file" ]; then
  count=$(cat "$count_file")
fi
count=$((count + 1))
printf '%s\n' "$count" > "$count_file"
if [ "$count" -eq 1 ]; then
  printf '%s\n' 'WARNING: proceeding, even though we could not create PATH aliases: Read-only file system (os error 30)' >&2
  printf '%s\n' 'Error: failed to initialize in-process app-server client: Read-only file system (os error 30)' >&2
  exit 1
fi
printf '%s\n' '{"type":"thread.started","thread_id":"thread-retry"}'
printf '%s\n' '{"type":"turn.started"}'
printf '%s\n' '{"type":"item.completed","item":{"id":"launch","type":"agent_message","text":"retry ok"}}'
"#,
    )
    .unwrap();
    make_executable(&codex);
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );

    let session = backend
        .launch(AgentLaunch::new(
            AgentId::new("chat-a").unwrap(),
            AgentKind::Codex,
            "parser",
            "implement parser",
        ))
        .unwrap();

    assert_eq!(session.messages[1].text, "retry ok");
    assert_eq!(fs::read_to_string(root.join("start-count")).unwrap(), "2\n");
}

#[test]
fn codex_backend_does_not_retry_after_session_start_event() {
    let root = temp_dir("codex-startup-no-retry-after-thread");
    let codex = root.join("codex");
    fs::write(
        &codex,
        r#"#!/bin/sh
dir=$(dirname "$0")
count_file="$dir/start-count"
count=0
if [ -f "$count_file" ]; then
  count=$(cat "$count_file")
fi
count=$((count + 1))
printf '%s\n' "$count" > "$count_file"
printf '%s\n' '{"type":"thread.started","thread_id":"thread-started-before-failure"}'
printf '%s\n' 'Error: failed to initialize in-process app-server client: Read-only file system (os error 30)' >&2
exit 1
"#,
    )
    .unwrap();
    make_executable(&codex);
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
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

    assert!(error.contains("thread-started-before-failure"));
    assert_eq!(fs::read_to_string(root.join("start-count")).unwrap(), "1\n");
}

#[test]
fn codex_backend_removes_parent_codex_and_work_leaf_runtime_environment() {
    let _guard = CODEX_ENV_LOCK.lock().unwrap();
    let root = temp_dir("codex-env-sanitized");
    let codex = root.join("codex");
    fs::write(
        &codex,
        r#"#!/bin/sh
if [ -n "${CODEX_THREAD_ID-}" ]; then
  printf 'CODEX_THREAD_ID leaked as %s\n' "$CODEX_THREAD_ID" >&2
  exit 7
fi
for name in CODEX_CI CODEX_MANAGED_BY_NPM CODEX_MANAGED_PACKAGE_ROOT WORK_LEAF_CODEX_TRACE WORK_LEAF_COMMAND_TMPDIR; do
  value=$(eval "printf '%s' \"\${$name-}\"")
  if [ -n "$value" ]; then
    printf '%s leaked as %s\n' "$name" "$value" >&2
    exit 8
  fi
done
printf '%s\n' '{"type":"thread.started","thread_id":"thread-clean-env"}'
printf '%s\n' '{"type":"turn.started"}'
printf '%s\n' '{"type":"item.completed","item":{"id":"env","type":"agent_message","text":"clean env"}}'
"#,
    )
    .unwrap();
    make_executable(&codex);
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
    ];
    for (name, _) in &saved {
        unsafe { std::env::set_var(name, "parent-value") };
    }

    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root).with_binary(&codex),
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
    let root = temp_dir("codex-same-agent-single-flight");
    let codex = root.join("codex");
    fs::write(
        &codex,
        r#"#!/bin/sh
seen_resume=0
for arg in "$@"; do
  if [ "$arg" = "resume" ]; then
    seen_resume=1
  fi
done
dir=$(dirname "$0")
if [ "$seen_resume" = "1" ]; then
  if ! mkdir "$dir/inflight" 2>/dev/null; then
    printf 'overlap\n' >> "$dir/overlap.log"
  fi
  sleep 0.3
  rmdir "$dir/inflight" 2>/dev/null
  printf '%s\n' '{"type":"item.completed","item":{"id":"resume","type":"agent_message","text":"resume reply"}}'
else
  printf '%s\n' '{"type":"thread.started","thread_id":"thread-123"}'
  printf '%s\n' '{"type":"item.completed","item":{"id":"launch","type":"agent_message","text":"launch reply"}}'
fi
"#,
    )
    .unwrap();
    make_executable(&codex);
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let agent_id = AgentId::new("user-1").unwrap();
    backend
        .record_launch_output(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "feature", "launch"),
            r#"{"type":"thread.started","thread_id":"thread-123"}"#.to_string(),
        )
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
        "same-agent resumes must not overlap"
    );
}

#[test]
fn codex_backend_serializes_concurrent_process_startup_until_turn_started() {
    let root = temp_dir("codex-process-start-single-flight");
    let codex = root.join("codex");
    fs::write(
        &codex,
        r#"#!/bin/sh
dir=$(dirname "$0")
if ! mkdir "$dir/start-inflight" 2>/dev/null; then
  printf 'overlap\n' >> "$dir/overlap.log"
fi
sleep 0.2
thread_id=$(printf '%s' "$*" | tr -c 'A-Za-z0-9' '-')
printf '{"type":"thread.started","thread_id":"%s"}\n' "$thread_id"
sleep 0.2
rmdir "$dir/start-inflight" 2>/dev/null
printf '%s\n' '{"type":"turn.started"}'
printf '%s\n' '{"type":"item.completed","item":{"id":"launch","type":"agent_message","text":"launch reply"}}'
"#,
    )
    .unwrap();
    make_executable(&codex);
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
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

    assert!(
        !root.join("overlap.log").exists(),
        "Codex process startup should be serialized until turn.started"
    );
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
            "--disable",
            "apps",
            "--cd",
            "/repo",
            "--sandbox",
            "read-only",
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
