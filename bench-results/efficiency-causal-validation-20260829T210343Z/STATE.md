# Study State

## Current Step

The causal study is complete. The endpoint audit, decomposition, direct-read control,
continued-response control, combined interaction control, final hypothesis audit, and plain-language
report are all recorded in this directory.

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
- Letting resumed provider output finish raises mean raw use by 5.05M tokens, or 27.07% of the
  current endpoint gap. It raises mean provider usage changes by 47 and context per change by 4,492
  tokens.
- The continued-response runs pass all workflow and accounting gates and complete 6/9 frozen
  feature checks. Eight turns activate fully; one additional turn reaches the 120-second bound and
  falls back to the original interrupt with exact later cumulative usage.
- Direct-read traces resume output much more often than normal mediated-read traces. The two causal
  fractions cannot be added until their interaction is measured.
- Three combined controls passed all workflow, accounting, activation, and candidate-replay gates.
  They completed 8/9 frozen feature checks and averaged 19.40M raw tokens.
- Direct reads and continued responses together move 1.93M raw tokens, 10.34% of the current endpoint
  gap. Their interaction is negative 4.87M, so separate percentages are not additive.
- Combined Work Leaf remains 46.29% below direct Codex. The remaining gap is 17.17M cached input
  tokens, while combined Work Leaf receives more uncached input and emits more output.
- Fewer provider generations explain 76.62% of the residual input gap arithmetically; smaller context
  per generation explains 23.38%. Provider histories show substantially fewer separate writes,
  repeated commands, and validation commands.

## Next

No required study step remains. `NEXT-STEPS.md` lists optional replication, cross-project, and
individual-mechanism controls. The latter require explicit authorization because they would change
Work Leaf behavior or prompts.
