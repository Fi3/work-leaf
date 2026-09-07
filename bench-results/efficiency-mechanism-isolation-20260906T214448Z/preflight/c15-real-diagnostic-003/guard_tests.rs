use std::collections::BTreeMap;

use c15_real_diagnostic_guards::{Budget, fixture_request, settled_turns, validate_settings};
use serde_json::{Value, json};

fn settings() -> (Value, Value, BTreeMap<String, String>) {
    let config = json!({"condition":"work-leaf", "run_id":"private-test-first-diagnostic-001",
        "model":"gpt-5.5", "effort":"xhigh", "primary_invocation_marker":"owned-primary"});
    let manifest = json!({"schema":"work-leaf-bench-experiment-v6", "condition":"private-test-first",
        "run_id":"private-test-first-diagnostic-001", "private_preview":{"project_root":"/owned/project"}});
    let variables = [
        ("WORK_LEAF_REAL_PRIVATE_TEST_FIRST_SMOKE", "1"),
        ("WORK_LEAF_BENCH_EXPERIMENT", "1"),
        (
            "WORK_LEAF_BENCH_RUN_ID",
            "private-test-first-diagnostic-001",
        ),
        ("WORK_LEAF_REAL_OBSERVER_PROJECT_DIR", "/owned/project"),
        ("WORK_LEAF_OBSERVER_PRIMARY_MARKER", "owned-primary"),
        ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "1"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "1000"),
        (
            "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
            "forward",
        ),
        ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "1"),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_owned(), b.to_owned()))
    .collect();
    (config, manifest, variables)
}

#[test]
fn natural_request_supplies_initial_source_without_a_test_or_measurement_recipe() {
    let request = fixture_request();
    assert!(request.contains("pub fn capped(value: u8, _limit: u8) -> u8 { value }"));
    assert!(request.contains("Write your own regression tests"));
    assert!(request.contains("cargo test --offline --locked"));
    for forbidden in [
        "assert_eq!",
        "#[test]",
        "declared_test_units",
        "byte offset",
        "Reviewer turn",
        "*** Begin Patch",
    ] {
        assert!(!request.contains(forbidden), "{forbidden}");
    }
}

fn settled_fixture() -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    let client = vec![
        json!({"id":"start1","method":"turn/start","params":{"threadId":"author"}}),
        json!({"id":"start2","method":"turn/start","params":{"threadId":"reviewer"}}),
        json!({"id":"stop","method":"turn/interrupt","params":{"threadId":"reviewer","turnId":"turn2"}}),
    ];
    let server = vec![
        json!({"id":"start1","result":{"turn":{"id":"turn1"}}}),
        json!({"id":"start2","result":{"turn":{"id":"turn2"}}}),
        json!({"id":"stop","result":{}}),
        json!({"method":"turn/completed","params":{"threadId":"author","turn":{"id":"turn1","status":"completed"}}}),
        json!({"method":"turn/completed","params":{"threadId":"reviewer","turn":{"id":"turn2","status":"interrupted"}}}),
    ];
    (client.clone(), client, server)
}

#[test]
fn terminal_cut_requires_exact_forwarded_requests_and_each_latest_turn_closed() {
    let (client, forwarded, server) = settled_fixture();
    assert_eq!(settled_turns(&client, &forwarded, &server).unwrap(), 2);
    for absent in 0..server.len() {
        let mut bad = server.clone();
        bad.remove(absent);
        assert!(settled_turns(&client, &forwarded, &bad).is_err());
    }
    let mut bad = forwarded.clone();
    bad[2]["params"]["turnId"] = json!("different");
    assert!(settled_turns(&client, &bad, &server).is_err());
    let mut bad = server.clone();
    bad[4]["params"]["turn"]["status"] = json!("inProgress");
    assert!(settled_turns(&client, &forwarded, &bad).is_err());
}

#[test]
fn unaccepted_duplicate_or_third_thread_is_not_a_settled_workflow() {
    assert!(settled_turns(&[], &[], &[]).is_err());
    let (client, forwarded, server) = settled_fixture();
    let mut bad = server.clone();
    bad.push(server[0].clone());
    assert!(settled_turns(&client, &forwarded, &bad).is_err());
    let mut bad = client.clone();
    bad.push(client[0].clone());
    assert!(settled_turns(&bad, &bad, &server).is_err());
    let mut third = client.clone();
    third.push(json!({"id":"start3","method":"turn/start","params":{"threadId":"third"}}));
    let mut bad = server.clone();
    bad.push(json!({"id":"start3","result":{"turn":{"id":"turn3"}}}));
    bad.push(json!({"method":"turn/completed","params":{"threadId":"third","turn":{"id":"turn3","status":"completed"}}}));
    assert!(settled_turns(&third, &third, &bad).is_err());
}

#[test]
fn actual_controller_phase_owns_roles_without_name_inference() {
    let mut budget = Budget::new("custom:author", 8).unwrap();
    assert!(budget.admit("some-reviewer", true).is_err());
    budget.admit("custom:author", true).unwrap();
    budget.admit("custom:author", false).unwrap();
    assert!(budget.admit("some-reviewer", true).is_err());
    budget.begin_review().unwrap();
    budget.admit("not-a-review-prefix", true).unwrap();
    budget.admit("not-a-review-prefix", false).unwrap();
    assert_eq!(budget.calls(), 4);
    assert_eq!(budget.real_roles(), 2);
    assert!(budget.admit("third-role", true).is_err());
    budget.admit("custom:author", false).unwrap();
    assert_eq!(budget.calls(), 5);
}

#[test]
fn review_findings_keep_the_ordinary_known_author_fix_route() {
    let mut budget = Budget::new("author", 8).unwrap();
    budget.admit("author", true).unwrap();
    budget.begin_review().unwrap();
    budget.admit("reviewer", true).unwrap();
    budget.admit("author", false).unwrap();
    budget.admit("reviewer", false).unwrap();
    assert!(budget.admit("author", true).is_err());
    assert_eq!(budget.calls(), 4);
}

#[test]
fn duplicates_unlaunched_sends_and_early_review_do_not_spend_slots() {
    let mut budget = Budget::new("author", 8).unwrap();
    assert!(budget.begin_review().is_err());
    assert!(budget.admit("author", false).is_err());
    assert_eq!(budget.calls(), 0);
    budget.admit("author", true).unwrap();
    assert!(budget.admit("author", true).is_err());
    budget.begin_review().unwrap();
    assert!(budget.begin_review().is_err());
    assert!(budget.admit("reviewer", false).is_err());
    assert_eq!(budget.calls(), 1);
}

#[test]
fn hard_eight_call_ceiling_is_not_filled_automatically() {
    assert!(Budget::new("", 8).is_err());
    assert!(Budget::new("author", 0).is_err());
    assert!(Budget::new("author", 9).is_err());
    let mut budget = Budget::new("author", 8).unwrap();
    budget.admit("author", true).unwrap();
    for _ in 1..8 {
        budget.admit("author", false).unwrap();
    }
    assert!(budget.admit("author", false).is_err());
    assert_eq!(budget.calls(), 8);
}

#[test]
fn exact_subscription_capture_and_private_activation_are_required() {
    let (config, manifest, variables) = settings();
    assert!(validate_settings(&config, &manifest, |k| variables.get(k).cloned()).is_ok());
    for omitted in variables.keys() {
        assert!(
            validate_settings(&config, &manifest, |k| if k == omitted {
                None
            } else {
                variables.get(k).cloned()
            })
            .is_err(),
            "{omitted}"
        );
    }
    for forbidden in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "OPENAI_BASE_URL",
        "OPENAI_API_BASE",
        "CODEX_BASE_URL",
        "CODEX_ACCESS_TOKEN",
        "WORK_LEAF_CODEX_TRACE",
        "WORK_LEAF_OBSERVER_PARENT_INVOCATION",
    ] {
        assert!(
            validate_settings(&config, &manifest, |k| if k == forbidden {
                Some(String::new())
            } else {
                variables.get(k).cloned()
            })
            .is_err(),
            "{forbidden}"
        );
    }
}

#[test]
fn wrong_factor_project_model_or_capture_identity_is_rejected() {
    let (config, manifest, variables) = settings();
    for key in [
        "condition",
        "run_id",
        "model",
        "effort",
        "primary_invocation_marker",
    ] {
        let mut changed = config.clone();
        changed[key] = json!("different");
        assert!(
            validate_settings(&changed, &manifest, |k| variables.get(k).cloned()).is_err(),
            "{key}"
        );
    }
    for key in ["schema", "condition", "run_id"] {
        let mut changed = manifest.clone();
        changed[key] = json!("different");
        assert!(
            validate_settings(&config, &changed, |k| variables.get(k).cloned()).is_err(),
            "{key}"
        );
    }
    let mut changed = manifest;
    changed["private_preview"]["project_root"] = json!("/another/project");
    assert!(validate_settings(&config, &changed, |k| variables.get(k).cloned()).is_err());
}
