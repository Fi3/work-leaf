#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

#[test]
fn bench_dashboard_self_test_exercises_parser_and_html_contract() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("bench-dashboard");
    let metadata = fs::metadata(&script).expect("bench-dashboard script exists");
    assert_ne!(
        metadata.permissions().mode() & 0o111,
        0,
        "bench-dashboard should be executable"
    );

    let output = Command::new("python3")
        .arg(&script)
        .arg("--self-test")
        .output()
        .expect("python3 should run bench-dashboard self-test");

    assert!(
        output.status.success(),
        "bench-dashboard self-test failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
