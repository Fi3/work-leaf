use work_leaf::{PaneFocus, UiHarness};

#[test]
fn quality_visual_behavior() {
    let mut line = UiHarness::new(96, 22);
    line.handle_byte(b'V');
    line.handle_byte(b'j');
    assert!(has_visual_status(&line.render_frame(), "left", "line"));
    line.handle_byte(b'y');
    assert!(has_nonempty_copy_evidence(&line.render_frame()));

    let mut block = UiHarness::new(96, 22);
    block.handle_byte(22);
    block.handle_byte(b'j');
    block.handle_byte(b'l');
    assert!(has_visual_status(&block.render_frame(), "left", "block"));
    block.handle_byte(b'y');
    assert!(has_nonempty_copy_evidence(&block.render_frame()));

    let mut character = UiHarness::new(96, 22);
    character.handle_byte(b'v');
    if character.render_frame().contains("mode=visual-cursor") {
        character.handle_byte(b'v');
    }
    character.handle_byte(b'j');
    assert!(has_visual_status(&character.render_frame(), "left", "char"));
    character.handle_byte(b'y');
    assert!(has_nonempty_copy_evidence(&character.render_frame()));

    let mut right = UiHarness::new(84, 22);
    right.handle_bytes(&[23, b'l']);
    assert_eq!(right.ui().focus(), PaneFocus::Right);
    right.handle_byte(b'V');
    assert!(has_visual_status(&right.render_frame(), "right", "line"));
    right.handle_byte(b'Y');
    assert!(has_nonempty_copy_evidence(&right.render_frame()));
}

fn has_visual_status(frame: &str, focus: &str, selection: &str) -> bool {
    let focus_field = format!("focus={focus}");
    let selection_field = format!("selection={selection}");
    frame.lines().any(|line| {
        let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
        let generic = (fields.contains(&"mode=visual") || fields.contains(&"mode=visual-cursor"))
            && fields.contains(&focus_field.as_str())
            && fields
                .iter()
                .all(|field| !field.starts_with("selection=") || *field == selection_field);
        let legacy = match selection {
            "char" => {
                line.contains(&format!("mode=visual-char focus={focus}"))
                    || line.contains(&format!("mode=visual-character focus={focus}"))
            }
            "line" => line.contains(&format!("mode=visual-line focus={focus}")),
            "block" => line.contains(&format!("mode=visual-block focus={focus}")),
            _ => false,
        };
        generic || legacy
    })
}

fn has_nonempty_copy_evidence(frame: &str) -> bool {
    if frame
        .strip_prefix("\u{1b}]52;c;")
        .and_then(|suffix| suffix.split_once('\u{7}'))
        .is_some_and(|(payload, _)| !payload.is_empty())
    {
        return true;
    }

    frame.lines().rev().any(|line| {
        if line
            .rsplit_once("CopySelection(\"")
            .and_then(|(_, suffix)| suffix.split_once("\")"))
            .is_some_and(|(selection, _)| !selection.is_empty())
        {
            return true;
        }
        let words = line
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        if words.iter().any(|word| {
            ["empty", "error", "failed", "failure", "none", "not"]
                .iter()
                .any(|failure| word.eq_ignore_ascii_case(failure))
        }) {
            return false;
        }
        words.iter().enumerate().any(|(index, word)| {
            let action = ["copy", "copied", "yank", "yanked"]
                .iter()
                .any(|candidate| word.eq_ignore_ascii_case(candidate));
            action
                && words
                    .get(index + 1)
                    .and_then(|count| count.parse::<usize>().ok())
                    .is_some_and(|count| count > 0)
        })
    })
}
