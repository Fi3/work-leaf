use std::path::PathBuf;
use std::process::Command;

use work_leaf::{CliCommand, parse_cli_args, run_cli_command};

#[test]
fn binary_help_exposes_orchestrator_commands_instead_of_greeting() {
    let output = Command::new(env!("CARGO_BIN_EXE_work-leaf"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: work-leaf <command>"));
    assert!(stdout.contains("new <agent-id> <feature> <prompt...>"));
    assert!(stdout.contains("patch <agent-id> <feature> <reason> <diff-file|->"));
    assert!(stdout.contains("review"));
    assert!(stdout.contains("linearize-questions"));
    assert!(!stdout.contains("Hello, world!"));
}

#[test]
fn cli_parser_builds_new_agent_command_from_project_directory_args() {
    let command = parse_cli_args([
        "work-leaf",
        "new",
        "chat-a",
        "parser",
        "implement",
        "the",
        "parser",
    ])
    .unwrap();

    assert_eq!(
        command,
        CliCommand::NewAgent {
            agent_id: "chat-a".to_string(),
            feature: "parser".to_string(),
            prompt: "implement the parser".to_string(),
            model: None,
        }
    );
}

#[test]
fn cli_parser_builds_patch_command() {
    let command = parse_cli_args([
        "work-leaf",
        "patch",
        "chat-a",
        "parser",
        "fix bug",
        "/tmp/change.diff",
    ])
    .unwrap();

    assert_eq!(
        command,
        CliCommand::Patch {
            agent_id: "chat-a".to_string(),
            feature: "parser".to_string(),
            reason: "fix bug".to_string(),
            diff_path: PathBuf::from("/tmp/change.diff"),
        }
    );
}

#[test]
fn locks_classify_command_reports_write_intent() {
    let output = run_cli_command(
        CliCommand::ClassifyCommand {
            command: vec!["cargo".to_string(), "test".to_string()],
        },
        PathBuf::from("/repo"),
        "",
    )
    .unwrap();

    assert!(output.contains("writes: yes"));
    assert!(output.contains("target"));
}

#[test]
fn missing_command_returns_usage_error() {
    let error = parse_cli_args(["work-leaf"]).unwrap_err().to_string();

    assert!(error.contains("missing command"));
    assert!(error.contains("Usage: work-leaf <command>"));
}
