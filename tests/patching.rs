use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentSession, ChatMessage, FileLockTable, GitPatcher,
    MessageRole, PatchCoordinator, PatchError, PatchRequest,
};

mod temp_cleanup;

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
    assert!(message.contains("validated with `git apply --recount --check`"));
}

#[test]
fn patcher_applies_structured_edit_patch_without_hunk_line_numbers() {
    let root = git_repo("patch-applies-structured-edit");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "\
pub fn value() -> u8 { 1 }
pub fn label() -> &'static str { \"old\" }
",
    )
    .unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial library fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let outcome = patcher
        .apply_edit(PatchRequest::new(
            AgentId::new("chat-1").unwrap(),
            "parser",
            "return the parsed label",
            "\
*** Begin Patch
*** Update File: src/lib.rs
@@
 pub fn value() -> u8 { 1 }
-pub fn label() -> &'static str { \"old\" }
+pub fn label() -> &'static str { \"new\" }
*** End Patch
",
        ))
        .unwrap();

    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "\
pub fn value() -> u8 { 1 }
pub fn label() -> &'static str { \"new\" }
"
    );
    assert_eq!(outcome.files, vec![PathBuf::from("src/lib.rs")]);
    assert_eq!(outcome.commit.len(), 40);

    let message = git_output(&root, ["log", "-1", "--pretty=%B"]);
    assert!(message.starts_with("UPDATE apply parser patch from chat-1"));
    assert!(message.contains("matched exact edit blocks"));
}

#[test]
fn patcher_rejects_ambiguous_structured_edit_without_writing() {
    let root = git_repo("patch-structured-edit-ambiguous");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "\
pub fn one() -> u8 {
    1
}

pub fn two() -> u8 {
    1
}
",
    )
    .unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial library fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let error = patcher
        .apply_edit(PatchRequest::new(
            AgentId::new("chat-1").unwrap(),
            "parser",
            "return a new value",
            "\
*** Begin Patch
*** Update File: src/lib.rs
@@
-    1
+    2
*** End Patch
",
        ))
        .unwrap_err();

    match error {
        PatchError::Conflict { files, diagnostic } => {
            assert_eq!(files, vec![PathBuf::from("src/lib.rs")]);
            assert!(diagnostic.contains("ambiguous"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "\
pub fn one() -> u8 {
    1
}

pub fn two() -> u8 {
    1
}
"
    );
    assert!(git_output(&root, ["status", "--short"]).is_empty());
}

#[test]
fn patcher_applies_unified_diff_with_incorrect_hunk_counts() {
    let root = git_repo("patch-recount-hunks");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial library fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let outcome = patcher
        .apply(PatchRequest::new(
            AgentId::new("chat-1").unwrap(),
            "parser",
            "return the recounted value",
            "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,99 +1,99 @@
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
fn patcher_rejects_multi_file_conflict_without_partially_applying_clean_hunks() {
    let root = git_repo("patch-multi-file-conflict");
    fs::write(root.join("a.txt"), "actual a\n").unwrap();
    fs::write(root.join("b.txt"), "actual b\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial multi file fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let error = patcher
        .apply(PatchRequest::new(
            AgentId::new("chat-7").unwrap(),
            "docs",
            "update two files atomically",
            "\
diff --git a/a.txt b/a.txt
--- a/a.txt
+++ b/a.txt
@@ -1 +1 @@
-actual a
+changed a
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1 +1 @@
-expected b
+changed b
",
        ))
        .unwrap_err();

    assert!(matches!(error, PatchError::Conflict { .. }));
    assert_eq!(
        fs::read_to_string(root.join("a.txt")).unwrap(),
        "actual a\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("b.txt")).unwrap(),
        "actual b\n"
    );
    assert!(git_output(&root, ["status", "--short"]).is_empty());
}

#[test]
fn patcher_applies_indented_fenced_unified_diff_from_agent_reply() {
    let root = git_repo("patch-indented-fenced-diff");
    fs::write(root.join("README.md"), "actual\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let outcome = patcher
        .apply(PatchRequest::new(
            AgentId::new("chat-5").unwrap(),
            "docs",
            "update readme from fenced diff",
            "\
Here is the patch:

```diff
    diff --git a/README.md b/README.md
    --- a/README.md
    +++ b/README.md
    @@ -1 +1 @@
    -actual
    +changed
```
",
        ))
        .unwrap();

    assert_eq!(outcome.files, vec![PathBuf::from("README.md")]);
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "changed\n"
    );
}

#[test]
fn patcher_applies_patch_without_running_project_required_checks() {
    let root = git_repo("patch-no-required-check-run");
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
    let outcome = patcher
        .apply(PatchRequest::new(
            AgentId::new("chat-3").unwrap(),
            "docs",
            "update readme after the agent handled checks",
            "\
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-actual
+changed
",
        ))
        .unwrap();

    assert_eq!(outcome.files, vec![PathBuf::from("README.md")]);
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "changed\n"
    );
    assert_eq!(
        git_output(&root, ["log", "-1", "--pretty=%s"]),
        "UPDATE apply docs patch from chat-3"
    );
    assert!(git_output(&root, ["status", "--short", "--untracked-files=no"]).is_empty());
}

#[test]
fn patcher_does_not_run_language_specific_fallback_checks() {
    let root = git_repo("patch-no-language-fallback-check");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"patch-no-language-fallback-check\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
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
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial rust test fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let outcome = patcher
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
        .unwrap();

    assert_eq!(outcome.files, vec![PathBuf::from("src/lib.rs")]);
    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "\
pub fn value() -> u8 { 2 }

#[cfg(test)]
mod tests {
    #[test]
    fn value_remains_one() {
        assert_eq!(super::value(), 1);
    }
}
"
    );
    assert!(outcome.commit.len() == 40);
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
fn patch_coordinator_applies_patch_without_running_project_required_checks() {
    let root = git_repo("patch-coordinator-no-required-check-run");
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
    let outcome = coordinator
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
        .unwrap();
    let backend = coordinator.into_backend();

    assert_eq!(outcome.files, vec![PathBuf::from("README.md")]);
    assert!(backend.sends.is_empty());
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "changed\n"
    );
    assert_eq!(
        git_output(&root, ["log", "-1", "--pretty=%s"]),
        "UPDATE apply docs patch from chat-4"
    );
    assert!(git_output(&root, ["status", "--short", "--untracked-files=no"]).is_empty());
}

#[test]
fn patch_coordinator_sends_no_file_header_diagnostics_back_to_agent() {
    let root = git_repo("patch-no-file-header-feedback");
    fs::write(root.join("README.md"), "actual\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial readme fixture"]);

    let patcher = GitPatcher::new(root.clone(), FileLockTable::new(root.clone()));
    let backend = FakeBackend::default();
    let mut coordinator = PatchCoordinator::new(patcher, backend);
    let error = coordinator
        .submit(PatchRequest::new(
            AgentId::new("chat-6").unwrap(),
            "docs",
            "malformed patch body",
            "\
README.md should contain changed instead of actual.
",
        ))
        .unwrap_err();
    let backend = coordinator.into_backend();

    assert!(matches!(error, PatchError::NoFiles));
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0.as_str(), "chat-6");
    assert!(
        backend.sends[0]
            .1
            .contains("recognizable unified diff file headers")
    );
    assert!(backend.sends[0].1.contains("@work-leaf patch <reason>"));
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "actual\n"
    );
    assert!(git_output(&root, ["status", "--short"]).is_empty());
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
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    temp_cleanup::register(&root);
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
