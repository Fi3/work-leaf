use std::collections::BTreeSet;

use crate::agent::AgentId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ChatTitleAgent {
    named_agents: BTreeSet<AgentId>,
}

impl ChatTitleAgent {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn mark_named(&mut self, agent_id: &AgentId) {
        self.named_agents.insert(agent_id.clone());
    }

    pub(crate) fn title_for_first_prompt(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
    ) -> Option<String> {
        if self.named_agents.contains(agent_id) {
            return None;
        }
        self.named_agents.insert(agent_id.clone());
        Some(chat_title_from_prompt(prompt))
    }
}

pub(crate) fn chat_title_from_prompt(prompt: &str) -> String {
    const STOP_WORDS: &[&str] = &[
        "a",
        "an",
        "and",
        "as",
        "build",
        "create",
        "fix",
        "for",
        "implement",
        "please",
        "the",
        "to",
        "update",
        "with",
    ];

    let meaningful = collect_title_words(
        normalized_prompt_words(prompt).filter(|word| !STOP_WORDS.contains(&word.as_str())),
    );
    if !meaningful.is_empty() {
        return meaningful.join(" ");
    }

    let fallback = collect_title_words(normalized_prompt_words(prompt));
    if fallback.is_empty() {
        "chat".to_string()
    } else {
        fallback.join(" ")
    }
}

fn normalized_prompt_words(prompt: &str) -> impl Iterator<Item = String> + '_ {
    prompt
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_')
                .to_ascii_lowercase()
        })
        .filter(|word| !word.is_empty())
}

fn collect_title_words(words: impl Iterator<Item = String>) -> Vec<String> {
    const MAX_TITLE_CHARS: usize = 32;
    const MAX_TITLE_WORDS: usize = 4;

    let mut title_words = Vec::new();
    let mut title_len = 0;

    for word in words {
        let next_len = if title_words.is_empty() {
            word.len()
        } else {
            title_len + 1 + word.len()
        };

        if next_len > MAX_TITLE_CHARS {
            if title_words.is_empty() {
                title_words.push(word.chars().take(MAX_TITLE_CHARS).collect());
            }
            continue;
        }

        title_len = next_len;
        title_words.push(word);
        if title_words.len() == MAX_TITLE_WORDS {
            break;
        }
    }

    title_words
}
