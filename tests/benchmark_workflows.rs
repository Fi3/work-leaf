#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

// Frozen from the three `post_command 'new ...'` tasks in origin/master:bench-three-features.
const ORIGIN_MASTER_FEATURE_HASH: &str =
    "c27bb64412deeee646d8d25753b599bf1650050e7b761c9dc119611abac58d1a";
const ORIGIN_MASTER_FEATURES: [&str; 3] = [
    "add vim like visual mode for both panes when I do v I can select the text in focused the panes same keystrokes of vim y Y for copy maiusc V line select block select with ctrl v block select",
    "implement strict selected-agent slash command execution. When a selected agent chat message starts with / followed by a non-whitespace command token, Work Leaf must treat it as a backend command for that selected agent, execute it immediately, and append the backend command output in the chat. This is not the existing raw pass-through behavior: passing /status to the normal agent send/resume/model prompt path is insufficient and must be covered by a failing test. Add coverage for /status and /fork, including a test that proves the normal backend send path does not receive /status as an ordinary prompt.",
    "when review process is done the patch agent chat must be highlighted and ask is this feature done with yes/no; yes closes it, typing again reopens it",
];

#[test]
fn product_benchmarks_keep_concurrent_work_leaf_and_direct_sequential_codex_distinct() {
    let work_leaf = read("bench-three-features");
    let direct = read("bench-three-features-direct-common");
    let sequential = read("bench-three-features-sequential");

    assert!(work_leaf.contains("readonly feature_schedule=\"concurrent\""));
    assert!(work_leaf.contains("readonly bench_mode=\"work-leaf\""));
    assert!(!work_leaf.contains("WORK_LEAF_BENCH_FEATURE_SCHEDULE"));
    assert!(!work_leaf.contains("work-leaf-sequential-protocol"));
    assert!(work_leaf.contains("post_feature_command 1"));
    assert!(work_leaf.contains("post_feature_command 2"));
    assert!(work_leaf.contains("post_feature_command 3"));

    assert!(sequential.contains("WORK_LEAF_DIRECT_BENCH_AGENT=codex"));
    assert!(sequential.contains("bench-three-features-direct-common"));
    assert!(!direct.contains("\"$bin_dir/work-leaf-orchestrator\""));
    assert!(!direct.contains("/agent/message"));
    assert!(direct.contains("if [[ \"$mode\" != \"sequential\" ]]"));
    assert!(direct.contains("readonly feature_schedule=\"sequential\""));
    assert!(direct.contains("- feature_schedule: $feature_schedule"));
    assert!(direct.contains("--arg feature_schedule \"$feature_schedule\""));
    assert!(direct.contains("feature_schedule:$feature_schedule"));
    assert!(!direct.contains("run_worktree_bench"));
    assert!(!direct.contains("bench-three-features-worktree"));
}

#[test]
fn product_benchmarks_preserve_the_origin_master_task_bytes() {
    let work_leaf = read("bench-three-features");
    let direct = read("bench-three-features-direct-common");

    let work_leaf_features = [1, 2, 3].map(|index| work_leaf_feature(&work_leaf, index));
    let direct_features = [1, 2, 3].map(|index| direct_feature(&direct, index));
    assert_eq!(work_leaf_features, ORIGIN_MASTER_FEATURES);
    assert_eq!(direct_features, ORIGIN_MASTER_FEATURES);

    let compact_json = serde_json::to_string(&ORIGIN_MASTER_FEATURES).unwrap() + "\n";
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(compact_json.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .next(),
        Some(ORIGIN_MASTER_FEATURE_HASH)
    );

    assert!(!direct.contains("A /fork implementation must"));
    assert!(!direct.contains("excluded ordinary prompt path"));
    assert!(!direct.contains("Provider-backed /status output"));
}

#[test]
fn paired_product_benchmarks_apply_the_same_validation_policy_and_final_gate() {
    let work_leaf = read("bench-three-features");
    let direct = read("bench-three-features-direct-common");
    let validation = read("bench-validation-common");

    for script in [&work_leaf, &direct] {
        assert!(script.contains("Run exactly one focused Cargo validation command"));
        assert!(script.contains("Starting no Cargo validation, or starting more than one"));
        assert!(script.contains("The benchmark driver owns the one final"));
        assert!(script.contains("disable_iteration_validation_budget"));
        assert!(script.contains("WORK_LEAF_BENCH_OBSERVER_BIN"));
        assert!(script.contains("source \"$repo_root/bench-candidate-common\""));
        assert!(script.contains("source \"$repo_root/bench-validation-common\""));
        assert_eq!(script.matches("bench_run_final_gate").count(), 1);
        assert!(script.contains("final repository changed during the final gate"));
        assert!(!script.contains("cargo fmt -- --check"));
        assert!(!script.contains("Run the repository required checks"));
    }

    assert_eq!(validation.matches("cargo fmt || exit $?\n").count(), 1);
    assert_eq!(
        validation
            .matches("cargo clippy --all-targets --all-features -- -D warnings || exit $?\n")
            .count(),
        1
    );
    assert_eq!(
        validation
            .matches("cargo test --all-targets --all-features || exit $?\n")
            .count(),
        1
    );

    assert!(work_leaf.contains("setup_observer || fail_bench"));
    assert!(direct.contains("setup_observer || fail_bench"));
    assert!(direct.matches("$(focused_validation_policy)").count() >= 2);
}

#[test]
fn direct_validation_audit_matches_the_shared_focused_policy_table() {
    let root = test_dir("validation-audit");
    for (index, (accepted, command)) in focused_validation_policy_cases().into_iter().enumerate() {
        let log = root.join(format!("case-{index}.jsonl"));
        write_command_log(&log, &[command]);
        let output = Command::new("python3")
            .args(["bench-audit-agent-validation", log.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            accepted,
            "direct auditor disagreed for `{command}`\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn direct_validation_audit_bounds_nested_shell_payloads() {
    let root = test_dir("validation-audit-nested-shell");
    for (depth, accepted) in [(4, true), (5, false)] {
        let command = nested_shell_command(depth);
        let log = root.join(format!("depth-{depth}.jsonl"));
        write_command_log(&log, &[&command]);
        let output = Command::new("python3")
            .args(["bench-audit-agent-validation", log.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            accepted,
            "unexpected depth-{depth} result for `{command}`\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn direct_validation_audit_rejects_broad_multiline_shell_commands() {
    let root = test_dir("validation-audit-multiline-shell");
    for (index, command) in [
        "cargo test;\n",
        "cargo test;\necho done",
        "cargo test &&\necho done",
        "cargo test |\ncat",
        "cargo test focused_case <<'EOF'\ncargo test\nEOF",
    ]
    .into_iter()
    .enumerate()
    {
        let log = root.join(format!("case-{index}.jsonl"));
        write_command_log(&log, &[command]);
        let output = Command::new("python3")
            .args(["bench-audit-agent-validation", log.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "direct auditor accepted broad multiline command `{command}`"
        );
    }
}

#[test]
fn validation_audit_rejects_missing_and_repeated_cargo_checks() {
    let root = test_dir("validation-audit-count");

    let no_validation = root.join("none.jsonl");
    fs::write(&no_validation, "{\"type\":\"item.completed\"}\n").unwrap();
    let none = Command::new("python3")
        .args([
            "bench-audit-agent-validation",
            no_validation.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!none.status.success());
    assert!(String::from_utf8_lossy(&none.stderr).contains("exactly one"));

    let repeated = root.join("repeated.jsonl");
    write_command_log(
        &repeated,
        &[
            "cargo test --test terminal_ui visual_mode",
            "cargo test --test terminal_ui visual_mode",
        ],
    );
    let repeated = Command::new("python3")
        .args(["bench-audit-agent-validation", repeated.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!repeated.status.success());
    assert!(String::from_utf8_lossy(&repeated.stderr).contains("exactly one"));

    let repeated_without_ids = root.join("repeated-without-ids.jsonl");
    let event = serde_json::json!({
        "type": "command_execution",
        "command": "cargo test --test terminal_ui visual_mode",
    })
    .to_string()
        + "\n";
    fs::write(&repeated_without_ids, format!("{event}{event}")).unwrap();
    let repeated_without_ids = Command::new("python3")
        .args([
            "bench-audit-agent-validation",
            repeated_without_ids.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!repeated_without_ids.status.success());
    assert!(String::from_utf8_lossy(&repeated_without_ids.stderr).contains("exactly one"));

    let lifecycle_pair = root.join("started-completed-pair.jsonl");
    let item = serde_json::json!({
        "type": "command_execution",
        "id": "item-7",
        "command": "cargo test --test terminal_ui visual_mode",
    });
    fs::write(
        &lifecycle_pair,
        ["item.started", "item.completed"]
            .into_iter()
            .map(|event_type| {
                serde_json::json!({"type": event_type, "item": item}).to_string() + "\n"
            })
            .collect::<String>(),
    )
    .unwrap();
    let lifecycle_pair = Command::new("python3")
        .args([
            "bench-audit-agent-validation",
            lifecycle_pair.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        lifecycle_pair.status.success(),
        "one command's started/completed lifecycle was counted twice:\n{}",
        String::from_utf8_lossy(&lifecycle_pair.stderr)
    );
}

fn focused_validation_policy_cases() -> Vec<(bool, &'static str)> {
    include_str!("../bench-observer/tests/focused-validation-policy.tsv")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (expected, command) = line.split_once('\t').unwrap();
            (expected == "accept", command)
        })
        .collect()
}

fn nested_shell_command(depth: usize) -> String {
    let mut command = "cargo test nested_case".to_string();
    for _ in 0..depth {
        let payload = command.replace('\\', "\\\\").replace(' ', "\\ ");
        command = format!("sh -c {payload}");
    }
    command
}

#[test]
fn candidate_tools_keep_production_and_historical_admission_separate_from_replay() {
    let candidate = read("bench-candidate-common");
    let materializer = read("materialize-bench-candidate");

    assert!(candidate.contains("bench_stage_candidate_runtime()"));
    assert!(candidate.contains("bench_publish_candidate_runtime()"));
    assert!(candidate.contains("bench_candidate_is_runnable()"));
    assert!(!candidate.contains("bench_exec_candidate_snapshot()"));
    assert!(!candidate.contains("bench_candidate_launch_snapshot()"));
    assert!(materializer.contains("source \"$repo_root/bench-candidate-common\""));
    assert!(materializer.contains("Rebuild at most one verified historical benchmark candidate"));

    let help = Command::new("bash")
        .args(["materialize-bench-candidate", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--inventory-only"));
}

#[test]
fn retained_scripts_have_valid_syntax_and_no_investigation_controls() {
    for path in [
        "bench-three-features",
        "bench-three-features-sequential",
        "bench-three-features-direct-common",
        "bench-candidate-common",
        "bench-validation-common",
        "materialize-bench-candidate",
    ] {
        let status = Command::new("bash").args(["-n", path]).status().unwrap();
        assert!(status.success(), "bash syntax failed for {path}");
    }

    for path in [
        "bench-three-features",
        "bench-three-features-direct-common",
        "bench-observer/README.md",
    ] {
        let text = read(path);
        assert!(!text.contains("WORK_LEAF_BENCH_FEATURE_SCHEDULE"));
        assert!(!text.contains("work-leaf-sequential-protocol"));
        assert!(!text.contains("hidden evaluator"));
        assert!(!text.contains("condition-blind"));
    }
}

#[test]
fn dashboard_fixes_the_accepted_baseline_profile() {
    let dashboard = read("bench-dashboard");
    let work_leaf = read("bench-three-features");
    assert!(dashboard.contains("ACCEPTED_BASELINE_MODEL = \"gpt-5.5\""));
    assert!(dashboard.contains("ACCEPTED_BASELINE_REASONING = \"xhigh\""));
    assert!(dashboard.contains("accepted_baseline_profile"));
    assert!(dashboard.contains("different model or reasoning profile cannot train the baseline"));
    assert!(work_leaf.contains("accepted_baseline_model = \"gpt-5.5\""));
    assert!(work_leaf.contains("accepted_baseline_reasoning = \"xhigh\""));
    assert!(work_leaf.contains("different model or reasoning profile cannot be fitted"));
}

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("cannot read {path}: {error}"))
}

fn work_leaf_feature(script: &str, index: usize) -> &str {
    let marker = format!("    {index})\n      printf '%s\\n' '");
    script
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing Work Leaf feature {index}"))
        .1
        .split_once("'\n")
        .unwrap()
        .0
}

fn direct_feature(script: &str, index: usize) -> &str {
    let marker = format!("feature_{index}='");
    script
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing direct feature {index}"))
        .1
        .split_once("'\n")
        .unwrap()
        .0
}

fn write_command_log(path: &std::path::Path, commands: &[&str]) {
    let text = commands
        .iter()
        .enumerate()
        .map(|(index, command)| {
            serde_json::json!({
                "type": "command_execution",
                "id": format!("command-{index}"),
                "command": command,
            })
            .to_string()
                + "\n"
        })
        .collect::<String>();
    fs::write(path, text).unwrap();
}

fn test_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "work-leaf-benchmark-workflows-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
