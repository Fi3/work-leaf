#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use work_leaf_bench_observer::{
    CaptureConfig, JsonlFrame, analyze, archive_context_bundles, capture_git_checkpoint,
    extract_rollout_metadata, record_controller_usage, record_timeline,
};

fn initialize(root: &TempDir) -> std::path::PathBuf {
    initialize_for_condition(root, "work-leaf")
}

fn initialize_for_condition(root: &TempDir, condition: &str) -> std::path::PathBuf {
    initialize_for_condition_with_real_tools(
        root,
        condition,
        std::path::Path::new("/bin/sh"),
        std::path::Path::new("/bin/true"),
    )
}

fn initialize_for_condition_with_real_sh(
    root: &TempDir,
    condition: &str,
    real_sh: &std::path::Path,
) -> std::path::PathBuf {
    initialize_for_condition_with_real_tools(
        root,
        condition,
        real_sh,
        std::path::Path::new("/bin/true"),
    )
}

fn initialize_for_condition_with_real_tools(
    root: &TempDir,
    condition: &str,
    real_sh: &std::path::Path,
    real_cargo: &std::path::Path,
) -> std::path::PathBuf {
    let observer = env!("CARGO_BIN_EXE_bench-observer");
    let fixture = env!("CARGO_BIN_EXE_bench-observer-fixture");
    let output = Command::new(observer)
        .args([
            "init",
            "--root",
            root.path().to_str().unwrap(),
            "--study-id",
            "efficiency-causal-study",
            "--pair-id",
            "pair-proxy-test",
            "--condition",
            condition,
            "--run-id",
            "proxy-test",
            "--real-codex",
            fixture,
            "--real-sh",
            real_sh.to_str().unwrap(),
            "--real-cargo",
            real_cargo.to_str().unwrap(),
            "--base-commit",
            "base",
            "--experiment-commit",
            "experiment",
            "--model",
            "gpt-5.5",
            "--effort",
            "xhigh",
        ])
        .output()
        .expect("observer init runs");
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    root.path().join("observer-config.json")
}

fn content_digest(text: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv64:{hash:016x}; bytes:{}", text.len())
}

fn json_lines(values: impl IntoIterator<Item = serde_json::Value>) -> String {
    values
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn app_server_proxy_forwards_and_captures_exact_bytes() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/codex");
    let mut input = concat!(
        "{\"id\":1,\"method\":\"initialize\"}\n",
        "{\"id\":2,\"method\":\"thread/start\"}\n",
        "{\"method\":\"server/request\",\"params\":{\"id\":3}}\n",
        "fragment\0bytes\n",
    )
    .as_bytes()
    .to_vec();
    input.extend(std::iter::repeat_n(b'x', 128 * 1024));
    input.push(b'\n');

    let mut child = Command::new(proxy)
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDERR", "fixture-stderr")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        for chunk in input.chunks(3) {
            stdin.write_all(chunk).unwrap();
            stdin.flush().unwrap();
        }
    }
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, input);
    assert_eq!(output.stderr, b"fixture-stderr");

    let app_server = fs::read_dir(root.path().join("app-server"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(
        fs::read(app_server.join("client-to-server.raw")).unwrap(),
        input
    );
    assert_eq!(
        fs::read(app_server.join("server-to-client.raw")).unwrap(),
        input
    );
    assert_eq!(
        fs::read(app_server.join("server-stderr.raw")).unwrap(),
        b"fixture-stderr"
    );
    let capture_config = CaptureConfig::load(&config).unwrap();
    let _summary = analyze(&capture_config).unwrap();
    let frames = fs::read_to_string(app_server.join("frames.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<JsonlFrame>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(!frames.is_empty());
    assert!(
        frames
            .iter()
            .all(|frame| frame.received_monotonic_ns.is_some())
    );
    assert!(frames.iter().all(|frame| frame.received_unix_ns.is_some()));
}

#[test]
fn app_server_analysis_accepts_content_length_framing() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let proxy = root.path().join("proxy-bin/codex");
    let messages = [
        serde_json::json!({
            "id": 1,
            "method": "initialize",
            "params": {
                "clientInfo": {
                    "name": "work_leaf",
                    "title": "Work Leaf",
                    "version": "0.1.0",
                },
            },
        }),
        serde_json::json!({"method": "initialized", "params": {}}),
        serde_json::json!({
            "id": 2,
            "method": "thread/read",
            "params": {"threadId": "thread-session-only", "includeTurns": false},
        }),
    ];
    let input = messages
        .iter()
        .map(|message| {
            let body = message.to_string();
            format!("Content-Length: {}\r\n\r\n{body}", body.len())
        })
        .collect::<String>();

    let mut child = Command::new(proxy)
        .args(["app-server", "--stdio"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());

    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(
        summary.session_only_threads,
        ["thread-session-only".to_string()]
    );

    let app_server = fs::read_dir(root.path().join("app-server"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(
        fs::read(app_server.join("client-to-server.raw")).unwrap(),
        input.as_bytes()
    );
    let frames = fs::read_to_string(app_server.join("frames.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<JsonlFrame>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(frames.len(), messages.len());
    assert!(frames.iter().all(|frame| frame.parsed));
    assert_eq!(
        frames
            .iter()
            .filter_map(|frame| frame.method.as_deref())
            .collect::<Vec<_>>(),
        ["initialize", "initialized", "thread/read"]
    );
}

#[test]
fn app_server_and_exec_proxies_preserve_nonzero_status_and_exact_argv_bytes() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/codex");
    let input = b"binary\0stdin\n";
    for args in [
        vec!["app-server", "--listen", "stdio://", "argument with spaces"],
        vec!["exec", "--json", "-", "argument with spaces"],
    ] {
        let mut child = Command::new(&proxy)
            .args(&args)
            .env("WORK_LEAF_OBSERVER_CONFIG", &config)
            .env("WORK_LEAF_OBSERVER_FIXTURE_EXIT_CODE", "23")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(23));
        assert_eq!(output.stdout, input);
    }

    let mut starts = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path().join("start.json");
            serde_json::from_slice::<serde_json::Value>(&fs::read(path).unwrap()).unwrap()
        })
        .collect::<Vec<_>>();
    starts.sort_by_key(|start| {
        start
            .get("invocation_id")
            .and_then(serde_json::Value::as_str)
            .unwrap()
            .to_string()
    });
    assert_eq!(starts.len(), 2);
    for start in starts {
        let argument = start
            .get("argv")
            .and_then(serde_json::Value::as_array)
            .unwrap()
            .iter()
            .find(|argument| {
                argument.get("display").and_then(serde_json::Value::as_str)
                    == Some("argument with spaces")
            })
            .unwrap();
        assert_eq!(
            argument
                .get("bytes_hex")
                .and_then(serde_json::Value::as_str),
            Some("617267756d656e74207769746820737061636573")
        );
    }
}

#[test]
fn locked_shell_proxy_preserves_streams_and_exit_status() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/sh");
    let working = root.path().join("working");
    fs::create_dir(&working).unwrap();
    let wrapped = concat!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ",
        "(printf 'shell-out\\001%s|%s' \"$PWD\" \"$OBSERVER_TEST_VALUE\"; ",
        "printf shell-err >&2; exit 7) & ",
        "work_leaf_child=$!; wait $work_leaf_child"
    );
    let marker = CaptureConfig::load(&config)
        .unwrap()
        .primary_invocation_marker;
    let output = Command::new(proxy)
        .args(["-c", wrapped])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .env("WORK_LEAF_OBSERVER_PRIMARY_MARKER", marker)
        .env("OBSERVER_TEST_VALUE", "environment-preserved")
        .current_dir(&working)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    let expected_stdout = format!("shell-out\u{1}{}|environment-preserved", working.display());
    assert_eq!(output.stdout, expected_stdout.as_bytes());
    assert_eq!(output.stderr, b"shell-err");

    let command = fs::read_dir(root.path().join("locked-commands"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(
        fs::read(command.join("stdout.raw")).unwrap(),
        expected_stdout.as_bytes()
    );
    assert_eq!(fs::read(command.join("stderr.raw")).unwrap(), b"shell-err");
    let meta: serde_json::Value =
        serde_json::from_slice(&fs::read(command.join("meta.json")).unwrap()).unwrap();
    assert_eq!(
        meta.pointer("/start/cwd")
            .and_then(serde_json::Value::as_str),
        Some(working.to_str().unwrap())
    );
    assert!(
        meta.pointer("/start/process_group")
            .and_then(serde_json::Value::as_i64)
            .is_some()
    );
    assert_eq!(
        meta.pointer("/start/primary")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
}

#[test]
fn locked_shell_proxy_preserves_binary_stdin_and_large_output() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/sh");
    let wrapped = concat!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ",
        "(cat; head -c 131072 /dev/zero | tr '\\000' x) & ",
        "work_leaf_child=$!; wait $work_leaf_child"
    );
    let input = b"binary\0stdin\n";
    let mut direct = Command::new("/bin/sh")
        .args(["-c", wrapped])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    direct.stdin.take().unwrap().write_all(input).unwrap();
    let direct_output = direct.wait_with_output().unwrap();
    let mut child = Command::new(proxy)
        .args(["-c", wrapped])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status, direct_output.status);
    assert_eq!(output.stdout, direct_output.stdout);
    assert_eq!(output.stderr, direct_output.stderr);
    assert_eq!(output.stdout.len(), 131_072);

    let command = fs::read_dir(root.path().join("locked-commands"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(fs::read(command.join("stdin.raw")).unwrap(), input);
    assert_eq!(fs::read(command.join("stdout.raw")).unwrap(), output.stdout);
}

#[test]
fn unrecognized_shell_invocations_execute_without_capture() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/sh");
    let output = Command::new(proxy)
        .args(["-c", "printf ordinary-shell"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"ordinary-shell");
    assert!(!root.path().join("locked-commands").exists());
}

#[test]
fn codex_passthrough_records_informational_calls_and_rejects_unclassified_calls() {
    let informational_root = TempDir::new().unwrap();
    let informational_config = initialize(&informational_root);
    let version = Command::new(informational_root.path().join("proxy-bin/codex"))
        .arg("--version")
        .env("WORK_LEAF_OBSERVER_CONFIG", &informational_config)
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(version.stdout, b"bench-observer-fixture 1.0\n");
    let help = Command::new(informational_root.path().join("proxy-bin/codex"))
        .args(["exec", "resume", "--help"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &informational_config)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", "")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(help.status.success());
    for args in [
        &["doctor"][..],
        &["app-server", "--help"][..],
        &["app-server", "proxy", "--help"][..],
        &["app-server", "daemon", "--help"][..],
        &["app-server", "generate-ts", "--out", "/tmp/observer-ts"][..],
        &[
            "app-server",
            "generate-json-schema",
            "--out",
            "/tmp/observer-schema",
        ][..],
    ] {
        let output = Command::new(informational_root.path().join("proxy-bin/codex"))
            .args(args)
            .env("WORK_LEAF_OBSERVER_CONFIG", &informational_config)
            .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", "")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(output.status.success(), "informational argv {args:?}");
    }
    assert!(
        informational_root
            .path()
            .join("codex-passthrough.jsonl")
            .is_file()
    );
    assert!(!informational_root.path().join("exec-json").exists());
    assert!(!informational_root.path().join("app-server").exists());
    let config = CaptureConfig::load(&informational_config).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);

    let unclassified_root = TempDir::new().unwrap();
    let unclassified_config = initialize(&unclassified_root);
    let call = Command::new(unclassified_root.path().join("proxy-bin/codex"))
        .arg("remote-control")
        .env("WORK_LEAF_OBSERVER_CONFIG", &unclassified_config)
        .output()
        .unwrap();
    assert!(call.status.success());
    let config = CaptureConfig::load(&unclassified_config).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(!summary.capture_complete);
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.contains("unclassified Codex passthrough")),
        "{:?}",
        summary.errors
    );
}

#[test]
fn help_text_after_option_delimiter_remains_provider_capture() {
    let root = TempDir::new().unwrap();
    let config = initialize_for_condition(&root, "direct");
    let output = Command::new(root.path().join("proxy-bin/codex"))
        .args(["exec", "--json", "--", "--help"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", "")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(root.path().join("exec-json").is_dir());
    assert!(!root.path().join("codex-passthrough.jsonl").exists());
}

#[test]
fn initialized_proxy_is_executable_and_analysis_builds_inventory() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let loaded = CaptureConfig::load(&config).unwrap();
    assert_eq!(loaded.study_id, "efficiency-causal-study");
    assert_eq!(loaded.pair_id, "pair-proxy-test");
    assert!(loaded.created_monotonic_ns > 0);
    assert!(!loaded.primary_invocation_marker.is_empty());
    let manifest = fs::read_to_string(root.path().join("manifest.json")).unwrap();
    assert!(manifest.contains("\"study_id\": \"efficiency-causal-study\""));
    assert!(manifest.contains("\"pair_id\": \"pair-proxy-test\""));
    assert!(!manifest.contains(&loaded.primary_invocation_marker));
    let proxy = root.path().join("proxy-bin/codex");
    assert_ne!(
        fs::metadata(&proxy).unwrap().permissions().mode() & 0o111,
        0
    );

    let output = Command::new(&proxy)
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .env(
            "WORK_LEAF_OBSERVER_PRIMARY_MARKER",
            &loaded.primary_invocation_marker,
        )
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDERR", "")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());

    let analyzed = Command::new(env!("CARGO_BIN_EXE_bench-observer"))
        .args(["analyze", "--config", config.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        analyzed.status.success(),
        "analysis failed: {}",
        String::from_utf8_lossy(&analyzed.stderr)
    );
    assert!(root.path().join("process-invocations.jsonl").is_file());
    assert!(root.path().join("counterfactuals.jsonl").is_file());
    assert!(root.path().join("mechanism-summary.json").is_file());
    assert!(root.path().join("capture-audit.txt").is_file());
}

#[test]
fn initialization_preserves_the_real_cargo_symlink_entrypoint() {
    let root = TempDir::new().unwrap();
    let fixture_root = TempDir::new().unwrap();
    let dispatch_log = fixture_root.path().join("dispatch.log");
    let dispatcher = fixture_root.path().join("tool-dispatch");
    fs::write(
        &dispatcher,
        "#!/bin/sh\nprintf '%s\\n' \"$0\" > \"$WORK_LEAF_OBSERVER_CARGO_LOG\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&dispatcher).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&dispatcher, permissions).unwrap();
    let cargo_entrypoint = fixture_root.path().join("cargo");
    std::os::unix::fs::symlink(&dispatcher, &cargo_entrypoint).unwrap();
    let config_path = initialize_for_condition_with_real_tools(
        &root,
        "work-leaf",
        std::path::Path::new("/bin/sh"),
        &cargo_entrypoint,
    );
    let config = CaptureConfig::load(&config_path).unwrap();

    assert_eq!(
        config
            .real_cargo
            .as_deref()
            .and_then(std::path::Path::file_name)
            .and_then(std::ffi::OsStr::to_str),
        Some("cargo")
    );
    let output = Command::new(root.path().join("proxy-bin/cargo"))
        .arg("metadata")
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_CARGO_LOG", &dispatch_log)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        std::path::Path::new(fs::read_to_string(dispatch_log).unwrap().trim())
            .file_name()
            .and_then(std::ffi::OsStr::to_str),
        Some("cargo")
    );
}

#[test]
fn app_server_proxy_forwards_termination_and_records_child_signal() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/codex");
    let mut child = Command::new(proxy)
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _open_stdin = child.stdin.take().unwrap();
    for _ in 0..100 {
        if root.path().join("app-server").is_dir() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    unsafe {
        libc::kill(child.id() as i32, libc::SIGTERM);
    }
    let status = child.wait().unwrap();
    assert_eq!(status.signal(), Some(libc::SIGTERM));

    let invocation = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let end: serde_json::Value =
        serde_json::from_slice(&fs::read(invocation.join("end.json")).unwrap()).unwrap();
    assert_eq!(
        end.get("terminating_signal")
            .and_then(serde_json::Value::as_i64),
        Some(libc::SIGTERM as i64)
    );
}

#[test]
fn exec_proxy_forwards_termination_and_records_child_signal() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/codex");
    let mut child = Command::new(proxy)
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _open_stdin = child.stdin.take().unwrap();
    for _ in 0..100 {
        if root.path().join("exec-json").is_dir() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    unsafe {
        libc::kill(child.id() as i32, libc::SIGTERM);
    }
    let status = child.wait().unwrap();
    assert_eq!(status.signal(), Some(libc::SIGTERM));
    let end_path = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("end.json");
    let end: serde_json::Value = serde_json::from_slice(&fs::read(end_path).unwrap()).unwrap();
    assert_eq!(
        end.get("terminating_signal")
            .and_then(serde_json::Value::as_i64),
        Some(libc::SIGTERM as i64)
    );
}

#[test]
fn locked_shell_proxy_forwards_termination_to_the_work_leaf_process_group() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/sh");
    let wrapped = concat!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ",
        "(sleep 60) & work_leaf_child=$!; wait $work_leaf_child"
    );
    let mut command = Command::new(proxy);
    command
        .args(["-c", wrapped])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    let mut child = command.spawn().unwrap();
    for _ in 0..100 {
        if root.path().join("locked-commands").is_dir() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    unsafe {
        libc::kill(child.id() as i32, libc::SIGTERM);
    }
    let status = child.wait().unwrap();
    assert_eq!(status.signal(), Some(libc::SIGTERM));

    let invocation = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let end: serde_json::Value =
        serde_json::from_slice(&fs::read(invocation.join("end.json")).unwrap()).unwrap();
    assert_eq!(
        end.get("terminating_signal")
            .and_then(serde_json::Value::as_i64),
        Some(libc::SIGTERM as i64)
    );
}

#[test]
fn locked_shell_proxy_matches_direct_execution_under_timeout() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/sh");
    let wrapped = concat!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ",
        "(printf 'before-timeout\\000'; sleep 60) & ",
        "work_leaf_child=$!; wait $work_leaf_child"
    );
    let run = |program: &std::path::Path, observed: bool| {
        let mut command = Command::new("timeout");
        command
            .args(["--foreground", "0.2s"])
            .arg(program)
            .args(["-c", wrapped])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        if observed {
            command.env("WORK_LEAF_OBSERVER_CONFIG", &config);
        }
        command.output().unwrap()
    };
    let direct = run(std::path::Path::new("/bin/sh"), false);
    let observed = run(&proxy, true);
    assert_eq!(direct.status.code(), Some(124));
    assert_eq!(observed.status, direct.status);
    assert_eq!(observed.stdout, direct.stdout);
    assert_eq!(observed.stderr, direct.stderr);
    assert_eq!(observed.stdout, b"before-timeout\0");

    let invocation = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let end: serde_json::Value =
        serde_json::from_slice(&fs::read(invocation.join("end.json")).unwrap()).unwrap();
    assert_eq!(
        end.get("terminating_signal")
            .and_then(serde_json::Value::as_i64),
        Some(libc::SIGTERM as i64)
    );
}

#[test]
fn online_validation_budget_blocks_a_second_cargo_process_before_launch() {
    let root = TempDir::new().unwrap();
    let fixture_root = TempDir::new().unwrap();
    let cargo_log = fixture_root.path().join("cargo.log");
    let real_cargo = fixture_root.path().join("cargo-fixture");
    fs::write(
        &real_cargo,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$WORK_LEAF_OBSERVER_CARGO_LOG\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&real_cargo).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&real_cargo, permissions).unwrap();
    let config = initialize_for_condition_with_real_tools(
        &root,
        "work-leaf",
        std::path::Path::new("/bin/sh"),
        &real_cargo,
    );
    let proxy_dir = root.path().join("proxy-bin");
    let wrapped = concat!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ",
        "(cargo test --test focused first_case && ",
        "cargo test --test focused second_case) & ",
        "work_leaf_child=$!; wait $work_leaf_child"
    );

    let output = Command::new(proxy_dir.join("sh"))
        .args(["-c", wrapped])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .env("WORK_LEAF_OBSERVER_CARGO_LOG", &cargo_log)
        .env("PATH", &proxy_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(&cargo_log).unwrap(),
        "test --test focused first_case\n"
    );
    let violation = fs::read_to_string(root.path().join("validation-budget-violation.txt"))
        .expect("the blocked launch is recorded for the benchmark driver");
    assert!(violation.contains("at most one Cargo validation process"));
    assert!(violation.contains("test --test focused second_case"));
}

#[test]
fn online_validation_budget_blocks_a_broad_cargo_process_before_launch() {
    let root = TempDir::new().unwrap();
    let fixture_root = TempDir::new().unwrap();
    let cargo_log = fixture_root.path().join("cargo.log");
    let real_cargo = fixture_root.path().join("cargo-fixture");
    fs::write(
        &real_cargo,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$WORK_LEAF_OBSERVER_CARGO_LOG\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&real_cargo).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&real_cargo, permissions).unwrap();
    let config = initialize_for_condition_with_real_tools(
        &root,
        "work-leaf",
        std::path::Path::new("/bin/sh"),
        &real_cargo,
    );
    let proxy_dir = root.path().join("proxy-bin");
    let wrapped = concat!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ",
        "(cargo test) & work_leaf_child=$!; wait $work_leaf_child"
    );

    let output = Command::new(proxy_dir.join("sh"))
        .args(["-c", wrapped])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .env("WORK_LEAF_OBSERVER_CARGO_LOG", &cargo_log)
        .env("PATH", &proxy_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(!cargo_log.exists(), "broad validation reached real Cargo");
    let violation = fs::read_to_string(root.path().join("validation-budget-violation.txt"))
        .expect("the blocked launch is recorded for the benchmark driver");
    assert!(violation.contains("repository-wide cargo test is not a focused validation"));

    let analyzed = analyze(&CaptureConfig::load(&config).unwrap()).unwrap();
    assert!(
        analyzed.errors.iter().any(|error| error.contains(
            "online validation budget violation: repository-wide cargo test is not a focused validation"
        )),
        "{:?}",
        analyzed.errors
    );
}

#[test]
fn online_validation_budget_recognizes_global_options_aliases_and_unnamed_scopes() {
    for arguments in [
        &["--color", "never", "t"][..],
        &["+stable", "--color=never", "c"][..],
        &["--locked", "b"][..],
        &["--offline", "d"][..],
        &["--", "t"][..],
        &["check", "--workspace", "--lib"][..],
        &["test", "--all", "--doc"][..],
    ] {
        let root = TempDir::new().unwrap();
        let fixture_root = TempDir::new().unwrap();
        let cargo_log = fixture_root.path().join("cargo.log");
        let real_cargo = fixture_root.path().join("cargo-fixture");
        fs::write(
            &real_cargo,
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$WORK_LEAF_OBSERVER_CARGO_LOG\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&real_cargo).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&real_cargo, permissions).unwrap();
        let config = initialize_for_condition_with_real_tools(
            &root,
            "direct",
            std::path::Path::new("/bin/sh"),
            &real_cargo,
        );
        let parent_id = "direct-global-alias-parent";
        let parent_dir = root.path().join("invocations").join(parent_id);
        fs::create_dir_all(&parent_dir).unwrap();
        fs::write(
            parent_dir.join("start.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "invocation_id": parent_id,
                "executable": "codex",
                "capture_kind": "exec-json",
                "argv": [],
                "cwd": root.path(),
                "pid": 1,
                "parent_pid": null,
                "process_group": null,
                "parent_invocation_id": null,
                "primary": true,
                "role": "sequential-feature-1-implement",
                "start_monotonic_ns": 1,
                "start_unix_ns": 1,
                "real_executable": env!("CARGO_BIN_EXE_bench-observer-fixture"),
                "real_executable_sha256": "fixture",
            }))
            .unwrap(),
        )
        .unwrap();

        let output = Command::new(root.path().join("proxy-bin/cargo"))
            .args(arguments)
            .env("WORK_LEAF_OBSERVER_CONFIG", &config)
            .env("WORK_LEAF_OBSERVER_PARENT_INVOCATION", parent_id)
            .env("WORK_LEAF_OBSERVER_CARGO_LOG", &cargo_log)
            .output()
            .unwrap();

        assert!(
            !output.status.success(),
            "live proxy accepted broad validation: cargo {}",
            arguments.join(" ")
        );
        assert!(
            !cargo_log.exists(),
            "broad validation reached real Cargo: cargo {}",
            arguments.join(" ")
        );
    }
}

#[test]
fn online_validation_budget_counts_a_global_option_alias_as_the_one_process() {
    let root = TempDir::new().unwrap();
    let fixture_root = TempDir::new().unwrap();
    let cargo_log = fixture_root.path().join("cargo.log");
    let real_cargo = fixture_root.path().join("cargo-fixture");
    fs::write(
        &real_cargo,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$WORK_LEAF_OBSERVER_CARGO_LOG\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&real_cargo).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&real_cargo, permissions).unwrap();
    let config = initialize_for_condition_with_real_tools(
        &root,
        "direct",
        std::path::Path::new("/bin/sh"),
        &real_cargo,
    );
    let parent_id = "direct-global-alias-budget-parent";
    let parent_dir = root.path().join("invocations").join(parent_id);
    fs::create_dir_all(&parent_dir).unwrap();
    fs::write(
        parent_dir.join("start.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "invocation_id": parent_id,
            "executable": "codex",
            "capture_kind": "exec-json",
            "argv": [],
            "cwd": root.path(),
            "pid": 1,
            "parent_pid": null,
            "process_group": null,
            "parent_invocation_id": null,
            "primary": true,
            "role": "sequential-feature-1-implement",
            "start_monotonic_ns": 1,
            "start_unix_ns": 1,
            "real_executable": env!("CARGO_BIN_EXE_bench-observer-fixture"),
            "real_executable_sha256": "fixture",
        }))
        .unwrap(),
    )
    .unwrap();
    let cargo_proxy = root.path().join("proxy-bin/cargo");
    let run = |arguments: &[&str]| {
        Command::new(&cargo_proxy)
            .args(arguments)
            .env("WORK_LEAF_OBSERVER_CONFIG", &config)
            .env("WORK_LEAF_OBSERVER_PARENT_INVOCATION", parent_id)
            .env("WORK_LEAF_OBSERVER_CARGO_LOG", &cargo_log)
            .output()
            .unwrap()
    };

    assert!(
        run(&[
            "+stable",
            "--color=never",
            "t",
            "--test",
            "focused",
            "one_case",
        ])
        .status
        .success()
    );
    assert!(!run(&["--color", "never", "t"]).status.success());
    assert_eq!(
        fs::read_to_string(cargo_log).unwrap(),
        "+stable --color=never t --test focused one_case\n"
    );
    let violation = fs::read_to_string(root.path().join("validation-budget-violation.txt"))
        .expect("the hidden broad alias is recorded as the second validation");
    assert!(violation.contains("repository-wide cargo test is not a focused validation"));
    assert!(violation.contains("--color never t"));
}

#[test]
fn direct_iteration_budget_blocks_the_second_process_and_disables_for_linearization() {
    let root = TempDir::new().unwrap();
    let fixture_root = TempDir::new().unwrap();
    let cargo_log = fixture_root.path().join("cargo.log");
    let real_cargo = fixture_root.path().join("cargo-fixture");
    fs::write(
        &real_cargo,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$WORK_LEAF_OBSERVER_CARGO_LOG\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&real_cargo).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&real_cargo, permissions).unwrap();
    let config = initialize_for_condition_with_real_tools(
        &root,
        "direct",
        std::path::Path::new("/bin/sh"),
        &real_cargo,
    );
    let parent_id = "direct-iteration-parent";
    let parent_dir = root.path().join("invocations").join(parent_id);
    fs::create_dir_all(&parent_dir).unwrap();
    fs::write(
        parent_dir.join("start.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "invocation_id": parent_id,
            "executable": "codex",
            "capture_kind": "exec-json",
            "argv": [],
            "cwd": root.path(),
            "pid": 1,
            "parent_pid": null,
            "process_group": null,
            "parent_invocation_id": null,
            "primary": true,
            "role": "sequential-feature-1-implement",
            "start_monotonic_ns": 1,
            "start_unix_ns": 1,
            "real_executable": env!("CARGO_BIN_EXE_bench-observer-fixture"),
            "real_executable_sha256": "fixture",
        }))
        .unwrap(),
    )
    .unwrap();
    let cargo_proxy = root.path().join("proxy-bin/cargo");
    let run = |case: &str| {
        Command::new(&cargo_proxy)
            .args(["test", "--test", "focused", case])
            .env("WORK_LEAF_OBSERVER_CONFIG", &config)
            .env("WORK_LEAF_OBSERVER_PARENT_INVOCATION", parent_id)
            .env("WORK_LEAF_OBSERVER_CARGO_LOG", &cargo_log)
            .output()
            .unwrap()
    };

    assert!(run("first_case").status.success());
    assert!(!run("second_case").status.success());
    let disable = Command::new(env!("CARGO_BIN_EXE_bench-observer"))
        .args([
            "validation-budget",
            "--config",
            config.to_str().unwrap(),
            "--state",
            "disabled",
        ])
        .output()
        .unwrap();
    assert!(
        disable.status.success(),
        "disable failed: {}",
        String::from_utf8_lossy(&disable.stderr)
    );
    assert!(run("linearize_case").status.success());
    assert_eq!(
        fs::read_to_string(cargo_log).unwrap(),
        concat!(
            "test --test focused first_case\n",
            "test --test focused linearize_case\n"
        )
    );
}

#[test]
fn locked_shell_proxy_finalizes_before_work_leaf_timeout_escalation() {
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/sh");
    let wrapped = concat!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ",
        "(sleep 60) & work_leaf_child=$!; wait $work_leaf_child"
    );
    let mut command = Command::new(proxy);
    command
        .args(["-c", wrapped])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    let mut child = command.spawn().unwrap();
    let invocation = loop {
        if let Ok(mut entries) = fs::read_dir(root.path().join("invocations"))
            && let Some(Ok(entry)) = entries.next()
            && entry.path().join("start.json").is_file()
        {
            break entry.path();
        }
        thread::sleep(Duration::from_millis(5));
    };

    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGTERM);
    }
    let deadline = Instant::now() + Duration::from_millis(100);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            break child.wait().unwrap();
        }
        thread::sleep(Duration::from_millis(5));
    };

    assert_ne!(status.signal(), Some(libc::SIGKILL));
    assert!(
        invocation.join("end.json").is_file(),
        "proxy did not persist end metadata before timeout escalation"
    );
}

#[test]
fn concurrent_locked_shell_timeouts_all_publish_completion_before_escalation() {
    const INVOCATIONS: usize = 16;
    let root = TempDir::new().unwrap();
    let config = initialize(&root);
    let proxy = root.path().join("proxy-bin/sh");
    let wrapped = concat!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ",
        "(sleep 60) & work_leaf_child=$!; wait $work_leaf_child"
    );
    let mut children = (0..INVOCATIONS)
        .map(|_| {
            let mut command = Command::new(&proxy);
            command
                .args(["-c", wrapped])
                .env("WORK_LEAF_OBSERVER_CONFIG", &config)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .process_group(0);
            command.spawn().unwrap()
        })
        .collect::<Vec<_>>();
    let start_deadline = Instant::now() + Duration::from_secs(5);
    while fs::read_dir(root.path().join("invocations"))
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().join("start.json").is_file())
                .count()
        })
        .unwrap_or_default()
        != INVOCATIONS
        && Instant::now() < start_deadline
    {
        thread::sleep(Duration::from_millis(2));
    }

    for child in &children {
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGTERM);
        }
    }
    let escalation_deadline = Instant::now() + Duration::from_millis(100);
    while Instant::now() < escalation_deadline
        && children
            .iter_mut()
            .any(|child| child.try_wait().unwrap().is_none())
    {
        thread::sleep(Duration::from_millis(2));
    }
    for child in &mut children {
        if child.try_wait().unwrap().is_none() {
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
        }
    }
    let statuses = children
        .iter_mut()
        .map(|child| child.wait().unwrap())
        .collect::<Vec<_>>();

    assert!(
        statuses
            .iter()
            .all(|status| status.signal() != Some(libc::SIGKILL)),
        "at least one proxy exceeded the timeout escalation window: {statuses:?}"
    );
    let completions = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join("end.json").is_file())
        .count();
    assert_eq!(completions, INVOCATIONS);
}

#[test]
fn proxy_installs_termination_capture_before_slow_artifact_setup() {
    let root = TempDir::new().unwrap();
    let real_sh_root = TempDir::new().unwrap();
    let real_sh = real_sh_root.path().join("early-signal-sh");
    fs::write(
        &real_sh,
        concat!(
            "#!/bin/sh\n",
            "sleep 0.01\n",
            ": > \"$WORK_LEAF_OBSERVER_EARLY_SIGNAL_READY\"\n",
            "kill -STOP \"$PPID\"\n",
            "sleep 60\n",
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&real_sh).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&real_sh, permissions).unwrap();
    let config = initialize_for_condition_with_real_sh(&root, "work-leaf", &real_sh);

    let clutter = root.path().join("artifact-clutter");
    fs::create_dir(&clutter).unwrap();
    for index in 0..20_000 {
        fs::write(clutter.join(format!("entry-{index:05}")), []).unwrap();
    }

    let ready = root.path().join("early-signal-ready");
    let wrapped = concat!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; ",
        "(sleep 60) & work_leaf_child=$!; wait $work_leaf_child"
    );
    let mut command = Command::new(root.path().join("proxy-bin/sh"));
    command
        .args(["-c", wrapped])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config)
        .env("WORK_LEAF_OBSERVER_EARLY_SIGNAL_READY", &ready)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    let mut child = command.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready.is_file() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    assert!(
        ready.is_file(),
        "fixture did not stop the proxy during setup"
    );
    thread::sleep(Duration::from_millis(20));

    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGTERM);
        libc::kill(-(child.id() as i32), libc::SIGCONT);
    }
    let status = child.wait().unwrap();
    assert_eq!(status.signal(), Some(libc::SIGTERM));

    let invocation = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(
        invocation.join("end.json").is_file(),
        "termination during artifact setup lost completion metadata"
    );
}

#[test]
fn analyzer_rejects_tampered_raw_capture() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let output = Command::new(root.path().join("proxy-bin/codex"))
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env(
            "WORK_LEAF_OBSERVER_FIXTURE_STDOUT",
            "{\"type\":\"thread.started\",\"thread_id\":\"tamper-thread\"}\n",
        )
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = fs::read_dir(root.path().join("exec-json"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("stdout.raw");
    fs::write(stdout, b"tampered\n").unwrap();

    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(!summary.capture_complete);
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.contains("SHA-256 differs")),
        "{:?}",
        summary.errors
    );
}

#[test]
fn analyzer_rejects_missing_process_completion_metadata() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let output = Command::new(root.path().join("proxy-bin/codex"))
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", "")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    let invocation = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::remove_file(invocation.join("end.json")).unwrap();

    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(!summary.capture_complete);
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.contains("has no end metadata")),
        "{:?}",
        summary.errors
    );
}

#[test]
fn analyzer_waits_for_inflight_process_completion_metadata() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let mut child = Command::new(root.path().join("proxy-bin/codex"))
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let stdin = child.stdin.take().unwrap();
    for _ in 0..100 {
        if root
            .path()
            .join("invocations")
            .read_dir()
            .is_ok_and(|mut entries| {
                entries
                    .any(|entry| entry.is_ok_and(|entry| entry.path().join("start.json").is_file()))
            })
        {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let finisher = thread::spawn(move || {
        thread::sleep(Duration::from_millis(150));
        drop(stdin);
        child.wait().unwrap()
    });

    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(finisher.join().unwrap().success());
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.complete_invocation_count, 1);
}

#[test]
fn stop_app_server_finalizes_the_real_child_before_proxy_teardown() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let marker = CaptureConfig::load(&config_path)
        .unwrap()
        .primary_invocation_marker;
    let mut proxy = Command::new(root.path().join("proxy-bin/codex"))
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_PRIMARY_MARKER", marker)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let stdin = proxy.stdin.take().unwrap();
    for _ in 0..100 {
        if root
            .path()
            .join("invocations")
            .read_dir()
            .is_ok_and(|mut entries| {
                entries
                    .any(|entry| entry.is_ok_and(|entry| entry.path().join("start.json").is_file()))
            })
        {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    let stop = Command::new(env!("CARGO_BIN_EXE_bench-observer"))
        .args(["stop-app-server", "--config", config_path.to_str().unwrap()])
        .output()
        .unwrap();
    drop(stdin);
    let _ = proxy.wait().unwrap();

    assert!(
        stop.status.success(),
        "stop helper failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&stop.stdout),
        String::from_utf8_lossy(&stop.stderr)
    );
    let invocation = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(invocation.join("child.json").is_file());
    assert!(invocation.join("end.json").is_file());
    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.complete_invocation_count, 1);
}

#[test]
fn analyzer_rejects_secrets_in_raw_streams_and_hardens_artifact_permissions() {
    fn assert_user_only(path: &std::path::Path) {
        let metadata = fs::symlink_metadata(path).unwrap();
        if metadata.file_type().is_symlink() {
            return;
        }
        assert_eq!(
            metadata.permissions().mode() & 0o077,
            0,
            "{} is accessible to group or other users",
            path.display()
        );
        if metadata.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                assert_user_only(&entry.unwrap().path());
            }
        }
    }

    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let proxy = root.path().join("proxy-bin/codex");
    let output = Command::new(proxy)
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env(
            "WORK_LEAF_OBSERVER_FIXTURE_STDERR",
            "Authorization: Bearer observer-test-secret-1234567890",
        )
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());

    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(!summary.capture_complete);
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.contains("secret marker") && error.contains("stderr.raw")),
        "{:?}",
        summary.errors
    );
    assert_user_only(root.path());
}

#[test]
fn rollout_extraction_uses_only_observed_thread_ids_and_allowlisted_fields() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let proxy = root.path().join("proxy-bin/codex");
    let jsonl = concat!(
        "{\"type\":\"thread.started\",\"thread_id\":\"thread-observed\"}\n",
        "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":100,",
        "\"cached_input_tokens\":80,\"output_tokens\":10,",
        "\"reasoning_output_tokens\":5}}\n",
    );
    let output = Command::new(proxy)
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_ROLE", "feature-1")
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", jsonl)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    let config = CaptureConfig::load(&config_path).unwrap();
    assert!(analyze(&config).unwrap().capture_complete);

    let sessions = root.path().join("sessions/2026/07/31");
    fs::create_dir_all(&sessions).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let rollout = format!(
        "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-observed\",\
         \"cwd\":{},\"cli_version\":\"1.0\",\"secret\":\"must-not-copy\"}}}}\n\
         {{\"type\":\"turn_context\",\"payload\":{{\"cwd\":{},\"model\":\"gpt-5.5\",\
         \"effort\":\"xhigh\",\"user_prompt\":\"must-not-copy\"}}}}\n\
         {{\"type\":\"event_msg\",\"payload\":{{\"type\":\"token_count\",\"info\":{{\
         \"total_token_usage\":{{\"input_tokens\":100,\"cached_input_tokens\":80,\
         \"output_tokens\":10,\"reasoning_output_tokens\":5}}}}}}}}\n",
        serde_json::to_string(&cwd).unwrap(),
        serde_json::to_string(&cwd).unwrap(),
    );
    fs::write(sessions.join("rollout-thread-observed.jsonl"), rollout).unwrap();
    fs::write(
        sessions.join("rollout-unrelated.jsonl"),
        format!(
            "{{\"timestamp\":\"2020-01-01T00:00:00Z\",\"type\":\"session_meta\",\
             \"payload\":{{\"id\":\"thread-unrelated\",\"cwd\":{}}}}}\n",
            serde_json::to_string(&cwd).unwrap()
        ),
    )
    .unwrap();

    let audit = extract_rollout_metadata(&config, &root.path().join("sessions")).unwrap();
    assert!(audit.errors.is_empty(), "{:?}", audit.errors);
    assert_eq!(audit.matched_threads, 1);
    let metadata = fs::read_to_string(root.path().join("rollout-metadata.jsonl")).unwrap();
    assert!(metadata.contains("thread-observed"));
    assert!(metadata.contains("\"model\":\"gpt-5.5\""));
    assert!(!metadata.contains("must-not-copy"));
    assert!(!metadata.contains("thread-unrelated"));
}

#[test]
fn ordinary_ask_for_approval_argv_is_not_a_secret_marker() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let output = Command::new(root.path().join("proxy-bin/codex"))
        .args(["--ask-for-approval", "never", "exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", "")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
}

#[test]
fn secret_scan_requires_a_token_boundary_before_sk_prefixes() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let harmless = serde_json::json!({
        "type": "notice",
        "text": "ask-for-approvalOpenResumePicker",
    })
    .to_string()
        + "\n";
    let output = Command::new(root.path().join("proxy-bin/codex"))
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", harmless)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());

    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);

    let secret_root = TempDir::new().unwrap();
    let secret_config_path = initialize(&secret_root);
    let output = Command::new(secret_root.path().join("proxy-bin/codex"))
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &secret_config_path)
        .env(
            "WORK_LEAF_OBSERVER_FIXTURE_STDERR",
            "sk-proj-observer-fixture-1234567890",
        )
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", "")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    let secret_config = CaptureConfig::load(&secret_config_path).unwrap();
    let secret_summary = analyze(&secret_config).unwrap();
    assert!(!secret_summary.capture_complete);
    assert!(
        secret_summary
            .errors
            .iter()
            .any(|error| error.contains("secret marker sk-token")),
        "{:?}",
        secret_summary.errors
    );
}

#[test]
fn app_server_usage_resolves_thread_from_the_matching_turn_response() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let marker = CaptureConfig::load(&config_path)
        .unwrap()
        .primary_invocation_marker;
    let server = concat!(
        "{\"id\":7,\"result\":{\"turn\":{\"id\":\"turn-a\"}}}\n",
        "{\"method\":\"thread/tokenUsage/updated\",\"params\":{\"turnId\":\"turn-a\",",
        "\"tokenUsage\":{\"last\":{\"inputTokens\":100,\"cachedInputTokens\":80,",
        "\"outputTokens\":10,\"reasoningOutputTokens\":5},",
        "\"total\":{\"inputTokens\":100,\"cachedInputTokens\":80,",
        "\"outputTokens\":10,\"reasoningOutputTokens\":5}}}}\n",
    );
    let mut child = Command::new(root.path().join("proxy-bin/codex"))
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_PRIMARY_MARKER", marker)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", server)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b"{\"id\":7,\"method\":\"turn/start\",\"params\":{\"threadId\":\"thread-a\",\
              \"input\":[{\"type\":\"text\",\"text\":\"Agent-ID: user-1\"}]}}\n",
        )
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success());

    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.threads.len(), 1);
    assert_eq!(summary.threads[0].thread_id, "thread-a");
    assert!(summary.threads[0].visible);
    assert_eq!(summary.threads[0].usage.input_tokens, 100);
}

#[test]
fn app_server_session_snapshot_without_turn_is_not_generation_usage() {
    let root = TempDir::new().unwrap();
    let config_path = initialize_for_condition(&root, "direct");
    let server = concat!(
        "{\"id\":\"fork\",\"result\":{\"thread\":{\"id\":\"thread-fork\"}}}\n",
        "{\"method\":\"thread/tokenUsage/updated\",\"params\":{",
        "\"threadId\":\"thread-fork\",\"tokenUsage\":{",
        "\"total\":{\"inputTokens\":100,\"cachedInputTokens\":80,",
        "\"outputTokens\":10,\"reasoningOutputTokens\":5}}}}\n",
    );
    let mut child = Command::new(root.path().join("proxy-bin/codex"))
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", server)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b"{\"id\":\"fork\",\"method\":\"thread/fork\",\"params\":{\"threadId\":\"thread-source\"}}\n",
        )
        .unwrap();
    assert!(child.wait().unwrap().success());

    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert!(summary.threads.is_empty());
    assert_eq!(
        summary.session_only_threads,
        ["thread-fork".to_string(), "thread-source".to_string()]
    );
    assert_eq!(summary.usage_scopes.total_workflow.thread_count, 0);

    let sessions = root.path().join("sessions");
    fs::create_dir(&sessions).unwrap();
    let timestamp = String::from_utf8(
        Command::new("date")
            .arg("--iso-8601=seconds")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    fs::write(
        sessions.join("rollout-thread-fork.jsonl"),
        json_lines([serde_json::json!({
            "timestamp": timestamp.trim(),
            "type": "session_meta",
            "payload": {
                "id": "thread-fork",
                "cwd": std::env::current_dir().unwrap(),
                "cli_version": "bench-observer-fixture 1.0",
            },
        })]),
    )
    .unwrap();
    let audit = extract_rollout_metadata(&config, &sessions).unwrap();
    assert!(audit.errors.is_empty(), "{:?}", audit.errors);
    assert_eq!(audit.observed_threads, 0);
    assert_eq!(audit.matched_threads, 0);
    assert_eq!(audit.session_only_threads, summary.session_only_threads);
}

#[test]
fn controller_usage_reconciles_to_events_delivered_before_directive_interrupt() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let config = CaptureConfig::load(&config_path).unwrap();
    let server = json_lines([
        serde_json::json!({"id": 7, "result": {"turn": {"id": "turn-a"}}}),
        serde_json::json!({
            "method": "thread/tokenUsage/updated",
            "params": {
                "turnId": "turn-a",
                "tokenUsage": {
                    "last": {
                        "inputTokens": 10,
                        "cachedInputTokens": 8,
                        "outputTokens": 2,
                        "reasoningOutputTokens": 1,
                    },
                    "total": {
                        "inputTokens": 10,
                        "cachedInputTokens": 8,
                        "outputTokens": 2,
                        "reasoningOutputTokens": 1,
                    },
                },
            },
        }),
        serde_json::json!({
            "method": "item/completed",
            "params": {
                "threadId": "thread-a",
                "turnId": "turn-a",
                "item": {"type": "agentMessage", "text": "continue\n@work-leaf read src/lib.rs\n"},
            },
        }),
        serde_json::json!({
            "method": "thread/tokenUsage/updated",
            "params": {
                "turnId": "turn-a",
                "tokenUsage": {
                    "last": {
                        "inputTokens": 20,
                        "cachedInputTokens": 16,
                        "outputTokens": 3,
                        "reasoningOutputTokens": 2,
                    },
                    "total": {
                        "inputTokens": 30,
                        "cachedInputTokens": 24,
                        "outputTokens": 5,
                        "reasoningOutputTokens": 3,
                    },
                },
            },
        }),
        serde_json::json!({
            "method": "turn/completed",
            "params": {
                "threadId": "thread-a",
                "turnId": "turn-a",
                "turn": {"id": "turn-a", "status": "interrupted"},
            },
        }),
    ]);
    let mut child = Command::new(root.path().join("proxy-bin/codex"))
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env(
            "WORK_LEAF_OBSERVER_PRIMARY_MARKER",
            &config.primary_invocation_marker,
        )
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", server)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            concat!(
                "{\"id\":7,\"method\":\"turn/start\",\"params\":{\"threadId\":\"thread-a\",",
                "\"input\":[{\"type\":\"text\",\"text\":\"Agent-ID: user-1\"}]}}\n",
                "{\"id\":8,\"method\":\"turn/interrupt\",\"params\":{\"threadId\":\"thread-a\",",
                "\"turnId\":\"turn-a\"}}\n",
            )
            .as_bytes(),
        )
        .unwrap();
    assert!(child.wait().unwrap().success());

    let state = root.path().join("controller-state.json");
    fs::write(
        &state,
        serde_json::to_vec(&serde_json::json!({
            "snapshot": {"sessions": [{
                "id": "user-1",
                "token_usage": {
                    "input_tokens": 10,
                    "cached_input_tokens": 8,
                    "output_tokens": 2,
                    "reasoning_output_tokens": 1,
                },
            }]},
        }))
        .unwrap(),
    )
    .unwrap();
    record_controller_usage(&config, &state).unwrap();

    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.threads[0].usage.input_tokens, 30);
    assert_eq!(summary.usage_scopes.visible_role.input_tokens, 30);
    let reconciliation = &summary.controller_usage_reconciliation[0];
    assert!(reconciliation.controller_matches_replay);
    assert_eq!(
        reconciliation
            .controller_streamed_usage
            .unwrap()
            .input_tokens,
        10
    );
    assert_eq!(
        reconciliation
            .provider_largest_cumulative_usage
            .unwrap()
            .input_tokens,
        30
    );
}

#[test]
fn end_to_end_usage_reconciliation_separates_visible_hidden_and_descendant_threads() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let marker = CaptureConfig::load(&config_path)
        .unwrap()
        .primary_invocation_marker;
    let proxy = root.path().join("proxy-bin/codex");
    let server = concat!(
        "{\"id\":1,\"result\":{\"turn\":{\"id\":\"turn-user\"}}}\n",
        "{\"id\":2,\"result\":{\"turn\":{\"id\":\"turn-title\"}}}\n",
        "{\"method\":\"thread/tokenUsage/updated\",\"params\":{\"turnId\":\"turn-user\",",
        "\"tokenUsage\":{\"last\":{\"inputTokens\":100,\"cachedInputTokens\":80,",
        "\"outputTokens\":10,\"reasoningOutputTokens\":5},",
        "\"total\":{\"inputTokens\":100,\"cachedInputTokens\":80,",
        "\"outputTokens\":10,\"reasoningOutputTokens\":5}}}}\n",
        "{\"method\":\"thread/tokenUsage/updated\",\"params\":{\"turnId\":\"turn-title\",",
        "\"tokenUsage\":{\"last\":{\"inputTokens\":20,\"cachedInputTokens\":10,",
        "\"outputTokens\":2,\"reasoningOutputTokens\":1},",
        "\"total\":{\"inputTokens\":20,\"cachedInputTokens\":10,",
        "\"outputTokens\":2,\"reasoningOutputTokens\":1}}}}\n",
    );
    let mut app_server = Command::new(&proxy)
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_PRIMARY_MARKER", marker)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", server)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    app_server
        .stdin
        .take()
        .unwrap()
        .write_all(
            concat!(
                "{\"id\":1,\"method\":\"turn/start\",\"params\":{\"threadId\":\"thread-user\",",
                "\"input\":[{\"type\":\"text\",\"text\":\"Agent-ID: user-1\"}]}}\n",
                "{\"id\":2,\"method\":\"turn/start\",\"params\":{\"threadId\":\"thread-title\",",
                "\"input\":[{\"type\":\"text\",\"text\":\"Agent-ID: title-agent\"}]}}\n",
            )
            .as_bytes(),
        )
        .unwrap();
    assert!(app_server.wait().unwrap().success());
    let parent_invocation = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .file_name()
        .to_string_lossy()
        .to_string();

    let nested = concat!(
        "{\"type\":\"thread.started\",\"thread_id\":\"thread-nested\"}\n",
        "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":30,",
        "\"cached_input_tokens\":0,\"output_tokens\":3,",
        "\"reasoning_output_tokens\":2}}\n",
    );
    let nested_output = Command::new(&proxy)
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_PARENT_INVOCATION", parent_invocation)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", nested)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(nested_output.status.success());

    let config = CaptureConfig::load(&config_path).unwrap();
    let controller_state = root.path().join("controller-state.json");
    fs::write(
        &controller_state,
        serde_json::to_vec(&serde_json::json!({
            "snapshot": {
                "sessions": [{
                    "id": "user-1",
                    "token_usage": {
                        "input_tokens": 100,
                        "cached_input_tokens": 80,
                        "output_tokens": 10,
                        "reasoning_output_tokens": 5,
                    },
                }],
            },
        }))
        .unwrap(),
    )
    .unwrap();
    record_controller_usage(&config, &controller_state).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.usage_scopes.visible_role.thread_count, 1);
    assert_eq!(summary.usage_scopes.visible_role.input_tokens, 100);
    assert_eq!(summary.usage_scopes.primary_condition.thread_count, 2);
    assert_eq!(summary.usage_scopes.primary_condition.input_tokens, 120);
    assert_eq!(summary.usage_scopes.total_workflow.thread_count, 3);
    assert_eq!(summary.usage_scopes.total_workflow.input_tokens, 150);

    let sessions = root.path().join("sessions/2026/07/31");
    fs::create_dir_all(&sessions).unwrap();
    let cwd = std::env::current_dir().unwrap();
    for (thread_id, model, effort, input, cached, output, reasoning) in [
        ("thread-user", "gpt-5.5", "xhigh", 100, 80, 10, 5),
        ("thread-title", "gpt-5.5", "xhigh", 20, 10, 2, 1),
        ("thread-nested", "gpt-5.6", "medium", 30, 0, 3, 2),
    ] {
        let rollout = [
            serde_json::json!({
                "timestamp": "2026-07-31T18:00:00+02:00",
                "type": "session_meta",
                "payload": {
                    "id": thread_id,
                    "cwd": cwd,
                    "cli_version": "1.0",
                },
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": { "cwd": cwd, "model": model, "effort": effort },
            }),
            serde_json::json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "total_token_usage": {
                            "input_tokens": input,
                            "cached_input_tokens": cached,
                            "output_tokens": output,
                            "reasoning_output_tokens": reasoning,
                        },
                    },
                },
            }),
        ]
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join("\n")
            + "\n";
        fs::write(sessions.join(format!("rollout-{thread_id}.jsonl")), rollout).unwrap();
    }
    let audit = extract_rollout_metadata(&config, &root.path().join("sessions")).unwrap();
    assert!(audit.errors.is_empty(), "{:?}", audit.errors);
    assert_eq!(audit.observed_threads, 3);
    assert_eq!(audit.matched_threads, 3);
    let final_summary = analyze(&config).unwrap();
    assert!(final_summary.capture_complete, "{:?}", final_summary.errors);
    assert_eq!(final_summary.model_strata.len(), 2);
    assert!(final_summary.model_strata.iter().any(|stratum| {
        stratum.model == "gpt-5.5"
            && stratum.effort == "xhigh"
            && stratum.thread_count == 2
            && stratum.primary_threads == 2
    }));
    assert!(final_summary.model_strata.iter().any(|stratum| {
        stratum.model == "gpt-5.6"
            && stratum.effort == "medium"
            && stratum.thread_count == 1
            && stratum.descendant_threads == 1
    }));
}

#[test]
fn rollout_recovers_captured_exec_thread_without_terminal_usage() {
    let root = TempDir::new().unwrap();
    let config_path = initialize_for_condition(&root, "direct");
    let config = CaptureConfig::load(&config_path).unwrap();
    let thread_id = "thread-rollout-only-usage";
    let events = json_lines([
        serde_json::json!({ "type": "thread.started", "thread_id": thread_id }),
        serde_json::json!({ "type": "turn.started" }),
        serde_json::json!({
            "type": "item.completed",
            "item": {
                "type": "agent_message",
                "text": "waiting for the nested real-agent check",
            },
        }),
    ]);
    let output = Command::new(root.path().join("proxy-bin/codex"))
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", events)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());

    let pre_rollout = analyze(&config).unwrap();
    assert!(pre_rollout.capture_complete, "{:?}", pre_rollout.errors);
    assert!(pre_rollout.threads.is_empty());

    let sessions = root.path().join("sessions");
    fs::create_dir(&sessions).unwrap();
    let cwd = std::env::current_dir().unwrap();
    fs::write(
        sessions.join("rollout-only-usage.jsonl"),
        json_lines([
            serde_json::json!({
                "type": "session_meta",
                "payload": { "id": thread_id, "cwd": cwd, "cli_version": "1.0" },
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": { "cwd": cwd, "model": "gpt-5.5", "effort": "xhigh" },
            }),
            serde_json::json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": { "total_token_usage": {
                        "input_tokens": 100,
                        "cached_input_tokens": 80,
                        "output_tokens": 10,
                        "reasoning_output_tokens": 5,
                    }},
                },
            }),
        ]),
    )
    .unwrap();

    let audit = extract_rollout_metadata(&config, &sessions).unwrap();
    assert!(audit.errors.is_empty(), "{:?}", audit.errors);
    assert_eq!(audit.observed_threads, 1);
    assert_eq!(audit.matched_threads, 1);

    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.threads.len(), 1);
    assert_eq!(summary.threads[0].thread_id, thread_id);
    assert_eq!(summary.usage_scopes.total_workflow.input_tokens, 100);
    assert_eq!(summary.usage_scopes.total_workflow.output_tokens, 10);
}

#[test]
fn rollout_recovers_newer_usage_after_output_free_resume() {
    let root = TempDir::new().unwrap();
    let config_path = initialize_for_condition(&root, "direct");
    let config = CaptureConfig::load(&config_path).unwrap();
    let proxy = root.path().join("proxy-bin/codex");
    let thread_id = "thread-output-free-resume";

    let initial = json_lines([
        serde_json::json!({ "type": "thread.started", "thread_id": thread_id }),
        serde_json::json!({
            "type": "turn.completed",
            "usage": {
                "input_tokens": 100,
                "cached_input_tokens": 80,
                "output_tokens": 10,
                "reasoning_output_tokens": 5,
            },
        }),
    ]);
    let output = Command::new(&proxy)
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", initial)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());

    let resumed = json_lines([
        serde_json::json!({ "type": "thread.started", "thread_id": thread_id }),
        serde_json::json!({ "type": "turn.started" }),
        serde_json::json!({
            "type": "item.completed",
            "item": {
                "type": "agent_message",
                "text": "@work-leaf patch ready\n@work-leaf end",
            },
        }),
    ]);
    let output = Command::new(&proxy)
        .args(["exec", "resume", "--json", thread_id, "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", resumed)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());

    let pre_rollout = analyze(&config).unwrap();
    assert!(pre_rollout.capture_complete, "{:?}", pre_rollout.errors);
    assert_eq!(pre_rollout.usage_scopes.total_workflow.input_tokens, 100);

    let sessions = root.path().join("sessions");
    fs::create_dir(&sessions).unwrap();
    let cwd = std::env::current_dir().unwrap();
    fs::write(
        sessions.join("rollout-output-free-resume.jsonl"),
        json_lines([
            serde_json::json!({
                "type": "session_meta",
                "payload": { "id": thread_id, "cwd": cwd, "cli_version": "1.0" },
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": { "cwd": cwd, "model": "gpt-5.5", "effort": "xhigh" },
            }),
            serde_json::json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": { "total_token_usage": {
                        "input_tokens": 250,
                        "cached_input_tokens": 200,
                        "output_tokens": 25,
                        "reasoning_output_tokens": 8,
                    }},
                },
            }),
        ]),
    )
    .unwrap();

    let audit = extract_rollout_metadata(&config, &sessions).unwrap();
    assert!(audit.errors.is_empty(), "{:?}", audit.errors);

    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.threads.len(), 1);
    assert_eq!(summary.usage_scopes.total_workflow.input_tokens, 250);
    assert_eq!(summary.usage_scopes.total_workflow.output_tokens, 25);
}

#[test]
fn direct_launch_and_resume_keep_only_final_cumulative_thread_usage() {
    let root = TempDir::new().unwrap();
    let config_path = initialize_for_condition(&root, "direct");
    let config = CaptureConfig::load(&config_path).unwrap();
    let proxy = root.path().join("proxy-bin/codex");
    let thread_id = "thread-direct-resume";

    for (role, args, input, cached, output) in [
        (
            "patch-1",
            vec!["exec", "--json", "-"],
            100_u64,
            80_u64,
            10_u64,
        ),
        (
            "patch-1-fix-1",
            vec!["exec", "resume", "--json", thread_id, "-"],
            250_u64,
            200_u64,
            25_u64,
        ),
    ] {
        let events = json_lines([
            serde_json::json!({ "type": "thread.started", "thread_id": thread_id }),
            serde_json::json!({
                "type": "turn.completed",
                "usage": {
                    "input_tokens": input,
                    "cached_input_tokens": cached,
                    "output_tokens": output,
                    "reasoning_output_tokens": 5,
                },
            }),
        ]);
        let invocation = Command::new(&proxy)
            .args(args)
            .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
            .env(
                "WORK_LEAF_OBSERVER_PRIMARY_MARKER",
                &config.primary_invocation_marker,
            )
            .env("WORK_LEAF_OBSERVER_ROLE", role)
            .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", events)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(invocation.status.success());
    }

    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.invocation_count, 2);
    assert_eq!(summary.threads.len(), 1);
    assert_eq!(summary.threads[0].usage.input_tokens, 250);
    assert_eq!(summary.usage_scopes.visible_role.input_tokens, 250);
    assert_eq!(summary.usage_scopes.primary_condition.input_tokens, 250);
    assert_eq!(summary.usage_scopes.total_workflow.input_tokens, 250);

    let sessions = root.path().join("sessions/2026/07/31");
    fs::create_dir_all(&sessions).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let rollout = |cli_version: &str, input: u64, cached_input: u64, output: u64| {
        json_lines([
            serde_json::json!({
                "type": "session_meta",
                "payload": { "id": thread_id, "cwd": cwd, "cli_version": cli_version },
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": { "cwd": cwd, "model": "gpt-5.5", "effort": "xhigh" },
            }),
            serde_json::json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": { "total_token_usage": {
                        "input_tokens": input,
                        "cached_input_tokens": cached_input,
                        "output_tokens": output,
                        "reasoning_output_tokens": 5,
                    }},
                },
            }),
        ])
    };
    fs::write(
        sessions.join("rollout-direct-initial.jsonl"),
        rollout("1.0", 100, 80, 10),
    )
    .unwrap();
    let rollout_path = sessions.join("rollout-direct-resume.jsonl");
    fs::write(&rollout_path, rollout("9.9", 250, 200, 25)).unwrap();
    let rejected = extract_rollout_metadata(&config, &root.path().join("sessions")).unwrap();
    assert!(
        rejected
            .errors
            .iter()
            .any(|error| error.contains("CLI version")),
        "{:?}",
        rejected.errors
    );
    fs::write(rollout_path, rollout("1.0", 250, 200, 25)).unwrap();
    let audit = extract_rollout_metadata(&config, &root.path().join("sessions")).unwrap();
    assert!(audit.errors.is_empty(), "{:?}", audit.errors);
    let reconciled = analyze(&config).unwrap();
    assert!(reconciled.capture_complete, "{:?}", reconciled.errors);
    assert_eq!(reconciled.model_strata.len(), 1);
    assert_eq!(reconciled.model_strata[0].thread_count, 1);
    assert_eq!(reconciled.model_strata[0].usage.input_tokens, 250);
}

#[test]
fn exec_command_completion_is_counted_once_after_started_event() {
    let root = TempDir::new().unwrap();
    let config_path = initialize_for_condition(&root, "direct");
    let config = CaptureConfig::load(&config_path).unwrap();
    let events = json_lines([
        serde_json::json!({
            "type": "thread.started",
            "thread_id": "thread-command-events",
        }),
        serde_json::json!({
            "type": "item.started",
            "item": {
                "type": "command_execution",
                "command": "cargo test --test focused one_case",
                "output": "",
            },
        }),
        serde_json::json!({
            "type": "item.completed",
            "item": {
                "type": "command_execution",
                "command": "cargo test --test focused one_case",
                "output": "ok",
            },
        }),
        serde_json::json!({
            "type": "turn.completed",
            "usage": {
                "input_tokens": 100,
                "cached_input_tokens": 80,
                "output_tokens": 10,
                "reasoning_output_tokens": 5,
            },
        }),
    ]);
    let output = Command::new(root.path().join("proxy-bin/codex"))
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env(
            "WORK_LEAF_OBSERVER_PRIMARY_MARKER",
            &config.primary_invocation_marker,
        )
        .env("WORK_LEAF_OBSERVER_ROLE", "feature-1-implement")
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", events)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());

    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.mechanisms.validation.validation_commands, 1);
    assert!(summary.mechanisms.validation.violations.is_empty());
}

#[test]
fn exec_turn_completion_resolves_terminal_directive_outcome() {
    let root = TempDir::new().unwrap();
    let config_path = initialize_for_condition(&root, "direct");
    let config = CaptureConfig::load(&config_path).unwrap();
    let events = json_lines([
        serde_json::json!({
            "type": "thread.started",
            "thread_id": "thread-terminal-directive",
        }),
        serde_json::json!({
            "type": "item.completed",
            "item": {
                "type": "agent_message",
                "text": "complete\n@work-leaf done\n",
            },
        }),
        serde_json::json!({
            "type": "turn.completed",
            "usage": {
                "input_tokens": 100,
                "cached_input_tokens": 80,
                "output_tokens": 10,
                "reasoning_output_tokens": 5,
            },
        }),
    ]);
    let output = Command::new(root.path().join("proxy-bin/codex"))
        .args(["exec", "--json", "-"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env(
            "WORK_LEAF_OBSERVER_PRIMARY_MARKER",
            &config.primary_invocation_marker,
        )
        .env("WORK_LEAF_OBSERVER_ROLE", "feature-1-implement")
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", events)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());

    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.mechanisms.terminal_directives.observed, 1);
    assert_eq!(
        summary.mechanisms.terminal_directives.naturally_completed,
        1
    );
    assert_eq!(summary.mechanisms.terminal_directives.unresolved, 0);
}

#[test]
fn raw_app_server_capture_drives_verified_mechanism_analysis() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let config = CaptureConfig::load(&config_path).unwrap();
    let proxy = root.path().join("proxy-bin");

    let command = "printf 'ok\\n'";
    let wrapper = format!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; \
         ({command}) & work_leaf_child=$!; wait $work_leaf_child"
    );
    let shell = Command::new(proxy.join("sh"))
        .args(["-c", &wrapper])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .output()
        .unwrap();
    assert!(shell.status.success());
    assert_eq!(shell.stdout, b"ok\n");

    let before = "before\n";
    let after = "after\n";
    let bundle_source = root.path().join("bundle-source/orchestrator-1");
    fs::create_dir_all(&bundle_source).unwrap();
    let bundle_path = bundle_source.join("bundle-0.md");
    let bundle_file_text = "bundle payload\n";
    let bundle = format!(
        "# Work Leaf Context Bundle\n\n\
         This file contains orchestrator-mediated read output.\n\n\
         ----- BEGIN FILE src/lib.rs -----\n\
         digest: {}\n\n\
         {bundle_file_text}\
         ----- END FILE src/lib.rs -----\n",
        content_digest(bundle_file_text)
    );
    fs::write(&bundle_path, &bundle).unwrap();
    assert_eq!(
        archive_context_bundles(&config, &root.path().join("bundle-source")).unwrap(),
        1
    );

    let repository = root.path().join("review-repository");
    fs::create_dir_all(&repository).unwrap();
    for args in [
        &["init", "-q"][..],
        &["config", "user.email", "observer@example.com"][..],
        &["config", "user.name", "Observer Test"][..],
    ] {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&repository)
                .status()
                .unwrap()
                .success()
        );
    }
    fs::write(repository.join("review.txt"), "first\n").unwrap();
    assert!(
        Command::new("git")
            .args(["add", "review.txt"])
            .current_dir(&repository)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["commit", "-q", "-m", "ADD first"])
            .current_dir(&repository)
            .status()
            .unwrap()
            .success()
    );
    let first_commit = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&repository)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    fs::write(repository.join("review.txt"), "second\n").unwrap();
    assert!(
        Command::new("git")
            .args(["commit", "-q", "-am", "FIX second"])
            .current_dir(&repository)
            .status()
            .unwrap()
            .success()
    );
    let second_commit = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&repository)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    capture_git_checkpoint(&config, &repository, "review-targets").unwrap();
    let second_short = &second_commit[..12];

    let prompts = [
        "Agent-ID: user-1".to_string(),
        format!("work-leaf file text\n\n--- README.md ---\n{before}"),
        format!(
            "work-leaf file text\n\nRepeated file reads unchanged\n\
             Work Leaf already sent this agent the exact text for these files.\n\
             - README.md ({})\n",
            content_digest(before)
        ),
        format!(
            "work-leaf file text\n\nRepeated file reads with changes\n\n\
             --- README.md ---\n\
             current digest: {}\n\
             previous digest: {}\n\
             status: changed since this agent's last snapshot\n\
             diff --git a/README.md b/README.md\n\
             --- a/README.md\n\
             +++ b/README.md\n\
             @@ -1 +1 @@\n\
             -before\n\
             +after\n",
            content_digest(after),
            content_digest(before)
        ),
        format!(
            "work-leaf command result\ncommand: {command}\nstatus: 0\n\
             locked paths: .\nstdout:\nok\nstderr:\n<empty>\n"
        ),
        format!(
            "work-leaf file text\n\
             Exact file text is in an orchestrator context bundle instead of this chat.\n\
             Context bundle: {}\nBundled files:\n- src/lib.rs ({})\n",
            bundle_path.display(),
            content_digest(bundle_file_text)
        ),
        format!(
            "Work Leaf collected this context from commits, git logs, and recorded chat history without querying Agent-ID user-1.\n\
             Git metadata:\nLatest commit: {second_commit}\n"
        ),
        format!(
            "You are the work-leaf linearizer for reviewed agent patches.\n\n\
         Final patch targets (1):\n\
         - Agent-ID: user-1\n\
           Commit: {second_commit}\n\
           Feature: second feature\n\
           Reason: Linearize 2 reviewed commits through {second_short}\n\
           Subject: FIX second\n\
           Context: Linearize target includes 2 reviewed provisional commits for patch agent user-1.\n\n\
         Reviewed commit: {first_commit}\n\
         Subject: ADD first\n\
         Feature: first feature\n\
         Reason: first reason\n\
         Context: first context\n\n\
         Reviewed commit: {second_commit}\n\
         Subject: FIX second\n\
         Feature: second feature\n\
         Reason: second reason\n\
         Context: second context\n\n\
         Scope and commit-shaping rules:\n"
        ),
    ];
    let client = json_lines(prompts.iter().enumerate().map(|(index, prompt)| {
        serde_json::json!({
            "id": index + 1,
            "method": "turn/start",
            "params": {
                "threadId": "thread-user",
                "input": [{ "type": "text", "text": prompt }],
            },
        })
    }));
    let mut server = prompts
        .iter()
        .enumerate()
        .map(|(index, _)| {
            serde_json::json!({
                "id": index + 1,
                "result": { "turn": { "id": format!("turn-{}", index + 1) } },
            })
        })
        .collect::<Vec<_>>();
    server.push(serde_json::json!({
        "method": "thread/tokenUsage/updated",
        "params": {
            "turnId": "turn-8",
            "tokenUsage": {
                "last": {
                    "inputTokens": 250,
                    "cachedInputTokens": 200,
                    "outputTokens": 25,
                    "reasoningOutputTokens": 10,
                },
                "total": {
                    "inputTokens": 250,
                    "cachedInputTokens": 200,
                    "outputTokens": 25,
                    "reasoningOutputTokens": 10,
                },
            },
        },
    }));
    server.push(serde_json::json!({
        "method": "item/completed",
        "params": {
            "threadId": "thread-user",
            "turnId": "turn-6",
            "item": {
                "type": "commandExecution",
                "command": format!("cat {}", bundle_path.display()),
                "aggregatedOutput": bundle,
            },
        },
    }));
    server.push(serde_json::json!({
        "method": "item/completed",
        "params": {
            "threadId": "thread-user",
            "turnId": "turn-8",
            "item": { "type": "agentMessage", "text": "complete\n@work-leaf done\n" },
        },
    }));
    server.push(serde_json::json!({
        "method": "turn/completed",
        "params": {
            "threadId": "thread-user",
            "turnId": "turn-8",
            "turn": { "id": "turn-8", "status": "interrupted" },
        },
    }));
    let server = json_lines(server);
    let mut app_server = Command::new(proxy.join("codex"))
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env(
            "WORK_LEAF_OBSERVER_PRIMARY_MARKER",
            &config.primary_invocation_marker,
        )
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", server)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let mut app_stdin = app_server.stdin.take().unwrap();
    app_stdin.write_all(client.as_bytes()).unwrap();
    app_stdin
        .write_all(
            serde_json::json!({
                "id": 99,
                "method": "turn/interrupt",
                "params": { "threadId": "thread-user", "turnId": "turn-8" },
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap();
    app_stdin.write_all(b"\n").unwrap();
    drop(app_stdin);
    assert!(app_server.wait().unwrap().success());

    record_timeline(&config, "condition-start", Some("schedule=sequential")).unwrap();
    record_timeline(&config, "feature-start", Some("feature=1")).unwrap();
    record_timeline(&config, "feature-cycle-complete", Some("feature=1")).unwrap();
    let state = root.path().join("controller-state.json");
    fs::write(
        &state,
        serde_json::to_vec(&serde_json::json!({
            "snapshot": { "sessions": [{
                "id": "user-1",
                "token_usage": {
                    "input_tokens": 250,
                    "cached_input_tokens": 200,
                    "output_tokens": 25,
                    "reasoning_output_tokens": 10,
                },
            }]},
        }))
        .unwrap(),
    )
    .unwrap();
    record_controller_usage(&config, &state).unwrap();

    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    for hypothesis in ["H1", "H2", "H4", "H7"] {
        assert!(
            summary
                .mechanisms
                .counterfactuals
                .iter()
                .any(|record| { record.hypothesis == hypothesis && record.status == "verified" })
        );
    }
    assert_eq!(summary.mechanisms.bundles.len(), 1);
    assert_eq!(summary.mechanisms.bundles[0].consumption, "full");
    assert_eq!(summary.mechanisms.terminal_directives.interrupted, 1);
    assert!(summary.mechanisms.sequential_timeline_valid);
}

#[test]
fn nested_locked_commands_remain_captured_without_becoming_h4_render_candidates() {
    let root = TempDir::new().unwrap();
    let config_path = initialize(&root);
    let proxy = root.path().join("proxy-bin");
    let command = "printf 'ok\\n'";
    let wrapper = format!(
        "trap 'trap - TERM INT; kill -TERM 0 2>/dev/null' TERM INT; \
         ({command}) & work_leaf_child=$!; wait $work_leaf_child"
    );
    let root_output = Command::new(proxy.join("sh"))
        .args(["-c", &wrapper])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .output()
        .unwrap();
    assert!(root_output.status.success());

    let root_invocation = fs::read_dir(root.path().join("invocations"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find_map(|directory| {
            let value: serde_json::Value =
                serde_json::from_slice(&fs::read(directory.join("start.json")).ok()?).ok()?;
            (value["capture_kind"] == "locked-command" && value["parent_invocation_id"].is_null())
                .then(|| value["invocation_id"].as_str().unwrap().to_string())
        })
        .unwrap();
    let nested_output = Command::new(proxy.join("sh"))
        .args(["-c", &wrapper])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env("WORK_LEAF_OBSERVER_PARENT_INVOCATION", &root_invocation)
        .output()
        .unwrap();
    assert!(nested_output.status.success());

    let foreign_workspace = TempDir::new().unwrap();
    let foreign_output = Command::new(proxy.join("sh"))
        .args(["-c", &wrapper])
        .current_dir(foreign_workspace.path())
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .output()
        .unwrap();
    assert!(foreign_output.status.success());

    let server = json_lines([serde_json::json!({
        "id": 1,
        "result": {"turn": {"id": "turn-command"}},
    })]);
    let mut app_server = Command::new(proxy.join("codex"))
        .args(["app-server", "--listen", "stdio://"])
        .env("WORK_LEAF_OBSERVER_CONFIG", &config_path)
        .env(
            "WORK_LEAF_OBSERVER_PRIMARY_MARKER",
            CaptureConfig::load(&config_path)
                .unwrap()
                .primary_invocation_marker,
        )
        .env("WORK_LEAF_OBSERVER_FIXTURE_STDOUT", server)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    app_server
        .stdin
        .take()
        .unwrap()
        .write_all(
            serde_json::json!({
                "id": 1,
                "method": "turn/start",
                "params": {
                    "threadId": "thread-user",
                    "input": [{
                        "type": "text",
                        "text": format!(
                            "work-leaf command result\ncommand: {command}\nstatus: 0\n\
                             locked paths: .\nstdout:\nok\nstderr:\n<empty>\n"
                        ),
                    }],
                },
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap();
    assert!(app_server.wait().unwrap().success());

    let config = CaptureConfig::load(&config_path).unwrap();
    let summary = analyze(&config).unwrap();
    assert!(summary.capture_complete, "{:?}", summary.errors);
    assert_eq!(summary.invocation_count, 4);
    assert!(
        summary
            .mechanisms
            .counterfactuals
            .iter()
            .any(|record| { record.hypothesis == "H4" && record.status == "verified" })
    );
}
