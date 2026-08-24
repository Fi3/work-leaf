#![cfg(unix)]

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn inline_fitter_uses_only_explicit_normal_work_leaf_rows() {
    let root = test_dir("workflow-admission");
    let accepted = [
        report(
            "accepted-a.md",
            &normal_work_leaf_metadata("gpt-5.5", "xhigh", "input=100 output=200"),
        ),
        report(
            "accepted-b.md",
            &normal_work_leaf_metadata("gpt-5.5", "xhigh", "input=200 output=300"),
        ),
        report(
            "accepted-legacy-field-names.md",
            &[
                ("result", "pass"),
                ("bench_mode", "work-leaf"),
                ("feature_schedule", "concurrent"),
                ("backend", "codex"),
                ("transport", "app-server"),
                ("agent_model", "gpt-5.5"),
                ("agent_reasoning_effort", "xhigh"),
                ("token_usage", "input=300 output=400"),
            ],
        ),
    ];
    let decoys = [
        report(
            "direct-sequential.md",
            &metadata(
                "sequential",
                "sequential",
                "codex",
                "direct-codex-cli",
                "gpt-5.5",
                "xhigh",
                "input=900 output=100",
            ),
        ),
        report(
            "sequential-work-leaf.md",
            &metadata(
                "work-leaf",
                "sequential",
                "codex",
                "app-server",
                "gpt-5.5",
                "xhigh",
                "input=900 output=200",
            ),
        ),
        report(
            "direct-claude.md",
            &metadata(
                "sequential",
                "sequential",
                "claude",
                "direct-claude-cli",
                "gpt-5.5",
                "xhigh",
                "input=900 output=300",
            ),
        ),
        report(
            "worktree.md",
            &metadata(
                "worktree",
                "sequential",
                "codex",
                "direct-codex-cli",
                "gpt-5.5",
                "xhigh",
                "input=900 output=400",
            ),
        ),
        report(
            "wrong-backend.md",
            &metadata(
                "work-leaf",
                "concurrent",
                "claude",
                "app-server",
                "gpt-5.5",
                "xhigh",
                "input=900 output=500",
            ),
        ),
        report(
            "wrong-transport.md",
            &metadata(
                "work-leaf",
                "concurrent",
                "codex",
                "worktree",
                "gpt-5.5",
                "xhigh",
                "input=900 output=600",
            ),
        ),
        report(
            "other-profile.md",
            &normal_work_leaf_metadata("gpt-5.4", "xhigh", "input=900 output=700"),
        ),
        report(
            "wrong-reasoning.md",
            &normal_work_leaf_metadata("gpt-5.5", "high", "input=900 output=700"),
        ),
        report(
            "missing-model.md",
            &[
                ("result", "pass"),
                ("bench_mode", "work-leaf"),
                ("feature_schedule", "concurrent"),
                ("agent_backend", "codex"),
                ("agent_transport", "app-server"),
                ("agent_reasoning_effort", "xhigh"),
                ("token_usage", "input=900 output=700"),
            ],
        ),
        report(
            "missing-reasoning.md",
            &[
                ("result", "pass"),
                ("bench_mode", "work-leaf"),
                ("feature_schedule", "concurrent"),
                ("agent_backend", "codex"),
                ("agent_transport", "app-server"),
                ("agent_model", "gpt-5.5"),
                ("token_usage", "input=900 output=700"),
            ],
        ),
        report(
            "missing-mode.md",
            &[
                ("result", "pass"),
                ("feature_schedule", "concurrent"),
                ("agent_backend", "codex"),
                ("agent_transport", "app-server"),
                ("agent_model", "gpt-5.5"),
                ("agent_reasoning_effort", "xhigh"),
                ("token_usage", "input=900 output=800"),
            ],
        ),
        report(
            "missing-schedule.md",
            &[
                ("result", "pass"),
                ("bench_mode", "work-leaf"),
                ("agent_backend", "codex"),
                ("agent_transport", "app-server"),
                ("agent_model", "gpt-5.5"),
                ("agent_reasoning_effort", "xhigh"),
                ("token_usage", "input=900 output=800"),
            ],
        ),
        report(
            "missing-backend.md",
            &[
                ("result", "pass"),
                ("bench_mode", "work-leaf"),
                ("feature_schedule", "concurrent"),
                ("agent_transport", "app-server"),
                ("agent_model", "gpt-5.5"),
                ("agent_reasoning_effort", "xhigh"),
                ("token_usage", "input=900 output=800"),
            ],
        ),
        report(
            "missing-transport.md",
            &[
                ("result", "pass"),
                ("bench_mode", "work-leaf"),
                ("feature_schedule", "concurrent"),
                ("agent_backend", "codex"),
                ("agent_model", "gpt-5.5"),
                ("agent_reasoning_effort", "xhigh"),
                ("token_usage", "input=900 output=800"),
            ],
        ),
    ];

    let report_names: Vec<&str> = accepted
        .iter()
        .chain(decoys.iter())
        .map(|(name, _)| *name)
        .collect();
    for (name, contents) in accepted.iter().chain(decoys.iter()) {
        fs::write(root.join(name), contents).unwrap();
    }
    fs::write(
        root.join("baseline-manifest.json"),
        serde_json::to_vec(&serde_json::json!({ "reports": report_names })).unwrap(),
    )
    .unwrap();

    let output = run_inline_fitter(&root);
    assert!(
        output.status.success(),
        "inline fitter failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let fields: HashMap<_, _> = stdout
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .collect();
    assert_eq!(fields.get("token_model_baseline_count"), Some(&"3"));
    assert_eq!(fields.get("token_model_mean"), Some(&"500"));

    fs::remove_dir_all(root).unwrap();
}

fn run_inline_fitter(results_dir: &Path) -> std::process::Output {
    let script = fs::read_to_string("bench-three-features").unwrap();
    let function = script.split_once("compute_token_model_fit() {").unwrap().1;
    let python = function
        .split_once("<<'PY'\n")
        .unwrap()
        .1
        .split_once("\nPY\n")
        .unwrap()
        .0;
    let mut child = Command::new("python3")
        .args([
            "-",
            results_dir.to_str().unwrap(),
            "pass",
            "input=180 output=220",
            "gpt-5.5",
            "xhigh",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(python.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn normal_work_leaf_metadata<'a>(
    model: &'a str,
    reasoning: &'a str,
    tokens: &'a str,
) -> [(&'a str, &'a str); 8] {
    metadata(
        "work-leaf",
        "concurrent",
        "codex",
        "app-server",
        model,
        reasoning,
        tokens,
    )
}

fn metadata<'a>(
    mode: &'a str,
    schedule: &'a str,
    backend: &'a str,
    transport: &'a str,
    model: &'a str,
    reasoning: &'a str,
    tokens: &'a str,
) -> [(&'a str, &'a str); 8] {
    [
        ("result", "pass"),
        ("bench_mode", mode),
        ("feature_schedule", schedule),
        ("agent_backend", backend),
        ("agent_transport", transport),
        ("agent_model", model),
        ("agent_reasoning_effort", reasoning),
        ("token_usage", tokens),
    ]
}

fn report<'a>(name: &'a str, fields: &[(&str, &str)]) -> (&'a str, String) {
    let mut contents = String::from("# Benchmark report\n\n");
    for (key, value) in fields {
        contents.push_str(&format!("- {key}: {value}\n"));
    }
    (name, contents)
}

fn test_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "work-leaf-inline-fitter-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
