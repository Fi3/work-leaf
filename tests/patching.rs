use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentSession, ChatMessage, FileLockTable, GitPatcher,
    MessageRole, PatchCoordinator, PatchError, PatchRequest,
};

#[test]
fn patcher_applies_unified_diff_and_creates_metadata_commit() {
    let root = git_repo("patch-applies");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial library fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let outcome = patcher
        .apply(PatchRequest::new(
            AgentId::new("chat-1").unwrap(),
            "parser",
            "return the parsed value",
            "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1 +1 @@
-pub fn value() -> u8 { 1 }
+pub fn value() -> u8 { 2 }
",
        ))
        .unwrap();

    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "pub fn value() -> u8 { 2 }\n"
    );
    assert_eq!(outcome.files, vec![PathBuf::from("src/lib.rs")]);
    assert_eq!(outcome.commit.len(), 40);

    let message = git_output(&root, ["log", "-1", "--pretty=%B"]);
    assert!(message.starts_with("UPDATE apply parser patch from chat-1"));
    assert!(message.contains("Agent-ID: chat-1"));
    assert!(message.contains("Feature: parser"));
    assert!(message.contains("Reason: return the parsed value"));
    assert!(message.contains("Context:"));
    assert!(message.contains("The orchestrator applied this provisional patch for chat-1"));
    assert!(message.contains("src/lib.rs"));
    assert!(message.contains("validated with `git apply --check`"));
}

#[test]
fn patcher_rejects_conflicting_patch_and_keeps_worktree_clean() {
    let root = git_repo("patch-conflict");
    fs::write(root.join("README.md"), "actual\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let error = patcher
        .apply(PatchRequest::new(
            AgentId::new("chat-2").unwrap(),
            "docs",
            "replace expected text",
            "\
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-expected
+changed
",
        ))
        .unwrap_err();

    match error {
        PatchError::Conflict { files, diagnostic } => {
            assert_eq!(files, vec![PathBuf::from("README.md")]);
            assert!(diagnostic.contains("patch failed") || diagnostic.contains("does not apply"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "actual\n"
    );
    assert!(git_output(&root, ["status", "--short"]).is_empty());
}

#[test]
fn patcher_rejects_patch_when_required_check_fails_and_does_not_commit() {
    let root = git_repo("patch-validation-fails");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"patch-validation-fails\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    let target_dir = std::env::temp_dir().join(format!(
        "work-leaf-patch-validation-target-{}",
        std::process::id()
    ));
    fs::write(
        root.join("AGENTS.md"),
        format!(
            "## Required Checks\n1. `cargo check --locked --target-dir {}`\n",
            target_dir.display()
        ),
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    let lockfile = Command::new("cargo")
        .current_dir(&root)
        .args(["generate-lockfile"])
        .output()
        .unwrap();
    assert!(
        lockfile.status.success(),
        "cargo generate-lockfile failed: {}\n{}",
        String::from_utf8_lossy(&lockfile.stdout),
        String::from_utf8_lossy(&lockfile.stderr)
    );
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial rust fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let error = patcher
        .apply(PatchRequest::new(
            AgentId::new("chat-3").unwrap(),
            "parser",
            "add a broken function",
            "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1 +1,2 @@
 pub fn value() -> u8 { 1 }
+pub fn broken(
",
        ))
        .unwrap_err();

    match error {
        PatchError::ValidationFailed {
            files,
            command,
            diagnostic,
        } => {
            assert_eq!(files, vec![PathBuf::from("src/lib.rs")]);
            assert!(command.starts_with("cargo check --locked --target-dir "));
            assert!(diagnostic.contains("error"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(
        git_output(&root, ["log", "-1", "--pretty=%s"]),
        "ADD initial rust fixture"
    );
    assert!(git_output(&root, ["status", "--short", "--untracked-files=no"]).is_empty());
}

#[test]
fn patcher_rejects_patch_when_forced_cargo_test_fails() {
    let root = git_repo("patch-forced-cargo-test-fails");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"patch-forced-cargo-test-fails\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    let target_dir = std::env::temp_dir().join(format!(
        "work-leaf-patch-forced-check-target-{}",
        std::process::id()
    ));
    fs::write(
        root.join("AGENTS.md"),
        format!(
            "## Required Checks\n1. `cargo check --locked --target-dir {}`\n",
            target_dir.display()
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "\
pub fn value() -> u8 { 1 }

#[cfg(test)]
mod tests {
    #[test]
    fn value_remains_one() {
        assert_eq!(super::value(), 1);
    }
}
",
    )
    .unwrap();
    let lockfile = Command::new("cargo")
        .current_dir(&root)
        .args(["generate-lockfile"])
        .output()
        .unwrap();
    assert!(
        lockfile.status.success(),
        "cargo generate-lockfile failed: {}\n{}",
        String::from_utf8_lossy(&lockfile.stdout),
        String::from_utf8_lossy(&lockfile.stderr)
    );
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial rust test fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let error = patcher
        .apply(PatchRequest::new(
            AgentId::new("chat-5").unwrap(),
            "parser",
            "change a value used by tests",
            "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,4 +1,4 @@
-pub fn value() -> u8 { 1 }
+pub fn value() -> u8 { 2 }

 #[cfg(test)]
 mod tests {
",
        ))
        .unwrap_err();

    match error {
        PatchError::ValidationFailed {
            files,
            command,
            diagnostic,
        } => {
            assert_eq!(files, vec![PathBuf::from("src/lib.rs")]);
            assert_eq!(command, "cargo test --all-targets --all-features");
            assert!(diagnostic.contains("value_remains_one"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "\
pub fn value() -> u8 { 1 }

#[cfg(test)]
mod tests {
    #[test]
    fn value_remains_one() {
        assert_eq!(super::value(), 1);
    }
}
"
    );
    assert_eq!(
        git_output(&root, ["log", "-1", "--pretty=%s"]),
        "ADD initial rust test fixture"
    );
    assert!(git_output(&root, ["status", "--short", "--untracked-files=no"]).is_empty());
}

#[test]
fn patch_coordinator_sends_conflict_diagnostics_back_to_agent() {
    let root = git_repo("patch-conflict-feedback");
    fs::write(root.join("README.md"), "actual\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);

    let patcher = GitPatcher::new(
        root,
        FileLockTable::new(git_repo_root("patch-conflict-feedback")),
    );
    let backend = FakeBackend::default();
    let mut coordinator = PatchCoordinator::new(patcher, backend);
    let error = coordinator
        .submit(PatchRequest::new(
            AgentId::new("chat-2").unwrap(),
            "docs",
            "replace expected text",
            "\
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-expected
+changed
",
        ))
        .unwrap_err();
    let backend = coordinator.into_backend();

    assert!(matches!(error, PatchError::Conflict { .. }));
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0.as_str(), "chat-2");
    assert!(backend.sends[0].1.contains("could not apply your patch"));
    assert!(backend.sends[0].1.contains("README.md"));
}

#[test]
fn patch_coordinator_sends_validation_diagnostics_back_to_agent() {
    let root = git_repo("patch-validation-feedback");
    fs::write(root.join("README.md"), "actual\n").unwrap();
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\n1. `sh validate.sh`\n",
    )
    .unwrap();
    fs::write(
        root.join("validate.sh"),
        "echo validation failed from fixture >&2\nexit 1\n",
    )
    .unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial validation fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let backend = FakeBackend::default();
    let mut coordinator = PatchCoordinator::new(patcher, backend);
    let error = coordinator
        .submit(PatchRequest::new(
            AgentId::new("chat-4").unwrap(),
            "docs",
            "update readme",
            "\
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-actual
+changed
",
        ))
        .unwrap_err();
    let backend = coordinator.into_backend();

    assert!(matches!(error, PatchError::ValidationFailed { .. }));
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0.as_str(), "chat-4");
    assert!(backend.sends[0].1.contains("repository validation failed"));
    assert!(backend.sends[0].1.contains("sh validate.sh"));
    assert!(
        backend.sends[0]
            .1
            .contains("validation failed from fixture")
    );
    assert_eq!(
        git_output(&root, ["log", "-1", "--pretty=%s"]),
        "ADD initial validation fixture"
    );
    assert!(git_output(&root, ["status", "--short", "--untracked-files=no"]).is_empty());
}

fn git_repo(name: &str) -> PathBuf {
    let root = git_repo_root(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    git(&root, ["init"]);
    git(&root, ["config", "user.name", "Work Leaf Test"]);
    git(&root, ["config", "user.email", "work-leaf@example.test"]);
    root
}

fn git_repo_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()))
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

#[derive(Default)]
struct FakeBackend {
    sends: Vec<(AgentId, String)>,
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, _request: work_leaf::AgentLaunch) -> Result<AgentSession, AgentError> {
        unreachable!("patch coordinator does not launch agents")
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.sends.push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(MessageRole::Agent, "will fix patch"))
    }
}
