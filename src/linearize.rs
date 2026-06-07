use std::collections::BTreeMap;
use std::fmt;

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
        commits
            .iter()
            .map(|commit| LinearizeQuestion {
                agent_id: commit.agent_id.clone(),
                feature: commit.feature.clone(),
                prompt: format!(
                    "For Agent-ID {} ({}) should this patch keep a final commit, integrate into another commit, or be grouped with other reviewed chat ids?",
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
        for commit in &plan.commits {
            if !plan.decisions.contains_key(&commit.agent_id) {
                return Err(LinearizeError::MissingDecision(commit.agent_id.clone()));
            }
        }
        Ok(())
    }
}

fn build_interactive_linearize_prompt(commits: &[AgentCommit]) -> String {
    let mut prompt = String::new();
    prompt.push_str("You are the work-leaf linearizer for reviewed agent patches.\n\n");
    prompt.push_str("Reviewed commits:\n");
    for commit in commits {
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
        "\nRequired workflow:\n\
1. Inspect git history, the current branch, and the merge base with main or master before proposing a rewrite.\n\
2. Propose the solution before changing history. For each reviewed patch, state which final commit message should be kept, which provisional commit message should be removed, and whether related work should be grouped or merged.\n\
3. Ask the user to accept the solution or request changes. Do not rewrite history until the user accepts.\n\
4. After acceptance, rewrite provisional work-leaf commits into coherent final commits and remove provisional agent commits.\n\
5. Keep the diff against main/master as small as possible while preserving reviewed behavior.\n\
6. Run the checks required by the repository instructions and iterate until they pass.\n\
7. Report the final commit messages, removed provisional messages, grouping decisions, and verification results.\n\
\nUse orchestrator mediation for file access, command classification, patches, and any operation that mutates files or history.\n",
    );
    prompt
}

fn build_linearize_prompt(plan: &LinearizePlan) -> String {
    let mut prompt = String::new();
    prompt.push_str("You are the work-leaf linearizer for reviewed agent patches.\n");
    prompt.push_str(
        "Rewrite the provisional git history into clean final commits using the decisions below.\n",
    );
    prompt.push_str("Merge commits selected for integration into the commit that best carries their behavior, and make the diff against master/main as small as possible.\n\n");

    prompt.push_str("Reviewed commits:\n");
    for commit in &plan.commits {
        prompt.push_str(&format!(
            "- {} {} feature={} reason={}\n",
            commit.agent_id, commit.hash, commit.feature, commit.reason
        ));
    }

    prompt.push_str("\nDecisions:\n");
    for commit in &plan.commits {
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

    prompt.push_str("\nIterate until the verification commands pass. Keep the resulting history minimal and coherent for human review.\n");
    prompt
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
