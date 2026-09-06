//! Response identities are independent evidence, never additions to cumulative thread totals.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde_json::{Value, json};

use super::{CapturedUsage, extract_thread_id};

fn rpc_key(value: &Value) -> Option<String> {
    let id = value.get("id")?;
    (id.is_string() || id.is_number()).then(|| id.to_string())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ResponseRecord {
    response_id: String,
    thread_id: String,
    turn_id: String,
    usage: Option<CapturedUsage>,
    valid: bool,
    first_server_sequence: usize,
}

#[derive(Default)]
struct ThreadLedger {
    response_count: usize,
    unknown_response_count: usize,
    usage: CapturedUsage,
    invalid: bool,
}

fn nonempty_string<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)?
        .as_str()
        .filter(|text| !text.trim().is_empty())
}

fn strict_usage(value: &Value) -> Result<CapturedUsage, String> {
    let counter = |name: &str| {
        value
            .get(name)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("missing or invalid {name}"))
    };
    let usage = CapturedUsage {
        input_tokens: counter("inputTokens")?,
        cached_input_tokens: counter("cachedInputTokens")?,
        output_tokens: counter("outputTokens")?,
        reasoning_output_tokens: counter("reasoningOutputTokens")?,
    };
    if usage.cached_input_tokens > usage.input_tokens
        || usage.reasoning_output_tokens > usage.output_tokens
        || usage.input_tokens.checked_add(usage.output_tokens) != Some(counter("totalTokens")?)
    {
        return Err("inconsistent token arithmetic".into());
    }
    Ok(usage)
}

fn checked_sum(left: CapturedUsage, right: CapturedUsage) -> Option<CapturedUsage> {
    Some(CapturedUsage {
        input_tokens: left.input_tokens.checked_add(right.input_tokens)?,
        cached_input_tokens: left
            .cached_input_tokens
            .checked_add(right.cached_input_tokens)?,
        output_tokens: left.output_tokens.checked_add(right.output_tokens)?,
        reasoning_output_tokens: left
            .reasoning_output_tokens
            .checked_add(right.reasoning_output_tokens)?,
    })
}

/// Reconcile only the observed completed responses. Equality never proves that an interrupted
/// response was reported, nor that this experimental notification inventories every provider call.
/// Indexed passes cost O(n log n) time and O(n) storage, independent of response ordering.
pub(super) fn analyze(client: &[Value], server: &[Value]) -> Value {
    let requests = client
        .iter()
        .filter(|value| value.get("method").and_then(Value::as_str).is_some())
        .filter_map(|value| Some((rpc_key(value)?, value)))
        .collect::<BTreeMap<_, _>>();
    let mut fresh_threads = BTreeSet::new();
    let mut captured_turns = BTreeSet::new();
    for reply in server {
        if reply.get("method").is_some() || reply.get("error").is_some() {
            continue;
        }
        let Some(request) = rpc_key(reply).and_then(|id| requests.get(&id)) else {
            continue;
        };
        match request.get("method").and_then(Value::as_str) {
            Some("thread/start") => {
                if let Some(thread) = reply.pointer("/result/thread")
                    && thread
                        .get("turns")
                        .and_then(Value::as_array)
                        .is_some_and(Vec::is_empty)
                    && let Some(thread_id) = nonempty_string(thread, "id")
                {
                    fresh_threads.insert(thread_id.to_owned());
                }
            }
            Some("turn/start") => {
                if let (Some(thread_id), Some(turn_id)) = (
                    extract_thread_id(request),
                    reply
                        .pointer("/result/turn")
                        .and_then(|turn| nonempty_string(turn, "id"))
                        .map(str::to_owned),
                ) && extract_thread_id(reply).is_none_or(|id| id == thread_id)
                {
                    captured_turns.insert((thread_id, turn_id));
                }
            }
            _ => {}
        }
    }

    let mut errors = Vec::new();
    let mut records = BTreeMap::<String, ResponseRecord>::new();
    let mut invalid_threads = BTreeSet::new();
    let mut duplicate_events = 0;
    for (sequence, value) in server.iter().enumerate() {
        if value.get("method").and_then(Value::as_str) != Some("rawResponse/completed") {
            continue;
        }
        let params = &value["params"];
        let (Some(response), Some(thread), Some(turn)) = (
            nonempty_string(params, "responseId"),
            nonempty_string(params, "threadId"),
            nonempty_string(params, "turnId"),
        ) else {
            errors.push(format!(
                "raw response at server sequence {sequence} lacks response/thread/turn identity"
            ));
            if let Some(thread) = nonempty_string(params, "threadId") {
                invalid_threads.insert(thread.to_owned());
            }
            continue;
        };
        let mut valid = captured_turns.contains(&(thread.to_owned(), turn.to_owned()));
        if !valid {
            errors.push(format!(
                "raw response {response} has no matching captured turn/start"
            ));
        }
        let usage = match params.get("usage").filter(|usage| !usage.is_null()) {
            None => None,
            Some(usage) => match strict_usage(usage) {
                Ok(usage) => Some(usage),
                Err(error) => {
                    valid = false;
                    errors.push(format!("raw response {response}: {error}"));
                    None
                }
            },
        };
        let record = ResponseRecord {
            response_id: response.into(),
            thread_id: thread.into(),
            turn_id: turn.into(),
            usage,
            valid,
            first_server_sequence: sequence,
        };
        if let Some(previous) = records.get(response) {
            if previous.thread_id != record.thread_id
                || previous.turn_id != record.turn_id
                || previous.usage != record.usage
                || previous.valid != record.valid
            {
                errors.push(format!(
                    "conflicting raw response identity or usage: {response}"
                ));
                invalid_threads.insert(previous.thread_id.clone());
                invalid_threads.insert(thread.to_owned());
            } else {
                duplicate_events += 1;
            }
        } else {
            records.insert(response.into(), record);
        }
    }
    let mut threads = BTreeMap::<String, ThreadLedger>::new();
    for record in records.values() {
        let ledger = threads.entry(record.thread_id.clone()).or_default();
        ledger.response_count += 1;
        ledger.invalid |= !record.valid || invalid_threads.contains(&record.thread_id);
        if let Some(usage) = record.usage.filter(|_| record.valid) {
            if let Some(total) = checked_sum(ledger.usage, usage) {
                ledger.usage = total;
            } else {
                ledger.invalid = true;
                errors.push(format!(
                    "raw response sum overflow for thread {}",
                    record.thread_id
                ));
            }
        } else {
            ledger.unknown_response_count += 1;
        }
    }
    let mut cumulative = BTreeMap::new();
    for value in server {
        if value.get("method").and_then(Value::as_str) != Some("thread/tokenUsage/updated") {
            continue;
        }
        let Some(thread) = extract_thread_id(value) else {
            continue;
        };
        let Some(ledger) = threads.get_mut(&thread) else {
            continue;
        };
        match strict_usage(&value["params"]["tokenUsage"]["total"]) {
            Ok(usage) => {
                if cumulative
                    .get(&thread)
                    .is_some_and(|previous| usage.checked_difference(*previous).is_none())
                {
                    ledger.invalid = true;
                    errors.push(format!("cumulative usage regressed for thread {thread}"));
                }
                cumulative.insert(thread, usage);
            }
            Err(error) => {
                ledger.invalid = true;
                errors.push(format!(
                    "raw response reconciliation for thread {thread}: {error}"
                ));
            }
        }
    }
    let rows = threads.into_iter().map(|(thread, ledger)| {
        let total = cumulative.get(&thread);
        let fresh = fresh_threads.contains(&thread);
        let status = if ledger.invalid { "invalid_response_evidence" }
            else if !fresh { "unavailable_thread_baseline" }
            else if ledger.unknown_response_count > 0 { "incomplete_response_usage" }
            else if total.is_none() { "missing_cumulative_usage" }
            else if total != Some(&ledger.usage) { "cumulative_mismatch" }
            else { "matched_cumulative" };
        json!({"thread_id":thread,"fresh_thread":fresh,"status":status,
            "response_count":ledger.response_count,"unknown_response_count":ledger.unknown_response_count,
            "response_usage":ledger.usage,"cumulative_usage":total})
    }).collect::<Vec<_>>();
    json!({"schema_version":1,"records":records.into_values().collect::<Vec<_>>(),
        "threads":rows,"duplicate_events":duplicate_events,"errors":errors,
        "changes_workflow_totals":false,"proves_interrupted_tail_coverage":false})
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn client() -> Vec<Value> {
        vec![
            json!({"id":1,"method":"thread/start","params":{}}),
            json!({"id":2,"method":"turn/start","params":{"threadId":"thread-a"}}),
        ]
    }

    fn server() -> Vec<Value> {
        vec![
            json!({"id":1,"result":{"thread":{"id":"thread-a","turns":[]}}}),
            json!({"id":2,"result":{"turn":{"id":"turn-a"}}}),
            raw("response-a", 100, 80, 10),
            cumulative(100, 80, 10),
        ]
    }

    fn raw(id: &str, input: u64, cached: u64, output: u64) -> Value {
        json!({"method":"rawResponse/completed","params":{
            "threadId":"thread-a","turnId":"turn-a","responseId":id,
            "usage":{"inputTokens":input,"cachedInputTokens":cached,"outputTokens":output,
                "reasoningOutputTokens":0,"totalTokens":input + output}
        }})
    }

    fn cumulative(input: u64, cached: u64, output: u64) -> Value {
        json!({"method":"thread/tokenUsage/updated","params":{
            "threadId":"thread-a","turnId":"turn-a","tokenUsage":{
                "total":{"inputTokens":input,"cachedInputTokens":cached,"outputTokens":output,
                    "reasoningOutputTokens":0,"totalTokens":input + output}
            }
        }})
    }

    #[test]
    fn reconciles_unique_response_ids_without_adding_cumulative_usage() {
        let mut events = server();
        events.insert(3, raw("response-a", 100, 80, 10));
        events.insert(4, raw("response-b", 50, 20, 5));
        events.push(cumulative(150, 100, 15));
        let report = analyze(&client(), &events);
        assert_eq!(report["records"].as_array().unwrap().len(), 2);
        assert_eq!(report["duplicate_events"], 1);
        assert_eq!(report["threads"][0]["status"], "matched_cumulative");
        assert_eq!(report["threads"][0]["response_usage"]["input_tokens"], 150);
        assert_eq!(
            report["threads"][0]["response_usage"]["cached_input_tokens"],
            100
        );
        assert_eq!(report["threads"][0]["response_usage"]["output_tokens"], 15);
        assert_eq!(report["errors"], json!([]));
    }

    #[test]
    fn missing_usage_is_unknown_not_zero() {
        let mut events = server();
        events[2]["params"]["usage"] = Value::Null;
        let report = analyze(&client(), &events);
        assert_eq!(report["records"][0]["usage"], Value::Null);
        assert_eq!(report["threads"][0]["unknown_response_count"], 1);
        assert_eq!(report["threads"][0]["status"], "incomplete_response_usage");
    }

    #[test]
    fn rejects_missing_negative_and_inconsistent_exact_counters() {
        for usage in [
            json!({}),
            json!({"inputTokens":100,"cachedInputTokens":null,"outputTokens":10,"reasoningOutputTokens":0,"totalTokens":110}),
            json!({"inputTokens":100,"cachedInputTokens":-1,"outputTokens":10,"reasoningOutputTokens":0,"totalTokens":110}),
            json!({"inputTokens":100,"cachedInputTokens":101,"outputTokens":10,"reasoningOutputTokens":0,"totalTokens":110}),
            json!({"inputTokens":100,"cachedInputTokens":80,"outputTokens":10,"reasoningOutputTokens":11,"totalTokens":110}),
            json!({"inputTokens":100,"cachedInputTokens":80,"outputTokens":10,"reasoningOutputTokens":0,"totalTokens":111}),
        ] {
            let mut events = server();
            events[2]["params"]["usage"] = usage;
            let report = analyze(&client(), &events);
            assert!(!report["errors"].as_array().unwrap().is_empty());
        }
    }

    #[test]
    fn rejects_conflicting_response_identity_or_counters() {
        for field in ["threadId", "turnId", "usage"] {
            let mut events = server();
            let mut conflict = events[2].clone();
            conflict["params"][field] = if field == "usage" {
                raw("response-a", 200, 80, 10)["params"]["usage"].clone()
            } else {
                json!("different")
            };
            events.push(conflict);
            assert!(
                !analyze(&client(), &events)["errors"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        }
    }

    #[test]
    fn requires_captured_turn_start_not_merely_notification_identity() {
        let requests = vec![client()[0].clone()];
        let report = analyze(&requests, &server());
        assert!(!report["errors"].as_array().unwrap().is_empty());
        assert_ne!(report["threads"][0]["status"], "matched_cumulative");
    }

    #[test]
    fn resume_and_fork_do_not_assume_a_zero_usage_baseline() {
        for method in ["thread/resume", "thread/fork"] {
            let mut requests = client();
            requests[0]["method"] = json!(method);
            let report = analyze(&requests, &server());
            assert_eq!(
                report["threads"][0]["status"],
                "unavailable_thread_baseline"
            );
        }
    }

    #[test]
    fn matching_completed_prefix_does_not_prove_interrupted_tail() {
        let mut events = server();
        events.push(json!({"method":"turn/completed","params":{
            "threadId":"thread-a","turn":{"id":"turn-a","status":"interrupted"}
        }}));
        let report = analyze(&client(), &events);
        assert_eq!(report["threads"][0]["status"], "matched_cumulative");
        assert_eq!(report["proves_interrupted_tail_coverage"], false);
        assert_eq!(report["changes_workflow_totals"], false);
    }

    #[test]
    fn missing_cumulative_or_unequal_totals_never_reconcile() {
        let mut events = server();
        events.pop();
        assert_eq!(
            analyze(&client(), &events)["threads"][0]["status"],
            "missing_cumulative_usage"
        );
        events.push(cumulative(200, 100, 20));
        assert_eq!(
            analyze(&client(), &events)["threads"][0]["status"],
            "cumulative_mismatch"
        );
    }

    #[test]
    fn response_events_may_precede_the_turn_start_reply() {
        let mut events = server();
        events.swap(1, 2);
        assert_eq!(
            analyze(&client(), &events)["threads"][0]["status"],
            "matched_cumulative"
        );
    }

    #[test]
    fn server_request_cannot_impersonate_a_turn_start_reply() {
        let mut events = server();
        events[1] = json!({"id":2,"method":"item/tool/call","params":{
            "threadId":"thread-a","turnId":"turn-a","callId":"call-a"}});
        let report = analyze(&client(), &events);
        assert!(!report["errors"].as_array().unwrap().is_empty());
    }

    #[test]
    fn client_callback_reply_cannot_shadow_its_outbound_request_id() {
        let mut requests = client();
        requests.push(json!({"id":2,"result":{"success":true}}));
        assert_eq!(
            analyze(&requests, &server())["threads"][0]["status"],
            "matched_cumulative"
        );
    }

    #[test]
    fn regressed_cumulative_snapshot_cannot_manufacture_reconciliation() {
        let mut events = server();
        events.insert(3, cumulative(200, 100, 20));
        let report = analyze(&client(), &events);
        assert!(!report["errors"].as_array().unwrap().is_empty());
        assert_eq!(report["threads"][0]["status"], "invalid_response_evidence");
    }

    #[test]
    fn rpc_string_and_numeric_ids_do_not_identify_the_same_request() {
        let mut events = server();
        events[1]["id"] = json!("2");
        assert!(
            !analyze(&client(), &events)["errors"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
}
