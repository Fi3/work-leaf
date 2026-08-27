# Fairness Contract

## Question

Does normal concurrent Work Leaf use fewer total provider tokens than fair direct sequential Codex
while producing a comparable implementation of the same three requests?

This one-pair pilot validates the repaired protocol and produces a descriptive result. It cannot by
itself establish an average effect or statistical confidence.

## Fixed Inputs

- Source base: `c92a0b7060a36eac6db2d869b85e589a7a9480f9`.
- Task source: the three `new` commands in `e70c933:bench-three-features`.
- Task-list SHA-256: `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`.
- Provider model: `gpt-5.5`.
- Reasoning effort: `xhigh`.
- Work Leaf read mode: normal orchestrator-mediated reads.
- Provider approval mode: `never`, matching the existing launchers.

The task text is byte-for-byte:

1. `add vim like visual mode for both panes when I do v I can select the text in focused the panes same keystrokes of vim y Y for copy maiusc V line select block select with ctrl v block select`
2. `when an user prompt start with / and is followed by something without whitespace that is a command for the agent; the orchestrator must send it to the selected backend agent and show that backend response`
3. `when review process is done the patch agent chat must be highlighted and ask is this feature done with yes/no; yes closes it, typing again reopens it`

`/fork` is not part of the task, launcher, scorer, or result.

## Workflows

The Work Leaf condition runs `bench-three-features`. It submits all three requests before waiting,
uses Work Leaf's normal patch, review, and linearize flow, and does not add validation instructions
to the requests.

The direct condition runs `bench-three-features-sequential`. It uses direct Codex without Work Leaf.
For each request it runs an implementation session followed by a separate review session, resumes
the implementation session when review finds a problem, and moves to the next request only after a
clean review. A final direct linearizer rewrites the three reviewed changes.

The direct prompts mirror Work Leaf's normal division of responsibility: feature turns use focused
checks and may fix and rerun them as needed; the final linearizer owns documentation, integration,
and all repository-required checks. Neither condition is limited to an exact command count, and the
observer never proxies or blocks Cargo.

## Equal Opportunity

Each Work Leaf feature/review stage receives up to 7,200 seconds, followed by a separate 7,200-second
linearize stage. Each direct feature/review stage receives up to 7,200 seconds, followed by a separate
7,200-second linearize stage. The serial condition therefore is not forced to fit three sequential
features into the same wall-clock window as three concurrent features.

Both linearizers must run the checks required by `AGENTS.md`, fix failures, and iterate until those
checks pass. After the model workflow ends, both drivers independently run the same non-mutating gate:

1. `cargo fmt -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-targets --all-features`

The driver gate verifies the saved result; it does not repair it.

## Model Pinning

Both paths use `bench-agent-profile-common`. It creates a per-run wrapper around the same resolved
Codex executable and injects `model_reasoning_effort="xhigh"` into every Codex invocation. The model
is passed explicitly as `gpt-5.5`. The wrapper does not read, edit, or replace the user's
`.codex/config.toml`.

The observer records the wrapper and actual Codex digests. Rollout extraction verifies that primary
provider threads report GPT-5.5 and xhigh.

## Measurement

The authoritative token value is `analysis.json -> usage_scopes.total_workflow`. It deduplicates by
provider thread and includes visible agents, helper agents, and descendant threads for both
conditions. The report gives both:

- raw tokens: input plus output; and
- uncached tokens: input minus cached input, plus output.

Observer capture status is separate from workflow quality. A capture problem cannot turn a correct
implementation into a workflow failure, and an implementation failure is never discarded. A token
comparison is usable only when both captures contain readable total-workflow usage and the recorded
model/reasoning strata match the fixed profile.

## Quality

The offline scorer tests only the three original requests:

- character, line, and block visual selection in both panes, with nonempty copy behavior;
- `/status` as a concrete selected-backend slash command whose response is shown; and
- close with `yes`, highlighted completion prompt, and reopen by typing again.

Every run contributes its observed feature count from zero through three. Workflow failures,
partials, and quality-test failures remain in the result. One condition is not silently retried or
discarded because it performs worse.

The fixed base already passes the literal `/status` quality check and fails the visual-selection and
review close/reopen checks. `SCORER-VALIDATION.md` records the source call path and the local positive
and negative runs. The slash-command task remains in the workload because removing or strengthening
it would change the original benchmark request.

## Pilot Stop Rule

The pilot runs exactly one workflow per condition, scores both saved implementations, writes a
provisional report, and stops. No larger replication batch and no mechanism-ablation condition may
start without the user's review of that report.
