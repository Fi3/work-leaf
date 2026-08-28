use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use work_leaf::{
    AgentBackend, AgentId, AgentKind, AgentLaunch, AgentStreamEvent, CodexBackend,
    CodexCommandConfig, PromptPolicy,
};

mod support;
mod temp_cleanup;

use support::fake_codex::write_app_server_script;

static CODEX_ENV_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
    root
}

fn exact_usage_backend(root: PathBuf, fake_codex: &PathBuf) -> CodexBackend {
    let previous = std::env::var_os("WORK_LEAF_CODEX_EXACT_USAGE");
    unsafe { std::env::set_var("WORK_LEAF_CODEX_EXACT_USAGE", "1") };
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root).with_binary(fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    match previous {
        Some(value) => unsafe { std::env::set_var("WORK_LEAF_CODEX_EXACT_USAGE", value) },
        None => unsafe { std::env::remove_var("WORK_LEAF_CODEX_EXACT_USAGE") },
    }
    backend
}

#[test]
fn codex_backend_app_server_can_enable_exact_raw_usage_events() {
    let _guard = CODEX_ENV_LOCK.lock().unwrap();
    let root = temp_dir("codex-app-server-exact-raw-usage");
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
      thread_result "$id" "thread-exact-usage"
      ;;
    *'"method":"turn/start"'*)
      turn_message_with_usage "$id" "thread-exact-usage" "ready"
      ;;
  esac
done
"#,
    );
    let mut backend = exact_usage_backend(root, &fake_codex);

    backend
        .launch(AgentLaunch::new(
            AgentId::new("user-1").unwrap(),
            AgentKind::Codex,
            "exact usage",
            "measure this turn",
        ))
        .unwrap();
    let requests = fs::read_to_string(fake_bin.join("requests.log")).unwrap();
    let thread_start = requests
        .lines()
        .find(|line| line.contains(r#""method":"thread/start""#))
        .expect("thread/start request is recorded");
    assert!(
        thread_start.contains(r#""experimentalRawEvents":true"#),
        "{thread_start}"
    );
}

#[test]
fn codex_backend_exact_usage_waits_for_response_usage_before_interrupting() {
    let _guard = CODEX_ENV_LOCK.lock().unwrap();
    let root = temp_dir("codex-app-server-exact-usage-before-interrupt");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let fake_codex = fake_bin.join("codex");
    write_app_server_script(
        &fake_codex,
        r#"#!/bin/sh
log="$(dirname "$0")/requests.log"
marker="$(dirname "$0")/exact-usage-emitted"
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
      printf '{"method":"rawResponse/completed","params":{"threadId":"thread-1","turnId":"turn-%s","responseId":"response-early","usage":{"inputTokens":50,"cachedInputTokens":40,"outputTokens":5,"reasoningOutputTokens":2,"totalTokens":55}}}\n' "$id"
      agent_message_item "$id" "thread-1" "$patch"
      sleep 1
      : > "$marker"
      printf '{"method":"rawResponse/completed","params":{"threadId":"thread-1","turnId":"turn-%s","responseId":"response-final","usage":{"inputTokens":100,"cachedInputTokens":80,"outputTokens":10,"reasoningOutputTokens":5,"totalTokens":110}}}\n' "$id"
      IFS= read -r interrupt_line
      printf '%s\n' "$interrupt_line" >> "$log"
      interrupt_id=$(request_id "$interrupt_line")
      rpc_ok "$interrupt_id"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-1" "ready"
      ;;
  esac
done
"#,
    );
    let mut backend = exact_usage_backend(root, &fake_codex);
    let agent_id = AgentId::new("chat-a").unwrap();
    backend
        .launch_streaming(
            AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "app-server", "launch"),
            &mut |_| {},
        )
        .unwrap();
    let mut should_interrupt = |event: &AgentStreamEvent| matches!(event, AgentStreamEvent::AgentMessage(text) if text.contains("@work-leaf end"));

    backend
        .send_streaming_interruptible(&agent_id, "continue", &mut |_| {}, &mut should_interrupt)
        .unwrap();
    assert!(
        fake_bin.join("exact-usage-emitted").exists(),
        "measurement mode returned before exact usage was emitted"
    );
    let requests_path = fake_bin.join("requests.log");
    let mut requests = String::new();
    for _ in 0..20 {
        requests = fs::read_to_string(&requests_path).unwrap();
        if requests.contains(r#""method":"turn/interrupt""#) {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(requests.contains(r#""method":"turn/interrupt""#));
}
