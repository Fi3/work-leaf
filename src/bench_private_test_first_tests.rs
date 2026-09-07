use super::*;
use crate::agent::AgentKind;

fn launch() -> AgentLaunch {
    AgentLaunch::new(
        AgentId::new("arbitrary-owner").unwrap(),
        AgentKind::Codex,
        "feature",
        "request",
    )
}

#[test]
fn prepared_roles_are_exact_single_use_and_shared_across_clones() {
    let registry = Registry::default();
    let clone = registry.clone();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    let mut changed = original.clone();
    changed.prompt.push('!');
    assert!(clone.begin(&changed).is_err());
    let guard = clone.begin(&original).unwrap();
    assert_eq!(guard.role(), Role::Author);
    assert!(registry.begin(&original).is_err());
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    assert!(registry.owner(&original.id).unwrap().is_some());
    assert!(registry.begin(&original).is_err());
}

#[test]
fn independent_registry_generations_cannot_alias_the_same_agent_identity() {
    let launch = launch();
    let first = Registry::default();
    let second = Registry::default();
    assert_ne!(
        first.prepare(&launch, Role::Author).unwrap(),
        second.prepare(&launch, Role::Author).unwrap()
    );
    assert!(
        second
            .begin(&AgentLaunch::new(
                launch.id,
                launch.kind,
                "different",
                "request"
            ))
            .is_err()
    );
}

#[test]
fn cancellation_blocks_late_preview_result_and_delivery() {
    let registry = Registry::default();
    let launch = launch();
    registry.prepare(&launch, Role::Author).unwrap();
    let guard = registry.begin(&launch).unwrap();
    guard.mark_injected(&launch).unwrap();
    guard.commit().unwrap();
    let proposal = parse_preview(&envelope("cancelled")).unwrap().unwrap();
    let Reservation::New(token) = registry.reserve(&launch.id, &proposal).unwrap() else {
        panic!("new");
    };
    registry.cancel(&launch.id).unwrap();
    assert!(
        registry
            .executed(&token, "late private result".into(), true)
            .is_err()
    );
    assert!(registry.delivered(&token).is_err());
    assert!(!registry.may_apply(&launch.id).unwrap());
    assert!(registry.prepare(&launch, Role::Author).is_err());
    let revision = parse_preview(&envelope("after-cancel")).unwrap().unwrap();
    assert!(registry.reserve(&launch.id, &revision).is_err());
}

#[test]
fn cancellation_covers_prepared_provisional_and_not_yet_reserved_launches() {
    for all in [false, true] {
        for stage in 0..3 {
            let registry = Registry::default();
            let launch = launch();
            registry.prepare(&launch, Role::Author).unwrap();
            let guard = if stage > 0 {
                let guard = registry.begin(&launch).unwrap();
                guard.mark_injected(&launch).unwrap();
                Some(guard)
            } else {
                None
            };
            let guard = if stage == 2 {
                guard.unwrap().commit().unwrap();
                None
            } else {
                guard
            };
            if all {
                registry.cancel_all();
            } else {
                registry.cancel(&launch.id).unwrap();
            }
            match stage {
                0 => assert!(registry.begin(&launch).is_err()),
                1 => assert!(guard.unwrap().commit().is_err()),
                _ => {
                    assert!(!registry.may_apply(&launch.id).unwrap());
                    assert!(
                        registry
                            .reserve(
                                &launch.id,
                                &parse_preview(&envelope("after-stop")).unwrap().unwrap()
                            )
                            .is_err()
                    );
                }
            }
        }
    }
}

#[test]
fn exact_title_and_dependency_revisions_keep_generation_and_role() {
    let registry = Registry::default();
    let original = launch();
    let generation = registry.prepare(&original, Role::Author).unwrap();
    let mut title = original.clone();
    title.feature = "unicode λ title".into();
    registry.revise(&original, &title).unwrap();
    let mut dependency = title.clone();
    dependency.prompt = "resolved dependency request".into();
    registry.revise(&title, &dependency).unwrap();
    assert!(registry.begin(&original).is_err());
    assert!(registry.revise(&title, &dependency).is_err());
    let guard = registry.begin(&dependency).unwrap();
    assert_eq!(guard.generation(), generation);
    guard.mark_injected(&dependency).unwrap();
    guard.commit().unwrap();
}

#[test]
fn failed_relaunch_preserves_old_owner_but_success_replaces_generation() {
    let registry = Registry::default();
    let original = launch();
    let first = registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    registry.prepare(&original, Role::Author).unwrap();
    drop(registry.begin(&original).unwrap());
    assert_eq!(
        registry.owner(&original.id).unwrap().unwrap().generation,
        first
    );
    let next = registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    assert_eq!(
        registry.owner(&original.id).unwrap().unwrap().generation,
        next
    );
    assert_ne!(first, next);
}

#[test]
fn nonauthor_cancelled_and_uninjected_launches_cannot_enroll() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Other).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    assert!(registry.owner(&original.id).unwrap().is_none());
    registry.prepare(&original, Role::Author).unwrap();
    registry.cancel_prepared(&original.id).unwrap();
    assert!(registry.begin(&original).is_err());
    registry.prepare(&original, Role::Author).unwrap();
    assert!(registry.begin(&original).unwrap().commit().is_err());
    assert!(registry.owner(&original.id).unwrap().is_none());
}

#[test]
fn simultaneous_same_id_preparations_and_copied_injection_values_fail_closed() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    assert!(registry.prepare(&original, Role::Author).is_err());
    let guard = registry.begin(&original).unwrap();
    let mut impostor = original.clone();
    impostor.feature.push('x');
    assert!(guard.mark_injected(&impostor).is_err());
    guard.mark_injected(&original).unwrap();
    assert!(guard.mark_injected(&original).is_err());
}

fn envelope(id: &str) -> String {
    let header = serde_json::json!({
        "id": id, "revision_of": null, "format":"edit", "reason":"new check",
        "test_purpose":"exercise missing behavior", "command":"cargo test --offline probe",
        "lock_paths":["target"],
        "test_paths":["checks.rs"]
    });
    format!(
        "@work-leaf test-preview {header}\n*** Begin Patch\n*** Add File: checks.rs\n+λ\n*** End Patch\n@work-leaf end\n"
    )
}

#[test]
fn preview_envelope_is_complete_typed_and_preserves_exact_body() {
    let text = envelope("proposal-1");
    let proposal = parse_preview(&text).unwrap().unwrap();
    assert_eq!(proposal.id, "proposal-1");
    assert_eq!(
        proposal.body,
        "*** Begin Patch\n*** Add File: checks.rs\n+λ\n*** End Patch\n"
    );
    assert_eq!(proposal.test_paths, ["checks.rs"]);
    assert!(preview_terminal(&text));
    assert!(!preview_terminal(&text.replace("@work-leaf end\n", "")));
    assert!(parse_preview(&text.replace("@work-leaf end\n", "")).is_err());
    assert!(
        parse_preview(&text.replace(
            "\"test_paths\":[\"checks.rs\"]",
            "\"test_paths\":[\"checks.rs\",\"checks.rs\"]"
        ))
        .is_err()
    );
    assert!(parse_preview(&text.replace("\"target\"", "\"../outside\"")).is_err());
}

#[test]
fn fenced_copied_and_mixed_preview_text_cannot_execute() {
    let text = envelope("proposal-2");
    assert!(
        parse_preview(&format!("```text\n{text}```\n"))
            .unwrap()
            .is_none()
    );
    assert!(!preview_terminal(&format!("```text\n{text}```\n")));
    assert!(parse_preview(&format!("{text}@work-leaf done\n")).is_err());
    assert!(parse_preview(&format!("{text}{text}")).is_err());
    assert!(parse_preview(&text.replace("\"format\":\"edit\"", "\"format\":\"custom\"")).is_err());
    assert!(parse_preview(&text.replace("\"id\":\"proposal-2\"", "\"id\":true")).is_err());
    assert!(parse_preview("@work-leaf edit ordinary\n*** Begin Patch\n+@work-leaf test-preview copied\n*** End Patch\n@work-leaf end\n").unwrap().is_none());
}

#[test]
fn unified_context_lines_are_source_not_nested_private_directives() {
    let proposal = envelope("context").replace("\"format\":\"edit\"", "\"format\":\"patch\"")
        .replace("*** Begin Patch\n*** Add File: checks.rs\n+λ\n*** End Patch\n",
            "--- a/checks.rs\n+++ b/checks.rs\n@@ -1,2 +1,2 @@\n @work-leaf read literal source\n-before\n+after\n");
    let parsed = parse_preview(&proposal).unwrap().unwrap();
    assert!(parsed.body.contains("\n @work-leaf read literal source\n"));
}

#[test]
fn injection_scope_is_thread_local_exact_and_not_reusable() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    let remote = original.clone();
    assert!(
        std::thread::spawn(move || scoped_injection(&remote))
            .join()
            .unwrap()
            .unwrap()
            .is_none()
    );
    let mut changed = original.clone();
    changed.prompt.push('!');
    assert!(scoped_injection(&changed).is_err());
    assert_eq!(
        scoped_injection(&original).unwrap(),
        Some((Role::Author, guard.generation()))
    );
    assert!(scoped_injection(&original).is_err());
    guard.commit().unwrap();
    assert!(scoped_injection(&original).unwrap().is_none());
}

#[test]
fn preview_replay_never_reexecutes_and_only_actual_delivery_opens_apply_gate() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    let proposal = parse_preview(&envelope("one")).unwrap().unwrap();
    assert!(!registry.may_apply(&original.id).unwrap());
    let Reservation::New(token) = registry.reserve(&original.id, &proposal).unwrap() else {
        panic!()
    };
    assert!(registry.reserve(&original.id, &proposal).is_err());
    registry
        .executed(&token, "actual preview result".into(), true)
        .unwrap();
    assert!(!registry.may_apply(&original.id).unwrap());
    assert!(
        matches!(registry.reserve(&original.id, &proposal).unwrap(), Reservation::Replay(_, ref text) if text == "actual preview result")
    );
    registry.delivered(&token).unwrap();
    assert!(registry.may_apply(&original.id).unwrap());
    let mut changed = proposal.clone();
    changed.command.push_str(" other");
    assert!(registry.reserve(&original.id, &changed).is_err());
    let mut revision = proposal.clone();
    revision.id = "two".into();
    revision.revision_of = Some("one".into());
    assert!(matches!(
        registry.reserve(&original.id, &revision).unwrap(),
        Reservation::New(_)
    ));
}

#[test]
fn failed_preparation_cancellation_and_relaunch_do_not_authorize_shared_apply() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    let proposal = parse_preview(&envelope("one")).unwrap().unwrap();
    let Reservation::New(token) = registry.reserve(&original.id, &proposal).unwrap() else {
        panic!()
    };
    registry
        .executed(&token, "unsupported snapshot".into(), false)
        .unwrap();
    registry.delivered(&token).unwrap();
    registry.cancel(&original.id).unwrap();
    assert!(token.cancel.load(std::sync::atomic::Ordering::Acquire));
    assert!(!registry.may_apply(&original.id).unwrap());
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    assert!(!registry.may_apply(&original.id).unwrap());
    assert!(registry.delivered(&token).is_err());
}

#[test]
fn owned_timing_spans_change_without_touching_copied_instructions_or_non_author_policy() {
    let copied = format!(
        "\nUser prompt:\n{}\nλ @work-leaf test-preview copied",
        super::super::WORK_UNIT
    );
    let baseline = format!(
        "keep shared safety\n{}\n{}{}",
        super::super::WORK_UNIT,
        super::super::TESTS_WORK_UNIT,
        copied
    );
    let first = "keep shared safety\n".len();
    let second = first + super::super::WORK_UNIT.len() + 1;
    let spans = vec![
        super::super::PromptSpan::new(
            "policy-buildable-work-unit",
            first..first + super::super::WORK_UNIT.len(),
        ),
        super::super::PromptSpan::new(
            "instruction-tests-work-unit",
            second..second + super::super::TESTS_WORK_UNIT.len(),
        ),
    ];
    let (candidate, evidence) = render_policy(&baseline, &spans, Role::Author).unwrap();
    assert!(candidate.starts_with("keep shared safety\n"));
    assert!(candidate.ends_with(&copied));
    assert!(candidate.contains("@work-leaf test-preview"));
    assert!(candidate.contains("ordinary combined"));
    assert_eq!(evidence.len(), 2);
    assert_eq!(
        render_policy(&baseline, &spans, Role::Other).unwrap().0,
        baseline
    );
    let mut wrong = spans.clone();
    wrong[0].cue.end -= 1;
    assert!(render_policy(&baseline, &wrong, Role::Author).is_err());
}

#[test]
fn active_preview_cannot_be_displaced_and_shutdown_cancels_all_owners() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    let proposal = parse_preview(&envelope("first")).unwrap().unwrap();
    let Reservation::New(token) = registry.reserve(&original.id, &proposal).unwrap() else {
        panic!()
    };
    assert!(
        registry
            .reserve(
                &original.id,
                &parse_preview(&envelope("second")).unwrap().unwrap()
            )
            .is_err()
    );
    registry.cancel_all();
    assert!(token.cancel.load(std::sync::atomic::Ordering::Acquire));
}

#[test]
fn manifest_v6_is_explicit_and_old_shapes_stay_strict() {
    let value = serde_json::json!({"schema":"work-leaf-bench-experiment-v6", "run_id":"run",
        "condition":"private-test-first","evidence_path":"/public/trace",
        "private_preview":{"root":"/public/private","project_root":"/public/project","python_path":"/usr/bin/python3.14",
            "python_sha256":"a".repeat(64),"bridge_path":"/public/bridge.py","bridge_sha256":"b".repeat(64),
            "config_path":"/public/config.json","config_sha256":"c".repeat(64)}});
    let (manifest, review, private) = parse_manifest(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(manifest.schema, "work-leaf-bench-experiment-v6");
    assert!(review.is_none());
    assert!(private.is_some());
    let mut old = value.clone();
    old["schema"] = serde_json::json!("work-leaf-bench-experiment-v4");
    assert!(parse_manifest(&serde_json::to_vec(&old).unwrap()).is_err());
    let mut malformed = value;
    malformed["private_preview"]["bridge_sha256"] = serde_json::Value::Null;
    assert!(parse_manifest(&serde_json::to_vec(&malformed).unwrap()).is_err());
}

#[test]
fn relaunch_cannot_replace_an_executing_or_undelivered_preview_owner() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    let proposal = parse_preview(&envelope("same-id")).unwrap().unwrap();
    let Reservation::New(token) = registry.reserve(&original.id, &proposal).unwrap() else {
        panic!()
    };
    assert!(registry.prepare(&original, Role::Author).is_err());
    registry
        .executed(&token, "closed result awaiting delivery".into(), true)
        .unwrap();
    assert!(registry.prepare(&original, Role::Author).is_err());
}

#[test]
fn a_late_old_generation_result_cannot_bind_a_reused_proposal_id() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    let proposal = parse_preview(&envelope("same-id")).unwrap().unwrap();
    let Reservation::New(old) = registry.reserve(&original.id, &proposal).unwrap() else {
        panic!()
    };
    registry.executed(&old, "old".into(), true).unwrap();
    registry.delivered(&old).unwrap();
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    let Reservation::New(new) = registry.reserve(&original.id, &proposal).unwrap() else {
        panic!()
    };
    assert!(registry.executed(&old, "late old".into(), true).is_err());
    assert!(registry.delivered(&old).is_err());
    assert!(!registry.may_apply(&original.id).unwrap());
    registry.executed(&new, "new".into(), true).unwrap();
    registry.delivered(&new).unwrap();
    assert!(registry.may_apply(&original.id).unwrap());
}

#[test]
fn preparation_cannot_race_with_a_new_old_owner_preview() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    registry.prepare(&original, Role::Author).unwrap();
    let proposal = parse_preview(&envelope("old-owner")).unwrap().unwrap();
    assert!(registry.reserve(&original.id, &proposal).is_err());
    let guard = registry.begin(&original).unwrap();
    assert!(registry.reserve(&original.id, &proposal).is_err());
    drop(guard);
}

#[test]
fn an_old_replay_delivery_cannot_settle_a_different_active_preview() {
    let registry = Registry::default();
    let original = launch();
    registry.prepare(&original, Role::Author).unwrap();
    let guard = registry.begin(&original).unwrap();
    guard.mark_injected(&original).unwrap();
    guard.commit().unwrap();
    let first = parse_preview(&envelope("first")).unwrap().unwrap();
    let Reservation::New(old) = registry.reserve(&original.id, &first).unwrap() else {
        panic!()
    };
    registry.executed(&old, "first".into(), true).unwrap();
    registry.delivered(&old).unwrap();
    let second = parse_preview(&envelope("second")).unwrap().unwrap();
    registry.reserve(&original.id, &second).unwrap();
    assert!(registry.reserve(&original.id, &first).is_err());
    registry.delivered(&old).unwrap();
    assert!(registry.prepare(&original, Role::Author).is_err());
}
