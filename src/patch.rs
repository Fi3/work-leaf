use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::agent::{AgentError, AgentId};
use crate::codex::AgentBackend;
use crate::instructions::{load_project_instructions, validation_checks};
use crate::locks::{FileAccessError, FileLockTable};

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
        let files = self.parse_patch_files(&request.diff)?;
        let mut outcome = None;
        self.locks.with_write_locks(&files, || {
            outcome = Some(self.apply_with_locks(request, files.clone()));
            Ok(())
        })?;
        outcome.expect("patch operation runs while locks are held")
    }

    fn apply_with_locks(
        &self,
        request: PatchRequest,
        files: Vec<PathBuf>,
    ) -> Result<PatchOutcome, PatchError> {
        let check = self
            .git_with_stdin(["apply", "--check", "-"], &request.diff)
            .map_err(PatchError::Git)?;
        if !check.status.success() {
            return Err(PatchError::Conflict {
                files,
                diagnostic: output_text(&check),
            });
        }

        self.git_with_stdin(["apply", "-"], &request.diff)
            .and_then(|output| self.require_success(output, "git apply"))
            .map_err(PatchError::Git)?;

        if let Err(error) = self.run_required_checks(&files) {
            let _ = self.reverse_patch(&request.diff);
            return Err(error);
        }

        self.git_add(&files).map_err(PatchError::Git)?;
        self.git_commit(&request, &files).map_err(PatchError::Git)?;
        let commit = self
            .git_output(["rev-parse", "HEAD"])
            .map_err(PatchError::Git)?;

        Ok(PatchOutcome { commit, files })
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

    fn run_required_checks(&self, files: &[PathBuf]) -> Result<(), PatchError> {
        let instructions = load_project_instructions(&self.root).map_err(PatchError::Git)?;
        for check in validation_checks(&self.root, &instructions) {
            let output = Command::new(check.program())
                .current_dir(&self.root)
                .args(check.args())
                .output()
                .map_err(PatchError::Git)?;
            if !output.status.success() {
                return Err(PatchError::ValidationFailed {
                    files: files.to_vec(),
                    command: check.command_line().to_string(),
                    diagnostic: output_text(&output),
                });
            }
        }
        Ok(())
    }

    fn reverse_patch(&self, diff: &str) -> Result<(), std::io::Error> {
        let check = self.git_with_stdin(["apply", "--reverse", "--check", "-"], diff)?;
        if !check.status.success() {
            return Ok(());
        }
        self.git_with_stdin(["apply", "--reverse", "-"], diff)
            .and_then(|output| self.require_success(output, "git apply --reverse"))
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
    format!(
        "The orchestrator applied this provisional patch for {} while the agent was working on feature `{}`. The agent stated the reason as `{}`. The patch touched {} and changed the working tree with {} added line(s) and {} removed line(s). The orchestrator validated with `git apply --check`, applied the submitted unified diff under the repository write locks, staged exactly the touched files, and saved this provisional commit so review and linearization can reason about what changed and why.",
        request.agent_id, request.feature, request.reason, files, additions, removals
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
                let prompt = format!(
                    "The orchestrator could not apply your patch.\nFiles: {}\n\nGit diagnostic:\n{}\n\nPlease provide a corrected unified diff patch.",
                    files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    diagnostic
                );
                self.backend
                    .send(&agent_id, &prompt)
                    .map_err(PatchError::Agent)?;
                Err(PatchError::Conflict { files, diagnostic })
            }
            Err(PatchError::ValidationFailed {
                files,
                command,
                diagnostic,
            }) => {
                let prompt = format!(
                    "The orchestrator rejected your patch because repository validation failed.\nFiles: {}\nCommand: {}\n\nDiagnostic:\n{}\n\nPlease provide a corrected unified diff patch.",
                    files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    command,
                    diagnostic
                );
                self.backend
                    .send(&agent_id, &prompt)
                    .map_err(PatchError::Agent)?;
                Err(PatchError::ValidationFailed {
                    files,
                    command,
                    diagnostic,
                })
            }
            Err(error) => Err(error),
        }
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
    ValidationFailed {
        files: Vec<PathBuf>,
        command: String,
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
            Self::ValidationFailed {
                command,
                diagnostic,
                ..
            } => write!(
                formatter,
                "patch validation `{command}` failed: {diagnostic}"
            ),
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
