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

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        _sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        self.send(agent_id, prompt)
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

    pub(crate) fn register(&self, pid: u32) -> ActiveAgentProcessGuard {
        self.register_process(ActiveAgentProcess::new(pid))
    }

    pub(crate) fn register_single_process(&self, pid: u32) -> ActiveAgentProcessGuard {
        self.register_process(ActiveAgentProcess::new_single(pid))
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

    pub(crate) fn terminate_process(&self, pid: u32) -> bool {
        let process = self
            .registry
            .lock()
            .expect("agent process registry mutex poisoned")
            .processes
            .get(&pid)
            .copied();
        if let Some(process) = process {
            process.terminate();
            true
        } else {
            false
        }
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
    kill_process_group: bool,
}

impl ActiveAgentProcess {
    fn new(pid: u32) -> Self {
        Self {
            pid,
            kill_process_group: agent_children_use_process_group(),
        }
    }

    fn new_single(pid: u32) -> Self {
        Self {
            pid,
            kill_process_group: false,
        }
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
pub(crate) fn configure_agent_child_process(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command.process_group(0);
    configure_parent_death_signal(command);
}

#[cfg(not(unix))]
pub(crate) fn configure_agent_child_process(_command: &mut Command) {}

#[cfg(unix)]
pub(crate) fn configure_persistent_agent_child_process(_command: &mut Command) {}

#[cfg(not(unix))]
pub(crate) fn configure_persistent_agent_child_process(_command: &mut Command) {}

#[cfg(target_os = "linux")]
fn configure_parent_death_signal(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    unsafe {
        command.pre_exec(|| {
            let _ = prctl(PR_SET_PDEATHSIG, SIGTERM as usize, 0, 0, 0);
            if getppid() == 1 {
                let _ = kill(getpid(), SIGTERM);
            }
            Ok(())
        });
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
fn configure_parent_death_signal(_command: &mut Command) {}

#[cfg(unix)]
fn agent_children_use_process_group() -> bool {
    true
}

#[cfg(not(unix))]
fn agent_children_use_process_group() -> bool {
    false
}

#[cfg(unix)]
fn signal_process(process: ActiveAgentProcess, signal: ProcessSignal) {
    const SIGKILL: i32 = 9;

    let pid = match i32::try_from(process.pid) {
        Ok(pid) => pid,
        Err(_) => return,
    };
    let target = if process.kill_process_group {
        -pid
    } else {
        pid
    };
    let signal = match signal {
        ProcessSignal::Terminate => SIGTERM,
        ProcessSignal::Kill => SIGKILL,
    };
    unsafe {
        let _ = kill(target, signal);
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

#[cfg(target_os = "linux")]
const PR_SET_PDEATHSIG: i32 = 1;

#[cfg(unix)]
const SIGTERM: i32 = 15;

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn prctl(option: i32, arg2: usize, arg3: usize, arg4: usize, arg5: usize) -> i32;
    fn getpid() -> i32;
    fn getppid() -> i32;
}
