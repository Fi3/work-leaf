use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, AgentTokenUsage, ChatMessage, MessageRole, PromptPolicy,
};
use crate::agent_runtime::{
    configure_agent_child_process, configure_persistent_agent_child_process,
};

const REMOVED_CODEX_CHILD_ENV: &[&str] = &[
    "CODEX_THREAD_ID",
    "CODEX_CI",
    "CODEX_MANAGED_BY_NPM",
    "CODEX_MANAGED_PACKAGE_ROOT",
    "WORK_LEAF_CODEX_TRACE",
    "WORK_LEAF_COMMAND_TMPDIR",
    "WORK_LEAF_CONTEXT_BUNDLE_DIR",
    WORK_LEAF_CODEX_SDK_PYTHON_ENV,
];
const CODEX_STARTUP_RETRY_DELAYS_MS: &[u64] = &[1_000, 2_500, 5_000, 10_000];
const CODEX_SDK_SIDECAR: &str = include_str!("codex_sdk_sidecar.py");
const WORK_LEAF_CODEX_SDK_PYTHON_ENV: &str = "WORK_LEAF_CODEX_SDK_PYTHON";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodexTransport {
    Exec,
    Sdk,
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
    pub transport: CodexTransport,
    pub sdk_python: Option<PathBuf>,
}

impl CodexCommandConfig {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            binary: PathBuf::from("codex"),
            project_dir,
            model: None,
            sandbox: SandboxMode::ReadOnly,
            transport: CodexTransport::Exec,
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

    pub fn with_sdk_transport(mut self) -> Self {
        self.transport = CodexTransport::Sdk;
        self
    }

    pub fn with_exec_transport(mut self) -> Self {
        self.transport = CodexTransport::Exec;
        self
    }

    pub fn with_sdk_python(mut self, python: impl Into<PathBuf>) -> Self {
        self.sdk_python = Some(python.into());
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodexInvocation {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub stdin: String,
}

#[derive(Debug)]
pub struct CodexBackend {
    config: CodexCommandConfig,
    policy: PromptPolicy,
    state: Arc<Mutex<CodexBackendState>>,
    operation_condvar: Arc<Condvar>,
    process_start_mutex: Arc<Mutex<()>>,
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
            process_start_mutex: self.process_start_mutex.clone(),
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
    active_processes: BTreeMap<AgentId, u32>,
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
        let mut interrupt_requested = false;
        loop {
            let inbound = self.next_message(request_id)?;
            if let Some(event) = inbound.event {
                match event {
                    CodexSdkEvent::Status { text } => {
                        let event = AgentStreamEvent::Status(text);
                        sink(event.clone());
                        if !interrupt_requested
                            && should_interrupt
                                .as_deref_mut()
                                .is_some_and(|detector| detector(&event))
                        {
                            interrupt_requested = true;
                            self.interrupt(turn.agent_id, shutdown)?;
                        }
                    }
                    CodexSdkEvent::Message { text } => {
                        streamed_messages.push(text.clone());
                        let event = AgentStreamEvent::AgentMessage(text);
                        sink(event.clone());
                        if !interrupt_requested
                            && should_interrupt
                                .as_deref_mut()
                                .is_some_and(|detector| detector(&event))
                        {
                            interrupt_requested = true;
                            self.interrupt(turn.agent_id, shutdown)?;
                        }
                    }
                    CodexSdkEvent::Usage { usage } => {
                        let event = AgentStreamEvent::Usage(usage);
                        sink(event.clone());
                        if !interrupt_requested
                            && should_interrupt
                                .as_deref_mut()
                                .is_some_and(|detector| detector(&event))
                        {
                            interrupt_requested = true;
                            self.interrupt(turn.agent_id, shutdown)?;
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

        let mut command = Command::new(&python);
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
        let mut child = command.spawn()?;
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

fn is_codex_slash_command(prompt: &str) -> bool {
    let mut chars = prompt.trim_start().chars();
    matches!(chars.next(), Some('/')) && chars.next().is_some_and(|ch| !ch.is_whitespace())
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
            process_start_mutex: Arc::new(Mutex::new(())),
            sdk,
            shutdown: AgentShutdownHandle::default(),
            lifecycle: Arc::new(()),
        }
    }

    pub fn build_launch_invocation(&self, request: &AgentLaunch) -> CodexInvocation {
        let stdin = self
            .policy
            .inject(&request.id, &request.feature, &request.prompt);
        self.exec_invocation(stdin, self.sandbox_for_agent(&request.id))
    }

    pub fn build_send_invocation(
        &self,
        agent_id: &AgentId,
        prompt: &str,
    ) -> Result<CodexInvocation, AgentError> {
        let (has_session, feature, resume_id) = {
            let state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            let feature = state
                .sessions
                .get(agent_id)
                .map(|session| session.feature.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let has_session = state.sessions.contains_key(agent_id);
            let resume_id = state
                .thread_ids
                .get(agent_id)
                .cloned()
                .unwrap_or_else(|| agent_id.as_str().to_string());
            (has_session, feature, resume_id)
        };
        let stdin = if has_session {
            prompt.to_string()
        } else {
            self.policy.inject(agent_id, &feature, prompt)
        };
        self.resume_invocation(&resume_id, stdin, self.sandbox_for_agent(agent_id))
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

    pub fn record_launch_output(
        &mut self,
        request: AgentLaunch,
        output: String,
    ) -> Result<AgentSession, AgentError> {
        let parsed = parse_codex_output(&output);
        {
            let mut state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            if let Some(thread_id) = parsed.thread_id {
                state.thread_ids.insert(request.id.clone(), thread_id);
            }
            if let Some(usage) = parsed.usage {
                record_usage(&mut state, &request.id, usage);
            }
        }
        self.record_launch_reply(request, parsed.agent_reply.unwrap_or(output))
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
            SandboxMode::WorkspaceWrite
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

    fn exec_invocation(&self, stdin: String, sandbox: SandboxMode) -> CodexInvocation {
        let mut args = vec![
            "--disable".to_string(),
            "apps".to_string(),
            "--cd".to_string(),
            self.config.project_dir.display().to_string(),
            "--sandbox".to_string(),
            sandbox.as_codex_arg().to_string(),
            "--ask-for-approval".to_string(),
            "never".to_string(),
        ];
        if let Some(model) = &self.config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        args.push("exec".to_string());
        args.push("--color".to_string());
        args.push("never".to_string());
        args.push("--json".to_string());
        args.push("-".to_string());
        CodexInvocation {
            program: self.config.binary.clone(),
            args,
            stdin,
        }
    }

    fn resume_invocation(
        &self,
        resume_id: &str,
        stdin: String,
        sandbox: SandboxMode,
    ) -> Result<CodexInvocation, AgentError> {
        let mut args = vec![
            "--disable".to_string(),
            "apps".to_string(),
            "--cd".to_string(),
            self.config.project_dir.display().to_string(),
            "--sandbox".to_string(),
            sandbox.as_codex_arg().to_string(),
            "--ask-for-approval".to_string(),
            "never".to_string(),
        ];
        if let Some(model) = &self.config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        args.extend([
            "exec".to_string(),
            "resume".to_string(),
            "--json".to_string(),
            resume_id.to_string(),
            "-".to_string(),
        ]);
        Ok(CodexInvocation {
            program: self.config.binary.clone(),
            args,
            stdin,
        })
    }

    fn run_invocation_streaming(
        &self,
        invocation: &CodexInvocation,
        agent_id: Option<&AgentId>,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<String, AgentError> {
        for attempt in 1..=CODEX_STARTUP_RETRY_DELAYS_MS.len() + 1 {
            match self.run_invocation_streaming_once(invocation, agent_id, sink) {
                Ok(output) => return Ok(output),
                Err(error)
                    if attempt <= CODEX_STARTUP_RETRY_DELAYS_MS.len()
                        && is_retryable_codex_startup_failure(&error) =>
                {
                    let child_path = codex_child_path(
                        env::var_os("PATH").as_deref(),
                        invocation.program.parent(),
                    );
                    trace_codex_child(
                        agent_id,
                        "startup-retry",
                        invocation,
                        child_path.as_deref(),
                        &format!("attempt={attempt}"),
                    );
                    thread::sleep(Duration::from_millis(
                        CODEX_STARTUP_RETRY_DELAYS_MS[attempt - 1],
                    ));
                }
                Err(error) => return Err(error),
            }
        }
        unreachable!("Codex startup attempt loop always returns");
    }

    fn run_invocation_streaming_once(
        &self,
        invocation: &CodexInvocation,
        agent_id: Option<&AgentId>,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<String, AgentError> {
        let child_path =
            codex_child_path(env::var_os("PATH").as_deref(), invocation.program.parent());
        trace_codex_child(
            agent_id,
            "start-lock-wait",
            invocation,
            child_path.as_deref(),
            "",
        );
        let mut process_start_guard = Some(
            self.process_start_mutex
                .lock()
                .expect("codex process start mutex poisoned"),
        );
        trace_codex_child(
            agent_id,
            "start-lock-acquired",
            invocation,
            child_path.as_deref(),
            "",
        );
        let mut command = codex_process_command(invocation);
        if let Some(path) = &child_path {
            command.env("PATH", path);
        }
        for name in REMOVED_CODEX_CHILD_ENV {
            command.env_remove(name);
        }
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        configure_agent_child_process(&mut command);
        trace_codex_child(agent_id, "spawn", invocation, child_path.as_deref(), "");
        let mut child = command.spawn()?;
        let child_pid = child.id();
        trace_codex_child(
            agent_id,
            "spawned",
            invocation,
            child_path.as_deref(),
            &format!("pid={child_pid}"),
        );
        let _process_guard = self.shutdown.register(child.id());
        if let Some(agent_id) = agent_id {
            self.state
                .lock()
                .expect("codex backend state mutex poisoned")
                .active_processes
                .insert(agent_id.clone(), child_pid);
        }

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(invocation.stdin.as_bytes())?;
        }
        drop(child.stdin.take());

        let stderr_reader = child.stderr.take().map(|mut stderr| {
            thread::spawn(move || {
                let mut text = String::new();
                let _ = stderr.read_to_string(&mut text);
                text
            })
        });

        let mut stdout_text = String::new();
        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines() {
                let line = line?;
                if let Some(event) = codex_stream_event(&line) {
                    sink(event);
                }
                if process_start_guard.is_some() && line.contains(r#""type":"turn.started""#) {
                    trace_codex_child(
                        agent_id,
                        "startup-ready",
                        invocation,
                        child_path.as_deref(),
                        "event=turn.started",
                    );
                    drop(process_start_guard.take());
                }
                stdout_text.push_str(&line);
                stdout_text.push('\n');
            }
        }
        drop(process_start_guard.take());

        let status = child.wait()?;
        trace_codex_child(
            agent_id,
            "exit",
            invocation,
            child_path.as_deref(),
            &format!("status={:?}", status.code()),
        );
        if let Some(agent_id) = agent_id {
            let mut state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            if state
                .active_processes
                .get(agent_id)
                .is_some_and(|pid| *pid == child_pid)
            {
                state.active_processes.remove(agent_id);
            }
        }
        let stderr = stderr_reader
            .and_then(|reader| reader.join().ok())
            .unwrap_or_default();

        if status.success() {
            Ok(stdout_text.trim().to_string())
        } else {
            Err(AgentError::ProcessFailed {
                program: invocation.program.clone(),
                status: status.code(),
                stderr: process_failure_output(stderr, &stdout_text),
            })
        }
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
            if self.config.transport == CodexTransport::Sdk {
                self.sdk.shutdown();
            } else {
                self.shutdown.shutdown();
            }
        }
    }
}

impl AgentBackend for CodexBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if self.config.transport == CodexTransport::Sdk {
            return self.launch_streaming(request, &mut |_| {});
        }
        let _operation_guard = self.acquire_agent_operation(&request.id);
        let invocation = self.build_launch_invocation(&request);
        let output = self.run_invocation_streaming(&invocation, Some(&request.id), &mut |_| {})?;
        self.record_launch_output(request, output)
    }

    fn session(&self, agent_id: &AgentId) -> Option<AgentSession> {
        CodexBackend::session(self, agent_id)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        if self.config.transport == CodexTransport::Sdk {
            return self.send_streaming(agent_id, prompt, &mut |_| {});
        }
        let _operation_guard = self.acquire_agent_operation(agent_id);
        let invocation = self.build_send_invocation(agent_id, prompt)?;
        let output = self.run_invocation_streaming(&invocation, Some(agent_id), &mut |_| {})?;
        let parsed = parse_codex_output(&output);
        if let Some(usage) = parsed.usage {
            let mut state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            record_usage(&mut state, agent_id, usage);
        }
        let reply = parsed.agent_reply.unwrap_or(output);
        self.record_send_reply(agent_id, prompt, reply)
    }

    fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.shutdown.clone()
    }

    fn interrupt(&mut self, agent_id: &AgentId) -> Result<(), AgentError> {
        if self.config.transport == CodexTransport::Sdk {
            return self.sdk.interrupt(agent_id, &self.shutdown);
        }
        let pid = self
            .state
            .lock()
            .expect("codex backend state mutex poisoned")
            .active_processes
            .get(agent_id)
            .copied();
        if let Some(pid) = pid {
            let _ = self.shutdown.terminate_process(pid);
        }
        Ok(())
    }

    fn shutdown(&mut self) {
        if self.config.transport == CodexTransport::Sdk {
            self.sdk.shutdown();
        } else {
            self.shutdown.shutdown();
        }
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        if self.config.transport == CodexTransport::Sdk {
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
            return self.record_launch_reply(request, output.reply);
        }
        let _operation_guard = self.acquire_agent_operation(&request.id);
        let invocation = self.build_launch_invocation(&request);
        let output = self.run_invocation_streaming(&invocation, Some(&request.id), sink)?;
        self.record_launch_output(request, output)
    }

    fn launch_streaming_interruptible(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<AgentSession, AgentError> {
        if self.config.transport == CodexTransport::Sdk {
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
            return self.record_launch_reply(request, output.reply);
        }
        let mut stream = |event: AgentStreamEvent| {
            sink(event.clone());
            let _ = should_interrupt(&event);
        };
        self.launch_streaming(request, &mut stream)
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        if self.config.transport == CodexTransport::Sdk {
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
            return self.record_send_reply(agent_id, prompt, output.reply);
        }
        let _operation_guard = self.acquire_agent_operation(agent_id);
        let invocation = self.build_send_invocation(agent_id, prompt)?;
        let output = self.run_invocation_streaming(&invocation, Some(agent_id), sink)?;
        let parsed = parse_codex_output(&output);
        if let Some(usage) = parsed.usage {
            let mut state = self
                .state
                .lock()
                .expect("codex backend state mutex poisoned");
            record_usage(&mut state, agent_id, usage);
        }
        let reply = parsed.agent_reply.unwrap_or(output);
        self.record_send_reply(agent_id, prompt, reply)
    }

    fn send_streaming_interruptible(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        if self.config.transport == CodexTransport::Sdk {
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
            return self.record_send_reply(agent_id, prompt, output.reply);
        }
        let mut stream = |event: AgentStreamEvent| {
            sink(event.clone());
            let _ = should_interrupt(&event);
        };
        self.send_streaming(agent_id, prompt, &mut stream)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ParsedCodexOutput {
    thread_id: Option<String>,
    agent_reply: Option<String>,
    usage: Option<AgentTokenUsage>,
}

fn parse_codex_output(output: &str) -> ParsedCodexOutput {
    let mut parsed = ParsedCodexOutput::default();
    for line in output.lines() {
        let compact = compact_json_line(line);
        if compact.contains(r#""type":"thread.started""#) {
            parsed.thread_id = json_string_field(&compact, "thread_id").or(parsed.thread_id);
        }
        if compact.contains(r#""type":"item.completed""#)
            && compact.contains(r#""type":"agent_message""#)
            && let Some(text) = json_string_field(&compact, "text")
        {
            append_agent_reply(&mut parsed.agent_reply, text);
        }
        if compact.contains(r#""type":"turn.completed""#) {
            let usage = codex_usage_from_compact_line(&compact);
            parsed.usage = Some(parsed.usage.unwrap_or_default().combine(usage));
        }
    }
    parsed
}

fn record_usage(state: &mut CodexBackendState, agent_id: &AgentId, usage: AgentTokenUsage) {
    let current = state.usage.get(agent_id).copied().unwrap_or_default();
    state.usage.insert(agent_id.clone(), current.combine(usage));
}

fn append_agent_reply(reply: &mut Option<String>, text: String) {
    match reply {
        Some(existing) if !existing.is_empty() && !text.is_empty() => {
            existing.push_str("\n\n");
            existing.push_str(&text);
        }
        Some(existing) => existing.push_str(&text),
        None => *reply = Some(text),
    }
}

fn codex_stream_event(line: &str) -> Option<AgentStreamEvent> {
    let compact = compact_json_line(line);
    if compact.contains(r#""type":"item.completed""#)
        && compact.contains(r#""type":"agent_message""#)
    {
        return json_string_field(&compact, "text").map(AgentStreamEvent::AgentMessage);
    }
    if compact.contains(r#""type":"error""#) {
        return json_string_field(&compact, "message").map(AgentStreamEvent::Error);
    }
    if compact.contains(r#""type":"thread.started""#) {
        return json_string_field(&compact, "thread_id")
            .map(|thread_id| AgentStreamEvent::Status(format!("Codex session {thread_id}")));
    }
    if compact.contains(r#""type":"turn.started""#) {
        return Some(AgentStreamEvent::Status("Codex is working".to_string()));
    }
    if compact.contains(r#""type":"turn.completed""#) {
        return Some(AgentStreamEvent::Usage(codex_usage_from_compact_line(
            &compact,
        )));
    }
    None
}

fn codex_usage_from_compact_line(compact: &str) -> AgentTokenUsage {
    AgentTokenUsage {
        input_tokens: json_u64_field(compact, "input_tokens").unwrap_or_default(),
        cached_input_tokens: json_u64_field(compact, "cached_input_tokens").unwrap_or_default(),
        output_tokens: json_u64_field(compact, "output_tokens").unwrap_or_default(),
        reasoning_output_tokens: json_u64_field(compact, "reasoning_output_tokens")
            .unwrap_or_default(),
    }
}

fn process_failure_output(stderr: String, stdout: &str) -> String {
    let stderr = stderr.trim();
    let stdout = stdout.trim();
    if !stderr.is_empty() && !stdout.is_empty() {
        return format!("{stderr}\nstdout:\n{stdout}");
    }
    if !stderr.is_empty() {
        return stderr.to_string();
    }
    if stdout.is_empty() {
        "process exited without stderr or stdout".to_string()
    } else {
        format!("stdout:\n{stdout}")
    }
}

fn is_retryable_codex_startup_failure(error: &AgentError) -> bool {
    let AgentError::ProcessFailed { stderr, .. } = error else {
        return false;
    };
    let compact = compact_json_line(stderr);
    if compact.contains(r#""type":"thread.started""#)
        || compact.contains(r#""type":"turn.started""#)
    {
        return false;
    }
    stderr.contains("failed to initialize in-process app-server client")
        || stderr.contains("could not create PATH aliases")
}

fn codex_child_path(path: Option<&OsStr>, program_dir: Option<&Path>) -> Option<OsString> {
    let path = path?;
    let Some(program_dir) = program_dir else {
        return Some(path.to_os_string());
    };
    let mut entries = vec![program_dir.to_path_buf()];
    entries.extend(env::split_paths(path).filter(|entry| entry.as_path() != program_dir));
    env::join_paths(entries)
        .ok()
        .or_else(|| Some(path.to_os_string()))
}

fn codex_process_command(invocation: &CodexInvocation) -> Command {
    let mut command = Command::new(&invocation.program);
    command.args(&invocation.args);
    command
}

fn trace_codex_child(
    agent_id: Option<&AgentId>,
    phase: &str,
    invocation: &CodexInvocation,
    child_path: Option<&OsStr>,
    detail: &str,
) {
    if env::var_os("WORK_LEAF_CODEX_TRACE").is_none() {
        return;
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let agent = agent_id.map(AgentId::as_str).unwrap_or("-");
    eprintln!(
        "work-leaf codex trace ts_ms={timestamp} phase={phase} agent={agent} program={} args={} path={} env={} {detail}",
        invocation.program.display(),
        shell_words_for_trace(&invocation.args),
        path_entries_for_trace(child_path),
        env_for_trace()
    );
}

fn shell_words_for_trace(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || "-_./:=+".contains(ch))
            {
                arg.clone()
            } else {
                format!("{arg:?}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn path_entries_for_trace(path: Option<&OsStr>) -> String {
    let Some(path) = path else {
        return "[]".to_string();
    };
    let entries = env::split_paths(path)
        .take(5)
        .map(|entry| entry.display().to_string())
        .collect::<Vec<_>>();
    format!("{entries:?}")
}

fn env_for_trace() -> String {
    let names = [
        "HOME",
        "XDG_RUNTIME_DIR",
        "TMPDIR",
        "CODEX_CI",
        "CODEX_MANAGED_BY_NPM",
        "CODEX_MANAGED_PACKAGE_ROOT",
        "CODEX_THREAD_ID",
        "WORK_LEAF_CODEX_TRACE",
        "WORK_LEAF_COMMAND_TMPDIR",
    ];
    let values = names
        .iter()
        .map(|name| {
            let value = if REMOVED_CODEX_CHILD_ENV.contains(name) {
                "<removed>".to_string()
            } else {
                env::var(name).unwrap_or_else(|_| "-".to_string())
            };
            format!("{name}={value:?}")
        })
        .collect::<Vec<_>>();
    format!("{values:?}")
}

fn compact_json_line(line: &str) -> String {
    let mut compact = String::with_capacity(line.len());
    let mut in_string = false;
    let mut escaped = false;

    for ch in line.chars() {
        if in_string {
            compact.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            compact.push(ch);
        } else if !ch.is_whitespace() {
            compact.push(ch);
        }
    }

    compact
}

fn json_string_field(line: &str, field: &str) -> Option<String> {
    let needle = format!(r#""{field}":"#);
    let start = line.find(&needle)? + needle.len();
    let mut chars = line[start..].chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            match ch {
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                '/' => value.push('/'),
                'b' => value.push('\u{0008}'),
                'f' => value.push('\u{000c}'),
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                other => value.push(other),
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(value);
        } else {
            value.push(ch);
        }
    }
    None
}

fn json_u64_field(line: &str, field: &str) -> Option<u64> {
    let needle = format!(r#""{field}":"#);
    let start = line.find(&needle)? + needle.len();
    let digits = line[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_parsing_accepts_json_field_whitespace() {
        let output = [
            r#"{ "type": "thread.started", "thread_id": "thread-spaced" }"#,
            r#"{ "type": "item.completed", "item": { "type": "agent_message", "text": "first reply" } }"#,
            r#"{ "type": "item.completed", "item": { "type": "agent_message", "text": "second reply" } }"#,
            r#"{ "type": "turn.completed", "usage": { "input_tokens": 10, "cached_input_tokens": 4, "output_tokens": 3, "reasoning_output_tokens": 2 } }"#,
            r#"{ "type": "turn.completed", "usage": { "input_tokens": 5, "cached_input_tokens": 1, "output_tokens": 2, "reasoning_output_tokens": 1 } }"#,
        ]
        .join("\n");

        let parsed = parse_codex_output(&output);

        assert_eq!(parsed.thread_id.as_deref(), Some("thread-spaced"));
        assert_eq!(
            parsed.agent_reply.as_deref(),
            Some("first reply\n\nsecond reply")
        );
        assert_eq!(
            parsed.usage,
            Some(AgentTokenUsage {
                input_tokens: 15,
                cached_input_tokens: 5,
                output_tokens: 5,
                reasoning_output_tokens: 3
            })
        );
        assert_eq!(
            codex_stream_event(
                r#"{ "type": "item.completed", "item": { "type": "agent_message", "text": "streamed reply" } }"#
            ),
            Some(AgentStreamEvent::AgentMessage("streamed reply".to_string()))
        );
        assert_eq!(
            codex_stream_event(r#"{ "type": "error", "message": "streamed error" }"#),
            Some(AgentStreamEvent::Error("streamed error".to_string()))
        );
        assert_eq!(
            codex_stream_event(r#"{ "type": "turn.started" }"#),
            Some(AgentStreamEvent::Status("Codex is working".to_string()))
        );
        assert_eq!(
            codex_stream_event(
                r#"{ "type": "turn.completed", "usage": { "input_tokens": 2, "cached_input_tokens": 1, "output_tokens": 3, "reasoning_output_tokens": 4 } }"#
            ),
            Some(AgentStreamEvent::Usage(AgentTokenUsage {
                input_tokens: 2,
                cached_input_tokens: 1,
                output_tokens: 3,
                reasoning_output_tokens: 4
            }))
        );
    }

    #[test]
    fn codex_child_path_prepends_invoked_program_directory() {
        let path = env::join_paths([
            PathBuf::from("/home/user/.codex/tmp/arg0/codex-arg0abc"),
            PathBuf::from("/usr/bin"),
        ])
        .unwrap();

        let child_path = codex_child_path(
            Some(path.as_os_str()),
            Some(Path::new("/tmp/work-leaf-codex-wrapper")),
        )
        .unwrap();
        let entries = env::split_paths(&child_path).collect::<Vec<_>>();

        assert_eq!(
            entries,
            vec![
                PathBuf::from("/tmp/work-leaf-codex-wrapper"),
                PathBuf::from("/home/user/.codex/tmp/arg0/codex-arg0abc"),
                PathBuf::from("/usr/bin"),
            ]
        );
    }

    #[test]
    fn codex_child_path_does_not_duplicate_invoked_program_directory() {
        let path = env::join_paths([
            PathBuf::from("/tmp/work-leaf-codex-wrapper"),
            PathBuf::from("/usr/bin"),
        ])
        .unwrap();

        let child_path = codex_child_path(
            Some(path.as_os_str()),
            Some(Path::new("/tmp/work-leaf-codex-wrapper")),
        )
        .unwrap();
        let entries = env::split_paths(&child_path).collect::<Vec<_>>();

        assert_eq!(
            entries,
            vec![
                PathBuf::from("/tmp/work-leaf-codex-wrapper"),
                PathBuf::from("/usr/bin")
            ]
        );
    }

    #[test]
    fn codex_process_command_uses_configured_program_directly() {
        let invocation = CodexInvocation {
            program: PathBuf::from("/usr/bin/codex"),
            args: vec!["exec".to_string(), "--json".to_string(), "-".to_string()],
            stdin: String::new(),
        };

        let command = codex_process_command(&invocation);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(command.get_program(), OsStr::new("/usr/bin/codex"));
        assert_eq!(args, vec!["exec", "--json", "-"]);
    }

    #[test]
    fn known_session_resume_invocation_uses_raw_follow_up_stdin() {
        let mut backend = CodexBackend::new(
            CodexCommandConfig::new(PathBuf::from("/repo")),
            PromptPolicy::for_restricted_agents(),
        );
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        backend
            .record_launch_output(
                AgentLaunch::new(agent_id.clone(), AgentKind::Codex, "user-agent", "start"),
                r#"{"type":"thread.started","thread_id":"thread-user-1"}"#.to_string(),
            )
            .expect("test launch output records the thread id");

        let invocation = backend
            .build_send_invocation(&agent_id, "continue")
            .expect("resume invocation is built");

        assert_eq!(invocation.stdin, "continue");
        assert!(invocation.args.iter().any(|arg| arg == "thread-user-1"));
    }
}
