use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::{fmt, fs};

use crate::agent::{AgentBackend, AgentError, AgentId, AgentStreamEvent, ChatMessage};
use crate::locks::{CommandWriteIntent, CommandWritePolicy, FileAccessError, FileLockTable};
use crate::patch::{GitPatcher, PatchError, PatchRequest, render_no_files_prompt};

#[derive(Debug)]
pub struct AgentOrchestrator<B> {
    locks: FileLockTable,
    file_reads: FileReadTracker,
    command_changes: CommandChangeTracker,
    command_policy: CommandWritePolicy,
    locked_command_timeout: Duration,
    backend: B,
}

impl<B> AgentOrchestrator<B>
where
    B: AgentBackend,
{
    pub fn new(root: PathBuf, backend: B) -> Self {
        Self {
            locks: FileLockTable::new(root),
            file_reads: FileReadTracker::default(),
            command_changes: CommandChangeTracker::default(),
            command_policy: CommandWritePolicy,
            locked_command_timeout: default_locked_command_timeout(),
            backend,
        }
    }

    pub fn with_locked_command_timeout(mut self, timeout: Duration) -> Self {
        self.locked_command_timeout = timeout;
        self
    }

    pub fn handle_agent_message(
        &mut self,
        agent_id: &AgentId,
        feature: &str,
        text: &str,
    ) -> Result<Vec<OrchestratorEvent>, OrchestratorError> {
        handle_agent_directives(
            &mut self.backend,
            DirectiveServices {
                locks: &self.locks,
                file_reads: &self.file_reads,
                command_changes: &self.command_changes,
                command_policy: &self.command_policy,
                locked_command_timeout: self.locked_command_timeout,
            },
            agent_id,
            feature,
            text,
        )
        .map(|run| run.events)
    }

    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrchestratorEvent {
    AgentDone {
        agent_id: AgentId,
    },
    ProtocolCorrectionSent {
        agent_id: AgentId,
    },
    FileTextSent {
        agent_id: AgentId,
        paths: Vec<PathBuf>,
    },
    FileTextUnavailable {
        agent_id: AgentId,
        paths: Vec<PathBuf>,
        diagnostic: String,
    },
    FileUpdateSent {
        agent_id: AgentId,
        paths: Vec<PathBuf>,
    },
    CommandClassified {
        agent_id: AgentId,
        writes: bool,
        paths: Vec<PathBuf>,
    },
    CommandRun {
        agent_id: AgentId,
        command: String,
        status: Option<i32>,
        locked_paths: Vec<PathBuf>,
        stdout: String,
        stderr: String,
    },
    PatchApplied {
        agent_id: AgentId,
        feature: String,
        reason: String,
        commit: String,
        files: Vec<PathBuf>,
    },
    PatchRejected {
        agent_id: AgentId,
        files: Vec<PathBuf>,
        diagnostic: String,
    },
    MessageRouted {
        from: AgentId,
        to: AgentId,
    },
}

impl OrchestratorEvent {
    pub fn summary(&self) -> String {
        match self {
            Self::AgentDone { agent_id } => {
                format!("agent {agent_id} reported done")
            }
            Self::ProtocolCorrectionSent { agent_id } => {
                format!("sent protocol correction to {agent_id}")
            }
            Self::FileTextSent { agent_id, paths } => {
                format!("sent file text to {agent_id}: {}", display_paths(paths))
            }
            Self::FileTextUnavailable {
                agent_id, paths, ..
            } => {
                format!(
                    "reported unavailable file text to {agent_id}: {}",
                    display_paths(paths)
                )
            }
            Self::FileUpdateSent { agent_id, paths } => {
                format!("sent file update to {agent_id}: {}", display_paths(paths))
            }
            Self::CommandClassified {
                agent_id,
                writes,
                paths,
            } => format!(
                "classified command for {agent_id}: writes={} paths={}",
                if *writes { "yes" } else { "no" },
                display_paths(paths)
            ),
            Self::CommandRun {
                agent_id,
                command,
                status,
                locked_paths,
                ..
            } => format!(
                "ran command for {agent_id}: status={} paths={} command={command}",
                display_status(*status),
                display_paths(locked_paths)
            ),
            Self::PatchApplied {
                agent_id,
                reason,
                commit,
                files,
                ..
            } => format!(
                "applied patch from {agent_id}: {reason}; commit={commit}; files={}",
                display_paths(files)
            ),
            Self::PatchRejected {
                agent_id, files, ..
            } => format!(
                "sent patch diagnostics to {agent_id}: {}",
                display_paths(files)
            ),
            Self::MessageRouted { from, to } => {
                format!("routed message from {from} to {to}")
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FileReadTracker {
    inner: Arc<Mutex<BTreeMap<AgentId, BTreeSet<PathBuf>>>>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CommandChangeTracker {
    inner: Arc<Mutex<BTreeMap<AgentId, BTreeSet<PathBuf>>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StaleFileUpdate {
    agent_id: AgentId,
    paths: Vec<PathBuf>,
}

#[derive(Clone, Copy)]
pub(crate) struct DirectiveServices<'a> {
    pub locks: &'a FileLockTable,
    pub file_reads: &'a FileReadTracker,
    pub command_changes: &'a CommandChangeTracker,
    pub command_policy: &'a CommandWritePolicy,
    pub locked_command_timeout: Duration,
}

impl FileReadTracker {
    fn record_snapshots(&self, agent_id: &AgentId, snapshots: &[crate::locks::FileSnapshot]) {
        if snapshots.is_empty() {
            return;
        }

        let mut reads = self.inner.lock().expect("file read tracker mutex poisoned");
        let paths = reads.entry(agent_id.clone()).or_default();
        for snapshot in snapshots {
            paths.insert(snapshot.path.clone());
        }
    }

    fn clear_files(&self, agent_id: &AgentId, files: &[PathBuf]) {
        let mut reads = self.inner.lock().expect("file read tracker mutex poisoned");
        if let Some(paths) = reads.get_mut(agent_id) {
            for file in files {
                paths.remove(file);
            }
            if paths.is_empty() {
                reads.remove(agent_id);
            }
        }
    }

    fn clear_agent(&self, agent_id: &AgentId) {
        self.inner
            .lock()
            .expect("file read tracker mutex poisoned")
            .remove(agent_id);
    }

    fn stale_readers(&self, changed_by: &AgentId, files: &[PathBuf]) -> Vec<StaleFileUpdate> {
        let changed = files.iter().collect::<BTreeSet<_>>();
        let reads = self.inner.lock().expect("file read tracker mutex poisoned");
        reads
            .iter()
            .filter(|(agent_id, _)| *agent_id != changed_by)
            .filter_map(|(agent_id, paths)| {
                let stale_paths = paths
                    .iter()
                    .filter(|path| changed.contains(path))
                    .cloned()
                    .collect::<Vec<_>>();
                if stale_paths.is_empty() {
                    None
                } else {
                    Some(StaleFileUpdate {
                        agent_id: agent_id.clone(),
                        paths: stale_paths,
                    })
                }
            })
            .collect()
    }
}

impl CommandChangeTracker {
    fn record_files(&self, agent_id: &AgentId, files: &[PathBuf]) {
        if files.is_empty() {
            return;
        }
        let mut pending = self
            .inner
            .lock()
            .expect("command change tracker mutex poisoned");
        pending
            .entry(agent_id.clone())
            .or_default()
            .extend(files.iter().cloned());
    }

    fn clear_files(&self, agent_id: &AgentId, files: &[PathBuf]) {
        let mut pending = self
            .inner
            .lock()
            .expect("command change tracker mutex poisoned");
        if let Some(paths) = pending.get_mut(agent_id) {
            for file in files {
                paths.remove(file);
            }
            if paths.is_empty() {
                pending.remove(agent_id);
            }
        }
    }

    fn clear_agent(&self, agent_id: &AgentId) {
        self.inner
            .lock()
            .expect("command change tracker mutex poisoned")
            .remove(agent_id);
    }

    fn pending_files(&self, agent_id: &AgentId) -> Vec<PathBuf> {
        self.inner
            .lock()
            .expect("command change tracker mutex poisoned")
            .get(agent_id)
            .map(|paths| paths.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AgentFollowUp {
    pub agent_id: AgentId,
    pub text: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DirectiveRun {
    pub events: Vec<OrchestratorEvent>,
    pub follow_up_replies: Vec<AgentFollowUp>,
    pub completed: bool,
}

pub(crate) fn handle_agent_directives<B>(
    backend: &mut B,
    services: DirectiveServices<'_>,
    agent_id: &AgentId,
    feature: &str,
    text: &str,
) -> Result<DirectiveRun, OrchestratorError>
where
    B: AgentBackend,
{
    handle_agent_directives_streaming(backend, services, agent_id, feature, text, &mut |_, _| {})
}

pub(crate) fn handle_agent_directives_streaming<B>(
    backend: &mut B,
    services: DirectiveServices<'_>,
    agent_id: &AgentId,
    feature: &str,
    text: &str,
    stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
) -> Result<DirectiveRun, OrchestratorError>
where
    B: AgentBackend,
{
    let directives = parse_agent_directives(text)?;
    let mut run = DirectiveRun::default();
    let mut applied_patch_files = BTreeSet::new();

    if directives.is_empty() && needs_protocol_correction(text) {
        let mut sink = |event| stream(agent_id, event);
        let reply =
            backend.send_streaming(agent_id, &render_protocol_correction_prompt(), &mut sink)?;
        run.follow_up_replies
            .push(follow_up(agent_id.clone(), reply));
        run.events.push(OrchestratorEvent::ProtocolCorrectionSent {
            agent_id: agent_id.clone(),
        });
        return Ok(run);
    }

    let mut directives = directives.into_iter().peekable();
    while let Some(directive) = directives.next() {
        match directive {
            AgentDirective::Read(mut paths) => {
                while matches!(directives.peek(), Some(AgentDirective::Read(_))) {
                    if let Some(AgentDirective::Read(next_paths)) = directives.next() {
                        paths.extend(next_paths);
                    }
                }
                send_file_read_response(backend, services, agent_id, &paths, stream, &mut run)?;
            }
            AgentDirective::Classify(command) => {
                let intent = services
                    .command_policy
                    .classify(command.iter().map(String::as_str));
                let mut sink = |event| stream(agent_id, event);
                let reply = backend.send_streaming(
                    agent_id,
                    &render_command_classification(&command, &intent),
                    &mut sink,
                )?;
                run.follow_up_replies
                    .push(follow_up(agent_id.clone(), reply));
                run.events.push(OrchestratorEvent::CommandClassified {
                    agent_id: agent_id.clone(),
                    writes: intent.writes,
                    paths: intent.paths,
                });
            }
            AgentDirective::Run {
                command,
                lock_paths,
            } => {
                run_command_for_agent(
                    backend,
                    services,
                    agent_id,
                    &command,
                    &lock_paths,
                    stream,
                    &mut run,
                )?;
            }
            AgentDirective::Patch { reason, diff } => {
                let patcher =
                    GitPatcher::new(services.locks.root().to_path_buf(), services.locks.clone());
                let request =
                    PatchRequest::new(agent_id.clone(), feature.to_string(), reason.clone(), diff);
                match patcher.apply(request) {
                    Ok(outcome) => {
                        let files = outcome.files.clone();
                        services.file_reads.clear_files(agent_id, &files);
                        services.command_changes.clear_files(agent_id, &files);
                        applied_patch_files.extend(files.iter().cloned());
                        run.events.push(OrchestratorEvent::PatchApplied {
                            agent_id: agent_id.clone(),
                            feature: feature.to_string(),
                            reason,
                            commit: outcome.commit,
                            files: outcome.files,
                        });
                        send_stale_file_updates(
                            backend, services, agent_id, &files, stream, &mut run,
                        )?;
                    }
                    Err(PatchError::Conflict { files, diagnostic }) => {
                        let response = read_requested_files(services.locks, &files)?;
                        let mut sink = |event| stream(agent_id, event);
                        let reply = backend.send_streaming(
                            agent_id,
                            &render_patch_conflict_prompt(&files, &diagnostic, &response),
                            &mut sink,
                        )?;
                        services
                            .file_reads
                            .record_snapshots(agent_id, &response.snapshots);
                        run.follow_up_replies
                            .push(follow_up(agent_id.clone(), reply));
                        run.events.push(OrchestratorEvent::PatchRejected {
                            agent_id: agent_id.clone(),
                            files,
                            diagnostic,
                        });
                    }
                    Err(PatchError::NoFiles) => {
                        let mut sink = |event| stream(agent_id, event);
                        let reply = backend.send_streaming(
                            agent_id,
                            &render_no_files_prompt(),
                            &mut sink,
                        )?;
                        run.follow_up_replies
                            .push(follow_up(agent_id.clone(), reply));
                        run.events.push(OrchestratorEvent::PatchRejected {
                            agent_id: agent_id.clone(),
                            files: Vec::new(),
                            diagnostic:
                                "patch body did not include recognizable unified diff file headers"
                                    .to_string(),
                        });
                    }
                    Err(error) => return Err(OrchestratorError::Patch(error)),
                }
            }
            AgentDirective::Send { target, message } => {
                let mut sink = |event| stream(&target, event);
                let reply = backend.send_streaming(
                    &target,
                    &format!("Message from {agent_id} about {feature}:\n{message}"),
                    &mut sink,
                )?;
                run.follow_up_replies.push(follow_up(target.clone(), reply));
                run.events.push(OrchestratorEvent::MessageRouted {
                    from: agent_id.clone(),
                    to: target,
                });
            }
            AgentDirective::Done => {
                let pending_files = services.command_changes.pending_files(agent_id);
                if !pending_files.is_empty() {
                    let diff = git_diff_head(services.locks.root(), &pending_files);
                    let mut sink = |event| stream(agent_id, event);
                    let reply = backend.send_streaming(
                        agent_id,
                        &render_pending_command_changes_prompt(&pending_files, &diff),
                        &mut sink,
                    )?;
                    run.follow_up_replies
                        .push(follow_up(agent_id.clone(), reply));
                    break;
                }
                services.file_reads.clear_agent(agent_id);
                services.command_changes.clear_agent(agent_id);
                run.completed = true;
                run.events.push(OrchestratorEvent::AgentDone {
                    agent_id: agent_id.clone(),
                });
                break;
            }
        }
    }

    if !applied_patch_files.is_empty() && !run.completed {
        let files = applied_patch_files.into_iter().collect::<Vec<_>>();
        let mut sink = |event| stream(agent_id, event);
        let reply =
            backend.send_streaming(agent_id, &render_patch_applied_prompt(&files), &mut sink)?;
        run.follow_up_replies
            .push(follow_up(agent_id.clone(), reply));
    }

    Ok(run)
}

fn send_file_read_response<B>(
    backend: &mut B,
    services: DirectiveServices<'_>,
    agent_id: &AgentId,
    paths: &[PathBuf],
    stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    run: &mut DirectiveRun,
) -> Result<(), OrchestratorError>
where
    B: AgentBackend,
{
    let response = read_requested_files(services.locks, paths)?;
    let normalized_paths = response
        .snapshots
        .iter()
        .map(|snapshot| snapshot.path.clone())
        .collect::<Vec<_>>();
    let unavailable_paths = response
        .failures
        .iter()
        .map(|failure| failure.path.clone())
        .collect::<Vec<_>>();
    let prompt = render_file_read_response(&response.snapshots, &response.failures);
    let mut sink = |event| stream(agent_id, event);
    let reply = backend.send_streaming(agent_id, &prompt, &mut sink)?;
    services
        .file_reads
        .record_snapshots(agent_id, &response.snapshots);
    run.follow_up_replies
        .push(follow_up(agent_id.clone(), reply));
    if !normalized_paths.is_empty() {
        run.events.push(OrchestratorEvent::FileTextSent {
            agent_id: agent_id.clone(),
            paths: normalized_paths,
        });
    }
    if !unavailable_paths.is_empty() {
        run.events.push(OrchestratorEvent::FileTextUnavailable {
            agent_id: agent_id.clone(),
            paths: unavailable_paths,
            diagnostic: render_file_read_failures(&response.failures),
        });
    }
    Ok(())
}

fn send_stale_file_updates<B>(
    backend: &mut B,
    services: DirectiveServices<'_>,
    changed_by: &AgentId,
    files: &[PathBuf],
    stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    run: &mut DirectiveRun,
) -> Result<(), OrchestratorError>
where
    B: AgentBackend,
{
    for update in services.file_reads.stale_readers(changed_by, files) {
        let response = read_requested_files(services.locks, &update.paths)?;
        let prompt = render_file_update_prompt(&response);
        let mut sink = |event| stream(&update.agent_id, event);
        let reply = backend.send_streaming(&update.agent_id, &prompt, &mut sink)?;
        services
            .file_reads
            .record_snapshots(&update.agent_id, &response.snapshots);
        run.follow_up_replies
            .push(follow_up(update.agent_id.clone(), reply));
        run.events.push(OrchestratorEvent::FileUpdateSent {
            agent_id: update.agent_id,
            paths: update.paths,
        });
    }
    Ok(())
}

fn run_command_for_agent<B>(
    backend: &mut B,
    services: DirectiveServices<'_>,
    agent_id: &AgentId,
    command: &str,
    lock_paths: &[PathBuf],
    stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    run: &mut DirectiveRun,
) -> Result<(), OrchestratorError>
where
    B: AgentBackend,
{
    let locked_paths = normalize_paths(services.locks, lock_paths)?;
    let dirty_before = tracked_changed_files(services.locks.root());
    let output = services.locks.with_write_locks(&locked_paths, || {
        run_shell_command(
            services.locks.root(),
            command,
            services.locked_command_timeout,
        )
        .map_err(FileAccessError::Io)
    })?;
    let command_changed_files = command_changed_files(
        &dirty_before,
        &tracked_changed_files(services.locks.root()),
        &locked_paths,
    );
    services
        .command_changes
        .record_files(agent_id, &command_changed_files);
    let prompt = render_command_result(command, &locked_paths, &output);
    let mut sink = |event| stream(agent_id, event);
    let reply = backend.send_streaming(agent_id, &prompt, &mut sink)?;
    run.follow_up_replies
        .push(follow_up(agent_id.clone(), reply));
    run.events.push(OrchestratorEvent::CommandRun {
        agent_id: agent_id.clone(),
        command: command.to_string(),
        status: output.status,
        locked_paths: locked_paths.clone(),
        stdout: output.stdout,
        stderr: output.stderr,
    });
    let file_paths = locked_paths
        .iter()
        .filter(|path| services.locks.root().join(path).is_file())
        .cloned()
        .collect::<Vec<_>>();
    if !file_paths.is_empty() {
        send_stale_file_updates(backend, services, agent_id, &file_paths, stream, run)?;
    }
    Ok(())
}

fn tracked_changed_files(root: &Path) -> BTreeSet<PathBuf> {
    let Ok(output) = Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", "HEAD", "--"])
        .output()
    else {
        return BTreeSet::new();
    };
    if !output.status.success() {
        return BTreeSet::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn command_changed_files(
    before: &BTreeSet<PathBuf>,
    after: &BTreeSet<PathBuf>,
    locked_paths: &[PathBuf],
) -> Vec<PathBuf> {
    after
        .difference(before)
        .filter(|path| {
            locked_paths
                .iter()
                .any(|locked| path_is_under_lock(path, locked))
        })
        .cloned()
        .collect()
}

fn path_is_under_lock(path: &Path, locked: &Path) -> bool {
    path == locked || path.starts_with(locked)
}

fn git_diff_head(root: &Path, files: &[PathBuf]) -> String {
    let mut command = Command::new("git");
    command.current_dir(root).args(["diff", "HEAD", "--"]);
    for file in files {
        command.arg(file);
    }
    let Ok(output) = command.output() else {
        return String::new();
    };
    if !output.status.success() {
        return String::new();
    }
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn normalize_paths(
    locks: &FileLockTable,
    paths: &[PathBuf],
) -> Result<Vec<PathBuf>, FileAccessError> {
    let mut normalized = paths
        .iter()
        .map(|path| locks.normalize_path(path))
        .collect::<Result<Vec<_>, _>>()?;
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn run_shell_command(
    root: &std::path::Path,
    command: &str,
    timeout: Duration,
) -> std::io::Result<CommandRunOutput> {
    #[cfg(windows)]
    let mut child = Command::new("cmd")
        .args(["/C", command])
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    #[cfg(not(windows))]
    let mut child = {
        use std::os::unix::process::CommandExt;

        let wrapped_command = format!(
            "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ({command}) & work_leaf_child=$!; wait $work_leaf_child"
        );
        let mut command_builder = Command::new("sh");
        command_builder
            .arg("-c")
            .arg(wrapped_command)
            .current_dir(root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command_builder.process_group(0).spawn()?
    };

    let start = Instant::now();
    while start.elapsed() < timeout {
        if child.try_wait()?.is_some() {
            let output = child.wait_with_output()?;
            return Ok(CommandRunOutput {
                status: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                timed_out: false,
                timeout,
            });
        }
        let remaining = timeout.saturating_sub(start.elapsed());
        thread::sleep(remaining.min(Duration::from_millis(10)));
    }

    terminate_child(&mut child);
    let output = child.wait_with_output()?;

    Ok(CommandRunOutput {
        status: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        timed_out: true,
        timeout,
    })
}

#[cfg(unix)]
fn terminate_child(child: &mut std::process::Child) {
    let child_id = child.id().to_string();
    let _ = Command::new("kill").args(["-TERM", &child_id]).status();
    thread::sleep(Duration::from_millis(10));
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
    }
}

#[cfg(windows)]
fn terminate_child(child: &mut std::process::Child) {
    let _ = child.kill();
}

fn needs_protocol_correction(text: &str) -> bool {
    text.contains("@work-leaf")
}

fn render_protocol_correction_prompt() -> String {
    [
        "work-leaf protocol correction",
        "`@work-leaf` is not a shell command. Do not run it in a shell and do not ask the user to run it.",
        "Emit orchestrator requests as top-level plain response lines, for example `@work-leaf read src/lib.rs` or `@work-leaf done`.",
        "Do not put directives in prose, quotes, or code fences. Continue the task by emitting the next required directive or `@work-leaf done`.",
    ]
    .join("\n")
}

fn follow_up(agent_id: AgentId, message: ChatMessage) -> AgentFollowUp {
    AgentFollowUp {
        agent_id,
        text: message.text,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileReadResponse {
    snapshots: Vec<crate::locks::FileSnapshot>,
    failures: Vec<FileReadFailure>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileReadFailure {
    path: PathBuf,
    diagnostic: String,
}

fn read_requested_files(
    locks: &FileLockTable,
    paths: &[PathBuf],
) -> Result<FileReadResponse, FileAccessError> {
    let mut failures = Vec::new();
    let mut normalized_paths = BTreeSet::new();

    for path in paths {
        match locks.normalize_path(path) {
            Ok(path) => {
                normalized_paths.insert(path);
            }
            Err(error) => {
                failures.push(FileReadFailure {
                    path: path.clone(),
                    diagnostic: error.to_string(),
                });
            }
        }
    }

    let normalized_paths = normalized_paths.into_iter().collect::<Vec<_>>();
    let mut snapshots = Vec::new();
    if !normalized_paths.is_empty() {
        let mut read_failures = Vec::new();
        snapshots = locks.with_read_locks(&normalized_paths, || {
            let mut snapshots = Vec::new();
            for normalized in &normalized_paths {
                match fs::read_to_string(locks.root().join(normalized)) {
                    Ok(text) => snapshots.push(crate::locks::FileSnapshot {
                        path: normalized.clone(),
                        text,
                    }),
                    Err(error) => read_failures.push(FileReadFailure {
                        path: normalized.clone(),
                        diagnostic: error.to_string(),
                    }),
                }
            }
            Ok(snapshots)
        })?;
        failures.extend(read_failures);
    }

    Ok(FileReadResponse {
        snapshots,
        failures,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AgentDirective {
    Read(Vec<PathBuf>),
    Classify(Vec<String>),
    Run {
        command: String,
        lock_paths: Vec<PathBuf>,
    },
    Patch {
        reason: String,
        diff: String,
    },
    Send {
        target: AgentId,
        message: String,
    },
    Done,
}

fn parse_agent_directives(text: &str) -> Result<Vec<AgentDirective>, OrchestratorError> {
    let mut directives = Vec::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        let Some(body) = directive_body(line) else {
            continue;
        };

        if body == "end" {
            continue;
        }

        if body == "done" {
            directives.push(AgentDirective::Done);
        } else if let Some(rest) = directive_rest(body, "read") {
            let paths = split_required(rest, "read requires at least one path")?
                .into_iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>();
            directives.push(AgentDirective::Read(paths));
        } else if let Some(rest) = directive_rest(body, "locks classify") {
            directives.push(AgentDirective::Classify(split_required(
                rest,
                "locks classify requires a command",
            )?));
        } else if let Some(rest) = directive_rest(body, "locks run") {
            let (lock_paths, command) = parse_locked_run(rest)?;
            directives.push(AgentDirective::Run {
                command,
                lock_paths,
            });
        } else if let Some(rest) = directive_rest(body, "patch") {
            let reason = rest.trim();
            if reason.is_empty() {
                return Err(OrchestratorError::Usage(
                    "patch requires a reason".to_string(),
                ));
            }
            let mut diff = String::new();
            while let Some(next) = lines.peek().copied() {
                if directive_body(next).is_some_and(|body| body == "end") {
                    lines.next();
                    break;
                }
                diff.push_str(next);
                diff.push('\n');
                lines.next();
            }
            if diff.trim().is_empty() {
                return Err(OrchestratorError::Usage(
                    "patch requires a unified diff body".to_string(),
                ));
            }
            directives.push(AgentDirective::Patch {
                reason: reason.to_string(),
                diff,
            });
        } else if let Some(rest) = directive_rest(body, "send") {
            let mut parts = rest.trim().splitn(2, char::is_whitespace);
            let target = parts
                .next()
                .filter(|part| !part.is_empty())
                .ok_or_else(|| OrchestratorError::Usage("send requires an agent id".to_string()))?;
            let message = parts.next().map(str::trim).filter(|part| !part.is_empty());
            let Some(message) = message else {
                return Err(OrchestratorError::Usage(
                    "send requires a message".to_string(),
                ));
            };
            directives.push(AgentDirective::Send {
                target: AgentId::new(target)?,
                message: message.to_string(),
            });
        } else {
            return Err(OrchestratorError::Usage(format!(
                "unknown work-leaf directive `{body}`"
            )));
        }
    }

    Ok(directives)
}

fn directive_body(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let rest = line.strip_prefix("@work-leaf")?;
    let mut chars = rest.chars();
    if !chars.next()?.is_whitespace() {
        return None;
    }
    Some(chars.as_str().trim_start())
}

fn directive_rest<'a>(body: &'a str, command: &str) -> Option<&'a str> {
    let rest = body.strip_prefix(command)?;
    if rest.is_empty() {
        return Some("");
    }
    let mut chars = rest.chars();
    if chars.next()?.is_whitespace() {
        Some(chars.as_str().trim_start())
    } else {
        None
    }
}

fn split_required(rest: &str, error: &str) -> Result<Vec<String>, OrchestratorError> {
    let parts = rest
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if parts.is_empty() {
        Err(OrchestratorError::Usage(error.to_string()))
    } else {
        Ok(parts)
    }
}

fn parse_locked_run(rest: &str) -> Result<(Vec<PathBuf>, String), OrchestratorError> {
    let Some((paths, command)) = rest.split_once(" -- ") else {
        return Err(OrchestratorError::Usage(
            "locks run requires lock paths, `--`, and a command".to_string(),
        ));
    };
    let lock_paths = split_required(paths, "locks run requires at least one lock path before --")?
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let command = command.trim();
    if command.is_empty() {
        return Err(OrchestratorError::Usage(
            "locks run requires a command after --".to_string(),
        ));
    }
    Ok((lock_paths, command.to_string()))
}

fn render_file_read_response(
    snapshots: &[crate::locks::FileSnapshot],
    failures: &[FileReadFailure],
) -> String {
    let mut text = String::from("work-leaf file text\n");
    for snapshot in snapshots {
        text.push_str("\n--- ");
        text.push_str(&snapshot.path.display().to_string());
        text.push_str(" ---\n");
        text.push_str(&snapshot.text);
        if !snapshot.text.ends_with('\n') {
            text.push('\n');
        }
    }
    if !failures.is_empty() {
        text.push_str("\nUnavailable file text\n");
        text.push_str(&render_file_read_failures(failures));
    }
    text
}

fn render_file_read_failures(failures: &[FileReadFailure]) -> String {
    let mut text = String::new();
    for failure in failures {
        text.push_str("- ");
        text.push_str(&failure.path.display().to_string());
        text.push_str(": ");
        text.push_str(&failure.diagnostic);
        text.push('\n');
    }
    text
}

fn render_command_classification(command: &[String], intent: &CommandWriteIntent) -> String {
    format!(
        "work-leaf command classification\ncommand: {}\nwrites: {}\npaths: {}",
        command.join(" "),
        if intent.writes { "yes" } else { "no" },
        display_paths(&intent.paths)
    )
}

fn render_command_result(
    command: &str,
    locked_paths: &[PathBuf],
    output: &CommandRunOutput,
) -> String {
    let mut text = format!(
        "work-leaf command result\ncommand: {command}\nstatus: {}\nlocked paths: {}",
        display_status(output.status),
        display_paths(locked_paths)
    );
    if output.timed_out {
        text.push_str("\ntimed out: yes\ntimeout: ");
        text.push_str(&format_duration(output.timeout));
        text.push_str(
            "\nuser authorization is required to rerun locked commands for longer than this limit.",
        );
    }
    text.push_str("\nstdout:\n");
    if output.stdout.is_empty() {
        text.push_str("<empty>\n");
    } else {
        text.push_str(&output.stdout);
        if !output.stdout.ends_with('\n') {
            text.push('\n');
        }
    }
    text.push_str("stderr:\n");
    if output.stderr.is_empty() {
        text.push_str("<empty>\n");
    } else {
        text.push_str(&output.stderr);
        if !output.stderr.ends_with('\n') {
            text.push('\n');
        }
    }
    text
}

fn render_patch_conflict_prompt(
    files: &[PathBuf],
    diagnostic: &str,
    response: &FileReadResponse,
) -> String {
    let mut text = format!(
        "The orchestrator could not apply your patch.\nFiles: {}\n\nGit diagnostic:\n{}\n\nRebase your patch against the fresh file text below and provide a corrected unified diff patch.",
        display_paths(files),
        diagnostic
    );
    append_file_response(&mut text, response);
    text
}

fn render_patch_applied_prompt(files: &[PathBuf]) -> String {
    let mut text = format!("work-leaf patch applied\nfiles: {}\n", display_paths(files));
    text.push_str("Continue from the repository instructions.\n");
    text.push_str("Run any required or relevant checks through `@work-leaf locks run <path>... -- <command>` when the command may write files.\n");
    text.push_str("Keep locked command runs within five minutes unless the user authorizes a longer lock-holding command.\n");
    text.push_str("Provide additional patches if checks fail or more work is needed; emit `@work-leaf done` only when this patch is ready for review.");
    text
}

fn render_pending_command_changes_prompt(files: &[PathBuf], diff: &str) -> String {
    let mut text = format!(
        "work-leaf command left tracked working-tree changes\nfiles: {}\n",
        display_paths(files)
    );
    text.push_str("Review cannot start until command-produced tracked changes are saved in a provisional commit or reverted.\n");
    text.push_str("If these changes are required, submit them with `@work-leaf patch <reason>` using the diff below; the orchestrator accepts matching already-applied diffs and commits them.\n");
    text.push_str("If they are not required, submit a patch that reverts them. Then rerun required checks if needed and emit `@work-leaf done` only when no tracked command changes remain.\n");
    if !diff.trim().is_empty() {
        text.push_str("\nCurrent tracked diff:\n");
        text.push_str(diff);
        if !diff.ends_with('\n') {
            text.push('\n');
        }
    }
    text
}

fn render_file_update_prompt(response: &FileReadResponse) -> String {
    let mut text = [
        "work-leaf file update",
        "Another agent changed files you previously read before you submitted a patch.",
        "Rebase any pending patch against the fresh file text below.",
    ]
    .join("\n");
    append_file_response(&mut text, response);
    text
}

fn append_file_response(text: &mut String, response: &FileReadResponse) {
    text.push_str("\n\n");
    text.push_str(&render_file_read_response(
        &response.snapshots,
        &response.failures,
    ));
}

fn display_paths(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        return "-".to_string();
    }
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn display_status(status: Option<i32>) -> String {
    status
        .map(|status| status.to_string())
        .unwrap_or_else(|| "terminated".to_string())
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{}s", duration.as_secs())
    } else {
        format!("{}ms", duration.as_millis())
    }
}

fn default_locked_command_timeout() -> Duration {
    Duration::from_secs(5 * 60)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommandRunOutput {
    status: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
    timeout: Duration,
}

#[derive(Debug)]
pub enum OrchestratorError {
    Usage(String),
    Agent(AgentError),
    FileAccess(FileAccessError),
    Patch(PatchError),
}

impl fmt::Display for OrchestratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => formatter.write_str(message),
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::FileAccess(error) => write!(formatter, "{error}"),
            Self::Patch(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for OrchestratorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Agent(error) => Some(error),
            Self::FileAccess(error) => Some(error),
            Self::Patch(error) => Some(error),
            Self::Usage(_) => None,
        }
    }
}

impl From<AgentError> for OrchestratorError {
    fn from(error: AgentError) -> Self {
        Self::Agent(error)
    }
}

impl From<FileAccessError> for OrchestratorError {
    fn from(error: FileAccessError) -> Self {
        Self::FileAccess(error)
    }
}
