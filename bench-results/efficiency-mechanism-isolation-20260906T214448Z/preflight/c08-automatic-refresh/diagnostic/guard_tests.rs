// Source-first contract. guards.rs is intentionally absent until the first
// observed RED can run after the current C15 execution hold.
use c08_automatic_refresh_diagnostic::{INITIAL_SOURCE, initial_read_owned, recovery_chain};
use serde_json::{Value, json};

fn read() -> String {
    format!("work-leaf file text\n\n--- src/lib.rs ---\n{INITIAL_SOURCE}")
}

#[test]
fn owned_whole_read_not_a_copied_marker_selects_the_single_stimulus() {
    assert!(initial_read_owned("@work-leaf read src/lib.rs", &read()));
    assert!(initial_read_owned(
        "I will inspect the source.\n@work-leaf read src/lib.rs",
        &read()
    ));
    for reply in [
        "```\n@work-leaf read src/lib.rs\n```",
        "    @work-leaf read src/lib.rs",
        "@work-leaf read other.rs",
        "@work-leaf read src/lib.rs\n@work-leaf done",
        "@work-leaf read src/lib.rs\n@work-leaf read src/lib.rs",
        "A copied @work-leaf read src/lib.rs marker.",
    ] {
        assert!(!initial_read_owned(reply, &read()), "{reply}");
    }
    assert!(!initial_read_owned(
        "@work-leaf read src/lib.rs",
        &(read() + "extra")
    ));
}

fn chain() -> (Vec<Value>, Vec<Value>, String) {
    let current = "pub fn capped(value: u8, _limit: u8) -> u8 { value + 0 }\n";
    let before = "unchanged diagnostic\ndiff body\nunchanged suffix";
    let candidate = format!("unchanged diagnostic\ncurrent full text:\n{current}unchanged suffix");
    let body_start = "unchanged diagnostic\ncurrent full text:\n".len();
    let after_component = body_start + current.len();
    let ack = "work-leaf patch applied\nfiles: src/lib.rs\n";
    let check = "work-leaf command result\ncommand: cargo test --offline --locked\nstatus: 0\nlocked paths: .\nstdout:\nok\nstderr:\n";
    let input = |n, text: &str, reply: &str| json!({"call":n,"agent_id":"author","launch":n==1,"prompt":text,"reply":reply,"returned":true});
    let calls = vec![
        input(1, "policy", "@work-leaf read src/lib.rs"),
        input(2, &read(), "actual stale request"),
        input(3, &candidate, "actual repair request"),
        input(4, ack, "actual check request"),
        input(5, check, "@work-leaf done"),
    ];
    let trace = vec![
        json!({"event":"activation","schema":"work-leaf-bench-experiment-v7","condition":"automatic-changed-refresh-full"}),
        json!({"event":"automatic-refresh","agent_id":"author","eligible":true,"selected_candidate":"candidate",
            "original_prompt":before,"candidate_prompt":candidate,
            "components":[{"id":"current-full-text","baseline_start":21,"baseline_end":31,
                "candidate_start":21,"candidate_end":after_component,"body_start":body_start,"body_end":after_component,"snapshot_index":0}],
            "metadata":{"kind":"edit","diagnostic":"old block not found","files":["src/lib.rs"],
                "snapshots":[{"path":"src/lib.rs","class":"changed","diff_disposition":"available"}]}}),
        json!({"event":"prompt","site":"patch-applied","agent_id":"author","original_prompt":ack,"forwarded_prompt":ack}),
        json!({"event":"prompt","site":"command-result","agent_id":"author","original_prompt":check,"forwarded_prompt":check}),
    ];
    (calls, trace, current.to_owned())
}

#[test]
fn exact_local_chain_is_not_itself_native_or_semantic_qualification() {
    let (calls, trace, current) = chain();
    let result = recovery_chain("author", &current, &calls, &trace).unwrap();
    assert_eq!(result["native_join_claimed"], false);
    assert_eq!(result["semantic_qualification_claimed"], false);
}

#[test]
fn repeated_ack_text_is_joined_to_distinct_ordered_deliveries() {
    let (mut calls, mut trace, current) = chain();
    let mut second_ack = calls[3].clone();
    second_ack["reply"] = json!("actual check request after second repair");
    calls[3]["reply"] = json!("actual second repair request");
    calls.insert(4, second_ack);
    for (i, call) in calls.iter_mut().enumerate() {
        call["call"] = json!(i + 1);
    }
    trace.insert(3, trace[2].clone());
    let result = recovery_chain("author", &current, &calls, &trace).unwrap();
    assert_eq!(result["repair_ack_call"], 5);
    assert_eq!(result["successful_check_call"], 6);
    trace.remove(3);
    assert!(
        recovery_chain("author", &current, &calls, &trace).is_err(),
        "an additional same-text input cannot be silently left unmatched"
    );
}

#[test]
fn foreign_duplicate_or_forged_current_inputs_are_rejected() {
    for change in 0..6 {
        let (mut calls, mut trace, current) = chain();
        match change {
            0 => calls[2]["agent_id"] = json!("other"),
            1 => {
                calls.insert(3, calls[2].clone());
            }
            2 => trace[1]["components"][0]["body_end"] = json!(999999),
            3 => trace[1]["original_prompt"] = json!("different non-target diagnostic"),
            4 => trace[1]["metadata"]["snapshots"][0]["diff_disposition"] = json!("omitted"),
            _ => calls[2]["prompt"] = json!("a copied current full text marker"),
        }
        assert!(
            recovery_chain("author", &current, &calls, &trace).is_err(),
            "case {change}"
        );
    }
}

#[test]
fn missing_failed_or_timeout_check_and_unprocessed_completion_are_not_success() {
    for replacement in [
        "@work-leaf done\n@work-leaf read src/lib.rs",
        "```\n@work-leaf done\n```",
        "    @work-leaf done",
        "not completed",
    ] {
        let (mut calls, trace, current) = chain();
        calls[4]["reply"] = json!(replacement);
        assert!(recovery_chain("author", &current, &calls, &trace).is_err());
    }
    for header in ["status: 1", "status: 0\ntimed out: yes"] {
        let (mut calls, mut trace, current) = chain();
        let check = calls[4]["prompt"]
            .as_str()
            .unwrap()
            .replace("status: 0", header);
        calls[4]["prompt"] = json!(check);
        trace[3]["original_prompt"] = json!(check);
        trace[3]["forwarded_prompt"] = json!(check);
        assert!(recovery_chain("author", &current, &calls, &trace).is_err());
    }
    let (mut calls, trace, current) = chain();
    calls[4]["returned"] = json!(false);
    assert!(recovery_chain("author", &current, &calls, &trace).is_err());
}
