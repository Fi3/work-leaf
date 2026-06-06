use std::path::PathBuf;

use crate::agent::AgentId;
use tui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMode {
    Command,
    Insert,
    Prompt,
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

    pub fn selected_agent(&self) -> Option<&AgentId> {
        self.selected_agent.as_ref()
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

    pub fn activate_agent_chat(&mut self, agent_id: &AgentId) -> Result<(), String> {
        self.select_agent(agent_id)?;
        self.focus = PaneFocus::Right;
        self.mode = UiMode::Insert;
        Ok(())
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
            UiKey::Char('i') if self.mode == UiMode::Command => {
                self.mode = UiMode::Insert;
                Vec::new()
            }
            UiKey::Char(':') if self.mode == UiMode::Command => {
                self.mode = UiMode::Prompt;
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
        self.render_screen_with_prompt(right_content, "")
    }

    pub fn render_screen_with_prompt(&self, right_content: &str, prompt: &str) -> String {
        let buffer = self.render_tui_buffer(right_content, prompt);
        let mut rendered = String::from("\u{1b}[2J\u{1b}[H");
        rendered.push_str(&buffer_to_string(&buffer));
        rendered.push_str(&self.cursor_sequence(prompt));
        rendered
    }

    fn render_tui_buffer(&self, right_content: &str, prompt: &str) -> Buffer {
        let backend = TestBackend::new(self.width, self.height);
        let mut terminal = Terminal::new(backend).expect("test backend is valid");
        terminal
            .draw(|frame| {
                let area = frame.size();
                let body_height = area.height.saturating_sub(1);
                let body = Rect::new(area.x, area.y, area.width, body_height);
                let bottom = Rect::new(area.x, body_height, area.width, 1);
                let layout = self.layout();
                let panes = if layout.right_surface.is_some() {
                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(layout.left_width),
                            Constraint::Length(layout.right_width),
                        ])
                        .split(body)
                } else {
                    vec![body]
                };

                frame.render_widget(self.left_widget(), panes[0]);
                if panes.len() > 1 {
                    frame.render_widget(self.right_widget(right_content), panes[1]);
                }
                frame.render_widget(Paragraph::new(self.bottom_line(prompt)), bottom);
            })
            .expect("test backend draw succeeds");
        terminal.backend().buffer().clone()
    }

    fn left_widget(&self) -> List<'static> {
        let mut items = vec![ListItem::new(if self.selected_agent.is_none() {
            Spans::from(vec![Span::raw("> work-leaf  command")])
        } else {
            Spans::from(vec![Span::raw("  work-leaf  command")])
        })];
        for agent in &self.agents {
            let mut line = Vec::new();
            let selected = self
                .selected_agent
                .as_ref()
                .is_some_and(|selected| selected == &agent.id);
            line.push(Span::raw(if selected { "> " } else { "  " }));
            line.push(Span::raw(agent.id.as_str().to_string()));
            line.push(Span::raw("  working: "));
            line.push(Span::raw(agent.feature.clone()));
            if agent.ready {
                line.push(Span::raw("  "));
                line.push(Span::styled(
                    "READY",
                    Style::default().add_modifier(Modifier::REVERSED),
                ));
            }
            items.push(ListItem::new(Spans::from(line)));
            if !agent.modified_files.is_empty() {
                items.push(ListItem::new(Spans::from(vec![Span::raw(format!(
                    "    files: {}",
                    agent
                        .modified_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))])));
            }
            for (label, agents) in [
                ("conflicts", &agent.conflicting_agents),
                ("depends-on", &agent.depends_on),
                ("depended-on-by", &agent.depended_on_by),
            ] {
                if !agents.is_empty() {
                    items.push(ListItem::new(Spans::from(vec![Span::raw(format!(
                        "    {label}: {}",
                        agents
                            .iter()
                            .map(AgentId::as_str)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))])));
                }
            }
        }
        List::new(items).block(Block::default().title("work-leaf").borders(Borders::ALL))
    }

    fn right_widget(&self, right_content: &str) -> Paragraph<'static> {
        let title = match self.windows[self.active_window].surface {
            UiSurface::WorkLeafCommand => "command",
            UiSurface::AgentChat => "chat",
        };
        Paragraph::new(right_content.to_string())
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: false })
    }

    fn bottom_line(&self, prompt: &str) -> String {
        if self.mode == UiMode::Prompt {
            format!(":{prompt}")
        } else {
            self.render_status_line()
        }
    }

    fn cursor_sequence(&self, prompt: &str) -> String {
        let layout = self.layout();
        let (row, column) = if self.mode == UiMode::Prompt {
            let prompt_column = layout.left_width.saturating_add(1);
            (self.height, prompt_column)
        } else {
            match self.focus {
                PaneFocus::Left => (1, 1),
                PaneFocus::Right => (1, layout.left_width.saturating_add(1)),
            }
        };
        let _ = prompt;
        format!("\u{1b}[{row};{column}H")
    }

    fn handle_pending_key(&mut self, pending: PendingKey, key: UiKey) -> Vec<UiAction> {
        match (pending, key) {
            (PendingKey::CtrlW, UiKey::Char('h')) if self.mode == UiMode::Command => {
                self.focus = PaneFocus::Left;
                Vec::new()
            }
            (PendingKey::CtrlW, UiKey::Char('k')) if self.mode == UiMode::Command => {
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
            (PendingKey::CtrlW, UiKey::Char('j'))
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
            Self::Prompt => "prompt",
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

fn buffer_to_string(buffer: &Buffer) -> String {
    let mut output = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            output.push_str(&buffer.get(x, y).symbol);
        }
        if y + 1 < buffer.area.height {
            output.push('\n');
        }
    }
    output
}
