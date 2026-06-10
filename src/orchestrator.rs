use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use std::{fmt, fs};

use crate::agent::{AgentBackend, AgentError, AgentId, AgentStreamEvent, ChatMessage};
use crate::locks::{CommandWriteIntent, CommandWritePolicy, FileAccessError, FileLockTable};
use crate::patch::{
    GitPatcher, PatchError, PatchRequest, is_already_applied_diagnostic, render_no_files_prompt,
    render_structured_edit_no_files_prompt, structured_edit_format_guidance,
    unified_diff_format_guidance,
};

const WORK_LEAF_CONTEXT_BUNDLE_DIR_ENV: &str = "WORK_LEAF_CONTEXT_BUNDLE_DIR";

#[derive(Debug)]
pub struct AgentOrchestrator<B> {
    locks: FileLockTable,
    file_reads: FileReadTracker,
    context_bundles: ContextBundleStore,
    command_changes: CommandChangeTracker,
    patch_ownership: PatchOwnershipTracker,
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
            context_bundles: ContextBundleStore::new(),
            command_changes: CommandChangeTracker::default(),
            patch_ownership: PatchOwnershipTracker::default(),
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
                context_bundles: &self.context_bundles,
                command_changes: &self.command_changes,
                patch_ownership: &self.patch_ownership,
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
    inner: Arc<Mutex<BTreeMap<AgentId, BTreeMap<PathBuf, TrackedFileSnapshot>>>>,
}

#[derive(Clone, Debug)]
pub(crate) struct ContextBundleStore {
    inner: Arc<ContextBundleStoreInner>,
}

#[derive(Debug)]
struct ContextBundleStoreInner {
    parent: PathBuf,
    dir: PathBuf,
    counter: AtomicUsize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CommandChangeTracker {
    inner: Arc<Mutex<BTreeMap<AgentId, BTreeSet<PathBuf>>>>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PatchOwnershipTracker {
    inner: Arc<Mutex<BTreeMap<PathBuf, OwnedPatchPath>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OwnedPatchPath {
    agent_id: AgentId,
    commit: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StaleFileUpdate {
    agent_id: AgentId,
    paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TrackedFileSnapshot {
    text: String,
    digest: String,
}

#[derive(Clone, Copy)]
pub(crate) struct DirectiveServices<'a> {
    pub locks: &'a FileLockTable,
    pub file_reads: &'a FileReadTracker,
    pub context_bundles: &'a ContextBundleStore,
    pub command_changes: &'a CommandChangeTracker,
    pub patch_ownership: &'a PatchOwnershipTracker,
    pub command_policy: &'a CommandWritePolicy,
    pub locked_command_timeout: Duration,
}

impl ContextBundleStore {
    pub(crate) fn new() -> Self {
        static STORE_COUNTER: AtomicUsize = AtomicUsize::new(0);

        let parent = std::env::var_os(WORK_LEAF_CONTEXT_BUNDLE_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("work-leaf-context-bundles"));
        cleanup_stale_context_bundle_dirs(&parent);
        let counter = STORE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = parent.join(format!("orchestrator-{}-{counter}", std::process::id()));
        Self {
            inner: Arc::new(ContextBundleStoreInner {
                parent,
                dir,
                counter: AtomicUsize::new(0),
            }),
        }
    }

    fn write(&self, snapshots: &[crate::locks::FileSnapshot]) -> Option<PathBuf> {
        let counter = self.inner.counter.fetch_add(1, Ordering::Relaxed);
        fs::create_dir_all(&self.inner.dir).ok()?;
        let path = self.inner.dir.join(format!("bundle-{counter}.md"));
        let mut text = String::from("# Work Leaf Context Bundle\n\n");
        text.push_str("This file contains orchestrator-mediated read output. Use it as read-only context; submit project changes through `@work-leaf edit`.\n");
        for snapshot in snapshots {
            text.push_str("\n----- BEGIN FILE ");
            text.push_str(&snapshot.path.display().to_string());
            text.push_str(" -----\n");
            text.push_str("digest: ");
            text.push_str(&content_digest(&snapshot.text));
            text.push_str("\n\n");
            text.push_str(&snapshot.text);
            if !snapshot.text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str("----- END FILE ");
            text.push_str(&snapshot.path.display().to_string());
            text.push_str(" -----\n");
        }
        fs::write(&path, text).ok()?;
        Some(path)
    }
}

impl Drop for ContextBundleStoreInner {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
        let _ = fs::remove_dir(&self.parent);
    }
}

fn cleanup_stale_context_bundle_dirs(parent: &Path) {
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let stale = metadata
            .modified()
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age > Duration::from_secs(24 * 60 * 60));
        if stale {
            if metadata.is_dir() {
                let _ = fs::remove_dir_all(entry.path());
            } else {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

impl FileReadTracker {
    fn record_snapshots(&self, agent_id: &AgentId, snapshots: &[crate::locks::FileSnapshot]) {
        if snapshots.is_empty() {
            return;
        }

        let mut reads = self.inner.lock().expect("file read tracker mutex poisoned");
        let paths = reads.entry(agent_id.clone()).or_default();
        for snapshot in snapshots {
            paths.insert(
                snapshot.path.clone(),
                TrackedFileSnapshot {
                    text: snapshot.text.clone(),
                    digest: content_digest(&snapshot.text),
                },
            );
        }
    }

    fn snapshot_for(&self, agent_id: &AgentId, path: &Path) -> Option<TrackedFileSnapshot> {
        self.inner
            .lock()
            .expect("file read tracker mutex poisoned")
            .get(agent_id)
            .and_then(|paths| paths.get(path).cloned())
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
                    .keys()
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

    fn clear_files_for_all(&self, files: &[PathBuf]) {
        if files.is_empty() {
            return;
        }
        let files = files.iter().collect::<BTreeSet<_>>();
        let mut pending = self
            .inner
            .lock()
            .expect("command change tracker mutex poisoned");
        pending.retain(|_, paths| {
            paths.retain(|path| !files.contains(path));
            !paths.is_empty()
        });
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

impl PatchOwnershipTracker {
    fn record_patch(&self, agent_id: &AgentId, commit: &str, files: &[PathBuf]) {
        let test_files = files
            .iter()
            .filter(|path| is_test_like_path(path))
            .cloned()
            .collect::<Vec<_>>();
        if test_files.is_empty() {
            return;
        }

        let mut ownership = self
            .inner
            .lock()
            .expect("patch ownership tracker mutex poisoned");
        for path in test_files {
            ownership.insert(
                path,
                OwnedPatchPath {
                    agent_id: agent_id.clone(),
                    commit: commit.to_string(),
                },
            );
        }
    }

    fn other_agent_test_locks(
        &self,
        agent_id: &AgentId,
        locked_paths: &[PathBuf],
        command_write_paths: &[PathBuf],
    ) -> Vec<(PathBuf, OwnedPatchPath)> {
        if locked_paths.is_empty() {
            return Vec::new();
        }

        let ownership = self
            .inner
            .lock()
            .expect("patch ownership tracker mutex poisoned");
        ownership
            .iter()
            .filter(|(_, owner)| &owner.agent_id != agent_id)
            .filter(|(owned_path, _)| {
                should_block_owned_test_lock(locked_paths, command_write_paths, owned_path)
            })
            .map(|(path, owner)| (path.clone(), owner.clone()))
            .collect()
    }
}

fn should_block_owned_test_lock(
    locked_paths: &[PathBuf],
    command_write_paths: &[PathBuf],
    owned_path: &Path,
) -> bool {
    let command_writes_owned_path = command_write_paths
        .iter()
        .any(|path| paths_overlap(path, owned_path));

    locked_paths.iter().any(|locked| {
        if locked == owned_path || locked.starts_with(owned_path) {
            return true;
        }
        if owned_path.starts_with(locked) {
            return command_write_paths.is_empty() || command_writes_owned_path;
        }
        false
    })
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    if is_repo_root_path(left) || is_repo_root_path(right) {
        return true;
    }
    left == right || left.starts_with(right) || right.starts_with(left)
}

fn is_repo_root_path(path: &Path) -> bool {
    path.as_os_str().is_empty() || path == Path::new(".")
}

fn is_test_like_path(path: &Path) -> bool {
    let mut saw_test_component = false;
    for component in path.components() {
        let text = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        if matches!(
            text.as_str(),
            "test"
                | "tests"
                | "__tests__"
                | "spec"
                | "specs"
                | "e2e"
                | "integration"
                | "integration-tests"
                | "integration_tests"
        ) {
            saw_test_component = true;
        }
    }

    if saw_test_component {
        return true;
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let file_name = file_name.to_ascii_lowercase();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    if extension
        .as_deref()
        .is_some_and(|extension| matches!(extension, "test" | "spec"))
    {
        return true;
    }

    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(file_name.as_str())
        .to_ascii_lowercase();
    let tokens = stem
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty());
    tokens.into_iter().any(|token| {
        matches!(token, "test" | "tests" | "spec" | "specs")
            || token.ends_with("test")
            || token.ends_with("tests")
            || token.ends_with("spec")
            || token.ends_with("specs")
    })
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

#[derive(Debug, Default)]
pub(crate) struct DirectiveStreamInterruptDetector {
    text: String,
    interrupted: bool,
}

impl DirectiveStreamInterruptDetector {
    pub fn observe(&mut self, event: &AgentStreamEvent) -> bool {
        if self.interrupted {
            return false;
        }
        let AgentStreamEvent::AgentMessage(text) = event else {
            return false;
        };
        if !self.text.is_empty() {
            self.text.push_str("\n\n");
        }
        self.text.push_str(text);
        if should_interrupt_after_streamed_directive(&self.text) {
            self.interrupted = true;
            true
        } else {
            false
        }
    }
}

pub(crate) fn send_agent_streaming_interruptible<B>(
    backend: &mut B,
    agent_id: &AgentId,
    prompt: &str,
    stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
) -> Result<ChatMessage, AgentError>
where
    B: AgentBackend,
{
    let mut detector = DirectiveStreamInterruptDetector::default();
    let mut sink = |event| stream(agent_id, event);
    let mut should_interrupt = |event: &AgentStreamEvent| detector.observe(event);
    backend.send_streaming_interruptible(agent_id, prompt, &mut sink, &mut should_interrupt)
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
        let reply = send_agent_streaming_interruptible(
            backend,
            agent_id,
            &render_protocol_correction_prompt(),
            stream,
        )?;
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
            AgentDirective::Read(mut request) => {
                while matches!(directives.peek(), Some(AgentDirective::Read(_))) {
                    if let Some(AgentDirective::Read(next_request)) = directives.next() {
                        request.paths.extend(next_request.paths);
                        request.force |= next_request.force;
                    }
                }
                send_file_read_response(
                    backend,
                    services,
                    agent_id,
                    &request.paths,
                    request.force,
                    stream,
                    &mut run,
                )?;
            }
            AgentDirective::Classify(command) => {
                let intent = services
                    .command_policy
                    .classify(command.iter().map(String::as_str));
                let reply = send_agent_streaming_interruptible(
                    backend,
                    agent_id,
                    &render_command_classification(&command, &intent),
                    stream,
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
                        services.command_changes.clear_files_for_all(&files);
                        services
                            .patch_ownership
                            .record_patch(agent_id, &outcome.commit, &files);
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
                        let prompt = if is_already_applied_diagnostic(&diagnostic) {
                            render_already_applied_patch_prompt(&files)
                        } else {
                            let response = read_requested_files(services.locks, &files)?;
                            let prompt = render_patch_conflict_prompt(
                                agent_id,
                                services.file_reads,
                                &files,
                                &diagnostic,
                                &response,
                            );
                            services
                                .file_reads
                                .record_snapshots(agent_id, &response.snapshots);
                            prompt
                        };
                        let reply =
                            send_agent_streaming_interruptible(backend, agent_id, &prompt, stream)?;
                        run.follow_up_replies
                            .push(follow_up(agent_id.clone(), reply));
                        run.events.push(OrchestratorEvent::PatchRejected {
                            agent_id: agent_id.clone(),
                            files,
                            diagnostic,
                        });
                    }
                    Err(PatchError::NoFiles) => {
                        let reply = send_agent_streaming_interruptible(
                            backend,
                            agent_id,
                            &render_no_files_prompt(),
                            stream,
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
            AgentDirective::Edit { reason, body } => {
                let patcher =
                    GitPatcher::new(services.locks.root().to_path_buf(), services.locks.clone());
                let request =
                    PatchRequest::new(agent_id.clone(), feature.to_string(), reason.clone(), body);
                match patcher.apply_edit(request) {
                    Ok(outcome) => {
                        let files = outcome.files.clone();
                        services.file_reads.clear_files(agent_id, &files);
                        services.command_changes.clear_files_for_all(&files);
                        services
                            .patch_ownership
                            .record_patch(agent_id, &outcome.commit, &files);
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
                        let prompt = if is_already_applied_diagnostic(&diagnostic) {
                            render_already_applied_patch_prompt(&files)
                        } else if files.is_empty() {
                            format!(
                                "The orchestrator could not apply your edit.\n\nDiagnostic:\n{diagnostic}\n\n{}",
                                structured_edit_format_guidance()
                            )
                        } else {
                            let response = read_requested_files(services.locks, &files)?;
                            let prompt = render_structured_edit_conflict_prompt(
                                agent_id,
                                services.file_reads,
                                &files,
                                &diagnostic,
                                &response,
                            );
                            services
                                .file_reads
                                .record_snapshots(agent_id, &response.snapshots);
                            prompt
                        };
                        let reply =
                            send_agent_streaming_interruptible(backend, agent_id, &prompt, stream)?;
                        run.follow_up_replies
                            .push(follow_up(agent_id.clone(), reply));
                        run.events.push(OrchestratorEvent::PatchRejected {
                            agent_id: agent_id.clone(),
                            files,
                            diagnostic,
                        });
                    }
                    Err(PatchError::NoFiles) => {
                        let reply = send_agent_streaming_interruptible(
                            backend,
                            agent_id,
                            &render_structured_edit_no_files_prompt(),
                            stream,
                        )?;
                        run.follow_up_replies
                            .push(follow_up(agent_id.clone(), reply));
                        run.events.push(OrchestratorEvent::PatchRejected {
                            agent_id: agent_id.clone(),
                            files: Vec::new(),
                            diagnostic:
                                "edit body did not include recognizable structured edit file headers"
                                    .to_string(),
                        });
                    }
                    Err(error) => return Err(OrchestratorError::Patch(error)),
                }
            }
            AgentDirective::Send { target, message } => {
                let reply = send_agent_streaming_interruptible(
                    backend,
                    &target,
                    &format!("Message from {agent_id} about {feature}:\n{message}"),
                    stream,
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
                    let reply = send_agent_streaming_interruptible(
                        backend,
                        agent_id,
                        &render_pending_command_changes_prompt(&pending_files, &diff),
                        stream,
                    )?;
                    run.follow_up_replies
                        .push(follow_up(agent_id.clone(), reply));
                    break;
                }
                if run
                    .follow_up_replies
                    .iter()
                    .any(|follow_up| follow_up.agent_id == *agent_id)
                {
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
        let reply = send_agent_streaming_interruptible(
            backend,
            agent_id,
            &render_patch_applied_prompt(&files),
            stream,
        )?;
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
    force: bool,
    stream: &mut dyn FnMut(&AgentId, AgentStreamEvent),
    run: &mut DirectiveRun,
) -> Result<(), OrchestratorError>
where
    B: AgentBackend,
{
    let response = read_requested_files(services.locks, paths)?;
    let (exact_snapshots, changed_snapshots, unchanged_snapshots) =
        split_repeated_file_reads(services.file_reads, agent_id, &response.snapshots, force);
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
    let prompt = render_file_read_response(
        services.context_bundles,
        services.file_reads,
        agent_id,
        &exact_snapshots,
        &changed_snapshots,
        &unchanged_snapshots,
        &response.failures,
    );
    let reply = send_agent_streaming_interruptible(backend, agent_id, &prompt, stream)?;
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

fn split_repeated_file_reads(
    file_reads: &FileReadTracker,
    agent_id: &AgentId,
    snapshots: &[crate::locks::FileSnapshot],
    _force: bool,
) -> (
    Vec<crate::locks::FileSnapshot>,
    Vec<crate::locks::FileSnapshot>,
    Vec<crate::locks::FileSnapshot>,
) {
    let mut exact = Vec::new();
    let mut changed = Vec::new();
    let mut unchanged = Vec::new();
    for snapshot in snapshots {
        let current_digest = content_digest(&snapshot.text);
        match file_reads.snapshot_for(agent_id, &snapshot.path) {
            Some(previous) if previous.digest == current_digest => unchanged.push(snapshot.clone()),
            Some(_) => changed.push(snapshot.clone()),
            None => exact.push(snapshot.clone()),
        }
    }
    (exact, changed, unchanged)
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
        let prompt = render_file_update_prompt(&update.agent_id, services.file_reads, &response);
        let reply = send_agent_streaming_interruptible(backend, &update.agent_id, &prompt, stream)?;
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
    let command_write_paths = normalize_paths(
        services.locks,
        &services
            .command_policy
            .classify(command.split_whitespace())
            .paths,
    )?;
    let blocked_paths = services.patch_ownership.other_agent_test_locks(
        agent_id,
        &locked_paths,
        &command_write_paths,
    );
    if !blocked_paths.is_empty() {
        let prompt = render_other_agent_test_command_prompt(&blocked_paths);
        let reply = send_agent_streaming_interruptible(backend, agent_id, &prompt, stream)?;
        run.follow_up_replies
            .push(follow_up(agent_id.clone(), reply));
        return Ok(());
    }

    if let Some(diagnostic) = masked_command_diagnostic(command) {
        let prompt = render_command_rejected(command, &locked_paths, &diagnostic);
        let reply = send_agent_streaming_interruptible(backend, agent_id, &prompt, stream)?;
        run.follow_up_replies
            .push(follow_up(agent_id.clone(), reply));
        return Ok(());
    }

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
    let reply = send_agent_streaming_interruptible(backend, agent_id, &prompt, stream)?;
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
        if let Some(tmp_dir) = std::env::var_os("WORK_LEAF_COMMAND_TMPDIR") {
            command_builder.env("TMPDIR", tmp_dir);
        }
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
    Read(FileReadRequest),
    Classify(Vec<String>),
    Run {
        command: String,
        lock_paths: Vec<PathBuf>,
    },
    Patch {
        reason: String,
        diff: String,
    },
    Edit {
        reason: String,
        body: String,
    },
    Send {
        target: AgentId,
        message: String,
    },
    Done,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileReadRequest {
    paths: Vec<PathBuf>,
    force: bool,
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
            directives.push(AgentDirective::Read(parse_read_request(rest)?));
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
        } else if let Some(rest) = directive_rest(body, "edit") {
            let reason = rest.trim();
            if reason.is_empty() {
                return Err(OrchestratorError::Usage(
                    "edit requires a reason".to_string(),
                ));
            }
            let mut body = String::new();
            while let Some(next) = lines.peek().copied() {
                if directive_body(next).is_some_and(|body| body == "end") {
                    lines.next();
                    break;
                }
                body.push_str(next);
                body.push('\n');
                lines.next();
            }
            if body.trim().is_empty() {
                return Err(OrchestratorError::Usage(
                    "edit requires a structured edit body".to_string(),
                ));
            }
            directives.push(AgentDirective::Edit {
                reason: reason.to_string(),
                body,
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

fn should_interrupt_after_streamed_directive(text: &str) -> bool {
    let mut in_patch = false;
    for line in text.lines() {
        let Some(body) = directive_body(line) else {
            continue;
        };
        if in_patch {
            if body == "end" {
                return true;
            }
            continue;
        }
        if body == "done" {
            return true;
        }
        if directive_rest(body, "read").is_some() {
            return true;
        }
        if directive_rest(body, "patch").is_some() || directive_rest(body, "edit").is_some() {
            in_patch = true;
            continue;
        }
        if directive_rest(body, "locks run").is_some()
            || directive_rest(body, "locks classify").is_some()
            || directive_rest(body, "send").is_some()
        {
            return true;
        }
    }
    false
}

fn parse_read_request(rest: &str) -> Result<FileReadRequest, OrchestratorError> {
    let mut force = false;
    let mut paths = Vec::new();
    for part in split_required(rest, "read requires at least one path")? {
        if part == "--force" {
            force = true;
        } else {
            paths.push(PathBuf::from(part));
        }
    }
    if paths.is_empty() {
        return Err(OrchestratorError::Usage(
            "read requires at least one path".to_string(),
        ));
    }
    Ok(FileReadRequest { paths, force })
}

fn directive_body(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let rest = line.strip_prefix("@work-leaf")?;
    let mut chars = rest.chars();
    if !chars.next()?.is_whitespace() {
        return None;
    }
    Some(chars.as_str().trim())
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum ShellToken {
    Word(String),
    OrIf,
    AndIf,
    Semi,
}

fn masked_command_diagnostic(command: &str) -> Option<String> {
    masked_command_diagnostic_inner(command, 0)
}

fn masked_command_diagnostic_inner(command: &str, depth: usize) -> Option<String> {
    if depth > 4 {
        return None;
    }
    let tokens = shell_tokens(command);
    if let Some(diagnostic) = masked_tokens_diagnostic(&tokens) {
        return Some(diagnostic);
    }
    for script in shell_script_arguments(&tokens) {
        if let Some(diagnostic) = masked_command_diagnostic_inner(&script, depth + 1) {
            return Some(format!("{diagnostic} inside a shell script argument"));
        }
    }
    None
}

fn shell_tokens(command: &str) -> Vec<ShellToken> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut chars = command.chars().peekable();
    let mut quote = None;

    let flush_word = |word: &mut String, tokens: &mut Vec<ShellToken>| {
        if !word.is_empty() {
            tokens.push(ShellToken::Word(std::mem::take(word)));
        }
    };

    while let Some(character) = chars.next() {
        if let Some(quote_character) = quote {
            if character == quote_character {
                quote = None;
            } else if quote_character == '"' && character == '\\' {
                if let Some(next) = chars.next() {
                    word.push(next);
                }
            } else {
                word.push(character);
            }
            continue;
        }

        match character {
            '\'' | '"' => quote = Some(character),
            '\\' => {
                if let Some(next) = chars.next() {
                    word.push(next);
                }
            }
            character if character.is_whitespace() => flush_word(&mut word, &mut tokens),
            '|' if chars.peek() == Some(&'|') => {
                chars.next();
                flush_word(&mut word, &mut tokens);
                tokens.push(ShellToken::OrIf);
            }
            '&' if chars.peek() == Some(&'&') => {
                chars.next();
                flush_word(&mut word, &mut tokens);
                tokens.push(ShellToken::AndIf);
            }
            ';' => {
                flush_word(&mut word, &mut tokens);
                tokens.push(ShellToken::Semi);
            }
            _ => word.push(character),
        }
    }
    flush_word(&mut word, &mut tokens);
    tokens
}

fn masked_tokens_diagnostic(tokens: &[ShellToken]) -> Option<String> {
    for (index, token) in tokens.iter().enumerate() {
        match token {
            ShellToken::Word(word) if word == "set" => {
                let next = word_token(tokens.get(index + 1));
                let after_next = word_token(tokens.get(index + 2));
                if next == Some("+e") {
                    return Some("uses `set +e` to ignore command failures".to_string());
                }
                if next == Some("+o") && after_next == Some("errexit") {
                    return Some("uses `set +o errexit` to ignore command failures".to_string());
                }
            }
            ShellToken::OrIf => {
                if let Some(word) = word_token(tokens.get(index + 1))
                    && is_success_literal(word)
                {
                    return Some(format!("uses `|| {word}` to mask command failures"));
                }
            }
            ShellToken::Semi => {
                if let Some(word) = word_token(tokens.get(index + 1))
                    && is_success_literal(word)
                    && tokens[index + 2..].iter().all(is_control_token)
                {
                    return Some(format!(
                        "ends with `; {word}`, which can hide the check result"
                    ));
                }
            }
            _ => {}
        }
    }
    None
}

fn word_token(token: Option<&ShellToken>) -> Option<&str> {
    match token {
        Some(ShellToken::Word(word)) => Some(word.as_str()),
        _ => None,
    }
}

fn is_success_literal(word: &str) -> bool {
    matches!(word, "true" | ":")
}

fn is_control_token(token: &ShellToken) -> bool {
    !matches!(token, ShellToken::Word(_))
}

fn shell_script_arguments(tokens: &[ShellToken]) -> Vec<String> {
    let mut scripts = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let Some(word) = word_token(tokens.get(index)) else {
            index += 1;
            continue;
        };
        if !is_shell_program(word) {
            index += 1;
            continue;
        }

        index += 1;
        while index < tokens.len() {
            let Some(flag) = word_token(tokens.get(index)) else {
                break;
            };
            if is_shell_command_flag(flag) {
                if let Some(script) = word_token(tokens.get(index + 1)) {
                    scripts.push(script.to_string());
                }
                break;
            }
            if !flag.starts_with('-') {
                break;
            }
            index += 1;
        }
    }
    scripts
}

fn is_shell_command_flag(flag: &str) -> bool {
    flag.strip_prefix('-')
        .is_some_and(|options| !options.starts_with('-') && options.contains('c'))
}

fn is_shell_program(word: &str) -> bool {
    let name = Path::new(word)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(word);
    matches!(name, "sh" | "bash" | "dash" | "zsh" | "ksh")
}

const COMMAND_OUTPUT_MAX_CHARS: usize = 12_000;
const COMMAND_OUTPUT_HEAD_CHARS: usize = 6_000;
const COMMAND_OUTPUT_TAIL_CHARS: usize = 4_000;
const COMMAND_OUTPUT_LONG_LINE_CHARS: usize = 4_096;
const COMMAND_OUTPUT_LONG_LINE_EDGE_CHARS: usize = 1_600;
const COMMAND_OUTPUT_BLANK_RUN_INLINE: usize = 8;

fn render_file_read_response(
    context_bundles: &ContextBundleStore,
    file_reads: &FileReadTracker,
    agent_id: &AgentId,
    exact_snapshots: &[crate::locks::FileSnapshot],
    changed_snapshots: &[crate::locks::FileSnapshot],
    unchanged_snapshots: &[crate::locks::FileSnapshot],
    failures: &[FileReadFailure],
) -> String {
    let exact_text = if should_bundle_file_read_response(exact_snapshots) {
        render_bundled_file_read_response(context_bundles, exact_snapshots, &[])
            .unwrap_or_else(|| render_file_read_response_inline(exact_snapshots, &[]))
    } else {
        render_file_read_response_inline(exact_snapshots, &[])
    };
    render_file_read_response_with_repeats(
        exact_text,
        file_reads,
        agent_id,
        changed_snapshots,
        unchanged_snapshots,
        failures,
    )
}

fn render_file_read_response_with_repeats(
    exact_text: String,
    file_reads: &FileReadTracker,
    agent_id: &AgentId,
    changed_snapshots: &[crate::locks::FileSnapshot],
    unchanged_snapshots: &[crate::locks::FileSnapshot],
    failures: &[FileReadFailure],
) -> String {
    let mut text = exact_text;
    if !changed_snapshots.is_empty() {
        text.push_str("\nRepeated file reads with changes\n");
        text.push_str(
            "These files changed since this agent's last mediated snapshot, so Work Leaf is sending diffs instead of full file text. Continue from the previous snapshot and request narrower related context if the diff is insufficient.\n",
        );
        for snapshot in changed_snapshots {
            render_changed_repeat_read_snapshot(&mut text, file_reads, agent_id, snapshot);
        }
    }
    if !unchanged_snapshots.is_empty() {
        text.push_str("\nRepeated file reads unchanged\n");
        text.push_str(
            "Work Leaf already sent this agent the exact text for these files, and the current digests still match. Full text is not resent; use the existing snapshot.\n",
        );
        for snapshot in unchanged_snapshots {
            text.push_str("- ");
            text.push_str(&snapshot.path.display().to_string());
            text.push_str(" (");
            text.push_str(&content_digest(&snapshot.text));
            text.push_str(")\n");
        }
    }
    if !failures.is_empty() {
        text.push_str("\nUnavailable file text\n");
        text.push_str(&render_file_read_failures(failures));
    }
    text
}

fn render_file_read_response_inline(
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

fn render_changed_repeat_read_snapshot(
    text: &mut String,
    file_reads: &FileReadTracker,
    agent_id: &AgentId,
    snapshot: &crate::locks::FileSnapshot,
) {
    text.push_str("\n--- ");
    text.push_str(&snapshot.path.display().to_string());
    text.push_str(" ---\n");
    text.push_str("current digest: ");
    text.push_str(&content_digest(&snapshot.text));
    text.push('\n');

    let Some(previous) = file_reads.snapshot_for(agent_id, &snapshot.path) else {
        render_untracked_refresh_snapshot(text, snapshot);
        return;
    };
    text.push_str("previous digest: ");
    text.push_str(&previous.digest);
    text.push('\n');
    match render_snapshot_diff(&snapshot.path, &previous.text, &snapshot.text) {
        Some(diff) => {
            text.push_str("status: changed since this agent's last snapshot\n");
            text.push_str(&diff);
            if !diff.ends_with('\n') {
                text.push('\n');
            }
        }
        None => {
            text.push_str("status: changed since this agent's last snapshot\n");
            text.push_str(
                "diff unavailable. Full text is not resent for repeated reads; request narrower related context or continue from the previous snapshot.\n",
            );
        }
    }
}

const MAX_INLINE_FILE_READ_BYTES: usize = 24 * 1024;
const MAX_INLINE_SINGLE_FILE_READ_BYTES: usize = 16 * 1024;

fn should_bundle_file_read_response(snapshots: &[crate::locks::FileSnapshot]) -> bool {
    let total = snapshots
        .iter()
        .map(|snapshot| snapshot.text.len())
        .sum::<usize>();
    total > MAX_INLINE_FILE_READ_BYTES
        || snapshots
            .iter()
            .any(|snapshot| snapshot.text.len() > MAX_INLINE_SINGLE_FILE_READ_BYTES)
}

fn render_bundled_file_read_response(
    context_bundles: &ContextBundleStore,
    snapshots: &[crate::locks::FileSnapshot],
    failures: &[FileReadFailure],
) -> Option<String> {
    let bundle_path = context_bundles.write(snapshots)?;
    let mut text = String::from("work-leaf file text\n");
    text.push_str("Exact file text is in an orchestrator context bundle instead of this chat to keep the agent session compact.\n");
    text.push_str("Context bundle: ");
    text.push_str(&bundle_path.display().to_string());
    text.push('\n');
    text.push_str("You may read this temporary bundle file for the exact mediated file text. Do not edit the bundle; project writes still require `@work-leaf edit`.\n");
    text.push_str("Bundled files:\n");
    for snapshot in snapshots {
        text.push_str("- ");
        text.push_str(&snapshot.path.display().to_string());
        text.push_str(" (");
        text.push_str(&content_digest(&snapshot.text));
        text.push_str(")\n");
    }
    if !failures.is_empty() {
        text.push_str("\nUnavailable file text\n");
        text.push_str(&render_file_read_failures(failures));
    }
    Some(text)
}

const MAX_AUTOMATIC_REFRESH_DIFF_BYTES: usize = 48 * 1024;
const MAX_AUTOMATIC_FULL_REFRESH_BYTES: usize = 8 * 1024;

fn render_file_refresh_response(
    agent_id: &AgentId,
    file_reads: &FileReadTracker,
    snapshots: &[crate::locks::FileSnapshot],
    failures: &[FileReadFailure],
) -> String {
    let mut text = String::from("work-leaf file refresh\n");
    text.push_str(
        "This is a compact refresh, not a patch to submit. It shows changes from the last file text this agent received. Repeated full-text refreshes are intentionally avoided to keep the session compact.\n",
    );

    for snapshot in snapshots {
        text.push_str("\n--- ");
        text.push_str(&snapshot.path.display().to_string());
        text.push_str(" ---\n");
        text.push_str("current digest: ");
        text.push_str(&content_digest(&snapshot.text));
        text.push('\n');

        let Some(previous) = file_reads.snapshot_for(agent_id, &snapshot.path) else {
            render_untracked_refresh_snapshot(&mut text, snapshot);
            continue;
        };

        text.push_str("previous digest: ");
        text.push_str(&previous.digest);
        text.push('\n');

        if previous.text == snapshot.text {
            text.push_str("status: unchanged since this agent's last snapshot\n");
            continue;
        }

        match render_snapshot_diff(&snapshot.path, &previous.text, &snapshot.text) {
            Some(diff) if diff.len() <= MAX_AUTOMATIC_REFRESH_DIFF_BYTES => {
                text.push_str("status: changed since this agent's last snapshot\n");
                text.push_str(&diff);
                if !diff.ends_with('\n') {
                    text.push('\n');
                }
            }
            Some(diff) => {
                text.push_str("status: changed since this agent's last snapshot\n");
                text.push_str("diff omitted: compact refresh would be ");
                text.push_str(&diff.len().to_string());
                text.push_str(
                    " bytes. Request narrower related context or continue from the previous snapshot if this file is still needed.\n",
                );
            }
            None => {
                text.push_str("status: changed since this agent's last snapshot\n");
                text.push_str(
                    "diff unavailable. Request narrower related context or continue from the previous snapshot if this file is still needed.\n",
                );
            }
        }
    }

    if !failures.is_empty() {
        text.push_str("\nUnavailable file text\n");
        text.push_str(&render_file_read_failures(failures));
    }
    text
}

fn render_untracked_refresh_snapshot(text: &mut String, snapshot: &crate::locks::FileSnapshot) {
    text.push_str("status: no previous snapshot recorded for this agent\n");
    if snapshot.text.len() <= MAX_AUTOMATIC_FULL_REFRESH_BYTES {
        text.push_str("work-leaf file text\n\n--- ");
        text.push_str(&snapshot.path.display().to_string());
        text.push_str(" ---\n");
        text.push_str(&snapshot.text);
        if !snapshot.text.ends_with('\n') {
            text.push('\n');
        }
    } else {
        text.push_str("current file text omitted: file is ");
        text.push_str(&snapshot.text.len().to_string());
        text.push_str(" bytes. Request mediated file text with `@work-leaf read ");
        text.push_str(&snapshot.path.display().to_string());
        text.push_str("` if this file is still needed.\n");
    }
}

fn render_snapshot_diff(path: &Path, previous: &str, current: &str) -> Option<String> {
    static DIFF_COUNTER: AtomicUsize = AtomicUsize::new(0);

    let counter = DIFF_COUNTER.fetch_add(1, Ordering::Relaxed);
    let base = std::env::temp_dir().join(format!(
        "work-leaf-refresh-diff-{}-{counter}",
        std::process::id()
    ));
    let previous_path = base.with_extension("previous");
    let current_path = base.with_extension("current");
    if fs::write(&previous_path, previous).is_err() || fs::write(&current_path, current).is_err() {
        let _ = fs::remove_file(&previous_path);
        let _ = fs::remove_file(&current_path);
        return None;
    }

    let output = Command::new("git")
        .args(["diff", "--no-index", "--no-color", "--unified=3", "--"])
        .arg(&previous_path)
        .arg(&current_path)
        .output()
        .ok();
    let _ = fs::remove_file(&previous_path);
    let _ = fs::remove_file(&current_path);

    let output = output?;
    if output.stdout.is_empty() {
        return Some(String::new());
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    Some(rewrite_diff_paths(&raw, path))
}

fn rewrite_diff_paths(diff: &str, path: &Path) -> String {
    let display = path.display();
    let mut rewritten = String::new();
    let mut old_header_rewritten = false;
    let mut new_header_rewritten = false;
    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            rewritten.push_str(&format!("diff --git a/{display} b/{display}\n"));
        } else if line.starts_with("--- ") && !old_header_rewritten {
            old_header_rewritten = true;
            rewritten.push_str(&format!("--- a/{display}\n"));
        } else if line.starts_with("+++ ") && old_header_rewritten && !new_header_rewritten {
            new_header_rewritten = true;
            rewritten.push_str(&format!("+++ b/{display}\n"));
        } else if line.starts_with("index ") {
            continue;
        } else {
            rewritten.push_str(line);
            rewritten.push('\n');
        }
    }
    rewritten
}

fn content_digest(text: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv64:{hash:016x}; bytes:{}", text.len())
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
    text.push_str(
        "\nnext: Reply with the next Work Leaf directive, such as `@work-leaf done`, `@work-leaf edit`, `@work-leaf read`, or another `@work-leaf locks run`. Keep any non-directive explanation brief.",
    );
    text.push_str("\nstdout:\n");
    text.push_str(&render_command_output(&output.stdout));
    text.push_str("stderr:\n");
    text.push_str(&render_command_output(&output.stderr));
    text
}

fn render_command_rejected(command: &str, locked_paths: &[PathBuf], diagnostic: &str) -> String {
    format!(
        "work-leaf command rejected\ncommand: {command}\nlocked paths: {}\nreason: {diagnostic}; this masks command failures, so Work Leaf did not run the command.\nRun the check normally and let Work Leaf capture the non-zero status, stdout, and stderr.",
        display_paths(locked_paths)
    )
}

fn render_command_output(output: &str) -> String {
    if output.is_empty() {
        return "<empty>\n".to_string();
    }

    let mut rendered = compact_blank_runs(output);
    rendered = compact_long_lines(&rendered);
    rendered = compact_total_chars(&rendered);
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
}

fn compact_blank_runs(output: &str) -> String {
    let mut compacted = String::new();
    let mut blank_run = 0_usize;
    for line in output.split_inclusive('\n') {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run <= COMMAND_OUTPUT_BLANK_RUN_INLINE {
                compacted.push_str(line);
            }
            continue;
        }
        if blank_run > COMMAND_OUTPUT_BLANK_RUN_INLINE {
            compacted.push_str(&format!(
                "[work-leaf compacted {} whitespace-only output lines]\n",
                blank_run - COMMAND_OUTPUT_BLANK_RUN_INLINE
            ));
        }
        blank_run = 0;
        compacted.push_str(line);
    }
    if blank_run > COMMAND_OUTPUT_BLANK_RUN_INLINE {
        compacted.push_str(&format!(
            "[work-leaf compacted {} whitespace-only output lines]\n",
            blank_run - COMMAND_OUTPUT_BLANK_RUN_INLINE
        ));
    }
    compacted
}

fn compact_long_lines(output: &str) -> String {
    let mut compacted = String::new();
    for line in output.split_inclusive('\n') {
        let had_newline = line.ends_with('\n');
        let content = line.strip_suffix('\n').unwrap_or(line);
        let content_chars = content.chars().count();
        if content_chars <= COMMAND_OUTPUT_LONG_LINE_CHARS {
            compacted.push_str(line);
            continue;
        }

        let omitted = content_chars.saturating_sub(COMMAND_OUTPUT_LONG_LINE_EDGE_CHARS * 2);
        compacted.push_str(&take_start_chars(
            content,
            COMMAND_OUTPUT_LONG_LINE_EDGE_CHARS,
        ));
        compacted.push_str(&format!(
            "\n[work-leaf compacted {omitted} characters from one long output line]\n"
        ));
        compacted.push_str(&take_end_chars(
            content,
            COMMAND_OUTPUT_LONG_LINE_EDGE_CHARS,
        ));
        if had_newline {
            compacted.push('\n');
        }
    }
    compacted
}

fn compact_total_chars(output: &str) -> String {
    let output_chars = output.chars().count();
    if output_chars <= COMMAND_OUTPUT_MAX_CHARS {
        return output.to_string();
    }

    let omitted =
        output_chars.saturating_sub(COMMAND_OUTPUT_HEAD_CHARS + COMMAND_OUTPUT_TAIL_CHARS);
    format!(
        "{}\n[work-leaf compacted {omitted} characters from command output]\n{}",
        take_start_chars(output, COMMAND_OUTPUT_HEAD_CHARS),
        take_end_chars(output, COMMAND_OUTPUT_TAIL_CHARS)
    )
}

fn take_start_chars(text: &str, count: usize) -> String {
    text.chars().take(count).collect()
}

fn take_end_chars(text: &str, count: usize) -> String {
    let mut chars = text.chars().rev().take(count).collect::<Vec<_>>();
    chars.reverse();
    chars.into_iter().collect()
}

fn render_patch_conflict_prompt(
    agent_id: &AgentId,
    file_reads: &FileReadTracker,
    files: &[PathBuf],
    diagnostic: &str,
    response: &FileReadResponse,
) -> String {
    let mut text = format!(
        "The orchestrator could not apply your patch.\nFiles: {}\n\nGit diagnostic:\n{}\n\nRebase your patch against the compact file refresh below.\n{}",
        display_paths(files),
        diagnostic,
        unified_diff_format_guidance()
    );
    append_file_refresh_response(&mut text, agent_id, file_reads, response);
    text
}

fn render_structured_edit_conflict_prompt(
    agent_id: &AgentId,
    file_reads: &FileReadTracker,
    files: &[PathBuf],
    diagnostic: &str,
    response: &FileReadResponse,
) -> String {
    let mut text = format!(
        "The orchestrator could not apply your edit.\nFiles: {}\n\nDiagnostic:\n{}\n\nRebase your exact edit blocks against the compact file refresh below.\n{}",
        display_paths(files),
        diagnostic,
        structured_edit_format_guidance()
    );
    append_file_refresh_response(&mut text, agent_id, file_reads, response);
    text
}

fn render_already_applied_patch_prompt(files: &[PathBuf]) -> String {
    let mut text = format!(
        "work-leaf patch already applied\nfiles: {}\n",
        display_paths(files)
    );
    text.push_str("The submitted patch is stale or already represented in the current repository state. Do not resend the same patch and do not rebase this same diff again.\n");
    text.push_str("Reread only the affected files if you still need context, then continue with your own feature work or emit `@work-leaf done` when ready.");
    text
}

fn render_patch_applied_prompt(files: &[PathBuf]) -> String {
    let mut text = format!("work-leaf patch applied\nfiles: {}\n", display_paths(files));
    text.push_str("Continue from the repository instructions.\n");
    text.push_str("Run checks that existed before your patch or checks you added yourself. Do not run another patch agent's focused tests as local validation; report those as integration conflicts unless your own source change clearly caused them.\n");
    text.push_str("Keep the shared worktree usable for the other patch agents: do not submit known-red, compile-breaking, or deliberately failing intermediate patches. If you prepared failing coverage first, include the test and the implementation needed for the shared tree to build in the same provisional patch.\n");
    text.push_str("Run any required or relevant checks through `@work-leaf locks run <path>... -- <command>` when the command may write files.\n");
    text.push_str("Keep locked command runs within five minutes unless the user authorizes a longer lock-holding command.\n");
    text.push_str("Provide additional edits if checks fail or more work is needed; emit `@work-leaf done` only when this patch is ready for review.");
    text
}

fn render_other_agent_test_command_prompt(blocked_paths: &[(PathBuf, OwnedPatchPath)]) -> String {
    let mut text = String::from("work-leaf command blocked by patch ownership\n");
    text.push_str("Do not run another patch agent's focused tests as local validation. Continue with checks that existed before your patch or checks you added yourself. If a broad integration check fails in another agent's test, report it as an integration conflict unless your own source change clearly caused it.\n");
    text.push_str("Blocked test paths:\n");
    for (path, owner) in blocked_paths {
        text.push_str("- ");
        text.push_str(&path.display().to_string());
        text.push_str(" owned by ");
        text.push_str(&owner.agent_id.to_string());
        text.push_str(" at ");
        text.push_str(&short_commit(&owner.commit));
        text.push('\n');
    }
    text
}

fn render_pending_command_changes_prompt(files: &[PathBuf], diff: &str) -> String {
    let mut text = format!(
        "work-leaf command left tracked working-tree changes\nfiles: {}\n",
        display_paths(files)
    );
    text.push_str("Review cannot start until command-produced tracked changes are saved in a provisional commit or reverted.\n");
    text.push_str("If these changes are required, submit them with `@work-leaf patch <reason>` using the diff below; the orchestrator accepts matching already-applied diffs and commits them.\n");
    text.push_str("In your next response, emit exactly one `@work-leaf patch` block or one revert patch block for these files, then stop your response immediately; do not repeat the same patch block and do not include `@work-leaf done` until Work Leaf reports the patch applied.\n");
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

fn render_file_update_prompt(
    agent_id: &AgentId,
    file_reads: &FileReadTracker,
    response: &FileReadResponse,
) -> String {
    let mut text = [
        "work-leaf file update",
        "Another agent changed files you previously read before you submitted a patch.",
        "Rebase any pending edit against the compact file refresh below.",
    ]
    .join("\n");
    append_file_refresh_response(&mut text, agent_id, file_reads, response);
    text
}

fn append_file_refresh_response(
    text: &mut String,
    agent_id: &AgentId,
    file_reads: &FileReadTracker,
    response: &FileReadResponse,
) {
    text.push_str("\n\n");
    text.push_str(&render_file_refresh_response(
        agent_id,
        file_reads,
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

fn short_commit(commit: &str) -> String {
    commit.chars().take(7).collect()
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

#[cfg(test)]
mod tests {
    use super::should_interrupt_after_streamed_directive;

    #[test]
    fn streamed_directive_interrupts_after_complete_edit_block() {
        let partial = "\
@work-leaf edit update value
*** Begin Patch
*** Update File: src/lib.rs
@@
-old
+new";
        assert!(!should_interrupt_after_streamed_directive(partial));

        let complete = format!("{partial}\n*** End Patch\n@work-leaf end");
        assert!(should_interrupt_after_streamed_directive(&complete));
    }

    #[test]
    fn streamed_directive_interrupts_after_read_request() {
        assert!(should_interrupt_after_streamed_directive(
            "I need context.\n@work-leaf read src/lib.rs tests/ui_harness.rs"
        ));
    }
}
