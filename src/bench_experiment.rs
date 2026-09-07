//! Provider-neutral, non-default benchmark prompt interventions.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::agent::AgentId;

const SCHEMA: &str = "work-leaf-bench-experiment-v1";
const SCHEMA_V2: &str = "work-leaf-bench-experiment-v2";
const SCHEMA_V3: &str = "work-leaf-bench-experiment-v3";
const SCHEMA_V4: &str = "work-leaf-bench-experiment-v4";
const SCHEMA_V5: &str = "work-leaf-bench-experiment-v5";
const SCHEMA_V6: &str = "work-leaf-bench-experiment-v6";

#[path = "bench_review_evidence.rs"]
mod review_evidence;
pub(crate) use review_evidence::forward_review_context;

#[path = "bench_private_test_first.rs"]
pub(crate) mod private_test_first;

#[cfg(test)]
#[path = "bench_review_evidence_tests.rs"]
mod review_evidence_tests;

#[path = "bench_candidate_experiment.rs"]
mod candidate;
pub(crate) use candidate::{candidates_active, forward_candidate, forward_candidate_policy};
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
    let bytes = fs::read(&path)?;
    let (manifest, review_root, private_preview) = private_test_first::parse_manifest(&bytes)?;
    if !matches!(
        manifest.schema.as_str(),
        SCHEMA | SCHEMA_V2 | SCHEMA_V3 | SCHEMA_V4 | SCHEMA_V5 | SCHEMA_V6
    ) {
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
        SCHEMA_V3 => matches!(
            manifest.condition.as_str(),
            "control" | "untracked-read-inline"
        ),
        SCHEMA_V4 => matches!(
            manifest.condition.as_str(),
            "requested-repeat-full" | "unified-diff-preferred" | "review-fix-request-resupply"
        ),
        SCHEMA_V5 => matches!(
            manifest.condition.as_str(),
            "review-evidence-native" | "review-evidence-inline"
        ),
        SCHEMA_V6 => manifest.condition == "private-test-first",
        _ => false,
    };
    if !permitted {
        return Err(invalid("unsupported condition"));
    }
    if !manifest.evidence_path.is_absolute() {
        return Err(invalid("evidence_path must be absolute"));
    }
    if let Some(root) = review_root.as_ref() {
        review_evidence::initialize_store(root)?;
    }
    if let Some(descriptor) = private_preview.as_ref() {
        private_test_first::initialize_bridge(descriptor.clone())?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&manifest.evidence_path)
        .map_err(invalid)?;
    let mut activation = json!({
        "event": "activation", "schema": manifest.schema, "run_id": manifest.run_id,
        "condition": manifest.condition, "process_id": std::process::id()
    });
    if let Some(root) = review_root {
        activation["review_evidence_root"] = json!(root);
    }
    if let Some(descriptor) = private_preview {
        activation["private_preview"] = json!(descriptor);
    }
    serde_json::to_writer(&mut file, &activation).map_err(invalid)?;
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
    if matches!(
        experiment.manifest.schema.as_str(),
        SCHEMA_V2 | SCHEMA_V3 | SCHEMA_V4 | SCHEMA_V5 | SCHEMA_V6
    ) {
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
        Some(experiment)
            if matches!(
                experiment.manifest.schema.as_str(),
                SCHEMA_V2 | SCHEMA_V3 | SCHEMA_V5
            ) =>
        {
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
        "event": "prompt", "schema": experiment.manifest.schema, "sequence": evidence.sequence,
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

#[derive(Serialize)]
pub(crate) struct ReadComponent {
    pub(crate) baseline_start: usize,
    pub(crate) baseline_end: usize,
    pub(crate) inline_start: usize,
    pub(crate) inline_end: usize,
}

#[derive(Serialize)]
pub(crate) struct ReadSnapshot {
    pub(crate) path: String,
    pub(crate) class: &'static str,
    pub(crate) bytes: usize,
    pub(crate) digest: String,
    pub(crate) inline_body_start: Option<usize>,
    pub(crate) inline_body_end: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct ReadBundle {
    pub(crate) threshold_eligible: bool,
    pub(crate) write_succeeded: bool,
    pub(crate) path: Option<PathBuf>,
}

#[derive(Serialize)]
pub(crate) struct ReadFailure {
    pub(crate) path: String,
    pub(crate) diagnostic: String,
}

pub(crate) struct ReadEvidence {
    pub(crate) component: Option<ReadComponent>,
    pub(crate) bundle: ReadBundle,
    pub(crate) eligibility_reason: &'static str,
    pub(crate) requested_paths: Vec<String>,
    pub(crate) snapshots: Vec<ReadSnapshot>,
    pub(crate) failures: Vec<ReadFailure>,
}

impl ReadEvidence {
    pub(crate) fn prepend(&mut self, bytes: usize) {
        if let Some(component) = &mut self.component {
            component.baseline_start += bytes;
            component.baseline_end += bytes;
            component.inline_start += bytes;
            component.inline_end += bytes;
        }
        for snapshot in &mut self.snapshots {
            snapshot.inline_body_start = snapshot.inline_body_start.map(|start| start + bytes);
            snapshot.inline_body_end = snapshot.inline_body_end.map(|end| end + bytes);
        }
    }
}

pub(crate) fn reads_active() -> io::Result<bool> {
    Ok(active()?.is_some_and(|experiment| experiment.manifest.schema == SCHEMA_V3))
}

pub(crate) fn forward_read(
    agent_id: &AgentId,
    baseline: String,
    inline: String,
    record: ReadEvidence,
) -> io::Result<String> {
    let experiment = active()?.ok_or_else(|| invalid("read evidence requires active v3"))?;
    if experiment.manifest.schema != SCHEMA_V3 {
        return Err(invalid("read evidence requires active v3"));
    }
    forward_read_with(experiment, agent_id, baseline, inline, record)
}

fn forward_read_with(
    experiment: &Experiment,
    agent_id: &AgentId,
    baseline: String,
    inline: String,
    record: ReadEvidence,
) -> io::Result<String> {
    let eligible = record.bundle.write_succeeded;
    if let Some(component) = &record.component {
        let baseline_span = component.baseline_start..component.baseline_end;
        let inline_span = component.inline_start..component.inline_end;
        if baseline.get(baseline_span.clone()).is_none()
            || inline.get(inline_span.clone()).is_none()
            || baseline_span.start != inline_span.start
            || baseline[..baseline_span.start] != inline[..inline_span.start]
            || baseline[baseline_span.end..] != inline[inline_span.end..]
        {
            return Err(invalid(
                "read candidates differ outside the renderer-owned component",
            ));
        }
    } else if eligible || baseline != inline {
        return Err(invalid("absent read component must preserve identity"));
    }
    if (!eligible && baseline != inline)
        || eligible != record.bundle.path.is_some()
        || (eligible && !record.bundle.threshold_eligible)
    {
        return Err(invalid(
            "read eligibility must follow successful ordinary bundle creation",
        ));
    }
    let mut previous_body_end = 0;
    for snapshot in &record.snapshots {
        match (
            snapshot.class,
            snapshot.inline_body_start,
            snapshot.inline_body_end,
        ) {
            ("untracked", Some(start), Some(end)) => {
                let valid = record.component.as_ref().is_some_and(|component| {
                    start >= component.inline_start && end <= component.inline_end
                }) && start >= previous_body_end
                    && inline
                        .get(start..end)
                        .is_some_and(|text| text.len() == snapshot.bytes);
                if !valid {
                    return Err(invalid("invalid renderer-owned snapshot body range"));
                }
                previous_body_end = end;
            }
            ("changed" | "unchanged" | "explicit-bundle", None, None) => {}
            _ => return Err(invalid("invalid read snapshot class or body range")),
        }
    }
    let choose_inline = eligible && experiment.manifest.condition == "untracked-read-inline";
    let selected = if choose_inline { &inline } else { &baseline };
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(invalid)?
        .as_nanos()
        .to_string();
    let mut evidence = experiment.evidence.lock().map_err(invalid)?;
    evidence.sequence += 1;
    let row = json!({
        "event": "read-response", "schema": SCHEMA_V3, "sequence": evidence.sequence,
        "run_id": experiment.manifest.run_id, "condition": experiment.manifest.condition,
        "process_id": std::process::id(), "unix_time_ns": timestamp,
        "site": "file-read", "agent_id": agent_id.to_string(),
        "baseline_prompt": baseline, "inline_candidate_prompt": inline,
        "baseline_bytes": baseline.len(), "inline_candidate_bytes": inline.len(),
        "selected_candidate": if choose_inline { "inline" } else { "baseline" },
        "selected_bytes": selected.len(), "changed": *selected != baseline,
        "candidate_byte_delta": inline.len() as i64 - baseline.len() as i64,
        "byte_delta": selected.len() as i64 - baseline.len() as i64,
        "eligible": eligible, "eligibility_reason": record.eligibility_reason,
        "component": record.component, "bundle": record.bundle,
        "requested_paths": record.requested_paths, "snapshots": record.snapshots,
        "failures": record.failures,
    });
    serde_json::to_writer(&mut evidence.file, &row).map_err(invalid)?;
    evidence.file.write_all(b"\n").map_err(invalid)?;
    evidence.file.flush().map_err(invalid)?;
    Ok(if choose_inline { inline } else { baseline })
}

#[cfg(test)]
#[path = "bench_read_experiment_tests.rs"]
mod read_tests;

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
