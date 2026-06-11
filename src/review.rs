use std::collections::BTreeMap;
use std::fmt::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use crate::agent::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentProfile, AgentSession, MessageRole,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommit {
    pub hash: String,
    pub agent_id: AgentId,
    pub feature: String,
    pub reason: String,
    pub context: String,
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

    pub fn latest_agent_review_commits(
        &self,
        reviewed_agent_commits: &BTreeMap<AgentId, String>,
        agent_baselines: &BTreeMap<AgentId, String>,
    ) -> Result<Vec<AgentCommit>, ReviewError> {
        let latest = self.latest_agent_commits()?;
        let mut targets = Vec::new();
        for commit in latest {
            if reviewed_agent_commits
                .get(&commit.agent_id)
                .is_some_and(|hash| hash == &commit.hash)
            {
                continue;
            }
            let boundary = reviewed_agent_commits
                .get(&commit.agent_id)
                .or_else(|| agent_baselines.get(&commit.agent_id))
                .map(String::as_str);
            if let Some(target) = self.agent_review_commit(&commit.agent_id, boundary)? {
                targets.push(target);
            }
        }
        Ok(targets)
    }

    pub fn agent_commit(&self, hash: &str) -> Result<Option<AgentCommit>, ReviewError> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(["show", "--no-patch", "--pretty=format:%H%x1f%B%x1e", hash])
            .output()
            .map_err(ReviewError::Io)?;
        if !output.status.success() {
            return Err(ReviewError::History(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        parse_agent_commit(String::from_utf8_lossy(&output.stdout).trim())
    }

    pub fn agent_review_commit(
        &self,
        agent_id: &AgentId,
        boundary: Option<&str>,
    ) -> Result<Option<AgentCommit>, ReviewError> {
        let records = self.agent_commits_in_range(boundary)?;
        let commits = if boundary.is_some() {
            records
                .into_iter()
                .filter(|commit| &commit.agent_id == agent_id)
                .collect()
        } else {
            latest_contiguous_agent_commits(records, agent_id)
        };
        Ok(combine_agent_review_commits(commits))
    }

    pub fn head_hash(&self) -> Result<Option<String>, ReviewError> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(["rev-parse", "--verify", "HEAD"])
            .output()
            .map_err(ReviewError::Io)?;
        if !output.status.success() {
            return Ok(None);
        }
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    }

    fn agent_commits_in_range(
        &self,
        boundary: Option<&str>,
    ) -> Result<Vec<AgentCommit>, ReviewError> {
        let mut args = vec![
            "log".to_string(),
            "--pretty=format:%H%x1f%B%x1e".to_string(),
        ];
        let range;
        if let Some(boundary) = boundary {
            range = format!("{boundary}..HEAD");
            args.push(range);
        }
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .map_err(ReviewError::Io)?;
        if !output.status.success() {
            return Err(ReviewError::History(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        let mut commits = Vec::new();
        for record in String::from_utf8_lossy(&output.stdout).split('\x1e') {
            let record = record.trim();
            if record.is_empty() {
                continue;
            }
            let Some(commit) = parse_agent_commit(record)? else {
                continue;
            };
            commits.push(commit);
        }
        Ok(commits)
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

pub(crate) fn render_review_source_context(
    commit: &AgentCommit,
    session: Option<&AgentSession>,
) -> String {
    let mut text = String::new();
    let _ = write!(
        text,
        "Work Leaf collected this context from commits, git logs, and recorded chat history without querying Agent-ID {}.\n\nGit metadata:\nLatest commit: {}\nFeature: {}\nReason: {}\nReview scope:\n{}\n\nGit commit log:\n{}",
        commit.agent_id, commit.hash, commit.feature, commit.reason, commit.context, commit.body
    );
    text.push_str("\n\nRecorded chat history:");
    match session {
        Some(session) if !session.messages.is_empty() => {
            for (index, message) in session.messages.iter().enumerate() {
                let role = match &message.role {
                    MessageRole::User => "user",
                    MessageRole::Agent => "agent",
                    MessageRole::Orchestrator => "orchestrator",
                    MessageRole::System => "system",
                };
                let _ = write!(
                    text,
                    "\n\n{} {role}:\n{}",
                    index + 1,
                    message.text.trim_end()
                );
            }
        }
        Some(_) => text.push_str("\n(no recorded messages)"),
        None => text.push_str("\n(unavailable from backend session state)"),
    }
    text
}

#[derive(Debug)]
pub struct ReviewCoordinator<B> {
    root: PathBuf,
    backend: B,
    agent_profile: AgentProfile,
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
            agent_profile: AgentProfile::codex(),
            max_rounds: 8,
        }
    }

    pub fn with_agent_profile(mut self, agent_profile: AgentProfile) -> Self {
        self.agent_profile = agent_profile;
        self
    }

    pub fn with_max_rounds(mut self, max_rounds: usize) -> Self {
        self.max_rounds = max_rounds.max(1);
        self
    }

    pub fn into_backend(self) -> B {
        self.backend
    }

    pub fn review_latest_agent_commits(&mut self) -> Result<Vec<ReviewResult>, ReviewError> {
        let commits = GitHistory::new(self.root.clone())
            .latest_agent_review_commits(&BTreeMap::new(), &BTreeMap::new())?;
        commits
            .into_iter()
            .map(|commit| self.review_commit(commit))
            .collect()
    }

    fn review_commit(&mut self, commit: AgentCommit) -> Result<ReviewResult, ReviewError> {
        let source_context = {
            let session = self.backend.session(&commit.agent_id);
            render_review_source_context(&commit, session.as_ref())
        };

        let reviewer_id = AgentId::new(format!("review-{}", commit.agent_id.as_str()))
            .map_err(ReviewError::Agent)?;
        let review_prompt = format!(
            "Review the full patch scope for Agent-ID {}.\nLatest commit: {}\nFeature: {}\nReason: {}\nReview scope:\n{}\n\nSource context from Work Leaf commits, logs, and chat history:\n{}\n\nReview every commit listed in the review scope and reply with NO_FINDINGS if there are no findings. Otherwise reply with FINDINGS followed by the issues.\n\nDocumentation and plain-text updates are deferred to the linearize agent. Do not treat missing docs, README, changelog, markdown, txt, or other prose-only updates as findings against this patch agent; review the code and behavior that the patch agent changed.\n\nFor agent-facing changes, missing required real-agent verification is a finding unless the source context includes the exact real-agent scenario and visible result, or the exact pre-agent blocker. If you report missing verification, state the precise evidence that would resolve it. When the patch agent responds with verification evidence or a blocker rather than code, evaluate that evidence instead of requiring another patch.",
            commit.agent_id,
            commit.hash,
            commit.feature,
            commit.reason,
            commit.context,
            source_context
        );
        let reviewer_session = self
            .backend
            .launch(AgentLaunch::new(
                reviewer_id.clone(),
                self.agent_profile.kind.clone(),
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
                "The reviewer found issues in your patch for commit {}.\n{}\n\nPlease fix the patch's code or test defects through the orchestrator patch flow. If a finding is about missing verification, missing explanation, or another non-code issue, resolve it by replying with the exact evidence, command result, real-agent scenario, or blocker; do not submit a cosmetic patch for non-code evidence. Do not modify documentation or plain-text files; documentation and prose updates are deferred to the linearize agent. Emit `@work-leaf done` when the findings are resolved.",
                commit.hash, review_text
            );
            let fix_reply = self
                .backend
                .send(&commit.agent_id, &fix_prompt)
                .map_err(ReviewError::Agent)?
                .text;

            let recheck_prompt = format!(
                "The original agent has responded to the findings for commit {}.\n{}\n\nPlease check the patch again and reply with NO_FINDINGS if resolved, otherwise list remaining FINDINGS. The response may include code patches, verification evidence, real-agent smoke results, or an exact blocker; evaluate that evidence directly and do not require a code patch for a non-code finding. Documentation and plain-text updates are deferred to the linearize agent and must not be reported as remaining patch-agent findings.",
                commit.hash, fix_reply
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
    let context = metadata_value(body, "Context:").unwrap_or_default();
    let subject = body.lines().next().unwrap_or_default().to_string();
    Ok(Some(AgentCommit {
        hash: hash.trim().to_string(),
        agent_id: AgentId::new(agent_id).map_err(ReviewError::Agent)?,
        feature,
        reason,
        context,
        subject,
        body: body.trim().to_string(),
    }))
}

fn latest_contiguous_agent_commits(
    commits: Vec<AgentCommit>,
    agent_id: &AgentId,
) -> Vec<AgentCommit> {
    let mut selected = Vec::new();
    let mut found_latest = false;
    for commit in commits {
        if &commit.agent_id == agent_id {
            found_latest = true;
            selected.push(commit);
        } else if found_latest {
            break;
        }
    }
    selected
}

fn combine_agent_review_commits(mut commits: Vec<AgentCommit>) -> Option<AgentCommit> {
    if commits.len() <= 1 {
        return commits.pop();
    }

    let latest = commits[0].clone();
    let mut context = format!(
        "Review scope includes {} provisional commits for Agent-ID {} from oldest to newest. Review the cumulative behavior and diff across all listed commits, not only the latest commit.",
        commits.len(),
        latest.agent_id
    );
    for commit in commits.iter().rev() {
        let _ = write!(
            context,
            "\n\nCommit: {}\nSubject: {}\nFeature: {}\nReason: {}\nContext: {}",
            commit.hash, commit.subject, commit.feature, commit.reason, commit.context
        );
    }

    let mut body = String::new();
    for commit in commits.iter().rev() {
        let _ = write!(body, "\n\n--- commit {} ---\n{}", commit.hash, commit.body);
    }

    let latest_hash = latest.hash.clone();
    Some(AgentCommit {
        hash: latest_hash.clone(),
        agent_id: latest.agent_id,
        feature: latest.feature,
        reason: format!(
            "Review {} provisional commits through {}",
            commits.len(),
            short_hash(&latest_hash)
        ),
        context,
        subject: latest.subject,
        body: body.trim().to_string(),
    })
}

fn short_hash(hash: &str) -> &str {
    hash.get(..12).unwrap_or(hash)
}

fn metadata_value(body: &str, prefix: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.trim().strip_prefix(prefix).map(str::trim))
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(crate) fn has_no_findings(text: &str) -> bool {
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
