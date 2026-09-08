//! Conservative local qualification guards, not a replacement protocol parser.
use std::collections::{HashMap, HashSet, VecDeque};
use std::io;

use serde_json::{Value, json};

pub const INITIAL_SOURCE: &str = "pub fn capped(value: u8, _limit: u8) -> u8 { value }\n";
pub const CURRENT_SOURCE: &str = "pub fn capped(value: u8, _limit: u8) -> u8 { value + 0 }\n";
pub const INITIAL_MANIFEST: &str =
    "[package]\nname = \"automatic_refresh_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
pub const INITIAL_LOCK: &str =
    "version = 4\n\n[[package]]\nname = \"automatic_refresh_fixture\"\nversion = \"0.1.0\"\n";

fn invalid() -> io::Error {
    io::Error::other("local automatic-refresh chain does not qualify")
}

// Deliberately narrower than the orchestrator grammar: ambiguous quoted/patch
// contexts are rejected, never promoted to an execution or success claim.
fn sole_directive(text: &str, expected: &str) -> bool {
    let mut count = 0;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with("```")
            || line.starts_with("~~~")
            || line.starts_with("*** ")
            || line.starts_with("diff --git ")
        {
            return false;
        }
        if line.starts_with("@work-leaf") {
            if raw.trim_start() != raw || line != expected {
                return false;
            }
            count += 1;
        }
    }
    count == 1
}

pub fn initial_read_owned(previous_reply: &str, actual_prompt: &str) -> bool {
    sole_directive(previous_reply, "@work-leaf read src/lib.rs")
        && actual_prompt == format!("work-leaf file text\n\n--- src/lib.rs ---\n{INITIAL_SOURCE}")
}

fn text<'a>(row: &'a Value, key: &str) -> io::Result<&'a str> {
    row[key].as_str().ok_or_else(invalid)
}

fn offset(row: &Value, key: &str) -> io::Result<usize> {
    row[key]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(invalid)
}

/// Join exact local backend inputs and the owned v7 evidence spans. Native
/// delivery, Git acceptance and test semantics require separate real witnesses.
pub fn recovery_chain(
    author: &str,
    current: &str,
    calls: &[Value],
    trace: &[Value],
) -> io::Result<Value> {
    if author.is_empty()
        || current.is_empty()
        || !(5..=8).contains(&calls.len())
        || trace.first().is_none_or(|row| {
            row["event"] != "activation"
                || row["schema"] != "work-leaf-bench-experiment-v7"
                || row["condition"] != "automatic-changed-refresh-full"
        })
    {
        return Err(invalid());
    }
    for (i, row) in calls.iter().enumerate() {
        if row["call"].as_u64() != Some((i + 1) as u64)
            || row["agent_id"] != author
            || row["launch"] != json!(i == 0)
            || row["returned"] != true
            || row.get("error").is_some()
            || row["prompt"].as_str().is_none()
            || row["reply"].as_str().is_none()
        {
            return Err(invalid());
        }
    }
    let reads: Vec<_> = (1..calls.len())
        .filter(|&i| {
            initial_read_owned(
                calls[i - 1]["reply"].as_str().unwrap(),
                calls[i]["prompt"].as_str().unwrap(),
            )
        })
        .collect();
    if reads.len() != 1 {
        return Err(invalid());
    }
    let rows: Vec<_> = trace
        .iter()
        .filter(|r| r["event"] == "automatic-refresh")
        .collect();
    if rows.len() != 1 {
        return Err(invalid());
    }
    let refresh = rows[0];
    if refresh["agent_id"] != author
        || refresh["eligible"] != true
        || refresh["selected_candidate"] != "candidate"
        || !matches!(refresh["metadata"]["kind"].as_str(), Some("edit" | "patch"))
    {
        return Err(invalid());
    }
    let original = text(refresh, "original_prompt")?;
    let candidate = text(refresh, "candidate_prompt")?;
    let snapshots = refresh["metadata"]["snapshots"]
        .as_array()
        .ok_or_else(invalid)?;
    let parts = refresh["components"]
        .as_array()
        .filter(|p| !p.is_empty())
        .ok_or_else(invalid)?;
    let (mut before, mut after, mut bodies) = (0, 0, 0);
    for part in parts {
        let (bs, be, cs, ce) = (
            offset(part, "baseline_start")?,
            offset(part, "baseline_end")?,
            offset(part, "candidate_start")?,
            offset(part, "candidate_end")?,
        );
        if bs < before
            || cs < after
            || original.get(bs..be).is_none()
            || candidate.get(cs..ce).is_none()
            || original.get(before..bs) != candidate.get(after..cs)
        {
            return Err(invalid());
        }
        match part["id"].as_str() {
            Some("current-full-text") => {
                let (start, end) = (offset(part, "body_start")?, offset(part, "body_end")?);
                let snapshot = snapshots
                    .get(offset(part, "snapshot_index")?)
                    .ok_or_else(invalid)?;
                if start < cs
                    || end > ce
                    || candidate.get(start..end) != Some(current)
                    || snapshot["path"] != "src/lib.rs"
                    || snapshot["class"] != "changed"
                    || snapshot["diff_disposition"] != "available"
                {
                    return Err(invalid());
                }
                bodies += 1;
            }
            // These are the renderer-owned coherence components; no other
            // replacement can be admitted by a permissive body substring.
            Some("refresh-intro" | "refresh-guidance") => {
                if !part["body_start"].is_null() || !part["body_end"].is_null() {
                    return Err(invalid());
                }
            }
            _ => return Err(invalid()),
        }
        before = be;
        after = ce;
    }
    if bodies != 1 || original.get(before..) != candidate.get(after..) {
        return Err(invalid());
    }
    let deliveries: Vec<_> = calls
        .iter()
        .enumerate()
        .filter(|(_, r)| r["prompt"] == candidate)
        .map(|(i, _)| i)
        .collect();
    if deliveries.len() != 1 || deliveries[0] <= reads[0] {
        return Err(invalid());
    }
    let refresh_call = deliveries[0];
    let mut ack = None;
    let mut check = None;
    let mut last = refresh_call;
    let mut occurrences: HashMap<&str, VecDeque<usize>> = HashMap::new();
    for (i, call) in calls.iter().enumerate() {
        occurrences
            .entry(text(call, "prompt")?)
            .or_default()
            .push_back(i);
    }
    let mut joined = HashSet::new();
    for row in trace.iter().filter(|r| {
        r["event"] == "prompt"
            && matches!(r["site"].as_str(), Some("patch-applied" | "command-result"))
    }) {
        if row["agent_id"] != author {
            return Err(invalid());
        }
        let input = text(row, "forwarded_prompt")?;
        if row["original_prompt"] != input {
            return Err(invalid());
        }
        let occurrence = occurrences
            .get_mut(input)
            .and_then(VecDeque::pop_front)
            .ok_or_else(invalid)?;
        if occurrence <= last {
            return Err(invalid());
        }
        joined.insert(input);
        last = occurrence;
        if row["site"] == "patch-applied" {
            if !input.starts_with("work-leaf patch applied\nfiles: ") {
                return Err(invalid());
            }
            ack = Some(last);
            check = None;
        } else {
            let valid = input.starts_with("work-leaf command result\ncommand: cargo test --offline --locked\nstatus: 0\nlocked paths: ")
                && input.split_once("\nstdout:\n").is_some_and(|(header,_)| !header.lines().any(|l|l=="timed out: yes"));
            if ack.is_some() && valid {
                check = Some(last);
            } else {
                check = None;
            }
        }
    }
    if joined.iter().any(|input| !occurrences[input].is_empty()) {
        return Err(invalid());
    }
    let Some(check_call) = check else {
        return Err(invalid());
    };
    if check_call != calls.len() - 1
        || !sole_directive(text(&calls[check_call], "reply")?, "@work-leaf done")
    {
        return Err(invalid());
    }
    Ok(
        json!({"author_id":author,"read_call":reads[0]+1,"refresh_call":refresh_call+1,
        "repair_ack_call":ack.map(|n|n+1),"successful_check_call":check_call+1,
        "full_current_body_bytes":current.len(),"native_join_claimed":false,"semantic_qualification_claimed":false}),
    )
}
