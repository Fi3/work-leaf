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
const SCHEMA_V2: &str = "work-leaf-bench-experiment-v2";
const ACK: &str = "run at most one focused validation step that is relevant to files you touched or checks you added.";
const UNLIMITED: &str = "run the required focused validation steps that are relevant to files you touched or checks you added.";
const GUIDANCE: &str = "\nnext: Reply with the next Work Leaf directive, such as `@work-leaf done`, `@work-leaf edit`, `@work-leaf read`, or another `@work-leaf locks run`. Keep any non-directive explanation brief.";
const WORK_UNIT: &str = "Design tests before implementation when required, but submit a cohesive patch that includes the test and the implementation needed for the shared tree to build.";
const INCREMENTAL_WORK_UNIT: &str = "Design tests before implementation when required. Prefer naturally separable, independently buildable feature increments, keeping mutually dependent test and implementation changes together. Submit each completed increment and continue any remaining requested feature work. Do not split inseparable work or create otherwise unnecessary patches.";
const TESTS_WORK_UNIT: &str = "Design the needed tests, but submit tests with the implementation needed to keep the shared worktree buildable.";
const INCREMENTAL_TESTS_WORK_UNIT: &str = "Design the needed tests, and prefer naturally separable, independently buildable feature increments, keeping mutually dependent tests and implementation together. Do not split inseparable work or create otherwise unnecessary patches.";
const REMAINING_WORK: &str = "After the focused validation passes, or after you report an external blocker, emit a top-level `@work-leaf done` so review can start. Send another edit only if validation found a concrete issue in your own patch.";
const INCREMENTAL_REMAINING_WORK: &str = "After the focused validation passes, or after you report an external blocker, continue with another independently buildable increment if requested feature work remains and can proceed without taking over another agent's work. Otherwise emit a top-level `@work-leaf done` so review can start. Send another edit only to implement remaining requested feature work or to repair a concrete issue that validation found in your own patch.";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PromptSpan {
    id: &'static str,
    pub(crate) cue: Range<usize>,
}

impl PromptSpan {
    pub(crate) fn new(id: &'static str, cue: Range<usize>) -> Self {
        Self { id, cue }
    }
}

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
    if !matches!(manifest.schema.as_str(), SCHEMA | SCHEMA_V2) {
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
    let permitted = match manifest.schema.as_str() {
        SCHEMA => matches!(
            manifest.condition.as_str(),
            "control" | "ack-validation-unlimited" | "command-guidance-neutral"
        ),
        SCHEMA_V2 => matches!(
            manifest.condition.as_str(),
            "control" | "buildable-work-unit-incremental"
        ),
        _ => false,
    };
    if !permitted {
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
            "event": "activation", "schema": manifest.schema, "run_id": manifest.run_id,
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

pub(crate) fn forward_continuation(
    site: &str,
    agent_id: &AgentId,
    original: String,
    cue: Range<usize>,
    remaining_work: Option<Range<usize>>,
) -> io::Result<String> {
    let Some(experiment) = active()? else {
        return Ok(original);
    };
    if experiment.manifest.schema == SCHEMA_V2 {
        let mut spans = match site {
            "patch-applied" => vec![PromptSpan::new("patch-applied-validation", cue)],
            "command-result" => vec![PromptSpan::new("command-result-guidance", cue)],
            _ => return Err(invalid("unsupported prompt boundary")),
        };
        match (site, remaining_work) {
            ("patch-applied", Some(cue)) => {
                spans.push(PromptSpan::new("patch-applied-remaining-work", cue))
            }
            ("command-result", None) => {}
            _ => return Err(invalid("unexpected remaining-work span")),
        }
        return forward_v2(experiment, site, agent_id, original, spans);
    }
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

pub(crate) fn forward_policy(
    agent_id: &AgentId,
    original: String,
    spans: Vec<PromptSpan>,
) -> io::Result<String> {
    match active()? {
        Some(experiment) if experiment.manifest.schema == SCHEMA_V2 => {
            forward_v2(experiment, "policy-injection", agent_id, original, spans)
        }
        _ => Ok(original),
    }
}

fn render_spans(
    site: &str,
    original: &str,
    spans: &[PromptSpan],
    treatment: bool,
) -> io::Result<(String, Vec<serde_json::Value>)> {
    let mut forwarded = String::with_capacity(original.len());
    let mut evidence = Vec::with_capacity(spans.len());
    let mut cursor = 0;
    for span in spans {
        let (expected, varied) = match (site, span.id) {
            ("policy-injection", "policy-buildable-work-unit") => {
                (WORK_UNIT, INCREMENTAL_WORK_UNIT)
            }
            ("policy-injection", "instruction-tests-work-unit") => {
                (TESTS_WORK_UNIT, INCREMENTAL_TESTS_WORK_UNIT)
            }
            ("patch-applied", "patch-applied-validation") => (ACK, ACK),
            ("patch-applied", "patch-applied-remaining-work") => {
                (REMAINING_WORK, INCREMENTAL_REMAINING_WORK)
            }
            ("command-result", "command-result-guidance") => (GUIDANCE, GUIDANCE),
            _ => return Err(invalid("unsupported owned prompt span")),
        };
        if span.cue.start < cursor || original.get(span.cue.clone()) != Some(expected) {
            return Err(invalid(
                "prompt spans must be ordered, disjoint, and match their declared text",
            ));
        }
        let replacement = if treatment { varied } else { expected };
        forwarded.push_str(&original[cursor..span.cue.start]);
        forwarded.push_str(replacement);
        cursor = span.cue.end;
        evidence.push(json!({
            "id": span.id, "cue_start": span.cue.start, "cue_end": span.cue.end,
            "original": expected, "replacement": replacement,
            "changed": expected != replacement,
            "byte_delta": replacement.len() as i64 - expected.len() as i64
        }));
    }
    forwarded.push_str(&original[cursor..]);
    Ok((forwarded, evidence))
}

fn forward_v2(
    experiment: &Experiment,
    site: &str,
    agent_id: &AgentId,
    original: String,
    spans: Vec<PromptSpan>,
) -> io::Result<String> {
    let (forwarded, spans) = render_spans(
        site,
        &original,
        &spans,
        experiment.manifest.condition == "buildable-work-unit-incremental",
    )?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(invalid)?
        .as_nanos()
        .to_string();
    let mut evidence = experiment.evidence.lock().map_err(invalid)?;
    evidence.sequence += 1;
    let row = json!({
        "event": "prompt", "schema": SCHEMA_V2, "sequence": evidence.sequence,
        "run_id": experiment.manifest.run_id, "condition": experiment.manifest.condition,
        "process_id": std::process::id(), "unix_time_ns": timestamp,
        "site": site, "agent_id": agent_id.to_string(), "spans": spans,
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

#[cfg(test)]
mod work_unit_tests {
    use super::*;

    #[test]
    fn rejects_unowned_misaligned_out_of_order_or_overlapping_spans() {
        let text = format!("λ {WORK_UNIT}\n{TESTS_WORK_UNIT}");
        let first = PromptSpan::new("policy-buildable-work-unit", 3..3 + WORK_UNIT.len());
        let second = PromptSpan::new("instruction-tests-work-unit", first.cue.end + 1..text.len());
        let mut overlap = second.clone();
        overlap.cue.start = first.cue.end - 1;
        for spans in [
            vec![PromptSpan::new("unowned", first.cue.clone())],
            vec![PromptSpan::new(
                "policy-buildable-work-unit",
                1..first.cue.end,
            )],
            vec![PromptSpan::new(
                "policy-buildable-work-unit",
                first.cue.start..text.len() + 1,
            )],
            vec![second.clone(), first.clone()],
            vec![first.clone(), first.clone()],
            vec![first.clone(), overlap],
        ] {
            for treatment in [false, true] {
                assert!(render_spans("policy-injection", &text, &spans, treatment).is_err());
            }
        }
        assert!(render_spans("command-result", &text, std::slice::from_ref(&first), true).is_err());
        let (forwarded, spans) =
            render_spans("policy-injection", &text, &[first, second], true).unwrap();
        assert_eq!(
            forwarded,
            format!("λ {INCREMENTAL_WORK_UNIT}\n{INCREMENTAL_TESTS_WORK_UNIT}")
        );
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn evidence_write_failure_never_returns_a_deliverable_prompt() {
        // A read-only descriptor fails actual writes without global environment mutation,
        // disk quotas, provider processes, or a fault-injection branch in production code.
        let experiment = Experiment {
            manifest: Manifest {
                schema: SCHEMA_V2.to_string(),
                run_id: "write-failure-fixture".to_string(),
                condition: "buildable-work-unit-incremental".to_string(),
                evidence_path: std::env::current_exe().unwrap(),
            },
            evidence: Mutex::new(Evidence {
                file: File::open(std::env::current_exe().unwrap()).unwrap(),
                sequence: 0,
            }),
        };
        let result = forward_v2(
            &experiment,
            "policy-injection",
            &AgentId::new("fixture").unwrap(),
            WORK_UNIT.to_string(),
            vec![PromptSpan::new(
                "policy-buildable-work-unit",
                0..WORK_UNIT.len(),
            )],
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("benchmark experiment")
        );
    }

    #[test]
    fn zero_span_identity_and_many_disjoint_spans_use_exact_owned_ranges() {
        assert_eq!(
            render_spans("policy-injection", "λ linearizer", &[], true)
                .unwrap()
                .0,
            "λ linearizer"
        );
        let mut original = String::new();
        let mut expected = String::new();
        let mut spans = Vec::new();
        for _ in 0..128 {
            // Identical copied data before each owned cue must stay untouched.
            original.push_str(TESTS_WORK_UNIT);
            expected.push_str(TESTS_WORK_UNIT);
            let start = original.len();
            original.push_str(TESTS_WORK_UNIT);
            spans.push(PromptSpan::new(
                "instruction-tests-work-unit",
                start..original.len(),
            ));
            expected.push_str(INCREMENTAL_TESTS_WORK_UNIT);
        }
        assert_eq!(
            render_spans("policy-injection", &original, &spans, true)
                .unwrap()
                .0,
            expected
        );
        assert_eq!(
            render_spans("policy-injection", &original, &spans, false)
                .unwrap()
                .0,
            original
        );
    }
}
