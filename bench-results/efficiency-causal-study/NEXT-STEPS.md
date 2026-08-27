# Next Steps For A Valid Efficiency Study

## Goal

Measure normal concurrent Work Leaf against fair normal direct sequential Codex on the exact same
three requests, then estimate which Work Leaf mechanisms account for any token difference. Preserve
every success, partial implementation, failure, and missing measurement.

## Current Evidence

The historical percentages are excluded because their workflows were artificially restricted and
some task text included `/fork`. The first normal-workflow pilot is in
`../efficiency-fair-normal-workflow-pilot-20260827T115642Z`. It completed one workflow per condition
and exposed measurement defects before a larger batch:

- descendant real-agent verification calls inherited GPT-5.6 Sol instead of GPT-5.5;
- direct `codex exec resume` usage was treated as cumulative when each invocation reports a fresh
  total;
- the live state file showed zero admitted workflows until finalization;
- Work Leaf scored 2/3 and direct Codex scored 3/3, so the pair is not an equal-output comparison.

## Before Another Provider Run

1. Pin `gpt-5.5` and `xhigh` in the run-local wrapper for every Codex invocation, including commands
   launched by benchmarked agents. Do not read or modify `.codex/config.toml`.
2. Count direct CLI usage once per invocation and sum all implementation, review, correction, and
   linearization invocations. Keep app-server thread accounting cumulative for Work Leaf.
3. Make rollout extraction recognize that resumed direct turns start a new usage total rather than
   requiring the last turn to equal the complete thread sum.
4. Add regression tests reproducing the saved pilot's launch-plus-resume pattern. Run all repository
   and observer checks locally.
5. Update the live state file immediately after each provider admission.
6. Reanalyze the saved pilot capture offline. Preserve the original reports and write corrected
   diagnostics separately.

## Next Paid Gate

Run one new pair only after the fixes above are green:

- one normal concurrent Work Leaf workflow;
- one normal direct sequential Codex workflow;
- exact original task, with `/status` and without `/fork`;
- fixed base `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
- GPT-5.5 with `xhigh` reasoning for every provider thread;
- at most two top-level workflows at once;
- no artificial validation-command limit;
- no retry after a task reaches a provider thread.

Stop after scoring that pair. A larger batch starts only after confirming that model strata,
per-invocation accounting, candidate replay, quality scoring, and live status are all correct.

## Later Collection

For the normal-product comparison, treat all independently launched direct runs as one group and
all independently launched normal Work Leaf runs as another group. Do not invent one-to-one pairs
after collection. Report token distributions, each feature's pass rate, and total feature score
together. Unequal-quality runs remain evidence rather than being discarded.

Mechanism allocation requires a separate randomized study of the eight Work Leaf settings for
changed-file rereads, unchanged-file rereads, and review context. Run that study in small batches
only after the normal-workflow gate is reliable. Absolute raw and uncached token amounts come before
percentages, and uncertainty must be reported from the collected observations.

Cross-project replication and other model profiles remain future work.
