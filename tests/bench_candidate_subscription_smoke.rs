#![cfg(feature = "bench-experiments")]

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, ChatMessage, CodexBackend, CodexCommandConfig, CommandChat,
    CommandChatResult, PromptPolicy, SandboxMode,
};

const READ: &str = "@work-leaf read fixture.rs";
const REPEAT: &str = "@work-leaf read --force fixture.rs";
const DONE: &str = "@work-leaf done";
const PATCH_ONE: &str = "@work-leaf patch set diagnostic value\ndiff --git a/fixture.rs b/fixture.rs\n--- a/fixture.rs\n+++ b/fixture.rs\n@@ -1 +1 @@\n-pub const VALUE: u8 = 0;\n+pub const VALUE: u8 = 1;\n@work-leaf end";
const EDIT_ONE: &str = "@work-leaf edit set diagnostic value\n*** Begin Patch\n*** Update File: fixture.rs\n@@\n-pub const VALUE: u8 = 0;\n+pub const VALUE: u8 = 1;\n*** End Patch\n@work-leaf end";
const EDIT_TWO: &str = "@work-leaf edit complete diagnostic value\n*** Begin Patch\n*** Update File: fixture.rs\n@@\n-pub const VALUE: u8 = 1;\n+pub const VALUE: u8 = 2;\n*** End Patch\n@work-leaf end";

fn rejected() -> AgentError {
    AgentError::Io(io::Error::other(
        "candidate smoke rejected unplanned call, identity, or action",
    ))
}

fn expected_step(condition: &str, index: usize) -> Option<(bool, bool, &'static str)> {
    match (condition, index) {
        ("requested-repeat-full" | "unified-diff-preferred" | "review-fix-request-resupply", 0) => {
            Some((false, true, ""))
        }
        ("requested-repeat-full", 1 | 2) => Some((false, false, "work-leaf file text\n")),
        ("unified-diff-preferred" | "review-fix-request-resupply", 1) => {
            Some((false, false, "work-leaf patch applied\n"))
        }
        ("review-fix-request-resupply", 2) => Some((true, true, "Review the full patch scope")),
        ("review-fix-request-resupply", 3) => {
            Some((false, false, "The reviewer found issues in your patch"))
        }
        ("review-fix-request-resupply", 4) => Some((false, false, "work-leaf patch applied\n")),
        ("review-fix-request-resupply", 5) => Some((
            true,
            false,
            "The original agent has responded to the findings",
        )),
        _ => None,
    }
}

struct Budget {
    condition: String,
    calls: usize,
}
impl Budget {
    fn new(condition: &str) -> Result<Self, AgentError> {
        expected_step(condition, 0).ok_or_else(rejected)?;
        Ok(Self {
            condition: condition.to_string(),
            calls: 0,
        })
    }
    fn admit(&mut self, id: &str, launch: bool, prompt: &str) -> Result<(), AgentError> {
        let (reviewer, expected_launch, prefix) =
            expected_step(&self.condition, self.calls).ok_or_else(rejected)?;
        let expected_id = if reviewer {
            "review-user-candidate"
        } else {
            "user-candidate"
        };
        if id != expected_id || launch != expected_launch || !prompt.starts_with(prefix) {
            return Err(rejected());
        }
        self.calls += 1;
        Ok(())
    }
    fn reply(&self, text: &str) -> Result<(), AgentError> {
        let expected = match (self.condition.as_str(), self.calls) {
            ("requested-repeat-full", 1) => READ,
            ("requested-repeat-full", 2) => REPEAT,
            ("unified-diff-preferred", 1) => PATCH_ONE,
            ("review-fix-request-resupply", 1) => EDIT_ONE,
            ("review-fix-request-resupply", 3) => {
                return if text.lines().next() == Some("FINDINGS")
                    && text.contains("VALUE")
                    && text.contains('2')
                {
                    Ok(())
                } else {
                    Err(rejected())
                };
            }
            ("review-fix-request-resupply", 4) => EDIT_TWO,
            ("review-fix-request-resupply", 6) => "NO_FINDINGS",
            _ => DONE,
        };
        if text.trim() == expected {
            Ok(())
        } else {
            Err(rejected())
        }
    }
}

struct BoundedBackend {
    inner: CodexBackend,
    budget: Budget,
}
impl AgentBackend for BoundedBackend {
    fn launch(&mut self, _: AgentLaunch) -> Result<AgentSession, AgentError> {
        Err(rejected())
    }
    fn send(&mut self, _: &AgentId, _: &str) -> Result<ChatMessage, AgentError> {
        Err(rejected())
    }
    fn session(&self, id: &AgentId) -> Option<AgentSession> {
        self.inner.session(id)
    }
    fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.inner.shutdown_handle()
    }
    fn launch_streaming_interruptible(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
        interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<AgentSession, AgentError> {
        self.budget
            .admit(request.id.as_str(), true, &request.prompt)?;
        let session = self
            .inner
            .launch_streaming_interruptible(request, sink, interrupt)?;
        self.budget
            .reply(&session.messages.last().ok_or_else(rejected)?.text)?;
        Ok(session)
    }
    fn send_streaming_interruptible(
        &mut self,
        id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        self.budget.admit(id.as_str(), false, prompt)?;
        let reply = self
            .inner
            .send_streaming_interruptible(id, prompt, sink, interrupt)?;
        self.budget.reply(&reply.text)?;
        Ok(reply)
    }
}

fn frames(path: &Path) -> io::Result<Vec<Value>> {
    fs::read(path)?
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| line.ends_with(b"\n"))
        .map(|line| serde_json::from_slice(line).map_err(io::Error::other))
        .collect()
}

fn closed_frame_bytes(bytes: &[u8]) -> io::Result<Vec<Value>> {
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        return Err(io::Error::other(
            "closed capture contains an incomplete final frame",
        ));
    }
    bytes
        .split_inclusive(|byte| *byte == b'\n')
        .map(|line| serde_json::from_slice(line).map_err(io::Error::other))
        .collect()
}

fn closed_frames(path: &Path) -> io::Result<Vec<Value>> {
    closed_frame_bytes(&fs::read(path)?)
}

#[test]
fn candidate_smoke_closed_capture_rejects_partial_last_frame() {
    assert!(closed_frame_bytes(b"{\"id\":1}\n").is_ok());
    assert!(closed_frame_bytes(b"{\"id\":1}\n{\"method\":\"turn/start\"").is_err());
    assert!(closed_frame_bytes(b"{\"id\":1}").is_err());
    assert!(closed_frame_bytes(b"{\"id\":1}\ntrailing").is_err());
    assert!(closed_frame_bytes(b"{\"id\":1}\ninvalid\n").is_err());
}

fn accepted_turn<'a>(start: &Value, server: &'a [Value]) -> Option<&'a str> {
    if !matches!(start.get("id"), Some(Value::String(_) | Value::Number(_))) {
        return None;
    }
    let mut replies = server
        .iter()
        .filter(|reply| reply.get("method").is_none() && reply.get("id") == start.get("id"));
    let reply = replies.next()?;
    if replies.next().is_some() || reply.get("error").is_some() {
        return None;
    }
    reply["result"]["turn"]["id"]
        .as_str()
        .filter(|id| !id.is_empty())
}

fn public_delivery(original: &[Value], forwarded: &[Value], server: &[Value]) -> bool {
    use std::collections::{HashMap, HashSet};
    let starts: Vec<_> = original
        .iter()
        .filter(|frame| frame["method"] == "turn/start")
        .collect();
    if starts.is_empty() || starts.len() > 6 {
        return false;
    }
    let mut forwarded_starts = HashMap::new();
    for frame in forwarded
        .iter()
        .filter(|frame| frame["method"] == "turn/start")
    {
        if forwarded_starts
            .insert(frame["id"].to_string(), frame)
            .is_some()
        {
            return false;
        }
    }
    if forwarded_starts.len() != starts.len() {
        return false;
    }
    let mut users = HashMap::new();
    let mut user_ids = HashSet::new();
    for frame in server {
        if !matches!(
            frame["method"].as_str(),
            Some("item/started" | "item/completed")
        ) {
            continue;
        }
        let item = &frame["params"]["item"];
        if !matches!(
            item["type"].as_str(),
            Some("userMessage" | "agentMessage" | "reasoning")
        ) {
            return false;
        }
        if frame["method"] != "item/completed" || item["type"] != "userMessage" {
            continue;
        }
        let Some(thread) = frame["params"]["threadId"]
            .as_str()
            .filter(|id| !id.is_empty())
        else {
            return false;
        };
        let Some(turn) = frame["params"]["turnId"]
            .as_str()
            .filter(|id| !id.is_empty())
        else {
            return false;
        };
        let Some(id) = item["id"].as_str().filter(|id| !id.is_empty()) else {
            return false;
        };
        if !user_ids.insert((thread, id)) || users.insert((thread, turn), item).is_some() {
            return false;
        }
    }
    if users.len() != starts.len() {
        return false;
    }
    for start in starts {
        if forwarded_starts.get(&start["id"].to_string()).copied() != Some(start) {
            return false;
        }
        let Some(turn) = accepted_turn(start, server) else {
            return false;
        };
        let Some(thread) = start["params"]["threadId"].as_str() else {
            return false;
        };
        let Some(user) = users.remove(&(thread, turn)) else {
            return false;
        };
        let Some(input) = start["params"]["input"].as_array() else {
            return false;
        };
        let Some(content) = user["content"].as_array() else {
            return false;
        };
        if input.len() != 1
            || content.len() != 1
            || input[0]["type"] != "text"
            || content[0]["type"] != "text"
            || !input[0]["text"].is_string()
            || input[0]["text"] != content[0]["text"]
        {
            return false;
        }
    }
    users.is_empty()
}

fn terminal_settled(original: &[Value], forwarded: &[Value], server: &[Value]) -> bool {
    let mut latest = std::collections::HashMap::new();
    for frame in original
        .iter()
        .filter(|frame| frame["method"] == "turn/start")
    {
        let Some(thread) = frame["params"]["threadId"]
            .as_str()
            .filter(|id| !id.is_empty())
        else {
            return false;
        };
        latest.insert(thread, frame);
    }
    !latest.is_empty()
        && latest.len() <= 2
        && latest
            .values()
            .all(|start| turn_settled(start, original, forwarded, server))
}

fn turn_settled(start: &Value, original: &[Value], forwarded: &[Value], server: &[Value]) -> bool {
    let Some(thread) = start["params"]["threadId"]
        .as_str()
        .filter(|id| !id.is_empty())
    else {
        return false;
    };
    let Some(turn) = accepted_turn(start, server) else {
        return false;
    };
    let interrupts: Vec<_> = original
        .iter()
        .filter(|frame| {
            frame["method"] == "turn/interrupt"
                && frame["params"]["threadId"] == thread
                && frame["params"]["turnId"] == turn
        })
        .collect();
    if interrupts.len() > 1 {
        return false;
    }
    if let Some(interrupt) = interrupts.first()
        && (!matches!(
            interrupt.get("id"),
            Some(Value::String(_) | Value::Number(_))
        ) || !forwarded.contains(interrupt)
            || !server.iter().any(|reply| {
                reply.get("method").is_none()
                    && reply.get("id") == interrupt.get("id")
                    && reply.get("result").is_some()
                    && reply.get("error").is_none()
            }))
    {
        return false;
    }
    server.iter().any(|frame| {
        frame["method"] == "turn/completed"
            && frame["params"]["threadId"] == thread
            && frame["params"]["turn"]["id"] == turn
            && (frame["params"]["turn"]["status"] == "completed"
                || (!interrupts.is_empty() && frame["params"]["turn"]["status"] == "interrupted"))
    })
}

fn validate_preconditions(
    config: &Value,
    lookup: impl Fn(&str) -> Option<String>,
) -> io::Result<()> {
    let marker = config["primary_invocation_marker"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| io::Error::other("missing primary marker"))?;
    if config["condition"] != "work-leaf"
        || lookup("WORK_LEAF_OBSERVER_PRIMARY_MARKER").as_deref() != Some(marker)
        || lookup("WORK_LEAF_OBSERVER_PARENT_INVOCATION").is_some()
    {
        return Err(io::Error::other(
            "not a primary work-leaf observer invocation",
        ));
    }
    for (name, value) in [
        ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "1"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "1000"),
        (
            "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
            "forward",
        ),
        ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "1"),
    ] {
        if lookup(name).as_deref() != Some(value) {
            return Err(io::Error::other(format!(
                "missing exact diagnostic setting {name}"
            )));
        }
    }
    Ok(())
}

fn required_path(name: &str, directory: bool) -> PathBuf {
    let path = PathBuf::from(env::var_os(name).unwrap_or_else(|| panic!("missing {name}")));
    assert!(
        path.is_absolute()
            && if directory {
                path.is_dir()
            } else {
                path.is_file()
            },
        "invalid {name}"
    );
    path
}

fn prepare_fixture(root: &Path, read: bool) -> String {
    assert!(
        fs::read_dir(root).unwrap().next().is_none(),
        "diagnostic checkout must be empty"
    );
    let content = if read {
        (0..500)
            .map(|index| format!("pub const ITEM_{index:04}: usize = {index};\n"))
            .collect()
    } else {
        "pub const VALUE: u8 = 0;\n".to_string()
    };
    fs::write(root.join("fixture.rs"), &content).unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Candidate Diagnostic"],
        vec!["config", "user.email", "candidate@example.invalid"],
        vec!["add", "fixture.rs"],
        vec![
            "commit",
            "-qm",
            "ADD diagnostic fixture for bounded real handoffs",
        ],
    ] {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    content
}

fn scenario_prompt(condition: &str) -> String {
    let rules = "This is an explicitly bounded protocol diagnostic, not a feature benchmark or token-saving assertion. Use no built-in tools, extra reads, commands, agents, prose, or fences. Follow only the declared responses below. The fixture's full initial text for patch diagnostics is `pub const VALUE: u8 = 0;` followed by a newline. No repository validation command is required for this diagnostic.";
    match condition {
        "requested-repeat-full" => format!(
            "{rules} First emit exactly:\n{READ}\nAfter the first file response, do not open any bundle; emit exactly:\n{REPEAT}\nAfter the second file response emit exactly:\n{DONE}"
        ),
        "unified-diff-preferred" => format!(
            "{rules} Emit exactly this valid unified diff:\n{PATCH_ONE}\nAfter the patch acknowledgement emit exactly:\n{DONE}"
        ),
        "review-fix-request-resupply" => format!(
            "{rules} This scenario intentionally submits one incomplete fixture increment so a real reviewer can identify its visible defect. The final requirement is VALUE = 2, not 1. Initially emit exactly:\n{EDIT_ONE}\nAfter the acknowledgement emit exactly:\n{DONE}\nWhen review findings arrive, emit exactly:\n{EDIT_TWO}\nAfter its acknowledgement emit exactly:\n{DONE}\nFor the reviewer: assess the final VALUE = 2 requirement, not whether the diagnostic intentionally produced the defect. The first reviewed diff sets VALUE to 1, so report FINDINGS followed by that concrete mismatch. The corrected diff sets VALUE to 2, so reply exactly NO_FINDINGS. Use the complete supplied diff and source context without tools or additional reads."
        ),
        _ => unreachable!(),
    }
}

fn run_scenario(
    backend: CodexBackend,
    root: &Path,
    condition: &str,
) -> Result<(usize, String), AgentError> {
    let prompt = scenario_prompt(condition);
    let mut chat = CommandChat::new(
        root.to_path_buf(),
        BoundedBackend {
            inner: backend,
            budget: Budget::new(condition)?,
        },
    )
    .with_max_review_rounds(3);
    chat.launch_prepared_agent_streaming(
        AgentLaunch::new(
            AgentId::new("user-candidate")?,
            AgentKind::Codex,
            "candidate diagnostic",
            &prompt,
        ),
        &mut |_| {},
    )
    .map_err(|error| AgentError::Io(io::Error::other(error.to_string())))?;
    if condition == "review-fix-request-resupply" {
        let result = chat
            .handle_line("review")
            .map_err(|error| AgentError::Io(io::Error::other(error.to_string())))?;
        let CommandChatResult::ReviewComplete(reviews) = result else {
            return Err(rejected());
        };
        if reviews.len() != 1 || !reviews[0].findings_resolved || reviews[0].rounds != 2 {
            return Err(rejected());
        }
    }
    let calls = chat.into_backend().budget.calls;
    Ok((calls, prompt))
}

fn match_trace_requests(traces: &[Value], starts: &[&Value]) -> Result<Vec<usize>, ()> {
    use std::collections::{HashMap, HashSet, VecDeque};
    if starts.len() > 6 || traces.len() > 6 {
        return Err(());
    }
    let mut by_text: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut by_thread_text: HashMap<(&str, &str), VecDeque<usize>> = HashMap::new();
    for (index, start) in starts.iter().enumerate() {
        let text = start["params"]["input"][0]["text"].as_str().ok_or(())?;
        let thread = start["params"]["threadId"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or(())?;
        by_text.entry(text).or_default().push(index);
        by_thread_text
            .entry((thread, text))
            .or_default()
            .push_back(index);
    }
    let mut owners = HashMap::new();
    let mut last = HashMap::new();
    let mut consumed = HashSet::new();
    let mut links = Vec::new();
    for row in traces {
        let selected = match row["event"].as_str() {
            Some("candidate-prompt") => match row["selected_candidate"].as_str() {
                Some("candidate") => row["candidate_prompt"].as_str(),
                Some("baseline") => row["original_prompt"].as_str(),
                _ => None,
            },
            Some("prompt") => row["forwarded_prompt"].as_str(),
            _ => None,
        }
        .ok_or(())?;
        let agent = row["agent_id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or(())?;
        if row["site"] == "candidate-policy" {
            let candidates = by_text.get(selected).ok_or(())?;
            if candidates.len() != 1 {
                return Err(());
            }
            let thread = starts[candidates[0]]["params"]["threadId"]
                .as_str()
                .ok_or(())?;
            if owners.insert(agent, thread).is_some() {
                return Err(());
            }
        }
        let thread = *owners.get(agent).ok_or(())?;
        let index = by_thread_text
            .get_mut(&(thread, selected))
            .and_then(VecDeque::pop_front)
            .ok_or(())?;
        if !consumed.insert(index) || last.get(agent).is_some_and(|previous| *previous >= index) {
            return Err(());
        }
        last.insert(agent, index);
        links.push(index);
    }
    Ok(links)
}

fn verify_capture_and_trace(
    capture: &Path,
    manifest: &Value,
    calls: usize,
    original_request: &str,
    expected_fixture: &str,
) {
    let client = closed_frames(&capture.join("client-to-server.raw")).unwrap();
    let forwarded = closed_frames(&capture.join("client-to-server.forwarded.raw")).unwrap();
    let server = closed_frames(&capture.join("server-to-client.raw")).unwrap();
    assert!(
        public_delivery(&client, &forwarded, &server),
        "forwarded/native public user delivery or no-native-tool guard failed"
    );
    let starts: Vec<_> = client
        .iter()
        .filter(|frame| frame["method"] == "turn/start")
        .collect();
    assert_eq!(starts.len(), calls);
    let sessions: Vec<_> = client
        .iter()
        .filter(|frame| frame["method"] == "thread/start")
        .collect();
    assert_eq!(
        sessions.len(),
        if manifest["condition"] == "review-fix-request-resupply" {
            2
        } else {
            1
        }
    );
    assert!(
        client
            .iter()
            .all(|frame| frame["method"] != "thread/resume")
    );
    for session in sessions {
        let replies: Vec<_> = server
            .iter()
            .filter(|reply| reply.get("method").is_none() && reply.get("id") == session.get("id"))
            .collect();
        assert_eq!(replies.len(), 1);
        assert!(replies[0].get("error").is_none());
        assert!(
            replies[0]["result"]["thread"]["id"]
                .as_str()
                .is_some_and(|id| !id.is_empty())
        );
    }
    assert!(
        starts
            .iter()
            .all(|start| accepted_turn(start, &server).is_some())
    );
    let rows = closed_frames(Path::new(manifest["evidence_path"].as_str().unwrap())).unwrap();
    assert_eq!(rows[0]["event"], "activation");
    let expected_sites = match manifest["condition"].as_str().unwrap() {
        "requested-repeat-full" => vec![
            "candidate-policy",
            "requested-repeat-read",
            "requested-repeat-read",
        ],
        "unified-diff-preferred" => vec!["candidate-policy", "patch-applied"],
        "review-fix-request-resupply" => vec![
            "candidate-policy",
            "patch-applied",
            "candidate-policy",
            "review-fix-request",
            "patch-applied",
        ],
        _ => panic!("unsupported diagnostic condition"),
    };
    assert_eq!(
        rows[1..]
            .iter()
            .map(|row| row["site"].as_str().unwrap())
            .collect::<Vec<_>>(),
        expected_sites
    );
    let links = match_trace_requests(&rows[1..], &starts)
        .expect("trace requests must match one-to-one in each agent thread's occurrence order");
    assert_eq!(
        links,
        (0..expected_sites.len()).collect::<Vec<_>>(),
        "all planned instrumented starts must be covered; only final reviewer recheck is uninstrumented"
    );
    let mut read_rows = 0;
    let mut fixes = 0;
    for row in &rows[1..] {
        assert_eq!(row["schema"], "work-leaf-bench-experiment-v4");
        let selected = if row["event"] == "candidate-prompt" {
            &row[if row["selected_candidate"] == "candidate" {
                "candidate_prompt"
            } else {
                "original_prompt"
            }]
        } else {
            &row["forwarded_prompt"]
        };
        match row["site"].as_str().unwrap() {
            "candidate-policy" => assert_eq!(
                row["changed"],
                manifest["condition"] != "review-fix-request-resupply"
            ),
            "requested-repeat-read" => {
                assert_eq!(manifest["condition"], "requested-repeat-full");
                assert_eq!(row["changed"], read_rows == 1);
                if read_rows == 1 {
                    let snapshots = row["metadata"]["snapshots"].as_array().unwrap();
                    assert_eq!(snapshots.len(), 1);
                    let snapshot = &snapshots[0];
                    assert_eq!(snapshot["path"], "fixture.rs");
                    assert_eq!(snapshot["class"], "unchanged");
                    let start = snapshot["inline_body_start"].as_u64().unwrap() as usize;
                    let end = snapshot["inline_body_end"].as_u64().unwrap() as usize;
                    assert_eq!(&selected.as_str().unwrap()[start..end], expected_fixture);
                    assert_eq!(snapshot["bytes"], expected_fixture.len());
                }
                read_rows += 1;
            }
            "review-fix-request" => {
                assert_eq!(manifest["condition"], "review-fix-request-resupply");
                assert_eq!(row["changed"], true);
                let candidate = row["candidate_prompt"].as_str().unwrap();
                let start = row["metadata"]["candidate_request_start"].as_u64().unwrap() as usize;
                let end = row["metadata"]["candidate_request_end"].as_u64().unwrap() as usize;
                assert_eq!(&candidate[start..end], original_request);
                fixes += 1;
            }
            "patch-applied" => assert_eq!(row["original_prompt"], row["forwarded_prompt"]),
            site => panic!("unexpected diagnostic boundary {site}"),
        }
    }
    assert_eq!(
        read_rows,
        if manifest["condition"] == "requested-repeat-full" {
            2
        } else {
            0
        }
    );
    assert_eq!(
        fixes,
        usize::from(manifest["condition"] == "review-fix-request-resupply")
    );
}

#[test]
#[ignore = "requires separately admitted real subscription diagnostic; maximum six provider turns"]
fn real_subscription_candidate_handoffs() {
    assert_eq!(
        env::var("WORK_LEAF_REAL_BENCH_CANDIDATE_SMOKE").as_deref(),
        Ok("1")
    );
    assert_eq!(env::var("WORK_LEAF_BENCH_EXPERIMENT").as_deref(), Ok("1"));
    assert!(env::var_os("WORK_LEAF_CODEX_TRACE").is_none());
    let manifest: Value = serde_json::from_slice(
        &fs::read(required_path("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", false)).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["schema"], "work-leaf-bench-experiment-v4");
    assert_eq!(
        manifest["run_id"].as_str(),
        env::var("WORK_LEAF_BENCH_RUN_ID").ok().as_deref()
    );
    let condition = manifest["condition"].as_str().unwrap();
    Budget::new(condition).unwrap();
    let root = required_path("WORK_LEAF_REAL_OBSERVER_PROJECT_DIR", true);
    let proxy = required_path("WORK_LEAF_REAL_OBSERVER_CODEX_PROXY", false);
    let observer = required_path("WORK_LEAF_REAL_OBSERVER_BIN", false);
    let config_path = required_path("WORK_LEAF_OBSERVER_CONFIG", false);
    let config: Value = serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
    validate_preconditions(&config, |key| env::var(key).ok()).unwrap();
    let initial = prepare_fixture(&root, condition == "requested-repeat-full");
    let mut backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone())
            .with_binary(proxy)
            .with_model("gpt-5.5")
            .with_sandbox(SandboxMode::ReadOnly),
        PromptPolicy::for_project(&root).unwrap(),
    );
    let shutdown = backend.shutdown_handle();
    let (tx, rx) = mpsc::channel();
    let watchdog = thread::spawn(move || {
        if rx.recv_timeout(Duration::from_secs(118)).is_err() {
            shutdown.shutdown();
            eprintln!("candidate diagnostic exceeded120second bound");
            process::exit(1);
        }
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_scenario(backend.clone(), &root, condition)
    }));
    let captures: Vec<_> =
        fs::read_dir(Path::new(config["root"].as_str().unwrap()).join("app-server"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect();
    assert_eq!(captures.len(), 1);
    let capture = &captures[0];
    let deadline = Instant::now() + Duration::from_secs(15);
    let settled = loop {
        let client = frames(&capture.join("client-to-server.raw")).unwrap_or_default();
        let forwarded = frames(&capture.join("client-to-server.forwarded.raw")).unwrap_or_default();
        let server = frames(&capture.join("server-to-client.raw")).unwrap_or_default();
        if terminal_settled(&client, &forwarded, &server) {
            break true;
        }
        if Instant::now() >= deadline {
            break false;
        }
        thread::sleep(Duration::from_millis(25));
    };
    let stopped = Command::new("timeout")
        .args(["--kill-after=1s", "10s"])
        .arg(observer)
        .args(["stop-app-server", "--config"])
        .arg(config_path)
        .stdin(Stdio::null())
        .output();
    backend.shutdown();
    let _ = tx.send(());
    watchdog.join().unwrap();
    assert!(
        settled,
        "terminal forwarding/accepted-turn receipt incomplete"
    );
    let stopped = stopped.unwrap();
    assert!(
        stopped.status.success() && stopped.stdout == b"1\n",
        "observer capture did not close"
    );
    let (calls, prompt) = result
        .expect("diagnostic panicked")
        .expect("diagnostic failed");
    let expected_calls = match condition {
        "requested-repeat-full" => 3,
        "unified-diff-preferred" => 2,
        _ => 6,
    };
    assert_eq!(calls, expected_calls);
    let expected_content = match condition {
        "requested-repeat-full" => initial,
        "unified-diff-preferred" => "pub const VALUE: u8 = 1;\n".to_string(),
        _ => "pub const VALUE: u8 = 2;\n".to_string(),
    };
    assert_eq!(
        fs::read_to_string(root.join("fixture.rs")).unwrap(),
        expected_content
    );
    verify_capture_and_trace(capture, &manifest, calls, &prompt, &expected_content);
    println!(
        "WORK_LEAF_CANDIDATE_SMOKE_OK condition={condition} turns={calls} sessions={}",
        if condition == "review-fix-request-resupply" {
            2
        } else {
            1
        }
    );
}

#[test]
#[ignore = "provider-free replay of one explicitly selected closed diagnostic; creates no files"]
fn replay_closed_candidate_handoffs() {
    assert_eq!(
        env::var("WORK_LEAF_CANDIDATE_OFFLINE_REPLAY").as_deref(),
        Ok("1")
    );
    let root = required_path("WORK_LEAF_CANDIDATE_REPLAY_ROOT", true);
    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("experiment.json")).unwrap()).unwrap();
    assert_eq!(manifest["schema"], "work-leaf-bench-experiment-v4");
    let condition = manifest["condition"].as_str().unwrap();
    Budget::new(condition).unwrap();
    let config: Value =
        serde_json::from_slice(&fs::read(root.join("observation/observer-config.json")).unwrap())
            .unwrap();
    assert_eq!(config["run_id"], manifest["run_id"]);
    let observation = PathBuf::from(config["root"].as_str().unwrap());
    assert_eq!(
        observation.canonicalize().unwrap(),
        root.join("observation").canonicalize().unwrap()
    );
    let captures: Vec<_> = fs::read_dir(observation.join("app-server"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    assert_eq!(captures.len(), 1);
    let capture = &captures[0];
    let end: Value = serde_json::from_slice(
        &fs::read(
            observation
                .join("invocations")
                .join(capture.file_name().unwrap())
                .join("end.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(end["exit_code"], 0);
    let client = closed_frames(&capture.join("client-to-server.raw")).unwrap();
    let forwarded = closed_frames(&capture.join("client-to-server.forwarded.raw")).unwrap();
    let server = closed_frames(&capture.join("server-to-client.raw")).unwrap();
    assert!(terminal_settled(&client, &forwarded, &server));
    let calls = match condition {
        "requested-repeat-full" => 3,
        "unified-diff-preferred" => 2,
        _ => 6,
    };
    let expected_fixture = match condition {
        "requested-repeat-full" => (0..500)
            .map(|index| format!("pub const ITEM_{index:04}: usize = {index};\n"))
            .collect(),
        "unified-diff-preferred" => "pub const VALUE: u8 = 1;\n".to_string(),
        _ => "pub const VALUE: u8 = 2;\n".to_string(),
    };
    verify_capture_and_trace(
        capture,
        &manifest,
        calls,
        &scenario_prompt(condition),
        &expected_fixture,
    );
    println!(
        "WORK_LEAF_CANDIDATE_OFFLINE_REPLAY_OK condition={condition} turns={calls}; original process outcome retained; native rollout/accounting audit separate"
    );
}

#[test]
fn candidate_smoke_observer_guard_requires_exact_primary_settings() {
    let config = json!({"condition":"work-leaf","primary_invocation_marker":"marker"});
    let values = [
        ("WORK_LEAF_OBSERVER_PRIMARY_MARKER", "marker"),
        ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "1"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "1000"),
        (
            "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
            "forward",
        ),
        ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "1"),
    ];
    let lookup = |key: &str| {
        values
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| value.to_string())
    };
    assert!(validate_preconditions(&config, lookup).is_ok());
    for (omitted, _) in values {
        assert!(
            validate_preconditions(&config, |key| if key == omitted {
                None
            } else {
                lookup(key)
            })
            .is_err()
        );
    }
    assert!(
        validate_preconditions(&config, |key| {
            if key == "WORK_LEAF_OBSERVER_PARENT_INVOCATION" {
                Some("parent".to_string())
            } else {
                lookup(key)
            }
        })
        .is_err()
    );
}

#[test]
fn candidate_smoke_budget_rejects_extra_calls_and_wrong_handoffs() {
    for (condition, count) in [
        ("requested-repeat-full", 3),
        ("unified-diff-preferred", 2),
        ("review-fix-request-resupply", 6),
    ] {
        let mut budget = Budget::new(condition).unwrap();
        assert!(budget.admit("other", false, "unplanned").is_err());
        assert_eq!(budget.calls, 0);
        for index in 0..count {
            let (reviewer, launch, prefix) = expected_step(condition, index).unwrap();
            let id = if reviewer {
                "review-user-candidate"
            } else {
                "user-candidate"
            };
            assert!(budget.admit(id, !launch, prefix).is_err());
            assert!(budget.admit(id, launch, prefix).is_ok());
        }
        assert!(
            budget
                .admit("user-candidate", false, "work-leaf file text\n")
                .is_err()
        );
        assert_eq!(budget.calls, count);
    }
    assert!(Budget::new("control").is_err());
}

#[test]
fn candidate_smoke_terminal_receipt_rejects_ambiguous_delivery() {
    let original =
        vec![serde_json::json!({"id":"1","method":"turn/start","params":{"threadId":"t"}})];
    let reply = serde_json::json!({"id":"1","result":{"turn":{"id":"v"}}});
    let done = serde_json::json!({"method":"turn/completed","params":{"threadId":"t","turn":{"id":"v","status":"completed"}}});
    assert!(terminal_settled(
        &original,
        &original,
        &[reply.clone(), done.clone()]
    ));
    let mut ambiguous = reply;
    ambiguous["error"] = serde_json::json!({"code":-1});
    assert!(!terminal_settled(&original, &original, &[ambiguous, done]));
    let mut queued = original.clone();
    queued.push(serde_json::json!({"id":"2","method":"turn/interrupt","params":{"threadId":"t","turnId":"v"}}));
    assert!(!terminal_settled(&queued, &original, &[]));
}

#[test]
fn candidate_smoke_public_capture_rejects_tools_or_unmatched_user_delivery() {
    let start = json!({"id":"1","method":"turn/start","params":{"threadId":"t","input":[{"type":"text","text":"literal input"}]}});
    let reply = json!({"id":"1","result":{"turn":{"id":"v"}}});
    let user = json!({"method":"item/completed","params":{"threadId":"t","turnId":"v","item":{"type":"userMessage","id":"user-id","content":[{"type":"text","text":"literal input"}]}}});
    let mut server = vec![reply, user];
    let original = std::slice::from_ref(&start);
    assert!(public_delivery(original, original, &server));
    assert!(!public_delivery(original, &[], &server));
    server[1]["params"]["item"]["content"][0]["text"] = json!("different");
    assert!(!public_delivery(original, original, &server));
    server[1]["params"]["item"]["content"][0]["text"] = json!("literal input");
    for kind in [
        "agentMessage",
        "reasoning",
        "commandExecution",
        "fileChange",
        "mcpToolCall",
        "webSearch",
        "unknown",
    ] {
        server.push(json!({"method":"item/started","params":{"threadId":"t","turnId":"v","item":{"type":kind,"id":"action"}}}));
        assert_eq!(
            public_delivery(original, original, &server),
            matches!(kind, "agentMessage" | "reasoning")
        );
        server.pop();
    }
    let duplicate = server[1].clone();
    server.push(duplicate);
    assert!(!public_delivery(original, original, &server));
}

#[test]
fn candidate_smoke_waits_for_each_threads_last_accepted_turn() {
    let original = vec![
        json!({"id":"1","method":"turn/start","params":{"threadId":"author"}}),
        json!({"id":"2","method":"turn/start","params":{"threadId":"reviewer"}}),
    ];
    let mut server = vec![
        json!({"id":"1","result":{"turn":{"id":"author-final"}}}),
        json!({"id":"2","result":{"turn":{"id":"reviewer-final"}}}),
        json!({"method":"turn/completed","params":{"threadId":"reviewer","turn":{"id":"reviewer-final","status":"completed"}}}),
    ];
    assert!(!terminal_settled(&original, &original, &server));
    server.push(json!({"method":"turn/completed","params":{"threadId":"author","turn":{"id":"author-final","status":"completed"}}}));
    assert!(terminal_settled(&original, &original, &server));
}

#[test]
fn candidate_smoke_rejects_reused_accepted_turn_with_unmatched_user_item() {
    let first = json!({"id":"1","method":"turn/start","params":{"threadId":"t","input":[{"type":"text","text":"literal input"}]}});
    let mut second = first.clone();
    second["id"] = json!("2");
    let original = [first, second];
    let server = [
        json!({"id":"1","result":{"turn":{"id":"reused"}}}),
        json!({"id":"2","result":{"turn":{"id":"reused"}}}),
        json!({"method":"item/completed","params":{"threadId":"t","turnId":"reused","item":{"type":"userMessage","id":"user-1","content":[{"type":"text","text":"literal input"}]}}}),
        json!({"method":"item/completed","params":{"threadId":"t","turnId":"unmatched","item":{"type":"userMessage","id":"user-2","content":[{"type":"text","text":"literal input"}]}}}),
    ];
    assert!(!public_delivery(&original, &original, &server));
}

#[test]
fn candidate_smoke_links_identical_acks_once_in_same_agent_turn_order() {
    let starts = vec![
        json!({"params":{"threadId":"author-thread","input":[{"text":"policy"}]}}),
        json!({"params":{"threadId":"author-thread","input":[{"text":"identical ACK"}]}}),
        json!({"params":{"threadId":"author-thread","input":[{"text":"fix request"}]}}),
        json!({"params":{"threadId":"author-thread","input":[{"text":"identical ACK"}]}}),
    ];
    let traces = vec![
        json!({"event":"candidate-prompt","site":"candidate-policy","agent_id":"author","selected_candidate":"candidate","candidate_prompt":"policy"}),
        json!({"event":"prompt","site":"patch-applied","agent_id":"author","forwarded_prompt":"identical ACK"}),
        json!({"event":"candidate-prompt","site":"review-fix-request","agent_id":"author","selected_candidate":"candidate","candidate_prompt":"fix request"}),
        json!({"event":"prompt","site":"patch-applied","agent_id":"author","forwarded_prompt":"identical ACK"}),
    ];
    let refs: Vec<_> = starts.iter().collect();
    assert_eq!(
        match_trace_requests(&traces, &refs).unwrap(),
        vec![0, 1, 2, 3]
    );
    let mut extra = traces.clone();
    extra.push(traces[3].clone());
    assert!(match_trace_requests(&extra, &refs).is_err());
    let mut wrong = starts.clone();
    wrong[3]["params"]["threadId"] = json!("different-thread");
    assert!(match_trace_requests(&traces, &wrong.iter().collect::<Vec<_>>()).is_err());
    let mut reordered = traces;
    reordered.swap(2, 3);
    assert!(match_trace_requests(&reordered, &refs).is_err());
}
