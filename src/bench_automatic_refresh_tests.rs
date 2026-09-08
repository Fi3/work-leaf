use super::*;

#[test]
fn owned_components_preserve_unicode_marker_data_and_body_without_formatter_newline() {
    let original = "λ compact\nold diff\ntail";
    let mut capture = Capture::default();
    capture.replace("refresh-guidance", 3..10, "full".into(), None, None);
    let held = "current full text:\nλ marker";
    capture.replace(
        "current-full-text",
        11..20,
        format!("current full text:\n{held}\n"),
        Some(19..19 + held.len()),
        Some(0),
    );
    let (candidate, parts) = capture.candidate(original).unwrap();
    assert_eq!(
        candidate,
        format!("λ full\ncurrent full text:\n{held}\ntail")
    );
    let start = parts[1]["body_start"].as_u64().unwrap() as usize;
    let end = parts[1]["body_end"].as_u64().unwrap() as usize;
    assert_eq!(&candidate[start..end], held);
}

#[test]
fn identity_without_eligible_body_keeps_coherence_and_has_no_components() {
    let mut capture = Capture::default();
    capture.replace("refresh-guidance", 0..7, "full".into(), None, None);
    let (candidate, parts) = capture.candidate("compact only").unwrap();
    assert_eq!(candidate, "compact only");
    assert!(parts.is_empty());
}

#[test]
fn invalid_owned_components_fail_closed() {
    for range in [1..2, Range { start: 4, end: 3 }, 0..100] {
        let mut capture = Capture::default();
        capture.replace(
            "current-full-text",
            range,
            "full".into(),
            Some(0..4),
            Some(0),
        );
        assert!(capture.candidate("λ data").is_err());
    }
    let mut capture = Capture::default();
    capture.replace("current-full-text", 3..5, "λ".into(), Some(1..2), Some(0));
    assert!(capture.candidate("λ data").is_err());
}

#[test]
fn source_evidence_failure_blocks_selected_and_identity_results() {
    for eligible in [false, true] {
        let experiment = Experiment {
            manifest: Manifest {
                schema: SCHEMA_V7.into(),
                run_id: "failure".into(),
                condition: "automatic-changed-refresh-full".into(),
                evidence_path: std::env::current_exe().unwrap(),
            },
            evidence: Mutex::new(Evidence {
                file: File::open(std::env::current_exe().unwrap()).unwrap(),
                sequence: 0,
            }),
        };
        let mut capture = Capture::default();
        if eligible {
            capture.replace(
                "current-full-text",
                0..3,
                "full".into(),
                Some(0..4),
                Some(0),
            );
        }
        assert!(
            capture
                .forward_with(
                    &experiment,
                    &AgentId::new("owner").unwrap(),
                    "edit",
                    "old".into(),
                    json!({"diagnostic":"diagnostic","files":[],"failures":[]})
                )
                .is_err()
        );
    }
}
