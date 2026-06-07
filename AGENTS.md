# DEMAND Repo Guide for Agents

This file exists to make agents productive quickly AND to prevent low-signal, ungrounded output.
When in doubt, inspect the repo and cite concrete file paths + symbols.

# Agent Behavior Contract (IMPORTANT)

## Grounding policy (no hand-waving)
When you describe behavior, you must point to:
- file paths (relative), and ideally symbols (types/functions/modules), and
- the chain of calls or configuration that makes the behavior true.

If uncertain, do more repo inspection until certain. Only leave TODOs if the repo truly lacks
information, and then state exactly what you searched and where.

## Large deliverables (especially documentation)
For any request that produces a large artifact (multi-thousand-line docs, specs, runbooks):
1) Work iteratively: write in chunks to the file; do NOT attempt a single huge response.
2) Use objective gates and do not stop until they pass.
3) Prefer writing to disk and only reporting results in chat.

## Documentation writing rule
When updating documentation anywhere in the repo, including any `README.md` and anything under `./docs`,
agents must describe the system in its current resulting state, not the fact that it was changed.

Do not write documentation in change-log style. In particular, do not use wording such as:
- `now the app does ...`
- `now the lib does ...`
- `this changes ...`
- `this adds ...`
- `it was updated to ...`

Documentation must explain the component as it is, how it works, what it requires, and how it behaves,
as if the reader is seeing the repository in that state for the first time.

The description must be system-oriented, not patch-oriented. 
The place to describe what changed and why is the commit message or PR description, not the documentation itself.
Only exepction is migrations.md file.

Update documentation only when the task changes the documented behavior, public workflow, required
checks, architecture, terminology, or developer operating model. Do not churn docs for unrelated
implementation-only edits, formatting-only edits, or private helper changes that do not alter what a
reader needs to know.

## Commit message rules
Every commit message must clearly say what was done and why it was done. When adding new features it
must describe which is the underling logic. Never include things that can be seen with a git diff,
like which file or function are changes unless they are necessary tho explain why something have
been done or the underling logic.

The first line must be written in imperative form and must read like the commit itself is performing
the action.

The first line must start with exactly one of these verbs:
- `ADD`: new feature or addition of something new
- `FIX`: bug fix
- `UPDATE`: improvement to something already present and backward compatible
- `UPGRADE`: improvement or change that is not backward compatible
- `DELETE`: delete something

Do not use other leading verbs.

The first line must be specific and concise, and it must describe both the change and the reason
when possible.

If more detail is needed, add a body after the first line explaining the rationale, constraints,
or important implementation notes, but keep the first line strong enough to stand on its own.

## Bug FIX rules
When asked to fix a bug, always write a test that reproduces the bug, verify that the test fails, and then write the fix.

## New feature rules
When asked to one or more feature, always write a test that test the feature (unit or integration), verify that the test fails, and then write the feature.

## Review rules
Any patch that increases algorithmic complexity to O(n²) or worse must be flagged.

Review agents MUST review only behavior introduced or modified by the reviewed patch. Do not report
pre-existing issues, unrelated style preferences, or broader repository problems unless the reviewed
patch makes them worse, depends on them, or claims to fix them.

Review must go trough all the docs and check if they needs updates.

## Required Checks
Run these before submitting changes. A patch is not ready until all required checks are green:

1. `cargo fmt`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-targets --all-features`

- Any code change must leave the relevant build and clippy invocations clean: no build warnings,
  clippy errors, or clippy warnings are allowed. Existing warnings or clippy findings encountered
  while validating the change must be fixed, not left in place.

## Tests
Adding a test do not require human permission, removing or changing one (that is committed in main) does.

## Architecture and API Governance
Before making code changes, inspect `docs/architecture.md` and preserve the documented ownership,
dependency direction, extension boundaries, and public interfaces.

Any change that can only be implemented by changing the documented architecture requires human
authorization before implementation. The same patch must update `docs/architecture.md` so it
describes the resulting architecture as current system behavior.

Breaking public API changes require human authorization before implementation. Treat the public
re-exports in `src/lib.rs`, the documented UI integration surface, the documented agent-provider
surface, and public items in public modules as public API unless repo inspection proves otherwise.
Non-breaking public API extensions do not require human authorization, but `docs/architecture.md`
must be updated when they affect documented UI, agent-provider, command, or core workflow
integration surfaces.

## Terminal UI Harness
Terminal UI behavior is exercised through `src/ui_harness.rs::UiHarness`. The harness accepts raw
input bytes and renders through the same `src/ui.rs::TerminalUi` frame path used by the interactive
example, so UI tests should drive `UiHarness::handle_byte` or `UiHarness::handle_bytes` instead of
duplicating modal-input logic.

Run `cargo test --test ui_harness` for automatic terminal UI state-machine checks. This target
covers full-width CRLF rendering, immediate nvim-style mode switches, `Ctrl-W h/j/k/l` pane
navigation, left-pane visibility toggling, prompt cursor placement, `new [prompt...]`, and
insert-mode chat text.

Run `cargo test --test terminal_pty` for automatic real-terminal checks. This target starts the
`work-leaf` binary under a pseudo-terminal, drives raw key bytes against the real UI event loop, and
uses a deterministic fake `codex` executable to verify agent creation, orchestrator file-read
follow-up, left-pane hide/show with `,`, left-pane keyboard and mouse chat selection, large agent
output, and visible chat prompt behavior.

Any feature or bug fix that requires the UI harness, a pseudo-terminal run, or manual terminal
interaction to verify behavior must include the same scenario in the automatic tests. Manual harness
runs can be used while developing, but the patch is not ready until the equivalent `cargo test`
coverage is present and green.

Run `cargo run --example ui_harness` in a real interactive terminal for visual UI development. The
manual fixture uses the same harness state machine and supports `Esc`, `i`, `:`, `Ctrl-W h/j/k/l`,
`,`, `new [prompt...]`, and `q`.

## Architecture and Extension Boundaries
`docs/architecture.md` is the source of truth for module ownership, public integration surfaces, and
extension paths. The short rule is that UIs integrate through `src/workspace.rs::WorkLeafController`
and its DTOs, agent providers integrate through `src/agent.rs::AgentProfile` and
`src/agent_runtime.rs::AgentBackend`, and Codex-specific logic stays in `src/codex.rs::CodexBackend`.
