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
    MouseClick { column: u16, row: u16 },
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
    pub hidden: bool,
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
            hidden: false,
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

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
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
enum LeftPaneClickTarget {
    Command,
    Agent(AgentId),
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
    control_selected: usize,
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
            control_selected: 0,
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

    pub fn control_selected_row(&self) -> usize {
        self.control_selected
    }

    pub fn add_agent(&mut self, agent: AgentListEntry) {
        self.agents.push(agent);
    }

    pub fn select_agent(&mut self, agent_id: &AgentId) -> Result<(), String> {
        if self.agents.iter().any(|agent| &agent.id == agent_id) {
            self.right_visible = true;
            self.selected_agent = Some(agent_id.clone());
            self.windows[self.active_window] = UiWindow::chat(agent_id.clone());
            self.control_selected = self
                .visible_agent_indices()
                .iter()
                .position(|index| self.agents[*index].id == *agent_id)
                .map(|position| position + 1)
                .unwrap_or(self.control_selected);
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
        self.control_selected = 0;
    }

    pub fn handle_key(&mut self, key: UiKey) -> Vec<UiAction> {
        if let UiKey::MouseClick { column, row } = key {
            self.pending = None;
            return self.handle_mouse_click(column, row);
        }

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
            UiKey::Char(',') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.right_visible = !self.right_visible;
                if !self.right_visible {
                    self.focus = PaneFocus::Left;
                }
                Vec::new()
            }
            UiKey::Char('j') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(1);
                Vec::new()
            }
            UiKey::Char('k') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(-1);
                Vec::new()
            }
            UiKey::Char('l') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.open_control_selection();
                Vec::new()
            }
            UiKey::Char('x') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.hide_control_selection();
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
        if self.control_selected == 0 {
            rendered.push_str("> work-leaf  command\n");
        } else {
            rendered.push_str("  work-leaf  command\n");
        }
        for (visible_position, agent_index) in self.visible_agent_indices().iter().enumerate() {
            let agent = &self.agents[*agent_index];
            if self.control_selected == visible_position + 1 {
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
        let mut rendered = String::from("\u{1b}[H");
        rendered.push_str(&buffer_to_string(&buffer));
        rendered.push_str(&self.cursor_sequence(right_content, prompt));
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
        let mut items = vec![ListItem::new(if self.control_selected == 0 {
            Spans::from(vec![Span::raw("> work-leaf  command")])
        } else {
            Spans::from(vec![Span::raw("  work-leaf  command")])
        })];
        for (visible_position, agent_index) in self.visible_agent_indices().iter().enumerate() {
            let agent = &self.agents[*agent_index];
            let mut line = Vec::new();
            line.push(Span::raw(
                if self.control_selected == visible_position + 1 {
                    "> "
                } else {
                    "  "
                },
            ));
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

    fn cursor_sequence(&self, right_content: &str, prompt: &str) -> String {
        let (row, column) = if self.mode == UiMode::Prompt {
            let prompt_column = prompt
                .chars()
                .count()
                .saturating_add(2)
                .min(usize::from(u16::MAX)) as u16;
            (self.height, prompt_column)
        } else {
            match self.focus {
                PaneFocus::Left => (self.control_cursor_row(), 2),
                PaneFocus::Right => self.right_cursor_position(right_content),
            }
        };
        let row = row.clamp(1, self.height.max(1));
        let column = column.clamp(1, self.width.max(1));
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
        let Some(agent_id) = self.action_agent_id() else {
            return Vec::new();
        };
        self.split_chats.push(agent_id.clone());
        vec![UiAction::OpenChatSamePane(agent_id)]
    }

    fn open_selected_new_window(&mut self) -> Vec<UiAction> {
        let Some(agent_id) = self.action_agent_id() else {
            return Vec::new();
        };
        self.windows.push(UiWindow::chat(agent_id.clone()));
        self.active_window = self.windows.len() - 1;
        self.right_visible = true;
        vec![UiAction::OpenChatNewWindow(agent_id)]
    }

    fn fork_selected_agent(&self) -> Vec<UiAction> {
        self.action_agent_id()
            .map(UiAction::ForkAgent)
            .into_iter()
            .collect()
    }

    fn open_control_selection(&mut self) {
        if self.control_selected == 0 {
            self.select_command_interface();
            self.right_visible = true;
            self.focus = PaneFocus::Right;
            return;
        }
        if let Some(agent_id) = self.control_selected_agent_id() {
            let _ = self.select_agent(&agent_id);
            self.focus = PaneFocus::Right;
        }
    }

    fn handle_mouse_click(&mut self, column: u16, row: u16) -> Vec<UiAction> {
        if column == 0 || row == 0 {
            return Vec::new();
        }

        let layout = self.layout();
        let left_width = if self.right_visible {
            layout.left_width
        } else {
            self.width
        };

        if column <= left_width {
            self.mode = UiMode::Command;
            self.focus = PaneFocus::Left;
            let Some(target) = self.left_pane_click_target(row) else {
                return Vec::new();
            };
            match target {
                LeftPaneClickTarget::Command => {
                    self.select_command_interface();
                    self.right_visible = true;
                    self.focus = PaneFocus::Right;
                }
                LeftPaneClickTarget::Agent(agent_id) => {
                    let _ = self.activate_agent_chat(&agent_id);
                }
            }
        } else if self.right_visible {
            self.focus = PaneFocus::Right;
            self.mode = if self.selected_agent.is_some()
                && self.windows[self.active_window].surface == UiSurface::AgentChat
            {
                UiMode::Insert
            } else {
                UiMode::Command
            };
        }

        Vec::new()
    }

    fn hide_control_selection(&mut self) {
        if self.control_selected == 0 {
            return;
        }
        let Some(agent_index) = self.control_selected_agent_index() else {
            return;
        };
        let hidden_agent = self.agents[agent_index].id.clone();
        self.agents[agent_index].hidden = true;
        if self
            .selected_agent
            .as_ref()
            .is_some_and(|selected| selected == &hidden_agent)
        {
            self.select_command_interface();
        }
        self.clamp_control_selection();
    }

    fn move_control_selection(&mut self, delta: isize) {
        let max_row = self.visible_agent_indices().len();
        let current = self.control_selected as isize;
        let next = (current + delta).clamp(0, max_row as isize);
        self.control_selected = next as usize;
    }

    fn clamp_control_selection(&mut self) {
        let max_row = self.visible_agent_indices().len();
        if self.control_selected > max_row {
            self.control_selected = max_row;
        }
    }

    fn visible_agent_indices(&self) -> Vec<usize> {
        self.agents
            .iter()
            .enumerate()
            .filter_map(|(index, agent)| (!agent.hidden).then_some(index))
            .collect()
    }

    fn control_selected_agent_index(&self) -> Option<usize> {
        if self.control_selected == 0 {
            return None;
        }
        self.visible_agent_indices()
            .get(self.control_selected - 1)
            .copied()
    }

    fn control_selected_agent_id(&self) -> Option<AgentId> {
        self.control_selected_agent_index()
            .map(|index| self.agents[index].id.clone())
    }

    fn left_pane_click_target(&mut self, row: u16) -> Option<LeftPaneClickTarget> {
        let list_row = usize::from(row.saturating_sub(2));
        if row < 2 {
            return None;
        }
        if list_row == 0 {
            self.control_selected = 0;
            return Some(LeftPaneClickTarget::Command);
        }

        let mut current_row = 1;
        for (visible_position, agent_index) in self.visible_agent_indices().iter().enumerate() {
            let agent = &self.agents[*agent_index];
            if list_row == current_row {
                self.control_selected = visible_position + 1;
                return Some(LeftPaneClickTarget::Agent(agent.id.clone()));
            }
            current_row += 1;

            if !agent.modified_files.is_empty() {
                if list_row == current_row {
                    self.control_selected = visible_position + 1;
                    return Some(LeftPaneClickTarget::Agent(agent.id.clone()));
                }
                current_row += 1;
            }

            for linked_agents in [
                &agent.conflicting_agents,
                &agent.depends_on,
                &agent.depended_on_by,
            ] {
                if !linked_agents.is_empty() {
                    if list_row == current_row {
                        self.control_selected = visible_position + 1;
                        return linked_agents
                            .first()
                            .cloned()
                            .map(LeftPaneClickTarget::Agent);
                    }
                    current_row += 1;
                }
            }
        }

        None
    }

    fn action_agent_id(&self) -> Option<AgentId> {
        self.control_selected_agent_id()
            .or_else(|| self.selected_agent.clone())
    }

    fn control_cursor_row(&self) -> u16 {
        (self.control_selected + 2).min(usize::from(u16::MAX)) as u16
    }

    fn right_cursor_position(&self, right_content: &str) -> (u16, u16) {
        let layout = self.layout();
        let inner_width = layout.right_width.saturating_sub(2).max(1);
        let lines = right_content.lines().collect::<Vec<_>>();
        let Some(line) = lines.last().copied() else {
            return (2, layout.left_width.saturating_add(2));
        };
        if !line.starts_with("chat> ") {
            return (2, layout.left_width.saturating_add(2));
        }
        let previous_rows = lines[..lines.len() - 1]
            .iter()
            .map(|line| visual_rows(line, inner_width))
            .sum::<u16>();
        let line_len = line.chars().count().min(usize::from(u16::MAX)) as u16;
        let row = 2_u16
            .saturating_add(previous_rows)
            .saturating_add(line_len / inner_width);
        let column = layout
            .left_width
            .saturating_add(2)
            .saturating_add(line_len % inner_width);
        (row, column)
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
            output.push_str("\r\n");
        }
    }
    output
}

fn visual_rows(line: &str, width: u16) -> u16 {
    let width = width.max(1);
    let len = line.chars().count().min(usize::from(u16::MAX)) as u16;
    (len / width).saturating_add(1)
}
