use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::{self, Write};

use crate::agent::{AgentBackend, AgentError, AgentId, AgentLaunch, AgentProfile};
use crate::review::AgentCommit;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinearizeAction {
    KeepFinalCommit,
    IntegrateInto(AgentId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearizeGroup {
    pub name: String,
    pub agent_ids: Vec<AgentId>,
}

impl LinearizeGroup {
    pub fn new<I>(name: impl Into<String>, agent_ids: I) -> Self
    where
        I: IntoIterator<Item = AgentId>,
    {
        Self {
            name: name.into(),
            agent_ids: agent_ids.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearizePlan {
    pub commits: Vec<AgentCommit>,
    pub decisions: BTreeMap<AgentId, LinearizeAction>,
    pub groups: Vec<LinearizeGroup>,
    pub test_commands: Vec<Vec<String>>,
}

impl LinearizePlan {
    pub fn new(commits: Vec<AgentCommit>) -> Self {
        Self {
            commits,
            decisions: BTreeMap::new(),
            groups: Vec::new(),
            test_commands: Vec::new(),
        }
    }

    pub fn decide(mut self, agent_id: AgentId, action: LinearizeAction) -> Self {
        self.decisions.insert(agent_id, action);
        self
    }

    pub fn group(mut self, group: LinearizeGroup) -> Self {
        self.groups.push(group);
        self
    }

    pub fn test_command<I, S>(mut self, command: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.test_commands
            .push(command.into_iter().map(Into::into).collect());
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearizeQuestion {
    pub agent_id: AgentId,
    pub feature: String,
    pub prompt: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearizeHandoff {
    pub linearizer_id: AgentId,
    pub prompt: String,
    pub initial_reply: String,
}

#[derive(Debug)]
pub struct LinearizePlanner<B> {
    backend: B,
    agent_profile: AgentProfile,
}

impl<B> LinearizePlanner<B>
where
    B: AgentBackend,
{
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            agent_profile: AgentProfile::codex(),
        }
    }

    pub fn with_agent_profile(mut self, agent_profile: AgentProfile) -> Self {
        self.agent_profile = agent_profile;
        self
    }

    pub fn into_backend(self) -> B {
        self.backend
    }

    pub fn interactive_prompt(commits: &[AgentCommit]) -> String {
        build_interactive_linearize_prompt(commits)
    }

    pub fn questions_for(commits: &[AgentCommit]) -> Vec<LinearizeQuestion> {
        compact_linearize_targets(commits)
            .iter()
            .map(|commit| LinearizeQuestion {
                agent_id: commit.agent_id.clone(),
                feature: commit.feature.clone(),
                prompt: format!(
                    "For Agent-ID {} ({}) should this patch keep a final commit as its one final commit, integrate into another commit, or be grouped with other reviewed chat ids?",
                    commit.agent_id, commit.feature
                ),
            })
            .collect()
    }

    pub fn launch_linearizer(
        &mut self,
        plan: LinearizePlan,
    ) -> Result<LinearizeHandoff, LinearizeError> {
        self.validate_plan(&plan)?;
        let linearizer_id = AgentId::new("linearize").map_err(LinearizeError::Agent)?;
        let prompt = build_linearize_prompt(&plan);
        let session = self
            .backend
            .launch(AgentLaunch::new(
                linearizer_id.clone(),
                self.agent_profile.kind.clone(),
                "linearize reviewed patches",
                prompt.clone(),
            ))
            .map_err(LinearizeError::Agent)?;
        let initial_reply = session
            .messages
            .last()
            .map(|message| message.text.clone())
            .unwrap_or_default();

        Ok(LinearizeHandoff {
            linearizer_id,
            prompt,
            initial_reply,
        })
    }

    fn validate_plan(&self, plan: &LinearizePlan) -> Result<(), LinearizeError> {
        for commit in compact_linearize_targets(&plan.commits) {
            if !plan.decisions.contains_key(&commit.agent_id) {
                return Err(LinearizeError::MissingDecision(commit.agent_id.clone()));
            }
        }
        Ok(())
    }
}

fn build_interactive_linearize_prompt(commits: &[AgentCommit]) -> String {
    let targets = compact_linearize_targets(commits);
    let final_count = final_commit_count_phrase(targets.len());
    let mut prompt = String::new();
    prompt.push_str("You are the work-leaf linearizer for reviewed agent patches.\n\n");
    prompt.push_str(&format!("Final patch targets ({}):\n", targets.len()));
    for commit in &targets {
        prompt.push_str(&format!(
            "- Agent-ID: {}\n  Commit: {}\n  Feature: {}\n  Reason: {}\n  Subject: {}\n  Context: {}\n",
            commit.agent_id,
            commit.hash,
            commit.feature,
            commit.reason,
            commit.subject,
            commit.context
        ));
    }

    prompt.push_str(
        "\nScope and commit-shaping rules:\n\
- Only the reviewed commits listed in this prompt are in scope for this linearization run. Ignore other provisional work-leaf commits in git history unless the user explicitly adds them.\n\
- Default to one final commit per listed patch agent target. If one listed patch agent produced multiple reviewed provisional commits, compact that agent's provisional commits into one final commit.\n\
- Produce exactly the number of final commits stated in this prompt unless the user explicitly accepts a different grouping.\n\
- Do not keep or create separate support, test-hygiene, review-fix, validation-fix, or documentation-only commits; fold necessary support changes into the closest reviewed feature commit.\n\
- Merge work from multiple listed patch agents only when they implemented the same feature or behavior.\n\
- The repository's AGENTS.md commit message rules have priority over all linearizer examples and must be followed exactly.\n\
- Keep each final commit's diff against main/master as small as possible while preserving the reviewed behavior.\n",
    );
    prompt.push_str(&format!(
        "\nFinal history contract: after acceptance, rewrite the reviewed provisional history into {final_count}. The final history must not contain extra Work Leaf support commits unless the user explicitly asks for them.\n",
    ));

    prompt.push_str(
        "\nRequired workflow:\n\
1. Inspect git history, the current branch, and the merge base with main or master before proposing a rewrite.\n\
2. Propose the solution before changing history. For each reviewed patch, state which final commit message should be kept, which provisional commit message should be removed, and whether related work should be grouped or merged.\n\
3. Ask the user to accept the solution or request changes. Do not rewrite history until the user accepts.\n\
4. After acceptance, rewrite provisional work-leaf commits into coherent final commits and remove provisional agent commits.\n\
5. Update documentation and plain-text files directly when the final reviewed behavior requires it; patch agents intentionally defer docs, README, changelog, markdown, txt, and other prose-only updates to this linearize step.\n\
6. Keep the diff against main/master as small as possible while preserving reviewed behavior.\n\
7. Run the checks required by the repository instructions and iterate until they pass.\n\
8. Report the final commit messages, removed provisional messages, grouping decisions, documentation/plain-text decisions, and verification results.\n\
\nYou are a direct workspace agent for linearization. Read files, write files, run commands, and rewrite git history directly; do not use `@work-leaf read`, `@work-leaf edit`, `@work-leaf patch`, or `@work-leaf locks run`.\n",
    );
    prompt
}

fn build_linearize_prompt(plan: &LinearizePlan) -> String {
    let targets = compact_linearize_targets(&plan.commits);
    let expected_count = expected_final_commit_count(plan, &targets);
    let final_count = final_commit_count_phrase(expected_count);
    let mut prompt = String::new();
    prompt.push_str("You are the work-leaf linearizer for reviewed agent patches.\n");
    prompt.push_str(
        "Rewrite the provisional git history into clean final commits using the decisions below.\n",
    );
    prompt.push_str("Only the reviewed commits listed below are in scope. Ignore other provisional work-leaf commits in git history unless the user explicitly adds them.\n");
    prompt.push_str("Default to one final commit per listed patch agent target, compact multiple provisional commits from that agent into that final commit, and merge multiple listed patch agents only when they implemented the same feature or behavior.\n");
    prompt.push_str(&format!("Produce {final_count} according to the decisions below. Do not keep or create separate support, test-hygiene, review-fix, validation-fix, or documentation-only commits; fold necessary support changes into the closest reviewed feature commit.\n"));
    prompt.push_str("The repository's AGENTS.md commit message rules have priority over all linearizer examples, and make the diff against master/main as small as possible for each final commit. Documentation and plain-text files intentionally deferred by patch agents are updated directly by the linearizer when the final reviewed behavior requires them.\n\n");

    prompt.push_str("Final patch targets:\n");
    for commit in &targets {
        prompt.push_str(&format!(
            "- {} {} feature={} reason={}\n",
            commit.agent_id, commit.hash, commit.feature, commit.reason
        ));
    }

    prompt.push_str("\nDecisions:\n");
    for commit in &targets {
        let Some(action) = plan.decisions.get(&commit.agent_id) else {
            continue;
        };
        match action {
            LinearizeAction::KeepFinalCommit => {
                prompt.push_str(&format!(
                    "- {}: keep a final commit for {}\n",
                    commit.agent_id, commit.feature
                ));
            }
            LinearizeAction::IntegrateInto(target) => {
                prompt.push_str(&format!(
                    "- {}: integrate into {}\n",
                    commit.agent_id, target
                ));
            }
        }
    }

    if !plan.groups.is_empty() {
        prompt.push_str("\nGroups:\n");
        for group in &plan.groups {
            let ids = group
                .agent_ids
                .iter()
                .map(AgentId::as_str)
                .collect::<Vec<_>>()
                .join(", ");
            prompt.push_str(&format!("- Group {}: {}\n", group.name, ids));
        }
    }

    if !plan.test_commands.is_empty() {
        prompt.push_str("\nVerification commands:\n");
        for command in &plan.test_commands {
            prompt.push_str(&format!("- {}\n", command.join(" ")));
        }
    }

    prompt.push_str("\nUse direct workspace reads, writes, commands, and git history rewrites; do not use `@work-leaf read`, `@work-leaf edit`, `@work-leaf patch`, or `@work-leaf locks run`. Iterate until the verification commands pass. Keep the resulting history minimal and coherent for human review.\n");
    prompt
}

fn compact_linearize_targets(commits: &[AgentCommit]) -> Vec<AgentCommit> {
    let mut groups: Vec<Vec<AgentCommit>> = Vec::new();
    for commit in commits {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group[0].agent_id == commit.agent_id)
        {
            group.push(commit.clone());
        } else {
            groups.push(vec![commit.clone()]);
        }
    }
    groups
        .into_iter()
        .filter_map(compact_linearize_target)
        .collect()
}

fn compact_linearize_target(mut commits: Vec<AgentCommit>) -> Option<AgentCommit> {
    if commits.len() <= 1 {
        return commits.pop();
    }

    let latest = commits.last()?.clone();
    let mut context = format!(
        "Linearize target includes {} reviewed provisional commits for patch agent {}. Fold every listed reviewed commit into one final feature commit.",
        commits.len(),
        latest.agent_id
    );
    for commit in &commits {
        let _ = write!(
            context,
            "\n\nReviewed commit: {}\nSubject: {}\nFeature: {}\nReason: {}\nContext: {}",
            commit.hash, commit.subject, commit.feature, commit.reason, commit.context
        );
    }

    let mut body = String::new();
    for commit in &commits {
        let _ = write!(
            body,
            "\n\n--- reviewed commit {} ---\n{}",
            commit.hash, commit.body
        );
    }

    Some(AgentCommit {
        hash: latest.hash.clone(),
        agent_id: latest.agent_id,
        feature: latest.feature,
        reason: format!(
            "Linearize {} reviewed commits through {}",
            commits.len(),
            short_hash(&latest.hash)
        ),
        context,
        subject: latest.subject,
        body: body.trim().to_string(),
    })
}

fn expected_final_commit_count(plan: &LinearizePlan, targets: &[AgentCommit]) -> usize {
    let mut final_agents = BTreeSet::new();
    for target in targets {
        match plan.decisions.get(&target.agent_id) {
            Some(LinearizeAction::IntegrateInto(agent_id)) => {
                final_agents.insert(agent_id.clone());
            }
            Some(LinearizeAction::KeepFinalCommit) | None => {
                final_agents.insert(target.agent_id.clone());
            }
        }
    }
    final_agents.len()
}

fn final_commit_count_phrase(count: usize) -> String {
    if count == 1 {
        "exactly 1 final commit".to_string()
    } else {
        format!("exactly {count} final commits")
    }
}

fn short_hash(hash: &str) -> &str {
    hash.get(..12).unwrap_or(hash)
}

#[derive(Debug)]
pub enum LinearizeError {
    Agent(AgentError),
    MissingDecision(AgentId),
}

impl fmt::Display for LinearizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::MissingDecision(agent_id) => {
                write!(formatter, "missing linearize decision for `{agent_id}`")
            }
        }
    }
}

impl std::error::Error for LinearizeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Agent(error) => Some(error),
            Self::MissingDecision(_) => None,
        }
    }
}
