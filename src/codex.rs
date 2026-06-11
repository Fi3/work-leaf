use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, AgentTokenUsage, ChatMessage, MessageRole, PromptPolicy,
};
use crate::agent_runtime::configure_persistent_agent_child_process;

const REMOVED_CODEX_CHILD_ENV: &[&str] = &[
    "CODEX_THREAD_ID",
    "CODEX_CI",
    "CODEX_MANAGED_BY_NPM",
    "CODEX_MANAGED_PACKAGE_ROOT",
    "WORK_LEAF_CODEX_TRACE",
    "WORK_LEAF_COMMAND_TMPDIR",
    "WORK_LEAF_CONTEXT_BUNDLE_DIR",
    "WORK_LEAF_CODEX_LINEARIZE_SANDBOX",
    WORK_LEAF_CODEX_SDK_PYTHON_ENV,
];
const CODEX_SDK_SIDECAR: &str = include_str!("codex_sdk_sidecar.py");
const CODEX_SDK_SPAWN_RETRY_DELAYS_MS: &[u64] = &[0, 25, 50, 100];
const WORK_LEAF_CODEX_SDK_PYTHON_ENV: &str = "WORK_LEAF_CODEX_SDK_PYTHON";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl SandboxMode {
    fn as_codex_arg(&self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::DangerFullAccess => "danger-full-access",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodexCommandConfig {
    pub binary: PathBuf,
    pub project_dir: PathBuf,
    pub model: Option<String>,
    pub sandbox: SandboxMode,
    pub linearize_sandbox: SandboxMode,
    pub sdk_python: Option<PathBuf>,
}

impl CodexCommandConfig {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            binary: PathBuf::from("codex"),
            project_dir,
            model: None,
            sandbox: SandboxMode::ReadOnly,
            linearize_sandbox: SandboxMode::WorkspaceWrite,
            sdk_python: None,
        }
    }

    pub fn with_binary(mut self, binary: impl Into<PathBuf>) -> Self {
        self.binary = binary.into();
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_sandbox(mut self, sandbox: SandboxMode) -> Self {
        self.sandbox = sandbox;
        self
    }

    pub fn with_linearize_sandbox(mut self, sandbox: SandboxMode) -> Self {
        self.linearize_sandbox = sandbox;
        self
    }

    pub fn with_sdk_python(mut self, python: impl Into<PathBuf>) -> Self {
        self.sdk_python = Some(python.into());
        self
    }
}

#[derive(Debug)]
pub struct CodexBackend {
    config: CodexCommandConfig,
    policy: PromptPolicy,
    state: Arc<Mutex<CodexBackendState>>,
    operation_condvar: Arc<Condvar>,
    sdk: Arc<CodexSdkSidecar>,
    shutdown: AgentShutdownHandle,
    lifecycle: Arc<()>,
}

impl Clone for CodexBackend {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            policy: self.policy.clone(),
            state: self.state.clone(),
            operation_condvar: self.operation_condvar.clone(),
            sdk: self.sdk.clone(),
            shutdown: self.shutdown.clone(),
            lifecycle: self.lifecycle.clone(),
        }
    }
}

#[derive(Debug, Default)]
struct CodexBackendState {
    sessions: BTreeMap<AgentId, AgentSession>,
    thread_ids: BTreeMap<AgentId, String>,
    usage: BTreeMap<AgentId, AgentTokenUsage>,
    active_agent_operations: BTreeSet<AgentId>,
}

#[derive(Debug)]
struct CodexSdkSidecar {
    config: CodexCommandConfig,
    process: Mutex<Option<CodexSdkProcess>>,
    router: Arc<(Mutex<CodexSdkRouterState>, Condvar)>,
    next_request_id: AtomicU64,
}

#[derive(Debug)]
struct CodexSdkProcess {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    _guard: crate::agent_runtime::ActiveAgentProcessGuard,
}

#[derive(Debug, Default)]
struct CodexSdkRouterState {
    queues: BTreeMap<u64, VecDeque<CodexSdkInbound>>,
    closed_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct CodexSdkInbound {
    id: Option<u64>,
    ok: Option<bool>,
    error: Option<String>,
    thread_id: Option<String>,
    reply: Option<String>,
    usage: Option<AgentTokenUsage>,
    event: Option<CodexSdkEvent>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
enum CodexSdkEvent {
    #[serde(rename = "status")]
    Status { text: String },
    #[serde(rename = "message")]
    Message { text: String },
    #[serde(rename = "usage")]
    Usage { usage: AgentTokenUsage },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CodexSdkTurnOutput {
    thread_id: String,
    reply: String,
    usage: Option<AgentTokenUsage>,
}

#[derive(Clone, Debug)]
struct CodexSdkTurnRequest<'a> {
    op: &'a str,
    agent_id: &'a AgentId,
    prompt: &'a str,
    thread_id: Option<&'a str>,
    sandbox: SandboxMode,
}

#[derive(Debug, Serialize)]
struct CodexSdkConfig {
    codex_bin: String,
    cwd: String,
    client_version: String,
}

impl CodexSdkSidecar {
    fn new(config: CodexCommandConfig) -> Self {
        Self {
            config,
            process: Mutex::new(None),
            router: Arc::new((Mutex::new(CodexSdkRouterState::default()), Condvar::new())),
            next_request_id: AtomicU64::new(1),
        }
    }

    fn request_streaming(
        &self,
        turn: CodexSdkTurnRequest<'_>,
        shutdown: &AgentShutdownHandle,
        sink: &mut dyn FnMut(AgentStreamEvent),
        mut should_interrupt: Option<&mut dyn FnMut(&AgentStreamEvent) -> bool>,
    ) -> Result<CodexSdkTurnOutput, AgentError> {
        self.ensure_started(shutdown)?;
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let request = json!({
            "id": request_id,
            "op": turn.op,
            "agent_id": turn.agent_id.as_str(),
            "thread_id": turn.thread_id,
            "prompt": turn.prompt,
            "cwd": self.config.project_dir.display().to_string(),
            "model": self.config.model.as_deref(),
            "sandbox": turn.sandbox.as_codex_arg(),
        });
        self.write_request_with_restart(request_id, &request, shutdown)?;

        let mut output = CodexSdkTurnOutput::default();
        let mut streamed_messages = Vec::new();
        loop {
            let inbound = self.next_message(request_id)?;
            if let Some(event) = inbound.event {
                match event {
                    CodexSdkEvent::Status { text } => {
                        if output.thread_id.is_empty()
                            && let Some(thread_id) = codex_sdk_session_id_from_status(&text)
                        {
                            output.thread_id = thread_id.to_string();
                        }
                        let event = AgentStreamEvent::Status(text);
                        sink(event.clone());
                        if should_interrupt
                            .as_deref_mut()
                            .is_some_and(|detector| detector(&event))
                        {
                            self.request_interrupt(turn.agent_id, shutdown)?;
                            self.unregister_request(request_id);
                            output.reply = streamed_messages.join("\n\n");
                            return Ok(output);
                        }
                    }
                    CodexSdkEvent::Message { text } => {
                        streamed_messages.push(text.clone());
                        let event = AgentStreamEvent::AgentMessage(text);
                        sink(event.clone());
                        if should_interrupt
                            .as_deref_mut()
                            .is_some_and(|detector| detector(&event))
                        {
                            self.request_interrupt(turn.agent_id, shutdown)?;
                            self.unregister_request(request_id);
                            output.reply = streamed_messages.join("\n\n");
                            return Ok(output);
                        }
                    }
                    CodexSdkEvent::Usage { usage } => {
                        output.usage = Some(usage);
                        let event = AgentStreamEvent::Usage(usage);
                        sink(event.clone());
                        if should_interrupt
                            .as_deref_mut()
                            .is_some_and(|detector| detector(&event))
                        {
                            self.request_interrupt(turn.agent_id, shutdown)?;
                            self.unregister_request(request_id);
                            output.reply = streamed_messages.join("\n\n");
                            return Ok(output);
                        }
                    }
                }
                continue;
            }
            match inbound.ok {
                Some(true) => {
                    self.unregister_request(request_id);
                    output.thread_id = inbound.thread_id.unwrap_or_default();
                    output.reply = if streamed_messages.is_empty() {
                        inbound.reply.unwrap_or_default()
                    } else {
                        streamed_messages.join("\n\n")
                    };
                    output.usage = inbound.usage;
                    return Ok(output);
                }
                Some(false) => {
                    self.unregister_request(request_id);
                    return Err(AgentError::ProcessFailed {
                        program: self.python_program(),
                        status: None,
                        stderr: inbound
                            .error
                            .unwrap_or_else(|| "Codex SDK sidecar request failed".to_string()),
                    });
                }
                None => {}
            }
        }
    }

    fn interrupt(
        &self,
        agent_id: &AgentId,
        shutdown: &AgentShutdownHandle,
    ) -> Result<(), AgentError> {
        self.ensure_started(shutdown)?;
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let request = json!({
            "id": request_id,
            "op": "interrupt",
            "agent_id": agent_id.as_str(),
        });
        self.write_request_with_restart(request_id, &request, shutdown)?;
        loop {
            let inbound = self.next_message(request_id)?;
            match inbound.ok {
                Some(true) => {
                    self.unregister_request(request_id);
                    return Ok(());
                }
                Some(false) => {
                    self.unregister_request(request_id);
                    return Err(AgentError::ProcessFailed {
                        program: self.python_program(),
                        status: None,
                        stderr: inbound
                            .error
                            .unwrap_or_else(|| "Codex SDK sidecar interrupt failed".to_string()),
                    });
                }
                None => {}
            }
        }
    }

    fn request_interrupt(
        &self,
        agent_id: &AgentId,
        shutdown: &AgentShutdownHandle,
    ) -> Result<(), AgentError> {
        self.ensure_started(shutdown)?;
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let request = json!({
            "id": request_id,
            "op": "interrupt",
            "agent_id": agent_id.as_str(),
        });
        self.write_request(&request)
    }

    fn ensure_started(&self, shutdown: &AgentShutdownHandle) -> Result<(), AgentError> {
        let mut process_guard = self
            .process
            .lock()
            .expect("codex sdk sidecar process mutex poisoned");
        if let Some(process) = process_guard.as_mut() {
            match process.child.try_wait() {
                Ok(Some(status)) => {
                    trace_codex_sdk(format_args!(
                        "sidecar exited before reuse with status {status:?}; restarting"
                    ));
                    *process_guard = None;
                    self.reset_router_after_restart();
                }
                Ok(None) => return Ok(()),
                Err(error) => {
                    return Err(AgentError::ProcessFailed {
                        program: self.python_program(),
                        status: None,
                        stderr: format!("failed to inspect Codex SDK sidecar: {error}"),
                    });
                }
            }
        }

        let python = self.python_program();
        let config = CodexSdkConfig {
            codex_bin: self.config.binary.display().to_string(),
            cwd: self.config.project_dir.display().to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let config_json =
            serde_json::to_string(&config).map_err(|error| AgentError::ProcessFailed {
                program: python.clone(),
                status: None,
                stderr: format!("failed to serialize Codex SDK sidecar config: {error}"),
            })?;

        let mut child = self.spawn_sidecar_process(&python, &config_json)?;
        let child_pid = child.id();
        trace_codex_sdk(format_args!("spawned sidecar pid {child_pid}"));
        let guard = shutdown.register_single_process(child_pid);
        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or_else(|| {
            AgentError::ProcessFailed {
                program: python.clone(),
                status: None,
                stderr: "Codex SDK sidecar did not expose stdin".to_string(),
            }
        })?));
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgentError::ProcessFailed {
                program: python.clone(),
                status: None,
                stderr: "Codex SDK sidecar did not expose stdout".to_string(),
            })?;
        let stderr = child.stderr.take();
        self.spawn_reader(stdout);
        self.spawn_stderr_drain(stderr);
        *process_guard = Some(CodexSdkProcess {
            child,
            stdin,
            _guard: guard,
        });
        drop(process_guard);
        self.wait_until_ready()
    }

    fn spawn_reader(&self, stdout: impl Read + Send + 'static) {
        let router = self.router.clone();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) => match serde_json::from_str::<CodexSdkInbound>(&line) {
                        Ok(inbound) => route_sdk_inbound(&router, inbound),
                        Err(error) => close_sdk_router(
                            &router,
                            format!("invalid Codex SDK sidecar JSON: {error}: {line}"),
                        ),
                    },
                    Err(error) => {
                        close_sdk_router(&router, format!("Codex SDK sidecar read failed: {error}"))
                    }
                }
            }
            close_sdk_router(&router, "Codex SDK sidecar closed stdout".to_string());
        });
    }

    fn spawn_sidecar_process(&self, python: &Path, config_json: &str) -> Result<Child, AgentError> {
        let mut last_busy_error = None;
        for delay_ms in CODEX_SDK_SPAWN_RETRY_DELAYS_MS {
            if *delay_ms > 0 {
                thread::sleep(Duration::from_millis(*delay_ms));
            }
            let mut command = Command::new(python);
            command
                .arg("-u")
                .arg("-c")
                .arg(CODEX_SDK_SIDECAR)
                .env("WORK_LEAF_CODEX_SDK_CONFIG", config_json)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            for name in REMOVED_CODEX_CHILD_ENV {
                command.env_remove(name);
            }
            configure_persistent_agent_child_process(&mut command);
            match command.spawn() {
                Ok(child) => return Ok(child),
                Err(error) if is_text_file_busy(&error) => {
                    trace_codex_sdk(format_args!(
                        "sidecar executable was busy during spawn; retrying"
                    ));
                    last_busy_error = Some(error);
                }
                Err(error) => return Err(AgentError::Io(error)),
            }
        }
        Err(AgentError::Io(last_busy_error.expect(
            "retry loop records the text-busy spawn error before exhausting retries",
        )))
    }

    fn spawn_stderr_drain(&self, stderr: Option<impl Read + Send + 'static>) {
        if let Some(mut stderr) = stderr {
            thread::spawn(move || {
                let mut text = String::new();
                let _ = stderr.read_to_string(&mut text);
                if env::var_os("WORK_LEAF_CODEX_TRACE").is_some() && !text.trim().is_empty() {
                    eprintln!("work-leaf codex sdk sidecar stderr:\n{}", text.trim());
                }
            });
        }
    }

    fn wait_until_ready(&self) -> Result<(), AgentError> {
        let inbound = self.next_message(0)?;
        if inbound.ok == Some(true) {
            self.unregister_request(0);
            Ok(())
        } else {
            Err(AgentError::ProcessFailed {
                program: self.python_program(),
                status: None,
                stderr: inbound
                    .error
                    .unwrap_or_else(|| "Codex SDK sidecar did not report ready".to_string()),
            })
        }
    }

    fn write_request(&self, request: &Value) -> Result<(), AgentError> {
        let text = serde_json::to_string(request).map_err(|error| AgentError::ProcessFailed {
            program: self.python_program(),
            status: None,
            stderr: format!("failed to serialize Codex SDK sidecar request: {error}"),
        })?;
        let stdin = {
            let process = self
                .process
                .lock()
                .expect("codex sdk sidecar process mutex poisoned");
            process
                .as_ref()
                .map(|process| process.stdin.clone())
                .ok_or_else(|| AgentError::ProcessFailed {
                    program: self.python_program(),
                    status: None,
                    stderr: "Codex SDK sidecar is not running".to_string(),
                })?
        };
        let mut stdin = stdin
            .lock()
            .expect("codex sdk sidecar stdin mutex poisoned");
        stdin.write_all(text.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
        Ok(())
    }

    fn write_request_with_restart(
        &self,
        request_id: u64,
        request: &Value,
        shutdown: &AgentShutdownHandle,
    ) -> Result<(), AgentError> {
        self.register_request(request_id);
        match self.write_request(request) {
            Ok(()) => Ok(()),
            Err(error) if is_broken_pipe(&error) => {
                self.unregister_request(request_id);
                trace_codex_sdk(format_args!(
                    "sidecar write failed with broken pipe; restarting once"
                ));
                self.stop_sidecar_process();
                self.reset_router_after_restart();
                self.ensure_started(shutdown)?;
                self.register_request(request_id);
                if let Err(error) = self.write_request(request) {
                    self.unregister_request(request_id);
                    return Err(error);
                }
                Ok(())
            }
            Err(error) => {
                self.unregister_request(request_id);
                Err(error)
            }
        }
    }

    fn stop_sidecar_process(&self) {
        if let Some(mut process) = self
            .process
            .lock()
            .expect("codex sdk sidecar process mutex poisoned")
            .take()
        {
            let pid = process.child.id();
            trace_codex_sdk(format_args!("stopping sidecar pid {pid}"));
            let _ = process.child.kill();
            let _ = process.child.wait();
        }
    }

    fn reset_router_after_restart(&self) {
        let (lock, condvar) = &*self.router;
        let mut state = lock.lock().expect("codex sdk router mutex poisoned");
        state.closed_error = None;
        state.queues.clear();
        condvar.notify_all();
    }

    fn shutdown(&self) {
        if self
            .process
            .lock()
            .expect("codex sdk sidecar process mutex poisoned")
            .is_none()
        {
            return;
        }
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let request = json!({
            "id": request_id,
            "op": "shutdown",
        });
        let write_result = self.write_request(&request);
        if let Err(error) = &write_result {
            trace_codex_sdk(format_args!("sidecar shutdown request failed: {error}"));
        }
        if let Some(mut process) = self
            .process
            .lock()
            .expect("codex sdk sidecar process mutex poisoned")
            .take()
        {
            let pid = process.child.id();
            if write_result.is_ok()
                && wait_for_child_exit(&mut process.child, Duration::from_secs(2))
            {
                trace_codex_sdk(format_args!("sidecar pid {pid} exited after shutdown"));
            } else {
                trace_codex_sdk(format_args!("killing sidecar pid {pid} during shutdown"));
                let _ = process.child.kill();
                let _ = process.child.wait();
            }
        }
        self.reset_router_after_restart();
    }

    fn register_request(&self, request_id: u64) {
        let (lock, _) = &*self.router;
        lock.lock()
            .expect("codex sdk router mutex poisoned")
            .queues
            .entry(request_id)
            .or_default();
    }

    fn unregister_request(&self, request_id: u64) {
        let (lock, _) = &*self.router;
        lock.lock()
            .expect("codex sdk router mutex poisoned")
            .queues
            .remove(&request_id);
    }

    fn next_message(&self, request_id: u64) -> Result<CodexSdkInbound, AgentError> {
        let (lock, condvar) = &*self.router;
        let mut state = lock.lock().expect("codex sdk router mutex poisoned");
        loop {
            if let Some(message) = state
                .queues
                .get_mut(&request_id)
                .and_then(VecDeque::pop_front)
            {
                return Ok(message);
            }
            if let Some(error) = &state.closed_error {
                return Err(AgentError::ProcessFailed {
                    program: self.python_program(),
                    status: None,
                    stderr: error.clone(),
                });
            }
            state = condvar
                .wait(state)
                .expect("codex sdk router mutex poisoned");
        }
    }

    fn python_program(&self) -> PathBuf {
        self.config
            .sdk_python
            .clone()
            .or_else(|| env::var_os(WORK_LEAF_CODEX_SDK_PYTHON_ENV).map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("python3"))
    }
}

impl Drop for CodexSdkSidecar {
    fn drop(&mut self) {
        if let Some(mut process) = self
            .process
            .lock()
            .expect("codex sdk sidecar process mutex poisoned")
            .take()
        {
            let _ = process.child.kill();
            let _ = process.child.wait();
        }
    }
}

fn route_sdk_inbound(
    router: &Arc<(Mutex<CodexSdkRouterState>, Condvar)>,
    inbound: CodexSdkInbound,
) {
    let request_id = inbound.id.unwrap_or(0);
    let (lock, condvar) = &**router;
    let mut state = lock.lock().expect("codex sdk router mutex poisoned");
    if request_id != 0 && !state.queues.contains_key(&request_id) {
        return;
    }
    state
        .queues
        .entry(request_id)
        .or_default()
        .push_back(inbound);
    condvar.notify_all();
}

fn close_sdk_router(router: &Arc<(Mutex<CodexSdkRouterState>, Condvar)>, error: String) {
    let (lock, condvar) = &**router;
    let mut state = lock.lock().expect("codex sdk router mutex poisoned");
    if state.closed_error.is_none() {
        state.closed_error = Some(error);
    }
    condvar.notify_all();
}

fn is_broken_pipe(error: &AgentError) -> bool {
    matches!(error, AgentError::Io(error) if error.kind() == std::io::ErrorKind::BrokenPipe)
}

fn is_text_file_busy(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(26)
}

fn is_codex_slash_command(prompt: &str) -> bool {
    let mut chars = prompt.trim_start().chars();
    matches!(chars.next(), Some('/')) && chars.next().is_some_and(|ch| !ch.is_whitespace())
}

fn codex_sdk_session_id_from_status(text: &str) -> Option<&str> {
    text.strip_prefix("Codex session ")
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn is_linearize_agent(agent_id: &AgentId) -> bool {
    let value = agent_id.as_str();
    value == "linearize" || value.starts_with("linearize-")
}

fn trace_codex_sdk(message: std::fmt::Arguments<'_>) {
    if env::var_os("WORK_LEAF_CODEX_TRACE").is_some() {
        eprintln!("work-leaf codex sdk: {message}");
    }
}

fn wait_for_child_exit(child: &mut Child, timeout: Duration) -> bool {
    let start = SystemTime::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => {}
            Err(_) => return false,
        }
        if start.elapsed().unwrap_or_default() >= timeout {
            return false;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

impl CodexBackend {
    pub fn new(config: CodexCommandConfig, policy: PromptPolicy) -> Self {
        let sdk = Arc::new(CodexSdkSidecar::new(config.clone()));
        Self {
            config,
            policy,
            state: Arc::new(Mutex::new(CodexBackendState::default())),
            operation_condvar: Arc::new(Condvar::new()),
            sdk,
            shutdown: AgentShutdownHandle::default(),
            lifecycle: Arc::new(()),
        }
    }

    pub fn record_launch_reply(
        &mut self,
        request: AgentLaunch,
        reply: String,
    ) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, reply);
        self.state
            .lock()
            .expect("codex backend state mutex poisoned")
            .sessions
            .insert(session.id.clone(), session.clone());
        Ok(session)
    }

    pub fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        self.state
            .lock()
            .expect("codex backend state mutex poisoned")
            .sessions
            .get(agent_id)
            .cloned()
    }

    fn sandbox_for_agent(&self, agent_id: &AgentId) -> SandboxMode {
        if is_linearize_agent(agent_id) {
            self.config.linearize_sandbox.clone()
        } else {
            self.config.sandbox.clone()
        }
    }

    fn record_send_reply(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        reply: String,
    ) -> Result<ChatMessage, AgentError> {
        let message = ChatMessage::new(MessageRole::Agent, reply);
        let mut state = self
            .state
            .lock()
            .expect("codex backend state mutex poisoned");
        if let Some(session) = state.sessions.get_mut(agent_id) {
            session.push_message(MessageRole::User, prompt);
        } else {
            state.sessions.insert(
                agent_id.clone(),
                AgentSession::new(AgentLaunch::new(
                    agent_id.clone(),
                    AgentKind::Codex,
                    "unknown",
                    prompt,
                )),
            );
        }
        let session = state
            .sessions
            .get_mut(agent_id)
            .ok_or_else(|| AgentError::UnknownSession(agent_id.clone()))?;
        session.messages.push(message.clone());
        Ok(message)
    }

    fn acquire_agent_operation(&self, agent_id: &AgentId) -> AgentOperationGuard {
        let mut state = self
            .state
            .lock()
            .expect("codex backend state mutex poisoned");
        while state.active_agent_operations.contains(agent_id) {
            state = self
                .operation_condvar
                .wait(state)
                .expect("codex backend state mutex poisoned");
        }
        state.active_agent_operations.insert(agent_id.clone());
        AgentOperationGuard {
            state: self.state.clone(),
            operation_condvar: self.operation_condvar.clone(),
            agent_id: agent_id.clone(),
        }
    }
}

#[derive(Debug)]
struct AgentOperationGuard {
    state: Arc<Mutex<CodexBackendState>>,
    operation_condvar: Arc<Condvar>,
    agent_id: AgentId,
}

impl Drop for AgentOperationGuard {
    fn drop(&mut self) {
        let mut state = self
            .state
            .lock()
            .expect("codex backend state mutex poisoned");
        state.active_agent_operations.remove(&self.agent_id);
        self.operation_condvar.notify_all();
    }
}

impl Drop for CodexBackend {
    fn drop(&mut self) {
        let strong_count = Arc::strong_count(&self.lifecycle);
        trace_codex_sdk(format_args!(
            "backend drop with lifecycle owners={strong_count}"
        ));
        if strong_count == 1 {
            trace_codex_sdk(format_args!(
                "last backend owner dropped; shutting down agents"
            ));
            self.sdk.shutdown();
        }
    }
}

impl AgentBackend for CodexBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        self.launch_streaming(request, &mut |_| {})
    }

    fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        CodexBackend::session(self, agent_id)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.send_streaming(agent_id, prompt, &mut |_| {})
    }

    fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.shutdown.clone()
    }

    fn interrupt(&mut self, agent_id: &AgentId) -> Result<(), AgentError> {
        self.sdk.interrupt(agent_id, &self.shutdown)
    }

    fn shutdown(&mut self) {
        self.sdk.shutdown();
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        let _operation_guard = self.acquire_agent_operation(&request.id);
        let prompt = self
            .policy
            .inject(&request.id, &request.feature, &request.prompt);
        let output = self.sdk.request_streaming(
            CodexSdkTurnRequest {
                op: "launch",
                agent_id: &request.id,
                prompt: &prompt,
                thread_id: None,
                sandbox: self.sandbox_for_agent(&request.id),
            },
            &self.shutdown,
            sink,
            None,
        )?;
        {
            let mut state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            if !output.thread_id.is_empty() {
                state
                    .thread_ids
                    .insert(request.id.clone(), output.thread_id.clone());
            }
            if let Some(usage) = output.usage {
                record_usage(&mut state, &request.id, usage);
            }
        }
        self.record_launch_reply(request, output.reply)
    }

    fn launch_streaming_interruptible(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<AgentSession, AgentError> {
        let _operation_guard = self.acquire_agent_operation(&request.id);
        let prompt = self
            .policy
            .inject(&request.id, &request.feature, &request.prompt);
        let output = self.sdk.request_streaming(
            CodexSdkTurnRequest {
                op: "launch",
                agent_id: &request.id,
                prompt: &prompt,
                thread_id: None,
                sandbox: self.sandbox_for_agent(&request.id),
            },
            &self.shutdown,
            sink,
            Some(should_interrupt),
        )?;
        {
            let mut state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            if !output.thread_id.is_empty() {
                state
                    .thread_ids
                    .insert(request.id.clone(), output.thread_id.clone());
            }
            if let Some(usage) = output.usage {
                record_usage(&mut state, &request.id, usage);
            }
        }
        self.record_launch_reply(request, output.reply)
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        let _operation_guard = self.acquire_agent_operation(agent_id);
        let (has_session, feature, thread_id) = {
            let state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            let feature = state
                .sessions
                .get(agent_id)
                .map(|session| session.feature.clone())
                .unwrap_or_else(|| "unknown".to_string());
            (
                state.sessions.contains_key(agent_id),
                feature,
                state.thread_ids.get(agent_id).cloned(),
            )
        };
        let sdk_prompt = if has_session {
            prompt.to_string()
        } else {
            self.policy.inject(agent_id, &feature, prompt)
        };
        let op = if is_codex_slash_command(prompt) {
            "command"
        } else {
            "send"
        };
        let output = self.sdk.request_streaming(
            CodexSdkTurnRequest {
                op,
                agent_id,
                prompt: &sdk_prompt,
                thread_id: thread_id.as_deref(),
                sandbox: self.sandbox_for_agent(agent_id),
            },
            &self.shutdown,
            sink,
            None,
        )?;
        if !output.thread_id.is_empty() || output.usage.is_some() {
            let mut state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            if !output.thread_id.is_empty() {
                state
                    .thread_ids
                    .insert(agent_id.clone(), output.thread_id.clone());
            }
            if let Some(usage) = output.usage {
                record_usage(&mut state, agent_id, usage);
            }
        }
        self.record_send_reply(agent_id, prompt, output.reply)
    }

    fn send_streaming_interruptible(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        let _operation_guard = self.acquire_agent_operation(agent_id);
        let (has_session, feature, thread_id) = {
            let state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            let feature = state
                .sessions
                .get(agent_id)
                .map(|session| session.feature.clone())
                .unwrap_or_else(|| "unknown".to_string());
            (
                state.sessions.contains_key(agent_id),
                feature,
                state.thread_ids.get(agent_id).cloned(),
            )
        };
        let sdk_prompt = if has_session {
            prompt.to_string()
        } else {
            self.policy.inject(agent_id, &feature, prompt)
        };
        let op = if is_codex_slash_command(prompt) {
            "command"
        } else {
            "send"
        };
        let output = self.sdk.request_streaming(
            CodexSdkTurnRequest {
                op,
                agent_id,
                prompt: &sdk_prompt,
                thread_id: thread_id.as_deref(),
                sandbox: self.sandbox_for_agent(agent_id),
            },
            &self.shutdown,
            sink,
            Some(should_interrupt),
        )?;
        if !output.thread_id.is_empty() || output.usage.is_some() {
            let mut state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            if !output.thread_id.is_empty() {
                state
                    .thread_ids
                    .insert(agent_id.clone(), output.thread_id.clone());
            }
            if let Some(usage) = output.usage {
                record_usage(&mut state, agent_id, usage);
            }
        }
        self.record_send_reply(agent_id, prompt, output.reply)
    }
}

fn record_usage(state: &mut CodexBackendState, agent_id: &AgentId, usage: AgentTokenUsage) {
    let current = state.usage.get(agent_id).copied().unwrap_or_default();
    state.usage.insert(agent_id.clone(), current.combine(usage));
}
