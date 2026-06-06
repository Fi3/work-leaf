use std::path::PathBuf;

use crate::agent::AgentId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMode {
    Command,
    Insert,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaneFocus {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSurface {
    WorkLeafCommand,
    AgentChat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiKey {
    Char(char),
    Esc,
    CtrlW,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAction {
    OpenChatSamePane(AgentId),
    OpenChatNewWindow(AgentId),
    ForkAgent(AgentId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalLayout {
    pub left_width: u16,
    pub right_width: u16,
    pub height: u16,
    pub right_surface: Option<UiSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentListEntry {
    pub id: AgentId,
    pub feature: String,
    pub ready: bool,
    pub modified_files: Vec<PathBuf>,
    pub conflicting_agents: Vec<AgentId>,
    pub depends_on: Vec<AgentId>,
    pub depended_on_by: Vec<AgentId>,
}

impl AgentListEntry {
    pub fn new(id: AgentId, feature: impl Into<String>) -> Self {
        Self {
            id,
            feature: feature.into(),
            ready: false,
            modified_files: Vec::new(),
            conflicting_agents: Vec::new(),
            depends_on: Vec::new(),
            depended_on_by: Vec::new(),
        }
    }

    pub fn with_ready(mut self, ready: bool) -> Self {
        self.ready = ready;
        self
    }

    pub fn with_modified_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.modified_files.push(path.into());
        self
    }

    pub fn with_conflicting_agent(mut self, agent_id: AgentId) -> Self {
        self.conflicting_agents.push(agent_id);
        self
    }

    pub fn with_dependency(mut self, agent_id: AgentId) -> Self {
        self.depends_on.push(agent_id);
        self
    }

    pub fn with_dependent(mut self, agent_id: AgentId) -> Self {
        self.depended_on_by.push(agent_id);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingKey {
    CtrlW,
    G,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiWindow {
    surface: UiSurface,
    agent_id: Option<AgentId>,
}

impl UiWindow {
    fn command() -> Self {
        Self {
            surface: UiSurface::WorkLeafCommand,
            agent_id: None,
        }
    }

    fn chat(agent_id: AgentId) -> Self {
        Self {
            surface: UiSurface::AgentChat,
            agent_id: Some(agent_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalUi {
    width: u16,
    height: u16,
    mode: UiMode,
    focus: PaneFocus,
    right_visible: bool,
    agents: Vec<AgentListEntry>,
    selected_agent: Option<AgentId>,
    split_chats: Vec<AgentId>,
    windows: Vec<UiWindow>,
    active_window: usize,
    pending: Option<PendingKey>,
}

impl TerminalUi {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            mode: UiMode::Command,
            focus: PaneFocus::Left,
            right_visible: true,
            agents: Vec::new(),
            selected_agent: None,
            split_chats: Vec::new(),
            windows: vec![UiWindow::command()],
            active_window: 0,
            pending: None,
        }
    }

    pub fn layout(&self) -> TerminalLayout {
        let left_width = if self.right_visible {
            self.width / 5
        } else {
            self.width
        };
        let right_width = self.width.saturating_sub(left_width);
        TerminalLayout {
            left_width,
            right_width,
            height: self.height,
            right_surface: self
                .right_visible
                .then_some(self.windows[self.active_window].surface),
        }
    }

    pub fn mode(&self) -> UiMode {
        self.mode
    }

    pub fn focus(&self) -> PaneFocus {
        self.focus
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn active_window(&self) -> usize {
        self.active_window
    }

    pub fn add_agent(&mut self, agent: AgentListEntry) {
        self.agents.push(agent);
    }

    pub fn select_agent(&mut self, agent_id: &AgentId) -> Result<(), String> {
        if self.agents.iter().any(|agent| &agent.id == agent_id) {
            self.selected_agent = Some(agent_id.clone());
            self.windows[self.active_window] = UiWindow::chat(agent_id.clone());
            Ok(())
        } else {
            Err(format!("unknown agent `{agent_id}`"))
        }
    }

    pub fn select_command_interface(&mut self) {
        self.selected_agent = None;
        self.windows[self.active_window] = UiWindow::command();
    }

    pub fn handle_key(&mut self, key: UiKey) -> Vec<UiAction> {
        if let Some(pending) = self.pending.take() {
            return self.handle_pending_key(pending, key);
        }

        match key {
            UiKey::Esc => {
                self.mode = UiMode::Command;
                Vec::new()
            }
            UiKey::CtrlW if self.mode == UiMode::Command => {
                self.pending = Some(PendingKey::CtrlW);
                Vec::new()
            }
            UiKey::Char('g') if self.mode == UiMode::Command => {
                self.pending = Some(PendingKey::G);
                Vec::new()
            }
            UiKey::Char('i') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.mode = UiMode::Insert;
                Vec::new()
            }
            UiKey::Char(',') if self.mode == UiMode::Command => {
                self.right_visible = !self.right_visible;
                if !self.right_visible {
                    self.focus = PaneFocus::Left;
                }
                Vec::new()
            }
            UiKey::Char('s') if self.mode == UiMode::Command => self.open_selected_same_pane(),
            UiKey::Char('t') if self.mode == UiMode::Command => self.open_selected_new_window(),
            UiKey::Char('f') if self.mode == UiMode::Command => self.fork_selected_agent(),
            _ => Vec::new(),
        }
    }

    pub fn render_left_pane(&self) -> String {
        let mut rendered = String::new();
        if self.selected_agent.is_none() {
            rendered.push_str("> work-leaf  command\n");
        } else {
            rendered.push_str("  work-leaf  command\n");
        }
        for agent in &self.agents {
            let selected = self
                .selected_agent
                .as_ref()
                .is_some_and(|selected| selected == &agent.id);
            if selected {
                rendered.push_str("> ");
            } else {
                rendered.push_str("  ");
            }
            rendered.push_str(agent.id.as_str());
            rendered.push_str("  working: ");
            rendered.push_str(&agent.feature);
            if agent.ready {
                rendered.push_str("  \u{1b}[7mREADY\u{1b}[0m");
            }
            rendered.push('\n');
            if !agent.modified_files.is_empty() {
                rendered.push_str("    ");
                rendered.push_str("files: ");
                rendered.push_str(
                    &agent
                        .modified_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                rendered.push('\n');
            }
            self.render_agent_links("conflicts", &agent.conflicting_agents, &mut rendered);
            self.render_agent_links("depends-on", &agent.depends_on, &mut rendered);
            self.render_agent_links("depended-on-by", &agent.depended_on_by, &mut rendered);
        }
        rendered
    }

    pub fn render_screen(&self, right_content: &str) -> String {
        let layout = self.layout();
        let left_lines = self.render_left_pane();
        let left_lines = left_lines.lines().collect::<Vec<_>>();
        let right_lines = self.render_right_pane(right_content);
        let right_lines = right_lines.lines().collect::<Vec<_>>();
        let mut rendered = String::from("\u{1b}[2J\u{1b}[H");

        let content_height = self.height.saturating_sub(1) as usize;
        for row in 0..content_height {
            let left = left_lines.get(row).copied().unwrap_or("");
            rendered.push_str(&fit_cell(left, layout.left_width as usize));
            if layout.right_surface.is_some() {
                rendered.push('│');
                let right = right_lines.get(row).copied().unwrap_or("");
                rendered.push_str(&fit_cell(
                    right,
                    layout.right_width.saturating_sub(1) as usize,
                ));
            }
            rendered.push('\n');
        }

        rendered.push_str(&self.render_status_line());
        rendered.push('\n');
        rendered
    }

    fn handle_pending_key(&mut self, pending: PendingKey, key: UiKey) -> Vec<UiAction> {
        match (pending, key) {
            (PendingKey::CtrlW, UiKey::Char('h')) if self.mode == UiMode::Command => {
                self.focus = PaneFocus::Left;
                Vec::new()
            }
            (PendingKey::CtrlW, UiKey::Char('l'))
                if self.mode == UiMode::Command && self.right_visible =>
            {
                self.focus = PaneFocus::Right;
                self.mode = UiMode::Command;
                Vec::new()
            }
            (PendingKey::G, UiKey::Char('t')) if self.mode == UiMode::Command => {
                self.next_window();
                Vec::new()
            }
            (PendingKey::G, UiKey::Char('T')) if self.mode == UiMode::Command => {
                self.previous_window();
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn open_selected_same_pane(&mut self) -> Vec<UiAction> {
        let Some(agent_id) = self.selected_agent.clone() else {
            return Vec::new();
        };
        self.split_chats.push(agent_id.clone());
        vec![UiAction::OpenChatSamePane(agent_id)]
    }

    fn open_selected_new_window(&mut self) -> Vec<UiAction> {
        let Some(agent_id) = self.selected_agent.clone() else {
            return Vec::new();
        };
        self.windows.push(UiWindow::chat(agent_id.clone()));
        self.active_window = self.windows.len() - 1;
        self.right_visible = true;
        vec![UiAction::OpenChatNewWindow(agent_id)]
    }

    fn fork_selected_agent(&self) -> Vec<UiAction> {
        self.selected_agent
            .clone()
            .map(UiAction::ForkAgent)
            .into_iter()
            .collect()
    }

    fn next_window(&mut self) {
        if !self.windows.is_empty() {
            self.active_window = (self.active_window + 1) % self.windows.len();
        }
    }

    fn previous_window(&mut self) {
        if !self.windows.is_empty() {
            self.active_window = if self.active_window == 0 {
                self.windows.len() - 1
            } else {
                self.active_window - 1
            };
        }
    }

    fn render_agent_links(&self, label: &str, agents: &[AgentId], rendered: &mut String) {
        if agents.is_empty() {
            return;
        }
        rendered.push_str("    ");
        rendered.push_str(label);
        rendered.push_str(": ");
        for (index, agent_id) in agents.iter().enumerate() {
            if index > 0 {
                rendered.push_str(", ");
            }
            rendered.push_str(agent_id.as_str());
        }
        rendered.push('\n');
    }

    fn render_right_pane(&self, right_content: &str) -> String {
        let mut rendered = String::new();
        match self.windows[self.active_window].surface {
            UiSurface::WorkLeafCommand => rendered.push_str("command\n"),
            UiSurface::AgentChat => {
                rendered.push_str("chat ");
                if let Some(agent_id) = &self.windows[self.active_window].agent_id {
                    rendered.push_str(agent_id.as_str());
                }
                rendered.push('\n');
            }
        }
        rendered.push_str(right_content);
        rendered
    }

    fn render_status_line(&self) -> String {
        format!(
            "mode={} focus={} window={}/{}",
            self.mode.as_str(),
            self.focus.as_str(),
            self.active_window + 1,
            self.windows.len()
        )
    }
}

impl UiMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Insert => "insert",
        }
    }
}

impl PaneFocus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

fn fit_cell(text: &str, width: usize) -> String {
    let clipped = clip_ansi_text(text, width);
    let visible = visible_width(&clipped);
    if visible >= width {
        clipped
    } else {
        format!("{clipped}{}", " ".repeat(width - visible))
    }
}

fn clip_ansi_text(text: &str, width: usize) -> String {
    let mut output = String::new();
    let mut visible = 0;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            output.push(ch);
            for next in chars.by_ref() {
                output.push(next);
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        if visible >= width {
            break;
        }
        output.push(ch);
        visible += 1;
    }
    output
}

fn visible_width(text: &str) -> usize {
    let mut width = 0;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            width += 1;
        }
    }
    width
}
