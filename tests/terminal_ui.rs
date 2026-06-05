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
    assert_eq!(ui.layout().right_surface, None);

    ui.handle_key(UiKey::Char(','));
    ui.handle_key(UiKey::CtrlW);
    ui.handle_key(UiKey::Char('l'));
    assert_eq!(ui.focus(), PaneFocus::Right);
    assert_eq!(ui.mode(), UiMode::Command);

    ui.handle_key(UiKey::Char('i'));
    assert_eq!(ui.mode(), UiMode::Command);

    ui.handle_key(UiKey::CtrlW);
    ui.handle_key(UiKey::Char('h'));
    assert_eq!(ui.focus(), PaneFocus::Left);

    ui.handle_key(UiKey::Char('t'));
    assert_eq!(ui.window_count(), 2);
    assert_eq!(ui.active_window(), 1);

    ui.handle_key(UiKey::Char('g'));
    ui.handle_key(UiKey::Char('T'));
    assert_eq!(ui.active_window(), 0);
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
