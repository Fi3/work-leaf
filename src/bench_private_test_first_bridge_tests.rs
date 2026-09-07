use super::*;
use std::process::Child;
use std::sync::atomic::AtomicU64;

fn root() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "work-leaf-private-supervisor-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn child(root: &Path, script: &str) -> Child {
    use std::os::unix::process::CommandExt;
    Command::new("/bin/sh")
        .arg("-c")
        .arg(script)
        .current_dir(root)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .unwrap()
}

#[test]
fn cancellation_publication_failure_and_watchdog_reap_parent_without_claiming_namespace_closure() {
    for blocked_cancel in [true, false] {
        let root = root();
        if blocked_cancel {
            fs::create_dir(root.join("cancel")).unwrap();
        }
        let mut child = child(&root, "sleep 30 & wait");
        let result = supervise(
            &mut child,
            &root,
            "test",
            &AtomicBool::new(blocked_cancel),
            Duration::from_millis(80),
        );
        assert!(result.is_err());
        assert!(child.try_wait().unwrap().is_some());
        let receipt: Value =
            serde_json::from_slice(&fs::read(root.join("test-unclosed.json")).unwrap()).unwrap();
        assert_eq!(receipt["closed"], false);
        assert_eq!(receipt["direct_child_reaped"], true);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn cooperative_cancel_is_exact_once_and_does_not_replace_a_normal_result() {
    let root = root();
    let mut child = child(&root, "while test ! -f cancel; do sleep 0.01; done; exit 1");
    let (status, _, sent) = supervise(
        &mut child,
        &root,
        "test",
        &AtomicBool::new(true),
        Duration::from_secs(2),
    )
    .unwrap();
    assert_eq!(status.code(), Some(1));
    assert!(sent);
    assert!(!root.join("test-unclosed.json").exists());
    let receipt: Value = serde_json::from_slice(&fs::read(root.join("cancel")).unwrap()).unwrap();
    assert_eq!(receipt, json!({"cancel_requested":true}));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn result_identity_includes_exact_run_and_nonaliased_generation() {
    let token = super::super::ReservationToken {
        agent_id: crate::agent::AgentId::new("owner").unwrap(),
        generation: 7,
        proposal_id: "proposal".into(),
        cancel: std::sync::Arc::new(AtomicBool::new(false)),
    };
    let mut value = json!({"run_id":"run-a","agent_id":"owner","launch_generation":7,
        "proposal_id":"proposal","closed":true,"result_path":"/receipt",
        "result_sha256":"a".repeat(64)});
    assert!(validate_result_identity(&value, "run-a", &token).is_ok());
    assert!(validate_result_identity(&value, "run-b", &token).is_err());
    value["launch_generation"] = json!(true);
    assert!(validate_result_identity(&value, "run-a", &token).is_err());
    value["launch_generation"] = json!(7);
    value["result_sha256"] = Value::Null;
    assert!(validate_result_identity(&value, "run-a", &token).is_err());
}
