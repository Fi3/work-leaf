use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentOrchestrator, AgentSession, ChatMessage, MessageRole,
    OrchestratorEvent,
};

mod temp_cleanup;

static PROCESS_ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvVarGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &Path) -> Self {
        let previous = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            match self.previous.take() {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}

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
fn orchestrator_protocol_batches_consecutive_file_reads_into_one_agent_follow_up() {
    let root = temp_git_repo("protocol-batched-reads");
    fs::write(root.join("README.md"), "readme\n").unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root, backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "parser",
            "@work-leaf read README.md\n@work-leaf read src/lib.rs",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(
        events,
        vec![OrchestratorEvent::FileTextSent {
            agent_id: agent_id.clone(),
            paths: vec![PathBuf::from("README.md"), PathBuf::from("src/lib.rs")]
        }]
    );
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0, agent_id);
    assert!(backend.sends[0].1.contains("--- README.md ---"));
    assert!(backend.sends[0].1.contains("readme"));
    assert!(backend.sends[0].1.contains("--- src/lib.rs ---"));
    assert!(backend.sends[0].1.contains("pub fn value()"));
}

#[test]
fn orchestrator_protocol_skips_unchanged_repeat_file_reads_by_digest() {
    let root = temp_git_repo("protocol-repeat-read-unchanged");
    fs::write(root.join("README.md"), "before\n").unwrap();
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root, backend);
    let agent_id = AgentId::new("user-1").unwrap();

    orchestrator
        .handle_agent_message(&agent_id, "docs", "@work-leaf read README.md")
        .unwrap();
    orchestrator
        .handle_agent_message(&agent_id, "docs", "@work-leaf read README.md")
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(backend.sends.len(), 2);
    assert!(backend.sends[0].1.contains("--- README.md ---"));
    let repeat_prompt = &backend.sends[1].1;
    assert!(repeat_prompt.contains("Repeated file reads unchanged"));
    assert!(repeat_prompt.contains("README.md (fnv64:"));
    assert!(
        !repeat_prompt.contains("--- README.md ---\nbefore"),
        "unchanged repeat reads should not resend exact file text: {repeat_prompt}"
    );
}

#[test]
fn orchestrator_protocol_sends_diff_for_changed_repeat_file_reads() {
    let root = temp_git_repo("protocol-repeat-read-changed");
    fs::write(root.join("README.md"), "before\n").unwrap();
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    orchestrator
        .handle_agent_message(&agent_id, "docs", "@work-leaf read README.md")
        .unwrap();
    fs::write(root.join("README.md"), "after\n").unwrap();
    orchestrator
        .handle_agent_message(&agent_id, "docs", "@work-leaf read README.md")
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(backend.sends.len(), 2);
    let repeat_prompt = &backend.sends[1].1;
    assert!(repeat_prompt.contains("Repeated file reads with changes"));
    assert!(repeat_prompt.contains("current digest: fnv64:"));
    assert!(repeat_prompt.contains("previous digest: fnv64:"));
    assert!(repeat_prompt.contains("-before"));
    assert!(repeat_prompt.contains("+after"));
    assert!(
        !repeat_prompt.contains("--- README.md ---\nafter"),
        "changed repeat reads should send a diff instead of full file text: {repeat_prompt}"
    );
}

#[test]
fn orchestrator_protocol_force_repeat_read_still_sends_diff() {
    let root = temp_git_repo("protocol-repeat-read-force");
    fs::write(root.join("README.md"), "before\n").unwrap();
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    orchestrator
        .handle_agent_message(&agent_id, "docs", "@work-leaf read README.md")
        .unwrap();
    fs::write(root.join("README.md"), "after\n").unwrap();
    orchestrator
        .handle_agent_message(&agent_id, "docs", "@work-leaf read --force README.md")
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(backend.sends.len(), 2);
    let force_prompt = &backend.sends[1].1;
    assert!(force_prompt.contains("Repeated file reads with changes"));
    assert!(force_prompt.contains("-before"));
    assert!(force_prompt.contains("+after"));
    assert!(
        !force_prompt.contains("--- README.md ---\nafter"),
        "forced repeat reads should still send a diff instead of full file text: {force_prompt}"
    );
}

#[test]
fn orchestrator_protocol_spills_large_file_reads_to_context_bundle() {
    let _env_guard = PROCESS_ENV_LOCK.lock().unwrap();
    let root = temp_git_repo("protocol-large-read-bundle");
    let bundle_root = temp_dir("protocol-context-bundle-root");
    let _bundle_env = EnvVarGuard::set("WORK_LEAF_CONTEXT_BUNDLE_DIR", &bundle_root);
    fs::write(
        root.join("large.txt"),
        numbered_lines("large-context", 3000),
    )
    .unwrap();
    let backend = SharedRecordingBackend::default();
    let sends = backend.sends.clone();
    let mut orchestrator = AgentOrchestrator::new(root, backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let events = orchestrator
        .handle_agent_message(&agent_id, "parser", "@work-leaf read large.txt")
        .unwrap();

    assert_eq!(
        events,
        vec![OrchestratorEvent::FileTextSent {
            agent_id: agent_id.clone(),
            paths: vec![PathBuf::from("large.txt")]
        }]
    );
    let sends = sends.lock().unwrap();
    assert_eq!(sends.len(), 1);
    let prompt = &sends[0].1;
    assert!(prompt.contains("orchestrator context bundle"), "{prompt}");
    assert!(prompt.contains("large.txt"));
    assert!(prompt.contains("fnv64:"));
    assert!(
        !prompt.contains("large-context-2999"),
        "large file text should not be copied into the chat prompt"
    );

    let bundle_path = PathBuf::from(
        prompt
            .lines()
            .find_map(|line| line.strip_prefix("Context bundle: "))
            .expect("bundle path should be present"),
    );
    assert!(
        bundle_path.starts_with(&bundle_root),
        "bundle should be written under the configured bench-owned temp root: {bundle_path:?}"
    );
    let bundle = fs::read_to_string(&bundle_path).unwrap();
    assert!(bundle.contains("----- BEGIN FILE large.txt -----"));
    assert!(bundle.contains("large-context-2999"));
    assert!(bundle.contains("----- END FILE large.txt -----"));
    drop(sends);
    drop(orchestrator);
    let bundle_dir = bundle_path.parent().unwrap().to_path_buf();
    assert!(
        !bundle_path.exists(),
        "context bundle should be removed when the orchestrator is dropped"
    );
    assert!(
        !bundle_dir.exists(),
        "context bundle directory should be removed when the orchestrator is dropped"
    );
}

#[test]
fn orchestrator_protocol_runs_command_under_requested_write_locks() {
    let root = temp_git_repo("protocol-locked-command-run");
    fs::write(root.join("README.md"), "before\n").unwrap();
    fs::write(
        root.join("format.sh"),
        "printf 'format stdout\\n'\nprintf 'after\\n' > README.md\n",
    )
    .unwrap();
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "@work-leaf locks run README.md -- sh format.sh",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after\n"
    );
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::CommandRun {
                agent_id: id,
                command,
                status,
                locked_paths,
                ..
            } if id == &agent_id
                && command == "sh format.sh"
                && status == &Some(0)
                && locked_paths == &vec![PathBuf::from("README.md")]
        )
    }));
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0, agent_id);
    assert!(backend.sends[0].1.contains("work-leaf command result"));
    assert!(backend.sends[0].1.contains("command: sh format.sh"));
    assert!(backend.sends[0].1.contains("status: 0"));
    assert!(backend.sends[0].1.contains("locked paths: README.md"));
    assert!(backend.sends[0].1.contains("format stdout"));
}

#[test]
fn orchestrator_protocol_scopes_locked_command_tmpdir_without_daemon_tmpdir() {
    let _env_guard = PROCESS_ENV_LOCK.lock().unwrap();
    let root = temp_git_repo("protocol-command-tmpdir");
    let command_tmp = root.join("agent-tmp");
    fs::create_dir_all(&command_tmp).unwrap();
    fs::write(root.join("README.md"), "tmpdir fixture\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial command tmp fixture"]);
    let _tmp_env = EnvVarGuard::set("WORK_LEAF_COMMAND_TMPDIR", &command_tmp);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let result = orchestrator.handle_agent_message(
        &agent_id,
        "tmp",
        "@work-leaf locks run tmpdir.txt -- sh -c 'printf \"%s\" \"$TMPDIR\" > tmpdir.txt'",
    );

    result.unwrap();

    assert_eq!(
        fs::read_to_string(root.join("tmpdir.txt")).unwrap(),
        command_tmp.to_string_lossy()
    );
}

#[test]
fn orchestrator_protocol_blocks_done_until_command_changes_are_committed() {
    let root = temp_git_repo("protocol-command-dirty-blocks-done");
    fs::write(root.join("README.md"), "before\n").unwrap();
    fs::write(root.join("format.sh"), "printf 'after\\n' > README.md\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial formatting fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "@work-leaf locks run README.md -- sh format.sh\n@work-leaf done",
        )
        .unwrap();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after\n"
    );
    assert!(git_output(&root, ["status", "--short"]).contains("README.md"));
    assert!(events.iter().any(|event| matches!(
        event,
        OrchestratorEvent::CommandRun {
            agent_id: id,
            command,
            status,
            ..
        } if id == &agent_id && command == "sh format.sh" && status == &Some(0)
    )));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, OrchestratorEvent::AgentDone { .. }))
    );

    let backend = orchestrator.into_backend();
    assert_eq!(backend.sends.len(), 2);
    assert!(backend.sends[1].1.contains("tracked working-tree changes"));
    assert!(backend.sends[1].1.contains("README.md"));
    assert!(
        backend.sends[1]
            .1
            .contains("diff --git a/README.md b/README.md")
    );
    assert!(backend.sends[1].1.contains("@work-leaf patch <reason>"));
}

#[test]
fn orchestrator_protocol_commits_already_applied_command_diff_before_done() {
    let root = temp_git_repo("protocol-command-dirty-already-applied-patch");
    fs::write(root.join("README.md"), "before\n").unwrap();
    fs::write(root.join("format.sh"), "printf 'after\\n' > README.md\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial formatting fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "@work-leaf locks run README.md -- sh format.sh\n@work-leaf done",
        )
        .unwrap();
    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "\
@work-leaf patch include formatter output
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-before
+after
@work-leaf end
@work-leaf done",
        )
        .unwrap();

    assert!(git_output(&root, ["status", "--short"]).is_empty());
    assert!(events.iter().any(|event| matches!(
        event,
        OrchestratorEvent::PatchApplied {
            agent_id: id,
            files,
            ..
        } if id == &agent_id && files == &vec![PathBuf::from("README.md")]
    )));
    assert!(events.iter().any(
        |event| matches!(event, OrchestratorEvent::AgentDone { agent_id: id } if id == &agent_id)
    ));
}

#[test]
fn orchestrator_protocol_accepts_trailing_whitespace_on_done_and_end_directives() {
    let root = temp_git_repo("protocol-trailing-whitespace-done");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "README.md"]);
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
-before
+after
@work-leaf end \t
@work-leaf done \t",
        )
        .unwrap();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after\n"
    );
    assert!(events.iter().any(|event| matches!(
        event,
        OrchestratorEvent::PatchApplied {
            agent_id: id,
            files,
            ..
        } if id == &agent_id && files == &vec![PathBuf::from("README.md")]
    )));
    assert!(events.iter().any(
        |event| matches!(event, OrchestratorEvent::AgentDone { agent_id: id } if id == &agent_id)
    ));
}

#[test]
fn orchestrator_protocol_times_out_long_locked_command_runs() {
    let root = temp_git_repo("protocol-locked-command-timeout");
    fs::write(root.join("README.md"), "before\n").unwrap();
    fs::write(
        root.join("slow.sh"),
        "sleep 1\nprintf 'late\\n' > README.md\n",
    )
    .unwrap();
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend)
        .with_locked_command_timeout(Duration::from_millis(50));
    let agent_id = AgentId::new("user-1").unwrap();

    let start = Instant::now();
    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "@work-leaf locks run README.md -- sh slow.sh",
        )
        .unwrap();
    assert!(
        start.elapsed() < Duration::from_millis(800),
        "locked command should release shortly after timeout"
    );
    let backend = orchestrator.into_backend();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "before\n"
    );
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::CommandRun {
                agent_id: id,
                command,
                status,
                locked_paths,
                ..
            } if id == &agent_id
                && command == "sh slow.sh"
                && status.is_none()
                && locked_paths == &vec![PathBuf::from("README.md")]
        )
    }));
    assert_eq!(backend.sends.len(), 1);
    assert!(backend.sends[0].1.contains("timed out"));
    assert!(backend.sends[0].1.contains("user authorization"));
}

#[test]
fn orchestrator_protocol_returns_failing_command_output_to_agent() {
    let root = temp_git_repo("protocol-failing-command-run");
    fs::write(
        root.join("validate.sh"),
        "echo bad stdout\necho bad stderr >&2\nexit 7\n",
    )
    .unwrap();
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root, backend);
    let agent_id = AgentId::new("user-1").unwrap();

    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "@work-leaf locks run target -- sh validate.sh",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::CommandRun {
                agent_id: id,
                command,
                status,
                locked_paths,
                ..
            } if id == &agent_id
                && command == "sh validate.sh"
                && status == &Some(7)
                && locked_paths == &vec![PathBuf::from("target")]
        )
    }));
    assert_eq!(backend.sends.len(), 1);
    assert!(backend.sends[0].1.contains("status: 7"));
    assert!(backend.sends[0].1.contains("bad stdout"));
    assert!(backend.sends[0].1.contains("bad stderr"));
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
    assert_eq!(backend.sends.len(), 2);
    assert_eq!(backend.sends[0].0, target);
    assert!(backend.sends[0].1.contains("user-1"));
    assert!(backend.sends[0].1.contains("please review"));
    assert_eq!(backend.sends[1].0, source);
    assert!(backend.sends[1].1.contains("work-leaf patch applied"));
    assert!(backend.sends[1].1.contains("do not submit known-red"));
    assert!(backend.sends[1].1.contains("same provisional patch"));
    assert!(backend.sends[1].1.contains("@work-leaf done"));

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
    assert!(backend.sends[0].1.contains("work-leaf file text"));
    assert!(backend.sends[0].1.contains("--- README.md ---"));
    assert!(backend.sends[0].1.contains("actual"));
}

#[test]
fn orchestrator_protocol_reports_already_applied_patches_without_rebase_refresh() {
    let root = temp_git_repo("protocol-already-applied-patch");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        ["commit", "-m", "ADD initial already-applied fixture"],
    );
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let first = AgentId::new("user-1").unwrap();
    let second = AgentId::new("user-2").unwrap();
    let patch = "\
@work-leaf patch update readme
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-before
+after
@work-leaf end";

    orchestrator
        .handle_agent_message(&first, "docs", patch)
        .unwrap();
    let events = orchestrator
        .handle_agent_message(&second, "docs", patch)
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after\n"
    );
    assert_eq!(git_output(&root, ["rev-list", "--count", "HEAD"]), "2");
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::PatchRejected {
                agent_id,
                files,
                diagnostic
            } if agent_id == &second
                && files == &vec![PathBuf::from("README.md")]
                && diagnostic.contains("already applied")
        )
    }));
    assert_eq!(backend.sends.len(), 2);
    let prompt = &backend.sends[1].1;
    assert!(prompt.contains("work-leaf patch already applied"));
    assert!(prompt.contains("Do not resend the same patch"));
    assert!(
        !prompt.contains("work-leaf file refresh"),
        "already-applied prompt should not send a rebase refresh: {prompt}"
    );
}

#[test]
fn orchestrator_protocol_sends_compact_patch_conflict_refresh_for_previously_read_files() {
    let root = temp_git_repo("protocol-compact-patch-conflict-refresh");
    let original = numbered_lines("before", 700);
    let current = original.replace("before-0350\n", "current-0350\n");
    fs::write(root.join("README.md"), &original).unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial large readme fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let agent_id = AgentId::new("user-1").unwrap();

    orchestrator
        .handle_agent_message(&agent_id, "docs", "@work-leaf read README.md")
        .unwrap();
    fs::write(root.join("README.md"), &current).unwrap();
    git(&root, ["add", "README.md"]);
    git(&root, ["commit", "-m", "UPDATE readme from another patch"]);

    let events = orchestrator
        .handle_agent_message(
            &agent_id,
            "docs",
            "\
@work-leaf patch edit stale readme
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -348,7 +348,7 @@
 before-0347
 before-0348
 before-0349
-before-0350
+agent-0350
 before-0351
 before-0352
 before-0353
@work-leaf end",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert!(events.iter().any(|event| {
        matches!(event, OrchestratorEvent::PatchRejected { agent_id: id, files, .. }
            if id == &agent_id && files == &vec![PathBuf::from("README.md")])
    }));
    assert_eq!(backend.sends.len(), 2);
    let conflict_prompt = &backend.sends[1].1;
    assert!(conflict_prompt.contains("work-leaf file refresh"));
    assert!(conflict_prompt.contains("-before-0350"));
    assert!(conflict_prompt.contains("+current-0350"));
    assert!(
        conflict_prompt.len() < original.len() / 4,
        "conflict prompt should be compact, prompt={} original={}",
        conflict_prompt.len(),
        original.len()
    );
    assert!(
        !conflict_prompt.contains("before-0000\nbefore-0001\nbefore-0002"),
        "compact refresh should not inline the whole file"
    );
}

#[test]
fn orchestrator_protocol_blocks_other_agent_owned_test_commands() {
    let root = temp_git_repo("protocol-other-agent-test-command");
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial ownership fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let owner = AgentId::new("user-1").unwrap();
    let other = AgentId::new("user-2").unwrap();

    orchestrator
        .handle_agent_message(
            &owner,
            "parser",
            "\
@work-leaf patch add focused test
diff --git a/tests/parser.test b/tests/parser.test
new file mode 100644
--- /dev/null
+++ b/tests/parser.test
@@ -0,0 +1 @@
+focused parser test
@work-leaf end",
        )
        .unwrap();
    let events = orchestrator
        .handle_agent_message(
            &other,
            "slash",
            "@work-leaf locks run tests/parser.test -- sh -c 'printf ran > blocked.txt'",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert!(
        !root.join("blocked.txt").exists(),
        "other-agent focused test command should not run"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, OrchestratorEvent::CommandRun { .. })),
        "blocked ownership commands should not be reported as executed"
    );
    assert_eq!(backend.sends.len(), 2);
    let prompt = &backend.sends[1].1;
    assert!(prompt.contains("work-leaf command blocked by patch ownership"));
    assert!(prompt.contains("tests/parser.test"));
    assert!(prompt.contains("user-1"));
    assert!(prompt.contains("Do not run another patch agent's focused tests"));
}

#[test]
fn orchestrator_protocol_allows_broad_validation_over_other_agent_test_dirs() {
    let root = temp_git_repo("protocol-other-agent-broad-validation");
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial ownership fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let owner = AgentId::new("user-1").unwrap();
    let other = AgentId::new("user-2").unwrap();

    orchestrator
        .handle_agent_message(
            &owner,
            "parser",
            "\
@work-leaf patch add focused test
diff --git a/tests/parser.test b/tests/parser.test
new file mode 100644
--- /dev/null
+++ b/tests/parser.test
@@ -0,0 +1 @@
+focused parser test
@work-leaf end",
        )
        .unwrap();
    let events = orchestrator
        .handle_agent_message(
            &other,
            "slash",
            "@work-leaf locks run tests target -- cargo test",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert!(
        events
            .iter()
            .any(|event| matches!(event, OrchestratorEvent::CommandRun { .. })),
        "broad validation commands should be allowed even when their broad lock names a test directory"
    );
    assert_eq!(backend.sends.len(), 2);
    let prompt = &backend.sends[1].1;
    assert!(prompt.contains("work-leaf command result"));
    assert!(!prompt.contains("work-leaf command blocked by patch ownership"));
}

#[test]
fn orchestrator_protocol_blocks_broad_test_locks_for_repo_writing_commands() {
    let root = temp_git_repo("protocol-other-agent-root-writing-command");
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial ownership fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let owner = AgentId::new("user-1").unwrap();
    let other = AgentId::new("user-2").unwrap();

    orchestrator
        .handle_agent_message(
            &owner,
            "parser",
            "\
@work-leaf patch add focused test
diff --git a/tests/parser.test b/tests/parser.test
new file mode 100644
--- /dev/null
+++ b/tests/parser.test
@@ -0,0 +1 @@
+focused parser test
@work-leaf end",
        )
        .unwrap();
    let events = orchestrator
        .handle_agent_message(&other, "slash", "@work-leaf locks run tests -- cargo fmt")
        .unwrap();
    let backend = orchestrator.into_backend();

    assert!(
        !events
            .iter()
            .any(|event| matches!(event, OrchestratorEvent::CommandRun { .. })),
        "commands that may write the repo root should not run through another agent's test directory"
    );
    assert_eq!(backend.sends.len(), 2);
    let prompt = &backend.sends[1].1;
    assert!(prompt.contains("work-leaf command blocked by patch ownership"));
}

#[test]
fn orchestrator_protocol_applies_patch_without_running_project_required_checks() {
    let root = temp_git_repo("protocol-no-required-check-run");
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
        "changed\n"
    );
    assert!(git_output(&root, ["status", "--short", "--untracked-files=no"]).is_empty());
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::PatchApplied {
                agent_id: id,
                feature,
                reason,
                commit,
                files,
            } if id == &agent_id
                && feature == "docs"
                && reason == "update readme"
                && commit.len() == 40
                && files == &vec![PathBuf::from("README.md")]
        )
    }));
    assert_eq!(backend.sends.len(), 1);
    assert_eq!(backend.sends[0].0, agent_id);
    assert!(backend.sends[0].1.contains("work-leaf patch applied"));
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

#[test]
fn orchestrator_protocol_proactively_sends_updated_files_to_stale_readers() {
    let root = temp_git_repo("protocol-stale-reader-update");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial stale reader fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let reader = AgentId::new("user-1").unwrap();
    let patcher = AgentId::new("user-2").unwrap();

    orchestrator
        .handle_agent_message(&reader, "docs", "@work-leaf read README.md")
        .unwrap();
    let events = orchestrator
        .handle_agent_message(
            &patcher,
            "docs",
            "\
@work-leaf patch update readme
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-before
+after
@work-leaf end",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "after\n"
    );
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::PatchApplied { agent_id, files, .. }
                if agent_id == &patcher && files == &vec![PathBuf::from("README.md")]
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::FileUpdateSent { agent_id, paths }
                if agent_id == &reader && paths == &vec![PathBuf::from("README.md")]
        )
    }));
    assert_eq!(backend.sends.len(), 3);
    assert_eq!(backend.sends[0].0, reader);
    assert!(backend.sends[0].1.contains("before"));
    assert_eq!(backend.sends[1].0, AgentId::new("user-1").unwrap());
    assert!(backend.sends[1].1.contains("work-leaf file update"));
    assert!(backend.sends[1].1.contains("README.md"));
    assert!(backend.sends[1].1.contains("after"));
    assert_eq!(backend.sends[2].0, patcher);
    assert!(backend.sends[2].1.contains("work-leaf patch applied"));
}

#[test]
fn orchestrator_protocol_sends_compact_stale_updates_to_other_agents() {
    let root = temp_git_repo("protocol-compact-stale-reader-update");
    let original = numbered_lines("before", 700);
    fs::write(root.join("README.md"), &original).unwrap();
    git(&root, ["add", "."]);
    git(
        &root,
        ["commit", "-m", "ADD initial large stale reader fixture"],
    );
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend);
    let reader = AgentId::new("user-1").unwrap();
    let patcher = AgentId::new("user-2").unwrap();

    orchestrator
        .handle_agent_message(&reader, "docs", "@work-leaf read README.md")
        .unwrap();
    let events = orchestrator
        .handle_agent_message(
            &patcher,
            "docs",
            "\
@work-leaf patch update one large readme line
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -348,7 +348,7 @@
 before-0347
 before-0348
 before-0349
-before-0350
+after-0350
 before-0351
 before-0352
 before-0353
@work-leaf end",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert!(events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::FileUpdateSent { agent_id, paths }
                if agent_id == &reader && paths == &vec![PathBuf::from("README.md")]
        )
    }));
    assert_eq!(backend.sends.len(), 3);
    let update_prompt = &backend.sends[1].1;
    assert!(update_prompt.contains("work-leaf file refresh"));
    assert!(update_prompt.contains("-before-0350"));
    assert!(update_prompt.contains("+after-0350"));
    assert!(
        update_prompt.len() < original.len() / 4,
        "stale update should be compact, prompt={} original={}",
        update_prompt.len(),
        original.len()
    );
    assert!(
        !update_prompt.contains("before-0000\nbefore-0001\nbefore-0002"),
        "compact refresh should not inline the whole file"
    );
}

#[test]
fn orchestrator_protocol_does_not_update_readers_after_done() {
    let root = temp_git_repo("protocol-stale-reader-done");
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(&root, ["add", "."]);
    git(&root, ["commit", "-m", "ADD initial done fixture"]);
    let backend = RecordingBackend::default();
    let mut orchestrator = AgentOrchestrator::new(root, backend);
    let reader = AgentId::new("user-1").unwrap();
    let patcher = AgentId::new("user-2").unwrap();

    orchestrator
        .handle_agent_message(&reader, "docs", "@work-leaf read README.md")
        .unwrap();
    orchestrator
        .handle_agent_message(&reader, "docs", "@work-leaf done")
        .unwrap();
    let events = orchestrator
        .handle_agent_message(
            &patcher,
            "docs",
            "\
@work-leaf patch update readme
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-before
+after
@work-leaf end",
        )
        .unwrap();
    let backend = orchestrator.into_backend();

    assert!(!events.iter().any(|event| {
        matches!(
            event,
            OrchestratorEvent::FileUpdateSent { agent_id, .. } if agent_id == &reader
        )
    }));
    assert_eq!(backend.sends.len(), 2);
    assert_eq!(backend.sends[1].0, patcher);
    assert!(backend.sends[1].1.contains("work-leaf patch applied"));
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

#[derive(Clone, Default)]
struct SharedRecordingBackend {
    sends: Arc<Mutex<Vec<(AgentId, String)>>>,
}

impl AgentBackend for SharedRecordingBackend {
    fn launch(&mut self, _request: work_leaf::AgentLaunch) -> Result<AgentSession, AgentError> {
        unreachable!("protocol tests route through existing agents")
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.sends
            .lock()
            .unwrap()
            .push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(MessageRole::Agent, "ok"))
    }
}

fn temp_git_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
    git(&root, ["init"]);
    git(&root, ["config", "user.name", "Work Leaf Test"]);
    git(&root, ["config", "user.email", "work-leaf@example.test"]);
    root
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
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

fn numbered_lines(prefix: &str, count: usize) -> String {
    (0..count)
        .map(|index| format!("{prefix}-{index:04}\n"))
        .collect()
}
