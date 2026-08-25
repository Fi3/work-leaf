use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, CommandChat,
    MessageRole, TerminalApp,
};

#[test]
fn quality_completion_behavior() {
    let root = temporary_git_repo();
    let backend = CompletionProbeBackend::new();
    let chat = CommandChat::new(root.clone(), backend.clone()).with_max_review_rounds(3);
    let mut app = TerminalApp::new(chat, 120, 30);

    app.handle_bytes(b":new complete the reviewed fixture\n");
    assert!(app.wait_for_idle(Duration::from_secs(4)));

    let prompt_frame = app.render_frame();
    assert!(has_completion_question(&prompt_frame), "{prompt_frame}");
    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(agent_row_is_highlighted(
        &app.ui().render_left_pane(),
        "user-1"
    ));

    let sends_after_review = backend.sends();
    app.handle_bytes(b"yes\n");
    assert_eq!(
        backend.sends(),
        sends_after_review,
        "yes must close locally instead of reaching the backend"
    );
    let closed_frame = app.render_frame();
    assert!(
        closed_frame.to_ascii_lowercase().contains("feature closed")
            || !agent_row_exists(&app.ui().render_left_pane(), "user-1"),
        "closing must have a visible effect\n{closed_frame}"
    );

    app.handle_bytes(b"follow up after closure\n");
    assert!(app.wait_for_idle(Duration::from_secs(4)));
    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(agent_row_exists(&app.ui().render_left_pane(), "user-1"));
    let reopened = app.render_frame();
    assert!(
        reopened.contains("user: follow up after closure"),
        "{reopened}"
    );
    assert!(backend.sends().iter().any(|(agent_id, prompt)| {
        agent_id.as_str() == "user-1" && prompt == "follow up after closure"
    }));

    drop(app);
    fs::remove_dir_all(root).unwrap();
}

fn agent_row_exists(left_pane: &str, agent_id: &str) -> bool {
    left_pane
        .lines()
        .any(|line| agent_row_matches(line, agent_id))
}

fn agent_row_is_highlighted(left_pane: &str, agent_id: &str) -> bool {
    left_pane
        .lines()
        .any(|line| agent_row_matches(line, agent_id) && line.contains("\u{1b}[7m"))
}

fn agent_row_matches(line: &str, agent_id: &str) -> bool {
    line.contains(&format!(" {agent_id} ")) || line.ends_with(&format!(" {agent_id}"))
}

fn has_completion_question(frame: &str) -> bool {
    let normalized = frame
        .chars()
        .map(|character| {
            if ('\u{2500}'..='\u{257f}').contains(&character) {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    normalized.contains("is this feature done")
        && normalized.contains("yes")
        && normalized.contains("no")
}

#[derive(Clone, Debug)]
struct CompletionProbeBackend {
    state: Arc<Mutex<CompletionProbeState>>,
}

#[derive(Debug)]
struct CompletionProbeState {
    sends: Vec<(AgentId, String)>,
    sessions: BTreeMap<AgentId, AgentSession>,
}

impl CompletionProbeBackend {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CompletionProbeState {
                sends: Vec::new(),
                sessions: BTreeMap::new(),
            })),
        }
    }

    fn sends(&self) -> Vec<(AgentId, String)> {
        self.state.lock().unwrap().sends.clone()
    }
}

impl AgentBackend for CompletionProbeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            session.push_message(MessageRole::Agent, "review-fixture");
            return Ok(session);
        }

        let mut state = self.state.lock().unwrap();
        let agent_id = request.id.clone();
        let mut session = AgentSession::new(request);
        let reply = if agent_id.as_str().starts_with("review-") {
            "NO_FINDINGS".to_string()
        } else {
            first_patch()
        };
        session.push_message(MessageRole::Agent, reply);
        state.sessions.insert(agent_id, session.clone());
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str().starts_with("title-") {
            return Ok(ChatMessage::new(MessageRole::Agent, "review-fixture"));
        }

        let mut state = self.state.lock().unwrap();
        state.sends.push((agent_id.clone(), prompt.to_string()));
        let reply = if agent_id.as_str().starts_with("review-") {
            "NO_FINDINGS".to_string()
        } else if prompt == "follow up after closure" {
            second_patch()
        } else if prompt.to_ascii_lowercase().contains("summar") {
            "summary: README fixture behavior".to_string()
        } else {
            "@work-leaf done".to_string()
        };
        if let Some(session) = state.sessions.get_mut(agent_id) {
            session.push_message(MessageRole::User, prompt);
            session.push_message(MessageRole::Agent, reply.clone());
        }
        Ok(ChatMessage::new(MessageRole::Agent, reply))
    }
}

fn first_patch() -> String {
    concat!(
        "implementation complete\n",
        "@work-leaf patch update fixture\n",
        "--- a/README.md\n",
        "+++ b/README.md\n",
        "@@ -1 +1 @@\n",
        "-before\n",
        "+after\n",
        "@work-leaf end\n",
        "@work-leaf done"
    )
    .to_string()
}

fn second_patch() -> String {
    concat!(
        "reopened implementation complete\n",
        "@work-leaf patch polish fixture\n",
        "--- a/README.md\n",
        "+++ b/README.md\n",
        "@@ -1 +1 @@\n",
        "-after\n",
        "+after reopened\n",
        "@work-leaf end\n",
        "@work-leaf done"
    )
    .to_string()
}

fn temporary_git_repo() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "work-leaf-quality-completion-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    run_git(&root, &["init", "-q"]);
    run_git(&root, &["config", "user.email", "quality@example.com"]);
    run_git(&root, &["config", "user.name", "Quality Evaluator"]);
    fs::write(root.join("README.md"), "before\n").unwrap();
    run_git(&root, &["add", "README.md"]);
    run_git(&root, &["commit", "-q", "-m", "ADD completion fixture"]);
    root
}

fn run_git(root: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}
