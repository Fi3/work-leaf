# Work Leaf Context Bundle

This file contains orchestrator-mediated read output. Use it as read-only context; submit project changes through `@work-leaf edit`.

----- BEGIN FILE src/ui.rs -----
digest: fnv64:b5b7e79f1d5d0104; bytes:57391

use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    time::{Duration, Instant},
};

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
    Visual,
    VisualLine,
    VisualBlock,
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
    CtrlV,
    Up,
    Down,
    Left,
    Right,
    MouseClick { column: u16, row: u16 },
    MouseScrollUp { column: u16, row: u16 },
    MouseScrollDown { column: u16, row: u16 },
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
struct StatusNotice {
    message: String,
    expires_at: Instant,
}

const STATUS_NOTICE_SECONDS: u64 = 5;
const COMMAND_MODE_TYPING_NOTICE_THRESHOLD: usize = 5;
const COMMAND_MODE_TYPING_NOTICE: &str = "command mode: press i for insert mode before typing";
const CTRL_C_EXIT_NOTICE: &str = "to exit, press Esc then :q then Enter";

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
struct PromptView {
    line: String,
    cursor_column: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PaneLine {
    text: String,
    reversed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisualKind {
    Character,
    Line,
    Block,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VisualPosition {
    row: usize,
    column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VisualSelection {
    pane: PaneFocus,
    kind: VisualKind,
    anchor: VisualPosition,
    cursor: VisualPosition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalUi {
    width: u16,
    height: u16,
    mode: UiMode,
    focus: PaneFocus,
    left_visible: bool,
    agents: Vec<AgentListEntry>,
    selected_agent: Option<AgentId>,
    control_selected: usize,
    split_chats: Vec<AgentId>,
    windows: Vec<UiWindow>,
    active_window: usize,
    right_scroll_rows: usize,
    visual_selection: Option<VisualSelection>,
    clipboard: String,
    pending_clipboard: RefCell<Option<String>>,
    last_right_content: RefCell<String>,
    last_right_cursor_column: Cell<Option<usize>>,
    pending: Option<PendingKey>,
    pending_bell: Cell<bool>,
    status_notice: Option<StatusNotice>,
    command_mode_typing_count: usize,
    command_mode_typing_controls_only: bool,
}

impl TerminalUi {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            mode: UiMode::Command,
            focus: PaneFocus::Left,
            left_visible: true,
            agents: Vec::new(),
            selected_agent: None,
            control_selected: 0,
            split_chats: Vec::new(),
            windows: vec![UiWindow::command()],
            active_window: 0,
            right_scroll_rows: 0,
            visual_selection: None,
            clipboard: String::new(),
            pending_clipboard: RefCell::new(None),
            last_right_content: RefCell::new(String::new()),
            last_right_cursor_column: Cell::new(None),
            pending: None,
            pending_bell: Cell::new(false),
            status_notice: None,
            command_mode_typing_count: 0,
            command_mode_typing_controls_only: true,
        }
    }

    pub fn layout(&self) -> TerminalLayout {
        let left_width = if self.left_visible { self.width / 5 } else { 0 };
        let right_width = self.width.saturating_sub(left_width);
        TerminalLayout {
            left_width,
            right_width,
            height: self.height,
            right_surface: Some(self.windows[self.active_window].surface),
        }
    }

    pub fn mode(&self) -> UiMode {
        self.mode
    }

    pub fn focus(&self) -> PaneFocus {
        self.focus
    }

    pub(crate) fn show_ctrl_c_exit_notice(&mut self) {
        self.show_status_notice(
            CTRL_C_EXIT_NOTICE,
            Duration::from_secs(STATUS_NOTICE_SECONDS),
        );
    }

    pub(crate) fn has_status_notice(&self) -> bool {
        self.status_notice.is_some()
    }

    pub(crate) fn clear_expired_status_notice(&mut self) {
        if self.status_notice_expired() {
            self.status_notice = None;
        }
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

    pub fn clipboard_text(&self) -> &str {
        &self.clipboard
    }

    pub fn add_agent(&mut self, agent: AgentListEntry) {
        self.agents.push(agent);
    }

    pub(crate) fn set_agent_feature(
        &mut self,
        agent_id: &AgentId,
        feature: impl Into<String>,
    ) -> Result<(), String> {
        let Some(agent) = self.agents.iter_mut().find(|agent| &agent.id == agent_id) else {
            return Err(format!("unknown agent `{agent_id}`"));
        };
        agent.feature = feature.into();
        Ok(())
    }

    pub(crate) fn set_agent_ready_state(
        &mut self,
        agent_id: &AgentId,
        ready: bool,
    ) -> Result<(), String> {
        let Some(agent) = self.agents.iter_mut().find(|agent| &agent.id == agent_id) else {
            return Err(format!("unknown agent `{agent_id}`"));
        };
        if ready && !agent.ready {
            self.pending_bell.set(true);
        }
        agent.ready = ready;
        Ok(())
    }

    pub fn select_agent(&mut self, agent_id: &AgentId) -> Result<(), String> {
        if self.agents.iter().any(|agent| &agent.id == agent_id) {
            self.selected_agent = Some(agent_id.clone());
            self.windows[self.active_window] = UiWindow::chat(agent_id.clone());
            self.control_selected = self
                .visible_agent_indices()
                .iter()
                .position(|index| self.agents[*index].id == *agent_id)
                .map(|position| position + 1)
                .unwrap_or(self.control_selected);
            self.reset_right_scroll();
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
        self.reset_right_scroll();
    }

    pub fn handle_key(&mut self, key: UiKey) -> Vec<UiAction> {
        let command_mode_text_key = self.command_mode_text_key_control_status(key);
        let actions = self.handle_key_inner(key);
        self.update_command_mode_typing_notice(command_mode_text_key);
        actions
    }

    fn handle_key_inner(&mut self, key: UiKey) -> Vec<UiAction> {
        match key {
            UiKey::MouseClick { column, row } => {
                self.pending = None;
                return self.handle_mouse_click(column, row);
            }
            UiKey::MouseScrollUp { column, row } => {
                self.pending = None;
                self.handle_mouse_scroll(column, row, true);
                return Vec::new();
            }
            UiKey::MouseScrollDown { column, row } => {
                self.pending = None;
                self.handle_mouse_scroll(column, row, false);
                return Vec::new();
            }
            _ => {}
        }

        if self.mode.is_visual() {
            return self.handle_visual_key(key);
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
            UiKey::CtrlV if self.mode == UiMode::Command => {
                self.start_visual_selection(VisualKind::Block);
                Vec::new()
            }
            UiKey::Char('g') if self.mode == UiMode::Command => {
                self.pending = Some(PendingKey::G);
                Vec::new()
            }
            UiKey::Char('v') if self.mode == UiMode::Command => {
                self.start_visual_selection(VisualKind::Character);
                Vec::new()
            }
            UiKey::Char('V') if self.mode == UiMode::Command => {
                self.start_visual_selection(VisualKind::Line);
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
                self.left_visible = !self.left_visible;
                self.focus = if self.left_visible {
                    PaneFocus::Left
                } else {
                    PaneFocus::Right
                };
                Vec::new()
            }
            UiKey::Char('j') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Down if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Right if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Char('k') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(-1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Up if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(-1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Left if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(-1);
                self.select_control_row_surface();
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
            let mut row = String::new();
            row.push(if self.control_selected == visible_position + 1 {
                '>'
            } else {
                ' '
            });
            let (primary, secondary) = agent_list_labels(agent);
            row.push_str(primary);
            row.push(' ');
            row.push_str(secondary);
            row.push_str("  working: ");
            row.push_str(&agent.feature);
            if agent.ready {
                row.push_str("  READY");
                rendered.push_str("\u{1b}[7m");
                rendered.push_str(&row);
                rendered.push_str("\u{1b}[0m");
            } else {
                rendered.push_str(&row);
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
        let visible_right_content = self.visible_right_content(right_content);
        self.remember_right_render_state(&visible_right_content, None);
        let prompt_cursor = prompt.len();
        let buffer = self.render_tui_buffer(&visible_right_content, prompt, prompt_cursor);
        let mut rendered = String::new();
        rendered.push_str(&self.terminal_prefix());
        rendered.push_str("\u{1b}[H");
        rendered.push_str(&buffer_to_string(&buffer));
        rendered.push_str(&self.cursor_sequence(&visible_right_content, prompt));
        rendered
    }

    pub fn render_screen_with_cursors(
        &self,
        right_content: &str,
        prompt: &str,
        prompt_cursor: usize,
        right_cursor_column: Option<usize>,
    ) -> String {
        let visible_right_content = self.visible_right_content(right_content);
        self.remember_right_render_state(&visible_right_content, right_cursor_column);
        let buffer = self.render_tui_buffer(&visible_right_content, prompt, prompt_cursor);
        let mut rendered = String::new();
        rendered.push_str(&self.terminal_prefix());
        rendered.push_str("\u{1b}[H");
        rendered.push_str(&buffer_to_string(&buffer));
        rendered.push_str(&self.cursor_sequence_with_cursors(
            &visible_right_content,
            prompt,
            prompt_cursor,
            right_cursor_column,
        ));
        rendered
    }

    pub fn scroll_right_pane_up(&mut self) {
        self.right_scroll_rows = self.right_scroll_rows.saturating_add(3);
    }

    pub fn scroll_right_pane_down(&mut self) {
        self.right_scroll_rows = self.right_scroll_rows.saturating_sub(3);
    }

    pub fn reset_right_scroll(&mut self) {
        self.right_scroll_rows = 0;
    }

    fn visible_right_content(&self, right_content: &str) -> String {
        let (inner_width, inner_height) = self.right_inner_size();
        visible_content(
            right_content,
            inner_width,
            inner_height,
            self.right_scroll_rows,
        )
    }

    fn render_tui_buffer(&self, right_content: &str, prompt: &str, prompt_cursor: usize) -> Buffer {
        let backend = TestBackend::new(self.width, self.height);
        let mut terminal = Terminal::new(backend).expect("test backend is valid");
        terminal
            .draw(|frame| {
                let area = frame.size();
                let body_height = area.height.saturating_sub(1);
                let body = Rect::new(area.x, area.y, area.width, body_height);
                let bottom = Rect::new(area.x, body_height, area.width, 1);
                let layout = self.layout();
                let panes = if layout.left_width > 0 {
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

                if layout.left_width > 0 {
                    frame.render_widget(self.left_widget(), panes[0]);
                    frame.render_widget(self.right_widget(right_content), panes[1]);
                } else {
                    frame.render_widget(self.right_widget(right_content), panes[0]);
                }
                frame.render_widget(
                    Paragraph::new(self.bottom_line(prompt, prompt_cursor)),
                    bottom,
                );
            })
            .expect("test backend draw succeeds");
        terminal.backend().buffer().clone()
    }

    fn left_widget(&self) -> List<'static> {
        let items = self
            .left_pane_lines()
            .into_iter()
            .enumerate()
            .map(|(row, line)| {
                let item = ListItem::new(self.styled_pane_line(PaneFocus::Left, row, &line.text));
                if line.reversed {
                    item.style(Style::default().add_modifier(Modifier::REVERSED))
                } else {
                    item
                }
            })
            .collect::<Vec<_>>();
        List::new(items).block(Block::default().title("work-leaf").borders(Borders::ALL))
    }

    fn right_widget(&self, right_content: &str) -> Paragraph<'static> {
        let title = match self.windows[self.active_window].surface {
            UiSurface::WorkLeafCommand => "command",
            UiSurface::AgentChat => "chat",
        };
        Paragraph::new(self.right_text(right_content))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: false })
    }

    fn bottom_line(&self, prompt: &str, prompt_cursor: usize) -> String {
        if self.mode == UiMode::Prompt {
            self.prompt_view(prompt, prompt_cursor).line
        } else {
            self.render_status_line()
        }
    }

    fn cursor_sequence(&self, right_content: &str, prompt: &str) -> String {
        self.cursor_sequence_with_cursors(right_content, prompt, prompt.len(), None)
    }

    fn cursor_sequence_with_cursors(
        &self,
        right_content: &str,
        prompt: &str,
        prompt_cursor: usize,
        right_cursor_column: Option<usize>,
    ) -> String {
        let (row, column) = if self.mode == UiMode::Prompt {
            (
                self.height,
                self.prompt_view(prompt, prompt_cursor).cursor_column,
            )
        } else {
            match self.focus {
                PaneFocus::Left => (self.control_cursor_row(), 2),
                PaneFocus::Right => {
                    self.right_cursor_position_with_cursor(right_content, right_cursor_column)
                }
            }
        };
        let row = row.clamp(1, self.height.max(1));
        let column = column.clamp(1, self.width.max(1));
        format!("\u{1b}[{row};{column}H")
    }

    fn prompt_view(&self, prompt: &str, prompt_cursor: usize) -> PromptView {
        let width = usize::from(self.width.max(1));
        let input_width = width.saturating_sub(1);
        let max_cursor_offset = width.saturating_sub(2);
        let cursor_chars = cursor_char_count(prompt, prompt_cursor);
        let start = cursor_chars.saturating_sub(max_cursor_offset);
        let visible_prompt = prompt
            .chars()
            .skip(start)
            .take(input_width)
            .collect::<String>();
        let cursor_offset = cursor_chars.saturating_sub(start).min(max_cursor_offset);
        let cursor_column = if self.width <= 1 {
            1
        } else {
            cursor_offset.saturating_add(2).min(width) as u16
        };
        PromptView {
            line: format!(":{visible_prompt}"),
            cursor_column,
        }
    }

    fn bell_prefix(&self) -> &'static str {
        if self.pending_bell.replace(false) {
            "\u{7}"
        } else {
            ""
        }
    }

    fn terminal_prefix(&self) -> String {
        let mut prefix = String::from(self.bell_prefix());
        if let Some(text) = self.pending_clipboard.borrow_mut().take() {
            prefix.push_str(&osc52_copy_sequence(&text));
        }
        prefix
    }

    fn remember_right_render_state(&self, right_content: &str, right_cursor_column: Option<usize>) {
        self.last_right_content.replace(right_content.to_string());
        self.last_right_cursor_column.set(right_cursor_column);
    }

    fn handle_pending_key(&mut self, pending: PendingKey, key: UiKey) -> Vec<UiAction> {
        match (pending, key) {
            (PendingKey::CtrlW, UiKey::Char('h')) if self.mode == UiMode::Command => {
                if self.left_visible {
                    self.focus = PaneFocus::Left;
                }
                Vec::new()
            }
            (PendingKey::CtrlW, UiKey::Char('k')) if self.mode == UiMode::Command => {
                if self.left_visible {
                    self.focus = PaneFocus::Left;
                }
                Vec::new()
            }
            (PendingKey::CtrlW, UiKey::Char('l')) if self.mode == UiMode::Command => {
                self.focus = PaneFocus::Right;
                self.mode = UiMode::Command;
                Vec::new()
            }
            (PendingKey::CtrlW, UiKey::Char('j')) if self.mode == UiMode::Command => {
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

    fn command_mode_text_key_control_status(&self, key: UiKey) -> Option<bool> {
        if self.mode != UiMode::Command || self.pending.is_some() {
            return None;
        }
        let UiKey::Char(ch) = key else {
            return None;
        };
        (ch.is_ascii_alphanumeric() || ch == ' ').then(|| self.is_command_control_char(ch))
    }

    fn is_command_control_char(&self, ch: char) -> bool {
        matches!(ch, 'i' | ':' | ',' | 's' | 't' | 'f' | 'g' | 'v' | 'V')
            || (self.focus == PaneFocus::Left && matches!(ch, 'j' | 'k' | 'l' | 'x'))
    }

    fn update_command_mode_typing_notice(&mut self, command_mode_text_key_control: Option<bool>) {
        if let Some(command_mode_text_key_control) = command_mode_text_key_control
            && self.mode == UiMode::Command
            && self.pending.is_none()
        {
            self.command_mode_typing_count = self.command_mode_typing_count.saturating_add(1);
            self.command_mode_typing_controls_only &= command_mode_text_key_control;
            if self.command_mode_typing_count >= COMMAND_MODE_TYPING_NOTICE_THRESHOLD {
                if !self.command_mode_typing_controls_only {
                    self.show_status_notice(
                        COMMAND_MODE_TYPING_NOTICE,
                        Duration::from_secs(STATUS_NOTICE_SECONDS),
                    );
                }
                self.command_mode_typing_count = 0;
                self.command_mode_typing_controls_only = true;
            }
        } else {
            self.command_mode_typing_count = 0;
            self.command_mode_typing_controls_only = true;
        }
    }

    fn show_status_notice(&mut self, message: impl Into<String>, duration: Duration) {
        self.status_notice = Some(StatusNotice {
            message: message.into(),
            expires_at: Instant::now() + duration,
        });
    }

    fn active_status_notice(&self) -> Option<&str> {
        self.status_notice
            .as_ref()
            .filter(|notice| Instant::now() < notice.expires_at)
            .map(|notice| notice.message.as_str())
    }

    fn status_notice_expired(&self) -> bool {
        self.status_notice
            .as_ref()
            .is_some_and(|notice| Instant::now() >= notice.expires_at)
    }

    fn open_selected_same_pane(&mut self) -> Vec<UiAction> {
        let Some(agent_id) = self.action_agent_id() else {
            return Vec::new();
        };
        self.split_chats.push(agent_id.clone());
        vec![UiAction::OpenChatSamePane(agent_id)]
    }

    fn handle_visual_key(&mut self, key: UiKey) -> Vec<UiAction> {
        match key {
            UiKey::Esc => {
                self.clear_visual_selection();
                Vec::new()
            }
            UiKey::Char('v') => {
                if self.mode == UiMode::Visual {
                    self.clear_visual_selection();
                } else {
                    self.set_visual_kind(VisualKind::Character);
                }
                Vec::new()
            }
            UiKey::Char('V') => {
                self.set_visual_kind(VisualKind::Line);
                Vec::new()
            }
            UiKey::CtrlV => {
                self.set_visual_kind(VisualKind::Block);
                Vec::new()
            }
            UiKey::Char('h') | UiKey::Left => {
                self.move_visual_cursor(0, -1);
                Vec::new()
            }
            UiKey::Char('j') | UiKey::Down => {
                self.move_visual_cursor(1, 0);
                Vec::new()
            }
            UiKey::Char('k') | UiKey::Up => {
                self.move_visual_cursor(-1, 0);
                Vec::new()
            }
            UiKey::Char('l') | UiKey::Right => {
                self.move_visual_cursor(0, 1);
                Vec::new()
            }
            UiKey::Char('y') => {
                self.yank_visual_selection(false);
                Vec::new()
            }
            UiKey::Char('Y') => {
                self.yank_visual_selection(true);
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn start_visual_selection(&mut self, kind: VisualKind) {
        let pane = if self.focus == PaneFocus::Left && self.left_visible {
            PaneFocus::Left
        } else {
            PaneFocus::Right
        };
        let position = self.current_visual_position(pane);
        self.mode = kind.mode();
        self.visual_selection = Some(VisualSelection {
            pane,
            kind,
            anchor: position,
            cursor: position,
        });
    }

    fn set_visual_kind(&mut self, kind: VisualKind) {
        if let Some(selection) = self.visual_selection.as_mut() {
            selection.kind = kind;
            self.mode = kind.mode();
        } else {
            self.start_visual_selection(kind);
        }
    }

    fn clear_visual_selection(&mut self) {
        self.visual_selection = None;
        self.mode = UiMode::Command;
    }

    fn move_visual_cursor(&mut self, row_delta: isize, column_delta: isize) {
        let Some(pane) = self.visual_selection.as_ref().map(|selection| selection.pane) else {
            return;
        };
        let lines = self.pane_text_lines(pane);
        let Some(selection) = self.visual_selection.as_mut() else {
            return;
        };
        if lines.is_empty() {
            selection.cursor = VisualPosition { row: 0, column: 0 };
            return;
        }
        let row = offset_index(selection.cursor.row, row_delta, lines.len() - 1);
        let max_column = line_char_len(&lines[row]).saturating_sub(1);
        let column = offset_index(selection.cursor.column, column_delta, max_column);
        selection.cursor = VisualPosition { row, column };
    }

    fn yank_visual_selection(&mut self, linewise: bool) {
        let Some(mut selection) = self.visual_selection.clone() else {
            return;
        };
        if linewise {
            selection.kind = VisualKind::Line;
        }
        let text = selected_text(&self.pane_text_lines(selection.pane), &selection);
        self.clipboard = text.clone();
        self.pending_clipboard.replace(Some(text.clone()));
        self.show_status_notice(
            format!("copied {} chars", text.chars().count()),
            Duration::from_secs(STATUS_NOTICE_SECONDS),
        );
        self.clear_visual_selection();
    }

    fn current_visual_position(&self, pane: PaneFocus) -> VisualPosition {
        let lines = self.pane_text_lines(pane);
        if lines.is_empty() {
            return VisualPosition { row: 0, column: 0 };
        }

        match pane {
            PaneFocus::Left => {
                let row = self.control_selected.min(lines.len() - 1);
                VisualPosition { row, column: 0 }
            }
            PaneFocus::Right => {
                let row = lines.len() - 1;
                let max_column = line_char_len(&lines[row]).saturating_sub(1);
                let column = self
                    .last_right_cursor_column
                    .get()
                    .unwrap_or(max_column)
                    .min(max_column);
                VisualPosition { row, column }
            }
        }
    }

    fn pane_text_lines(&self, pane: PaneFocus) -> Vec<String> {
        match pane {
            PaneFocus::Left => self
                .left_pane_lines()
                .into_iter()
                .map(|line| line.text)
                .collect(),
            PaneFocus::Right => {
                let content = self.last_right_content.borrow();
                pane_text_lines(content.as_str())
            }
        }
    }

    fn left_pane_lines(&self) -> Vec<PaneLine> {
        let inner_width = usize::from(self.layout().left_width.saturating_sub(2).max(1));
        let mut lines = vec![PaneLine {
            text: if self.control_selected == 0 {
                "> work-leaf  command".to_string()
            } else {
                "  work-leaf  command".to_string()
            },
            reversed: false,
        }];
        for (visible_position, agent_index) in self.visible_agent_indices().iter().enumerate() {
            let agent = &self.agents[*agent_index];
            let selected = self.control_selected == visible_position + 1;
            lines.push(PaneLine {
                text: compact_agent_row(agent, selected, inner_width),
                reversed: agent.ready,
            });
            if !agent.modified_files.is_empty() {
                lines.push(PaneLine {
                    text: format!(
                        "    files: {}",
                        agent
                            .modified_files
                            .iter()
                            .map(|path| path.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    reversed: false,
                });
            }
            for (label, agents) in [
                ("conflicts", &agent.conflicting_agents),
                ("depends-on", &agent.depends_on),
                ("depended-on-by", &agent.depended_on_by),
            ] {
                if !agents.is_empty() {
                    lines.push(PaneLine {
                        text: format!(
                            "    {label}: {}",
                            agents
                                .iter()
                                .map(AgentId::as_str)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        reversed: false,
                    });
                }
            }
        }
        lines
    }

    fn right_text(&self, right_content: &str) -> Vec<Spans<'static>> {
        pane_text_lines(right_content)
            .into_iter()
            .enumerate()
            .map(|(row, line)| self.styled_pane_line(PaneFocus::Right, row, &line))
            .collect()
    }

    fn styled_pane_line(&self, pane: PaneFocus, row: usize, line: &str) -> Spans<'static> {
        let line_len = line_char_len(line);
        if let Some((start, end)) = self.visual_selection_range(pane, row, line_len) {
            styled_line_spans(line, start, end)
        } else {
            Spans::from(vec![Span::raw(line.to_string())])
        }
    }

    fn visual_selection_range(
        &self,
        pane: PaneFocus,
        row: usize,
        line_len: usize,
    ) -> Option<(usize, usize)> {
        let selection = self.visual_selection.as_ref()?;
        if selection.pane != pane {
            return None;
        }
        selected_columns(selection, row, line_len)
    }

    fn open_selected_new_window(&mut self) -> Vec<UiAction> {
        let Some(agent_id) = self.action_agent_id() else {
            return Vec::new();
        };
        self.windows.push(UiWindow::chat(agent_id.clone()));
        self.active_window = self.windows.len() - 1;
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
            self.focus = PaneFocus::Right;
            return;
        }
        if let Some(agent_id) = self.control_selected_agent_id() {
            let _ = self.select_agent(&agent_id);
            self.focus = PaneFocus::Right;
        }
    }

    fn select_control_row_surface(&mut self) {
        if self.control_selected == 0 {
            self.select_command_interface();
            self.focus = PaneFocus::Left;
            return;
        }
        if let Some(agent_id) = self.control_selected_agent_id() {
            let _ = self.select_agent(&agent_id);
            self.focus = PaneFocus::Left;
        }
    }

    fn handle_mouse_click(&mut self, column: u16, row: u16) -> Vec<UiAction> {
        if column == 0 || row == 0 {
            return Vec::new();
        }

        let left_width = self.layout().left_width;

        if self.left_visible && column <= left_width {
            self.mode = UiMode::Command;
            self.focus = PaneFocus::Left;
            let Some(target) = self.left_pane_click_target(row) else {
                return Vec::new();
            };
            match target {
                LeftPaneClickTarget::Command => {
                    self.select_command_interface();
                    self.focus = PaneFocus::Right;
                }
                LeftPaneClickTarget::Agent(agent_id) => {
                    let _ = self.activate_agent_chat(&agent_id);
                }
            }
        } else {
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

    fn handle_mouse_scroll(&mut self, column: u16, row: u16, up: bool) {
        if column == 0 || row == 0 || row >= self.height {
            return;
        }

        let left_width = self.layout().left_width;
        if self.left_visible && column <= left_width {
            return;
        }

        if up {
            self.scroll_right_pane_up();
        } else {
            self.scroll_right_pane_down();
        }
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
        let hidden_was_selected = self
            .selected_agent
            .as_ref()
            .is_some_and(|selected| selected == &hidden_agent);
        self.clamp_control_selection();
        if hidden_was_selected {
            self.select_control_row_surface();
        }
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

    fn right_cursor_position_with_cursor(
        &self,
        right_content: &str,
        cursor_column: Option<usize>,
    ) -> (u16, u16) {
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
        let line_chars = line.chars().count();
        let line_len = cursor_column
            .unwrap_or(line_chars)
            .min(line_chars)
            .min(usize::from(u16::MAX)) as u16;
        let row = 2_u16
            .saturating_add(previous_rows)
            .saturating_add(line_len / inner_width);
        let column = layout
            .left_width
            .saturating_add(2)
            .saturating_add(line_len % inner_width);
        (row, column)
    }
    fn right_inner_size(&self) -> (u16, u16) {
        let layout = self.layout();
        let inner_width = layout.right_width.saturating_sub(2).max(1);
        let body_height = self.height.saturating_sub(1);
        let inner_height = body_height.saturating_sub(2).max(1);
        (inner_width, inner_height)
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
        if let Some(notice) = self.active_status_notice() {
            return notice.to_string();
        }

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
    pub(crate) fn is_visual(&self) -> bool {
        matches!(self, Self::Visual | Self::VisualLine | Self::VisualBlock)
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Insert => "insert",
            Self::Prompt => "prompt",
            Self::Visual => "visual",
            Self::VisualLine => "visual-line",
            Self::VisualBlock => "visual-block",
        }
    }
}

impl VisualKind {
    fn mode(&self) -> UiMode {
        match self {
            Self::Character => UiMode::Visual,
            Self::Line => UiMode::VisualLine,
            Self::Block => UiMode::VisualBlock,
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

fn agent_list_labels(agent: &AgentListEntry) -> (&str, &str) {
    if agent.id.as_str().starts_with("review-") {
        (agent.id.as_str(), &agent.feature)
    } else {
        (&agent.feature, agent.id.as_str())
    }
}

fn compact_agent_row(agent: &AgentListEntry, selected: bool, width: usize) -> String {
    let prefix = if selected { ">" } else { " " };
    let status = if agent.ready { " READY" } else { "" };
    let id = agent.id.as_str();
    let width = width.max(1);

    let row = if id.starts_with("review-") {
        compact_fixed_first(prefix, id, &agent.feature, status, width)
    } else {
        compact_fixed_last(prefix, &agent.feature, id, status, width)
    };
    truncate_to_width(&row, width)
}

fn compact_fixed_first(
    prefix: &str,
    fixed: &str,
    flexible: &str,
    status: &str,
    width: usize,
) -> String {
    let fixed_width = prefix.chars().count() + fixed.chars().count() + status.chars().count();
    if fixed_width >= width {
        return format!("{prefix}{fixed}{status}");
    }

    let flexible_width = width.saturating_sub(fixed_width + 1);
    format!(
        "{prefix}{fixed} {}{status}",
        truncate_to_width(flexible, flexible_width)
    )
}

fn compact_fixed_last(
    prefix: &str,
    flexible: &str,
    fixed: &str,
    status: &str,
    width: usize,
) -> String {
    let fixed_width = prefix.chars().count() + 1 + fixed.chars().count() + status.chars().count();
    if fixed_width >= width {
        return format!("{prefix}{fixed}{status}");
    }

    let flexible_width = width.saturating_sub(fixed_width);
    format!(
        "{prefix}{} {fixed}{status}",
        truncate_to_width(flexible, flexible_width)
    )
}

fn truncate_to_width(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn pane_text_lines(content: &str) -> Vec<String> {
    content.split('\n').map(ToString::to_string).collect()
}

fn styled_line_spans(line: &str, start: usize, end: usize) -> Spans<'static> {
    if start >= end {
        return Spans::from(vec![Span::raw(line.to_string())]);
    }

    let before = line.chars().take(start).collect::<String>();
    let selected = line
        .chars()
        .skip(start)
        .take(end - start)
        .collect::<String>();
    let after = line.chars().skip(end).collect::<String>();
    let mut spans = Vec::new();
    if !before.is_empty() {
        spans.push(Span::raw(before));
    }
    spans.push(Span::styled(
        selected,
        Style::default().add_modifier(Modifier::REVERSED),
    ));
    if !after.is_empty() {
        spans.push(Span::raw(after));
    }
    Spans::from(spans)
}

fn selected_text(lines: &[String], selection: &VisualSelection) -> String {
    let mut selected = Vec::new();
    for row in selected_row_range(selection) {
        let Some(line) = lines.get(row) else {
            continue;
        };
        if let Some((start, end)) = selected_columns(selection, row, line_char_len(line)) {
            selected.push(line.chars().skip(start).take(end - start).collect::<String>());
        }
    }
    selected.join("\n")
}

fn selected_row_range(selection: &VisualSelection) -> std::ops::RangeInclusive<usize> {
    selection.anchor.row.min(selection.cursor.row)..=selection.anchor.row.max(selection.cursor.row)
}

fn selected_columns(
    selection: &VisualSelection,
    row: usize,
    line_len: usize,
) -> Option<(usize, usize)> {
    if !selected_row_range(selection).contains(&row) {
        return None;
    }

    let range = match selection.kind {
        VisualKind::Line => (0, line_len),
        VisualKind::Block => {
            let start = selection.anchor.column.min(selection.cursor.column).min(line_len);
            let end = selection
                .anchor
                .column
                .max(selection.cursor.column)
                .saturating_add(1)
                .min(line_len);
            (start, end)
        }
        VisualKind::Character => character_selected_columns(selection, row, line_len),
    };

    Some(range).filter(|(start, end)| start <= end)
}

fn character_selected_columns(
    selection: &VisualSelection,
    row: usize,
    line_len: usize,
) -> (usize, usize) {
    let start_row = selection.anchor.row.min(selection.cursor.row);
    let end_row = selection.anchor.row.max(selection.cursor.row);
    let start_column = if selection.anchor.row <= selection.cursor.row {
        selection.anchor.column
    } else {
        selection.cursor.column
    };
    let end_column = if selection.anchor.row <= selection.cursor.row {
        selection.cursor.column
    } else {
        selection.anchor.column
    };

    if start_row == end_row {
        return (
            start_column.min(line_len),
            end_column.saturating_add(1).min(line_len),
        );
    }
    if row == start_row {
        (start_column.min(line_len), line_len)
    } else if row == end_row {
        (0, end_column.saturating_add(1).min(line_len))
    } else {
        (0, line_len)
    }
}

fn offset_index(index: usize, delta: isize, max: usize) -> usize {
    if delta < 0 {
        index.saturating_sub(delta.unsigned_abs()).min(max)
    } else {
        index.saturating_add(delta as usize).min(max)
    }
}

fn line_char_len(line: &str) -> usize {
    line.chars().count()
}

fn osc52_copy_sequence(text: &str) -> String {
    format!("\u{1b}]52;c;{}\u{7}", base64_encode(text.as_bytes()))
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        let value = ((first as u32) << 16) | ((second as u32) << 8) | third as u32;
        encoded.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        encoded.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(value & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

fn buffer_to_string(buffer: &Buffer) -> String {
    const ANSI_REVERSE_VIDEO: &str = "\u{1b}[7m";
    const ANSI_RESET: &str = "\u{1b}[0m";
    let mut output = String::new();
    for y in 0..buffer.area.height {
        let mut reversed = false;
        for x in 0..buffer.area.width {
            let cell = buffer.get(x, y);
            let cell_reversed = cell.modifier.contains(Modifier::REVERSED);
            if cell_reversed != reversed {
                output.push_str(if cell_reversed {
                    ANSI_REVERSE_VIDEO
                } else {
                    ANSI_RESET
                });
                reversed = cell_reversed;
            }
            output.push_str(&cell.symbol);
        }
        if reversed {
            output.push_str(ANSI_RESET);
        }
        if y + 1 < buffer.area.height {
            output.push_str("\r\n");
        }
    }
    output
}

fn visual_rows(line: &str, width: u16) -> u16 {
    visual_row_count(line, width).min(usize::from(u16::MAX)) as u16
}

fn visual_row_count(line: &str, width: u16) -> usize {
    let width = usize::from(width.max(1));
    let len = line.chars().count().min(usize::from(u16::MAX));
    (len / width).saturating_add(1)
}

fn cursor_char_count(text: &str, cursor: usize) -> usize {
    text.char_indices()
        .take_while(|(index, _)| *index < cursor)
        .count()
}

fn visible_content(content: &str, width: u16, height: u16, scroll_rows: usize) -> String {
    let height = usize::from(height);
    let Some((history, prompt)) = split_chat_prompt(content) else {
        return tail_visible_content(content, width, height, scroll_rows);
    };

    let prompt_rows = visual_row_count(prompt, width);
    let history_height = height.saturating_sub(prompt_rows).max(1);
    let visible_history = tail_visible_content(history, width, history_height, scroll_rows);
    if visible_history.is_empty() {
        prompt.to_string()
    } else {
        format!("{visible_history}\n{prompt}")
    }
}

fn split_chat_prompt(content: &str) -> Option<(&str, &str)> {
    let (history, prompt) = content.rsplit_once('\n')?;
    prompt.starts_with("chat> ").then_some((history, prompt))
}

fn tail_visible_content(content: &str, width: u16, height: usize, scroll_rows: usize) -> String {
    if content.is_empty() || height == 0 {
        return String::new();
    }

    let lines = content.lines().collect::<Vec<_>>();
    let rows_to_skip = scroll_rows.min(
        lines
            .iter()
            .map(|line| visual_row_count(line, width))
            .sum::<usize>()
            .saturating_sub(height),
    );
    let mut visible = Vec::new();
    let mut skipped_rows = 0_usize;
    let mut used_rows = 0_usize;
    for line in lines.iter().rev() {
        let rows = visual_row_count(line, width);
        if skipped_rows.saturating_add(rows) <= rows_to_skip {
            skipped_rows = skipped_rows.saturating_add(rows);
            continue;
        }
        if visible.is_empty() || used_rows.saturating_add(rows) <= height {
            visible.push(*line);
            used_rows = used_rows.saturating_add(rows);
        } else {
            break;
        }
    }
    visible.reverse();
    visible.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_agent_ready_queues_one_bell_for_next_render() {
        let mut ui = TerminalUi::new(80, 24);
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        ui.add_agent(AgentListEntry::new(agent_id.clone(), "parser"));

        ui.set_agent_ready_state(&agent_id, true)
            .expect("test agent is registered");

        assert!(ui.render_screen("reply").starts_with('\u{7}'));
        assert!(!ui.render_screen("reply").contains('\u{7}'));
    }

    #[test]
    fn ready_agent_row_is_reversed_across_the_tui_left_pane() {
        let mut ui = TerminalUi::new(100, 24);
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        ui.add_agent(AgentListEntry::new(agent_id, "parser").with_ready(true));

        let buffer = ui.render_tui_buffer("reply", "", 0);
        let left_width = ui.layout().left_width;

        for column in 1..left_width.saturating_sub(1) {
            assert!(
                buffer.get(column, 2).modifier.contains(Modifier::REVERSED),
                "column {column} on the ready agent row should be reversed"
            );
        }
    }
}
----- END FILE src/ui.rs -----

----- BEGIN FILE tests/terminal_ui.rs -----
digest: fnv64:4ed464b0f7a89909; bytes:11332

use work_leaf::{
    AgentId, AgentListEntry, PaneFocus, TerminalUi, UiAction, UiKey, UiMode, UiSurface,
};

#[test]
fn terminal_layout_reserves_left_fifth_for_agents() {
    let ui = TerminalUi::new(100, 40);
    let layout = ui.layout();

    assert_eq!(layout.left_width, 20);
    assert_eq!(layout.right_width, 80);
    assert_eq!(layout.height, 40);
    assert_eq!(layout.right_surface, Some(UiSurface::WorkLeafCommand));
}

#[test]
fn vim_style_keys_drive_mode_focus_visibility_and_tabs() {
    let mut ui = TerminalUi::new(100, 40);
    let agent_id = AgentId::new("chat-nav").unwrap();
    ui.add_agent(AgentListEntry::new(agent_id.clone(), "navigation"));
    ui.select_agent(&agent_id).unwrap();

    ui.handle_key(UiKey::Char('i'));
    assert_eq!(ui.mode(), UiMode::Insert);

    ui.handle_key(UiKey::Esc);
    ui.handle_key(UiKey::Char(','));
    assert_eq!(ui.layout().left_width, 0);
    assert_eq!(ui.layout().right_width, 100);
    assert_eq!(ui.layout().right_surface, Some(UiSurface::AgentChat));

    ui.handle_key(UiKey::Char(','));
    assert_eq!(ui.layout().left_width, 20);
    ui.handle_key(UiKey::CtrlW);
    ui.handle_key(UiKey::Char('l'));
    assert_eq!(ui.focus(), PaneFocus::Right);
    assert_eq!(ui.mode(), UiMode::Command);

    ui.handle_key(UiKey::Char('i'));
    assert_eq!(ui.mode(), UiMode::Insert);
    ui.handle_key(UiKey::Esc);
    assert_eq!(ui.mode(), UiMode::Command);

    ui.handle_key(UiKey::CtrlW);
    ui.handle_key(UiKey::Char('h'));
    assert_eq!(ui.focus(), PaneFocus::Left);

    ui.handle_key(UiKey::CtrlW);
    ui.handle_key(UiKey::Char('j'));
    assert_eq!(ui.focus(), PaneFocus::Right);

    ui.handle_key(UiKey::CtrlW);
    ui.handle_key(UiKey::Char('k'));
    assert_eq!(ui.focus(), PaneFocus::Left);

    ui.handle_key(UiKey::Char('t'));
    assert_eq!(ui.window_count(), 2);
    assert_eq!(ui.active_window(), 1);

    ui.handle_key(UiKey::Char('g'));
    ui.handle_key(UiKey::Char('T'));
    assert_eq!(ui.active_window(), 0);
}

#[test]
fn comma_does_not_hide_the_right_chat_while_chat_focus_is_active() {
    let mut ui = TerminalUi::new(100, 40);
    let agent_id = AgentId::new("chat-comma").unwrap();
    ui.add_agent(AgentListEntry::new(agent_id.clone(), "chat"));
    ui.activate_agent_chat(&agent_id).unwrap();

    ui.handle_key(UiKey::Esc);
    ui.handle_key(UiKey::Char(','));

    assert_eq!(ui.focus(), PaneFocus::Right);
    assert_eq!(ui.layout().left_width, 0);
    assert_eq!(ui.layout().right_width, 100);
    assert_eq!(ui.layout().right_surface, Some(UiSurface::AgentChat));

    ui.handle_key(UiKey::CtrlW);
    ui.handle_key(UiKey::Char('h'));

    assert_eq!(ui.focus(), PaneFocus::Right);

    ui.handle_key(UiKey::Char(','));

    assert_eq!(ui.focus(), PaneFocus::Left);
    assert_eq!(ui.layout().left_width, 20);
    assert_eq!(ui.layout().right_width, 80);
    assert_eq!(ui.layout().right_surface, Some(UiSurface::AgentChat));
}

#[test]
fn colon_enters_command_prompt_only_from_command_mode() {
    let mut ui = TerminalUi::new(100, 40);

    ui.handle_key(UiKey::Char(':'));
    assert_eq!(ui.mode(), UiMode::Prompt);

    ui.handle_key(UiKey::Esc);
    assert_eq!(ui.mode(), UiMode::Command);

    ui.handle_key(UiKey::Char('i'));
    assert_eq!(ui.mode(), UiMode::Insert);
    ui.handle_key(UiKey::Char(':'));
    assert_eq!(ui.mode(), UiMode::Insert);
}

#[test]
fn agent_list_actions_expose_split_window_fork_and_ready_highlight() {
    let mut ui = TerminalUi::new(120, 30);
    let agent_id = AgentId::new("chat-1").unwrap();
    ui.add_agent(
        AgentListEntry::new(agent_id.clone(), "parser")
            .with_ready(true)
            .with_modified_file("src/parser.rs"),
    );
    ui.select_agent(&agent_id).unwrap();

    assert_eq!(
        ui.handle_key(UiKey::Char('s')),
        vec![UiAction::OpenChatSamePane(agent_id.clone())]
    );
    assert_eq!(
        ui.handle_key(UiKey::Char('t')),
        vec![UiAction::OpenChatNewWindow(agent_id.clone())]
    );
    assert_eq!(
        ui.handle_key(UiKey::Char('f')),
        vec![UiAction::ForkAgent(agent_id.clone())]
    );

    let rendered = ui.render_left_pane();
    assert!(rendered.contains("chat-1"));
    assert!(rendered.contains("parser"));
    assert!(rendered.contains("READY"));
    assert!(rendered.contains("src/parser.rs"));
}

#[test]
fn left_pane_includes_command_interface_and_agent_introspection() {
    let mut ui = TerminalUi::new(120, 30);
    let chat_a = AgentId::new("chat-a").unwrap();
    let chat_b = AgentId::new("chat-b").unwrap();
    ui.add_agent(
        AgentListEntry::new(chat_a.clone(), "parser")
            .with_ready(true)
            .with_modified_file("src/parser.rs")
            .with_conflicting_agent(chat_b.clone())
            .with_dependency(chat_b.clone()),
    );
    ui.add_agent(AgentListEntry::new(chat_b.clone(), "docs").with_dependent(chat_a.clone()));

    let rendered = ui.render_left_pane();

    assert!(rendered.contains("work-leaf"));
    assert!(rendered.contains("chat-a"));
    assert!(rendered.contains("working: parser"));
    assert!(rendered.contains("files: src/parser.rs"));
    assert!(rendered.contains("conflicts: chat-b"));
    assert!(rendered.contains("depends-on: chat-b"));
    assert!(rendered.contains("depended-on-by: chat-a"));
    assert!(rendered.contains("\u{1b}[7m parser chat-a  working: parser  READY\u{1b}[0m"));
}

#[test]
fn screen_renderer_draws_left_fifth_right_pane_and_status_line() {
    let mut ui = TerminalUi::new(60, 12);
    let agent_id = AgentId::new("chat-a").unwrap();
    ui.add_agent(AgentListEntry::new(agent_id.clone(), "parser").with_ready(true));

    let rendered = ui.render_screen("new chat-a parser implement parser");

    assert!(rendered.starts_with("\u{1b}[H"));
    assert!(!rendered.contains("\u{1b}[2J"));
    assert!(rendered.contains("work-leaf"));
    assert!(rendered.contains("chat-a"));
    assert!(rendered.contains("command"));
    assert!(rendered.contains("new chat-a parser implement parser"));
    assert!(rendered.contains("mode=command"));
    assert!(rendered.contains("focus=left"));
    assert!(rendered.lines().any(|line| line.contains('│')));
    assert_eq!(rendered.lines().count(), 12);
    assert!(
        rendered
            .lines()
            .take(11)
            .all(|line| strip_ansi(line).chars().count() == 60)
    );
}

#[test]
fn selecting_agent_changes_right_surface_to_agent_chat() {
    let mut ui = TerminalUi::new(80, 20);
    let agent_id = AgentId::new("chat-a").unwrap();
    ui.add_agent(AgentListEntry::new(agent_id.clone(), "parser"));

    ui.select_agent(&agent_id).unwrap();

    assert_eq!(ui.layout().right_surface, Some(UiSurface::AgentChat));
    assert!(ui.render_screen("agent reply").contains("agent reply"));
}

#[test]
fn activating_agent_chat_moves_cursor_to_right_insert_mode() {
    let mut ui = TerminalUi::new(80, 20);
    let agent_id = AgentId::new("chat-a").unwrap();
    ui.add_agent(AgentListEntry::new(agent_id.clone(), "parser"));

    ui.activate_agent_chat(&agent_id).unwrap();

    assert_eq!(ui.focus(), PaneFocus::Right);
    assert_eq!(ui.mode(), UiMode::Insert);
    assert_eq!(ui.selected_agent().map(AgentId::as_str), Some("chat-a"));
}

#[test]
fn prompt_mode_renders_colon_command_line_and_cursor_on_bottom_row() {
    let mut ui = TerminalUi::new(80, 10);
    ui.handle_key(UiKey::Char(':'));

    let rendered = ui.render_screen_with_prompt("chat", "new p");

    assert!(rendered.contains(":new p"));
    assert!(rendered.ends_with("\u{1b}[10;7H"));
}

#[test]
fn focus_cursor_stays_inside_left_or_right_pane() {
    let mut ui = TerminalUi::new(100, 20);
    let left = ui.render_screen("command chat");
    assert!(left.ends_with("\u{1b}[2;2H"));

    ui.handle_key(UiKey::CtrlW);
    ui.handle_key(UiKey::Char('l'));
    let right = ui.render_screen("command chat");
    assert!(right.ends_with("\u{1b}[2;22H"));
}

#[test]
fn left_pane_navigation_moves_cursor_to_selected_agent_row_and_opens_chat() {
    let mut ui = TerminalUi::new(100, 20);
    let chat_a = AgentId::new("chat-a").unwrap();
    let chat_b = AgentId::new("chat-b").unwrap();
    ui.add_agent(AgentListEntry::new(chat_a.clone(), "parser"));
    ui.add_agent(AgentListEntry::new(chat_b.clone(), "docs"));

    assert!(ui.render_screen("command chat").ends_with("\u{1b}[2;2H"));

    ui.handle_key(UiKey::Char('j'));
    assert!(ui.render_screen("command chat").ends_with("\u{1b}[3;2H"));
    assert_eq!(ui.selected_agent(), Some(&chat_a));
    assert_eq!(ui.focus(), PaneFocus::Left);
    assert!(ui.render_screen("agent chat").contains("chat-a"));

    ui.handle_key(UiKey::Char('j'));

    assert_eq!(ui.selected_agent(), Some(&chat_b));
    assert_eq!(ui.focus(), PaneFocus::Left);
    assert!(ui.render_screen("agent chat").contains("chat-b"));
}

#[test]
fn mouse_clicking_a_left_pane_agent_row_opens_that_agent_chat() {
    let mut ui = TerminalUi::new(100, 20);
    let chat_a = AgentId::new("chat-a").unwrap();
    let chat_b = AgentId::new("chat-b").unwrap();
    ui.add_agent(AgentListEntry::new(chat_a.clone(), "parser"));
    ui.add_agent(AgentListEntry::new(chat_b.clone(), "docs"));
    ui.select_agent(&chat_a).unwrap();

    ui.handle_key(UiKey::MouseClick { column: 4, row: 4 });

    assert_eq!(ui.selected_agent(), Some(&chat_b));
    assert_eq!(ui.focus(), PaneFocus::Right);
    assert_eq!(ui.layout().right_surface, Some(UiSurface::AgentChat));
}

#[test]
fn left_pane_can_hide_selected_agents_from_control_list() {
    let mut ui = TerminalUi::new(100, 20);
    let chat_a = AgentId::new("chat-a").unwrap();
    let chat_b = AgentId::new("chat-b").unwrap();
    ui.add_agent(AgentListEntry::new(chat_a.clone(), "parser"));
    ui.add_agent(AgentListEntry::new(chat_b.clone(), "docs"));

    ui.handle_key(UiKey::Char('j'));
    ui.handle_key(UiKey::Char('x'));

    let rendered = ui.render_left_pane();
    assert!(!rendered.contains("chat-a"));
    assert!(rendered.contains("chat-b"));
    assert!(ui.render_screen("command chat").ends_with("\u{1b}[3;2H"));
}

#[test]
fn raw_mode_screen_uses_crlf_so_frame_fills_terminal_width() {
    let ui = TerminalUi::new(80, 8);
    let rendered = ui.render_screen("command chat");

    assert!(rendered.contains("\r\n"));
    assert!(!rendered.contains(" \n"));
}

#[test]
fn visual_selection_highlights_and_yanks_right_pane_lines() {
    let mut ui = TerminalUi::new(80, 12);

    ui.handle_key(UiKey::CtrlW);
    ui.handle_key(UiKey::Char('l'));
    ui.render_screen("alpha\nbeta\nchat> ");
    ui.handle_key(UiKey::Char('V'));
    ui.handle_key(UiKey::Char('k'));

    let selected = ui.render_screen("alpha\nbeta\nchat> ");
    assert!(selected.contains("\u{1b}[7mbeta\u{1b}[0m"));

    ui.handle_key(UiKey::Char('Y'));

    assert_eq!(ui.mode(), UiMode::Command);
    assert_eq!(ui.clipboard_text(), "beta\nchat> ");
    assert!(ui.render_screen("alpha\nbeta\nchat> ").starts_with("\u{1b}]52;c;"));
}

fn strip_ansi(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}
----- END FILE tests/terminal_ui.rs -----

----- BEGIN FILE tests/ui_harness.rs -----
digest: fnv64:20aaa543a874f567; bytes:19268

use work_leaf::{PaneFocus, UiHarness, UiMode};

#[test]
fn scripted_harness_renders_full_width_crlf_frame() {
    let harness = UiHarness::new(80, 24);
    let rendered = harness.render_frame();
    let lines = rendered.split("\r\n").collect::<Vec<_>>();

    assert!(rendered.starts_with("\u{1b}[H"));
    assert!(!rendered.contains("\u{1b}[2J"));
    assert!(rendered.contains("UI harness"));
    assert!(rendered.contains("user-1"));
    assert_eq!(lines.len(), 24);
    assert!(
        lines
            .iter()
            .all(|line| strip_ansi(line).chars().count() == 80)
    );
}

#[test]
fn scripted_harness_rings_and_highlights_ready_chat() {
    let mut harness = UiHarness::new(100, 24);

    assert!(!harness.render_frame().starts_with('\u{7}'));

    harness
        .mark_agent_ready("user-2")
        .expect("fixture user-2 agent is registered");

    let ready_frame = harness.render_frame();
    assert!(ready_frame.starts_with('\u{7}'));
    assert!(ready_frame.contains("\u{1b}[7m test user-2 READY\u{1b}[0m"));
    assert!(!harness.render_frame().starts_with('\u{7}'));
}

#[test]
fn scripted_harness_feature_done_prompt_closes_and_reopens_chat() {
    let mut harness = UiHarness::new(100, 24);

    harness
        .ask_feature_done("user-2")
        .expect("fixture user-2 agent is registered");

    let pending_frame = harness.render_frame();
    assert!(pending_frame.starts_with('\u{7}'));
    assert!(pending_frame.contains("work-leaf: is this feature done? yes/no"));
    assert!(pending_frame.contains("\u{1b}[7m test user-2 READY\u{1b}[0m"));

    harness.handle_bytes(b"jiYes\n");

    let closed_left_pane = harness.ui().render_left_pane();
    assert!(!closed_left_pane.contains(">tests user-2  working: tests  READY"));
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "work-leaf: feature closed")
    );

    harness.handle_bytes(b"please reopen this feature\n");

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "work-leaf: feature reopened")
    );
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-2> please reopen this feature")
    );
    assert!(
        harness
            .ui()
            .render_left_pane()
            .contains(">tests user-2  working: tests  READY")
    );
}

#[test]
fn scripted_harness_switches_modes_without_enter() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_byte(b'i');
    assert_eq!(harness.ui().mode(), UiMode::Insert);

    harness.handle_byte(27);
    assert_eq!(harness.ui().mode(), UiMode::Command);

    harness.handle_byte(b':');
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    assert!(harness.render_frame().ends_with("\u{1b}[24;2H"));

    harness.handle_byte(b'n');
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    assert!(harness.render_frame().ends_with("\u{1b}[24;3H"));
}

#[test]
fn scripted_harness_prompt_arrow_keys_move_visible_cursor() {
    let mut harness = UiHarness::new(80, 24);
    harness.handle_bytes(b":ab\x1b[D");
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    assert!(harness.render_frame().ends_with("\u{1b}[24;3H"));
    harness.handle_bytes(b"\x1b[C");
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    assert!(harness.render_frame().ends_with("\u{1b}[24;4H"));
}

#[test]
fn scripted_harness_long_prompt_arrows_keep_rendered_cursor_at_edit_position() {
    let mut harness = UiHarness::new(20, 10);

    harness.handle_bytes(b":abcdefghijklmnopqrstuvwxyz0123\x1b[D\x1b[D\x1b[D\x1b[D\x1b[DX");

    let frame = harness.render_frame();
    assert!(frame.contains(":ijklmnopqrstuvwxyXz"));
    assert!(frame.ends_with("\u{1b}[10;20H"));

    harness.handle_bytes(b"\n");

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "unknown fixture command: abcdefghijklmnopqrstuvwxyXz0123")
    );
}
#[test]
fn scripted_harness_prompt_arrow_keys_recall_prompt_history() {
    let mut harness = UiHarness::new(80, 24);
    harness.handle_bytes(b":review\n:linearize\n:\x1b[A\x1b[A\x1b[B\n");
    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert_eq!(
        harness
            .transcript()
            .iter()
            .filter(|line| line.as_str() == "work-leaf> linearize")
            .count(),
        2
    );
}

#[test]
fn scripted_harness_prompt_history_down_restores_in_progress_prompt() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":review\n:draft command\x1b[A\x1b[B\n");

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "unknown fixture command: draft command")
    );
}

#[test]
fn scripted_harness_bytewise_prompt_arrow_keys_edit_without_leaving_prompt() {
    let mut harness = UiHarness::new(80, 24);
    harness.handle_byte(b':');
    harness.handle_byte(b'a');
    harness.handle_byte(b'b');
    harness.handle_byte(27);
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    harness.handle_byte(b'[');
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    harness.handle_byte(b'D');
    harness.handle_byte(b'Z');
    harness.handle_byte(b'\n');
    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "unknown fixture command: aZb")
    );
}
#[test]
fn scripted_harness_drives_ctrl_w_navigation_and_left_toggle() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert!(harness.render_frame().ends_with("\u{1b}[5;24H"));

    harness.handle_bytes(&[23, b'h']);
    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert!(harness.render_frame().ends_with("\u{1b}[3;2H"));

    harness.handle_byte(b',');
    assert_eq!(harness.ui().layout().left_width, 0);
    assert_eq!(harness.ui().layout().right_width, 80);
    assert!(harness.ui().layout().right_surface.is_some());
    assert_eq!(harness.ui().focus(), PaneFocus::Right);

    harness.handle_byte(b',');
    assert_eq!(harness.ui().layout().left_width, 16);
    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    harness.handle_bytes(&[23, b'j']);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);

    harness.handle_bytes(&[23, b'k']);
    assert_eq!(harness.ui().focus(), PaneFocus::Left);
}

#[test]
fn scripted_harness_ctrl_c_never_quits_and_only_right_focus_interrupts_agent() {
    let mut harness = UiHarness::new(80, 24);

    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert!(harness.handle_byte(3));
    assert!(!harness.is_quit());
    assert!(
        !harness
            .transcript()
            .iter()
            .any(|line| line.contains("sent Ctrl-C"))
    );

    harness.handle_bytes(&[23, b'l']);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert!(harness.handle_byte(3));
    assert!(!harness.is_quit());
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "work-leaf: sent Ctrl-C to user-1")
    );
}

#[test]
fn scripted_harness_command_mode_typing_shows_insert_mode_notice() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"hello");

    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert!(
        harness
            .render_frame()
            .contains("command mode: press i for insert mode before typing")
    );
}

#[test]
fn scripted_harness_ctrl_c_shows_quit_notice() {
    let mut harness = UiHarness::new(80, 24);

    assert!(harness.handle_byte(3));
    assert!(!harness.is_quit());
    assert!(
        harness
            .render_frame()
            .contains("to exit, press Esc then :q then Enter")
    );
}

#[test]
fn scripted_harness_structural_command_keys_do_not_show_typing_notice() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);

    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert!(!harness.render_frame().contains("command mode: press i"));
}

#[test]
fn scripted_harness_left_pane_navigation_does_not_show_typing_notice() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"jjjjj");

    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-2")
    );
    assert!(!harness.render_frame().contains("command mode: press i"));
}

#[test]
fn scripted_harness_quits_only_through_colon_q() {
    let mut harness = UiHarness::new(80, 24);

    assert!(harness.handle_byte(b'q'));
    assert!(!harness.is_quit());

    assert!(!harness.handle_bytes(b":q\n"));
    assert!(harness.is_quit());
}

#[test]
fn scripted_harness_new_commands_select_new_agent_chat() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":new ui automation\n");

    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-3")
    );
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line.contains("agent user-3 launched for: ui automation"))
    );
    assert!(harness.render_frame().contains("user-3"));

    harness.handle_bytes(b"\x1b:new\n");

    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-4")
    );
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert_eq!(harness.ui().mode(), UiMode::Insert);
}

#[test]
fn scripted_harness_names_new_chat_from_first_inserted_prompt() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":new\n");
    assert!(
        harness
            .ui()
            .render_left_pane()
            .contains(">harness-agent user-3  working: harness-agent")
    );

    harness.handle_bytes(b"please fix the OAuth redirect handler\n");

    let named_left_pane = harness.ui().render_left_pane();
    assert!(named_left_pane.contains(
        ">please-fix-the-oauth-redirect-handler user-3  working: please-fix-the-oauth-redirect-handler"
    ));
    assert!(!named_left_pane.contains("harness-agent user-3"));

    harness.handle_bytes(b"add cookie coverage\n");

    let unchanged_left_pane = harness.ui().render_left_pane();
    assert!(unchanged_left_pane.contains(
        ">please-fix-the-oauth-redirect-handler user-3  working: please-fix-the-oauth-redirect-handler"
    ));
    assert!(!unchanged_left_pane.contains("add-cookie-coverage user-3"));
}

#[test]
fn scripted_harness_insert_mode_records_chat_text_and_literal_colons() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"ihello:world\n");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> hello:world")
    );
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "fixture reply: message recorded")
    );
}

#[test]
fn scripted_harness_slash_command_starts_agent_chat_command_from_chat_view() {
    let mut harness = UiHarness::new(80, 24);

    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );

    harness.handle_bytes(b"/status\n");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> /status")
    );
}

#[test]
fn scripted_harness_prompt_slash_command_routes_to_selected_agent() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":/status\n");

    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> /status")
    );
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "fixture reply: message recorded")
    );
}

#[test]
fn scripted_harness_mouse_wheel_scrolls_chat_history() {
    let mut harness = UiHarness::new(80, 10);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_byte(b'i');
    for index in 0..12 {
        harness.handle_bytes(format!("message-{index:02}\n").as_bytes());
    }

    let bottom_frame = harness.render_frame();
    assert!(!bottom_frame.contains("UI harness"));
    assert!(bottom_frame.contains("message-11"));

    for _ in 0..8 {
        harness.handle_bytes(b"\x1b[<64;20;3M");
    }

    let scrolled_frame = harness.render_frame();
    assert!(scrolled_frame.contains("UI harness"));
    assert!(scrolled_frame.contains("chat> "));

    for _ in 0..8 {
        harness.handle_bytes(b"\x1b[<65;20;3M");
    }

    let bottom_again = harness.render_frame();
    assert!(!bottom_again.contains("UI harness"));
    assert!(bottom_again.contains("message-11"));
}

#[test]
fn scripted_harness_arrow_keys_edit_focused_chat_without_switching_to_command() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_bytes(b"iab\x1b[DZ\n");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> aZb")
    );
}

#[test]
fn scripted_harness_insert_arrow_keys_move_visible_chat_cursor() {
    let mut harness = UiHarness::new(80, 24);
    harness.handle_bytes(&[23, b'l']);
    harness.handle_bytes(b"iab\x1b[D");
    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(harness.render_frame().ends_with("\u{1b}[5;25H"));
    harness.handle_bytes(b"\x1b[C");
    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(harness.render_frame().ends_with("\u{1b}[5;26H"));
}
#[test]
fn scripted_harness_arrow_keys_recall_chat_history() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_bytes(b"ifirst\nsecond\n\x1b[A\x1b[A\x1b[B\n");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert_eq!(
        harness
            .transcript()
            .iter()
            .filter(|line| line.as_str() == "user-1> second")
            .count(),
        2
    );
}

#[test]
fn scripted_harness_chat_history_down_restores_in_progress_message() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"ifirst\nsecond draft\x1b[A\x1b[B\n");

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> second draft")
    );
}

#[test]
fn scripted_harness_bytewise_arrow_keys_edit_focused_chat_without_switching_to_command() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_byte(b'i');
    harness.handle_byte(b'a');
    harness.handle_byte(b'b');
    harness.handle_byte(27);
    harness.handle_byte(b'[');
    harness.handle_byte(b'D');
    harness.handle_byte(b'Z');
    harness.handle_byte(b'\n');

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> aZb")
    );
}

#[test]
fn scripted_harness_bytewise_arrow_prefix_keeps_focused_chat_in_insert_mode() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_byte(b'i');
    harness.handle_byte(b'a');
    harness.handle_byte(27);

    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(harness.render_frame().contains("mode=insert focus=right"));

    harness.handle_byte(b'[');
    assert_eq!(harness.ui().mode(), UiMode::Insert);

    harness.handle_byte(b'D');
    harness.handle_byte(b'Z');
    harness.handle_byte(b'\n');

    assert!(harness.transcript().iter().any(|line| line == "user-1> Za"));
}

#[test]
fn scripted_harness_arrow_keys_move_left_pane_selection_like_j_k() {
    let mut harness = UiHarness::new(80, 24);

    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );

    harness.handle_bytes(b"\x1b[B");
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-2")
    );

    harness.handle_bytes(b"\x1b[A");
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );
}

#[test]
fn scripted_harness_left_right_arrows_move_left_pane_selection_like_j_k() {
    let mut harness = UiHarness::new(80, 24);

    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );

    harness.handle_bytes(b"\x1b[C");
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-2")
    );

    harness.handle_bytes(b"\x1b[D");
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );
}

#[test]
fn scripted_harness_visual_mode_yanks_focused_left_pane_text() {
    let mut harness = UiHarness::new(100, 24);

    harness.render_frame();
    harness.handle_byte(b'v');
    assert_eq!(harness.ui().mode(), UiMode::Visual);

    harness.handle_bytes(b"llll");
    harness.handle_byte(b'y');

    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert_eq!(harness.ui().clipboard_text(), ">pars");
    assert!(harness.render_frame().contains("copied"));
}

#[test]
fn scripted_harness_visual_line_mode_yanks_focused_right_pane_lines() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    harness.render_frame();
    harness.handle_byte(b'V');
    assert_eq!(harness.ui().mode(), UiMode::VisualLine);

    harness.handle_byte(b'k');
    harness.handle_byte(b'y');

    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert!(harness.ui().clipboard_text().contains("Esc command"));
    assert!(harness.ui().clipboard_text().contains("chat> "));
}

#[test]
fn scripted_harness_visual_block_mode_yanks_rectangular_left_pane_text() {
    let mut harness = UiHarness::new(100, 24);

    harness.render_frame();
    harness.handle_byte(22);
    assert_eq!(harness.ui().mode(), UiMode::VisualBlock);

    harness.handle_bytes(b"lljy");

    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert_eq!(harness.ui().clipboard_text(), ">pa\n   ");
}

fn strip_ansi(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}
----- END FILE tests/ui_harness.rs -----
