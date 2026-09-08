// Public workflow fixture, synthetic provider only. Each activation owns a process.
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentOrchestrator, ChatMessage, MessageRole,
    OrchestratorEvent,
};
mod temp_cleanup;

const SCHEMA: &str = "work-leaf-bench-experiment-v7";
const CONDITION: &str = "automatic-changed-refresh-full";

fn temp() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "work-leaf-auto-refresh-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    temp_cleanup::register(&root);
    root
}

fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// Only the existing exact-source validation bootstrap runs. There is no
// private execution, author enrollment or provider qualification in this fixture.
fn private_validation_descriptor(root: &Path) -> Value {
    fn sha(path: &Path) -> String {
        let result = Command::new("/usr/bin/sha256sum")
            .arg(path)
            .output()
            .unwrap();
        assert!(result.status.success());
        String::from_utf8(result.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .into()
    }
    let preview = root.join("private-preview");
    fs::create_dir(&preview).unwrap();
    let bridge = root.join("validation-only.py");
    let config = root.join("validation-only.json");
    fs::write(&config, "{\"synthetic_validation_only\":true}").unwrap();
    fs::write(&bridge, r#"import hashlib,json,pathlib
def main(argv):
 inp,out=map(pathlib.Path,argv);request=json.loads(inp.read_text())
 assert request['schema']=='work-leaf-private-preview-bridge-v1'
 assert request['operation']=='validate'
 assert __compiled_sha256__==hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
 assert json.loads(pathlib.Path(request['config_path']).read_text())=={'synthetic_validation_only':True}
 with out.open('x') as stream: json.dump({'status':'completed','closed':True,'synthetic_validation_only':True},stream)
 return 0
"#).unwrap();
    let python = Path::new("/usr/bin/python3").canonicalize().unwrap();
    json!({"root":preview,"project_root":root.join("project"),"python_path":python,
        "python_sha256":sha(&python),"bridge_path":bridge,"bridge_sha256":sha(&bridge),
        "config_path":config,"config_sha256":sha(&config)})
}

fn exercise(schema: Option<&str>, condition: &str, kind: &str) -> (Value, Vec<Value>) {
    let root = temp();
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.args([
        "--exact",
        "automatic_refresh_child",
        "--ignored",
        "--nocapture",
    ])
    .env("AUTO_REFRESH_ROOT", &root)
    .env("AUTO_REFRESH_KIND", kind)
    .env("WORK_LEAF_CONTEXT_BUNDLE_DIR", root.join("bundles"))
    .env_remove("WORK_LEAF_BENCH_EXPERIMENT")
    .env_remove("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST")
    .env_remove("WORK_LEAF_BENCH_RUN_ID");
    if let Some(schema) = schema {
        let manifest = root.join("manifest.json");
        let mut row = json!({"schema":schema,"condition":condition,"run_id":"automatic-refresh-fixture","evidence_path":root.join("trace.jsonl")});
        if schema == "work-leaf-bench-experiment-v5" {
            let archive = root.join("review-evidence");
            fs::create_dir(&archive).unwrap();
            row["review_evidence_root"] = json!(archive);
        }
        if schema == "work-leaf-bench-experiment-v6" {
            row["private_preview"] = private_validation_descriptor(&root);
        }
        fs::write(&manifest, serde_json::to_vec(&row).unwrap()).unwrap();
        cmd.env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST", manifest)
            .env("WORK_LEAF_BENCH_RUN_ID", "automatic-refresh-fixture");
    }
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if cfg!(feature = "bench-experiments") && schema == Some("work-leaf-bench-experiment-v6") {
        let validation: Value = serde_json::from_slice(
            &fs::read(root.join("private-preview/validation/validate-output.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validation,
            json!({"status":"completed","closed":true,"synthetic_validation_only":true})
        );
        assert_eq!(
            fs::read_dir(root.join("private-preview")).unwrap().count(),
            1
        );
    }
    let result = serde_json::from_slice(&fs::read(root.join("result.json")).unwrap()).unwrap();
    let trace = fs::read_to_string(root.join("trace.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (result, trace)
}

#[test]
fn full_automatic_refresh_preserves_real_edit_and_patch_recovery() {
    for kind in ["edit", "patch", "failed-send"] {
        let (normal, _) = exercise(None, "unused", kind);
        let (full, trace) = exercise(Some(SCHEMA), CONDITION, kind);
        assert_eq!(normal["read"], full["read"]);
        assert_eq!(normal["after_refresh_read"], full["after_refresh_read"]);
        assert_eq!(normal["final_text"], full["final_text"]);
        assert_eq!(normal["ack"], full["ack"]);
        assert_eq!(normal["bundles"], full["bundles"]);
        if cfg!(feature = "bench-experiments") {
            let rows: Vec<_> = trace
                .iter()
                .filter(|row| row["event"] == "automatic-refresh")
                .collect();
            assert_eq!(rows.len(), 1);
            let row = rows[0];
            assert_eq!(row["original_prompt"], normal["refresh"]);
            assert_eq!(row["candidate_prompt"], full["refresh"]);
            assert_eq!(row["selected_candidate"], "candidate");
            assert_eq!(
                row["metadata"]["snapshots"][0]["diff_disposition"],
                "available"
            );
            let before = row["original_prompt"].as_str().unwrap();
            let candidate = row["candidate_prompt"].as_str().unwrap();
            let (mut b, mut c) = (0, 0);
            for part in row["components"].as_array().unwrap() {
                let bs = part["baseline_start"].as_u64().unwrap() as usize;
                let be = part["baseline_end"].as_u64().unwrap() as usize;
                let cs = part["candidate_start"].as_u64().unwrap() as usize;
                let ce = part["candidate_end"].as_u64().unwrap() as usize;
                assert_eq!(&before[b..bs], &candidate[c..cs]);
                b = be;
                c = ce;
                if part["id"] == "current-full-text" {
                    let start = part["body_start"].as_u64().unwrap() as usize;
                    let end = part["body_end"].as_u64().unwrap() as usize;
                    assert_eq!(&candidate[start..end], full["current"].as_str().unwrap());
                    assert!(end - start > 8 * 1024);
                }
            }
            assert_eq!(&before[b..], &candidate[c..]);
            assert!(candidate.contains("current full text:\n"));
        } else {
            assert!(trace.is_empty());
            assert_eq!(normal, full);
        }
    }
}

#[test]
fn legacy_refresh_bytes_remain_baseline() {
    let (baseline, _) = exercise(None, "unused", "edit");
    for (schema, condition) in [
        ("work-leaf-bench-experiment-v1", "control"),
        ("work-leaf-bench-experiment-v2", "control"),
        ("work-leaf-bench-experiment-v3", "control"),
        ("work-leaf-bench-experiment-v4", "unified-diff-preferred"),
        ("work-leaf-bench-experiment-v5", "review-evidence-inline"),
        ("work-leaf-bench-experiment-v5", "review-evidence-native"),
    ] {
        let (actual, trace) = exercise(Some(schema), condition, "edit");
        assert_eq!(actual, baseline);
        assert!(trace.iter().all(|row| row["event"] != "automatic-refresh"));
    }
}

#[test]
#[cfg(all(feature = "bench-experiments", target_os = "linux"))]
fn valid_v6_bootstrap_preserves_actual_public_recovery_bytes() {
    for kind in ["edit", "patch"] {
        let (baseline, _) = exercise(None, "unused", kind);
        let (actual, trace) = exercise(
            Some("work-leaf-bench-experiment-v6"),
            "private-test-first",
            kind,
        );
        assert_eq!(actual, baseline);
        assert_eq!(trace[0]["schema"], "work-leaf-bench-experiment-v6");
        assert!(trace.iter().all(|row| row["event"] != "automatic-refresh"));
    }
}

#[test]
#[cfg(all(feature = "bench-experiments", target_os = "linux"))]
fn failed_evidence_write_blocks_actual_recovery_send_but_keeps_snapshot_advance() {
    for kind in ["evidence-failure-edit", "evidence-failure-patch"] {
        let (actual, trace) = exercise(Some(SCHEMA), CONDITION, kind);
        assert_eq!(actual["evidence_publication_failed"], true);
        assert_eq!(actual["recovery_backend_deliveries"], 0);
        assert_eq!(actual["source_preserved_after_failure"], true);
        assert!(
            actual["after_refresh_read"]
                .as_str()
                .unwrap()
                .contains("unchanged")
        );
        assert!(trace.iter().all(|row| row["event"] != "automatic-refresh"));
    }
}

#[test]
fn nonstale_no_file_and_already_applied_paths_do_not_acquire_refresh_treatment() {
    for kind in ["nonstale", "no-files", "already-applied"] {
        let (baseline, _) = exercise(None, "unused", kind);
        let (actual, trace) = exercise(Some(SCHEMA), CONDITION, kind);
        assert_eq!(actual, baseline);
        assert!(
            !actual["refresh"]
                .as_str()
                .unwrap()
                .contains("work-leaf file refresh")
        );
        assert!(trace.iter().all(|row| row["event"] != "automatic-refresh"));
    }
}

#[derive(Clone, Default)]
struct Recording(Arc<Mutex<Vec<String>>>, Arc<AtomicBool>);
impl AgentBackend for Recording {
    fn launch(&mut self, _: work_leaf::AgentLaunch) -> Result<work_leaf::AgentSession, AgentError> {
        panic!("no provider launch in recovery fixture")
    }
    fn send(&mut self, _: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        self.0.lock().unwrap().push(prompt.to_string());
        if self.1.swap(false, Ordering::Relaxed) {
            return Err(AgentError::Io(std::io::Error::other(
                "fixture send failure",
            )));
        }
        Ok(ChatMessage::new(MessageRole::Agent, "fixture reply"))
    }
}

#[cfg(all(feature = "bench-experiments", target_os = "linux"))]
mod evidence_failure {
    use super::*;
    use std::os::fd::{AsRawFd, BorrowedFd, OwnedFd, RawFd};

    unsafe extern "C" {
        fn dup2(old: RawFd, new: RawFd) -> RawFd;
        fn fcntl(fd: RawFd, command: i32, ...) -> i32;
    }

    pub(super) struct ReadOnlyEvidence {
        target: RawFd,
        saved: OwnedFd,
        flags: i32,
    }
    impl ReadOnlyEvidence {
        pub(super) fn new(path: &Path) -> Self {
            let descriptors: Vec<RawFd> = fs::read_dir("/proc/self/fd")
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| fs::read_link(entry.path()).is_ok_and(|p| p == path))
                .map(|entry| entry.file_name().to_str().unwrap().parse().unwrap())
                .collect();
            assert_eq!(
                descriptors.len(),
                1,
                "only the runtime owns the trace writer"
            );
            let target = descriptors[0];
            let flags = unsafe { fcntl(target, 1) }; // Linux F_GETFD.
            assert!(flags >= 0);
            // The ignored subprocess owns this live descriptor throughout the
            // call; cloning does not take the runtime File's ownership.
            let saved = unsafe { BorrowedFd::borrow_raw(target) }
                .try_clone_to_owned()
                .unwrap();
            let readonly = fs::File::open(path).unwrap();
            assert_eq!(unsafe { dup2(readonly.as_raw_fd(), target) }, target);
            assert_eq!(unsafe { fcntl(target, 2, flags) }, 0); // Preserve FD_CLOEXEC.
            Self {
                target,
                saved,
                flags,
            }
        }
    }
    impl Drop for ReadOnlyEvidence {
        fn drop(&mut self) {
            // Restore even while unwinding. No other test shares this process.
            assert_eq!(
                unsafe { dup2(self.saved.as_raw_fd(), self.target) },
                self.target
            );
            assert_eq!(unsafe { fcntl(self.target, 2, self.flags) }, 0);
        }
    }
}

#[test]
#[ignore = "provider-free subprocess owned by automatic tests"]
fn automatic_refresh_child() {
    let root = PathBuf::from(std::env::var_os("AUTO_REFRESH_ROOT").unwrap());
    let kind = std::env::var("AUTO_REFRESH_KIND").unwrap();
    let project = root.join("project");
    fs::create_dir(&project).unwrap();
    let old = (0..600)
        .map(|n| format!("unchanged context line {n}\n"))
        .collect::<String>()
        + "old value\n";
    let mut current = old.replace("old value\n", "λ current full text:\n--- marker ---\n");
    if matches!(kind.as_str(), "nonstale" | "no-files") {
        current = old.clone();
    }
    fs::write(project.join("source.txt"), &old).unwrap();
    git(&project, &["init", "-q"]);
    git(&project, &["config", "user.name", "Fixture"]);
    git(
        &project,
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(&project, &["add", "."]);
    git(&project, &["commit", "-qm", "initial"]);
    let backend = Recording::default();
    let mut orch = AgentOrchestrator::new(project.clone(), backend.clone());
    let id = AgentId::new("owner").unwrap();
    orch.handle_agent_message(&id, "fixture", "@work-leaf read source.txt")
        .unwrap();
    let normalize = |s: &str| s.replace(&root.display().to_string(), "<ROOT>");
    let read = normalize(backend.0.lock().unwrap().last().unwrap());
    if current != old {
        fs::write(project.join("source.txt"), &current).unwrap();
        git(&project, &["add", "."]);
        git(
            &project,
            &["commit", "-qm", "accepted intervening fixture change"],
        );
    }
    let stale = if kind == "nonstale" {
        "@work-leaf edit stale\n*** Begin Patch\n*** Update File: source.txt\n@@\n-absent old block\n+submitted value\n*** End Patch\n@work-leaf end"
    } else if kind == "no-files" {
        "@work-leaf edit no files\n*** Begin Patch\n*** End Patch\n@work-leaf end"
    } else if kind == "already-applied" {
        "@work-leaf patch already present\n--- a/source.txt\n+++ b/source.txt\n@@ -601 +601,2 @@\n-old value\n+λ current full text:\n+--- marker ---\n@work-leaf end"
    } else if matches!(kind.as_str(), "patch" | "evidence-failure-patch") {
        "@work-leaf patch stale\n--- a/source.txt\n+++ b/source.txt\n@@ -601 +601 @@\n-old value\n+submitted value\n@work-leaf end"
    } else {
        "@work-leaf edit stale\n*** Begin Patch\n*** Update File: source.txt\n@@\n-old value\n+submitted value\n*** End Patch\n@work-leaf end"
    };
    let evidence_failure = cfg!(all(feature = "bench-experiments", target_os = "linux"))
        && kind.starts_with("evidence-failure-");
    let before_recovery = backend.0.lock().unwrap().len();
    #[cfg(all(feature = "bench-experiments", target_os = "linux"))]
    let evidence_guard = evidence_failure
        .then(|| evidence_failure::ReadOnlyEvidence::new(&root.join("trace.jsonl")));
    backend.1.store(kind == "failed-send", Ordering::Relaxed);
    let rejected = orch.handle_agent_message(&id, "fixture", stale);
    #[cfg(all(feature = "bench-experiments", target_os = "linux"))]
    drop(evidence_guard);
    assert_eq!(rejected.is_err(), kind == "failed-send" || evidence_failure);
    let recovery_deliveries = backend.0.lock().unwrap().len() - before_recovery;
    let source_preserved = fs::read_to_string(project.join("source.txt")).unwrap() == current;
    if evidence_failure {
        assert!(matches!(
            &rejected,
            Err(work_leaf::OrchestratorError::Agent(AgentError::Io(_)))
        ));
        assert_eq!(recovery_deliveries, 0);
        assert!(source_preserved);
    }
    if let Ok(events) = rejected {
        assert!(
            events
                .iter()
                .any(|event| matches!(event, OrchestratorEvent::PatchRejected { .. }))
        );
    }
    let refresh = normalize(backend.0.lock().unwrap().last().unwrap());
    orch.handle_agent_message(&id, "fixture", "@work-leaf read source.txt")
        .unwrap();
    let after_read = normalize(backend.0.lock().unwrap().last().unwrap());
    if kind != "already-applied" {
        assert!(
            after_read.contains("unchanged"),
            "automatic snapshot advances even before failed send"
        );
    }
    let repair = if matches!(kind.as_str(), "nonstale" | "no-files") {
        "@work-leaf edit repair\n*** Begin Patch\n*** Update File: source.txt\n@@\n-old value\n+repaired value\n*** End Patch\n@work-leaf end"
    } else {
        "@work-leaf edit repair\n*** Begin Patch\n*** Update File: source.txt\n@@\n-λ current full text:\n+repaired value\n*** End Patch\n@work-leaf end"
    };
    let events = orch.handle_agent_message(&id, "fixture", repair).unwrap();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, OrchestratorEvent::PatchApplied { .. }))
    );
    let ack = normalize(backend.0.lock().unwrap().last().unwrap());
    let bundles = fs::read_dir(root.join("bundles"))
        .map(|dirs| dirs.count())
        .unwrap_or(0);
    let mut result = json!({"read":read,"refresh":refresh,"after_refresh_read":after_read,"ack":ack,"current":current,"final_text":fs::read_to_string(project.join("source.txt")).unwrap(),"bundles":bundles});
    if evidence_failure {
        result["evidence_publication_failed"] = json!(true);
        result["recovery_backend_deliveries"] = json!(recovery_deliveries);
        result["source_preserved_after_failure"] = json!(source_preserved);
    }
    fs::write(
        root.join("result.json"),
        serde_json::to_vec(&result).unwrap(),
    )
    .unwrap();
}
