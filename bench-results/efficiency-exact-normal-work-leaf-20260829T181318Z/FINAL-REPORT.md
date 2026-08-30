# Bounded Normal Work Leaf Follow-up

## Abstract

This study compares six normal concurrent Work Leaf runs with six normal direct sequential Codex
runs on the same three-feature Rust task. Every run used GPT-5.5 with `xhigh` reasoning, the same
base commit, normal validation freedom, the same final checks, and the same feature scorer.

The original analysis called all six Work Leaf token totals exact. That was wrong. A later
cumulative thread total proves an earlier interrupted response was counted only when the arithmetic
contains an extra token increase that cannot belong to the later response. The corrected observer
finds ten unresolved interrupted responses across five runs; only one Work Leaf run is exact.

Using a conservative allowance of 400,000 raw tokens for every unresolved response, Work Leaf
averaged between 17.47 and 18.14 million raw tokens. Direct Codex averaged 36.12 million. Work Leaf
therefore used between 49.78% and 51.62% fewer raw tokens in these samples, even under the maximum
allowance. The raw-token reduction is proven for this collected group, but its exact percentage is
not known.

The average feature score differs: direct Codex completed 17 of 18 checks and Work Leaf completed
13 of 18. The result is not an equal-quality average comparison. The uncached-token result is also
inconclusive because the missing responses do not report their cached-input split.

## What Was Compared

| Workflow | Runs | Scheduling | Model | Reasoning | Feature checks |
| --- | ---: | --- | --- | --- | ---: |
| Direct Codex | 6 | normal sequential | GPT-5.5 | `xhigh` | 17/18 |
| Work Leaf | 6 | normal concurrent | GPT-5.5 | `xhigh` | 13/18 |

The task asks for visual selection and copy, `/status` forwarding, and reviewed-patch close/reopen.
`/fork` is not part of the task and is not scored. Runs are independent group observations, not
matched pairs. No success, partial implementation, or measurement gap was discarded.

Both workflows use the same base commit, task text, model, reasoning level, time allowances, normal
validation freedom, final format/Clippy/test gate, and scorer. Direct Codex does not use Work Leaf.
Work Leaf uses its normal concurrent implementation.

The Work Leaf observer held an already requested interrupt for at most one second while waiting for
provider usage. This changes interrupt timing and can permit extra generation after a directive.
That observed work is counted. Across 287 interrupts, the combined delay was 15.0 seconds. The
incoming and forwarded request bytes match in all six captures.

## Accounting Correction

Codex reports `tokenUsage.total` as a cumulative total for a provider thread and `tokenUsage.last`
as the usage of the response that produced that event. The earlier analyzer treated any later
cumulative event on a thread as proof that a prior interrupted response was included. That is not
enough.

For example, if the next cumulative total rises by exactly the next event's `last` value, the
increase contains no tokens attributable to the earlier interruption. The corrected rule recovers
an interrupted response only when all of these facts hold:

1. The next cumulative total is later than the interrupted directive.
2. Exactly one unresolved interrupted response lies between the surrounding cumulative totals.
3. Subtracting the previous total and the next event's `last` value leaves a nonzero token increase.

The implementation is in
`bench-observer/src/lib.rs::later_cumulative_usage_proves_interrupted_turn`. The regression test is
`bench-observer/tests/proxy.rs::later_cumulative_usage_without_an_unreported_increment_does_not_cover_interrupted_turn`.

Replaying the six saved provider streams with that rule gives one exact run and five bounded runs:

| Work Leaf run | Recorded raw lower bound | Conservative raw upper bound | Unresolved responses | Features |
| --- | ---: | ---: | ---: | ---: |
| `exact-normal-001` | 20,221,714 | 20,621,714 | 1 | 2/3 |
| `exact-normal-002` | 13,214,206 | 14,414,206 | 3 | 3/3 |
| `exact-normal-003` | 14,243,707 | 15,443,707 | 3 | 3/3 |
| `exact-normal-004` | 15,798,407 | 15,798,407 | 0 | 1/3 |
| `exact-normal-005` | 21,800,967 | 22,600,967 | 2 | 2/3 |
| `exact-normal-006` | 19,550,191 | 19,950,191 | 1 | 2/3 |

The 400,000-token allowance comes from the previously frozen bound: a 258,400-token active context
window, a 128,000-token maximum output, and enough room for the captured new-turn prompt. Its source
and arithmetic are retained in
`bench-results/efficiency-point7-bounded-accounting-20260828T142614Z/evidence.json`.

## Token Result

Raw tokens are input plus output, including cached input. Uncached tokens are uncached input plus
output.

| Group | Mean raw tokens | Mean uncached tokens |
| --- | ---: | ---: |
| Direct Codex | 36,116,382 exact | 1,608,712 exact |
| Work Leaf | 17,471,532-18,138,199 | 1,343,404-2,010,071 |
| Work Leaf change | 49.78%-51.62% fewer | 24.95% more to 16.49% fewer |

The full six-run group proves a raw-token reduction under the conservative bound. It does not prove
an uncached-token reduction.

As a limited quality check, five fully successful direct runs averaged 37.56 million raw tokens.
The two fully successful Work Leaf runs averaged between 13.73 and 14.93 million. That is a bounded
60.26%-63.45% reduction, but the Work Leaf subset has only two observations and was selected after
scoring. It supports a real effect; it does not replace a planned quality-balanced comparison.

## Why No Replacement Runs Were Needed

Repeating the same normal Work Leaf workflow would not make interrupted-response accounting exact.
Codex 0.150.1 can acknowledge an interrupted turn without emitting terminal usage for that
response. The saved lower bounds plus the conservative maximum already prove the raw result, so new
paid runs would add samples without repairing this transport limitation.

Exact normal-workflow accounting would require provider telemetry that reports usage for cancelled
responses. Until that exists, normal Work Leaf results must remain bounded whenever the corrected
observer reports unresolved interrupted responses.

## Conclusion

The defensible result is:

- Work Leaf used at least 49.78% fewer raw tokens in this six-versus-six sample.
- The exact raw reduction lies between 49.78% and 51.62%.
- The uncached-token direction is unknown.
- The all-run quality averages differ, so this is not an equal-quality average claim.
- The two fully successful Work Leaf runs remain far below every fully successful direct average
  even under the conservative bound, but that subset is small and descriptive.

The corrected machine-readable result is `evidence.json`. Full corrected replay outputs are named
`analysis-request-accounting.json` inside each saved run's `observation` directory. The older
`analysis-cumulative.json` files are retained as superseded evidence of the rejected rule.

## Reproduce The Analysis

From the repository root:

```sh
python3 bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/scorer/test_score.py
python3 bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/test_analyze.py
python3 bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/analyze.py
```
