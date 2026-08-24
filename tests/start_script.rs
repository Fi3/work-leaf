#![cfg(unix)]

use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::os::fd::FromRawFd;
use std::os::raw::{c_char, c_int, c_void};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[test]
fn start_script_builds_release_binaries_and_stops_daemon_after_cli_exit() {
    let script = fs::read_to_string("start").expect("root start script exists");
    assert!(script.contains("cargo build --release"));
    assert!(script.contains("--bin work-leaf"));
    assert!(script.contains("exec \"$cli_bin\" \"$@\""));
    assert!(!script.contains("work-leaf-orchestrator"));
    assert!(!script.contains("ensure_codex_sdk_python"));
    assert!(!script.contains("openai-codex"));
    assert!(!script.contains("-m venv"));
    assert!(!script.contains("work-leaf-codex-sdk-venv"));

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
            .env("WORK_LEAF_IN_PROCESS", "1")
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
fn start_script_delegates_daemon_lifecycle_to_single_binary() {
    let script = fs::read_to_string("start").expect("root start script exists");
    assert!(script.contains("WORK_LEAF_START_BIN_DIR"));
    assert!(script.contains("cli_bin=\"$bin_dir/work-leaf\""));
    assert!(!script.contains("WORK_LEAF_START_LISTEN"));
    assert!(!script.contains("WORK_LEAF_ORCHESTRATOR_URL"));
    assert!(!script.contains("WORK_LEAF_CODEX_SDK_PYTHON"));
}

#[test]
fn start_script_bench_mode_lists_nested_admitted_candidates() {
    let root = temp_dir("start-script-bench-mode");
    let results_dir = root.path().join("bench-results");
    let current_bin_dir = root.path().join("current-bin");
    fs::create_dir_all(&current_bin_dir).unwrap();
    write_executable(
        &current_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho current fake work-leaf\n",
    );
    let old_bin_dir = results_dir.join("20260101T000000-artifacts/bin");
    fs::create_dir_all(&old_bin_dir).unwrap();
    write_executable(
        &old_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho old fake work-leaf \"$@\"\n",
    );
    write_executable(
        &old_bin_dir.join("work-leaf-orchestrator"),
        "#!/usr/bin/env bash\necho fake orchestrator\n",
    );
    let nested_artifact_dir = results_dir
        .join("parallel-fixed-base-12-20260623T195828+0200")
        .join("runs/run-1/20260623T195828+0200-fixedbase12-r1-three-feature-bench-artifacts");
    write_valid_replay_candidate(
        &nested_artifact_dir,
        "#!/usr/bin/env bash\necho nested fake work-leaf \"$@\"\n",
    );

    let output = run_start_bench(&results_dir, &current_bin_dir, &["--from-test"], b"1\n");

    assert!(
        output.status.success(),
        "bench launch should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available benchmark candidates:"));
    assert!(!stdout.contains("20260101T000000"));
    assert!(stdout.contains("20260623T195828+0200-fixedbase12-r1-three-feature-bench"));
    assert!(stdout.contains("nested fake work-leaf --from-test"));
}

#[test]
fn start_script_bench_mode_lists_only_admitted_candidates_and_runs_the_selected_inode() {
    let root = temp_dir("start-bench-admitted-candidates");
    let results_dir = root.path().join("bench-results");
    let current_bin_dir = root.path().join("current-bin");
    fs::create_dir_all(&current_bin_dir).unwrap();
    write_executable(
        &current_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho CURRENT-BINARY-MUST-NOT-RUN\n",
    );

    let generic_bin = results_dir.join("generic-runner-artifacts/bin");
    fs::create_dir_all(&generic_bin).unwrap();
    write_executable(
        &generic_bin.join("work-leaf"),
        "#!/usr/bin/env bash\necho GENERIC-RUNNER-MUST-NOT-RUN\n",
    );
    write_executable(
        &generic_bin.join("work-leaf-orchestrator"),
        "#!/usr/bin/env bash\nexit 0\n",
    );

    let newer = results_dir.join("z-newer-candidate-artifacts");
    write_valid_replay_candidate(
        &newer,
        "#!/usr/bin/env bash\necho NEWER-CANDIDATE-MUST-NOT-RUN \"$@\"\n",
    );
    let selected = results_dir.join("a-selected-candidate-artifacts");
    let selected_bin = selected.join("candidate/bin/work-leaf");
    write_identity_replay_binary(&selected_bin, "SELECTED-CANDIDATE");
    write_replay_candidate_metadata(&selected, &["work-leaf"], "pass");
    let selected_metadata = fs::metadata(&selected_bin).unwrap();
    let selected_identity = format!("{}:{}", selected_metadata.dev(), selected_metadata.ino());

    let absent = results_dir.join("report-with-absent-runtime-artifacts");
    write_replay_candidate_metadata_with_digests(&absent, &[("work-leaf", "0".repeat(64))], "pass");

    let incomplete = results_dir.join("incomplete-runtime-artifacts");
    fs::create_dir_all(incomplete.join("candidate/bin")).unwrap();
    write_executable(
        &incomplete.join("candidate/bin/work-leaf"),
        "#!/usr/bin/env bash\necho INCOMPLETE-MUST-NOT-RUN\n",
    );
    let incomplete_work_leaf = sha256_file(&incomplete.join("candidate/bin/work-leaf"));
    write_replay_candidate_metadata_with_digests(
        &incomplete,
        &[
            ("work-leaf", incomplete_work_leaf),
            ("work-leaf-orchestrator", "0".repeat(64)),
        ],
        "pass",
    );

    let stale = results_dir.join("stale-runtime-artifacts");
    write_valid_replay_candidate(&stale, "#!/usr/bin/env bash\necho STALE-MUST-NOT-RUN\n");
    write_executable(
        &stale.join("candidate/bin/work-leaf"),
        "#!/usr/bin/env bash\necho REPLACED-STALE-MUST-NOT-RUN\n",
    );

    let rejected = results_dir.join("rejected-report-artifacts");
    write_valid_replay_candidate(
        &rejected,
        "#!/usr/bin/env bash\necho REJECTED-MUST-NOT-RUN\n",
    );
    let rejected_provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(rejected.join("candidate/PROVENANCE")).unwrap()).unwrap();
    write_replay_report(&rejected, &rejected_provenance, "fail");

    let output = run_start_bench(
        &results_dir,
        &current_bin_dir,
        &["--identity-argument"],
        b"2\n",
    );

    assert!(
        output.status.success(),
        "admitted candidate replay failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("  1) z-newer-candidate"));
    assert!(stdout.contains("  2) a-selected-candidate"));
    assert!(stdout.contains(&format!(
        "SELECTED-CANDIDATE {selected_identity} --identity-argument"
    )));
    for forbidden in [
        "generic-runner",
        "report-with-absent-runtime",
        "incomplete-runtime",
        "stale-runtime",
        "rejected-report",
        "CURRENT-BINARY-MUST-NOT-RUN",
        "GENERIC-RUNNER-MUST-NOT-RUN",
        "NEWER-CANDIDATE-MUST-NOT-RUN",
        "INCOMPLETE-MUST-NOT-RUN",
        "REPLACED-STALE-MUST-NOT-RUN",
        "REJECTED-MUST-NOT-RUN",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "non-admitted or non-selected runtime appeared in output: {forbidden}\n{stdout}"
        );
    }
}

#[test]
fn start_script_bench_mode_keeps_displayed_numbering_when_inventory_changes() {
    let root = temp_dir("start-bench-stable-inventory");
    let results_dir = root.path().join("bench-results");
    let current_bin_dir = root.path().join("current-bin");
    fs::create_dir_all(&current_bin_dir).unwrap();
    write_executable(
        &current_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho CURRENT-BINARY-MUST-NOT-RUN\n",
    );
    write_valid_replay_candidate(
        &results_dir.join("z-original-first-artifacts"),
        "#!/usr/bin/env bash\necho ORIGINAL-FIRST-MUST-NOT-RUN\n",
    );
    write_valid_replay_candidate(
        &results_dir.join("a-original-second-artifacts"),
        "#!/usr/bin/env bash\necho ORIGINAL-SECOND-RAN\n",
    );

    let (status, stdout, stderr) =
        run_start_bench_after_prompt(&results_dir, &current_bin_dir, b"2\n", || {
            write_valid_replay_candidate(
                &results_dir.join("zz-added-after-menu-artifacts"),
                "#!/usr/bin/env bash\necho ADDED-CANDIDATE-MUST-NOT-RUN\n",
            );
        });

    assert!(
        status.success(),
        "stable menu replay failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    );
    let stdout = String::from_utf8_lossy(&stdout);
    assert!(stdout.contains("  1) z-original-first"));
    assert!(stdout.contains("  2) a-original-second"));
    assert!(!stdout.contains("zz-added-after-menu"));
    assert!(stdout.contains("ORIGINAL-SECOND-RAN"));
    assert!(!stdout.contains("ORIGINAL-FIRST-MUST-NOT-RUN"));
    assert!(!stdout.contains("ADDED-CANDIDATE-MUST-NOT-RUN"));
}

#[test]
fn start_script_bench_mode_rejects_a_fully_admitted_path_replacement_after_listing() {
    let root = temp_dir("start-bench-artifact-replacement");
    let results_dir = root.path().join("bench-results");
    let current_bin_dir = root.path().join("current-bin");
    fs::create_dir_all(&current_bin_dir).unwrap();
    write_executable(
        &current_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho CURRENT-BINARY-MUST-NOT-RUN\n",
    );
    let displayed = results_dir.join("displayed-artifact");
    write_valid_replay_candidate(
        &displayed,
        "#!/usr/bin/env bash\necho ORIGINAL-DISPLAYED-CANDIDATE\n",
    );

    let (status, stdout, stderr) =
        run_start_bench_after_prompt(&results_dir, &current_bin_dir, b"1\n", || {
            fs::rename(&displayed, results_dir.join("original-moved-away")).unwrap();
            write_valid_replay_candidate(
                &displayed,
                "#!/usr/bin/env bash\necho VALID-REPLACEMENT-MUST-NOT-RUN\n",
            );
        });

    assert!(
        !status.success(),
        "a different admitted inode replaced the displayed choice\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    );
    assert!(!String::from_utf8_lossy(&stdout).contains("VALID-REPLACEMENT-MUST-NOT-RUN"));
    assert!(!String::from_utf8_lossy(&stdout).contains("CURRENT-BINARY-MUST-NOT-RUN"));
}

#[test]
fn start_script_bench_mode_never_snapshots_a_rejected_aba_replacement() {
    let root = temp_dir("start-bench-admission-snapshot-aba");
    let results_dir = root.path().join("bench-results");
    let displayed = results_dir.join("displayed-artifact");
    let accepted_hold = results_dir.join("accepted-hold");
    let rejected_hold = results_dir.join("rejected-hold");
    let current_bin_dir = root.path().join("current-bin");
    let wrapper_dir = root.path().join("wrapper-bin");
    fs::create_dir_all(&current_bin_dir).unwrap();
    fs::create_dir_all(&wrapper_dir).unwrap();
    write_executable(
        &current_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho CURRENT-BINARY-MUST-NOT-RUN\n",
    );
    write_valid_replay_candidate(
        &displayed,
        "#!/usr/bin/env bash\necho ACCEPTED-ABA-CANDIDATE-MUST-NOT-RUN\n",
    );
    write_valid_replay_candidate(
        &rejected_hold,
        "#!/usr/bin/env bash\necho REJECTED-ABA-CANDIDATE-MUST-NOT-RUN\n",
    );
    let rejected_provenance_path = rejected_hold.join("candidate/PROVENANCE");
    let mut rejected_provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(&rejected_provenance_path).unwrap()).unwrap();
    rejected_provenance["smoke"]["gate"] = serde_json::json!("rejected-smoke-gate");
    fs::write(
        &rejected_provenance_path,
        serde_json::to_vec_pretty(&rejected_provenance).unwrap(),
    )
    .unwrap();
    write_replay_report(&rejected_hold, &rejected_provenance, "pass");

    let real_python = Command::new("sh")
        .args(["-c", "command -v python3"])
        .output()
        .unwrap();
    assert!(real_python.status.success());
    let real_python = String::from_utf8(real_python.stdout)
        .unwrap()
        .trim()
        .to_owned();
    write_executable(
        &wrapper_dir.join("python3"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "if [[ \"${1:-}\" != */bench-candidate-replay ]]; then\n",
            "  exec \"$REAL_PYTHON3\" \"$@\"\n",
            "fi\n",
            "swap_to_rejected() {\n",
            "  mv -- \"$ABA_DISPLAYED\" \"$ABA_ACCEPTED_HOLD\"\n",
            "  mv -- \"$ABA_REJECTED_HOLD\" \"$ABA_DISPLAYED\"\n",
            "}\n",
            "restore_accepted() {\n",
            "  mv -- \"$ABA_DISPLAYED\" \"$ABA_REJECTED_HOLD\"\n",
            "  mv -- \"$ABA_ACCEPTED_HOLD\" \"$ABA_DISPLAYED\"\n",
            "}\n",
            "case \"${2:-}\" in\n",
            "  snapshot)\n",
            "    swap_to_rejected\n",
            "    set +e\n",
            "    \"$REAL_PYTHON3\" \"$@\"\n",
            "    status=$?\n",
            "    set -e\n",
            "    restore_accepted\n",
            "    exit \"$status\"\n",
            "    ;;\n",
            "  exec|exec-fd)\n",
            "    swap_to_rejected\n",
            "    exec \"$REAL_PYTHON3\" \"$@\"\n",
            "    ;;\n",
            "  *) exec \"$REAL_PYTHON3\" \"$@\" ;;\n",
            "esac\n",
        ),
    );
    let path = format!(
        "{}:{}",
        wrapper_dir.display(),
        std::env::var("PATH").unwrap()
    );
    let mut child = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("start"))
        .arg("--bench")
        .env("PATH", path)
        .env("REAL_PYTHON3", real_python)
        .env("ABA_DISPLAYED", &displayed)
        .env("ABA_ACCEPTED_HOLD", &accepted_hold)
        .env("ABA_REJECTED_HOLD", &rejected_hold)
        .env("WORK_LEAF_START_BENCH_RESULTS_DIR", &results_dir)
        .env("WORK_LEAF_START_BIN_DIR", &current_bin_dir)
        .env("WORK_LEAF_START_SKIP_BUILD", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"1\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(
        !output.status.success(),
        "a rejected ABA snapshot was listed and executed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("no runnable benchmark candidates found"),
        "the rejected snapshot was not excluded during discovery\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for forbidden in [
        "Available benchmark candidates:",
        "ACCEPTED-ABA-CANDIDATE-MUST-NOT-RUN",
        "REJECTED-ABA-CANDIDATE-MUST-NOT-RUN",
        "CURRENT-BINARY-MUST-NOT-RUN",
    ] {
        assert!(!String::from_utf8_lossy(&output.stdout).contains(forbidden));
    }
}

#[cfg(target_os = "linux")]
#[test]
fn start_script_bench_mode_transports_large_snapshots_outside_argv() {
    let root = temp_dir("start-bench-large-snapshot");
    let results_dir = root.path().join("bench-results");
    let current_bin_dir = root.path().join("current-bin");
    fs::create_dir_all(&current_bin_dir).unwrap();
    write_executable(
        &current_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho CURRENT-BINARY-MUST-NOT-RUN\n",
    );

    let artifact = results_dir.join("large-snapshot-artifacts");
    let candidate_bin = artifact.join("candidate/bin");
    fs::create_dir_all(&candidate_bin).unwrap();
    let entrypoint = candidate_bin.join("work-leaf");
    let cat = Command::new("sh")
        .args(["-c", "command -v cat"])
        .output()
        .unwrap();
    assert!(cat.status.success());
    let cat = PathBuf::from(String::from_utf8(cat.stdout).unwrap().trim());
    fs::copy(cat, &entrypoint).unwrap();
    let executable_digest = sha256_file(&entrypoint);
    let long_suffix = "x".repeat(210);
    let mut executable_names = vec!["work-leaf".to_owned()];
    for index in 0..365 {
        let name = format!("runtime-{index:04}-{long_suffix}");
        fs::hard_link(&entrypoint, candidate_bin.join(&name)).unwrap();
        executable_names.push(name);
    }
    let executable_digests = executable_names
        .iter()
        .map(|name| (name.as_str(), executable_digest.as_str()))
        .collect::<Vec<_>>();
    write_replay_candidate_metadata_with_digests(&artifact, &executable_digests, "pass");

    let snapshot =
        Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("bench-candidate-replay"))
            .arg("snapshot")
            .arg(&artifact)
            .output()
            .unwrap();
    assert!(
        snapshot.status.success(),
        "large admitted snapshot capture failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&snapshot.stdout),
        String::from_utf8_lossy(&snapshot.stderr)
    );
    assert!(
        snapshot.stdout.len() > 128 * 1024,
        "fixture snapshot did not exceed Linux's single-argument limit: {} bytes",
        snapshot.stdout.len()
    );

    let output = run_start_bench(
        &results_dir,
        &current_bin_dir,
        &[],
        b"1\nLARGE-SNAPSHOT-CANDIDATE-RAN\n",
    );
    assert!(
        output.status.success(),
        "large snapshot replay failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("LARGE-SNAPSHOT-CANDIDATE-RAN"));
    assert!(!stdout.contains("CURRENT-BINARY-MUST-NOT-RUN"));
}

#[cfg(target_os = "linux")]
#[test]
fn start_script_bench_mode_supervises_every_declared_runtime_exec_identity() {
    let root = temp_dir("start-bench-runtime-supervisor");
    let results_dir = root.path().join("bench-results");
    let current_bin_dir = root.path().join("current-bin");
    fs::create_dir_all(&current_bin_dir).unwrap();
    write_executable(
        &current_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho CURRENT-BINARY-MUST-NOT-RUN\n",
    );
    let artifact = results_dir.join("supervised-runtime-artifacts");
    let sibling = write_supervised_runtime_candidate(&artifact);
    let sibling_metadata = fs::metadata(&sibling).unwrap();
    let sibling_identity = format!("{}:{}", sibling_metadata.dev(), sibling_metadata.ino());

    let normal = run_start_bench(&results_dir, &current_bin_dir, &[], b"1\n");
    assert!(
        normal.status.success(),
        "normal two-runtime replay failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&normal.stdout),
        String::from_utf8_lossy(&normal.stderr)
    );
    assert!(
        String::from_utf8_lossy(&normal.stdout)
            .contains(&format!("ORIGINAL-SAVED-SIBLING {sibling_identity}"))
    );

    let nonzero = run_start_bench(&results_dir, &current_bin_dir, &["--exit-37"], b"1\n");
    assert_eq!(nonzero.status.code(), Some(37));
    let signaled = run_start_bench(&results_dir, &current_bin_dir, &["--signal-term"], b"1\n");
    assert_eq!(signaled.status.signal(), Some(15));

    let aba_malicious = artifact.join("aba-malicious-replacement");
    compile_rust_executable(
        &aba_malicious,
        r#"fn main() { println!("ABA-MALICIOUS-REPLACEMENT-RAN"); }"#,
    );
    let aba_trigger = root.path().join("launch-after-bin-aba");
    let aba_trigger_arg = aba_trigger.to_str().unwrap();
    let (aba_status, aba_stdout, aba_stderr) = run_start_bench_after_output(
        &results_dir,
        &current_bin_dir,
        &["--wait-for-sibling", aba_trigger_arg],
        "SIBLING-LAUNCH-READY",
        || {
            let original_bin = sibling.parent().unwrap();
            let moved_bin = artifact.join("candidate/aba-pinned-bin");
            let attacker_bin = artifact.join("candidate/bin");
            let retired_attacker_bin = artifact.join("candidate/aba-attacker-bin");
            fs::rename(original_bin, &moved_bin).unwrap();
            fs::create_dir(&attacker_bin).unwrap();
            fs::rename(&aba_malicious, attacker_bin.join("work-leaf-orchestrator")).unwrap();
            fs::rename(&attacker_bin, &retired_attacker_bin).unwrap();
            fs::rename(&moved_bin, original_bin).unwrap();
            fs::write(&aba_trigger, b"launch\n").unwrap();
        },
    );
    assert!(!aba_status.success());
    assert!(!String::from_utf8_lossy(&aba_stdout).contains("ABA-MALICIOUS-REPLACEMENT-RAN"));
    assert!(!String::from_utf8_lossy(&aba_stdout).contains("ORIGINAL-SAVED-SIBLING"));
    assert!(
        String::from_utf8_lossy(&aba_stderr)
            .contains("candidate runtime path changed during supervised replay")
    );

    let malicious = artifact.join("malicious-replacement");
    compile_rust_executable(
        &malicious,
        r#"fn main() { println!("MALICIOUS-REPLACEMENT-RAN"); }"#,
    );
    let trigger = root.path().join("launch-sibling");
    let trigger_arg = trigger.to_str().unwrap();
    let (status, stdout, stderr) = run_start_bench_after_output(
        &results_dir,
        &current_bin_dir,
        &["--wait-for-sibling", trigger_arg],
        "SIBLING-LAUNCH-READY",
        || {
            let relocated_bin = artifact.join("candidate/relocated-bin");
            fs::rename(sibling.parent().unwrap(), &relocated_bin).unwrap();
            fs::rename(&malicious, relocated_bin.join("work-leaf-orchestrator")).unwrap();
            fs::write(&trigger, b"launch\n").unwrap();
        },
    );
    assert!(
        !status.success(),
        "replaced sibling executed successfully\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    );
    let stdout = String::from_utf8_lossy(&stdout);
    let stderr = String::from_utf8_lossy(&stderr);
    assert!(stdout.contains("SIBLING-LAUNCH-READY"));
    assert!(!stdout.contains("MALICIOUS-REPLACEMENT-RAN"));
    assert!(!stdout.contains("ORIGINAL-SAVED-SIBLING"));
    assert!(
        stderr.contains("candidate runtime path changed during supervised replay"),
        "runtime identity rejection was not reported plainly:\n{stderr}"
    );
    assert!(!stdout.contains("CURRENT-BINARY-MUST-NOT-RUN"));
}

#[test]
fn start_script_bench_mode_handles_empty_and_invalid_choices_without_fallback() {
    let root = temp_dir("start-bench-menu-errors");
    let empty_results = root.path().join("empty-results");
    let current_bin_dir = root.path().join("current-bin");
    fs::create_dir_all(&empty_results).unwrap();
    fs::create_dir_all(&current_bin_dir).unwrap();
    write_executable(
        &current_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho CURRENT-BINARY-MUST-NOT-RUN\n",
    );
    let absent = empty_results.join("passing-report-without-runtime-artifacts");
    write_replay_candidate_metadata_with_digests(&absent, &[("work-leaf", "0".repeat(64))], "pass");

    let empty = run_start_bench(&empty_results, &current_bin_dir, &[], b"1\n");
    assert!(!empty.status.success());
    assert!(
        String::from_utf8_lossy(&empty.stderr).contains("no runnable benchmark candidates found"),
        "empty-inventory error was not plain:\n{}",
        String::from_utf8_lossy(&empty.stderr)
    );
    assert!(!String::from_utf8_lossy(&empty.stdout).contains("Choose a benchmark"));
    assert!(!String::from_utf8_lossy(&empty.stdout).contains("CURRENT-BINARY-MUST-NOT-RUN"));

    let one_result = root.path().join("one-result");
    write_valid_replay_candidate(
        &one_result.join("only-candidate-artifacts"),
        "#!/usr/bin/env bash\necho ONLY-CANDIDATE-MUST-NOT-RUN\n",
    );
    for (input, expected) in [
        (&b""[..], "no benchmark selection received"),
        (&b"0\n"[..], "invalid benchmark selection: 0"),
        (
            &b"not-a-number\n"[..],
            "invalid benchmark selection: not-a-number",
        ),
        (&b"2\n"[..], "invalid benchmark selection: 2"),
        (
            &b"999999999999999999999999999999999999999999\n"[..],
            "invalid benchmark selection",
        ),
    ] {
        let output = run_start_bench(&one_result, &current_bin_dir, &[], input);
        assert!(
            !output.status.success(),
            "invalid input unexpectedly launched a candidate: {:?}",
            String::from_utf8_lossy(input)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "invalid input did not produce {expected:?}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("ONLY-CANDIDATE-MUST-NOT-RUN"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("CURRENT-BINARY-MUST-NOT-RUN"));
    }
}

#[test]
fn bench_candidate_replay_rejects_boolean_snapshot_integers() {
    let root = temp_dir("bench-replay-boolean-integer");
    let artifact = root.path().join("candidate-artifacts");
    write_valid_replay_candidate(
        &artifact,
        "#!/usr/bin/env bash\necho BOOLEAN-SNAPSHOT-MUST-NOT-RUN\n",
    );
    let replay = Path::new(env!("CARGO_MANIFEST_DIR")).join("bench-candidate-replay");
    let captured = Command::new(&replay)
        .arg("snapshot")
        .arg(&artifact)
        .output()
        .unwrap();
    assert!(captured.status.success());
    let mut snapshot: serde_json::Value = serde_json::from_slice(&captured.stdout).unwrap();
    snapshot["directories"][0]["mode"] = serde_json::Value::Bool(true);
    let snapshot_path = root.path().join("boolean-snapshot.json");
    fs::write(&snapshot_path, serde_json::to_vec(&snapshot).unwrap()).unwrap();

    let output = Command::new("bash")
        .arg("-c")
        .arg("exec 9<\"$2\"; exec \"$1\" exec-fd 9")
        .arg("bench-replay-boolean-fixture")
        .arg(&replay)
        .arg(&snapshot_path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("candidate directory snapshot has an invalid mode")
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("BOOLEAN-SNAPSHOT-MUST-NOT-RUN"));
}

#[test]
fn start_script_bench_mode_never_lists_an_unverifiable_script_companion() {
    let root = temp_dir("bench-replay-script-companion");
    let results_dir = root.path().join("bench-results");
    let artifact = results_dir.join("script-companion-artifacts");
    let candidate_bin = artifact.join("candidate/bin");
    fs::create_dir_all(&candidate_bin).unwrap();
    write_executable(
        &candidate_bin.join("work-leaf"),
        "#!/usr/bin/env bash\necho ENTRYPOINT-MUST-NOT-RUN\n",
    );
    write_executable(
        &candidate_bin.join("work-leaf-orchestrator"),
        "#!/usr/bin/env bash\necho SCRIPT-COMPANION-MUST-NOT-RUN\n",
    );
    write_replay_candidate_metadata(&artifact, &["work-leaf", "work-leaf-orchestrator"], "pass");
    let current_bin_dir = root.path().join("current-bin");
    fs::create_dir_all(&current_bin_dir).unwrap();
    write_executable(
        &current_bin_dir.join("work-leaf"),
        "#!/usr/bin/env bash\necho CURRENT-BINARY-MUST-NOT-RUN\n",
    );

    let output = run_start_bench(&results_dir, &current_bin_dir, &[], b"1\n");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("no runnable benchmark candidates found")
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("ENTRYPOINT-MUST-NOT-RUN"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("SCRIPT-COMPANION-MUST-NOT-RUN"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("CURRENT-BINARY-MUST-NOT-RUN"));
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
    assert!(script.contains(":new implement strict selected-agent slash command execution"));
    assert!(script.contains("normal agent send/resume/model prompt path is insufficient"));
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

#[test]
fn three_feature_bench_script_drives_default_http_benchmark_and_reports_results() {
    let script =
        fs::read_to_string("bench-three-features").expect("three-feature bench script exists");
    let mode = fs::metadata("bench-three-features")
        .expect("three-feature bench script is statable")
        .permissions()
        .mode();

    assert_ne!(mode & 0o111, 0, "bench script should be executable");
    assert!(script.contains("readonly base_commit=\"c92a0b7060a36eac6db2d869b85e589a7a9480f9\""));
    assert!(script.contains("prepare_base_checkout()"));
    assert!(!script.contains("prepare_worktree_checkout()"));
    assert!(!script.contains("copy_untracked_worktree_files()"));
    assert!(!script.contains("default_results_dir_relative_to_repo()"));
    assert!(!script.contains("WORK_LEAF_BENCH_WORKTREE_SNAPSHOT"));
    assert!(script.contains("WORK_LEAF_BENCH_BASE is not supported"));
    assert!(script.contains("worktree_source_commit"));
    assert!(script.contains("WORK_LEAF_BENCH_SOURCE_HEAD"));
    assert!(!script.contains("WORK_LEAF_BENCH_BASE:-"));
    assert!(script.contains("fetch_state_snapshot()"));
    assert!(script.contains("best_state_snapshot()"));
    assert!(script.contains("curl -fsS \"$url/state\" > \"$next\""));
    assert!(script.contains("state_snapshot_is_valid \"$next\""));
    assert!(script.contains("mv \"$next\" \"$destination\""));
    assert!(script.contains("state_snapshot_is_valid \"$tmp_root/state.json\" && return 0"));
    assert!(script.contains("session_summary \"$report_state_file\""));
    assert!(!script.contains("cp \"$tmp_root/state.json\" \"$tmp_root/final-state.json\""));
    assert!(script.contains(
        "fetch_state_snapshot \"$tmp_root/state.json\" || fail_bench \"orchestrator state request failed\""
    ));
    assert!(!script.contains("curl -fsS \"$url/state\" > \"$tmp_root/state.json\""));
    assert!(!script.contains("curl -fsS \"$url/state\" > \"$tmp_root/final-state.json\""));
    assert!(script.contains("bench-results"));
    assert!(script.contains("results_dir=\"$(cd \"$results_dir\" && pwd)\""));
    assert!(script.contains(
        "WORK_LEAF_BENCH_BUSY_STALL_SECS no-progress timeout while agents are busy, default 1800"
    ));
    assert!(script.contains(
        "WORK_LEAF_BENCH_IDLE_STALL_SECS no-progress timeout while agents are idle, default 300"
    ));
    assert!(script.contains("index_path=\"$results_dir/${report_stem}.jsonl\""));
    assert!(script.contains("WORK_LEAF_BENCH_WEB_UI"));
    assert!(script.contains("web-ui/serve.py"));
    assert!(script.contains("WORK_LEAF_BENCH_SUPERVISED=1"));
    assert!(script.contains("WORK_LEAF_BENCH_RUN_ID"));
    assert!(script.contains("tmux new-session -d -s"));
    assert!(script.contains("\"bash -lc $(shell_quote \"$command_text\")\""));
    assert!(script.contains("${report_stem}-supervisor.log"));
    assert!(script.contains("${report_stem}-supervisor.status"));
    assert!(script.contains("${report_stem}-supervisor.command"));
    assert!(script.contains("work-leaf-orchestrator"));
    assert!(script.contains("$artifact_dir/runner-bin"));
    assert!(script.contains(
        "cp --no-dereference -- \"$bin_dir/$binary\" \"$artifact_dir/runner-bin/$binary\""
    ));
    assert!(script.contains("sha256sum * > SHA256SUMS"));
    assert!(script.contains("save_repo_snapshot()"));
    assert!(script.contains("$artifact_dir/patches/$safe_label"));
    assert!(script.contains("implement strict selected-agent slash command execution. When a selected agent chat message starts with / followed by a non-whitespace command token, Work Leaf must treat it as a backend command for that selected agent, execute it immediately, and append the backend command output in the chat. This is not the existing raw pass-through behavior: passing /status to the normal agent send/resume/model prompt path is insufficient and must be covered by a failing test. Add coverage for /status and /fork, including a test that proves the normal backend send path does not receive /status as an ordinary prompt."));
    assert!(
        script.contains("format-patch --no-signature -o \"$patch_dir\" \"$base_commit\"..HEAD")
    );
    assert!(
        script.contains("bundle create \"$snapshot_dir/commits.bundle\" \"$base_commit\"..HEAD")
    );
    assert!(script.contains("patch_artifacts"));
    assert!(script.contains("WORK_LEAF_CODEX_TRACE=1"));
    assert!(script.contains(
        "exec env WORK_LEAF_OBSERVER_PRIMARY_MARKER=\"$observer_primary_marker\" WORK_LEAF_CONTEXT_BUNDLE_DIR=\"$tmp_root/context-bundles\" WORK_LEAF_COMMAND_TMPDIR=\"$child_tmp_dir\" WORK_LEAF_CODEX_TRACE=1 WORK_LEAF_CODEX_LINEARIZE_SANDBOX=danger-full-access"
    ));
    assert!(!script.contains("ensure_codex_sdk_python"));
    assert!(!script.contains("codex-sdk-venv"));
    assert!(!script.contains("sdk-install.log"));
    assert!(!script.contains("codex-sdk-python.txt"));
    assert!(!script.contains("openai-codex"));
    assert!(script.contains("redact_sensitive_env()"));
    assert!(script.contains("<redacted>"));
    assert!(script.contains("TMPDIR=\"$child_tmp_dir\""));
    assert!(script.contains("local next=\"${destination}.next\""));
    assert!(script.contains("daemon-env.txt"));
    assert!(script.contains("daemon-ps.txt"));
    assert!(script.contains("web-ui.out"));
    assert!(script.contains("web-ui.err"));
    assert!(script.contains("/proc/$daemon_pid/environ"));
    assert!(script.contains("abort-reason"));
    assert!(script.contains("work-leaf-codex-wrapper"));
    assert!(script.contains("-name \"${daemon_pid}-*\""));
    assert!(script.contains("review_completed"));
    assert!(script.contains("select(.id|startswith(\"review-\"))"));
    assert!(!script.contains(".feature|test(\"review\""));
    assert!(!script.contains(".title|test(\"review\""));
    assert!(script.contains("done_users="));
    assert!(script.contains("terminal_users="));
    assert!(script.contains("\"$terminal_users\" == \"$user_count\""));
    assert!(!script.contains("\"$done_users\" == \"3\""));
    assert!(script.contains("patch_agents_with_commits="));
    assert!(script.contains("expected_final_commits="));
    assert!(script.contains("if [[ \"$patch_agents_with_commits\" != \"3\" ]]; then"));
    assert!(script.contains("expected all three patch agents to produce reviewed commits"));
    assert!(
        !script.contains("completed without reviewed commits; running checks without linearize")
    );
    assert!(script.contains("linearize_completed"));
    assert!(script.contains("post_command 'force-linearize'"));
    assert!(script.contains("linearize_started=1"));
    assert!(script.contains("post_agent 'linearize' 'Accept the proposed linearization plan."));
    assert!(script.contains("accepted_linearize=1\n    sleep 5\n    continue"));
    assert!(script.contains("token_usage"));
    assert!(script.contains("$session.token_usage.input_tokens"));
    assert!(script.contains("compute_token_model_fit()"));
    assert!(script.contains("baseline-manifest.json"));
    assert!(script.contains("token_model_status"));
    assert!(script.contains("token_model_delta_tokens"));
    assert!(script.contains("token_model_rerun"));
    assert!(script.contains("## Token Model Fit"));
    assert!(script.contains("rerun recommendation: $token_model_rerun"));
    assert!(script.contains("outside fitted central 98%"));
    assert!(!script.contains("post_agent \"$session_id\" \"/status\""));
    assert!(script.contains("code_quality"));
    assert!(script.contains("agent_backend: codex"));
    assert!(script.contains("agent_transport: app-server"));
    assert!(script.contains("codex_cli_path: $codex_cli_path"));
    assert!(script.contains("codex_cli_version: $codex_cli_version"));
    assert!(script.contains("detect_codex_cli_version()"));
    assert!(script.contains("codex --version"));
    assert!(script.contains("agent_model"));
    assert!(script.contains("agent_model_source"));
    assert!(script.contains("agent_reasoning_effort"));
    assert!(script.contains("agent_reasoning_effort_source"));
    assert!(script.contains("read_codex_config_value"));
    assert!(!script.contains("bench_model=\"unknown\""));
    assert!(script.contains("daemon_args+=(\"--model\" \"$bench_model\")"));
    assert!(script.contains("requested_agent_model"));
    assert!(script.contains("detect_codex_model()"));
    assert!(!script.contains("ConfigReadResponse"));
    assert!(!script.contains("\"config/read\""));
    assert!(script.contains("no_read_permission"));
    assert!(script.contains("read_permission_mode"));
    assert!(script.contains("WORK_LEAF_BENCH_NO_READ_PERMISSION"));
    assert!(script.contains("daemon_args+=(\"--no-read-permission\")"));
    assert!(script.contains("changed_files"));
    assert!(script.contains("changed_lines_total"));
    assert!(script.contains("benched_binary_commit"));
    assert!(script.contains("rm -rf \"$tmp_root\""));
    assert!(script.contains("bench exited unexpectedly with status"));
    assert!(script.contains("orchestrator state request failed"));
    assert!(script.contains("-mmin +10"));
    assert!(!script.contains("WORK_LEAF_BENCH_KEEP_TEMP"));
}

#[test]
fn three_feature_bench_script_cleans_temp_checkout_and_writes_dry_run_report() {
    let root = temp_dir("three-feature-bench-dry-run");
    let results = root.path().join("results");
    let output = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("bench-three-features"))
        .arg("--dry-run")
        .env("WORK_LEAF_BENCH_TMPDIR", root.path())
        .env("WORK_LEAF_BENCH_RESULTS_DIR", &results)
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WORK_LEAF_BENCH_BASE=c92a0b7060a36eac6db2d869b85e589a7a9480f9"));
    let temp_root = stdout
        .lines()
        .find_map(|line| {
            line.split_once("WORK_LEAF_BENCH_TEMP=")
                .map(|(_, path)| PathBuf::from(path))
        })
        .expect("dry run should print temp root");
    assert!(
        !temp_root.exists(),
        "bench script should remove dry-run temp root {temp_root:?}"
    );
    let reports = fs::read_dir(&results)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    assert!(
        reports.iter().any(|path| path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with("-three-feature-bench.md"))),
        "dry run should write a markdown bench report"
    );
    assert!(results.join("three-feature-bench.jsonl").exists());
}

#[test]
fn three_feature_bench_script_rejects_base_override() {
    let root = temp_dir("three-feature-bench-base-override");
    let results = root.path().join("results");
    let output = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("bench-three-features"))
        .arg("--dry-run")
        .env("WORK_LEAF_BENCH_BASE", "HEAD")
        .env("WORK_LEAF_BENCH_TMPDIR", root.path())
        .env("WORK_LEAF_BENCH_RESULTS_DIR", &results)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "base override must be rejected\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("WORK_LEAF_BENCH_BASE is not supported")
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

fn run_start_bench(
    results_dir: &Path,
    current_bin_dir: &Path,
    args: &[&str],
    input: &[u8],
) -> std::process::Output {
    let mut child = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("start"))
        .arg("--bench")
        .args(args)
        .env("WORK_LEAF_START_BENCH_RESULTS_DIR", results_dir)
        .env("WORK_LEAF_START_BIN_DIR", current_bin_dir)
        .env("WORK_LEAF_START_SKIP_BUILD", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

fn run_start_bench_after_prompt<F>(
    results_dir: &Path,
    current_bin_dir: &Path,
    choice: &[u8],
    after_prompt: F,
) -> (ExitStatus, Vec<u8>, Vec<u8>)
where
    F: FnOnce(),
{
    let mut child = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("start"))
        .arg("--bench")
        .env("WORK_LEAF_START_BENCH_RESULTS_DIR", results_dir)
        .env("WORK_LEAF_START_BIN_DIR", current_bin_dir)
        .env("WORK_LEAF_START_SKIP_BUILD", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let transcript = Arc::new(Mutex::new(Vec::new()));
    let reader_transcript = Arc::clone(&transcript);
    let reader = thread::spawn(move || {
        let mut chunk = [0_u8; 4096];
        loop {
            match stdout.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => reader_transcript
                    .lock()
                    .unwrap()
                    .extend_from_slice(&chunk[..count]),
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) => panic!("failed to read benchmark menu: {error}"),
            }
        }
    });

    let started = Instant::now();
    loop {
        let output = transcript.lock().unwrap().clone();
        if String::from_utf8_lossy(&output).contains("Choose a") {
            break;
        }
        if let Some(status) = child.try_wait().unwrap() {
            reader.join().unwrap();
            let stdout = transcript.lock().unwrap().clone();
            let mut stderr = Vec::new();
            child
                .stderr
                .take()
                .unwrap()
                .read_to_end(&mut stderr)
                .unwrap();
            panic!(
                "start exited before displaying a benchmark menu ({status})\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            );
        }
        if started.elapsed() >= Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            reader.join().unwrap();
            panic!(
                "start did not display a benchmark menu\n{}",
                String::from_utf8_lossy(&transcript.lock().unwrap())
            );
        }
        thread::sleep(Duration::from_millis(20));
    }

    after_prompt();
    child.stdin.take().unwrap().write_all(choice).unwrap();
    let status = child.wait().unwrap();
    reader.join().unwrap();
    let stdout = transcript.lock().unwrap().clone();
    let mut stderr = Vec::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_end(&mut stderr)
        .unwrap();
    (status, stdout, stderr)
}

fn run_start_bench_after_output<F>(
    results_dir: &Path,
    current_bin_dir: &Path,
    args: &[&str],
    expected_output: &str,
    after_output: F,
) -> (ExitStatus, Vec<u8>, Vec<u8>)
where
    F: FnOnce(),
{
    let mut child = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("start"))
        .arg("--bench")
        .args(args)
        .env("WORK_LEAF_START_BENCH_RESULTS_DIR", results_dir)
        .env("WORK_LEAF_START_BIN_DIR", current_bin_dir)
        .env("WORK_LEAF_START_SKIP_BUILD", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"1\n").unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let transcript = Arc::new(Mutex::new(Vec::new()));
    let reader_transcript = Arc::clone(&transcript);
    let reader = thread::spawn(move || {
        let mut chunk = [0_u8; 4096];
        loop {
            match stdout.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => reader_transcript
                    .lock()
                    .unwrap()
                    .extend_from_slice(&chunk[..count]),
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) => panic!("failed to read supervised candidate output: {error}"),
            }
        }
    });
    let started = Instant::now();
    loop {
        let output = transcript.lock().unwrap().clone();
        if String::from_utf8_lossy(&output).contains(expected_output) {
            break;
        }
        if let Some(status) = child.try_wait().unwrap() {
            reader.join().unwrap();
            let stdout = transcript.lock().unwrap().clone();
            let mut stderr = Vec::new();
            child
                .stderr
                .take()
                .unwrap()
                .read_to_end(&mut stderr)
                .unwrap();
            panic!(
                "start exited before {expected_output:?} ({status})\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            );
        }
        if started.elapsed() >= Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            reader.join().unwrap();
            panic!(
                "start did not emit {expected_output:?}\n{}",
                String::from_utf8_lossy(&transcript.lock().unwrap())
            );
        }
        thread::sleep(Duration::from_millis(20));
    }
    after_output();
    let status = child.wait().unwrap();
    reader.join().unwrap();
    let stdout = transcript.lock().unwrap().clone();
    let mut stderr = Vec::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_end(&mut stderr)
        .unwrap();
    (status, stdout, stderr)
}

fn write_valid_replay_candidate(artifact_dir: &Path, executable: &str) {
    let binary = artifact_dir.join("candidate/bin/work-leaf");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    write_executable(&binary, executable);
    write_replay_candidate_metadata(artifact_dir, &["work-leaf"], "pass");
}

fn write_supervised_runtime_candidate(artifact_dir: &Path) -> PathBuf {
    let candidate_bin = artifact_dir.join("candidate/bin");
    fs::create_dir_all(&candidate_bin).unwrap();
    compile_rust_executable(
        &candidate_bin.join("work-leaf"),
        r#"
use std::env;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().map(String::as_str) == Some("--exit-37") {
        std::process::exit(37);
    }
    if arguments.first().map(String::as_str) == Some("--signal-term") {
        unsafe extern "C" {
            fn raise(signal: i32) -> i32;
        }
        unsafe {
            raise(15);
        }
        std::process::exit(125);
    }
    if arguments.first().map(String::as_str) == Some("--wait-for-sibling") {
        let trigger = arguments.get(1).expect("trigger path");
        println!("SIBLING-LAUNCH-READY");
        io::stdout().flush().unwrap();
        let started = Instant::now();
        while !std::path::Path::new(trigger).exists() {
            assert!(started.elapsed() < Duration::from_secs(5), "trigger timeout");
            thread::sleep(Duration::from_millis(5));
        }
    }
    let runtime_directory = env::current_exe().unwrap().parent().unwrap().to_owned();
    env::set_current_dir(runtime_directory).unwrap();
    let status = Command::new("./work-leaf-orchestrator")
        .arg0("unrelated-argv-zero")
        .status()
        .unwrap();
    if !status.success() {
        std::process::exit(41);
    }
}
"#,
    );
    let sibling = candidate_bin.join("work-leaf-orchestrator");
    compile_rust_executable(
        &sibling,
        r#"
use std::fs;
use std::os::unix::fs::MetadataExt;

fn main() {
    let metadata = fs::metadata("/proc/self/exe").unwrap();
    println!("ORIGINAL-SAVED-SIBLING {}:{}", metadata.dev(), metadata.ino());
}
"#,
    );
    write_replay_candidate_metadata(
        artifact_dir,
        &["work-leaf", "work-leaf-orchestrator"],
        "pass",
    );
    sibling
}

fn compile_rust_executable(path: &Path, source_text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let source = path.with_extension("fixture.rs");
    fs::write(&source, source_text).unwrap();
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-name")
        .arg("replay_fixture")
        .arg("-O")
        .arg("-o")
        .arg(path)
        .arg(&source)
        .output()
        .unwrap();
    fs::remove_file(&source).unwrap();
    assert!(
        output.status.success(),
        "runtime fixture compilation failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_identity_replay_binary(path: &Path, label: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let source = path.parent().unwrap().join("identity-replay-fixture.rs");
    fs::write(
        &source,
        format!(
            concat!(
                "use std::env;\n",
                "use std::fs;\n",
                "use std::os::unix::fs::MetadataExt;\n",
                "fn main() {{\n",
                "    let metadata = fs::metadata(\"/proc/self/exe\").unwrap();\n",
                "    let args = env::args().skip(1).collect::<Vec<_>>().join(\" \" );\n",
                "    println!(\"{} {{}}:{{}} {{}}\", metadata.dev(), metadata.ino(), args);\n",
                "}}\n",
            ),
            label
        ),
    )
    .unwrap();
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-O")
        .arg("-o")
        .arg(path)
        .arg(&source)
        .output()
        .unwrap();
    fs::remove_file(&source).unwrap();
    assert!(
        output.status.success(),
        "identity fixture compilation failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_replay_candidate_metadata(artifact_dir: &Path, names: &[&str], result: &str) {
    let digests = names
        .iter()
        .map(|name| {
            (
                *name,
                sha256_file(&artifact_dir.join("candidate/bin").join(name)),
            )
        })
        .collect::<Vec<_>>();
    write_replay_candidate_metadata_with_digests(artifact_dir, &digests, result);
}

fn write_replay_candidate_metadata_with_digests<S: AsRef<str>>(
    artifact_dir: &Path,
    executables: &[(&str, S)],
    result: &str,
) {
    let candidate_dir = artifact_dir.join("candidate");
    fs::create_dir_all(&candidate_dir).unwrap();
    let mut executable_records = executables
        .iter()
        .map(|(name, digest)| ((*name).to_owned(), digest.as_ref().to_owned()))
        .collect::<Vec<_>>();
    executable_records.sort_by(|left, right| left.0.cmp(&right.0));
    let target_names = executable_records
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let targets = serde_json::to_vec(&target_names).unwrap();
    let smoke_stdout = b"fixture startup completed\n";
    let smoke_stderr = b"";
    let provenance = serde_json::json!({
        "schema": "work-leaf-benchmark-candidate-v2",
        "source_commit": "0123456789abcdef0123456789abcdef01234567",
        "entrypoint": "candidate/bin/work-leaf",
        "executables": executable_records.iter().map(|(name, digest)| serde_json::json!({
            "path": format!("candidate/bin/{name}"),
            "sha256": digest,
        })).collect::<Vec<_>>(),
        "targets": {
            "path": "candidate/SOURCE_TARGETS.json",
            "sha256": sha256_bytes(&targets),
        },
        "smoke": {
            "schema": "work-leaf-benchmark-candidate-smoke-v2",
            "gate": "bounded-startup-quit",
            "command": ["candidate/bin/work-leaf"],
            "stdin_sha256": sha256_bytes(b"quit\n"),
            "timeout_seconds": 20,
            "exit_code": 0,
            "stdout": {
                "path": "candidate/SMOKE.stdout",
                "sha256": sha256_bytes(smoke_stdout),
            },
            "stderr": {
                "path": "candidate/SMOKE.stderr",
                "sha256": sha256_bytes(smoke_stderr),
            },
        },
    });
    fs::write(
        candidate_dir.join("PROVENANCE"),
        serde_json::to_vec_pretty(&provenance).unwrap(),
    )
    .unwrap();
    fs::write(candidate_dir.join("SOURCE_TARGETS.json"), targets).unwrap();
    fs::write(candidate_dir.join("SMOKE.stdout"), smoke_stdout).unwrap();
    fs::write(candidate_dir.join("SMOKE.stderr"), smoke_stderr).unwrap();
    let checksums = executable_records
        .iter()
        .map(|(name, digest)| format!("{digest}  bin/{name}\n"))
        .collect::<String>();
    fs::write(candidate_dir.join("SHA256SUMS"), checksums).unwrap();
    write_replay_report(artifact_dir, &provenance, result);
}

fn write_replay_report(artifact_dir: &Path, provenance: &serde_json::Value, result: &str) {
    fs::write(
        artifact_dir.join("report.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "run_id": artifact_dir.file_name().unwrap().to_string_lossy(),
            "result": result,
            "commits_after_base": 3,
            "changed_files": 1,
            "candidate": provenance,
        }))
        .unwrap(),
    )
    .unwrap();
}

fn sha256_file(path: &Path) -> String {
    let output = Command::new("sha256sum").arg(path).output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
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
