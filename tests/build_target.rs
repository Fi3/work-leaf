#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[test]
fn build_target_script_builds_single_binary_for_major_platforms() {
    let script_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("build-target");
    let script = fs::read_to_string(&script_path).expect("build-target script exists");
    let mode = fs::metadata(&script_path)
        .expect("build-target script is statable")
        .permissions()
        .mode();

    assert_ne!(mode & 0o111, 0, "build-target should be executable");
    assert!(script.contains("WORK_LEAF_BUILD_TARGETS"));
    assert!(script.contains("x86_64-unknown-linux-gnu"));
    assert!(script.contains("aarch64-unknown-linux-gnu"));
    assert!(script.contains("x86_64-apple-darwin"));
    assert!(script.contains("aarch64-apple-darwin"));
    assert!(script.contains("x86_64-pc-windows-msvc"));
    assert!(script.contains("aarch64-pc-windows-msvc"));
    assert!(script.contains("cargo build --release --locked --bin work-leaf"));
    assert!(script.contains("dist"));
    assert!(
        !script.contains("work-leaf-orchestrator"),
        "release packages should contain only the user-facing binary"
    );
}
