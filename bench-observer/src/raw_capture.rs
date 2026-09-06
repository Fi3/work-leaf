//! Opt-in app-server notification metadata; no model-facing request changes.

use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use serde_json::{Value, json};

use super::{ObserverError, ObserverResult, ProviderUsageGrace};

pub(super) const ENV: &str = "WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE";

const ARTIFACTS: [&str; 3] = [
    "raw-response-usage.json",
    "raw-response-rewrites.jsonl",
    "client-to-server.forwarded.raw",
];

pub(super) fn start_metadata(start: &impl Serialize, enabled: bool) -> ObserverResult<Value> {
    let mut value = serde_json::to_value(start)?;
    if enabled {
        value["raw_response_usage"] = json!(true);
    }
    Ok(value)
}

pub(super) fn end_metadata(
    end: &impl Serialize,
    enabled: bool,
    with_grace: bool,
    start_path: &Path,
    directory: &Path,
) -> ObserverResult<Value> {
    let mut value = serde_json::to_value(end)?;
    if enabled {
        let mut hashes = serde_json::Map::new();
        for name in ARTIFACTS
            .into_iter()
            .chain(with_grace.then_some("provider-usage-grace.jsonl"))
        {
            hashes.insert(
                name.to_string(),
                json!(super::sha256_file(&directory.join(name))?),
            );
        }
        value["raw_response_usage_sha256"] = Value::Object(hashes);
        value["raw_response_usage_start_sha256"] = json!(super::sha256_file(start_path)?);
    }
    Ok(value)
}

pub(super) fn verify_capture(invocation_directory: &Path, directory: &Path) -> ObserverResult<()> {
    let invalid =
        |reason: &str| ObserverError::new(format!("raw response capture provenance: {reason}"));
    let start_bytes = std::fs::read(invocation_directory.join("start.json"))?;
    let start: Value = serde_json::from_slice(&start_bytes)?;
    let end: Value =
        serde_json::from_slice(&std::fs::read(invocation_directory.join("end.json"))?)?;
    let active = match start.get("raw_response_usage") {
        None | Some(Value::Bool(false)) => false,
        Some(Value::Bool(true)) => true,
        Some(_) => return Err(invalid("invalid opt-in marker")),
    };
    if !active {
        if end.get("raw_response_usage_sha256").is_some()
            || end.get("raw_response_usage_start_sha256").is_some()
            || directory.join(ARTIFACTS[0]).exists()
            || directory.join(ARTIFACTS[1]).exists()
        {
            return Err(invalid("raw metadata exists without its opt-in marker"));
        }
        // Legacy grace captures also have a forwarded stream, without raw metadata.
        return Ok(());
    }
    if end
        .get("raw_response_usage_start_sha256")
        .and_then(Value::as_str)
        != Some(super::sha256_bytes(&start_bytes).as_str())
    {
        return Err(invalid("start metadata SHA-256 mismatch or missing digest"));
    }
    let with_grace = start
        .get("provider_usage_grace_ms")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("missing usage-grace configuration"))?
        > 0;
    let hashes = end
        .get("raw_response_usage_sha256")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("missing artifact digest inventory"))?;
    if hashes.len() != ARTIFACTS.len() + usize::from(with_grace) {
        return Err(invalid(
            "incomplete or unexpected artifact digest inventory",
        ));
    }
    let read_verified = |name: &str| -> ObserverResult<Vec<u8>> {
        let expected = hashes
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("missing artifact digest"))?;
        let bytes = std::fs::read(directory.join(name))?;
        if super::sha256_bytes(&bytes) != expected {
            return Err(invalid(&format!("{name} SHA-256 mismatch")));
        }
        Ok(bytes)
    };
    if with_grace {
        read_verified("provider-usage-grace.jsonl")?;
    }
    let settings: Value = serde_json::from_slice(&read_verified(ARTIFACTS[0])?)?;
    if settings != capture_settings() {
        return Err(invalid(
            "settings do not describe the allowed metadata policy",
        ));
    }
    let rewrite_bytes = read_verified(ARTIFACTS[1])?;
    let rewrite_text = std::str::from_utf8(&rewrite_bytes)
        .map_err(|_| invalid("rewrite decisions are not UTF-8"))?;
    let mut decisions = rewrite_text.lines();
    let forwarded = read_verified(ARTIFACTS[2])?;
    let original = std::fs::read(directory.join("client-to-server.raw"))?;
    let mut offset = 0usize;
    for frame in original.split_inclusive(|byte| *byte == b'\n') {
        let rewritten = rewrite_frame(frame);
        let expected = rewritten.as_deref().unwrap_or(frame);
        let end = offset
            .checked_add(expected.len())
            .ok_or_else(|| invalid("stream length overflow"))?;
        if forwarded.get(offset..end) != Some(expected) {
            return Err(invalid(
                "forwarded bytes differ from the allowed metadata rewrite",
            ));
        }
        offset = end;
        if let Some(mut expected_record) = rewrite_record(frame, expected, rewritten.is_some()) {
            let line = decisions
                .next()
                .ok_or_else(|| invalid("missing rewrite decision"))?;
            let mut actual: Value = serde_json::from_str(line)?;
            if actual
                .get("observed_monotonic_ns")
                .and_then(Value::as_u64)
                .is_none()
            {
                return Err(invalid("missing or invalid rewrite timestamp"));
            }
            actual
                .as_object_mut()
                .ok_or_else(|| invalid("invalid rewrite decision"))?
                .remove("observed_monotonic_ns");
            expected_record
                .as_object_mut()
                .expect("rewrite record object")
                .remove("observed_monotonic_ns");
            if actual != expected_record {
                return Err(invalid(
                    "rewrite decision does not match its original and forwarded frame",
                ));
            }
        }
    }
    if offset != forwarded.len() || decisions.next().is_some() {
        return Err(invalid("extra forwarded bytes or rewrite decisions"));
    }
    Ok(())
}

fn capture_settings() -> Value {
    json!({
        "schema_version": 1,
        "enabled": true,
        "environment_variable": ENV,
        "request_metadata_overrides": {
            "initialize.params.capabilities.experimentalApi": true,
            "initialize.params.capabilities.optOutNotificationMethods.append": "rawResponseItem/completed",
            "thread/start.params.experimentalRawEvents": true
        },
        "original_client_stream": "client-to-server.raw",
        "forwarded_client_stream": "client-to-server.forwarded.raw",
        "rewrite_decisions": "raw-response-rewrites.jsonl",
        "interrupt_policy_unchanged": true
    })
}

pub(super) fn enabled_from_environment() -> ObserverResult<bool> {
    enabled(std::env::var_os(ENV).as_deref())
}

fn enabled(value: Option<&OsStr>) -> ObserverResult<bool> {
    match value {
        None => Ok(false),
        Some(value) if value == "0" => Ok(false),
        Some(value) if value == "1" => Ok(true),
        Some(_) => Err(ObserverError::new(format!("{ENV} must be 0 or 1"))),
    }
}

pub(super) struct CaptureFiles {
    forwarded: File,
    rewrites: File,
    grace_decisions: Option<File>,
}

impl CaptureFiles {
    pub(super) fn create(directory: &Path, with_grace: bool) -> ObserverResult<Self> {
        super::write_json_atomic(
            &directory.join("raw-response-usage.json"),
            &capture_settings(),
        )?;
        Ok(Self {
            forwarded: super::create_private_file(
                &directory.join("client-to-server.forwarded.raw"),
            )?,
            rewrites: super::create_private_file(&directory.join("raw-response-rewrites.jsonl"))?,
            grace_decisions: if with_grace {
                Some(super::create_private_file(
                    &directory.join("provider-usage-grace.jsonl"),
                )?)
            } else {
                None
            },
        })
    }
}

pub(super) fn pump_stdin<W: Write>(
    mut destination: W,
    mut capture: File,
    mut chunks: File,
    mut files: CaptureFiles,
    done: Arc<AtomicBool>,
    usage_grace: Option<Arc<ProviderUsageGrace>>,
) -> io::Result<()> {
    let mut buffer = [0_u8; 16 * 1024];
    let mut pending = Vec::new();
    let mut offset = 0;
    while !done.load(Ordering::Acquire) {
        let Some(read) = super::read_stdin_chunk(&mut buffer)? else {
            continue;
        };
        if read == 0 {
            break;
        }
        super::capture_stream_chunk(&buffer[..read], &mut capture, &mut chunks, &mut offset)?;
        append_chunk(&mut pending, &buffer[..read], &mut |frame| {
            forward_frame(
                frame,
                &mut destination,
                &mut files,
                &done,
                usage_grace.as_deref(),
            )
        })?;
    }
    if !pending.is_empty() {
        forward_frame(
            &pending,
            &mut destination,
            &mut files,
            &done,
            usage_grace.as_deref(),
        )?;
    }
    Ok(())
}

fn append_chunk(
    pending: &mut Vec<u8>,
    chunk: &[u8],
    forward: &mut impl FnMut(&[u8]) -> io::Result<()>,
) -> io::Result<()> {
    // Only scan new bytes: a large frame arriving in small chunks stays linear.
    for segment in chunk.split_inclusive(|byte| *byte == b'\n') {
        pending.extend_from_slice(segment);
        if segment.ends_with(b"\n") {
            forward(pending)?;
            pending.clear();
        }
    }
    Ok(())
}

fn forward_frame<W: Write>(
    frame: &[u8],
    destination: &mut W,
    files: &mut CaptureFiles,
    done: &AtomicBool,
    usage_grace: Option<&ProviderUsageGrace>,
) -> io::Result<()> {
    let rewritten = rewrite_frame(frame);
    let forwarded = rewritten.as_deref().unwrap_or(frame);
    if let Some(record) = rewrite_record(frame, forwarded, rewritten.is_some()) {
        serde_json::to_writer(&mut files.rewrites, &record)?;
        files.rewrites.write_all(b"\n")?;
        files.rewrites.flush()?;
    }
    if let (Some(grace), Some(decisions)) = (usage_grace, &mut files.grace_decisions) {
        return super::forward_app_server_client_frame(
            forwarded,
            destination,
            &mut files.forwarded,
            decisions,
            done,
            grace,
        );
    }
    destination.write_all(forwarded)?;
    destination.flush()?;
    files.forwarded.write_all(forwarded)?;
    files.forwarded.flush()
}

fn rewrite_record(frame: &[u8], forwarded: &[u8], changed: bool) -> Option<Value> {
    let value: Value = serde_json::from_slice(frame).ok()?;
    if !matches!(
        value.get("method")?.as_str()?,
        "initialize" | "thread/start"
    ) {
        return None;
    }
    Some(json!({
        "method": value.get("method"),
        "id": value.get("id"),
        "changed": changed,
        "original_sha256": super::sha256_bytes(frame),
        "forwarded_sha256": super::sha256_bytes(forwarded),
        "original_bytes": frame.len(),
        "forwarded_bytes": forwarded.len(),
        "observed_monotonic_ns": super::monotonic_time_ns()
    }))
}

fn rewrite_frame(frame: &[u8]) -> Option<Vec<u8>> {
    let mut value: Value = serde_json::from_slice(frame).ok()?;
    value.get("id")?;
    if !matches!(
        value.get("method")?.as_str()?,
        "initialize" | "thread/start"
    ) {
        return None;
    }
    let original = value.clone();
    match value.get("method")?.as_str()? {
        "initialize" => {
            let params = value.get_mut("params")?.as_object_mut()?;
            let capabilities = params.entry("capabilities").or_insert_with(|| json!({}));
            if capabilities.is_null() {
                *capabilities = json!({});
            }
            let capabilities = capabilities.as_object_mut()?;
            let opt_out = capabilities
                .entry("optOutNotificationMethods")
                .or_insert_with(|| json!([]));
            if opt_out.is_null() {
                *opt_out = json!([]);
            }
            let opt_out = opt_out.as_array_mut()?;
            if !opt_out.iter().all(Value::is_string) {
                return None;
            }
            if !opt_out
                .iter()
                .any(|method| method == "rawResponseItem/completed")
            {
                opt_out.push(json!("rawResponseItem/completed"));
            }
            capabilities.insert("experimentalApi".to_string(), json!(true));
        }
        "thread/start" => {
            value
                .get_mut("params")?
                .as_object_mut()?
                .insert("experimentalRawEvents".to_string(), json!(true));
        }
        _ => return None,
    }
    if value == original {
        return None;
    }
    let mut forwarded = serde_json::to_vec(&value).ok()?;
    if frame.ends_with(b"\r\n") {
        forwarded.extend_from_slice(b"\r\n");
    } else if frame.ends_with(b"\n") {
        forwarded.push(b'\n');
    }
    Some(forwarded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rewritten(value: &Value) -> Value {
        let original = serde_json::to_vec(value).unwrap();
        let forwarded = rewrite_frame(&original).unwrap_or(original);
        serde_json::from_slice(&forwarded).unwrap()
    }

    #[test]
    fn initialize_requests_only_completion_metadata_and_preserves_capabilities() {
        let original = json!({
            "id": 7,
            "method": "initialize",
            "params": {
                "clientInfo": {"name": "existing-client", "version": "9"},
                "capabilities": {
                    "experimentalApi": false,
                    "requestAttestation": true,
                    "optOutNotificationMethods": ["thread/started"]
                }
            }
        });
        let mut expected = original.clone();
        expected["params"]["capabilities"]["experimentalApi"] = json!(true);
        expected["params"]["capabilities"]["optOutNotificationMethods"] =
            json!(["thread/started", "rawResponseItem/completed"]);
        assert_eq!(rewritten(&original), expected);
    }

    #[test]
    fn initialize_handles_absent_or_null_capabilities() {
        for mut original in [
            json!({"id": 1, "method": "initialize", "params": {}}),
            json!({"id": 1, "method": "initialize", "params": {"capabilities": null}}),
        ] {
            let forwarded = rewritten(&original);
            original["params"]["capabilities"] = json!({
                "experimentalApi": true,
                "optOutNotificationMethods": ["rawResponseItem/completed"]
            });
            assert_eq!(forwarded, original);
        }
    }

    #[test]
    fn thread_start_changes_only_raw_event_flag() {
        let original = json!({"id": "start", "method": "thread/start", "params": {
            "model": "configured-model", "sandbox": "read-only", "cwd": "/project",
            "approvalPolicy": "never", "config": {"model_reasoning_effort": "configured-effort"},
            "baseInstructions": "Keep these instructions exactly.", "experimentalRawEvents": false
        }});
        let mut expected = original.clone();
        expected["params"]["experimentalRawEvents"] = json!(true);
        assert_eq!(rewritten(&original), expected);
    }

    #[test]
    fn unrelated_requests_and_responses_remain_byte_identical() {
        for frame in [
            b" {\"id\":4, \"method\":\"turn/interrupt\",\"params\":{\"threadId\":\"t\",\"turnId\":\"u\"}}\r\n".as_slice(),
            b"{\"id\":5,\"method\":\"turn/start\",\"params\":{\"input\":[{\"type\":\"text\",\"text\":\"Prompt unchanged\"}]}}\n",
            b"{\"id\":6,\"method\":\"thread/resume\",\"params\":{\"threadId\":\"t\"}}\n",
            b"{\"id\":1,\"result\":{\"success\":true}}\n",
            b"not-json\n",
            b"{\"method\":\"thread/start\",\"params\":null}\n",
        ] {
            assert!(rewrite_frame(frame).is_none());
        }
    }

    #[test]
    fn existing_opt_in_is_byte_identical_and_not_duplicated() {
        for frame in [
            b" {\"id\":1,\"method\":\"initialize\",\"params\":{\"capabilities\":{\"experimentalApi\":true,\"optOutNotificationMethods\":[\"rawResponseItem/completed\"]}}}\n".as_slice(),
            b"{\"id\":2,\"method\":\"thread/start\",\"params\":{\"experimentalRawEvents\":true}}\n",
        ] {
            assert!(rewrite_frame(frame).is_none());
        }
    }

    #[test]
    fn malformed_capabilities_are_not_repaired_or_partially_rewritten() {
        for capabilities in [json!(42), json!({"optOutNotificationMethods": 42})] {
            let original = json!({"id": 1, "method": "initialize", "params": {
                "capabilities": capabilities
            }});
            assert_eq!(rewritten(&original), original);
        }
    }

    #[test]
    fn rewritten_frames_keep_the_original_line_ending() {
        for ending in ["\n", "\r\n", ""] {
            let frame = format!("{{\"id\":1,\"method\":\"thread/start\",\"params\":{{}}}}{ending}");
            let forwarded = rewrite_frame(frame.as_bytes()).expect("metadata rewrite");
            let expected = format!(
                "{{\"id\":1,\"method\":\"thread/start\",\"params\":{{\"experimentalRawEvents\":true}}}}{ending}"
            );
            assert_eq!(forwarded, expected.as_bytes());
        }
    }

    #[test]
    fn opt_in_accepts_only_explicit_boolean_digits() {
        assert!(!enabled(None).unwrap());
        assert!(!enabled(Some(OsStr::new("0"))).unwrap());
        assert!(enabled(Some(OsStr::new("1"))).unwrap());
        for value in ["", "true", "yes", "2", " 1"] {
            assert!(enabled(Some(OsStr::new(value))).is_err());
        }
    }

    #[test]
    fn fragmented_frames_preserve_order_and_final_unterminated_bytes() {
        let frames = b"{\"id\":1,\"method\":\"thread/start\",\"params\":{}}\n{\"id\":2,\"method\":\"turn/interrupt\",\"params\":{}}\ntrailing";
        let mut pending = Vec::new();
        let mut forwarded = Vec::new();
        let mut send = |frame: &[u8]| {
            forwarded.extend_from_slice(rewrite_frame(frame).as_deref().unwrap_or(frame));
            Ok(())
        };
        for chunk in frames.chunks(3) {
            append_chunk(&mut pending, chunk, &mut send).unwrap();
        }
        send(&pending).unwrap();
        assert_eq!(forwarded, b"{\"id\":1,\"method\":\"thread/start\",\"params\":{\"experimentalRawEvents\":true}}\n{\"id\":2,\"method\":\"turn/interrupt\",\"params\":{}}\ntrailing");
    }

    #[test]
    fn forwarded_capture_and_rewrite_provenance_match_actual_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let mut files = CaptureFiles::create(directory.path(), false).unwrap();
        let mut destination = Vec::new();
        let done = AtomicBool::new(false);
        let start = b"{\"id\":1,\"method\":\"thread/start\",\"params\":{}}\n";
        let interrupt = b" {\"id\":2,\"method\":\"turn/interrupt\",\"params\":{\"threadId\":\"t\",\"turnId\":\"u\"}}\r\n";
        forward_frame(start, &mut destination, &mut files, &done, None).unwrap();
        forward_frame(interrupt, &mut destination, &mut files, &done, None).unwrap();
        assert!(destination.ends_with(interrupt));
        assert_eq!(
            std::fs::read(directory.path().join("client-to-server.forwarded.raw")).unwrap(),
            destination
        );
        let records =
            std::fs::read_to_string(directory.path().join("raw-response-rewrites.jsonl")).unwrap();
        assert_eq!(records.lines().count(), 1);
        let record: Value = serde_json::from_str(records.trim()).unwrap();
        assert_eq!(record["original_sha256"], super::super::sha256_bytes(start));
        assert_eq!(
            record["forwarded_sha256"],
            super::super::sha256_bytes(&rewrite_frame(start).unwrap())
        );
        assert_eq!(record["changed"], true);
        let provenance: Value = serde_json::from_slice(
            &std::fs::read(directory.path().join("raw-response-usage.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(provenance["enabled"], true);
        assert_eq!(provenance["interrupt_policy_unchanged"], true);
        assert!(!directory.path().join("provider-usage-grace.jsonl").exists());
    }

    fn directive() -> Value {
        json!({"method": "item/completed", "params": {
            "threadId": "thread", "turnId": "turn",
            "item": {"type": "agentMessage", "id": "message", "text": "@work-leaf read src/example.rs"}
        }})
    }

    #[test]
    fn raw_completion_neither_releases_grace_nor_marks_output_resumed() {
        let grace = ProviderUsageGrace::new(
            std::time::Duration::from_millis(1000),
            super::super::ProviderUsageGraceOutputResume::Forward,
        );
        grace.observe_server_value(&directive());
        grace.observe_server_value(&json!({"method": "rawResponse/completed", "params": {
            "threadId": "thread", "turnId": "turn", "responseId": "response",
            "usage": {"inputTokens": 100, "cachedInputTokens": 50, "outputTokens": 10,
                "reasoningOutputTokens": 0, "totalTokens": 110}
        }}));
        let state = grace.state.lock().unwrap();
        let turn = state
            .turns
            .get(&super::super::ProviderTurnKey {
                thread_id: "thread".to_string(),
                turn_id: "turn".to_string(),
            })
            .unwrap();
        assert!(turn.directive_complete);
        assert!(!turn.exact_usage_seen);
        assert!(!turn.output_resumed);
    }

    #[test]
    fn raw_capture_uses_existing_grace_release_and_output_resume_policy() {
        let directory = tempfile::tempdir().unwrap();
        let mut files = CaptureFiles::create(directory.path(), true).unwrap();
        let grace = ProviderUsageGrace::new(
            std::time::Duration::from_millis(1000),
            super::super::ProviderUsageGraceOutputResume::Forward,
        );
        grace.observe_server_value(&directive());
        grace.observe_server_value(&json!({"method": "item/started", "params": {
            "threadId": "thread", "turnId": "turn", "item": {"type": "reasoning", "id": "continuation"}
        }}));
        let interrupt = b" {\"id\":2,\"method\":\"turn/interrupt\",\"params\":{\"threadId\":\"thread\",\"turnId\":\"turn\"}}\r\n";
        let mut destination = Vec::new();
        forward_frame(
            interrupt,
            &mut destination,
            &mut files,
            &AtomicBool::new(false),
            Some(&grace),
        )
        .unwrap();
        assert_eq!(destination, interrupt);
        let record: Value = serde_json::from_str(
            &std::fs::read_to_string(directory.path().join("provider-usage-grace.jsonl")).unwrap(),
        )
        .unwrap();
        assert_eq!(record["configured_grace_ms"], 1000);
        assert_eq!(record["output_resume_policy"], "forward");
        assert_eq!(record["outcome"], "forwarded-after-output-resumed");
    }

    fn provenance_fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let temporary = tempfile::tempdir().unwrap();
        let invocation = temporary.path().join("invocation");
        let capture = temporary.path().join("capture");
        std::fs::create_dir_all(&invocation).unwrap();
        std::fs::create_dir_all(&capture).unwrap();
        let mut files = CaptureFiles::create(&capture, false).unwrap();
        let mut destination = Vec::new();
        let original = concat!(
            "{\"id\":1,\"method\":\"initialize\",\"params\":{}}\n",
            "{\"id\":2,\"method\":\"thread/start\",\"params\":{}}\n",
            "{\"id\":3,\"method\":\"thread/start\",\"params\":{\"experimentalRawEvents\":true}}\n",
            "{\"id\":4,\"method\":\"turn/start\",\"params\":{\"input\":\"Original prompt\"}}\n",
            "{\"id\":5,\"method\":\"turn/interrupt\",\"params\":{\"threadId\":\"thread\",\"turnId\":\"turn\"}}\n"
        );
        for frame in original.as_bytes().split_inclusive(|byte| *byte == b'\n') {
            forward_frame(
                frame,
                &mut destination,
                &mut files,
                &AtomicBool::new(false),
                None,
            )
            .unwrap();
        }
        std::fs::write(capture.join("client-to-server.raw"), original).unwrap();
        std::fs::write(
            invocation.join("start.json"),
            serde_json::to_vec(&json!({
                "raw_response_usage": true, "provider_usage_grace_ms": 0
            }))
            .unwrap(),
        )
        .unwrap();
        refresh_provenance_hashes(&invocation, &capture);
        (temporary, invocation, capture)
    }

    fn refresh_provenance_hashes(invocation: &Path, capture: &Path) {
        let mut hashes = serde_json::Map::new();
        for name in [
            "raw-response-usage.json",
            "raw-response-rewrites.jsonl",
            "client-to-server.forwarded.raw",
        ] {
            hashes.insert(
                name.to_owned(),
                json!(super::super::sha256_file(&capture.join(name)).unwrap()),
            );
        }
        std::fs::write(invocation.join("end.json"), serde_json::to_vec(&json!({
            "raw_response_usage_sha256": hashes,
            "raw_response_usage_start_sha256": super::super::sha256_file(&invocation.join("start.json")).unwrap()
        })).unwrap()).unwrap();
    }

    #[test]
    fn provenance_accepts_valid_opt_in_and_legacy_grace_only_capture() {
        let (_temporary, invocation, capture) = provenance_fixture();
        verify_capture(&invocation, &capture).unwrap();
        let legacy = tempfile::tempdir().unwrap();
        std::fs::write(legacy.path().join("start.json"), b"{}").unwrap();
        std::fs::write(legacy.path().join("end.json"), b"{}").unwrap();
        std::fs::write(
            legacy.path().join("client-to-server.forwarded.raw"),
            b"legacy grace stream",
        )
        .unwrap();
        verify_capture(legacy.path(), legacy.path()).unwrap();
    }

    #[test]
    fn provenance_rejects_missing_or_tampered_opt_in_artifacts() {
        for name in [
            "raw-response-usage.json",
            "raw-response-rewrites.jsonl",
            "client-to-server.forwarded.raw",
        ] {
            for missing in [true, false] {
                let (_temporary, invocation, capture) = provenance_fixture();
                if missing {
                    std::fs::remove_file(capture.join(name)).unwrap();
                } else {
                    std::fs::write(capture.join(name), b"{}\n").unwrap();
                }
                assert!(
                    verify_capture(&invocation, &capture).is_err(),
                    "{name}, missing={missing}"
                );
            }
        }
    }

    #[test]
    fn provenance_rejects_missing_partial_or_malformed_digest_metadata() {
        for replacement in [
            Value::Null,
            json!({}),
            json!({"raw-response-usage.json": "not-a-digest"}),
        ] {
            let (_temporary, invocation, capture) = provenance_fixture();
            let mut end: Value =
                serde_json::from_slice(&std::fs::read(invocation.join("end.json")).unwrap())
                    .unwrap();
            end["raw_response_usage_sha256"] = replacement;
            std::fs::write(
                invocation.join("end.json"),
                serde_json::to_vec(&end).unwrap(),
            )
            .unwrap();
            assert!(verify_capture(&invocation, &capture).is_err());
        }
    }

    #[test]
    fn provenance_rejects_conflicting_or_removed_opt_in_marker() {
        for marker in [Value::Null, json!(false), json!("true")] {
            let (_temporary, invocation, capture) = provenance_fixture();
            std::fs::write(
                invocation.join("start.json"),
                serde_json::to_vec(&json!({"raw_response_usage": marker})).unwrap(),
            )
            .unwrap();
            refresh_provenance_hashes(&invocation, &capture);
            assert!(verify_capture(&invocation, &capture).is_err());
        }
    }

    #[test]
    fn provenance_rejects_rehashed_model_facing_changes_or_wrong_settings() {
        for settings in [false, true] {
            let (_temporary, invocation, capture) = provenance_fixture();
            if settings {
                let path = capture.join("raw-response-usage.json");
                let mut value: Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                value["interrupt_policy_unchanged"] = json!(false);
                std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            } else {
                let path = capture.join("client-to-server.forwarded.raw");
                let value = std::fs::read_to_string(&path)
                    .unwrap()
                    .replace("Original prompt", "Different prompt");
                std::fs::write(path, value).unwrap();
            }
            refresh_provenance_hashes(&invocation, &capture);
            assert!(verify_capture(&invocation, &capture).is_err());
        }
    }

    #[test]
    fn provenance_rejects_rehashed_missing_extra_reordered_or_false_decisions() {
        for mutation in ["missing", "extra", "reordered", "changed-false"] {
            let (_temporary, invocation, capture) = provenance_fixture();
            let path = capture.join("raw-response-rewrites.jsonl");
            let mut values = std::fs::read_to_string(&path)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str::<Value>(line).unwrap())
                .collect::<Vec<_>>();
            match mutation {
                "missing" => {
                    values.pop();
                }
                "extra" => values.push(values[0].clone()),
                "reordered" => values.swap(0, 1),
                "changed-false" => values[2]["forwarded_sha256"] = json!("false-hash"),
                _ => unreachable!(),
            }
            std::fs::write(
                path,
                values
                    .iter()
                    .map(|value| serde_json::to_string(value).unwrap() + "\n")
                    .collect::<String>(),
            )
            .unwrap();
            refresh_provenance_hashes(&invocation, &capture);
            assert!(verify_capture(&invocation, &capture).is_err(), "{mutation}");
        }
    }
}
