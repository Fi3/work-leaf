use std::collections::{HashMap, HashSet};
use std::io;

use serde_json::Value;

pub const INITIAL_SOURCE: &str = "pub fn capped(value: u8, _limit: u8) -> u8 { value }\n";
pub const INITIAL_MANIFEST: &str =
    "[package]\nname = \"private_preview_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
pub const INITIAL_LOCK: &str =
    "version = 4\n\n[[package]]\nname = \"private_preview_fixture\"\nversion = \"0.1.0\"\n";

pub fn fixture_request() -> String {
    format!(
        "Implement capped(value, limit) so it returns the smaller input, retaining below-limit and equal-limit behavior. Write your own regression tests using the private test-first workflow described by Work Leaf, then submit the ordinary combined shared patch and validate it. This is a bounded plumbing diagnostic, not a token-saving estimate. The complete current src/lib.rs is:\n{INITIAL_SOURCE}\nThe complete Cargo.toml is:\n{INITIAL_MANIFEST}\nCargo.lock is already committed; the crate has no dependencies. The focused check is cargo test --offline --locked. All initial project source is supplied here. Use Work Leaf directives for changes and checks, and finish with @work-leaf done when your ordinary work is complete."
    )
}

pub fn diagnostic_chat<B: work_leaf::AgentBackend>(
    project: std::path::PathBuf,
    backend: B,
) -> work_leaf::CommandChat<B> {
    work_leaf::CommandChat::new(project, backend)
        .with_locked_command_timeout(std::time::Duration::from_secs(30))
}

pub fn await_author_completion(
    timeout: std::time::Duration,
    mut inspect: impl FnMut() -> io::Result<Value>,
) -> io::Result<Value> {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if let Ok(value) = inspect()
            && std::time::Instant::now() < deadline
        {
            return Ok(value);
        }
        std::thread::sleep(
            std::time::Duration::from_millis(25)
                .min(deadline.saturating_duration_since(std::time::Instant::now())),
        );
    }
    Err(io::Error::other(
        "author check/DONE capture did not qualify within the bounded readiness interval",
    ))
}

/// Pre-review public delivery gate, not a native-rollout or semantic test audit.
pub fn author_completion(
    author: &str,
    command: &str,
    trace: &[Value],
    client: &[Value],
    forwarded: &[Value],
    server: &[Value],
) -> io::Result<Value> {
    use std::collections::VecDeque;
    fn string(v: &Value) -> io::Result<&str> {
        v.as_str().filter(|s| !s.is_empty()).ok_or_else(invalid)
    }
    fn input(v: &Value) -> io::Result<&str> {
        let items = v.as_array().filter(|a| a.len() == 1).ok_or_else(invalid)?;
        if items[0]["type"] != "text" {
            return Err(invalid());
        }
        string(&items[0]["text"])
    }
    fn done(text: &str) -> bool {
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
            if line == "@work-leaf done" {
                if raw.trim_start() != raw {
                    return false;
                }
                count += 1;
            } else if line.starts_with("@work-leaf") {
                return false;
            }
        }
        count == 1
    }
    if author.is_empty()
        || command.is_empty()
        || command.contains(['\r', '\n'])
        || trace.first().is_none_or(|r| {
            r["event"] != "activation"
                || r["schema"] != "work-leaf-bench-experiment-v6"
                || r["condition"] != "private-test-first"
        })
    {
        return Err(invalid());
    }
    let mut replies = HashMap::new();
    let mut users = HashMap::new();
    let mut messages: HashMap<(&str, &str), Vec<(usize, &Value)>> = HashMap::new();
    let mut item_ids = HashSet::new();
    for (line, row) in server.iter().enumerate() {
        if row.get("method").is_none()
            && row.get("id").is_some()
            && replies.insert(string(&row["id"])?, (line, row)).is_some()
        {
            return Err(invalid());
        }
        if row["method"] != "item/completed" {
            continue;
        }
        let p = &row["params"];
        let item = &p["item"];
        if item["type"] != "userMessage" && item["type"] != "agentMessage" {
            continue;
        }
        let key = (string(&p["threadId"])?, string(&p["turnId"])?);
        if !item_ids.insert((key.0, string(&item["id"])?)) {
            return Err(invalid());
        }
        if item["type"] == "userMessage" {
            if users.insert(key, (line, item)).is_some() {
                return Err(invalid());
            }
        } else {
            messages.entry(key).or_default().push((line, item));
        }
    }
    let mut sent = HashMap::new();
    for row in forwarded.iter().filter(|r| r["method"] == "turn/start") {
        if sent.insert(string(&row["id"])?, row).is_some() {
            return Err(invalid());
        }
    }
    let mut turns = Vec::new();
    let mut turn_ids = HashSet::new();
    for (line, row) in client
        .iter()
        .enumerate()
        .filter(|(_, r)| r["method"] == "turn/start")
    {
        let id = string(&row["id"])?;
        if sent.remove(id) != Some(row) {
            return Err(invalid());
        }
        let (reply_line, reply) = replies.get(id).ok_or_else(invalid)?;
        if reply.get("error").is_some() {
            return Err(invalid());
        }
        let key = (
            string(&row["params"]["threadId"])?,
            string(&reply["result"]["turn"]["id"])?,
        );
        if !turn_ids.insert(key) {
            return Err(invalid());
        }
        let text = input(&row["params"]["input"])?;
        let (user_line, user) = users.remove(&key).ok_or_else(invalid)?;
        if input(&user["content"])? != text
            || user["content"][0]["text_elements"] != serde_json::json!([])
            || user_line <= *reply_line
        {
            return Err(invalid());
        }
        turns.push((line, id, key, text, user_line, user));
    }
    if turns.is_empty() || turns.len() > 8 || !sent.is_empty() || !users.is_empty() {
        return Err(invalid());
    }
    let thread = turns[0].2.0;
    if turns.iter().any(|t| t.2.0 != thread) {
        return Err(invalid());
    }
    let mut by_text: HashMap<&str, VecDeque<usize>> = HashMap::new();
    for (i, t) in turns.iter().enumerate() {
        by_text.entry(t.3).or_default().push_back(i);
    }
    let mut policy = false;
    let mut private_delivered = false;
    let mut last = None;
    let mut ack = None;
    let mut check = None;
    for (i, row) in trace.iter().enumerate().skip(1) {
        if row["sequence"].as_u64() != Some(i as u64) || row["agent_id"] != author {
            return Err(invalid());
        }
        if row["event"] == "private-preview-delivered" {
            if !policy || row["private_preview"]["runtime_send_returned"] != true {
                return Err(invalid());
            }
            private_delivered = true;
        }
        if row["event"] != "prompt" {
            continue;
        }
        let text = string(&row["forwarded_prompt"])?;
        let occurrence = by_text
            .get_mut(text)
            .and_then(VecDeque::pop_front)
            .ok_or_else(invalid)?;
        if last.is_some_and(|p| occurrence <= p) {
            return Err(invalid());
        }
        last = Some(occurrence);
        match row["site"].as_str() {
            Some("policy-injection") => {
                if policy || occurrence != 0 || row["owned_role"] != "author" {
                    return Err(invalid());
                }
                policy = true;
            }
            Some("patch-applied") => {
                if !policy
                    || !private_delivered
                    || row["original_prompt"] != text
                    || !text.starts_with("work-leaf patch applied\nfiles: ")
                {
                    return Err(invalid());
                }
                ack = Some(occurrence);
                check = None;
            }
            Some("command-result") => {
                let expected = format!(
                    "work-leaf command result\ncommand: {command}\nstatus: 0\nlocked paths: "
                );
                let not_timed_out = text.split_once("\nstdout:\n").is_some_and(|(header, _)| {
                    !header.lines().any(|line| line == "timed out: yes")
                });
                check = if ack.is_some_and(|a| occurrence > a)
                    && row["original_prompt"] == text
                    && text.starts_with(&expected)
                    && not_timed_out
                {
                    Some(occurrence)
                } else {
                    None
                };
            }
            _ => {}
        }
    }
    let check = check
        .filter(|i| *i == turns.len() - 1 && last == Some(*i))
        .ok_or_else(invalid)?;
    let t = &turns[check];
    let reply = messages.get(&t.2).ok_or_else(invalid)?;
    let final_reply = reply.last().ok_or_else(invalid)?;
    if reply.iter().any(|r| r.0 <= t.4) || !done(string(&final_reply.1["text"])?) {
        return Err(invalid());
    }
    for (_, item) in &reply[..reply.len() - 1] {
        if string(&item["text"])?.lines().map(str::trim).any(|line| {
            line.starts_with("@work-leaf")
                || line.starts_with("```")
                || line.starts_with("~~~")
                || line.starts_with("*** ")
                || line.starts_with("diff --git ")
        }) {
            return Err(invalid());
        }
    }
    if messages.keys().any(|key| !turn_ids.contains(key)) {
        return Err(invalid());
    }
    Ok(
        serde_json::json!({"author_id":author,"thread_id":thread,"command":command,
        "post_patch_status":0,"ack_client_line":turns[ack.ok_or_else(invalid)?].0+1,
        "check_client_line":t.0+1,"check_request_id":t.1,"check_turn_id":t.2.1,
        "check_public_user_line":t.4+1,"check_public_user_id":t.5["id"],
        "done_public_line":final_reply.0+1,"done_public_item_id":final_reply.1["id"],
        "native_join_claimed":false}),
    )
}

fn invalid() -> io::Error {
    io::Error::other("private test-first diagnostic scope, role or call limit rejected")
}

/// Public transport closure only; native activity needs its own closed inventory.
pub fn settled_turns(client: &[Value], forwarded: &[Value], server: &[Value]) -> io::Result<usize> {
    fn string(value: &Value) -> io::Result<&str> {
        value.as_str().filter(|s| !s.is_empty()).ok_or_else(invalid)
    }
    let mut replies = HashMap::new();
    for row in server
        .iter()
        .filter(|v| v.get("method").is_none() && v.get("id").is_some())
    {
        if replies.insert(string(&row["id"])?, row).is_some() {
            return Err(invalid());
        }
    }
    let input: Vec<_> = client
        .iter()
        .filter(|v| v["method"] == "turn/start")
        .collect();
    let sent: Vec<_> = forwarded
        .iter()
        .filter(|v| v["method"] == "turn/start")
        .collect();
    if input.is_empty() || input.len() > 8 || input != sent {
        return Err(invalid());
    }
    let mut latest = HashMap::new();
    let mut unique = HashSet::new();
    let mut ids = HashSet::new();
    for row in &input {
        let id = string(&row["id"])?;
        let reply = replies.get(id).ok_or_else(invalid)?;
        let key = (
            string(&row["params"]["threadId"])?,
            string(&reply["result"]["turn"]["id"])?,
        );
        if reply.get("error").is_some() || !ids.insert(id) || !unique.insert(key) {
            return Err(invalid());
        }
        latest.insert(key.0, key.1);
    }
    if latest.len() > 2 {
        return Err(invalid());
    }
    let mut forwarded_ids = HashMap::new();
    for row in forwarded.iter().filter(|v| v.get("id").is_some()) {
        if forwarded_ids.insert(string(&row["id"])?, row).is_some() {
            return Err(invalid());
        }
    }
    let mut interrupts = HashSet::new();
    for row in client.iter().filter(|v| v["method"] == "turn/interrupt") {
        let key = (
            string(&row["params"]["threadId"])?,
            string(&row["params"]["turnId"])?,
        );
        if latest.get(key.0).copied() != Some(key.1) {
            continue;
        }
        let id = string(&row["id"])?;
        let reply = replies.get(id).ok_or_else(invalid)?;
        if !interrupts.insert(key)
            || !ids.insert(id)
            || forwarded_ids.get(id).copied() != Some(row)
            || reply.get("error").is_some()
            || reply.get("result").is_none()
        {
            return Err(invalid());
        }
    }
    let mut closed = HashSet::new();
    for row in server.iter().filter(|v| v["method"] == "turn/completed") {
        let key = (
            string(&row["params"]["threadId"])?,
            string(&row["params"]["turn"]["id"])?,
        );
        if latest.get(key.0).copied() != Some(key.1) {
            continue;
        }
        let status = &row["params"]["turn"]["status"];
        if !closed.insert(key)
            || !(status == "completed" || status == "interrupted" && interrupts.contains(&key))
        {
            return Err(invalid());
        }
    }
    if closed.len() != latest.len() {
        return Err(invalid());
    }
    Ok(input.len())
}

/// The fixture controller sets the phase; names never determine real-agent roles.
pub struct Budget {
    author: String,
    author_started: bool,
    review_phase: bool,
    reviewer: Option<String>,
    calls: usize,
    maximum: usize,
}

impl Budget {
    pub fn new(author: &str, maximum: usize) -> io::Result<Self> {
        if author.is_empty() || !(1..=8).contains(&maximum) {
            return Err(invalid());
        }
        Ok(Self {
            author: author.to_owned(),
            author_started: false,
            review_phase: false,
            reviewer: None,
            calls: 0,
            maximum,
        })
    }

    pub fn begin_review(&mut self) -> io::Result<()> {
        if !self.author_started || self.review_phase {
            return Err(invalid());
        }
        self.review_phase = true;
        Ok(())
    }

    pub fn admit(&mut self, agent: &str, launch: bool) -> io::Result<()> {
        if self.calls >= self.maximum || agent.is_empty() {
            return Err(invalid());
        }
        if !self.review_phase {
            if agent != self.author || launch == self.author_started {
                return Err(invalid());
            }
            self.author_started = true;
        } else if agent == self.author {
            if launch {
                return Err(invalid());
            }
        } else if let Some(reviewer) = &self.reviewer {
            if launch || reviewer != agent {
                return Err(invalid());
            }
        } else if launch {
            self.reviewer = Some(agent.to_owned());
        } else {
            return Err(invalid());
        }
        self.calls += 1;
        Ok(())
    }

    pub fn calls(&self) -> usize {
        self.calls
    }
    pub fn real_roles(&self) -> usize {
        usize::from(self.author_started) + usize::from(self.reviewer.is_some())
    }
}

pub fn validate_settings(
    config: &Value,
    manifest: &Value,
    lookup: impl Fn(&str) -> Option<String>,
) -> io::Result<()> {
    for key in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "OPENAI_BASE_URL",
        "OPENAI_API_BASE",
        "CODEX_BASE_URL",
        "CODEX_ACCESS_TOKEN",
        "WORK_LEAF_CODEX_TRACE",
        "WORK_LEAF_OBSERVER_PARENT_INVOCATION",
    ] {
        if lookup(key).is_some() {
            return Err(invalid());
        }
    }
    for (key, expected) in [
        ("WORK_LEAF_REAL_PRIVATE_TEST_FIRST_SMOKE", "1"),
        ("WORK_LEAF_BENCH_EXPERIMENT", "1"),
        ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "1"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "1000"),
        (
            "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
            "forward",
        ),
        ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "1"),
    ] {
        if lookup(key).as_deref() != Some(expected) {
            return Err(invalid());
        }
    }
    if config["condition"] != "work-leaf"
        || config["model"] != "gpt-5.5"
        || config["effort"] != "xhigh"
        || manifest["schema"] != "work-leaf-bench-experiment-v6"
        || manifest["condition"] != "private-test-first"
    {
        return Err(invalid());
    }
    let run = manifest["run_id"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(invalid)?;
    let marker = config["primary_invocation_marker"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(invalid)?;
    let project = manifest["private_preview"]["project_root"]
        .as_str()
        .filter(|s| s.starts_with('/'))
        .ok_or_else(invalid)?;
    if config["run_id"] != run
        || lookup("WORK_LEAF_BENCH_RUN_ID").as_deref() != Some(run)
        || lookup("WORK_LEAF_OBSERVER_PRIMARY_MARKER").as_deref() != Some(marker)
        || lookup("WORK_LEAF_REAL_OBSERVER_PROJECT_DIR").as_deref() != Some(project)
    {
        return Err(invalid());
    }
    Ok(())
}
