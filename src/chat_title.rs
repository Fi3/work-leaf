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
    sanitized_chat_title(first_prompt).unwrap_or_else(|| "chat".to_string())
}

fn sanitized_chat_title(raw: &str) -> Option<String> {
    const MAX_TITLE_CHARS: usize = 80;

    let mut title = String::new();
    let mut pending_separator = false;

    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_separator && !title.is_empty() && title.len() < MAX_TITLE_CHARS {
                title.push('-');
            }
            pending_separator = false;
            if title.len() == MAX_TITLE_CHARS {
                break;
            }
            title.push(ch.to_ascii_lowercase());
        } else {
            pending_separator = true;
        }
    }

    while title.ends_with('-') {
        title.pop();
    }

    if title.is_empty() { None } else { Some(title) }
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
}
