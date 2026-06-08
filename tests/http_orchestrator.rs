#![cfg(unix)]

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use work_leaf::{AgentId, HttpControllerClient};

#[test]
fn localhost_http_controller_preserves_terminal_workflow_state() {
    let root = temp_dir("http-controller");
    let fake_bin = write_fake_codex(root.path(), HTTP_CODEX);
    let mut daemon = Daemon::spawn(root.path(), &fake_bin);
    let mut client = HttpControllerClient::connect(daemon.url()).unwrap();

    client.execute_command_line("new http boundary").unwrap();
    assert!(client.wait_for_idle(Duration::from_secs(5)).unwrap());

    let agent_id = AgentId::new("user-1").unwrap();
    let snapshot = client.snapshot().unwrap();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.id, agent_id);
    assert!(session.lines.iter().any(|line| line == "launch over http"));

    client
        .send_message(&agent_id, "continue over http")
        .unwrap();
    assert!(client.wait_for_idle(Duration::from_secs(5)).unwrap());

    let snapshot = client.snapshot().unwrap();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert!(
        session
            .lines
            .iter()
            .any(|line| line == "user: continue over http")
    );
    assert!(session.lines.iter().any(|line| line == "resume over http"));

    client.shutdown().unwrap();
    daemon.wait_for_exit(Duration::from_secs(2));
}

struct Daemon {
    child: Child,
    url: String,
}

impl Daemon {
    fn spawn(project_dir: &Path, fake_bin: &Path) -> Self {
        let path = format!(
            "{}:{}",
            fake_bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut child = Command::new(env!("CARGO_BIN_EXE_work-leaf-orchestrator"))
            .arg("--listen")
            .arg("127.0.0.1:0")
            .current_dir(project_dir)
            .env("PATH", path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut lines = BufReader::new(stdout).lines();
        let line = lines
            .next()
            .expect("daemon should print its URL")
            .expect("daemon URL line should be readable");
        let url = line
            .strip_prefix("WORK_LEAF_ORCHESTRATOR_URL=")
            .expect("daemon should print machine-readable URL")
            .to_string();
        thread::spawn(move || for _ in lines {});
        Self { child, url }
    }

    fn url(&self) -> &str {
        &self.url
    }

    fn wait_for_exit(&mut self, timeout: Duration) {
        let start = Instant::now();
        loop {
            if self.child.try_wait().unwrap().is_some() {
                return;
            }
            assert!(
                start.elapsed() < timeout,
                "daemon did not exit after shutdown"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn temp_dir(name: &str) -> TempProject {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "work-leaf-{name}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    TempProject { root }
}

fn write_fake_codex(root: &Path, script: &str) -> PathBuf {
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let codex = bin.join("codex");
    fs::write(&codex, script).unwrap();
    make_executable(&codex);
    bin
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

const HTTP_CODEX: &str = r#"#!/bin/sh
seen_resume=0
for arg in "$@"; do
  if [ "$arg" = "resume" ]; then
    seen_resume=1
  fi
done
if [ "$seen_resume" = "1" ]; then
  printf '%s\n' '{"type":"item.completed","item":{"id":"resume","type":"agent_message","text":"resume over http"}}'
else
  input=$(cat)
  case "$input" in
    *"Name this work-leaf chat"*)
      printf '%s\n' '{"type":"thread.started","thread_id":"thread-title"}'
      printf '%s\n' '{"type":"item.completed","item":{"id":"title","type":"agent_message","text":"http-boundary"}}'
      ;;
    *)
      printf '%s\n' '{"type":"thread.started","thread_id":"thread-http"}'
      printf '%s\n' '{"type":"item.completed","item":{"id":"launch","type":"agent_message","text":"launch over http"}}'
      ;;
  esac
fi
"#;
