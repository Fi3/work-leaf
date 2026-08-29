# Study State

## Current Step

The endpoint audit, offline decomposition, and three-run direct-read control are complete. The
current step is auditing an immediate-interruption control for the remaining raw-token gap.

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
- All three direct-read controls passed the workflow, final checks, candidate replay, and all three
  frozen feature fixtures. Their exact mean is 19.22M raw and 1.61M uncached tokens.
- Direct reads add 1.75M raw tokens over the six-run normal Work Leaf mean, or 9.38% of the current
  direct-versus-normal raw gap. The full-quality subset gives a larger but imprecise 23.04% sample
  fraction.
- Direct reads add 264,000 uncached tokens, accounting for 99.49% of the current uncached gap.
- Direct reads increase context per provider usage change but do not increase the number of usage
  changes. Work Leaf still uses 46.78% fewer raw tokens than direct Codex with direct reads enabled.

## Next

1. Audit a benchmark-only control that lets provider turns finish naturally after a complete
   orchestrator directive while leaving Work Leaf, task text, validation, review, and scoring fixed.
2. Reject the control before launch if it can race later turns, hide output from accounting, or
   change anything besides directive interruption.
3. If the control is valid, run three independent workflows concurrently and apply the same
   activation, exact-accounting, and quality gates before interpreting the remaining raw gap.
