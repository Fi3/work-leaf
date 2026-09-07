//! Synthetic bridge plumbing uses actual CommandChat/patch/command flows. It is
//! not confinement or real-agent qualification; the pinned real bridge has its
//! separate provider-free executions and admitted provider gate.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use crate::agent::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, MessageRole,
    PromptPolicy,
};
use crate::cli::CommandChat;
use serde_json::{Value, json};

const CHILD: &str =
    "bench_experiment::private_test_first::integration_tests::private_runtime_child";
const STUB: &str = r#"import hashlib,json,pathlib
def main(argv):
 inp,out=map(pathlib.Path,argv);r=json.loads(inp.read_text());root=pathlib.Path(r['operation_root']);op=r['operation']
 cfg=json.loads(pathlib.Path(r['config_path']).read_text());mode=cfg['mode']
 status='failed' if mode=='invalid-config' and op=='validate' else 'completed'
 closed=not(mode=='unclosed' and op=='test')
 result={'status':status,'closed':closed,'exit_code':1 if op=='test' else None,'stop_reason':None,'stdout':'synthetic bridge result; not real test execution','stderr':''}
 for k in ['run_id','agent_id','launch_generation','proposal_id']:
  if k in r:result[k]=r[k]
 receipt=root/(op+'-fixture-receipt.json');receipt.write_text(json.dumps({'request':r,'result':result}))
 ref={'path':str(receipt),'sha256':hashlib.sha256(receipt.read_bytes()).hexdigest()}
 result.update(result_path=ref['path'],result_sha256=ref['sha256'])
 if op=='capture':result['selection']=ref
 out.write_text(json.dumps(result))
 return 0 if status=='completed' else 1
"#;

fn sha(path: &Path) -> String {
    let output = Command::new("/usr/bin/python3.14").args(["-I", "-B", "-c",
        "import hashlib,pathlib,sys;print(hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest())"])
        .arg(path).output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn exercise(mode: &str) -> (Value, Vec<Value>) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let generated_root = std::env::temp_dir().join(format!(
        "work-leaf-private-runtime-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let root = if mode == "qualified" {
        PathBuf::from(
            std::env::var_os("WORK_LEAF_PRIVATE_QUALIFIED_ROOT")
                .expect("explicit create-new qualification root"),
        )
    } else {
        generated_root
    };
    fs::create_dir(&root).unwrap();
    for part in ["repo", "private"] {
        fs::create_dir(root.join(part)).unwrap();
    }
    fs::write(root.join("repo/value.txt"), "old\n").unwrap();
    git(&root.join("repo"), &["init", "-q"]);
    git(&root.join("repo"), &["config", "user.name", "Fixture"]);
    git(
        &root.join("repo"),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(&root.join("repo"), &["add", "."]);
    git(&root.join("repo"), &["commit", "-qm", "fixture"]);
    let bridge = if mode == "qualified" {
        PathBuf::from(
            std::env::var_os("WORK_LEAF_PRIVATE_QUALIFIED_BRIDGE")
                .expect("exact qualified bridge source"),
        )
    } else {
        fs::write(root.join("bridge.py"), STUB).unwrap();
        root.join("bridge.py")
    };
    let config = if mode == "qualified" {
        let path = PathBuf::from(
            std::env::var_os("WORK_LEAF_PRIVATE_QUALIFIED_CONFIG")
                .expect("qualified public-input config"),
        );
        let mut config: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        // This separately owned generic fixture has no effective source overlay.
        config["overlays"] = json!([]);
        config
    } else {
        json!({"mode":mode})
    };
    fs::write(root.join("config.json"), config.to_string()).unwrap();
    let manifest = json!({"schema":"work-leaf-bench-experiment-v6","run_id":"private-fixture",
        "condition":"private-test-first","evidence_path":root.join("trace.jsonl"),"private_preview":{
            "root":root.join("private"),"project_root":root.join("repo"),"python_path":"/usr/bin/python3.14",
            "python_sha256":sha(Path::new("/usr/bin/python3.14")),"bridge_path":bridge,
            "bridge_sha256":sha(&bridge),"config_path":root.join("config.json"),
            "config_sha256":sha(&root.join("config.json"))}});
    fs::write(root.join("manifest.json"), manifest.to_string()).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env("WORK_LEAF_PRIVATE_TEST_ROOT", &root)
        .env("WORK_LEAF_PRIVATE_TEST_MODE", mode)
        .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
        .env("WORK_LEAF_BENCH_RUN_ID", "private-fixture")
        .env(
            "WORK_LEAF_BENCH_EXPERIMENT_MANIFEST",
            root.join("manifest.json"),
        )
        .env(
            "WORK_LEAF_CONTEXT_BUNDLE_DIR",
            if mode == "bundle-overlap" {
                root.clone()
            } else {
                root.join("bundles")
            },
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fixture {}: {}\n{}",
        root.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let result = serde_json::from_slice(&fs::read(root.join("result.json")).unwrap()).unwrap();
    let trace = fs::read_to_string(root.join("trace.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    if mode != "qualified" {
        fs::remove_dir_all(&root).unwrap();
    }
    (result, trace)
}

fn preview() -> String {
    format!(
        "@work-leaf test-preview {}\n*** Begin Patch\n*** Add File: check.sh\n+test \"$(cat value.txt)\" = new\n*** End Patch\n@work-leaf end\n",
        json!({"id":"first","revision_of":null,"format":"edit","reason":"held check",
            "test_purpose":"check requested value","test_paths":["check.sh"],"command":"sh check.sh","lock_paths":["."]})
    )
}

fn combined() -> String {
    "@work-leaf edit implement and retain check\n*** Begin Patch\n*** Update File: value.txt\n@@\n-old\n+new\n*** Add File: check.sh\n+test \"$(cat value.txt)\" = new\n*** End Patch\n@work-leaf end\n".into()
}

#[derive(Default)]
struct Observed {
    launches: usize,
    inputs: Vec<String>,
    private_inputs: usize,
    sessions: BTreeMap<AgentId, AgentSession>,
    deferred: Option<AgentLaunch>,
}

#[derive(Clone)]
struct Backend {
    root: PathBuf,
    mode: String,
    observed: Arc<Mutex<Observed>>,
}

impl AgentBackend for Backend {
    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(crate::agent::AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        let session = self.launch(request)?;
        sink(crate::agent::AgentStreamEvent::Status(
            "fixture backend response ready".into(),
        ));
        Ok(session)
    }

    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut observed = self.observed.lock().unwrap();
        observed.launches += 1;
        if self.mode == "deferred-injection" {
            observed.deferred = Some(request.clone());
            return Ok(AgentSession::new(request));
        }
        let injected = if self.mode == "cross-thread-injection" {
            let root = self.root.clone();
            let request = request.clone();
            std::thread::spawn(move || {
                PromptPolicy::for_project(&root)?.inject_for_delivery(
                    &request.id,
                    &request.feature,
                    &request.prompt,
                )
            })
            .join()
            .unwrap()?
        } else {
            PromptPolicy::for_project(&self.root)?.inject_for_delivery(
                &request.id,
                &request.feature,
                &request.prompt,
            )?
        };
        if self.mode == "cross-thread-injection" {
            return Ok(AgentSession::new(request));
        }
        assert!(injected.contains("@work-leaf test-preview"));
        let mut session = AgentSession::new(request);
        session.messages[0].text = injected;
        session.push_message(
            MessageRole::Agent,
            if self.mode == "gate-first" {
                combined()
            } else {
                preview()
            },
        );
        observed
            .sessions
            .insert(session.id.clone(), session.clone());
        Ok(session)
    }
    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let mut observed = self.observed.lock().unwrap();
        observed.inputs.push(prompt.into());
        let reply = if prompt.starts_with("work-leaf private test preview required") {
            assert_eq!(
                fs::read_to_string(self.root.join("value.txt")).unwrap(),
                "old\n"
            );
            preview()
        } else if prompt.starts_with("work-leaf private test preview result") {
            assert_eq!(
                fs::read_to_string(self.root.join("value.txt")).unwrap(),
                "old\n"
            );
            assert!(!self.root.join("check.sh").exists());
            observed.private_inputs += 1;
            if self.mode == "reinject" {
                PromptPolicy::for_project(&self.root)?.inject_for_delivery(
                    agent_id,
                    "recreated",
                    prompt,
                )?;
            }
            if self.mode == "replay" && observed.private_inputs == 1 {
                preview()
            } else {
                combined()
            }
        } else if prompt.starts_with("work-leaf patch applied") {
            assert_eq!(
                fs::read_to_string(self.root.join("value.txt")).unwrap(),
                "new\n"
            );
            "@work-leaf locks run . -- sh check.sh".into()
        } else if prompt.starts_with("work-leaf command result") {
            "@work-leaf done".into()
        } else {
            panic!("unexpected prompt: {prompt}");
        };
        Ok(ChatMessage::new(MessageRole::Agent, reply))
    }
}

#[test]
fn private_command_chat_gates_apply_preserves_ack_and_replays_without_execution() {
    for mode in ["ordinary", "gate-first", "replay"] {
        let (result, trace) = exercise(mode);
        assert_eq!(result["ok"], true, "{result}");
        assert_eq!(result["launches"], 1);
        assert_eq!(result["value"], "new\n");
        assert_eq!(
            trace
                .iter()
                .filter(|row| row["event"] == "private-preview-proposal")
                .count(),
            1
        );
        assert_eq!(
            trace
                .iter()
                .filter(|row| row["event"] == "private-preview-result")
                .count(),
            1
        );
        assert_eq!(
            trace
                .iter()
                .filter(|row| row["site"] == "patch-applied")
                .count(),
            1
        );
    }
}

#[test]
fn task_facing_feedback_omits_host_identity_bookkeeping_but_retains_it_in_evidence() {
    let (result, trace) = exercise("ordinary");
    let prompt = result["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .find(|text| text.starts_with("work-leaf private test preview result"))
        .unwrap();
    for prefix in ["launch-generation:", "receipt:", "receipt-sha256:"] {
        assert!(
            !prompt.lines().any(|line| line.starts_with(prefix)),
            "host-only field {prefix} leaked into feedback"
        );
    }
    assert!(prompt.contains("proposal: first\n"));
    assert!(prompt.contains("command: sh check.sh\nstatus: 1\n"));
    let event = trace
        .iter()
        .find(|row| row["event"] == "private-preview-result")
        .unwrap();
    assert!(
        event["private_preview"]["launch_generation"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(
        event["private_preview"]["result"]["result_path"]
            .as_str()
            .unwrap()
            .starts_with('/')
    );
    assert_eq!(
        event["private_preview"]["result"]["result_sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
}

#[test]
fn invalid_activation_and_uncertain_closure_never_deliver_or_apply() {
    for mode in ["invalid-config", "unclosed"] {
        let (result, _) = exercise(mode);
        assert_eq!(result["ok"], false);
        assert_eq!(result["value"], "old\n");
        assert_eq!(result["inputs"], json!([]));
        if mode == "invalid-config" {
            assert_eq!(result["launches"], 0);
        }
    }
}

#[test]
fn private_evidence_cannot_use_the_ordinary_bundle_namespace() {
    let (result, trace) = exercise("bundle-overlap");
    assert_eq!(result["ok"], false);
    assert_eq!(result["launches"], 0);
    assert_eq!(result["inputs"], json!([]));
    assert!(trace.is_empty());
}

#[test]
fn preview_result_must_resume_the_existing_session_without_policy_reinjection() {
    let (result, _) = exercise("reinject");
    assert_eq!(
        result["ok"], false,
        "recreated session must not open the shared-apply gate"
    );
    assert_eq!(result["value"], "old\n");
}

#[test]
fn cross_thread_owned_author_injection_rejects_before_any_policy_delivery() {
    let (result, trace) = exercise("cross-thread-injection");
    assert_eq!(result["ok"], false);
    assert_eq!(result["value"], "old\n");
    assert_eq!(
        trace
            .iter()
            .filter(|row| row["site"] == "policy-injection")
            .count(),
        0,
        "unsupported author injection must not publish a baseline policy for provider delivery"
    );
}

#[test]
fn deferred_author_injection_and_stopped_preview_never_resume_or_apply() {
    for mode in ["deferred-injection", "cancel-preview"] {
        let (result, trace) = exercise(mode);
        assert_eq!(result["ok"], false, "{result}");
        assert_eq!(result["value"], "old\n");
        assert_eq!(result["inputs"], json!([]));
        assert_eq!(
            trace
                .iter()
                .filter(|row| row["event"] == "private-preview-delivered")
                .count(),
            0
        );
        if mode == "deferred-injection" {
            assert_eq!(result["deferred_rejected"], true);
        }
    }
}

#[test]
fn interrupt_and_shutdown_before_launch_commit_cannot_start_private_work() {
    for mode in ["interrupt-provisional", "shutdown-provisional"] {
        let (result, trace) = exercise(mode);
        assert_eq!(result["ok"], false, "{result}");
        assert_eq!(result["inputs"], json!([]));
        assert_eq!(result["value"], "old\n");
        assert_eq!(
            trace
                .iter()
                .filter(|row| row["event"] == "private-preview-proposal")
                .count(),
            0
        );
    }
}

#[test]
fn active_controller_title_and_dependency_revisions_preserve_owned_launches_and_show_cancellation()
{
    for mode in [
        "controller-title",
        "controller-dependency",
        "controller-cancel-title",
        "controller-cancel-dependency",
    ] {
        let (result, trace) = exercise(mode);
        let cancelled = mode.contains("cancel");
        assert_eq!(result["ok"], !cancelled, "{result}");
        assert_eq!(result["launches"], if cancelled { 0 } else { 1 });
        if cancelled {
            assert_eq!(result["value"], "old\n");
            assert_eq!(result["controller"]["ui_notice_visible"], true);
            assert_eq!(
                trace
                    .iter()
                    .filter(|row| row["site"] == "policy-injection")
                    .count(),
                0
            );
        } else {
            assert_eq!(result["value"], "new\n");
            let policy = trace
                .iter()
                .find(|row| row["site"] == "policy-injection")
                .unwrap();
            assert_eq!(policy["owned_role"], "author");
            assert!(
                policy["forwarded_prompt"]
                    .as_str()
                    .unwrap()
                    .contains(result["controller"]["expected_prompt"].as_str().unwrap())
            );
        }
    }
}

#[test]
#[ignore = "provider-free actual confined bridge qualification requires explicit exact public-input pins and create-new retained root"]
fn qualified_bridge_runs_held_test_then_ordinary_shared_apply_and_check() {
    let (result, trace) = exercise("qualified");
    assert_eq!(result["ok"], true, "{result}");
    assert_eq!(result["launches"], 1);
    assert_eq!(result["value"], "new\n");
    let results = trace
        .iter()
        .filter(|row| row["event"] == "private-preview-result")
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0]["private_preview"]["result"]["status"],
        "completed"
    );
    assert_eq!(results[0]["private_preview"]["result"]["exit_code"], 1);
    assert_eq!(results[0]["private_preview"]["result"]["closed"], true);
    assert_eq!(
        trace
            .iter()
            .filter(|row| row["site"] == "patch-applied")
            .count(),
        1
    );
    assert_eq!(
        trace
            .iter()
            .filter(|row| row["event"] == "private-preview-delivered")
            .count(),
        1
    );
}

#[test]
#[ignore = "isolated provider-free CommandChat subprocess; parent tests invoke it"]
fn private_runtime_child() {
    let root = PathBuf::from(std::env::var_os("WORK_LEAF_PRIVATE_TEST_ROOT").unwrap());
    let mode = std::env::var("WORK_LEAF_PRIVATE_TEST_MODE").unwrap();
    let observed = Arc::new(Mutex::new(Observed::default()));
    let backend = Backend {
        root: root.join("repo"),
        mode: mode.clone(),
        observed: Arc::clone(&observed),
    };
    let mut chat = CommandChat::new(root.join("repo"), backend).with_max_review_rounds(10);
    if mode.starts_with("controller-") {
        let controller = crate::workspace::private_test_first_tests::exercise(chat, &mode);
        let observed = observed.lock().unwrap();
        fs::write(
            root.join("result.json"),
            json!({"ok":controller["ok"],"controller":controller,
            "launches":observed.launches,"inputs":observed.inputs,
            "value":fs::read_to_string(root.join("repo/value.txt")).unwrap()})
            .to_string(),
        )
        .unwrap();
        return;
    }
    let mut stopper = chat.clone();
    let result = chat
        .prepare_agent_launch(&["implement requested value".into()])
        .and_then(|launch| {
            let id = launch.id.clone();
            chat.launch_prepared_agent_streaming(launch, &mut |event| {
                if matches!(event, crate::agent::AgentStreamEvent::Status(ref text) if text == "fixture backend response ready") {
                    if mode=="interrupt-provisional" { let _ = stopper.interrupt_agent(&id); }
                    if mode=="shutdown-provisional" { stopper.shutdown_agents(); }
                }
                if mode=="cancel-preview" && matches!(event, crate::agent::AgentStreamEvent::Status(ref text) if text.starts_with("running private test preview")) {
                    let _ = stopper.interrupt_agent(&id);
                }
            })
        });
    let observed = observed.lock().unwrap();
    let deferred_rejected = observed.deferred.as_ref().is_some_and(|launch| {
        PromptPolicy::for_project(root.join("repo"))
            .unwrap()
            .inject_for_delivery(&launch.id, &launch.feature, &launch.prompt)
            .is_err()
    });
    fs::write(
        root.join("result.json"),
        json!({"ok":result.is_ok(),"error":result.err().map(|e| e.to_string()),"deferred_rejected":deferred_rejected,
        "launches":observed.launches,"inputs":observed.inputs,
        "value":fs::read_to_string(root.join("repo/value.txt")).unwrap()})
        .to_string(),
    )
    .unwrap();
}
