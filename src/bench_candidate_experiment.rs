//! Private v4 candidate boundaries; earlier experiment contracts remain independent.

use super::*;

pub(crate) fn candidates_active() -> io::Result<bool> {
    Ok(active()?.is_some_and(|experiment| experiment.manifest.schema == SCHEMA_V4))
}

// Renderer coordinates delimit every permitted difference. Walking disjoint slices is
// linear in total prompt bytes plus component count, including zero-width insertions.
fn validate_components(
    baseline: &str,
    candidate: &str,
    components: &[ReadComponent],
) -> io::Result<()> {
    let (mut baseline_cursor, mut candidate_cursor) = (0, 0);
    for component in components {
        if component.baseline_start < baseline_cursor
            || component.inline_start < candidate_cursor
            || baseline
                .get(component.baseline_start..component.baseline_end)
                .is_none()
            || candidate
                .get(component.inline_start..component.inline_end)
                .is_none()
            || baseline.get(baseline_cursor..component.baseline_start)
                != candidate.get(candidate_cursor..component.inline_start)
        {
            return Err(invalid(
                "candidate differences must stay within ordered renderer-owned UTF-8 components",
            ));
        }
        baseline_cursor = component.baseline_end;
        candidate_cursor = component.inline_end;
    }
    if baseline.get(baseline_cursor..) != candidate.get(candidate_cursor..) {
        return Err(invalid(
            "candidate differs outside its renderer-owned components",
        ));
    }
    Ok(())
}

pub(crate) fn forward_candidate(
    site: &str,
    agent_id: &AgentId,
    baseline: String,
    candidate: String,
    components: Vec<ReadComponent>,
    metadata: serde_json::Value,
) -> io::Result<String> {
    match active()? {
        Some(experiment) if experiment.manifest.schema == SCHEMA_V4 => forward_candidate_with(
            experiment, site, agent_id, baseline, candidate, components, metadata,
        ),
        _ => Ok(baseline),
    }
}

fn forward_candidate_with(
    experiment: &Experiment,
    site: &str,
    agent_id: &AgentId,
    baseline: String,
    candidate: String,
    components: Vec<ReadComponent>,
    metadata: serde_json::Value,
) -> io::Result<String> {
    if experiment.manifest.schema != SCHEMA_V4 {
        return Err(invalid("candidate evidence requires active v4"));
    }
    let select_candidate = match site {
        "candidate-policy" => true,
        "requested-repeat-read" => experiment.manifest.condition == "requested-repeat-full",
        "review-fix-request" => experiment.manifest.condition == "review-fix-request-resupply",
        _ => return Err(invalid("unsupported candidate boundary")),
    };
    validate_components(&baseline, &candidate, &components)?;
    let selected = if select_candidate {
        &candidate
    } else {
        &baseline
    };
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(invalid)?
        .as_nanos()
        .to_string();
    let mut evidence = experiment.evidence.lock().map_err(invalid)?;
    evidence.sequence += 1;
    let row = json!({
        "event": "candidate-prompt", "schema": SCHEMA_V4,
        "sequence": evidence.sequence, "run_id": experiment.manifest.run_id,
        "condition": experiment.manifest.condition, "process_id": std::process::id(),
        "unix_time_ns": timestamp, "site": site, "agent_id": agent_id.to_string(),
        "original_prompt": baseline, "candidate_prompt": candidate,
        "selected_candidate": if select_candidate { "candidate" } else { "baseline" },
        "original_bytes": baseline.len(), "candidate_bytes": candidate.len(),
        "selected_bytes": selected.len(), "changed": *selected != baseline,
        "byte_delta": selected.len() as i64 - baseline.len() as i64,
        "components": components, "metadata": metadata
    });
    serde_json::to_writer(&mut evidence.file, &row).map_err(invalid)?;
    evidence.file.write_all(b"\n").map_err(invalid)?;
    evidence.file.flush().map_err(invalid)?;
    Ok(if select_candidate {
        candidate
    } else {
        baseline
    })
}

fn policy_pair(id: &str) -> io::Result<(&'static str, &'static str, &'static str)> {
    match id {
        "policy-requested-repeat-contract" => Ok((
            "requested-repeat-full",
            "If you request a file you already received, Work Leaf compares digests and returns either unchanged status or a diff from your last snapshot; do not use repeated reads to reload whole files.",
            "If you request a file you already received, Work Leaf returns its current full text; avoid unnecessary repeated reads.",
        )),
        "policy-repeat-authority" => Ok((
            "requested-repeat-full",
            "Treat compact file refreshes and repeated-read digests as authoritative.",
            "Treat compact automatic file refreshes and current full-text requested reads as authoritative.",
        )),
        "policy-write-format" => Ok((
            "unified-diff-preferred",
            "You are not allowed to write files directly; submit a structured edit patch for every file you want to change.",
            "You are not allowed to write files directly; submit an orchestrator patch for every file you want to change.",
        )),
        "policy-edit-request" => Ok((
            "unified-diff-preferred",
            "Use `@work-leaf edit <reason>` followed by an apply-patch-style exact edit body and `@work-leaf end` to request a write.",
            "Use `@work-leaf patch <reason>` followed by a complete valid unified diff with real hunk ranges and `@work-leaf end` to request a write.",
        )),
        "policy-edit-preference" => Ok((
            "unified-diff-preferred",
            "The legacy `@work-leaf patch <reason>` unified-diff directive is still accepted only when you already have a complete valid unified diff with real hunk ranges; prefer `@work-leaf edit` for manual code, configuration, and test changes.",
            "The `@work-leaf edit <reason>` directive followed by an apply-patch-style exact edit body and `@work-leaf end` remains accepted; prefer `@work-leaf patch` for manual code, configuration, and test changes.",
        )),
        "policy-manual-write-format" => Ok((
            "unified-diff-preferred",
            "Do not use command locks for manual feature edits; manual code, configuration, and test changes must still be submitted with the structured edit directive.",
            "Do not use command locks for manual feature edits; manual code, configuration, and test changes must still be submitted with a patch directive.",
        )),
        "instruction-commit-format" => Ok((
            "unified-diff-preferred",
            "- Commit-message rules remain mandatory. Patch agents express intent through the `@work-leaf edit <reason>` reason; final commit-message compliance is enforced through patch reason and final linearized commits.",
            "- Commit-message rules remain mandatory. Patch agents express intent through the `@work-leaf patch <reason>` or `@work-leaf edit <reason>` reason; final commit-message compliance is enforced through patch reason and final linearized commits.",
        )),
        _ => Err(invalid("unsupported candidate policy span")),
    }
}

pub(crate) fn forward_candidate_policy(
    agent_id: &AgentId,
    original: String,
    spans: Vec<PromptSpan>,
) -> io::Result<String> {
    let Some(experiment) = active()?.filter(|experiment| experiment.manifest.schema == SCHEMA_V4)
    else {
        return Ok(original);
    };
    let mut candidate = String::with_capacity(original.len());
    let mut components = Vec::with_capacity(spans.len());
    let mut metadata = Vec::with_capacity(spans.len());
    let mut cursor = 0;
    for span in spans {
        let (condition, expected, replacement) = policy_pair(span.id)?;
        if span.cue.start < cursor || original.get(span.cue.clone()) != Some(expected) {
            return Err(invalid(
                "candidate policy spans must be ordered, disjoint and match their declared text",
            ));
        }
        candidate.push_str(&original[cursor..span.cue.start]);
        let start = candidate.len();
        candidate.push_str(if experiment.manifest.condition == condition {
            replacement
        } else {
            expected
        });
        components.push(ReadComponent {
            baseline_start: span.cue.start,
            baseline_end: span.cue.end,
            inline_start: start,
            inline_end: candidate.len(),
        });
        metadata.push(json!({"id": span.id, "condition": condition, "original": expected, "replacement": replacement}));
        cursor = span.cue.end;
    }
    candidate.push_str(&original[cursor..]);
    forward_candidate_with(
        experiment,
        "candidate-policy",
        agent_id,
        original,
        candidate,
        components,
        json!({"spans": metadata}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component(bs: usize, be: usize, cs: usize, ce: usize) -> ReadComponent {
        ReadComponent {
            baseline_start: bs,
            baseline_end: be,
            inline_start: cs,
            inline_end: ce,
        }
    }

    #[test]
    fn component_validation_covers_insertions_unicode_and_multiple_owned_spans() {
        assert!(
            validate_components("λ|old|tail", "λ|new text|tail", &[component(3, 6, 3, 11)]).is_ok()
        );
        assert!(validate_components("λ|tail", "λ|request|tail", &[component(3, 3, 3, 11)]).is_ok());
        assert!(
            validate_components(
                "a1b2c",
                "aAAA bBBc",
                &[component(1, 2, 1, 5), component(3, 4, 6, 8)]
            )
            .is_ok()
        );
        assert!(validate_components("same", "same", &[]).is_ok());
        assert!(validate_components("same", "changed", &[]).is_err());
        assert!(validate_components("λ|old|tail", "λ|new|TAIL", &[component(3, 6, 3, 6)]).is_err());
        assert!(validate_components("λ|old|tail", "λ|new|tail", &[component(1, 6, 1, 6)]).is_err());
        assert!(
            validate_components(
                "a1b2c",
                "aAbBc",
                &[component(3, 4, 3, 4), component(1, 2, 1, 2)]
            )
            .is_err()
        );
        assert!(validate_components("a1b", "a2b", &[component(2, 1, 1, 2)]).is_err());
        assert!(validate_components("a1b", "a2b", &[component(1, usize::MAX, 1, 2)]).is_err());
    }

    #[test]
    fn evidence_failure_never_returns_either_selected_or_identity_prompt() {
        for condition in [
            "requested-repeat-full",
            "unified-diff-preferred",
            "review-fix-request-resupply",
        ] {
            let experiment = Experiment {
                manifest: Manifest {
                    schema: SCHEMA_V4.to_string(),
                    run_id: "candidate-io".to_string(),
                    condition: condition.to_string(),
                    evidence_path: std::env::current_exe().unwrap(),
                },
                evidence: Mutex::new(Evidence {
                    file: File::open(std::env::current_exe().unwrap()).unwrap(),
                    sequence: 0,
                }),
            };
            assert!(
                forward_candidate_with(
                    &experiment,
                    "requested-repeat-read",
                    &AgentId::new("a").unwrap(),
                    "prefix old suffix".to_string(),
                    "prefix new suffix".to_string(),
                    vec![component(7, 10, 7, 10)],
                    json!({})
                )
                .is_err()
            );
        }
    }

    #[test]
    #[cfg(unix)]
    fn nonfactor_selection_preserves_original_and_records_candidate_symmetrically() {
        for condition in [
            "requested-repeat-full",
            "unified-diff-preferred",
            "review-fix-request-resupply",
        ] {
            let experiment = Experiment {
                manifest: Manifest {
                    schema: SCHEMA_V4.to_string(),
                    run_id: "candidate-selection".to_string(),
                    condition: condition.to_string(),
                    evidence_path: PathBuf::from("/dev/null"),
                },
                evidence: Mutex::new(Evidence {
                    file: OpenOptions::new().write(true).open("/dev/null").unwrap(),
                    sequence: 0,
                }),
            };
            for (site, selected_condition) in [
                ("requested-repeat-read", "requested-repeat-full"),
                ("review-fix-request", "review-fix-request-resupply"),
            ] {
                let result = forward_candidate_with(
                    &experiment,
                    site,
                    &AgentId::new("a").unwrap(),
                    "old".to_string(),
                    "new".to_string(),
                    vec![component(0, 3, 0, 3)],
                    json!({}),
                )
                .unwrap();
                assert_eq!(
                    result,
                    if condition == selected_condition {
                        "new"
                    } else {
                        "old"
                    }
                );
            }
            assert_eq!(experiment.evidence.lock().unwrap().sequence, 2);
        }
    }
}
