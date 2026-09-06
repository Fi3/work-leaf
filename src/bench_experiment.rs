//! Provider-neutral, non-default benchmark prompt interventions.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::json;

use crate::agent::AgentId;

const SCHEMA: &str = "work-leaf-bench-experiment-v1";
const ACK: &str = "run at most one focused validation step that is relevant to files you touched or checks you added.";
const UNLIMITED: &str = "run the required focused validation steps that are relevant to files you touched or checks you added.";
const GUIDANCE: &str = "\nnext: Reply with the next Work Leaf directive, such as `@work-leaf done`, `@work-leaf edit`, `@work-leaf read`, or another `@work-leaf locks run`. Keep any non-directive explanation brief.";

static EXPERIMENT: OnceLock<Result<Option<Experiment>, String>> = OnceLock::new();

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    run_id: String,
    condition: String,
    evidence_path: PathBuf,
}

struct Experiment {
    manifest: Manifest,
    evidence: Mutex<Evidence>,
}

struct Evidence {
    file: File,
    sequence: u64,
}

fn invalid(message: impl std::fmt::Display) -> io::Error {
    io::Error::other(format!("benchmark experiment: {message}"))
}

fn load() -> io::Result<Option<Experiment>> {
    let marker = std::env::var_os("WORK_LEAF_BENCH_EXPERIMENT");
    let path = std::env::var_os("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST");
    if marker.is_none() && path.is_none() {
        return Ok(None);
    }
    if marker.as_deref() != Some(std::ffi::OsStr::new("1")) {
        return Err(invalid("WORK_LEAF_BENCH_EXPERIMENT=1 is required"));
    }
    let path = PathBuf::from(path.ok_or_else(|| invalid("manifest path is required"))?);
    if !path.is_absolute() || !fs::symlink_metadata(&path)?.file_type().is_file() {
        return Err(invalid("manifest must be an absolute regular-file path"));
    }
    let manifest: Manifest = serde_json::from_slice(&fs::read(&path)?).map_err(invalid)?;
    if manifest.schema != SCHEMA {
        return Err(invalid("unsupported manifest schema"));
    }
    if manifest.run_id.is_empty()
        || !manifest
            .run_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
        || std::env::var("WORK_LEAF_BENCH_RUN_ID").ok().as_deref() != Some(&manifest.run_id)
    {
        return Err(invalid("manifest run_id must match WORK_LEAF_BENCH_RUN_ID"));
    }
    if !matches!(
        manifest.condition.as_str(),
        "control" | "ack-validation-unlimited" | "command-guidance-neutral"
    ) {
        return Err(invalid("unsupported condition"));
    }
    if !manifest.evidence_path.is_absolute() {
        return Err(invalid("evidence_path must be absolute"));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&manifest.evidence_path)
        .map_err(invalid)?;
    serde_json::to_writer(
        &mut file,
        &json!({
            "event": "activation", "schema": SCHEMA, "run_id": manifest.run_id,
            "condition": manifest.condition, "process_id": std::process::id()
        }),
    )
    .map_err(invalid)?;
    file.write_all(b"\n").map_err(invalid)?;
    file.flush().map_err(invalid)?;
    Ok(Some(Experiment {
        manifest,
        evidence: Mutex::new(Evidence { file, sequence: 0 }),
    }))
}

fn active() -> io::Result<Option<&'static Experiment>> {
    match EXPERIMENT.get_or_init(|| load().map_err(|error| error.to_string())) {
        Ok(experiment) => Ok(experiment.as_ref()),
        Err(error) => Err(invalid(error)),
    }
}

pub(crate) fn initialize() -> io::Result<()> {
    active().map(|_| ())
}

pub(crate) fn forward(
    site: &str,
    agent_id: &AgentId,
    original: String,
    cue: Range<usize>,
) -> io::Result<String> {
    let Some(experiment) = active()? else {
        return Ok(original);
    };
    let expected = match site {
        "patch-applied" => ACK,
        "command-result" => GUIDANCE,
        _ => return Err(invalid("unsupported prompt boundary")),
    };
    if original.get(cue.clone()) != Some(expected) {
        return Err(invalid(
            "prompt boundary does not match its declared treatment span",
        ));
    }
    let replacement = match (experiment.manifest.condition.as_str(), site) {
        ("ack-validation-unlimited", "patch-applied") => UNLIMITED,
        ("command-guidance-neutral", "command-result") => "",
        _ => expected,
    };
    let mut forwarded = original.clone();
    forwarded.replace_range(cue.clone(), replacement);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(invalid)?
        .as_nanos()
        .to_string();
    let mut evidence = experiment.evidence.lock().map_err(invalid)?;
    evidence.sequence += 1;
    let row = json!({
        "event": "prompt", "sequence": evidence.sequence,
        "run_id": experiment.manifest.run_id, "condition": experiment.manifest.condition,
        "process_id": std::process::id(), "unix_time_ns": timestamp,
        "site": site, "agent_id": agent_id.to_string(),
        "cue_start": cue.start, "cue_end": cue.end,
        "original_prompt": original, "forwarded_prompt": forwarded,
        "original_bytes": original.len(), "forwarded_bytes": forwarded.len(),
        "byte_delta": forwarded.len() as i64 - original.len() as i64,
        "changed": original != forwarded
    });
    serde_json::to_writer(&mut evidence.file, &row).map_err(invalid)?;
    evidence.file.write_all(b"\n").map_err(invalid)?;
    evidence.file.flush().map_err(invalid)?;
    Ok(forwarded)
}
