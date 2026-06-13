use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use work_leaf::{
    AgentBackend, AgentId, AgentLaunch, AgentStreamEvent, AgentTokenUsage, ClaudeBackend,
    ClaudeCommandConfig, PromptPolicy,
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
            AgentStreamEvent::Usage(AgentTokenUsage {
                input_tokens: 7,
                cached_input_tokens: 2,
                output_tokens: 3,
                reasoning_output_tokens: 0,
            }),
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
            AgentStreamEvent::Usage(AgentTokenUsage {
                input_tokens: 7,
                cached_input_tokens: 2,
                output_tokens: 3,
                reasoning_output_tokens: 0,
            }),
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

#[test]
fn claude_backend_interrupt_terminates_active_turn() {
    let root = temp_dir("claude-backend-interrupts-active-turn");
    let binary = root.join("claude");
    let log = root.join("claude.log");
    write_interruptible_fake_claude(&binary, &log);
    let config = ClaudeCommandConfig::new(root.clone()).with_binary(binary);
    let mut backend = ClaudeBackend::new(config, PromptPolicy::for_restricted_agents());
    let mut worker_backend = backend.clone();
    let agent_id = AgentId::new("user-1").unwrap();
    let launch = AgentLaunch::new(
        agent_id.clone(),
        work_leaf::AgentKind::External("claude".into()),
        "user-agent",
        "launch prompt",
    );
    let (started_tx, started_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut started_tx = Some(started_tx);
        let result = worker_backend.launch_streaming(launch, &mut |event| {
            if matches!(
                event,
                AgentStreamEvent::Status(ref text) if text == "Claude session session-interrupt"
            ) && let Some(sender) = started_tx.take()
            {
                let _ = sender.send(());
            }
        });
        let _ = done_tx.send(result.map(|session| session.id));
    });

    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("fake Claude turn should start");
    backend.interrupt(&agent_id).unwrap();
    let finished_after_interrupt = done_rx.recv_timeout(Duration::from_secs(1)).is_ok();
    if !finished_after_interrupt {
        backend.shutdown_handle().shutdown();
        let _ = done_rx.recv_timeout(Duration::from_secs(1));
    }
    handle.join().unwrap();

    assert!(
        finished_after_interrupt,
        "Claude interrupt should terminate the active turn without waiting for shutdown cleanup"
    );
    let log = fs::read_to_string(log).unwrap();
    assert!(log.contains("TERM"));
}

#[test]
fn claude_backend_interrupt_treats_signaled_exit_as_cancelled_turn() {
    let root = temp_dir("claude-backend-interrupts-signaled-turn");
    let binary = root.join("claude");
    let log = root.join("claude.log");
    write_hanging_fake_claude(&binary, &log, "still working");
    let config = ClaudeCommandConfig::new(root.clone()).with_binary(binary);
    let mut backend = ClaudeBackend::new(config, PromptPolicy::for_restricted_agents());
    let mut worker_backend = backend.clone();
    let agent_id = AgentId::new("user-1").unwrap();
    let launch = AgentLaunch::new(
        agent_id.clone(),
        work_leaf::AgentKind::External("claude".into()),
        "user-agent",
        "launch prompt",
    );
    let (started_tx, started_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut started_tx = Some(started_tx);
        let result = worker_backend.launch_streaming(launch, &mut |event| {
            if matches!(
                event,
                AgentStreamEvent::Status(ref text) if text == "Claude session session-hanging"
            ) && let Some(sender) = started_tx.take()
            {
                let _ = sender.send(());
            }
        });
        let _ = done_tx.send(result.map(|session| session.id));
    });

    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("fake Claude turn should start");
    backend.interrupt(&agent_id).unwrap();
    let result = done_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("interrupted Claude turn should finish");
    handle.join().unwrap();

    assert!(
        result.is_ok(),
        "user interrupts should not surface process-failed errors: {result:?}"
    );
}

#[test]
fn claude_backend_interruptible_streaming_stops_after_terminal_directive() {
    let root = temp_dir("claude-backend-auto-interrupts-directive");
    let binary = root.join("claude");
    let log = root.join("claude.log");
    write_hanging_fake_claude(&binary, &log, "@work-leaf read README.md");
    let config = ClaudeCommandConfig::new(root.clone()).with_binary(binary);
    let backend = ClaudeBackend::new(config, PromptPolicy::for_restricted_agents());
    let mut worker_backend = backend.clone();
    let agent_id = AgentId::new("user-1").unwrap();
    let launch = AgentLaunch::new(
        agent_id,
        work_leaf::AgentKind::External("claude".into()),
        "user-agent",
        "launch prompt",
    );
    let (done_tx, done_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut should_interrupt = |event: &AgentStreamEvent| {
            matches!(
                event,
                AgentStreamEvent::AgentMessage(text) if text == "@work-leaf read README.md"
            )
        };
        let result = worker_backend.launch_streaming_interruptible(
            launch,
            &mut |_| {},
            &mut should_interrupt,
        );
        let _ = done_tx.send(result.map(|session| session.id));
    });

    let result = match done_rx.recv_timeout(Duration::from_secs(1)) {
        Ok(result) => result,
        Err(error) => {
            backend.shutdown_handle().shutdown();
            let _ = done_rx.recv_timeout(Duration::from_secs(1));
            handle.join().unwrap();
            panic!("complete directive did not interrupt Claude turn: {error}");
        }
    };
    handle.join().unwrap();

    assert!(
        result.is_ok(),
        "auto-interrupted Claude turn should finish without process failure: {result:?}"
    );
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

fn write_hanging_fake_claude(path: &Path, log: &Path, stream_text: &str) {
    fs::write(
        path,
        format!(
            r#"#!/bin/sh
printf 'CALL %s\n' "$*" >> '{log}'
while IFS= read -r line; do
  printf 'STDIN %s\n' "$line" >> '{log}'
done
printf '{{"type":"system","subtype":"init","session_id":"session-hanging"}}\n'
printf '{{"type":"system","subtype":"status","status":"requesting","session_id":"session-hanging"}}\n'
printf '{{"type":"stream_event","session_id":"session-hanging","event":{{"type":"content_block_delta","delta":{{"type":"text_delta","text":"{stream_text}"}}}}}}\n'
while :; do
  sleep 0.05 &
  wait $!
done
"#,
            log = log.display(),
            stream_text = stream_text
        ),
    )
    .unwrap();
    make_executable(path);
}

fn write_interruptible_fake_claude(path: &Path, log: &Path) {
    fs::write(
        path,
        format!(
            r#"#!/bin/sh
printf 'CALL %s\n' "$*" >> '{log}'
while IFS= read -r line; do
  printf 'STDIN %s\n' "$line" >> '{log}'
done
trap 'printf "TERM\n" >> "{log}"; exit 0' TERM
printf '{{"type":"system","subtype":"init","session_id":"session-interrupt"}}\n'
printf '{{"type":"system","subtype":"status","status":"requesting","session_id":"session-interrupt"}}\n'
while :; do
  sleep 0.05 &
  wait $!
done
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
