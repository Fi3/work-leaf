#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

// Frozen from the three `post_command 'new ...'` tasks in e70c933:bench-three-features.
const ORIGIN_MASTER_FEATURE_HASH: &str =
    "45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a";
const ORIGIN_MASTER_FEATURES: [&str; 3] = [
    "add vim like visual mode for both panes when I do v I can select the text in focused the panes same keystrokes of vim y Y for copy maiusc V line select block select with ctrl v block select",
    "when an user prompt start with / and is followed by something without whitespace that is a command for the agent; the orchestrator must send it to the selected backend agent and show that backend response",
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

    for script in [&work_leaf, &direct] {
        assert!(!script.contains("/fork"));
        assert!(!script.contains("strict selected-agent"));
        assert!(!script.contains("ordinary prompt path"));
    }
}

#[test]
fn paired_product_benchmarks_preserve_normal_validation_and_use_a_check_only_final_gate() {
    let work_leaf = read("bench-three-features");
    let direct = read("bench-three-features-direct-common");
    let validation = read("bench-validation-common");
    let observer_main = read("bench-observer/src/main.rs");
    let observer_lib = read("bench-observer/src/lib.rs");

    for script in [&work_leaf, &direct] {
        assert!(!script.contains("Run exactly one focused Cargo validation command"));
        assert!(!script.contains("Starting no Cargo validation, or starting more than one"));
        assert!(!script.contains("validation-budget-violation.txt"));
        assert!(!script.contains("disable_iteration_validation_budget"));
        assert!(!script.contains("bench-audit-agent-validation"));
        assert!(script.contains("WORK_LEAF_BENCH_OBSERVER_BIN"));
        assert!(script.contains("source \"$repo_root/bench-candidate-common\""));
        assert!(script.contains("source \"$repo_root/bench-validation-common\""));
        assert_eq!(script.matches("bench_run_final_gate").count(), 1);
        assert!(!script.contains("Do not run Cargo validation"));
        assert!(!script.contains("benchmark driver owns the one final"));
    }

    assert_eq!(
        validation
            .matches("cargo fmt -- --check || exit $?\n")
            .count(),
        1
    );
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
    assert!(direct.contains("Run focused checks while implementing"));
    assert!(direct.contains("Run the checks required by AGENTS.md and iterate until they pass"));
    assert!(work_leaf.contains(
        "Run the checks required by the repository instructions and iterate until they pass"
    ));

    assert!(!observer_main.contains("run_cargo_proxy"));
    assert!(!observer_main.contains("validation-budget"));
    assert!(!observer_lib.contains("for name in [\"codex\", \"sh\", \"cargo\"]"));
    assert!(!observer_lib.contains("set_validation_budget(&config, true)"));
    assert!(!observer_lib.contains("load_online_validation_budget_violations(config"));
}

#[test]
fn paired_product_benchmarks_pin_the_same_profile_without_editing_codex_config() {
    let work_leaf = read("bench-three-features");
    let direct = read("bench-three-features-direct-common");
    let profile = read("bench-agent-profile-common");

    for script in [&work_leaf, &direct] {
        assert!(script.contains("source \"$repo_root/bench-agent-profile-common\""));
        assert!(script.contains("bench_prepare_codex_profile"));
        assert!(script.contains("bench_profiled_codex_bin"));
        assert!(script.contains("WORK_LEAF_BENCH_REASONING_EFFORT"));
        assert!(!script.contains("model_reasoning_effort=\"$agent_reasoning_effort\""));
        assert!(!script.contains("sed -i"));
        assert!(script.contains("bench_install_no_recursive_agent_policy"));
        assert!(script.contains("bench_restore_no_recursive_agent_policy"));
        assert!(script.contains("bench_assert_no_recursive_codex"));
    }
    assert!(profile.contains("model=\\\"$model\\\""));
    assert!(profile.contains("model_reasoning_effort=\\\"$reasoning_effort\\\""));
    assert!(profile.contains("WORK_LEAF_BENCH_CODEX_ACTIVE"));
    assert!(!profile.contains("config.toml"));
    assert!(!direct.contains(
        "The temporary benchmark provider-isolation policy waives recursive real-agent verification"
    ));
}

#[test]
fn profiled_codex_wrapper_injects_model_and_xhigh_and_preserves_real_cli_arguments() {
    let root = test_dir("profiled-codex");
    let fake = root.join("codex-real");
    let argument_log = root.join("args.txt");
    fs::write(
        &fake,
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" >> \"$BENCH_PROFILE_TEST_LOG\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    permissions.set_mode(0o700);
    fs::set_permissions(&fake, permissions).unwrap();

    let command = format!(
        "source ./bench-agent-profile-common; bench_prepare_codex_profile {} gpt-5.5 xhigh {}; : > {}; BENCH_PROFILE_TEST_LOG={} \"$bench_profiled_codex_bin\" --model gpt-5.5 exec --json -",
        shell_path(&fake),
        shell_path(&root.join("profile")),
        shell_path(&argument_log),
        shell_path(&argument_log),
    );
    let output = Command::new("bash")
        .args(["-c", &command])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "profile helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(argument_log).unwrap(),
        "-c\nmodel=\"gpt-5.5\"\n-c\nmodel_reasoning_effort=\"xhigh\"\n--model\ngpt-5.5\nexec\n--json\n-\n"
    );
}

#[test]
fn profiled_codex_wrapper_blocks_and_records_recursive_provider_launches() {
    let root = test_dir("profiled-codex-recursion");
    let fake = root.join("codex-real");
    let argument_log = root.join("args.txt");
    let status_log = root.join("status.txt");
    fs::write(
        &fake,
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" >> \"$BENCH_PROFILE_TEST_LOG\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    permissions.set_mode(0o700);
    fs::set_permissions(&fake, permissions).unwrap();

    let command = format!(
        "source ./bench-agent-profile-common; bench_prepare_codex_profile {} gpt-5.5 xhigh {}; : > {}; set +e; WORK_LEAF_BENCH_CODEX_ACTIVE=1 BENCH_PROFILE_TEST_LOG={} \"$bench_profiled_codex_bin\" exec --json -; printf '%s\\n' $? > {}",
        shell_path(&fake),
        shell_path(&root.join("profile")),
        shell_path(&argument_log),
        shell_path(&argument_log),
        shell_path(&status_log),
    );
    let output = Command::new("bash")
        .args(["-c", &command])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(argument_log).unwrap(), "");
    assert_eq!(fs::read_to_string(status_log).unwrap(), "86\n");
    assert!(
        fs::read_to_string(root.join("profile/recursive-codex-attempts.log"))
            .unwrap()
            .contains("blocked recursive Codex launch")
    );
}

#[test]
fn temporary_benchmark_policy_waives_only_recursive_real_agent_verification() {
    let root = test_dir("benchmark-agent-policy");
    let repo = root.join("repo");
    let policy = root.join("policy");
    fs::create_dir_all(&repo).unwrap();
    fs::write(
        repo.join("AGENTS.md"),
        "# Original instructions\nRun all checks.\n",
    )
    .unwrap();
    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "bench@example.com"]);
    run_git(&repo, &["config", "user.name", "Bench Test"]);
    run_git(&repo, &["add", "AGENTS.md"]);
    run_git(
        &repo,
        &["commit", "-m", "ADD fixture instructions for testing"],
    );

    let command = format!(
        "source ./bench-agent-profile-common; bench_install_no_recursive_agent_policy {} {}; git -C {} status --porcelain; bench_restore_no_recursive_agent_policy {} {}",
        shell_path(&repo),
        shell_path(&policy),
        shell_path(&repo),
        shell_path(&repo),
        shell_path(&policy),
    );
    let output = Command::new("bash")
        .args(["-c", &command])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "policy helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(
        fs::read_to_string(repo.join("AGENTS.md")).unwrap(),
        "# Original instructions\nRun all checks.\n"
    );
    let policy_text = fs::read_to_string(policy.join("effective-policy.txt")).unwrap();
    assert!(policy_text.contains("Do not launch another Codex or agent provider"));
    assert!(policy_text.contains("All repository tests and validation checks remain required"));
}

#[test]
fn benchmark_policy_restore_accepts_a_checkout_already_reset_to_the_fixed_base() {
    let root = test_dir("benchmark-agent-policy-reset");
    let repo = root.join("repo");
    let policy = root.join("policy");
    fs::create_dir_all(&repo).unwrap();
    fs::write(
        repo.join("AGENTS.md"),
        "# Original instructions\nRun all checks.\n",
    )
    .unwrap();
    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "bench@example.com"]);
    run_git(&repo, &["config", "user.name", "Bench Test"]);
    run_git(&repo, &["add", "AGENTS.md"]);
    run_git(
        &repo,
        &["commit", "-m", "ADD fixture instructions for testing"],
    );

    let command = format!(
        "source ./bench-agent-profile-common; bench_install_no_recursive_agent_policy {} {}; cp {}/original-AGENTS.md {}/AGENTS.md; bench_restore_no_recursive_agent_policy {} {}",
        shell_path(&repo),
        shell_path(&policy),
        shell_path(&policy),
        shell_path(&repo),
        shell_path(&repo),
        shell_path(&policy),
    );
    let output = Command::new("bash")
        .args(["-c", &command])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "policy helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(repo.join("AGENTS.md")).unwrap(),
        "# Original instructions\nRun all checks.\n"
    );
}

#[test]
fn paired_product_benchmarks_give_each_stage_a_full_timeout() {
    let work_leaf = read("bench-three-features");
    let direct = read("bench-three-features-direct-common");

    assert!(work_leaf.contains("feature_stage_deadline"));
    assert!(work_leaf.contains("linearize_stage_deadline"));
    assert!(!work_leaf.contains("elapsed <= timeout_secs"));
    assert!(direct.contains("begin_stage_deadline"));
    assert!(direct.contains("begin_stage_deadline \"feature-$feature_index\""));
    assert!(direct.contains("begin_stage_deadline \"linearize\""));
    assert!(!direct.contains("start_active_seconds + timeout_secs"));
}

#[test]
fn paired_product_benchmarks_isolate_provider_tmp_and_treat_stream_growth_as_progress() {
    let work_leaf = read("bench-three-features");
    let direct = read("bench-three-features-direct-common");
    let progress = read("bench-progress-common");

    assert!(work_leaf.contains("source \"$repo_root/bench-progress-common\""));
    assert!(
        work_leaf.contains("TMPDIR=\"$child_tmp_dir\" exec env WORK_LEAF_OBSERVER_PRIMARY_MARKER=")
    );
    assert_eq!(
        direct
            .matches("env TMPDIR=\"$child_tmp_dir\" WORK_LEAF_OBSERVER_ROLE=\"$label\"")
            .count(),
        4,
        "initial/resumed direct calls, with and without timeout, must use isolated provider temp state"
    );
    assert!(work_leaf.contains("bench_provider_stream_signature \"$observer_root\""));

    let root = test_dir("provider-stream-progress");
    let capture = root.join("app-server/invocation/server-to-client.raw");
    fs::create_dir_all(capture.parent().unwrap()).unwrap();
    fs::write(&capture, b"first").unwrap();
    let command = format!(
        "source ./bench-progress-common; first=$(bench_provider_stream_signature {}); printf second >> {}; second=$(bench_provider_stream_signature {}); test \"$first\" != \"$second\"",
        shell_path(&root),
        shell_path(&capture),
        shell_path(&root),
    );
    let output = Command::new("bash")
        .args(["-c", &command])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "provider stream growth was not detected: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(progress.contains("server-to-client.raw"));
}

#[test]
fn observer_measurement_cannot_change_the_workflow_result() {
    for path in ["bench-three-features", "bench-three-features-direct-common"] {
        let script = read(path);
        assert!(script.contains("measurement_status="));
        assert!(script.contains("workflow_result"));
        assert!(script.contains("usage_scopes.total_workflow"));
        assert!(!script.contains("finalize_observation || fail_bench"));
        assert!(!script.contains("observer analysis rejected this capture"));
    }
}

#[test]
fn paired_product_benchmarks_preserve_uncommitted_and_untracked_candidate_changes() {
    for path in ["bench-three-features", "bench-three-features-direct-common"] {
        let script = read(path);
        assert!(script.contains("snapshot.index"));
        assert!(script.contains("git -C \"$checkout_dir\" add -N -- ."));
        assert!(script.contains("git -C \"$checkout_dir\" diff --binary"));
        assert!(script.contains("git -C \"$checkout_dir\" diff --cached --binary"));
    }
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
fn dashboard_owns_historical_baselines_while_launchers_report_raw_measurements() {
    let dashboard = read("bench-dashboard");
    let work_leaf = read("bench-three-features");
    assert!(dashboard.contains("ACCEPTED_BASELINE_MODEL = \"gpt-5.5\""));
    assert!(dashboard.contains("ACCEPTED_BASELINE_REASONING = \"xhigh\""));
    assert!(dashboard.contains("accepted_baseline_profile"));
    assert!(dashboard.contains("different model or reasoning profile cannot train the baseline"));
    assert!(!work_leaf.contains("accepted_baseline_model"));
    assert!(!work_leaf.contains("token_model_status"));
    assert!(work_leaf.contains("total_workflow_raw_tokens"));
}

#[test]
fn fair_normal_workflow_pilot_is_one_shot_paired_and_stops_after_scoring() {
    let study = "bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z";
    let launcher = read(&format!("{study}/run-pilot"));
    let scorer = read(&format!("{study}/scorer/score.py"));

    assert_eq!(
        launcher
            .matches("\"$repo_root/bench-three-features\"")
            .count(),
        1
    );
    assert_eq!(
        launcher
            .matches("\"$repo_root/bench-three-features-sequential\"")
            .count(),
        1
    );
    assert!(launcher.contains("WORK_LEAF_BENCH_MODEL=gpt-5.5"));
    assert!(launcher.contains("WORK_LEAF_BENCH_REASONING_EFFORT=xhigh"));
    assert!(launcher.contains("WORK_LEAF_DIRECT_BENCH_MODEL=gpt-5.5"));
    assert!(launcher.contains("WORK_LEAF_DIRECT_BENCH_REASONING_EFFORT=xhigh"));
    assert!(launcher.contains("WORK_LEAF_BENCH_NO_READ_PERMISSION=0"));
    assert!(launcher.contains("work_leaf_pid=$!"));
    assert!(launcher.contains("direct_pid=$!"));
    assert!(launcher.contains("wait \"$work_leaf_pid\""));
    assert!(launcher.contains("wait \"$direct_pid\""));
    assert!(!launcher.contains("WORK_LEAF_BENCH_RETRY"));
    assert!(!launcher.contains("wl-000"));
    assert!(!launcher.contains("wl-111"));
    assert!(launcher.contains("scorer/score.py"));
    assert!(launcher.contains("The study stops after this provisional pilot"));

    assert!(!scorer.contains("/fork"));
    assert!(scorer.contains("quality_match_in_this_pair"));
    assert!(scorer.contains("One pilot pair is descriptive"));
}

#[test]
fn fair_normal_workflow_pilot_rerun_keeps_the_repaired_one_pair_contract() {
    let study = "bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z";
    let launcher = read(&format!("{study}/run-pilot"));
    let scorer = read(&format!("{study}/scorer/score.py"));

    assert_eq!(
        launcher
            .matches("\"$repo_root/bench-three-features\"")
            .count(),
        1
    );
    assert_eq!(
        launcher
            .matches("\"$repo_root/bench-three-features-sequential\"")
            .count(),
        1
    );
    assert!(launcher.contains("WORK_LEAF_BENCH_MODEL=gpt-5.5"));
    assert!(launcher.contains("WORK_LEAF_BENCH_REASONING_EFFORT=xhigh"));
    assert!(launcher.contains("WORK_LEAF_DIRECT_BENCH_MODEL=gpt-5.5"));
    assert!(launcher.contains("WORK_LEAF_DIRECT_BENCH_REASONING_EFFORT=xhigh"));
    assert!(launcher.contains("WORK_LEAF_BENCH_NO_READ_PERMISSION=0"));
    assert!(launcher.contains("provider_workflows_admitted=1"));
    assert!(launcher.contains("provider_workflows_admitted=2"));
    assert!(launcher.contains("work_leaf_pid=$!"));
    assert!(launcher.contains("direct_pid=$!"));
    assert!(launcher.contains("wait \"$work_leaf_pid\""));
    assert!(launcher.contains("wait \"$direct_pid\""));
    assert!(!launcher.contains("WORK_LEAF_BENCH_RETRY"));
    assert!(!launcher.contains("wl-000"));
    assert!(!launcher.contains("wl-111"));
    assert!(launcher.contains("scorer/score.py"));
    assert!(launcher.contains("Steps 8 and 9 require user review"));

    assert!(!scorer.contains("/fork"));
    assert!(scorer.contains("quality_match_in_this_pair"));
    assert!(scorer.contains("One pilot pair is descriptive"));
}

#[test]
fn work_leaf_benchmark_preserves_immediate_directive_interruption() {
    let work_leaf = read("bench-three-features");
    let direct = read("bench-three-features-direct-common");

    assert!(!work_leaf.contains("WORK_LEAF_CODEX_EXACT_USAGE"));
    assert!(!direct.contains("WORK_LEAF_CODEX_EXACT_USAGE"));
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

fn shell_path(path: &std::path::Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

fn run_git(cwd: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
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
