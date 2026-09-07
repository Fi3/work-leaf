//! Namespace checks in fresh subprocesses; no global environment mutation.
use super::*;
use std::sync::atomic::AtomicU64;

const CHILD: &str =
    "bench_experiment::private_test_first::bridge::bundle_path_tests::bundle_path_child";

fn fresh_root() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "work-leaf-private-bundle-path-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn exercise(case: &str, expected: bool) {
    let root = fresh_root();
    let preview = root.join("preview");
    fs::create_dir(&preview).unwrap();
    let bundles = root.join("ordinary");
    let parent = match case {
        "existing-empty" => {
            fs::create_dir(&bundles).unwrap();
            bundles
        }
        "existing-populated" => {
            fs::create_dir(&bundles).unwrap();
            fs::write(
                bundles.join("retained.txt"),
                b"existing unrelated evidence\n",
            )
            .unwrap();
            bundles
        }
        "missing-descendants" => bundles.join("not-created"),
        "equal" => preview.clone(),
        "inside-preview" => preview.join("not-created"),
        "contains-preview" => root.clone(),
        "symlink-existing" | "symlink-missing-descendant" => {
            fs::create_dir(&bundles).unwrap();
            let alias = root.join("alias");
            std::os::unix::fs::symlink(&bundles, &alias).unwrap();
            if case == "symlink-existing" {
                alias
            } else {
                alias.join("not-created")
            }
        }
        "relative" => PathBuf::from("relative-bundles"),
        _ => panic!("unknown fixture"),
    };
    let before = inventory(&root);
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .current_dir(&root)
        .env("WORK_LEAF_CONTEXT_BUNDLE_DIR", &parent)
        .env("WORK_LEAF_PRIVATE_BUNDLE_PATH_ROOT", &preview)
        .env(
            "WORK_LEAF_PRIVATE_BUNDLE_PATH_EXPECTED",
            if expected { "pass" } else { "reject" },
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "retained fixture {} ({case}):\n{}\n{}",
        root.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        before,
        inventory(&root),
        "validation cannot create or change namespace files"
    );
    fs::remove_dir_all(root).unwrap();
}

fn inventory(root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut result = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            let body = if metadata.is_file() {
                Some(fs::read(&path).unwrap())
            } else {
                None
            };
            result.push((path.strip_prefix(root).unwrap().to_path_buf(), body));
            if metadata.is_dir() {
                pending.push(path);
            }
        }
    }
    result.sort();
    result
}

#[test]
fn existing_disjoint_empty_bundle_directory_is_valid() {
    exercise("existing-empty", true);
}

#[test]
fn existing_disjoint_populated_bundle_directory_is_valid() {
    exercise("existing-populated", true);
}

#[test]
fn missing_bundle_descendants_remain_uncreated() {
    exercise("missing-descendants", true);
}

#[test]
fn overlap_and_aliases_still_fail_closed() {
    for case in [
        "equal",
        "inside-preview",
        "contains-preview",
        "symlink-existing",
        "symlink-missing-descendant",
        "relative",
    ] {
        exercise(case, false);
    }
}

#[test]
#[ignore = "isolated provider-free namespace child; parent tests invoke it"]
fn bundle_path_child() {
    let preview = PathBuf::from(std::env::var_os("WORK_LEAF_PRIVATE_BUNDLE_PATH_ROOT").unwrap());
    let expected = std::env::var("WORK_LEAF_PRIVATE_BUNDLE_PATH_EXPECTED").unwrap() == "pass";
    // Probe counters only in the valid ordinary namespace. Suppress its normal
    // Drop cleanup so the parent's inventory measures only validator effects;
    // the isolated child exits immediately and neither store writes a bundle.
    let before = expected
        .then(|| std::mem::ManuallyDrop::new(crate::orchestrator::ContextBundleStore::new()));
    let before_debug = format!("{before:?}");
    if expected {
        assert!(before_debug.contains(&format!("orchestrator-{}-0", std::process::id())));
    }
    let result = validate_bundle_separation(&preview);
    assert_eq!(result.is_ok(), expected, "{result:?}");
    assert_eq!(before_debug, format!("{before:?}"));
    if expected {
        let after = std::mem::ManuallyDrop::new(crate::orchestrator::ContextBundleStore::new());
        assert!(format!("{after:?}").contains(&format!("orchestrator-{}-1", std::process::id())));
    }
}

const STARTUP_CHILD: &str =
    "bench_experiment::private_test_first::bridge::bundle_path_tests::command_chat_startup_child";
// Deliberately only a validation sentinel. The actual Rust bootstrap must load
// these pinned bytes; this fixture does not claim executor or provider coverage.
const VALIDATION_SENTINEL: &str = r#"import hashlib,json,pathlib
def main(argv):
 inp,out=map(pathlib.Path,argv);r=json.loads(inp.read_text())
 assert r['schema']=='work-leaf-private-preview-bridge-v1' and r['operation']=='validate'
 assert __compiled_sha256__==hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
 out.write_text(json.dumps({'status':'completed','closed':True,'operation':r['operation']}))
 return 0
"#;

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

#[test]
fn command_chat_startup_accepts_precreated_bundle_parents_without_provider_calls() {
    for populated in [false, true] {
        let root = fresh_root();
        for name in ["project", "preview", "ordinary"] {
            fs::create_dir(root.join(name)).unwrap();
        }
        if populated {
            fs::write(
                root.join("ordinary/retained.txt"),
                b"unrelated existing evidence\n",
            )
            .unwrap();
        }
        fs::write(root.join("project/source.txt"), b"unchanged source\n").unwrap();
        fs::write(root.join("bridge.py"), VALIDATION_SENTINEL).unwrap();
        fs::write(root.join("config.json"), "{}").unwrap();
        let manifest = json!({"schema":"work-leaf-bench-experiment-v6","run_id":"namespace-startup",
            "condition":"private-test-first","evidence_path":root.join("trace.jsonl"),"private_preview":{
            "root":root.join("preview"),"project_root":root.join("project"),"python_path":"/usr/bin/python3.14",
            "python_sha256":sha(Path::new("/usr/bin/python3.14")),"bridge_path":root.join("bridge.py"),
            "bridge_sha256":sha(&root.join("bridge.py")),"config_path":root.join("config.json"),
            "config_sha256":sha(&root.join("config.json"))}});
        fs::write(root.join("manifest.json"), manifest.to_string()).unwrap();
        let ordinary_before = inventory(&root.join("ordinary"));
        let source_before = inventory(&root.join("project"));
        let result = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", STARTUP_CHILD, "--ignored", "--nocapture"])
            .current_dir(&root)
            .env("WORK_LEAF_PRIVATE_STARTUP_ROOT", &root)
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env("WORK_LEAF_BENCH_RUN_ID", "namespace-startup")
            .env(
                "WORK_LEAF_BENCH_EXPERIMENT_MANIFEST",
                root.join("manifest.json"),
            )
            .env("WORK_LEAF_CONTEXT_BUNDLE_DIR", root.join("ordinary"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "retained startup fixture {}:\n{}\n{}",
            root.display(),
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(ordinary_before, inventory(&root.join("ordinary")));
        assert_eq!(source_before, inventory(&root.join("project")));
        let input: Value = serde_json::from_slice(
            &fs::read(root.join("preview/validation/validate-input.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(input["operation"], "validate");
        assert_eq!(fs::read_dir(root.join("preview")).unwrap().count(), 1);
        let trace = fs::read_to_string(root.join("trace.jsonl")).unwrap();
        assert_eq!(trace.lines().count(), 1);
        let activation: Value = serde_json::from_str(&trace).unwrap();
        assert_eq!(activation["event"], "activation");
        assert_eq!(activation["schema"], "work-leaf-bench-experiment-v6");
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
#[ignore = "isolated provider-free actual CommandChat startup; parent test invokes it"]
fn command_chat_startup_child() {
    use crate::agent::{AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage};
    struct NeverLaunch;
    impl AgentBackend for NeverLaunch {
        fn launch(&mut self, _: AgentLaunch) -> Result<AgentSession, AgentError> {
            panic!("startup qualification must stop before provider launch")
        }
        fn send(&mut self, _: &AgentId, _: &str) -> Result<ChatMessage, AgentError> {
            panic!("startup qualification must not send to a provider")
        }
    }
    let root = PathBuf::from(std::env::var_os("WORK_LEAF_PRIVATE_STARTUP_ROOT").unwrap());
    let before = std::mem::ManuallyDrop::new(crate::orchestrator::ContextBundleStore::new());
    let before_debug = format!("{before:?}");
    assert!(before_debug.contains(&format!("orchestrator-{}-0", std::process::id())));
    // Suppress ordinary teardown for an exact before/after namespace comparison.
    let mut chat = std::mem::ManuallyDrop::new(crate::cli::CommandChat::new(
        root.join("project"),
        NeverLaunch,
    ));
    let launch = chat
        .prepare_agent_launch(&["Check the requested behavior".into()])
        .unwrap();
    assert_eq!(launch.id.as_str(), "user-1");
    assert!(
        RUNTIME.get().is_some(),
        "the real initializer/bootstrap must have completed"
    );
    assert_eq!(before_debug, format!("{before:?}"));
    let after = std::mem::ManuallyDrop::new(crate::orchestrator::ContextBundleStore::new());
    assert!(
        format!("{after:?}").contains(&format!("orchestrator-{}-2", std::process::id())),
        "only the before store and CommandChat may consume an instance identity"
    );
}
