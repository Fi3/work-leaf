#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use work_leaf::{CodexBackend, CodexCommandConfig, CommandChat, PromptPolicy, TerminalApp};

#[test]
fn quality_status_behavior() {
    let root = temporary_root();
    let fake_codex = root.join("fake-codex");
    write_provider_probe(&fake_codex);
    let mut app = terminal_app(&root, &fake_codex);

    app.handle_bytes(b":new provider command fixture\n");
    assert!(app.wait_for_idle(Duration::from_secs(4)));
    app.handle_bytes(b"\x1b/status\n");
    assert!(app.wait_for_idle(Duration::from_secs(4)));

    let frame = app.render_frame();
    let log = fs::read_to_string(root.join("requests.log")).unwrap();
    let provider_command = frame.contains("thread-source") && log.contains("thread/read");
    let normal_backend_send = frame.contains("backend status response")
        && log.lines().any(|line| line == "input:/status");
    let backend_local_command = frame.contains("user: /status")
        && frame.contains("Codex backend status")
        && frame.contains("session: thread-source");
    assert!(
        provider_command || normal_backend_send || backend_local_command,
        "selected backend did not show a response for /status\nframe:\n{frame}\nlog:\n{log}"
    );

    drop(app);
    fs::remove_dir_all(root).unwrap();
}

fn terminal_app(root: &Path, fake_codex: &Path) -> TerminalApp<CodexBackend> {
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.to_path_buf()).with_binary(fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    TerminalApp::new(CommandChat::new(root.to_path_buf(), backend), 120, 30)
}

fn temporary_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "work-leaf-quality-status-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    run_git(&root, &["init", "-q"]);
    run_git(&root, &["config", "user.email", "quality@example.com"]);
    run_git(&root, &["config", "user.name", "Quality Evaluator"]);
    fs::write(root.join("README.md"), "provider probe\n").unwrap();
    run_git(&root, &["add", "README.md"]);
    run_git(&root, &["commit", "-q", "-m", "ADD provider probe fixture"]);
    root
}

fn run_git(root: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

fn write_provider_probe(path: &Path) {
    fs::write(
        path,
        r###"#!/bin/sh
log="$(dirname "$0")/requests.log"
printf 'argv:%s\n' "$*" >> "$log"

project_dir="$PWD"
previous=
for arg in "$@"; do
  if [ "$previous" = "--cd" ]; then
    project_dir="$arg"
  fi
  previous="$arg"
done

write_session() {
  session_dir="$project_dir/.codex/sessions/2026/08/20"
  mkdir -p "$session_dir"
  printf '{"timestamp":"2026-08-20T00:00:00Z","type":"session_meta","payload":{"session_id":"thread-source","id":"thread-source","cwd":"%s"}}\n' "$project_dir" > "$session_dir/rollout-thread-source.jsonl"
}

request_id() {
  value=$(printf '%s' "$1" | sed -n 's/.*"id":[[:space:]]*\([^,}]*\).*/\1/p')
  if [ -n "$value" ]; then
    printf '%s' "$value"
  else
    printf '1'
  fi
}

case " $* " in
  *" app-server "*)
    while IFS= read -r line; do
      printf 'rpc:%s\n' "$line" >> "$log"
      id=$(request_id "$line")
      case "$line" in
        *'"method":"initialize"'*)
          printf '{"id":%s,"result":{"userAgent":"quality-probe","codexHome":"/tmp/quality","platformFamily":"unix","platformOs":"linux"}}\n' "$id"
          ;;
        *'"method":"thread/read"'*)
          printf '{"id":%s,"result":{"thread":{"id":"thread-source","cwd":"%s","status":{"type":"idle"},"name":"provider fixture"}}}\n' "$id" "$project_dir"
          ;;
        *'"method":"thread/start"'*)
          write_session
          printf '{"id":%s,"result":{"thread":{"id":"thread-source","cwd":"%s","status":{"type":"idle"}}}}\n' "$id" "$project_dir"
          ;;
        *'"method":"thread/resume"'*)
          printf '{"id":%s,"result":{"thread":{"id":"thread-source","cwd":"%s","status":{"type":"idle"}}}}\n' "$id" "$project_dir"
          ;;
        *'"method":"turn/start"'*)
          printf '{"id":%s,"result":{"turn":{"id":"turn-quality"}}}\n' "$id"
          printf '{"method":"turn/started","params":{"threadId":"thread-source","turnId":"turn-quality","turn":{"id":"turn-quality","status":"inProgress"}}}\n'
          printf '{"method":"item/completed","params":{"threadId":"thread-source","turnId":"turn-quality","item":{"id":"message-quality","type":"agentMessage","text":"launch provider reply"}}}\n'
          printf '{"method":"turn/completed","params":{"threadId":"thread-source","turnId":"turn-quality","turn":{"id":"turn-quality","status":"completed"}}}\n'
          ;;
        *'"method":"config/read"'*)
          printf '{"id":%s,"result":{"config":{"model":"quality-model","modelContextWindow":4096}}}\n' "$id"
          ;;
        *'"method":"account/read"'*)
          printf '{"id":%s,"result":{"account":{"type":"quality-account"}}}\n' "$id"
          ;;
      esac
    done
    exit 0
    ;;
esac

input=$(cat)
printf 'input:%s\n' "$input" >> "$log"
case "$input" in
  /status*) reply='backend status response' ;;
  *) reply='launch provider reply' ;;
esac
write_session
printf '{"type":"thread.started","thread_id":"thread-source"}\n'
printf '{"type":"item.completed","item":{"id":"message","type":"agent_message","text":"%s"}}\n' "$reply"
printf '{"type":"turn.completed","usage":{"input_tokens":1,"cached_input_tokens":0,"output_tokens":1}}\n'
"###,
    )
    .unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}
