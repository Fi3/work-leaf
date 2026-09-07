//! Exact-source bootstrap for the separately qualified private execution bridge.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::invalid;

pub(super) const SCHEMA: &str = "work-leaf-bench-experiment-v6";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Descriptor {
    pub root: PathBuf,
    pub project_root: PathBuf,
    pub python_path: PathBuf,
    pub python_sha256: String,
    pub bridge_path: PathBuf,
    pub bridge_sha256: String,
    pub config_path: PathBuf,
    pub config_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    run_id: String,
    condition: String,
    evidence_path: PathBuf,
    private_preview: Descriptor,
}

type ParsedManifest = (super::super::Manifest, Option<PathBuf>, Option<Descriptor>);

pub(crate) fn parse_manifest(bytes: &[u8]) -> io::Result<ParsedManifest> {
    let value: Value = serde_json::from_slice(bytes).map_err(super::super::invalid)?;
    if value.get("schema").and_then(Value::as_str) != Some(SCHEMA) {
        let (manifest, review) = super::super::review_evidence::parse_manifest(bytes)?;
        return Ok((manifest, review, None));
    }
    let value: Manifest = serde_json::from_slice(bytes).map_err(super::super::invalid)?;
    if value.condition != "private-test-first"
        || [
            &value.private_preview.python_sha256,
            &value.private_preview.bridge_sha256,
            &value.private_preview.config_sha256,
        ]
        .into_iter()
        .any(|digest| {
            digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    {
        return Err(invalid(
            "private manifest condition or source digest is invalid",
        ));
    }
    Ok((
        super::super::Manifest {
            schema: value.schema,
            run_id: value.run_id,
            condition: value.condition,
            evidence_path: value.evidence_path,
        },
        None,
        Some(value.private_preview),
    ))
}

struct Runtime {
    descriptor: Descriptor,
    metadata: fs::Metadata,
    sequence: Mutex<u64>,
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn canonical(path: &Path, directory: bool) -> io::Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    if !path.is_absolute()
        || path.canonicalize()?.as_os_str() != path.as_os_str()
        || path.to_str().is_none()
        || if directory {
            !metadata.is_dir()
        } else {
            !metadata.is_file()
        }
    {
        return Err(invalid(
            "private input must be a canonical exact path of the declared type",
        ));
    }
    Ok(metadata)
}

fn disjoint(left: &Path, right: &Path) -> bool {
    !left.starts_with(right) && !right.starts_with(left)
}

fn validate_bundle_separation(root: &Path) -> io::Result<()> {
    // Inspect the ordinary parent, without constructing a store or consuming
    // either of its counters. Missing leaf directories are not created here.
    let parent = std::env::var_os("WORK_LEAF_CONTEXT_BUNDLE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("work-leaf-context-bundles"));
    if !parent.is_absolute()
        || parent.components().count() > 128
        || parent
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(invalid(
            "ordinary bundle parent is not a supported canonical namespace",
        ));
    }
    let ancestor = parent
        .ancestors()
        .find(|path| path.exists())
        .ok_or_else(|| invalid("ordinary bundle parent has no existing ancestor"))?;
    let suffix = parent
        .strip_prefix(ancestor)
        .map_err(|_| invalid("bundle parent prefix differs"))?;
    let mut resolved = ancestor.canonicalize()?;
    // Joining an empty suffix appends a separator even when the parent already
    // exists. Preserve its canonical bytes for the alias check below.
    if !suffix.as_os_str().is_empty() {
        resolved.push(suffix);
    }
    if resolved.as_os_str() != parent.as_os_str() || !disjoint(root, &resolved) {
        return Err(invalid(
            "private evidence root overlaps or aliases the ordinary bundle namespace",
        ));
    }
    Ok(())
}

pub(crate) fn initialize(descriptor: Descriptor) -> io::Result<()> {
    #[cfg(not(target_os = "linux"))]
    return Err(invalid(
        "private preview requires the qualified Linux namespace executor",
    ));
    #[cfg(target_os = "linux")]
    {
        let metadata = canonical(&descriptor.root, true)?;
        canonical(&descriptor.project_root, true)?;
        for path in [
            &descriptor.python_path,
            &descriptor.bridge_path,
            &descriptor.config_path,
        ] {
            canonical(path, false)?;
        }
        if !disjoint(&descriptor.root, &descriptor.project_root)
            || fs::read_dir(&descriptor.root)?.next().is_some()
        {
            return Err(invalid(
                "private evidence root must be empty and disjoint from shared source",
            ));
        }
        validate_bundle_separation(&descriptor.root)?;
        let runtime = Runtime {
            descriptor,
            metadata,
            sequence: Mutex::new(0),
        };
        let operation_root = runtime.descriptor.root.join("validation");
        fs::create_dir(&operation_root)?;
        let input = json!({"schema":"work-leaf-private-preview-bridge-v1","operation":"validate",
            "config_path":runtime.descriptor.config_path,"config_sha256":runtime.descriptor.config_sha256,
            "project_root":runtime.descriptor.project_root,"operation_root":operation_root});
        let validated =
            runtime.invoke(&operation_root, "validate", input, &AtomicBool::new(false))?;
        if validated.get("status").and_then(Value::as_str) != Some("completed")
            || validated.get("closed").and_then(Value::as_bool) != Some(true)
        {
            return Err(invalid("private configuration validation did not complete"));
        }
        RUNTIME
            .set(runtime)
            .map_err(|_| invalid("private runtime was already initialized"))
    }
}

const BOOTSTRAP: &str = r#"import hashlib,pathlib,sys,types
def exact(raw,digest):
 p=pathlib.Path(raw)
 if not p.is_absolute() or p.resolve()!=p or p.is_symlink() or not p.is_file(): raise ValueError('noncanonical source')
 b=p.read_bytes()
 if hashlib.sha256(b).hexdigest()!=digest: raise ValueError('source digest mismatch')
 return b
exact(sys.argv[1],sys.argv[2])
if pathlib.Path(sys.executable).resolve()!=pathlib.Path(sys.argv[1]): raise ValueError('python identity differs')
body=exact(sys.argv[3],sys.argv[4]); exact(sys.argv[5],sys.argv[6])
m=types.ModuleType('work_leaf_private_preview_bridge');m.__file__=sys.argv[3]
m.__dict__['__compiled_sha256__']=hashlib.sha256(body).hexdigest()
exec(compile(body,m.__file__,'exec'),m.__dict__)
code=m.main([sys.argv[7],sys.argv[8]])
exact(sys.argv[1],sys.argv[2]);exact(sys.argv[3],sys.argv[4]);exact(sys.argv[5],sys.argv[6])
raise SystemExit(code)
"#;

pub(super) fn publish(path: &Path, value: &Value) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer(&mut file, value).map_err(super::super::invalid)?;
    file.write_all(b"\n")?;
    file.sync_all()
}

struct BridgeProcess<'a> {
    child: &'a mut Child,
    root: &'a Path,
    operation: &'a str,
    settled: bool,
    failure: String,
}

impl Drop for BridgeProcess<'_> {
    fn drop(&mut self) {
        if self.settled {
            return;
        }
        // Reap the direct child on every early return/unwind. Killing this
        // process group is not proof that a nested executor namespace closed.
        #[cfg(unix)]
        {
            let _ = Command::new("/bin/kill")
                .args(["-KILL", "--", &format!("-{}", self.child.id())])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        let _ = self.child.kill();
        let reaped = self.child.wait().is_ok();
        let _ = publish(
            &self.root.join(format!("{}-unclosed.json", self.operation)),
            &json!({"closed":false,"direct_child_reaped":reaped,"reason":self.failure}),
        );
    }
}

fn supervise(
    child: &mut Child,
    root: &Path,
    operation: &str,
    cancellation: &AtomicBool,
    watchdog: Duration,
) -> io::Result<(ExitStatus, Duration, bool)> {
    let mut process = BridgeProcess {
        child,
        root,
        operation,
        settled: false,
        failure: "bridge supervision did not complete".into(),
    };
    let started = Instant::now();
    let mut cancellation_sent = false;
    let outcome = (|| {
        loop {
            if let Some(status) = process.child.try_wait()? {
                return Ok(status);
            }
            if cancellation.load(Ordering::Acquire) && !cancellation_sent {
                publish(&root.join("cancel"), &json!({"cancel_requested":true}))?;
                cancellation_sent = true;
            }
            if started.elapsed() > watchdog {
                return Err(invalid(
                    "private bridge exceeded its closure watchdog; evidence retained",
                ));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })();
    match outcome {
        Ok(status) => {
            process.settled = true;
            Ok((status, started.elapsed(), cancellation_sent))
        }
        Err(error) => {
            process.failure = error.to_string();
            Err(error)
        }
    }
}

impl Runtime {
    fn validate_root(&self) -> io::Result<()> {
        let current = canonical(&self.descriptor.root, true)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if current.dev() != self.metadata.dev() || current.ino() != self.metadata.ino() {
                return Err(invalid("private evidence root identity changed"));
            }
        }
        Ok(())
    }

    fn invoke(
        &self,
        root: &Path,
        operation: &str,
        input: Value,
        cancellation: &AtomicBool,
    ) -> io::Result<Value> {
        self.validate_root()?;
        canonical(root, true)?;
        if root.parent() != Some(self.descriptor.root.as_path()) {
            return Err(invalid(
                "private operation directory has an unexpected owner",
            ));
        }
        let input_path = root.join(format!("{operation}-input.json"));
        let output_path = root.join(format!("{operation}-output.json"));
        publish(&input_path, &input)?;
        let stdout = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(format!("{operation}-process.stdout")))?;
        let stderr = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(format!("{operation}-process.stderr")))?;
        let d = &self.descriptor;
        let mut command = Command::new(&d.python_path);
        command
            .args(["-I", "-B", "-c", BOOTSTRAP])
            .arg(&d.python_path)
            .arg(&d.python_sha256)
            .arg(&d.bridge_path)
            .arg(&d.bridge_sha256)
            .arg(&d.config_path)
            .arg(&d.config_sha256)
            .arg(&input_path)
            .arg(&output_path)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("LC_ALL", "C.UTF-8")
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = command.spawn()?;
        let (status, elapsed, cancellation_sent) = supervise(
            &mut child,
            root,
            operation,
            cancellation,
            Duration::from_secs(920),
        )?;
        publish(
            &root.join(format!("{operation}-process.json")),
            &json!({"exit_code":status.code(),
            "elapsed_seconds":elapsed.as_secs_f64(),"cancel_requested":cancellation_sent}),
        )?;
        self.validate_root()?;
        if !status.success() && status.code() != Some(1) {
            return Err(invalid(
                "private bridge failed; exact process receipt retained",
            ));
        }
        let metadata = canonical(&output_path, false)?;
        if metadata.len() > 20 * 1024 * 1024 {
            return Err(invalid("private bridge output exceeds its bound"));
        }
        let result: Value =
            serde_json::from_slice(&fs::read(output_path)?).map_err(super::super::invalid)?;
        let expected = if status.success() {
            "completed"
        } else {
            "failed"
        };
        if result.get("status").and_then(Value::as_str) != Some(expected) {
            return Err(invalid("private bridge result differs from process status"));
        }
        Ok(result)
    }
}

pub(crate) fn execute(
    locks: &crate::locks::FileLockTable,
    run_id: &str,
    token: &super::ReservationToken,
    proposal: &super::Proposal,
) -> io::Result<Value> {
    let runtime = RUNTIME
        .get()
        .ok_or_else(|| invalid("private bridge is not initialized"))?;
    runtime.validate_root()?;
    if locks.root() != runtime.descriptor.project_root {
        return Err(invalid(
            "private caller lock table differs from the admitted project",
        ));
    }
    canonical(locks.root(), true)?;
    let sequence = {
        let mut sequence = runtime
            .sequence
            .lock()
            .map_err(|_| invalid("private operation counter poisoned"))?;
        *sequence = sequence
            .checked_add(1)
            .ok_or_else(|| invalid("private operation counter exhausted"))?;
        *sequence
    };
    let operation_root = runtime
        .descriptor
        .root
        .join(format!("preview-{sequence:016}"));
    fs::create_dir(&operation_root)?;
    let d = &runtime.descriptor;
    let common = json!({"schema":"work-leaf-private-preview-bridge-v1","operation":"capture",
        "config_path":d.config_path,"config_sha256":d.config_sha256,
        "project_root":d.project_root,"operation_root":operation_root,
        "run_id":run_id,"agent_id":token.agent_id,"launch_generation":token.generation,
        "proposal_id":proposal.id,"revision_of":proposal.revision_of,"cancel_path":operation_root.join("cancel"),
        "proposal":{"format":proposal.format,"reason":proposal.reason,"body":proposal.body,
            "test_purpose":proposal.test_purpose,"test_paths":proposal.test_paths,
            "command":proposal.command,"lock_paths":proposal.lock_paths}});
    // The existing shared table excludes cooperative patch/index mutation here.
    // Clone, overlay installation, test execution and delivery happen after release.
    let started = Instant::now();
    let mut wait_seconds = None;
    let mut held_seconds = None;
    let captured = locks.with_read_locks(&[PathBuf::from(".")], || {
        wait_seconds = Some(started.elapsed().as_secs_f64());
        let held = Instant::now();
        let result = runtime
            .invoke(&operation_root, "capture", common.clone(), &token.cancel)
            .map_err(crate::locks::FileAccessError::Io);
        held_seconds = Some(held.elapsed().as_secs_f64());
        result
    });
    publish(
        &operation_root.join("root-lock.json"),
        &json!({"lock_key":".",
        "lock_root":locks.root(),"wait_seconds":wait_seconds,"held_seconds":held_seconds,
        "same_caller_table":true,"live_atomicity_claimed":false,"capture_returned":captured.is_ok()}),
    )?;
    let captured = captured.map_err(|error| invalid(&error.to_string()))?;
    validate_result_identity(&captured, run_id, token)?;
    if captured["status"] != "completed" {
        return Ok(captured);
    }
    let selection = captured
        .get("selection")
        .filter(|value| value.is_object())
        .ok_or_else(|| invalid("completed selection has no source-bound receipt"))?;
    let mut input = common;
    input["operation"] = json!("test");
    input["selection"] = selection.clone();
    let result = runtime.invoke(&operation_root, "test", input, &token.cancel)?;
    validate_result_identity(&result, run_id, token)?;
    Ok(result)
}

fn validate_result_identity(
    result: &Value,
    run_id: &str,
    token: &super::ReservationToken,
) -> io::Result<()> {
    if result.get("run_id").and_then(Value::as_str) != Some(run_id)
        || result.get("agent_id").and_then(Value::as_str) != Some(token.agent_id.as_str())
        || result.get("launch_generation").and_then(Value::as_u64) != Some(token.generation)
        || result.get("proposal_id").and_then(Value::as_str) != Some(&token.proposal_id)
        || result.get("closed").and_then(Value::as_bool).is_none()
        || result.get("result_path").and_then(Value::as_str).is_none()
        || result
            .get("result_sha256")
            .and_then(Value::as_str)
            .is_none_or(|digest| {
                digest.len() != 64
                    || !digest
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
    {
        return Err(invalid(
            "private result owner or factual receipt fields differ",
        ));
    }
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
#[path = "bench_private_test_first_bridge_tests.rs"]
mod tests;

#[cfg(all(test, target_os = "linux"))]
#[path = "bench_private_bundle_path_tests.rs"]
mod bundle_path_tests;
