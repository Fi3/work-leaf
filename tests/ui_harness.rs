use work_leaf::{PaneFocus, UiHarness, UiMode};

#[test]
fn scripted_harness_renders_full_width_crlf_frame() {
    let harness = UiHarness::new(80, 24);
    let rendered = harness.render_frame();
    let lines = rendered.split("\r\n").collect::<Vec<_>>();

    assert!(rendered.starts_with("\u{1b}[2J\u{1b}[H"));
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
fn scripted_harness_drives_ctrl_w_navigation_and_right_toggle() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert!(harness.render_frame().ends_with("\u{1b}[5;24H"));

    harness.handle_bytes(&[23, b'h']);
    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert!(harness.render_frame().ends_with("\u{1b}[3;2H"));

    harness.handle_byte(b',');
    assert_eq!(harness.ui().layout().right_surface, None);
    assert_eq!(harness.ui().focus(), PaneFocus::Left);

    harness.handle_byte(b',');
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
