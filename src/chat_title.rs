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

pub(crate) fn chat_title_prompt(first_prompt: &str) -> String {
    format!(
        "Name this work-leaf chat from the user's first prompt.\n\
Rules:\n\
- Return only the chat name, no prose and no quotes.\n\
- Derive the name from the first prompt only.\n\
- Use at most 80 characters.\n\
- Use lowercase words separated by hyphens.\n\
- Do not use spaces.\n\n\
First prompt:\n{first_prompt}"
    )
}

pub(crate) fn chat_title_from_llm_reply(reply: &str, first_prompt: &str) -> String {
    sanitized_chat_title(reply).unwrap_or_else(|| fallback_chat_title_from_prompt(first_prompt))
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
    use super::{chat_title_from_llm_reply, fallback_chat_title_from_prompt};

    #[test]
    fn llm_title_reply_is_sanitized_to_kebab_case_with_eighty_character_limit() {
        let title = chat_title_from_llm_reply(
            "OAuth Redirect Handler With Cookie Coverage And Callback Retry Audit Trail",
            "fallback prompt",
        );

        assert_eq!(
            title,
            "oauth-redirect-handler-with-cookie-coverage-and-callback-retry-audit-trail"
        );
        assert!(title.len() <= 80);
        assert!(!title.contains(' '));
    }

    #[test]
    fn empty_llm_title_falls_back_to_first_prompt() {
        assert_eq!(
            chat_title_from_llm_reply("!!!", "Please fix the OAuth redirect handler"),
            "please-fix-the-oauth-redirect-handler"
        );
    }

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
