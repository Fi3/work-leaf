# Study State

## Current Step

The endpoint audit, offline decomposition, and direct-read control audit are complete. The current
step is the three-run direct-read causal batch.

## Completed

- The exact six-run study established that later cumulative provider totals recover interrupted
  response usage.
- The initial arithmetic audit found that cached input accounts for 98.58% of the observed raw-token
  difference; this is a lead, not yet a causal conclusion.
- Command counts were rejected as a standalone causal measure because file reads are represented
  differently in the two workflows.
- The three uninstrumented Work Leaf captures reconcile to final hash-verified provider totals with
  no estimate. Their group completed 8/9 features, matching the direct group.
- Work Leaf averaged 13.99 million raw tokens versus 35.20 million for direct Codex, a 60.25%
  reduction. It averaged 1.09 million uncached tokens versus 1.74 million, a 37.58% reduction.
- Every Work Leaf observation is below every direct observation for both token measures. The sample
  remains small and its historical Work Leaf runner versions differ, so it is a sanity check rather
  than a precise current-version population estimate.
- Cached input accounts for 98.58% of the current detailed raw-token gap. Work Leaf does not save
  primarily by emitting less reasoning or final output.
- Work Leaf has 33.68% fewer distinct provider usage changes and 27.45% less input context per
  change in the current detailed cohort. The older balanced cohort independently shows 45.03% fewer
  changes and 27.88% less context per change.
- Fewer usage changes describe 56.00% of the current input gap and smaller context describes 44.00%
  when their interaction is split equally. This arithmetic is not yet a causal feature allocation.
- Implementation and review-fix work contains 76.48% of the raw gap. Review and linearization are
  too small to explain the result alone.
- The existing `--no-read-permission` mode has been traced from `bench-three-features` through
  `src/cli.rs` into `src/agent.rs::PromptPolicy`. Despite its historical name, the switch enables
  direct agent reads while preserving structured writes, concurrent routing, validation, review,
  linearization, task, model, scorer, and accounting.
- Earlier one-run full-reread controls are non-monotonic and cannot establish a causal percentage.
  Three independent direct-read runs are required before interpreting the read route.

## Next

1. Run three isolated direct-read Work Leaf workflows concurrently.
2. Check direct-read activation, exact provider accounting, model settings, and feature quality for
   every outcome before comparing tokens.
3. Compare their cycle count, context per change, tokens, and quality with the six normal detailed
   Work Leaf runs, then decide whether another causal control is justified.
