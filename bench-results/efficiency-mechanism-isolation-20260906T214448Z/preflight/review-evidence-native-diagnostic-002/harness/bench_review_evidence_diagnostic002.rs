#![cfg(feature = "bench-experiments")]

use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentSession, AgentShutdownHandle,
    AgentStreamEvent, ChatMessage, CodexBackend, CodexCommandConfig, CommandChat,
    CommandChatResult, PromptPolicy, SandboxMode,
};

const AUTHOR: &str = "user-review-evidence";
const REVIEWER: &str = "review-user-review-evidence";
const MAX_CALLS: usize = 6;
const INITIAL: &str = "pub const VALUE: u8 = 0;\n";
const FINAL: &str = "pub const VALUE: u8 = 1;\n";
const EDIT: &str = "@work-leaf edit set diagnostic value\n*** Begin Patch\n*** Update File: fixture.rs\n@@\n-pub const VALUE: u8 = 0;\n+pub const VALUE: u8 = 1;\n*** End Patch\n@work-leaf end";
const PUBLIC_DONE: &str = "C21_PUBLIC_EVIDENCE: I used the supplied initial fixture text and submitted only the VALUE=1 edit; I ran no native tool or verification command.\n@work-leaf done";
const CHECK: &str = "@work-leaf locks run fixture.rs -- sh ./check-fixture.sh";
const CHECK_SCRIPT: &str = "#!/bin/sh\nset -eu\ngrep -Fxq 'pub const VALUE: u8 = 1;' fixture.rs\nprintf '%s\\n' C21_CHECK_OK\n";
const FEATURE: &str = "C21 opaque-evidence diagnostic: set fixture VALUE to 1. Reviewer instructions for this bounded diagnostic are an ordered two-turn qualification. Reviewer turn 1 (mandatory even if the edit is obviously correct): use exactly one native exec_command call to read the complete issued opaque evidence path, with cmd cat -- '<the exact supplied path>', login=false, max_output_tokens=20000. Use no other native tool, agent, poll or native command. Read the complete returned context including the author's non-Git public evidence. Do not return NO_FINDINGS on this turn. After that full read, your entire public reply must be exactly:\n@work-leaf locks run fixture.rs -- sh ./check-fixture.sh\nThe native-tool limit does not prohibit this required Work Leaf locked check. This preexisting focused script is the only required check; do not create a check or run broad checks. Wait for the actual command result in the next reviewer turn; do not infer success from the edit or archive. Reviewer turn 2, only after the actual Work Leaf command result reports status: 0 and C21_CHECK_OK: reply exactly NO_FINDINGS if the requested VALUE=1 edit and public evidence are present. No commentary, fences, extra reads or fabricated verdict.";

fn scenario_prompt() -> String {
    format!(
        "C21 bounded diagnostic, not a token-saving assertion. Set VALUE to 1. The complete initial fixture.rs is `{}` followed by a newline. Use no native tools, additional reads, commands, agents, prose or fences. Emit exactly:\n{EDIT}\nAfter the actual patch acknowledgement emit exactly:\n{PUBLIC_DONE}",
        INITIAL.trim_end()
    )
}

fn frame_bytes(bytes: &[u8], closed: bool) -> io::Result<Vec<Value>> {
    if closed && !bytes.is_empty() && !bytes.ends_with(b"\n") {
        return Err(invalid());
    }
    bytes
        .split_inclusive(|b| *b == b'\n')
        .filter(|line| line.ends_with(b"\n"))
        .map(|line| serde_json::from_slice(line).map_err(io::Error::other))
        .collect()
}

fn string(value: &Value) -> io::Result<&str> {
    value
        .as_str()
        .filter(|text| !text.is_empty())
        .ok_or_else(invalid)
}

fn starts(client: &[Value]) -> Vec<&Value> {
    client
        .iter()
        .filter(|v| v["method"] == "turn/start")
        .collect()
}

fn accepted_keys<'a>(
    client: &'a [Value],
    server: &'a [Value],
) -> io::Result<Vec<(&'a str, &'a str)>> {
    let mut replies = HashMap::new();
    for row in server
        .iter()
        .filter(|v| v.get("method").is_none() && v.get("id").is_some())
    {
        if replies.insert(string(&row["id"])?, row).is_some() {
            return Err(invalid());
        }
    }
    let inputs = starts(client);
    if inputs.is_empty() || inputs.len() > MAX_CALLS {
        return Err(invalid());
    }
    let mut unique = HashSet::new();
    let mut rpcs = HashSet::new();
    let mut keys = Vec::new();
    for row in inputs {
        let rpc = string(&row["id"])?;
        let reply = replies.get(rpc).ok_or_else(invalid)?;
        if !rpcs.insert(rpc) || reply.get("error").is_some() {
            return Err(invalid());
        }
        let key = (
            string(&row["params"]["threadId"])?,
            string(&reply["result"]["turn"]["id"])?,
        );
        if !unique.insert(key) {
            return Err(invalid());
        }
        keys.push(key);
    }
    Ok(keys)
}

fn terminal_settled(client: &[Value], forwarded: &[Value], server: &[Value]) -> io::Result<()> {
    let keys = accepted_keys(client, server)?;
    let latest: HashMap<_, _> = keys.into_iter().collect();
    if latest.len() > 2 {
        return Err(invalid());
    }
    let forwarded_ids: HashMap<_, _> = forwarded
        .iter()
        .filter_map(|v| v["id"].as_str().map(|id| (id, v)))
        .collect();
    let replies: HashMap<_, _> = server
        .iter()
        .filter(|v| v.get("method").is_none())
        .filter_map(|v| v["id"].as_str().map(|id| (id, v)))
        .collect();
    let mut interrupts = HashMap::new();
    for row in client.iter().filter(|v| v["method"] == "turn/interrupt") {
        let key = (
            string(&row["params"]["threadId"])?,
            string(&row["params"]["turnId"])?,
        );
        if latest.get(key.0).copied() != Some(key.1) {
            continue;
        }
        let rpc = string(&row["id"])?;
        let reply = replies.get(rpc).ok_or_else(invalid)?;
        if interrupts.insert(key, row).is_some()
            || forwarded_ids.get(rpc).copied() != Some(row)
            || reply.get("error").is_some()
            || reply.get("result").is_none()
        {
            return Err(invalid());
        }
    }
    let mut terminal = HashSet::new();
    for row in server.iter().filter(|v| v["method"] == "turn/completed") {
        let key = (
            string(&row["params"]["threadId"])?,
            string(&row["params"]["turn"]["id"])?,
        );
        if latest.get(key.0).copied() != Some(key.1) {
            continue;
        }
        let status = &row["params"]["turn"]["status"];
        if !terminal.insert(key)
            || !(status == "completed" || status == "interrupted" && interrupts.contains_key(&key))
        {
            return Err(invalid());
        }
    }
    if terminal.len() != latest.len() {
        return Err(invalid());
    }
    Ok(())
}

fn verify_public_capture(
    client: &[Value],
    forwarded: &[Value],
    server: &[Value],
    archive: &str,
    path: &str,
) -> io::Result<()> {
    terminal_settled(client, forwarded, server)?;
    let inputs = starts(client);
    let keys = accepted_keys(client, server)?;
    if keys.len() != 4 || keys[0].0 != keys[1].0 || keys[2].0 != keys[3].0 || keys[0].0 == keys[2].0
    {
        return Err(invalid());
    }
    let sent = starts(forwarded);
    if sent != inputs {
        return Err(invalid());
    }
    let mut users = HashMap::new();
    let mut user_ids = HashSet::new();
    let mut read_started = None;
    let mut read_completed = None;
    let mut author_evidence = false;
    for row in server.iter().filter(|v| {
        matches!(
            v["method"].as_str(),
            Some("item/started" | "item/completed")
        )
    }) {
        let p = &row["params"];
        let item = &p["item"];
        let kind = string(&item["type"])?;
        let key = (string(&p["threadId"])?, string(&p["turnId"])?);
        match kind {
            "userMessage" if row["method"] == "item/completed" => {
                if !user_ids.insert((key.0, string(&item["id"])?))
                    || users.insert(key, item).is_some()
                {
                    return Err(invalid());
                }
            }
            "commandExecution" => {
                if key != keys[2] {
                    return Err(invalid());
                }
                let id = string(&item["id"])?;
                if row["method"] == "item/started" {
                    if read_started.replace(id).is_some() {
                        return Err(invalid());
                    }
                } else {
                    if read_completed.replace(id).is_some()
                        || item["status"] != "completed"
                        || item["exitCode"] != 0
                        || item["aggregatedOutput"] != archive
                    {
                        return Err(invalid());
                    }
                    let actions = item["commandActions"].as_array().ok_or_else(invalid)?;
                    let command = format!("cat -- '{path}'");
                    let actual = string(&item["command"])?;
                    let plain_bash = ["/usr/bin/bash", "/bin/bash", "bash"]
                        .into_iter()
                        .any(|shell| actual == format!("{shell} -c \"{command}\""));
                    if actual != command && !plain_bash {
                        return Err(invalid());
                    }
                    if actions.len() != 1
                        || actions[0]["type"] != "read"
                        || actions[0]["path"] != path
                        || actions[0]["command"] != format!("cat -- '{path}'")
                    {
                        return Err(invalid());
                    }
                }
            }
            "agentMessage" => {
                if row["method"] == "item/completed"
                    && key == keys[1]
                    && item["text"] == PUBLIC_DONE
                {
                    author_evidence = true;
                }
            }
            "userMessage" | "reasoning" => {}
            _ => return Err(invalid()),
        }
    }
    if read_started.is_none()
        || read_started != read_completed
        || !author_evidence
        || users.len() != 4
    {
        return Err(invalid());
    }
    for (input, key) in inputs.iter().zip(keys) {
        let user = users.remove(&key).ok_or_else(invalid)?;
        let text = input["params"]["input"].as_array().ok_or_else(invalid)?;
        let content = user["content"].as_array().ok_or_else(invalid)?;
        if text.len() != 1
            || content.len() != 1
            || text[0]["type"] != "text"
            || content[0]["type"] != "text"
            || !text[0]["text"].is_string()
            || text[0]["text"] != content[0]["text"]
        {
            return Err(invalid());
        }
    }
    if !users.is_empty() {
        return Err(invalid());
    }
    Ok(())
}

fn public_fixture() -> (Vec<Value>, Vec<Value>) {
    let mut c = Vec::new();
    let mut s = Vec::new();
    for (index, thread) in ["a", "a", "r", "r"].into_iter().enumerate() {
        let rpc = format!("rpc{index}");
        let turn = format!("turn{index}");
        let text = format!("prompt{index}");
        c.push(json!({"id":rpc,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":text}]}}));
        s.push(json!({"id":rpc,"result":{"turn":{"id":turn}}}));
        s.push(json!({"method":"item/completed","params":{"threadId":thread,"turnId":turn,"item":{"type":"userMessage","id":format!("user{index}"),"content":[{"type":"text","text":text}]}}}));
        s.push(json!({"method":"turn/completed","params":{"threadId":thread,"turn":{"id":turn,"status":"completed"}}}));
    }
    s.push(json!({"method":"item/completed","params":{"threadId":"a","turnId":"turn1","item":{"type":"agentMessage","id":"done","text":PUBLIC_DONE}}}));
    let item = json!({"type":"commandExecution","id":"read","command":"/usr/bin/bash -c \"cat -- '/opaque/review.txt'\"","status":"completed","exitCode":0,"aggregatedOutput":"archive","commandActions":[{"type":"read","path":"/opaque/review.txt","command":"cat -- '/opaque/review.txt'"}]});
    for method in ["item/started", "item/completed"] {
        s.push(json!({"method":method,"params":{"threadId":"r","turnId":"turn2","item":item}}));
    }
    (c, s)
}

#[test]
fn review_smoke_public_capture_requires_exact_full_read_and_both_thread_terminals() {
    let (c, s) = public_fixture();
    assert!(verify_public_capture(&c, &c, &s, "archive", "/opaque/review.txt").is_ok());
    let mut missing = s.clone();
    missing.retain(|v| !(v["method"] == "turn/completed" && v["params"]["turn"]["id"] == "turn1"));
    assert!(verify_public_capture(&c, &c, &missing, "archive", "/opaque/review.txt").is_err());
    assert!(verify_public_capture(&c, &c, &s, "different archive", "/opaque/review.txt").is_err());
    let mut extra_command = s.clone();
    extra_command.last_mut().unwrap()["params"]["item"]["command"] =
        json!("/usr/bin/bash -c \"cat -- '/opaque/review.txt'; true\"");
    assert!(
        verify_public_capture(&c, &c, &extra_command, "archive", "/opaque/review.txt").is_err()
    );
    let mut extra = s.clone();
    extra.push(json!({"method":"item/started","params":{"item":{"type":"mcpToolCall"}}}));
    assert!(verify_public_capture(&c, &c, &extra, "archive", "/opaque/review.txt").is_err());
    let mut alias = s.clone();
    alias[3]["result"]["turn"]["id"] = json!("turn0");
    assert!(verify_public_capture(&c, &c, &alias, "archive", "/opaque/review.txt").is_err());
    let mut mixed = s.clone();
    mixed[0]["error"] = json!({"code":-1});
    assert!(verify_public_capture(&c, &c, &mixed, "archive", "/opaque/review.txt").is_err());
    assert!(frame_bytes(b"{}\n{", true).is_err());
    assert_eq!(frame_bytes(b"{}\n{", false).unwrap().len(), 1);
}

fn invalid() -> io::Error {
    io::Error::other(
        "review-evidence diagnostic rejected an unplanned setting, identity, or action",
    )
}

fn capture_closed(success: bool, stdout: &[u8], stderr: &[u8]) -> io::Result<()> {
    if success && stdout == b"1\n" {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "observer capture did not close; success={success}; stdout={} stderr={}",
            String::from_utf8_lossy(stdout),
            String::from_utf8_lossy(stderr)
        )))
    }
}

#[test]
fn review_smoke_capture_close_failure_retains_operator_diagnostic() {
    assert!(capture_closed(true, b"1\n", b"").is_ok());
    let error = capture_closed(false, b"0\n", b"retained stop diagnostic").unwrap_err();
    assert!(error.to_string().contains("retained stop diagnostic"));
}

fn validate_preconditions(
    config: &Value,
    manifest: &Value,
    lookup: impl Fn(&str) -> Option<String>,
) -> io::Result<()> {
    let marker = config["primary_invocation_marker"]
        .as_str()
        .filter(|v| !v.is_empty())
        .ok_or_else(invalid)?;
    let run = manifest["run_id"]
        .as_str()
        .filter(|v| !v.is_empty())
        .ok_or_else(invalid)?;
    if config["condition"] != "work-leaf"
        || config["run_id"] != run
        || config["model"] != "gpt-5.5"
        || config["effort"] != "xhigh"
        || manifest["schema"] != "work-leaf-bench-experiment-v5"
        || manifest["condition"] != "review-evidence-native"
    {
        return Err(invalid());
    }
    for (name, value) in [
        ("WORK_LEAF_OBSERVER_PRIMARY_MARKER", marker),
        ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "1"),
        ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "1000"),
        (
            "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
            "forward",
        ),
        ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "1"),
        ("WORK_LEAF_BENCH_EXPERIMENT", "1"),
        ("WORK_LEAF_BENCH_RUN_ID", run),
        ("WORK_LEAF_REAL_REVIEW_EVIDENCE_SMOKE", "1"),
    ] {
        if lookup(name).as_deref() != Some(value) {
            return Err(invalid());
        }
    }
    for name in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "OPENAI_BASE_URL",
        "OPENAI_API_BASE",
        "CODEX_BASE_URL",
        "CODEX_ACCESS_TOKEN",
        "WORK_LEAF_OBSERVER_PARENT_INVOCATION",
        "WORK_LEAF_CODEX_TRACE",
    ] {
        if lookup(name).is_some() {
            return Err(invalid());
        }
    }
    Ok(())
}

#[derive(Default)]
struct Budget {
    calls: usize,
}
impl Budget {
    fn admit(&mut self, id: &str, launch: bool, prompt: &str) -> io::Result<()> {
        if self.calls >= MAX_CALLS {
            return Err(invalid());
        }
        let expected = match self.calls {
            0 => (AUTHOR, true, "C21 bounded diagnostic"),
            1 => (AUTHOR, false, "work-leaf patch applied\n"),
            2 => (REVIEWER, true, "Review the full patch scope"),
            3 => (REVIEWER, false, "work-leaf command result\n"),
            _ => return Err(invalid()),
        };
        if id != expected.0 || launch != expected.1 || !prompt.starts_with(expected.2) {
            return Err(invalid());
        }
        self.calls += 1;
        Ok(())
    }
}

fn guard_fixture() -> (Value, Value, Vec<(&'static str, &'static str)>) {
    (
        json!({"condition":"work-leaf","run_id":"diagnostic","primary_invocation_marker":"marker","model":"gpt-5.5","effort":"xhigh"}),
        json!({"schema":"work-leaf-bench-experiment-v5","condition":"review-evidence-native","run_id":"diagnostic"}),
        vec![
            ("WORK_LEAF_OBSERVER_PRIMARY_MARKER", "marker"),
            ("WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE", "1"),
            ("WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS", "1000"),
            (
                "WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME",
                "forward",
            ),
            ("WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY", "1"),
            ("WORK_LEAF_BENCH_EXPERIMENT", "1"),
            ("WORK_LEAF_BENCH_RUN_ID", "diagnostic"),
            ("WORK_LEAF_REAL_REVIEW_EVIDENCE_SMOKE", "1"),
        ],
    )
}

#[test]
fn review_smoke_guard_requires_exact_observer_identity_and_no_auth_override() {
    let (config, manifest, values) = guard_fixture();
    let lookup = |key: &str| {
        values
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, v)| v.to_string())
    };
    assert!(validate_preconditions(&config, &manifest, lookup).is_ok());
    for (missing, _) in &values {
        assert!(
            validate_preconditions(&config, &manifest, |key| if key == *missing {
                None
            } else {
                lookup(key)
            })
            .is_err(),
            "missing {missing}"
        );
    }
    for forbidden in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "OPENAI_BASE_URL",
        "OPENAI_API_BASE",
        "CODEX_BASE_URL",
        "CODEX_ACCESS_TOKEN",
        "WORK_LEAF_OBSERVER_PARENT_INVOCATION",
        "WORK_LEAF_CODEX_TRACE",
    ] {
        assert!(
            validate_preconditions(&config, &manifest, |key| if key == forbidden {
                Some("forbidden".to_string())
            } else {
                lookup(key)
            })
            .is_err(),
            "forbidden {forbidden}"
        );
    }
    for (key, value) in [
        ("condition", json!("direct")),
        ("run_id", json!("other")),
        ("primary_invocation_marker", json!("")),
    ] {
        let mut bad = config.clone();
        bad[key] = value;
        assert!(validate_preconditions(&bad, &manifest, lookup).is_err());
    }
    let mut bad = manifest.clone();
    bad["condition"] = json!("review-evidence-inline");
    assert!(validate_preconditions(&config, &bad, lookup).is_err());
}

#[test]
fn review_smoke_budget_requires_owned_four_step_path_and_hard_six_ceiling() {
    let mut budget = Budget::default();
    assert!(budget.admit("unplanned", true, "").is_err());
    assert_eq!(budget.calls, 0);
    for (id, launch, prefix) in [
        (AUTHOR, true, "C21 bounded diagnostic"),
        (AUTHOR, false, "work-leaf patch applied\n"),
        (REVIEWER, true, "Review the full patch scope"),
        (REVIEWER, false, "work-leaf command result\n"),
    ] {
        assert!(budget.admit(id, !launch, prefix).is_err());
        budget.admit(id, launch, prefix).unwrap();
    }
    assert_eq!(budget.calls, 4);
    assert!(
        budget
            .admit(REVIEWER, false, "work-leaf command result\n")
            .is_err()
    );
    let mut exhausted = Budget { calls: MAX_CALLS };
    assert!(
        exhausted
            .admit(AUTHOR, true, "C21 bounded diagnostic")
            .is_err()
    );
}

struct BoundedBackend {
    inner: CodexBackend,
    budget: Budget,
}

impl BoundedBackend {
    fn check_reply(&self, text: &str) -> Result<(), AgentError> {
        let expected = match self.budget.calls {
            1 => EDIT,
            2 => PUBLIC_DONE,
            3 => CHECK,
            4 => "NO_FINDINGS",
            _ => return Err(AgentError::Io(invalid())),
        };
        if text.trim() != expected {
            return Err(AgentError::Io(invalid()));
        }
        Ok(())
    }
}

impl AgentBackend for BoundedBackend {
    fn launch(&mut self, _: AgentLaunch) -> Result<AgentSession, AgentError> {
        Err(AgentError::Io(invalid()))
    }
    fn send(&mut self, _: &AgentId, _: &str) -> Result<ChatMessage, AgentError> {
        Err(AgentError::Io(invalid()))
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
            .admit(request.id.as_str(), true, &request.prompt)
            .map_err(AgentError::Io)?;
        let session = self
            .inner
            .launch_streaming_interruptible(request, sink, interrupt)?;
        self.check_reply(
            &session
                .messages
                .last()
                .ok_or_else(|| AgentError::Io(invalid()))?
                .text,
        )?;
        Ok(session)
    }
    fn send_streaming_interruptible(
        &mut self,
        id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
        interrupt: &mut dyn FnMut(&AgentStreamEvent) -> bool,
    ) -> Result<ChatMessage, AgentError> {
        self.budget
            .admit(id.as_str(), false, prompt)
            .map_err(AgentError::Io)?;
        if self.budget.calls == 4
            && !(prompt.contains("\nstatus: 0\n") && prompt.contains("C21_CHECK_OK"))
        {
            return Err(AgentError::Io(invalid()));
        }
        let message = self
            .inner
            .send_streaming_interruptible(id, prompt, sink, interrupt)?;
        self.check_reply(&message.text)?;
        Ok(message)
    }
}

fn required_path(name: &str, directory: bool) -> PathBuf {
    let path = PathBuf::from(env::var_os(name).unwrap_or_else(|| panic!("missing {name}")));
    let meta = fs::symlink_metadata(&path).unwrap();
    assert!(
        path.is_absolute()
            && path.canonicalize().unwrap() == path
            && if directory {
                meta.is_dir()
            } else {
                meta.is_file()
            },
        "invalid {name}"
    );
    path
}

fn json_file(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn prepare_fixture(root: &Path) {
    assert!(
        fs::read_dir(root).unwrap().next().is_none(),
        "diagnostic checkout must be empty"
    );
    fs::write(root.join("fixture.rs"), INITIAL).unwrap();
    fs::write(root.join("check-fixture.sh"), CHECK_SCRIPT).unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Review Evidence Diagnostic"],
        vec!["config", "user.email", "review-evidence@example.invalid"],
        vec!["add", "fixture.rs", "check-fixture.sh"],
        vec![
            "commit",
            "-qm",
            "ADD diagnostic fixture for bounded archive verification",
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
}

fn run_scenario(backend: CodexBackend, root: &Path) -> Result<usize, AgentError> {
    let mut chat = CommandChat::new(
        root.to_path_buf(),
        BoundedBackend {
            inner: backend,
            budget: Budget::default(),
        },
    )
    .with_max_review_rounds(1);
    chat.launch_prepared_agent_streaming(
        AgentLaunch::new(
            AgentId::new(AUTHOR)?,
            AgentKind::Codex,
            FEATURE,
            scenario_prompt(),
        ),
        &mut |_| {},
    )
    .map_err(|error| AgentError::Io(io::Error::other(error.to_string())))?;
    let result = chat
        .handle_line("review")
        .map_err(|error| AgentError::Io(io::Error::other(error.to_string())))?;
    let CommandChatResult::ReviewComplete(reviews) = result else {
        return Err(AgentError::Io(invalid()));
    };
    if reviews.len() != 1 || !reviews[0].findings_resolved || reviews[0].rounds != 1 {
        return Err(AgentError::Io(invalid()));
    }
    Ok(chat.into_backend().budget.calls)
}

fn trace_archive(manifest: &Value, root: &Path, client: &[Value]) -> (String, String) {
    let rows = frame_bytes(
        &fs::read(string(&manifest["evidence_path"]).unwrap()).unwrap(),
        true,
    )
    .unwrap();
    assert_eq!(rows.len(), 6);
    assert_eq!(rows[0]["event"], "activation");
    assert_eq!(
        rows[1..]
            .iter()
            .map(|v| v["site"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "policy-injection",
            "patch-applied",
            "review-source-context",
            "policy-injection",
            "command-result"
        ]
    );
    for row in &rows {
        assert_eq!(row["run_id"], manifest["run_id"]);
        assert_eq!(row["condition"], manifest["condition"]);
        assert_eq!(row["schema"], "work-leaf-bench-experiment-v5");
    }
    let review = &rows[3];
    assert_eq!(review["event"], "review-context");
    assert_eq!(review["source_agent_id"], AUTHOR);
    assert_eq!(review["reviewer_id"], REVIEWER);
    assert_eq!(review["selected_candidate"], "candidate");
    assert_eq!(review["forwarded_prompt"], review["candidate_prompt"]);
    assert_eq!(review["changed"], true);
    let original = string(&review["original_prompt"]).unwrap();
    let candidate = string(&review["candidate_prompt"]).unwrap();
    let a = usize::try_from(review["context_start"].as_u64().unwrap()).unwrap();
    let b = usize::try_from(review["context_end"].as_u64().unwrap()).unwrap();
    let x = usize::try_from(review["candidate_start"].as_u64().unwrap()).unwrap();
    let y = usize::try_from(review["candidate_end"].as_u64().unwrap()).unwrap();
    assert_eq!(&original[..a], &candidate[..x]);
    assert_eq!(&original[b..], &candidate[y..]);
    assert_eq!(a, x);
    let archive = &review["archive"];
    let path = PathBuf::from(string(&archive["path"]).unwrap());
    let archive_root = PathBuf::from(string(&manifest["review_evidence_root"]).unwrap());
    assert_eq!(path.parent(), Some(archive_root.as_path()));
    assert_eq!(path.canonicalize().unwrap(), path);
    assert!(fs::symlink_metadata(&path).unwrap().is_file());
    assert!(fs::metadata(&path).unwrap().permissions().readonly());
    let body = fs::read_to_string(&path).unwrap();
    assert_eq!(body, &original[a..b]);
    assert_eq!(archive["bytes"], body.len());
    assert!(
        body.contains(&format!(" agent:\n{PUBLIC_DONE}")),
        "public agent evidence must be retained, not only the launch instruction"
    );
    let metadata_path = PathBuf::from(string(&archive["manifest_path"]).unwrap());
    assert_eq!(metadata_path.parent(), Some(archive_root.as_path()));
    assert_eq!(metadata_path.canonicalize().unwrap(), metadata_path);
    assert!(fs::symlink_metadata(&metadata_path).unwrap().is_file());
    let metadata = json_file(&metadata_path);
    assert_eq!(metadata["archive"], *archive);
    assert_eq!(metadata["schema"], "work-leaf-review-evidence-v1");
    assert_eq!(metadata["context_start"], review["context_start"]);
    assert_eq!(metadata["context_end"], review["context_end"]);
    assert_eq!(metadata["project_snapshots_applicable"], false);
    let input = starts(client);
    assert_eq!(input.len(), 4);
    for (index, row_index, agent) in [
        (0, 1, AUTHOR),
        (1, 2, AUTHOR),
        (2, 4, REVIEWER),
        (3, 5, REVIEWER),
    ] {
        let row = &rows[row_index];
        assert_eq!(row["event"], "prompt");
        assert_eq!(row["agent_id"], agent);
        assert_eq!(row["original_prompt"], row["forwarded_prompt"]);
        assert_eq!(row["changed"], false);
        assert_eq!(
            input[index]["params"]["input"][0]["text"],
            row["forwarded_prompt"]
        );
    }
    let policy = PromptPolicy::for_project(root).unwrap();
    assert_eq!(
        rows[1]["forwarded_prompt"],
        policy.inject(&AgentId::new(AUTHOR).unwrap(), FEATURE, &scenario_prompt())
    );
    assert_eq!(
        rows[4]["forwarded_prompt"],
        policy.inject(
            &AgentId::new(REVIEWER).unwrap(),
            &format!("review {FEATURE}"),
            candidate
        )
    );
    (body, path.to_str().unwrap().to_string())
}

#[test]
#[ignore = "requires explicit separate real subscription admission; expected four turns, hard six-turn/118-second ceiling"]
fn real_subscription_review_evidence_handoffs() {
    let manifest = json_file(&required_path("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", false));
    let config_path = required_path("WORK_LEAF_OBSERVER_CONFIG", false);
    let config = json_file(&config_path);
    validate_preconditions(&config, &manifest, |key| env::var(key).ok()).unwrap();
    let root = required_path("WORK_LEAF_REAL_OBSERVER_PROJECT_DIR", true);
    let proxy = required_path("WORK_LEAF_REAL_OBSERVER_CODEX_PROXY", false);
    let observer = required_path("WORK_LEAF_REAL_OBSERVER_BIN", false);
    let opaque = PathBuf::from(string(&manifest["review_evidence_root"]).unwrap());
    assert!(
        opaque.is_dir()
            && opaque.is_absolute()
            && opaque.canonicalize().unwrap() == opaque
            && !opaque.starts_with(&root)
            && !root.starts_with(&opaque)
    );
    prepare_fixture(&root);
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
            eprintln!("review-evidence diagnostic reached its 118-second ceiling");
            process::exit(1);
        }
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_scenario(backend.clone(), &root)
    }));
    println!(
        "WORK_LEAF_REVIEW_EVIDENCE_SCENARIO_RETURNED success={}; postcapture checks follow and cannot replace this outcome",
        matches!(&result, Ok(Ok(4)))
    );
    let observation = PathBuf::from(string(&config["root"]).unwrap());
    let captures: Vec<_> = fs::read_dir(observation.join("app-server"))
        .unwrap()
        .map(|v| v.unwrap().path())
        .filter(|v| v.is_dir())
        .collect();
    assert_eq!(captures.len(), 1);
    let capture = &captures[0];
    let deadline = Instant::now() + Duration::from_secs(15);
    let settled = loop {
        let read = |name: &str| {
            fs::read(capture.join(name))
                .ok()
                .and_then(|b| frame_bytes(&b, false).ok())
                .unwrap_or_default()
        };
        if terminal_settled(
            &read("client-to-server.raw"),
            &read("client-to-server.forwarded.raw"),
            &read("server-to-client.raw"),
        )
        .is_ok()
        {
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
    let stopped = stopped.unwrap();
    println!("WORK_LEAF_REVIEW_EVIDENCE_TERMINAL_SETTLED value={settled}");
    capture_closed(stopped.status.success(), &stopped.stdout, &stopped.stderr).unwrap();
    assert!(settled, "last accepted turn in each thread must settle");
    let calls = result
        .expect("diagnostic panicked")
        .expect("diagnostic workflow failed");
    assert_eq!(calls, 4);
    assert_eq!(fs::read_to_string(root.join("fixture.rs")).unwrap(), FINAL);
    assert_eq!(
        fs::read_to_string(root.join("check-fixture.sh")).unwrap(),
        CHECK_SCRIPT
    );
    let client = frame_bytes(
        &fs::read(capture.join("client-to-server.raw")).unwrap(),
        true,
    )
    .unwrap();
    let forwarded = frame_bytes(
        &fs::read(capture.join("client-to-server.forwarded.raw")).unwrap(),
        true,
    )
    .unwrap();
    let server = frame_bytes(
        &fs::read(capture.join("server-to-client.raw")).unwrap(),
        true,
    )
    .unwrap();
    let sessions: Vec<_> = client
        .iter()
        .filter(|v| v["method"] == "thread/start")
        .collect();
    assert_eq!(sessions.len(), 2);
    assert!(!client.iter().any(|v| v["method"] == "thread/resume"));
    for session in sessions {
        assert_eq!(session["params"]["model"], "gpt-5.5");
        assert_eq!(session["params"]["sandbox"], "read-only");
        assert_eq!(session["params"]["approvalPolicy"], "never");
    }
    let (archive, path) = trace_archive(&manifest, &root, &client);
    verify_public_capture(&client, &forwarded, &server, &archive, &path).unwrap();
    println!(
        "WORK_LEAF_REVIEW_EVIDENCE_SMOKE_OK turns={calls} sessions=2 full_public_archive_read=1 focused_locked_check=1; native source/hash/accounting audit remains separate"
    );
}

#[test]
fn diagnostic002_instruction_requires_check_before_verdict() {
    let first = FEATURE
        .find("Reviewer turn 1 (mandatory even if the edit is obviously correct):")
        .unwrap();
    let no_verdict = FEATURE
        .find("Do not return NO_FINDINGS on this turn.")
        .unwrap();
    let check = FEATURE.find(CHECK).unwrap();
    let second = FEATURE
        .find("Reviewer turn 2, only after the actual Work Leaf command result")
        .unwrap();
    assert!(first < no_verdict && no_verdict < check && check < second);
    assert!(FEATURE.contains("status: 0 and C21_CHECK_OK"));
    assert!(
        FEATURE.contains(
            "The native-tool limit does not prohibit this required Work Leaf locked check."
        )
    );
    assert_eq!(FEATURE.matches(CHECK).count(), 1);
    assert!(FEATURE.contains("login=false, max_output_tokens=20000"));
}
