#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use serde_json::Value;
use tempfile::TempDir;
use work_leaf_bench_observer::{CaptureConfig, analyze};

const ACTIVATION: &str = "WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY";

struct Fixture {
    root: TempDir,
    repo: PathBuf,
    home: PathBuf,
    config: CaptureConfig,
}

impl Fixture {
    fn new() -> Self {
        let root = TempDir::new().unwrap();
        let repo = root.path().join("project");
        let home = root.path().join("codex-settings");
        fs::create_dir_all(repo.join(".codex/rules")).unwrap();
        fs::create_dir_all(&home).unwrap();
        fs::write(home.join("config.toml"), "model = 'fixture-model'\n").unwrap();
        fs::write(
            repo.join(".codex/config.toml"),
            "approval_policy = 'never'\nsandbox_mode = 'workspace-write'\n[sandbox_workspace_write]\nnetwork_access = true\n",
        )
        .unwrap();
        fs::write(repo.join(".codex/rules/local.rules"), "rule-bytes\n").unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .arg(&repo)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "commit",
                    "--allow-empty",
                    "--quiet",
                    "-m",
                    "fixture base"
                ])
                .status()
                .unwrap()
                .success()
        );
        let provider = root.path().join("provider");
        fs::write(&provider, concat!(
            "#!/bin/sh\n",
            "if [ \"$1\" = --version ]; then printf 'inventory-fixture 1.0\\n'; exit 0; fi\n",
            "if [ \"${TEST_REQUIRE_INVENTORY:-}\" = 1 ]; then\n",
            "  test -s \"$TEST_OBSERVER_ROOT/project-layer-inventory/manifest.jsonl\" || exit 91\n",
            "fi\n",
            "printf started > \"$TEST_CHILD_STARTED\"\n",
            "printf '%s\\n' \"$*\" > \"$TEST_CHILD_ARGS\"\n",
            "printf '%s' \"${WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY:-absent}\" > \"$TEST_CHILD_ENV\"\n",
            "exec /bin/cat\n",
        )).unwrap();
        fs::set_permissions(&provider, fs::Permissions::from_mode(0o700)).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_bench-observer"))
            .args(["init", "--root"])
            .arg(root.path().join("observation"))
            .args([
                "--study-id",
                "inventory-study",
                "--pair-id",
                "inventory-block",
                "--condition",
                "work-leaf",
                "--run-id",
                "inventory-run",
                "--real-codex",
            ])
            .arg(provider)
            .args([
                "--real-sh",
                "/bin/sh",
                "--real-cargo",
                "/bin/true",
                "--base-commit",
                "base",
                "--experiment-commit",
                "experiment",
                "--model",
                "fixture-model",
                "--effort",
                "high",
            ])
            .env_remove(ACTIVATION)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let config =
            CaptureConfig::load(&root.path().join("observation/observer-config.json")).unwrap();
        Self {
            root,
            repo,
            home,
            config,
        }
    }

    fn command(&self, marker: Option<&str>) -> Command {
        let mut command = Command::new(self.config.root.join("proxy-bin/codex"));
        command
            .args(["app-server", "--listen", "stdio://"])
            .current_dir(&self.repo)
            .env("WORK_LEAF_OBSERVER_CONFIG", self.config.path())
            .env(
                "WORK_LEAF_OBSERVER_PRIMARY_MARKER",
                &self.config.primary_invocation_marker,
            )
            .env("CODEX_HOME", &self.home)
            .env("TEST_OBSERVER_ROOT", &self.config.root)
            .env("TEST_CHILD_STARTED", self.root.path().join("child-started"))
            .env("TEST_CHILD_ARGS", self.root.path().join("child-args"))
            .env("TEST_CHILD_ENV", self.root.path().join("child-env"))
            .env_remove(ACTIVATION)
            .env_remove("WORK_LEAF_OBSERVER_PARENT_INVOCATION")
            .env_remove("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE")
            .env_remove("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS");
        if let Some(marker) = marker {
            command.env(ACTIVATION, marker);
        }
        command
    }

    fn launch(&self, marker: Option<&str>) -> Output {
        let mut child = self
            .command(marker)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let _ = child.stdin.take().unwrap().write_all(b"unchanged input\n");
        child.wait_with_output().unwrap()
    }

    fn records(&self) -> Vec<Value> {
        fs::read_to_string(
            self.config
                .root
                .join("project-layer-inventory/manifest.jsonl"),
        )
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
    }

    fn checkpoint(&self, label: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_bench-observer"))
            .args(["git-checkpoint", "--config"])
            .arg(self.config.path())
            .arg("--repo")
            .arg(&self.repo)
            .args(["--label", label])
            .env(ACTIVATION, "1")
            .env("CODEX_HOME", &self.home)
            .output()
            .unwrap()
    }
}

fn entry<'a>(record: &'a Value, path: &str) -> &'a Value {
    record["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
        .unwrap()
}

#[test]
fn explicit_inventory_is_durable_before_child_and_preserves_stream_argv_and_environment() {
    let fixture = Fixture::new();
    let output = fixture
        .command(Some("1"))
        .env("TEST_REQUIRE_INVENTORY", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let records = fixture.records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["valid"], true);
    assert_eq!(records[0]["label"], "pre-spawn");
    assert_eq!(records[0]["repository"], fixture.repo.to_str().unwrap());
    assert_eq!(records[0]["run_id"], "inventory-run");
    assert!(records[0]["invocation_id"].is_string());
    assert_eq!(
        fs::read_to_string(fixture.root.path().join("child-args")).unwrap(),
        "app-server --listen stdio://\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.root.path().join("child-env")).unwrap(),
        "1"
    );
    let forwarded = fixture.launch(Some("1"));
    assert!(forwarded.status.success());
    assert_eq!(forwarded.stdout, b"unchanged input\n");
    assert!(forwarded.stderr.is_empty());
}

#[test]
fn absent_marker_does_not_inventory_or_validate_project_settings() {
    let fixture = Fixture::new();
    fs::write(fixture.repo.join(".codex/config.toml"), "not valid toml\n").unwrap();
    let output = fixture.launch(None);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"unchanged input\n");
    assert!(!fixture.config.root.join("project-layer-inventory").exists());
}

#[test]
fn invalid_marker_fails_before_provider_spawn() {
    for value in ["", "0", "true", "2"] {
        let fixture = Fixture::new();
        let output = fixture.launch(Some(value));
        assert!(
            !output.status.success(),
            "invalid marker {value:?} was accepted"
        );
        assert!(!fixture.root.path().join("child-started").exists());
    }
}

#[test]
fn inventory_includes_ignored_untracked_files_directories_and_absent_layers_without_contents() {
    let fixture = Fixture::new();
    fs::write(fixture.repo.join(".gitignore"), ".codex/\n").unwrap();
    fs::write(
        fixture.repo.join(".codex/private-settings.txt"),
        "UNIQUE_SECRET_MUST_NOT_APPEAR",
    )
    .unwrap();
    fs::create_dir(fixture.repo.join(".codex/hooks")).unwrap();
    let output = fixture.launch(Some("1"));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let record = &fixture.records()[0];
    assert_eq!(entry(record, ".codex")["kind"], "directory");
    assert_eq!(entry(record, ".codex/private-settings.txt")["kind"], "file");
    assert!(entry(record, ".codex/private-settings.txt")["sha256"].is_string());
    assert_eq!(entry(record, ".codex/hooks.json")["kind"], "absent");
    assert_eq!(entry(record, ".codex/hooks")["kind"], "directory");
    assert!(!record.to_string().contains("UNIQUE_SECRET_MUST_NOT_APPEAR"));
}

#[test]
fn checkpoint_snapshots_retain_actual_changed_layers_and_failure_evidence() {
    let fixture = Fixture::new();
    assert!(fixture.launch(Some("1")).status.success());
    assert!(fixture.checkpoint("pre-linearize").status.success());
    let initial = fixture.records();
    assert_eq!(
        initial[0]["inventory_sha256"],
        initial[1]["inventory_sha256"]
    );
    fs::write(
        fixture.repo.join(".codex/rules/local.rules"),
        "different rule\n",
    )
    .unwrap();
    assert!(!fixture.checkpoint("final").status.success());
    let records = fixture.records();
    assert_ne!(
        records[0]["inventory_sha256"],
        records[2]["inventory_sha256"]
    );
    assert_eq!(records[2]["label"], "checkpoint:final");
    fs::write(
        fixture.repo.join(".codex/config.toml"),
        "model_instructions_file = '/external/private-resource'\n",
    )
    .unwrap();
    assert!(!fixture.checkpoint("unsupported").status.success());
    assert_eq!(fixture.records().last().unwrap()["valid"], false);
    assert!(
        !fixture
            .records()
            .last()
            .unwrap()
            .to_string()
            .contains("/external/private-resource")
    );
}

#[test]
fn symlink_layer_is_recorded_without_following_and_prevents_spawn() {
    let fixture = Fixture::new();
    let external = fixture.root.path().join("outside");
    fs::write(&external, "DO_NOT_CAPTURE_EXTERNAL_CONTENTS").unwrap();
    symlink(&external, fixture.repo.join(".codex/rules/link.rules")).unwrap();
    assert!(!fixture.launch(Some("1")).status.success());
    assert!(!fixture.root.path().join("child-started").exists());
    let record = &fixture.records()[0];
    assert_eq!(record["valid"], false);
    assert_eq!(entry(record, ".codex/rules/link.rules")["kind"], "symlink");
    assert!(entry(record, ".codex/rules/link.rules")["sha256"].is_null());
    assert!(
        !record
            .to_string()
            .contains("DO_NOT_CAPTURE_EXTERNAL_CONTENTS")
    );
}

#[test]
fn credential_file_is_not_read_or_hashed() {
    let fixture = Fixture::new();
    fs::write(fixture.repo.join(".codex/auth.json"), "CREDENTIAL_SENTINEL").unwrap();
    assert!(!fixture.launch(Some("1")).status.success());
    let record = &fixture.records()[0];
    assert_eq!(
        entry(record, ".codex/auth.json")["kind"],
        "credential-file-rejected"
    );
    assert!(entry(record, ".codex/auth.json")["sha256"].is_null());
    assert!(!record.to_string().contains("CREDENTIAL_SENTINEL"));
}

#[test]
fn unsupported_global_and_project_configuration_is_rejected_without_values() {
    for (global, config) in [
        (true, "project_root_markers = ['.external-root']\n"),
        (true, "profile = 'private-profile'\n"),
        (true, "[profiles.private]\nmodel = 'fixture-model'\n"),
        (
            false,
            "model_instructions_file = '/external/private-resource'\n",
        ),
        (
            false,
            "[agents.worker]\nconfig_file = '/external/private-role'\n",
        ),
        (false, "project_root_markers = []\n"),
        (false, "not valid toml\n"),
    ] {
        let fixture = Fixture::new();
        let path = if global {
            fixture.home.join("config.toml")
        } else {
            fixture.repo.join(".codex/config.toml")
        };
        fs::write(path, config).unwrap();
        let output = fixture.launch(Some("1"));
        assert!(
            !output.status.success(),
            "accepted unsupported configuration {config}"
        );
        assert!(!fixture.root.path().join("child-started").exists());
        assert_eq!(fixture.records()[0]["valid"], false);
        assert!(!String::from_utf8_lossy(&output.stderr).contains("/external/private-resource"));
    }
}

#[test]
fn nested_cwd_without_declared_project_root_is_rejected() {
    let fixture = Fixture::new();
    let nested = fixture.repo.join("nested");
    fs::create_dir(&nested).unwrap();
    let output = fixture
        .command(Some("1"))
        .current_dir(nested)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!fixture.root.path().join("child-started").exists());
    assert_eq!(fixture.records()[0]["valid"], false);
}

#[test]
fn nonprimary_invocation_is_not_misrepresented_as_required_primary_inventory() {
    let fixture = Fixture::new();
    let output = fixture
        .command(Some("1"))
        .env_remove("WORK_LEAF_OBSERVER_PRIMARY_MARKER")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!fixture.config.root.join("project-layer-inventory").exists());
}

#[test]
fn oversized_file_retains_bounded_failure_before_spawn() {
    let fixture = Fixture::new();
    let file = fs::File::create(fixture.repo.join(".codex/huge-file")).unwrap();
    file.set_len(2 * 1024 * 1024).unwrap();
    assert!(!fixture.launch(Some("1")).status.success());
    assert!(!fixture.root.path().join("child-started").exists());
    assert_eq!(fixture.records()[0]["valid"], false);
}

#[test]
fn absent_project_codex_directory_is_an_explicit_valid_inventory() {
    let fixture = Fixture::new();
    fs::remove_dir_all(fixture.repo.join(".codex")).unwrap();
    assert!(fixture.launch(Some("1")).status.success());
    assert_eq!(entry(&fixture.records()[0], ".codex")["kind"], "absent");
    assert_eq!(
        entry(&fixture.records()[0], ".codex/config.toml")["kind"],
        "absent"
    );
}

#[test]
fn configuration_home_symlink_is_not_silently_followed() {
    let fixture = Fixture::new();
    let alias = fixture.root.path().join("settings-alias");
    symlink(&fixture.home, &alias).unwrap();
    let output = fixture
        .command(Some("1"))
        .env("CODEX_HOME", alias)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fixture.records()[0]["valid"], false);
}

#[test]
fn layer_permission_changes_are_part_of_inventory_identity() {
    let fixture = Fixture::new();
    assert!(fixture.launch(Some("1")).status.success());
    fs::set_permissions(
        fixture.repo.join(".codex/rules/local.rules"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    assert!(!fixture.checkpoint("pre-linearize").status.success());
    let records = fixture.records();
    assert_ne!(
        records[0]["inventory_sha256"],
        records[1]["inventory_sha256"]
    );
    assert_eq!(records[1]["matches_pre_spawn"], false);
}

#[test]
fn expected_configuration_file_cannot_be_a_directory() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.repo.join(".codex/config.toml")).unwrap();
    fs::create_dir(fixture.repo.join(".codex/config.toml")).unwrap();
    assert!(!fixture.launch(Some("1")).status.success());
    assert_eq!(fixture.records()[0]["valid"], false);
}

#[test]
fn unrelated_project_names_are_not_interpreted_as_configuration_option_names() {
    let fixture = Fixture::new();
    fs::write(
        fixture.home.join("config.toml"),
        "[projects.'/unrelated/example_file']\ntrust_level = 'trusted'\n",
    )
    .unwrap();
    assert!(fixture.launch(Some("1")).status.success());
    assert_eq!(fixture.records()[0]["valid"], true);
}

#[test]
fn retained_checkpoint_failure_invalidates_offline_analysis_without_activation_environment() {
    let fixture = Fixture::new();
    assert!(fixture.launch(Some("1")).status.success());
    fs::write(
        fixture.repo.join(".codex/config.toml"),
        "profile = 'unhandled'\n",
    )
    .unwrap();
    assert!(!fixture.checkpoint("pre-linearize").status.success());
    let summary = analyze(&fixture.config).unwrap();
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.starts_with("project-layer inventory:"))
    );
    assert!(!summary.capture_complete);
}

#[test]
fn offline_analysis_rejects_inventory_entry_digest_tampering() {
    let fixture = Fixture::new();
    assert!(fixture.launch(Some("1")).status.success());
    let mut records = fixture.records();
    records[0]["entries"][0]["kind"] = Value::String("tampered".into());
    fs::write(
        fixture
            .config
            .root
            .join("project-layer-inventory/manifest.jsonl"),
        format!("{}\n", records[0]),
    )
    .unwrap();
    let summary = analyze(&fixture.config).unwrap();
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.starts_with("project-layer inventory:"))
    );
}

#[test]
fn offline_analysis_detects_deleted_required_inventory() {
    let fixture = Fixture::new();
    assert!(fixture.launch(Some("1")).status.success());
    fs::remove_dir_all(fixture.config.root.join("project-layer-inventory")).unwrap();
    let summary = analyze(&fixture.config).unwrap();
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.starts_with("project-layer inventory:"))
    );
}

#[test]
fn offline_analysis_derives_cross_snapshot_drift_instead_of_trusting_valid_flags() {
    let fixture = Fixture::new();
    assert!(fixture.launch(Some("1")).status.success());
    fs::write(fixture.repo.join(".codex/rules/local.rules"), "changed\n").unwrap();
    assert!(!fixture.checkpoint("pre-linearize").status.success());
    let mut records = fixture.records();
    records[1]["valid"] = Value::Bool(true);
    records[1]["errors"] = serde_json::json!([]);
    records[1]["matches_pre_spawn"] = Value::Bool(true);
    fs::write(
        fixture
            .config
            .root
            .join("project-layer-inventory/manifest.jsonl"),
        records
            .iter()
            .map(|record| format!("{record}\n"))
            .collect::<String>(),
    )
    .unwrap();
    let summary = analyze(&fixture.config).unwrap();
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.starts_with("project-layer inventory:"))
    );
}

#[test]
fn failed_entry_limit_does_not_exceed_the_retained_record_bound() {
    let fixture = Fixture::new();
    for number in 0..600 {
        fs::write(fixture.repo.join(format!(".codex/entry-{number}")), "x").unwrap();
    }
    assert!(!fixture.launch(Some("1")).status.success());
    let record = &fixture.records()[0];
    assert!(record["entries"].as_array().unwrap().len() <= 512);
}

#[cfg(unix)]
#[test]
fn rejected_non_utf8_children_consume_the_enumeration_budget() {
    use std::os::unix::ffi::OsStringExt;
    let fixture = Fixture::new();
    for number in 0..600 {
        let mut name = format!("rejected-{number}-").into_bytes();
        name.push(0xff);
        fs::write(
            fixture
                .repo
                .join(".codex")
                .join(std::ffi::OsString::from_vec(name)),
            "x",
        )
        .unwrap();
    }
    assert!(!fixture.launch(Some("1")).status.success());
    let records = fixture.records();
    let errors = records[0]["errors"].as_array().unwrap();
    assert!(
        errors
            .iter()
            .any(|error| error == "inventory-entry-or-depth-limit")
    );
}
