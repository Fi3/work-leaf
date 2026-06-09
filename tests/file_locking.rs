use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use work_leaf::{CommandWritePolicy, FileLockTable};

mod temp_cleanup;

#[test]
fn orchestrator_reads_files_through_shared_read_locks() {
    let root = unique_temp_dir("read-locks");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();

    let locks = FileLockTable::new(root.clone());
    let snapshots = locks.read_files(&[PathBuf::from("src/lib.rs")]).unwrap();

    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].path, PathBuf::from("src/lib.rs"));
    assert_eq!(snapshots[0].text, "pub fn value() -> u8 { 1 }\n");
}

#[test]
fn write_lock_waits_until_existing_readers_release_file() {
    let root = unique_temp_dir("write-waits");
    fs::write(root.join("README.md"), "before\n").unwrap();
    let locks = FileLockTable::new(root);
    let locks_for_reader = locks.clone();
    let locks_for_writer = locks.clone();
    let (reader_locked_tx, reader_locked_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (writer_done_tx, writer_done_rx) = mpsc::channel();

    let reader = thread::spawn(move || {
        locks_for_reader
            .with_read_locks(&[PathBuf::from("README.md")], || {
                reader_locked_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
            .unwrap();
    });

    reader_locked_rx.recv().unwrap();
    let writer = thread::spawn(move || {
        locks_for_writer
            .with_write_locks(&[PathBuf::from("README.md")], || {
                writer_done_tx.send(()).unwrap();
                Ok(())
            })
            .unwrap();
    });

    assert!(
        writer_done_rx
            .recv_timeout(Duration::from_millis(100))
            .is_err()
    );
    release_tx.send(()).unwrap();
    assert!(writer_done_rx.recv_timeout(Duration::from_secs(2)).is_ok());

    reader.join().unwrap();
    writer.join().unwrap();
}

#[test]
fn file_paths_cannot_escape_project_root() {
    let root = unique_temp_dir("path-escape");
    let locks = FileLockTable::new(root);

    let error = locks
        .read_files(&[PathBuf::from("../outside.txt")])
        .unwrap_err()
        .to_string();

    assert!(error.contains("escapes project root"));
}

#[test]
fn command_policy_marks_known_build_commands_as_write_intents() {
    let policy = CommandWritePolicy;

    let cargo = policy.classify(["cargo", "test"]);
    assert!(cargo.writes);
    assert!(cargo.paths.contains(&PathBuf::from("target")));

    let formatter = policy.classify(["cargo", "fmt"]);
    assert!(formatter.writes);
    assert!(formatter.paths.contains(&PathBuf::from(".")));

    let search = policy.classify(["rg", "AgentId"]);
    assert!(!search.writes);
    assert!(search.paths.is_empty());
}

#[test]
fn command_policy_covers_common_language_build_and_test_tools() {
    let policy = CommandWritePolicy;
    for command in [
        vec!["node", "build.js"],
        vec!["deno", "test"],
        vec!["bun", "test"],
        vec!["python", "setup.py", "build"],
        vec!["python3", "-m", "build"],
        vec!["pip", "install", "-r", "requirements.txt"],
        vec!["go", "test", "./..."],
        vec!["mvn", "test"],
        vec!["gradle", "build"],
        vec!["dotnet", "test"],
        vec!["ruby", "test.rb"],
        vec!["bundle", "exec", "rspec"],
        vec!["php", "vendor/bin/phpunit"],
        vec!["composer", "install"],
        vec!["swift", "test"],
        vec!["zig", "build"],
        vec!["gcc", "-o", "app", "main.c"],
        vec!["clang", "-o", "app", "main.c"],
    ] {
        let intent = policy.classify(command.clone());
        assert!(intent.writes, "{command:?} should be write-producing");
        assert!(
            !intent.paths.is_empty(),
            "{command:?} should include lock paths"
        );
    }
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "work-leaf-{name}-{}-{:?}",
        std::process::id(),
        thread::current().id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    temp_cleanup::register(&root);
    root
}
