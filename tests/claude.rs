use std::fs;
use std::path::{Path, PathBuf};

use work_leaf::{
    AgentBackend, AgentId, AgentLaunch, AgentStreamEvent, ClaudeBackend, ClaudeCommandConfig,
    PromptPolicy,
};

mod temp_cleanup;

#[test]
fn claude_backend_streams_launch_and_resumes_existing_session() {
    let root = temp_dir("claude-backend-streams-and-resumes");
    let binary = root.join("claude");
    let log = root.join("claude.log");
    write_fake_claude(&binary, &log);
    let config = ClaudeCommandConfig::new(root.clone()).with_binary(binary);
    let mut backend = ClaudeBackend::new(config, PromptPolicy::for_restricted_agents());
    let agent_id = AgentId::new("user-1").unwrap();
    let mut launch_events = Vec::new();

    let session = backend
        .launch_streaming(
            AgentLaunch::new(
                agent_id.clone(),
                work_leaf::AgentKind::External("claude".into()),
                "user-agent",
                "launch prompt",
            ),
            &mut |event| launch_events.push(event),
        )
        .unwrap();

    assert_eq!(session.id, agent_id);
    assert_eq!(session.messages.last().unwrap().text, "launch reply");
    assert_eq!(
        launch_events,
        vec![
            AgentStreamEvent::Status("Claude session session-launch".to_string()),
            AgentStreamEvent::Status("Claude is working".to_string()),
            AgentStreamEvent::AgentMessage("launch ".to_string()),
            AgentStreamEvent::AgentMessage("reply".to_string()),
        ]
    );

    let mut send_events = Vec::new();
    let reply = backend
        .send_streaming(&agent_id, "follow up", &mut |event| send_events.push(event))
        .unwrap();

    assert_eq!(reply.text, "send reply");
    assert_eq!(
        send_events,
        vec![
            AgentStreamEvent::Status("Claude is working".to_string()),
            AgentStreamEvent::AgentMessage("send ".to_string()),
            AgentStreamEvent::AgentMessage("reply".to_string()),
        ]
    );

    let log = fs::read_to_string(log).unwrap();
    assert!(log.contains("CALL --print --input-format stream-json --output-format stream-json --verbose --include-partial-messages --permission-mode dontAsk --tools "));
    assert!(log.contains("STDIN {"));
    assert!(log.contains("\"type\":\"user\""));
    assert!(log.contains("launch prompt"));
    assert!(log.contains("CALL --print --input-format stream-json --output-format stream-json --verbose --include-partial-messages --permission-mode dontAsk --tools  --resume session-launch"));
    assert!(log.contains("follow up"));
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
    root
}

fn write_fake_claude(path: &Path, log: &Path) {
    fs::write(
        path,
        format!(
            r#"#!/bin/sh
printf 'CALL %s\n' "$*" >> '{log}'
while IFS= read -r line; do
  printf 'STDIN %s\n' "$line" >> '{log}'
done
case " $* " in
  *" --resume session-launch "*)
    session_id=session-launch
    first='send '
    second='reply'
    result='send reply'
    ;;
  *)
    session_id=session-launch
    first='launch '
    second='reply'
    result='launch reply'
    ;;
esac
printf '{{"type":"system","subtype":"init","session_id":"%s"}}\n' "$session_id"
printf '{{"type":"system","subtype":"status","status":"requesting","session_id":"%s"}}\n' "$session_id"
printf '{{"type":"stream_event","session_id":"%s","event":{{"type":"content_block_delta","delta":{{"type":"text_delta","text":"%s"}}}}}}\n' "$session_id" "$first"
printf '{{"type":"stream_event","session_id":"%s","event":{{"type":"content_block_delta","delta":{{"type":"text_delta","text":"%s"}}}}}}\n' "$session_id" "$second"
printf '{{"type":"result","subtype":"success","session_id":"%s","result":"%s","usage":{{"input_tokens":7,"cache_read_input_tokens":2,"output_tokens":3}}}}\n' "$session_id" "$result"
"#,
            log = log.display()
        ),
    )
    .unwrap();
    make_executable(path);
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
