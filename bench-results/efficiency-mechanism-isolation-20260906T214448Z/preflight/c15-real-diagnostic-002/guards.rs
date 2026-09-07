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
