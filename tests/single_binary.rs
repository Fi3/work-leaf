#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn work_leaf_binary_runs_without_sibling_orchestrator_binary() {
    let root = temp_dir("single-binary");
    let bin_dir = root.path().join("isolated-bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let binary = bin_dir.join("work-leaf");
    fs::copy(env!("CARGO_BIN_EXE_work-leaf"), &binary).unwrap();
    make_executable(&binary);

    let mut child = Command::new(&binary)
        .current_dir(root.path())
        .env_remove("WORK_LEAF_ORCHESTRATOR_URL")
        .env("WORK_LEAF_IN_PROCESS", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(b"quit\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(
        output.status.success(),
        "single binary should run without a sibling daemon\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Command chat:"),
        "launcher should reach the command chat\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
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

fn make_executable(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}
