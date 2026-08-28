# Next Steps For A Valid Efficiency Study

## Goal

Measure normal concurrent Work Leaf against fair normal direct sequential Codex on the same three
requests, then identify which Work Leaf mechanisms account for any real token difference. Preserve
every success, partial implementation, failure, and missing measurement. Analyze direct and Work
Leaf runs as independent groups rather than discarding one run when another fails.

## Current Evidence

Point 7 is complete in
`../efficiency-point7-bounded-accounting-20260828T142614Z/FINAL-RESULT.md`.

- Direct sequential Codex completed 3/3 features with 41,035,124 exact raw tokens.
- Normal Work Leaf completed 3/3 and is at least 19.15% lower after a conservative allowance for
  every interrupted response.
- Work Leaf with all three tested delivery mechanisms disabled completed 3/3 and is at least 4.73%
  lower under the same bound.
- One observation per condition cannot show whether that 4.73% difference is a repeatable residual
  or ordinary run-to-run variation. Whether the three mechanisms explain the average saving remains
  unknown.

Earlier exact percentages remain withdrawn because their Work Leaf totals omitted interrupted
responses. The completed bound proves a raw saving for the selected observations; it does not turn
those old percentages into valid measurements.

## What Is Verified

The fair benchmark setup can hold these controls constant:

1. the original task with `/status` and without `/fork`;
2. base commit `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
3. GPT-5.5 with xhigh reasoning for every provider thread;
4. normal validation behavior and the same final repository checks;
5. no recursive provider-verification sessions;
6. concurrent Work Leaf versus sequential direct Codex; and
7. the same frozen three-feature scorer, with every quality outcome retained.

Direct token accounting is also verified: every completed initial and resumed Codex invocation is
added once and reconciled with saved provider records.

## Remaining Measurement Limit

Normal Work Leaf immediately interrupts a provider response after a complete orchestrator directive.
On the current ChatGPT Codex transport, exact usage exists only on `response.completed`; interruption
produces no exact or cumulative usage for that response. Waiting for completion changes Work Leaf's
normal behavior and is not a fair accounting fix.

Real GPT-5.5/xhigh probes reproduced this with Codex CLI 0.149.1 and 0.150.1. Server-side account
usage was unavailable, and the same endpoint rejected stored and background responses. The complete
evidence and Codex source call chain are in
`../efficiency-point7-exact-accounting-20260828T113610Z/FAILURE-ANALYSIS.md`.

## Points 8 And 9

Point 8 tests candidate causes of the raw saving. Start from the previously observed candidates,
especially the number of model/tool cycles, and use controlled conditions that preserve normal
validation and task behavior. Each condition needs the same conservative interrupted-response bound
unless exact cancelled-response telemetry becomes available.

Point 9 repeats direct, normal Work Leaf, all-three-disabled Work Leaf, and any successful causal
condition as independent groups. It estimates normal variance, average feature completion, and the
average raw-token difference. Runs remain evidence even when another condition in the same launch
batch fails.

Exact raw and uncached percentages require a transport that reports usage for cancelled responses.
Until then, reports must use bounds for raw tokens and must not claim an uncached reduction.

Cross-project replication and other model profiles remain future work.
