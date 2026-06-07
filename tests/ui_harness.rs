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
fn scripted_harness_bells_and_highlights_ready_chat_in_left_pane() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":new ready notification\n");
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-3")
    );
    assert!(!harness.render_frame().contains('\u{7}'));

    harness
        .mark_agent_ready("user-3")
        .expect("new fixture agent can become ready");

    let ready_frame = harness.render_frame();
    assert!(ready_frame.contains('\u{7}'));
    assert!(
        harness
            .ui()
            .render_left_pane()
            .contains("\u{1b}[7m>harness-agent user-3  working: harness-agent  READY\u{1b}[0m")
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
    assert!(
        named_left_pane.contains(">oauth redirect handler user-3  working: oauth redirect handler")
    );
    assert!(!named_left_pane.contains("harness-agent user-3"));

    harness.handle_bytes(b"add cookie coverage\n");

    let unchanged_left_pane = harness.ui().render_left_pane();
    assert!(
        unchanged_left_pane
            .contains(">oauth redirect handler user-3  working: oauth redirect handler")
    );
    assert!(!unchanged_left_pane.contains("cookie coverage user-3"));
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
