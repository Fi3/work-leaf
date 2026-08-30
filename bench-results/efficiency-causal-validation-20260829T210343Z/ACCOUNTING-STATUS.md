# Accounting Status

The normal concurrent Work Leaf endpoint is bounded, not exact. Strict replay finds 35 interrupted
responses without provable terminal usage across five of its six runs. A same-turn usage event
counts only when its cumulative total advances and its nonzero `last` usage fits inside that
advance. A later cumulative event covers an earlier interrupted response only when subtracting the
previous total and the later event's own `last` usage leaves a nonzero increase attributable to
exactly one unresolved interruption.

The normal Work Leaf mean is 17,471,532 recorded raw tokens with a conservative upper bound of
23,304,865. The exact direct mean is 36,116,382. The resulting normal raw-token reduction is
35.47%-51.62%. The uncached direction is unknown. The upper bound charges 1,000,000 raw tokens to
every unresolved final response, which exceeds the observed context-plus-output single-response
limit by 613,600 tokens.

These parts of this study remain exact:

- six direct sequential observations;
- three direct-read Work Leaf controls;
- three continued-response Work Leaf controls;
- three combined direct-read and continued-response Work Leaf controls;
- their saved provider histories and quality scores.

Tables that use the 17,471,532 normal Work Leaf value describe the recorded lower-bound scenario.
They do not include the conservative missing-response allowance. The current causal interpretation
and bounded allocation are in
`bench-results/efficiency-mechanism-attribution-20260830T081131Z/FINAL-REPORT.md`.
