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
fn scripted_harness_left_pane_groups_command_and_patch_chats() {
    let harness = UiHarness::new(80, 24);
    let left_pane = strip_ansi(&harness.ui().render_left_pane());

    let command = left_pane
        .find("[command]")
        .expect("command section renders");
    let patches = left_pane.find("[patches]").expect("patch section renders");

    assert!(command < patches);
    assert!(left_pane.contains("[command]\n  work-leaf  command"));
    assert!(left_pane.contains("[patches]\n>parser user-1  working: parser  READY"));
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
fn scripted_harness_empty_chat_escape_enters_command_mode_and_forks_selected_agent() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    harness.handle_byte(b'i');
    assert_eq!(harness.ui().mode(), UiMode::Insert);

    harness.handle_byte(27);
    assert_eq!(harness.ui().mode(), UiMode::Command);
    harness.handle_byte(b'f');

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| { line.contains("ForkAgent") && line.contains("user-1") })
    );
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
fn scripted_harness_blocked_linearize_renders_command_message_from_chat() {
    let mut harness = UiHarness::new(100, 24);

    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );

    harness.handle_bytes(b":linearize\n");

    assert!(harness.ui().selected_agent().is_none());
    let frame = harness.render_frame();
    assert!(frame.contains("work-leaf: reviewed patch chats must be classified"));
    assert!(frame.contains("Use force-linearize to bypass."));
}

#[test]
fn scripted_harness_ctrl_c_discards_prompt_history_draft() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":review\n:draft command\x1b[A");
    assert!(harness.render_frame().contains(":review"));

    assert!(harness.handle_byte(3));
    assert!(!harness.is_quit());
    assert_eq!(harness.ui().mode(), UiMode::Command);

    harness.handle_bytes(b"\x1b[B\n");

    assert!(
        !harness
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
    assert!(harness.render_frame().ends_with("\u{1b}[5;2H"));

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
fn scripted_harness_visual_mode_yanks_right_pane_line_to_clipboard() {
    let mut harness = UiHarness::new(80, 24);
    let expected = "Esc command, i insert, : prompt, Ctrl-W h/j/k/l focus, , toggle right, q quit";

    harness.handle_bytes(&[23, b'l']);
    harness.handle_byte(b'V');

    assert!(harness.ui().visual_selection_active());
    assert!(
        harness
            .render_frame()
            .contains("mode=visual-line focus=right")
    );

    harness.handle_byte(b'Y');

    assert_eq!(harness.ui().copied_text(), Some(expected));
    let frame = harness.render_frame();
    assert!(frame.starts_with("\u{1b}]52;c;"));
    assert!(frame.contains("copied "));
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
fn scripted_harness_summarizes_noisy_chat_title_from_first_inserted_prompt() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":new\n");
    harness.handle_bytes(b"it looks like that we there have been a bad regression chat name for patch agents is not created by the system agent but it has to summarize it\n");

    let named_left_pane = harness.ui().render_left_pane();
    assert!(named_left_pane.contains(
        ">bad-regression-chat-name-patch-agents user-3  working: bad-regression-chat-name-patch-agents"
    ));
    assert!(!named_left_pane.contains("it-looks-like"));
}

#[test]
fn scripted_harness_caps_chat_title_for_left_pane_space() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":new\n");
    harness.handle_bytes(
        b"fix authentication authorization migration workflow regressions before release\n",
    );

    let named_left_pane = harness.ui().render_left_pane();
    assert!(named_left_pane.contains(
        ">fix-authentication-authorization user-3  working: fix-authentication-authorization"
    ));
    assert!(!named_left_pane.contains("migration-workflow-regressions"));
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
fn scripted_harness_shift_enter_keeps_insert_prompt_multiline_until_plain_enter() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"ifirst\x1b[27;2;13~second");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        !harness
            .transcript()
            .iter()
            .any(|line| line.starts_with("user-1>"))
    );
    let frame = harness.render_frame();
    assert!(frame.contains("chat> first"));
    assert!(frame.contains("second"));

    harness.handle_byte(b'\n');

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> first\nsecond")
    );
}

#[test]
fn scripted_harness_kitty_shift_enter_key_press_keeps_insert_prompt_multiline_until_plain_enter() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"ifirst\x1b[13;2:1usecond");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        !harness
            .transcript()
            .iter()
            .any(|line| line.starts_with("user-1>"))
    );
    let frame = harness.render_frame();
    assert!(frame.contains("chat> first"));
    assert!(frame.contains("second"));

    harness.handle_byte(b'\n');

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> first\nsecond")
    );
}

#[test]
fn scripted_harness_terminal_line_feed_keeps_insert_prompt_multiline_until_carriage_return() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_terminal_bytes(b"ifirst\nsecond");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        !harness
            .transcript()
            .iter()
            .any(|line| line.starts_with("user-1>"))
    );
    let frame = harness.render_frame();
    assert!(frame.contains("chat> first"));
    assert!(frame.contains("second"));

    harness.handle_byte(b'\r');

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> first\nsecond")
    );
}

#[test]
fn scripted_harness_modified_f3_tilde_does_not_insert_line_break() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"ifirst\x1b[13;2~second\n");

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> firstsecond")
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
fn scripted_harness_gg_and_uppercase_g_jump_chat_history_edges() {
    let mut harness = UiHarness::new(80, 10);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_byte(b'i');
    for index in 0..12 {
        harness.handle_bytes(format!("message-{index:02}\n").as_bytes());
    }
    harness.handle_byte(27);

    let bottom_frame = harness.render_frame();
    assert!(!bottom_frame.contains("UI harness"));
    assert!(bottom_frame.contains("message-11"));

    harness.handle_bytes(b"gg");

    let top_frame = harness.render_frame();
    assert!(top_frame.contains("UI harness"));
    assert!(top_frame.contains("chat> "));
    assert!(!top_frame.contains("message-11"));

    harness.handle_byte(b'G');

    let bottom_again = harness.render_frame();
    assert!(!bottom_again.contains("UI harness"));
    assert!(bottom_again.contains("message-11"));
}

#[test]
fn scripted_harness_visual_cursor_scrolls_and_jumps_chat_history_edges() {
    let mut harness = UiHarness::new(80, 10);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_byte(b'i');
    for index in 0..12 {
        harness.handle_bytes(format!("message-{index:02}\n").as_bytes());
    }
    harness.handle_byte(27);
    harness.handle_byte(b'v');

    for _ in 0..32 {
        harness.handle_byte(b'k');
    }

    let top_frame = harness.render_frame();
    assert!(top_frame.contains("UI harness"));
    assert!(top_frame.contains("mode=visual-cursor focus=right"));

    harness.handle_byte(b'G');

    let bottom_frame = harness.render_frame();
    assert!(!bottom_frame.contains("UI harness"));
    assert!(bottom_frame.contains("message-11"));

    harness.handle_bytes(b"gg");

    let top_again = harness.render_frame();
    assert!(top_again.contains("UI harness"));
    assert!(top_again.contains("chat> "));
}

#[test]
fn scripted_harness_visual_selection_rebases_after_chat_history_jumps_and_scrolls() {
    let mut jump_harness = UiHarness::new(80, 10);

    jump_harness.handle_bytes(&[23, b'l']);
    jump_harness.handle_byte(b'i');
    for index in 0..12 {
        jump_harness.handle_bytes(format!("message-{index:02}\n").as_bytes());
    }
    jump_harness.handle_byte(27);
    jump_harness.handle_byte(b'V');
    jump_harness.handle_bytes(b"gg");
    jump_harness.handle_byte(b'Y');

    assert_eq!(jump_harness.ui().copied_text(), Some("UI harness"));

    let mut scroll_harness = UiHarness::new(80, 10);

    scroll_harness.handle_bytes(&[23, b'l']);
    scroll_harness.handle_byte(b'i');
    for index in 0..12 {
        scroll_harness.handle_bytes(format!("message-{index:02}\n").as_bytes());
    }
    scroll_harness.handle_byte(27);
    scroll_harness.handle_byte(b'V');
    for _ in 0..32 {
        scroll_harness.handle_byte(b'k');
    }
    scroll_harness.handle_byte(b'Y');

    assert_eq!(scroll_harness.ui().copied_text(), Some("UI harness"));
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
fn scripted_harness_chat_cursor_stays_on_prompt_after_full_width_history_line() {
    let mut harness = UiHarness::new(80, 24);
    harness.handle_bytes(&[23, b'l']);

    let inner_width = usize::from(harness.ui().layout().right_width.saturating_sub(2));
    let message = "x".repeat(inner_width - "user-1> ".chars().count());
    harness.handle_byte(b'i');
    harness.handle_bytes(message.as_bytes());
    harness.handle_byte(b'\n');

    let frame = harness.render_frame();
    assert!(frame.contains(&format!("user-1> {message}")));
    assert!(frame.contains("chat> "));
    assert!(frame.ends_with("\u{1b}[7;24H"));
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
