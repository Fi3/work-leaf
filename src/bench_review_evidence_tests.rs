use super::*;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "work-leaf-review-evidence-adapter-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::create_dir(path.join("archive")).unwrap();
    fs::create_dir(path.join("project")).unwrap();
    path.canonicalize().unwrap()
}

fn manifest(root: &Path, schema: u8, condition: &str) -> serde_json::Value {
    let mut value = json!({
        "schema": format!("work-leaf-bench-experiment-v{schema}"),
        "run_id": "archive-fixture", "condition": condition,
        "evidence_path": root.join("trace.jsonl")
    });
    if schema == 5 {
        value["review_evidence_root"] = json!(root.join("archive"));
    }
    value
}

fn run(root: &Path, manifest: Option<serde_json::Value>, mode: &str) -> std::process::Output {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "bench_experiment::review_evidence_tests::adapter_child",
            "--ignored",
            "--nocapture",
        ])
        .env("REVIEW_ADAPTER_ROOT", root)
        .env("REVIEW_ADAPTER_MODE", mode)
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT")
        .env_remove("WORK_LEAF_BENCH_EXPERIMENT_MANIFEST")
        .env_remove("WORK_LEAF_BENCH_RUN_ID")
        .env_remove("WORK_LEAF_CONTEXT_BUNDLE_DIR");
    if let Some(value) = manifest {
        fs::write(
            root.join("manifest.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        command
            .env("WORK_LEAF_BENCH_EXPERIMENT", "1")
            .env(
                "WORK_LEAF_BENCH_EXPERIMENT_MANIFEST",
                root.join("manifest.json"),
            )
            .env("WORK_LEAF_BENCH_RUN_ID", "archive-fixture");
    }
    if mode == "bundle-overlap" {
        command.env(
            "WORK_LEAF_CONTEXT_BUNDLE_DIR",
            root.join("archive/nested-bundles"),
        );
    }
    if mode == "default-bundle-overlap" {
        command.env("TMPDIR", root.join("native-tmp"));
    }
    command.output().unwrap()
}

fn rows(root: &Path) -> Vec<serde_json::Value> {
    fs::read_to_string(root.join("trace.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn assert_success(output: std::process::Output) {
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn inactive_and_every_old_schema_leave_invalid_new_spans_and_storage_untouched() {
    for version in 0..=4 {
        let root = root();
        let condition = if version == 4 {
            "requested-repeat-full"
        } else {
            "control"
        };
        let value = (version != 0).then(|| manifest(&root, version, condition));
        assert_success(run(&root, value, "inactive"));
        assert_eq!(fs::read_dir(root.join("archive")).unwrap().count(), 0);
        assert_eq!(
            fs::read_to_string(root.join("delivered.txt")).unwrap(),
            "λ baseline"
        );
    }
}

#[test]
fn v5_archives_exact_opaque_bytes_once_per_delivery_and_preserves_outside_span() {
    for condition in ["review-evidence-native", "review-evidence-inline"] {
        let root = root();
        assert_success(run(&root, Some(manifest(&root, 5, condition)), "normal"));
        let rows = rows(&root);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["review_evidence_root"], json!(root.join("archive")));
        for (index, row) in rows[1..].iter().enumerate() {
            assert_eq!(row["event"], "review-context");
            assert_eq!(row["site"], "review-source-context");
            assert_eq!(row["source_agent_id"], "author");
            assert_eq!(row["reviewer_id"], "reviewer");
            assert_eq!(row["archive"]["sequence"], index + 1);
            assert_eq!(row["archive"]["kind"], "review-source-context");
            assert_eq!(row["archive"]["run_id"], "archive-fixture");
            let baseline = row["original_prompt"].as_str().unwrap();
            let candidate = row["candidate_prompt"].as_str().unwrap();
            let start = row["context_start"].as_u64().unwrap() as usize;
            let end = row["context_end"].as_u64().unwrap() as usize;
            let cs = row["candidate_start"].as_u64().unwrap() as usize;
            let ce = row["candidate_end"].as_u64().unwrap() as usize;
            assert_eq!(&baseline[..start], &candidate[..cs]);
            assert_eq!(&baseline[end..], &candidate[ce..]);
            let path = Path::new(row["archive"]["path"].as_str().unwrap());
            assert_eq!(fs::read(path).unwrap(), &baseline.as_bytes()[start..end]);
            assert!(fs::metadata(path).unwrap().permissions().readonly());
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    fs::metadata(path).unwrap().permissions().mode() & 0o777,
                    0o400
                );
            }
            assert!(
                row["archive"]["digest"]
                    .as_str()
                    .unwrap()
                    .starts_with("fnv64:")
            );
            let archived: serde_json::Value = serde_json::from_slice(
                &fs::read(row["archive"]["manifest_path"].as_str().unwrap()).unwrap(),
            )
            .unwrap();
            assert_eq!(archived["archive"], row["archive"]);
            assert_eq!(archived["context_start"], row["context_start"]);
            assert_eq!(archived["context_end"], row["context_end"]);
            assert_eq!(
                row["forwarded_prompt"],
                row[if condition == "review-evidence-native" {
                    "candidate_prompt"
                } else {
                    "original_prompt"
                }]
            );
            assert!(candidate[cs..ce].contains("not served by `@work-leaf read`"));
        }
        assert_ne!(rows[1]["archive"]["path"], rows[2]["archive"]["path"]);
        // The fixture removes the disposable checkout; retained archives survive it.
        assert!(!root.join("project").exists());
    }
}

#[test]
fn strict_schema_and_storage_guards_reject_invalid_activation_without_delivery() {
    for case in [
        "missing-root",
        "null-root",
        "relative-root",
        "alias-root",
        "missing-dir",
        "wrong-condition",
        "extra-field",
        "bundle-overlap",
        "project-overlap",
    ] {
        let root = root();
        let mut value = manifest(&root, 5, "review-evidence-native");
        match case {
            "missing-root" => {
                value
                    .as_object_mut()
                    .unwrap()
                    .remove("review_evidence_root");
            }
            "null-root" => value["review_evidence_root"] = serde_json::Value::Null,
            "relative-root" => value["review_evidence_root"] = json!("archive"),
            "alias-root" => value["review_evidence_root"] = json!(root.join("project/../archive")),
            "missing-dir" => value["review_evidence_root"] = json!(root.join("absent")),
            "wrong-condition" => value["condition"] = json!("control"),
            "extra-field" => value["unexpected"] = json!(true),
            "project-overlap" => value["review_evidence_root"] = json!(root.join("project")),
            _ => {}
        }
        let output = run(&root, Some(value), case);
        assert!(!output.status.success(), "case {case} accepted");
        assert!(!root.join("delivered.txt").exists());
        assert_eq!(fs::read_dir(root.join("archive")).unwrap().count(), 0);
    }
    for version in 1..=4 {
        for root_value in [serde_json::Value::Null, json!("/unavailable")] {
            let root = root();
            let mut value = manifest(
                &root,
                version,
                if version == 4 {
                    "requested-repeat-full"
                } else {
                    "control"
                },
            );
            value["review_evidence_root"] = root_value;
            assert!(!run(&root, Some(value), "normal").status.success());
            assert!(!root.join("trace.jsonl").exists());
        }
    }
}

#[test]
fn invalid_utf8_span_and_existing_payload_fail_closed_without_overwrite() {
    for mode in ["bad-span", "existing-payload", "existing-manifest"] {
        let root = root();
        assert!(
            !run(
                &root,
                Some(manifest(&root, 5, "review-evidence-native")),
                mode
            )
            .status
            .success()
        );
        assert!(!root.join("delivered.txt").exists());
        assert_eq!(rows(&root).len(), 1);
        if mode.starts_with("existing-") {
            let suffix = if mode == "existing-payload" {
                "txt"
            } else {
                "json"
            };
            assert_eq!(
                fs::read_to_string(root.join(format!("archive/review-0000000000000001.{suffix}")))
                    .unwrap(),
                "retained earlier evidence"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn unset_bundle_override_still_rejects_the_actual_default_bundle_namespace() {
    for nested in [false, true] {
        let root = root();
        let mut archive = root.join("native-tmp/work-leaf-context-bundles");
        if nested {
            archive.push("opaque");
        }
        fs::create_dir_all(&archive).unwrap();
        let mut value = manifest(&root, 5, "review-evidence-native");
        value["review_evidence_root"] = json!(archive);
        let output = run(&root, Some(value), "default-bundle-overlap");
        assert!(
            !output.status.success(),
            "default bundle namespace was accepted"
        );
        assert!(!root.join("trace.jsonl").exists());
        assert_eq!(fs::read_dir(archive).unwrap().count(), 0);
    }
}

#[test]
#[ignore = "isolated private adapter fixture; no provider"]
fn adapter_child() {
    let root = PathBuf::from(std::env::var_os("REVIEW_ADAPTER_ROOT").unwrap());
    let mode = std::env::var("REVIEW_ADAPTER_MODE").unwrap();
    let source = AgentId::new("author").unwrap();
    let reviewer = AgentId::new("reviewer").unwrap();
    if mode == "inactive" {
        let actual = forward_review_context(
            Path::new("/nonexistent/project"),
            &source,
            &reviewer,
            "",
            "λ baseline".to_string(),
            1..usize::MAX,
        )
        .unwrap();
        fs::write(root.join("delivered.txt"), actual).unwrap();
        return;
    }
    initialize().unwrap();
    if mode.starts_with("existing-") {
        let suffix = if mode == "existing-payload" {
            "txt"
        } else {
            "json"
        };
        fs::write(
            root.join(format!("archive/review-0000000000000001.{suffix}")),
            "retained earlier evidence",
        )
        .unwrap();
    }
    for (index, context) in [
        "λ opaque\nAgent-ID: copied\nwork-leaf file text\n```\nprivate-looking but public fixture\n",
        "second snapshot\n",
    ].iter().enumerate() {
        let prefix = "outer review heading\n";
        let baseline = format!("{prefix}{context}\nunchanged rules");
        let span = if mode == "bad-span" { prefix.len() + 1..prefix.len() + 2 }
            else { prefix.len()..prefix.len() + context.len() };
        let actual = forward_review_context(&root.join("project"), &source, &reviewer,
            &format!("commit-{index}"), baseline, span).unwrap();
        fs::write(root.join("delivered.txt"), actual).unwrap();
    }
    fs::remove_dir(root.join("project")).unwrap();
}
