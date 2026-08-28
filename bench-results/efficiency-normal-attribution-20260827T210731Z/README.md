# Normal-Workflow Token Attribution Study

> **Superseded token evidence:** The corrected Point 7 investigation proved that this study's Work
> Leaf totals omit responses interrupted at orchestrator directives. All token percentages,
> usability labels, and allocation conclusions in this directory are withdrawn. Candidate quality
> outcomes and raw artifacts remain preserved. See
> `../efficiency-point7-exact-accounting-20260828T113610Z/FINAL-RESULT.md`.

## Abstract

This study asks two questions about the same frozen three-feature benchmark:

1. Does normal concurrent Work Leaf use fewer tokens than a fair normal direct sequential Codex
   workflow when both produce comparable implementations?
2. How much of the observed difference is associated with three concrete Work Leaf context-delivery
   mechanisms?

The three mechanisms are changed-file diffs, unchanged-file digests, and review context sent
directly by Work Leaf. The study runs every on/off combination of those mechanisms, including
`wl-111`, where all three are disabled. Any difference still remaining between direct Codex and
`wl-111` is reported as residual behavior; it is not assigned to these mechanisms.

The collection originally classified seven of the eight settings as exact completed observations.
That classification is withdrawn because usage from interrupted directive responses was absent from
every Work Leaf condition. The eighth, `wl-001`, also had one visibly missing telemetry record and
two interrupted replacements. The directory supports workflow and candidate-quality inspection, not
a token endpoint or three-factor allocation.

## Fixed Benchmark

- implementation base: `c92a0b7060a36eac6db2d869b85e589a7a9480f9`
- task SHA-256: `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`
- benchmark infrastructure base: `34f4ed98a34a159fb21e69f5daea5cf60fc824ce`
- isolated study instrumentation: `aa712768b5f3f06b37b930115661de89b83c8490`
- model: GPT-5.5
- reasoning: xhigh
- Codex CLI: the same installed binary for every run
- quality surface: visual selection/copy, selected-agent `/status`, and reviewed-patch close/reopen

The task does not require `/fork`, and `/fork` is not scored.

## Condition Names

Every Work Leaf condition is named `wl-abc`. Each digit is either `0` for normal Work Leaf behavior
or `1` for the less compact control:

| Digit | Mechanism | `0`: normal Work Leaf | `1`: controlled replacement |
| --- | --- | --- | --- |
| first | changed-file reread | send a diff | resend the full current file |
| second | unchanged-file reread | send only a digest | resend the full current file |
| third | review context | send exact context directly | make the reviewer reconstruct it from Git |

Examples: `wl-000` is normal Work Leaf. `wl-100` changes only changed-file rereads. `wl-111`
disables all three tested mechanisms. These names describe experiment settings, not product modes.

## Fairness

The direct and Work Leaf launchers are the green point-7 launchers at infrastructure commit
`34f4ed9`. Both workflows receive the exact same feature requests, GPT-5.5/xhigh profile, stage
timeouts, normal opportunities to validate their work, and the same final formatting, Clippy, full
test, replay-build, startup-smoke, token-accounting, and quality checks. Recursive provider sessions
are blocked for both workflows and are not counted.

The intended product difference remains: direct Codex handles the requests sequentially without
Work Leaf, while Work Leaf submits the requests concurrently and uses its normal orchestrator.
Direct Codex is not forced to run formatting, Clippy, or tests after every iteration.

The isolated instrumentation changes only what Work Leaf sends for the three controlled context
mechanisms. With no control selected, its default branches are the normal product branches tested by
the full repository suite. The production checkout is not modified.

## Collection Rules

- Run no more than two top-level workflows at once.
- Give every workflow its own checkout, build directory, temporary directory, observer identity,
  result directory, and run ID.
- Preserve every admitted success, partial implementation, failure, and missing value.
- Do not retry after a task reaches the provider. Analyze a missing report before considering a
  linked pre-admission retry.
- Inspect model, reasoning, accounting, recursive-call log, control environment, and observed prompt
  markers after every batch.
- Use absolute token amounts before percentages.
- Do not pair independently launched runs for analysis merely because they ran at the same time.

## Historical Analysis Method

The normal-product result compares all valid direct observations with all valid `wl-000`
observations, including the completed point-7 observations as a separately identified replication.
Feature results are retained alongside tokens so a lower-quality implementation is never presented
as an efficiency win.

The planned analysis assigned each mechanism the average increase in
tokens when that mechanism is disabled, averaged over all orders in which the three mechanisms could
be disabled. This is the three-factor Shapley allocation. The three contributions sum exactly to
`wl-111 - wl-000`. The residual is `direct - wl-111`.

That calculation was not valid because its input token totals were incomplete. The method remains
documented so a later study can reuse it after valid measurements exist. A mechanism with no
prompt-level opportunity in a run must still be reported as inactive rather than credited with a
numerical difference.

No Shapley calculation is supported. In addition to `wl-001` lacking a complete telemetry record,
all Work Leaf conditions lack usage for interrupted directive responses. Separately named attempts
and their failures remain preserved in `FAILURES.md`.

`TOKEN-RECOVERY-AUDIT.md` records the final offline search for the missing `wl-001` token value.
`PARTIAL-ALLOCATION.md` records the exact equations and conclusions supported by the seven measured
cells without estimating that value.

`PRELAUNCH-ANALYSIS-SHA256SUMS` preserves the original tool hashes. `ANALYSIS-SHA256SUMS` covers the
resulting study tools after replacement-path support and the saved-rollout token-total adjudicator
were added. Those post-launch changes do not alter candidates, prompts, token events, or quality
fixtures; `FAILURES.md` records why each change was needed and which tests guard it.

`SCHEDULE.tsv` is the frozen launch order. `STATE.md` records current progress, `FAILURES.md` records
problems before any retry, and `inspections/` records whether each saved observation is usable.
