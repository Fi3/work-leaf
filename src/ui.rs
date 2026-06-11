use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
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
    Z,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisualSelectionMode {
    Character,
    Line,
    Block,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VisualPoint {
    row: usize,
    column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VisualSelection {
    pane: PaneFocus,
    mode: VisualSelectionMode,
    anchor: VisualPoint,
    cursor: VisualPoint,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ChatMessageFoldKey {
    first_line: String,
    occurrence: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ChatDisplayLine {
    text: String,
    message_key: Option<ChatMessageFoldKey>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ChatHistoryMessage {
    key: ChatMessageFoldKey,
    lines: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VisualCursor {
    pane: PaneFocus,
    point: VisualPoint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LeftPaneLine {
    text: String,
    ready: bool,
    click_target: Option<LeftPaneClickTarget>,
    control_target: Option<LeftPaneClickTarget>,
}

impl LeftPaneLine {
    fn section(title: &str, format: LeftPaneLineFormat) -> Self {
        Self {
            text: match format {
                LeftPaneLineFormat::Detailed => format!("[{title}]"),
                LeftPaneLineFormat::Compact { inner_width } => {
                    truncate_to_width(&format!("[{title}]"), inner_width.max(1))
                }
            },
            ready: false,
            click_target: None,
            control_target: None,
        }
    }

    fn command(text: String) -> Self {
        Self {
            text,
            ready: false,
            click_target: Some(LeftPaneClickTarget::Command),
            control_target: Some(LeftPaneClickTarget::Command),
        }
    }

    fn agent_row(agent: &AgentListEntry, text: String, ready: bool) -> Self {
        let target = LeftPaneClickTarget::Agent(agent.id.clone());
        Self {
            text,
            ready,
            click_target: Some(target.clone()),
            control_target: Some(target),
        }
    }

    fn agent_detail(text: String, target: Option<LeftPaneClickTarget>) -> Self {
        Self {
            text,
            ready: false,
            click_target: target,
            control_target: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LeftPaneLineFormat {
    Detailed,
    Compact { inner_width: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LeftPaneAgentSection {
    ClosedPatches,
    NewPatches,
    ReadyPatches,
    WorkingPatches,
    Reviewing,
    Reviewed,
    Reads,
    Linearize,
}

impl LeftPaneAgentSection {
    fn title(self) -> &'static str {
        match self {
            Self::ClosedPatches => "closed",
            Self::NewPatches => "new",
            Self::ReadyPatches => "ready",
            Self::WorkingPatches => "working",
            Self::Reviewing => "reviewing",
            Self::Reviewed => "reviewed",
            Self::Reads => "reads",
            Self::Linearize => "linearize",
        }
    }
}

const LEFT_PANE_AGENT_SECTIONS: [LeftPaneAgentSection; 8] = [
    LeftPaneAgentSection::ClosedPatches,
    LeftPaneAgentSection::NewPatches,
    LeftPaneAgentSection::ReadyPatches,
    LeftPaneAgentSection::WorkingPatches,
    LeftPaneAgentSection::Reviewing,
    LeftPaneAgentSection::Reviewed,
    LeftPaneAgentSection::Reads,
    LeftPaneAgentSection::Linearize,
];

#[derive(Clone, Debug, Eq, PartialEq)]
struct StatusNotice {
    message: String,
    expires_at: Instant,
}

const STATUS_NOTICE_SECONDS: u64 = 5;
const COMMAND_MODE_TYPING_NOTICE_THRESHOLD: usize = 5;
const COMMAND_MODE_TYPING_NOTICE: &str = "command mode: press i for insert mode before typing";
const CTRL_C_EXIT_NOTICE: &str = "to exit, press Esc then :q then Enter";
const CTRL_V: char = '\u{16}';
const CHAT_TRANSCRIPT_MESSAGE_SEPARATOR: char = '\u{1e}';

#[derive(Clone, Debug, Eq, PartialEq)]
enum LeftPaneClickTarget {
    Command,
    Agent(AgentId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AgentListPatchState {
    has_user_message: bool,
    closed: bool,
}

impl AgentListPatchState {
    fn default_for_agent() -> Self {
        Self {
            has_user_message: true,
            closed: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiWindow {
    surface: UiSurface,
    agent_id: Option<AgentId>,
    folds: ChatFoldState,
}

impl UiWindow {
    fn command() -> Self {
        Self {
            surface: UiSurface::WorkLeafCommand,
            agent_id: None,
            folds: ChatFoldState::default(),
        }
    }

    fn chat(agent_id: AgentId) -> Self {
        Self {
            surface: UiSurface::AgentChat,
            agent_id: Some(agent_id),
            folds: ChatFoldState::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ChatFoldState {
    fold_all_messages: bool,
    folded_messages: BTreeSet<ChatMessageFoldKey>,
    unfolded_messages: BTreeSet<ChatMessageFoldKey>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PromptView {
    line: String,
    cursor_column: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalUi {
    width: u16,
    height: u16,
    mode: UiMode,
    focus: PaneFocus,
    left_visible: bool,
    agents: Vec<AgentListEntry>,
    agent_patch_states: BTreeMap<AgentId, AgentListPatchState>,
    review_no_findings: BTreeSet<AgentId>,
    selected_agent: Option<AgentId>,
    control_selected: usize,
    split_chats: Vec<AgentId>,
    windows: Vec<UiWindow>,
    active_window: usize,
    right_scroll_rows: usize,
    pending: Option<PendingKey>,
    pending_bell: Cell<bool>,
    pending_clipboard: RefCell<Option<String>>,
    last_copied_text: Option<String>,
    visual_cursor: Option<VisualCursor>,
    visual_selection: Option<VisualSelection>,
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
            agent_patch_states: BTreeMap::new(),
            review_no_findings: BTreeSet::new(),
            selected_agent: None,
            control_selected: 0,
            split_chats: Vec::new(),
            windows: vec![UiWindow::command()],
            active_window: 0,
            right_scroll_rows: 0,
            pending: None,
            pending_bell: Cell::new(false),
            pending_clipboard: RefCell::new(None),
            last_copied_text: None,
            visual_cursor: None,
            visual_selection: None,
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

    pub fn visual_selection_active(&self) -> bool {
        self.visual_selection.is_some()
    }

    pub fn copied_text(&self) -> Option<&str> {
        self.last_copied_text.as_deref()
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

    pub(crate) fn set_agent_relationships(
        &mut self,
        agent_id: &AgentId,
        depends_on: Vec<AgentId>,
        depended_on_by: Vec<AgentId>,
    ) -> Result<(), String> {
        let Some(agent) = self.agents.iter_mut().find(|agent| &agent.id == agent_id) else {
            return Err(format!("unknown agent `{agent_id}`"));
        };
        agent.depends_on = depends_on;
        agent.depended_on_by = depended_on_by;
        Ok(())
    }

    pub fn set_agent_patch_lifecycle(
        &mut self,
        agent_id: &AgentId,
        has_user_message: bool,
        closed: bool,
    ) -> Result<(), String> {
        let control_agent = self.control_selected_agent_id();
        if !self.agents.iter().any(|agent| &agent.id == agent_id) {
            return Err(format!("unknown agent `{agent_id}`"));
        }
        self.agent_patch_states.insert(
            agent_id.clone(),
            AgentListPatchState {
                has_user_message,
                closed,
            },
        );
        self.restore_control_selection(control_agent.as_ref());
        Ok(())
    }

    pub(crate) fn set_agent_review_no_findings(
        &mut self,
        agent_id: &AgentId,
        no_findings: bool,
    ) -> Result<(), String> {
        let control_agent = self.control_selected_agent_id();
        if !self.agents.iter().any(|agent| &agent.id == agent_id) {
            return Err(format!("unknown agent `{agent_id}`"));
        }
        if no_findings {
            self.review_no_findings.insert(agent_id.clone());
        } else {
            self.review_no_findings.remove(agent_id);
        }
        self.restore_control_selection(control_agent.as_ref());
        Ok(())
    }

    pub(crate) fn set_agent_ready_state(
        &mut self,
        agent_id: &AgentId,
        ready: bool,
    ) -> Result<(), String> {
        let control_agent = self.control_selected_agent_id();
        let Some(agent_index) = self.agents.iter().position(|agent| &agent.id == agent_id) else {
            return Err(format!("unknown agent `{agent_id}`"));
        };
        let should_ring = {
            let agent = &self.agents[agent_index];
            ready && !agent.ready && self.agent_allows_ready_highlight(agent)
        };
        if should_ring {
            self.pending_bell.set(true);
        }
        self.agents[agent_index].ready = ready;
        self.restore_control_selection(control_agent.as_ref());
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
        self.clear_visual_selection();
        Ok(())
    }

    pub fn select_command_interface(&mut self) {
        self.selected_agent = None;
        self.windows[self.active_window] = UiWindow::command();
        self.control_selected = 0;
        self.clear_visual_selection();
        self.reset_right_scroll();
    }

    pub fn handle_key(&mut self, key: UiKey) -> Vec<UiAction> {
        self.handle_key_with_context(key, "", None)
    }

    pub fn handle_key_with_context(
        &mut self,
        key: UiKey,
        right_content: &str,
        right_cursor_column: Option<usize>,
    ) -> Vec<UiAction> {
        let visible_right_content = self.visible_right_content(right_content);
        let command_mode_text_key = self.command_mode_text_key_control_status(key);
        let actions = self.handle_key_inner(
            key,
            right_content,
            &visible_right_content,
            right_cursor_column,
        );
        self.update_command_mode_typing_notice(command_mode_text_key);
        actions
    }

    fn handle_key_inner(
        &mut self,
        key: UiKey,
        right_content: &str,
        visible_right_content: &str,
        right_cursor_column: Option<usize>,
    ) -> Vec<UiAction> {
        match key {
            UiKey::MouseClick { column, row } => {
                self.pending = None;
                self.clear_visual_selection();
                return self.handle_mouse_click(column, row);
            }
            UiKey::MouseScrollUp { column, row } => {
                self.pending = None;
                self.clear_visual_selection();
                self.handle_mouse_scroll(column, row, true);
                return Vec::new();
            }
            UiKey::MouseScrollDown { column, row } => {
                self.pending = None;
                self.clear_visual_selection();
                self.handle_mouse_scroll(column, row, false);
                return Vec::new();
            }
            _ => {}
        }

        if let Some(pending) = self.pending.take() {
            return self.handle_pending_key(pending, key, right_content);
        }

        if self.visual_selection.is_some() {
            return self.handle_visual_key(key, right_content, visible_right_content);
        }

        if self.visual_cursor.is_some() {
            return self.handle_visual_cursor_key(
                key,
                right_content,
                visible_right_content,
                right_cursor_column,
            );
        }

        match key {
            UiKey::Esc => {
                self.mode = UiMode::Command;
                self.clear_visual_selection();
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
            UiKey::Char('G') if self.mode == UiMode::Command => {
                self.reset_right_scroll();
                Vec::new()
            }
            UiKey::Char('v') if self.mode == UiMode::Command => {
                self.start_visual_cursor(visible_right_content, right_cursor_column);
                Vec::new()
            }
            UiKey::Char('V') if self.mode == UiMode::Command => {
                self.start_visual_selection(
                    VisualSelectionMode::Line,
                    visible_right_content,
                    right_cursor_column,
                );
                Vec::new()
            }
            UiKey::Char(CTRL_V) if self.mode == UiMode::Command => {
                self.start_visual_selection(
                    VisualSelectionMode::Block,
                    visible_right_content,
                    right_cursor_column,
                );
                Vec::new()
            }
            UiKey::Char('Y') if self.mode == UiMode::Command => {
                self.yank_current_line(visible_right_content, right_cursor_column);
                Vec::new()
            }
            UiKey::Char('z') if self.mode == UiMode::Command && self.chat_folding_enabled() => {
                self.pending = Some(PendingKey::Z);
                Vec::new()
            }
            UiKey::Char('i') if self.mode == UiMode::Command => {
                self.clear_visual_selection();
                self.mode = UiMode::Insert;
                Vec::new()
            }
            UiKey::Char(':') if self.mode == UiMode::Command => {
                self.clear_visual_selection();
                self.mode = UiMode::Prompt;
                Vec::new()
            }
            UiKey::Char(',') if self.mode == UiMode::Command => {
                self.clear_visual_selection();
                if self.focus == PaneFocus::Right && self.left_visible {
                    self.focus = PaneFocus::Left;
                } else {
                    self.left_visible = !self.left_visible;
                    self.focus = if self.left_visible {
                        PaneFocus::Left
                    } else {
                        PaneFocus::Right
                    };
                }
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
            UiKey::Char('\n') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.open_control_selection_in_insert_mode();
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
        for line in self.left_pane_detail_lines() {
            if line.ready {
                rendered.push_str("\u{1b}[7m");
                rendered.push_str(&line.text);
                rendered.push_str("\u{1b}[0m");
            } else {
                rendered.push_str(&line.text);
            }
            rendered.push('\n');
        }
        rendered
    }

    pub fn render_screen(&self, right_content: &str) -> String {
        self.render_screen_with_prompt(right_content, "")
    }

    pub fn render_screen_with_prompt(&self, right_content: &str, prompt: &str) -> String {
        let visible_right_content = self.visible_right_content(right_content);
        let prompt_cursor = prompt.len();
        let buffer = self.render_tui_buffer(&visible_right_content, prompt, prompt_cursor);
        let mut rendered = String::new();
        rendered.push_str(&self.clipboard_prefix());
        rendered.push_str(self.bell_prefix());
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
        let buffer = self.render_tui_buffer(&visible_right_content, prompt, prompt_cursor);
        let mut rendered = String::new();
        rendered.push_str(&self.clipboard_prefix());
        rendered.push_str(self.bell_prefix());
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

    fn scroll_right_pane_to_top(&mut self, right_content: &str) {
        let (inner_width, inner_height) = self.right_inner_size();
        let folds = self.active_chat_fold_state();
        self.right_scroll_rows = max_scroll_rows(
            right_content,
            inner_width,
            inner_height,
            folds.fold_all_messages,
            &folds.folded_messages,
            &folds.unfolded_messages,
        );
    }

    fn visible_right_content(&self, right_content: &str) -> String {
        chat_display_lines_to_string(&self.visible_right_content_lines(right_content))
    }

    fn visible_right_content_lines(&self, right_content: &str) -> Vec<ChatDisplayLine> {
        let (inner_width, inner_height) = self.right_inner_size();
        let folds = self.active_chat_fold_state();
        visible_content_lines(
            right_content,
            inner_width,
            inner_height,
            self.right_scroll_rows,
            folds.fold_all_messages,
            &folds.folded_messages,
            &folds.unfolded_messages,
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
        let mut buffer = terminal.backend().buffer().clone();
        self.apply_visual_selection(&mut buffer, right_content);
        buffer
    }

    fn left_widget(&self) -> List<'static> {
        let inner_width = usize::from(self.layout().left_width.saturating_sub(2).max(1));
        let items = self
            .left_pane_lines(inner_width)
            .into_iter()
            .map(|line| {
                let item = ListItem::new(Spans::from(vec![Span::raw(line.text)]));
                if line.ready {
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
        Paragraph::new(right_content.to_string())
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
        } else if let Some(position) = self.visual_cursor_position(right_content) {
            position
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

    fn clipboard_prefix(&self) -> String {
        self.pending_clipboard
            .borrow_mut()
            .take()
            .map(|text| format!("\u{1b}]52;c;{}\u{7}", base64_encode(text.as_bytes())))
            .unwrap_or_default()
    }

    fn handle_visual_key(
        &mut self,
        key: UiKey,
        right_content: &str,
        visible_right_content: &str,
    ) -> Vec<UiAction> {
        match key {
            UiKey::Esc => self.clear_visual_selection(),
            UiKey::Char('v') => self.set_visual_mode(VisualSelectionMode::Character),
            UiKey::Char('V') => self.set_visual_mode(VisualSelectionMode::Line),
            UiKey::Char(CTRL_V) => self.set_visual_mode(VisualSelectionMode::Block),
            UiKey::Char('y') => self.yank_visual_selection(visible_right_content, false),
            UiKey::Char('Y') => self.yank_visual_selection(visible_right_content, true),
            UiKey::Char('g') => self.pending = Some(PendingKey::G),
            UiKey::Char('z') if self.chat_folding_enabled() => self.pending = Some(PendingKey::Z),
            UiKey::Char('G') => self.jump_right_visual_cursor_to_bottom(right_content),
            UiKey::Char('h') | UiKey::Left => {
                self.move_visual_cursor(0, -1, right_content, visible_right_content);
            }
            UiKey::Char('l') | UiKey::Right => {
                self.move_visual_cursor(0, 1, right_content, visible_right_content);
            }
            UiKey::Char('j') | UiKey::Down => {
                self.move_visual_cursor(1, 0, right_content, visible_right_content);
            }
            UiKey::Char('k') | UiKey::Up => {
                self.move_visual_cursor(-1, 0, right_content, visible_right_content);
            }
            _ => {}
        }
        Vec::new()
    }

    fn handle_visual_cursor_key(
        &mut self,
        key: UiKey,
        right_content: &str,
        visible_right_content: &str,
        right_cursor_column: Option<usize>,
    ) -> Vec<UiAction> {
        match key {
            UiKey::Esc => self.clear_visual_selection(),
            UiKey::Char('v') => self.start_visual_selection(
                VisualSelectionMode::Character,
                visible_right_content,
                right_cursor_column,
            ),
            UiKey::Char('V') => self.start_visual_selection(
                VisualSelectionMode::Line,
                visible_right_content,
                right_cursor_column,
            ),
            UiKey::Char(CTRL_V) => self.start_visual_selection(
                VisualSelectionMode::Block,
                visible_right_content,
                right_cursor_column,
            ),
            UiKey::Char('Y') => {
                self.yank_current_line(visible_right_content, right_cursor_column);
                self.clear_visual_selection();
            }
            UiKey::Char('g') => self.pending = Some(PendingKey::G),
            UiKey::Char('z') if self.chat_folding_enabled() => self.pending = Some(PendingKey::Z),
            UiKey::Char('G') => self.jump_right_visual_cursor_to_bottom(right_content),
            UiKey::Char('h') | UiKey::Left => {
                self.move_visual_cursor(0, -1, right_content, visible_right_content);
            }
            UiKey::Char('l') | UiKey::Right => {
                self.move_visual_cursor(0, 1, right_content, visible_right_content);
            }
            UiKey::Char('j') | UiKey::Down => {
                self.move_visual_cursor(1, 0, right_content, visible_right_content);
            }
            UiKey::Char('k') | UiKey::Up => {
                self.move_visual_cursor(-1, 0, right_content, visible_right_content);
            }
            _ => {}
        }
        Vec::new()
    }

    fn start_visual_cursor(
        &mut self,
        visible_right_content: &str,
        right_cursor_column: Option<usize>,
    ) {
        let point = self.visual_start_point(visible_right_content, right_cursor_column);
        self.visual_cursor = Some(VisualCursor {
            pane: self.focus,
            point,
        });
    }

    fn start_visual_selection(
        &mut self,
        mode: VisualSelectionMode,
        visible_right_content: &str,
        right_cursor_column: Option<usize>,
    ) {
        let (pane, point) = self
            .visual_cursor
            .take()
            .map(|cursor| (cursor.pane, cursor.point))
            .unwrap_or_else(|| {
                (
                    self.focus,
                    self.visual_start_point(visible_right_content, right_cursor_column),
                )
            });
        self.visual_selection = Some(VisualSelection {
            pane,
            mode,
            anchor: point,
            cursor: point,
        });
    }

    fn set_visual_mode(&mut self, mode: VisualSelectionMode) {
        if let Some(selection) = self.visual_selection.as_mut() {
            selection.mode = mode;
        }
    }

    fn clear_visual_selection(&mut self) {
        self.visual_cursor = None;
        self.visual_selection = None;
    }

    fn jump_right_visual_cursor_to_top(&mut self, right_content: &str) {
        self.scroll_right_pane_to_top(right_content);
        let visible_right_content = self.visible_right_content(right_content);
        self.set_right_visual_cursor_row(&visible_right_content, 0);
    }

    fn jump_right_visual_cursor_to_bottom(&mut self, right_content: &str) {
        self.reset_right_scroll();
        let visible_right_content = self.visible_right_content(right_content);
        let lines = self.visual_pane_lines(&visible_right_content, PaneFocus::Right);
        let row = self.right_visual_start_row(&lines);
        self.set_right_visual_cursor_row(&visible_right_content, row);
    }

    fn set_right_visual_cursor_row(&mut self, visible_right_content: &str, row: usize) {
        let lines = self.visual_pane_lines(visible_right_content, PaneFocus::Right);
        if lines.is_empty() {
            return;
        }
        let row = row.min(lines.len().saturating_sub(1));

        if let Some(selection) = self.visual_selection.as_mut() {
            if selection.pane != PaneFocus::Right {
                return;
            }
            let column = selection
                .cursor
                .column
                .min(lines[row].chars().count().saturating_sub(1));
            let point = VisualPoint { row, column };
            selection.anchor = point;
            selection.cursor = point;
        } else if let Some(cursor) = self.visual_cursor.as_mut() {
            if cursor.pane != PaneFocus::Right {
                return;
            }
            let column = cursor
                .point
                .column
                .min(lines[row].chars().count().saturating_sub(1));
            cursor.point = VisualPoint { row, column };
        }
    }

    fn visual_start_point(
        &self,
        visible_right_content: &str,
        right_cursor_column: Option<usize>,
    ) -> VisualPoint {
        match self.focus {
            PaneFocus::Left => {
                let lines = self.visual_pane_lines(visible_right_content, PaneFocus::Left);
                let row = self.control_selected.min(lines.len().saturating_sub(1));
                VisualPoint { row, column: 0 }
            }
            PaneFocus::Right => {
                let lines = self.visual_pane_lines(visible_right_content, PaneFocus::Right);
                let row = self.right_visual_start_row(&lines);
                let line_len = lines
                    .get(row)
                    .map(|line| line.chars().count())
                    .unwrap_or_default();
                let max_column = line_len.saturating_sub(1);
                VisualPoint {
                    row,
                    column: if self.mode == UiMode::Command {
                        0
                    } else {
                        right_cursor_column.unwrap_or(0).min(max_column)
                    },
                }
            }
        }
    }

    fn right_visual_start_row(&self, lines: &[String]) -> usize {
        if lines.is_empty() {
            return 0;
        }
        if self.mode == UiMode::Command
            && let Some(prompt_row) = lines.iter().rposition(|line| line.starts_with("chat> "))
            && let Some(history_row) = prompt_row.checked_sub(1)
        {
            return history_row;
        }
        lines.len().saturating_sub(1)
    }

    fn move_visual_cursor(
        &mut self,
        row_delta: isize,
        column_delta: isize,
        right_content: &str,
        visible_right_content: &str,
    ) {
        let Some((pane, point)) = self
            .visual_selection
            .as_ref()
            .map(|selection| (selection.pane, selection.cursor))
            .or_else(|| {
                self.visual_cursor
                    .as_ref()
                    .map(|cursor| (cursor.pane, cursor.point))
            })
        else {
            return;
        };
        let lines = self.visual_pane_lines(visible_right_content, pane);
        if lines.is_empty() {
            return;
        }
        let max_row = lines.len().saturating_sub(1) as isize;
        let requested_row = point.row as isize + row_delta;
        if pane == PaneFocus::Right {
            if requested_row < 0 && self.scroll_right_visual_view(right_content, true) {
                return;
            }
            if requested_row > max_row && self.scroll_right_visual_view(right_content, false) {
                return;
            }
        }
        let next_row = requested_row.clamp(0, max_row) as usize;
        let line_len = lines[next_row].chars().count();
        let max_column = line_len.saturating_sub(1) as isize;
        let next_column = (point.column as isize + column_delta).clamp(0, max_column) as usize;
        if let Some(selection) = self.visual_selection.as_mut() {
            selection.cursor = VisualPoint {
                row: next_row,
                column: next_column,
            };
        } else if let Some(cursor) = self.visual_cursor.as_mut() {
            cursor.point = VisualPoint {
                row: next_row,
                column: next_column,
            };
        }
    }

    fn scroll_right_visual_view(&mut self, right_content: &str, up: bool) -> bool {
        let previous_scroll_rows = self.right_scroll_rows;
        if up {
            let (inner_width, inner_height) = self.right_inner_size();
            let folds = self.active_chat_fold_state();
            self.right_scroll_rows = self
                .right_scroll_rows
                .saturating_add(1)
                .min(max_scroll_rows(
                    right_content,
                    inner_width,
                    inner_height,
                    folds.fold_all_messages,
                    &folds.folded_messages,
                    &folds.unfolded_messages,
                ));
        } else {
            self.right_scroll_rows = self.right_scroll_rows.saturating_sub(1);
        }
        let changed = self.right_scroll_rows != previous_scroll_rows;
        if changed {
            let visible_right_content = self.visible_right_content(right_content);
            let lines = self.visual_pane_lines(&visible_right_content, PaneFocus::Right);
            if !lines.is_empty() {
                let row = if up {
                    0
                } else {
                    self.right_visual_start_row(&lines)
                };
                self.set_right_visual_cursor_row(&visible_right_content, row);
            }
        }
        changed
    }

    fn yank_current_line(
        &mut self,
        visible_right_content: &str,
        right_cursor_column: Option<usize>,
    ) {
        let (pane, point) = self
            .visual_cursor
            .as_ref()
            .map(|cursor| (cursor.pane, cursor.point))
            .unwrap_or_else(|| {
                (
                    self.focus,
                    self.visual_start_point(visible_right_content, right_cursor_column),
                )
            });
        let lines = self.visual_pane_lines(visible_right_content, pane);
        if let Some(text) = lines.get(point.row).cloned() {
            self.copy_text_to_clipboard(text);
        }
    }

    fn yank_visual_selection(&mut self, visible_right_content: &str, force_line: bool) {
        if let Some(text) = self.selected_visual_text(visible_right_content, force_line) {
            self.copy_text_to_clipboard(text);
        }
        self.clear_visual_selection();
    }

    fn selected_visual_text(
        &self,
        visible_right_content: &str,
        force_line: bool,
    ) -> Option<String> {
        let selection = self.visual_selection.as_ref()?;
        let lines = self.visual_pane_lines(visible_right_content, selection.pane);
        (!lines.is_empty()).then(|| extract_visual_text(&lines, selection, force_line))
    }

    fn copy_text_to_clipboard(&mut self, text: String) {
        let char_count = text.chars().count();
        self.last_copied_text = Some(text.clone());
        let _ = write_system_clipboard(&text);
        self.pending_clipboard.replace(Some(text));
        self.show_status_notice(
            format!("copied {char_count} chars"),
            Duration::from_secs(STATUS_NOTICE_SECONDS),
        );
    }

    fn visual_pane_lines(&self, visible_right_content: &str, pane: PaneFocus) -> Vec<String> {
        match pane {
            PaneFocus::Left => {
                let inner_width = usize::from(self.layout().left_width.saturating_sub(2).max(1));
                self.left_pane_lines(inner_width)
                    .into_iter()
                    .map(|line| line.text)
                    .collect()
            }
            PaneFocus::Right => content_lines(visible_right_content),
        }
    }

    fn left_pane_detail_lines(&self) -> Vec<LeftPaneLine> {
        self.left_pane_lines_with_format(LeftPaneLineFormat::Detailed)
    }

    fn left_pane_lines(&self, inner_width: usize) -> Vec<LeftPaneLine> {
        self.left_pane_lines_with_format(LeftPaneLineFormat::Compact { inner_width })
    }

    fn left_pane_lines_with_format(&self, format: LeftPaneLineFormat) -> Vec<LeftPaneLine> {
        let mut lines = Vec::new();
        let command_row = if self.control_selected == 0 {
            "> work-leaf  command".to_string()
        } else {
            "  work-leaf  command".to_string()
        };
        lines.push(LeftPaneLine::section("command", format));
        lines.push(LeftPaneLine::command(command_row));

        let visible_agent_indices = self.visible_agent_indices();
        for section in LEFT_PANE_AGENT_SECTIONS {
            let mut section_started = false;
            for (visible_position, agent_index) in visible_agent_indices.iter().enumerate() {
                let agent = &self.agents[*agent_index];
                if self.agent_left_pane_section(agent) != section {
                    continue;
                }
                if !section_started {
                    lines.push(LeftPaneLine::section(section.title(), format));
                    section_started = true;
                }
                let selected = self.control_selected == visible_position + 1;
                let ready = self.agent_ready_visible(agent);
                let row = match format {
                    LeftPaneLineFormat::Detailed => detailed_agent_row(agent, selected, ready),
                    LeftPaneLineFormat::Compact { inner_width } => {
                        compact_agent_row(agent, selected, inner_width, ready)
                    }
                };
                lines.push(LeftPaneLine::agent_row(agent, row, ready));
                if !agent.modified_files.is_empty() {
                    lines.push(LeftPaneLine::agent_detail(
                        format!(
                            "    files: {}",
                            agent
                                .modified_files
                                .iter()
                                .map(|path| path.display().to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        Some(LeftPaneClickTarget::Agent(agent.id.clone())),
                    ));
                }
                for (label, agents) in [
                    ("conflicts", &agent.conflicting_agents),
                    ("depends-on", &agent.depends_on),
                    ("depended-on-by", &agent.depended_on_by),
                ] {
                    if !agents.is_empty() {
                        lines.push(LeftPaneLine::agent_detail(
                            format!(
                                "    {label}: {}",
                                agents
                                    .iter()
                                    .map(AgentId::as_str)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            agents.first().cloned().map(LeftPaneClickTarget::Agent),
                        ));
                    }
                }
            }
        }
        lines
    }

    fn apply_visual_selection(&self, buffer: &mut Buffer, visible_right_content: &str) {
        let Some(selection) = self.visual_selection.as_ref() else {
            return;
        };
        let Some(area) = self.visual_pane_area(selection.pane) else {
            return;
        };
        let lines = self.visual_pane_lines(visible_right_content, selection.pane);
        let width = usize::from(area.width.max(1));
        for row in visual_row_range(selection, lines.len()) {
            if row >= usize::from(area.height) {
                break;
            }
            let line_len = lines
                .get(row)
                .map(|line| line.chars().count())
                .unwrap_or_default();
            let Some((start, end)) = visual_column_range(selection, row, line_len, width, false)
            else {
                continue;
            };
            for column in start..=end.min(width.saturating_sub(1)) {
                let x = area
                    .x
                    .saturating_add(column.min(usize::from(u16::MAX)) as u16);
                let y = area.y.saturating_add(row.min(usize::from(u16::MAX)) as u16);
                if x < buffer.area.width && y < buffer.area.height {
                    buffer.get_mut(x, y).modifier.insert(Modifier::REVERSED);
                }
            }
        }
    }

    fn visual_pane_area(&self, pane: PaneFocus) -> Option<Rect> {
        let layout = self.layout();
        let body_height = self.height.saturating_sub(1);
        let pane_height = body_height.saturating_sub(2);
        match pane {
            PaneFocus::Left if self.left_visible && layout.left_width > 2 => Some(Rect::new(
                1,
                1,
                layout.left_width.saturating_sub(2),
                pane_height,
            )),
            PaneFocus::Right if layout.right_width > 2 => Some(Rect::new(
                layout.left_width.saturating_add(1),
                1,
                layout.right_width.saturating_sub(2),
                pane_height,
            )),
            _ => None,
        }
    }

    fn visual_cursor_position(&self, visible_right_content: &str) -> Option<(u16, u16)> {
        let (pane, point) = self
            .visual_selection
            .as_ref()
            .map(|selection| (selection.pane, selection.cursor))
            .or_else(|| {
                self.visual_cursor
                    .as_ref()
                    .map(|cursor| (cursor.pane, cursor.point))
            })?;
        let area = self.visual_pane_area(pane)?;
        let lines = self.visual_pane_lines(visible_right_content, pane);
        if lines.is_empty() {
            return Some((area.y.saturating_add(1), area.x.saturating_add(1)));
        }
        let row = point.row.min(lines.len().saturating_sub(1));
        let column = point
            .column
            .min(lines[row].chars().count().saturating_sub(1));
        Some((
            area.y.saturating_add(row.min(usize::from(u16::MAX)) as u16),
            area.x
                .saturating_add(column.min(usize::from(u16::MAX)) as u16),
        ))
    }

    fn handle_pending_key(
        &mut self,
        pending: PendingKey,
        key: UiKey,
        right_content: &str,
    ) -> Vec<UiAction> {
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
            (PendingKey::G, UiKey::Char('g')) if self.mode == UiMode::Command => {
                self.jump_right_visual_cursor_to_top(right_content);
                Vec::new()
            }
            (PendingKey::Z, UiKey::Char('M')) if self.chat_folding_enabled() => {
                self.fold_all_chat_messages(right_content);
                Vec::new()
            }
            (PendingKey::Z, UiKey::Char('R')) if self.chat_folding_enabled() => {
                self.unfold_all_chat_messages(right_content);
                Vec::new()
            }
            (PendingKey::Z, UiKey::Char('c'))
                if self.chat_folding_enabled() && self.visual_mode_active() =>
            {
                self.set_visual_chat_message_folded(right_content, true);
                Vec::new()
            }
            (PendingKey::Z, UiKey::Char('o'))
                if self.chat_folding_enabled() && self.visual_mode_active() =>
            {
                self.set_visual_chat_message_folded(right_content, false);
                Vec::new()
            }
            (PendingKey::Z, UiKey::Char('a'))
                if self.chat_folding_enabled() && self.visual_mode_active() =>
            {
                self.toggle_visual_chat_message_fold(right_content);
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
        matches!(
            ch,
            'i' | ':' | ',' | 's' | 't' | 'f' | 'g' | 'G' | 'v' | 'V' | 'Y' | 'z' | CTRL_V
        ) || (self.focus == PaneFocus::Left && matches!(ch, 'j' | 'k' | 'l' | 'x'))
    }

    fn chat_folding_enabled(&self) -> bool {
        self.focus == PaneFocus::Right
            && self.windows[self.active_window].surface == UiSurface::AgentChat
    }

    fn visual_mode_active(&self) -> bool {
        self.visual_cursor.is_some() || self.visual_selection.is_some()
    }

    fn chat_message_is_folded(&self, key: &ChatMessageFoldKey) -> bool {
        let folds = self.active_chat_fold_state();
        chat_message_is_folded(
            key,
            folds.fold_all_messages,
            &folds.folded_messages,
            &folds.unfolded_messages,
        )
    }

    fn fold_all_chat_messages(&mut self, right_content: &str) {
        let folds = self.active_chat_fold_state_mut();
        folds.fold_all_messages = true;
        folds.folded_messages.clear();
        folds.unfolded_messages.clear();
        self.clamp_right_scroll(right_content);
    }

    fn unfold_all_chat_messages(&mut self, right_content: &str) {
        let folds = self.active_chat_fold_state_mut();
        folds.fold_all_messages = false;
        folds.folded_messages.clear();
        folds.unfolded_messages.clear();
        self.clamp_right_scroll(right_content);
    }

    fn set_visual_chat_message_folded(&mut self, right_content: &str, folded: bool) {
        let Some(key) = self.right_visual_message_key(right_content) else {
            return;
        };
        self.set_chat_message_fold_state(key.clone(), folded);
        self.clamp_right_scroll(right_content);
        self.set_right_visual_cursor_to_message_key(right_content, &key);
    }

    fn toggle_visual_chat_message_fold(&mut self, right_content: &str) {
        let Some(key) = self.right_visual_message_key(right_content) else {
            return;
        };
        let folded = !self.chat_message_is_folded(&key);
        self.set_chat_message_fold_state(key.clone(), folded);
        self.clamp_right_scroll(right_content);
        self.set_right_visual_cursor_to_message_key(right_content, &key);
    }

    fn set_chat_message_fold_state(&mut self, key: ChatMessageFoldKey, folded: bool) {
        let folds = self.active_chat_fold_state_mut();
        if folds.fold_all_messages {
            if folded {
                folds.unfolded_messages.remove(&key);
            } else {
                folds.unfolded_messages.insert(key);
            }
        } else if folded {
            folds.folded_messages.insert(key);
        } else {
            folds.folded_messages.remove(&key);
        }
    }

    fn right_visual_message_key(&self, right_content: &str) -> Option<ChatMessageFoldKey> {
        let (pane, point) = self
            .visual_selection
            .as_ref()
            .map(|selection| (selection.pane, selection.cursor))
            .or_else(|| {
                self.visual_cursor
                    .as_ref()
                    .map(|cursor| (cursor.pane, cursor.point))
            })?;
        if pane != PaneFocus::Right {
            return None;
        }
        let visible_lines = self.visible_right_content_lines(right_content);
        let row = point.row.min(visible_lines.len().saturating_sub(1));
        visible_lines
            .get(row)
            .and_then(|line| line.message_key.clone())
    }

    fn set_right_visual_cursor_to_message_key(
        &mut self,
        right_content: &str,
        key: &ChatMessageFoldKey,
    ) {
        let visible_lines = self.visible_right_content_lines(right_content);
        let Some(row) = visible_lines
            .iter()
            .position(|line| line.message_key.as_ref() == Some(key))
        else {
            return;
        };
        let visible_right_content = chat_display_lines_to_string(&visible_lines);
        self.set_right_visual_cursor_row(&visible_right_content, row);
    }

    fn clamp_right_scroll(&mut self, right_content: &str) {
        let (inner_width, inner_height) = self.right_inner_size();
        let folds = self.active_chat_fold_state();
        self.right_scroll_rows = self.right_scroll_rows.min(max_scroll_rows(
            right_content,
            inner_width,
            inner_height,
            folds.fold_all_messages,
            &folds.folded_messages,
            &folds.unfolded_messages,
        ));
    }

    fn active_chat_fold_state(&self) -> &ChatFoldState {
        &self.windows[self.active_window].folds
    }

    fn active_chat_fold_state_mut(&mut self) -> &mut ChatFoldState {
        &mut self.windows[self.active_window].folds
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

    fn open_control_selection_in_insert_mode(&mut self) {
        self.open_control_selection();
        if self.focus == PaneFocus::Right {
            self.mode = UiMode::Insert;
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

    fn restore_control_selection(&mut self, agent_id: Option<&AgentId>) {
        if let Some(agent_id) = agent_id
            && let Some(position) = self
                .visible_agent_indices()
                .iter()
                .position(|index| self.agents[*index].id == *agent_id)
        {
            self.control_selected = position + 1;
            return;
        }
        self.clamp_control_selection();
    }

    fn visible_agent_indices(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        for section in LEFT_PANE_AGENT_SECTIONS {
            indices.extend(self.agents.iter().enumerate().filter_map(|(index, agent)| {
                (!agent.hidden && self.agent_left_pane_section(agent) == section).then_some(index)
            }));
        }
        indices
    }

    fn agent_patch_state(&self, agent: &AgentListEntry) -> AgentListPatchState {
        self.agent_patch_states
            .get(&agent.id)
            .copied()
            .unwrap_or_else(AgentListPatchState::default_for_agent)
    }

    fn agent_left_pane_section(&self, agent: &AgentListEntry) -> LeftPaneAgentSection {
        let id = agent.id.as_str();
        if id.starts_with("review-") {
            if self.review_no_findings.contains(&agent.id) {
                LeftPaneAgentSection::Reviewed
            } else {
                LeftPaneAgentSection::Reviewing
            }
        } else if id == "linearize" || id.starts_with("linearize-") {
            LeftPaneAgentSection::Linearize
        } else if id == "read" || id.starts_with("read-") || id.starts_with("reads-") {
            LeftPaneAgentSection::Reads
        } else {
            let state = self.agent_patch_state(agent);
            if state.closed {
                LeftPaneAgentSection::ClosedPatches
            } else if !state.has_user_message {
                LeftPaneAgentSection::NewPatches
            } else if agent.ready {
                LeftPaneAgentSection::ReadyPatches
            } else {
                LeftPaneAgentSection::WorkingPatches
            }
        }
    }

    fn agent_ready_visible(&self, agent: &AgentListEntry) -> bool {
        agent.ready && self.agent_allows_ready_highlight(agent)
    }

    fn agent_allows_ready_highlight(&self, agent: &AgentListEntry) -> bool {
        !matches!(
            self.agent_left_pane_section(agent),
            LeftPaneAgentSection::ClosedPatches
                | LeftPaneAgentSection::NewPatches
                | LeftPaneAgentSection::Reviewing
                | LeftPaneAgentSection::Reviewed
        )
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
        let inner_width = usize::from(self.layout().left_width.saturating_sub(2).max(1));
        let target = self
            .left_pane_lines(inner_width)
            .get(list_row)
            .and_then(|line| line.click_target.clone())?;
        self.select_left_pane_click_target(&target);
        Some(target)
    }

    fn select_left_pane_click_target(&mut self, target: &LeftPaneClickTarget) {
        match target {
            LeftPaneClickTarget::Command => {
                self.control_selected = 0;
            }
            LeftPaneClickTarget::Agent(agent_id) => {
                if let Some(position) = self
                    .visible_agent_indices()
                    .iter()
                    .position(|index| self.agents[*index].id == *agent_id)
                {
                    self.control_selected = position + 1;
                }
            }
        }
    }

    fn action_agent_id(&self) -> Option<AgentId> {
        self.control_selected_agent_id()
            .or_else(|| self.selected_agent.clone())
    }

    fn control_cursor_row(&self) -> u16 {
        let target = if self.control_selected == 0 {
            LeftPaneClickTarget::Command
        } else {
            let Some(agent_id) = self.control_selected_agent_id() else {
                return 2;
            };
            LeftPaneClickTarget::Agent(agent_id)
        };
        let inner_width = usize::from(self.layout().left_width.saturating_sub(2).max(1));
        self.left_pane_lines(inner_width)
            .iter()
            .position(|line| line.control_target.as_ref() == Some(&target))
            .map(|row| (row + 2).min(usize::from(u16::MAX)) as u16)
            .unwrap_or(2)
    }

    fn right_cursor_position_with_cursor(
        &self,
        right_content: &str,
        cursor_column: Option<usize>,
    ) -> (u16, u16) {
        let layout = self.layout();
        let inner_width = layout.right_width.saturating_sub(2).max(1);
        let Some((history, prompt)) = split_chat_prompt(right_content) else {
            return (2, layout.left_width.saturating_add(2));
        };
        let previous_rows = if history.is_empty() {
            0
        } else {
            visual_block_row_count(history, inner_width).min(usize::from(u16::MAX)) as u16
        };
        let prompt_chars = prompt.chars().count();
        let cursor_chars = cursor_column.unwrap_or(prompt_chars).min(prompt_chars);
        let (prompt_row, prompt_column) =
            visual_text_cursor_position(prompt, cursor_chars, inner_width);
        let row = 2_u16
            .saturating_add(previous_rows)
            .saturating_add(prompt_row);
        let column = layout
            .left_width
            .saturating_add(2)
            .saturating_add(prompt_column);
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

    fn render_status_line(&self) -> String {
        if let Some(selection) = &self.visual_selection {
            return format!(
                "mode=visual-{} focus={} window={}/{}",
                selection.mode.as_str(),
                self.focus.as_str(),
                self.active_window + 1,
                self.windows.len()
            );
        }

        if self.visual_cursor.is_some() {
            return format!(
                "mode=visual-cursor focus={} window={}/{}",
                self.focus.as_str(),
                self.active_window + 1,
                self.windows.len()
            );
        }

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
    fn as_str(&self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Insert => "insert",
            Self::Prompt => "prompt",
        }
    }
}

impl VisualSelectionMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Character => "char",
            Self::Line => "line",
            Self::Block => "block",
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

fn detailed_agent_row(agent: &AgentListEntry, selected: bool, ready: bool) -> String {
    let mut row = String::new();
    row.push(if selected { '>' } else { ' ' });
    let (primary, secondary) = agent_list_labels(agent);
    row.push_str(primary);
    row.push(' ');
    row.push_str(secondary);
    row.push_str("  working: ");
    row.push_str(&agent.feature);
    if ready {
        row.push_str("  READY");
    }
    row
}

fn compact_agent_row(agent: &AgentListEntry, selected: bool, width: usize, ready: bool) -> String {
    let prefix = if selected { ">" } else { " " };
    let status = if ready { " READY" } else { "" };
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

fn content_lines(content: &str) -> Vec<String> {
    if content.is_empty() {
        vec![String::new()]
    } else {
        content.lines().map(str::to_string).collect()
    }
}

fn visual_row_range(
    selection: &VisualSelection,
    line_count: usize,
) -> std::ops::RangeInclusive<usize> {
    if line_count == 0 {
        return 0..=0;
    }
    let start = selection
        .anchor
        .row
        .min(selection.cursor.row)
        .min(line_count - 1);
    let end = selection
        .anchor
        .row
        .max(selection.cursor.row)
        .min(line_count - 1);
    start..=end
}

fn visual_column_range(
    selection: &VisualSelection,
    row: usize,
    line_len: usize,
    pane_width: usize,
    force_line: bool,
) -> Option<(usize, usize)> {
    if force_line || selection.mode == VisualSelectionMode::Line {
        return Some((0, pane_width.saturating_sub(1)));
    }
    let line_end = line_len.saturating_sub(1);
    match selection.mode {
        VisualSelectionMode::Block => {
            let start = selection.anchor.column.min(selection.cursor.column);
            let end = selection
                .anchor
                .column
                .max(selection.cursor.column)
                .min(line_end);
            Some((start.min(line_end), end))
        }
        VisualSelectionMode::Character => {
            let (top, bottom) = ordered_visual_points(selection);
            if row == top.row && row == bottom.row {
                Some((top.column.min(line_end), bottom.column.min(line_end)))
            } else if row == top.row {
                Some((top.column.min(line_end), line_end))
            } else if row == bottom.row {
                Some((0, bottom.column.min(line_end)))
            } else {
                Some((0, line_end))
            }
        }
        VisualSelectionMode::Line => Some((0, pane_width.saturating_sub(1))),
    }
}

fn ordered_visual_points(selection: &VisualSelection) -> (VisualPoint, VisualPoint) {
    if selection.anchor.row < selection.cursor.row
        || (selection.anchor.row == selection.cursor.row
            && selection.anchor.column <= selection.cursor.column)
    {
        (selection.anchor, selection.cursor)
    } else {
        (selection.cursor, selection.anchor)
    }
}

fn extract_visual_text(lines: &[String], selection: &VisualSelection, force_line: bool) -> String {
    let rows = visual_row_range(selection, lines.len()).collect::<Vec<_>>();
    if force_line || selection.mode == VisualSelectionMode::Line {
        return rows
            .into_iter()
            .filter_map(|row| lines.get(row))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
    }

    rows.into_iter()
        .filter_map(|row| {
            let line = lines.get(row)?;
            let (start, end) =
                visual_column_range(selection, row, line.chars().count(), usize::MAX, false)?;
            Some(slice_chars(line, start, end))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn slice_chars(line: &str, start: usize, end: usize) -> String {
    line.chars()
        .skip(start)
        .take(end.saturating_sub(start).saturating_add(1))
        .collect()
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        encoded.push(TABLE[(b0 >> 2) as usize] as char);
        encoded.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

fn write_system_clipboard(text: &str) -> bool {
    clipboard_commands()
        .into_iter()
        .any(|(program, args)| run_clipboard_command(program, &args, text))
}

fn clipboard_commands() -> Vec<(&'static str, Vec<&'static str>)> {
    let mut commands = Vec::new();
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        commands.push(("wl-copy", Vec::new()));
    }
    if std::env::var_os("DISPLAY").is_some() {
        commands.push(("xclip", vec!["-selection", "clipboard"]));
        commands.push(("xsel", vec!["--clipboard", "--input"]));
    }
    if std::env::var_os("TMUX").is_some() {
        commands.push(("tmux", vec!["load-buffer", "-w", "-"]));
    }
    #[cfg(target_os = "macos")]
    commands.push(("pbcopy", Vec::new()));
    #[cfg(target_os = "windows")]
    commands.push(("clip.exe", Vec::new()));
    commands
}

fn run_clipboard_command(program: &str, args: &[&str], text: &str) -> bool {
    let Ok(mut child) = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };

    let Some(mut stdin) = child.stdin.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return false;
    };
    if stdin.write_all(text.as_bytes()).is_err() {
        drop(stdin);
        let _ = child.kill();
        let _ = child.wait();
        return false;
    }
    drop(stdin);
    child.wait().is_ok_and(|status| status.success())
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

fn visual_row_count(line: &str, width: u16) -> usize {
    let width = usize::from(width.max(1));
    let len = line.chars().count().min(usize::from(u16::MAX));
    visual_line_row_count(len, width)
}

fn visual_line_row_count(len: usize, width: usize) -> usize {
    len.saturating_sub(1)
        .checked_div(width)
        .unwrap_or(0)
        .saturating_add(1)
}

fn visual_block_row_count(text: &str, width: u16) -> usize {
    text.split('\n')
        .map(|line| visual_row_count(line, width))
        .sum()
}

fn visual_text_cursor_position(text: &str, cursor_chars: usize, width: u16) -> (u16, u16) {
    let width = usize::from(width.max(1));
    let mut row = 0_usize;
    let mut line_chars = 0_usize;
    let mut chars = text.chars();
    for _ in 0..cursor_chars {
        let Some(ch) = chars.next() else {
            break;
        };
        if ch == '\n' {
            row = row.saturating_add(visual_line_row_count(line_chars, width));
            line_chars = 0;
            continue;
        }
        line_chars = line_chars.saturating_add(1);
    }
    let at_text_end = chars.next().is_none();
    let (line_rows, column) = if at_text_end && line_chars > 0 && line_chars.is_multiple_of(width) {
        (line_chars / width - 1, width.saturating_sub(1))
    } else {
        (line_chars / width, line_chars % width)
    };
    row = row.saturating_add(line_rows);
    (
        row.min(usize::from(u16::MAX)) as u16,
        column.min(usize::from(u16::MAX)) as u16,
    )
}

#[cfg(test)]
mod visual_cursor_tests {
    use super::*;

    #[test]
    fn cursor_at_full_line_end_stays_on_last_visible_cell() {
        assert_eq!(visual_text_cursor_position("abcd", 4, 4), (0, 3));
        assert_eq!(visual_text_cursor_position("abcde", 4, 4), (1, 0));
        assert_eq!(visual_text_cursor_position("abcde", 5, 4), (1, 1));
        assert_eq!(visual_text_cursor_position("abcd\n", 5, 4), (1, 0));
        assert_eq!(visual_text_cursor_position("abcdefgh", 8, 4), (1, 3));
    }
}

fn cursor_char_count(text: &str, cursor: usize) -> usize {
    text.char_indices()
        .take_while(|(index, _)| *index < cursor)
        .count()
}

pub(crate) fn chat_content_from_transcript(chat_buffer: &str, transcript: &[String]) -> String {
    let mut content = String::new();
    for message in transcript {
        content.push_str(message);
        content.push(CHAT_TRANSCRIPT_MESSAGE_SEPARATOR);
        content.push('\n');
    }
    content.push_str("chat> ");
    content.push_str(chat_buffer);
    content
}

fn visible_content_lines(
    content: &str,
    width: u16,
    height: u16,
    scroll_rows: usize,
    fold_all: bool,
    folded_messages: &BTreeSet<ChatMessageFoldKey>,
    unfolded_messages: &BTreeSet<ChatMessageFoldKey>,
) -> Vec<ChatDisplayLine> {
    let height = usize::from(height);
    let Some((history, prompt)) = split_chat_prompt(content) else {
        return tail_visible_lines(&plain_display_lines(content), width, height, scroll_rows);
    };

    let history =
        chat_history_display_lines(history, width, fold_all, folded_messages, unfolded_messages);
    let prompt_rows = visual_block_row_count(prompt, width);
    let history_height = height.saturating_sub(prompt_rows).max(1);
    let mut visible_history = tail_visible_lines(&history, width, history_height, scroll_rows);
    let prompt_lines = plain_display_lines(prompt);
    if visible_history.is_empty() {
        prompt_lines
    } else {
        visible_history.extend(prompt_lines);
        visible_history
    }
}

fn max_scroll_rows(
    content: &str,
    width: u16,
    height: u16,
    fold_all: bool,
    folded_messages: &BTreeSet<ChatMessageFoldKey>,
    unfolded_messages: &BTreeSet<ChatMessageFoldKey>,
) -> usize {
    let height = usize::from(height);
    let Some((history, prompt)) = split_chat_prompt(content) else {
        return if content.is_empty() {
            0
        } else {
            visual_block_row_count(content, width).saturating_sub(height)
        };
    };

    if history.is_empty() {
        return 0;
    }

    let history =
        chat_history_display_lines(history, width, fold_all, folded_messages, unfolded_messages);
    let prompt_rows = visual_block_row_count(prompt, width);
    let history_height = height.saturating_sub(prompt_rows).max(1);
    display_lines_row_count(&history, width).saturating_sub(history_height)
}

fn split_chat_prompt(content: &str) -> Option<(&str, &str)> {
    if content.starts_with("chat> ") {
        return Some(("", content));
    }
    let prompt_start = content.rfind("\nchat> ")?;
    Some((&content[..prompt_start], &content[prompt_start + 1..]))
}

fn tail_visible_lines(
    lines: &[ChatDisplayLine],
    width: u16,
    height: usize,
    scroll_rows: usize,
) -> Vec<ChatDisplayLine> {
    if lines.is_empty() || height == 0 {
        return Vec::new();
    }

    let rows_to_skip = scroll_rows.min(
        lines
            .iter()
            .map(|line| visual_row_count(&line.text, width))
            .sum::<usize>()
            .saturating_sub(height),
    );
    let mut visible = Vec::new();
    let mut skipped_rows = 0_usize;
    let mut used_rows = 0_usize;
    for line in lines.iter().rev() {
        let rows = visual_row_count(&line.text, width);
        if skipped_rows.saturating_add(rows) <= rows_to_skip {
            skipped_rows = skipped_rows.saturating_add(rows);
            continue;
        }
        if visible.is_empty() || used_rows.saturating_add(rows) <= height {
            visible.push(line.clone());
            used_rows = used_rows.saturating_add(rows);
        } else {
            break;
        }
    }
    visible.reverse();
    visible
}

fn plain_display_lines(content: &str) -> Vec<ChatDisplayLine> {
    content_lines(content)
        .into_iter()
        .map(|text| ChatDisplayLine {
            text,
            message_key: None,
        })
        .collect()
}

fn chat_display_lines_to_string(lines: &[ChatDisplayLine]) -> String {
    lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn display_lines_row_count(lines: &[ChatDisplayLine], width: u16) -> usize {
    lines
        .iter()
        .map(|line| visual_row_count(&line.text, width))
        .sum()
}

fn chat_history_display_lines(
    history: &str,
    width: u16,
    fold_all: bool,
    folded_messages: &BTreeSet<ChatMessageFoldKey>,
    unfolded_messages: &BTreeSet<ChatMessageFoldKey>,
) -> Vec<ChatDisplayLine> {
    if history.is_empty() {
        return Vec::new();
    }

    let separator = chat_separator_line(width);
    let mut lines = Vec::new();
    for message in chat_history_messages(history) {
        let Some(first_line) = message.lines.first() else {
            continue;
        };
        if is_user_prompt_line(first_line) && !lines.is_empty() {
            lines.push(ChatDisplayLine {
                text: String::new(),
                message_key: None,
            });
            lines.push(ChatDisplayLine {
                text: separator.clone(),
                message_key: None,
            });
            lines.push(ChatDisplayLine {
                text: String::new(),
                message_key: None,
            });
        }
        let folded =
            chat_message_is_folded(&message.key, fold_all, folded_messages, unfolded_messages);
        for (index, line) in message.lines.iter().enumerate() {
            if folded && index > 0 {
                continue;
            }
            lines.push(ChatDisplayLine {
                text: line.to_string(),
                message_key: Some(message.key.clone()),
            });
        }
    }
    lines
}

fn chat_message_is_folded(
    key: &ChatMessageFoldKey,
    fold_all: bool,
    folded_messages: &BTreeSet<ChatMessageFoldKey>,
    unfolded_messages: &BTreeSet<ChatMessageFoldKey>,
) -> bool {
    if fold_all {
        !unfolded_messages.contains(key)
    } else {
        folded_messages.contains(key)
    }
}

fn chat_history_messages(history: &str) -> Vec<ChatHistoryMessage> {
    let texts = if history.contains(CHAT_TRANSCRIPT_MESSAGE_SEPARATOR) {
        marked_chat_history_message_texts(history)
    } else {
        inferred_chat_history_message_texts(history)
    };
    keyed_chat_history_messages(texts)
}

fn marked_chat_history_message_texts(history: &str) -> Vec<String> {
    history
        .split(CHAT_TRANSCRIPT_MESSAGE_SEPARATOR)
        .filter_map(|message| {
            let message = message.strip_prefix('\n').unwrap_or(message);
            (!message.is_empty()).then(|| message.to_string())
        })
        .collect()
}

fn inferred_chat_history_message_texts(history: &str) -> Vec<String> {
    let mut messages = Vec::new();
    let mut current = Vec::new();
    let mut previous_line_was_user_prompt = false;
    for line in history.lines() {
        let starts_new_message = current.is_empty()
            || is_user_prompt_line(line)
            || (previous_line_was_user_prompt && !line.is_empty())
            || is_labeled_chat_message_start(line);
        if starts_new_message && !current.is_empty() {
            messages.push(current.join("\n"));
            current.clear();
        }
        current.push(line.to_string());
        previous_line_was_user_prompt = is_user_prompt_line(line);
    }
    if !current.is_empty() {
        messages.push(current.join("\n"));
    }
    messages
}

fn keyed_chat_history_messages(texts: Vec<String>) -> Vec<ChatHistoryMessage> {
    let mut occurrences = BTreeMap::<String, usize>::new();
    texts
        .into_iter()
        .filter_map(|text| {
            let lines = content_lines(&text);
            let first_line = lines.first()?.clone();
            let occurrence = occurrences.entry(first_line.clone()).or_default();
            let key = ChatMessageFoldKey {
                first_line,
                occurrence: *occurrence,
            };
            *occurrence = (*occurrence).saturating_add(1);
            Some(ChatHistoryMessage { key, lines })
        })
        .collect()
}

fn is_labeled_chat_message_start(line: &str) -> bool {
    line.starts_with("command-agent: ")
        || line.starts_with("codex: ")
        || line.starts_with("codex error: ")
        || line.starts_with("fixture reply: ")
        || line.starts_with("orchestrator:")
        || line.starts_with("review: ")
        || line.starts_with("work-leaf: ")
}

fn chat_separator_line(width: u16) -> String {
    "-".repeat(usize::from(width.max(1)))
}

fn is_user_prompt_line(line: &str) -> bool {
    line.starts_with("user: ")
        || line.starts_with("chat> ")
        || line.starts_with("work-leaf> ")
        || prompt_prefix(line).is_some_and(|prefix| prefix.starts_with("user-"))
}

fn prompt_prefix(line: &str) -> Option<&str> {
    let (prefix, _) = line.split_once("> ")?;
    (!prefix.is_empty()
        && prefix
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
    .then_some(prefix)
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
                buffer.get(column, 4).modifier.contains(Modifier::REVERSED),
                "column {column} on the ready agent row should be reversed"
            );
        }
    }

    #[test]
    fn left_pane_groups_patch_rows_by_lifecycle() {
        let mut ui = TerminalUi::new(120, 24);
        let closed_id = AgentId::new("user-1").expect("test agent id is valid");
        let new_id = AgentId::new("user-2").expect("test agent id is valid");
        let ready_id = AgentId::new("user-3").expect("test agent id is valid");
        let working_id = AgentId::new("user-4").expect("test agent id is valid");
        let reviewing_id = AgentId::new("review-user-4").expect("test agent id is valid");
        let reviewed_id = AgentId::new("review-user-5").expect("test agent id is valid");
        let read_id = AgentId::new("read-user-4").expect("test agent id is valid");
        let linearize_id = AgentId::new("linearize").expect("test agent id is valid");
        ui.add_agent(AgentListEntry::new(
            closed_id.clone(),
            "closed parser CLOSED",
        ));
        ui.set_agent_patch_lifecycle(&closed_id, true, true)
            .expect("closed patch agent is registered");
        ui.add_agent(AgentListEntry::new(new_id.clone(), "new parser").with_ready(true));
        ui.set_agent_patch_lifecycle(&new_id, false, false)
            .expect("new patch agent is registered");
        ui.add_agent(AgentListEntry::new(ready_id.clone(), "ready parser").with_ready(true));
        ui.set_agent_patch_lifecycle(&ready_id, true, false)
            .expect("ready patch agent is registered");
        ui.add_agent(AgentListEntry::new(working_id.clone(), "working parser"));
        ui.set_agent_patch_lifecycle(&working_id, true, false)
            .expect("working patch agent is registered");
        ui.add_agent(AgentListEntry::new(reviewing_id, "reviewing parser").with_ready(true));
        ui.add_agent(AgentListEntry::new(reviewed_id.clone(), "reviewed parser").with_ready(true));
        ui.set_agent_review_no_findings(&reviewed_id, true)
            .expect("reviewed agent is registered");
        ui.add_agent(AgentListEntry::new(read_id, "read parser"));
        ui.add_agent(AgentListEntry::new(linearize_id, "linearize"));
        ui.select_agent(&working_id)
            .expect("test patch agent is registered");

        let left_pane = ui.render_left_pane();

        let command = left_pane
            .find("[command]")
            .expect("command section renders");
        let closed = left_pane.find("[closed]").expect("closed section renders");
        let new = left_pane.find("[new]").expect("new section renders");
        let ready = left_pane.find("[ready]").expect("ready section renders");
        let working = left_pane
            .find("[working]")
            .expect("working section renders");
        let reviewing = left_pane
            .find("[reviewing]")
            .expect("reviewing section renders");
        let reviewed = left_pane
            .find("[reviewed]")
            .expect("reviewed section renders");
        let reads = left_pane.find("[reads]").expect("read section renders");
        let linearize = left_pane
            .find("[linearize]")
            .expect("linearize section renders");
        assert!(command < closed);
        assert!(closed < new);
        assert!(new < ready);
        assert!(ready < working);
        assert!(working < reviewing);
        assert!(reviewing < reviewed);
        assert!(reviewed < reads);
        assert!(reads < linearize);
        assert!(left_pane.contains(" closed parser CLOSED user-1  working: closed parser CLOSED"));
        assert!(left_pane.contains(" new parser user-2  working: new parser"));
        assert!(!left_pane.contains("new parser  READY"));
        assert!(left_pane.contains(" ready parser user-3  working: ready parser  READY"));
        assert!(left_pane.contains(">working parser user-4  working: working parser"));
        assert!(left_pane.contains(" review-user-4 reviewing parser  working: reviewing parser"));
        assert!(left_pane.contains(" review-user-5 reviewed parser  working: reviewed parser"));
        assert!(!left_pane.contains("reviewing parser  READY"));
        assert!(!left_pane.contains("reviewed parser  READY"));
    }

    #[test]
    fn lifecycle_changes_keep_control_selection_on_same_agent() {
        let mut ui = TerminalUi::new(120, 24);
        let first_id = AgentId::new("user-1").expect("test agent id is valid");
        let second_id = AgentId::new("user-2").expect("test agent id is valid");
        ui.add_agent(AgentListEntry::new(first_id.clone(), "first"));
        ui.set_agent_patch_lifecycle(&first_id, true, false)
            .expect("first patch agent is registered");
        ui.add_agent(AgentListEntry::new(second_id.clone(), "second"));
        ui.set_agent_patch_lifecycle(&second_id, true, false)
            .expect("second patch agent is registered");
        ui.select_agent(&second_id)
            .expect("second patch agent is registered");

        ui.set_agent_ready_state(&second_id, true)
            .expect("second patch agent is registered");

        let ready_left_pane = ui
            .render_left_pane()
            .replace("\u{1b}[7m", "")
            .replace("\u{1b}[0m", "");
        assert!(ready_left_pane.contains("[ready]\n>second user-2  working: second  READY"));
        assert!(ready_left_pane.contains("[working]\n first user-1  working: first"));

        ui.set_agent_patch_lifecycle(&second_id, true, true)
            .expect("second patch agent is registered");

        let closed_left_pane = ui
            .render_left_pane()
            .replace("\u{1b}[7m", "")
            .replace("\u{1b}[0m", "");
        assert!(closed_left_pane.contains("[closed]\n>second user-2  working: second"));
        assert!(closed_left_pane.contains("[working]\n first user-1  working: first"));
    }
}
