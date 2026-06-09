#![cfg(unix)]

use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::os::fd::FromRawFd;
use std::os::raw::{c_char, c_int, c_void};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[test]
fn start_script_builds_release_binaries_and_stops_daemon_after_cli_exit() {
    let script = fs::read_to_string("start").expect("root start script exists");
    assert!(script.contains("cargo build --release"));
    assert!(script.contains("work-leaf-orchestrator"));
    assert!(script.contains("kill"));

    let root = temp_dir("start-script");
    let mut app = PtyStart::spawn(root.path(), Path::new(env!("CARGO_BIN_EXE_work-leaf")));

    app.wait_for_output_contains("Command chat:", Duration::from_secs(5));
    app.send(b":q\n");
    app.wait_for_exit(Duration::from_secs(5));
    let output = app.output();
    assert!(output.contains("Command chat:"));
}

struct PtyStart {
    child: Child,
    writer: fs::File,
    transcript: Arc<Mutex<String>>,
    reader: Option<JoinHandle<()>>,
}

impl PtyStart {
    fn spawn(project_dir: &Path, cli_bin: &Path) -> Self {
        let (master, slave) = open_pty(100, 24);
        let master_file = unsafe { fs::File::from_raw_fd(master) };
        let slave_file = unsafe { fs::File::from_raw_fd(slave) };
        let stdin = Stdio::from(slave_file.try_clone().unwrap());
        let stdout = Stdio::from(slave_file.try_clone().unwrap());
        let stderr = Stdio::from(slave_file);
        let bin_dir = cli_bin.parent().unwrap();
        let child = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("start"))
            .current_dir(project_dir)
            .env("WORK_LEAF_START_SKIP_BUILD", "1")
            .env("WORK_LEAF_START_BIN_DIR", bin_dir)
            .env("WORK_LEAF_START_LISTEN", "127.0.0.1:0")
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .unwrap();

        let transcript = Arc::new(Mutex::new(String::new()));
        let reader_transcript = Arc::clone(&transcript);
        let mut reader_file = master_file.try_clone().unwrap();
        let reader = thread::spawn(move || {
            let mut buffer = [0_u8; 4096];
            loop {
                match reader_file.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) => {
                        let text = String::from_utf8_lossy(&buffer[..count]);
                        reader_transcript.lock().unwrap().push_str(&text);
                    }
                    Err(error) if error.kind() == ErrorKind::Interrupted => {}
                    Err(_) => break,
                }
            }
        });

        Self {
            child,
            writer: master_file,
            transcript,
            reader: Some(reader),
        }
    }

    fn send(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).unwrap();
        self.writer.flush().unwrap();
    }

    fn wait_for_output_contains(&self, needle: &str, timeout: Duration) {
        let start = Instant::now();
        loop {
            if self.output().contains(needle) {
                return;
            }
            assert!(
                start.elapsed() < timeout,
                "timed out waiting for {needle}\n{}",
                self.output()
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn wait_for_exit(&mut self, timeout: Duration) {
        let start = Instant::now();
        loop {
            if self.child.try_wait().unwrap().is_some() {
                return;
            }
            assert!(
                start.elapsed() < timeout,
                "start script did not exit after CLI quit\n{}",
                self.output()
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn output(&self) -> String {
        self.transcript.lock().unwrap().clone()
    }
}

impl Drop for PtyStart {
    fn drop(&mut self) {
        let _ = self.writer.write_all(b":q\n");
        let _ = self.writer.flush();
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if self.child.try_wait().ok().flatten().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[test]
fn start_script_uses_default_daemon_port_and_fails_when_unavailable() {
    let script = fs::read_to_string("start").expect("root start script exists");
    assert!(script.contains("WORK_LEAF_START_LISTEN:-127.0.0.1:7878"));

    let root = temp_dir("start-script-port-busy");
    let _listener = TcpListener::bind("127.0.0.1:7878").ok();
    let bin_dir = Path::new(env!("CARGO_BIN_EXE_work-leaf")).parent().unwrap();

    let output = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("start"))
        .current_dir(root.path())
        .env("WORK_LEAF_START_SKIP_BUILD", "1")
        .env("WORK_LEAF_START_BIN_DIR", bin_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "start should fail when the default daemon port is unavailable"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("work-leaf orchestrator exited before startup")
            || stderr.contains("Address already in use")
            || stderr.contains("AddrInUse"),
        "{stderr}"
    );
}

#[test]
fn three_feature_smoke_script_describes_head_binary_old_base_workflow() {
    let script =
        fs::read_to_string("smoke-three-features").expect("three-feature smoke script exists");
    let mode = fs::metadata("smoke-three-features")
        .expect("three-feature smoke script is statable")
        .permissions()
        .mode();

    assert_ne!(mode & 0o111, 0, "smoke script should be executable");
    assert!(script.contains("WORK_LEAF_SMOKE_BASE:-c92a0b7060a36eac6db2d869b85e589a7a9480f9"));
    assert!(script.contains(
        "git -C \"$repo_root\" clone --no-checkout --no-hardlinks \"$repo_root\" \"$checkout_dir\""
    ));
    assert!(script.contains("git -C \"$checkout_dir\" checkout --detach \"$base_commit\""));
    assert!(script.contains("rm -rf \"$tmp_root\""));
    assert!(script.contains("trap cleanup EXIT INT TERM"));
    assert!(script.contains("WORK_LEAF_START_BIN_DIR=\"$bin_dir\""));
    assert!(script.contains("\"$repo_root/start\""));
    assert!(script.contains(":new add vim like visual mode"));
    assert!(script.contains(":new when an user prompt start with /"));
    assert!(script.contains(":new when review process is done"));
}

#[test]
fn three_feature_smoke_script_cleans_temp_checkout_after_dry_run() {
    let root = temp_dir("three-feature-smoke-dry-run");
    let output = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("smoke-three-features"))
        .arg("--dry-run")
        .env("WORK_LEAF_SMOKE_SKIP_BUILD", "1")
        .env("WORK_LEAF_SMOKE_BASE", "HEAD")
        .env("WORK_LEAF_SMOKE_TMPDIR", root.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "dry run should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let temp_root = smoke_temp_root(&output.stdout);
    assert!(
        !temp_root.exists(),
        "smoke script should remove dry-run temp root {temp_root:?}"
    );
}

#[test]
fn three_feature_smoke_script_cleans_temp_checkout_after_launch_failure() {
    let root = temp_dir("three-feature-smoke-failure");
    let output = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("smoke-three-features"))
        .env("WORK_LEAF_SMOKE_SKIP_BUILD", "1")
        .env("WORK_LEAF_SMOKE_BASE", "HEAD")
        .env("WORK_LEAF_SMOKE_TMPDIR", root.path())
        .env("WORK_LEAF_SMOKE_BIN_DIR", root.path().join("missing-bin"))
        .env("WORK_LEAF_SMOKE_LISTEN", "127.0.0.1:0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "launch should fail with missing binaries\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let temp_root = smoke_temp_root(&output.stdout);
    assert!(
        !temp_root.exists(),
        "smoke script should remove failed-launch temp root {temp_root:?}"
    );
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

fn smoke_temp_root(stdout: &[u8]) -> PathBuf {
    let stdout = String::from_utf8_lossy(stdout);
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("WORK_LEAF_SMOKE_TEMP="))
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("smoke output did not include temp root:\n{stdout}"))
}

#[repr(C)]
struct Winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

#[link(name = "util")]
unsafe extern "C" {
    fn openpty(
        amaster: *mut c_int,
        aslave: *mut c_int,
        name: *mut c_char,
        termp: *const c_void,
        winp: *const Winsize,
    ) -> c_int;
}

fn open_pty(width: u16, height: u16) -> (c_int, c_int) {
    let size = Winsize {
        ws_row: height,
        ws_col: width,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut master = -1;
    let mut slave = -1;
    let status = unsafe {
        openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null(),
            &size,
        )
    };
    assert_eq!(status, 0, "openpty failed");
    (master, slave)
}
