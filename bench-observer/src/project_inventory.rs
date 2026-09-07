//! Opt-in, snapshot-only project-layer evidence. No provider configuration is written.
//!
//! Work is O(B + F log F) per bounded snapshot, for B file bytes and F entries.
//! Snapshots establish equality only at recorded boundaries, not continuous immutability.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use crate::{
    CaptureConfig, CaptureKind, ObserverError, ObserverResult, ProcessInventoryRecord,
    create_private_directory, invocation_id, monotonic_time_ns, sha256_bytes, unix_time_ns,
};

const ACTIVATION: &str = "WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY";
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 8 * 1024 * 1024;
const MAX_ENTRIES: usize = 512;
const MAX_DEPTH: usize = 16;
const REQUIRED_PATHS: &[&str] = &[
    ".codex",
    ".codex/config.toml",
    ".codex/rules",
    ".codex/hooks.json",
    ".codex/hooks",
];

pub(super) fn enabled() -> ObserverResult<bool> {
    match std::env::var_os(ACTIVATION) {
        None => Ok(false),
        Some(value) if value == OsStr::new("1") => Ok(true),
        Some(_) => Err(ObserverError::new(format!(
            "{ACTIVATION} must be absent or exactly 1"
        ))),
    }
}

#[derive(Default)]
struct Inventory {
    entries: BTreeMap<String, Value>,
    config_sources: Vec<Value>,
    errors: Vec<&'static str>,
    bytes: u64,
    visited: usize,
}

impl Inventory {
    fn error(&mut self, code: &'static str) {
        if !self.errors.contains(&code) {
            self.errors.push(code);
        }
    }

    fn collect(&mut self, repository: &Path, args: Option<&[OsString]>) {
        if args
            .is_some_and(|args| args != ["app-server", "--listen", "stdio://"].map(OsString::from))
        {
            self.error("unsupported-provider-arguments");
        }
        if !repository.is_absolute()
            || fs::canonicalize(repository).ok().as_deref() != Some(repository)
            || !fs::symlink_metadata(repository.join(".git"))
                .is_ok_and(|metadata| metadata.file_type().is_dir())
        {
            self.error("cwd-must-be-canonical-git-project-root");
            return;
        }
        let git_root = Command::new("git")
            .arg("-C")
            .arg(repository)
            .args(["rev-parse", "--show-toplevel"])
            .output();
        if !git_root.is_ok_and(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).trim() == repository.to_string_lossy()
        }) {
            self.error("cwd-must-be-canonical-git-project-root");
            return;
        }
        let codex_home = std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".codex")));
        match codex_home {
            Some(home) if home.is_absolute() => self.config_source(&home.join("config.toml")),
            _ => self.error("missing-or-relative-codex-home"),
        }
        self.config_source(Path::new("/etc/codex/config.toml"));
        self.walk(repository, Path::new(".codex"), 0);
        for path in REQUIRED_PATHS {
            if !self.entries.contains_key(*path) {
                if self.entries.len() >= MAX_ENTRIES {
                    self.error("inventory-entry-or-depth-limit");
                    break;
                }
                // A rejected ancestor (including a symlink) is not an absent child layer.
                let actual = repository.join(path);
                if fs::symlink_metadata(&actual)
                    .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
                {
                    self.entries
                        .insert((*path).into(), json!({"path": path, "kind": "absent"}));
                }
            }
        }
    }

    fn config_source(&mut self, path: &Path) {
        let mut record = json!({"path": path});
        if path.ancestors().skip(1).any(|ancestor| {
            fs::symlink_metadata(ancestor).is_ok_and(|metadata| metadata.file_type().is_symlink())
        }) {
            record["kind"] = json!("symlinked-ancestor-rejected");
            self.error("unsupported-config-source-type");
            self.config_sources.push(record);
            return;
        }
        match fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                record["kind"] = json!("absent");
            }
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                record["kind"] = json!("file");
                match self.read_bounded(path, &metadata) {
                    Ok(bytes) => {
                        record["sha256"] = json!(sha256_bytes(&bytes));
                        self.validate_toml(&bytes);
                    }
                    Err(code) => self.error(code),
                }
            }
            Ok(_) => {
                record["kind"] = json!("unsupported-file-type");
                self.error("unsupported-config-source-type");
            }
            Err(_) => {
                record["kind"] = json!("unreadable");
                self.error("config-source-unreadable");
            }
        }
        self.config_sources.push(record);
    }

    fn validate_toml(&mut self, bytes: &[u8]) {
        let parsed = std::str::from_utf8(bytes)
            .ok()
            .and_then(|text| toml::from_str::<toml::Value>(text).ok());
        let Some(parsed) = parsed else {
            self.error("invalid-config-toml");
            return;
        };
        if has_unsupported_configuration(&parsed) {
            self.error("unsupported-root-profile-or-resource-reference");
        }
    }

    fn walk(&mut self, repository: &Path, relative: &Path, depth: usize) {
        if self.visited >= MAX_ENTRIES || self.entries.len() >= MAX_ENTRIES {
            self.error("inventory-entry-or-depth-limit");
            return;
        }
        self.visited += 1;
        if depth > MAX_DEPTH {
            self.error("inventory-entry-or-depth-limit");
            return;
        }
        let Some(name) = relative.to_str() else {
            self.error("non-utf8-project-path");
            return;
        };
        let path = repository.join(relative);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.entries
                    .insert(name.into(), json!({"path": name, "kind": "absent"}));
                return;
            }
            Err(_) => {
                self.error("project-layer-unreadable");
                return;
            }
        };
        let mut record = json!({"path": name});
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            record["mode"] = json!(metadata.permissions().mode() & 0o7777);
        }
        if (matches!(name, ".codex/config.toml" | ".codex/hooks.json") && !metadata.is_file())
            || (matches!(name, ".codex" | ".codex/rules" | ".codex/hooks") && !metadata.is_dir())
        {
            self.error("unexpected-project-layer-type");
        }
        if metadata.file_type().is_symlink() {
            record["kind"] = json!("symlink");
            #[cfg(unix)]
            if let Ok(target) = fs::read_link(&path) {
                use std::os::unix::ffi::OsStrExt;
                record["link_target_sha256"] = json!(sha256_bytes(target.as_os_str().as_bytes()));
            }
            self.error("project-layer-symlink-unsupported");
        } else if metadata.is_dir() {
            record["kind"] = json!("directory");
            self.entries.insert(name.into(), record);
            let Ok(children) = fs::read_dir(&path) else {
                self.error("project-layer-unreadable");
                return;
            };
            // No unbounded directory collection: the entry cap applies while enumerating.
            let mut children = children;
            loop {
                if self.visited >= MAX_ENTRIES {
                    self.error("inventory-entry-or-depth-limit");
                    break;
                }
                let Some(child) = children.next() else { break };
                match child {
                    Ok(child) => {
                        self.walk(repository, &relative.join(child.file_name()), depth + 1)
                    }
                    Err(_) => {
                        self.visited += 1;
                        self.error("project-layer-unreadable");
                    }
                }
            }
            if !fs::symlink_metadata(&path).is_ok_and(|after| same_file_state(&metadata, &after)) {
                self.error("inventory-directory-changed-during-read");
            }
            return;
        } else if metadata.is_file() {
            record["size"] = json!(metadata.len());
            let filename = relative
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or_default();
            if matches!(
                filename,
                "auth.json" | "credentials.json" | "credentials" | "id_rsa" | "id_ed25519"
            ) {
                record["kind"] = json!("credential-file-rejected");
                self.error("credential-file-in-project-layer");
            } else {
                record["kind"] = json!("file");
                match self.read_bounded(&path, &metadata) {
                    Ok(bytes) => {
                        record["sha256"] = json!(sha256_bytes(&bytes));
                        if name == ".codex/config.toml" {
                            self.validate_toml(&bytes);
                        } else if name == ".codex/hooks.json" {
                            // Hook commands can reference arbitrary resources beyond this scope.
                            match serde_json::from_slice::<Value>(&bytes) {
                                Ok(Value::Object(hooks)) if hooks.is_empty() => {}
                                Ok(_) => self.error("hook-resource-references-unsupported"),
                                Err(_) => self.error("invalid-project-hooks-json"),
                            }
                        }
                    }
                    Err(code) => self.error(code),
                }
            }
        } else {
            record["kind"] = json!("special-file-rejected");
            self.error("project-layer-special-file-unsupported");
        }
        self.entries.insert(name.into(), record);
    }

    fn read_bounded(
        &mut self,
        path: &Path,
        expected: &fs::Metadata,
    ) -> Result<Vec<u8>, &'static str> {
        if expected.len() > MAX_FILE_BYTES || self.bytes + expected.len() > MAX_TOTAL_BYTES {
            return Err("inventory-byte-limit");
        }
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options
            .open(path)
            .map_err(|_| "inventory-file-unreadable")?;
        let opened = file.metadata().map_err(|_| "inventory-file-unreadable")?;
        if !opened.is_file() || !same_file_state(expected, &opened) {
            return Err("inventory-file-changed-during-read");
        }
        let bytes = self.read_budgeted(&file)?;
        let final_metadata = file.metadata().map_err(|_| "inventory-file-unreadable")?;
        if bytes.len() as u64 != expected.len() || !same_file_state(expected, &final_metadata) {
            return Err("inventory-file-changed-during-read");
        }
        Ok(bytes)
    }

    fn read_budgeted(&mut self, reader: impl Read) -> Result<Vec<u8>, &'static str> {
        let remaining = MAX_TOTAL_BYTES.saturating_sub(self.bytes);
        let mut bytes = Vec::new();
        let result = reader
            .take(remaining.min(MAX_FILE_BYTES + 1))
            .read_to_end(&mut bytes);
        // Partial reads and later identity failures still consumed real filesystem bytes.
        self.bytes += bytes.len() as u64;
        result.map_err(|_| "inventory-file-unreadable")?;
        Ok(bytes)
    }
}

fn same_file_state(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    let same = before.len() == after.len() && before.modified().ok() == after.modified().ok();
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        same && before.dev() == after.dev()
            && before.ino() == after.ino()
            && before.mode() == after.mode()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec()
    }
    #[cfg(not(unix))]
    same
}

fn has_unsupported_configuration(value: &toml::Value) -> bool {
    match value {
        toml::Value::Table(table) => table.iter().any(|(key, value)| {
            if matches!(
                key.as_str(),
                "projects" | "mcp_servers" | "apps" | "model_providers"
            ) {
                // These maps have user-selected identifier keys, not option names.
                return value
                    .as_table()
                    .is_some_and(|entries| entries.values().any(has_unsupported_configuration));
            }
            matches!(
                key.as_str(),
                "project_root_markers" | "profile" | "profiles" | "include" | "imports" | "hooks"
            ) || ["_file", "_files", "_path", "_paths", "_dir", "_dirs"]
                .iter()
                .any(|suffix| key.ends_with(suffix))
                || has_unsupported_configuration(value)
        }),
        toml::Value::Array(values) => values.iter().any(has_unsupported_configuration),
        _ => false,
    }
}

pub(super) fn capture(
    config: &CaptureConfig,
    repository: &Path,
    label: &str,
    invocation: Option<&str>,
    args: Option<&[OsString]>,
) -> ObserverResult<()> {
    let started_monotonic_ns = monotonic_time_ns();
    let started_unix_ns = unix_time_ns();
    let mut inventory = Inventory::default();
    inventory.collect(repository, args);
    let entries = std::mem::take(&mut inventory.entries)
        .into_values()
        .collect::<Vec<_>>();
    let digest = sha256_bytes(&serde_json::to_vec(&entries)?);
    let root = config.root.join("project-layer-inventory");
    create_private_directory(&root)?;
    let baseline_path = root.join("pre-spawn-baseline.json");
    let matches_pre_spawn = if baseline_path.exists() {
        match read_evidence(&baseline_path, MAX_TOTAL_BYTES)
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        {
            Some(baseline) => Some(
                baseline["repository"] == repository.to_string_lossy().as_ref()
                    && baseline["inventory_sha256"] == digest
                    && baseline["valid"] == true,
            ),
            None => Some(false),
        }
    } else {
        None
    };
    if matches_pre_spawn == Some(false) {
        inventory.error("project-layer-drift-from-pre-spawn");
    }
    let record = json!({
        "schema": "work-leaf-project-layer-inventory-v1",
        "study_id": config.study_id, "run_id": config.run_id,
        "label": label, "invocation_id": invocation, "repository": repository,
        "root_policy": "canonical-cwd-is-git-root; default-markers-only; no-profile-or-resource-reference",
        "scope": "project .codex subtree at recorded boundary; not continuous immutability or complete effective settings",
        "started_monotonic_ns": started_monotonic_ns.to_string(),
        "completed_monotonic_ns": monotonic_time_ns().to_string(),
        "started_unix_ns": started_unix_ns.to_string(), "completed_unix_ns": unix_time_ns().to_string(),
        "valid": inventory.errors.is_empty(), "errors": inventory.errors,
        "entries": entries, "inventory_sha256": digest,
        "config_sources": inventory.config_sources,
        "work": {"visited_entries": inventory.visited, "read_bytes": inventory.bytes},
        "matches_pre_spawn": matches_pre_spawn,
        "limits": {"file_bytes": MAX_FILE_BYTES, "total_bytes": MAX_TOTAL_BYTES, "entries": MAX_ENTRIES, "depth": MAX_DEPTH},
    });
    let snapshot_path = root.join(format!("{}.json", invocation_id()));
    write_new(&snapshot_path, &record)?;
    append_durable(&root.join("manifest.jsonl"), &record)?;
    if label == "pre-spawn" && !baseline_path.exists() && record["valid"] == true {
        write_new(&baseline_path, &record)?;
    }
    if record["valid"] != true {
        return Err(ObserverError::new(
            "project-layer inventory rejected; see retained inventory error codes",
        ));
    }
    Ok(())
}

fn write_new(path: &Path, value: &Value) -> ObserverResult<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(path)?;
    serde_json::to_writer(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn append_durable(path: &Path, value: &Value) -> ObserverResult<()> {
    let mut options = OpenOptions::new();
    options.append(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(path)?;
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    file.write_all(&bytes)?;
    file.sync_all()?;
    // Required pre-spawn evidence is durable before the provider child can execute.
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn read_evidence(path: &Path, maximum: u64) -> Option<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(maximum + 1).read_to_end(&mut bytes).ok()?;
    (bytes.len() as u64 <= maximum).then_some(bytes)
}

/// Retained opt-in evidence is checked even when the offline analyzer has no activation env.
pub(super) fn audit(
    config: &CaptureConfig,
    processes: &[ProcessInventoryRecord],
    errors: &mut Vec<String>,
) {
    let root = config.root.join("project-layer-inventory");
    let required = processes
        .iter()
        .filter(|process| {
            if !process.start.primary || process.start.capture_kind != CaptureKind::AppServer {
                return false;
            }
            let path = config
                .root
                .join("invocations")
                .join(&process.start.invocation_id)
                .join("start.json");
            read_evidence(&path, MAX_FILE_BYTES)
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                .is_some_and(|value| value["project_layer_inventory_required"] == true)
        })
        .collect::<Vec<_>>();
    if required.is_empty() && !root.exists() {
        return;
    }
    let fail = |errors: &mut Vec<String>, code: &str| {
        errors.push(format!("project-layer inventory: {code}"))
    };
    let Some(bytes) = read_evidence(&root.join("manifest.jsonl"), MAX_TOTAL_BYTES) else {
        fail(errors, "required manifest missing, unreadable or oversized");
        return;
    };
    let Some(text) = std::str::from_utf8(&bytes).ok() else {
        fail(errors, "invalid manifest encoding");
        return;
    };
    let mut records = Vec::new();
    for line in text.lines().take(65) {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            fail(errors, "malformed manifest record");
            return;
        };
        if record["schema"] != "work-leaf-project-layer-inventory-v1"
            || record["run_id"] != config.run_id
            || record["study_id"] != config.study_id
            || !record["entries"].is_array()
            || record["inventory_sha256"]
                != sha256_bytes(&serde_json::to_vec(&record["entries"]).unwrap_or_default())
        {
            fail(errors, "snapshot identity or digest mismatch");
        }
        if record["valid"] != true
            || record["errors"] != json!([])
            || record["matches_pre_spawn"] == false
        {
            fail(errors, "retained invalid or changed snapshot");
        }
        records.push(record);
    }
    if records.is_empty() || records.len() > 64 {
        fail(errors, "empty or excessive snapshot inventory");
        return;
    }
    let baseline = records.iter().find(|record| record["label"] == "pre-spawn");
    if let Some(baseline) = baseline {
        if records.iter().any(|record| {
            record["repository"] != baseline["repository"]
                || record["inventory_sha256"] != baseline["inventory_sha256"]
        }) {
            fail(errors, "cross-snapshot project-layer drift");
        }
    } else if !required.is_empty() {
        fail(errors, "pre-spawn baseline absent");
    }
    // Index once rather than scanning S snapshots for each of P provider invocations.
    let mut by_invocation = BTreeMap::<&str, Vec<&Value>>::new();
    for record in &records {
        if record["label"] == "pre-spawn"
            && let Some(id) = record["invocation_id"].as_str()
        {
            by_invocation.entry(id).or_default().push(record);
        }
    }
    for process in required {
        let candidates = by_invocation
            .get(process.start.invocation_id.as_str())
            .map(Vec::as_slice)
            .unwrap_or_default();
        if candidates.len() != 1
            || candidates[0]["repository"] != process.start.cwd.to_string_lossy().as_ref()
        {
            fail(
                errors,
                "required primary pre-spawn snapshot missing or mismatched",
            );
            continue;
        }
        let child_path = config
            .root
            .join("invocations")
            .join(&process.start.invocation_id)
            .join("child.json");
        let child = read_evidence(&child_path, MAX_FILE_BYTES)
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        let completed = candidates[0]["completed_monotonic_ns"]
            .as_str()
            .and_then(|value| value.parse::<u128>().ok());
        let child_started = child
            .as_ref()
            .and_then(|value| value["started_monotonic_ns"].as_u64())
            .map(u128::from);
        if !matches!((completed, child_started), (Some(before), Some(after)) if before <= after) {
            fail(errors, "pre-spawn ordering or child metadata is unverified");
        }
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;

    #[test]
    fn failed_partial_reads_are_charged_and_remaining_budget_limits_reading() {
        struct FailingReader(bool);
        impl Read for FailingReader {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                if self.0 {
                    return Err(std::io::Error::other("synthetic read failure"));
                }
                self.0 = true;
                let count = buffer.len().min(3);
                buffer[..count].fill(b'x');
                Ok(count)
            }
        }
        let mut inventory = Inventory::default();
        assert!(inventory.read_budgeted(FailingReader(false)).is_err());
        assert_eq!(inventory.bytes, 3);
        inventory.bytes = MAX_TOTAL_BYTES - 2;
        assert_eq!(
            inventory
                .read_budgeted(std::io::repeat(b'x'))
                .unwrap()
                .len(),
            2
        );
        assert_eq!(inventory.bytes, MAX_TOTAL_BYTES);
    }
}
