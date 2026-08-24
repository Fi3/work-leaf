#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "work-leaf-benchmark-validation-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn final_gate_stops_when_clippy_fails_even_inside_a_condition() {
    let root = TestDir::new();
    let checkout = root.path().join("checkout");
    let child_tmp = root.path().join("tmp");
    let fake_bin = root.path().join("bin");
    let calls = root.path().join("cargo-calls.txt");
    let checks = root.path().join("checks.log");
    fs::create_dir_all(&checkout).unwrap();
    fs::create_dir_all(&child_tmp).unwrap();
    fs::create_dir_all(&fake_bin).unwrap();

    let cargo = fake_bin.join("cargo");
    fs::write(
        &cargo,
        concat!(
            "#!/usr/bin/env bash\n",
            "printf '%s\\n' \"$1\" >> \"$BENCH_VALIDATION_CALLS\"\n",
            "case \"$1\" in\n",
            "  fmt) exit 0 ;;\n",
            "  clippy) exit 42 ;;\n",
            "  test) exit 0 ;;\n",
            "  *) exit 99 ;;\n",
            "esac\n",
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&cargo).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&cargo, permissions).unwrap();

    let path = format!(
        "{}:{}",
        fake_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new("bash")
        .args([
            "-c",
            concat!(
                "source \"$1\"\n",
                "if bench_run_final_gate \"$2\" \"$3\" \"$4\"; then\n",
                "  exit 0\n",
                "else\n",
                "  exit $?\n",
                "fi\n",
            ),
            "benchmark-final-gate",
            "bench-validation-common",
            checkout.to_str().unwrap(),
            child_tmp.to_str().unwrap(),
            checks.to_str().unwrap(),
        ])
        .env("PATH", path)
        .env("BENCH_VALIDATION_CALLS", &calls)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(42),
        "a failed Clippy gate was hidden by a later successful test\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(calls).unwrap(), "fmt\nclippy\n");
}
