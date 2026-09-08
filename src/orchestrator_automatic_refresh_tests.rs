use super::*;
use crate::bench_experiment::automatic_refresh::Capture;
use crate::locks::FileSnapshot;

fn snapshot(path: &str, text: &str) -> FileSnapshot {
    FileSnapshot {
        path: path.into(),
        text: text.into(),
    }
}

fn prompt() -> RefreshPrompt {
    let mut prompt = RefreshPrompt {
        text: "Diagnostic: compact file refresh is source data\n".into(),
        capture: Some(Capture::default()),
    };
    append_refresh_guidance(
        &mut prompt,
        "Rebase your patch against the compact file refresh below.",
        "Rebase your patch against the file refresh below.",
    );
    prompt
}

#[test]
fn ordinary_diff_limits_empty_unavailable_and_untracked_cap_keep_identity() {
    let id = AgentId::new("owner").unwrap();
    let tracker = FileReadTracker::default();
    tracker.record_snapshots(&id, &[snapshot("tracked", "old")]);
    for (diff, expected, eligible) in [
        (
            Some("d".repeat(MAX_AUTOMATIC_REFRESH_DIFF_BYTES)),
            "available",
            true,
        ),
        (
            Some("d".repeat(MAX_AUTOMATIC_REFRESH_DIFF_BYTES + 1)),
            "omitted",
            false,
        ),
        (Some(String::new()), "empty", false),
        (None, "unavailable", false),
    ] {
        let current = "λ".repeat(MAX_AUTOMATIC_FULL_REFRESH_BYTES);
        let mut rendered = prompt();
        let mut calls = 0;
        render_file_refresh_response_with_diff(
            &mut rendered,
            &id,
            &tracker,
            &[snapshot("tracked", &current)],
            &[],
            |_, before, after| {
                calls += 1;
                assert_eq!(before, "old");
                assert_eq!(after, current);
                diff.clone()
            },
        );
        assert_eq!(calls, 1);
        assert_eq!(
            tracker
                .snapshot_for(&id, Path::new("tracked"))
                .unwrap()
                .text,
            "old",
            "rendering never advances source state"
        );
        let capture = rendered.capture.unwrap();
        assert_eq!(capture.snapshots[0]["diff_disposition"], expected);
        let (candidate, components) = capture.candidate(&rendered.text).unwrap();
        assert_eq!(!components.is_empty(), eligible);
        assert_eq!(candidate != rendered.text, eligible);
        if eligible {
            assert!(candidate.contains(&current));
        } else {
            assert_eq!(candidate, rendered.text);
        }
    }
    for size in [
        MAX_AUTOMATIC_FULL_REFRESH_BYTES,
        MAX_AUTOMATIC_FULL_REFRESH_BYTES + 1,
    ] {
        let mut rendered = prompt();
        render_file_refresh_response_with_diff(
            &mut rendered,
            &id,
            &tracker,
            &[snapshot("untracked", &"x".repeat(size))],
            &[],
            |_, _, _| panic!("untracked cannot render a diff"),
        );
        let capture = rendered.capture.unwrap();
        let (candidate, parts) = capture.candidate(&rendered.text).unwrap();
        assert_eq!(candidate, rendered.text);
        assert!(parts.is_empty());
        assert_eq!(
            candidate.contains("current file text omitted"),
            size > MAX_AUTOMATIC_FULL_REFRESH_BYTES
        );
        assert_eq!(capture.snapshots[0]["class"], "untracked");
    }
}

#[test]
fn mixed_sections_preserve_untracked_unchanged_failure_and_owned_utf8_ranges() {
    let id = AgentId::new("owner").unwrap();
    let tracker = FileReadTracker::default();
    tracker.record_snapshots(
        &id,
        &[
            snapshot("changed", "old"),
            snapshot("same", "same"),
            snapshot("empty-current", "before"),
        ],
    );
    let current =
        "λ\ncurrent full text:\n--- same ---\nThis is a compact refresh, not a patch to submit.";
    let snapshots = [
        snapshot("changed", current),
        snapshot("same", "same"),
        snapshot("fresh", "fresh λ"),
        snapshot("empty-current", ""),
    ];
    let failures = [FileReadFailure {
        path: "absent".into(),
        diagnostic: "Rebase your patch against the compact file refresh below.".into(),
    }];
    let mut rendered = prompt();
    let mut calls = 0;
    render_file_refresh_response_with_diff(
        &mut rendered,
        &id,
        &tracker,
        &snapshots,
        &failures,
        |_, _, _| {
            calls += 1;
            Some("diff without newline".into())
        },
    );
    assert_eq!(calls, 2);
    let capture = rendered.capture.unwrap();
    assert_eq!(
        capture
            .snapshots
            .iter()
            .map(|row| row["class"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["changed", "unchanged", "untracked", "changed"]
    );
    assert_eq!(capture.failures[0]["diagnostic"], failures[0].diagnostic);
    let (candidate, parts) = capture.candidate(&rendered.text).unwrap();
    assert_eq!(parts.len(), 4);
    assert!(candidate.starts_with("Diagnostic: compact file refresh is source data\n"));
    assert!(candidate.ends_with(&render_file_read_failures(&failures)));
    let mut b = 0;
    let mut c = 0;
    for part in parts {
        let bs = part["baseline_start"].as_u64().unwrap() as usize;
        let be = part["baseline_end"].as_u64().unwrap() as usize;
        let cs = part["candidate_start"].as_u64().unwrap() as usize;
        let ce = part["candidate_end"].as_u64().unwrap() as usize;
        assert_eq!(&rendered.text[b..bs], &candidate[c..cs]);
        b = be;
        c = ce;
        if let Some(index) = part["snapshot_index"].as_u64() {
            let start = part["body_start"].as_u64().unwrap() as usize;
            let end = part["body_end"].as_u64().unwrap() as usize;
            assert_eq!(&candidate[start..end], snapshots[index as usize].text);
            assert_eq!(&candidate[end..end + 1], "\n");
        }
    }
    assert_eq!(&rendered.text[b..], &candidate[c..]);
}

#[test]
fn no_snapshot_failure_only_and_unchanged_refresh_are_identity() {
    let id = AgentId::new("owner").unwrap();
    let tracker = FileReadTracker::default();
    tracker.record_snapshots(&id, &[snapshot("same", "same")]);
    for snapshots in [Vec::new(), vec![snapshot("same", "same")]] {
        let mut rendered = prompt();
        render_file_refresh_response_with_diff(
            &mut rendered,
            &id,
            &tracker,
            &snapshots,
            &[FileReadFailure {
                path: "absent".into(),
                diagnostic: "missing".into(),
            }],
            |_, _, _| panic!("no changed file"),
        );
        let (candidate, parts) = rendered.capture.unwrap().candidate(&rendered.text).unwrap();
        assert_eq!(candidate, rendered.text);
        assert!(parts.is_empty());
    }
}
