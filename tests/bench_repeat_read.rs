// Reuse the committed provider-free read/state fixture unchanged. Each manifest
// executes in a subprocess, so parallel tests never mutate process-global env.
include!("bench_read_inline.rs");

const REPEAT_SCHEMA: &str = "work-leaf-bench-experiment-v4";
const REPEAT_VARIANT: &str = "requested-repeat-full";
const OTHER_VARIANT: &str = "unified-diff-preferred";

fn repeat_rows(trace: &[Value]) -> Vec<&Value> {
    trace
        .iter()
        .filter(|row| row["site"] == "requested-repeat-read")
        .collect()
}

fn validate_repeat_trace(fixture: &Value, trace: &[Value], treatment: bool) {
    let rows = repeat_rows(trace);
    let steps = fixture["steps"].as_array().unwrap();
    assert_eq!(rows.len(), steps.len(), "every requested read is traced");
    for (step, row) in steps.iter().zip(rows) {
        let baseline = row["original_prompt"].as_str().unwrap();
        let candidate = row["candidate_prompt"].as_str().unwrap();
        let selected = if treatment { candidate } else { baseline };
        assert_eq!(step["prompt"], selected);
        let components = row["components"].as_array().unwrap();
        let eligible = row["metadata"]["eligible"].as_bool().unwrap();
        assert_eq!(components.len(), usize::from(eligible));
        if let Some(component) = components.first() {
            let start = component["baseline_start"].as_u64().unwrap() as usize;
            let end = component["baseline_end"].as_u64().unwrap() as usize;
            let candidate_start = component["inline_start"].as_u64().unwrap() as usize;
            let candidate_end = component["inline_end"].as_u64().unwrap() as usize;
            assert_eq!(&baseline[..start], &candidate[..candidate_start]);
            assert_eq!(&baseline[end..], &candidate[candidate_end..]);
            assert!(
                candidate[candidate_start..candidate_end]
                    .starts_with("\nRepeated file reads: current full text\n")
            );
            if !row["metadata"]["failures"].as_array().unwrap().is_empty() {
                assert!(baseline[end..].starts_with("\nUnavailable file text\n"));
            }
        } else {
            assert_eq!(baseline, candidate);
        }
        for snapshot in row["metadata"]["snapshots"].as_array().unwrap() {
            let class = snapshot["class"].as_str().unwrap();
            if matches!(class, "changed" | "unchanged") {
                let start = snapshot["inline_body_start"].as_u64().unwrap() as usize;
                let end = snapshot["inline_body_end"].as_u64().unwrap() as usize;
                let path = snapshot["path"].as_str().unwrap();
                assert_eq!(
                    &candidate[start..end],
                    step["sources"][path].as_str().unwrap()
                );
                assert_eq!(snapshot["bytes"], end - start);
                assert!(snapshot["digest"].as_str().unwrap().starts_with("fnv64:"));
            } else {
                assert!(snapshot["inline_body_start"].is_null());
                assert!(snapshot["inline_body_end"].is_null());
            }
        }
    }
}

#[test]
fn repeat_full_changes_only_tracked_components_and_keeps_bundle_allocations() {
    let (baseline, _) = exercise(None, "unused", "matrix");
    let (identity, identity_trace) = exercise(Some(REPEAT_SCHEMA), OTHER_VARIANT, "matrix");
    let (variant, trace) = exercise(Some(REPEAT_SCHEMA), REPEAT_VARIANT, "matrix");
    assert_eq!(baseline["bundles"], identity["bundles"]);
    assert_eq!(baseline["bundles"], variant["bundles"]);
    assert_eq!(normalized(&baseline), normalized(&identity));
    if cfg!(feature = "bench-experiments") {
        validate_repeat_trace(&identity, &identity_trace, false);
        validate_repeat_trace(&variant, &trace, true);
        for (index, (normal, varied)) in normalized(&baseline)
            .iter()
            .zip(normalized(&variant))
            .enumerate()
        {
            assert_eq!(normal != &varied, [7, 8].contains(&index), "step {index}");
        }
        let rows = repeat_rows(&trace);
        assert_eq!(
            rows[7]["metadata"]["requested_paths"]
                .as_array()
                .unwrap()
                .len(),
            6
        );
        assert_eq!(rows[7]["metadata"]["failures"][0]["path"], "absent");
        assert!(
            rows[7]["metadata"]["snapshots"]
                .as_array()
                .unwrap()
                .iter()
                .any(|snapshot| snapshot["class"] == "explicit-bundle")
        );
        assert_eq!(rows[8]["metadata"]["bundle"]["write_succeeded"], false);
        assert!(
            rows[8]["metadata"]["snapshots"]
                .as_array()
                .unwrap()
                .iter()
                .all(|snapshot| snapshot["class"] == "unchanged")
        );
    } else {
        assert!(trace.is_empty());
        assert!(identity_trace.is_empty());
        assert_eq!(normalized(&baseline), normalized(&variant));
    }
}

#[test]
fn repeat_full_failed_changed_delivery_retains_old_snapshot_and_done_clears_it() {
    for condition in [OTHER_VARIANT, REPEAT_VARIANT] {
        let root = root();
        let configured = child(&root, Some(REPEAT_SCHEMA), condition, "repeat-state");
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args(["--exact", "repeat_state_child", "--ignored", "--nocapture"]);
        for (key, value) in configured.get_envs() {
            match value {
                Some(value) => command.env(key, value),
                None => command.env_remove(key),
            };
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let mut fixture: Value =
            serde_json::from_slice(&fs::read(root.join("result.json")).unwrap()).unwrap();
        fixture["root"] = json!(root);
        let trace: Vec<Value> = fs::read_to_string(root.join("evidence.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        if cfg!(feature = "bench-experiments") {
            validate_repeat_trace(&fixture, &trace, condition == REPEAT_VARIANT);
            let rows = repeat_rows(&trace);
            let classes: Vec<_> = rows
                .iter()
                .map(|row| row["metadata"]["snapshots"][0]["class"].as_str().unwrap())
                .collect();
            assert_eq!(
                classes,
                ["untracked", "changed", "changed", "unchanged", "untracked"]
            );
            assert_eq!(
                rows[1]["metadata"]["snapshots"],
                rows[2]["metadata"]["snapshots"]
            );
        } else {
            assert!(trace.is_empty());
        }
    }
}

#[test]
#[ignore = "isolated provider-free subprocess fixture"]
fn repeat_state_child() {
    let root = PathBuf::from(std::env::var_os("READ_INLINE_TEST_ROOT").unwrap());
    let path = "context-λ.txt";
    fs::write(root.join(path), "old\n").unwrap();
    let backend = Recording::default();
    let mut orchestrator = AgentOrchestrator::new(root.clone(), backend.clone());
    let id = AgentId::new("repeat-owner").unwrap();
    let mut steps = Vec::new();
    for index in 0..5 {
        if index == 1 {
            fs::write(
                root.join(path),
                "λ\nUnavailable file text\nRepeated file reads unchanged\n--- context-λ.txt ---",
            )
            .unwrap();
        }
        if index == 4 {
            let events = orchestrator
                .handle_agent_message(&id, "repeat fixture", "@work-leaf done")
                .unwrap();
            assert!(
                events
                    .iter()
                    .any(|event| matches!(event, work_leaf::OrchestratorEvent::AgentDone { .. }))
            );
        }
        backend.1.store(index == 1, Ordering::Relaxed);
        let result = orchestrator.handle_agent_message(
            &id,
            "repeat fixture",
            "@work-leaf read context-λ.txt",
        );
        assert_eq!(result.is_err(), index == 1);
        let current = fs::read_to_string(root.join(path)).unwrap();
        steps.push(json!({"prompt": backend.0.lock().unwrap().last().unwrap(), "sources": {path: current}, "failed": result.is_err()}));
    }
    fs::write(
        root.join("result.json"),
        serde_json::to_vec(&json!({"steps": steps, "bundle_dir": ""})).unwrap(),
    )
    .unwrap();
}

#[test]
fn repeat_full_retains_ordinary_fallback_send_failure_and_owned_edit_tracking() {
    for scenario in ["bundle-failure", "send-failure", "state"] {
        let (baseline, _) = exercise(None, "unused", scenario);
        let (variant, trace) = exercise(Some(REPEAT_SCHEMA), REPEAT_VARIANT, scenario);
        assert_eq!(baseline["bundles"], variant["bundles"], "{scenario}");
        assert_eq!(baseline["refresh"], variant["refresh"]);
        assert_eq!(baseline["final_text"], variant["final_text"]);
        if cfg!(feature = "bench-experiments") {
            validate_repeat_trace(&variant, &trace, true);
            let rows = repeat_rows(&trace);
            for (index, (normal, varied)) in normalized(&baseline)
                .iter()
                .zip(normalized(&variant))
                .enumerate()
            {
                let eligible = scenario == "state" && index == 2;
                assert_eq!(rows[index]["metadata"]["eligible"], eligible);
                assert_eq!(normal != &varied, eligible);
            }
            if scenario == "state" {
                for row in trace.iter().filter(|row| row["event"] == "prompt") {
                    assert_eq!(
                        row["changed"], false,
                        "ordinary ACK and command stay normal"
                    );
                }
            }
        } else {
            assert_eq!(normalized(&baseline), normalized(&variant));
        }
    }
}
