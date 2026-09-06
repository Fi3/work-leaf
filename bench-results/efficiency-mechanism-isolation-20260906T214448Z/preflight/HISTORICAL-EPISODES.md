# Historical implementation episodes

This is an exploratory audit of the already completed 2026-09-06 pilot, not a new
observation, an intervention, or a causal allocation. The fresh screening and confirmation
datasets exclude these episodes. All selected native rollout files match the observer's
saved SHA-256 metadata. Exact response identities, usage, emitted actions, tool-result
locations, and source hashes are in [HISTORICAL-EPISODES.json](HISTORICAL-EPISODES.json).

## Strongest larger hypothesis: cohesive edits and validation cycles

The historical pilot's recorded implementation/fix gap has this exact arithmetic:

| Component | Direct minus WL tokens |
| --- | ---: |
| Cached input | 15,696,640 |
| Uncached input | 3,106 |
| Output, including reasoning | 5,464 |
| Raw input plus output | 15,705,210 |

Direct recorded 30,149,576 implementation/fix raw tokens and WL 14,444,366. This
identity locates the difference in repeatedly processed input; it does not explain why
the underlying action sequences differ. Cached tokens still count in the declared raw
metric. The separate compaction scope discrepancy below is not silently added here.

The concrete larger candidate is the **unit of implementation work**: test-only changes
and per-file edit/check continuations versus cohesive multi-file test-and-implementation
submissions, together with the scope and repetition of local validation. The two small
acknowledgement experiments do not isolate this whole mechanism.

### Source-to-prompt chain

- The archived Direct `agent-policy/effective-AGENTS.md:69` and `:72` require a failing
  regression/feature test before implementation. `bench-three-features-direct-common`'s
  implementation prompt retains repository instructions and normal direct tools.
- `src/agent.rs::PromptPolicy::for_read_permission` includes the cohesive/buildable
  shared-tree instruction at line 267. `PromptPolicy::inject` includes
  `concurrent_instruction_translation`; its `topics.tests` branch at line 362 instructs
  the agent to design tests but submit them together with the implementation needed to
  keep the shared tree buildable. The original repository instruction also remains in
  the prompt. This is a real difference in the interpretation of test timing.
- Successful mediated edits reach `src/orchestrator.rs::render_patch_applied_prompt`,
  which acknowledges the provisional commit, asks for a focused check, and asks for
  `@work-leaf done` after that check or a reported external blocker.

### Direct feature 3: observed RED, per-file edits, repair

The native source is
`/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T22-00-16-01a0784e-abd2-7803-a0b5-e83fd25653a8.jsonl`.
Indices below identify unique response-usage records within that file, not provider
turns. This entire initial implementation is one CLI task turn.

1. Responses 16 and 17 separately edit `tests/workspace.rs` and
   `tests/terminal_app.rs` before the corresponding implementation. The assistant
   explicitly describes adding failing coverage and verifying the intended failures.
2. Response 18 (usage line 267) issues both new focused tests. Tool outputs at lines
   269 and 271 exit 101: the controller does not ask the completion question and the
   terminal still selects the reviewer. This is observed RED, not an inferred failure.
3. Responses 20, 21 and 22 separately edit `src/workspace.rs`,
   `src/terminal_app.rs`, and `src/ui.rs`. Their raw usage is respectively 137,628,
   138,637 and 139,210 tokens; their outputs are only 1,280, 931 and 494 tokens.
   The same accumulating conversation is processed for each edit continuation.
4. Response 23 reruns both tests: controller passes, terminal still fails. Response
   24 adjusts the terminal test; response 25 reruns it and passes. Later initial
   implementation responses add more behavior, tests, checks and repairs.

One exact response example is
`resp_01d562340375954e016a9dc836dfb887d2a4a65dbf57cddce7` (line 297):
137,706 input, 135,552 cached input, 931 output. It emits the `src/terminal_app.rs`
edit. A large raw cost here does not mean that edit itself is 138,637 tokens long.

### Direct feature 3: repeated validation batteries

Responses 47–50, 53–55 and 60–62 run three batteries spanning the workspace,
terminal-app, UI-harness, terminal-UI and library test targets. Their **ten model
responses** total 1,768,172 raw tokens: 1,765,889 input (1,754,880 cached) plus
2,283 output. Exact commands and outcomes are preserved in the JSON.

These are not declared useless checks: formatting and edits occur between batteries,
and some checks discover real problems. The causal candidate is whether workload
partitioning and edit packaging lead to fewer such model-mediated cycles for the same
accepted work, not whether tests should simply be removed.

### WL feature 3: cohesive patch, focused repair, completion

The native source is
`/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-15-00-01a07825-376e-7661-a88d-4be15e65dd86.jsonl`.

| Response | Observed action | Input | Cached input | Output |
| --- | --- | ---: | ---: | ---: |
| 22, line 164 | One structured edit contains 4 source files and 3 test files | 99,480 | 97,664 | 15,019 |
| 23, line 181 | Resubmit corrected cohesive edit after a hunk mismatch | 102,744 | 57,728 | 10,357 |
| 24, line 194 | After patch acknowledgement, request focused `review_done_prompt` test | 107,184 | 101,760 | 543 |
| 25, line 207 | After exit 101, correct one terminal test assertion | 108,716 | 106,880 | 635 |
| 26, line 218 | After acknowledgement, request the narrower terminal completion test | 109,093 | 107,904 | 34 |
| 27, line 229 | After exit 0, emit `@work-leaf done` | 109,471 | 108,928 | 9 |

The first edit is rejected, so the audit retains that cost. The applied cohesive
submission is response
`resp_03fee1970559bc89016a9dbd76b47c87d28294ed77b14dd34c`; the subsequent focused
check is `resp_03fee1970559bc89016a9dbe35050c87d2a72688f4a1e475e3`; final `done`
is `resp_03fee1970559bc89016a9dbe57b67487d2a5dbd57caf060816`.

Through first ready-for-review, Direct has 67 responses and 18 native edit calls,
costing 9,742,633 raw tokens. WL has 27 responses across eight provider turns and
three structured-edit submissions (one rejected), costing 1,918,799 raw tokens.
These first-ready artifacts have **not** been shown to have equal intermediate
quality or scope. Both workflows subsequently incur reviews and fixes. Consequently
the 7,823,834-token checkpoint difference is an observed episode difference, not a
causal estimate of cohesive batching or a claim that the earlier artifact was done.

## Equal-scope compaction audit

All seven Direct and eight WL native thread rollouts were checked, not only the
largest Direct episode. Direct has 389 unique native response IDs; WL has 242.
Exactly one `compacted` record occurs: Direct feature 3 line 838, immediately after
response `resp_0b59e72c1bec5db3016a9dcce4c60c87d29322821e8d60b29f` at line 837.
That native response records 213,688 input, 212,352 cached input and 1,611 output,
or **215,299 raw tokens**. Native thread totals include it; the later legacy
`token_count`/CLI totals do not. This explains the one-record difference from the
historical 388 usage-advance proxy.

Every other thread's sum of unique native response usage equals its observer thread
counters. WL has no observed compaction marker, so this pilot cannot establish whether
the app-server raw-response ledger covers compaction. Its matching completed prefix
also does not resolve its four missing interrupted tails. The frozen pilot totals and
primary analyzer are unchanged; compaction scope requires a separate, equally applied
audit before whole-workflow claims in the fresh study.

## Interpretation and next hypothesis

This evidence supports testing the cohesive/buildable patch preference and translated
test timing as a larger candidate after the predeclared two-cue screen. A useful
intervention must change that preference consistently at both instruction sites while
preserving read/write mediation, ownership, required tests, review and final checks.
Removing one duplicated sentence while the other still directs cohesion is not a
clean removal of the mechanism. Permitting known-red shared-tree changes can interact
with concurrency and ownership, so a buildable smaller-batch treatment and a true
RED-before-implementation treatment must not be conflated.

The proposed causal chain remains a hypothesis:
**work packaging / validation scope → number and ordering of edit/check/read
continuations → repeated conversation input → raw tokens**. Ordinary run variation,
different intermediate solutions and interactions with review can also produce the
observed sequences. No percentage causal share is assigned.
