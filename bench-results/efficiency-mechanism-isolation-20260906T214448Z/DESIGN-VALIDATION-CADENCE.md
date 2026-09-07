# Validation breadth, cadence and test timing

This is a source-grounded candidate assessment, not an admission or an implementation plan. It preserves the accepted H endpoint and the existing normal-WL baseline. No new control, Direct run, provider generation, parked-read cost analysis or token allocation is authorized.

## Disposition

**Validation breadth preference is largely shared in observed H. No new C10 run is justified by command counts alone.** Actual Direct prompts already request focused checks, fixing and rerunning them as needed, while leaving broad formatting/Clippy/full-suite integration checks to the final linearizer unless necessary for the feature. For example, the retained point7 Direct `runs/sequential-feature-1-implement.prompt.txt:13` and feature-1 fix prompts carry this exact guidance. The current source is `bench-three-features-direct-common::normal_validation_guidance:916`, included by `implementation_prompt` and `fix_prompt` at lines 941/964.

Root's separate H command inspection finds repeated focused target suites, not evidence that Direct repeatedly ran broad all-target gates. Lexical command counts do not identify checks that actually executed, upstream responses, causal overhead, or why a check was repeated. The meaningful unresolved distinctions are:

- **Breadth (C10):** which eligible validation targets are selected.
- **Cadence/cardinality (C13):** how often checks recur, including the at-most-one post-ACK instruction; one command can itself contain multiple checks.
- **Repair/test timing (C15):** whether a newly introduced test is actually observed failing before its first corresponding implementation, versus later regression repair.
- **Work units/completion (C14/C38):** how edits, checks, ACKs, done and review divide the same work.
- **Command waiting (root-owned):** host-blocking execution versus native asynchronous commands and generated polls, independent of validation breadth.

The initial S screen changed only ACK cardinality and did not establish a qualifying saving. W tested a combined buildable-work-unit policy and its primary mediator gate failed. Neither result is a test of the entire validation family; neither authorizes rerunning the same cue under a broader name.

## Safe source-owned C10 boundary, if later justified

The narrowly isolatable factor is **validation-breadth preference within unchanged ownership and safety eligibility**. It is not removal of every scope restriction. Default policy has no existing whole-C10 switch.

| Renderer | Owned preference substring | Minimal prospective treatment |
| --- | --- | --- |
| `agent.rs::concurrent_work_leaf_interpretation:466` | `Prefer focused checks for files you touched, checks that existed before your patch, and checks you added yourself.` | Neutral command selection from repository requirements/current change; no instruction to run more or broader checks. |
| `agent.rs::concurrent_instruction_translation:507` | `run focused checks for files you touched and checks you added or changed` | Neutral breadth selection; preserve mandatory-check prefix and external-blocker suffix. Repeated once for each applicable loaded instruction file. |
| `orchestrator.rs::render_patch_applied_prompt:3017` | `focused validation step that is relevant to files you touched or checks you added` | Validation step selected from repository requirements/current change; preserve `run at most one` and the locked-command requirement. |

Four consistency-only fragments also repeat the breadth preference inside otherwise preserved stop conditions: `agent.rs:316` (`your own focused checks`), `agent.rs:470` (`your focused checks`), `orchestrator.rs::append_cross_agent_validation_guard:2815` (`your own focused validation`), and ACK `:3027` (`the focused validation`). An experiment claiming neutralized breadth should remove only the breadth adjective or refer to the selected validation there. It must preserve the own-work requirement, failure-cause condition, report-once rule, done timing and concrete-own-defect repair condition. Leaving these clauses untouched is permissible only if the factor is explicitly described as a narrower preference-cue change.

No replacement text is frozen or approved by this assessment. Broadness-neutral wording does not guarantee the model will choose different checks. Actual selected commands and exposed changed spans would determine compliance, not condition names.

## Non-target constraints

The following remain normal, even though some contain the word “focused”:

- `agent.rs:315`: preexisting/own-check eligibility and the prohibition on using another patch agent's focused tests as local validation. “Focused tests” here identifies protected ownership, not an editable breadth adjective.
- `agent.rs:467/508`: check-only or file-scoped formatter preference while other agents are active. Removing this affects write safety and concurrent work, not just test breadth.
- `agent.rs:346–349`: no known-red, compile-breaking or deliberately failing intermediate shared-tree patches; test/design and cohesive-buildable requirements.
- `agent.rs:368` and the unmodified repository instruction bodies: required checks remain mandatory.
- `agent.rs:468–470`, command failure guard and ACK ownership clauses: no unrelated repairs, exact external-blocker reporting and stop rules.
- `orchestrator.rs::run_command_for_agent:1176 → PatchOwnershipTracker::other_agent_test_locks:495 → render_other_agent_test_command_prompt:3039`: normal enforcement, lock scope, blocked-command evidence and allowed-check eligibility.
- Command masking rejection, timeouts, outputs, pending-change capture, parser, acceptance, commits, reads, review routing and session reuse.
- `agent.rs::linearize_preamble:658` and `bench-validation-common::bench_run_final_gate:3`: final formatting, Clippy and full-test requirements.

This leaves substantive scope restrictions in force by design. Calling such a treatment “all scope guidance removed” would be false. Removing ownership enforcement, permitting knowingly failing shared-tree states, forcing extra broad checks or changing a final gate would be a materially different intervention.

## Implementation feasibility and evidence requirements

A future benchmark-only extension can retain the existing `PolicyText` renderer-owned spans and add explicit private ranges at continuation/guard construction sites. It must not search-replace matching text in user prompts, instruction bodies, source files, stdout/stderr or edit bodies. Existing default and v1–v4 text/trace contracts remain unchanged; no public API change is needed for a private renderer hook.

Prospective tests would cover multiple loaded instruction translations, no-instruction/linearizer paths, matching words embedded in copied input/output, exact non-target preservation, and success/external-failure/owned-failure continuations. Actual accepted typed delivery and source hashes would establish exposure. A pure preference change remains a behavioral hypothesis and cannot identify H's historical cause simply because command totals differ.

## Source identities and next evidence

Inspected current source SHA-256: `src/agent.rs` `a9e6065a450d05bf299202a2d6f44dd2dae33a3324e8c70e9f91d36a22b242fd`; `src/orchestrator.rs` `027c35a9eaf24e99335001e68f7d0612a262831af80f8677874525db95e9332b`; `bench-three-features-direct-common` `489289165601e00a545f76ab0631d5e0f48d644b928376488677e1875651e386`; `bench-validation-common` `235ef644d692098a670c4e61c3572f60f8830c445896f3639a56430c57781dca`.

The next provider-free evidence pass is actual C15 ordering across H's 18 Direct author threads and WL's 53 accepted ACK/58 mediated-command handoffs. It must distinguish a new-test RED before first implementation from syntax errors, existing-test failures, deliberate masking, and tests first run after cohesive code/test delivery. It does not assume that either test timing or check cadence explains the accepted saving. This document changes no agent-facing workflow and requires no real-agent generation.
