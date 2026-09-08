//! Private v7 automatic refresh evidence; no reads, diff execution or tracking.
use super::*;
use serde_json::Value;

#[derive(Default)]
pub(crate) struct Capture {
    replacements: Vec<Replacement>,
    pub(crate) snapshots: Vec<Value>,
    pub(crate) failures: Vec<Value>,
}

struct Replacement {
    id: &'static str,
    cue: Range<usize>,
    text: String,
    body: Option<Range<usize>>,
    snapshot: Option<usize>,
}

impl Capture {
    pub(crate) fn start() -> io::Result<Option<Self>> {
        Ok(active()?
            .filter(|e| e.manifest.schema == SCHEMA_V7)
            .map(|_| Self::default()))
    }

    pub(crate) fn replace(
        &mut self,
        id: &'static str,
        cue: Range<usize>,
        text: String,
        body: Option<Range<usize>>,
        snapshot: Option<usize>,
    ) {
        self.replacements.push(Replacement {
            id,
            cue,
            text,
            body,
            snapshot,
        });
    }

    // One ordered walk; no search through diagnostics or copied source markers.
    pub(crate) fn candidate(&self, original: &str) -> io::Result<(String, Vec<Value>)> {
        if !self.replacements.iter().any(|part| part.body.is_some()) {
            return Ok((original.to_owned(), Vec::new()));
        }
        let mut text = String::with_capacity(original.len());
        let mut parts = Vec::with_capacity(self.replacements.len());
        let mut cursor = 0;
        for part in &self.replacements {
            if part.cue.start < cursor
                || original.get(part.cue.clone()).is_none()
                || part
                    .body
                    .as_ref()
                    .is_some_and(|range| part.text.get(range.clone()).is_none())
            {
                return Err(invalid(
                    "automatic refresh components must be ordered owned UTF-8 ranges",
                ));
            }
            text.push_str(&original[cursor..part.cue.start]);
            let start = text.len();
            text.push_str(&part.text);
            parts.push(json!({"id":part.id,"baseline_start":part.cue.start,"baseline_end":part.cue.end,
                "candidate_start":start,"candidate_end":text.len(),"snapshot_index":part.snapshot,
                "body_start":part.body.as_ref().map(|r|start+r.start),"body_end":part.body.as_ref().map(|r|start+r.end)}));
            cursor = part.cue.end;
        }
        text.push_str(&original[cursor..]);
        Ok((text, parts))
    }

    pub(crate) fn forward(
        self,
        agent_id: &AgentId,
        kind: &str,
        original: String,
        metadata: Value,
    ) -> io::Result<String> {
        let experiment = active()?
            .filter(|e| e.manifest.schema == SCHEMA_V7)
            .ok_or_else(|| invalid("automatic refresh evidence requires active v7"))?;
        self.forward_with(experiment, agent_id, kind, original, metadata)
    }

    fn forward_with(
        self,
        experiment: &Experiment,
        agent_id: &AgentId,
        kind: &str,
        original: String,
        mut metadata: Value,
    ) -> io::Result<String> {
        let site = match kind {
            "patch" => "automatic-patch-refresh",
            "edit" => "automatic-edit-refresh",
            _ => return Err(invalid("unknown automatic refresh kind")),
        };
        if experiment.manifest.schema != SCHEMA_V7
            || experiment.manifest.condition != "automatic-changed-refresh-full"
        {
            return Err(invalid(
                "automatic refresh evidence requires its single v7 condition",
            ));
        }
        let (candidate, components) = self.candidate(&original)?;
        metadata["snapshots"] = json!(self.snapshots);
        metadata["failures"] = json!(self.failures);
        metadata["kind"] = json!(kind);
        let eligible = !components.is_empty();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(invalid)?
            .as_nanos()
            .to_string();
        let mut evidence = experiment.evidence.lock().map_err(invalid)?;
        evidence.sequence += 1;
        let row = json!({"event":"automatic-refresh","schema":SCHEMA_V7,"sequence":evidence.sequence,
            "run_id":experiment.manifest.run_id,"condition":experiment.manifest.condition,"process_id":std::process::id(),
            "unix_time_ns":timestamp,"site":site,"agent_id":agent_id.to_string(),"original_prompt":original,"candidate_prompt":candidate,
            "selected_candidate":if eligible {"candidate"}else{"baseline"},"original_bytes":original.len(),"candidate_bytes":candidate.len(),
            "changed":original!=candidate,"eligible":eligible,"components":components,"metadata":metadata});
        serde_json::to_writer(&mut evidence.file, &row).map_err(invalid)?;
        evidence.file.write_all(b"\n").map_err(invalid)?;
        evidence.file.flush().map_err(invalid)?;
        Ok(candidate)
    }
}

#[cfg(test)]
#[path = "bench_automatic_refresh_tests.rs"]
mod tests;
