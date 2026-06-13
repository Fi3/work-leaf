#![cfg(unix)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use work_leaf::{AgentId, AgentKind, HttpControllerClient};

mod support;

use support::fake_codex::write_path_app_server;

#[test]
fn localhost_http_controller_preserves_terminal_workflow_state() {
    if localhost_tcp_is_unavailable() {
        return;
    }
    let root = temp_dir("http-controller");
    let fake_bin = write_fake_codex_app_server(root.path(), HTTP_APP_SERVER);
    let mut daemon = Daemon::spawn(root.path(), &fake_bin);
    let mut client = HttpControllerClient::connect(daemon.url()).unwrap();

    client.execute_command_line("new http boundary").unwrap();
    let launching_state = client.state().unwrap();
    assert!(
        launching_state
            .snapshot
            .session(&AgentId::new("user-1").unwrap())
            .is_some()
    );
    assert!(client.wait_for_idle(Duration::from_secs(5)).unwrap());
    assert!(!client.state().unwrap().busy);

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

#[test]
fn localhost_http_controller_serves_static_web_ui_assets() {
    if localhost_tcp_is_unavailable() {
        return;
    }
    let root = temp_dir("http-web-ui");
    let fake_bin = write_fake_codex_app_server(root.path(), HTTP_APP_SERVER);
    let mut daemon = Daemon::spawn(root.path(), &fake_bin);

    let html = http_get(daemon.url(), "/web-ui/");
    assert!(html.starts_with("HTTP/1.1 200 OK"));
    assert!(html.contains("Content-Type: text/html; charset=utf-8"));
    assert!(html.contains(r#"<main"#));
    assert!(html.contains(r#"href="./styles.css""#));
    assert!(html.contains(r#"src="./app.js""#));

    let css = http_get(daemon.url(), "/web-ui/styles.css");
    assert!(css.starts_with("HTTP/1.1 200 OK"));
    assert!(css.contains("Content-Type: text/css; charset=utf-8"));
    assert!(css.contains("@media"));

    let js = http_get(daemon.url(), "/web-ui/app.js");
    assert!(js.starts_with("HTTP/1.1 200 OK"));
    assert!(js.contains("Content-Type: text/javascript; charset=utf-8"));
    assert!(js.contains("/state"));
    assert!(js.contains("/events/drain"));
    assert!(js.contains("/agent/message"));

    let mut client = HttpControllerClient::connect(daemon.url()).unwrap();
    client.shutdown().unwrap();
    daemon.wait_for_exit(Duration::from_secs(2));
}

#[test]
fn localhost_http_controller_uses_selected_claude_agent_for_web_ui_sessions() {
    if localhost_tcp_is_unavailable() {
        return;
    }
    let root = temp_dir("http-claude-agent");
    let fake_bin = write_fake_claude(root.path());
    let mut daemon = Daemon::spawn_with_args(root.path(), &fake_bin, ["--agent", "claude"]);
    let mut client = HttpControllerClient::connect(daemon.url()).unwrap();

    client.execute_command_line("new http claude").unwrap();
    assert!(client.wait_for_idle(Duration::from_secs(5)).unwrap());

    let agent_id = AgentId::new("user-1").unwrap();
    let snapshot = client.snapshot().unwrap();
    let session = snapshot.session(&agent_id).expect("session exists");
    assert_eq!(session.kind, AgentKind::External("claude".to_string()));
    assert!(
        session
            .lines
            .iter()
            .any(|line| line == "launch over claude")
    );

    client.shutdown().unwrap();
    daemon.wait_for_exit(Duration::from_secs(2));
}

fn http_get(base_url: &str, path: &str) -> String {
    let address = base_url.strip_prefix("http://").unwrap();
    let mut stream = TcpStream::connect(address).unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn localhost_tcp_is_unavailable() -> bool {
    match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            drop(listener);
            false
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping localhost HTTP test: {error}");
            true
        }
        Err(error) => panic!("unexpected localhost bind failure: {error}"),
    }
}

struct Daemon {
    child: Child,
    url: String,
}

impl Daemon {
    fn spawn(project_dir: &Path, fake_bin: &Path) -> Self {
        Self::spawn_with_args(project_dir, fake_bin, std::iter::empty::<&str>())
    }

    fn spawn_with_args<I, S>(project_dir: &Path, fake_bin: &Path, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let path = format!(
            "{}:{}",
            fake_bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut child = Command::new(env!("CARGO_BIN_EXE_work-leaf-orchestrator"))
            .args(args)
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

fn write_fake_codex_app_server(root: &Path, script: &str) -> PathBuf {
    write_path_app_server(root, script)
}

fn write_fake_claude(root: &Path) -> PathBuf {
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let claude = bin.join("claude");
    fs::write(
        &claude,
        r#"#!/bin/sh
while IFS= read -r _line; do :; done
printf '{"type":"system","subtype":"init","session_id":"session-http-claude"}\n'
printf '{"type":"system","subtype":"status","status":"requesting","session_id":"session-http-claude"}\n'
printf '{"type":"stream_event","session_id":"session-http-claude","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"launch over claude"}}}\n'
printf '{"type":"result","subtype":"success","session_id":"session-http-claude","result":"launch over claude"}\n'
"#,
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(&claude).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&claude, permissions).unwrap();
    bin
}

const HTTP_APP_SERVER: &str = r#"#!/bin/sh
while IFS= read -r line; do
  id=$(request_id "$line")
  case "$line" in
    *'"method":"initialize"'*)
      rpc_ok "$id"
      ;;
    *'"method":"thread/start"'*)
      thread_result "$id" "thread-http"
      ;;
    *'"method":"turn/start"'*"continue over http"*)
      turn_message "$id" "thread-http" "resume over http"
      ;;
    *'"method":"turn/start"'*)
      turn_message "$id" "thread-http" "launch over http"
      ;;
  esac
done
"#;
