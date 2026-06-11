use std::collections::BTreeMap;
use std::env;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::agent::{AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage};

pub trait AgentBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError>;
    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError>;

    /// Optional snapshot lookup for providers that keep local session metadata.
    ///
    /// Work Leaf does not require providers to override this method for persistent hidden
    /// system-agent turns; `CommandChat` tracks whether those agents have been launched.
    fn session(&self, _agent_id: &AgentId) -> Option<AgentSession> {
        None
    }
    fn interrupt(&mut self, _agent_id: &AgentId) -> Result<(), AgentError> {
        Ok(())
    }

    fn shutdown_handle(&self) -> AgentShutdownHandle {
        AgentShutdownHandle::default()
    }

    fn shutdown(&mut self) {
        self.shutdown_handle().shutdown();
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        _sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        self.launch(request)
    }

    fn launch_streaming_interruptible(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<AgentSession, AgentError> {
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
        _sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        self.send(agent_id, prompt)
    }

    fn send_streaming_interruptible(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        should_interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        let mut stream = |event: AgentStreamEvent| {
            sink(event.clone());
            let _ = should_interrupt(&event);
        };
        self.send_streaming(agent_id, prompt, &mut stream)
    }
}

#[derive(Clone, Debug, Default)]
pub struct AgentShutdownHandle {
    registry: Arc<Mutex<AgentProcessRegistry>>,
}

impl AgentShutdownHandle {
    pub fn shutdown(&self) {
        trace_agent_shutdown(format_args!("shutdown requested"));
        let processes = {
            let mut registry = self
                .registry
                .lock()
                .expect("agent process registry mutex poisoned");
            registry.shutting_down = true;
            registry.processes.values().copied().collect::<Vec<_>>()
        };
        for process in &processes {
            trace_agent_shutdown(format_args!("terminating pid {}", process.pid));
            process.terminate();
        }

        if self.wait_for_processes(Duration::from_millis(500)) {
            return;
        }

        for process in self.active_processes() {
            trace_agent_shutdown(format_args!("killing pid {}", process.pid));
            process.kill();
        }
        let _ = self.wait_for_processes(Duration::from_millis(500));
    }

    pub(crate) fn register_single_process(&self, pid: u32) -> ActiveAgentProcessGuard {
        self.register_process(ActiveAgentProcess::new(pid))
    }

    fn register_process(&self, process: ActiveAgentProcess) -> ActiveAgentProcessGuard {
        let pid = process.pid;
        let shutting_down = {
            let mut registry = self
                .registry
                .lock()
                .expect("agent process registry mutex poisoned");
            registry.processes.insert(pid, process);
            registry.shutting_down
        };
        if shutting_down {
            process.terminate();
        }
        ActiveAgentProcessGuard {
            shutdown: self.clone(),
            pid,
        }
    }

    fn remove(&self, pid: u32) {
        self.registry
            .lock()
            .expect("agent process registry mutex poisoned")
            .processes
            .remove(&pid);
    }

    fn active_processes(&self) -> Vec<ActiveAgentProcess> {
        self.registry
            .lock()
            .expect("agent process registry mutex poisoned")
            .processes
            .values()
            .copied()
            .collect()
    }

    fn wait_for_processes(&self, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if self
                .registry
                .lock()
                .expect("agent process registry mutex poisoned")
                .processes
                .is_empty()
            {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.registry
            .lock()
            .expect("agent process registry mutex poisoned")
            .processes
            .is_empty()
    }
}

fn trace_agent_shutdown(message: std::fmt::Arguments<'_>) {
    if env::var_os("WORK_LEAF_CODEX_TRACE").is_some() {
        eprintln!("work-leaf agent shutdown: {message}");
    }
}

#[derive(Debug, Default)]
struct AgentProcessRegistry {
    processes: BTreeMap<u32, ActiveAgentProcess>,
    shutting_down: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ActiveAgentProcess {
    pid: u32,
}

impl ActiveAgentProcess {
    fn new(pid: u32) -> Self {
        Self { pid }
    }

    fn terminate(&self) {
        signal_process(*self, ProcessSignal::Terminate);
    }

    fn kill(&self) {
        signal_process(*self, ProcessSignal::Kill);
    }
}

#[derive(Debug)]
pub(crate) struct ActiveAgentProcessGuard {
    shutdown: AgentShutdownHandle,
    pid: u32,
}

impl Drop for ActiveAgentProcessGuard {
    fn drop(&mut self) {
        self.shutdown.remove(self.pid);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessSignal {
    Terminate,
    Kill,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentStreamEvent {
    Status(String),
    AgentMessage(String),
    Error(String),
    Usage(AgentTokenUsage),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentTokenUsage {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
}

impl AgentTokenUsage {
    pub fn combine(self, other: Self) -> Self {
        Self {
            input_tokens: self.input_tokens + other.input_tokens,
            cached_input_tokens: self.cached_input_tokens + other.cached_input_tokens,
            output_tokens: self.output_tokens + other.output_tokens,
            reasoning_output_tokens: self.reasoning_output_tokens + other.reasoning_output_tokens,
        }
    }
}

#[cfg(unix)]
pub(crate) fn configure_persistent_agent_child_process(_command: &mut Command) {}

#[cfg(not(unix))]
pub(crate) fn configure_persistent_agent_child_process(_command: &mut Command) {}

#[cfg(unix)]
fn signal_process(process: ActiveAgentProcess, signal: ProcessSignal) {
    const SIGKILL: i32 = 9;

    let pid = match i32::try_from(process.pid) {
        Ok(pid) => pid,
        Err(_) => return,
    };
    let signal = match signal {
        ProcessSignal::Terminate => SIGTERM,
        ProcessSignal::Kill => SIGKILL,
    };
    unsafe {
        let _ = kill(pid, signal);
    }
}

#[cfg(windows)]
fn signal_process(process: ActiveAgentProcess, signal: ProcessSignal) {
    let mut command = Command::new("taskkill");
    command.arg("/PID").arg(process.pid.to_string()).arg("/T");
    if matches!(signal, ProcessSignal::Kill) {
        command.arg("/F");
    }
    let _ = command.status();
}

#[cfg(all(not(unix), not(windows)))]
fn signal_process(_process: ActiveAgentProcess, _signal: ProcessSignal) {}

#[cfg(unix)]
unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

#[cfg(unix)]
const SIGTERM: i32 = 15;
