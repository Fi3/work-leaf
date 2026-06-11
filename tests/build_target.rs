#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

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
    assert!(script.contains("cargo build --release --locked --bin work-leaf"));
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
