use c15_real_diagnostic_guards::{author_completion, await_author_completion};
use serde_json::{Value, json};
use std::path::Path;

type Capture = (Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>);

fn fixture() -> Capture {
    let texts = [
        "owned launch",
        "work-leaf patch applied\nfiles: source.rs",
        "work-leaf command result\ncommand: check-current\nstatus: 0\nlocked paths: target\nnext: reply\nstdout:\nPASS\nstderr:\n",
    ];
    let mut trace = vec![
        json!({"event":"activation","schema":"work-leaf-bench-experiment-v6","condition":"private-test-first"}),
    ];
    let mut client = Vec::new();
    let mut server = Vec::new();
    for (i, text) in texts.into_iter().enumerate() {
        let id = format!("request-{i}");
        let turn = format!("turn-{i}");
        client.push(json!({"method":"turn/start","id":id,"params":{"threadId":"owned-thread","input":[{"type":"text","text":text}]}}));
        server.push(json!({"id":id,"result":{"turn":{"id":turn}}}));
        server.push(json!({"method":"item/completed","params":{"threadId":"owned-thread","turnId":turn,"item":{"id":format!("user-{i}"),"type":"userMessage","content":[{"type":"text","text":text,"text_elements":[]}]}}}));
        server.push(json!({"method":"item/completed","params":{"threadId":"owned-thread","turnId":turn,"item":{"id":format!("reply-{i}"),"type":"agentMessage","text":if i==2 {"@work-leaf done"} else {"@work-leaf read source.rs"}}}}));
        if i == 1 {
            trace.push(json!({"event":"private-preview-delivered","agent_id":"author","sequence":2,"private_preview":{"runtime_send_returned":true}}));
        }
        let site = ["policy-injection", "patch-applied", "command-result"][i];
        trace.push(json!({"event":"prompt","site":site,"sequence":trace.len(),"agent_id":"author","owned_role":if i==0 {json!("author")} else {Value::Null},"original_prompt":text,"forwarded_prompt":text}));
    }
    (trace, client.clone(), client, server)
}

#[test]
fn owned_executed_check_then_real_done_qualifies() {
    let (t, c, f, s) = fixture();
    assert!(author_completion("author", "check-current", &t, &c, &f, &s).is_ok());
    let mut prose = s.clone();
    prose[8]["params"]["item"]["text"] = json!("The focused check passed.\n@work-leaf done\n");
    assert!(author_completion("author", "check-current", &t, &c, &f, &prose).is_ok());
}

#[test]
fn preceding_commentary_item_does_not_hide_the_final_done_item() {
    let (t, c, f, mut s) = fixture();
    let mut prose = s[8].clone();
    prose["params"]["item"]["id"] = json!("commentary");
    prose["params"]["item"]["text"] = json!("The focused validation passed.");
    s.insert(8, prose);
    assert!(author_completion("author", "check-current", &t, &c, &f, &s).is_ok());
    s[8]["params"]["item"]["text"] = json!("@work-leaf read another.rs");
    assert!(author_completion("author", "check-current", &t, &c, &f, &s).is_err());
}

#[test]
fn capture_publication_can_settle_without_another_provider_call() {
    let (t, c, f, s) = fixture();
    let mut calls = 0;
    let result = await_author_completion(std::time::Duration::from_secs(1), || {
        calls += 1;
        author_completion(
            "author",
            "check-current",
            &t,
            &c,
            &f,
            if calls == 1 { &s[..s.len() - 1] } else { &s },
        )
    });
    assert!(result.is_ok());
    assert_eq!(calls, 2);
    assert!(await_author_completion(std::time::Duration::ZERO, || Ok(json!({}))).is_err());
}

#[test]
fn actual_nonzero_status_and_later_ack_invalidate_prior_green() {
    let (mut t, mut c, _, mut s) = fixture();
    let failed = t[4]["forwarded_prompt"]
        .as_str()
        .unwrap()
        .replace("status: 0\n", "status: 101\n");
    t[4]["forwarded_prompt"] = json!(failed);
    t[4]["original_prompt"] = json!(failed);
    c[2]["params"]["input"][0]["text"] = json!(failed);
    s[7]["params"]["item"]["content"][0]["text"] = json!(failed);
    assert!(author_completion("author", "check-current", &t, &c, &c, &s).is_err());
    let (mut t, mut c, _, mut s) = fixture();
    let text = "work-leaf patch applied\nfiles: another.rs";
    t.push(json!({"event":"prompt","site":"patch-applied","agent_id":"author","sequence":5,"original_prompt":text,"forwarded_prompt":text}));
    c.push(json!({"id":"later","method":"turn/start","params":{"threadId":"owned-thread","input":[{"type":"text","text":text}]}}));
    s.push(json!({"id":"later","result":{"turn":{"id":"later-turn"}}}));
    s.push(json!({"method":"item/completed","params":{"threadId":"owned-thread","turnId":"later-turn","item":{"id":"later-user","type":"userMessage","content":[{"type":"text","text":text,"text_elements":[]}]}}}));
    s.push(json!({"method":"item/completed","params":{"threadId":"owned-thread","turnId":"later-turn","item":{"id":"later-reply","type":"agentMessage","text":"@work-leaf done"}}}));
    assert!(author_completion("author", "check-current", &t, &c, &c, &s).is_err());
}

#[test]
fn indented_code_marker_does_not_qualify() {
    let (t, c, f, mut s) = fixture();
    s[8]["params"]["item"]["text"] = json!("Example:\n    @work-leaf done");
    assert!(author_completion("author", "check-current", &t, &c, &f, &s).is_err());
}

#[test]
fn timed_out_zero_status_does_not_qualify() {
    let (mut t, mut c, _, mut s) = fixture();
    let text = t[4]["forwarded_prompt"]
        .as_str()
        .unwrap()
        .replace("\nnext:", "\ntimed out: yes\nnext:");
    t[4]["original_prompt"] = json!(text);
    t[4]["forwarded_prompt"] = json!(text);
    c[2]["params"]["input"][0]["text"] = json!(text);
    s[7]["params"]["item"]["content"][0]["text"] = json!(text);
    assert!(author_completion("author", "check-current", &t, &c, &c, &s).is_err());
}

#[test]
fn missing_failed_stale_spoofed_or_wrong_owner_completion_rejects() {
    let (t, c, f, s) = fixture();
    let mut variants = Vec::new();
    let mut x = s.clone();
    x.pop();
    variants.push((t.clone(), c.clone(), f.clone(), x));
    for text in [
        "Tests passed",
        "> @work-leaf done",
        "```\n@work-leaf done\n```",
        "*** Begin Patch\n+@work-leaf done\n*** End Patch",
        "@work-leaf done\n@work-leaf read source.rs",
    ] {
        let mut x = s.clone();
        x[8]["params"]["item"]["text"] = json!(text);
        variants.push((t.clone(), c.clone(), f.clone(), x));
    }
    let mut x = s.clone();
    x[8]["params"]["threadId"] = json!("reviewer");
    variants.push((t.clone(), c.clone(), f.clone(), x));
    let mut x = s.clone();
    x.push(s[8].clone());
    variants.push((t.clone(), c.clone(), f.clone(), x));
    let mut x = s.clone();
    x[6]["error"] = Value::Null;
    variants.push((t.clone(), c.clone(), f.clone(), x));
    let mut x = f.clone();
    x[2]["params"]["input"][0]["text"] = json!("different");
    variants.push((t.clone(), c.clone(), x, s.clone()));
    let mut x = c.clone();
    x.push(c[2].clone());
    variants.push((t.clone(), x.clone(), x, s.clone()));
    let mut x = t.clone();
    x.pop();
    variants.push((x, c.clone(), f.clone(), s.clone()));
    let mut x = t.clone();
    x[1]["agent_id"] = json!("another");
    variants.push((x, c.clone(), f.clone(), s.clone()));
    let mut x = t.clone();
    x[4]["site"] = json!("private-preview-result");
    variants.push((x, c.clone(), f.clone(), s.clone()));
    let mut x = t.clone();
    x.swap(3, 4);
    for (i, row) in x.iter_mut().enumerate().skip(1) {
        row["sequence"] = json!(i);
    }
    variants.push((x, c.clone(), f.clone(), s.clone()));
    for (n, (t, c, f, s)) in variants.into_iter().enumerate() {
        assert!(
            author_completion("author", "check-current", &t, &c, &f, &s).is_err(),
            "variant {n}"
        );
    }
    assert!(author_completion("author", "other-check", &t, &c, &f, &s).is_err());
}

#[test]
fn saved_002_exit_zero_is_not_a_completed_author_chain() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../c15-real-diagnostic-002");
    let harness: Value =
        serde_json::from_slice(&std::fs::read(base.join("HARNESS-RESULT.json")).unwrap()).unwrap();
    assert_eq!(harness["scenario"]["workflow_returned"], true);
    let rows = |p: &Path| -> Vec<Value> {
        std::fs::read_to_string(p)
            .unwrap()
            .lines()
            .map(|x| serde_json::from_str(x).unwrap())
            .collect()
    };
    let capture = base.join("observation/app-server/00000511898560227962-11");
    assert!(
        author_completion(
            "user-1",
            "cargo test --offline --locked",
            &rows(&base.join("prompt-events.jsonl")),
            &rows(&capture.join("client-to-server.raw")),
            &rows(&capture.join("client-to-server.forwarded.raw")),
            &rows(&capture.join("server-to-client.raw"))
        )
        .is_err()
    );
}
