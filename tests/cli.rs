use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, AgentStreamEvent, ChatMessage,
    CodexBackend, CodexCommandConfig, CommandChat, CommandChatResult, MessageRole, ProcessCommand,
    PromptPolicy, ReadPermission, parse_process_args, render_process_help,
};

mod support;
mod temp_cleanup;

use support::fake_codex::write_app_server_script;

#[test]
fn binary_help_describes_launching_orchestrator_not_internal_operations() {
    let output = Command::new(env!("CARGO_BIN_EXE_work-leaf"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: work-leaf [--model <model>] [--no-read-permission]"));
    assert!(stdout.contains("launches the orchestrator"));
    assert!(stdout.contains("command chat"));
    assert!(stdout.contains("allow agents to read project files directly"));
    assert!(!stdout.contains("patch <agent-id>"));
    assert!(!stdout.contains("locks classify"));
    assert!(!stdout.contains("linearize-questions"));
}

#[test]
fn no_args_launches_orchestrator_from_current_project_directory() {
    let command = parse_process_args(["work-leaf"]).unwrap();

    assert_eq!(
        command,
        ProcessCommand::Launch {
            model: None,
            read_permission: ReadPermission::Orchestrator,
            cli_url: None,
        }
    );
}

#[test]
fn no_read_permission_allows_direct_filesystem_reads() {
    let command = parse_process_args(["work-leaf", "--no-read-permission"]).unwrap();

    assert_eq!(
        command,
        ProcessCommand::Launch {
            model: None,
            read_permission: ReadPermission::DirectFilesystem,
            cli_url: None,
        }
    );
}

#[test]
fn process_args_accept_model_and_no_read_permission_together() {
    let command =
        parse_process_args(["work-leaf", "--no-read-permission", "--model", "gpt-5"]).unwrap();

    assert_eq!(
        command,
        ProcessCommand::Launch {
            model: Some("gpt-5".to_string()),
            read_permission: ReadPermission::DirectFilesystem,
            cli_url: None,
        }
    );
}

#[test]
fn top_level_internal_commands_are_rejected() {
    for args in [
        vec!["work-leaf", "new", "chat-a", "parser", "implement parser"],
        vec![
            "work-leaf",
            "patch",
            "chat-a",
            "parser",
            "reason",
            "diff.patch",
        ],
        vec!["work-leaf", "review"],
        vec!["work-leaf", "linearize-questions"],
        vec!["work-leaf", "locks", "classify", "cargo", "test"],
    ] {
        let error = parse_process_args(args).unwrap_err().to_string();

        assert!(error.contains("work-leaf does not accept top-level workflow commands"));
        assert!(error.contains("command chat"));
    }
}

#[test]
fn command_chat_launches_agents_inside_the_orchestrator() {
    let backend = FakeBackend::new(["agent ready"]);
    let mut chat = CommandChat::new(PathBuf::from("/repo"), backend);

    let result = chat.handle_line("new implement the parser").unwrap();

    assert_eq!(
        result,
        CommandChatResult::AgentLaunched {
            agent_id: AgentId::new("user-1").unwrap(),
            feature: "user-agent".to_string(),
            reply: "agent ready".to_string(),
        }
    );
    let backend = chat.into_backend();
    assert_eq!(backend.launches.len(), 1);
    assert_eq!(backend.launches[0].id.as_str(), "user-1");
    assert_eq!(backend.launches[0].feature, "user-agent");
    assert_eq!(backend.launches[0].prompt, "implement the parser");
}

#[test]
fn command_chat_new_without_prompt_opens_interactive_agent_session() {
    let backend = FakeBackend::new(["agent ready"]);
    let mut chat = CommandChat::new(PathBuf::from("/repo"), backend);

    let result = chat.handle_line("new").unwrap();

    assert_eq!(
        result,
        CommandChatResult::AgentLaunched {
            agent_id: AgentId::new("user-1").unwrap(),
            feature: "user-agent".to_string(),
            reply: "agent ready".to_string(),
        }
    );
    let backend = chat.into_backend();
    assert_eq!(backend.launches.len(), 1);
    assert!(
        backend.launches[0]
            .prompt
            .contains("Ask the user what to work on")
    );
}

#[test]
fn restricted_agent_prompt_advertises_completion_signal() {
    let prompt = PromptPolicy::for_restricted_agents().inject(
        &AgentId::new("user-1").unwrap(),
        "user-agent",
        "finish the task",
    );

    assert!(prompt.contains("Use `@work-leaf done` when no more orchestrator work is required."));
}

#[test]
fn command_chat_processes_agent_orchestrator_requests_automatically() {
    let root = temp_dir("command-chat-agent-side-channel");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn parsed() -> bool { true }\n",
    )
    .unwrap();
    let backend = FakeBackend::new([
        "@work-leaf read src/lib.rs",
        "the parser can use the provided source text",
    ]);
    let mut chat = CommandChat::new(root, backend);

    let result = chat.handle_line("new inspect parser").unwrap();

    let CommandChatResult::AgentLaunched {
        agent_id,
        feature,
        reply,
    } = result
    else {
        panic!("expected launched agent");
    };
    assert_eq!(agent_id, AgentId::new("user-1").unwrap());
    assert_eq!(feature, "user-agent");
    assert!(reply.contains("@work-leaf read src/lib.rs"));
    assert!(reply.contains("orchestrator:"));
    assert!(reply.contains("sent file text to user-1"));
    assert!(reply.contains("agent follow-up from user-1:"));
    assert!(reply.contains("the parser can use the provided source text"));

    let backend = chat.into_backend();
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0, AgentId::new("user-1").unwrap());
    assert!(backend.sends[0].1.contains("src/lib.rs"));
    assert!(backend.sends[0].1.contains("pub fn parsed()"));
}

#[test]
fn command_chat_times_out_long_locked_command_runs() {
    let root = temp_git_repo("command-chat-locked-command-timeout");
    fs::write(root.join("README.md"), "before\n").unwrap();
    fs::write(
        root.join("slow.sh"),
        "sleep 1\nprintf 'late\\n' > README.md\n",
    )
    .unwrap();
    let backend = FakeBackend::new([
        "@work-leaf locks run README.md -- sh slow.sh",
        "@work-leaf done",
    ]);
    let mut chat = CommandChat::new(root.clone(), backend)
        .with_locked_command_timeout(Duration::from_millis(50));

    let result = chat.handle_line("new validate timeout").unwrap();

    let CommandChatResult::AgentLaunched { reply, .. } = result else {
        panic!("expected launched agent");
    };
    assert!(reply.contains("agent follow-up from user-1"));
    let backend = chat.into_backend();
    assert_eq!(backend.sends.len(), 1);
    assert!(backend.sends[0].1.contains("timed out: yes"));
    assert!(backend.sends[0].1.contains("user authorization"));
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "before\n"
    );
}

#[test]
fn command_chat_corrects_agents_that_treat_work_leaf_as_a_shell_command() {
    let root = temp_dir("command-chat-agent-protocol-correction");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn parsed() -> bool { true }\n",
    )
    .unwrap();
    let backend = FakeBackend::new([
        "The orchestrator mediation path is not applying patches or returning the requested runtime files, and the shell does not have an `@work-leaf` command available. To finish the requested feature end to end, I am switching to the local workspace tools.",
        "@work-leaf read src/lib.rs",
        "received orchestrator file text\n@work-leaf done",
    ]);
    let mut chat = CommandChat::new(root, backend);

    let result = chat.handle_line("new inspect parser").unwrap();

    let CommandChatResult::AgentLaunched { reply, .. } = result else {
        panic!("expected launched agent");
    };
    assert!(reply.contains("sent protocol correction to user-1"));
    assert!(reply.contains("sent file text to user-1: src/lib.rs"));
    assert!(reply.contains("agent user-1 reported done"));

    let backend = chat.into_backend();
    assert_eq!(backend.sends.len(), 2);
    assert!(
        backend.sends[0]
            .1
            .contains("`@work-leaf` is not a shell command")
    );
    assert!(backend.sends[0].1.contains("plain response lines"));
    assert!(backend.sends[1].1.contains("pub fn parsed()"));
}

#[test]
fn command_chat_sends_malformed_patch_feedback_instead_of_erroring() {
    let root = temp_git_repo("command-chat-malformed-patch-feedback");
    fs::write(root.join("README.md"), "actual\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = FakeBackend::new([
        "\
@work-leaf patch update readme
README.md should say changed.
@work-leaf end",
        "@work-leaf done",
    ]);
    let mut chat = CommandChat::new(root.clone(), backend);

    let result = chat.handle_line("new update readme").unwrap();

    let CommandChatResult::AgentLaunched { reply, .. } = result else {
        panic!("expected launched agent");
    };
    assert!(!reply.contains("error: patch does not touch any files"));
    assert!(reply.contains("sent patch diagnostics to user-1"));
    assert!(reply.contains("agent user-1 reported done"));
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "actual\n"
    );
    assert!(git_output(&root, ["status", "--short"]).is_empty());

    let backend = chat.into_backend();
    assert_eq!(backend.sends.len(), 1);
    assert!(
        backend.sends[0]
            .1
            .contains("recognizable unified diff file headers")
    );
    assert!(backend.sends[0].1.contains("@work-leaf patch <reason>"));
}

#[test]
fn command_chat_continues_past_old_round_cutoff_until_agent_reports_done() {
    let root = temp_dir("command-chat-agent-done-convergence");
    let mut replies = Vec::new();
    for index in 0..10 {
        let path = format!("round-{index}.txt");
        fs::write(root.join(&path), format!("round {index}\n")).unwrap();
        replies.push(format!("@work-leaf read {path}"));
    }
    replies.push("@work-leaf done".to_string());
    let backend = FakeBackend::from_replies(replies);
    let mut chat = CommandChat::new(root, backend);

    let result = chat.handle_line("new converge through reads").unwrap();

    let CommandChatResult::AgentLaunched { reply, .. } = result else {
        panic!("expected launched agent");
    };
    assert!(reply.contains("sent file text to user-1: round-9.txt"));
    assert!(reply.contains("agent user-1 reported done"));
    assert!(!reply.contains("agent did not converge"));

    let backend = chat.into_backend();
    assert_eq!(backend.sends.len(), 10);
    assert!(backend.sends[9].1.contains("round-9.txt"));
}

#[test]
fn command_chat_reports_non_convergence_when_emergency_guard_trips() {
    let root = temp_dir("command-chat-agent-non-convergence");
    fs::write(root.join("loop.txt"), "loop\n").unwrap();
    let backend = FakeBackend::from_replies([
        "@work-leaf read loop.txt",
        "@work-leaf read loop.txt",
        "@work-leaf read loop.txt",
        "@work-leaf read loop.txt",
    ]);
    let mut chat = CommandChat::new(root, backend).with_max_review_rounds(3);

    let result = chat.handle_line("new loop forever").unwrap();

    let CommandChatResult::AgentLaunched { reply, .. } = result else {
        panic!("expected launched agent");
    };
    assert!(reply.contains("agent did not converge after 3 orchestrator rounds"));
    assert!(
        !reply.contains("stopped processing agent directives after the configured round limit")
    );

    let backend = chat.into_backend();
    assert_eq!(backend.sends.len(), 3);
}

#[test]
fn command_chat_applies_agent_patch_requests_automatically() {
    let root = temp_git_repo("command-chat-agent-patch");
    fs::write(root.join("lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial patch fixture"]);
    let backend = FakeBackend::new([
        "\
@work-leaf patch return value two
diff --git a/lib.rs b/lib.rs
--- a/lib.rs
+++ b/lib.rs
@@ -1 +1 @@
-pub fn value() -> u8 { 1 }
+pub fn value() -> u8 { 2 }
@work-leaf end",
        "@work-leaf done",
    ]);
    let mut chat = CommandChat::new(root.clone(), backend);

    let result = chat.handle_line("new update value").unwrap();

    let CommandChatResult::AgentLaunched { reply, .. } = result else {
        panic!("expected launched agent");
    };
    assert!(reply.contains("orchestrator:"));
    assert!(reply.contains("applied patch from user-1"));
    assert_eq!(
        fs::read_to_string(root.join("lib.rs")).unwrap(),
        "pub fn value() -> u8 { 2 }\n"
    );
    let message = git_output(&root, ["log", "-1", "--pretty=%B"]);
    assert!(message.contains("Agent-ID: user-1"));
    assert!(message.contains("Feature: user-agent"));
    assert!(message.contains("Reason: return value two"));
    let backend = chat.into_backend();
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0, AgentId::new("user-1").unwrap());
    assert!(backend.sends[0].1.contains("work-leaf patch applied"));
    assert!(backend.sends[0].1.contains("@work-leaf done"));
}

#[test]
fn command_chat_interrupts_streaming_after_complete_patch_directive() {
    let root = temp_git_repo("command-chat-interrupts-after-streamed-patch");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        ["commit", "-m", "ADD initial streaming patch fixture"],
    );
    let backend = DuplicateStreamingPatchBackend::default();
    let mut chat = CommandChat::new(root.clone(), backend);

    chat.handle_line("new update readme").unwrap();
    let result = chat
        .send_to_agent(&AgentId::new("user-1").unwrap(), "continue")
        .unwrap();

    let CommandChatResult::AgentMessage { reply, .. } = result else {
        panic!("expected agent reply");
    };
    assert!(reply.contains("applied patch from user-1"));
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after\n"
    );
    assert_eq!(
        git_output(&root, ["log", "--oneline", "--all"])
            .lines()
            .count(),
        2
    );

    let backend = chat.into_backend();
    assert!(
        backend.interrupted,
        "backend should have stopped after the first complete patch block"
    );
    assert_eq!(
        backend.streamed_patch_blocks, 1,
        "duplicate streamed patch blocks should not be emitted"
    );
}

#[test]
fn command_chat_reuses_reviewer_for_later_commit_from_same_patch_agent() {
    let root = temp_git_repo("command-chat-reuses-reviewer");
    fs::write(root.join("README.md"), "first\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply docs patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: docs\nReason: first pass\nContext: docs context",
        ],
    );
    let backend = FakeBackend::new(["NO_FINDINGS", "NO_FINDINGS"]);
    let mut chat = CommandChat::new(root.clone(), backend);

    let first = chat.handle_line("review").unwrap();
    assert!(matches!(first, CommandChatResult::ReviewComplete(_)));

    fs::write(root.join("README.md"), "second\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply docs patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: docs\nReason: second pass\nContext: docs context",
        ],
    );

    let second = chat.handle_line("review").unwrap();
    assert!(matches!(second, CommandChatResult::ReviewComplete(_)));

    let backend = chat.into_backend();
    assert_eq!(
        backend
            .launches
            .iter()
            .filter(|launch| launch.id.as_str() == "review-user-1")
            .count(),
        1
    );
    assert!(
        backend.launches[0]
            .prompt
            .contains("Source context from Work Leaf commits, logs, and chat history")
    );
    assert!(backend.launches[0].prompt.contains(
        "Work Leaf collected this context from commits, git logs, and recorded chat history without querying Agent-ID user-1"
    ));
    assert!(backend.launches[0].prompt.contains("first pass"));
    assert!(backend.sends.iter().any(|(target, prompt)| {
        target.as_str() == "review-user-1"
            && prompt.contains("Review the full patch scope")
            && prompt.contains("second pass")
            && prompt.contains(
                "Documentation and plain-text updates are deferred to the linearize agent",
            )
    }));
}

#[test]
fn command_chat_requires_force_linearize_for_direct_linearize_launch() {
    let root = temp_git_repo("command-chat-force-linearize");
    fs::write(root.join("README.md"), "first\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply docs patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: docs\nReason: first pass\nContext: docs context",
        ],
    );
    let backend = FakeBackend::new(["NO_FINDINGS", "linearizer ready"]);
    let mut chat = CommandChat::new(root, backend);

    let review = chat.handle_line("review").unwrap();
    assert!(matches!(review, CommandChatResult::ReviewComplete(_)));

    let error = chat.handle_line("linearize").unwrap_err().to_string();
    assert!(error.contains("reviewed patch chats must be classified as closed"));
    assert!(error.contains("force-linearize"));

    let result = chat.handle_line("force-linearize").unwrap();
    assert!(matches!(
        result,
        CommandChatResult::AgentLaunched { agent_id, .. } if agent_id.as_str() == "linearize"
    ));
}

#[test]
fn command_chat_processes_reviewer_orchestrator_directives_before_findings() {
    let root = temp_git_repo("command-chat-reviewer-directives");
    fs::write(root.join("README.md"), "review target\n").unwrap();
    git(&root, ["add", "README.md"]);
    git(
        &root,
        [
            "commit",
            "-m",
            "UPDATE apply docs patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: docs\nReason: review docs\nContext: docs context",
        ],
    );
    let backend = FakeBackend::new(["@work-leaf read README.md", "NO_FINDINGS"]);
    let mut chat = CommandChat::new(root, backend);

    let result = chat.handle_line("review").unwrap();

    let CommandChatResult::ReviewComplete(results) = result else {
        panic!("expected review results");
    };
    assert_eq!(results.len(), 1);
    assert!(results[0].findings_resolved);

    let backend = chat.into_backend();
    assert!(
        backend.launches[0]
            .prompt
            .contains("Source context from Work Leaf commits, logs, and chat history")
    );
    assert!(backend.launches[0].prompt.contains(
        "Work Leaf collected this context from commits, git logs, and recorded chat history without querying Agent-ID user-1"
    ));
    assert!(backend.sends.iter().any(|(target, prompt)| {
        target.as_str() == "review-user-1"
            && prompt.contains("work-leaf file text")
            && prompt.contains("review target")
    }));
    assert!(!backend.sends.iter().any(|(target, prompt)| {
        target.as_str() == "user-1" && prompt.contains("The reviewer found issues")
    }));
}

#[test]
fn command_chat_does_not_interrupt_agents_with_stale_file_updates() {
    let root = temp_git_repo("command-chat-no-stale-file-interrupt");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial stale file fixture"]);
    let backend = FakeBackend::from_replies([
        "@work-leaf read README.md",
        "user-1 is drafting from the provided snapshot",
        "\
@work-leaf patch update readme
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-before
+after
@work-leaf end",
        "@work-leaf done",
    ]);
    let mut chat = CommandChat::new(root.clone(), backend);

    let first = chat.handle_line("new inspect readme").unwrap();
    let CommandChatResult::AgentLaunched { reply, .. } = first else {
        panic!("expected first launched agent");
    };
    assert!(reply.contains("sent file text to user-1: README.md"));
    assert!(reply.contains("user-1 is drafting from the provided snapshot"));

    let second = chat.handle_line("new patch readme").unwrap();
    let CommandChatResult::AgentLaunched { reply, .. } = second else {
        panic!("expected second launched agent");
    };

    assert!(reply.contains("applied patch from user-2"));
    assert!(reply.contains("agent user-2 reported done"));
    assert!(!reply.contains("sent file update to user-1"));
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after\n"
    );

    let backend = chat.into_backend();
    assert_eq!(backend.sends.len(), 2);
    assert_eq!(backend.sends[0].0, AgentId::new("user-1").unwrap());
    assert!(backend.sends[0].1.contains("before"));
    assert_eq!(backend.sends[1].0, AgentId::new("user-2").unwrap());
    assert!(backend.sends[1].1.contains("work-leaf patch applied"));
}

#[test]
fn command_chat_spawned_codex_handles_read_classify_patch_and_route_directives() {
    let root = temp_git_repo("command-chat-spawned-codex-protocol");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        ["commit", "-m", "ADD initial spawned protocol fixture"],
    );
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    write_app_server_script(
        &codex,
        r#"#!/bin/sh
dir=$(dirname "$0")
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$dir/requests.log"
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-protocol"
      ;;
    *'"method":"turn/start"'*"work-leaf file text"*)
      turn_message "$id" "thread-protocol" "read follow-up received src/lib.rs"
      ;;
    *'"method":"turn/start"'*"work-leaf command classification"*)
      turn_message "$id" "thread-protocol" "classification follow-up received target lock"
      ;;
    *'"method":"turn/start"'*"work-leaf command result"*)
      turn_message "$id" "thread-protocol" "command result follow-up received"
      ;;
    *'"method":"turn/start"'*"Message from user-1"*)
      turn_message "$id" "thread-protocol" "routed follow-up received"
      ;;
    *'"method":"turn/start"'*"run full protocol"*)
      reply='@work-leaf read src/lib.rs\n@work-leaf locks classify cargo test\n@work-leaf locks run target -- sh -c \"printf command-run-ok\"\n@work-leaf patch return value two\ndiff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-pub fn value() -> u8 { 1 }\n+pub fn value() -> u8 { 2 }\n@work-leaf end\n@work-leaf send user-2 please check this patch'
      turn_message "$id" "thread-protocol" "$reply"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-protocol" "unexpected app-server prompt"
      ;;
  esac
done
"#,
    );
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let mut chat = CommandChat::new(root.clone(), backend);

    let result = chat.handle_line("new run full protocol").unwrap();

    let CommandChatResult::AgentLaunched { reply, .. } = result else {
        panic!("expected launched agent");
    };
    assert!(reply.contains("sent file text to user-1: src/lib.rs"));
    assert!(reply.contains("classified command for user-1: writes=yes paths=target"));
    assert!(reply.contains("ran command for user-1: status=0 paths=target"));
    assert!(
        reply.contains("applied patch from user-1: return value two"),
        "{reply}"
    );
    assert!(reply.contains("routed message from user-1 to user-2"));
    assert!(reply.contains("read follow-up received src/lib.rs"));
    assert!(reply.contains("classification follow-up received target lock"));
    assert!(reply.contains("command result follow-up received"));
    assert!(reply.contains("routed follow-up received"));
    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "pub fn value() -> u8 { 2 }\n"
    );
    let message = git_output(&root, ["log", "-1", "--pretty=%B"]);
    assert!(message.contains("Agent-ID: user-1"));
    assert!(message.contains("Feature: user-agent"));
    assert!(message.contains("Reason: return value two"));
}

#[test]
fn command_chat_processes_app_server_streamed_directives_before_final_reply() {
    let root = temp_git_repo("command-chat-app-server-streamed-directives");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, ["add", "src/lib.rs"]);
    git(
        &root,
        ["commit", "-m", "ADD initial streamed directive fixture"],
    );
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
      patch='@work-leaf patch return value two\ndiff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-pub fn value() -> u8 { 1 }\n+pub fn value() -> u8 { 2 }\n@work-leaf end\n@work-leaf done'
      turn_started "$id" "thread-1"
      agent_message_item "$id" "thread-1" "$patch"
      IFS= read -r interrupt_line
      interrupt_id=$(request_id "$interrupt_line")
      rpc_ok "$interrupt_id"
      turn_completed "$id" "thread-1"
      ;;
  esac
done
"#,
    );
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&fake_codex),
        PromptPolicy::for_restricted_agents(),
    );
    let mut chat = CommandChat::new(root.clone(), backend);

    let result = chat
        .handle_line("new run app-server streamed directives")
        .unwrap();

    let CommandChatResult::AgentLaunched { reply, .. } = result else {
        panic!("expected launched agent");
    };
    assert!(reply.contains("applied patch from user-1: return value two"));
    assert!(reply.contains("agent user-1 reported done"));
    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "pub fn value() -> u8 { 2 }\n"
    );
    let message = git_output(&root, ["log", "-1", "--pretty=%B"]);
    assert!(message.contains("Agent-ID: user-1"));
    assert!(message.contains("Reason: return value two"));
}

#[test]
fn command_chat_streams_locked_command_progress_status() {
    let root = temp_git_repo("command-chat-locked-command-progress");
    let backend = FakeBackend::new([
        "@work-leaf locks run target -- sh -c 'printf command-ok'",
        "@work-leaf done",
    ]);
    let mut chat = CommandChat::new(root, backend);
    let launch = chat
        .prepare_agent_launch(&["validate".to_string()])
        .expect("launch is prepared");
    let mut events = Vec::new();

    let result = chat
        .launch_prepared_agent_streaming_with_ids(launch, &mut |agent_id, event| {
            events.push((agent_id.clone(), event));
        })
        .unwrap();

    assert!(matches!(result, CommandChatResult::AgentLaunched { .. }));
    assert!(events.iter().any(|(agent_id, event)| {
        agent_id.as_str() == "user-1"
            && matches!(
                event,
                AgentStreamEvent::Status(text)
                    if text.contains("running locked command")
                        && text.contains("sh -c 'printf command-ok'")
            )
    }));
}

#[test]
fn failed_agent_launch_does_not_consume_user_agent_id() {
    let backend = FlakyLaunchBackend::default();
    let mut chat = CommandChat::new(PathBuf::from("/repo"), backend);

    let error = chat.handle_line("new first try").unwrap_err().to_string();
    assert!(error.contains("first launch failed"));

    let result = chat.handle_line("new retry").unwrap();

    assert_eq!(
        result,
        CommandChatResult::AgentLaunched {
            agent_id: AgentId::new("user-1").unwrap(),
            feature: "user-agent".to_string(),
            reply: "agent ready".to_string(),
        }
    );
}

#[test]
fn scripted_command_chat_reports_agent_launch_error_without_exiting() {
    let root = temp_dir("scripted-new-error");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    write_app_server_script(
        &codex,
        r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      rpc_error "$id" "codex launch failed"
      ;;
  esac
done
"#,
    );

    let mut child = Command::new(env!("CARGO_BIN_EXE_work-leaf"))
        .env("PATH", format!("{}:{}", fake_bin.display(), current_path()))
        .env("WORK_LEAF_IN_PROCESS", "1")
        .env_remove("WORK_LEAF_ORCHESTRATOR_URL")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"new\nhelp\nquit\n")
        .unwrap();

    let output = child.wait_with_output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "status: {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    );
    assert!(stdout.contains("work-leaf orchestrator"));
    assert!(stdout.contains("codex launch failed"));
    assert!(stdout.contains("Command chat:"));
    assert!(stderr.is_empty());
}

#[test]
fn process_help_mentions_internal_actions_as_in_app_commands_only() {
    let help = render_process_help();

    assert!(help.contains("Inside command chat"));
    assert!(help.contains("--no-read-permission"));
    assert!(help.contains("new [prompt...]"));
    assert!(help.contains("review"));
    assert!(help.contains("linearize"));
    assert!(!help.contains("Usage: work-leaf <command>"));
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
    root
}

fn temp_git_repo(name: &str) -> PathBuf {
    let root = temp_dir(name);
    git(&root, ["init"]);
    git(&root, ["config", "user.name", "Work Leaf Test"]);
    git(&root, ["config", "user.email", "work-leaf@example.test"]);
    root
}

fn git<const N: usize>(root: &std::path::Path, args: [&str; N]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output<const N: usize>(root: &std::path::Path, args: [&str; N]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn current_path() -> String {
    std::env::var("PATH").unwrap_or_default()
}

#[derive(Debug)]
struct FakeBackend {
    replies: VecDeque<String>,
    launches: Vec<work_leaf::AgentLaunch>,
    sends: Vec<(AgentId, String)>,
}

#[derive(Debug, Default)]
struct DuplicateStreamingPatchBackend {
    interrupted: bool,
    streamed_patch_blocks: usize,
}

impl AgentBackend for DuplicateStreamingPatchBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "done"))
    }

    fn send_streaming_interruptible(
        &mut self,
        _agent_id: &AgentId,
        _prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        if self.streamed_patch_blocks > 0 {
            let done = AgentStreamEvent::AgentMessage("@work-leaf done".to_string());
            sink(done.clone());
            let _ = should_interrupt(&done);
            return Ok(ChatMessage::new(MessageRole::Agent, "@work-leaf done"));
        }

        let patch = "\
@work-leaf patch update readme
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-before
+after
@work-leaf end";
        let event = AgentStreamEvent::AgentMessage(patch.to_string());
        sink(event.clone());
        self.streamed_patch_blocks += 1;
        if should_interrupt(&event) {
            self.interrupted = true;
            return Ok(ChatMessage::new(MessageRole::Agent, patch));
        }

        sink(AgentStreamEvent::AgentMessage(patch.to_string()));
        self.streamed_patch_blocks += 1;
        Ok(ChatMessage::new(
            MessageRole::Agent,
            format!("{patch}\n\n{patch}"),
        ))
    }
}

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self::from_replies(replies)
    }

    fn from_replies<I, S>(replies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            replies: replies.into_iter().map(Into::into).collect(),
            launches: Vec::new(),
            sends: Vec::new(),
        }
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: work_leaf::AgentLaunch) -> Result<AgentSession, AgentError> {
        self.launches.push(request.clone());
        let mut session = AgentSession::new(request);
        session.push_message(
            MessageRole::Agent,
            self.replies.pop_front().expect("missing fake reply"),
        );
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.sends.push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(
            MessageRole::Agent,
            self.replies.pop_front().expect("missing fake reply"),
        ))
    }
}

#[derive(Debug, Default)]
struct FlakyLaunchBackend {
    attempts: usize,
}

impl AgentBackend for FlakyLaunchBackend {
    fn launch(&mut self, request: work_leaf::AgentLaunch) -> Result<AgentSession, AgentError> {
        self.attempts += 1;
        if self.attempts == 1 {
            return Err(AgentError::ProcessFailed {
                program: PathBuf::from("codex"),
                status: Some(42),
                stderr: "first launch failed".to_string(),
            });
        }
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "agent ready");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "reply"))
    }
}
