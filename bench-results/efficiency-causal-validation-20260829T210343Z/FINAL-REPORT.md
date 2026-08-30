# Final Report: Causal Validation Controls

## Abstract

This study collected controlled Work Leaf workflows to test file-read mediation, directive
interruption, and their combination. Those control runs remain valid and have complete provider
usage. The normal Work Leaf endpoint used for the original comparison does not: corrected
accounting finds ten interrupted responses without terminal usage.

The normal endpoint is therefore bounded. Work Leaf averaged 17.47-18.14 million raw tokens versus
36.12 million for direct Codex, a 49.78%-51.62% reduction in the collected samples. The uncached
direction is unknown. Direct completed 17 of 18 feature checks and Work Leaf completed 13 of 18, so
the all-run result is not a formal equal-quality comparison.

Raw tokens mean input plus output, including cached input. Uncached tokens mean fresh input plus
output.

This directory preserves the controls and their history. The current causal answer, including the
exact orchestration control and bounded allocation, is
`bench-results/efficiency-mechanism-attribution-20260830T081131Z/FINAL-REPORT.md`.

## Valid Evidence

These measurements remain exact:

| Condition | Runs | Mean raw tokens | Feature checks |
| --- | ---: | ---: | ---: |
| Direct reads with normal Work Leaf interruption | 3 | 19,220,509 | 9/9 |
| Mediated reads with completed responses | 3 | 22,517,835 | 6/9 |
| Direct reads with completed responses | 3 | 19,399,622 | 8/9 |

The controls use the frozen task, base commit, GPT-5.5/`xhigh`, normal validation freedom, final
checks, and quality scorer. Their incoming provider requests, rollout identities, and cumulative
totals were preserved. No unresolved interrupted response enters these three means.

The combined-control result shows that direct reads plus completed responses do not remove the
large difference from direct Codex. It does not by itself prove which remaining Work Leaf mechanism
causes the difference. That question is answered by the later compact-direct versus sequential Work
Leaf control.

## Withdrawn Interpretation

The earlier endpoint analyzer assumed that any later cumulative usage event included an earlier
interrupted response. That assumption is false when the cumulative increase equals the later
event's own `last` usage. In that case there is no token increment attributable to the interrupted
response.

`decompose.py` therefore refuses to regenerate an exact endpoint cycle/context decomposition.
`decomposition-evidence.json` records that refusal and points to the bounded current analysis. Old
single-value normal-endpoint percentages are lower-bound scenarios, not exact measurements.

## Current Result

The exact main mechanism control in the later study compares compact direct Codex with sequential
Work Leaf while holding scheduling, reads, response completion, validation, and compact
linearization targets fixed. It measures 35.66 million versus 19.31 million raw tokens. Saved
provider histories show 311 versus 198 model generations, with the largest reduction during
implementation and review.

After propagating the normal endpoint bound, Work Leaf's orchestration protocol plus mediated reads
and early interruption under the recorded measurement grace explain 97.95%-98.02% of the observed
raw-token gap.

## Evidence Map

- `ACCOUNTING-STATUS.md`: current endpoint accounting.
- `01-ENDPOINT-AUDIT.md`: preserved endpoint and quality history, with the lower-bound warning.
- `03-CONTROL-DESIGN.md` through `08-COMBINED-CONTROL-RESULT.md`: control design and collection.
- `09-FINAL-HYPOTHESIS-AUDIT.md`: alternatives checked during this study.
- `decomposition-evidence.json`: explicit rejection of the former exact decomposition.
- `combined-evidence.json`: preserved control data; its normal endpoint rows are recorded lower
  bounds and are corrected by the later mechanism study.
