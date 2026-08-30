use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};

pub const CONFIG_ENV: &str = "WORK_LEAF_OBSERVER_CONFIG";
pub const PARENT_INVOCATION_ENV: &str = "WORK_LEAF_OBSERVER_PARENT_INVOCATION";
pub const PRIMARY_MARKER_ENV: &str = "WORK_LEAF_OBSERVER_PRIMARY_MARKER";
pub const ROLE_ENV: &str = "WORK_LEAF_OBSERVER_ROLE";
pub const PROVIDER_USAGE_GRACE_ENV: &str = "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS";
pub const PROVIDER_USAGE_GRACE_OUTPUT_RESUME_ENV: &str =
    "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME";
const CONFIG_FILE: &str = "observer-config.json";
const LOCKED_COMMAND_PREFIX: &str = "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; (";
const MAX_PROVIDER_USAGE_GRACE_MS: u64 = 120_000;

#[derive(Debug)]
pub struct ObserverError {
    message: String,
}

impl ObserverError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ObserverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ObserverError {}

impl From<io::Error> for ObserverError {
    fn from(error: io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<serde_json::Error> for ObserverError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}

pub type ObserverResult<T> = Result<T, ObserverError>;

#[derive(Clone, Debug)]
pub struct InitSpec {
    pub root: PathBuf,
    pub study_id: String,
    pub pair_id: String,
    pub condition: String,
    pub run_id: String,
    pub real_codex: PathBuf,
    pub real_sh: PathBuf,
    pub real_cargo: PathBuf,
    pub base_commit: String,
    pub experiment_commit: String,
    pub model: String,
    pub effort: String,
    pub require_complete_provider_usage: bool,
    pub observer_executable: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CaptureConfig {
    pub schema_version: u32,
    pub root: PathBuf,
    pub study_id: String,
    pub pair_id: String,
    pub condition: String,
    pub run_id: String,
    pub real_codex: PathBuf,
    pub real_codex_sha256: String,
    pub real_codex_version: String,
    pub real_sh: PathBuf,
    pub real_sh_sha256: String,
    #[serde(default)]
    pub real_cargo: Option<PathBuf>,
    #[serde(default)]
    pub real_cargo_sha256: Option<String>,
    pub base_commit: String,
    pub experiment_commit: String,
    pub model: String,
    pub effort: String,
    #[serde(default)]
    pub require_complete_provider_usage: bool,
    pub observer_executable: PathBuf,
    pub observer_sha256: String,
    pub primary_invocation_marker: String,
    pub created_monotonic_ns: u128,
    pub created_unix_ns: u128,
}

impl CaptureConfig {
    pub fn load(path: &Path) -> ObserverResult<Self> {
        let config: Self = serde_json::from_slice(&fs::read(path)?)?;
        if config.schema_version != 1 {
            return Err(ObserverError::new(format!(
                "unsupported observer schema version {}",
                config.schema_version
            )));
        }
        Ok(config)
    }

    pub fn path(&self) -> PathBuf {
        self.root.join(CONFIG_FILE)
    }
}

pub fn initialize(spec: InitSpec) -> ObserverResult<CaptureConfig> {
    fs::create_dir_all(&spec.root)?;
    let root = fs::canonicalize(&spec.root)?;
    let real_codex = fs::canonicalize(&spec.real_codex).map_err(|error| {
        ObserverError::new(format!(
            "cannot resolve real Codex executable {}: {error}",
            spec.real_codex.display()
        ))
    })?;
    let real_sh = fs::canonicalize(&spec.real_sh).map_err(|error| {
        ObserverError::new(format!(
            "cannot resolve real shell executable {}: {error}",
            spec.real_sh.display()
        ))
    })?;
    let real_cargo = executable_entrypoint(&spec.real_cargo, "Cargo")?;
    let resolved_real_cargo = fs::canonicalize(&real_cargo).map_err(|error| {
        ObserverError::new(format!(
            "cannot resolve real Cargo executable {}: {error}",
            spec.real_cargo.display()
        ))
    })?;
    let observer_executable = fs::canonicalize(&spec.observer_executable).map_err(|error| {
        ObserverError::new(format!(
            "cannot resolve observer executable {}: {error}",
            spec.observer_executable.display()
        ))
    })?;
    if real_codex == observer_executable || resolved_real_cargo == observer_executable {
        return Err(ObserverError::new(
            "a real executable resolves to the observer itself",
        ));
    }

    let version = Command::new(&real_codex)
        .arg("--version")
        .output()
        .map(|output| {
            let mut text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.is_empty() {
                text = String::from_utf8_lossy(&output.stderr).trim().to_string();
            }
            text
        })
        .unwrap_or_else(|error| format!("unavailable: {error}"));
    let config = CaptureConfig {
        schema_version: 1,
        root,
        study_id: spec.study_id,
        pair_id: spec.pair_id,
        condition: spec.condition,
        run_id: spec.run_id,
        real_codex_sha256: sha256_file(&real_codex)?,
        real_codex_version: version,
        real_codex,
        real_sh_sha256: sha256_file(&real_sh)?,
        real_sh,
        real_cargo_sha256: Some(sha256_file(&real_cargo)?),
        real_cargo: Some(real_cargo),
        base_commit: spec.base_commit,
        experiment_commit: spec.experiment_commit,
        model: spec.model,
        effort: spec.effort,
        require_complete_provider_usage: spec.require_complete_provider_usage,
        observer_sha256: sha256_file(&observer_executable)?,
        observer_executable,
        primary_invocation_marker: random_marker()?,
        created_monotonic_ns: monotonic_time_ns(),
        created_unix_ns: unix_time_ns(),
    };
    write_json_atomic(&config.path(), &config)?;
    install_proxy_links(&config)?;
    write_manifest(&config)?;
    harden_artifact_permissions(&config.root)?;
    Ok(config)
}

fn executable_entrypoint(path: &Path, label: &str) -> ObserverResult<PathBuf> {
    let file_name = path.file_name().ok_or_else(|| {
        ObserverError::new(format!(
            "cannot resolve real {label} executable {}: path has no file name",
            path.display()
        ))
    })?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let parent = fs::canonicalize(&parent).map_err(|error| {
        ObserverError::new(format!(
            "cannot resolve real {label} executable {}: {error}",
            path.display()
        ))
    })?;
    let entrypoint = parent.join(file_name);
    fs::metadata(&entrypoint).map_err(|error| {
        ObserverError::new(format!(
            "cannot resolve real {label} executable {}: {error}",
            path.display()
        ))
    })?;
    Ok(entrypoint)
}

fn install_proxy_links(config: &CaptureConfig) -> ObserverResult<()> {
    let proxy_dir = config.root.join("proxy-bin");
    fs::create_dir_all(&proxy_dir)?;
    for name in ["codex", "sh"] {
        let path = proxy_dir.join(name);
        if path.symlink_metadata().is_ok() {
            fs::remove_file(&path)?;
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&config.observer_executable, &path)?;
        #[cfg(not(unix))]
        fs::copy(&config.observer_executable, &path)?;
    }
    Ok(())
}

fn write_manifest(config: &CaptureConfig) -> ObserverResult<()> {
    let manifest = json!({
        "schema_version": 1,
        "study_id": config.study_id,
        "pair_id": config.pair_id,
        "run_id": config.run_id,
        "condition": config.condition,
        "base_commit": config.base_commit,
        "experiment_commit": config.experiment_commit,
        "model": config.model,
        "reasoning_effort": config.effort,
        "require_complete_provider_usage": config.require_complete_provider_usage,
        "real_codex": {
            "path": config.real_codex,
            "sha256": config.real_codex_sha256,
            "version": config.real_codex_version,
        },
        "real_sh": {
            "path": config.real_sh,
            "sha256": config.real_sh_sha256,
        },
        "real_cargo": {
            "path": config.real_cargo,
            "sha256": config.real_cargo_sha256,
        },
        "observer": {
            "path": config.observer_executable,
            "sha256": config.observer_sha256,
        },
        "primary_invocation_marker_sha256": sha256_bytes(config.primary_invocation_marker.as_bytes()),
        "created_monotonic_ns": config.created_monotonic_ns.to_string(),
        "created_unix_ns": config.created_unix_ns.to_string(),
    });
    write_json_atomic(&config.root.join("manifest.json"), &manifest)
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureKind {
    AppServer,
    ExecJson,
    LockedCommand,
}

impl CaptureKind {
    fn artifact_directory(self) -> &'static str {
        match self {
            Self::AppServer => "app-server",
            Self::ExecJson => "exec-json",
            Self::LockedCommand => "locked-commands",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncodedArgument {
    pub display: String,
    pub bytes_hex: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvocationStart {
    pub invocation_id: String,
    pub executable: String,
    pub capture_kind: CaptureKind,
    pub argv: Vec<EncodedArgument>,
    pub cwd: PathBuf,
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub process_group: Option<i32>,
    pub parent_invocation_id: Option<String>,
    pub primary: bool,
    pub role: Option<String>,
    #[serde(default)]
    pub provider_usage_grace_ms: u64,
    #[serde(default)]
    pub provider_usage_grace_output_resume: String,
    pub start_monotonic_ns: u128,
    pub start_unix_ns: u128,
    pub real_executable: PathBuf,
    pub real_executable_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvocationEnd {
    pub invocation_id: String,
    pub end_monotonic_ns: u128,
    pub end_unix_ns: u128,
    pub exit_code: Option<i32>,
    pub terminating_signal: Option<i32>,
    pub stdin_sha256: String,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvocationChild {
    pub invocation_id: String,
    pub pid: u32,
    pub started_monotonic_ns: u128,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CodexPassthroughRecord {
    pub invocation_id: String,
    pub argv: Vec<EncodedArgument>,
    pub cwd: PathBuf,
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub process_group: Option<i32>,
    pub parent_invocation_id: Option<String>,
    pub primary_marker_present: bool,
    pub role: Option<String>,
    pub informational: bool,
    pub start_monotonic_ns: u128,
    pub start_unix_ns: u128,
    pub real_executable: PathBuf,
    pub real_executable_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StreamChunk {
    pub offset: u64,
    pub length: usize,
    pub sha256: String,
    pub received_monotonic_ns: u128,
    pub received_unix_ns: u128,
}

#[derive(Clone, Debug)]
pub struct ProxyOutcome {
    pub status: ExitStatus,
}

pub fn classify_codex_invocation(args: &[OsString]) -> Option<CaptureKind> {
    if is_informational_codex_invocation(args) {
        return None;
    }
    let displays = args
        .iter()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>();
    if displays.first().is_some_and(|arg| arg == "app-server") {
        return Some(CaptureKind::AppServer);
    }
    if displays.iter().any(|arg| arg == "exec") && displays.iter().any(|arg| arg == "--json") {
        return Some(CaptureKind::ExecJson);
    }
    None
}

pub fn is_locked_shell_invocation(args: &[OsString]) -> bool {
    args.len() == 2
        && args[0] == OsStr::new("-c")
        && args[1].to_string_lossy().starts_with(LOCKED_COMMAND_PREFIX)
}

pub fn run_captured_process(
    config: &CaptureConfig,
    executable_name: &str,
    kind: CaptureKind,
    real_executable: &Path,
    real_sha256: &str,
    args: &[OsString],
) -> ObserverResult<ProxyOutcome> {
    let done = Arc::new(AtomicBool::new(false));
    let mut signal_forwarder = SignalForwarder::install(done.clone())?;
    let invocation_id = invocation_id();
    let parent_invocation_id = std::env::var(PARENT_INVOCATION_ENV).ok();
    let supplied_primary_marker = std::env::var(PRIMARY_MARKER_ENV).ok();
    if supplied_primary_marker.is_some()
        && supplied_primary_marker.as_deref() != Some(config.primary_invocation_marker.as_str())
    {
        return Err(ObserverError::new(
            "primary invocation marker does not match observer configuration",
        ));
    }
    let role = std::env::var(ROLE_ENV).ok();
    let primary = supplied_primary_marker.is_some()
        && parent_invocation_id.is_none()
        && match (config.condition.as_str(), kind) {
            ("work-leaf", CaptureKind::AppServer) => true,
            ("direct", CaptureKind::ExecJson) => role.is_some(),
            _ => false,
        };
    let provider_usage_grace_ms = if primary && kind == CaptureKind::AppServer {
        provider_usage_grace_ms_from_environment()?
    } else {
        0
    };
    let provider_usage_grace_output_resume = if provider_usage_grace_ms > 0 {
        provider_usage_grace_output_resume_from_environment()?
    } else {
        ProviderUsageGraceOutputResume::Forward
    };
    let start = InvocationStart {
        invocation_id: invocation_id.clone(),
        executable: executable_name.to_string(),
        capture_kind: kind,
        argv: std::env::args_os()
            .map(|arg| encode_argument(&arg))
            .collect(),
        cwd: std::env::current_dir()?,
        pid: std::process::id(),
        parent_pid: process_parent_id(),
        process_group: process_group_id(),
        primary,
        parent_invocation_id,
        role,
        provider_usage_grace_ms,
        provider_usage_grace_output_resume: provider_usage_grace_output_resume.as_str().to_string(),
        start_monotonic_ns: monotonic_time_ns(),
        start_unix_ns: unix_time_ns(),
        real_executable: real_executable.to_path_buf(),
        real_executable_sha256: real_sha256.to_string(),
    };
    let invocation_root = config.root.join("invocations");
    let capture_root = config.root.join(kind.artifact_directory());
    let invocation_dir = invocation_root.join(&invocation_id);
    let capture_dir = capture_root.join(&invocation_id);
    create_private_directory(&invocation_root)?;
    create_private_directory(&capture_root)?;
    create_private_directory(&invocation_dir)?;
    create_private_directory(&capture_dir)?;
    write_json_atomic(&invocation_dir.join("start.json"), &start)?;

    let stdin_path = match kind {
        CaptureKind::AppServer => capture_dir.join("client-to-server.raw"),
        _ => capture_dir.join("stdin.raw"),
    };
    let stdout_path = match kind {
        CaptureKind::AppServer => capture_dir.join("server-to-client.raw"),
        _ => capture_dir.join("stdout.raw"),
    };
    let stderr_path = match kind {
        CaptureKind::AppServer => capture_dir.join("server-stderr.raw"),
        _ => capture_dir.join("stderr.raw"),
    };
    create_private_file(&stdin_path)?;
    create_private_file(&stdout_path)?;
    create_private_file(&stderr_path)?;

    let mut command = Command::new(real_executable);
    command
        .args(args)
        .env(PARENT_INVOCATION_ENV, &invocation_id)
        .env_remove(PRIMARY_MARKER_ENV)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| {
        ObserverError::new(format!(
            "failed to start real executable {}: {error}",
            real_executable.display()
        ))
    })?;
    let child_pid = child.id();
    signal_forwarder.attach_child(child_pid);
    let child_metadata = InvocationChild {
        invocation_id: invocation_id.clone(),
        pid: child_pid,
        started_monotonic_ns: monotonic_time_ns(),
    };
    if let Err(error) = write_json_atomic(&invocation_dir.join("child.json"), &child_metadata) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| ObserverError::new("captured child has no stdin"))?;
    let child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| ObserverError::new("captured child has no stdout"))?;
    let child_stderr = child
        .stderr
        .take()
        .ok_or_else(|| ObserverError::new("captured child has no stderr"))?;

    let stdin_capture = create_private_file(&stdin_path)?;
    let stdout_capture = create_private_file(&stdout_path)?;
    let stderr_capture = create_private_file(&stderr_path)?;
    let stdin_chunks = create_private_file(&capture_dir.join("stdin-chunks.jsonl"))?;
    let stdout_chunks = create_private_file(&capture_dir.join("stdout-chunks.jsonl"))?;
    let stderr_chunks = create_private_file(&capture_dir.join("stderr-chunks.jsonl"))?;
    let stdin_done = done.clone();
    let usage_grace = (provider_usage_grace_ms > 0).then(|| {
        Arc::new(ProviderUsageGrace::new(
            Duration::from_millis(provider_usage_grace_ms),
            provider_usage_grace_output_resume,
        ))
    });
    let stdin_thread = if let Some(usage_grace) = usage_grace.clone() {
        let forwarded_capture =
            create_private_file(&capture_dir.join("client-to-server.forwarded.raw"))?;
        let decisions = create_private_file(&capture_dir.join("provider-usage-grace.jsonl"))?;
        thread::spawn(move || {
            pump_app_server_stdin_with_usage_grace(
                child_stdin,
                stdin_capture,
                stdin_chunks,
                forwarded_capture,
                decisions,
                stdin_done,
                usage_grace,
            )
        })
    } else {
        thread::spawn(move || {
            pump_stdin_forward_first(child_stdin, stdin_capture, stdin_chunks, stdin_done)
        })
    };
    let stdout_thread = if let Some(usage_grace) = usage_grace {
        thread::spawn(move || {
            let output = io::stdout();
            pump_app_server_stdout_with_usage_grace(
                child_stdout,
                output.lock(),
                stdout_capture,
                stdout_chunks,
                usage_grace,
            )
        })
    } else {
        thread::spawn(move || {
            let output = io::stdout();
            pump_forward_first(child_stdout, output.lock(), stdout_capture, stdout_chunks)
        })
    };
    let stderr_thread = thread::spawn(move || {
        let output = io::stderr();
        pump_forward_first(child_stderr, output.lock(), stderr_capture, stderr_chunks)
    });

    let status = child.wait()?;
    done.store(true, Ordering::Release);
    signal_forwarder.finish();
    join_pump(stdout_thread, "stdout")?;
    join_pump(stderr_thread, "stderr")?;
    join_pump(stdin_thread, "stdin")?;

    let end = InvocationEnd {
        invocation_id,
        end_monotonic_ns: monotonic_time_ns(),
        end_unix_ns: unix_time_ns(),
        exit_code: status.code(),
        terminating_signal: exit_signal(&status),
        stdin_sha256: sha256_file(&stdin_path)?,
        stdout_sha256: sha256_file(&stdout_path)?,
        stderr_sha256: sha256_file(&stderr_path)?,
    };
    write_completion_json(&invocation_dir.join("end.json"), &end)?;
    write_json_atomic(
        &capture_dir.join("meta.json"),
        &json!({ "start": start, "end": end }),
    )?;
    harden_artifact_permissions(&invocation_dir)?;
    harden_artifact_permissions(&capture_dir)?;
    Ok(ProxyOutcome { status })
}

fn provider_usage_grace_ms_from_environment() -> ObserverResult<u64> {
    let Some(value) = std::env::var_os(PROVIDER_USAGE_GRACE_ENV) else {
        return Ok(0);
    };
    let value = value.to_string_lossy();
    let milliseconds = value.parse::<u64>().map_err(|error| {
        ObserverError::new(format!(
            "{PROVIDER_USAGE_GRACE_ENV} must be an integer number of milliseconds: {error}"
        ))
    })?;
    if milliseconds > MAX_PROVIDER_USAGE_GRACE_MS {
        return Err(ObserverError::new(format!(
            "{PROVIDER_USAGE_GRACE_ENV} must not exceed {MAX_PROVIDER_USAGE_GRACE_MS} milliseconds"
        )));
    }
    Ok(milliseconds)
}

fn provider_usage_grace_output_resume_from_environment()
-> ObserverResult<ProviderUsageGraceOutputResume> {
    let Some(value) = std::env::var_os(PROVIDER_USAGE_GRACE_OUTPUT_RESUME_ENV) else {
        return Ok(ProviderUsageGraceOutputResume::Forward);
    };
    match value.to_string_lossy().as_ref() {
        "forward" => Ok(ProviderUsageGraceOutputResume::Forward),
        "wait-for-usage" => Ok(ProviderUsageGraceOutputResume::WaitForUsage),
        value => Err(ObserverError::new(format!(
            "{PROVIDER_USAGE_GRACE_OUTPUT_RESUME_ENV} must be forward or wait-for-usage, got {value}"
        ))),
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ProviderUsageGraceOutputResume {
    #[default]
    Forward,
    WaitForUsage,
}

impl ProviderUsageGraceOutputResume {
    fn as_str(self) -> &'static str {
        match self {
            Self::Forward => "forward",
            Self::WaitForUsage => "wait-for-usage",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ProviderTurnKey {
    thread_id: String,
    turn_id: String,
}

#[derive(Default)]
struct ProviderTurnGraceState {
    directive_complete: bool,
    exact_usage_seen: bool,
    output_resumed: bool,
    turn_completed: bool,
}

#[derive(Default)]
struct ProviderUsageGraceState {
    turns: BTreeMap<ProviderTurnKey, ProviderTurnGraceState>,
    thread_totals: BTreeMap<String, CapturedUsage>,
}

struct ProviderUsageGrace {
    timeout: Duration,
    output_resume: ProviderUsageGraceOutputResume,
    state: Mutex<ProviderUsageGraceState>,
    changed: Condvar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ProviderUsageGraceOutcome {
    NotEligible,
    ForwardedAfterExactUsage,
    ForwardedAfterResumedOutputUsage,
    ForwardedAfterOutputResumed,
    ForwardedAfterTurnCompleted,
    ForwardedAfterTimeout,
}

#[derive(Debug, Serialize)]
struct ProviderUsageGraceRecord<'a> {
    thread_id: &'a str,
    turn_id: &'a str,
    configured_grace_ms: u128,
    output_resume_policy: ProviderUsageGraceOutputResume,
    waited_ms: u128,
    outcome: ProviderUsageGraceOutcome,
}

impl ProviderUsageGrace {
    fn new(timeout: Duration, output_resume: ProviderUsageGraceOutputResume) -> Self {
        Self {
            timeout,
            output_resume,
            state: Mutex::new(ProviderUsageGraceState::default()),
            changed: Condvar::new(),
        }
    }

    fn observe_server_value(&self, value: &Value) {
        let Some(key) = provider_turn_key(value) else {
            return;
        };
        let method = value.get("method").and_then(Value::as_str);
        let mut state = self
            .state
            .lock()
            .expect("provider usage grace mutex poisoned");
        let fresh_usage = if method == Some("thread/tokenUsage/updated") {
            extract_usage(value).is_some_and(|(kind, total)| {
                if kind != "thread-total" {
                    return false;
                }
                let previous = state.thread_totals.insert(key.thread_id.clone(), total);
                cumulative_usage_contains_last_response(previous, total, extract_last_usage(value))
            })
        } else {
            false
        };
        let turn = state.turns.entry(key).or_default();
        if method == Some("item/completed")
            && extract_assistant_message(value)
                .is_some_and(assistant_text_completes_work_leaf_directive)
        {
            turn.directive_complete = true;
            turn.exact_usage_seen = false;
        } else if fresh_usage && turn.directive_complete {
            turn.exact_usage_seen = true;
        } else if method == Some("turn/completed") {
            turn.turn_completed = true;
        } else if turn.directive_complete && provider_output_resumed(method) {
            turn.output_resumed = true;
        }
        self.changed.notify_all();
    }

    fn wait_before_interrupt(
        &self,
        key: &ProviderTurnKey,
        done: &AtomicBool,
    ) -> (ProviderUsageGraceOutcome, Duration) {
        let started = Instant::now();
        let mut state = self
            .state
            .lock()
            .expect("provider usage grace mutex poisoned");
        if !state
            .turns
            .get(key)
            .is_some_and(|turn| turn.directive_complete)
        {
            return (ProviderUsageGraceOutcome::NotEligible, started.elapsed());
        }
        loop {
            let turn = state
                .turns
                .get(key)
                .expect("eligible turn state disappeared");
            if turn.exact_usage_seen {
                return (
                    if turn.output_resumed
                        && self.output_resume == ProviderUsageGraceOutputResume::WaitForUsage
                    {
                        ProviderUsageGraceOutcome::ForwardedAfterResumedOutputUsage
                    } else {
                        ProviderUsageGraceOutcome::ForwardedAfterExactUsage
                    },
                    started.elapsed(),
                );
            }
            if turn.output_resumed && self.output_resume == ProviderUsageGraceOutputResume::Forward
            {
                return (
                    ProviderUsageGraceOutcome::ForwardedAfterOutputResumed,
                    started.elapsed(),
                );
            }
            if turn.turn_completed || done.load(Ordering::Acquire) {
                return (
                    ProviderUsageGraceOutcome::ForwardedAfterTurnCompleted,
                    started.elapsed(),
                );
            }
            let Some(remaining) = self.timeout.checked_sub(started.elapsed()) else {
                return (
                    ProviderUsageGraceOutcome::ForwardedAfterTimeout,
                    started.elapsed(),
                );
            };
            let (next_state, timeout) = self
                .changed
                .wait_timeout(state, remaining)
                .expect("provider usage grace mutex poisoned");
            state = next_state;
            if timeout.timed_out() {
                return (
                    ProviderUsageGraceOutcome::ForwardedAfterTimeout,
                    started.elapsed(),
                );
            }
        }
    }
}

fn provider_turn_key(value: &Value) -> Option<ProviderTurnKey> {
    Some(ProviderTurnKey {
        thread_id: extract_thread_id(value)?,
        turn_id: extract_turn_id(value)?,
    })
}

fn provider_output_resumed(method: Option<&str>) -> bool {
    matches!(method, Some("item/started" | "item/completed"))
        || method.is_some_and(|method| method.starts_with("item/") && method.ends_with("/delta"))
}

fn join_pump(handle: thread::JoinHandle<io::Result<()>>, stream: &str) -> ObserverResult<()> {
    handle
        .join()
        .map_err(|_| ObserverError::new(format!("{stream} capture thread panicked")))?
        .map_err(|error| ObserverError::new(format!("{stream} capture failed: {error}")))
}

fn pump_forward_first<R, W>(
    mut reader: R,
    mut destination: W,
    mut capture: File,
    mut chunks: File,
) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    let mut buffer = [0_u8; 16 * 1024];
    let mut offset = 0_u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let received_monotonic_ns = monotonic_time_ns();
        let received_unix_ns = unix_time_ns();
        destination.write_all(&buffer[..read])?;
        destination.flush()?;
        capture.write_all(&buffer[..read])?;
        capture.flush()?;
        let chunk = StreamChunk {
            offset,
            length: read,
            sha256: sha256_bytes(&buffer[..read]),
            received_monotonic_ns,
            received_unix_ns,
        };
        serde_json::to_writer(&mut chunks, &chunk)?;
        chunks.write_all(b"\n")?;
        chunks.flush()?;
        offset = offset.saturating_add(read as u64);
    }
    Ok(())
}

#[cfg(unix)]
fn pump_stdin_forward_first<W>(
    mut destination: W,
    mut capture: File,
    mut chunks: File,
    done: Arc<AtomicBool>,
) -> io::Result<()>
where
    W: Write,
{
    let mut buffer = [0_u8; 16 * 1024];
    let mut offset = 0_u64;
    while !done.load(Ordering::Acquire) {
        let mut descriptor = libc::pollfd {
            fd: libc::STDIN_FILENO,
            events: libc::POLLIN | libc::POLLHUP,
            revents: 0,
        };
        let result = unsafe { libc::poll(&mut descriptor, 1, 5) };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if result == 0 {
            continue;
        }
        let read = unsafe {
            libc::read(
                libc::STDIN_FILENO,
                buffer.as_mut_ptr().cast::<libc::c_void>(),
                buffer.len(),
            )
        };
        if read < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if read == 0 {
            break;
        }
        let read = read as usize;
        forward_capture_chunk(
            &buffer[..read],
            &mut destination,
            &mut capture,
            &mut chunks,
            &mut offset,
        )?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn pump_stdin_forward_first<W>(
    destination: W,
    capture: File,
    chunks: File,
    _done: Arc<AtomicBool>,
) -> io::Result<()>
where
    W: Write,
{
    let input = io::stdin();
    pump_forward_first(input.lock(), destination, capture, chunks)
}

fn pump_app_server_stdin_with_usage_grace<W>(
    mut destination: W,
    mut capture: File,
    mut chunks: File,
    mut forwarded_capture: File,
    mut decisions: File,
    done: Arc<AtomicBool>,
    usage_grace: Arc<ProviderUsageGrace>,
) -> io::Result<()>
where
    W: Write,
{
    let mut buffer = [0_u8; 16 * 1024];
    let mut pending = Vec::new();
    let mut offset = 0_u64;
    while !done.load(Ordering::Acquire) {
        let Some(read) = read_stdin_chunk(&mut buffer)? else {
            continue;
        };
        if read == 0 {
            break;
        }
        capture_stream_chunk(&buffer[..read], &mut capture, &mut chunks, &mut offset)?;
        pending.extend_from_slice(&buffer[..read]);
        forward_complete_client_lines(
            &mut pending,
            &mut destination,
            &mut forwarded_capture,
            &mut decisions,
            &done,
            &usage_grace,
        )?;
    }
    if !pending.is_empty() {
        forward_app_server_client_frame(
            &pending,
            &mut destination,
            &mut forwarded_capture,
            &mut decisions,
            &done,
            &usage_grace,
        )?;
    }
    Ok(())
}

#[cfg(unix)]
fn read_stdin_chunk(buffer: &mut [u8]) -> io::Result<Option<usize>> {
    let mut descriptor = libc::pollfd {
        fd: libc::STDIN_FILENO,
        events: libc::POLLIN | libc::POLLHUP,
        revents: 0,
    };
    let result = unsafe { libc::poll(&mut descriptor, 1, 5) };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            return Ok(None);
        }
        return Err(error);
    }
    if result == 0 {
        return Ok(None);
    }
    let read = unsafe {
        libc::read(
            libc::STDIN_FILENO,
            buffer.as_mut_ptr().cast::<libc::c_void>(),
            buffer.len(),
        )
    };
    if read < 0 {
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            return Ok(None);
        }
        return Err(error);
    }
    Ok(Some(read as usize))
}

#[cfg(not(unix))]
fn read_stdin_chunk(buffer: &mut [u8]) -> io::Result<Option<usize>> {
    io::stdin().read(buffer).map(Some)
}

fn capture_stream_chunk(
    bytes: &[u8],
    capture: &mut File,
    chunks: &mut File,
    offset: &mut u64,
) -> io::Result<()> {
    let received_monotonic_ns = monotonic_time_ns();
    let received_unix_ns = unix_time_ns();
    capture.write_all(bytes)?;
    capture.flush()?;
    let chunk = StreamChunk {
        offset: *offset,
        length: bytes.len(),
        sha256: sha256_bytes(bytes),
        received_monotonic_ns,
        received_unix_ns,
    };
    serde_json::to_writer(&mut *chunks, &chunk)?;
    chunks.write_all(b"\n")?;
    chunks.flush()?;
    *offset = offset.saturating_add(bytes.len() as u64);
    Ok(())
}

fn forward_complete_client_lines<W>(
    pending: &mut Vec<u8>,
    destination: &mut W,
    forwarded_capture: &mut File,
    decisions: &mut File,
    done: &AtomicBool,
    usage_grace: &ProviderUsageGrace,
) -> io::Result<()>
where
    W: Write,
{
    let mut consumed = 0;
    while let Some(relative_end) = pending[consumed..].iter().position(|byte| *byte == b'\n') {
        let end = consumed + relative_end + 1;
        forward_app_server_client_frame(
            &pending[consumed..end],
            destination,
            forwarded_capture,
            decisions,
            done,
            usage_grace,
        )?;
        consumed = end;
    }
    if consumed > 0 {
        pending.drain(..consumed);
    }
    Ok(())
}

fn forward_app_server_client_frame<W>(
    frame: &[u8],
    destination: &mut W,
    forwarded_capture: &mut File,
    decisions: &mut File,
    done: &AtomicBool,
    usage_grace: &ProviderUsageGrace,
) -> io::Result<()>
where
    W: Write,
{
    if let Ok(value) = serde_json::from_slice::<Value>(frame)
        && value.get("method").and_then(Value::as_str) == Some("turn/interrupt")
        && let Some(key) = provider_turn_key(&value)
    {
        let (outcome, waited) = usage_grace.wait_before_interrupt(&key, done);
        serde_json::to_writer(
            &mut *decisions,
            &ProviderUsageGraceRecord {
                thread_id: &key.thread_id,
                turn_id: &key.turn_id,
                configured_grace_ms: usage_grace.timeout.as_millis(),
                output_resume_policy: usage_grace.output_resume,
                waited_ms: waited.as_millis(),
                outcome,
            },
        )?;
        decisions.write_all(b"\n")?;
        decisions.flush()?;
    }
    destination.write_all(frame)?;
    destination.flush()?;
    forwarded_capture.write_all(frame)?;
    forwarded_capture.flush()
}

fn pump_app_server_stdout_with_usage_grace<R, W>(
    mut reader: R,
    mut destination: W,
    mut capture: File,
    mut chunks: File,
    usage_grace: Arc<ProviderUsageGrace>,
) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    let mut buffer = [0_u8; 16 * 1024];
    let mut pending = Vec::new();
    let mut offset = 0_u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        pending.extend_from_slice(&buffer[..read]);
        observe_complete_server_lines(&mut pending, &usage_grace);
        destination.write_all(&buffer[..read])?;
        destination.flush()?;
        capture_stream_chunk(&buffer[..read], &mut capture, &mut chunks, &mut offset)?;
    }
    if !pending.is_empty()
        && let Ok(value) = serde_json::from_slice::<Value>(&pending)
    {
        usage_grace.observe_server_value(&value);
    }
    Ok(())
}

fn observe_complete_server_lines(pending: &mut Vec<u8>, usage_grace: &ProviderUsageGrace) {
    let mut consumed = 0;
    while let Some(relative_end) = pending[consumed..].iter().position(|byte| *byte == b'\n') {
        let end = consumed + relative_end + 1;
        if let Ok(value) = serde_json::from_slice::<Value>(&pending[consumed..end]) {
            usage_grace.observe_server_value(&value);
        }
        consumed = end;
    }
    if consumed > 0 {
        pending.drain(..consumed);
    }
}

fn forward_capture_chunk<W>(
    bytes: &[u8],
    destination: &mut W,
    capture: &mut File,
    chunks: &mut File,
    offset: &mut u64,
) -> io::Result<()>
where
    W: Write,
{
    let received_monotonic_ns = monotonic_time_ns();
    let received_unix_ns = unix_time_ns();
    destination.write_all(bytes)?;
    destination.flush()?;
    capture.write_all(bytes)?;
    capture.flush()?;
    let chunk = StreamChunk {
        offset: *offset,
        length: bytes.len(),
        sha256: sha256_bytes(bytes),
        received_monotonic_ns,
        received_unix_ns,
    };
    serde_json::to_writer(&mut *chunks, &chunk)?;
    chunks.write_all(b"\n")?;
    chunks.flush()?;
    *offset = offset.saturating_add(bytes.len() as u64);
    Ok(())
}

#[derive(Default)]
struct SignalTarget {
    child_pid: Option<u32>,
    pending_signal: Option<i32>,
}

struct SignalForwarder {
    done: Arc<AtomicBool>,
    target: Arc<Mutex<SignalTarget>>,
    handle: signal_hook::iterator::Handle,
    thread: Option<thread::JoinHandle<()>>,
}

impl SignalForwarder {
    fn install(done: Arc<AtomicBool>) -> ObserverResult<Self> {
        let mut signals =
            signal_hook::iterator::Signals::new([libc::SIGTERM, libc::SIGINT, libc::SIGHUP])
                .map_err(|error| {
                    ObserverError::new(format!("cannot install signal capture: {error}"))
                })?;
        let handle = signals.handle();
        let target = Arc::new(Mutex::new(SignalTarget::default()));
        let thread_target = target.clone();
        let thread_done = done.clone();
        let thread = thread::spawn(move || {
            for signal in signals.forever() {
                if thread_done.load(Ordering::Acquire) {
                    break;
                }
                let child_pid = {
                    let mut target = thread_target.lock().unwrap();
                    if target.child_pid.is_none() {
                        target.pending_signal = Some(signal);
                    }
                    target.child_pid
                };
                if let Some(child_pid) = child_pid {
                    forward_signal(child_pid, signal);
                }
            }
        });
        Ok(Self {
            done,
            target,
            handle,
            thread: Some(thread),
        })
    }

    fn attach_child(&self, child_pid: u32) {
        let pending_signal = {
            let mut target = self.target.lock().unwrap();
            target.child_pid = Some(child_pid);
            target.pending_signal.take()
        };
        if let Some(signal) = pending_signal {
            forward_signal(child_pid, signal);
        }
    }

    fn finish(&mut self) {
        self.done.store(true, Ordering::Release);
        self.handle.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for SignalForwarder {
    fn drop(&mut self) {
        self.finish();
    }
}

fn forward_signal(child_pid: u32, signal: i32) {
    #[cfg(unix)]
    unsafe {
        libc::kill(child_pid as i32, signal);
    }
}

#[cfg(unix)]
fn exit_signal(status: &ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn exit_signal(_status: &ExitStatus) -> Option<i32> {
    None
}

#[cfg(unix)]
fn encode_argument(argument: &OsStr) -> EncodedArgument {
    use std::os::unix::ffi::OsStrExt;
    EncodedArgument {
        display: argument.to_string_lossy().to_string(),
        bytes_hex: hex_bytes(argument.as_bytes()),
    }
}

#[cfg(not(unix))]
fn encode_argument(argument: &OsStr) -> EncodedArgument {
    let display = argument.to_string_lossy().to_string();
    EncodedArgument {
        bytes_hex: hex_bytes(display.as_bytes()),
        display,
    }
}

fn process_parent_id() -> Option<u32> {
    process_stat_fields().and_then(|fields| fields.first()?.parse().ok())
}

fn process_group_id() -> Option<i32> {
    process_stat_fields().and_then(|fields| fields.get(1)?.parse().ok())
}

fn process_stat_fields() -> Option<Vec<String>> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;
    let end = stat.rfind(')')?;
    let fields = stat.get(end + 2..)?.split_whitespace().collect::<Vec<_>>();
    Some(fields.into_iter().skip(1).map(str::to_string).collect())
}

fn monotonic_time_ns() -> u128 {
    #[cfg(unix)]
    {
        let mut time = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time);
        }
        return (time.tv_sec as u128) * 1_000_000_000 + time.tv_nsec as u128;
    }
    #[allow(unreachable_code)]
    unix_time_ns()
}

fn unix_time_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn random_marker() -> ObserverResult<String> {
    let mut bytes = [0_u8; 32];
    #[cfg(unix)]
    {
        File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    }
    #[cfg(not(unix))]
    {
        let seed = format!(
            "{}:{}:{}",
            unix_time_ns(),
            monotonic_time_ns(),
            std::process::id()
        );
        bytes.copy_from_slice(&Sha256::digest(seed.as_bytes()));
    }
    Ok(hex_bytes(&bytes))
}

fn invocation_id() -> String {
    format!("{:020}-{}", monotonic_time_ns(), std::process::id())
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use fmt::Write as _;
        let _ = write!(text, "{byte:02x}");
    }
    text
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_file(path: &Path) -> ObserverResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn write_json_atomic(path: &Path, value: &impl Serialize) -> ObserverResult<()> {
    write_json_atomic_with_order(path, value, false)
}

fn write_completion_json(path: &Path, value: &impl Serialize) -> ObserverResult<()> {
    write_json_atomic_with_order(path, value, true)
}

fn write_json_atomic_with_order(
    path: &Path,
    value: &impl Serialize,
    publish_before_sync: bool,
) -> ObserverResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&temporary)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.flush()?;
    if publish_before_sync {
        fs::rename(&temporary, path)?;
        file.sync_all()?;
    } else {
        file.sync_all()?;
        fs::rename(temporary, path)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapturedUsage {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
}

impl CapturedUsage {
    fn combine(self, other: Self) -> Self {
        Self {
            input_tokens: self.input_tokens.saturating_add(other.input_tokens),
            cached_input_tokens: self
                .cached_input_tokens
                .saturating_add(other.cached_input_tokens),
            output_tokens: self.output_tokens.saturating_add(other.output_tokens),
            reasoning_output_tokens: self
                .reasoning_output_tokens
                .saturating_add(other.reasoning_output_tokens),
        }
    }

    fn score(self) -> (u64, u64, u64, u64) {
        (
            self.input_tokens.saturating_add(self.output_tokens),
            self.input_tokens,
            self.output_tokens,
            self.reasoning_output_tokens,
        )
    }

    fn checked_difference(self, previous: Self) -> Option<Self> {
        Some(Self {
            input_tokens: self.input_tokens.checked_sub(previous.input_tokens)?,
            cached_input_tokens: self
                .cached_input_tokens
                .checked_sub(previous.cached_input_tokens)?,
            output_tokens: self.output_tokens.checked_sub(previous.output_tokens)?,
            reasoning_output_tokens: self
                .reasoning_output_tokens
                .checked_sub(previous.reasoning_output_tokens)?,
        })
    }

    pub fn uncached_input_tokens(self) -> u64 {
        self.input_tokens.saturating_sub(self.cached_input_tokens)
    }
}

fn cumulative_usage_contains_last_response(
    previous: Option<CapturedUsage>,
    total: CapturedUsage,
    last: Option<CapturedUsage>,
) -> bool {
    let previous_total = previous.unwrap_or_default();
    let Some(increase) = total.checked_difference(previous_total) else {
        return false;
    };
    if increase == CapturedUsage::default() {
        return false;
    }
    let Some(last) = last.filter(|usage| *usage != CapturedUsage::default()) else {
        return false;
    };
    increase.checked_difference(last).is_some()
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsonlFrame {
    pub sequence: usize,
    pub direction: String,
    pub offset: usize,
    pub length: usize,
    pub sha256: String,
    pub received_monotonic_ns: Option<u128>,
    pub received_unix_ns: Option<u128>,
    pub parsed: bool,
    pub parse_error: Option<String>,
    pub rpc_id: Option<String>,
    pub method: Option<String>,
    pub event_type: Option<String>,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub prompt_bytes: Option<usize>,
    pub agent_id: Option<String>,
    pub usage_kind: Option<String>,
    pub usage: Option<CapturedUsage>,
}

struct WireJsonMessage<'a> {
    offset: usize,
    wire: &'a [u8],
    body: &'a [u8],
    framing_error: Option<String>,
}

pub fn index_jsonl(bytes: &[u8], direction: &str) -> Vec<JsonlFrame> {
    index_jsonl_with_chunks(bytes, direction, &[])
}

fn index_jsonl_with_chunks(
    bytes: &[u8],
    direction: &str,
    chunks: &[StreamChunk],
) -> Vec<JsonlFrame> {
    let mut frames = Vec::new();
    let mut chunk_index = 0;
    for (sequence, message) in split_wire_json_messages(bytes).into_iter().enumerate() {
        let (value, parse_error) = match message.framing_error {
            Some(error) => (None, Some(error)),
            None => match serde_json::from_slice::<Value>(message.body) {
                Ok(value) => (Some(value), None),
                Err(error) => (None, Some(error.to_string())),
            },
        };
        let rpc_id = value.as_ref().and_then(json_rpc_id);
        let method = value
            .as_ref()
            .and_then(|value| value.get("method"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let event_type = value
            .as_ref()
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let thread_id = value.as_ref().and_then(extract_thread_id);
        let turn_id = value.as_ref().and_then(extract_turn_id);
        let prompt = value.as_ref().and_then(extract_turn_prompt);
        let (usage_kind, usage) = value
            .as_ref()
            .and_then(extract_usage)
            .map(|(kind, usage)| (Some(kind), Some(usage)))
            .unwrap_or((None, None));
        let end = message.offset.saturating_add(message.wire.len());
        // Capture chunks are contiguous and ordered by offset. Advancing one cursor visits every
        // frame and chunk once instead of scanning the complete chunk list for every frame.
        while chunks.get(chunk_index).is_some_and(|chunk| {
            let chunk_end = chunk.offset.saturating_add(chunk.length as u64);
            chunk_end < end as u64
        }) {
            chunk_index += 1;
        }
        let received = chunks.get(chunk_index).filter(|chunk| {
            let chunk_end = chunk.offset.saturating_add(chunk.length as u64);
            chunk.offset < end as u64 && chunk_end >= end as u64
        });
        frames.push(JsonlFrame {
            sequence,
            direction: direction.to_string(),
            offset: message.offset,
            length: message.wire.len(),
            sha256: sha256_bytes(message.wire),
            received_monotonic_ns: received.map(|chunk| chunk.received_monotonic_ns),
            received_unix_ns: received.map(|chunk| chunk.received_unix_ns),
            parsed: value.is_some(),
            parse_error,
            rpc_id,
            method,
            event_type,
            thread_id,
            turn_id,
            prompt_bytes: prompt.map(str::len),
            agent_id: prompt.and_then(extract_agent_id),
            usage_kind,
            usage,
        });
    }
    frames
}

fn split_wire_json_messages(bytes: &[u8]) -> Vec<WireJsonMessage<'_>> {
    let mut messages = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let remaining = &bytes[offset..];
        let message = if starts_with_content_length(remaining) {
            content_length_message(bytes, offset)
        } else {
            json_line_message(bytes, offset)
        };
        offset = offset.saturating_add(message.wire.len());
        messages.push(message);
    }
    messages
}

fn starts_with_content_length(bytes: &[u8]) -> bool {
    const PREFIX: &[u8] = b"content-length:";
    bytes
        .get(..PREFIX.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(PREFIX))
}

fn content_length_message(bytes: &[u8], offset: usize) -> WireJsonMessage<'_> {
    let remaining = &bytes[offset..];
    let Some((header_length, delimiter_length)) = content_length_header_end(remaining) else {
        return WireJsonMessage {
            offset,
            wire: remaining,
            body: &[],
            framing_error: Some("Content-Length frame has no header terminator".to_string()),
        };
    };
    let header = &remaining[..header_length];
    let content_length = header.split(|byte| *byte == b'\n').find_map(|line| {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let colon = line.iter().position(|byte| *byte == b':')?;
        line[..colon]
            .eq_ignore_ascii_case(b"content-length")
            .then(|| parse_ascii_usize(&line[colon + 1..]))?
    });
    let Some(content_length) = content_length else {
        return WireJsonMessage {
            offset,
            wire: remaining,
            body: &[],
            framing_error: Some("Content-Length frame has an invalid length".to_string()),
        };
    };
    let body_start = header_length.saturating_add(delimiter_length);
    let Some(frame_length) = body_start.checked_add(content_length) else {
        return WireJsonMessage {
            offset,
            wire: remaining,
            body: &[],
            framing_error: Some("Content-Length frame length overflows usize".to_string()),
        };
    };
    if frame_length > remaining.len() {
        return WireJsonMessage {
            offset,
            wire: remaining,
            body: &remaining[body_start..],
            framing_error: Some(format!(
                "Content-Length frame declares {content_length} payload bytes but only {} remain",
                remaining.len().saturating_sub(body_start)
            )),
        };
    }
    WireJsonMessage {
        offset,
        wire: &remaining[..frame_length],
        body: &remaining[body_start..frame_length],
        framing_error: None,
    }
}

fn content_length_header_end(bytes: &[u8]) -> Option<(usize, usize)> {
    let crlf = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| (position, 4));
    let lf = bytes
        .windows(2)
        .position(|window| window == b"\n\n")
        .map(|position| (position, 2));
    match (crlf, lf) {
        (Some(left), Some(right)) => Some(if left.0 <= right.0 { left } else { right }),
        (Some(result), None) | (None, Some(result)) => Some(result),
        (None, None) => None,
    }
}

fn parse_ascii_usize(bytes: &[u8]) -> Option<usize> {
    let start = bytes.iter().position(|byte| !byte.is_ascii_whitespace())?;
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())?
        .saturating_add(1);
    std::str::from_utf8(&bytes[start..end]).ok()?.parse().ok()
}

fn json_line_message(bytes: &[u8], offset: usize) -> WireJsonMessage<'_> {
    let remaining = &bytes[offset..];
    let length = remaining
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|position| position + 1)
        .unwrap_or(remaining.len());
    let wire = &remaining[..length];
    let body = wire
        .strip_suffix(b"\n")
        .unwrap_or(wire)
        .strip_suffix(b"\r")
        .unwrap_or_else(|| wire.strip_suffix(b"\n").unwrap_or(wire));
    WireJsonMessage {
        offset,
        wire,
        body,
        framing_error: None,
    }
}

fn read_stream_chunks(path: &Path) -> ObserverResult<Vec<StreamChunk>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let mut chunks = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        if !line.trim().is_empty() {
            chunks.push(serde_json::from_str(&line)?);
        }
    }
    Ok(chunks)
}

fn json_rpc_id(value: &Value) -> Option<String> {
    match value.get("id")? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn extract_thread_id(value: &Value) -> Option<String> {
    value
        .get("thread_id")
        .or_else(|| value.get("threadId"))
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .get("params")
                .and_then(|params| params.get("threadId").or_else(|| params.get("thread_id")))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            value
                .get("result")
                .and_then(|result| result.get("thread"))
                .and_then(|thread| thread.get("id"))
                .and_then(Value::as_str)
        })
        .map(str::to_string)
}

fn extract_turn_id(value: &Value) -> Option<String> {
    value
        .get("turn_id")
        .or_else(|| value.get("turnId"))
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .get("params")
                .and_then(|params| params.get("turnId").or_else(|| params.get("turn_id")))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            value
                .get("params")
                .and_then(|params| params.get("turn"))
                .and_then(|turn| turn.get("id"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            value
                .get("result")
                .and_then(|result| result.get("turn"))
                .and_then(|turn| turn.get("id"))
                .and_then(Value::as_str)
        })
        .map(str::to_string)
}

fn extract_turn_prompt(value: &Value) -> Option<&str> {
    if value.get("method").and_then(Value::as_str) != Some("turn/start") {
        return None;
    }
    value
        .get("params")?
        .get("input")?
        .as_array()?
        .iter()
        .find(|input| input.get("type").and_then(Value::as_str) == Some("text"))?
        .get("text")?
        .as_str()
}

fn extract_agent_id(prompt: &str) -> Option<String> {
    prompt
        .lines()
        .find_map(|line| line.strip_prefix("Agent-ID: "))
        .map(str::trim)
        .filter(|agent_id| !agent_id.is_empty())
        .map(str::to_string)
}

fn extract_usage(value: &Value) -> Option<(String, CapturedUsage)> {
    if value.get("type").and_then(Value::as_str) == Some("turn.completed") {
        return usage_from_object(value.get("usage")?)
            .map(|usage| ("invocation-total".to_string(), usage));
    }
    if value.get("method").and_then(Value::as_str) == Some("thread/tokenUsage/updated") {
        let token_usage = value.get("params")?.get("tokenUsage").or_else(|| {
            value
                .get("params")
                .and_then(|params| params.get("token_usage"))
        })?;
        if let Some(total) = token_usage.get("total").and_then(usage_from_object) {
            return Some(("thread-total".to_string(), total));
        }
        return token_usage
            .get("last")
            .and_then(usage_from_object)
            .map(|usage| ("turn-last".to_string(), usage));
    }
    None
}

fn extract_last_usage(value: &Value) -> Option<CapturedUsage> {
    if value.get("method").and_then(Value::as_str) != Some("thread/tokenUsage/updated") {
        return None;
    }
    value
        .get("params")?
        .get("tokenUsage")
        .or_else(|| {
            value
                .get("params")
                .and_then(|params| params.get("token_usage"))
        })?
        .get("last")
        .and_then(usage_from_object)
}

fn assistant_text_completes_work_leaf_directive(text: &str) -> bool {
    let mut in_patch = false;
    for line in text.lines() {
        let Some(body) = observer_directive_body(line) else {
            continue;
        };
        if in_patch {
            if body == "end" {
                return true;
            }
            continue;
        }
        if body == "done"
            || observer_directive_rest(body, "read").is_some()
            || observer_directive_rest(body, "locks run").is_some()
            || observer_directive_rest(body, "locks classify").is_some()
            || observer_directive_rest(body, "send").is_some()
        {
            return true;
        }
        if observer_directive_rest(body, "patch").is_some()
            || observer_directive_rest(body, "edit").is_some()
        {
            in_patch = true;
        }
    }
    false
}

fn observer_directive_body(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("@work-leaf")?;
    let mut characters = rest.chars();
    if !characters.next()?.is_whitespace() {
        return None;
    }
    Some(characters.as_str().trim())
}

fn observer_directive_rest<'a>(body: &'a str, command: &str) -> Option<&'a str> {
    let rest = body.strip_prefix(command)?;
    if rest.is_empty() {
        return Some("");
    }
    let mut characters = rest.chars();
    characters
        .next()?
        .is_whitespace()
        .then(|| characters.as_str().trim_start())
}

fn usage_from_object(value: &Value) -> Option<CapturedUsage> {
    if !value.is_object() {
        return None;
    }
    Some(CapturedUsage {
        input_tokens: json_u64(value, "inputTokens", "input_tokens"),
        cached_input_tokens: json_u64(value, "cachedInputTokens", "cached_input_tokens"),
        output_tokens: json_u64(value, "outputTokens", "output_tokens"),
        reasoning_output_tokens: json_u64(
            value,
            "reasoningOutputTokens",
            "reasoning_output_tokens",
        ),
    })
}

fn json_u64(value: &Value, camel_case: &str, snake_case: &str) -> u64 {
    value
        .get(camel_case)
        .or_else(|| value.get(snake_case))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

#[derive(Clone, Debug)]
pub struct UsageObservation {
    pub thread_id: String,
    pub invocation_id: String,
    pub primary: bool,
    pub visible: bool,
    pub usage: CapturedUsage,
    accounting: UsageAccounting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UsageAccounting {
    CumulativeThread,
    PerInvocation,
}

impl UsageObservation {
    pub fn new(
        thread_id: impl Into<String>,
        invocation_id: impl Into<String>,
        primary: bool,
        visible: bool,
        usage: CapturedUsage,
    ) -> Self {
        Self {
            thread_id: thread_id.into(),
            invocation_id: invocation_id.into(),
            primary,
            visible,
            usage,
            accounting: UsageAccounting::CumulativeThread,
        }
    }

    fn per_invocation(
        thread_id: impl Into<String>,
        invocation_id: impl Into<String>,
        primary: bool,
        visible: bool,
        usage: CapturedUsage,
    ) -> Self {
        Self {
            thread_id: thread_id.into(),
            invocation_id: invocation_id.into(),
            primary,
            visible,
            usage,
            accounting: UsageAccounting::PerInvocation,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct UsageAggregate {
    pub thread_count: usize,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub uncached_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
    pub raw_input_plus_output: u64,
    pub uncached_input_plus_output: u64,
    #[serde(skip)]
    pub usage: CapturedUsage,
}

impl UsageAggregate {
    fn from_threads<'a>(threads: impl Iterator<Item = &'a UsageObservation>) -> Self {
        let records = threads.collect::<Vec<_>>();
        let usage = records
            .iter()
            .fold(CapturedUsage::default(), |total, record| {
                total.combine(record.usage)
            });
        Self {
            thread_count: records.len(),
            input_tokens: usage.input_tokens,
            cached_input_tokens: usage.cached_input_tokens,
            uncached_input_tokens: usage.uncached_input_tokens(),
            output_tokens: usage.output_tokens,
            reasoning_output_tokens: usage.reasoning_output_tokens,
            raw_input_plus_output: usage.input_tokens.saturating_add(usage.output_tokens),
            uncached_input_plus_output: usage
                .uncached_input_tokens()
                .saturating_add(usage.output_tokens),
            usage,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct UsageScopes {
    pub visible_role: UsageAggregate,
    pub primary_condition: UsageAggregate,
    pub total_workflow: UsageAggregate,
}

pub fn summarize_usage(observations: &[UsageObservation]) -> UsageScopes {
    let threads = final_thread_observations(observations);
    UsageScopes {
        visible_role: UsageAggregate::from_threads(threads.iter().filter(|thread| thread.visible)),
        primary_condition: UsageAggregate::from_threads(
            threads.iter().filter(|thread| thread.primary),
        ),
        total_workflow: UsageAggregate::from_threads(threads.iter()),
    }
}

#[derive(Clone, Debug)]
pub enum EvidenceInput {
    Prompt(String),
    Interrupt,
    ThreadTopology,
    Command { class: String, repeated: bool },
    Usage,
    SequentialTimeline,
    GenerationUsage,
    GitCheckpoint,
    AccountingReconciliation,
    ProtocolBytes(u64),
}

#[derive(Clone, Debug)]
pub struct BundleArchiveObservation {
    pub source_path: PathBuf,
    pub archived_path: PathBuf,
    pub bytes: Vec<u8>,
    pub sha256: String,
}

#[derive(Clone, Debug)]
pub struct CommandObservation {
    pub thread_id: String,
    pub turn_id: String,
    pub command: String,
    pub output: Vec<u8>,
    pub duration_ns: Option<u128>,
}

#[derive(Clone, Debug)]
pub struct LockedCommandObservation {
    pub invocation_id: String,
    pub command: String,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: Option<i32>,
    pub terminating_signal: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct TimelineObservation {
    pub event: String,
    pub detail: Option<String>,
    pub monotonic_ns: u128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TurnOutcome {
    Completed,
    Interrupted,
    Failed,
}

#[derive(Clone, Debug)]
struct SnapshotEvidence {
    text: Arc<str>,
    digest: String,
}

#[derive(Clone, Debug)]
struct OmittedRefreshEvidence {
    digest: String,
    declared_bytes: u64,
}

#[derive(Clone, Debug)]
struct ContextBundleSnapshot {
    path: String,
    digest: String,
    text: Arc<str>,
    bundle_component_bytes: u64,
}

impl ContextBundleSnapshot {
    fn manifest_value(&self) -> Value {
        json!({
            "path": self.path,
            "digest": self.digest,
            "bytes": self.text.len(),
            "bundle_component_bytes": self.bundle_component_bytes,
        })
    }
}

#[derive(Clone, Debug)]
struct CommandResultEvidence {
    thread_id: String,
    turn_id: String,
    key: CommandResultKey,
    status: String,
    timed_out: bool,
    timeout: Option<String>,
    prompt_bytes: u64,
    received_monotonic_ns: Option<u128>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CommandResultKey {
    command: String,
    rendered_output_sha256: String,
}

#[derive(Clone, Debug)]
struct BundleAnnouncement {
    thread_id: String,
    turn_id: String,
    path: PathBuf,
    manifest_bytes: u64,
    file_digests: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default)]
struct TerminalTurnEvidence {
    directive: bool,
    interrupted: bool,
    outcome: Option<TurnOutcome>,
}

#[derive(Clone, Debug)]
struct TimedLockedCommand {
    observation: LockedCommandObservation,
    start_monotonic_ns: Option<u128>,
    end_monotonic_ns: Option<u128>,
    h4_candidate: bool,
}

#[derive(Clone, Debug, Default)]
pub struct MechanismAnalyzer {
    counts: [u64; 16],
    protocol_bytes: u64,
    command_classes: BTreeMap<String, u64>,
    repeated_commands: u64,
    counterfactuals: Vec<CounterfactualRecord>,
    snapshots: BTreeMap<(String, String), SnapshotEvidence>,
    omitted_refreshes: BTreeMap<(String, String), OmittedRefreshEvidence>,
    locked_commands: Vec<TimedLockedCommand>,
    command_results: BTreeMap<CommandResultKey, Vec<CommandResultEvidence>>,
    bundle_archives: BTreeMap<PathBuf, BundleArchiveObservation>,
    bundle_snapshots: BTreeMap<PathBuf, BTreeMap<String, ContextBundleSnapshot>>,
    bundle_announcements: BTreeMap<PathBuf, Vec<BundleAnnouncement>>,
    commands: Vec<CommandObservation>,
    command_signatures: BTreeMap<String, u64>,
    thread_roles: BTreeMap<String, String>,
    terminal_turns: BTreeMap<(String, String), TerminalTurnEvidence>,
    edit_submissions: BTreeMap<String, u64>,
    edit_acknowledgements: u64,
    edit_rejections: u64,
    review_prompts: u64,
    review_targets: BTreeMap<String, u64>,
    git_commit_hashes: BTreeSet<String>,
    protocol_components: BTreeMap<String, u64>,
    timelines: Vec<TimelineObservation>,
    timeline_asserted: bool,
    errors: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CounterfactualRecord {
    pub hypothesis: String,
    pub status: String,
    pub observed_prompt_bytes: Option<u64>,
    pub actual_component_bytes: Option<u64>,
    pub counterfactual_component_bytes: Option<u64>,
    pub avoided_bytes: Option<i64>,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HypothesisEvidence {
    pub id: String,
    pub observed: bool,
    pub evidence_count: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BundleMeasurement {
    pub thread_id: String,
    pub turn_id: String,
    pub source_path: PathBuf,
    pub archived_path: PathBuf,
    pub payload_bytes: u64,
    pub manifest_bytes: u64,
    pub observed_follow_up_bytes: u64,
    pub consumption: String,
    pub deferred_bytes: i64,
    pub observed_path_net_bytes: i64,
    pub sha256: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TerminalDirectiveSummary {
    pub observed: u64,
    pub interrupted: u64,
    pub naturally_completed: u64,
    pub failed: u64,
    pub unresolved: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct StructuredEditSummary {
    pub submissions: u64,
    pub duplicate_submissions: u64,
    pub acknowledgements: u64,
    pub rejections: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReviewEvidenceSummary {
    pub prompts: u64,
    pub unique_targets: u64,
    pub duplicate_targets: u64,
    pub validated_targets: u64,
    pub unresolved_targets: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidationBudgetSummary {
    pub validation_commands: u64,
    pub violations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MechanismSummary {
    pub hypotheses: Vec<HypothesisEvidence>,
    pub protocol_bytes: u64,
    pub command_classes: BTreeMap<String, u64>,
    pub command_count: u64,
    pub command_output_bytes: u64,
    pub command_duration_ns: u128,
    pub repeated_commands: u64,
    pub counterfactuals: Vec<CounterfactualRecord>,
    pub bundles: Vec<BundleMeasurement>,
    pub terminal_directives: TerminalDirectiveSummary,
    pub structured_edits: StructuredEditSummary,
    pub reviews: ReviewEvidenceSummary,
    pub validation: ValidationBudgetSummary,
    pub protocol_components: BTreeMap<String, u64>,
    pub sequential_timeline_valid: bool,
    pub errors: Vec<String>,
}

impl MechanismAnalyzer {
    pub fn observe(&mut self, input: EvidenceInput) {
        match input {
            EvidenceInput::Prompt(prompt) => {
                let turn_id = format!("unscoped-{}", self.protocol_bytes);
                self.observe_prompt("unscoped", &turn_id, &prompt);
            }
            EvidenceInput::Interrupt => {
                self.bump(8);
                self.counterfactuals.push(CounterfactualRecord {
                    hypothesis: "H8".to_string(),
                    status: "requires-ablation".to_string(),
                    observed_prompt_bytes: None,
                    actual_component_bytes: None,
                    counterfactual_component_bytes: None,
                    avoided_bytes: None,
                    note: "prevented generation is not observable in an interrupted turn"
                        .to_string(),
                });
            }
            EvidenceInput::ThreadTopology => self.bump(9),
            EvidenceInput::Command { class, repeated } => {
                self.bump(10);
                *self.command_classes.entry(class).or_default() += 1;
                if repeated {
                    self.repeated_commands += 1;
                }
            }
            EvidenceInput::Usage => self.bump(11),
            EvidenceInput::SequentialTimeline => {
                self.bump(12);
                self.timeline_asserted = true;
            }
            EvidenceInput::GenerationUsage => self.bump(13),
            EvidenceInput::GitCheckpoint => self.bump(14),
            EvidenceInput::AccountingReconciliation => self.bump(15),
            EvidenceInput::ProtocolBytes(bytes) => {
                self.bump(16);
                self.protocol_bytes = self.protocol_bytes.saturating_add(bytes);
            }
        }
    }

    pub fn observe_prompt(&mut self, thread_id: &str, turn_id: &str, prompt: &str) {
        self.observe_prompt_at(thread_id, turn_id, prompt, None);
    }

    fn observe_prompt_at(
        &mut self,
        thread_id: &str,
        turn_id: &str,
        prompt: &str,
        received_monotonic_ns: Option<u128>,
    ) {
        self.observe_protocol_component(prompt);
        self.protocol_bytes = self.protocol_bytes.saturating_add(prompt.len() as u64);
        self.bump(16);
        for refresh in parse_omitted_refresh_snapshots(prompt) {
            self.omitted_refreshes.insert(
                (thread_id.to_string(), refresh.path),
                OmittedRefreshEvidence {
                    digest: refresh.digest,
                    declared_bytes: refresh.declared_bytes,
                },
            );
        }
        let is_file_read_envelope = prompt.starts_with("work-leaf file text\n");

        if is_file_read_envelope
            && !prompt.contains("Repeated file reads")
            && !prompt.contains("Context bundle:")
        {
            for (path, text) in parse_exact_file_sections(prompt) {
                self.snapshots.insert(
                    (thread_id.to_string(), path),
                    SnapshotEvidence {
                        digest: content_digest_for_observer(&text),
                        text: Arc::from(text),
                    },
                );
            }
        }

        if is_file_read_envelope
            && (prompt.contains("orchestrator context bundle instead of this chat")
                || prompt.contains("Context bundle:"))
        {
            self.bump(3);
            self.record_prompt_counterfactual(
                3,
                prompt,
                "descriptive-observed-path",
                "bundle payload and observed follow-up reads are reported separately",
            );
            if let Some(path) = prompt
                .lines()
                .find_map(|line| line.strip_prefix("Context bundle: "))
            {
                let announcement = BundleAnnouncement {
                    thread_id: thread_id.to_string(),
                    turn_id: turn_id.to_string(),
                    path: PathBuf::from(path.trim()),
                    manifest_bytes: prompt.len() as u64,
                    file_digests: parse_declared_bundle_files(prompt),
                };
                self.associate_bundle_snapshots(&announcement);
                self.bundle_announcements
                    .entry(announcement.path.clone())
                    .or_default()
                    .push(announcement);
            }
        }
        if is_file_read_envelope
            && (prompt.contains("Repeated file reads with changes")
                || prompt.contains("status: changed since this agent's last snapshot"))
        {
            self.bump(2);
            self.observe_changed_reads(thread_id, prompt);
        }
        if is_file_read_envelope && prompt.contains("Repeated file reads unchanged") {
            self.bump(1);
            self.observe_unchanged_reads(thread_id, prompt);
        }
        if prompt.starts_with("work-leaf command result\n") {
            self.bump(4);
            if let Some(result) =
                parse_command_result(thread_id, turn_id, prompt, received_monotonic_ns)
            {
                self.command_results
                    .entry(result.key.clone())
                    .or_default()
                    .push(result);
            } else {
                self.record_prompt_counterfactual(
                    4,
                    prompt,
                    "requires-locked-command-pairing",
                    "the rendered result did not contain an unambiguous command/stdout/stderr envelope",
                );
            }
        }
        if prompt.starts_with("work-leaf patch applied\n")
            || prompt.starts_with("work-leaf patch already applied\n")
            || prompt.contains("The orchestrator could not apply your edit.")
        {
            self.bump(5);
            if prompt.starts_with("work-leaf patch applied\n")
                || prompt.starts_with("work-leaf patch already applied\n")
            {
                self.edit_acknowledgements = self.edit_acknowledgements.saturating_add(1);
            } else {
                self.edit_rejections = self.edit_rejections.saturating_add(1);
            }
        }
        if prompt.contains(
            "Work Leaf collected this context from commits, git logs, and recorded chat history",
        ) {
            self.bump(6);
            self.review_prompts = self.review_prompts.saturating_add(1);
            for hash in review_target_hashes(prompt) {
                *self.review_targets.entry(hash).or_default() += 1;
            }
        }
        if prompt.contains("work-leaf linearizer") && prompt.contains("Final patch targets") {
            self.bump(7);
            self.observe_linearize_targets(prompt);
        }
    }

    pub fn observe_assistant(&mut self, thread_id: &str, turn_id: &str, text: &str) {
        if text.lines().any(is_terminal_directive_line) {
            self.terminal_turns
                .entry((thread_id.to_string(), turn_id.to_string()))
                .or_default()
                .directive = true;
            self.bump(8);
        }
        if let Some(edit_start) = text.find("@work-leaf edit") {
            let body = text[edit_start..].trim().to_string();
            *self.edit_submissions.entry(body).or_default() += 1;
            self.bump(5);
        }
    }

    pub fn observe_interrupt(&mut self, thread_id: &str, turn_id: &str) {
        self.terminal_turns
            .entry((thread_id.to_string(), turn_id.to_string()))
            .or_default()
            .interrupted = true;
        self.bump(8);
    }

    pub fn observe_turn_outcome(&mut self, thread_id: &str, turn_id: &str, outcome: TurnOutcome) {
        self.terminal_turns
            .entry((thread_id.to_string(), turn_id.to_string()))
            .or_default()
            .outcome = Some(outcome);
    }

    pub fn observe_command(&mut self, observation: CommandObservation) {
        self.bump(10);
        let class = classify_command(&observation.command);
        *self.command_classes.entry(class).or_default() += 1;
        let signature = normalize_command_signature(&observation.command);
        let count = self.command_signatures.entry(signature).or_default();
        if *count > 0 {
            self.repeated_commands = self.repeated_commands.saturating_add(1);
        }
        *count = count.saturating_add(1);
        self.commands.push(observation);
    }

    pub fn observe_locked_command(&mut self, observation: LockedCommandObservation) {
        self.observe_locked_command_at(observation, None, None, true);
    }

    fn observe_locked_command_at(
        &mut self,
        observation: LockedCommandObservation,
        start_monotonic_ns: Option<u128>,
        end_monotonic_ns: Option<u128>,
        h4_candidate: bool,
    ) {
        self.bump(10);
        *self
            .command_classes
            .entry("locked-command".to_string())
            .or_default() += 1;
        self.locked_commands.push(TimedLockedCommand {
            observation,
            start_monotonic_ns,
            end_monotonic_ns,
            h4_candidate,
        });
    }

    pub fn observe_bundle_archive(&mut self, observation: BundleArchiveObservation) {
        let source_path = observation.source_path.clone();
        if self.bundle_archives.contains_key(&source_path) {
            self.errors.push(format!(
                "context bundle {} has more than one archived payload",
                source_path.display()
            ));
            return;
        }
        if let Ok(snapshots) = parse_context_bundle_snapshots(&observation.bytes) {
            let mut indexed = BTreeMap::new();
            for snapshot in snapshots {
                let path = snapshot.path.clone();
                if indexed.insert(path.clone(), snapshot).is_some() {
                    self.errors.push(format!(
                        "context bundle {} contains duplicate snapshot path {path}",
                        source_path.display()
                    ));
                }
            }
            self.bundle_snapshots.insert(source_path.clone(), indexed);
        }
        self.bundle_archives
            .insert(source_path.clone(), observation);
        let announcements = self
            .bundle_announcements
            .get(&source_path)
            .cloned()
            .unwrap_or_default();
        for announcement in &announcements {
            self.associate_bundle_snapshots(announcement);
        }
    }

    fn associate_bundle_snapshots(&mut self, announcement: &BundleAnnouncement) {
        let Some(snapshots) = self.bundle_snapshots.get(&announcement.path) else {
            return;
        };
        // Both sides are path indexes. Snapshot text is shared through Arc, so repeated
        // announcements copy only their declared path/digest metadata. Omitted archive rows
        // produce one aggregate error instead of announcement x archive fanout.
        let mut declared_archive_paths = 0_usize;
        let mut matched = Vec::new();
        let mut association_errors = Vec::new();
        for (path, digest) in &announcement.file_digests {
            match snapshots.get(path) {
                Some(snapshot) if digest == &snapshot.digest => {
                    declared_archive_paths += 1;
                    matched.push(snapshot.clone());
                }
                Some(snapshot) => {
                    declared_archive_paths += 1;
                    association_errors.push(format!(
                        "context bundle {} announced digest {digest} for {} but archived digest is {}",
                        announcement.path.display(),
                        snapshot.path,
                        snapshot.digest
                    ));
                }
                None => association_errors.push(format!(
                    "context bundle {} announced {} without a matching archived snapshot",
                    announcement.path.display(),
                    path
                )),
            }
        }
        let omitted = snapshots.len().saturating_sub(declared_archive_paths);
        if omitted > 0 {
            association_errors.push(format!(
                "context bundle {} archived {omitted} snapshot paths without matching announcement rows",
                announcement.path.display()
            ));
        }
        self.errors.extend(association_errors);
        for snapshot in matched {
            self.snapshots.insert(
                (announcement.thread_id.clone(), snapshot.path),
                SnapshotEvidence {
                    text: snapshot.text,
                    digest: snapshot.digest,
                },
            );
        }
    }

    pub fn observe_timeline(&mut self, observation: TimelineObservation) {
        self.bump(12);
        self.timeline_asserted = true;
        self.timelines.push(observation);
    }

    pub fn observe_git_commit_hashes(&mut self, hashes: impl IntoIterator<Item = String>) {
        self.bump(14);
        self.git_commit_hashes.extend(hashes);
    }

    pub fn observe_thread_role(&mut self, thread_id: &str, role: &str) {
        self.thread_roles
            .insert(thread_id.to_string(), role.to_string());
    }

    fn observe_protocol_component(&mut self, prompt: &str) {
        let component = if prompt.starts_with("work-leaf file text\n") {
            "file-read"
        } else if prompt.starts_with("work-leaf command result\n") {
            "command-result"
        } else if prompt.starts_with("work-leaf patch applied\n")
            || prompt.starts_with("work-leaf patch already applied\n")
        {
            "edit-acknowledgement"
        } else if prompt.contains("work-leaf linearizer") {
            "linearization"
        } else if prompt.contains("Work Leaf collected this context from commits") {
            "review-provenance"
        } else {
            "other-orchestration"
        };
        *self
            .protocol_components
            .entry(component.to_string())
            .or_default() += prompt.len() as u64;
    }

    fn observe_unchanged_reads(&mut self, thread_id: &str, prompt: &str) {
        let rows = parse_unchanged_read_rows(prompt);
        for row in &rows {
            let path = &row.path;
            let digest = &row.digest;
            let actual = row.raw.len() as u64;
            let key = (thread_id.to_string(), path.clone());
            if let Some(delivery) = &row.full_current_delivery {
                match &delivery.text {
                    Some(text) => {
                        self.counterfactuals.push(CounterfactualRecord {
                            hypothesis: "H1".to_string(),
                            status: "full-current-delivery".to_string(),
                            observed_prompt_bytes: Some(prompt.len() as u64),
                            actual_component_bytes: Some(delivery.component_bytes),
                            counterfactual_component_bytes: Some(actual),
                            avoided_bytes: Some(byte_difference(
                                actual,
                                delivery.component_bytes,
                            )),
                            note: format!(
                                "digest-verified full current text was delivered for {path} instead of an unchanged digest row"
                            ),
                        });
                        self.snapshots.insert(
                            key,
                            SnapshotEvidence {
                                digest: digest.clone(),
                                text: Arc::from(text.as_str()),
                            },
                        );
                    }
                    None => {
                        self.counterfactuals.push(CounterfactualRecord {
                            hypothesis: "H1".to_string(),
                            status: "invalid-full-current-delivery".to_string(),
                            observed_prompt_bytes: Some(prompt.len() as u64),
                            actual_component_bytes: Some(delivery.component_bytes),
                            counterfactual_component_bytes: Some(actual),
                            avoided_bytes: None,
                            note: format!(
                                "full current text for {path} did not match its declared digest {digest}"
                            ),
                        });
                        self.errors.push(format!(
                            "H1 full-current verification failed for thread {thread_id} path {path}"
                        ));
                    }
                }
                continue;
            }
            if let Some(snapshot) = self
                .snapshots
                .get(&key)
                .filter(|snapshot| snapshot.digest == digest.as_str())
            {
                let replacement = render_exact_snapshot_component(path, &snapshot.text);
                self.counterfactuals.push(CounterfactualRecord {
                    hypothesis: "H1".to_string(),
                    status: "verified".to_string(),
                    observed_prompt_bytes: Some(prompt.len() as u64),
                    actual_component_bytes: Some(actual),
                    counterfactual_component_bytes: Some(replacement.len() as u64),
                    avoided_bytes: Some(byte_difference(replacement.len() as u64, actual)),
                    note: format!(
                        "matched unchanged snapshot {path} against captured digest {digest}"
                    ),
                });
            } else if let Some(refresh) = self
                .omitted_refreshes
                .get(&key)
                .filter(|refresh| refresh.digest == digest.as_str())
            {
                let declared_bytes = refresh.declared_bytes;
                self.counterfactuals.push(CounterfactualRecord {
                    hypothesis: "H1".to_string(),
                    status: "source-text-omitted".to_string(),
                    observed_prompt_bytes: Some(prompt.len() as u64),
                    actual_component_bytes: Some(actual),
                    counterfactual_component_bytes: None,
                    avoided_bytes: None,
                    note: format!(
                        "matching prior refresh omitted exact source text for {path}; digest={digest}; declared_bytes={declared_bytes}"
                    ),
                });
            } else {
                self.counterfactuals.push(CounterfactualRecord {
                    hypothesis: "H1".to_string(),
                    status: "requires-snapshot-resolution".to_string(),
                    observed_prompt_bytes: Some(prompt.len() as u64),
                    actual_component_bytes: Some(actual),
                    counterfactual_component_bytes: None,
                    avoided_bytes: None,
                    note: format!("no captured prior snapshot for {path} matched digest {digest}"),
                });
                self.errors.push(format!(
                    "H1 snapshot resolution failed for thread {thread_id} path {path}"
                ));
            }
        }
        if rows.is_empty() {
            self.record_prompt_counterfactual(
                1,
                prompt,
                "requires-snapshot-resolution",
                "the unchanged-read heading contained no parseable path/digest row",
            );
        }
    }

    fn observe_changed_reads(&mut self, thread_id: &str, prompt: &str) {
        let blocks = parse_changed_read_blocks(prompt);
        let full_current_blocks = parse_changed_full_current_blocks(prompt);
        if blocks.is_empty() && full_current_blocks.is_empty() {
            self.record_prompt_counterfactual(
                2,
                prompt,
                "requires-diff-verification",
                "the changed-read heading contained no complete diff or full-current block",
            );
            return;
        }
        for block in blocks {
            let key = (thread_id.to_string(), block.path.clone());
            let previous = self.snapshots.get(&key).cloned();
            let validation = previous
                .as_ref()
                .filter(|snapshot| snapshot.digest == block.previous_digest)
                .and_then(|snapshot| {
                    apply_unified_diff(&snapshot.text, &block.diff)
                        .map(|current| (snapshot, current))
                });
            match validation {
                Some((_previous, current))
                    if content_digest_for_observer(&current) == block.current_digest =>
                {
                    let counterfactual = render_exact_snapshot_component(&block.path, &current);
                    self.counterfactuals.push(CounterfactualRecord {
                        hypothesis: "H2".to_string(),
                        status: "verified".to_string(),
                        observed_prompt_bytes: Some(prompt.len() as u64),
                        actual_component_bytes: Some(block.raw.len() as u64),
                        counterfactual_component_bytes: Some(counterfactual.len() as u64),
                        avoided_bytes: Some(byte_difference(
                            counterfactual.len() as u64,
                            block.raw.len() as u64,
                        )),
                        note: format!(
                            "applied captured diff for {} and verified previous/current digests",
                            block.path
                        ),
                    });
                    self.snapshots.insert(
                        key,
                        SnapshotEvidence {
                            digest: block.current_digest,
                            text: Arc::from(current),
                        },
                    );
                }
                _ => {
                    self.counterfactuals.push(CounterfactualRecord {
                        hypothesis: "H2".to_string(),
                        status: "invalid-diff-reconstruction".to_string(),
                        observed_prompt_bytes: Some(prompt.len() as u64),
                        actual_component_bytes: Some(block.raw.len() as u64),
                        counterfactual_component_bytes: None,
                        avoided_bytes: None,
                        note: format!(
                            "captured diff for {} did not reconstruct a digest-verified snapshot",
                            block.path
                        ),
                    });
                    self.errors.push(format!(
                        "H2 diff reconstruction failed for thread {thread_id} path {}",
                        block.path
                    ));
                }
            }
        }
        for block in full_current_blocks {
            let key = (thread_id.to_string(), block.path.clone());
            let previous_matches = self
                .snapshots
                .get(&key)
                .is_none_or(|snapshot| snapshot.digest == block.previous_digest);
            match block.text {
                Some(text)
                    if previous_matches
                        && content_digest_for_observer(&text) == block.current_digest =>
                {
                    self.counterfactuals.push(CounterfactualRecord {
                        hypothesis: "H2".to_string(),
                        status: "full-current-delivery".to_string(),
                        observed_prompt_bytes: Some(prompt.len() as u64),
                        actual_component_bytes: Some(block.component_bytes),
                        counterfactual_component_bytes: None,
                        avoided_bytes: None,
                        note: format!(
                            "captured full current text for {} and verified its digest; no diff counterfactual was reconstructed",
                            block.path
                        ),
                    });
                    self.snapshots.insert(
                        key,
                        SnapshotEvidence {
                            digest: block.current_digest,
                            text: Arc::from(text),
                        },
                    );
                }
                _ => {
                    self.counterfactuals.push(CounterfactualRecord {
                        hypothesis: "H2".to_string(),
                        status: "invalid-full-current-delivery".to_string(),
                        observed_prompt_bytes: Some(prompt.len() as u64),
                        actual_component_bytes: Some(block.component_bytes),
                        counterfactual_component_bytes: None,
                        avoided_bytes: None,
                        note: format!(
                            "full current text for {} did not establish a digest-consistent snapshot",
                            block.path
                        ),
                    });
                    self.errors.push(format!(
                        "H2 full-current verification failed for thread {thread_id} path {}",
                        block.path
                    ));
                }
            }
        }
    }

    fn observe_linearize_targets(&mut self, prompt: &str) {
        let Some(actual) = linearize_target_block(prompt) else {
            self.record_prompt_counterfactual(
                7,
                prompt,
                "requires-target-reconstruction",
                "the compact target block could not be isolated",
            );
            return;
        };
        let Some(counterfactual) = reconstruct_ungrouped_targets(actual) else {
            self.record_prompt_counterfactual(
                7,
                prompt,
                "requires-target-reconstruction",
                "the reviewed AgentCommit fields could not be reconstructed from the target block",
            );
            return;
        };
        self.counterfactuals.push(CounterfactualRecord {
            hypothesis: "H7".to_string(),
            status: "verified".to_string(),
            observed_prompt_bytes: Some(prompt.len() as u64),
            actual_component_bytes: Some(actual.len() as u64),
            counterfactual_component_bytes: Some(counterfactual.len() as u64),
            avoided_bytes: Some(byte_difference(
                counterfactual.len() as u64,
                actual.len() as u64,
            )),
            note: "reconstructed an ungrouped target block from the same reviewed commit fields"
                .to_string(),
        });
    }

    fn record_prompt_counterfactual(
        &mut self,
        hypothesis: usize,
        prompt: &str,
        status: &str,
        note: &str,
    ) {
        self.counterfactuals.push(CounterfactualRecord {
            hypothesis: format!("H{hypothesis}"),
            status: status.to_string(),
            observed_prompt_bytes: Some(prompt.len() as u64),
            actual_component_bytes: None,
            counterfactual_component_bytes: None,
            avoided_bytes: None,
            note: note.to_string(),
        });
    }

    fn bump(&mut self, number: usize) {
        self.counts[number - 1] = self.counts[number - 1].saturating_add(1);
    }

    pub fn finish(mut self) -> MechanismSummary {
        let validation = self.summarize_validation_activity();
        self.resolve_locked_commands();
        let bundles = self.resolve_bundles();
        let terminal_directives = self.summarize_terminal_directives();
        let sequential_timeline_valid = self.validate_timeline();
        let command_count = (self.commands.len() + self.locked_commands.len()) as u64;
        let command_output_bytes = self
            .commands
            .iter()
            .map(|command| command.output.len() as u64)
            .chain(self.locked_commands.iter().map(|command| {
                (command.observation.stdout.len() + command.observation.stderr.len()) as u64
            }))
            .fold(0_u64, u64::saturating_add);
        let command_duration_ns = self
            .commands
            .iter()
            .filter_map(|command| command.duration_ns)
            .chain(self.locked_commands.iter().filter_map(|command| {
                command
                    .end_monotonic_ns
                    .zip(command.start_monotonic_ns)
                    .map(|(end, start)| end.saturating_sub(start))
            }))
            .fold(0_u128, u128::saturating_add);
        let submissions = self.edit_submissions.values().copied().sum();
        let duplicate_submissions = self
            .edit_submissions
            .values()
            .map(|count| count.saturating_sub(1))
            .sum();
        let duplicate_targets = self
            .review_targets
            .values()
            .map(|count| count.saturating_sub(1))
            .sum();
        let (validated_targets, unresolved_targets) = self.validate_review_targets();
        MechanismSummary {
            hypotheses: self
                .counts
                .into_iter()
                .enumerate()
                .map(|(index, count)| HypothesisEvidence {
                    id: format!("H{}", index + 1),
                    observed: count > 0,
                    evidence_count: count,
                })
                .collect(),
            protocol_bytes: self.protocol_bytes,
            command_classes: self.command_classes,
            command_count,
            command_output_bytes,
            command_duration_ns,
            repeated_commands: self.repeated_commands,
            counterfactuals: self.counterfactuals,
            bundles,
            terminal_directives,
            structured_edits: StructuredEditSummary {
                submissions,
                duplicate_submissions,
                acknowledgements: self.edit_acknowledgements,
                rejections: self.edit_rejections,
            },
            reviews: ReviewEvidenceSummary {
                prompts: self.review_prompts,
                unique_targets: self.review_targets.len() as u64,
                duplicate_targets,
                validated_targets,
                unresolved_targets,
            },
            validation,
            protocol_components: self.protocol_components,
            sequential_timeline_valid,
            errors: self.errors,
        }
    }

    fn validate_review_targets(&mut self) -> (u64, u64) {
        const MAX_GIT_OBJECT_ID_CHARS: usize = 64;

        let mut validated = 0_u64;
        let mut unresolved = 0_u64;
        let mut matches_by_prefix = self
            .review_targets
            .keys()
            .cloned()
            .map(|target| (target, 0_u64))
            .collect::<BTreeMap<_, _>>();
        // Git object IDs are bounded hexadecimal strings. Index every possible abbreviated prefix
        // in one bounded pass over commits, then resolve each review target by map lookup.
        for commit in &self.git_commit_hashes {
            if !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                continue;
            }
            for length in 7..=commit.len().min(MAX_GIT_OBJECT_ID_CHARS) {
                if let Some(matches) = matches_by_prefix.get_mut(&commit[..length]) {
                    *matches = matches.saturating_add(1);
                }
            }
        }
        for (target, matches) in matches_by_prefix {
            if matches == 1 {
                validated = validated.saturating_add(1);
            } else {
                unresolved = unresolved.saturating_add(1);
                self.errors.push(format!(
                    "review target {target} resolves to {matches} captured Git commits"
                ));
            }
        }
        if self.review_prompts > 0 && self.review_targets.is_empty() {
            self.errors
                .push("review prompt contains no parseable target commit".to_string());
        }
        (validated, unresolved)
    }

    fn summarize_validation_activity(&self) -> ValidationBudgetSummary {
        let mut summary = ValidationBudgetSummary::default();
        let commands = self
            .commands
            .iter()
            .map(|command| {
                (
                    command.thread_id.clone(),
                    command.turn_id.clone(),
                    command.command.clone(),
                )
            })
            .chain(self.command_results.values().flatten().map(|result| {
                (
                    result.thread_id.clone(),
                    result.turn_id.clone(),
                    result.key.command.clone(),
                )
            }))
            .collect::<BTreeSet<_>>();

        for (_, _, command) in commands {
            let validations = cargo_validation_segments(&command).len() as u64;
            summary.validation_commands = summary.validation_commands.saturating_add(validations);
        }
        summary
    }

    fn resolve_locked_commands(&mut self) {
        self.locked_commands
            .sort_by_key(|command| command.start_monotonic_ns.unwrap_or(u128::MAX));
        // Every locked command and rendered result belongs to one digest-keyed bucket. Within a
        // bucket, the time-ordered pending set visits each candidate and result once. Unrelated
        // commands are never compared, so pairing is O((C + R) log(C + R)), not C x R.
        let mut candidate_indexes = BTreeMap::<CommandResultKey, Vec<usize>>::new();
        for (index, command) in self.locked_commands.iter().enumerate() {
            if command.h4_candidate {
                candidate_indexes
                    .entry(locked_command_result_key(&command.observation))
                    .or_default()
                    .push(index);
            }
        }
        let mut used = BTreeSet::new();
        let result_groups = std::mem::take(&mut self.command_results);
        for (key, results) in result_groups {
            let candidates = candidate_indexes.remove(&key).unwrap_or_default();
            let pairings =
                indexed_locked_command_pairings(&candidates, &results, &self.locked_commands);
            for (result, pairing) in results.iter().zip(pairings) {
                let Some(index) = pairing.candidate else {
                    self.counterfactuals.push(CounterfactualRecord {
                        hypothesis: "H4".to_string(),
                        status: "ambiguous-locked-command-pairing".to_string(),
                        observed_prompt_bytes: Some(result.prompt_bytes),
                        actual_component_bytes: None,
                        counterfactual_component_bytes: None,
                        avoided_bytes: None,
                        note: format!(
                            "{} candidate locked commands matched thread {} turn {} command {}",
                            pairing.eligible, result.thread_id, result.turn_id, result.key.command
                        ),
                    });
                    self.errors.push(format!(
                        "ambiguous locked-command pairing for thread {} turn {}: {} candidates",
                        result.thread_id, result.turn_id, pairing.eligible
                    ));
                    continue;
                };
                used.insert(index);
                let command = &self.locked_commands[index].observation;
                let expected_status = command
                    .exit_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "terminated".to_string());
                let timeout_metadata_valid = !result.timed_out
                    || (result.timeout.is_some()
                        && (command.terminating_signal.is_some() || command.exit_code != Some(0)));
                if result.status != expected_status || !timeout_metadata_valid {
                    self.counterfactuals.push(CounterfactualRecord {
                        hypothesis: "H4".to_string(),
                        status: "process-metadata-mismatch".to_string(),
                        observed_prompt_bytes: Some(result.prompt_bytes),
                        actual_component_bytes: None,
                        counterfactual_component_bytes: None,
                        avoided_bytes: None,
                        note: format!(
                            "rendered status={} timed_out={} timeout={:?}; captured exit={:?} signal={:?}",
                            result.status,
                            result.timed_out,
                            result.timeout,
                            command.exit_code,
                            command.terminating_signal
                        ),
                    });
                    self.errors.push(format!(
                        "H4 status mismatch for locked invocation {}: rendered {} versus captured {}",
                        command.invocation_id, result.status, expected_status
                    ));
                    continue;
                }
                let expected_stdout = render_observed_command_output(&command.stdout);
                let expected_stderr = render_observed_command_output(&command.stderr);
                let counterfactual_stdout = render_uncompacted_command_output(&command.stdout);
                let counterfactual_stderr = render_uncompacted_command_output(&command.stderr);
                let actual = (expected_stdout.len() + expected_stderr.len()) as u64;
                let counterfactual =
                    (counterfactual_stdout.len() + counterfactual_stderr.len()) as u64;
                self.counterfactuals.push(CounterfactualRecord {
                    hypothesis: "H4".to_string(),
                    status: "verified".to_string(),
                    observed_prompt_bytes: Some(result.prompt_bytes),
                    actual_component_bytes: Some(actual),
                    counterfactual_component_bytes: Some(counterfactual),
                    avoided_bytes: Some(byte_difference(counterfactual, actual)),
                    note: format!(
                        "paired rendered result with process-backed locked invocation {}; status={}; timed_out={}; timeout={:?}",
                        command.invocation_id, result.status, result.timed_out, result.timeout
                    ),
                });
            }
        }
        for (index, command) in self.locked_commands.iter().enumerate() {
            if command.h4_candidate && !used.contains(&index) {
                self.errors.push(format!(
                    "locked invocation {} has no rendered command-result prompt",
                    command.observation.invocation_id
                ));
            }
        }
    }

    fn resolve_bundles(&mut self) -> Vec<BundleMeasurement> {
        // Bundle paths are explicit shell arguments in captured follow-up reads. Build this index
        // with one token pass over commands, then resolve announcements and archives by path. A
        // command may populate only a fixed number of path buckets, so later payload searches stay
        // linear in captured output size apart from that fixed factor.
        let known_paths = self
            .bundle_announcements
            .keys()
            .chain(self.bundle_archives.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut outputs_by_path = BTreeMap::<PathBuf, Vec<&[u8]>>::new();
        for command in &self.commands {
            let paths = bundle_paths_in_command(&command.command, &known_paths);
            if paths.len() > MAX_BUNDLE_PATHS_PER_COMMAND {
                self.errors.push(format!(
                    "command in thread {} turn {} references {} context bundle paths; at most {} context bundle paths may be resolved from one command",
                    command.thread_id,
                    command.turn_id,
                    paths.len(),
                    MAX_BUNDLE_PATHS_PER_COMMAND
                ));
                continue;
            }
            for path in paths {
                outputs_by_path
                    .entry(path)
                    .or_default()
                    .push(command.output.as_slice());
            }
        }

        let mut measurements = Vec::new();
        for (path, announcements) in &self.bundle_announcements {
            let Some(archive) = self.bundle_archives.get(path) else {
                for announcement in announcements {
                    self.errors.push(format!(
                        "announced context bundle {} has no archived payload",
                        announcement.path.display()
                    ));
                }
                continue;
            };
            let matching_outputs = outputs_by_path.get(path).map(Vec::as_slice).unwrap_or(&[]);
            let observed_follow_up_bytes = matching_outputs
                .iter()
                .map(|output| output.len() as u64)
                .sum::<u64>();
            // Build the archive search table once, then scan each associated command output once.
            // The total work for one archive is linear in its bytes plus its observed output bytes.
            let archive_search_prefix = byte_search_prefix(&archive.bytes);
            let full = matching_outputs.iter().any(|output| {
                bytes_contain_with_prefix(output, &archive.bytes, &archive_search_prefix)
            });
            let consumption = if full {
                "full"
            } else if observed_follow_up_bytes > 0 {
                "partial"
            } else {
                "unread"
            };
            let payload_bytes = archive.bytes.len() as u64;
            for announcement in announcements {
                let deferred_bytes = byte_difference(payload_bytes, announcement.manifest_bytes);
                let observed_path_net_bytes = byte_difference(
                    payload_bytes,
                    announcement
                        .manifest_bytes
                        .saturating_add(observed_follow_up_bytes),
                );
                measurements.push(BundleMeasurement {
                    thread_id: announcement.thread_id.clone(),
                    turn_id: announcement.turn_id.clone(),
                    source_path: announcement.path.clone(),
                    archived_path: archive.archived_path.clone(),
                    payload_bytes,
                    manifest_bytes: announcement.manifest_bytes,
                    observed_follow_up_bytes,
                    consumption: consumption.to_string(),
                    deferred_bytes,
                    observed_path_net_bytes,
                    sha256: archive.sha256.clone(),
                });
            }
        }
        for path in self.bundle_archives.keys() {
            if !self.bundle_announcements.contains_key(path) {
                self.errors.push(format!(
                    "archived context bundle {} was never announced in captured prompts",
                    path.display()
                ));
            }
        }
        measurements.sort_by(|left, right| left.source_path.cmp(&right.source_path));
        measurements
    }

    fn summarize_terminal_directives(&mut self) -> TerminalDirectiveSummary {
        let mut summary = TerminalDirectiveSummary::default();
        for ((thread_id, turn_id), turn) in &self.terminal_turns {
            if !turn.directive {
                continue;
            }
            summary.observed = summary.observed.saturating_add(1);
            match (turn.interrupted, turn.outcome) {
                (true, Some(TurnOutcome::Interrupted)) => {
                    summary.interrupted = summary.interrupted.saturating_add(1)
                }
                (false, Some(TurnOutcome::Completed)) => {
                    summary.naturally_completed = summary.naturally_completed.saturating_add(1)
                }
                (_, Some(TurnOutcome::Failed)) => summary.failed = summary.failed.saturating_add(1),
                _ => {
                    summary.unresolved = summary.unresolved.saturating_add(1);
                    self.errors.push(format!(
                        "terminal directive outcome is unresolved for thread {thread_id} turn {turn_id}"
                    ));
                }
            }
        }
        if summary.interrupted > 0 {
            self.counterfactuals.push(CounterfactualRecord {
                hypothesis: "H8".to_string(),
                status: "requires-ablation".to_string(),
                observed_prompt_bytes: None,
                actual_component_bytes: None,
                counterfactual_component_bytes: None,
                avoided_bytes: None,
                note: "interrupted terminal directives are measured, but prevented generation is not observable"
                    .to_string(),
            });
        }
        summary
    }

    fn validate_timeline(&mut self) -> bool {
        if self.timelines.is_empty() {
            return self.timeline_asserted;
        }
        self.timelines
            .sort_by_key(|observation| observation.monotonic_ns);
        if self.timelines.iter().any(|observation| {
            observation.event == "condition-start"
                && observation.detail.as_deref().is_some_and(|detail| {
                    detail
                        .split_whitespace()
                        .any(|item| item == "schedule=concurrent")
                })
        }) {
            return true;
        }
        let mut active_feature = None::<String>;
        let mut valid = true;
        for observation in &self.timelines {
            let feature = observation
                .detail
                .as_deref()
                .and_then(|detail| {
                    detail
                        .split_whitespace()
                        .find_map(|item| item.strip_prefix("feature="))
                })
                .map(str::to_string);
            match observation.event.as_str() {
                "feature-start" => {
                    if active_feature.is_some() || feature.is_none() {
                        valid = false;
                    } else {
                        active_feature = feature;
                    }
                }
                "feature-cycle-complete" => {
                    if active_feature.as_ref() != feature.as_ref() {
                        valid = false;
                    }
                    active_feature = None;
                }
                "linearize-start" if active_feature.is_some() => valid = false,
                _ => {}
            }
        }
        if active_feature.is_some() {
            valid = false;
        }
        if !valid {
            self.errors.push(
                "sequential timeline contains overlap or unmatched feature boundaries".to_string(),
            );
        }
        valid
    }
}

#[derive(Clone, Debug)]
struct ChangedReadBlock {
    path: String,
    previous_digest: String,
    current_digest: String,
    diff: String,
    raw: String,
}

#[derive(Clone, Debug)]
struct ChangedFullCurrentBlock {
    path: String,
    previous_digest: String,
    current_digest: String,
    text: Option<String>,
    component_bytes: u64,
}

#[derive(Clone, Debug)]
struct UnchangedReadRow {
    path: String,
    digest: String,
    raw: String,
    full_current_delivery: Option<FullCurrentDelivery>,
}

#[derive(Clone, Debug)]
struct FullCurrentDelivery {
    text: Option<String>,
    component_bytes: u64,
}

#[derive(Clone, Debug)]
struct FileSectionHeader {
    start: usize,
    body_start: usize,
    path: String,
}

fn file_section_headers(text: &str) -> Vec<FileSectionHeader> {
    let mut headers = Vec::new();
    let mut offset = 0_usize;
    for line_with_ending in text.split_inclusive('\n') {
        let line = line_with_ending
            .strip_suffix('\n')
            .unwrap_or(line_with_ending);
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some(path) = line
            .strip_prefix("--- ")
            .and_then(|line| line.strip_suffix(" ---"))
        {
            headers.push(FileSectionHeader {
                start: offset,
                body_start: offset + line_with_ending.len(),
                path: path.to_string(),
            });
        }
        offset += line_with_ending.len();
    }
    headers
}

fn parse_exact_file_sections(prompt: &str) -> Vec<(String, String)> {
    let headers = file_section_headers(prompt);
    headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            let section_end = headers
                .get(index + 1)
                .map(|next| next.start.saturating_sub(1))
                .unwrap_or(prompt.len());
            // Marker searches stay inside this disjoint file section, so parsing all headers is
            // linear in the prompt instead of rescanning the remaining prompt for every header.
            let section = &prompt[header.body_start..section_end];
            let next_section = [
                "\nRepeated file reads with changes",
                "\nRepeated file reads unchanged",
                "\nUnavailable file text",
            ]
            .into_iter()
            .filter_map(|marker| section.find(marker))
            .map(|relative| header.body_start + relative)
            .min();
            let end = next_section.unwrap_or(section_end);
            (
                header.path.clone(),
                prompt[header.body_start..end].to_string(),
            )
        })
        .collect()
}

fn parse_unchanged_read_rows(prompt: &str) -> Vec<UnchangedReadRow> {
    let Some((_, section)) = prompt.split_once("Repeated file reads unchanged") else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    let mut cursor = 0_usize;
    while cursor < section.len() {
        let Some((raw_line, next_cursor)) = line_at(section, cursor) else {
            break;
        };
        let line = raw_line.trim_end_matches(['\r', '\n']).trim();
        if line.is_empty() {
            cursor = next_cursor;
            continue;
        }
        if let Some(item) = line.strip_prefix("- ")
            && let Some((path, digest_with_end)) = item.rsplit_once(" (")
            && let Some(digest) = digest_with_end.strip_suffix(')')
            && digest.starts_with("fnv64:")
        {
            let mut row = UnchangedReadRow {
                path: path.to_string(),
                digest: digest.to_string(),
                raw: raw_line.to_string(),
                full_current_delivery: None,
            };
            cursor = next_cursor;
            let marker = format!("full current text follows for {path}:");
            if let Some((marker_line, payload_start)) = line_at(section, cursor)
                && marker_line.trim_end_matches(['\r', '\n']) == marker
            {
                let declared_bytes = digest_declared_bytes(digest);
                let payload_end = declared_bytes
                    .and_then(|bytes| payload_start.checked_add(bytes))
                    .filter(|end| *end <= section.len());
                let text = payload_end.and_then(|end| {
                    verified_digest_text(&section[payload_start..end], digest)
                        .map(ToOwned::to_owned)
                });
                let component_end = payload_end.unwrap_or(section.len());
                row.full_current_delivery = Some(FullCurrentDelivery {
                    text,
                    component_bytes: component_end
                        .saturating_sub(cursor.saturating_sub(raw_line.len()))
                        as u64,
                });
                rows.push(row);
                if let Some(payload_end) = payload_end {
                    cursor = payload_end;
                    continue;
                }
                break;
            }
            rows.push(row);
            continue;
        }
        if !rows.is_empty()
            || line.starts_with("Repeated file reads")
            || line.starts_with("Unavailable file text")
            || line.starts_with("work-leaf file text")
            || (line.starts_with("--- ") && line.ends_with(" ---"))
        {
            break;
        }
        cursor = next_cursor;
    }
    rows
}

fn line_at(text: &str, start: usize) -> Option<(&str, usize)> {
    let tail = text.get(start..)?;
    if tail.is_empty() {
        return None;
    }
    let length = tail.find('\n').map_or(tail.len(), |index| index + 1);
    Some((&tail[..length], start + length))
}

fn digest_declared_bytes(digest: &str) -> Option<usize> {
    digest
        .rsplit_once("; bytes:")
        .and_then(|(_, bytes)| bytes.parse().ok())
}

fn verified_digest_text<'a>(text: &'a str, digest: &str) -> Option<&'a str> {
    (content_digest_for_observer(text) == digest).then_some(text)
}

fn parse_changed_read_blocks(prompt: &str) -> Vec<ChangedReadBlock> {
    let mut blocks = Vec::new();
    let starts = file_section_headers(prompt)
        .into_iter()
        .filter(|header| prompt[header.body_start..].starts_with("current digest: "))
        .collect::<Vec<_>>();
    for (index, header) in starts.iter().enumerate() {
        let next = starts
            .get(index + 1)
            .map(|next| next.start)
            .unwrap_or(prompt.len());
        let raw = &prompt[header.start..next];
        let current_digest = raw
            .lines()
            .find_map(|line| line.strip_prefix("current digest: "));
        let previous_digest = raw
            .lines()
            .find_map(|line| line.strip_prefix("previous digest: "));
        let diff_start = raw.find("diff --git ");
        if let (Some(current_digest), Some(previous_digest), Some(diff_start)) =
            (current_digest, previous_digest, diff_start)
        {
            let diff_tail = &raw[diff_start..];
            let diff_end = [
                "\nRepeated file reads unchanged",
                "\nUnavailable file text",
                "\nwork-leaf file text",
            ]
            .into_iter()
            .filter_map(|marker| diff_tail.find(marker))
            .min()
            .unwrap_or(diff_tail.len());
            blocks.push(ChangedReadBlock {
                path: header.path.clone(),
                previous_digest: previous_digest.to_string(),
                current_digest: current_digest.to_string(),
                diff: diff_tail[..diff_end].to_string(),
                raw: raw.to_string(),
            });
        }
    }
    blocks
}

fn parse_changed_full_current_blocks(prompt: &str) -> Vec<ChangedFullCurrentBlock> {
    let headers = file_section_headers(prompt);
    let mut blocks = Vec::new();
    for (index, header) in headers.iter().enumerate() {
        let section_end = headers
            .get(index + 1)
            .map(|next| next.start)
            .unwrap_or(prompt.len());
        let Some(section) = prompt.get(header.body_start..section_end) else {
            continue;
        };
        if !section.starts_with("current digest: ") {
            continue;
        }
        let current_digest = section
            .lines()
            .find_map(|line| line.strip_prefix("current digest: "));
        let previous_digest = section
            .lines()
            .find_map(|line| line.strip_prefix("previous digest: "));
        let marker = "full current text follows:\n";
        let marker_end = section.find(marker).map(|start| start + marker.len());
        let (Some(current_digest), Some(previous_digest), Some(marker_end)) =
            (current_digest, previous_digest, marker_end)
        else {
            continue;
        };
        let payload = &section[marker_end..];
        let payload_end =
            digest_declared_bytes(current_digest).filter(|bytes| *bytes <= payload.len());
        let text = payload_end.and_then(|end| {
            verified_digest_text(&payload[..end], current_digest).map(ToOwned::to_owned)
        });
        let component_end = payload_end.unwrap_or(payload.len());
        blocks.push(ChangedFullCurrentBlock {
            path: header.path.clone(),
            previous_digest: previous_digest.to_string(),
            current_digest: current_digest.to_string(),
            text,
            component_bytes: (header.body_start - header.start + marker_end + component_end) as u64,
        });
    }
    blocks
}

fn apply_unified_diff(previous: &str, diff: &str) -> Option<String> {
    let previous_lines = previous.split_inclusive('\n').collect::<Vec<_>>();
    let mut diff_lines = diff.split_inclusive('\n').collect::<Vec<_>>();
    while diff_lines
        .last()
        .is_some_and(|line| matches!(*line, "\n" | "\r\n"))
    {
        diff_lines.pop();
    }
    let mut output = String::new();
    let mut previous_index = 0_usize;
    let mut index = 0_usize;
    let mut saw_hunk = false;
    while index < diff_lines.len() {
        let line = diff_lines[index];
        if !line.starts_with("@@ ") {
            index += 1;
            continue;
        }
        saw_hunk = true;
        let old_start = parse_hunk_old_start(line)?;
        let target_index = old_start.saturating_sub(1);
        if target_index < previous_index || target_index > previous_lines.len() {
            return None;
        }
        for line in &previous_lines[previous_index..target_index] {
            output.push_str(line);
        }
        previous_index = target_index;
        index += 1;
        while index < diff_lines.len() && !diff_lines[index].starts_with("@@ ") {
            let change = diff_lines[index];
            if change.starts_with("diff --git ")
                || change.starts_with("--- ")
                || change.starts_with("+++ ")
            {
                break;
            }
            let no_newline = diff_lines
                .get(index + 1)
                .is_some_and(|line| line.starts_with("\\ No newline at end of file"));
            let raw_content = &change[1..];
            let content = if no_newline {
                raw_content.strip_suffix('\n').unwrap_or(raw_content)
            } else {
                raw_content
            };
            match change.as_bytes().first().copied() {
                Some(b' ') => {
                    if previous_lines.get(previous_index).copied() != Some(content) {
                        return None;
                    }
                    output.push_str(content);
                    previous_index += 1;
                }
                Some(b'-') => {
                    if previous_lines.get(previous_index).copied() != Some(content) {
                        return None;
                    }
                    previous_index += 1;
                }
                Some(b'+') => output.push_str(content),
                Some(b'\\') => {}
                _ => return None,
            }
            index += if no_newline { 2 } else { 1 };
        }
    }
    if !saw_hunk {
        return None;
    }
    for line in &previous_lines[previous_index..] {
        output.push_str(line);
    }
    Some(output)
}

fn parse_hunk_old_start(line: &str) -> Option<usize> {
    let old = line.strip_prefix("@@ -")?.split_whitespace().next()?;
    old.split(',').next()?.parse().ok()
}

fn content_digest_for_observer(text: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv64:{hash:016x}; bytes:{}", text.len())
}

#[derive(Clone, Debug)]
struct OmittedRefreshRecord {
    path: String,
    digest: String,
    declared_bytes: u64,
}

fn parse_omitted_refresh_snapshots(prompt: &str) -> Vec<OmittedRefreshRecord> {
    if prompt.contains(
        "Work Leaf collected this context from commits, git logs, and recorded chat history",
    ) || prompt.contains("work-leaf linearizer")
    {
        return Vec::new();
    }
    let refresh = if prompt.starts_with("work-leaf file refresh\n") {
        prompt
    } else if let Some((_, suffix)) = prompt.rsplit_once("\n\nwork-leaf file refresh\n") {
        suffix
    } else {
        return Vec::new();
    };

    let mut records = Vec::new();
    let mut path = None::<String>;
    let mut digest = None::<String>;
    let mut has_untracked_status = false;
    for line in refresh.lines() {
        if let Some(candidate) = line
            .strip_prefix("--- ")
            .and_then(|line| line.strip_suffix(" ---"))
        {
            path = Some(candidate.to_string());
            digest = None;
            has_untracked_status = false;
            continue;
        }
        if let Some(candidate) = line.strip_prefix("current digest: ") {
            digest = Some(candidate.to_string());
            continue;
        }
        if line == "status: no previous snapshot recorded for this agent" {
            has_untracked_status = true;
            continue;
        }
        let Some(bytes) = line
            .strip_prefix("current file text omitted: file is ")
            .and_then(|line| line.split_once(" bytes."))
            .and_then(|(bytes, _)| bytes.parse::<u64>().ok())
        else {
            continue;
        };
        let (Some(path), Some(digest)) = (path.as_ref(), digest.as_ref()) else {
            continue;
        };
        let digest_bytes = digest
            .rsplit_once("; bytes:")
            .and_then(|(_, bytes)| bytes.parse::<u64>().ok());
        if has_untracked_status && digest_bytes == Some(bytes) {
            records.push(OmittedRefreshRecord {
                path: path.clone(),
                digest: digest.clone(),
                declared_bytes: bytes,
            });
        }
    }
    records
}

fn render_exact_snapshot_component(path: &str, text: &str) -> String {
    let mut component = format!("\n--- {path} ---\n{text}");
    if !text.ends_with('\n') {
        component.push('\n');
    }
    component
}

fn parse_declared_bundle_files(prompt: &str) -> BTreeMap<String, String> {
    prompt
        .lines()
        .skip_while(|line| line.trim() != "Bundled files:")
        .skip(1)
        .take_while(|line| line.trim_start().starts_with("- "))
        .filter_map(|line| {
            let item = line.trim_start().strip_prefix("- ")?;
            let (path, digest_with_end) = item.rsplit_once(" (")?;
            let digest = digest_with_end.strip_suffix(')')?;
            digest
                .starts_with("fnv64:")
                .then(|| (path.to_string(), digest.to_string()))
        })
        .collect()
}

fn parse_command_result(
    thread_id: &str,
    turn_id: &str,
    prompt: &str,
    received_monotonic_ns: Option<u128>,
) -> Option<CommandResultEvidence> {
    let command = prompt
        .lines()
        .find_map(|line| line.strip_prefix("command: "))?
        .to_string();
    let status = prompt
        .lines()
        .find_map(|line| line.strip_prefix("status: "))?
        .to_string();
    let timed_out = prompt.lines().any(|line| line == "timed out: yes");
    let timeout = prompt
        .lines()
        .find_map(|line| line.strip_prefix("timeout: "))
        .map(str::to_string);
    if timed_out != timeout.is_some() {
        return None;
    }
    let (_, rendered_output) = prompt.split_once("\nstdout:\n")?;
    rendered_output.find("\nstderr:\n")?;
    let key = command_result_key(&command, rendered_output);
    Some(CommandResultEvidence {
        thread_id: thread_id.to_string(),
        turn_id: turn_id.to_string(),
        key,
        status,
        timed_out,
        timeout,
        prompt_bytes: prompt.len() as u64,
        received_monotonic_ns,
    })
}

fn command_result_key(command: &str, rendered_output: &str) -> CommandResultKey {
    CommandResultKey {
        command: command.to_string(),
        rendered_output_sha256: sha256_bytes(rendered_output.as_bytes()),
    }
}

fn locked_command_result_key(command: &LockedCommandObservation) -> CommandResultKey {
    let rendered_stdout = render_observed_command_output(&command.stdout);
    let rendered_stderr = render_observed_command_output(&command.stderr);
    command_result_key(
        &command.command,
        &format!("{rendered_stdout}stderr:\n{rendered_stderr}"),
    )
}

#[derive(Clone, Copy, Debug, Default)]
struct LockedCommandPairing {
    candidate: Option<usize>,
    eligible: usize,
}

fn indexed_locked_command_pairings(
    candidate_indexes: &[usize],
    results: &[CommandResultEvidence],
    commands: &[TimedLockedCommand],
) -> Vec<LockedCommandPairing> {
    let mut timeless = BTreeSet::new();
    let mut timed = Vec::new();
    for index in candidate_indexes.iter().copied() {
        match commands[index].end_monotonic_ns {
            Some(end) => timed.push((end, index)),
            None => {
                timeless.insert(index);
            }
        }
    }
    timed.sort_unstable();

    let mut result_order = results
        .iter()
        .enumerate()
        .map(|(index, result)| (result.received_monotonic_ns.unwrap_or(u128::MAX), index))
        .collect::<Vec<_>>();
    result_order.sort_unstable();

    let mut pairings = vec![LockedCommandPairing::default(); results.len()];
    let mut pending = timeless;
    let mut timed_index = 0;
    for (prompt_time, result_index) in result_order {
        while timed_index < timed.len()
            && (prompt_time == u128::MAX || timed[timed_index].0 <= prompt_time)
        {
            pending.insert(timed[timed_index].1);
            timed_index += 1;
        }
        let eligible = pending.len();
        let candidate =
            (eligible == 1).then(|| pending.first().copied().expect("one pending candidate"));
        if let Some(candidate) = candidate {
            pending.remove(&candidate);
        }
        pairings[result_index] = LockedCommandPairing {
            candidate,
            eligible,
        };
    }
    pairings
}

const OBSERVER_COMMAND_OUTPUT_MAX_CHARS: usize = 12_000;
const OBSERVER_COMMAND_OUTPUT_HEAD_CHARS: usize = 6_000;
const OBSERVER_COMMAND_OUTPUT_TAIL_CHARS: usize = 4_000;
const OBSERVER_COMMAND_OUTPUT_LONG_LINE_CHARS: usize = 4_096;
const OBSERVER_COMMAND_OUTPUT_LONG_LINE_EDGE_CHARS: usize = 1_600;
const OBSERVER_COMMAND_OUTPUT_BLANK_RUN_INLINE: usize = 8;

fn render_observed_command_output(bytes: &[u8]) -> String {
    let output = String::from_utf8_lossy(bytes);
    if output.is_empty() {
        return "<empty>\n".to_string();
    }
    let mut rendered = compact_observed_blank_runs(&output);
    rendered = compact_observed_long_lines(&rendered);
    rendered = compact_observed_total_chars(&rendered);
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
}

fn render_uncompacted_command_output(bytes: &[u8]) -> String {
    let output = String::from_utf8_lossy(bytes);
    if output.is_empty() {
        return "<empty>\n".to_string();
    }
    let mut rendered = output.to_string();
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
}

fn compact_observed_blank_runs(output: &str) -> String {
    let mut compacted = String::new();
    let mut blank_run = 0_usize;
    for line in output.split_inclusive('\n') {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run <= OBSERVER_COMMAND_OUTPUT_BLANK_RUN_INLINE {
                compacted.push_str(line);
            }
            continue;
        }
        if blank_run > OBSERVER_COMMAND_OUTPUT_BLANK_RUN_INLINE {
            let omitted = blank_run - OBSERVER_COMMAND_OUTPUT_BLANK_RUN_INLINE;
            compacted.push_str(&format!(
                "[work-leaf compacted {omitted} whitespace-only output lines]\n"
            ));
        }
        blank_run = 0;
        compacted.push_str(line);
    }
    if blank_run > OBSERVER_COMMAND_OUTPUT_BLANK_RUN_INLINE {
        let omitted = blank_run - OBSERVER_COMMAND_OUTPUT_BLANK_RUN_INLINE;
        compacted.push_str(&format!(
            "[work-leaf compacted {omitted} whitespace-only output lines]\n"
        ));
    }
    compacted
}

fn compact_observed_long_lines(output: &str) -> String {
    let mut compacted = String::new();
    for line in output.split_inclusive('\n') {
        let had_newline = line.ends_with('\n');
        let content = line.strip_suffix('\n').unwrap_or(line);
        let chars = content.chars().count();
        if chars <= OBSERVER_COMMAND_OUTPUT_LONG_LINE_CHARS {
            compacted.push_str(line);
            continue;
        }
        let omitted = chars.saturating_sub(OBSERVER_COMMAND_OUTPUT_LONG_LINE_EDGE_CHARS * 2);
        compacted.extend(
            content
                .chars()
                .take(OBSERVER_COMMAND_OUTPUT_LONG_LINE_EDGE_CHARS),
        );
        compacted.push_str(&format!(
            "\n[work-leaf compacted {omitted} characters from one long output line]\n"
        ));
        let mut tail = content
            .chars()
            .rev()
            .take(OBSERVER_COMMAND_OUTPUT_LONG_LINE_EDGE_CHARS)
            .collect::<Vec<_>>();
        tail.reverse();
        compacted.extend(tail);
        if had_newline {
            compacted.push('\n');
        }
    }
    compacted
}

fn compact_observed_total_chars(output: &str) -> String {
    let chars = output.chars().count();
    if chars <= OBSERVER_COMMAND_OUTPUT_MAX_CHARS {
        return output.to_string();
    }
    let omitted = chars
        .saturating_sub(OBSERVER_COMMAND_OUTPUT_HEAD_CHARS + OBSERVER_COMMAND_OUTPUT_TAIL_CHARS);
    let head = output
        .chars()
        .take(OBSERVER_COMMAND_OUTPUT_HEAD_CHARS)
        .collect::<String>();
    let mut tail = output
        .chars()
        .rev()
        .take(OBSERVER_COMMAND_OUTPUT_TAIL_CHARS)
        .collect::<Vec<_>>();
    tail.reverse();
    format!(
        "{head}\n[work-leaf compacted {omitted} characters from command output]\n{}",
        tail.into_iter().collect::<String>()
    )
}

fn normalize_command_signature(command: &str) -> String {
    command.to_string()
}

fn is_terminal_directive_line(line: &str) -> bool {
    matches!(
        line.trim(),
        "@work-leaf done" | "@work-leaf patch" | "@work-leaf review done"
    )
}

fn review_target_hashes(prompt: &str) -> Vec<String> {
    prompt
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("Latest commit: ")
                .or_else(|| line.trim().strip_prefix("Commit: "))
        })
        .flat_map(|value| value.split_whitespace())
        .filter(|word| {
            word.len() >= 7 && word.chars().all(|character| character.is_ascii_hexdigit())
        })
        .map(str::to_string)
        .collect()
}

fn linearize_target_block(prompt: &str) -> Option<&str> {
    let marker = "Final patch targets (";
    let heading = prompt.find(marker)?;
    let start = prompt[heading..].find('\n')? + heading + 1;
    let end = prompt[start..]
        .find("\nScope and commit-shaping rules:")
        .map(|relative| start + relative)
        .unwrap_or(prompt.len());
    Some(prompt[start..end].trim_end())
}

fn reconstruct_ungrouped_targets(actual: &str) -> Option<String> {
    let agent_id = actual
        .lines()
        .find_map(|line| line.trim().strip_prefix("- Agent-ID: "))?;
    let context = actual.split_once("Context: ")?.1;
    let (_, reviewed) = context.split_once("Reviewed commit: ")?;
    let mut output = String::new();
    for block in reviewed.split("\n\nReviewed commit: ") {
        let hash = block.lines().next()?.trim();
        let subject = labeled_value(block, "Subject: ")?;
        let feature = labeled_value(block, "Feature: ")?;
        let reason = labeled_value(block, "Reason: ")?;
        let context = block.split_once("Context: ")?.1.trim_end();
        output.push_str(&format!(
            "- Agent-ID: {agent_id}\n  Commit: {hash}\n  Feature: {feature}\n  Reason: {reason}\n  Subject: {subject}\n  Context: {context}\n"
        ));
    }
    (!output.is_empty()).then_some(output)
}

fn labeled_value<'a>(block: &'a str, label: &str) -> Option<&'a str> {
    block
        .lines()
        .find_map(|line| line.strip_prefix(label))
        .map(str::trim_end)
}

fn byte_difference(counterfactual: u64, actual: u64) -> i64 {
    let difference = i128::from(counterfactual) - i128::from(actual);
    difference.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessInventoryRecord {
    #[serde(flatten)]
    pub start: InvocationStart,
    pub end: Option<InvocationEnd>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThreadSummary {
    pub thread_id: String,
    pub invocation_id: String,
    pub primary: bool,
    pub visible: bool,
    pub agent_id: Option<String>,
    pub role: Option<String>,
    pub usage: CapturedUsage,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ModelStratumSummary {
    pub model: String,
    pub effort: String,
    pub thread_count: usize,
    pub primary_threads: usize,
    pub visible_threads: usize,
    pub descendant_threads: usize,
    pub usage: CapturedUsage,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ControllerUsageRow {
    pub agent_id: String,
    pub usage: CapturedUsage,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ControllerUsageReconciliationRow {
    pub agent_id: String,
    pub provider_thread_ids: Vec<String>,
    pub controller_streamed_usage: Option<CapturedUsage>,
    pub replayed_streamed_usage: Option<CapturedUsage>,
    pub provider_largest_cumulative_usage: Option<CapturedUsage>,
    pub controller_matches_replay: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AnalysisSummary {
    pub schema_version: u32,
    pub run_id: String,
    pub condition: String,
    pub capture_complete: bool,
    pub invocation_count: usize,
    pub complete_invocation_count: usize,
    pub passthrough_invocation_count: usize,
    #[serde(default)]
    pub interrupted_provider_turns: u64,
    pub threads: Vec<ThreadSummary>,
    #[serde(default)]
    pub session_only_threads: Vec<String>,
    pub model_strata: Vec<ModelStratumSummary>,
    pub usage_scopes: UsageScopes,
    pub controller_usage_reconciliation: Vec<ControllerUsageReconciliationRow>,
    pub mechanisms: MechanismSummary,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug)]
struct CapturedThreadMetadata {
    agent_id: Option<String>,
    role: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct TurnDeliveryReplay {
    thread_id: String,
    first_sequence: Option<usize>,
    assistant_text: String,
    usage: CapturedUsage,
    usage_events: u64,
    interrupted_after_directive: bool,
    directive_complete_sequence: Option<usize>,
}

#[derive(Clone, Debug)]
struct CumulativeUsageReplay {
    sequence: usize,
    total: CapturedUsage,
    last: Option<CapturedUsage>,
}

pub fn analyze(config: &CaptureConfig) -> ObserverResult<AnalysisSummary> {
    wait_for_active_invocations(config, Duration::from_secs(5))?;
    let inventory = read_process_inventory(config)?;
    let passthrough = read_codex_passthrough(config)?;
    write_json_lines(
        &config.root.join("process-invocations.jsonl"),
        inventory.iter(),
    )?;

    let mut observations = Vec::new();
    let mut thread_metadata = BTreeMap::<String, CapturedThreadMetadata>::new();
    let mut session_only_threads = BTreeSet::new();
    let mut replayed_controller_usage = BTreeMap::<String, CapturedUsage>::new();
    let mut interrupted_provider_turns = 0_u64;
    let mut mechanisms = MechanismAnalyzer::default();
    let mut errors = Vec::new();
    let h4_workspaces = inventory
        .iter()
        .filter(|process| {
            config.condition == "work-leaf"
                && process.start.primary
                && process.start.capture_kind == CaptureKind::AppServer
        })
        .map(|process| process.start.cwd.clone())
        .collect::<BTreeSet<_>>();

    verify_manifest_executables(config, &mut errors)?;
    for record in &passthrough {
        if !record.informational {
            errors.push(format!(
                "unclassified Codex passthrough invocation {} is not eligible for provider accounting",
                record.invocation_id
            ));
        }
        if record.real_executable != config.real_codex
            || record.real_executable_sha256 != config.real_codex_sha256
        {
            errors.push(format!(
                "Codex passthrough invocation {} executable identity differs from the immutable manifest",
                record.invocation_id
            ));
        }
    }
    for process in &inventory {
        verify_process_capture(config, process, &mut errors)?;
    }
    load_bundle_observations(config, &mut mechanisms, &mut errors)?;

    for process in &inventory {
        let invocation_id = &process.start.invocation_id;
        let capture_dir = config
            .root
            .join(process.start.capture_kind.artifact_directory())
            .join(invocation_id);
        match process.start.capture_kind {
            CaptureKind::AppServer => {
                let mut thread_accounting = AppServerThreadAccounting {
                    metadata: &mut thread_metadata,
                    session_only: &mut session_only_threads,
                    interrupted_provider_turns: &mut interrupted_provider_turns,
                };
                analyze_app_server(
                    &process.start,
                    &capture_dir,
                    &mut observations,
                    &mut thread_accounting,
                    &mut replayed_controller_usage,
                    &mut mechanisms,
                    &mut errors,
                )?;
            }
            CaptureKind::ExecJson => {
                analyze_exec(
                    &process.start,
                    &capture_dir,
                    &mut observations,
                    &mut thread_metadata,
                    &mut mechanisms,
                    &mut errors,
                )?;
            }
            CaptureKind::LockedCommand => {
                analyze_locked_command(
                    &process.start,
                    process.end.as_ref(),
                    &capture_dir,
                    process.start.parent_invocation_id.is_none()
                        && h4_workspaces.contains(&process.start.cwd),
                    &mut mechanisms,
                )?;
            }
        }
    }

    let rollout_audit_path = config.root.join("rollout-audit.json");
    let rollout_audit = if rollout_audit_path.is_file() {
        Some(serde_json::from_slice::<RolloutAudit>(&fs::read(
            &rollout_audit_path,
        )?)?)
    } else {
        None
    };
    if rollout_audit
        .as_ref()
        .is_some_and(|audit| audit.errors.is_empty())
    {
        supplement_usage_from_rollouts(
            config,
            &inventory,
            &mut observations,
            &mut thread_metadata,
            &mut errors,
        )?;
    }

    load_timeline_observations(config, &mut mechanisms, &mut errors)?;
    load_git_checkpoint_observations(config, &mut mechanisms, &mut errors)?;
    if !observations.is_empty() {
        mechanisms.observe(EvidenceInput::ThreadTopology);
        mechanisms.observe(EvidenceInput::Usage);
        mechanisms.observe(EvidenceInput::GenerationUsage);
        mechanisms.observe(EvidenceInput::AccountingReconciliation);
    }

    validate_usage_accounting(&observations, &mut errors);
    let usage_scopes = summarize_usage(&observations);
    let final_threads = final_thread_observations(&observations);
    let roles_by_invocation = inventory
        .iter()
        .filter_map(|record| {
            record
                .start
                .role
                .clone()
                .map(|role| (record.start.invocation_id.clone(), role))
        })
        .collect::<BTreeMap<_, _>>();
    let threads = final_threads
        .iter()
        .map(|record| {
            let metadata = thread_metadata.get(&record.thread_id);
            ThreadSummary {
                thread_id: record.thread_id.clone(),
                invocation_id: record.invocation_id.clone(),
                primary: record.primary,
                visible: record.visible,
                agent_id: metadata.and_then(|entry| entry.agent_id.clone()),
                role: metadata
                    .and_then(|entry| entry.role.clone())
                    .or_else(|| roles_by_invocation.get(&record.invocation_id).cloned()),
                usage: record.usage,
            }
        })
        .collect::<Vec<_>>();
    for thread in &threads {
        session_only_threads.remove(&thread.thread_id);
    }
    for thread in &threads {
        if let Some(role) = thread.agent_id.as_deref().or(thread.role.as_deref()) {
            mechanisms.observe_thread_role(&thread.thread_id, role);
        }
    }
    let controller_usage_reconciliation =
        reconcile_controller_usage(config, &threads, &replayed_controller_usage, &mut errors)?;
    let complete_invocation_count = inventory
        .iter()
        .filter(|record| record.end.is_some())
        .count();
    for record in &inventory {
        if record.end.is_none() {
            errors.push(format!(
                "invocation {} has no end metadata",
                record.start.invocation_id
            ));
        }
    }
    if let Some(rollout_audit) = rollout_audit {
        errors.extend(
            rollout_audit
                .errors
                .into_iter()
                .map(|error| format!("rollout: {error}")),
        );
    }
    if config.require_complete_provider_usage && interrupted_provider_turns > 0 {
        errors.push(format!(
            "interrupted provider turn has no complete usage: count={interrupted_provider_turns}"
        ));
    }
    let model_strata = summarize_model_strata(config, &mut errors)?;
    errors.extend(scan_for_secret_markers(config)?);
    let mechanism_summary = mechanisms.finish();
    let capture_complete = errors.is_empty();
    write_json_lines(
        &config.root.join("counterfactuals.jsonl"),
        mechanism_summary.counterfactuals.iter(),
    )?;
    let summary = AnalysisSummary {
        schema_version: 2,
        run_id: config.run_id.clone(),
        condition: config.condition.clone(),
        capture_complete,
        invocation_count: inventory.len(),
        complete_invocation_count,
        passthrough_invocation_count: passthrough.len(),
        interrupted_provider_turns,
        threads,
        session_only_threads: session_only_threads.into_iter().collect(),
        model_strata,
        usage_scopes,
        controller_usage_reconciliation,
        mechanisms: mechanism_summary,
        errors,
    };
    write_json_atomic(&config.root.join("mechanism-summary.json"), &summary)?;
    write_capture_audit(config, &summary)?;
    harden_artifact_permissions(&config.root)?;
    Ok(summary)
}

pub fn record_controller_usage(config: &CaptureConfig, state: &Path) -> ObserverResult<usize> {
    let value: Value = serde_json::from_slice(&fs::read(state)?)?;
    let sessions = value
        .get("snapshot")
        .and_then(|snapshot| snapshot.get("sessions"))
        .and_then(Value::as_array)
        .ok_or_else(|| ObserverError::new("controller state has no snapshot.sessions array"))?;
    let mut rows = Vec::new();
    let mut agent_ids = BTreeSet::new();
    for session in sessions {
        let Some(token_usage) = session.get("token_usage").filter(|usage| !usage.is_null()) else {
            continue;
        };
        let agent_id = session
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| ObserverError::new("controller session with usage has no id"))?;
        if !agent_ids.insert(agent_id.to_string()) {
            return Err(ObserverError::new(format!(
                "controller state contains duplicate usage for {agent_id}"
            )));
        }
        let usage = usage_from_object(token_usage).ok_or_else(|| {
            ObserverError::new(format!(
                "controller session {agent_id} has invalid token usage"
            ))
        })?;
        rows.push(ControllerUsageRow {
            agent_id: agent_id.to_string(),
            usage,
        });
    }
    rows.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    write_json_atomic(&config.root.join("controller-usage.json"), &rows)?;
    harden_artifact_permissions(&config.root)?;
    Ok(rows.len())
}

fn reconcile_controller_usage(
    config: &CaptureConfig,
    threads: &[ThreadSummary],
    replayed: &BTreeMap<String, CapturedUsage>,
    errors: &mut Vec<String>,
) -> ObserverResult<Vec<ControllerUsageReconciliationRow>> {
    let path = config.root.join("controller-usage.json");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let rows: Vec<ControllerUsageRow> = serde_json::from_slice(&fs::read(path)?)?;
    let controller = rows
        .iter()
        .map(|row| (row.agent_id.as_str(), row.usage))
        .collect::<BTreeMap<_, _>>();
    let mut provider = BTreeMap::<String, CapturedUsage>::new();
    let mut provider_threads = BTreeMap::<String, Vec<String>>::new();
    for thread in threads.iter().filter(|thread| thread.visible) {
        let Some(agent_id) = thread.agent_id.as_deref() else {
            errors.push(format!(
                "visible provider thread {} has no Work Leaf agent ID",
                thread.thread_id
            ));
            continue;
        };
        provider
            .entry(agent_id.to_string())
            .and_modify(|usage| *usage = usage.combine(thread.usage))
            .or_insert(thread.usage);
        provider_threads
            .entry(agent_id.to_string())
            .or_default()
            .push(thread.thread_id.clone());
    }
    let agent_ids = controller
        .keys()
        .copied()
        .map(str::to_string)
        .chain(provider.keys().cloned())
        .chain(replayed.keys().cloned())
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for agent_id in agent_ids {
        let controller_usage = controller.get(agent_id.as_str()).copied();
        let replayed_usage = replayed.get(&agent_id).copied();
        let provider_usage = provider.get(&agent_id).copied();
        if controller_usage.is_none() && provider_usage.is_some() {
            errors.push(format!(
                "visible provider agent {agent_id} has no controller usage row"
            ));
        }
        if controller_usage.is_some() && provider_usage.is_none() {
            errors.push(format!(
                "controller usage row {agent_id} has no visible provider thread"
            ));
        }
        if replayed_usage.is_none() && controller_usage.is_some() {
            errors.push(format!(
                "controller usage row {agent_id} has no replayable streamed usage"
            ));
        }
        if replayed_usage.is_some() && provider_usage.is_none() {
            errors.push(format!(
                "replayed streamed usage for {agent_id} has no visible provider thread"
            ));
        }
        let controller_matches_replay = controller_usage.is_some()
            && controller_usage == replayed_usage
            && provider_usage.is_some();
        if controller_usage.is_some()
            && replayed_usage.is_some()
            && controller_usage != replayed_usage
        {
            errors.push(format!(
                "controller usage for {agent_id} does not match replayed pre-interrupt usage"
            ));
        }
        rows.push(ControllerUsageReconciliationRow {
            agent_id: agent_id.clone(),
            provider_thread_ids: provider_threads.remove(&agent_id).unwrap_or_default(),
            controller_streamed_usage: controller_usage,
            replayed_streamed_usage: replayed_usage,
            provider_largest_cumulative_usage: provider_usage,
            controller_matches_replay,
        });
    }
    write_json_atomic(
        &config.root.join("controller-usage-reconciliation.json"),
        &rows,
    )?;
    Ok(rows)
}

fn verify_process_capture(
    config: &CaptureConfig,
    process: &ProcessInventoryRecord,
    errors: &mut Vec<String>,
) -> ObserverResult<()> {
    let (expected_path, expected_sha) = match process.start.capture_kind {
        CaptureKind::LockedCommand => (&config.real_sh, &config.real_sh_sha256),
        CaptureKind::AppServer | CaptureKind::ExecJson => {
            (&config.real_codex, &config.real_codex_sha256)
        }
    };
    if &process.start.real_executable != expected_path {
        errors.push(format!(
            "invocation {} executable path differs from the immutable manifest",
            process.start.invocation_id
        ));
    }
    if &process.start.real_executable_sha256 != expected_sha {
        errors.push(format!(
            "invocation {} executable SHA-256 differs from the immutable manifest",
            process.start.invocation_id
        ));
    }
    let directory = config
        .root
        .join(process.start.capture_kind.artifact_directory())
        .join(&process.start.invocation_id);
    let (stdin_name, stdout_name, stderr_name) = match process.start.capture_kind {
        CaptureKind::AppServer => (
            "client-to-server.raw",
            "server-to-client.raw",
            "server-stderr.raw",
        ),
        CaptureKind::ExecJson | CaptureKind::LockedCommand => {
            ("stdin.raw", "stdout.raw", "stderr.raw")
        }
    };
    let Some(end) = process.end.as_ref() else {
        return Ok(());
    };
    if end.invocation_id != process.start.invocation_id {
        errors.push(format!(
            "invocation {} start/end IDs do not match",
            process.start.invocation_id
        ));
    }
    for (stream, name, expected) in [
        ("stdin", stdin_name, &end.stdin_sha256),
        ("stdout", stdout_name, &end.stdout_sha256),
        ("stderr", stderr_name, &end.stderr_sha256),
    ] {
        let path = directory.join(name);
        if !path.is_file() {
            errors.push(format!(
                "invocation {} is missing captured {stream} bytes",
                process.start.invocation_id
            ));
            continue;
        }
        if &sha256_file(&path)? != expected {
            errors.push(format!(
                "invocation {} captured {stream} SHA-256 differs from end metadata",
                process.start.invocation_id
            ));
        }
        verify_chunk_inventory(
            &path,
            &directory.join(format!("{stream}-chunks.jsonl")),
            &process.start.invocation_id,
            stream,
            errors,
        )?;
    }
    Ok(())
}

fn verify_manifest_executables(
    config: &CaptureConfig,
    errors: &mut Vec<String>,
) -> ObserverResult<()> {
    let mut executables = vec![
        ("real Codex", &config.real_codex, &config.real_codex_sha256),
        ("real shell", &config.real_sh, &config.real_sh_sha256),
        (
            "observer",
            &config.observer_executable,
            &config.observer_sha256,
        ),
    ];
    match (&config.real_cargo, &config.real_cargo_sha256) {
        (Some(path), Some(sha256)) => executables.push(("real Cargo", path, sha256)),
        (None, None) => {}
        _ => errors.push("real Cargo executable identity is incomplete".to_string()),
    }
    for (label, path, expected) in executables {
        match sha256_file(path) {
            Ok(actual) if &actual == expected => {}
            Ok(_) => errors.push(format!(
                "{label} executable SHA-256 changed after observer initialization"
            )),
            Err(error) => errors.push(format!(
                "{label} executable cannot be verified at {}: {error}",
                path.display()
            )),
        }
    }
    Ok(())
}

fn verify_chunk_inventory(
    raw_path: &Path,
    chunk_path: &Path,
    invocation_id: &str,
    stream: &str,
    errors: &mut Vec<String>,
) -> ObserverResult<()> {
    let raw = fs::read(raw_path)?;
    let chunks = read_stream_chunks(chunk_path)?;
    if raw.is_empty() && chunks.is_empty() {
        return Ok(());
    }
    if !raw.is_empty() && chunks.is_empty() {
        errors.push(format!(
            "invocation {invocation_id} captured {stream} bytes without chunk timing metadata"
        ));
        return Ok(());
    }
    let mut offset = 0_usize;
    for chunk in chunks {
        if chunk.offset != offset as u64 {
            errors.push(format!(
                "invocation {invocation_id} {stream} chunk offsets are not contiguous"
            ));
            return Ok(());
        }
        let end = offset.saturating_add(chunk.length);
        let Some(bytes) = raw.get(offset..end) else {
            errors.push(format!(
                "invocation {invocation_id} {stream} chunk exceeds captured bytes"
            ));
            return Ok(());
        };
        if sha256_bytes(bytes) != chunk.sha256 {
            errors.push(format!(
                "invocation {invocation_id} {stream} chunk SHA-256 mismatch"
            ));
        }
        offset = end;
    }
    if offset != raw.len() {
        errors.push(format!(
            "invocation {invocation_id} {stream} chunks do not cover all captured bytes"
        ));
    }
    Ok(())
}

fn summarize_model_strata(
    config: &CaptureConfig,
    errors: &mut Vec<String>,
) -> ObserverResult<Vec<ModelStratumSummary>> {
    let path = config.root.join("rollout-metadata.jsonl");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let mut strata = BTreeMap::<(String, String), ModelStratumSummary>::new();
    let mut thread_ids = BTreeSet::new();
    for (line_number, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row: RolloutMetadata = serde_json::from_str(&line).map_err(|error| {
            ObserverError::new(format!(
                "invalid rollout metadata line {}: {error}",
                line_number + 1
            ))
        })?;
        if !thread_ids.insert(row.thread_id.clone()) {
            errors.push(format!(
                "rollout metadata contains duplicate thread {}",
                row.thread_id
            ));
            continue;
        }
        if row.model.is_empty() || row.effort.is_empty() {
            errors.push(format!(
                "rollout metadata thread {} has an empty model or effort stratum",
                row.thread_id
            ));
        }
        let stratum = strata
            .entry((row.model.clone(), row.effort.clone()))
            .or_insert_with(|| ModelStratumSummary {
                model: row.model.clone(),
                effort: row.effort.clone(),
                ..ModelStratumSummary::default()
            });
        stratum.thread_count += 1;
        stratum.primary_threads += usize::from(row.primary);
        stratum.visible_threads += usize::from(row.visible);
        stratum.descendant_threads += usize::from(row.descendant);
        stratum.usage = stratum.usage.combine(row.usage);
    }
    Ok(strata.into_values().collect())
}

fn load_bundle_observations(
    config: &CaptureConfig,
    mechanisms: &mut MechanismAnalyzer,
    errors: &mut Vec<String>,
) -> ObserverResult<()> {
    let manifest = config.root.join("context-bundles/manifest.jsonl");
    if !manifest.is_file() {
        return Ok(());
    }
    for (line_number, line) in BufReader::new(File::open(&manifest)?).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            ObserverError::new(format!(
                "invalid context-bundle manifest line {}: {error}",
                line_number + 1
            ))
        })?;
        if let Some(parse_error) = value.get("parse_error").and_then(Value::as_str) {
            errors.push(format!(
                "context-bundle manifest line {} could not parse production bundle metadata: {parse_error}",
                line_number + 1
            ));
        }
        let Some(source) = value.get("source").and_then(Value::as_str) else {
            errors.push(format!(
                "context-bundle manifest line {} has no source path",
                line_number + 1
            ));
            continue;
        };
        let Some(archived_path) = value.get("archived_path").and_then(Value::as_str) else {
            errors.push(format!(
                "context-bundle manifest line {} has no archived path",
                line_number + 1
            ));
            continue;
        };
        let path = config.root.join("context-bundles").join(archived_path);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                errors.push(format!(
                    "archived context bundle {} cannot be read: {error}",
                    path.display()
                ));
                continue;
            }
        };
        let sha256 = sha256_bytes(&bytes);
        if value.get("sha256").and_then(Value::as_str) != Some(sha256.as_str()) {
            errors.push(format!(
                "archived context bundle {} SHA-256 does not match its manifest",
                path.display()
            ));
        }
        if value.get("bytes").and_then(Value::as_u64) != Some(bytes.len() as u64) {
            errors.push(format!(
                "archived context bundle {} byte length does not match its manifest",
                path.display()
            ));
        }
        match parse_context_bundle_snapshots(&bytes) {
            Ok(snapshots) => {
                let metadata = snapshots
                    .iter()
                    .map(ContextBundleSnapshot::manifest_value)
                    .collect::<Vec<_>>();
                if value.get("file_snapshots") != Some(&Value::Array(metadata)) {
                    errors.push(format!(
                        "archived context bundle {} snapshot metadata does not match its manifest",
                        path.display()
                    ));
                }
            }
            Err(error) => errors.push(format!(
                "archived context bundle {} cannot reconstruct snapshots: {error}",
                path.display()
            )),
        }
        mechanisms.observe_bundle_archive(BundleArchiveObservation {
            source_path: PathBuf::from(source),
            archived_path: PathBuf::from(archived_path),
            bytes,
            sha256,
        });
    }
    Ok(())
}

fn load_git_checkpoint_observations(
    config: &CaptureConfig,
    mechanisms: &mut MechanismAnalyzer,
    errors: &mut Vec<String>,
) -> ObserverResult<()> {
    let manifest = config.root.join("git-checkpoints/manifest.jsonl");
    if !manifest.is_file() {
        return Ok(());
    }
    let mut commits = BTreeSet::new();
    for (line_number, line) in BufReader::new(File::open(&manifest)?).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            ObserverError::new(format!(
                "invalid Git checkpoint manifest line {}: {error}",
                line_number + 1
            ))
        })?;
        let Some(safe_label) = value.get("safe_label").and_then(Value::as_str) else {
            errors.push(format!(
                "Git checkpoint manifest line {} has no safe label",
                line_number + 1
            ));
            continue;
        };
        if safe_component(safe_label) != safe_label {
            errors.push(format!(
                "Git checkpoint manifest line {} has an unsafe label",
                line_number + 1
            ));
            continue;
        }
        let graph = config
            .root
            .join("git-checkpoints/files")
            .join(safe_label)
            .join("commit-graph.txt");
        let bytes = match fs::read(&graph) {
            Ok(bytes) => bytes,
            Err(error) => {
                errors.push(format!(
                    "Git checkpoint {} has no readable commit graph: {error}",
                    safe_label
                ));
                continue;
            }
        };
        let expected = value
            .pointer("/files/commit-graph.txt")
            .and_then(Value::as_str);
        if expected != Some(sha256_bytes(&bytes).as_str()) {
            errors.push(format!(
                "Git checkpoint {} commit graph SHA-256 mismatch",
                safe_label
            ));
            continue;
        }
        for line in String::from_utf8_lossy(&bytes).lines() {
            if let Some(hash) = line
                .split_whitespace()
                .find(|word| word.len() == 40 && word.chars().all(|ch| ch.is_ascii_hexdigit()))
            {
                commits.insert(hash.to_string());
            }
        }
    }
    mechanisms.observe_git_commit_hashes(commits);
    Ok(())
}

fn load_timeline_observations(
    config: &CaptureConfig,
    mechanisms: &mut MechanismAnalyzer,
    errors: &mut Vec<String>,
) -> ObserverResult<()> {
    let path = config.root.join("timeline.jsonl");
    if !path.is_file() {
        return Ok(());
    }
    for (line_number, line) in BufReader::new(File::open(&path)?).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            ObserverError::new(format!(
                "invalid timeline line {}: {error}",
                line_number + 1
            ))
        })?;
        let Some(event) = value.get("event").and_then(Value::as_str) else {
            errors.push(format!("timeline line {} has no event", line_number + 1));
            continue;
        };
        let monotonic = value
            .get("monotonic_ns")
            .and_then(Value::as_str)
            .and_then(|value| value.parse::<u128>().ok());
        let Some(monotonic_ns) = monotonic else {
            errors.push(format!(
                "timeline line {} has no valid monotonic timestamp",
                line_number + 1
            ));
            continue;
        };
        mechanisms.observe_timeline(TimelineObservation {
            event: event.to_string(),
            detail: value
                .get("detail")
                .and_then(Value::as_str)
                .map(str::to_string),
            monotonic_ns,
        });
    }
    Ok(())
}

fn read_process_inventory(config: &CaptureConfig) -> ObserverResult<Vec<ProcessInventoryRecord>> {
    let root = config.root.join("invocations");
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut directories = fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .collect::<Vec<_>>();
    directories.sort_by_key(|entry| entry.file_name());
    let mut records = Vec::new();
    for directory in directories {
        let start_path = directory.path().join("start.json");
        if !start_path.is_file() {
            continue;
        }
        let start = serde_json::from_slice(&fs::read(start_path)?)?;
        let end_path = directory.path().join("end.json");
        let end = if end_path.is_file() {
            Some(serde_json::from_slice(&fs::read(end_path)?)?)
        } else {
            None
        };
        records.push(ProcessInventoryRecord { start, end });
    }
    Ok(records)
}

fn wait_for_active_invocations(config: &CaptureConfig, timeout: Duration) -> ObserverResult<()> {
    let deadline = Instant::now() + timeout;
    loop {
        let active = read_process_inventory(config)?
            .into_iter()
            .any(|record| record.end.is_none() && process_is_active(&record.start));
        if !active || Instant::now() >= deadline {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
}

pub fn stop_active_primary_app_server(
    config: &CaptureConfig,
    graceful_timeout: Duration,
) -> ObserverResult<usize> {
    let active = read_process_inventory(config)?
        .into_iter()
        .filter(|record| {
            record.start.primary
                && record.start.capture_kind == CaptureKind::AppServer
                && record.end.is_none()
        })
        .collect::<Vec<_>>();
    if active.len() != 1 {
        return Err(ObserverError::new(format!(
            "expected exactly one active primary app-server invocation, found {}",
            active.len()
        )));
    }
    let record = &active[0];
    let invocation_dir = config
        .root
        .join("invocations")
        .join(&record.start.invocation_id);
    let child_path = invocation_dir.join("child.json");
    let child: InvocationChild =
        serde_json::from_slice(&fs::read(&child_path).map_err(|error| {
            ObserverError::new(format!(
                "cannot read app-server child metadata {}: {error}",
                child_path.display()
            ))
        })?)?;
    if child.invocation_id != record.start.invocation_id {
        return Err(ObserverError::new(format!(
            "app-server child metadata belongs to {} instead of {}",
            child.invocation_id, record.start.invocation_id
        )));
    }

    signal_captured_child(child.pid, libc::SIGTERM)?;
    let end_path = invocation_dir.join("end.json");
    if !wait_for_file(&end_path, graceful_timeout) {
        signal_captured_child(child.pid, libc::SIGKILL)?;
        if !wait_for_file(&end_path, Duration::from_secs(5)) {
            return Err(ObserverError::new(format!(
                "app-server proxy {} did not persist end metadata after child termination",
                record.start.invocation_id
            )));
        }
    }
    Ok(1)
}

fn wait_for_file(path: &Path, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if path.is_file() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(unix)]
fn signal_captured_child(pid: u32, signal: i32) -> ObserverResult<()> {
    let pid = i32::try_from(pid)
        .map_err(|_| ObserverError::new(format!("captured child PID {pid} is out of range")))?;
    let result = unsafe { libc::kill(pid, signal) };
    if result == 0 || io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(ObserverError::new(format!(
            "cannot signal captured child {pid} with signal {signal}: {}",
            io::Error::last_os_error()
        )))
    }
}

#[cfg(not(unix))]
fn signal_captured_child(pid: u32, signal: i32) -> ObserverResult<()> {
    Err(ObserverError::new(format!(
        "captured child signaling is unavailable for PID {pid} and signal {signal}"
    )))
}

fn process_is_active(start: &InvocationStart) -> bool {
    let Ok(pid) = i32::try_from(start.pid) else {
        return false;
    };
    let result = unsafe { libc::kill(pid, 0) };
    if result != 0 && io::Error::last_os_error().raw_os_error() != Some(libc::EPERM) {
        return false;
    }
    let Ok(command_line) = fs::read(format!("/proc/{pid}/cmdline")) else {
        return true;
    };
    let actual_argv0 = command_line
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default();
    start
        .argv
        .first()
        .is_some_and(|expected| actual_argv0 == expected.display.as_bytes())
}

fn read_codex_passthrough(config: &CaptureConfig) -> ObserverResult<Vec<CodexPassthroughRecord>> {
    let path = config.root.join("codex-passthrough.jsonl");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for (line_number, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(serde_json::from_str(&line).map_err(|error| {
            ObserverError::new(format!(
                "invalid Codex passthrough record line {}: {error}",
                line_number + 1
            ))
        })?);
    }
    Ok(records)
}

struct AppServerThreadAccounting<'a> {
    metadata: &'a mut BTreeMap<String, CapturedThreadMetadata>,
    session_only: &'a mut BTreeSet<String>,
    interrupted_provider_turns: &'a mut u64,
}

fn analyze_app_server(
    start: &InvocationStart,
    directory: &Path,
    observations: &mut Vec<UsageObservation>,
    thread_accounting: &mut AppServerThreadAccounting<'_>,
    replayed_controller_usage: &mut BTreeMap<String, CapturedUsage>,
    mechanisms: &mut MechanismAnalyzer,
    errors: &mut Vec<String>,
) -> ObserverResult<()> {
    let client_bytes = read_or_empty(&directory.join("client-to-server.raw"))?;
    let server_bytes = read_or_empty(&directory.join("server-to-client.raw"))?;
    let client_chunks = read_stream_chunks(&directory.join("stdin-chunks.jsonl"))?;
    let server_chunks = read_stream_chunks(&directory.join("stdout-chunks.jsonl"))?;
    let client_frames = index_jsonl_with_chunks(&client_bytes, "client-to-server", &client_chunks);
    let client_frame_times = client_frames
        .iter()
        .filter_map(|frame| Some((frame.rpc_id.clone()?, frame.received_monotonic_ns?)))
        .collect::<BTreeMap<_, _>>();
    let mut server_frames =
        index_jsonl_with_chunks(&server_bytes, "server-to-client", &server_chunks);
    let server_sequence_offset = client_frames.len();
    for frame in &mut server_frames {
        frame.sequence += server_sequence_offset;
    }
    let mut frames = client_frames;
    frames.extend(server_frames);
    write_json_lines(&directory.join("frames.jsonl"), frames.iter())?;

    let client_values = parse_top_level_json_lines(&client_bytes, errors, start, "client")?;
    let server_values = parse_top_level_json_lines(&server_bytes, errors, start, "server")?;
    let mut agents = BTreeMap::<String, String>::new();
    let mut pending_turn_threads = BTreeMap::<String, String>::new();
    for value in &client_values {
        if value.get("method").and_then(Value::as_str) == Some("turn/start")
            && let (Some(request_id), Some(thread_id)) =
                (json_rpc_id(value), extract_thread_id(value))
        {
            pending_turn_threads.insert(request_id, thread_id);
        }
    }
    let generation_threads = pending_turn_threads
        .values()
        .cloned()
        .collect::<BTreeSet<_>>();
    thread_accounting.session_only.extend(
        client_values
            .iter()
            .chain(&server_values)
            .filter_map(extract_thread_id)
            .filter(|thread_id| !generation_threads.contains(thread_id)),
    );

    let mut request_turns = BTreeMap::<String, String>::new();
    let mut turn_threads = BTreeMap::<String, String>::new();
    for value in &server_values {
        if let (Some(request_id), Some(turn_id)) = (json_rpc_id(value), extract_turn_id(value))
            && let Some(thread_id) = pending_turn_threads.get(&request_id)
        {
            request_turns.insert(request_id, turn_id.clone());
            turn_threads.insert(turn_id, thread_id.clone());
        }
        if let (Some(turn_id), Some(thread_id)) = (extract_turn_id(value), extract_thread_id(value))
        {
            turn_threads.insert(turn_id, thread_id);
        }
    }

    let mut requested_interrupts = BTreeSet::<(String, String)>::new();
    for value in &client_values {
        if let Some(prompt) = extract_turn_prompt(value)
            && let Some(thread_id) = extract_thread_id(value)
        {
            let turn_id = json_rpc_id(value)
                .and_then(|request_id| request_turns.get(&request_id).cloned())
                .unwrap_or_else(|| format!("request-{}", json_rpc_id(value).unwrap_or_default()));
            let received = json_rpc_id(value)
                .and_then(|request_id| client_frame_times.get(&request_id).copied());
            mechanisms.observe_prompt_at(&thread_id, &turn_id, prompt, received);
            if let Some(agent_id) = extract_agent_id(prompt) {
                agents.insert(thread_id, agent_id);
            }
        }
        if value.get("method").and_then(Value::as_str) == Some("turn/interrupt") {
            if let (Some(thread_id), Some(turn_id)) =
                (extract_thread_id(value), extract_turn_id(value))
            {
                requested_interrupts.insert((thread_id.clone(), turn_id.clone()));
                mechanisms.observe_interrupt(&thread_id, &turn_id);
            } else {
                errors.push(format!(
                    "turn/interrupt in invocation {} has no thread/turn identity",
                    start.invocation_id
                ));
            }
        }
    }
    let mut delivery_replays = BTreeMap::<String, TurnDeliveryReplay>::new();
    let mut interrupted_turns = BTreeSet::<(String, String)>::new();
    let mut turns_with_terminal_usage = BTreeSet::<(String, String)>::new();
    let mut cumulative_usage_replays = BTreeMap::<String, Vec<CumulativeUsageReplay>>::new();
    for (sequence, value) in server_values.iter().enumerate() {
        let thread_id = extract_thread_id(value).or_else(|| {
            extract_turn_id(value).and_then(|turn_id| turn_threads.get(&turn_id).cloned())
        });
        let turn_id = extract_turn_id(value);
        if let (Some(thread_id), Some(turn_id)) = (thread_id.clone(), turn_id.clone()) {
            let replay = delivery_replays
                .entry(turn_id)
                .or_insert_with(|| TurnDeliveryReplay {
                    thread_id,
                    first_sequence: Some(sequence),
                    ..TurnDeliveryReplay::default()
                });
            if !replay.interrupted_after_directive {
                if let Some(usage) = extract_last_usage(value) {
                    replay.usage = replay.usage.combine(usage);
                    replay.usage_events = replay.usage_events.saturating_add(1);
                }
                if let Some(message) = extract_assistant_message(value) {
                    if !replay.assistant_text.is_empty() {
                        replay.assistant_text.push_str("\n\n");
                    }
                    replay.assistant_text.push_str(message);
                    if assistant_text_completes_work_leaf_directive(&replay.assistant_text) {
                        replay.interrupted_after_directive = true;
                        replay.directive_complete_sequence.get_or_insert(sequence);
                    }
                }
            }
        }
        if let Some((command, output)) = extract_command_item_with_output(value)
            && let (Some(thread_id), Some(turn_id)) = (thread_id.clone(), turn_id.clone())
        {
            mechanisms.observe_command(CommandObservation {
                thread_id,
                turn_id,
                command,
                output,
                duration_ns: None,
            });
        }
        if let Some(message) = extract_assistant_message(value)
            && let (Some(thread_id), Some(turn_id)) = (thread_id.clone(), turn_id.clone())
        {
            mechanisms.observe_assistant(&thread_id, &turn_id, message);
        }
        if value.get("method").and_then(Value::as_str) == Some("turn/completed")
            && let (Some(thread_id), Some(turn_id)) = (thread_id.clone(), turn_id.clone())
        {
            let outcome = extract_turn_outcome(value);
            if outcome == TurnOutcome::Interrupted {
                interrupted_turns.insert((thread_id.clone(), turn_id.clone()));
            }
            mechanisms.observe_turn_outcome(&thread_id, &turn_id, outcome);
        }
        let Some((usage_kind, usage)) = extract_usage(value) else {
            continue;
        };
        if usage_kind != "thread-total" {
            errors.push(format!(
                "usage event in invocation {} is not a cumulative thread total",
                start.invocation_id
            ));
            continue;
        }
        let Some(thread_id) = thread_id else {
            errors.push(format!(
                "usage event in invocation {} has no thread id",
                start.invocation_id
            ));
            continue;
        };
        let last = extract_last_usage(value);
        let previous = cumulative_usage_replays
            .get(&thread_id)
            .and_then(|events| events.last())
            .map(|event| event.total);
        let same_turn_usage_is_fresh =
            turn_id.is_some() && cumulative_usage_contains_last_response(previous, usage, last);
        cumulative_usage_replays
            .entry(thread_id.clone())
            .or_default()
            .push(CumulativeUsageReplay {
                sequence,
                total: usage,
                last,
            });
        if let Some(turn_id) = turn_id
            && delivery_replays
                .get(&turn_id)
                .is_some_and(|replay| replay.interrupted_after_directive)
            && same_turn_usage_is_fresh
        {
            turns_with_terminal_usage.insert((thread_id.clone(), turn_id));
        }
        if !generation_threads.contains(&thread_id) {
            continue;
        }
        let agent_id = agents.get(&thread_id).cloned();
        let visible = start.primary && agent_id.as_deref().is_some_and(is_visible_work_leaf_agent);
        observations.push(UsageObservation::new(
            &thread_id,
            &start.invocation_id,
            start.primary,
            visible,
            usage,
        ));
        thread_accounting.metadata.insert(
            thread_id,
            CapturedThreadMetadata {
                agent_id,
                role: start.role.clone(),
            },
        );
    }
    if start.primary {
        for replay in delivery_replays
            .values()
            .filter(|replay| replay.usage_events > 0)
        {
            let Some(agent_id) = agents.get(&replay.thread_id) else {
                continue;
            };
            if !is_visible_work_leaf_agent(agent_id) {
                continue;
            }
            replayed_controller_usage
                .entry(agent_id.clone())
                .and_modify(|usage| *usage = usage.combine(replay.usage))
                .or_insert(replay.usage);
        }
    }
    interrupted_turns.extend(requested_interrupts);
    let interrupted_without_terminal_usage = interrupted_turns
        .difference(&turns_with_terminal_usage)
        .cloned()
        .collect::<BTreeSet<_>>();
    let recovered_from_later_cumulative_usage = interrupted_without_terminal_usage
        .iter()
        .filter(|turn| {
            later_cumulative_usage_proves_interrupted_turn(
                turn,
                &interrupted_without_terminal_usage,
                &delivery_replays,
                &cumulative_usage_replays,
            )
        })
        .count();
    let interrupted_without_usage = interrupted_without_terminal_usage
        .len()
        .saturating_sub(recovered_from_later_cumulative_usage);
    *thread_accounting.interrupted_provider_turns = thread_accounting
        .interrupted_provider_turns
        .saturating_add(u64::try_from(interrupted_without_usage).unwrap_or(u64::MAX));
    Ok(())
}

fn later_cumulative_usage_proves_interrupted_turn(
    turn: &(String, String),
    unresolved_turns: &BTreeSet<(String, String)>,
    delivery_replays: &BTreeMap<String, TurnDeliveryReplay>,
    cumulative_usage_replays: &BTreeMap<String, Vec<CumulativeUsageReplay>>,
) -> bool {
    let (thread_id, turn_id) = turn;
    let Some(replay) = delivery_replays.get(turn_id) else {
        return false;
    };
    let Some(directive_sequence) = replay.directive_complete_sequence else {
        return false;
    };
    let Some(events) = cumulative_usage_replays.get(thread_id) else {
        return false;
    };
    let previous = events
        .iter()
        .rev()
        .find(|event| event.sequence < directive_sequence);
    let previous_total = previous.map_or_else(CapturedUsage::default, |event| event.total);
    let Some(next) = events.iter().find(|event| {
        event.sequence > directive_sequence
            && event
                .total
                .checked_difference(previous_total)
                .is_some_and(|increase| increase != CapturedUsage::default())
    }) else {
        return false;
    };

    if unresolved_turns
        .iter()
        .any(|(candidate_thread, candidate_turn)| {
            candidate_thread == thread_id
                && delivery_replays
                    .get(candidate_turn)
                    .and_then(|candidate| candidate.directive_complete_sequence)
                    .is_none()
        })
    {
        return false;
    }

    let unresolved_in_interval = unresolved_turns
        .iter()
        .filter(|(candidate_thread, candidate_turn)| {
            if candidate_thread != thread_id {
                return false;
            }
            delivery_replays
                .get(candidate_turn)
                .and_then(|candidate| candidate.directive_complete_sequence)
                .is_some_and(|sequence| {
                    previous.is_none_or(|event| sequence > event.sequence)
                        && sequence < next.sequence
                })
        })
        .count();
    if unresolved_in_interval != 1 {
        return false;
    }

    if previous.is_none()
        && delivery_replays
            .iter()
            .any(|(candidate_turn_id, candidate)| {
                candidate_turn_id != turn_id
                    && candidate.thread_id == *thread_id
                    && candidate
                        .first_sequence
                        .is_some_and(|sequence| sequence < directive_sequence)
            })
    {
        return false;
    }

    let Some(increase) = next.total.checked_difference(previous_total) else {
        return false;
    };
    let Some(last) = next.last else {
        return false;
    };
    let Some(unreported) = increase.checked_difference(last) else {
        return false;
    };
    unreported != CapturedUsage::default()
}

fn analyze_exec(
    start: &InvocationStart,
    directory: &Path,
    observations: &mut Vec<UsageObservation>,
    metadata: &mut BTreeMap<String, CapturedThreadMetadata>,
    mechanisms: &mut MechanismAnalyzer,
    errors: &mut Vec<String>,
) -> ObserverResult<()> {
    let bytes = read_or_empty(&directory.join("stdout.raw"))?;
    let chunks = read_stream_chunks(&directory.join("stdout-chunks.jsonl"))?;
    let frames = index_jsonl_with_chunks(&bytes, "stdout", &chunks);
    write_json_lines(&directory.join("events.jsonl"), frames.iter())?;
    let values = parse_top_level_json_lines(&bytes, errors, start, "stdout")?;
    let mut current_thread = resume_thread_from_argv(&start.argv);
    for value in &values {
        if value.get("type").and_then(Value::as_str) == Some("thread.started")
            && let Some(thread_id) = extract_thread_id(value)
        {
            current_thread = Some(thread_id);
        }
        let event_turn = start.invocation_id.clone();
        if let Some((command, output)) = extract_command_item_with_output(value)
            && let Some(thread_id) = current_thread.clone()
        {
            mechanisms.observe_command(CommandObservation {
                thread_id,
                turn_id: event_turn.clone(),
                command,
                output,
                duration_ns: None,
            });
        }
        if let Some(message) = extract_assistant_message(value)
            && let Some(thread_id) = current_thread.clone()
        {
            mechanisms.observe_assistant(&thread_id, &event_turn, message);
        }
        if value.get("type").and_then(Value::as_str) == Some("turn.completed")
            && let Some(thread_id) = current_thread.clone()
        {
            mechanisms.observe_turn_outcome(&thread_id, &event_turn, TurnOutcome::Completed);
        }
        let Some((usage_kind, usage)) = extract_usage(value) else {
            continue;
        };
        if usage_kind != "invocation-total" {
            errors.push(format!(
                "usage event in invocation {} is not a terminal invocation total",
                start.invocation_id
            ));
            continue;
        }
        let Some(thread_id) = current_thread.clone().or_else(|| extract_thread_id(value)) else {
            errors.push(format!(
                "usage event in invocation {} has no thread id",
                start.invocation_id
            ));
            continue;
        };
        let visible = start.primary && start.role.as_deref().is_some_and(is_visible_direct_role);
        observations.push(UsageObservation::per_invocation(
            &thread_id,
            &start.invocation_id,
            start.primary,
            visible,
            usage,
        ));
        metadata.insert(
            thread_id,
            CapturedThreadMetadata {
                agent_id: None,
                role: start.role.clone(),
            },
        );
    }
    Ok(())
}

fn analyze_locked_command(
    start: &InvocationStart,
    end: Option<&InvocationEnd>,
    directory: &Path,
    h4_candidate: bool,
    mechanisms: &mut MechanismAnalyzer,
) -> ObserverResult<()> {
    let stdout = read_or_empty(&directory.join("stdout.raw"))?;
    let stderr = read_or_empty(&directory.join("stderr.raw"))?;
    let command = locked_command_from_argv(&start.argv).unwrap_or_default();
    mechanisms.observe_locked_command_at(
        LockedCommandObservation {
            invocation_id: start.invocation_id.clone(),
            command,
            stdout,
            stderr,
            exit_code: end.and_then(|end| end.exit_code),
            terminating_signal: end.and_then(|end| end.terminating_signal),
        },
        Some(start.start_monotonic_ns),
        end.map(|end| end.end_monotonic_ns),
        h4_candidate,
    );
    mechanisms.observe(EvidenceInput::ProtocolBytes(
        directory
            .join("stdout.raw")
            .metadata()
            .map(|metadata| metadata.len())
            .unwrap_or(0)
            .saturating_add(
                directory
                    .join("stderr.raw")
                    .metadata()
                    .map(|metadata| metadata.len())
                    .unwrap_or(0),
            ),
    ));
    Ok(())
}

fn locked_command_from_argv(argv: &[EncodedArgument]) -> Option<String> {
    let wrapper = argv
        .iter()
        .map(|argument| argument.display.as_str())
        .find(|argument| argument.starts_with(LOCKED_COMMAND_PREFIX))?;
    let command = wrapper.strip_prefix(LOCKED_COMMAND_PREFIX)?;
    command
        .strip_suffix(") & work_leaf_child=$!; wait $work_leaf_child")
        .map(str::to_string)
}

fn parse_top_level_json_lines(
    bytes: &[u8],
    errors: &mut Vec<String>,
    start: &InvocationStart,
    stream: &str,
) -> ObserverResult<Vec<Value>> {
    let mut values = Vec::new();
    for (message_number, message) in split_wire_json_messages(bytes).into_iter().enumerate() {
        if let Some(error) = message.framing_error {
            errors.push(format!(
                "{} {stream} message {} has invalid framing: {error}",
                start.invocation_id,
                message_number + 1
            ));
            continue;
        }
        if message.body.is_empty() {
            continue;
        }
        match serde_json::from_slice(message.body) {
            Ok(value) => values.push(value),
            Err(error) => errors.push(format!(
                "{} {stream} message {} is not JSON: {error}",
                start.invocation_id,
                message_number + 1
            )),
        }
    }
    Ok(values)
}

fn extract_command_item_with_output(value: &Value) -> Option<(String, Vec<u8>)> {
    if value.get("method").and_then(Value::as_str) != Some("item/completed")
        && value.get("type").and_then(Value::as_str) != Some("item.completed")
    {
        return None;
    }
    let item = value
        .get("params")
        .and_then(|params| params.get("item"))
        .or_else(|| value.get("item"))?;
    if !matches!(
        item.get("type").and_then(Value::as_str),
        Some("commandExecution" | "command_execution")
    ) {
        return None;
    }
    let command = item
        .get("command")
        .and_then(Value::as_str)
        .map(str::to_string)?;
    let output = item
        .get("aggregatedOutput")
        .or_else(|| item.get("aggregated_output"))
        .or_else(|| item.get("output"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .as_bytes()
        .to_vec();
    Some((command, output))
}

fn extract_assistant_message(value: &Value) -> Option<&str> {
    if !matches!(
        value.get("method").and_then(Value::as_str),
        Some("item/completed")
    ) && !matches!(
        value.get("type").and_then(Value::as_str),
        Some("item.completed")
    ) {
        return None;
    }
    let item = value
        .get("params")
        .and_then(|params| params.get("item"))
        .or_else(|| value.get("item"))?;
    if !matches!(
        item.get("type").and_then(Value::as_str),
        Some("agentMessage" | "agent_message")
    ) {
        return None;
    }
    item.get("text").and_then(Value::as_str)
}

fn extract_turn_outcome(value: &Value) -> TurnOutcome {
    let status = value
        .get("params")
        .and_then(|params| params.get("turn"))
        .and_then(|turn| turn.get("status"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    match status {
        "completed" => TurnOutcome::Completed,
        "interrupted" | "cancelled" | "canceled" => TurnOutcome::Interrupted,
        _ => TurnOutcome::Failed,
    }
}

fn classify_command(command: &str) -> String {
    let trimmed = command.trim_start();
    if trimmed.starts_with("git ") || trimmed == "git" {
        "git-history"
    } else if starts_with_any_command(
        trimmed,
        &["cargo", "rustc", "rustfmt", "make", "cmake", "ninja"],
    ) {
        "build-check-test"
    } else if starts_with_any_command(
        trimmed,
        &[
            "rg", "grep", "sed", "cat", "head", "tail", "find", "fd", "ls", "pwd", "wc", "less",
            "bat",
        ],
    ) {
        "source-read-search"
    } else if starts_with_any_command(
        trimmed,
        &[
            "apply_patch",
            "cp",
            "install",
            "mkdir",
            "mv",
            "rm",
            "rmdir",
            "tee",
            "touch",
            "truncate",
        ],
    ) || trimmed.starts_with("sed -i")
        || trimmed.starts_with("perl -i")
    {
        "edit-write"
    } else if starts_with_any_command(trimmed, &["codex", "claude"]) {
        "provider-verification"
    } else if starts_with_any_command(
        trimmed,
        &[
            "command", "env", "printenv", "type", "uname", "which", "whereis",
        ],
    ) {
        "environment-tool-introspection"
    } else if starts_with_any_command(trimmed, &["jobs", "pgrep", "pidof", "ps", "pstree", "top"]) {
        "benchmark-introspection"
    } else {
        "other"
    }
    .to_string()
}

fn starts_with_any_command(command: &str, names: &[&str]) -> bool {
    names.iter().any(|name| {
        command == *name
            || command
                .strip_prefix(name)
                .is_some_and(|suffix| suffix.starts_with(char::is_whitespace))
    })
}

fn is_visible_work_leaf_agent(agent_id: &str) -> bool {
    agent_id.starts_with("user-")
        || agent_id.starts_with("review-")
        || agent_id.starts_with("linearize")
}

fn is_visible_direct_role(role: &str) -> bool {
    !role.is_empty() && role != "title-agent" && role != "command-agent"
}

fn cargo_validation_segments(command: &str) -> Vec<Vec<String>> {
    cargo_validation_segments_at_depth(command, 0)
}

const MAX_SHELL_PAYLOAD_DEPTH: usize = 4;

fn cargo_validation_segments_at_depth(command: &str, depth: usize) -> Vec<Vec<String>> {
    if depth > MAX_SHELL_PAYLOAD_DEPTH {
        return Vec::new();
    }
    let Some(raw_tokens) = shell_tokens(command) else {
        return Vec::new();
    };
    let opaque_indexes = shell_payload_indexes(&raw_tokens);
    let raw_arguments = raw_tokens.iter().map(String::as_str).collect::<Vec<_>>();
    let tokens = shell_effective_arguments(&raw_arguments, &opaque_indexes)
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let payloads = shell_payloads(&tokens);
    let mut validations = Vec::new();
    // Segment ranges are disjoint. Every token is classified once at this depth, so Cargo-looking
    // arguments cannot cause repeated tail scans.
    for (start, end) in shell_command_ranges(&tokens) {
        if shell_segment_has_ineligible_wrapper(&tokens, start, end) {
            validations.push(vec!["cargo".to_string(), "test".to_string()]);
            continue;
        }
        let Some(index) = shell_command_index(&tokens, start, end) else {
            continue;
        };
        let executable = executable_name(&tokens[index]);
        if executable == "eval" {
            validations.push(vec!["cargo".to_string(), "test".to_string()]);
            continue;
        }
        if executable != "cargo" {
            continue;
        }
        let segment = tokens[index..end].to_vec();
        if cargo_subcommand_and_arguments(&segment).0.is_none() {
            continue;
        }
        validations.push(segment);
    }
    if has_ineligible_shell_syntax(command) {
        validations.push(vec!["cargo".to_string(), "test".to_string()]);
    }
    for payload in payloads {
        validations.extend(cargo_validation_segments_at_depth(payload, depth + 1));
    }
    validations
}

fn shell_tokens(command: &str) -> Option<Vec<String>> {
    #[derive(Clone, Copy, Eq, PartialEq)]
    enum Quote {
        None,
        Single,
        Double,
    }

    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = Quote::None;
    let mut escaped = false;
    let mut token_started = false;
    let mut characters = command.chars().peekable();
    while let Some(character) = characters.next() {
        if escaped {
            current.push(character);
            escaped = false;
            token_started = true;
            continue;
        }
        match quote {
            Quote::Single => {
                if character == '\'' {
                    quote = Quote::None;
                } else {
                    current.push(character);
                }
            }
            Quote::Double => match character {
                '"' => quote = Quote::None,
                '\\' => escaped = true,
                _ => current.push(character),
            },
            Quote::None => match character {
                '\'' => {
                    quote = Quote::Single;
                    token_started = true;
                }
                '"' => {
                    quote = Quote::Double;
                    token_started = true;
                }
                '\\' => escaped = true,
                ' ' | '\t' | '\r' => {
                    push_shell_token(&mut tokens, &mut current, &mut token_started)
                }
                '\n' | ';' | '(' | ')' => {
                    push_shell_token(&mut tokens, &mut current, &mut token_started);
                    tokens.push(character.to_string());
                }
                '&' | '|' => {
                    push_shell_token(&mut tokens, &mut current, &mut token_started);
                    let mut separator = character.to_string();
                    if characters.peek() == Some(&character) {
                        separator.push(characters.next().expect("peeked separator exists"));
                    }
                    tokens.push(separator);
                }
                _ => {
                    current.push(character);
                    token_started = true;
                }
            },
        }
    }
    if escaped || quote != Quote::None {
        return None;
    }
    push_shell_token(&mut tokens, &mut current, &mut token_started);
    Some(tokens)
}

fn push_shell_token(tokens: &mut Vec<String>, current: &mut String, token_started: &mut bool) {
    if *token_started {
        tokens.push(std::mem::take(current));
        *token_started = false;
    }
}

fn shell_command_ranges(tokens: &[String]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0_usize;
    for (index, token) in tokens.iter().enumerate() {
        if !is_shell_separator(token) {
            continue;
        }
        if start < index {
            ranges.push((start, index));
        }
        start = index + 1;
    }
    if start < tokens.len() {
        ranges.push((start, tokens.len()));
    }
    ranges
}

fn is_shell_assignment(token: &str) -> bool {
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn shell_command_index(tokens: &[String], start: usize, end: usize) -> Option<usize> {
    let mut index = start;
    while index < end && is_shell_assignment(&tokens[index]) {
        index += 1;
    }
    while index < end {
        match executable_name(&tokens[index]).as_str() {
            "env" => {
                index += 1;
                while index < end {
                    let argument = &tokens[index];
                    if argument == "--" {
                        index += 1;
                        break;
                    }
                    if is_shell_assignment(argument) {
                        index += 1;
                        continue;
                    }
                    if matches!(
                        argument.as_str(),
                        "-u" | "--unset" | "-C" | "--chdir" | "-a" | "--argv0"
                    ) {
                        index = index.saturating_add(2);
                        continue;
                    }
                    if matches!(argument.as_str(), "-S" | "--split-string")
                        || argument.starts_with("--split-string=")
                        || (argument.starts_with("-S") && argument.len() > 2)
                    {
                        return None;
                    }
                    if argument.starts_with('-') {
                        index += 1;
                        continue;
                    }
                    break;
                }
                while index < end && is_shell_assignment(&tokens[index]) {
                    index += 1;
                }
            }
            "command" => {
                index += 1;
                while index < end && tokens[index].starts_with('-') {
                    if tokens[index][1..].contains(['v', 'V']) {
                        return None;
                    }
                    index += 1;
                }
            }
            _ => break,
        }
    }
    (index < end).then_some(index)
}

fn shell_segment_has_ineligible_wrapper(tokens: &[String], start: usize, end: usize) -> bool {
    let mut index = start;
    while index < end && is_shell_assignment(&tokens[index]) {
        index += 1;
    }
    while index < end {
        match executable_name(&tokens[index]).as_str() {
            "command" => {
                index += 1;
                while index < end && tokens[index].starts_with('-') {
                    if tokens[index][1..].contains(['v', 'V']) {
                        return false;
                    }
                    index += 1;
                }
            }
            "env" => {
                index += 1;
                while index < end {
                    let argument = &tokens[index];
                    if argument == "--" {
                        index += 1;
                        break;
                    }
                    if matches!(argument.as_str(), "-S" | "--split-string")
                        || argument.starts_with("--split-string=")
                        || (argument.starts_with("-S") && argument.len() > 2)
                    {
                        return true;
                    }
                    if is_shell_assignment(argument) {
                        index += 1;
                        continue;
                    }
                    if matches!(
                        argument.as_str(),
                        "-u" | "--unset" | "-C" | "--chdir" | "-a" | "--argv0"
                    ) {
                        index = index.saturating_add(2);
                        continue;
                    }
                    if !argument.starts_with('-') {
                        break;
                    }
                    index += 1;
                }
                while index < end && is_shell_assignment(&tokens[index]) {
                    index += 1;
                }
            }
            _ => return false,
        }
        if index >= end {
            return false;
        }
        if matches!(executable_name(&tokens[index]).as_str(), "env" | "command") {
            continue;
        }
        return false;
    }
    false
}

fn has_ineligible_shell_syntax(command: &str) -> bool {
    let bytes = command.as_bytes();
    let mut quote = 0_u8;
    let mut escaped = false;
    let mut in_comment = false;
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_comment {
            if byte == b'\n' {
                in_comment = false;
            }
            index += 1;
            continue;
        }
        if escaped {
            escaped = false;
            index += 1;
            continue;
        }
        if byte == b'\\' && quote != b'\'' {
            escaped = true;
            index += 1;
            continue;
        }
        if quote == b'\'' {
            if byte == b'\'' {
                quote = 0;
            }
            index += 1;
            continue;
        }
        if byte == b'\'' && quote == 0 {
            quote = b'\'';
            index += 1;
            continue;
        }
        if byte == b'"' {
            quote = if quote == b'"' { 0 } else { b'"' };
            index += 1;
            continue;
        }
        if quote == 0
            && byte == b'#'
            && (index == 0
                || bytes[index - 1].is_ascii_whitespace()
                || matches!(bytes[index - 1], b';' | b'&' | b'|' | b'(' | b')'))
        {
            in_comment = true;
            index += 1;
            continue;
        }
        if byte == b'`'
            || (index + 1 < bytes.len()
                && bytes[index + 1] == b'('
                && matches!(byte, b'$' | b'<' | b'>'))
            || (quote == 0 && index + 1 < bytes.len() && byte == b'<' && bytes[index + 1] == b'<')
        {
            return true;
        }
        index += 1;
    }
    false
}

fn shell_payloads(tokens: &[String]) -> Vec<&str> {
    shell_payload_indexes(tokens)
        .into_iter()
        .filter_map(|index| tokens.get(index).map(String::as_str))
        .collect()
}

fn shell_payload_indexes(tokens: &[String]) -> BTreeSet<usize> {
    let mut payload_indexes = BTreeSet::new();
    for (start, end) in shell_command_ranges(tokens) {
        let Some(index) = shell_command_index(tokens, start, end) else {
            continue;
        };
        if !matches!(
            executable_name(&tokens[index]).as_str(),
            "bash" | "dash" | "ksh" | "sh" | "zsh"
        ) {
            continue;
        }
        let mut option_index = index + 1;
        while option_index < end {
            let option = &tokens[option_index];
            if option == "--" || !option.starts_with(['-', '+']) {
                break;
            }
            if option.starts_with('-') && option[1..].contains('c') && option_index + 1 < end {
                payload_indexes.insert(option_index + 1);
                break;
            }
            if matches!(
                option.as_str(),
                "-o" | "+o" | "-O" | "+O" | "--rcfile" | "--init-file"
            ) {
                option_index = option_index.saturating_add(2);
                continue;
            }
            option_index += 1;
        }
    }
    payload_indexes
}

fn executable_name(argument: &str) -> String {
    Path::new(argument)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(argument)
        .to_ascii_lowercase()
}

fn is_shell_separator(token: &str) -> bool {
    matches!(token, "\n" | "&" | "&&" | "(" | ")" | ";" | "|" | "||")
}

fn bundle_paths_in_command(command: &str, known_paths: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
    bundle_paths_in_command_at_depth(command, known_paths, 0)
}

const MAX_BUNDLE_PATHS_PER_COMMAND: usize = 64;

fn bundle_paths_in_command_at_depth(
    command: &str,
    known_paths: &BTreeSet<PathBuf>,
    depth: usize,
) -> BTreeSet<PathBuf> {
    let mut paths = BTreeSet::new();
    if depth > MAX_SHELL_PAYLOAD_DEPTH {
        return paths;
    }
    let Some(raw_tokens) = shell_tokens(command) else {
        return paths;
    };
    let opaque_indexes = shell_payload_indexes(&raw_tokens);
    let raw_arguments = raw_tokens.iter().map(String::as_str).collect::<Vec<_>>();
    let tokens = shell_comment_arguments(&raw_arguments, &opaque_indexes)
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    for token in &tokens {
        let token = token.trim_matches(|character| matches!(character, '<' | '>'));
        let candidates = [Some(token), token.split_once('=').map(|(_, value)| value)];
        for candidate in candidates.into_iter().flatten().map(Path::new) {
            if known_paths.contains(candidate) {
                paths.insert(candidate.to_path_buf());
            }
        }
    }
    for payload in shell_payloads(&tokens) {
        paths.extend(bundle_paths_in_command_at_depth(
            payload,
            known_paths,
            depth + 1,
        ));
    }
    paths
}

fn byte_search_prefix(needle: &[u8]) -> Vec<usize> {
    let mut prefix = vec![0_usize; needle.len()];
    let mut matched = 0_usize;
    for index in 1..needle.len() {
        while matched > 0 && needle[index] != needle[matched] {
            matched = prefix[matched - 1];
        }
        if needle[index] == needle[matched] {
            matched += 1;
            prefix[index] = matched;
        }
    }
    prefix
}

fn bytes_contain_with_prefix(haystack: &[u8], needle: &[u8], prefix: &[usize]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() || prefix.len() != needle.len() {
        return false;
    }
    let mut matched = 0_usize;
    for byte in haystack {
        while matched > 0 && *byte != needle[matched] {
            matched = prefix[matched - 1];
        }
        if *byte == needle[matched] {
            matched += 1;
            if matched == needle.len() {
                return true;
            }
        }
    }
    false
}

fn canonical_cargo_validation_subcommand(subcommand: &str) -> Option<&'static str> {
    match subcommand.to_ascii_lowercase().as_str() {
        "b" | "build" => Some("build"),
        "c" | "check" => Some("check"),
        "clippy" => Some("clippy"),
        "d" | "doc" => Some("doc"),
        "fmt" => Some("fmt"),
        "nextest" => Some("nextest"),
        "t" | "test" => Some("test"),
        _ => None,
    }
}

fn cargo_subcommand_and_arguments(segment: &[String]) -> (Option<&'static str>, Vec<&str>) {
    if segment
        .first()
        .is_none_or(|token| executable_name(token) != "cargo")
    {
        return (None, Vec::new());
    }
    let mut index = 1;
    if segment
        .get(index)
        .is_some_and(|argument| argument.starts_with('+'))
    {
        index += 1;
    }
    while let Some(argument) = segment.get(index).map(String::as_str) {
        if argument == "--" {
            index += 1;
            break;
        }
        if matches!(
            argument,
            "-V" | "--version" | "--list" | "--explain" | "-h" | "--help"
        ) {
            return (None, Vec::new());
        }
        if matches!(argument, "--color" | "-C" | "--config" | "-Z") {
            if segment.get(index + 1).is_none() {
                return (None, Vec::new());
            }
            index += 2;
            continue;
        }
        if argument.starts_with("--color=")
            || argument.starts_with("--config=")
            || argument.starts_with("-C") && argument.len() > 2
            || argument.starts_with("-Z") && argument.len() > 2
        {
            index += 1;
            continue;
        }
        if matches!(
            argument,
            "-q" | "--quiet" | "--verbose" | "--locked" | "--offline" | "--frozen"
        ) || argument
            .strip_prefix('-')
            .is_some_and(|suffix| !suffix.is_empty() && suffix.bytes().all(|byte| byte == b'v'))
        {
            index += 1;
            continue;
        }
        break;
    }
    let subcommand = segment
        .get(index)
        .and_then(|argument| canonical_cargo_validation_subcommand(argument));
    let arguments = segment.iter().skip(index + 1).map(String::as_str).collect();
    (subcommand, arguments)
}

fn shell_comment_arguments<'a>(
    arguments: &[&'a str],
    opaque_indexes: &BTreeSet<usize>,
) -> Vec<&'a str> {
    let mut filtered = Vec::new();
    let mut in_comment = false;
    for (index, argument) in arguments.iter().enumerate() {
        if in_comment {
            if *argument == "\n" {
                in_comment = false;
                filtered.push(*argument);
            }
            continue;
        }
        if opaque_indexes.contains(&index) {
            filtered.push(*argument);
            continue;
        }
        if argument.starts_with('#') {
            in_comment = true;
            continue;
        }
        filtered.push(*argument);
    }
    filtered
}

fn shell_effective_arguments<'a>(
    arguments: &[&'a str],
    opaque_indexes: &BTreeSet<usize>,
) -> Vec<&'a str> {
    const REDIRECTION_OPERATORS: [&str; 10] =
        ["<<<", "<<-", ">>", "<<", "<>", ">&", "<&", ">|", ">", "<"];

    let mut filtered = Vec::new();
    let mut skip_redirection_target = false;
    let mut in_comment = false;
    for (index, argument) in arguments.iter().enumerate() {
        if in_comment {
            if *argument == "\n" {
                in_comment = false;
                filtered.push(*argument);
            }
            continue;
        }
        if skip_redirection_target {
            skip_redirection_target = *argument == "&";
            continue;
        }
        if opaque_indexes.contains(&index) {
            filtered.push(*argument);
            continue;
        }
        if argument.starts_with('#') {
            in_comment = true;
            continue;
        }
        let redirect_at = argument.find(['<', '>']);
        let Some(redirect_at) = redirect_at else {
            filtered.push(*argument);
            continue;
        };
        let prefix = &argument[..redirect_at];
        if !prefix.is_empty() && !prefix.bytes().all(|byte| byte.is_ascii_digit()) {
            filtered.push(prefix);
        }
        let redirection = &argument[redirect_at..];
        let operator = REDIRECTION_OPERATORS
            .iter()
            .find(|operator| redirection.starts_with(**operator))
            .copied()
            .unwrap_or(redirection);
        skip_redirection_target = redirection.len() == operator.len();
    }
    filtered
}

fn resume_thread_from_argv(argv: &[EncodedArgument]) -> Option<String> {
    let displays = argv
        .iter()
        .map(|argument| argument.display.as_str())
        .collect::<Vec<_>>();
    let resume = displays.iter().position(|argument| *argument == "resume")?;
    displays
        .iter()
        .skip(resume + 1)
        .find(|argument| !argument.starts_with('-'))
        .map(|thread_id| (*thread_id).to_string())
}

#[derive(Default)]
struct ThreadUsageAccumulator {
    observation: Option<UsageObservation>,
    cumulative: Option<CapturedUsage>,
    invocations: BTreeMap<String, CapturedUsage>,
}

fn validate_usage_accounting(observations: &[UsageObservation], errors: &mut Vec<String>) {
    let mut accounting = BTreeMap::<&str, UsageAccounting>::new();
    for observation in observations {
        if let Some(previous) = accounting.insert(&observation.thread_id, observation.accounting)
            && previous != observation.accounting
        {
            errors.push(format!(
                "thread {} mixes cumulative and per-invocation usage records",
                observation.thread_id
            ));
        }
    }
}

fn final_thread_observations(observations: &[UsageObservation]) -> Vec<UsageObservation> {
    let mut threads = BTreeMap::<String, ThreadUsageAccumulator>::new();
    for observation in observations {
        let entry = threads.entry(observation.thread_id.clone()).or_default();
        let representative = entry.observation.get_or_insert_with(|| observation.clone());
        representative.primary |= observation.primary;
        representative.visible |= observation.visible;
        if observation.invocation_id > representative.invocation_id {
            representative.invocation_id = observation.invocation_id.clone();
        }
        match observation.accounting {
            UsageAccounting::CumulativeThread => {
                if entry
                    .cumulative
                    .is_none_or(|usage| observation.usage.score() > usage.score())
                {
                    entry.cumulative = Some(observation.usage);
                }
            }
            UsageAccounting::PerInvocation => {
                let invocation = entry
                    .invocations
                    .entry(observation.invocation_id.clone())
                    .or_insert(observation.usage);
                if observation.usage.score() > invocation.score() {
                    *invocation = observation.usage;
                }
            }
        }
    }
    threads
        .into_values()
        .filter_map(|entry| {
            let mut observation = entry.observation?;
            observation.usage = entry.cumulative.unwrap_or_else(|| {
                entry
                    .invocations
                    .into_values()
                    .fold(CapturedUsage::default(), CapturedUsage::combine)
            });
            Some(observation)
        })
        .collect()
}

fn read_or_empty(path: &Path) -> ObserverResult<Vec<u8>> {
    match fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

fn write_json_lines<'a, T>(path: &Path, records: impl Iterator<Item = &'a T>) -> ObserverResult<()>
where
    T: Serialize + 'a,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

pub fn record_timeline(
    config: &CaptureConfig,
    event: &str,
    detail: Option<&str>,
) -> ObserverResult<()> {
    let record = json!({
        "event": event,
        "detail": detail,
        "monotonic_ns": monotonic_time_ns().to_string(),
        "unix_ns": unix_time_ns().to_string(),
    });
    // The append helper secures the touched file in constant work. Timeline growth must not trigger
    // a recursive pass over the complete capture tree for every event.
    append_json_line(&config.root.join("timeline.jsonl"), &record)
}

pub fn capture_git_checkpoint(
    config: &CaptureConfig,
    repository: &Path,
    label: &str,
) -> ObserverResult<()> {
    let safe_label = safe_component(label);
    let root = config.root.join("git-checkpoints/files").join(&safe_label);
    fs::create_dir_all(&root)?;
    let commands: [(&str, &[&str]); 6] = [
        ("head.txt", &["rev-parse", "HEAD"]),
        ("status.txt", &["status", "--porcelain=v1"]),
        ("worktree.diff", &["diff", "--binary"]),
        ("index.diff", &["diff", "--cached", "--binary"]),
        (
            "commit-graph.txt",
            &["log", "--graph", "--format=%H%x09%P%x09%s", "--all"],
        ),
        ("reflog.txt", &["reflog", "--format=%H%x09%gD%x09%gs"]),
    ];
    let mut hashes = BTreeMap::new();
    for (name, args) in commands {
        let output = Command::new("git")
            .arg("-C")
            .arg(repository)
            .args(args)
            .output()
            .map_err(|error| {
                ObserverError::new(format!(
                    "failed to capture Git checkpoint {label} command {args:?}: {error}"
                ))
            })?;
        if !output.status.success() {
            return Err(ObserverError::new(format!(
                "Git checkpoint {label} command {args:?} failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        let path = root.join(name);
        fs::write(&path, &output.stdout)?;
        hashes.insert(name.to_string(), sha256_bytes(&output.stdout));
    }
    let head = fs::read_to_string(root.join("head.txt"))?
        .trim()
        .to_string();
    let record = json!({
        "label": label,
        "safe_label": safe_label,
        "head": head,
        "monotonic_ns": monotonic_time_ns().to_string(),
        "files": hashes,
    });
    append_json_line(&config.root.join("git-checkpoints/manifest.jsonl"), &record)?;
    record_timeline(config, "git-checkpoint", Some(label))
}

pub fn archive_context_bundles(config: &CaptureConfig, source: &Path) -> ObserverResult<usize> {
    let destination = config.root.join("context-bundles/files");
    fs::create_dir_all(&destination)?;
    let mut files = Vec::new();
    collect_regular_files(source, source, &mut files)?;
    files.sort();
    let manifest_path = config.root.join("context-bundles/manifest.jsonl");
    let mut archived = 0;
    for relative in files {
        let from = source.join(&relative);
        let to = destination.join(&relative);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&from, &to)?;
        let bytes = fs::read(&to)?;
        let (file_snapshots, parse_error) = match parse_context_bundle_snapshots(&bytes) {
            Ok(snapshots) => (
                snapshots
                    .iter()
                    .map(ContextBundleSnapshot::manifest_value)
                    .collect::<Vec<_>>(),
                None,
            ),
            Err(error) => (Vec::new(), Some(error)),
        };
        let record = json!({
            "source": from,
            "archived_path": Path::new("files").join(&relative),
            "bytes": bytes.len(),
            "sha256": sha256_bytes(&bytes),
            "file_snapshots": file_snapshots,
            "parse_error": parse_error,
        });
        append_json_line(&manifest_path, &record)?;
        archived += 1;
    }
    record_timeline(
        config,
        "context-bundles-archived",
        Some(&archived.to_string()),
    )?;
    harden_artifact_permissions(&config.root)?;
    Ok(archived)
}

fn parse_context_bundle_snapshots(bytes: &[u8]) -> Result<Vec<ContextBundleSnapshot>, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| format!("context bundle is not UTF-8: {error}"))?;
    if !text.starts_with("# Work Leaf Context Bundle\n") {
        return Err("context bundle header is missing".to_string());
    }
    let mut snapshots = Vec::new();
    let mut cursor = 0_usize;
    let begin_marker = "\n----- BEGIN FILE ";
    while let Some(relative) = text[cursor..].find(begin_marker) {
        let path_start = cursor + relative + begin_marker.len();
        let Some(path_end_relative) = text[path_start..].find(" -----\n") else {
            return Err("context bundle begin marker is incomplete".to_string());
        };
        let path_end = path_start + path_end_relative;
        let path = &text[path_start..path_end];
        let digest_start = path_end + " -----\n".len();
        let Some(digest_line) = text[digest_start..].lines().next() else {
            return Err(format!("context bundle file {path} has no digest"));
        };
        let Some(digest) = digest_line.strip_prefix("digest: ") else {
            return Err(format!(
                "context bundle file {path} has an invalid digest line"
            ));
        };
        let content_start = digest_start + digest_line.len() + 2;
        let end_marker = format!("----- END FILE {path} -----\n");
        let Some(content_end_relative) = text[content_start..].find(&end_marker) else {
            return Err(format!(
                "context bundle file {path} has no matching end marker"
            ));
        };
        let content_end = content_start + content_end_relative;
        let content = &text[content_start..content_end];
        let declared_bytes = digest
            .rsplit_once("; bytes:")
            .and_then(|(_, bytes)| bytes.parse::<usize>().ok())
            .ok_or_else(|| format!("context bundle file {path} digest has no byte length"))?;
        let snapshot_text = content
            .get(..declared_bytes)
            .ok_or_else(|| format!("context bundle file {path} is shorter than its digest"))?;
        let computed = content_digest_for_observer(snapshot_text);
        if computed != digest {
            return Err(format!(
                "context bundle file {path} digest {digest} does not match {computed}"
            ));
        }
        snapshots.push(ContextBundleSnapshot {
            path: path.to_string(),
            digest: digest.to_string(),
            text: Arc::from(snapshot_text),
            bundle_component_bytes: content.len() as u64,
        });
        cursor = content_end + end_marker.len();
    }
    if snapshots.is_empty() {
        return Err("context bundle contains no file snapshots".to_string());
    }
    Ok(snapshots)
}

fn collect_regular_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> ObserverResult<()> {
    if !current.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            return Err(ObserverError::new(format!(
                "context bundle archive refuses symlink {}",
                entry.path().display()
            )));
        }
        if file_type.is_dir() {
            collect_regular_files(root, &entry.path(), files)?;
        } else if file_type.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|error| ObserverError::new(error.to_string()))?
                .to_path_buf();
            files.push(relative);
        }
    }
    Ok(())
}

fn append_json_line(path: &Path, value: &impl Serialize) -> ObserverResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        options.mode(0o600);
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(path)?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    file.write_all(&bytes)?;
    file.flush()?;
    Ok(())
}

fn safe_component(value: &str) -> String {
    let text = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    if text.is_empty() {
        "checkpoint".to_string()
    } else {
        text
    }
}

fn scan_for_secret_markers(config: &CaptureConfig) -> ObserverResult<Vec<String>> {
    let mut files = Vec::new();
    collect_artifact_files_for_secret_scan(&config.root, &config.root, &mut files)?;
    let mut errors = Vec::new();
    for (path, relative) in files {
        let bytes = fs::read(&path)?;
        for marker in detected_secret_markers(&bytes) {
            errors.push(format!(
                "secret marker {marker} found in {}",
                relative.display()
            ));
        }
    }
    Ok(errors)
}

fn collect_artifact_files_for_secret_scan(
    root: &Path,
    current: &Path,
    files: &mut Vec<(PathBuf, PathBuf)>,
) -> ObserverResult<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_artifact_files_for_secret_scan(root, &entry.path(), files)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| ObserverError::new(error.to_string()))?
            .to_path_buf();
        if matches!(
            relative.file_name().and_then(OsStr::to_str),
            Some(
                "analysis.json"
                    | "analysis.err"
                    | "analysis-pre-rollout.json"
                    | "analysis-pre-rollout.err"
                    | "capture-audit.txt"
                    | "counterfactuals.jsonl"
                    | "mechanism-summary.json"
                    | "rollout-extraction.json"
                    | "rollout-extraction.err"
            )
        ) {
            continue;
        }
        files.push((entry.path(), relative));
    }
    Ok(())
}

fn detected_secret_markers(bytes: &[u8]) -> Vec<&'static str> {
    let mut markers = Vec::new();
    if contains_credential_token(bytes, b"sk-", 16) {
        markers.push("sk-token");
    }
    if contains_credential_token(bytes, b"Bearer ", 16) {
        markers.push("bearer-token");
    }
    for (label, prefix) in [
        ("OPENAI_API_KEY", b"OPENAI_API_KEY=".as_slice()),
        ("ANTHROPIC_API_KEY", b"ANTHROPIC_API_KEY=".as_slice()),
        ("access_token", b"\"access_token\"".as_slice()),
        ("refresh_token", b"\"refresh_token\"".as_slice()),
    ] {
        if contains_credential_assignment(bytes, prefix) {
            markers.push(label);
        }
    }
    markers
}

fn contains_credential_token(bytes: &[u8], prefix: &[u8], minimum_value_bytes: usize) -> bool {
    bytes
        .windows(prefix.len())
        .enumerate()
        .filter(|(_, window)| window.eq_ignore_ascii_case(prefix))
        .any(|(index, _)| {
            if index > 0 && (bytes[index - 1].is_ascii_alphanumeric() || bytes[index - 1] == b'_') {
                return false;
            }
            let value = &bytes[index + prefix.len()..];
            value
                .iter()
                .take_while(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
                })
                .count()
                >= minimum_value_bytes
        })
}

fn contains_credential_assignment(bytes: &[u8], prefix: &[u8]) -> bool {
    bytes
        .windows(prefix.len())
        .enumerate()
        .filter(|(_, window)| window.eq_ignore_ascii_case(prefix))
        .any(|(index, _)| {
            let mut value = &bytes[index + prefix.len()..];
            while value.first().is_some_and(|byte| {
                byte.is_ascii_whitespace() || matches!(byte, b':' | b'=' | b'\'' | b'"')
            }) {
                value = &value[1..];
            }
            let length = value
                .iter()
                .take_while(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
                })
                .count();
            length >= 16 && !value.starts_with(b"<redacted>")
        })
}

fn harden_artifact_permissions(root: &Path) -> ObserverResult<()> {
    #[cfg(unix)]
    fn harden(path: &Path) -> io::Result<()> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() {
            return Ok(());
        }
        if metadata.is_dir() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
            for entry in fs::read_dir(path)? {
                harden(&entry?.path())?;
            }
        } else if metadata.is_file() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    #[cfg(unix)]
    harden(root)?;
    #[cfg(not(unix))]
    let _ = root;
    Ok(())
}

fn create_private_directory(path: &Path) -> ObserverResult<()> {
    #[cfg(unix)]
    {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true).mode(0o700).create(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(not(unix))]
    fs::create_dir_all(path)?;
    Ok(())
}

fn create_private_file(path: &Path) -> ObserverResult<File> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        options.mode(0o600);
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path)?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

fn write_capture_audit(config: &CaptureConfig, summary: &AnalysisSummary) -> ObserverResult<()> {
    let mut text = String::new();
    use fmt::Write as _;
    let _ = writeln!(text, "run_id={}", config.run_id);
    let _ = writeln!(text, "condition={}", config.condition);
    let _ = writeln!(text, "capture_complete={}", summary.capture_complete);
    let _ = writeln!(text, "invocations={}", summary.invocation_count);
    let _ = writeln!(
        text,
        "complete_invocations={}",
        summary.complete_invocation_count
    );
    let _ = writeln!(
        text,
        "passthrough_invocations={}",
        summary.passthrough_invocation_count
    );
    let _ = writeln!(
        text,
        "visible_threads={}",
        summary.usage_scopes.visible_role.thread_count
    );
    let _ = writeln!(
        text,
        "primary_threads={}",
        summary.usage_scopes.primary_condition.thread_count
    );
    let _ = writeln!(
        text,
        "total_threads={}",
        summary.usage_scopes.total_workflow.thread_count
    );
    for error in &summary.errors {
        let _ = writeln!(text, "error={error}");
    }
    fs::write(config.root.join("capture-audit.txt"), text)?;
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RolloutMetadata {
    pub thread_id: String,
    pub cwd: PathBuf,
    pub model: String,
    pub effort: String,
    pub cli_version: String,
    pub usage: CapturedUsage,
    pub source_sha256: String,
    pub source_relative_path: PathBuf,
    pub primary: bool,
    pub visible: bool,
    pub descendant: bool,
}

#[derive(Clone, Debug)]
struct CapturedExecThread {
    thread_id: String,
    invocation_id: String,
    cwd: PathBuf,
    started_unix_ns: u128,
    primary: bool,
    visible: bool,
    role: Option<String>,
    has_terminal_usage: bool,
}

#[derive(Clone, Debug)]
struct RolloutExpectation {
    thread: ThreadSummary,
    expected_cwd: Option<PathBuf>,
    allow_rollout_supersession: bool,
    latest_capture_unix_ns: u128,
    invocation_count: usize,
}

fn captured_exec_threads(
    config: &CaptureConfig,
    inventory: &[ProcessInventoryRecord],
) -> ObserverResult<Vec<CapturedExecThread>> {
    let mut threads = Vec::new();
    for process in inventory.iter().filter(|process| {
        process.start.capture_kind == CaptureKind::ExecJson && process.end.is_some()
    }) {
        let directory = config
            .root
            .join(process.start.capture_kind.artifact_directory())
            .join(&process.start.invocation_id);
        let frames = index_jsonl(&read_or_empty(&directory.join("stdout.raw"))?, "stdout");
        let has_terminal_usage = frames
            .iter()
            .any(|frame| frame.usage_kind.as_deref() == Some("invocation-total"));
        let visible = process.start.primary
            && process
                .start
                .role
                .as_deref()
                .is_some_and(is_visible_direct_role);
        let mut invocation_threads = BTreeSet::new();
        for frame in frames.iter().filter(|frame| {
            frame.event_type.as_deref() == Some("thread.started") && frame.thread_id.is_some()
        }) {
            let thread_id = frame
                .thread_id
                .as_ref()
                .expect("filtered thread.started frame has a thread id");
            if !invocation_threads.insert(thread_id.clone()) {
                continue;
            }
            threads.push(CapturedExecThread {
                thread_id: thread_id.clone(),
                invocation_id: process.start.invocation_id.clone(),
                cwd: process.start.cwd.clone(),
                started_unix_ns: process.start.start_unix_ns,
                primary: process.start.primary,
                visible,
                role: process.start.role.clone(),
                has_terminal_usage,
            });
        }
    }
    Ok(threads)
}

fn usage_contains(full: CapturedUsage, partial: CapturedUsage) -> bool {
    full.input_tokens >= partial.input_tokens
        && full.cached_input_tokens >= partial.cached_input_tokens
        && full.output_tokens >= partial.output_tokens
        && full.reasoning_output_tokens >= partial.reasoning_output_tokens
}

fn supplement_usage_from_rollouts(
    config: &CaptureConfig,
    inventory: &[ProcessInventoryRecord],
    observations: &mut Vec<UsageObservation>,
    metadata: &mut BTreeMap<String, CapturedThreadMetadata>,
    errors: &mut Vec<String>,
) -> ObserverResult<()> {
    let path = config.root.join("rollout-metadata.jsonl");
    if !path.is_file() {
        errors.push("rollout audit passed without rollout metadata".to_string());
        return Ok(());
    }
    let captured = captured_exec_threads(config, inventory)?;
    let mut latest_capture = BTreeMap::<String, CapturedExecThread>::new();
    for thread in captured {
        let replace = latest_capture
            .get(&thread.thread_id)
            .is_none_or(|current| thread.started_unix_ns > current.started_unix_ns);
        if replace {
            latest_capture.insert(thread.thread_id.clone(), thread);
        }
    }
    let current = final_thread_observations(observations)
        .into_iter()
        .map(|thread| (thread.thread_id.clone(), thread))
        .collect::<BTreeMap<_, _>>();

    for (line_number, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row: RolloutMetadata = serde_json::from_str(&line).map_err(|error| {
            ObserverError::new(format!(
                "invalid rollout metadata line {}: {error}",
                line_number + 1
            ))
        })?;
        let observed = current.get(&row.thread_id);
        let capture = latest_capture.get(&row.thread_id);
        if observed.is_none() && capture.is_none() {
            errors.push(format!(
                "rollout metadata thread {} has no captured provider process",
                row.thread_id
            ));
            continue;
        }
        if observed.is_some_and(|thread| !usage_contains(row.usage, thread.usage)) {
            errors.push(format!(
                "rollout metadata thread {} regresses captured provider usage",
                row.thread_id
            ));
            continue;
        }
        let invocation_id = capture
            .map(|thread| thread.invocation_id.clone())
            .or_else(|| observed.map(|thread| thread.invocation_id.clone()))
            .expect("captured or observed rollout thread has an invocation");
        let per_invocation = capture.is_some()
            || observed.is_some_and(|thread| thread.accounting == UsageAccounting::PerInvocation);
        if per_invocation {
            if observed.is_none_or(|thread| thread.usage != row.usage) {
                let usage = observed
                    .and_then(|thread| row.usage.checked_difference(thread.usage))
                    .unwrap_or(row.usage);
                observations.push(UsageObservation::per_invocation(
                    &row.thread_id,
                    invocation_id,
                    row.primary,
                    row.visible,
                    usage,
                ));
            }
        } else {
            observations.push(UsageObservation::new(
                &row.thread_id,
                invocation_id,
                row.primary,
                row.visible,
                row.usage,
            ));
        }
        if let Some(capture) = capture {
            metadata
                .entry(row.thread_id.clone())
                .or_insert_with(|| CapturedThreadMetadata {
                    agent_id: None,
                    role: capture.role.clone(),
                });
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RolloutAudit {
    pub observed_threads: usize,
    pub matched_threads: usize,
    #[serde(default)]
    pub session_only_threads: Vec<String>,
    pub missing_threads: Vec<String>,
    pub unobserved_cwd_threads: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct ParsedRollout {
    thread_id: String,
    cwd: PathBuf,
    started_unix_ns: Option<u128>,
    model: String,
    effort: String,
    cli_version: String,
    usage: Option<CapturedUsage>,
    task_usages: Vec<CapturedUsage>,
    profiles: BTreeSet<(String, String)>,
}

impl ParsedRollout {
    fn normalize_direct_invocation_usage(&mut self) {
        if self.task_usages.is_empty() {
            return;
        }
        self.usage = Some(
            self.task_usages
                .iter()
                .copied()
                .fold(CapturedUsage::default(), CapturedUsage::combine),
        );
    }

    fn invocation_count(&self) -> usize {
        if self.task_usages.is_empty() {
            usize::from(self.usage.is_some())
        } else {
            self.task_usages.len()
        }
    }
}

pub fn extract_rollout_metadata(
    config: &CaptureConfig,
    sessions_root: &Path,
) -> ObserverResult<RolloutAudit> {
    let summary_path = config.root.join("mechanism-summary.json");
    if !summary_path.is_file() {
        return Err(ObserverError::new(
            "analyze must run before exact-thread rollout extraction",
        ));
    }
    let summary: AnalysisSummary = serde_json::from_slice(&fs::read(summary_path)?)?;
    let inventory = read_process_inventory(config)?;
    let cwd_by_invocation = inventory
        .iter()
        .map(|record| (record.start.invocation_id.clone(), record.start.cwd.clone()))
        .collect::<BTreeMap<_, _>>();
    let start_by_invocation = inventory
        .iter()
        .map(|record| {
            (
                record.start.invocation_id.clone(),
                record.start.start_unix_ns,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut expected = summary
        .threads
        .iter()
        .cloned()
        .map(|thread| {
            let expected_cwd = cwd_by_invocation.get(&thread.invocation_id).cloned();
            let latest_capture_unix_ns = start_by_invocation
                .get(&thread.invocation_id)
                .copied()
                .unwrap_or_default();
            (
                thread.thread_id.clone(),
                RolloutExpectation {
                    thread,
                    expected_cwd,
                    allow_rollout_supersession: false,
                    latest_capture_unix_ns,
                    invocation_count: 0,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for capture in captured_exec_threads(config, &inventory)? {
        if let Some(expectation) = expected.get_mut(&capture.thread_id) {
            expectation.invocation_count = expectation.invocation_count.saturating_add(1);
            expectation.thread.primary |= capture.primary;
            expectation.thread.visible |= capture.visible;
            if !capture.has_terminal_usage
                && capture.started_unix_ns >= expectation.latest_capture_unix_ns
            {
                expectation.thread.invocation_id = capture.invocation_id.clone();
                expectation.expected_cwd = Some(capture.cwd.clone());
                expectation.allow_rollout_supersession = true;
                expectation.latest_capture_unix_ns = capture.started_unix_ns;
                if expectation.thread.role.is_none() {
                    expectation.thread.role = capture.role.clone();
                }
            }
            continue;
        }
        expected.insert(
            capture.thread_id.clone(),
            RolloutExpectation {
                thread: ThreadSummary {
                    thread_id: capture.thread_id,
                    invocation_id: capture.invocation_id,
                    primary: capture.primary,
                    visible: capture.visible,
                    agent_id: None,
                    role: capture.role,
                    usage: CapturedUsage::default(),
                },
                expected_cwd: Some(capture.cwd),
                allow_rollout_supersession: !capture.has_terminal_usage,
                latest_capture_unix_ns: capture.started_unix_ns,
                invocation_count: 1,
            },
        );
    }
    let session_only_threads = summary
        .session_only_threads
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let observed_cwds = inventory
        .iter()
        .map(|record| record.start.cwd.clone())
        .collect::<BTreeSet<_>>();
    let mut paths = Vec::new();
    collect_rollout_files(sessions_root, &mut paths)?;
    paths.sort();

    let mut matches = BTreeMap::<String, Vec<(PathBuf, ParsedRollout)>>::new();
    let mut unobserved_cwd_threads = BTreeSet::new();
    let mut errors = Vec::new();
    let extraction_unix_ns = unix_time_ns();
    let run_start_with_skew = config
        .created_unix_ns
        .saturating_sub(60_u128 * 1_000_000_000);
    for path in paths {
        let header = parse_rollout_header(&path)?;
        if header.thread_id.is_empty() {
            continue;
        }
        if !expected.contains_key(&header.thread_id) {
            if !session_only_threads.contains(&header.thread_id)
                && observed_cwds.contains(&header.cwd)
                && header.started_unix_ns.is_some_and(|started| {
                    started >= run_start_with_skew && started <= extraction_unix_ns
                })
            {
                unobserved_cwd_threads.insert(header.thread_id);
            }
            continue;
        }
        let mut parsed = parse_rollout(&path)?;
        if config.condition == "direct" {
            parsed.normalize_direct_invocation_usage();
        }
        let thread_id = parsed.thread_id.clone();
        matches.entry(thread_id).or_default().push((path, parsed));
    }

    let mut rows = Vec::new();
    let mut missing_threads = Vec::new();
    for (thread_id, expectation) in &expected {
        let thread = &expectation.thread;
        let Some(candidates) = matches.get(thread_id) else {
            missing_threads.push(thread_id.clone());
            continue;
        };
        let expected_cwd = expectation.expected_cwd.as_ref();
        let (path, rollout, ambiguous) =
            select_rollout_candidate(candidates, thread, expected_cwd, config);
        if ambiguous {
            errors.push(format!(
                "multiple conflicting rollout files are equally valid for observed thread {thread_id}"
            ));
        }
        if expected_cwd != Some(&rollout.cwd) {
            errors.push(format!(
                "thread {thread_id} rollout cwd {} does not match captured invocation cwd {}",
                rollout.cwd.display(),
                expected_cwd
                    .map(|cwd| cwd.display().to_string())
                    .unwrap_or_else(|| "missing".to_string())
            ));
        }
        if config.condition == "direct"
            && rollout.invocation_count() != expectation.invocation_count
        {
            errors.push(format!(
                "thread {thread_id} rollout has {} task usage epoch(s) for {} captured direct invocation(s)",
                rollout.invocation_count(),
                expectation.invocation_count
            ));
        }
        let Some(usage) = rollout.usage else {
            errors.push(format!(
                "thread {thread_id} rollout has no final token usage"
            ));
            continue;
        };
        if usage != thread.usage
            && (!expectation.allow_rollout_supersession || !usage_contains(usage, thread.usage))
        {
            errors.push(format!(
                "thread {thread_id} rollout usage does not match captured provider usage"
            ));
        }
        if thread.primary && rollout.model != config.model {
            errors.push(format!(
                "primary thread {thread_id} model {} does not match configured {}",
                rollout.model, config.model
            ));
        }
        if thread.primary && rollout.effort != config.effort {
            errors.push(format!(
                "primary thread {thread_id} effort {} does not match configured {}",
                rollout.effort, config.effort
            ));
        }
        for (model, effort) in rollout.profiles.iter().filter(|_| thread.primary) {
            if model != &config.model || effort != &config.effort {
                errors.push(format!(
                    "thread {thread_id} task profile {model}/{effort} does not match configured {}/{}",
                    config.model, config.effort
                ));
            }
        }
        if !cli_versions_match(&config.real_codex_version, &rollout.cli_version) {
            errors.push(format!(
                "thread {thread_id} rollout CLI version {} does not match configured {}",
                rollout.cli_version, config.real_codex_version
            ));
        }
        rows.push(RolloutMetadata {
            thread_id: thread_id.clone(),
            cwd: rollout.cwd.clone(),
            model: rollout.model.clone(),
            effort: rollout.effort.clone(),
            cli_version: rollout.cli_version.clone(),
            usage,
            source_sha256: sha256_file(path)?,
            source_relative_path: path
                .strip_prefix(sessions_root)
                .unwrap_or(path)
                .to_path_buf(),
            primary: thread.primary,
            visible: thread.visible,
            descendant: !thread.primary,
        });
    }
    rows.sort_by(|left, right| left.thread_id.cmp(&right.thread_id));
    write_json_lines(&config.root.join("rollout-metadata.jsonl"), rows.iter())?;

    if !missing_threads.is_empty() {
        errors.push(format!(
            "missing rollout metadata for {} observed thread(s)",
            missing_threads.len()
        ));
    }
    if !unobserved_cwd_threads.is_empty() {
        errors.push(format!(
            "{} rollout thread(s) share an observed cwd but are absent from process capture",
            unobserved_cwd_threads.len()
        ));
    }
    let audit = RolloutAudit {
        observed_threads: expected.len(),
        matched_threads: rows.len(),
        session_only_threads: session_only_threads.into_iter().collect(),
        missing_threads,
        unobserved_cwd_threads: unobserved_cwd_threads.into_iter().collect(),
        errors,
    };
    write_json_atomic(&config.root.join("rollout-audit.json"), &audit)?;
    harden_artifact_permissions(&config.root)?;
    Ok(audit)
}

fn cli_versions_match(configured: &str, observed: &str) -> bool {
    configured == observed
        || version_identifier(configured)
            .zip(version_identifier(observed))
            .is_some_and(|(configured, observed)| configured == observed)
}

fn select_rollout_candidate<'a>(
    candidates: &'a [(PathBuf, ParsedRollout)],
    thread: &ThreadSummary,
    expected_cwd: Option<&PathBuf>,
    config: &CaptureConfig,
) -> (&'a PathBuf, &'a ParsedRollout, bool) {
    let score = |rollout: &ParsedRollout| {
        let mut value = 0_u8;
        if rollout.usage == Some(thread.usage) {
            value = value.saturating_add(16);
        }
        if expected_cwd == Some(&rollout.cwd) {
            value = value.saturating_add(8);
        }
        if cli_versions_match(&config.real_codex_version, &rollout.cli_version) {
            value = value.saturating_add(4);
        }
        if thread.primary && rollout.model == config.model {
            value = value.saturating_add(2);
        }
        if thread.primary && rollout.effort == config.effort {
            value = value.saturating_add(1);
        }
        value
    };
    let best_score = candidates
        .iter()
        .map(|(_, rollout)| score(rollout))
        .max()
        .expect("an observed rollout thread has at least one candidate");
    let newest = candidates
        .iter()
        .filter(|(_, rollout)| score(rollout) == best_score)
        .map(|(_, rollout)| rollout.started_unix_ns)
        .max()
        .unwrap_or(None);
    let finalists = candidates
        .iter()
        .filter(|(_, rollout)| score(rollout) == best_score && rollout.started_unix_ns == newest)
        .collect::<Vec<_>>();
    let selected = finalists
        .iter()
        .max_by(|left, right| left.0.cmp(&right.0))
        .expect("a best rollout candidate exists");
    let ambiguous = finalists.iter().any(|candidate| {
        candidate.1.cwd != selected.1.cwd
            || candidate.1.model != selected.1.model
            || candidate.1.effort != selected.1.effort
            || candidate.1.cli_version != selected.1.cli_version
            || candidate.1.usage != selected.1.usage
    });
    (&selected.0, &selected.1, ambiguous)
}

fn version_identifier(value: &str) -> Option<&str> {
    value.split_whitespace().find_map(|word| {
        let word = word.trim_matches(|character: char| {
            !character.is_ascii_alphanumeric() && character != '.' && character != '-'
        });
        (word
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_digit())
            && word.contains('.'))
        .then_some(word)
    })
}

fn collect_rollout_files(current: &Path, paths: &mut Vec<PathBuf>) -> ObserverResult<()> {
    if !current.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_rollout_files(&entry.path(), paths)?;
        } else if file_type.is_file()
            && entry.path().extension().and_then(OsStr::to_str) == Some("jsonl")
        {
            paths.push(entry.path());
        }
    }
    Ok(())
}

fn parse_rollout(path: &Path) -> ObserverResult<ParsedRollout> {
    let file = File::open(path)?;
    let mut parsed = ParsedRollout::default();
    let mut task_open = false;
    let mut task_usage = None;
    for line in BufReader::new(file).lines() {
        let line = line?;
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        match value.get("type").and_then(Value::as_str) {
            Some("session_meta") => {
                let Some(payload) = value.get("payload") else {
                    continue;
                };
                if let Some(thread_id) = payload.get("id").and_then(Value::as_str) {
                    parsed.thread_id = thread_id.to_string();
                }
                if let Some(cwd) = payload.get("cwd").and_then(Value::as_str) {
                    parsed.cwd = PathBuf::from(cwd);
                }
                if let Some(version) = payload.get("cli_version").and_then(Value::as_str) {
                    parsed.cli_version = version.to_string();
                }
                parsed.started_unix_ns = value
                    .get("timestamp")
                    .and_then(Value::as_str)
                    .and_then(parse_rfc3339_unix_ns);
            }
            Some("turn_context") => {
                let Some(payload) = value.get("payload") else {
                    continue;
                };
                if let Some(cwd) = payload.get("cwd").and_then(Value::as_str) {
                    parsed.cwd = PathBuf::from(cwd);
                }
                if let Some(model) = payload.get("model").and_then(Value::as_str) {
                    parsed.model = model.to_string();
                }
                if let Some(effort) = payload.get("effort").and_then(Value::as_str) {
                    parsed.effort = effort.to_string();
                }
                if !parsed.model.is_empty() && !parsed.effort.is_empty() {
                    parsed
                        .profiles
                        .insert((parsed.model.clone(), parsed.effort.clone()));
                }
            }
            Some("event_msg")
                if value
                    .get("payload")
                    .and_then(|payload| payload.get("type"))
                    .and_then(Value::as_str)
                    == Some("task_started") =>
            {
                if task_open && let Some(usage) = task_usage.take() {
                    parsed.task_usages.push(usage);
                }
                task_open = true;
                task_usage = None;
            }
            Some("event_msg")
                if value
                    .get("payload")
                    .and_then(|payload| payload.get("type"))
                    .and_then(Value::as_str)
                    == Some("token_count") =>
            {
                let usage = value
                    .get("payload")
                    .and_then(|payload| payload.get("info"))
                    .and_then(|info| info.get("total_token_usage"))
                    .and_then(usage_from_object);
                parsed.usage = usage;
                if task_open {
                    task_usage = usage;
                }
            }
            Some("event_msg")
                if value
                    .get("payload")
                    .and_then(|payload| payload.get("type"))
                    .and_then(Value::as_str)
                    == Some("task_complete") =>
            {
                if task_open && let Some(usage) = task_usage.take() {
                    parsed.task_usages.push(usage);
                }
                task_open = false;
            }
            _ => {}
        }
    }
    if task_open && let Some(usage) = task_usage {
        parsed.task_usages.push(usage);
    }
    Ok(parsed)
}

fn parse_rollout_header(path: &Path) -> ObserverResult<ParsedRollout> {
    let file = File::open(path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) != Some("session_meta") {
            continue;
        }
        let Some(payload) = value.get("payload") else {
            return Ok(ParsedRollout::default());
        };
        return Ok(ParsedRollout {
            thread_id: payload
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            cwd: payload
                .get("cwd")
                .and_then(Value::as_str)
                .map(PathBuf::from)
                .unwrap_or_default(),
            started_unix_ns: value
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(parse_rfc3339_unix_ns),
            cli_version: payload
                .get("cli_version")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            ..ParsedRollout::default()
        });
    }
    Ok(ParsedRollout::default())
}

fn parse_rfc3339_unix_ns(timestamp: &str) -> Option<u128> {
    let bytes = timestamp.as_bytes();
    if bytes.len() < 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return None;
    }
    let year = parse_decimal(bytes.get(0..4)?)? as i64;
    let month = parse_decimal(bytes.get(5..7)?)? as i64;
    let day = parse_decimal(bytes.get(8..10)?)? as i64;
    let hour = parse_decimal(bytes.get(11..13)?)? as i64;
    let minute = parse_decimal(bytes.get(14..16)?)? as i64;
    let second = parse_decimal(bytes.get(17..19)?)? as i64;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }

    let mut cursor = 19;
    let mut nanoseconds = 0_u128;
    if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
        let fraction_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        let fraction = bytes.get(fraction_start..cursor)?;
        let digits = fraction.len().min(9);
        nanoseconds = parse_decimal(fraction.get(..digits)?)? as u128;
        for _ in digits..9 {
            nanoseconds *= 10;
        }
    }

    let timezone_offset_seconds = match bytes.get(cursor) {
        Some(b'Z') => 0_i64,
        Some(sign @ (b'+' | b'-')) => {
            let timezone = bytes.get(cursor + 1..)?;
            if timezone.len() < 5 || timezone.get(2) != Some(&b':') {
                return None;
            }
            let hours = parse_decimal(timezone.get(0..2)?)? as i64;
            let minutes = parse_decimal(timezone.get(3..5)?)? as i64;
            let offset = hours.checked_mul(3600)?.checked_add(minutes * 60)?;
            if *sign == b'+' { offset } else { -offset }
        }
        _ => return None,
    };
    let days = days_since_unix_epoch(year, month, day)?;
    let seconds = days
        .checked_mul(86_400)?
        .checked_add(hour * 3600 + minute * 60 + second)?
        .checked_sub(timezone_offset_seconds)?;
    let seconds = u128::try_from(seconds).ok()?;
    Some(seconds * 1_000_000_000 + nanoseconds)
}

fn parse_decimal(bytes: &[u8]) -> Option<u64> {
    bytes.iter().try_fold(0_u64, |value, byte| {
        if !byte.is_ascii_digit() {
            return None;
        }
        value.checked_mul(10)?.checked_add(u64::from(byte - b'0'))
    })
}

fn days_since_unix_epoch(year: i64, month: i64, day: i64) -> Option<i64> {
    let adjusted_year = year - i64::from(month <= 2);
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Some(days)
}

pub fn config_from_environment() -> ObserverResult<CaptureConfig> {
    let path = std::env::var_os(CONFIG_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| ObserverError::new(format!("{CONFIG_ENV} is required")))?;
    CaptureConfig::load(&path)
}

pub fn pass_through_codex(config: &CaptureConfig, args: &[OsString]) -> ObserverError {
    if let Ok(marker) = std::env::var(PRIMARY_MARKER_ENV)
        && marker != config.primary_invocation_marker
    {
        return ObserverError::new(
            "primary invocation marker does not match observer configuration",
        );
    }
    let invocation_id = invocation_id();
    let record = CodexPassthroughRecord {
        invocation_id: invocation_id.clone(),
        argv: std::env::args_os()
            .map(|argument| encode_argument(&argument))
            .collect(),
        cwd: match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(error) => return error.into(),
        },
        pid: std::process::id(),
        parent_pid: process_parent_id(),
        process_group: process_group_id(),
        parent_invocation_id: std::env::var(PARENT_INVOCATION_ENV).ok(),
        primary_marker_present: std::env::var(PRIMARY_MARKER_ENV).is_ok(),
        role: std::env::var(ROLE_ENV).ok(),
        informational: is_informational_codex_invocation(args),
        start_monotonic_ns: monotonic_time_ns(),
        start_unix_ns: unix_time_ns(),
        real_executable: config.real_codex.clone(),
        real_executable_sha256: config.real_codex_sha256.clone(),
    };
    // The append helper secures only the touched file, so passthrough capture remains constant work
    // as the observation tree grows.
    if let Err(error) = append_json_line(&config.root.join("codex-passthrough.jsonl"), &record) {
        return error;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = Command::new(&config.real_codex)
            .args(args)
            .env(PARENT_INVOCATION_ENV, invocation_id)
            .env_remove(PRIMARY_MARKER_ENV)
            .exec();
        ObserverError::new(format!(
            "failed to execute {}: {error}",
            config.real_codex.display()
        ))
    }
    #[cfg(not(unix))]
    {
        match Command::new(&config.real_codex)
            .args(args)
            .env(PARENT_INVOCATION_ENV, invocation_id)
            .env_remove(PRIMARY_MARKER_ENV)
            .status()
        {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(error) => ObserverError::new(format!(
                "failed to execute {}: {error}",
                config.real_codex.display()
            )),
        }
    }
}

fn is_informational_codex_invocation(args: &[OsString]) -> bool {
    let options = args
        .iter()
        .take_while(|argument| argument.as_os_str() != "--");
    if options.into_iter().any(|argument| {
        matches!(
            argument.to_str(),
            Some("--version" | "-V" | "--help" | "-h")
        )
    }) || args
        .first()
        .is_some_and(|argument| argument == "help" || argument == "doctor")
    {
        return true;
    }
    matches!(
        (
            args.first().and_then(|arg| arg.to_str()),
            args.get(1).and_then(|arg| arg.to_str())
        ),
        (
            Some("app-server"),
            Some("generate-ts" | "generate-json-schema")
        )
    )
}

pub fn pass_through(real_executable: &Path, args: &[OsString]) -> ObserverError {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = Command::new(real_executable).args(args).exec();
        ObserverError::new(format!(
            "failed to execute {}: {error}",
            real_executable.display()
        ))
    }
    #[cfg(not(unix))]
    {
        match Command::new(real_executable).args(args).status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(error) => ObserverError::new(format!(
                "failed to execute {}: {error}",
                real_executable.display()
            )),
        }
    }
}

pub fn exit_like_child(outcome: ProxyOutcome) -> ! {
    if let Some(code) = outcome.status.code() {
        std::process::exit(code);
    }
    #[cfg(unix)]
    if let Some(signal) = exit_signal(&outcome.status) {
        unsafe {
            libc::signal(signal, libc::SIG_DFL);
            libc::raise(signal);
        }
        std::process::exit(128 + signal);
    }
    std::process::exit(1)
}

#[cfg(test)]
mod tests {
    use super::{
        CapturedUsage, assistant_text_completes_work_leaf_directive,
        cumulative_usage_contains_last_response,
    };

    #[test]
    fn streamed_directive_replay_waits_for_complete_edit_blocks() {
        let partial = "working\n@work-leaf edit update value\n*** Begin Patch\n";
        assert!(!assistant_text_completes_work_leaf_directive(partial));
        assert!(assistant_text_completes_work_leaf_directive(&format!(
            "{partial}*** End Patch\n@work-leaf end\n"
        )));
        assert!(assistant_text_completes_work_leaf_directive(
            "need context\n@work-leaf read src/lib.rs\n"
        ));
    }

    #[test]
    fn cumulative_increase_without_last_usage_does_not_prove_a_response() {
        let previous = CapturedUsage {
            input_tokens: 100,
            cached_input_tokens: 80,
            output_tokens: 10,
            reasoning_output_tokens: 5,
        };
        let total = CapturedUsage {
            input_tokens: 200,
            cached_input_tokens: 160,
            output_tokens: 20,
            reasoning_output_tokens: 10,
        };

        assert!(!cumulative_usage_contains_last_response(
            Some(previous),
            total,
            None,
        ));
    }
}
