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

## Provider-Run Infrastructure

The next one-pair gate uses these fixed controls:

1. The run-local wrapper pins GPT-5.5 and xhigh without reading or modifying `.codex/config.toml`.
2. Both benchmark paths prohibit recursive real-agent provider sessions while retaining normal tests
   and validation. The wrapper blocks and records any attempted recursive Codex launch.
3. Direct CLI usage is counted once per invocation and summed across launch and resume commands.
   Work Leaf app-server usage remains cumulative per conversation.
4. Direct rollout reconciliation adds the final usage from each task epoch and requires the result to
   match the captured CLI invocation sum.
5. The pilot state file records each provider admission immediately.
6. Regression tests reproduce the first pilot's launch-and-resume pattern. The complete saved direct
   capture reanalyzes to 35,947,089 raw and 1,353,041 uncached tokens with all 15 rollout conversations
   matched and no accounting errors.

## Next Paid Gate

Run one new pair only after the fixes above are green:

- one normal concurrent Work Leaf workflow;
- one normal direct sequential Codex workflow;
- exact original task, with `/status` and without `/fork`;
- fixed base `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
- GPT-5.5 with `xhigh` reasoning for every provider thread;
- at most two top-level workflows at once;
- no artificial validation-command limit;
- no recursive provider-verification sessions;
- no retry after a task reaches a provider thread.

Stop after scoring that pair. A larger batch starts only after confirming that model strata,
per-invocation accounting, candidate replay, quality scoring, and live status are all correct.

## Latest Gate Outcome

The one-pair gate in
`../efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z` is complete but not green. It cannot
support a token comparison:

- direct Codex could not start write-capable tools because its nested sandbox registry was mounted
  read-only by the outer environment; and
- Work Leaf's active feature-3 turn was terminated by the 30-minute visible-state stall guard even
  though the captured provider stream was still growing.

The next one-pair gate requires isolated writable provider temporary directories for both conditions,
a bounded real workspace-write smoke for the direct path, and busy-progress detection that recognizes
active app-server capture growth. The original task, Work Leaf implementation, direct workflow,
quality scorer, GPT-5.5/xhigh profile, and fixed base remain unchanged. Steps 8 and 9 stay stopped
until that replacement gate is green.

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
