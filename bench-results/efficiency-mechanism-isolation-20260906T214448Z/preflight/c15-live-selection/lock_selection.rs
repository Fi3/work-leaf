//! Standalone qualification of one real shared lock table, not daemon integration.
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use work_leaf::locks::FileAccessError;
use work_leaf::{AgentId, FileLockTable, GitPatcher, PatchRequest};

fn failure(message: impl Into<String>) -> FileAccessError {
    FileAccessError::Io(io::Error::other(message.into()))
}

fn execute() -> Result<Value, Box<dyn std::error::Error>> {
    let args: Vec<PathBuf> = env::args_os().skip(1).map(PathBuf::from).collect();
    if args.len() != 5 {
        return Err("expected ROOT OWNED OVERLAYS HELPER PATCH".into());
    }
    let root = args[0].canonicalize()?;
    if root != args[0] {
        return Err("fixture root alias".into());
    }
    let proposal: Value = serde_json::from_slice(&fs::read(&args[4])?)?;
    let request = PatchRequest::new(
        AgentId::new("qualification-writer")?,
        "Cooperative writer qualification",
        "Verify the exact shared root lock",
        proposal["body"].as_str().ok_or("missing proposal body")?,
    );
    let locks = FileLockTable::new(root.clone());
    let patcher = GitPatcher::new(root.clone(), locks.clone());
    let (start_tx, start_rx) = mpsc::channel();
    let (attempt_tx, attempt_rx) = mpsc::channel();
    let (finish_tx, finish_rx) = mpsc::channel();
    let writer = thread::spawn(move || {
        if start_rx.recv().is_ok() {
            let _ = attempt_tx.send(());
            let result = patcher
                .apply_edit(request)
                .map_err(|error| error.to_string());
            let _ = finish_tx.send(result);
        }
    });
    let mut owner_lock_seconds = 0.0;
    let selected = locks.with_read_locks(&[PathBuf::from(".")], || {
        let started = Instant::now();
        let result = (|| {
            start_tx
                .send(())
                .map_err(|error| failure(error.to_string()))?;
            attempt_rx
                .recv_timeout(Duration::from_secs(2))
                .map_err(|error| failure(error.to_string()))?;
            if !matches!(
                finish_rx.recv_timeout(Duration::from_millis(50)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ) {
                return Err(failure(
                    "writer did not remain pending inside root read lock",
                ));
            }
            let head = Command::new("/usr/bin/git")
                .args(["-C", root.to_str().ok_or_else(|| failure("non-UTF8 root"))?])
                .args(["rev-parse", "--verify", "HEAD^{commit}"])
                .output()
                .map_err(FileAccessError::Io)?;
            if !head.status.success() {
                return Err(failure("fixture HEAD lookup failed"));
            }
            let head =
                String::from_utf8(head.stdout).map_err(|error| failure(error.to_string()))?;
            let output = Command::new("/usr/bin/python3")
                .arg("-B")
                .arg(&args[3])
                .arg("capture")
                .arg(&root)
                .arg(&args[1])
                .arg(head.trim())
                .arg(&args[2])
                .output()
                .map_err(FileAccessError::Io)?;
            if !output.status.success() {
                return Err(failure(format!(
                    "selection child failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
            serde_json::from_slice::<Value>(&output.stdout)
                .map_err(|error| failure(error.to_string()))
        })();
        owner_lock_seconds = started.elapsed().as_secs_f64();
        result
    });
    drop(start_tx);
    let applied = finish_rx.recv_timeout(Duration::from_secs(5));
    // A failing selection still releases the guard and joins its cooperative writer.
    writer.join().map_err(|_| "writer panicked")?;
    let applied = applied??;
    let common = json!({
        "caller_scope": "standalone shared FileLockTable and actual GitPatcher, not running daemon",
        "writer_blocked_inside_owner_closure": true,
        "writer_applied_after_release": true,
        "writer_commit": applied.commit,
        "owner_lock_seconds": owner_lock_seconds,
        "selection_succeeded": selected.is_ok(),
        "selected": selected.as_ref().ok(),
        "selection_error": selected.as_ref().err().map(ToString::to_string),
    });
    // Persist timing even when the child has retained a selection failure.
    let path = args[1].join("CALLER.json");
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(serde_json::to_string_pretty(&common)?.as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    selected?;
    Ok(common)
}

fn main() {
    match execute() {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("private lock qualification failed: {error}");
            std::process::exit(1);
        }
    }
}
