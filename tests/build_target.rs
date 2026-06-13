#![cfg(unix)]

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn build_target_script_defaults_to_the_rust_host_target() {
    let script_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("build-target");
    let script = fs::read_to_string(&script_path).expect("build-target script exists");
    let mode = fs::metadata(&script_path)
        .expect("build-target script is statable")
        .permissions()
        .mode();

    assert_ne!(mode & 0o111, 0, "build-target should be executable");
    assert!(script.contains("rustc -vV"));
    assert!(script.contains("host:"));
    assert!(script.contains("host_target="));
    assert!(script.contains(r#"target_list="${WORK_LEAF_BUILD_TARGETS:-$host_target}""#));
    assert!(script.contains("rustup target list --installed"));
    assert!(script.contains("rustup target add"));
    assert!(script.contains("cargo build --release --locked --bin work-leaf"));
    assert!(script.contains("Claude available on PATH when"));
    assert!(script.contains("Python SDK"));
    assert!(script.contains("TypeScript SDK"));
    assert!(script.contains("dist"));
    assert!(
        !script.contains("work-leaf-orchestrator"),
        "release packages should contain only the user-facing binary"
    );
}

#[test]
fn build_target_script_keeps_an_explicit_target_override_for_ci() {
    let script_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("build-target");
    let script = fs::read_to_string(&script_path).expect("build-target script exists");

    assert!(script.contains("WORK_LEAF_BUILD_TARGETS"));
    assert!(script.contains("for target in $target_list"));
}

#[test]
fn build_target_script_installs_missing_rustup_target_before_build() {
    let script_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("build-target");
    let temp_dir = unique_temp_dir("work-leaf-build-target");
    let fake_bin_dir = temp_dir.join("bin");
    fs::create_dir_all(&fake_bin_dir).expect("fake bin directory is created");
    let command_log = temp_dir.join("commands.log");
    let target = "work-leaf-test-target";

    write_executable(
        &fake_bin_dir.join("rustc"),
        r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "-vV" ]]; then
  printf 'rustc 1.0.0\n'
  printf 'host: x86_64-unknown-linux-gnu\n'
  exit 0
fi
echo "unexpected rustc args: $*" >&2
exit 1
"#,
    );
    write_executable(
        &fake_bin_dir.join("rustup"),
        r#"#!/usr/bin/env bash
set -euo pipefail
printf 'rustup' >> "$WORK_LEAF_FAKE_COMMAND_LOG"
for arg in "$@"; do
  printf ' %s' "$arg" >> "$WORK_LEAF_FAKE_COMMAND_LOG"
done
printf '\n' >> "$WORK_LEAF_FAKE_COMMAND_LOG"

if [[ "${1:-}" == "target" && "${2:-}" == "list" && "${3:-}" == "--installed" ]]; then
  printf '%s\n' "${WORK_LEAF_FAKE_INSTALLED_TARGETS:-}"
  exit 0
fi

if [[ "${1:-}" == "target" && "${2:-}" == "add" && -n "${3:-}" ]]; then
  exit 0
fi

echo "unexpected rustup args: $*" >&2
exit 1
"#,
    );
    write_executable(
        &fake_bin_dir.join("cargo"),
        r#"#!/usr/bin/env bash
set -euo pipefail
printf 'cargo' >> "$WORK_LEAF_FAKE_COMMAND_LOG"
for arg in "$@"; do
  printf ' %s' "$arg" >> "$WORK_LEAF_FAKE_COMMAND_LOG"
done
printf '\n' >> "$WORK_LEAF_FAKE_COMMAND_LOG"

target=""
previous=""
for arg in "$@"; do
  if [[ "$previous" == "--target" ]]; then
    target="$arg"
  fi
  previous="$arg"
done

if [[ -z "$target" ]]; then
  echo "cargo build did not receive --target" >&2
  exit 1
fi

binary_name="work-leaf"
case "$target" in
  *-pc-windows-*)
    binary_name="work-leaf.exe"
    ;;
esac

mkdir -p "target/$target/release"
printf 'fake binary\n' > "target/$target/release/$binary_name"
"#,
    );

    let original_path = env::var_os("PATH").expect("PATH is set for build-target test");
    let mut paths = vec![fake_bin_dir.clone()];
    paths.extend(env::split_paths(&original_path));
    let path = env::join_paths(paths).expect("fake command PATH can be joined");

    let output = Command::new("bash")
        .arg(&script_path)
        .env("PATH", path)
        .env("WORK_LEAF_BUILD_TARGETS", target)
        .env("WORK_LEAF_DIST_DIR", temp_dir.join("dist"))
        .env("WORK_LEAF_FAKE_COMMAND_LOG", &command_log)
        .env(
            "WORK_LEAF_FAKE_INSTALLED_TARGETS",
            "x86_64-unknown-linux-gnu",
        )
        .output()
        .expect("build-target script runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "build-target should succeed with fake commands\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        temp_dir
            .join("dist")
            .join(format!("work-leaf-{target}"))
            .join("work-leaf")
            .exists(),
        "build-target should package the fake built binary"
    );

    let log = fs::read_to_string(&command_log).expect("fake command log is readable");
    let rustup_list = log
        .find("rustup target list --installed")
        .expect("script checks installed Rust targets");
    let rustup_add = log
        .find(&format!("rustup target add {target}"))
        .expect("script installs the missing Rust target");
    let cargo_build = log
        .find(&format!(
            "cargo build --release --locked --bin work-leaf --target {target}"
        ))
        .expect("script builds after preparing the target");

    assert!(
        rustup_list < rustup_add && rustup_add < cargo_build,
        "expected rustup target add before cargo build, got:\n{log}"
    );

    let repo_target_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(target);
    let _ = fs::remove_dir_all(repo_target_dir);
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn github_release_workflow_builds_each_binary_on_its_native_runner() {
    let workflow_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".github")
        .join("workflows")
        .join("release-binaries.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("release workflow exists");

    assert!(workflow.contains("ubuntu-24.04"));
    assert!(workflow.contains("ubuntu-24.04-arm"));
    assert!(workflow.contains("macos-15-intel"));
    assert!(workflow.contains("macos-15"));
    assert!(workflow.contains("windows-2025"));
    assert!(workflow.contains("windows-11-arm"));
    assert!(workflow.contains("x86_64-unknown-linux-gnu"));
    assert!(workflow.contains("aarch64-unknown-linux-gnu"));
    assert!(workflow.contains("x86_64-apple-darwin"));
    assert!(workflow.contains("aarch64-apple-darwin"));
    assert!(workflow.contains("x86_64-pc-windows-msvc"));
    assert!(workflow.contains("aarch64-pc-windows-msvc"));
    assert!(workflow.contains("rustup toolchain install stable --profile minimal"));
    assert!(workflow.contains("WORK_LEAF_BUILD_TARGETS"));
    assert!(workflow.contains("./build-target"));
    assert!(workflow.contains("actions/upload-artifact"));
    assert!(
        workflow.contains("build-essential"),
        "linux runners should install the native compiler/linker package explicitly"
    );
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after the Unix epoch")
        .as_nanos();
    let dir = env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("temporary directory is created");
    dir
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).expect("fake executable is written");
    let mut permissions = fs::metadata(path)
        .expect("fake executable metadata is readable")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("fake executable is executable");
}
