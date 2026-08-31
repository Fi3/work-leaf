# Endpoint Audit

## Current Result

The current endpoint contains six direct sequential Codex runs and six normal concurrent Work Leaf
runs on the same frozen task:

| Workflow | Feature checks | Mean raw tokens |
| --- | ---: | ---: |
| Direct Codex | 17/18 | 36,116,382 exact |
| Work Leaf | 13/18 | 17,471,532-19,725,532 bounded |

The bounded raw-token reduction is 45.38%-51.62%. The uncached direction is unknown because 35
interrupted Work Leaf responses do not report their cached-input split.

Runs are independent group observations, not matched pairs. Every quality result is retained. The
quality difference prevents a formal equal-quality average claim, but the conservative raw bound
still establishes a large difference in the collected sample.

## Accounting

The recorded Work Leaf mean is a lower bound. A same-turn usage notification counts only when its
cumulative total advances and its nonzero `last` usage fits inside that advance. The observer
recovers an interrupted response from a later cumulative event only when subtracting the previous
total and the later event's `last` usage leaves a nonzero increase attributable to exactly one
unresolved interruption. Five runs contain 35 responses that do not meet these rules.

The raw event streams isolate exactly one response and zero intervening tool boundaries for every
unresolved gap. Each response receives the derived 386,400-token ceiling: the frozen Codex client
enforces a 258,400-token hard active-context limit and GPT-5.5 permits 128,000 output tokens. This
produces the 19,725,532 upper mean.
The full audit and calculation are in
`bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/FINAL-REPORT.md`.

## Fairness

Both groups use:

- the same base commit and three feature requests;
- GPT-5.5 with `xhigh` reasoning;
- normal validation freedom and the same final format, Clippy, and test checks;
- the same scorer for visual behavior, `/status`, and close/reopen behavior;
- complete workflow, quality, and measurement records without outcome-based retries.

Direct Codex uses its normal sequential workflow without Work Leaf. Work Leaf uses its concurrent
orchestrated workflow. The Work Leaf endpoint includes the recorded one-second provider-usage grace,
which delayed already requested interrupts by 15.0 seconds in total across 287 interrupts. This can
add tokens and alter later timing; it is a measurement limitation, not hidden from the result.

## Historical Cohort

An older three-versus-three cohort scored 8/9 feature checks in each group and showed the same
direction. Its Work Leaf accounting used the rejected later-cumulative assumption and different CLI
versions. It is retained only as a qualitative sanity check, not as an exact percentage or as input
to the final allocation.

## Conclusion

The endpoint audit establishes a bounded raw-token difference for this benchmark. It does not by
itself identify the cause or prove equal-quality population efficiency. The exact causal control and
bounded mechanism allocation are in
`bench-results/efficiency-mechanism-attribution-20260830T081131Z/FINAL-REPORT.md`.
