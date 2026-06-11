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

    pub(crate) fn reserve_first_prompt_title(&mut self, agent_id: &AgentId) -> bool {
        if self.named_agents.contains(agent_id) {
            return false;
        }
        self.named_agents.insert(agent_id.clone());
        true
    }
}

pub(crate) fn fallback_chat_title_from_prompt(first_prompt: &str) -> String {
    summarized_chat_title(first_prompt).unwrap_or_else(|| "chat".to_string())
}

fn summarized_chat_title(raw: &str) -> Option<String> {
    const MAX_TITLE_CHARS: usize = 80;
    const MAX_TITLE_WORDS: usize = 6;

    let words = title_words(raw);
    if words.is_empty() {
        return None;
    }

    let mut selected = words
        .iter()
        .enumerate()
        .filter_map(|(index, word)| {
            (!is_low_signal_title_word(word) || should_keep_title_article(&words, index))
                .then_some(word)
        })
        .collect::<Vec<_>>();
    if selected.is_empty() {
        selected = words.iter().collect();
    }

    let mut title = String::new();
    for word in selected.into_iter().take(MAX_TITLE_WORDS) {
        let separator_len = usize::from(!title.is_empty());
        if title.len() + separator_len + word.len() > MAX_TITLE_CHARS {
            if title.is_empty() {
                title.push_str(&word[..MAX_TITLE_CHARS.min(word.len())]);
            }
            break;
        }
        if !title.is_empty() {
            title.push('-');
        }
        title.push_str(word);
    }

    if title.is_empty() { None } else { Some(title) }
}

fn title_words(raw: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            word.push(ch.to_ascii_lowercase());
        } else {
            push_title_word(&mut words, &mut word);
        }
    }
    push_title_word(&mut words, &mut word);
    words
}

fn push_title_word(words: &mut Vec<String>, word: &mut String) {
    if !word.is_empty() {
        words.push(std::mem::take(word));
    }
}

fn should_keep_title_article(words: &[String], index: usize) -> bool {
    matches!(words[index].as_str(), "a" | "an" | "the")
        && index > 0
        && is_action_title_word(&words[index - 1])
}

fn is_action_title_word(word: &str) -> bool {
    matches!(
        word,
        "add"
            | "build"
            | "create"
            | "delete"
            | "fix"
            | "implement"
            | "launch"
            | "make"
            | "refactor"
            | "remove"
            | "start"
            | "update"
            | "upgrade"
            | "write"
    )
}

fn is_low_signal_title_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "at"
            | "be"
            | "been"
            | "being"
            | "but"
            | "by"
            | "can"
            | "could"
            | "did"
            | "do"
            | "does"
            | "for"
            | "from"
            | "get"
            | "gets"
            | "got"
            | "had"
            | "has"
            | "have"
            | "if"
            | "in"
            | "is"
            | "it"
            | "just"
            | "like"
            | "look"
            | "looks"
            | "of"
            | "on"
            | "or"
            | "should"
            | "so"
            | "that"
            | "the"
            | "there"
            | "these"
            | "this"
            | "those"
            | "to"
            | "was"
            | "we"
            | "were"
            | "with"
            | "would"
            | "you"
    )
}

#[cfg(test)]
mod tests {
    use super::fallback_chat_title_from_prompt;

    #[test]
    fn fallback_title_uses_first_prompt_and_caps_long_names() {
        let title = fallback_chat_title_from_prompt(
            "implement a very long migration workflow with retries telemetry audit trail and rollback support",
        );

        assert!(title.len() <= 80);
        assert!(!title.contains(' '));
        assert!(title.starts_with("implement-a-very-long"));
    }

    #[test]
    fn fallback_title_summarizes_noisy_prompt_around_salient_words() {
        let title = fallback_chat_title_from_prompt(
            "it looks like that we there have been a BAD regression chat name for patch agents is not created by the system agent but it has to SUMMARIZE it",
        );

        assert_eq!(title, "bad-regression-chat-name-patch-agents");
    }
}
