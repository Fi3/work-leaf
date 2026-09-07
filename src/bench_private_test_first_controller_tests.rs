//! Actual pending-controller seams in the isolated active-v6 child process.
use super::*;
use serde_json::{Value, json};

pub(crate) fn exercise<B>(chat: CommandChat<B>, mode: &str) -> Value
where
    B: AgentBackend + Clone + Send + 'static,
{
    let mut controller = WorkLeafController::new(chat);
    let dependent = mode.ends_with("dependency");
    let cancelled = mode.contains("cancel");
    let id = if dependent {
        let dependency = controller.create_agent("independent prerequisite").unwrap();
        controller
            .create_agent(format!("--depends-on {dependency}"))
            .unwrap()
    } else {
        controller.create_agent("original task λ").unwrap()
    };
    if cancelled {
        controller.interrupt_agent(&id);
    }
    let expected_prompt = if dependent {
        "resolved dependent task λ"
    } else {
        "original task λ"
    };
    let revision = if dependent {
        // Both first-title and pending-dependency prompt changes use the real path.
        controller.send_message(&id, expected_prompt)
    } else {
        controller.apply_agent_title(&id, "specific renamed task λ".into());
        Ok(())
    };
    let launch = if dependent {
        let pending = controller.pending_dependent_launches.remove(&id).unwrap();
        if !cancelled {
            assert!(!pending.prompt_pending);
            assert_eq!(pending.launch.prompt, expected_prompt);
        }
        pending.launch
    } else {
        let launch = controller.pending_launches.pop_front().unwrap();
        assert_eq!(launch.id, id);
        assert_eq!(launch.feature, "specific renamed task λ");
        launch
    };
    let lines = controller.snapshot().session(&id).unwrap().lines.clone();
    let ui_notice_visible = if cancelled {
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("work-leaf: private preparation rejected:"))
        );
        crate::ui_harness::private_test_first_tests::display_actual_controller_notice(&lines)
    } else {
        false
    };
    if cancelled && dependent {
        assert!(revision.is_err());
    }
    if !cancelled {
        assert!(revision.is_ok());
    }
    // No title agent, dependency or review worker is started by this bounded seam
    // check. The exactly revised author launch traverses real CommandChat next.
    let mut chat = controller.chat.take().unwrap();
    controller.shutdown_on_drop = false;
    let result = chat.launch_prepared_agent_streaming(launch, &mut |_| {});
    json!({"ok":result.is_ok(),"error":result.err().map(|error| error.to_string()),
        "expected_prompt":expected_prompt,"lines":lines,"ui_notice_visible":ui_notice_visible})
}
