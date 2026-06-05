use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use crate::agent::{AgentError, AgentId, AgentKind, AgentLaunch};
use crate::codex::AgentBackend;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommit {
    pub hash: String,
    pub agent_id: AgentId,
    pub feature: String,
    pub reason: String,
    pub subject: String,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHistory {
    root: PathBuf,
}

impl GitHistory {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn latest_agent_commits(&self) -> Result<Vec<AgentCommit>, ReviewError> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(["log", "--pretty=format:%H%x1f%B%x1e"])
            .output()
            .map_err(ReviewError::Io)?;
        if !output.status.success() {
            return Err(ReviewError::History(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        let mut latest = BTreeMap::new();
        for record in String::from_utf8_lossy(&output.stdout).split('\x1e') {
            let record = record.trim();
            if record.is_empty() {
                continue;
            }
            let Some(commit) = parse_agent_commit(record)? else {
                continue;
            };
            latest.entry(commit.agent_id.clone()).or_insert(commit);
        }

        Ok(latest.into_values().collect())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewResult {
    pub agent_id: AgentId,
    pub reviewer_id: AgentId,
    pub commit: AgentCommit,
    pub rounds: usize,
    pub findings_resolved: bool,
}

#[derive(Debug)]
pub struct ReviewCoordinator<B> {
    root: PathBuf,
    backend: B,
    max_rounds: usize,
}

impl<B> ReviewCoordinator<B>
where
    B: AgentBackend,
{
    pub fn new(root: PathBuf, backend: B) -> Self {
        Self {
            root,
            backend,
            max_rounds: 8,
        }
    }

    pub fn with_max_rounds(mut self, max_rounds: usize) -> Self {
        self.max_rounds = max_rounds.max(1);
        self
    }

    pub fn into_backend(self) -> B {
        self.backend
    }

    pub fn review_latest_agent_commits(&mut self) -> Result<Vec<ReviewResult>, ReviewError> {
        let commits = GitHistory::new(self.root.clone()).latest_agent_commits()?;
        commits
            .into_iter()
            .map(|commit| self.review_commit(commit))
            .collect()
    }

    fn review_commit(&mut self, commit: AgentCommit) -> Result<ReviewResult, ReviewError> {
        let summary_prompt = format!(
            "Please summarize the final patch for Agent-ID {}.\nCommit: {}\nFeature: {}\nReason: {}\n\nFocus on what behavior the patch changes.",
            commit.agent_id, commit.hash, commit.feature, commit.reason
        );
        let summary = self
            .backend
            .send(&commit.agent_id, &summary_prompt)
            .map_err(ReviewError::Agent)?
            .text;

        let reviewer_id = AgentId::new(format!("review-{}", commit.agent_id.as_str()))
            .map_err(ReviewError::Agent)?;
        let review_prompt = format!(
            "Review the final patch for Agent-ID {}.\nCommit: {}\nFeature: {}\nReason: {}\nSummary from original agent:\n{}\n\nReply with NO_FINDINGS if there are no findings. Otherwise reply with FINDINGS followed by the issues.",
            commit.agent_id, commit.hash, commit.feature, commit.reason, summary
        );
        let reviewer_session = self
            .backend
            .launch(AgentLaunch::new(
                reviewer_id.clone(),
                AgentKind::Codex,
                format!("review {}", commit.feature),
                review_prompt,
            ))
            .map_err(ReviewError::Agent)?;
        let mut review_text = reviewer_session
            .messages
            .last()
            .map(|message| message.text.clone())
            .unwrap_or_default();
        let mut rounds = 1;

        while !has_no_findings(&review_text) && rounds < self.max_rounds {
            let fix_prompt = format!(
                "The reviewer found issues in your patch for commit {}.\n{}\n\nPlease fix the patch through the orchestrator patch flow.",
                commit.hash, review_text
            );
            self.backend
                .send(&commit.agent_id, &fix_prompt)
                .map_err(ReviewError::Agent)?;

            let recheck_prompt = format!(
                "The original agent has responded to the findings for commit {}. Please check the patch again and reply with NO_FINDINGS if resolved, otherwise list remaining FINDINGS.",
                commit.hash
            );
            review_text = self
                .backend
                .send(&reviewer_id, &recheck_prompt)
                .map_err(ReviewError::Agent)?
                .text;
            rounds += 1;
        }

        Ok(ReviewResult {
            agent_id: commit.agent_id.clone(),
            reviewer_id,
            findings_resolved: has_no_findings(&review_text),
            rounds,
            commit,
        })
    }
}

fn parse_agent_commit(record: &str) -> Result<Option<AgentCommit>, ReviewError> {
    let Some((hash, body)) = record.split_once('\x1f') else {
        return Ok(None);
    };
    let Some(agent_id) = metadata_value(body, "Agent-ID:") else {
        return Ok(None);
    };
    let feature = metadata_value(body, "Feature:").unwrap_or_default();
    let reason = metadata_value(body, "Reason:").unwrap_or_default();
    let subject = body.lines().next().unwrap_or_default().to_string();
    Ok(Some(AgentCommit {
        hash: hash.trim().to_string(),
        agent_id: AgentId::new(agent_id).map_err(ReviewError::Agent)?,
        feature,
        reason,
        subject,
        body: body.trim().to_string(),
    }))
}

fn metadata_value(body: &str, prefix: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.trim().strip_prefix(prefix).map(str::trim))
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn has_no_findings(text: &str) -> bool {
    text.lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.trim().eq_ignore_ascii_case("NO_FINDINGS"))
}

#[derive(Debug)]
pub enum ReviewError {
    Agent(AgentError),
    History(String),
    Io(std::io::Error),
}

impl fmt::Display for ReviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::History(message) => write!(formatter, "git history error: {message}"),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for ReviewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Agent(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::History(_) => None,
        }
    }
}
