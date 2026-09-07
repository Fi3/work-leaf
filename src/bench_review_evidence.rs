//! Private, retained opaque review archives. Work is linear in the held context and
//! candidate bytes per delivery; earlier archives are not rescanned. Missing bundle
//! ancestors require O(D²) filesystem component resolution in path depth D (not in
//! session-message count). Read-only modes
//! and canonical/create-new checks are not hostile-concurrent-writer containment.

use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path};

use super::*;

static STORE: OnceLock<ReviewStore> = OnceLock::new();

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewManifest {
    schema: String,
    run_id: String,
    condition: String,
    evidence_path: PathBuf,
    review_evidence_root: PathBuf,
}

pub(super) fn parse_manifest(bytes: &[u8]) -> io::Result<(Manifest, Option<PathBuf>)> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(invalid)?;
    if value.get("schema").and_then(serde_json::Value::as_str) != Some(SCHEMA_V5) {
        // Deserialize old versions directly with their original deny-unknown-fields
        // struct. Even an explicit null new-schema field must remain an error.
        return Ok((serde_json::from_slice(bytes).map_err(invalid)?, None));
    }
    let parsed: ReviewManifest = serde_json::from_slice(bytes).map_err(invalid)?;
    Ok((
        Manifest {
            schema: parsed.schema,
            run_id: parsed.run_id,
            condition: parsed.condition,
            evidence_path: parsed.evidence_path,
        },
        Some(parsed.review_evidence_root),
    ))
}

struct ReviewStore {
    root: PathBuf,
    root_metadata: fs::Metadata,
    next: Mutex<u64>,
}

fn exact_directory(path: &Path) -> io::Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path).map_err(invalid)?;
    if !path.is_absolute()
        || !metadata.file_type().is_dir()
        || path.canonicalize().map_err(invalid)?.as_os_str() != path.as_os_str()
        || path.to_str().is_none()
    {
        return Err(invalid(
            "review evidence root must be an exact canonical existing directory",
        ));
    }
    Ok(metadata)
}

fn overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

// The normal bundle writer may not have created its admitted directory yet.
// Validate an exact canonical existing ancestor and only ordinary missing children.
fn exact_future_path(path: &Path) -> io::Result<PathBuf> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::CurDir | Component::ParentDir))
    {
        return Err(invalid(
            "context bundle directory must be an exact absolute path",
        ));
    }
    let mut ancestor = path;
    let mut missing = Vec::new();
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                missing.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| invalid("invalid bundle path"))?,
                );
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| invalid("invalid bundle parent"))?;
            }
            Err(error) => return Err(invalid(error)),
        }
    }
    exact_directory(ancestor)?;
    let mut resolved = ancestor.to_path_buf();
    for child in missing.into_iter().rev() {
        resolved.push(child);
    }
    if resolved.as_os_str() != path.as_os_str() {
        return Err(invalid("context bundle directory is aliased"));
    }
    Ok(resolved)
}

fn validate_bundle_separation(root: &Path) -> io::Result<()> {
    // Match ContextBundleStore::new's ordinary parent without allocating a store or
    // consuming its process/store/bundle counters.
    let parent = std::env::var_os("WORK_LEAF_CONTEXT_BUNDLE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("work-leaf-context-bundles"));
    if overlap(root, &exact_future_path(&parent)?) {
        return Err(invalid(
            "review evidence root overlaps the ordinary context bundle directory",
        ));
    }
    Ok(())
}

pub(super) fn initialize_store(root: &Path) -> io::Result<()> {
    let root_metadata = exact_directory(root)?;
    validate_bundle_separation(root)?;
    STORE
        .set(ReviewStore {
            root: root.to_path_buf(),
            root_metadata,
            next: Mutex::new(0),
        })
        .map_err(|_| invalid("review evidence store was already initialized"))
}

impl ReviewStore {
    fn validate(&self, project_dir: &Path) -> io::Result<()> {
        let current = exact_directory(&self.root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if current.dev() != self.root_metadata.dev()
                || current.ino() != self.root_metadata.ino()
            {
                return Err(invalid("review evidence root identity changed"));
            }
        }
        #[cfg(not(unix))]
        let _ = (&current, &self.root_metadata);
        let project = project_dir.canonicalize().map_err(invalid)?;
        if !fs::metadata(&project).map_err(invalid)?.is_dir() || overlap(&self.root, &project) {
            return Err(invalid(
                "review evidence root overlaps the project directory",
            ));
        }
        validate_bundle_separation(&self.root)
    }
}

fn digest(bytes: &[u8]) -> String {
    let mut value = 0xcbf29ce484222325_u64;
    for byte in bytes {
        value ^= u64::from(*byte);
        value = value.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{value:016x}")
}

fn publish(path: &Path, bytes: &[u8]) -> io::Result<()> {
    // create_new atomically refuses an existing final component, including a symlink.
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(invalid)?;
    file.write_all(bytes).map_err(invalid)?;
    file.flush().map_err(invalid)?;
    let mut permissions = file.metadata().map_err(invalid)?.permissions();
    permissions.set_readonly(true);
    file.set_permissions(permissions).map_err(invalid)?;
    verify_publication(path, &mut file, bytes)
}

fn verify_publication(path: &Path, file: &mut File, bytes: &[u8]) -> io::Result<()> {
    let path_metadata = fs::symlink_metadata(path).map_err(invalid)?;
    if !path_metadata.file_type().is_file() {
        return Err(invalid("review archive publication is not a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let held_metadata = file.metadata().map_err(invalid)?;
        if path_metadata.dev() != held_metadata.dev() || path_metadata.ino() != held_metadata.ino()
        {
            return Err(invalid(
                "review archive final path no longer identifies its created file",
            ));
        }
    }
    file.seek(SeekFrom::Start(0)).map_err(invalid)?;
    let mut readback = Vec::new();
    file.take(bytes.len() as u64 + 1)
        .read_to_end(&mut readback)
        .map_err(invalid)?;
    if readback != bytes {
        return Err(invalid("review archive bytes changed during publication"));
    }
    Ok(())
}

pub(crate) fn forward_review_context(
    project_dir: &Path,
    source_agent: &AgentId,
    reviewer: &AgentId,
    target_commit: &str,
    baseline_prompt: String,
    context_span: Range<usize>,
) -> io::Result<String> {
    let Some(experiment) = active()? else {
        return Ok(baseline_prompt);
    };
    if experiment.manifest.schema != SCHEMA_V5 {
        return Ok(baseline_prompt);
    }
    let store = STORE
        .get()
        .ok_or_else(|| invalid("review evidence store is unavailable"))?;
    forward_with(
        experiment,
        store,
        project_dir,
        source_agent,
        reviewer,
        target_commit,
        baseline_prompt,
        context_span,
    )
}

#[allow(clippy::too_many_arguments)]
fn forward_with(
    experiment: &Experiment,
    store: &ReviewStore,
    project_dir: &Path,
    source_agent: &AgentId,
    reviewer: &AgentId,
    target_commit: &str,
    baseline_prompt: String,
    context_span: Range<usize>,
) -> io::Result<String> {
    store.validate(project_dir)?;
    if target_commit.is_empty() {
        return Err(invalid("review target commit must be nonempty"));
    }
    let context = baseline_prompt
        .get(context_span.clone())
        .ok_or_else(|| invalid("review context span must identify valid UTF-8 bytes"))?;
    let mut sequence = store.next.lock().map_err(invalid)?;
    *sequence = sequence
        .checked_add(1)
        .ok_or_else(|| invalid("review archive sequence exhausted"))?;
    let path = store.root.join(format!("review-{sequence:016}.txt"));
    let manifest_path = store.root.join(format!("review-{sequence:016}.json"));
    let archive = json!({
        "kind": "review-source-context", "path": path, "manifest_path": manifest_path,
        "sequence": *sequence, "bytes": context.len(), "digest": digest(context.as_bytes()),
        "run_id": experiment.manifest.run_id, "source_agent_id": source_agent.to_string(),
        "reviewer_id": reviewer.to_string(), "target_commit": target_commit
    });
    publish(&path, context.as_bytes())?;
    let manifest = json!({
        "schema": "work-leaf-review-evidence-v1", "archive": archive,
        "context_start": context_span.start, "context_end": context_span.end,
        "project_snapshots_applicable": false
    });
    publish(
        &manifest_path,
        &serde_json::to_vec(&manifest).map_err(invalid)?,
    )?;
    store.validate(project_dir)?;
    let receipt = format!(
        "Work Leaf opaque review evidence (temporary read-only context):\n\
         Path: {}\nBytes: {}\nEvidence identity: {}\n\
         This artifact contains the same complete commit/log/recorded-chat evidence for this review. \
         Native read-only inspection of this exact supplied path is permitted; it is not served by `@work-leaf read`. \
         Consult relevant archived evidence before declaring required evidence missing.\n",
        serde_json::to_string(&path).map_err(invalid)?,
        context.len(),
        archive["digest"].as_str().unwrap()
    );
    let mut candidate =
        String::with_capacity(baseline_prompt.len() - context.len() + receipt.len());
    candidate.push_str(&baseline_prompt[..context_span.start]);
    candidate.push_str(&receipt);
    candidate.push_str(&baseline_prompt[context_span.end..]);
    let selected = experiment.manifest.condition == "review-evidence-native";
    let forwarded = if selected {
        &candidate
    } else {
        &baseline_prompt
    };
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(invalid)?
        .as_nanos()
        .to_string();
    let mut evidence = experiment.evidence.lock().map_err(invalid)?;
    evidence.sequence = evidence
        .sequence
        .checked_add(1)
        .ok_or_else(|| invalid("prompt sequence exhausted"))?;
    let row = json!({
        "event": "review-context", "schema": SCHEMA_V5, "site": "review-source-context",
        "sequence": evidence.sequence, "run_id": experiment.manifest.run_id,
        "condition": experiment.manifest.condition, "process_id": std::process::id(), "unix_time_ns": timestamp,
        "source_agent_id": source_agent.to_string(), "reviewer_id": reviewer.to_string(), "target_commit": target_commit,
        "original_prompt": baseline_prompt, "candidate_prompt": candidate, "forwarded_prompt": forwarded,
        "context_start": context_span.start, "context_end": context_span.end,
        "candidate_start": context_span.start, "candidate_end": context_span.start + receipt.len(),
        "selected_candidate": if selected { "candidate" } else { "baseline" },
        "original_bytes": baseline_prompt.len(), "candidate_bytes": candidate.len(), "forwarded_bytes": forwarded.len(),
        "byte_delta": forwarded.len() as i64 - baseline_prompt.len() as i64,
        "changed": forwarded != &baseline_prompt, "archive": archive
    });
    serde_json::to_writer(&mut evidence.file, &row).map_err(invalid)?;
    evidence.file.write_all(b"\n").map_err(invalid)?;
    evidence.file.flush().map_err(invalid)?;
    Ok(if selected { candidate } else { baseline_prompt })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn root() -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "work-leaf-opaque-review-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("archive")).unwrap();
        fs::create_dir(path.join("project")).unwrap();
        path.canonicalize().unwrap()
    }

    #[test]
    fn runtime_checksum_has_explicit_fnv_not_cryptographic_identity() {
        assert_eq!(digest(b""), "fnv64:cbf29ce484222325");
        assert_eq!(digest(b"a"), "fnv64:af63dc4c8601ec8c");
        assert_eq!(digest(b"foobar"), "fnv64:85944171f73967e8");
    }

    #[test]
    fn trace_write_failure_retains_immutable_payload_and_blocks_delivery() {
        let root = root();
        let trace = root.join("read-only-trace");
        fs::write(&trace, "original trace").unwrap();
        let experiment = Experiment {
            manifest: Manifest {
                schema: SCHEMA_V5.to_string(),
                run_id: "fixture".into(),
                condition: "review-evidence-native".into(),
                evidence_path: trace.clone(),
            },
            evidence: Mutex::new(Evidence {
                file: File::open(&trace).unwrap(),
                sequence: 0,
            }),
        };
        let store = ReviewStore {
            root: root.join("archive"),
            root_metadata: exact_directory(&root.join("archive")).unwrap(),
            next: Mutex::new(0),
        };
        let result = forward_with(
            &experiment,
            &store,
            &root.join("project"),
            &AgentId::new("source").unwrap(),
            &AgentId::new("reviewer").unwrap(),
            "commit",
            "prefixλ source contextsuffix".into(),
            6..23,
        );
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(trace).unwrap(), "original trace");
        assert_eq!(
            fs::read_to_string(root.join("archive/review-0000000000000001.txt")).unwrap(),
            "λ source context"
        );
        assert!(root.join("archive/review-0000000000000001.json").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_roots_and_final_components_are_rejected_without_touching_targets() {
        use std::os::unix::fs::symlink;
        let root = root();
        symlink(root.join("archive"), root.join("alias")).unwrap();
        assert!(exact_directory(&root.join("alias")).is_err());
        assert!(exact_future_path(&root.join("alias/not-created")).is_err());
        let retained = root.join("retained");
        fs::write(&retained, "keep exact").unwrap();
        symlink(&retained, root.join("archive/new.txt")).unwrap();
        assert!(publish(&root.join("archive/new.txt"), b"must not replace").is_err());
        assert_eq!(fs::read_to_string(retained).unwrap(), "keep exact");
    }

    #[cfg(unix)]
    #[test]
    fn publication_verifies_the_created_descriptor_not_a_replacement_alias() {
        use std::os::unix::fs::symlink;
        let root = root();
        let path = root.join("archive/held.txt");
        let mut held = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        held.write_all(b"exact bytes").unwrap();
        fs::rename(&path, root.join("retained.txt")).unwrap();
        symlink(root.join("retained.txt"), &path).unwrap();
        assert!(verify_publication(&path, &mut held, b"exact bytes").is_err());
        fs::remove_file(&path).unwrap();
        fs::write(&path, b"exact bytes").unwrap();
        assert!(verify_publication(&path, &mut held, b"exact bytes").is_err());
        fs::remove_file(&path).unwrap();
        fs::rename(root.join("retained.txt"), &path).unwrap();
        assert!(verify_publication(&path, &mut held, b"exact bytes").is_ok());
        assert!(verify_publication(&path, &mut held, b"altered bytes").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn replaced_canonical_directory_is_not_the_admitted_root() {
        let root = root();
        let store = ReviewStore {
            root: root.join("archive"),
            root_metadata: exact_directory(&root.join("archive")).unwrap(),
            next: Mutex::new(0),
        };
        fs::rename(root.join("archive"), root.join("retained-archive")).unwrap();
        fs::create_dir(root.join("archive")).unwrap();
        assert!(store.validate(&root.join("project")).is_err());
    }
}
