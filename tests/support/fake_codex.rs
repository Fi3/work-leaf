use std::fs;
use std::path::{Path, PathBuf};

pub fn write_app_server_script(path: &Path, body: &str) {
    fs::write(path, format!("{APP_SERVER_HEADER}\n{body}")).unwrap();
    make_executable(path);
}

#[allow(dead_code)]
pub fn write_path_app_server(root: &Path, body: &str) -> PathBuf {
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let codex = bin.join("codex");
    write_app_server_script(&codex, body);
    bin
}

const APP_SERVER_HEADER: &str = r#"#!/bin/sh
request_id() {
  printf '%s' "$1" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p'
}

rpc_ok() {
  printf '{"id":"%s","result":{}}\n' "$1"
}

rpc_error() {
  printf '{"id":"%s","error":{"code":-32000,"message":"%s"}}\n' "$1" "$2"
}

thread_result() {
  printf '{"id":"%s","result":{"thread":{"id":"%s","cwd":"%s","status":"loaded"}}}\n' "$1" "$2" "$PWD"
}

turn_started() {
  printf '{"id":"%s","result":{"turn":{"id":"turn-%s"}}}\n' "$1" "$1"
  printf '{"method":"turn/started","params":{"threadId":"%s","turnId":"turn-%s","turn":{"id":"turn-%s","status":"inProgress"}}}\n' "$2" "$1" "$1"
}

turn_completed() {
  printf '{"method":"turn/completed","params":{"threadId":"%s","turnId":"turn-%s","turn":{"id":"turn-%s","status":"completed"}}}\n' "$2" "$1" "$1"
}

agent_message_item() {
  printf '{"method":"item/completed","params":{"threadId":"%s","turnId":"turn-%s","item":{"id":"message-%s","type":"agentMessage","text":"%s"}}}\n' "$2" "$1" "$1" "$3"
}

turn_message() {
  turn_started "$1" "$2"
  agent_message_item "$1" "$2" "$3"
  turn_completed "$1" "$2"
}

turn_message_with_usage() {
  turn_started "$1" "$2"
  printf '{"method":"thread/tokenUsage/updated","params":{"threadId":"%s","turnId":"turn-%s","tokenUsage":{"last":{"inputTokens":7,"cachedInputTokens":3,"outputTokens":2,"reasoningOutputTokens":1}}}}\n' "$2" "$1"
  agent_message_item "$1" "$2" "$3"
  turn_completed "$1" "$2"
}

command_started_item() {
  printf '{"method":"item/started","params":{"threadId":"%s","turnId":"turn-%s","item":{"id":"command-%s","type":"commandExecution","command":"%s","status":"running"}}}\n' "$2" "$1" "$1" "$3"
}
"#;

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
