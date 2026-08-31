# Bounded Normal Work Leaf Follow-up

## Abstract

This study compares six normal concurrent Work Leaf runs with six normal direct sequential Codex
runs on the same three-feature Rust task. Every run used GPT-5.5 with `xhigh` reasoning, the same
base commit, normal validation freedom, the same final checks, and the same feature scorer.

Codex does not always report terminal usage for a response interrupted after a complete Work Leaf
directive. Replaying the six saved provider streams with strict cumulative accounting finds 35
unresolved responses across five runs; one Work Leaf run is exact. A repeated cumulative total does
not count as new usage, and a later turn proves coverage only when its cumulative increase contains
tokens beyond that later response's own usage.

The recorded Work Leaf totals average 17.47 million raw tokens. Charging the derived maximum of
386,400 raw tokens to every unresolved response raises the average to 19.73 million. Direct Codex
averaged 36.12 million. Work Leaf therefore used between 45.38% and
51.62% fewer raw tokens in these samples. The raw-token reduction survives the conservative bound,
but its exact percentage is unknown.

Direct Codex completed 17 of 18 feature checks and Work Leaf completed 13 of 18. The all-run result
is not an equal-quality average comparison. The uncached-token direction is also unknown because
the unresolved responses do not report their cached-input split.

## What Was Compared

| Workflow | Runs | Scheduling | Model | Reasoning | Feature checks |
| --- | ---: | --- | --- | --- | ---: |
| Direct Codex | 6 | normal sequential | GPT-5.5 | `xhigh` | 17/18 |
| Work Leaf | 6 | normal concurrent | GPT-5.5 | `xhigh` | 13/18 |

The task asks for visual selection and copy, `/status` forwarding, and reviewed-patch close/reopen.
`/fork` is not part of the task and is not scored. Runs are independent group observations, not
matched pairs. Every success, partial implementation, failure, and measurement gap remains in the
study.

Both workflows use the same base commit, task text, model, reasoning level, time allowances, normal
validation freedom, final format/Clippy/test gate, and scorer. Direct Codex does not use Work Leaf.
Work Leaf uses its normal concurrent implementation.

The Work Leaf observer held an already requested interrupt for at most one second while waiting for
provider usage. This can permit extra generation after a directive. That observed work is counted.
Across 287 interrupts, the combined delay was 15.0 seconds. The incoming and forwarded request
bytes match in all six captures.

## Usage Accounting

Codex reports `tokenUsage.total` as a cumulative total for a provider thread and `tokenUsage.last`
as the usage of the response associated with that event. Two rules prevent a later notification
from being mistaken for usage from an interrupted response:

1. A notification attached to the interrupted turn counts only when its cumulative total advances
   beyond the preceding total and its nonzero `last` usage fits inside that advance. Repeated totals
   and advances without attributable `last` usage are not proof of the response.
2. A notification from a later turn recovers one earlier response only when exactly one unresolved
   interruption lies in that interval and subtracting the later response's `last` usage leaves a
   nonzero increase.

The implementation is in
`bench-observer/src/lib.rs::cumulative_usage_contains_last_response` and
`bench-observer/src/lib.rs::later_cumulative_usage_proves_interrupted_turn`. Regression coverage is
in
`bench-observer/tests/proxy.rs::unchanged_same_turn_usage_does_not_cover_interrupted_response` and
`bench-observer/tests/proxy.rs::later_cumulative_usage_without_an_unreported_increment_does_not_cover_interrupted_turn`;
missing `last` coverage is tested by
`bench-observer/src/lib.rs::tests::cumulative_increase_without_last_usage_does_not_prove_a_response`.

Applying those rules to the saved streams gives:

| Work Leaf run | Recorded raw lower bound | Conservative raw upper bound | Unresolved responses | Features |
| --- | ---: | ---: | ---: | ---: |
| `exact-normal-001` | 20,221,714 | 20,994,514 | 2 | 2/3 |
| `exact-normal-002` | 13,214,206 | 15,146,206 | 5 | 3/3 |
| `exact-normal-003` | 14,243,707 | 22,744,507 | 22 | 3/3 |
| `exact-normal-004` | 15,798,407 | 15,798,407 | 0 | 1/3 |
| `exact-normal-005` | 21,800,967 | 23,346,567 | 4 | 2/3 |
| `exact-normal-006` | 19,550,191 | 20,322,991 | 2 | 2/3 |

## Why One Bound Applies To Each Gap

Each unresolved unit is the final model response that produced a complete Work Leaf directive, not
an entire multi-response app-server turn. Earlier usage notifications in that turn remain in the
recorded cumulative lower bound.

The evidence generator checked all 35 unresolved event tails. Every tail contains exactly one
completed directive response and no tool call, tool result, or second user input between the last
counted usage event and that directive. After the directive, 25 tails contain only a duplicate of
the previous cumulative usage total and 10 contain no usage event. In 34 tails one continuation
item starts but never completes before interruption; the remaining tail reaches the grace timeout.
No tail contains an unrecognized protocol event or a second completed output item. Work Leaf thus
interrupts the same response before another model/tool cycle can begin. The per-turn audit and
raw-stream hashes are recorded in `evidence.json`.

Each missing final response receives this derived 386,400-token ceiling:

| Component | Maximum |
| --- | ---: |
| [Frozen Codex 0.150.1 GPT-5.5 catalog context](https://github.com/openai/codex/blob/rust-v0.150.1/codex-rs/models-manager/models.json#L613-L645) | 272,000 tokens |
| [Hard active-context limit after the client's 95% factor](https://github.com/openai/codex/blob/rust-v0.150.1/codex-rs/core/src/session/turn_context.rs#L368-L375) | 258,400 tokens |
| [GPT-5.5 output limit](https://developers.openai.com/api/docs/models/gpt-5.5) | 128,000 tokens |
| Maximum raw tokens in one response | 386,400 tokens |

Codex 0.150.1 explicitly treats the effective active-context window as a
[hard cap](https://github.com/openai/codex/blob/rust-v0.150.1/codex-rs/core/src/session/context_window.rs#L53-L76).
Adding the full 258,400-token hard context and the full 128,000-token output allowance is
conservative because it lets both maxima occur together. The largest provider-reported response in
the six captures is 180,949 raw tokens. The 386,400 value is a worst-case bound, not an estimate of
likely usage. `response-bound.json` freezes the binary identity, sources, and arithmetic; the
analysis fails if the run version, binary hash, captured window, protocol tail, or arithmetic
disagrees.

## Token Result

Raw tokens are input plus output, including cached input. Uncached tokens are uncached input plus
output.

| Group | Mean raw tokens | Mean uncached tokens |
| --- | ---: | ---: |
| Direct Codex | 36,116,382 exact | 1,608,712 exact |
| Work Leaf | 17,471,532-19,725,532 | 1,343,404-3,597,404 |
| Work Leaf change | 45.38%-51.62% fewer | 123.62% more to 16.49% fewer |

The six-versus-six sample proves a raw-token reduction under the declared ceiling. It does not
prove an uncached-token reduction.

As a limited quality check, five fully successful direct runs averaged 37.56 million raw tokens.
The two fully successful Work Leaf runs averaged between 13.73 and 18.95 million. This is a bounded
49.57%-63.45% reduction. The Work Leaf subset has only two observations and was selected after
scoring, so it is descriptive and does not replace a planned quality-balanced comparison.

## Limits

- The all-run feature totals differ, so the primary result is not an equal-quality average claim.
- Thirty-five responses are bounded rather than measured exactly.
- The uncached result is inconclusive because the missing cached-input split is unknown.
- Six observations per group do not establish a population effect or cross-project generality.
- The one-second measurement delay can change post-directive timing; all resulting output and usage
  are included in the captures.

Repeating the same workflow cannot guarantee exact totals because Codex 0.150.1 can acknowledge an
interrupted turn without terminal usage. Exact normal-workflow accounting requires provider
telemetry for cancelled responses. The conservative ceiling is sufficient to establish the raw
result for this collected sample without another paid run.

## Conclusion

The defensible result is:

- Work Leaf used at least 45.38% fewer raw tokens in this six-versus-six sample.
- The bounded raw reduction lies between 45.38% and 51.62%.
- The uncached-token direction is unknown.
- The all-run comparison does not establish equal-quality efficiency.
- The post-hoc fully successful subset also remains below direct Codex under the ceiling, but it is
  small and descriptive.

`evidence.json` is the machine-readable authority. Each run's canonical replay is
`analysis-request-accounting.json`. The retained `analysis-pre-same-turn-accounting.json` files show
the superseded interpretation that accepted repeated same-turn totals, while
`analysis-cumulative.json` preserves the still earlier rejected cumulative-recovery rule.

## Reproduce The Analysis

From the repository root:

```sh
python3 bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/scorer/test_score.py
python3 bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/test_analyze.py
python3 bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/analyze.py
```
