use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use work_leaf::{AgentId, FileLockTable, GitPatcher, PatchError, PatchRequest};

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

fn git_repo(name: &str) -> PathBuf {
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
