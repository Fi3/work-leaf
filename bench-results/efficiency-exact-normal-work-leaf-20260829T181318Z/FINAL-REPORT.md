# Exact Normal Work Leaf Follow-up

## Abstract

This study answers whether interrupted Work Leaf responses were being omitted from token counts. It
ran six concurrent workflows using the normal Work Leaf implementation on the frozen three-feature
benchmark with GPT-5.5 at `xhigh`, three runs at a time. It compares them with six previously
collected, exactly measured normal direct Codex runs using the same task and validation rules.

No Work Leaf response is missing from the final totals. Ten interrupted responses did not have a
usage event immediately before interruption, but each affected provider thread later reported a
cumulative total that includes the response. No estimate or 400,000-token ceiling is used.

Work Leaf averaged 17.47 million raw tokens, 51.62% below direct Codex, and 1.34 million uncached
tokens, 16.49% below direct Codex. This is an exact descriptive token difference. It is not yet a
clean equal-quality efficiency result because Work Leaf completed 13 of 18 scored features while
direct Codex completed 17 of 18.

## Direct Answer

The interrupted-response accounting problem is resolved for these six runs: all provider usage is
included exactly. The earlier wide interval from adding up to 400,000 tokens per apparently missing
response was unnecessary.

The data still show lower Work Leaf token use, but they do not by themselves prove how much Work
Leaf would save at the same average implementation quality. The quality gap must remain visible.

## What Ran

| Workflow | Runs | Scheduling | Model | Reasoning |
| --- | ---: | --- | --- | --- |
| Direct Codex | 6 | normal sequential | GPT-5.5 | `xhigh` |
| Work Leaf | 6 | concurrent, with measured interrupt delay | GPT-5.5 | `xhigh` |

The Work Leaf runs were launched in two batches of three. Simultaneous runs are independent
observations, not matched pairs. Every success and partial implementation is retained.

The fixed task has three features: visual selection and copy, slash-command forwarding, and
reviewed-patch close/reopen. The scorer exercises the slash-command feature with `/status`.
`/fork` is not required and is not scored.

Both workflows use the same base commit, task information, model, reasoning level, time allowances,
normal validation freedom, final format/Clippy/test gate, and quality scorer. Direct Codex does not
use Work Leaf. Work Leaf uses its normal concurrent implementation. The observer changes only when
an already requested interrupt reaches the provider; incoming and forwarded request bytes match in
all six Work Leaf captures.

The observer can hold that interrupt for up to one second. Across 287 interrupts, 252 were released
after immediate exact usage, 34 were released when output resumed, and one reached the timeout. The
mean wait was 52 milliseconds and the combined wait was 15.0 seconds. This can add post-directive
provider work, and every added token is included. It means the workflow code is normal but the
provider timing is instrumented. The timing effect tends to add Work Leaf tokens on the affected
turn, but its possible effect on later thread behavior is not measured.

## Exact Accounting

Work Leaf communicates with Codex through app-server threads. Each
`thread/tokenUsage/updated.params.tokenUsage.total` value is the cumulative token count for its
thread. The observer keeps the largest final cumulative value for each thread and sums the threads.

The first analysis required a usage event immediately after every completed directive. It therefore
flagged ten interrupted responses. Replaying the saved streams showed that all ten affected threads
later emitted a cumulative total. Because that later total includes all earlier work on the thread,
the response was already in the recorded final total.

The observer analysis in `bench-observer/src/lib.rs::analyze_app_server` treats an interrupted turn
as missing only when there is neither an immediate usage event nor a later cumulative total on the
same thread. The regression test
`bench-observer/tests/proxy.rs::later_cumulative_thread_usage_covers_an_interrupted_turn_without_immediate_usage`
locks this behavior down.

All six captures are complete, report zero unresolved interrupted responses, and preserve identical
incoming and forwarded client streams. These results use exact totals, not bounds.

## Token Result

Raw tokens count input plus output, including cached input. Uncached tokens count uncached input plus
output.

| Group | Runs | Mean raw | Mean uncached |
| --- | ---: | ---: | ---: |
| Direct Codex | 6 | 36,116,382 | 1,608,712 |
| Work Leaf | 6 | 17,471,532 | 1,343,404 |
| Work Leaf reduction | | 51.62% | 16.49% |

The exact mean differences are 18,644,850 fewer raw tokens and 265,308 fewer uncached tokens per
workflow.

| Work Leaf run | Raw tokens | Uncached tokens | Scored features |
| --- | ---: | ---: | ---: |
| `exact-normal-001` | 20,221,714 | 1,471,762 | 2/3 |
| `exact-normal-002` | 13,214,206 | 1,179,134 | 3/3 |
| `exact-normal-003` | 14,243,707 | 1,188,347 | 3/3 |
| `exact-normal-004` | 15,798,407 | 1,348,359 | 1/3 |
| `exact-normal-005` | 21,800,967 | 1,497,991 | 2/3 |
| `exact-normal-006` | 19,550,191 | 1,374,831 | 2/3 |

## Quality Result

| Group | Visual | `/status` | Close/reopen | Total | Mean per run |
| --- | ---: | ---: | ---: | ---: | ---: |
| Direct Codex | 6/6 | 6/6 | 5/6 | 17/18 | 2.83/3 |
| Work Leaf | 5/6 | 6/6 | 2/6 | 13/18 | 2.17/3 |

All six Work Leaf benchmark workflows and final repository gates passed. That does not mean every
requested feature passed the independent scorer. Workflow validity, token completeness, and feature
quality are separate results.

The lower-quality Work Leaf average could require less model work, so the 51.62% and 16.49%
reductions must not be described as equal-quality savings.

As a limited check, the fully successful candidates averaged 37,564,061 raw and 1,549,060 uncached
tokens for five direct runs, versus 13,728,957 raw and 1,183,741 uncached tokens for two Work Leaf
runs. That is 63.45% lower raw and 23.58% lower uncached use for Work Leaf. This supports a real
effect, but the two-run Work Leaf subset is too small and was selected after scoring, so it is not a
replacement for a quality-balanced comparison.

## Conclusion

This study establishes three facts for the frozen benchmark:

1. The six new Work Leaf token totals are exact; interrupted responses are not undercounted.
2. The observed Work Leaf group mean is lower for both raw and uncached tokens.
3. The complete six-run groups differ in implementation quality, so the exact fraction attributable
   to workflow efficiency at equal quality is not established.

The token difference is therefore not explained by the interrupted-response accounting bug. The
remaining likely explanation is that Work Leaf reaches its result through fewer or shorter model
and tool cycles, but this study does not isolate which mechanism causes what fraction. A causal
allocation requires normal-workflow controls that disable one mechanism at a time without changing
validation behavior or task quality.

## Reproduce The Analysis

From the repository root:

```sh
python3 bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/scorer/test_score.py
python3 bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/test_analyze.py
python3 bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/analyze.py
```

The machine-readable result is `evidence.json`; per-run quality evidence is `quality.json`. Raw
provider streams and the cumulative reanalysis are retained under `runs/`.
