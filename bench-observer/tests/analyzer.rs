use std::collections::BTreeSet;
use std::fs;
use std::process::Command;

use tempfile::TempDir;
use work_leaf_bench_observer::{
    BundleArchiveObservation, CapturedUsage, CommandObservation, EvidenceInput, InitSpec,
    LockedCommandObservation, MechanismAnalyzer, TimelineObservation, TurnOutcome,
    UsageObservation, archive_context_bundles, capture_git_checkpoint, index_jsonl, initialize,
    record_timeline, summarize_usage,
};

fn usage(input: u64, cached: u64, output: u64, reasoning: u64) -> CapturedUsage {
    CapturedUsage {
        input_tokens: input,
        cached_input_tokens: cached,
        output_tokens: output,
        reasoning_output_tokens: reasoning,
    }
}

fn content_digest(text: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv64:{hash:016x}; bytes:{}", text.len())
}

#[test]
fn jsonl_index_ignores_provider_json_embedded_in_command_output() {
    let bytes = concat!(
        "{\"type\":\"thread.started\",\"thread_id\":\"primary\"}\n",
        "{\"type\":\"item.completed\",\"item\":{\"type\":\"command_execution\",",
        "\"aggregated_output\":\"{\\\"type\\\":\\\"turn.completed\\\",",
        "\\\"usage\\\":{\\\"input_tokens\\\":999999}}\\n\"}}\n",
        "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":100,",
        "\"cached_input_tokens\":80,\"output_tokens\":10,",
        "\"reasoning_output_tokens\":5}}\n",
    )
    .as_bytes();

    let frames = index_jsonl(bytes, "server-to-client");
    assert_eq!(frames.len(), 3);
    let usage_frames = frames
        .iter()
        .filter(|frame| frame.usage.is_some())
        .collect::<Vec<_>>();
    assert_eq!(usage_frames.len(), 1);
    assert_eq!(
        usage_frames[0].usage_kind.as_deref(),
        Some("invocation-total")
    );
    assert_eq!(usage_frames[0].usage, Some(usage(100, 80, 10, 5)));
}

#[test]
fn app_server_total_usage_and_malformed_frames_remain_distinct() {
    let bytes = concat!(
        "{\"method\":\"thread/tokenUsage/updated\",\"params\":{\"threadId\":\"thread-a\",",
        "\"tokenUsage\":{\"last\":{\"inputTokens\":10,\"cachedInputTokens\":8,",
        "\"outputTokens\":2,\"reasoningOutputTokens\":1},",
        "\"total\":{\"inputTokens\":100,\"cachedInputTokens\":80,",
        "\"outputTokens\":20,\"reasoningOutputTokens\":10}}}}\n",
        "not-json\n",
        "{\"id\":7,\"method\":\"turn/interrupt\",\"params\":{\"threadId\":\"thread-a\",",
        "\"turnId\":\"turn-a\"}}\n",
    )
    .as_bytes();
    let frames = index_jsonl(bytes, "server-to-client");
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0].thread_id.as_deref(), Some("thread-a"));
    assert_eq!(frames[0].usage_kind.as_deref(), Some("thread-total"));
    assert_eq!(frames[0].usage, Some(usage(100, 80, 20, 10)));
    assert!(!frames[1].parsed);
    assert!(frames[1].parse_error.is_some());
    assert_eq!(frames[2].rpc_id.as_deref(), Some("7"));
    assert_eq!(frames[2].turn_id.as_deref(), Some("turn-a"));
}

#[test]
fn usage_scopes_keep_one_final_cumulative_snapshot_per_thread() {
    let observations = vec![
        UsageObservation::new("feature", "launch", true, true, usage(100, 80, 10, 5)),
        UsageObservation::new("feature", "resume", true, true, usage(150, 120, 15, 8)),
        UsageObservation::new("title", "app-server", true, false, usage(20, 10, 2, 1)),
        UsageObservation::new("nested", "child", false, false, usage(30, 0, 3, 2)),
    ];

    let scopes = summarize_usage(&observations);
    assert_eq!(scopes.visible_role.thread_count, 1);
    assert_eq!(scopes.visible_role.usage, usage(150, 120, 15, 8));
    assert_eq!(scopes.primary_condition.thread_count, 2);
    assert_eq!(scopes.primary_condition.usage, usage(170, 130, 17, 9));
    assert_eq!(scopes.total_workflow.thread_count, 3);
    assert_eq!(scopes.total_workflow.usage, usage(200, 130, 20, 11));
}

#[test]
fn mechanism_analyzer_exposes_evidence_for_every_registered_hypothesis() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe(EvidenceInput::Prompt(
        "work-leaf file text\n\nRepeated file reads unchanged\n- src/lib.rs (fnv64:1; bytes:1)\n"
            .into(),
    ));
    analyzer.observe(EvidenceInput::Prompt(
        "work-leaf file text\n\nRepeated file reads with changes\nstatus: changed since this agent's last snapshot\n"
            .into(),
    ));
    analyzer.observe(EvidenceInput::Prompt(
        "work-leaf file text\nExact file text is in an orchestrator context bundle instead of this chat\n"
            .into(),
    ));
    analyzer.observe(EvidenceInput::Prompt(
        "work-leaf command result\nstdout:\n...\nstderr:\n".into(),
    ));
    analyzer.observe(EvidenceInput::Prompt(
        "work-leaf patch applied\nDo not resend this patch\n".into(),
    ));
    analyzer.observe(EvidenceInput::Prompt(
        "Work Leaf collected this context from commits, git logs, and recorded chat history\n"
            .into(),
    ));
    analyzer.observe(EvidenceInput::Prompt(
        "You are the work-leaf linearizer\nFinal patch targets (3):\n".into(),
    ));
    analyzer.observe(EvidenceInput::Interrupt);
    analyzer.observe(EvidenceInput::ThreadTopology);
    analyzer.observe(EvidenceInput::Command {
        class: "source-read".into(),
        repeated: true,
    });
    analyzer.observe(EvidenceInput::Usage);
    analyzer.observe(EvidenceInput::SequentialTimeline);
    analyzer.observe(EvidenceInput::GenerationUsage);
    analyzer.observe(EvidenceInput::GitCheckpoint);
    analyzer.observe(EvidenceInput::AccountingReconciliation);
    analyzer.observe(EvidenceInput::ProtocolBytes(42));

    let summary = analyzer.finish();
    let statuses = summary
        .counterfactuals
        .iter()
        .map(|record| (record.hypothesis.as_str(), record.status.as_str()))
        .collect::<BTreeSet<_>>();
    assert!(statuses.contains(&("H1", "requires-snapshot-resolution")));
    assert!(statuses.contains(&("H2", "requires-diff-verification")));
    assert!(statuses.contains(&("H3", "descriptive-observed-path")));
    assert!(statuses.contains(&("H4", "requires-locked-command-pairing")));
    assert!(statuses.contains(&("H7", "requires-target-reconstruction")));
    assert!(statuses.contains(&("H8", "requires-ablation")));

    let observed = summary
        .hypotheses
        .iter()
        .filter(|entry| entry.observed)
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    let expected = (1..=16).map(|number| format!("H{number}")).collect();
    assert_eq!(observed, expected);

    let golden = serde_json::json!({
        "observed_hypotheses": summary
            .hypotheses
            .iter()
            .filter(|entry| entry.observed)
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        "counterfactual_statuses": summary
            .counterfactuals
            .iter()
            .map(|record| [&record.hypothesis, &record.status])
            .collect::<Vec<_>>(),
        "command_classes": summary.command_classes,
        "repeated_commands": summary.repeated_commands,
        "reviews": summary.reviews,
        "errors": summary.errors,
    });
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/mechanism-golden.json")).unwrap();
    assert_eq!(golden, expected);
}

#[test]
fn deterministic_mechanism_counterfactuals_are_verified_from_captured_evidence() {
    let mut analyzer = MechanismAnalyzer::default();
    let before = "before\n";
    let after = "after\n";
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-read-initial",
        &format!("work-leaf file text\n\n--- README.md ---\n{before}"),
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-read-unchanged",
        &format!(
            "work-leaf file text\n\nRepeated file reads unchanged\n\
             Work Leaf already sent this agent the exact text for these files.\n\
             - README.md ({})\n",
            content_digest(before)
        ),
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-read-changed",
        &format!(
            "work-leaf file text\n\nRepeated file reads with changes\n\
             These files changed since this agent's last mediated snapshot.\n\n\
             --- README.md ---\n\
             current digest: {}\n\
             previous digest: {}\n\
             status: changed since this agent's last snapshot\n\
             diff --git a/README.md b/README.md\n\
             --- a/README.md\n\
             +++ b/README.md\n\
             @@ -1 +1 @@\n\
             -before\n\
             +after\n",
            content_digest(after),
            content_digest(before)
        ),
    );

    let raw_stdout = "x".repeat(13_000);
    let compacted_stdout = format!(
        "{}\n[work-leaf compacted 9800 characters from one long output line]\n{}\n",
        "x".repeat(1_600),
        "x".repeat(1_600)
    );
    analyzer.observe_locked_command(LockedCommandObservation {
        invocation_id: "locked-1".into(),
        command: "cargo test --test focused".into(),
        stdout: raw_stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
        exit_code: Some(0),
        terminating_signal: None,
    });
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-command-result",
        &format!(
            "work-leaf command result\n\
             command: cargo test --test focused\n\
             status: 0\n\
             locked paths: .\n\
             next: continue\n\
             stdout:\n{compacted_stdout}\
             stderr:\n<empty>\n"
        ),
    );

    let bundle_path = "/tmp/work-leaf-context/bundle-0.md";
    let bundle =
        b"----- BEGIN FILE src/lib.rs -----\nexact bundle text\n----- END FILE src/lib.rs -----\n";
    analyzer.observe_bundle_archive(BundleArchiveObservation {
        source_path: bundle_path.into(),
        archived_path: "files/bundle-0.md".into(),
        bytes: bundle.to_vec(),
        sha256: "fixture-sha".into(),
    });
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-bundle",
        &format!(
            "work-leaf file text\n\
             Exact file text is in an orchestrator context bundle instead of this chat to keep the agent session compact.\n\
             Context bundle: {bundle_path}\n\
             Bundled files:\n- src/lib.rs ({})\n",
            content_digest("exact bundle text\n")
        ),
    );
    analyzer.observe_command(CommandObservation {
        thread_id: "thread-user-1".into(),
        turn_id: "turn-bundle-read".into(),
        command: format!("sed -n '1,2p' {bundle_path}"),
        output: b"----- BEGIN FILE src/lib.rs -----\nexact bundle text\n".to_vec(),
        duration_ns: Some(10),
    });

    analyzer.observe_prompt(
        "thread-linearize",
        "turn-linearize",
        "You are the work-leaf linearizer for reviewed agent patches.\n\n\
         Final patch targets (1):\n\
         - Agent-ID: user-1\n\
           Commit: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n\
           Feature: second feature\n\
           Reason: Linearize 2 reviewed commits through bbbbbbbbbbbb\n\
           Subject: FIX second\n\
           Context: Linearize target includes 2 reviewed provisional commits for patch agent user-1. Fold every listed reviewed commit into one final feature commit.\n\n\
         Reviewed commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
         Subject: ADD first\n\
         Feature: first feature\n\
         Reason: first reason\n\
         Context: first context\n\n\
         Reviewed commit: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n\
         Subject: FIX second\n\
         Feature: second feature\n\
         Reason: second reason\n\
         Context: second context\n\n\
         Scope and commit-shaping rules:\n",
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    for hypothesis in ["H1", "H2", "H4", "H7"] {
        let records = summary
            .counterfactuals
            .iter()
            .filter(|record| record.hypothesis == hypothesis)
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 1, "{hypothesis}: {records:?}");
        assert_eq!(records[0].status, "verified", "{hypothesis}");
        assert!(records[0].actual_component_bytes.is_some());
        assert!(records[0].counterfactual_component_bytes.is_some());
        assert!(records[0].avoided_bytes.is_some());
    }
    assert_eq!(summary.bundles.len(), 1);
    assert_eq!(summary.bundles[0].consumption, "partial");
    assert_eq!(summary.bundles[0].payload_bytes, bundle.len() as u64);
    assert!(summary.bundles[0].observed_follow_up_bytes > 0);
}

#[test]
fn command_results_and_bundle_reads_resolve_through_disjoint_index_buckets() {
    const CASES: usize = 128;
    let mut analyzer = MechanismAnalyzer::default();
    for index in 0..CASES {
        let command = format!("cargo test --test indexed case-{index}");
        let output = format!("result-{index}\n");
        analyzer.observe_locked_command(LockedCommandObservation {
            invocation_id: format!("locked-{index}"),
            command: command.clone(),
            stdout: output.as_bytes().to_vec(),
            stderr: Vec::new(),
            exit_code: Some(0),
            terminating_signal: None,
        });
        analyzer.observe_prompt(
            "thread-indexed",
            &format!("turn-result-{index}"),
            &format!(
                "work-leaf command result\ncommand: {command}\nstatus: 0\n\
                 locked paths: .\nstdout:\n{output}stderr:\n<empty>\n"
            ),
        );

        let path = format!("/tmp/work-leaf-indexed-bundle-{index}.md");
        let payload = format!("bundle-{index}\n");
        analyzer.observe_bundle_archive(BundleArchiveObservation {
            source_path: path.clone().into(),
            archived_path: format!("files/bundle-{index}.md").into(),
            bytes: payload.as_bytes().to_vec(),
            sha256: format!("fixture-sha-{index}"),
        });
        analyzer.observe_prompt(
            "thread-indexed",
            &format!("turn-bundle-{index}"),
            &format!(
                "work-leaf file text\n\
                 Exact file text is in an orchestrator context bundle instead of this chat.\n\
                 Context bundle: {path}\nBundled files:\n"
            ),
        );
        analyzer.observe_command(CommandObservation {
            thread_id: "thread-indexed".into(),
            turn_id: format!("turn-read-{index}"),
            command: if index == 0 {
                format!("bash -lc 'sed -n 1p {path}'")
            } else {
                format!("cat {path}")
            },
            output: payload.into_bytes(),
            duration_ns: Some(1),
        });
    }

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert_eq!(
        summary
            .counterfactuals
            .iter()
            .filter(|row| row.hypothesis == "H4" && row.status == "verified")
            .count(),
        CASES
    );
    assert_eq!(summary.bundles.len(), CASES);
    assert!(
        summary
            .bundles
            .iter()
            .all(|bundle| bundle.consumption == "full")
    );
}

#[test]
fn bundle_command_path_fanout_is_bounded() {
    let mut analyzer = MechanismAnalyzer::default();
    let mut paths = Vec::new();
    for index in 0..65 {
        let path = format!("/tmp/work-leaf-bundle-fanout-{index}.md");
        analyzer.observe_bundle_archive(BundleArchiveObservation {
            source_path: path.clone().into(),
            archived_path: format!("files/bundle-fanout-{index}.md").into(),
            bytes: b"bundle payload".to_vec(),
            sha256: format!("bundle-fanout-{index}-sha"),
        });
        analyzer.observe_prompt(
            "thread-user-1",
            &format!("turn-bundle-{index}"),
            &format!(
                "work-leaf file text\nExact file text is in an orchestrator context bundle instead of this chat.\nContext bundle: {path}\n"
            ),
        );
        paths.push(path);
    }
    analyzer.observe_command(CommandObservation {
        thread_id: "thread-user-1".into(),
        turn_id: "turn-read-many-bundles".into(),
        command: format!("cat {}", paths.join(" ")),
        output: b"bundle payload".to_vec(),
        duration_ns: Some(1),
    });

    let summary = analyzer.finish();
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.contains("at most 64 context bundle paths")),
        "{:?}",
        summary.errors
    );
}

#[test]
fn invalid_and_ambiguous_mechanism_evidence_is_rejected_instead_of_priced() {
    let mut analyzer = MechanismAnalyzer::default();
    let before = "before\n";
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-initial",
        &format!("work-leaf file text\n\n--- README.md ---\n{before}"),
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-invalid-diff",
        &format!(
            "work-leaf file text\n\nRepeated file reads with changes\n\n\
             --- README.md ---\n\
             current digest: {}\n\
             previous digest: {}\n\
             status: changed since this agent's last snapshot\n\
             diff --git a/README.md b/README.md\n\
             --- a/README.md\n\
             +++ b/README.md\n\
             @@ -1 +1 @@\n\
             -not-the-previous-text\n\
             +after\n",
            content_digest("after\n"),
            content_digest(before)
        ),
    );
    for invocation_id in ["locked-a", "locked-b"] {
        analyzer.observe_locked_command(LockedCommandObservation {
            invocation_id: invocation_id.into(),
            command: "cargo test --test focused".into(),
            stdout: b"ok\n".to_vec(),
            stderr: Vec::new(),
            exit_code: Some(0),
            terminating_signal: None,
        });
    }
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-ambiguous-command",
        "work-leaf command result\n\
         command: cargo test --test focused\n\
         status: 0\n\
         locked paths: .\n\
         stdout:\nok\n\
         stderr:\n<empty>\n",
    );

    let summary = analyzer.finish();
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.contains("diff reconstruction")),
        "{:?}",
        summary.errors
    );
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.contains("ambiguous locked-command pairing")),
        "{:?}",
        summary.errors
    );
    assert!(summary.counterfactuals.iter().all(|record| {
        record.status != "verified" || !matches!(record.hypothesis.as_str(), "H2" | "H4")
    }));
}

#[test]
fn locked_command_pairing_validates_status_and_timeout_metadata() {
    let mut mismatch = MechanismAnalyzer::default();
    mismatch.observe_locked_command(LockedCommandObservation {
        invocation_id: "locked-status".into(),
        command: "cargo test --test focused".into(),
        stdout: b"ok\n".to_vec(),
        stderr: Vec::new(),
        exit_code: Some(0),
        terminating_signal: None,
    });
    mismatch.observe_prompt(
        "thread-user-1",
        "turn-status-mismatch",
        "work-leaf command result\n\
         command: cargo test --test focused\n\
         status: 1\n\
         locked paths: .\n\
         stdout:\nok\n\
         stderr:\n<empty>\n",
    );
    let mismatch = mismatch.finish();
    assert!(
        mismatch
            .errors
            .iter()
            .any(|error| error.contains("status mismatch")),
        "{:?}",
        mismatch.errors
    );
    assert!(
        !mismatch
            .counterfactuals
            .iter()
            .any(|record| { record.hypothesis == "H4" && record.status == "verified" })
    );

    let mut timeout = MechanismAnalyzer::default();
    timeout.observe_locked_command(LockedCommandObservation {
        invocation_id: "locked-timeout".into(),
        command: "cargo test --test focused".into(),
        stdout: Vec::new(),
        stderr: Vec::new(),
        exit_code: None,
        terminating_signal: Some(libc::SIGTERM),
    });
    timeout.observe_prompt(
        "thread-user-1",
        "turn-timeout",
        "work-leaf command result\n\
         command: cargo test --test focused\n\
         status: terminated\n\
         locked paths: .\n\
         timed out: yes\n\
         timeout: 30s\n\
         user authorization is required to rerun locked commands for longer than this limit.\n\
         stdout:\n<empty>\n\
         stderr:\n<empty>\n",
    );
    let timeout = timeout.finish();
    assert!(timeout.errors.is_empty(), "{:?}", timeout.errors);
    let record = timeout
        .counterfactuals
        .iter()
        .find(|record| record.hypothesis == "H4")
        .unwrap();
    assert_eq!(record.status, "verified");
    assert!(record.note.contains("timed_out=true"));
}

#[test]
fn review_targets_must_resolve_to_exactly_one_captured_git_commit() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_git_commit_hashes([
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
    ]);
    analyzer.observe_prompt(
        "thread-review-1",
        "turn-review-valid",
        "Work Leaf collected this context from commits, git logs, and recorded chat history without querying Agent-ID user-1.\n\
         Git metadata:\nLatest commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
    );
    analyzer.observe_prompt(
        "thread-review-1",
        "turn-review-valid-duplicate",
        "Work Leaf collected this context from commits, git logs, and recorded chat history without querying Agent-ID user-1.\n\
         Git metadata:\nLatest commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
    );
    analyzer.observe_prompt(
        "thread-review-1",
        "turn-review-invalid",
        "Work Leaf collected this context from commits, git logs, and recorded chat history without querying Agent-ID user-1.\n\
         Git metadata:\nLatest commit: cccccccccccccccccccccccccccccccccccccccc\n",
    );

    let summary = analyzer.finish();
    assert_eq!(summary.reviews.prompts, 3);
    assert_eq!(summary.reviews.duplicate_targets, 1);
    assert_eq!(summary.reviews.validated_targets, 1);
    assert_eq!(summary.reviews.unresolved_targets, 1);
    assert!(
        summary
            .errors
            .iter()
            .any(|error| error.contains("review target") && error.contains("cccc")),
        "{:?}",
        summary.errors
    );
}

#[test]
fn changed_read_counterfactual_handles_files_without_trailing_newlines() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-initial",
        "work-leaf file text\n\n--- README.md ---\nbefore",
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-changed",
        &format!(
            "work-leaf file text\n\nRepeated file reads with changes\n\n\
             --- README.md ---\n\
             current digest: {}\n\
             previous digest: {}\n\
             status: changed since this agent's last snapshot\n\
             diff --git a/README.md b/README.md\n\
             --- a/README.md\n\
             +++ b/README.md\n\
             @@ -1 +1 @@\n\
             -before\n\
             \\ No newline at end of file\n\
             +after\n\
             \\ No newline at end of file\n",
            content_digest("after"),
            content_digest("before")
        ),
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert!(
        summary
            .counterfactuals
            .iter()
            .any(|record| { record.hypothesis == "H2" && record.status == "verified" })
    );
}

#[test]
fn changed_read_parser_keeps_unified_diff_headers_inside_each_file_block() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-initial",
        "work-leaf file text\n\n--- first.rs ---\none\n\n--- second.rs ---\nalpha\n",
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-changed",
        &format!(
            "work-leaf file text\n\nRepeated file reads with changes\n\n\
             --- first.rs ---\n\
             current digest: {}\n\
             previous digest: {}\n\
             status: changed since this agent's last snapshot\n\
             diff --git a/first.rs b/first.rs\n\
             --- a/first.rs\n\
             +++ b/first.rs\n\
             @@ -1 +1 @@\n\
             -one\n\
             +two\n\
             --- second.rs ---\n\
             current digest: {}\n\
             previous digest: {}\n\
             status: changed since this agent's last snapshot\n\
             diff --git a/second.rs b/second.rs\n\
             --- a/second.rs\n\
             +++ b/second.rs\n\
             @@ -1 +1 @@\n\
             -alpha\n\
             +beta\n",
            content_digest("two\n"),
            content_digest("one\n"),
            content_digest("beta\n"),
            content_digest("alpha\n"),
        ),
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert_eq!(
        summary
            .counterfactuals
            .iter()
            .filter(|record| record.hypothesis == "H2" && record.status == "verified")
            .count(),
        2
    );
}

#[test]
fn changed_read_parser_accepts_blank_separators_between_file_blocks() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-initial",
        "work-leaf file text\n\n--- first.rs ---\none\n\n--- second.rs ---\nalpha\n",
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-changed",
        &format!(
            "work-leaf file text\n\nRepeated file reads with changes\n\n\
             --- first.rs ---\n\
             current digest: {}\n\
             previous digest: {}\n\
             status: changed since this agent's last snapshot\n\
             diff --git a/first.rs b/first.rs\n\
             --- a/first.rs\n\
             +++ b/first.rs\n\
             @@ -1 +1 @@\n\
             -one\n\
             +two\n\n\
             --- second.rs ---\n\
             current digest: {}\n\
             previous digest: {}\n\
             status: changed since this agent's last snapshot\n\
             diff --git a/second.rs b/second.rs\n\
             --- a/second.rs\n\
             +++ b/second.rs\n\
             @@ -1 +1 @@\n\
             -alpha\n\
             +beta\n",
            content_digest("two\n"),
            content_digest("one\n"),
            content_digest("beta\n"),
            content_digest("alpha\n"),
        ),
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert_eq!(
        summary
            .counterfactuals
            .iter()
            .filter(|record| record.hypothesis == "H2" && record.status == "verified")
            .count(),
        2
    );
}

#[test]
fn announced_bundle_snapshots_resolve_unchanged_reads_for_the_same_thread() {
    let mut analyzer = MechanismAnalyzer::default();
    let source = "/tmp/work-leaf-context/bundle-thread.md";
    let exact = "exact bundle text\n";
    let digest = content_digest(exact);
    let bundle = format!(
        "# Work Leaf Context Bundle\n\n\
         This file contains orchestrator-mediated read output.\n\n\
         ----- BEGIN FILE src/lib.rs -----\n\
         digest: {digest}\n\n\
         {exact}\
         ----- END FILE src/lib.rs -----\n"
    );
    analyzer.observe_bundle_archive(BundleArchiveObservation {
        source_path: source.into(),
        archived_path: "files/bundle-thread.md".into(),
        bytes: bundle.into_bytes(),
        sha256: "fixture-sha".into(),
    });
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-bundle",
        &format!(
            "work-leaf file text\n\
             Exact file text is in an orchestrator context bundle instead of this chat.\n\
             Context bundle: {source}\n\
             Bundled files:\n- src/lib.rs ({digest})\n"
        ),
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-unchanged",
        &format!(
            "work-leaf file text\n\nRepeated file reads unchanged\n\
             Work Leaf already sent this agent the exact text for these files.\n\
             - src/lib.rs ({digest})\n"
        ),
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert!(summary.counterfactuals.iter().any(|record| {
        record.hypothesis == "H1"
            && record.status == "verified"
            && record.note.contains("src/lib.rs")
    }));
}

#[test]
fn bundle_manifest_rows_are_not_treated_as_unchanged_read_rows() {
    let mut analyzer = MechanismAnalyzer::default();
    let source = "/tmp/work-leaf-context/bundle-mixed.md";
    let bundled_text = "bundled exact text\n";
    let bundled_digest = content_digest(bundled_text);
    let unchanged_text = "previous exact text\n";
    let unchanged_digest = content_digest(unchanged_text);
    let bundle = format!(
        "# Work Leaf Context Bundle\n\n\
         This file contains orchestrator-mediated read output.\n\n\
         ----- BEGIN FILE src/bundled.rs -----\n\
         digest: {bundled_digest}\n\n\
         {bundled_text}\
         ----- END FILE src/bundled.rs -----\n"
    );
    analyzer.observe_bundle_archive(BundleArchiveObservation {
        source_path: source.into(),
        archived_path: "files/bundle-mixed.md".into(),
        bytes: bundle.into_bytes(),
        sha256: "fixture-sha".into(),
    });
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-initial",
        &format!("work-leaf file text\n\n--- src/unchanged.rs ---\n{unchanged_text}"),
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-mixed",
        &format!(
            "work-leaf file text\n\
             Exact file text is in an orchestrator context bundle instead of this chat.\n\
             Context bundle: {source}\n\
             Bundled files:\n\
             - src/bundled.rs ({bundled_digest})\n\n\
             Repeated file reads unchanged\n\
             Work Leaf already sent this agent the exact text for these files.\n\
             - src/unchanged.rs ({unchanged_digest})\n"
        ),
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    let h1 = summary
        .counterfactuals
        .iter()
        .filter(|record| record.hypothesis == "H1")
        .collect::<Vec<_>>();
    assert_eq!(h1.len(), 1, "{h1:?}");
    assert_eq!(h1[0].status, "verified");
    assert!(h1[0].note.contains("src/unchanged.rs"));
    assert!(!h1[0].note.contains("src/bundled.rs"));
}

#[test]
fn changed_full_current_delivery_establishes_the_next_digest_snapshot() {
    let mut analyzer = MechanismAnalyzer::default();
    let previous = "pub fn value() -> u8 { 1 }\n";
    let current = "pub fn value() -> u8 { 2 }\n";
    let previous_digest = content_digest(previous);
    let current_digest = content_digest(current);
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-initial",
        &format!("work-leaf file text\n\n--- src/lib.rs ---\n{previous}"),
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-full-current",
        &format!(
            "work-leaf file text\n\nRepeated file reads with changes\n\n\
             --- src/lib.rs ---\n\
             current digest: {current_digest}\n\
             previous digest: {previous_digest}\n\
             status: changed since this agent's last snapshot\n\
             full current text follows:\n\
             {current}"
        ),
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-unchanged",
        &format!(
            "work-leaf file text\n\nRepeated file reads unchanged\n\
             Work Leaf already sent this agent the exact text for these files.\n\
             - src/lib.rs ({current_digest})\n"
        ),
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert!(summary.counterfactuals.iter().any(|record| {
        record.hypothesis == "H2"
            && record.status == "full-current-delivery"
            && record.note.contains("src/lib.rs")
    }));
    assert!(summary.counterfactuals.iter().any(|record| {
        record.hypothesis == "H1"
            && record.status == "verified"
            && record.note.contains("src/lib.rs")
    }));
}

#[test]
fn unchanged_full_resend_is_not_counted_as_digest_saving() {
    let mut analyzer = MechanismAnalyzer::default();
    let text = "pub fn value() -> u8 { 1 }\n";
    let digest = content_digest(text);
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-initial",
        &format!("work-leaf file text\n\n--- src/lib.rs ---\n{text}"),
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-full-resend",
        &format!(
            "work-leaf file text\n\nRepeated file reads unchanged\n\
             Work Leaf already sent this agent the exact text for these files.\n\
             - src/lib.rs ({digest})\n\
             full current text follows for src/lib.rs:\n\
             {text}"
        ),
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert!(summary.counterfactuals.iter().any(|record| {
        record.hypothesis == "H1"
            && record.status == "full-current-delivery"
            && record.note.contains("src/lib.rs")
            && record.avoided_bytes.is_some_and(|bytes| bytes < 0)
    }));
    assert!(!summary.counterfactuals.iter().any(|record| {
        record.hypothesis == "H1"
            && record.status == "verified"
            && record.note.contains("src/lib.rs")
    }));
}

#[test]
fn announced_bundle_snapshots_do_not_cross_thread_identity() {
    let mut analyzer = MechanismAnalyzer::default();
    let source = "/tmp/work-leaf-context/bundle-isolated.md";
    let exact = "thread-owned bundle text\n";
    let digest = content_digest(exact);
    let bundle = format!(
        "# Work Leaf Context Bundle\n\n\
         This file contains orchestrator-mediated read output.\n\n\
         ----- BEGIN FILE src/lib.rs -----\n\
         digest: {digest}\n\n\
         {exact}\
         ----- END FILE src/lib.rs -----\n"
    );
    analyzer.observe_bundle_archive(BundleArchiveObservation {
        source_path: source.into(),
        archived_path: "files/bundle-isolated.md".into(),
        bytes: bundle.into_bytes(),
        sha256: "fixture-sha".into(),
    });
    analyzer.observe_prompt(
        "thread-owner",
        "turn-bundle",
        &format!(
            "work-leaf file text\n\
             Exact file text is in an orchestrator context bundle instead of this chat.\n\
             Context bundle: {source}\n\
             Bundled files:\n- src/lib.rs ({digest})\n"
        ),
    );
    analyzer.observe_prompt(
        "thread-other",
        "turn-unchanged",
        &format!(
            "work-leaf file text\n\nRepeated file reads unchanged\n\
             Work Leaf already sent this agent the exact text for these files.\n\
             - src/lib.rs ({digest})\n"
        ),
    );

    let summary = analyzer.finish();
    assert!(summary.errors.iter().any(|error| {
        error.contains("H1 snapshot resolution failed") && error.contains("thread-other")
    }));
    assert!(
        !summary
            .counterfactuals
            .iter()
            .any(|record| { record.hypothesis == "H1" && record.status == "verified" })
    );
}

#[test]
fn omitted_refresh_provenance_is_not_reported_as_unknown_capture_loss() {
    let mut analyzer = MechanismAnalyzer::default();
    let digest = "fnv64:e6892fb6530b2e73; bytes:57872";
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-refresh",
        &format!(
            "The orchestrator could not apply your edit.\n\n\
             work-leaf file refresh\n\
             This is a compact refresh, not a patch to submit.\n\n\
             --- src/ui.rs ---\n\
             current digest: {digest}\n\
             status: no previous snapshot recorded for this agent\n\
             current file text omitted: file is 57872 bytes. Request mediated file text with \
             `@work-leaf read src/ui.rs` if this file is still needed.\n"
        ),
    );
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-unchanged",
        &format!(
            "work-leaf file text\n\nRepeated file reads unchanged\n\
             Work Leaf already sent this agent the exact text for these files.\n\
             - src/ui.rs ({digest})\n"
        ),
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert!(summary.counterfactuals.iter().any(|record| {
        record.hypothesis == "H1"
            && record.status == "source-text-omitted"
            && record.note.contains("src/ui.rs")
            && record.counterfactual_component_bytes.is_none()
            && record.avoided_bytes.is_none()
    }));
    assert!(!summary.counterfactuals.iter().any(|record| {
        record.hypothesis == "H1"
            && record.status == "verified"
            && record.note.contains("src/ui.rs")
    }));
}

#[test]
fn protocol_text_embedded_in_review_history_is_not_current_thread_evidence() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_prompt(
        "thread-review",
        "turn-review",
        "You are reviewing a captured agent history.\n\n\
         3 user:\n\
         work-leaf file text\n\n\
         Repeated file reads unchanged\n\
         Work Leaf already sent this agent the exact text for these files.\n\
         - src/lib.rs (fnv64:deadbeefdeadbeef; bytes:4)\n",
    );

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert!(!summary.hypotheses[0].observed);
    assert!(
        summary
            .counterfactuals
            .iter()
            .all(|record| record.hypothesis != "H1")
    );
}

#[test]
fn descriptive_mechanism_rows_distinguish_outcomes_repeats_and_bundle_consumption() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_assistant(
        "thread-user-1",
        "turn-interrupted",
        "done\n@work-leaf done\n",
    );
    analyzer.observe_interrupt("thread-user-1", "turn-interrupted");
    analyzer.observe_turn_outcome(
        "thread-user-1",
        "turn-interrupted",
        TurnOutcome::Interrupted,
    );
    analyzer.observe_assistant("thread-user-1", "turn-natural", "done\n@work-leaf done\n");
    analyzer.observe_turn_outcome("thread-user-1", "turn-natural", TurnOutcome::Completed);
    analyzer.observe_assistant(
        "thread-user-1",
        "turn-edit-1",
        "@work-leaf edit\n*** Begin Patch\n*** End Patch\n",
    );
    analyzer.observe_assistant(
        "thread-user-1",
        "turn-edit-2",
        "@work-leaf edit\n*** Begin Patch\n*** End Patch\n",
    );
    for turn_id in ["turn-command-1", "turn-command-2"] {
        analyzer.observe_command(CommandObservation {
            thread_id: "thread-user-1".into(),
            turn_id: turn_id.into(),
            command: "rg -n value src".into(),
            output: b"src/lib.rs:1:value\n".to_vec(),
            duration_ns: Some(5),
        });
    }
    for (name, consumed) in [("unread", None), ("full", Some(b"full".as_slice()))] {
        let path = format!("/tmp/{name}-bundle.md");
        analyzer.observe_bundle_archive(BundleArchiveObservation {
            source_path: path.clone().into(),
            archived_path: format!("files/{name}-bundle.md").into(),
            bytes: b"full".to_vec(),
            sha256: format!("{name}-sha"),
        });
        analyzer.observe_prompt(
            "thread-user-1",
            &format!("turn-{name}-bundle"),
            &format!(
                "work-leaf file text\nExact file text is in an orchestrator context bundle instead of this chat.\nContext bundle: {path}\n"
            ),
        );
        if let Some(output) = consumed {
            analyzer.observe_command(CommandObservation {
                thread_id: "thread-user-1".into(),
                turn_id: format!("turn-{name}-read"),
                command: format!("cat {path}"),
                output: output.to_vec(),
                duration_ns: Some(2),
            });
        }
    }
    analyzer.observe_timeline(TimelineObservation {
        event: "feature-start".into(),
        detail: Some("feature=1".into()),
        monotonic_ns: 1,
    });
    analyzer.observe_timeline(TimelineObservation {
        event: "feature-cycle-complete".into(),
        detail: Some("feature=1".into()),
        monotonic_ns: 2,
    });

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert_eq!(summary.terminal_directives.interrupted, 1);
    assert_eq!(summary.terminal_directives.naturally_completed, 1);
    assert_eq!(summary.structured_edits.submissions, 2);
    assert_eq!(summary.structured_edits.duplicate_submissions, 1);
    assert_eq!(summary.repeated_commands, 1);
    assert_eq!(summary.command_count, 3);
    assert_eq!(summary.command_output_bytes, 42);
    assert_eq!(summary.command_duration_ns, 12);
    assert_eq!(summary.bundles.len(), 2);
    assert_eq!(summary.bundles[0].consumption, "full");
    assert_eq!(summary.bundles[1].consumption, "unread");
    assert!(summary.sequential_timeline_valid);
}

#[test]
fn bundle_reads_through_input_redirection_are_counted_as_consumed() {
    let mut analyzer = MechanismAnalyzer::default();
    let path = "/tmp/input-redirection-bundle.md";
    analyzer.observe_bundle_archive(BundleArchiveObservation {
        source_path: path.into(),
        archived_path: "files/input-redirection-bundle.md".into(),
        bytes: b"full bundle payload".to_vec(),
        sha256: "input-redirection-sha".into(),
    });
    analyzer.observe_prompt(
        "thread-user-1",
        "turn-bundle",
        &format!(
            "work-leaf file text\nExact file text is in an orchestrator context bundle instead of this chat.\nContext bundle: {path}\n"
        ),
    );
    analyzer.observe_command(CommandObservation {
        thread_id: "thread-user-1".into(),
        turn_id: "turn-read".into(),
        command: format!("cat < {path}"),
        output: b"full bundle payload".to_vec(),
        duration_ns: Some(1),
    });

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert_eq!(summary.bundles.len(), 1);
    assert_eq!(summary.bundles[0].consumption, "full");
}

#[test]
fn concurrent_timeline_accepts_additional_condition_metadata() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_timeline(TimelineObservation {
        event: "condition-start".into(),
        detail: Some("schedule=concurrent ablation=changed-full".into()),
        monotonic_ns: 1,
    });
    analyzer.observe_timeline(TimelineObservation {
        event: "feature-start".into(),
        detail: Some("features=1,2,3".into()),
        monotonic_ns: 2,
    });
    analyzer.observe_timeline(TimelineObservation {
        event: "feature-cycle-complete".into(),
        detail: Some("feature=3".into()),
        monotonic_ns: 3,
    });

    let summary = analyzer.finish();
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
    assert!(summary.sequential_timeline_valid);
}

#[test]
fn validation_activity_records_multiple_processes_without_judging_the_workflow() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_thread_role("thread-patch", "user-1");
    analyzer.observe_prompt(
        "thread-patch",
        "turn-launch",
        "Agent-ID: user-1\n\nImplement the requested feature and validate it as needed.\n",
    );
    for (turn_id, command) in [
        ("turn-check-1", "cargo test --test focused first_case"),
        (
            "turn-check-2",
            "/bin/bash -lc 'cargo check --package focused-package --lib'",
        ),
    ] {
        analyzer.observe_prompt("thread-patch", turn_id, "continue implementation");
        analyzer.observe_command(CommandObservation {
            thread_id: "thread-patch".into(),
            turn_id: turn_id.into(),
            command: command.into(),
            output: Vec::new(),
            duration_ns: Some(1),
        });
    }

    let summary = analyzer.finish();
    assert_eq!(summary.validation.validation_commands, 2);
    assert!(summary.validation.violations.is_empty());
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
}

#[test]
fn validation_activity_spans_implementation_and_review_fix_turns() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_thread_role("thread-patch", "user-1");
    analyzer.observe_prompt(
        "thread-patch",
        "turn-launch",
        "Agent-ID: user-1\n\nImplement the requested feature and validate it as needed.\n",
    );
    analyzer.observe_prompt(
        "thread-patch",
        "turn-implementation-check",
        "continue implementation",
    );
    analyzer.observe_command(CommandObservation {
        thread_id: "thread-patch".into(),
        turn_id: "turn-implementation-check".into(),
        command: "cargo test --test focused implementation_case".into(),
        output: Vec::new(),
        duration_ns: Some(1),
    });
    analyzer.observe_prompt(
        "thread-patch",
        "turn-fix-start",
        "The reviewer found issues in your patch for commit abc123.\nPlease fix the patch's code or test defects through the orchestrator patch flow.",
    );
    analyzer.observe_prompt("thread-patch", "turn-fix-check", "continue fix");
    analyzer.observe_command(CommandObservation {
        thread_id: "thread-patch".into(),
        turn_id: "turn-fix-check".into(),
        command: "cargo check --package focused-package --lib".into(),
        output: Vec::new(),
        duration_ns: Some(1),
    });

    let summary = analyzer.finish();
    assert_eq!(summary.validation.validation_commands, 2);
    assert!(
        summary.validation.violations.is_empty(),
        "{:?}",
        summary.validation.violations
    );
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
}

#[test]
fn validation_activity_allows_a_turn_without_a_cargo_process() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_thread_role("thread-patch", "user-1");
    analyzer.observe_prompt(
        "thread-patch",
        "turn-launch",
        "Agent-ID: user-1\n\nImplement the requested feature and validate it as needed.\n",
    );

    let summary = analyzer.finish();
    assert_eq!(summary.validation.validation_commands, 0);
    assert!(summary.validation.violations.is_empty());
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
}

#[test]
fn validation_activity_allows_linearizer_checks() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_thread_role("thread-linearize", "linearize");
    analyzer.observe_command(CommandObservation {
        thread_id: "thread-linearize".into(),
        turn_id: "turn-linearize".into(),
        command: "cargo fmt".into(),
        output: Vec::new(),
        duration_ns: Some(1),
    });

    let summary = analyzer.finish();
    assert_eq!(summary.validation.validation_commands, 1);
    assert!(summary.validation.violations.is_empty());
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
}

#[test]
fn review_only_turns_are_measured_without_patch_iteration_caps() {
    let mut analyzer = MechanismAnalyzer::default();
    analyzer.observe_thread_role("thread-review", "feature-1-review-1");
    for command in [
        "cargo test --test focused first_case",
        "cargo test --test focused second_case",
    ] {
        analyzer.observe_command(CommandObservation {
            thread_id: "thread-review".into(),
            turn_id: "turn-review".into(),
            command: command.into(),
            output: Vec::new(),
            duration_ns: Some(1),
        });
    }

    let summary = analyzer.finish();
    assert_eq!(summary.validation.validation_commands, 2);
    assert!(
        summary.validation.violations.is_empty(),
        "{:?}",
        summary.validation.violations
    );
    assert!(summary.errors.is_empty(), "{:?}", summary.errors);
}

#[test]
fn bundle_archive_timeline_and_git_checkpoint_are_self_contained() {
    let root = TempDir::new().unwrap();
    let fixture = env!("CARGO_BIN_EXE_bench-observer-fixture");
    let observer = env!("CARGO_BIN_EXE_bench-observer");
    let config = initialize(InitSpec {
        root: root.path().join("observation"),
        study_id: "efficiency-causal-study".into(),
        pair_id: "pair-archive-test".into(),
        condition: "work-leaf".into(),
        run_id: "archive-test".into(),
        real_codex: fixture.into(),
        real_sh: "/bin/sh".into(),
        real_cargo: "/bin/true".into(),
        base_commit: "base".into(),
        experiment_commit: "experiment".into(),
        model: "gpt-5.5".into(),
        effort: "xhigh".into(),
        observer_executable: observer.into(),
    })
    .unwrap();

    let bundles = root.path().join("bundles/orchestrator-1");
    fs::create_dir_all(&bundles).unwrap();
    let exact_text = "exact bundle bytes\n";
    let exact_digest = content_digest(exact_text);
    let bundle = format!(
        "# Work Leaf Context Bundle\n\n\
         This file contains orchestrator-mediated read output.\n\
         \n----- BEGIN FILE src/lib.rs -----\n\
         digest: {}\n\n\
         {exact_text}\
         ----- END FILE src/lib.rs -----\n",
        exact_digest
    );
    fs::write(bundles.join("bundle-0.md"), &bundle).unwrap();
    assert_eq!(
        archive_context_bundles(&config, &root.path().join("bundles")).unwrap(),
        1
    );
    fs::remove_dir_all(root.path().join("bundles")).unwrap();
    assert_eq!(
        fs::read(
            config
                .root
                .join("context-bundles/files/orchestrator-1/bundle-0.md")
        )
        .unwrap(),
        bundle.as_bytes()
    );
    let manifest = fs::read_to_string(config.root.join("context-bundles/manifest.jsonl")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(manifest.trim()).unwrap();
    assert_eq!(
        manifest
            .pointer("/file_snapshots/0/path")
            .and_then(serde_json::Value::as_str),
        Some("src/lib.rs")
    );
    assert_eq!(
        manifest
            .pointer("/file_snapshots/0/digest")
            .and_then(serde_json::Value::as_str),
        Some(exact_digest.as_str())
    );
    assert!(manifest.get("parse_error").unwrap().is_null());

    let repository = root.path().join("repository");
    fs::create_dir_all(&repository).unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&repository)
            .status()
            .unwrap()
            .success()
    );
    Command::new("git")
        .args(["config", "user.email", "observer@example.com"])
        .current_dir(&repository)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Observer Test"])
        .current_dir(&repository)
        .status()
        .unwrap();
    fs::write(repository.join("tracked.txt"), "tracked\n").unwrap();
    Command::new("git")
        .args(["add", "tracked.txt"])
        .current_dir(&repository)
        .status()
        .unwrap();
    assert!(
        Command::new("git")
            .args(["commit", "-qm", "ADD observer fixture"])
            .current_dir(&repository)
            .status()
            .unwrap()
            .success()
    );

    capture_git_checkpoint(&config, &repository, "base state").unwrap();
    record_timeline(&config, "feature-start", Some("feature=1")).unwrap();
    assert!(
        config
            .root
            .join("git-checkpoints/files/base-state/commit-graph.txt")
            .is_file()
    );
    let timeline = fs::read_to_string(config.root.join("timeline.jsonl")).unwrap();
    assert!(timeline.contains("context-bundles-archived"));
    assert!(timeline.contains("git-checkpoint"));
    assert!(timeline.contains("feature-start"));
}
