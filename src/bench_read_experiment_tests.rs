use super::*;

fn record() -> ReadEvidence {
    ReadEvidence {
        component: Some(ReadComponent {
            baseline_start: 4,
            baseline_end: 12,
            inline_start: 4,
            inline_end: 8,
        }),
        bundle: ReadBundle {
            threshold_eligible: true,
            write_succeeded: true,
            path: Some(PathBuf::from("/fixture/bundle.md")),
        },
        eligibility_reason: "bundled-untracked",
        requested_paths: vec!["file.rs".to_string()],
        snapshots: vec![ReadSnapshot {
            path: "file.rs".to_string(),
            class: "untracked",
            bytes: 4,
            digest: "fnv64:fixture; bytes:4".to_string(),
            inline_body_start: Some(4),
            inline_body_end: Some(8),
        }],
        failures: Vec::new(),
    }
}

fn experiment(file: File, condition: &str) -> Experiment {
    Experiment {
        manifest: Manifest {
            schema: SCHEMA_V3.to_string(),
            run_id: "read-io-fixture".to_string(),
            condition: condition.to_string(),
            evidence_path: PathBuf::from("/fixture/evidence"),
        },
        evidence: Mutex::new(Evidence { file, sequence: 0 }),
    }
}

#[test]
fn read_evidence_write_failure_cannot_return_a_provider_prompt_in_either_arm() {
    for condition in ["control", "untracked-read-inline"] {
        let experiment = experiment(
            File::open(std::env::current_exe().unwrap()).unwrap(),
            condition,
        );
        let error = forward_read_with(
            &experiment,
            &AgentId::new("a").unwrap(),
            "pre|manifest|post".to_string(),
            "pre|λ\nz|post".to_string(),
            record(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("benchmark experiment"));
    }
}

#[test]
#[cfg(unix)]
fn malformed_owned_read_spans_and_ineligible_differences_fail_before_recording() {
    for case in 0..8 {
        let experiment = experiment(
            OpenOptions::new().write(true).open("/dev/null").unwrap(),
            "untracked-read-inline",
        );
        let mut record = record();
        match case {
            0 => record.component.as_mut().unwrap().baseline_end = usize::MAX,
            1 => record.component.as_mut().unwrap().inline_start = 5,
            2 => record.snapshots[0].inline_body_start = Some(5),
            3 => record.snapshots[0].inline_body_end = Some(20),
            4 => record.component = None,
            5 => record.bundle.write_succeeded = false,
            6 => record.bundle.path = None,
            7 => record.snapshots[0].class = "unsupported",
            _ => unreachable!(),
        }
        assert!(
            forward_read_with(
                &experiment,
                &AgentId::new("a").unwrap(),
                "pre|manifest|post".to_string(),
                "pre|λ\nz|post".to_string(),
                record
            )
            .is_err()
        );
        assert_eq!(experiment.evidence.lock().unwrap().sequence, 0);
    }
}

#[test]
#[cfg(unix)]
fn v3_policy_ack_command_and_linearizer_are_identity_in_both_arms() {
    for condition in ["control", "untracked-read-inline"] {
        let experiment = experiment(
            OpenOptions::new().write(true).open("/dev/null").unwrap(),
            condition,
        );
        for (site, original, spans) in [
            (
                "policy-injection",
                WORK_UNIT.to_string(),
                vec![PromptSpan::new(
                    "policy-buildable-work-unit",
                    0..WORK_UNIT.len(),
                )],
            ),
            (
                "policy-injection",
                TESTS_WORK_UNIT.to_string(),
                vec![PromptSpan::new(
                    "instruction-tests-work-unit",
                    0..TESTS_WORK_UNIT.len(),
                )],
            ),
            ("policy-injection", "linearizer λ".to_string(), vec![]),
            (
                "patch-applied",
                format!("{ACK}{REMAINING_WORK}"),
                vec![
                    PromptSpan::new("patch-applied-validation", 0..ACK.len()),
                    PromptSpan::new(
                        "patch-applied-remaining-work",
                        ACK.len()..ACK.len() + REMAINING_WORK.len(),
                    ),
                ],
            ),
            (
                "command-result",
                GUIDANCE.to_string(),
                vec![PromptSpan::new(
                    "command-result-guidance",
                    0..GUIDANCE.len(),
                )],
            ),
        ] {
            assert_eq!(
                forward_v2(
                    &experiment,
                    site,
                    &AgentId::new("a").unwrap(),
                    original.clone(),
                    spans
                )
                .unwrap(),
                original
            );
        }
    }
}
