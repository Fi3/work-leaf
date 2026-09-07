//! Test-only transcript adapter; no public harness/controller surface.
use super::*;

pub(crate) fn display_actual_controller_notice(lines: &[String]) -> bool {
    let mut harness = UiHarness::new(180, 30);
    harness.transcript = lines.to_vec();
    harness.handle_bytes(&[23, b'l', b'i']);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert_eq!(harness.ui().mode(), UiMode::Insert);
    harness.handle_bytes(b"inspect retained failure");
    let frame = harness.render_frame();
    assert!(frame.contains("inspect retained failure"));
    harness.handle_bytes(&[27]);
    assert_eq!(harness.ui().mode(), UiMode::Command);
    frame.contains("private preparation rejected:")
}
