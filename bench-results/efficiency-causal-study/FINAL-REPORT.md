# Efficiency Study Status

## Abstract

This directory records an investigation into why concurrent Work Leaf appeared to use fewer model
tokens than direct sequential Codex on a three-feature Rust task. The historical collection found
several plausible mechanisms, including compact repeated-file responses and direct review context.
It does not establish a normal-product token-saving percentage.

The historical full-workflow runs used an artificial validation rule that limited feature agents to
one focused Cargo command. Normal Work Leaf usage has no such restriction. Some runs also used a
later slash-command request containing `/fork`, although the original task required only generic
slash-command forwarding and was scored with `/status`. Those runs answer questions about the
artificial protocol, not normal Work Leaf versus normal direct Codex.

Later normal-workflow runs corrected the task, model profile, validation policy, and direct resume
accounting. The final exact-accounting gate is stored in
`../efficiency-point7-exact-accounting-20260828T113610Z/FINAL-RESULT.md`. It found a remaining
transport limit: normal Work Leaf interrupts model responses at orchestrator directives, but the
ChatGPT Codex transport exposes usage only for completed responses. Earlier Work Leaf totals
therefore omitted interrupted-response tokens.

Point 7 is completed by a conservative bound in
`../efficiency-point7-bounded-accounting-20260828T142614Z/FINAL-RESULT.md`. It charges every omitted
response 400,000 raw tokens. Normal Work Leaf remains at least 19.15% below exact direct Codex, and
Work Leaf with all three tested mechanisms disabled remains at least 4.73% below. All selected
candidates passed 3/3.

## Supported Findings

- The original three requests can be scored independently in saved candidate histories.
- `/status` is a concrete test of the original generic slash-command request. `/fork` is excluded.
- Workflow completion and requested-feature completion are different measurements and must both be
  reported.
- Changed-file diffs, unchanged-file digests, and direct review context remain plausible token-saving
  mechanisms in their small controlled traces.
- Percentages from those small traces have different scopes and cannot be added or allocated to a
  whole-workflow difference.
- Direct sequential Codex can be measured exactly by adding every completed launch and resume
  invocation.
- Normal Work Leaf cannot be measured exactly on the current ChatGPT Codex transport because usage
  for interrupted directive responses is unavailable.
- A conservative upper bound proves a raw-token saving in the selected fair 3/3 observations.
- No current evidence supports a precise average reduction, an uncached reduction, or formal
  population-level quality equivalence.

## Historical Names

Historical Work Leaf conditions use `wl-XYZ`. `X`, `Y`, and `Z` identify changed-file reread,
unchanged-file reread, and review-context settings. `0` means normal Work Leaf behavior and `1`
means the less compact control. For example, `wl-110` returns full files for both reread types while
still supplying review context directly. These labels describe archived artificial-protocol runs;
they are not current normal-product results.

`R19`, `step190`, and similar names are archive sequence labels, not model versions, feature counts,
or sample sizes. Raw artifacts, `evidence.json`, and `verify.py` remain available for audit. They
must not be mixed into a new normal-workflow result.

## Current Conclusion

The original headline reductions of 72.0567% raw tokens and 49.0420% uncached tokens are excluded
from the normal-workflow question. They describe one unequal-quality comparison collected under the
artificial protocol. The later 49.039% raw and 26.256% uncached Point 7 reductions and subsequent
attribution percentages are also withdrawn because their Work Leaf totals omit interrupted-response
usage.

The corrected direct run completed all three features with exact totals of 41,035,124 raw and
1,982,580 uncached tokens. A later analysis selected independent normal and all-three-disabled Work
Leaf candidates that also completed all three features. After applying the conservative missing-
response bound, normal Work Leaf is at least 19.15% lower in raw tokens and all-three-disabled Work
Leaf is at least 4.73% lower. This proves a raw saving for those observations. It does not determine
whether the three tested context-delivery mechanisms explain the average saving because the
all-three-disabled result is a single observation and may reflect normal run-to-run variation.

The exact percentage, uncached percentage, population average, and remaining causal allocation are
still unknown.

The required work before another paid batch is listed in `NEXT-STEPS.md`.
