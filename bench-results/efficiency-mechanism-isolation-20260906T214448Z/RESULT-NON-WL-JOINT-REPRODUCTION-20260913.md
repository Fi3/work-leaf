# Verified non-WL joint mechanism and completed cost reproduction

## Answer and exact scope

**A complete workflow without the WL orchestrator used 18,545,304 raw tokens:
48.6513% below the historical six-Direct mean of 36,116,381⅔.** It completed
all three original feature/review chains, integration, three final commits,
formatting, strict Clippy and the full repository test suite in 58m24.72s.
Native, public and observer ledgers agree exactly, and independent review passes.

The verified upstream mechanism is a **joint work-publication and feedback
contract**, not a magical discount per call:

1. The author assembles a buildable implementation and its tests into a cohesive
   proposal, rather than being required to execute a deliberately failing test
   before it can publish implementation.
2. An external host validates and applies the entire proposal, commits it and
   gives the same author thread an actual acceptance receipt. Source inspection
   stays available; the model does not itself manage source writes or commits.
3. That receipt directs the next action toward a relevant focused check and
   completion after success, while explicitly allowing repairs and remaining
   requirements. Real command results, including failures, return to the author.
4. Reviewers still inspect the result, request real repairs and recheck them;
   final integration still performs the required full checks.

All three initial authors in this run actually followed that sequence, each
with one accepted implementation-plus-tests edit. The mechanism changes the
unit of work and the decision immediately after acceptance/checking. Fewer
generation/history-processing steps are a consequence, not the explanation.

**This verifies the mechanism, its historical exposure and one full non-WL
reproduction of the observed magnitude. It does not identify the percentage of
the old six-versus-six gap causally attributable to that package.** That final
same-target attribution and bounded residual remain G03, not a completed claim.
The two numerical questions must remain separate:

| Question | Verified answer |
| --- | --- |
| How far below the saved Direct average was the completed non-WL run? | 48.6513%, exact recorded accounting. |
| Does that reproduce the approximate historical cost magnitude? | Yes, for this one complete observation. |
| What fraction of the historical saving is jointly causally identified? | Not established; matching cross-cohort totals is not that estimate. |

The user's explicit no-quality-loss working assumption governs the comparison.
Actual feature failures remain visible below; no expensive outcome is excluded.

## Historical source and actual work-level witnesses

Historical source commit `5b1d1ef9590850faed26052f909ddff7ff8f127d` supplies
the chain: `src/agent.rs::PromptPolicy::for_read_permission` and
`concurrent_instruction_translation` couple tests with buildable implementation;
`src/orchestrator.rs` around lines 2641–2644 supplies the accepted-edit focused
validation/handoff instruction. All 18 historical WL initial launch deliveries
contain the coupled contract. Direct retains executed RED before implementation.
Both systems commit work; automatic Git commits alone are not a WL-only cause.

The source-pinned [initial implementation analysis](RESPONSE-CAUSE-INITIAL-IMPLEMENTATION.md)
and [test-timing exposure evidence](EVIDENCE-H-TEST-TIMING.md) contain exact
native/event locators and source identities. Two concrete contrasts matter:

- Historical Direct point7 visual runs a missing-API RED, then publishes eight
  dependent production patches before its first passing check. The patches
  progressively introduce helpers and adapter wiring: they are not eight real
  failed-test repair cycles. WL004 visual instead publishes one 30,582-byte
  core/adapters/tests proposal, receives acceptance, checks and returns DONE.
- Historical Direct003 slash reaches an initial controller GREEN, then still
  has real selected-terminal routing to implement. WL004's first cohesive
  proposal includes the frontend route; it still repairs a genuine negative-test
  failure. Therefore post-GREEN work is not all unnecessary work, and simply
  cutting off a Direct author at its first GREEN would not reproduce the method.

Across historical WL initial stages, first successful focused validation is
followed by DONE in all 18 observations. This is an observed handoff response
to the contract, not proof that those focused commands cover every requirement.
The saved static-instruction-only experiment retains fragmented native edits
in four of nine initials and achieves a smaller whole-cost difference. Wording
alone is not established as a substitute for the complete operational package.

The current [mechanism screen](phases/standalone-global-hunk-pilot-01/postcapture/MECHANISM-SCREEN.md)
grounds each submitted proposal, actual host event, validation and next response.
It also retains five real rejected proposals, eight real author checks and all
review/fix work. There is no rule forcing a successful first proposal or DONE
after one check regardless of outstanding defects.

## Why this changes raw token usage

Raw tokens are `sum(input + output)` over distinct charged responses. Cached
input is already inside input; reasoning is already inside output. A response
that revisits a growing conversation charges its retained input again even
when that input is cached. Cohesive publication avoids intermediate partial
implementation/feedback histories and their continuation decisions; the actual
acceptance/check handoff makes completion an explicit next action instead of
leaving the author to decide which additional work or reporting to perform.

That is an operational pathway to fewer repeated input charges and generated
continuations. It is not valid to remove every token of a historical intermediate
response retrospectively: regrouping also changes the later conversation and
implementation. The measured whole-workflow result includes those downstream
effects. Neither RED alone, the acceptance sentence alone, nor the private hunk
correction alone has been assigned the 48.6513% effect.

In this run, input is 18,379,041 and output is 166,263. Cached input is 17,324,672;
uncached input plus output is 1,220,632. **Raw reduction is not a claim of the
same percentage reduction in subscription quota consumption or uncached work.**

## Complete accounting and historical reconciliation

The [native ledger](phases/standalone-global-hunk-pilot-01/postcapture/NATIVE-USAGE-SUPPLEMENT.json)
and [independent audit](phases/standalone-global-hunk-pilot-01/postcapture/NATIVE-INDEPENDENT-REVIEW.md)
join 191 unique native response IDs to all 31 completed public turns in seven
threads. There are no compaction markers, duplicate responses or recorded
unfinished turns. Both integration turns are included. No cumulative counter
is added as a new response, and no failed proposal is subtracted.

| Work | Recorded raw tokens |
| --- | ---: |
| Three initial authors | 4,308,399 |
| Author fixes | 3,050,077 |
| Reviews and rechecks | 4,259,299 |
| Integration plan and acceptance | 6,927,529 |
| Total | 18,545,304 |

Historical Direct total is 216,698,290 across six workflows. The exact
comparison is `1 − 18,545,304 / (216,698,290 / 6) = 48.65126808337989%`.
The observed difference is 17,571,077⅔ raw tokens. Its descriptive partition is
10,894,924⅓ in initial stages and 6,676,153⅓ in later stages. These are mutually
exclusive accounting windows, **not independently identified causal shares**.

Historical WL's recorded mean is 17,471,532; its original conditional upper
mean is 19,725,532. The non-WL observation lies inside that cost range. The
historical 45.38%–51.62% reduction remains conditional on the original maximum
assigned to 35 missing WL charges. That maximum is not independently established
as an upstream provider request cap. No recovered correction establishes that
the historical gap disappears.

Historical CLI versions are 0.149.1 for the first three Direct observations and
0.150.1 for the other three Direct and all six WL observations; this pilot uses
0.153.4. GPT-5.5/xhigh and the original task/base identities are retained, but
CLI/calendar/provider-state and stochastic sample comparability are not made
identical by those facts. There is no new ordinary control or API-key run.

## Reproducing the operational mechanism without WL

The complete exercised sources are retained under
`phases/standalone-global-hunk-pilot-01/infrastructure/drivers/`:

- `bench-three-features-serialized-host-custody`: `host_author_policy`,
  `implementation_prompt`, `run_host_author`, review/fix and integration flow.
- `host_custody.py`: `parse_operations`, `plan_edit`, `Host.apply`,
  `Host.command`, `invoke` and `main`.
- `bench-candidate-common`: artifact/checkout retention and publication.

The author host is Python plus Git/real shell checks and the subscription Codex
CLI; no WL library, orchestrator or controller runs the author workflow.
The work-leaf binaries produced at the end are benchmark output, not its host.

The exercised host entry point has this argument structure:

```text
python3 host_custody.py
  --repo <separate-clean-checkout>
  --feature <feature-number>
  --stage <unique-stage-label>
  --prompt-file <task-and-host-contract>
  --artifact-dir <new-owned-stage-directory>
  --codex-bin <subscription-only-wrapper>
  --deadline <monotonic-absolute-seconds>
  --model gpt-5.5
  --serialized-feedback
  [--resume-thread <the-same-author-thread-for-a-fix>]
```

The real driver supplies these values, the pinned xhigh profile, the original
tasks and later reviewer/integration stages. `main` requires an explicit
deadline. A reproduction needs a fresh identity and checkout; the frozen
admission is one-shot and must not be reused or overwritten. This recipe
documents the actual tested path, not authorization for another observation.

The host accepts one complete owned-final operation at a time. Edits are
prevalidated, applied and committed before acceptance feedback; checks execute
and return their real status/output before another operation. Pending work,
rejections and failed commands remain visible. There is no scripted answer to
the feature tasks, ignored review finding, forced final success or truncated
agent response. Instructions and feedback remain in the saved actual prompts.

The corrected private matcher searches a unique exact old block across the
evolving file for a bare `@@` hunk, while keeping explicit context anchored.
An actual accepted five-file visual proposal fails at a unique old block
behind the previous host's cursor and plans successfully under this host.
The read-only replay proves this correction is active. Other missing/ambiguous
and duplicate-file proposals still fail. This removes an experimental host
obstruction; it is **not a demonstrated error in the historical benchmark**.

## All outcomes, feature qualification and verification

The accepted observation is
`standalone-global-hunk-pilot-001`, 2026-09-12 23:27:08.984780 to
2026-09-13 00:25:33.707328 UTC. Driver and final repository gates pass.
The final HEAD is `b66836d950de093529f63d24cbc6265e2bec517b`, based on
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`.
The final full test suite reports 193 tests passed; the candidate startup/quit
smoke exits successfully. The frozen report and commit bundle remain intact.

The separate frozen feature scorer passes visual and status but fails
completion at line 23: the question is appended to the author chat, but the UI
still selects the reviewer. The implementation's own test clicks the author
first. Historical WL004 and WL006 have the identical failure frame and same
source-level missing selection transition. This is a recurring specimen
defect, not evidence that the treatment explicitly removed a requirement.
Later frozen close/reopen assertions are unreached, not passing.
The [saved-source diagnostic](phases/standalone-global-hunk-pilot-01/postcapture/COMPLETION-FAILURE-DIAGNOSTIC.md)
documents the exact chain. No scorer, task, result or original source is changed
to improve the score; the user's quality premise is explicit, not inferred.

Earlier package outcomes are not replacements for, or silently filtered out of,
this result:

| Earlier condition | Actual retained result |
| --- | --- |
| Static author contract | Three full workflow passes, mean 28,173,945 raw; 21.9912% below saved Direct. Not this dynamic host condition. |
| Bounded dynamic package | Three wall failures; 37,991,570 recorded native raw in total plus unknown tails. Initial stages and seven review-closed chains do not make full workflow passes. |
| Previous completion pilot | One 90-minute wall failure; 18,467,802 native raw including 235,169 compaction raw; unfinished integration tail unknown. Five large proposals hit the private forward-cursor obstruction. |
| Corrected global-hunk pilot | The single complete observation in this report; every failed proposal, check, repair and integration charge included. |

These conditions have different host fidelity/budgets and are not pooled into
a favorable new mean. One corrected-host success does not establish its
repeatability or success rate. The [independent custody review](phases/standalone-global-hunk-pilot-01/postcapture/CUSTODY-QUALIFICATION.md)
verifies all 61 frozen inputs, published source/bundle, all 117 resource
samples and the exact owned trust-entry configuration qualification. Original
drift flags and the resolved accounting-wrapper sorting false positive remain
in their original records.

Normal WL source/public APIs are untouched. The private runner/host/retention
checks and root `cargo fmt`, strict all-target/all-feature Clippy and full tests
pass. Real-agent verification is the completed subscription-backed three-feature
workflow itself; it exercises the corrected matcher and actual review/fix/
integration paths, not a fake backend. No API credits or copied credentials.

## Acceptance and remaining requirement

The [independent final acceptance assessment](phases/standalone-global-hunk-pilot-01/postcapture/FINAL-ACCEPTANCE-INDEPENDENT.md)
supports G01 for the verified historically exposed upstream mechanism and G02
for its complete one-observation non-WL reproduction, qualified exactly as above.
Neither acceptance asserts an exhaustively allocated historical saving.

G03 remains the specific unfinished deliverable: a **same-target joint effect
and defensible residual bound for the historical six-versus-six gap**. The
population/unit arithmetic, actual downstream cost and identified limitations
are recorded; a causal residual bound is absent. Single-factor contrasts cannot
be added, and the numerical match here cannot substitute for that bound.
No source-backed benchmark correction currently makes the historical gap vanish.

Three generic repetitions could test repeatability of this package, but would
not alone fix historical comparison or residual identification. They must not
be sold as a guaranteed way to finish G03. Further generation requires a concrete
discriminating design and prospective admission, not another uninformative
long batch. The counter keeps the full analysis unfinished while G03 is open.
