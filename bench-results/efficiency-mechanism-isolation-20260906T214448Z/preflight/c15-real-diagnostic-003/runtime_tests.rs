//! Provider-free control-flow regression, not private confinement qualification.
//! The public Codex adapter talks only to the embedded synthetic app-server.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use c15_real_diagnostic_guards::diagnostic_chat;
use serde_json::{Value, json};
use work_leaf::{CodexBackend, CodexCommandConfig, CommandChatResult, PromptPolicy};

const CHECK: &str = "test \"$(cat value.txt)\" = new && printf checked > ordinary-check.txt\n";
const BRIDGE: &str = r#"import hashlib,json,pathlib
def main(argv):
 inp,out=map(pathlib.Path,argv);r=json.loads(inp.read_text());root=pathlib.Path(r['operation_root']);op=r['operation']
 assert op in ['validate','capture','test']
 result={'status':'completed','closed':True,'exit_code':1 if op=='test' else None,'stop_reason':None,'stdout':'synthetic private feedback, not executed test evidence','stderr':''}
 for k in ['run_id','agent_id','launch_generation','proposal_id']:
  if k in r:result[k]=r[k]
 receipt=root/(op+'-synthetic.json')
 with receipt.open('x') as f:json.dump({'request':r,'result':result},f)
 ref={'path':str(receipt),'sha256':hashlib.sha256(receipt.read_bytes()).hexdigest()}
 result.update(result_path=ref['path'],result_sha256=ref['sha256'])
 if op=='capture':result['selection']=ref
 with out.open('x') as f:json.dump(result,f)
 return 0
"#;
const APP_SERVER: &str = r#"#!/usr/bin/python3.14
import json,pathlib,sys
root=pathlib.Path(__file__).parent;fixture=json.loads((root/'replies.json').read_text());turn=0
def emit(v):print(json.dumps(v),flush=True)
for line in sys.stdin:
 r=json.loads(line);method=r.get('method');rid=r.get('id')
 if method=='initialize':emit({'id':rid,'result':{}})
 elif method in ['thread/start','thread/resume']:
  emit({'id':rid,'result':{'thread':{'id':'synthetic-thread','cwd':str(root/'repo'),'status':'loaded'}}})
 elif method=='turn/start':
  text=r['params']['input'][0]['text']
  with (root/'inputs.jsonl').open('a') as f:f.write(json.dumps({'turn':turn,'text':text})+'\n')
  assert turn<len(fixture), 'unexpected extra synthetic turn'
  if turn:assert text.startswith(fixture[turn]['prefix']),text
  if turn==1:assert (root/'repo/value.txt').read_text()=='old\n' and not (root/'repo/check.sh').exists()
  tid='synthetic-turn-'+str(turn);reply=fixture[turn]['reply'];turn+=1
  emit({'id':rid,'result':{'turn':{'id':tid}}})
  emit({'method':'turn/started','params':{'threadId':'synthetic-thread','turnId':tid,'turn':{'id':tid,'status':'inProgress'}}})
  emit({'method':'item/completed','params':{'threadId':'synthetic-thread','turnId':tid,'item':{'id':'message-'+tid,'type':'agentMessage','text':reply}}})
  emit({'method':'turn/completed','params':{'threadId':'synthetic-thread','turnId':tid,'turn':{'id':tid,'status':'completed'}}})
 elif rid is not None:emit({'id':rid,'result':{}})
"#;

fn sha(path: &Path) -> String {
    let output = Command::new("/usr/bin/python3.14")
        .args(["-I", "-B", "-c", "import hashlib,pathlib,sys;print(hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest())"])
        .arg(path)
        .output()
        .unwrap();
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

fn lines(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn actual_chat_reaches_ordinary_check_and_done_after_private_feedback_and_patch() {
    let root = std::env::temp_dir().join(format!(
        "c15-diagnostic-sequence-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    for name in ["repo", "preview", "ordinary-bundles"] {
        fs::create_dir(root.join(name)).unwrap();
    }
    fs::write(root.join("repo/value.txt"), "old\n").unwrap();
    git(&root.join("repo"), &["init", "-q"]);
    git(
        &root.join("repo"),
        &["config", "user.name", "Synthetic Fixture"],
    );
    git(
        &root.join("repo"),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(&root.join("repo"), &["add", "value.txt"]);
    git(&root.join("repo"), &["commit", "-qm", "initial fixture"]);
    fs::write(root.join("bridge.py"), BRIDGE).unwrap();
    fs::write(root.join("config.json"), "{\"synthetic\":true}\n").unwrap();
    fs::write(root.join("synthetic-codex"), APP_SERVER).unwrap();
    fs::set_permissions(
        root.join("synthetic-codex"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    let held = format!("*** Begin Patch\n*** Add File: check.sh\n+{CHECK}*** End Patch\n");
    let preview = format!(
        "@work-leaf test-preview {}\n{held}@work-leaf end\n",
        json!({"id":"one","revision_of":null,"format":"edit","reason":"held check",
            "test_purpose":"check requested value","test_paths":["check.sh"],
            "command":"sh check.sh","lock_paths":["."]})
    );
    let combined = format!(
        "@work-leaf edit implement and preserve check\n*** Begin Patch\n*** Update File: value.txt\n@@\n-old\n+new\n*** Add File: check.sh\n+{CHECK}*** End Patch\n@work-leaf end\n"
    );
    fs::write(
        root.join("replies.json"),
        json!([
            {"prefix":"", "reply":preview},
            {"prefix":"work-leaf private test preview result", "reply":combined},
            {"prefix":"work-leaf patch applied", "reply":"@work-leaf locks run . -- sh check.sh"},
            {"prefix":"work-leaf command result", "reply":"@work-leaf done"}
        ])
        .to_string(),
    )
    .unwrap();
    let manifest = json!({"schema":"work-leaf-bench-experiment-v6","run_id":"synthetic-sequence",
        "condition":"private-test-first","evidence_path":root.join("trace.jsonl"),"private_preview":{
            "root":root.join("preview"),"project_root":root.join("repo"),"python_path":"/usr/bin/python3.14",
            "python_sha256":sha(Path::new("/usr/bin/python3.14")),"bridge_path":root.join("bridge.py"),
            "bridge_sha256":sha(&root.join("bridge.py")),"config_path":root.join("config.json"),
            "config_sha256":sha(&root.join("config.json"))}});
    fs::write(root.join("manifest.json"), manifest.to_string()).unwrap();
    let output = Command::new("/usr/bin/timeout")
        .args(["--kill-after=5s", "40s"])
        .arg(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "runtime_sequence_child",
            "--ignored",
            "--nocapture",
        ])
        .env("C15_SYNTHETIC_SEQUENCE_ROOT", &root)
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
        .env("WORK_LEAF_BENCH_RUN_ID", "synthetic-sequence")
        .env(
            "WORK_LEAF_BENCH_EXPERIMENT_MANIFEST",
            root.join("manifest.json"),
        )
        .env(
            "WORK_LEAF_CONTEXT_BUNDLE_DIR",
            root.join("ordinary-bundles"),
        )
        .env_remove("WORK_LEAF_OBSERVER_CONFIG")
        .env_remove("WORK_LEAF_OBSERVER_PARENT_INVOCATION")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "retained fixture {}:\n{}\n{}",
        root.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "synthetic subprocess only; parent invokes this with an owned fixture"]
fn runtime_sequence_child() {
    let root = std::path::PathBuf::from(std::env::var_os("C15_SYNTHETIC_SEQUENCE_ROOT").unwrap());
    let project = root.join("repo");
    let backend = CodexBackend::new(
        CodexCommandConfig::new(project.clone()).with_binary(root.join("synthetic-codex")),
        PromptPolicy::for_project(&project).unwrap(),
    );
    let mut chat = diagnostic_chat(project.clone(), backend);
    let launch = chat
        .prepare_agent_launch(&["Change value and test it".into()])
        .unwrap();
    let result = chat.launch_prepared_agent_streaming(launch, &mut |_| {});
    chat.shutdown_agents();
    let CommandChatResult::AgentLaunched {
        agent_id, reply, ..
    } = result.unwrap()
    else {
        panic!("ordinary author launch result required");
    };
    assert_eq!(
        fs::read_to_string(project.join("value.txt")).unwrap(),
        "new\n"
    );
    assert_eq!(fs::read_to_string(project.join("check.sh")).unwrap(), CHECK);
    assert!(
        project.join("ordinary-check.txt").is_file(),
        "normal focused test command was never executed after preview and ACK: {reply}"
    );
    assert_eq!(
        fs::read_to_string(project.join("ordinary-check.txt")).unwrap(),
        "checked"
    );
    let inputs = lines(&root.join("inputs.jsonl"));
    assert_eq!(inputs.len(), 4);
    assert!(inputs[3]["text"].as_str().unwrap().contains("status: 0"));
    assert!(
        reply.contains("@work-leaf done"),
        "actual DONE is required: {reply}"
    );
    assert!(!reply.contains("did not converge"));
    assert!(
        reply
            .lines()
            .any(|line| line == format!("agent {agent_id} reported done")),
        "the actual orchestrator completion event must be processed: {reply}"
    );
    let trace = lines(&root.join("trace.jsonl"));
    for event in [
        "private-preview-proposal",
        "private-preview-result",
        "private-preview-delivered",
    ] {
        assert_eq!(
            trace.iter().filter(|r| r["event"] == event).count(),
            1,
            "{event}"
        );
    }
    for site in ["patch-applied", "command-result"] {
        assert_eq!(
            trace.iter().filter(|r| r["site"] == site).count(),
            1,
            "{site}"
        );
    }
}
