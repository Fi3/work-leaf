use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::agent::{AgentBackend, AgentError, AgentId};
use crate::locks::{FileAccessError, FileLockTable};

pub(crate) const ALREADY_APPLIED_PATCH_DIAGNOSTIC: &str =
    "patch already applied to current repository state";

pub(crate) fn is_already_applied_diagnostic(diagnostic: &str) -> bool {
    diagnostic.contains(ALREADY_APPLIED_PATCH_DIAGNOSTIC)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatchRequest {
    pub agent_id: AgentId,
    pub feature: String,
    pub reason: String,
    pub diff: String,
}

impl PatchRequest {
    pub fn new(
        agent_id: AgentId,
        feature: impl Into<String>,
        reason: impl Into<String>,
        diff: impl Into<String>,
    ) -> Self {
        Self {
            agent_id,
            feature: feature.into(),
            reason: reason.into(),
            diff: diff.into(),
        }
    }

    fn with_diff(mut self, diff: String) -> Self {
        self.diff = diff;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatchOutcome {
    pub commit: String,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct GitPatcher {
    root: PathBuf,
    locks: FileLockTable,
}

impl GitPatcher {
    pub fn new(root: PathBuf, locks: FileLockTable) -> Self {
        Self { root, locks }
    }

    pub fn apply(&self, request: PatchRequest) -> Result<PatchOutcome, PatchError> {
        let diff = extract_unified_diff(&request.diff);
        let request = request.with_diff(diff);
        let files = self.parse_patch_files(&request.diff)?;
        let lock_paths = patch_lock_paths(&files);
        let mut outcome = None;
        self.locks.with_write_locks(&lock_paths, || {
            outcome = Some(self.apply_with_locks(request, files.clone()));
            Ok(())
        })?;
        outcome.expect("patch operation runs while locks are held")
    }

    pub fn apply_edit(&self, request: PatchRequest) -> Result<PatchOutcome, PatchError> {
        let body = extract_structured_edit_patch(&request.diff);
        let patch = StructuredEditPatch::parse(&body, &self.locks)?;
        let files = patch.files.clone();
        let lock_paths = patch_lock_paths(&files);
        let mut outcome = None;
        self.locks.with_write_locks(&lock_paths, || {
            outcome = Some(self.apply_edit_with_locks(request.with_diff(body), patch));
            Ok(())
        })?;
        outcome.expect("patch operation runs while locks are held")
    }

    fn apply_with_locks(
        &self,
        request: PatchRequest,
        files: Vec<PathBuf>,
    ) -> Result<PatchOutcome, PatchError> {
        if let Some(diagnostic) = malformed_hunk_header_diagnostic(&request.diff) {
            return Err(PatchError::Conflict { files, diagnostic });
        }

        let check = self
            .git_with_stdin(["apply", "--recount", "--check", "-"], &request.diff)
            .map_err(PatchError::Git)?;
        if !check.status.success() {
            let reverse_check = self
                .git_with_stdin(
                    ["apply", "--reverse", "--recount", "--check", "-"],
                    &request.diff,
                )
                .map_err(PatchError::Git)?;
            if reverse_check.status.success()
                && self.has_head_diff(&files).map_err(PatchError::Git)?
            {
                self.git_add(&files).map_err(PatchError::Git)?;
                if !self.has_staged_diff(&files).map_err(PatchError::Git)? {
                    return Err(PatchError::Conflict {
                        files,
                        diagnostic: ALREADY_APPLIED_PATCH_DIAGNOSTIC.to_string(),
                    });
                }
                self.git_commit(&request, &files).map_err(PatchError::Git)?;
                let commit = self
                    .git_output(["rev-parse", "HEAD"])
                    .map_err(PatchError::Git)?;
                return Ok(PatchOutcome { commit, files });
            }
            if reverse_check.status.success() {
                return Err(PatchError::Conflict {
                    files,
                    diagnostic: ALREADY_APPLIED_PATCH_DIAGNOSTIC.to_string(),
                });
            }
            return Err(PatchError::Conflict {
                files,
                diagnostic: output_text(&check),
            });
        }

        self.git_with_stdin(["apply", "--recount", "-"], &request.diff)
            .and_then(|output| self.require_success(output, "git apply"))
            .map_err(PatchError::Git)?;

        self.git_add(&files).map_err(PatchError::Git)?;
        self.git_commit(&request, &files).map_err(PatchError::Git)?;
        let commit = self
            .git_output(["rev-parse", "HEAD"])
            .map_err(PatchError::Git)?;

        Ok(PatchOutcome { commit, files })
    }

    fn apply_edit_with_locks(
        &self,
        request: PatchRequest,
        patch: StructuredEditPatch,
    ) -> Result<PatchOutcome, PatchError> {
        let files = patch.files.clone();
        let changes = self.compute_structured_edit_changes(&patch)?;
        for change in changes {
            let path = self.root.join(&change.path);
            match change.content {
                Some(content) => {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).map_err(PatchError::Git)?;
                    }
                    fs::write(path, content).map_err(PatchError::Git)?;
                }
                None => {
                    fs::remove_file(path).map_err(PatchError::Git)?;
                }
            }
        }

        self.git_add(&files).map_err(PatchError::Git)?;
        if !self.has_staged_diff(&files).map_err(PatchError::Git)? {
            return Err(PatchError::Conflict {
                files,
                diagnostic: ALREADY_APPLIED_PATCH_DIAGNOSTIC.to_string(),
            });
        }
        self.git_commit(&request, &files).map_err(PatchError::Git)?;
        let commit = self
            .git_output(["rev-parse", "HEAD"])
            .map_err(PatchError::Git)?;

        Ok(PatchOutcome { commit, files })
    }

    fn compute_structured_edit_changes(
        &self,
        patch: &StructuredEditPatch,
    ) -> Result<Vec<StructuredFileChange>, PatchError> {
        let mut changes = Vec::new();
        for operation in &patch.operations {
            match operation {
                StructuredEditOperation::Add { path, content } => {
                    let absolute = self.root.join(path);
                    if absolute.exists() {
                        return Err(PatchError::Conflict {
                            files: vec![path.clone()],
                            diagnostic: format!(
                                "structured edit cannot add {} because it already exists",
                                path.display()
                            ),
                        });
                    }
                    changes.push(StructuredFileChange {
                        path: path.clone(),
                        content: Some(content.clone()),
                    });
                }
                StructuredEditOperation::Delete { path } => {
                    let absolute = self.root.join(path);
                    if !absolute.exists() {
                        return Err(PatchError::Conflict {
                            files: vec![path.clone()],
                            diagnostic: format!(
                                "structured edit cannot delete {} because it does not exist",
                                path.display()
                            ),
                        });
                    }
                    changes.push(StructuredFileChange {
                        path: path.clone(),
                        content: None,
                    });
                }
                StructuredEditOperation::Update { path, hunks } => {
                    let absolute = self.root.join(path);
                    let mut content =
                        fs::read_to_string(&absolute).map_err(|error| PatchError::Conflict {
                            files: vec![path.clone()],
                            diagnostic: format!(
                                "structured edit cannot read {} as UTF-8 text: {error}",
                                path.display()
                            ),
                        })?;
                    for (index, hunk) in hunks.iter().enumerate() {
                        content = apply_structured_hunk(&content, hunk).map_err(|diagnostic| {
                            PatchError::Conflict {
                                files: vec![path.clone()],
                                diagnostic: format!(
                                    "structured edit hunk {} for {} failed: {diagnostic}",
                                    index + 1,
                                    path.display()
                                ),
                            }
                        })?;
                    }
                    changes.push(StructuredFileChange {
                        path: path.clone(),
                        content: Some(content),
                    });
                }
            }
        }
        Ok(changes)
    }

    fn parse_patch_files(&self, diff: &str) -> Result<Vec<PathBuf>, PatchError> {
        let mut files = Vec::new();
        for line in diff.lines() {
            if let Some(rest) = line.strip_prefix("+++ ") {
                if let Some(path) = parse_patch_path(rest, "b/") {
                    files.push(self.locks.normalize_path(&path)?);
                }
            } else if let Some(rest) = line.strip_prefix("--- ") {
                if let Some(path) = parse_patch_path(rest, "a/") {
                    files.push(self.locks.normalize_path(&path)?);
                }
            } else if let Some(rest) = line.strip_prefix("diff --git ") {
                let mut parts = rest.split_whitespace();
                let old = parts.next();
                let new = parts.next();
                if let Some(path) = new.and_then(|part| parse_patch_path(part, "b/")) {
                    files.push(self.locks.normalize_path(&path)?);
                } else if let Some(path) = old.and_then(|part| parse_patch_path(part, "a/")) {
                    files.push(self.locks.normalize_path(&path)?);
                }
            }
        }
        files.sort();
        files.dedup();
        if files.is_empty() {
            Err(PatchError::NoFiles)
        } else {
            Ok(files)
        }
    }

    fn git_add(&self, files: &[PathBuf]) -> Result<(), std::io::Error> {
        let mut command = Command::new("git");
        command.current_dir(&self.root).arg("add").arg("--");
        for file in files {
            command.arg(file);
        }
        self.require_success(command.output()?, "git add")
            .map(|_| ())
    }

    fn git_commit(&self, request: &PatchRequest, files: &[PathBuf]) -> Result<(), std::io::Error> {
        let subject = format!(
            "UPDATE apply {} patch from {}",
            request.feature, request.agent_id
        );
        let context = render_patch_context(request, files);
        let body = format!(
            "Agent-ID: {}\nFeature: {}\nReason: {}\nContext: {}",
            request.agent_id, request.feature, request.reason, context
        );
        self.require_success(
            Command::new("git")
                .current_dir(&self.root)
                .args(["commit", "-m", &subject, "-m", &body])
                .output()?,
            "git commit",
        )
        .map(|_| ())
    }

    fn git_output<const N: usize>(&self, args: [&str; N]) -> Result<String, std::io::Error> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()?;
        self.require_success(output, "git output")
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn has_head_diff(&self, files: &[PathBuf]) -> Result<bool, std::io::Error> {
        let mut command = Command::new("git");
        command
            .current_dir(&self.root)
            .args(["diff", "--quiet", "HEAD", "--"]);
        for file in files {
            command.arg(file);
        }
        Ok(!command.status()?.success())
    }

    fn has_staged_diff(&self, files: &[PathBuf]) -> Result<bool, std::io::Error> {
        let mut command = Command::new("git");
        command
            .current_dir(&self.root)
            .args(["diff", "--cached", "--quiet", "--"]);
        for file in files {
            command.arg(file);
        }
        Ok(!command.status()?.success())
    }

    fn git_with_stdin<const N: usize>(
        &self,
        args: [&str; N],
        stdin: &str,
    ) -> Result<Output, std::io::Error> {
        let mut child = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(child_stdin) = child.stdin.as_mut() {
            child_stdin.write_all(stdin.as_bytes())?;
        }

        child.wait_with_output()
    }

    fn require_success(&self, output: Output, context: &str) -> Result<Output, std::io::Error> {
        if output.status.success() {
            Ok(output)
        } else {
            Err(std::io::Error::other(format!(
                "{context} failed: {}",
                output_text(&output)
            )))
        }
    }
}

fn patch_lock_paths(files: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = files.to_vec();
    paths.push(PathBuf::from("."));
    paths.sort();
    paths.dedup();
    paths
}

#[derive(Clone, Debug)]
struct StructuredEditPatch {
    operations: Vec<StructuredEditOperation>,
    files: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
enum StructuredEditOperation {
    Add {
        path: PathBuf,
        content: String,
    },
    Update {
        path: PathBuf,
        hunks: Vec<StructuredEditHunk>,
    },
    Delete {
        path: PathBuf,
    },
}

#[derive(Clone, Debug)]
struct StructuredEditHunk {
    old: String,
    new: String,
}

#[derive(Clone, Debug)]
struct StructuredFileChange {
    path: PathBuf,
    content: Option<String>,
}

impl StructuredEditPatch {
    fn parse(raw: &str, locks: &FileLockTable) -> Result<Self, PatchError> {
        let lines = raw
            .lines()
            .map(|line| line.trim_end_matches('\r'))
            .collect::<Vec<_>>();
        let Some(begin) = lines
            .iter()
            .position(|line| line.trim() == "*** Begin Patch")
        else {
            return Err(PatchError::NoFiles);
        };
        let Some(end) = lines
            .iter()
            .rposition(|line| line.trim() == "*** End Patch")
        else {
            return Err(PatchError::Conflict {
                files: Vec::new(),
                diagnostic: "structured edit is missing `*** End Patch`".to_string(),
            });
        };
        if end <= begin {
            return Err(PatchError::NoFiles);
        }

        let mut operations = Vec::new();
        let mut files = BTreeSet::new();
        let mut index = begin + 1;
        while index < end {
            let line = lines[index];
            if line.trim().is_empty() {
                index += 1;
                continue;
            }
            if let Some(path) = line.strip_prefix("*** Add File: ") {
                let path = normalize_structured_edit_path(locks, path)?;
                insert_structured_edit_file(&mut files, &path)?;
                index += 1;
                let mut content = String::new();
                while index < end && !lines[index].starts_with("*** ") {
                    let next = lines[index];
                    let Some(rest) = next.strip_prefix('+') else {
                        return Err(PatchError::Conflict {
                            files: vec![path],
                            diagnostic: "structured add-file lines must start with `+`".to_string(),
                        });
                    };
                    content.push_str(rest);
                    content.push('\n');
                    index += 1;
                }
                operations.push(StructuredEditOperation::Add { path, content });
                continue;
            }
            if let Some(path) = line.strip_prefix("*** Delete File: ") {
                let path = normalize_structured_edit_path(locks, path)?;
                insert_structured_edit_file(&mut files, &path)?;
                operations.push(StructuredEditOperation::Delete { path });
                index += 1;
                continue;
            }
            if let Some(path) = line.strip_prefix("*** Update File: ") {
                let path = normalize_structured_edit_path(locks, path)?;
                insert_structured_edit_file(&mut files, &path)?;
                index += 1;
                let mut hunks = Vec::new();
                while index < end && !lines[index].starts_with("*** ") {
                    if lines[index].trim().is_empty() {
                        index += 1;
                        continue;
                    }
                    if !lines[index].starts_with("@@") {
                        return Err(PatchError::Conflict {
                            files: vec![path.clone()],
                            diagnostic: "structured update sections must contain `@@` hunk headers"
                                .to_string(),
                        });
                    }
                    index += 1;
                    let mut old = String::new();
                    let mut new = String::new();
                    while index < end
                        && !lines[index].starts_with("@@")
                        && !lines[index].starts_with("*** ")
                    {
                        let next = lines[index];
                        if next.starts_with("\\ ") {
                            return Err(PatchError::Conflict {
                                files: vec![path.clone()],
                                diagnostic: "structured edits do not support no-newline markers"
                                    .to_string(),
                            });
                        }
                        let Some((prefix, rest)) = split_structured_hunk_line(next) else {
                            return Err(PatchError::Conflict {
                                files: vec![path.clone()],
                                diagnostic:
                                    "structured hunk lines must start with space, `-`, or `+`"
                                        .to_string(),
                            });
                        };
                        match prefix {
                            ' ' => {
                                old.push_str(rest);
                                old.push('\n');
                                new.push_str(rest);
                                new.push('\n');
                            }
                            '-' => {
                                old.push_str(rest);
                                old.push('\n');
                            }
                            '+' => {
                                new.push_str(rest);
                                new.push('\n');
                            }
                            _ => unreachable!("split_structured_hunk_line returns known prefixes"),
                        }
                        index += 1;
                    }
                    if old.is_empty() {
                        return Err(PatchError::Conflict {
                            files: vec![path.clone()],
                            diagnostic:
                                "structured update hunk must include context or removed text"
                                    .to_string(),
                        });
                    }
                    hunks.push(StructuredEditHunk { old, new });
                }
                if hunks.is_empty() {
                    return Err(PatchError::Conflict {
                        files: vec![path.clone()],
                        diagnostic: "structured update must contain at least one hunk".to_string(),
                    });
                }
                operations.push(StructuredEditOperation::Update { path, hunks });
                continue;
            }

            return Err(PatchError::Conflict {
                files: Vec::new(),
                diagnostic: format!("unknown structured edit header `{line}`"),
            });
        }

        if operations.is_empty() {
            return Err(PatchError::NoFiles);
        }
        Ok(Self {
            operations,
            files: files.into_iter().collect(),
        })
    }
}

fn normalize_structured_edit_path(locks: &FileLockTable, raw: &str) -> Result<PathBuf, PatchError> {
    let path = raw.trim();
    if path.is_empty() {
        return Err(PatchError::NoFiles);
    }
    locks
        .normalize_path(&PathBuf::from(path))
        .map_err(PatchError::FileAccess)
}

fn insert_structured_edit_file(
    files: &mut BTreeSet<PathBuf>,
    path: &Path,
) -> Result<(), PatchError> {
    if files.insert(path.to_path_buf()) {
        Ok(())
    } else {
        Err(PatchError::Conflict {
            files: vec![path.to_path_buf()],
            diagnostic:
                "structured edit touches the same file more than once; combine hunks under one file header"
                    .to_string(),
        })
    }
}

fn split_structured_hunk_line(line: &str) -> Option<(char, &str)> {
    let mut chars = line.chars();
    let prefix = chars.next()?;
    if matches!(prefix, ' ' | '-' | '+') {
        Some((prefix, chars.as_str()))
    } else {
        None
    }
}

fn apply_structured_hunk(content: &str, hunk: &StructuredEditHunk) -> Result<String, String> {
    let matches = content.match_indices(&hunk.old).collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Err("old block was not found in current file text".to_string()),
        [single] => {
            let start = single.0;
            let end = start + hunk.old.len();
            let mut updated = String::with_capacity(
                content.len() + hunk.new.len().saturating_sub(hunk.old.len()),
            );
            updated.push_str(&content[..start]);
            updated.push_str(&hunk.new);
            updated.push_str(&content[end..]);
            Ok(updated)
        }
        many => Err(format!(
            "ambiguous old block matched {} locations; include more unchanged context lines",
            many.len()
        )),
    }
}

fn extract_structured_edit_patch(raw: &str) -> String {
    let lines = raw.lines().collect::<Vec<_>>();
    let Some((start, prefix)) = lines
        .iter()
        .enumerate()
        .find_map(|(index, line)| structured_edit_line_prefix(line).map(|prefix| (index, prefix)))
    else {
        return raw.to_string();
    };

    let mut body = String::new();
    for line in &lines[start..] {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            continue;
        }
        if let Some(stripped) = line.strip_prefix(&prefix) {
            body.push_str(stripped);
        } else {
            body.push_str(line);
        }
        body.push('\n');
    }
    body
}

fn structured_edit_line_prefix(line: &str) -> Option<String> {
    if line.starts_with("*** Begin Patch") {
        return Some(String::new());
    }
    let trimmed = line.trim_start();
    if trimmed.starts_with("*** Begin Patch") {
        let indent_len = line.len() - trimmed.len();
        return Some(line[..indent_len].to_string());
    }
    let quote_trimmed = line.trim_start_matches([' ', '\t']);
    if quote_trimmed.starts_with("> *** Begin Patch") {
        let quote_start = line.len() - quote_trimmed.len();
        let prefix_len = quote_start + 2;
        return Some(line[..prefix_len].to_string());
    }
    None
}

fn extract_unified_diff(raw: &str) -> String {
    let lines = raw.lines().collect::<Vec<_>>();
    let Some((start, prefix)) = lines
        .iter()
        .enumerate()
        .find_map(|(index, line)| diff_line_prefix(line).map(|prefix| (index, prefix)))
    else {
        return raw.to_string();
    };

    let mut diff = String::new();
    for line in &lines[start..] {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            continue;
        }
        if let Some(stripped) = line.strip_prefix(&prefix) {
            diff.push_str(stripped);
        } else {
            diff.push_str(line);
        }
        diff.push('\n');
    }
    diff
}

fn diff_line_prefix(line: &str) -> Option<String> {
    for marker in ["diff --git ", "--- ", "Index: "] {
        if line.starts_with(marker) {
            return Some(String::new());
        }
    }

    let trimmed = line.trim_start();
    if ["diff --git ", "--- ", "Index: "]
        .iter()
        .any(|marker| trimmed.starts_with(marker))
    {
        let indent_len = line.len() - trimmed.len();
        return Some(line[..indent_len].to_string());
    }

    let quote_trimmed = line.trim_start_matches([' ', '\t']);
    if let Some(after_quote) = quote_trimmed.strip_prefix("> ")
        && ["diff --git ", "--- ", "Index: "]
            .iter()
            .any(|marker| after_quote.starts_with(marker))
    {
        let quote_start = line.len() - quote_trimmed.len();
        let prefix_len = quote_start + 2;
        return Some(line[..prefix_len].to_string());
    }

    None
}

fn render_patch_context(request: &PatchRequest, files: &[PathBuf]) -> String {
    let files = files
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let additions = request
        .diff
        .lines()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .count();
    let removals = request
        .diff
        .lines()
        .filter(|line| line.starts_with('-') && !line.starts_with("---"))
        .count();
    let mechanism = if request.diff.contains("*** Begin Patch") {
        "The orchestrator matched exact edit blocks against the current file text under the repository write locks, wrote the resulting files, staged exactly the touched files, and saved this provisional commit so review and linearization can reason about what changed and why."
    } else {
        "The orchestrator validated with `git apply --recount --check`, applied the submitted unified diff under the repository write locks, staged exactly the touched files, and saved this provisional commit so review and linearization can reason about what changed and why."
    };
    format!(
        "The orchestrator applied this provisional patch for {} while the agent was working on feature `{}`. The agent stated the reason as `{}`. The patch touched {} and changed the working tree with {} added line(s) and {} removed line(s). {}",
        request.agent_id, request.feature, request.reason, files, additions, removals, mechanism
    )
}

#[derive(Debug)]
pub struct PatchCoordinator<B> {
    patcher: GitPatcher,
    backend: B,
}

impl<B> PatchCoordinator<B>
where
    B: AgentBackend,
{
    pub fn new(patcher: GitPatcher, backend: B) -> Self {
        Self { patcher, backend }
    }

    pub fn into_backend(self) -> B {
        self.backend
    }

    pub fn submit(&mut self, request: PatchRequest) -> Result<PatchOutcome, PatchError> {
        let agent_id = request.agent_id.clone();
        match self.patcher.apply(request) {
            Ok(outcome) => Ok(outcome),
            Err(PatchError::Conflict { files, diagnostic }) => {
                let files_text = files
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let prompt = if is_already_applied_diagnostic(&diagnostic) {
                    format!(
                        "work-leaf patch already applied\nfiles: {files_text}\nThe submitted patch is stale or already represented in the current repository state. Do not resend the same patch. Reread only the affected files if you still need context, then continue with your own feature or emit `@work-leaf done` when ready."
                    )
                } else {
                    format!(
                        "The orchestrator could not apply your patch.\nFiles: {files_text}\n\nGit diagnostic:\n{diagnostic}\n\n{}",
                        unified_diff_format_guidance()
                    )
                };
                self.backend
                    .send(&agent_id, &prompt)
                    .map_err(PatchError::Agent)?;
                Err(PatchError::Conflict { files, diagnostic })
            }
            Err(PatchError::NoFiles) => {
                let prompt = render_no_files_prompt();
                self.backend
                    .send(&agent_id, &prompt)
                    .map_err(PatchError::Agent)?;
                Err(PatchError::NoFiles)
            }
            Err(error) => Err(error),
        }
    }
}

pub(crate) fn render_no_files_prompt() -> String {
    format!(
        "The orchestrator could not apply your patch because the patch body did not include recognizable unified diff file headers such as `diff --git a/path b/path`, `--- a/path`, and `+++ b/path`.\n\n{}",
        unified_diff_format_guidance()
    )
}

pub(crate) fn unified_diff_format_guidance() -> &'static str {
    "Resend the complete unified diff through `@work-leaf patch <reason>` followed by the patch body and `@work-leaf end`. Do not use placeholder `@@` hunk headers. Every hunk must use real unified-diff line ranges such as `@@ -old_start,old_count +new_start,new_count @@` from the current file text."
}

pub(crate) fn render_structured_edit_no_files_prompt() -> String {
    format!(
        "The orchestrator could not apply your edit because the body did not include a structured edit file header such as `*** Update File: path`, `*** Add File: path`, or `*** Delete File: path`.\n\n{}",
        structured_edit_format_guidance()
    )
}

pub(crate) fn structured_edit_format_guidance() -> &'static str {
    "Resend the complete edit through `@work-leaf edit <reason>` followed by an apply-patch-style body and `@work-leaf end`. Use `*** Begin Patch`, one or more `*** Update File: path` sections, `@@` hunk separators without line numbers, exact unchanged context lines prefixed with a space, old lines prefixed with `-`, new lines prefixed with `+`, and `*** End Patch`. Include enough unchanged context for each old block to match exactly one place in the current file."
}

fn malformed_hunk_header_diagnostic(diff: &str) -> Option<String> {
    for (index, line) in diff.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.starts_with("@@") && !valid_unified_hunk_header(line) {
            return Some(format!(
                "malformed unified diff: hunk header on line {} is missing unified diff line ranges; use `@@ -old_start,old_count +new_start,new_count @@` with real line numbers from the current file text",
                index + 1
            ));
        }
    }
    None
}

fn valid_unified_hunk_header(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("@@ ") else {
        return false;
    };
    let Some((ranges, _heading)) = rest.split_once(" @@") else {
        return false;
    };
    let mut parts = ranges.split_whitespace();
    let Some(old_range) = parts.next() else {
        return false;
    };
    let Some(new_range) = parts.next() else {
        return false;
    };
    old_range.strip_prefix('-').is_some_and(valid_unified_range)
        && new_range.strip_prefix('+').is_some_and(valid_unified_range)
}

fn valid_unified_range(range: &str) -> bool {
    let mut parts = range.split(',');
    let Some(start) = parts.next() else {
        return false;
    };
    if start.is_empty() || !start.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    match (parts.next(), parts.next()) {
        (None, None) => true,
        (Some(count), None) => !count.is_empty() && count.chars().all(|ch| ch.is_ascii_digit()),
        _ => false,
    }
}

fn parse_patch_path(raw: &str, prefix: &str) -> Option<PathBuf> {
    let path = raw.split('\t').next().unwrap_or(raw);
    if path == "/dev/null" {
        return None;
    }
    path.strip_prefix(prefix).map(PathBuf::from)
}

fn output_text(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    [stdout.trim(), stderr.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug)]
pub enum PatchError {
    NoFiles,
    Conflict {
        files: Vec<PathBuf>,
        diagnostic: String,
    },
    Agent(AgentError),
    FileAccess(FileAccessError),
    Git(std::io::Error),
}

impl fmt::Display for PatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFiles => formatter.write_str("patch does not touch any files"),
            Self::Conflict { diagnostic, .. } => write!(formatter, "patch conflict: {diagnostic}"),
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::FileAccess(error) => write!(formatter, "{error}"),
            Self::Git(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for PatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Agent(error) => Some(error),
            Self::FileAccess(error) => Some(error),
            Self::Git(error) => Some(error),
            _ => None,
        }
    }
}

impl From<FileAccessError> for PatchError {
    fn from(error: FileAccessError) -> Self {
        Self::FileAccess(error)
    }
}

impl From<std::io::Error> for PatchError {
    fn from(error: std::io::Error) -> Self {
        Self::Git(error)
    }
}
