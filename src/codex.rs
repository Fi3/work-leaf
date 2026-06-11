use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

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
];
const CODEX_APP_SERVER_SPAWN_RETRY_DELAYS_MS: &[u64] = &[0, 25, 50, 100];

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
}

impl CodexCommandConfig {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            binary: PathBuf::from("codex"),
            project_dir,
            model: None,
            sandbox: SandboxMode::ReadOnly,
            linearize_sandbox: SandboxMode::WorkspaceWrite,
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
}

#[derive(Debug)]
pub struct CodexBackend {
    config: CodexCommandConfig,
    policy: PromptPolicy,
    state: Arc<Mutex<CodexBackendState>>,
    operation_condvar: Arc<Condvar>,
    app_server: Arc<CodexAppServer>,
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
            app_server: self.app_server.clone(),
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
struct CodexAppServer {
    config: CodexCommandConfig,
    process: Mutex<Option<CodexAppServerProcess>>,
    router: Arc<(Mutex<CodexAppServerRouterState>, Condvar)>,
    loaded_threads: Mutex<BTreeSet<String>>,
    active_turns: Arc<Mutex<BTreeMap<AgentId, ActiveTurn>>>,
    next_request_id: AtomicU64,
}

#[derive(Debug)]
struct CodexAppServerProcess {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    _guard: crate::agent_runtime::ActiveAgentProcessGuard,
}

#[derive(Debug, Default)]
struct CodexAppServerRouterState {
    responses: BTreeMap<String, VecDeque<CodexJsonRpcResponse>>,
    turn_notifications: BTreeMap<String, VecDeque<CodexAppServerNotification>>,
    pending_turn_notifications: BTreeMap<String, VecDeque<CodexAppServerNotification>>,
    closed_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CodexJsonRpcResponse {
    result: Option<Value>,
    error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CodexAppServerNotification {
    method: String,
    params: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveTurn {
    thread_id: String,
    turn_id: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CodexTurnOutput {
    thread_id: String,
    reply: String,
    usage: Option<AgentTokenUsage>,
}

#[derive(Clone, Debug)]
struct CodexTurnRequest<'a> {
    agent_id: &'a AgentId,
    prompt: &'a str,
    thread_id: Option<&'a str>,
    sandbox: SandboxMode,
}

impl CodexAppServer {
    fn new(config: CodexCommandConfig) -> Self {
        Self {
            config,
            process: Mutex::new(None),
            router: Arc::new((
                Mutex::new(CodexAppServerRouterState::default()),
                Condvar::new(),
            )),
            loaded_threads: Mutex::new(BTreeSet::new()),
            active_turns: Arc::new(Mutex::new(BTreeMap::new())),
            next_request_id: AtomicU64::new(1),
        }
    }

    fn request_turn_streaming(
        &self,
        turn: CodexTurnRequest<'_>,
        shutdown: &AgentShutdownHandle,
        sink: &mut dyn FnMut(AgentStreamEvent),
        mut should_interrupt: Option<&mut dyn FnMut(&AgentStreamEvent) -> bool>,
    ) -> Result<CodexTurnOutput, AgentError> {
        self.ensure_started(shutdown)?;
        let thread_id = match turn.thread_id {
            Some(thread_id) => {
                self.ensure_thread_loaded(thread_id, &turn.sandbox, shutdown)?;
                thread_id.to_string()
            }
            None => {
                let thread_id = self.start_thread(&turn.sandbox, shutdown)?;
                sink(AgentStreamEvent::Status(format!(
                    "Codex session {thread_id}"
                )));
                thread_id
            }
        };
        let started = self.request_raw(
            "turn/start",
            Some(turn_start_params(
                &thread_id,
                turn.prompt,
                &self.config,
                &turn.sandbox,
            )),
            shutdown,
        )?;
        let turn_id = json_pointer_str(&started, &["turn", "id"])
            .ok_or_else(|| {
                self.protocol_error("turn/start response did not include turn.id".to_string())
            })?
            .to_string();
        self.register_turn(&turn_id);
        let _active_turn = ActiveTurnGuard::new(
            self.active_turns.clone(),
            turn.agent_id.clone(),
            thread_id.clone(),
            turn_id.clone(),
        );

        let mut output = CodexTurnOutput {
            thread_id: thread_id.clone(),
            reply: String::new(),
            usage: None,
        };
        let mut streamed_messages = Vec::new();
        loop {
            let notification = self.next_turn_notification(&turn_id)?;
            match notification.method.as_str() {
                "turn/started" => {
                    let event = AgentStreamEvent::Status("Codex is working".to_string());
                    sink(event.clone());
                    if should_interrupt
                        .as_deref_mut()
                        .is_some_and(|detector| detector(&event))
                    {
                        self.request_interrupt(turn.agent_id)?;
                        self.unregister_turn(&turn_id);
                        output.reply = streamed_messages.join("\n\n");
                        return Ok(output);
                    }
                }
                "item/started" | "item/completed" => {
                    if let Some(item) = notification.params.get("item") {
                        if let Some(status) = activity_from_item(&notification.method, item) {
                            let event = AgentStreamEvent::Status(status);
                            sink(event.clone());
                            if should_interrupt
                                .as_deref_mut()
                                .is_some_and(|detector| detector(&event))
                            {
                                self.request_interrupt(turn.agent_id)?;
                                self.unregister_turn(&turn_id);
                                output.reply = streamed_messages.join("\n\n");
                                return Ok(output);
                            }
                        }
                        if notification.method == "item/completed"
                            && let Some(message) = agent_message_text(item)
                        {
                            output.reply = message.to_string();
                            streamed_messages.push(message.to_string());
                            let event = AgentStreamEvent::AgentMessage(message.to_string());
                            sink(event.clone());
                            if should_interrupt
                                .as_deref_mut()
                                .is_some_and(|detector| detector(&event))
                            {
                                self.request_interrupt(turn.agent_id)?;
                                self.unregister_turn(&turn_id);
                                output.reply = streamed_messages.join("\n\n");
                                return Ok(output);
                            }
                        }
                    }
                }
                "thread/tokenUsage/updated" => {
                    if let Some(usage) = token_usage_from_params(&notification.params) {
                        output.usage = Some(usage);
                        let event = AgentStreamEvent::Usage(usage);
                        sink(event.clone());
                        if should_interrupt
                            .as_deref_mut()
                            .is_some_and(|detector| detector(&event))
                        {
                            self.request_interrupt(turn.agent_id)?;
                            self.unregister_turn(&turn_id);
                            output.reply = streamed_messages.join("\n\n");
                            return Ok(output);
                        }
                    }
                }
                "turn/completed" => {
                    self.unregister_turn(&turn_id);
                    if turn_failed(&notification.params) {
                        return Err(self.protocol_error(
                            turn_error_message(&notification.params)
                                .unwrap_or("Codex turn failed")
                                .to_string(),
                        ));
                    }
                    if !streamed_messages.is_empty() {
                        output.reply = streamed_messages.join("\n\n");
                    }
                    return Ok(output);
                }
                _ => {}
            }
        }
    }

    fn command(
        &self,
        agent_id: &AgentId,
        prompt: &str,
        thread_id: &str,
        sandbox: &SandboxMode,
        shutdown: &AgentShutdownHandle,
    ) -> Result<CodexTurnOutput, AgentError> {
        self.ensure_thread_loaded(thread_id, sandbox, shutdown)?;
        let (command, args) = slash_command_name(prompt);
        let Some(command) = command else {
            return self.request_turn_streaming(
                CodexTurnRequest {
                    agent_id,
                    prompt,
                    thread_id: Some(thread_id),
                    sandbox: sandbox.clone(),
                },
                shutdown,
                &mut |_| {},
                None,
            );
        };

        let mut reply_thread_id = thread_id.to_string();
        let reply = match command.as_str() {
            "status" | "st" => self.render_status(thread_id, sandbox, shutdown),
            "fork" => {
                let forked = self.request_raw(
                    "thread/fork",
                    Some(thread_resume_params(thread_id, &self.config, sandbox)),
                    shutdown,
                )?;
                reply_thread_id = json_pointer_str(&forked, &["thread", "id"])
                    .ok_or_else(|| {
                        self.protocol_error(
                            "thread/fork response did not include thread.id".to_string(),
                        )
                    })?
                    .to_string();
                self.loaded_threads
                    .lock()
                    .expect("loaded threads mutex poisoned")
                    .insert(reply_thread_id.clone());
                format!("Forked Codex thread {thread_id} -> {reply_thread_id}")
            }
            "compact" | "compress" => {
                self.request_raw(
                    "thread/compact/start",
                    Some(json!({ "threadId": thread_id })),
                    shutdown,
                )?;
                format!("Compaction started for Codex thread {thread_id}.")
            }
            "rename" | "name" => {
                if args.is_empty() {
                    "Usage: /rename <name>".to_string()
                } else {
                    self.request_raw(
                        "thread/name/set",
                        Some(json!({ "threadId": thread_id, "name": args })),
                        shutdown,
                    )?;
                    format!("Renamed Codex thread {thread_id} to {args}.")
                }
            }
            "archive" => {
                self.request_raw(
                    "thread/archive",
                    Some(json!({ "threadId": thread_id })),
                    shutdown,
                )?;
                format!("Archived Codex thread {thread_id}.")
            }
            "unarchive" => {
                self.request_raw(
                    "thread/unarchive",
                    Some(json!({ "threadId": thread_id })),
                    shutdown,
                )?;
                format!("Unarchived Codex thread {thread_id}.")
            }
            "help" | "?" => {
                "Supported app-server slash commands: /status, /fork, /compact, /rename <name>, /archive, /unarchive, /help.".to_string()
            }
            _ => format!(
                "Codex app-server command /{command} is not exposed by Work Leaf; no model request was sent."
            ),
        };

        Ok(CodexTurnOutput {
            thread_id: reply_thread_id,
            reply,
            usage: None,
        })
    }

    fn interrupt(
        &self,
        agent_id: &AgentId,
        shutdown: &AgentShutdownHandle,
    ) -> Result<(), AgentError> {
        let active = self
            .active_turns
            .lock()
            .expect("active turns mutex poisoned")
            .get(agent_id)
            .cloned();
        if let Some(active) = active {
            self.request_raw(
                "turn/interrupt",
                Some(json!({
                    "threadId": active.thread_id,
                    "turnId": active.turn_id,
                })),
                shutdown,
            )?;
        }
        Ok(())
    }

    fn request_interrupt(&self, agent_id: &AgentId) -> Result<(), AgentError> {
        let active = self
            .active_turns
            .lock()
            .expect("active turns mutex poisoned")
            .get(agent_id)
            .cloned();
        if let Some(active) = active {
            self.write_request_no_wait(
                "turn/interrupt",
                Some(json!({
                    "threadId": active.thread_id,
                    "turnId": active.turn_id,
                })),
            )?;
        }
        Ok(())
    }

    fn start_thread(
        &self,
        sandbox: &SandboxMode,
        shutdown: &AgentShutdownHandle,
    ) -> Result<String, AgentError> {
        let response = self.request_raw(
            "thread/start",
            Some(thread_start_params(&self.config, sandbox)),
            shutdown,
        )?;
        let thread_id = json_pointer_str(&response, &["thread", "id"])
            .ok_or_else(|| {
                self.protocol_error("thread/start response did not include thread.id".to_string())
            })?
            .to_string();
        self.loaded_threads
            .lock()
            .expect("loaded threads mutex poisoned")
            .insert(thread_id.clone());
        Ok(thread_id)
    }

    fn ensure_thread_loaded(
        &self,
        thread_id: &str,
        sandbox: &SandboxMode,
        shutdown: &AgentShutdownHandle,
    ) -> Result<(), AgentError> {
        if self
            .loaded_threads
            .lock()
            .expect("loaded threads mutex poisoned")
            .contains(thread_id)
        {
            return Ok(());
        }
        self.request_raw(
            "thread/resume",
            Some(thread_resume_params(thread_id, &self.config, sandbox)),
            shutdown,
        )?;
        self.loaded_threads
            .lock()
            .expect("loaded threads mutex poisoned")
            .insert(thread_id.to_string());
        Ok(())
    }

    fn render_status(
        &self,
        thread_id: &str,
        sandbox: &SandboxMode,
        shutdown: &AgentShutdownHandle,
    ) -> String {
        let thread = self
            .request_raw(
                "thread/read",
                Some(json!({ "threadId": thread_id, "includeTurns": false })),
                shutdown,
            )
            .ok()
            .and_then(|value| value.get("thread").cloned());
        let config = self
            .request_raw(
                "config/read",
                Some(json!({ "cwd": self.config.project_dir.display().to_string(), "includeLayers": false })),
                shutdown,
            )
            .ok()
            .and_then(|value| value.get("config").cloned());
        let account = self
            .request_raw("account/read", None, shutdown)
            .ok()
            .and_then(|value| value.get("account").cloned());

        let model = self
            .config
            .model
            .clone()
            .or_else(|| {
                config
                    .as_ref()
                    .and_then(|value| json_string(value, "model"))
            })
            .unwrap_or_else(|| "default".to_string());
        let thread_cwd = thread
            .as_ref()
            .and_then(|value| json_string(value, "cwd"))
            .unwrap_or_else(|| self.config.project_dir.display().to_string());
        let account_text = account
            .as_ref()
            .and_then(|value| json_string(value, "type").or_else(|| json_string(value, "kind")))
            .unwrap_or_else(|| "unknown".to_string());
        let thread_status = thread
            .as_ref()
            .and_then(|value| value.get("status"))
            .map(display_json_value)
            .unwrap_or_else(|| "unknown".to_string());
        let context_window = config
            .as_ref()
            .and_then(|value| {
                value
                    .get("modelContextWindow")
                    .or_else(|| value.get("model_context_window"))
            })
            .and_then(Value::as_u64);

        let mut lines = vec![
            "OpenAI Codex app-server status".to_string(),
            format!("Model: {model}"),
            format!("Directory: {thread_cwd}"),
            format!(
                "Permissions: sandbox {}, approval never",
                sandbox.as_codex_arg()
            ),
            format!("Account: {account_text}"),
            format!("Session: {thread_id}"),
        ];
        if let Some(value) = context_window {
            lines.push(format!("Context window: {value} tokens"));
        }
        lines.push(format!("Thread status: {thread_status}"));
        lines.join("\n")
    }

    fn request_raw(
        &self,
        method: &str,
        params: Option<Value>,
        shutdown: &AgentShutdownHandle,
    ) -> Result<Value, AgentError> {
        self.ensure_started(shutdown)?;
        self.request_raw_started(method, params)
    }

    fn request_raw_started(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, AgentError> {
        let request_id = self
            .next_request_id
            .fetch_add(1, Ordering::Relaxed)
            .to_string();
        self.register_response(&request_id);
        match self.write_request(&request_id, method, params) {
            Ok(()) => {}
            Err(error) if is_broken_pipe(&error) => {
                self.unregister_response(&request_id);
                return Err(error);
            }
            Err(error) => {
                self.unregister_response(&request_id);
                return Err(error);
            }
        }
        self.next_response(&request_id)
    }

    fn write_request_no_wait(&self, method: &str, params: Option<Value>) -> Result<(), AgentError> {
        let request_id = self
            .next_request_id
            .fetch_add(1, Ordering::Relaxed)
            .to_string();
        self.write_request(&request_id, method, params)
    }

    fn write_request(
        &self,
        request_id: &str,
        method: &str,
        params: Option<Value>,
    ) -> Result<(), AgentError> {
        let mut message = json!({
            "id": request_id,
            "method": method,
        });
        if let Some(params) = params {
            message["params"] = params;
        }
        self.write_message(&message)
    }

    fn write_notification(&self, method: &str, params: Option<Value>) -> Result<(), AgentError> {
        let mut message = json!({ "method": method });
        if let Some(params) = params {
            message["params"] = params;
        }
        self.write_message(&message)
    }

    fn write_message(&self, message: &Value) -> Result<(), AgentError> {
        let text = serde_json::to_string(message).map_err(|error| {
            self.protocol_error(format!(
                "failed to serialize Codex app-server request: {error}"
            ))
        })?;
        let stdin = {
            let process = self
                .process
                .lock()
                .expect("codex app-server process mutex poisoned");
            process
                .as_ref()
                .map(|process| process.stdin.clone())
                .ok_or_else(|| self.protocol_error("Codex app-server is not running".to_string()))?
        };
        write_json_line(&stdin, &text).map_err(AgentError::Io)
    }

    fn ensure_started(&self, shutdown: &AgentShutdownHandle) -> Result<(), AgentError> {
        let mut process_guard = self
            .process
            .lock()
            .expect("codex app-server process mutex poisoned");
        if let Some(process) = process_guard.as_mut() {
            match process.child.try_wait() {
                Ok(Some(status)) => {
                    trace_codex_app_server(format_args!(
                        "app-server exited before reuse with status {status:?}; restarting"
                    ));
                    *process_guard = None;
                    self.reset_after_restart();
                }
                Ok(None) => return Ok(()),
                Err(error) => {
                    return Err(
                        self.protocol_error(format!("failed to inspect Codex app-server: {error}"))
                    );
                }
            }
        }

        let mut child = self.spawn_app_server_process()?;
        let child_pid = child.id();
        trace_codex_app_server(format_args!("spawned app-server pid {child_pid}"));
        let guard = shutdown.register_single_process(child_pid);
        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or_else(|| {
            self.protocol_error("Codex app-server did not expose stdin".to_string())
        })?));
        let stdout = child.stdout.take().ok_or_else(|| {
            self.protocol_error("Codex app-server did not expose stdout".to_string())
        })?;
        let stderr = child.stderr.take();
        self.spawn_reader(stdout, stdin.clone());
        self.spawn_stderr_drain(stderr);
        *process_guard = Some(CodexAppServerProcess {
            child,
            stdin,
            _guard: guard,
        });
        drop(process_guard);
        self.initialize()
    }

    fn initialize(&self) -> Result<(), AgentError> {
        self.request_raw_started(
            "initialize",
            Some(json!({
                "clientInfo": {
                    "name": "work_leaf",
                    "title": "Work Leaf",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "capabilities": {
                    "experimentalApi": true,
                },
            })),
        )?;
        self.write_notification("initialized", None)
    }

    fn spawn_app_server_process(&self) -> Result<Child, AgentError> {
        let mut last_busy_error = None;
        for delay_ms in CODEX_APP_SERVER_SPAWN_RETRY_DELAYS_MS {
            if *delay_ms > 0 {
                thread::sleep(Duration::from_millis(*delay_ms));
            }
            let mut command = Command::new(&self.config.binary);
            command
                .arg("app-server")
                .arg("--listen")
                .arg("stdio://")
                .current_dir(&self.config.project_dir)
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
                    trace_codex_app_server(format_args!(
                        "app-server executable was busy during spawn; retrying"
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

    fn spawn_reader(&self, stdout: impl Read + Send + 'static, stdin: Arc<Mutex<ChildStdin>>) {
        let router = self.router.clone();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) => match serde_json::from_str::<Value>(&line) {
                        Ok(message) => route_app_server_message(&router, &stdin, message),
                        Err(error) => close_app_server_router(
                            &router,
                            format!("invalid Codex app-server JSON: {error}: {line}"),
                        ),
                    },
                    Err(error) => close_app_server_router(
                        &router,
                        format!("Codex app-server read failed: {error}"),
                    ),
                }
            }
            close_app_server_router(&router, "Codex app-server closed stdout".to_string());
        });
    }

    fn spawn_stderr_drain(&self, stderr: Option<impl Read + Send + 'static>) {
        if let Some(mut stderr) = stderr {
            thread::spawn(move || {
                let mut text = String::new();
                let _ = stderr.read_to_string(&mut text);
                if env::var_os("WORK_LEAF_CODEX_TRACE").is_some() && !text.trim().is_empty() {
                    eprintln!("work-leaf codex app-server stderr:\n{}", text.trim());
                }
            });
        }
    }

    fn stop_process(&self) {
        if let Some(mut process) = self
            .process
            .lock()
            .expect("codex app-server process mutex poisoned")
            .take()
        {
            let pid = process.child.id();
            trace_codex_app_server(format_args!("stopping app-server pid {pid}"));
            let _ = process.child.kill();
            let _ = process.child.wait();
        }
    }

    fn reset_after_restart(&self) {
        let (lock, condvar) = &*self.router;
        let mut state = lock.lock().expect("codex app-server router mutex poisoned");
        state.closed_error = None;
        state.responses.clear();
        state.turn_notifications.clear();
        state.pending_turn_notifications.clear();
        self.loaded_threads
            .lock()
            .expect("loaded threads mutex poisoned")
            .clear();
        self.active_turns
            .lock()
            .expect("active turns mutex poisoned")
            .clear();
        condvar.notify_all();
    }

    fn shutdown(&self) {
        if self
            .process
            .lock()
            .expect("codex app-server process mutex poisoned")
            .is_none()
        {
            return;
        }
        self.stop_process();
        self.reset_after_restart();
    }

    fn register_response(&self, request_id: &str) {
        let (lock, _) = &*self.router;
        lock.lock()
            .expect("codex app-server router mutex poisoned")
            .responses
            .entry(request_id.to_string())
            .or_default();
    }

    fn unregister_response(&self, request_id: &str) {
        let (lock, _) = &*self.router;
        lock.lock()
            .expect("codex app-server router mutex poisoned")
            .responses
            .remove(request_id);
    }

    fn next_response(&self, request_id: &str) -> Result<Value, AgentError> {
        let (lock, condvar) = &*self.router;
        let mut state = lock.lock().expect("codex app-server router mutex poisoned");
        loop {
            if let Some(response) = state
                .responses
                .get_mut(request_id)
                .and_then(VecDeque::pop_front)
            {
                state.responses.remove(request_id);
                if let Some(error) = response.error {
                    return Err(self.protocol_error(error));
                }
                return Ok(response.result.unwrap_or(Value::Null));
            }
            if let Some(error) = &state.closed_error {
                return Err(self.protocol_error(error.clone()));
            }
            state = condvar
                .wait(state)
                .expect("codex app-server router mutex poisoned");
        }
    }

    fn register_turn(&self, turn_id: &str) {
        let (lock, condvar) = &*self.router;
        let mut state = lock.lock().expect("codex app-server router mutex poisoned");
        let pending = state
            .pending_turn_notifications
            .remove(turn_id)
            .unwrap_or_default();
        state
            .turn_notifications
            .entry(turn_id.to_string())
            .or_default()
            .extend(pending);
        condvar.notify_all();
    }

    fn unregister_turn(&self, turn_id: &str) {
        let (lock, _) = &*self.router;
        lock.lock()
            .expect("codex app-server router mutex poisoned")
            .turn_notifications
            .remove(turn_id);
    }

    fn next_turn_notification(
        &self,
        turn_id: &str,
    ) -> Result<CodexAppServerNotification, AgentError> {
        let (lock, condvar) = &*self.router;
        let mut state = lock.lock().expect("codex app-server router mutex poisoned");
        loop {
            if let Some(notification) = state
                .turn_notifications
                .get_mut(turn_id)
                .and_then(VecDeque::pop_front)
            {
                return Ok(notification);
            }
            if let Some(error) = &state.closed_error {
                return Err(self.protocol_error(error.clone()));
            }
            state = condvar
                .wait(state)
                .expect("codex app-server router mutex poisoned");
        }
    }

    fn protocol_error(&self, message: String) -> AgentError {
        AgentError::ProcessFailed {
            program: self.config.binary.clone(),
            status: None,
            stderr: message,
        }
    }
}

impl Drop for CodexAppServer {
    fn drop(&mut self) {
        if let Some(mut process) = self
            .process
            .lock()
            .expect("codex app-server process mutex poisoned")
            .take()
        {
            let _ = process.child.kill();
            let _ = process.child.wait();
        }
    }
}

struct ActiveTurnGuard {
    active_turns: Arc<Mutex<BTreeMap<AgentId, ActiveTurn>>>,
    agent_id: AgentId,
    active: ActiveTurn,
}

impl ActiveTurnGuard {
    fn new(
        active_turns: Arc<Mutex<BTreeMap<AgentId, ActiveTurn>>>,
        agent_id: AgentId,
        thread_id: String,
        turn_id: String,
    ) -> Self {
        let active = ActiveTurn { thread_id, turn_id };
        active_turns
            .lock()
            .expect("active turns mutex poisoned")
            .insert(agent_id.clone(), active.clone());
        Self {
            active_turns,
            agent_id,
            active,
        }
    }
}

impl Drop for ActiveTurnGuard {
    fn drop(&mut self) {
        let mut active_turns = self
            .active_turns
            .lock()
            .expect("active turns mutex poisoned");
        if active_turns.get(&self.agent_id) == Some(&self.active) {
            active_turns.remove(&self.agent_id);
        }
    }
}

fn route_app_server_message(
    router: &Arc<(Mutex<CodexAppServerRouterState>, Condvar)>,
    stdin: &Arc<Mutex<ChildStdin>>,
    message: Value,
) {
    if let Some(method) = message.get("method").and_then(Value::as_str) {
        if let Some(id) = message.get("id").cloned() {
            respond_to_server_request(stdin, id, method);
            return;
        }
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        route_notification(
            router,
            CodexAppServerNotification {
                method: method.to_string(),
                params,
            },
        );
        return;
    }

    let Some(id) = message.get("id").map(json_rpc_id_string) else {
        return;
    };
    let response = if let Some(error) = message.get("error") {
        CodexJsonRpcResponse {
            result: None,
            error: Some(json_rpc_error_message(error)),
        }
    } else {
        CodexJsonRpcResponse {
            result: Some(message.get("result").cloned().unwrap_or(Value::Null)),
            error: None,
        }
    };
    let (lock, condvar) = &**router;
    let mut state = lock.lock().expect("codex app-server router mutex poisoned");
    if let Some(queue) = state.responses.get_mut(&id) {
        queue.push_back(response);
        condvar.notify_all();
    }
}

fn route_notification(
    router: &Arc<(Mutex<CodexAppServerRouterState>, Condvar)>,
    notification: CodexAppServerNotification,
) {
    let Some(turn_id) = notification_turn_id(&notification) else {
        return;
    };
    let (lock, condvar) = &**router;
    let mut state = lock.lock().expect("codex app-server router mutex poisoned");
    if let Some(queue) = state.turn_notifications.get_mut(&turn_id) {
        queue.push_back(notification);
    } else {
        state
            .pending_turn_notifications
            .entry(turn_id)
            .or_default()
            .push_back(notification);
    }
    condvar.notify_all();
}

fn respond_to_server_request(stdin: &Arc<Mutex<ChildStdin>>, id: Value, method: &str) {
    let result = match method {
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval" => {
            json!({ "decision": "accept" })
        }
        _ => json!({}),
    };
    let response = json!({ "id": id, "result": result });
    if let Ok(text) = serde_json::to_string(&response) {
        let _ = write_json_line(stdin, &text);
    }
}

fn close_app_server_router(
    router: &Arc<(Mutex<CodexAppServerRouterState>, Condvar)>,
    error: String,
) {
    let (lock, condvar) = &**router;
    let mut state = lock.lock().expect("codex app-server router mutex poisoned");
    if state.closed_error.is_none() {
        state.closed_error = Some(error);
    }
    condvar.notify_all();
}

fn write_json_line(stdin: &Arc<Mutex<ChildStdin>>, text: &str) -> std::io::Result<()> {
    let mut stdin = stdin.lock().expect("codex app-server stdin mutex poisoned");
    stdin.write_all(text.as_bytes())?;
    stdin.write_all(b"\n")?;
    stdin.flush()
}

fn thread_start_params(config: &CodexCommandConfig, sandbox: &SandboxMode) -> Value {
    let mut params = json!({
        "approvalPolicy": "never",
        "cwd": config.project_dir.display().to_string(),
        "sandbox": sandbox.as_codex_arg(),
    });
    if let Some(model) = &config.model {
        params["model"] = json!(model);
    }
    params
}

fn thread_resume_params(
    thread_id: &str,
    config: &CodexCommandConfig,
    sandbox: &SandboxMode,
) -> Value {
    let mut params = thread_start_params(config, sandbox);
    params["threadId"] = json!(thread_id);
    params
}

fn turn_start_params(
    thread_id: &str,
    prompt: &str,
    config: &CodexCommandConfig,
    sandbox: &SandboxMode,
) -> Value {
    let mut params = json!({
        "approvalPolicy": "never",
        "cwd": config.project_dir.display().to_string(),
        "sandboxPolicy": sandbox_policy(sandbox),
        "threadId": thread_id,
        "input": [{ "type": "text", "text": prompt }],
    });
    if let Some(model) = &config.model {
        params["model"] = json!(model);
    }
    params
}

fn sandbox_policy(sandbox: &SandboxMode) -> Value {
    match sandbox {
        SandboxMode::WorkspaceWrite => json!({ "type": "workspaceWrite" }),
        SandboxMode::DangerFullAccess => json!({ "type": "dangerFullAccess" }),
        SandboxMode::ReadOnly => json!({ "type": "readOnly" }),
    }
}

fn notification_turn_id(notification: &CodexAppServerNotification) -> Option<String> {
    notification
        .params
        .get("turnId")
        .and_then(Value::as_str)
        .or_else(|| {
            notification
                .params
                .get("turn")
                .and_then(|turn| turn.get("id"))
                .and_then(Value::as_str)
        })
        .map(ToString::to_string)
}

fn token_usage_from_params(params: &Value) -> Option<AgentTokenUsage> {
    let usage = params
        .get("tokenUsage")
        .or_else(|| params.get("token_usage"))?
        .get("last")?;
    Some(AgentTokenUsage {
        input_tokens: json_u64(usage, "inputTokens", "input_tokens"),
        cached_input_tokens: json_u64(usage, "cachedInputTokens", "cached_input_tokens"),
        output_tokens: json_u64(usage, "outputTokens", "output_tokens"),
        reasoning_output_tokens: json_u64(
            usage,
            "reasoningOutputTokens",
            "reasoning_output_tokens",
        ),
    })
}

fn json_u64(value: &Value, camel: &str, snake: &str) -> u64 {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn agent_message_text(item: &Value) -> Option<&str> {
    let item = item_root(item);
    (json_string_ref(item, "type") == Some("agentMessage"))
        .then(|| json_string_ref(item, "text"))
        .flatten()
}

fn activity_from_item(method: &str, item: &Value) -> Option<String> {
    let item = item_root(item);
    let phase = if method == "item/started" {
        "started"
    } else {
        "completed"
    };
    match json_string_ref(item, "type")? {
        "commandExecution" => {
            let command = compact_text(json_string_ref(item, "command")?, 160);
            if phase == "completed" {
                if let Some(exit_code) = item.get("exitCode").and_then(Value::as_i64) {
                    Some(format!("command completed: {command} (exit {exit_code})"))
                } else {
                    Some(format!("command completed: {command}"))
                }
            } else {
                Some(format!("command started: {command}"))
            }
        }
        "fileChange" => {
            let paths = item_change_paths(item);
            let paths = if paths.is_empty() {
                "unknown paths".to_string()
            } else {
                compact_text(&paths.join(", "), 160)
            };
            Some(format!("file change {phase}: {paths}"))
        }
        "mcpToolCall" => {
            let server = json_string_ref(item, "server").unwrap_or("mcp");
            let tool = json_string_ref(item, "tool").unwrap_or("tool");
            Some(format!("mcp tool {phase}: {server}/{tool}"))
        }
        "dynamicToolCall" => {
            let namespace = json_string_ref(item, "namespace");
            let tool = json_string_ref(item, "tool").unwrap_or("tool");
            Some(match namespace {
                Some(namespace) if !namespace.is_empty() => {
                    format!("tool {phase}: {namespace}/{tool}")
                }
                _ => format!("tool {phase}: {tool}"),
            })
        }
        "webSearch" => json_string_ref(item, "query")
            .map(|query| format!("web search {phase}: {}", compact_text(query, 160))),
        "plan" if phase == "completed" => json_string_ref(item, "text")
            .map(|plan| format!("plan updated: {}", compact_text(plan, 160))),
        _ => None,
    }
}

fn item_root(item: &Value) -> &Value {
    item.get("root").unwrap_or(item)
}

fn item_change_paths(item: &Value) -> Vec<String> {
    item.get("changes")
        .and_then(Value::as_array)
        .map(|changes| {
            changes
                .iter()
                .flat_map(|change| {
                    ["path", "oldPath", "newPath", "movePath"]
                        .into_iter()
                        .filter_map(|key| json_string(change, key))
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        })
        .unwrap_or_default()
}

fn compact_text(text: &str, limit: usize) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= limit {
        return normalized;
    }
    let mut compacted = normalized
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>();
    compacted.push_str("...");
    compacted
}

fn turn_failed(params: &Value) -> bool {
    params
        .get("turn")
        .and_then(|turn| turn.get("status"))
        .map(display_json_value)
        .is_some_and(|status| status == "failed")
}

fn turn_error_message(params: &Value) -> Option<&str> {
    params
        .get("turn")
        .and_then(|turn| turn.get("error"))
        .and_then(|error| json_string_ref(error, "message"))
}

fn json_pointer_str<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    json_string_ref(value, key).map(ToString::to_string)
}

fn json_string_ref<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn display_json_value(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.to_string();
    }
    if let Some(root) = value.get("root") {
        return display_json_value(root);
    }
    if let Some(value_type) = value.get("type").and_then(Value::as_str) {
        return value_type.to_string();
    }
    value.to_string()
}

fn json_rpc_id_string(value: &Value) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn json_rpc_error_message(error: &Value) -> String {
    error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("Codex app-server request failed")
        .to_string()
}

fn slash_command_name(prompt: &str) -> (Option<String>, &str) {
    let stripped = prompt.trim();
    if !stripped.starts_with('/') || stripped.len() == 1 {
        return (None, "");
    }
    let without_slash = &stripped[1..];
    if without_slash.chars().next().is_none_or(char::is_whitespace) {
        return (None, "");
    }
    let (command, args) = without_slash
        .split_once(char::is_whitespace)
        .unwrap_or((without_slash, ""));
    (Some(command.to_ascii_lowercase()), args.trim())
}

fn is_codex_slash_command(prompt: &str) -> bool {
    slash_command_name(prompt).0.is_some()
}

fn is_linearize_agent(agent_id: &AgentId) -> bool {
    let value = agent_id.as_str();
    value == "linearize" || value.starts_with("linearize-")
}

fn trace_codex_app_server(message: std::fmt::Arguments<'_>) {
    if env::var_os("WORK_LEAF_CODEX_TRACE").is_some() {
        eprintln!("work-leaf codex app-server: {message}");
    }
}

fn is_broken_pipe(error: &AgentError) -> bool {
    matches!(error, AgentError::Io(error) if error.kind() == std::io::ErrorKind::BrokenPipe)
}

fn is_text_file_busy(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(26)
}

impl CodexBackend {
    pub fn new(config: CodexCommandConfig, policy: PromptPolicy) -> Self {
        let app_server = Arc::new(CodexAppServer::new(config.clone()));
        Self {
            config,
            policy,
            state: Arc::new(Mutex::new(CodexBackendState::default())),
            operation_condvar: Arc::new(Condvar::new()),
            app_server,
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
        trace_codex_app_server(format_args!(
            "backend drop with lifecycle owners={strong_count}"
        ));
        if strong_count == 1 {
            trace_codex_app_server(format_args!(
                "last backend owner dropped; shutting down agents"
            ));
            self.app_server.shutdown();
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
        self.app_server.interrupt(agent_id, &self.shutdown)
    }

    fn shutdown(&mut self) {
        self.app_server.shutdown();
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
        let output = self.app_server.request_turn_streaming(
            CodexTurnRequest {
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
        let output = self.app_server.request_turn_streaming(
            CodexTurnRequest {
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
        let prompt = if has_session {
            prompt.to_string()
        } else {
            self.policy.inject(agent_id, &feature, prompt)
        };
        let sandbox = self.sandbox_for_agent(agent_id);
        let output = if is_codex_slash_command(&prompt) {
            let thread_id = thread_id
                .as_deref()
                .ok_or_else(|| AgentError::UnknownSession(agent_id.clone()))?;
            self.app_server
                .command(agent_id, &prompt, thread_id, &sandbox, &self.shutdown)?
        } else {
            self.app_server.request_turn_streaming(
                CodexTurnRequest {
                    agent_id,
                    prompt: &prompt,
                    thread_id: thread_id.as_deref(),
                    sandbox,
                },
                &self.shutdown,
                sink,
                None,
            )?
        };
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
        self.record_send_reply(agent_id, prompt.as_str(), output.reply)
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
        let prompt = if has_session {
            prompt.to_string()
        } else {
            self.policy.inject(agent_id, &feature, prompt)
        };
        let sandbox = self.sandbox_for_agent(agent_id);
        let output = if is_codex_slash_command(&prompt) {
            let thread_id = thread_id
                .as_deref()
                .ok_or_else(|| AgentError::UnknownSession(agent_id.clone()))?;
            self.app_server
                .command(agent_id, &prompt, thread_id, &sandbox, &self.shutdown)?
        } else {
            self.app_server.request_turn_streaming(
                CodexTurnRequest {
                    agent_id,
                    prompt: &prompt,
                    thread_id: thread_id.as_deref(),
                    sandbox,
                },
                &self.shutdown,
                sink,
                Some(should_interrupt),
            )?
        };
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
        self.record_send_reply(agent_id, prompt.as_str(), output.reply)
    }
}

fn record_usage(state: &mut CodexBackendState, agent_id: &AgentId, usage: AgentTokenUsage) {
    let current = state.usage.get(agent_id).copied().unwrap_or_default();
    state.usage.insert(agent_id.clone(), current.combine(usage));
}
