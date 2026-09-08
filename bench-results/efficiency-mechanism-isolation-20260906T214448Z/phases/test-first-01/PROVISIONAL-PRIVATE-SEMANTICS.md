# Provisional semantics: two closed private previews

The two inspected proposals contain test-only changes and both closed with assertion failures after successful compilation. Neither receipt represents a shared apply or a complete benchmark outcome.

## Exact cutoff

This note covers only `user-2`, launch generation 2, first private operation `preview-0000000000000001` in workflows `test-first-01-workflow-001` and `test-first-01-workflow-003`. Other operations, changing shared checkouts, later accepted edits, normal checks, DONE/review, model/native messages and accounting are outside this cutoff. No test, executor, Cargo, provider or extractor was invoked for this review.

The [JSON witness](PROVISIONAL-PRIVATE-SEMANTICS.json) retains 34 direct source hashes and typed record locators. Read-only check `353b81` passed 125 before/after endpoint checks, including all 47 retained private source-file byte/mode/hash entries per operation. Check `a9203b` produced the bounded projection. Existing 65 dependency pins per result remain in their original receipts; this is not another dependency or confinement audit.

Both selections name accepted commit `c92a0b7060a36eac6db2d869b85e589a7a9480f9`, tree `acb6583fbb0e25347f70ee44cbdacafd6df95e37`, and a 287,837-byte Git bundle with SHA-256 `ec1c907aa703d68e19b655c3389d5c6ad66398a4a314b945cb738c2a351cdc73`. Each exact request points to its own captured selection. The selected records explicitly do not claim Python proof of the caller lock or all-live-tree atomicity. Snapshot `shared.root` is the owned accepted clone, not the changing original checkout.

## Workflow 001: terminal input and malformed slash

[Request](private-preview/test-first-01-workflow-001/preview-0000000000000001/test-input.json), [result](private-preview/test-first-01-workflow-001/preview-0000000000000001/TEST-RESULT.json), [execution](private-preview/test-first-01-workflow-001/preview-0000000000000001/TEST-EXECUTION.json).

Proposal `slash-agent-command-test-3` declares `revision_of: null`; its numeric suffix is not evidence of a preceding execution or revision. Its stated purpose is selected-agent slash routing only when the slash is immediately followed by a non-whitespace command name. Exact command: `cargo test slash_command --all-targets --all-features`.

The held edit strengthens the existing UiHarness `/status` test with a backend-reply assertion, adds a `/ status` rejection test, and adds `terminal_app::tests::slash_command_routes_selected_agent_prompt_to_backend_and_renders_reply` with a recording fake backend. Only `tests/ui_harness.rs` and the existing `#[cfg(test)] mod tests` in `src/terminal_app.rs` differ. The latter's first 36,072 bytes are unchanged (SHA-256 `b5116a614cb04b156f8770fc49027c7cb9d49a64d2be461edda3dc1f40950170`); full diffs `049d91` and `c346d4` contain only test code/assertions. No co-located production implementation change is present in this proposal.

The actual library run passed the existing Codex slash-resume test and failed the new terminal-app test: 1 passed, 1 failed, 12 filtered. At private `src/terminal_app.rs:1413`, after `Escape / status Enter`, the mode assertion observed `Insert` instead of `Command`. Earlier assertions in that same test established exactly one valid `/status` send to `user-1` and rendered input/backend reply. Later malformed-send-count/render assertions were not reached. Cargo stopped at the failed library target, so the new/strengthened integration UiHarness tests were authored but did not execute in this command.

The retained accepted implementation explains the exercised boundary: `src/terminal_app.rs:545` dispatches `/` to `start_agent_slash_command`; `should_start_agent_slash_command` at 794 checks command mode and selected agent, and `start_agent_slash_command` at 798 activates chat immediately. `send_chat_buffer` at 633 submits nonempty text to the selected agent. This supports a narrow malformed-slash regression, not a claim that all positive routing was initially absent.

## Workflow 003: command-agent controller routing

[Request](private-preview/test-first-01-workflow-003/preview-0000000000000001/test-input.json), [result](private-preview/test-first-01-workflow-003/preview-0000000000000001/TEST-RESULT.json), [execution](private-preview/test-first-01-workflow-003/preview-0000000000000001/TEST-EXECUTION.json).

Proposal `slash-agent-command-test-2` also declares `revision_of: null`. Its stated purpose is delivery of a command-agent slash prompt to the selected user session, rather than command-agent help. Exact command: `cargo test command_agent_slash_message_routes_to_selected_agent`.

Only `src/workspace.rs` differs. It contains a new test-only backend and `workspace::tests::command_agent_slash_message_routes_to_selected_agent`. Removing the exact UTF-8 byte range `[29538,31583)`—a 2,045-byte `#[cfg(test)] mod tests` insertion—restores every baseline byte. Thus this held proposal also contains no production implementation change.

The actual library run compiled and failed its sole selected test: 0 passed, 1 failed, 13 filtered. At private `src/workspace.rs:867`, `wait_for_session_line` did not observe `backend response to /status` in the selected agent session within one second after `controller.send_command_agent_message("/status")`. The later assertions about user/reply transcript lines and absence of help were not reached; the failure alone does not establish which help text appeared or exclude an arbitrarily late reply.

The selected accepted `src/workspace.rs:158` method routes through `command_agent_response` at 180 and writes to the command transcript or executes a parsed command. The test intentionally exercises that direct controller method, not workflow 001's terminal keystroke path. Their failures therefore must not be described as the same test or identical routing coverage.

## Interpretation limits

Both `TEST-RESULT` records are `status: completed`, `closed: true`, `exit_code: 101`, with no timeout/cancellation, and `shared_accepted: false`, `promoted: false`. Their held proposal bodies match the private patch receipt; all three declared full-file afterimages match retained private files exactly. The recorded private before/after differences are precisely the declared test paths, and the complete private file census is unchanged between after-patch and after-command receipts.

Test-only classification here rests on complete file diffs and exact test-module boundaries, not filenames, test-purpose strings or whole-file hashes alone. Requirement mapping is to each exact proposal's declared purpose; this cutoff does not re-open the original launch/task prompt or establish full feature-spec conformance. It makes no shared-final test equivalence, workflow completion, semantic RED-to-GREEN, token-saving direction or causal-share claim. Those joins require later closed accepted-edit/check/DONE evidence.
