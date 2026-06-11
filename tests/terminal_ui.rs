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

    assert_eq!(ui.focus(), PaneFocus::Left);
    assert_eq!(ui.layout().left_width, 20);
    assert_eq!(ui.layout().right_width, 80);
    assert_eq!(ui.layout().right_surface, Some(UiSurface::AgentChat));

    ui.handle_key(UiKey::Char(','));

    assert_eq!(ui.focus(), PaneFocus::Right);
    assert_eq!(ui.layout().left_width, 0);
    assert_eq!(ui.layout().right_width, 100);
    assert_eq!(ui.layout().right_surface, Some(UiSurface::AgentChat));

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
    assert!(left.ends_with("\u{1b}[3;2H"));

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

    assert!(ui.render_screen("command chat").ends_with("\u{1b}[3;2H"));

    ui.handle_key(UiKey::Char('j'));
    assert!(ui.render_screen("command chat").ends_with("\u{1b}[5;2H"));
    assert_eq!(ui.selected_agent(), Some(&chat_a));
    assert_eq!(ui.focus(), PaneFocus::Left);
    assert!(ui.render_screen("agent chat").contains("chat-a"));

    ui.handle_key(UiKey::Char('j'));

    assert_eq!(ui.selected_agent(), Some(&chat_b));
    assert_eq!(ui.focus(), PaneFocus::Left);
    assert!(ui.render_screen("agent chat").contains("chat-b"));
}

#[test]
fn left_pane_navigation_follows_grouped_chat_section_order() {
    let mut ui = TerminalUi::new(100, 20);
    let review = AgentId::new("review-chat-a").unwrap();
    let patch = AgentId::new("chat-a").unwrap();
    let read = AgentId::new("read-chat-a").unwrap();
    ui.add_agent(AgentListEntry::new(review, "review parser"));
    ui.add_agent(AgentListEntry::new(read, "read parser"));
    ui.add_agent(AgentListEntry::new(patch.clone(), "parser"));

    ui.handle_key(UiKey::Char('j'));

    assert_eq!(ui.selected_agent(), Some(&patch));
    assert!(ui.render_screen("agent chat").contains("chat-a"));
}

#[test]
fn mouse_clicking_a_left_pane_agent_row_opens_that_agent_chat() {
    let mut ui = TerminalUi::new(100, 20);
    let chat_a = AgentId::new("chat-a").unwrap();
    let chat_b = AgentId::new("chat-b").unwrap();
    ui.add_agent(AgentListEntry::new(chat_a.clone(), "parser"));
    ui.add_agent(AgentListEntry::new(chat_b.clone(), "docs"));
    ui.select_agent(&chat_a).unwrap();

    ui.handle_key(UiKey::MouseClick { column: 4, row: 6 });

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
    assert!(ui.render_screen("command chat").ends_with("\u{1b}[5;2H"));
}

#[test]
fn visual_mode_yanks_left_pane_line_and_block_selections() {
    let mut ui = TerminalUi::new(120, 20);
    let chat_a = AgentId::new("chat-a").unwrap();
    let chat_b = AgentId::new("chat-b").unwrap();
    ui.add_agent(AgentListEntry::new(chat_a, "parser"));
    ui.add_agent(AgentListEntry::new(chat_b, "docs"));

    ui.handle_key(UiKey::Char('V'));
    assert!(ui.visual_selection_active());
    ui.handle_key(UiKey::Char('j'));

    let selected = ui.render_screen("right pane");
    assert!(selected.contains("mode=visual-line focus=left"));
    assert!(selected.contains("\u{1b}[7m> work-leaf  command"));

    ui.handle_key(UiKey::Char('y'));

    assert_eq!(ui.copied_text(), Some("[command]\n> work-leaf  command"));
    assert!(!ui.visual_selection_active());
    assert!(ui.render_screen("right pane").starts_with("\u{1b}]52;c;"));

    ui.handle_key(UiKey::Char('\u{16}'));
    ui.handle_key(UiKey::Char('j'));
    ui.handle_key(UiKey::Char('l'));
    ui.handle_key(UiKey::Char('l'));
    assert!(
        ui.render_screen("right pane")
            .contains("mode=visual-block focus=left")
    );
    ui.handle_key(UiKey::Char('y'));

    assert_eq!(ui.copied_text(), Some("[co\n> w"));
}

#[test]
fn visual_mode_v_enters_cursor_mode_before_selecting_text() {
    let mut ui = TerminalUi::new(120, 20);
    let chat_a = AgentId::new("chat-a").unwrap();
    let chat_b = AgentId::new("chat-b").unwrap();
    ui.add_agent(AgentListEntry::new(chat_a, "parser"));
    ui.add_agent(AgentListEntry::new(chat_b, "docs"));

    ui.handle_key(UiKey::Char('v'));

    assert!(!ui.visual_selection_active());
    assert!(
        ui.render_screen("right pane")
            .contains("mode=visual-cursor focus=left")
    );

    ui.handle_key(UiKey::Char('j'));
    assert!(!ui.visual_selection_active());
    assert!(
        ui.render_screen("right pane")
            .contains("mode=visual-cursor focus=left")
    );

    ui.handle_key(UiKey::Char('v'));
    assert!(ui.visual_selection_active());
    ui.handle_key(UiKey::Esc);

    ui.handle_key(UiKey::Char('v'));
    ui.handle_key(UiKey::Char('j'));
    ui.handle_key(UiKey::Char('j'));
    ui.handle_key(UiKey::Char('j'));
    ui.handle_key(UiKey::Char('V'));
    ui.handle_key(UiKey::Char('y'));

    assert_eq!(ui.copied_text(), Some(" parser chat-a"));
}

#[test]
fn raw_mode_screen_uses_crlf_so_frame_fills_terminal_width() {
    let ui = TerminalUi::new(80, 8);
    let rendered = ui.render_screen("command chat");

    assert!(rendered.contains("\r\n"));
    assert!(!rendered.contains(" \n"));
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
