# Accounting Status

The normal concurrent Work Leaf endpoint is bounded, not exact. Strict replay finds 35 interrupted
responses without provable terminal usage across five of its six runs. A same-turn usage event
counts only when its cumulative total advances and its nonzero `last` usage fits inside that
advance. A later cumulative event covers an earlier interrupted response only when subtracting the
previous total and the later event's own `last` usage leaves a nonzero increase attributable to
exactly one unresolved interruption.

The normal Work Leaf mean is 17,471,532 recorded raw tokens with a conservative upper bound of
19,725,532. The exact direct mean is 36,116,382. The resulting normal raw-token reduction is
45.38%-51.62%. The uncached direction is unknown. Raw-event replay isolates one response and zero
intervening tool boundaries for every gap. The upper bound charges the derived maximum of 386,400
raw tokens to each response: the frozen client enforces a 258,400-token hard active-context limit
and GPT-5.5 permits 128,000 output tokens.

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
