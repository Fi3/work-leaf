use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentOrchestrator, AgentSession, ChatMessage, MessageRole,
    OrchestratorEvent,
};

#[test]
fn orchestrator_protocol_reads_files_and_classifies_commands_for_agents() {
    let root = temp_git_repo("protocol-read-classify");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root, backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "parser",
            "@work-leaf read src/lib.rs\n@work-leaf locks classify cargo test",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(
        events,
        vec![
            OrchestratorEvent::FileTextSent {
                agent_id: agent_id.clone(),
                paths: vec![PathBuf::from("src/lib.rs")]
            },
            OrchestratorEvent::CommandClassified {
                agent_id: agent_id.clone(),
                writes: true,
                paths: vec![PathBuf::from("target")]
            }
        ]
    );
    assert_eq!(backend.sends.len(), 2);
    assert_eq!(backend.sends[0].0, agent_id);
    assert!(backend.sends[0].1.contains("src/lib.rs"));
    assert!(backend.sends[0].1.contains("pub fn value()"));
    assert!(backend.sends[1].1.contains("writes: yes"));
    assert!(backend.sends[1].1.contains("target"));
}

#[test]
fn orchestrator_protocol_applies_agent_patch_and_routes_messages_between_agents() {
    let root = temp_git_repo("protocol-patch-route");
    fs::write(root.join("lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let source = AgentId::new("user-1").unwrap();
    let target = AgentId::new("user-2").unwrap();

    let events = orchestrator
        .handle_agent_message(
            &source,
            "parser",
            "\
@work-leaf patch return parsed value
diff --git a/lib.rs b/lib.rs
--- a/lib.rs
+++ b/lib.rs
@@ -1 +1 @@
-pub fn value() -> u8 { 1 }
+pub fn value() -> u8 { 2 }
@work-leaf end
@work-leaf send user-2 please review my parser patch",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(
        fs::read_to_string(root.join("lib.rs")).unwrap(),
        "pub fn value() -> u8 { 2 }\n"
    );
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::PatchApplied {
                agent_id,
                files,
                ..
            } if agent_id == &source && files == &vec![PathBuf::from("lib.rs")]
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::MessageRouted { from, to } if from == &source && to == &target
        )
    }));
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0, target);
    assert!(backend.sends[0].1.contains("user-1"));
    assert!(backend.sends[0].1.contains("please review"));

    let message = git_output(&root, ["log", "-1", "--pretty=%B"]);
    assert!(message.contains("Agent-ID: user-1"));
    assert!(message.contains("Feature: parser"));
    assert!(message.contains("Reason: return parsed value"));
}

#[test]
fn orchestrator_protocol_sends_patch_conflicts_back_to_the_agent() {
    let root = temp_git_repo("protocol-patch-conflict");
    fs::write(root.join("README.md"), "actual\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "\
@work-leaf patch update readme
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-expected
+changed
@work-leaf end",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "actual\n"
    );
    assert!(git_output(&root, ["status", "--short"]).is_empty());
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::PatchRejected { agent_id: id, files, .. }
                if id == &agent_id && files == &vec![PathBuf::from("README.md")]
        )
    }));
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0, agent_id);
    assert!(backend.sends[0].1.contains("could not apply your patch"));
    assert!(backend.sends[0].1.contains("README.md"));
}

#[test]
fn orchestrator_protocol_sends_validation_failures_back_to_the_agent() {
    let root = temp_git_repo("protocol-patch-validation");
    fs::write(root.join("README.md"), "actual\n").unwrap();
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\n1. `sh validate.sh`\n",
    )
    .unwrap();
    fs::write(
        root.join("validate.sh"),
        "echo validation failed from orchestrator fixture >&2\nexit 1\n",
    )
    .unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial validation fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "\
@work-leaf patch update readme
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-actual
+changed
@work-leaf end",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "actual\n"
    );
    assert!(git_output(&root, ["status", "--short", "--untracked-files=no"]).is_empty());
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::PatchRejected {
                agent_id: id,
                files,
                diagnostic
            } if id == &agent_id
                && files == &vec![PathBuf::from("README.md")]
                && diagnostic.contains("sh validate.sh")
        )
    }));
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0, agent_id);
    assert!(backend.sends[0].1.contains("repository validation failed"));
    assert!(backend.sends[0].1.contains("sh validate.sh"));
    assert!(
        backend.sends[0]
            .1
            .contains("validation failed from orchestrator fixture")
    );
}

#[test]
fn orchestrator_protocol_sends_malformed_patch_feedback_back_to_the_agent() {
    let root = temp_git_repo("protocol-malformed-patch");
    fs::write(root.join("README.md"), "actual\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "\
@work-leaf patch update readme
README.md should say changed.
@work-leaf end",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "actual\n"
    );
    assert!(git_output(&root, ["status", "--short"]).is_empty());
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::PatchRejected {
                agent_id: id,
                files,
                diagnostic
            } if id == &agent_id
                && files.is_empty()
                && diagnostic.contains("recognizable unified diff file headers")
        )
    }));
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0, agent_id);
    assert!(
        backend.sends[0]
            .1
            .contains("recognizable unified diff file headers")
    );
}

#[derive(Default)]
struct RecordingBackend {
    sends: Vec<(AgentId, String)>,
}

impl AgentBackend for RecordingBackend {
    fn launch(&mut self, _request: work_leaf::AgentLaunch) -> Result<AgentSession, AgentError> {
        unreachable!("protocol tests route through existing agents")
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.sends.push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(MessageRole::Agent, "ok"))
    }
}

fn temp_git_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    git(&root, ["init"]);
    git(&root, ["config", "user.name", "Work Leaf Test"]);
    git(&root, ["config", "user.email", "work-leaf@example.test"]);
    root
}

fn git<const N: usize>(root: &Path, args: [&str; N]) {
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

fn git_output<const N: usize>(root: &Path, args: [&str; N]) -> String {
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
