# Study State

## Current Step

The endpoint audit is complete. The current step is provider-free decomposition of the token gap by
token class, workflow stage, provider usage changes, and context carried through each change.

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

## Next

1. Split the exact gap into cached input, uncached input, and output.
2. Measure where the gap occurs and whether it comes from fewer provider cycles, smaller context per
   cycle, or both.
3. Challenge the leading explanation against quality, timing, hidden-thread, and accounting
   alternatives before selecting a paid control.
