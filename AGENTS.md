# Work-Leaf Repo Guide for Agents

This file exists to make agents productive quickly AND to prevent low-signal, ungrounded output.
When in doubt, inspect the repo and cite concrete file paths + symbols.

# Agent Behavior Contract (IMPORTANT)

## Grounding policy (no hand-waving)
When you describe behavior, you must point to:
- file paths (relative), and ideally symbols (types/functions/modules), and
- the chain of calls or configuration that makes the behavior true.

If uncertain, do more repo inspection until certain. Only leave TODOs if the repo truly lacks
information, and then state exactly what you searched and where.

## Generality policy (do not overfit)
Work Leaf is a general-purpose orchestrator. Do not solve bugs, benchmarks, tests, or reviews by
recognizing this repository's current smoke-test features, temporary paths, exact filenames, exact
command output, local model names, or one-off run artifacts.

When implementing patches, be specific to the repository you are editing: follow its architecture,
public APIs, naming, style, instructions, and required checks as closely as possible. Ground changes
in the actual files and behavior of that repository.

When changing Work Leaf itself, keep orchestrator mechanisms provider-neutral and project-neutral
unless the relevant module is explicitly provider-specific. Tests for orchestrator behavior should
assert durable protocol rules, state transitions, invariants, and user-visible behavior that would
still make sense for a different repository using Work Leaf.

If a proposed fix only works because it knows the current benchmark scenario, stop and redesign it
before coding.

## Large deliverables (especially documentation)
For any request that produces a large artifact (multi-thousand-line docs, specs, runbooks):
1) Work iteratively: write in chunks to the file; do NOT attempt a single huge response.
2) Use objective gates and do not stop until they pass.
3) Prefer writing to disk and only reporting results in chat.

## Benchmark execution policy

For this user's benchmark studies, the default is **three complete benchmark workflows running
concurrently**, with separate checkouts and artifact directories, whenever at least three approved
runs are available. This counts top-level benchmarks, not feature agents inside one Work Leaf run.
Do not assume a two-workflow concurrency limit. A smaller explicitly approved pilot or remaining
batch may use fewer slots; available capacity does not authorize extra observations, replacements,
or bypassing a requested pause.

Codex benchmark generation uses the user's existing ChatGPT subscription, not API-key authentication
or API credits. Before planning or launching a batch, read `docs/benchmark-operator-policy.md` and
its study's frozen protocol. Preserve admitted runs and record all outcomes.

### Mandatory shallow screening before expensive experiments

The first line of `hypotesis.md` uses the user-approved finite task ledger:
`DONE + TODO = TOTAL = 74`. Its source is
`bench-results/efficiency-mechanism-isolation-20260906T214448Z/PROGRESS-CHECKLIST.json`.
The 2026-09-14 one-time correction preserves all 54 completed historical records and replaces
the misleading G03 aggregate with twelve explicitly bounded tasks, R01–R12. Definitions, budgets
and completion criteria are frozen in `PROGRESS-TASK-SCOPE-20260914.json` and the linked approved
plan beside the ledger. These are tasks, not independent hypotheses or a percentage of effort.
DONE never decreases; TODO never increases without subsequent explicit user approval.
No added, split, replaced, reopened or broadened task; no hidden supplementary experiment.
A listed task closes only with its saved dated result, evidence and honest terminal outcome.
Negative, inconclusive and failed-setup results finish bounded checks, not scientific proof.
New necessary work is an unapproved scope issue, not an execution queue.
**At zero TODO, stop.** Report a supported answer, or explicitly state case (1) additional work
beyond the list and/or case (2) an incorrect initial checklist; name the exact proposed tasks/count
and ask the user before increasing TODO or executing them. Zero alone is not scientific success.
Only subsequent explicit user approval permits a scope/count extension; preserve the zero checkpoint.
Before every progress report or commit affecting this ledger, run
`python3 bench-results/efficiency-mechanism-isolation-20260906T214448Z/test_progress_counter.py`
and require success. The guard checks declarations and evidence files, not scientific truth.
`PROGRESS-PUBLICATION-HISTORY.json` retains the old 52/3 and 54/1 checkpoints plus the approved
54/12 correction, the completed 66/0 checkpoint and the explicitly approved 66/8/74 extension.
The frozen `PROGRESS-TASK-EXTENSION-20260914.json` defines R13–R20 and pins the original
zero ledger, publication history and eight-task proposal. All 66 completed records remain immutable.
Every completion appends a matching checkpoint, including its result declaration.
Never rewrite or truncate previous checkpoints. Publication validation rejects rollback even when
the live history is truncated; the pinned task contract also rejects same-ID scope substitution.
The user's “ok you can now proceed with the new 8 task” authorizes exactly R13–R20.
Run the additive `test_progress_extension.py` beside the mandatory original progress tests.

The mechanism-research objective is to explain the historical ~50% result as fast as evidence
permits, not to complete candidates in ID order. Maintain a live most-promising ranking near the
top of `hypotesis.md`, with evidence, potential explanatory reach, next cheap check and stop gate.
Re-rank after each material finding. Promote a promising joint cause immediately when it offers
more explanatory value than unfinished singles; no fixed candidate order or single-factor-first
prerequisite applies. A joint screen must declare its whole factor set and preserve non-targets.
Use short, representative checks to classify promise; park weak or broken approaches promptly.
Do not spend a night extending a dead end or building machinery before a mechanism signal.
The active goal and `ephemeral-note.md` preserve decisions, activity, costs and reasons to pivot.
This adaptive selection authority does not waive subscription-only use, no extra ordinary controls,
frozen admitted outcomes, prospective budgets or benchmark-only isolation of interventions.

Before expanding a hypothesis into long benchmarks or substantial experimental infrastructure,
use the smallest representative check that can expose a broken setup or an unpromising mechanism.
Read prior candidate results first; do not rename and repeat an unsuccessful screen without a
specific new distinction. Freeze the hypothesis, single changed factor or declared joint-factor set,
predicted behavior,
non-target invariants, observation count, time/usage budgets, and stop/advance criteria before
generation. Test relevant rejection, retry and stale-state paths, not only a happy-path smoke.
A working harness is not a positive mechanism result. Escalation requires an observed mechanism
signal relevant to the question, not merely successful execution or a favorable-looking total.
An inconclusive shallow screen is not proof of zero effect; preserve it and deprioritize or seek
direction instead of automatically extending, replacing, or repairing-and-rerunning whole batches.
Budget ceilings are stopping conditions, not targets to spend. Preserve partial/failed outcomes and
accounting tails. Keep screen, harness qualification and full-workflow causal confirmation distinct.
Follow `docs/benchmark-operator-policy.md` for admission and monitoring details. This policy does not
authorize new runs or controls, alter the approved three-workflow concurrency rule, or permit
injecting operator instructions into frozen benchmark-agent prompts.

For token-mechanism investigations, read root `hypotesis.md` and `ephemeral-note.md` before resuming.
`hypotesis.md` is the primary hypothesis register: update the individual and joint entries when a
check starts, finishes, fails or stops, and before reporting status. Include what/how tested,
observed direction and limits, exact evidence, dated last check and truthful current activity.
Preserve prior results; proposals, local checks, failed experiments and causal confirmation are
different states. Record new hypotheses and their relation to already-tested factors there.
Maintain its fixed-scope done/to-do counts and explicit remaining-check list with every closure.
The very first line of `hypotesis.md` must show the absolute DONE / TODO counts. Immediately below
it, show the actual pending check and current verified result. Keep this opening block current;
priorities, explanations and historical detail belong below it, never before it.
TODO contains ONLY selected, necessary pending checks. Completed negative/inconclusive/deprioritized
screens are DONE; proposals and remaining scientific uncertainties stay outside TODO. Do not reopen
a screen merely because its effect has not been proved zero; require a concrete new reason and
explicitly record any selected scope extension. Distinguish overlapping register IDs from work items.
Preserve cumulative completed checks; flag invalidated conclusions and genuinely new scope openly.
Do not mark exhaustive causal coverage complete merely because experiments ran or returned null:
the register's measurement, boundary and joint-residual closure requirements must be satisfied.

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
When asked to add one or more feature, always write a test that test the feature (unit or integration), verify that the test fails, and then write the feature.

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
Every UI change MUST add an ui_harness automatic test.

## Real Agent Verification
Every patch must include a real-agent verification step before it is marked ready. Automated tests
with fake backends, deterministic fake `codex` binaries, UI harnesses, or mocked providers are useful
but are not enough when the changed behavior is supposed to work with an actual agent.

For any change that affects agent launch, agent send/resume, slash commands, terminal chat behavior,
orchestrator protocol, patch/review/linearize flow, locked command execution, or provider
integration, the implementing agent must verify the behavior with the real configured agent backend
during implementation. The ready report must state the exact real-agent scenario that was run and the
user-visible result. If the patch does not affect any agent-facing behavior, the ready report must
state that no real-agent workflow is affected and explain why.

Review agents must treat missing real-agent verification as a finding for agent-facing changes.
Do not accept fake-backend or harness-only coverage as proof that an agent-facing workflow works in
the real orchestrator.

### Real Codex verification troubleshooting
When the real configured backend is Codex, use a bounded smoke check and do not loop indefinitely on
local Codex CLI setup failures.

Use the same top-level option ordering as `src/codex.rs::CodexBackend`:

```sh
printf 'Reply exactly with WORK_LEAF_REAL_AGENT_OK and do not modify files.\n' | \
  codex --disable apps --cd "$PWD" --sandbox read-only --ask-for-approval never \
    exec --color never --json -
```

For resume behavior, capture the `thread.started` `thread_id` from the launch JSONL, then send a raw
follow-up:

```sh
printf 'Reply exactly with WORK_LEAF_REAL_AGENT_RESUME_OK and do not modify files.\n' | \
  codex --disable apps --cd "$PWD" --sandbox read-only --ask-for-approval never \
    exec resume --json <thread-id> -
```

The follow-up must not include a new copy of `AGENTS.md` or the full launch policy unless the
behavior being tested specifically requires a fresh launch.

If the smoke check fails before `thread.started` with
`failed to initialize in-process app-server client: Read-only file system`, run `codex doctor` once
and record that Codex could not initialize its local app-server/state from the current sandbox. Do
not keep retrying the same command.

If using a temporary `CODEX_HOME` under `/tmp` gets past app-server initialization but fails with
`401 Unauthorized` or missing bearer/basic authentication, record that the temporary home does not
have the stored Codex auth state. Do not copy `~/.codex/auth.json`, API keys, ChatGPT tokens, or any
other credential material into `/tmp` or into the repository.

When real Codex verification is blocked by local Codex initialization or auth-state isolation, the
ready report must say exactly which command was attempted, which pre-agent error occurred, and which
automated tests cover the behavior instead. A blocked real-agent smoke check is not a green
real-agent verification.

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
