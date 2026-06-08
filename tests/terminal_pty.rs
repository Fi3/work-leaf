#![cfg(unix)]

use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::os::fd::FromRawFd;
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[test]
fn real_terminal_pty_handles_file_read_left_toggle_and_chat_switching() {
    let _guard = pty_test_lock();
    let root = temp_dir("workflow");
    fs::write(root.path().join("Readme.md"), "pty workflow fixture\n").unwrap();
    let fake_bin = write_fake_codex(root.path(), WORKFLOW_CODEX);
    let mut app = PtyWorkLeaf::spawn(root.path(), &fake_bin, 120, 30);

    app.wait_for_output_contains("Command chat:", Duration::from_secs(2));
    app.send(b":new patch ui\n");
    app.wait_for_output_contains(
        "sent file text to user-1: Readme.md",
        Duration::from_secs(5),
    );
    app.wait_for_output_contains(
        "first follow-up answer after file text",
        Duration::from_secs(5),
    );

    app.send(b"\x1b,");
    app.wait_for_frame(Duration::from_secs(2), |frame| {
        frame.starts_with("┌chat") && frame.contains("first follow-up answer after file text")
    });

    app.send(b",:new second\n");
    app.wait_for_frame(Duration::from_secs(5), |frame| {
        frame.starts_with("┌work-leaf")
            && frame.contains(">second user-2")
            && frame.contains("second launch ready")
            && !frame.contains("first follow-up answer after file text")
    });

    app.send(&[27, 23, b'h', b'k']);
    app.wait_for_frame(Duration::from_secs(2), |frame| {
        frame.contains(">patch-ui user-1")
            && frame.contains("first follow-up answer after file text")
            && !frame.contains("second launch ready")
    });

    app.send(b"j");
    app.wait_for_frame(Duration::from_secs(2), |frame| {
        frame.contains(">second user-2") && frame.contains("second launch ready")
    });

    app.send(b"\x1b[<0;4;3M");
    app.wait_for_frame(Duration::from_secs(2), |frame| {
        frame.contains(">patch-ui user-1")
            && frame.contains("first follow-up answer after file text")
            && !frame.contains("second launch ready")
    });
}

#[test]
fn real_terminal_pty_keeps_chat_prompt_visible_after_large_agent_output() {
    let _guard = pty_test_lock();
    let root = temp_dir("large-output");
    let fake_bin = write_fake_codex(root.path(), LARGE_OUTPUT_CODEX);
    let mut app = PtyWorkLeaf::spawn(root.path(), &fake_bin, 80, 12);

    app.wait_for_output_contains("Command chat:", Duration::from_secs(2));
    app.send(b":new large\n");
    app.wait_for_frame(Duration::from_secs(5), |frame| {
        frame.contains("agent-output-line-39") && frame.contains("chat> ")
    });

    app.send(b"hello after overflow");
    app.wait_for_frame(Duration::from_secs(2), |frame| {
        frame.contains("chat> hello after overflow")
    });

    app.send(b"\n");
    app.wait_for_frame(Duration::from_secs(5), |frame| {
        frame.contains("resume reply after large output") && frame.contains("chat> ")
    });
}

#[test]
fn real_terminal_pty_ignores_ctrl_c_and_exits_on_colon_q() {
    let _guard = pty_test_lock();
    let root = temp_dir("quit");
    let fake_bin = write_fake_codex(root.path(), LARGE_OUTPUT_CODEX);
    let mut app = PtyWorkLeaf::spawn(root.path(), &fake_bin, 80, 12);

    app.wait_for_output_contains("Command chat:", Duration::from_secs(2));
    app.send(&[3]);
    thread::sleep(Duration::from_millis(100));
    assert_pty_running(&mut app);

    app.send(b":q\n");
    wait_for_pty_exit(&mut app, Duration::from_secs(2));
}

fn pty_test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn assert_pty_running(app: &mut PtyWorkLeaf) {
    assert!(
        app.child.try_wait().unwrap().is_none(),
        "work-leaf should still be running"
    );
}

fn wait_for_pty_exit(app: &mut PtyWorkLeaf, timeout: Duration) {
    let start = Instant::now();
    loop {
        if app.child.try_wait().unwrap().is_some() {
            return;
        }
        assert!(
            start.elapsed() < timeout,
            "timed out waiting for work-leaf to exit\nlast frame:\n{}",
            last_frame(&app.output())
        );
        thread::sleep(Duration::from_millis(20));
    }
}

struct PtyWorkLeaf {
    child: Child,
    writer: File,
    transcript: Arc<Mutex<String>>,
    reader: Option<JoinHandle<()>>,
}

impl PtyWorkLeaf {
    fn spawn(project_dir: &Path, fake_bin: &Path, width: u16, height: u16) -> Self {
        let (master, slave) = open_pty(width, height);
        let master_file = unsafe { File::from_raw_fd(master) };
        let mut slave_file = unsafe { File::from_raw_fd(slave) };
        let stdin = Stdio::from(slave_file.try_clone().unwrap());
        let stdout = Stdio::from(slave_file.try_clone().unwrap());
        let stderr = Stdio::from(slave_file.try_clone().unwrap());
        let path = format!(
            "{}:{}",
            fake_bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let child = Command::new(env!("CARGO_BIN_EXE_work-leaf"))
            .current_dir(project_dir)
            .env("PATH", path)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .unwrap();
        let _ = slave_file.flush();
        drop(slave_file);

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
        self.wait_for(timeout, |output| output.contains(needle), needle);
    }

    fn wait_for_frame<F>(&self, timeout: Duration, predicate: F)
    where
        F: Fn(&str) -> bool,
    {
        self.wait_for(
            timeout,
            |output| predicate(&last_frame(output)),
            "matching frame",
        );
    }

    fn wait_for<F>(&self, timeout: Duration, predicate: F, expected: &str)
    where
        F: Fn(&str) -> bool,
    {
        let start = Instant::now();
        loop {
            let output = self.output();
            if predicate(&output) {
                return;
            }
            assert!(
                start.elapsed() < timeout,
                "timed out waiting for {expected}\nlast frame:\n{}",
                last_frame(&output)
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn output(&self) -> String {
        self.transcript.lock().unwrap().clone()
    }
}

impl Drop for PtyWorkLeaf {
    fn drop(&mut self) {
        let _ = self.writer.write_all(&[3]);
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

fn last_frame(output: &str) -> String {
    output
        .rsplit_once("\u{1b}[H")
        .map(|(_, frame)| frame.to_string())
        .unwrap_or_else(|| output.to_string())
}

fn write_fake_codex(root: &Path, script: &str) -> PathBuf {
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let codex = bin.join("codex");
    fs::write(&codex, script).unwrap();
    make_executable(&codex);
    bin
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
        "work-leaf-terminal-pty-{name}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    TempProject { root }
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
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

const WORKFLOW_CODEX: &str = r#"#!/bin/sh
seen_resume=0
for arg in "$@"; do
  if [ "$arg" = "resume" ]; then
    seen_resume=1
  fi
done
input=$(cat)
if [ "$seen_resume" = "1" ]; then
  case "$input" in
    *"work-leaf file text"*)
      printf '%s\n' '{"type":"item.completed","item":{"id":"follow","type":"agent_message","text":"first follow-up answer after file text"}}'
      ;;
    *)
      printf '%s\n' '{"type":"item.completed","item":{"id":"unexpected","type":"agent_message","text":"unexpected resume prompt"}}'
      ;;
  esac
else
  case "$input" in
    *"Name this work-leaf chat"*"second"*)
      printf '%s\n' '{"type":"thread.started","thread_id":"thread-title-second"}'
      printf '%s\n' '{"type":"item.completed","item":{"id":"title-second","type":"agent_message","text":"second"}}'
      ;;
    *"Name this work-leaf chat"*)
      printf '%s\n' '{"type":"thread.started","thread_id":"thread-title-first"}'
      printf '%s\n' '{"type":"item.completed","item":{"id":"title-first","type":"agent_message","text":"patch-ui"}}'
      ;;
    *"second"*)
      printf '%s\n' '{"type":"thread.started","thread_id":"thread-second"}'
      printf '%s\n' '{"type":"item.completed","item":{"id":"second","type":"agent_message","text":"second launch ready"}}'
      ;;
    *)
      printf '%s\n' '{"type":"thread.started","thread_id":"thread-first"}'
      printf '%s\n' '{"type":"turn.started"}'
      printf '%s\n' '{"type":"item.completed","item":{"id":"read","type":"agent_message","text":"@work-leaf read Readme.md\nI requested file text from work-leaf."}}'
      ;;
  esac
fi
"#;

const LARGE_OUTPUT_CODEX: &str = r#"#!/bin/sh
seen_resume=0
for arg in "$@"; do
  if [ "$arg" = "resume" ]; then
    seen_resume=1
  fi
done
if [ "$seen_resume" = "1" ]; then
  printf '%s\n' '{"type":"item.completed","item":{"id":"resume","type":"agent_message","text":"resume reply after large output"}}'
else
  printf '%s\n' '{"type":"thread.started","thread_id":"thread-big-output"}'
  printf '%s\n' '{"type":"turn.started"}'
  printf '%s\n' '{"type":"item.completed","item":{"id":"big","type":"agent_message","text":"agent-output-line-00\nagent-output-line-01\nagent-output-line-02\nagent-output-line-03\nagent-output-line-04\nagent-output-line-05\nagent-output-line-06\nagent-output-line-07\nagent-output-line-08\nagent-output-line-09\nagent-output-line-10\nagent-output-line-11\nagent-output-line-12\nagent-output-line-13\nagent-output-line-14\nagent-output-line-15\nagent-output-line-16\nagent-output-line-17\nagent-output-line-18\nagent-output-line-19\nagent-output-line-20\nagent-output-line-21\nagent-output-line-22\nagent-output-line-23\nagent-output-line-24\nagent-output-line-25\nagent-output-line-26\nagent-output-line-27\nagent-output-line-28\nagent-output-line-29\nagent-output-line-30\nagent-output-line-31\nagent-output-line-32\nagent-output-line-33\nagent-output-line-34\nagent-output-line-35\nagent-output-line-36\nagent-output-line-37\nagent-output-line-38\nagent-output-line-39"}}'
fi
"#;
