# Study Report: Raw-Token Mechanism Controls And Measurement Limits

## Abstract

This study asks why normal concurrent Work Leaf used fewer GPT-5.5/`xhigh` tokens than a fair normal
direct sequential Codex workflow on the same three-feature Rust task.

The main controlled comparison supports Work Leaf's orchestration protocol as a source of the
raw-token difference. Patch agents return complete structured edits and mediated write commands;
Work Leaf applies and commits them, returns compact results, and starts
review from recorded commits. Direct Codex performs the same kind of work through many more native
edit, command, and review cycles. Every extra cycle asks the model to generate again with the growing
conversation, which repeatedly replays cached input tokens.

Provider usage in the main controls is exact: compact direct Codex averaged 35,659,265 raw tokens,
while sequential Work Leaf averaged 19,311,710, a 45.84% reduction. Direct Codex averaged 311 model
generations and sequential Work Leaf averaged 198. Most of the reduction occurs during
implementation and review. Sequential Work Leaf averaged 1,785,694 uncached tokens against direct
Codex's 1,547,137, or 15.42% more. Exact usage does not make these sample differences precise estimates
of expected effects.

The normal Work Leaf endpoint contains 35 interrupted responses without provable terminal usage.
Raw-event replay proves that every gap contains one response and no intervening tool boundary.
Applying the derived maximum of 386,400 raw tokens to each response puts the normal endpoint
reduction between 45.38% and 51.62%. The orchestration protocol plus the bounded mediated-read and
interruption transition net to 97.75%-98.02% of the observed raw-token gap as a descriptive allocation
of the collected sample means. This range accounts for missing usage only; it is not a confidence
interval and does not establish at least 90% causal coverage. The joint transition's direction
remains unresolved. The separate completed-response control uses 2.79M-5.05M more raw tokens than the
interrupted-response endpoint in the collected samples.

Normal-endpoint measurement remains incomplete. This is not a formal equal-quality population
estimate. The six normal direct runs completed 17 of 18 feature checks and the six normal Work Leaf
runs completed 13 of 18. The exact main control is
closer at 9/9 versus 8/9, and both fully correct Work Leaf control runs used fewer tokens than every
direct control run.

## What Was Compared

The endpoint contains six normal runs from each workflow:

| Normal workflow | Runs | Feature checks | Mean raw tokens |
| --- | ---: | ---: | ---: |
| Direct sequential Codex | 6 | 17/18 | 36,116,382 exact |
| Concurrent Work Leaf | 6 | 13/18 | 17,471,532-19,725,532 |

"Raw tokens" means all input plus output tokens, including cached input. "Uncached tokens" means
fresh input plus output. The uncached endpoint is not conclusive because the 35 missing responses
do not report their cached-input split.

All workflows use the original three requests, base commit
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`, GPT-5.5, `xhigh` reasoning, normal validation freedom,
the same final checks, and the same quality scorer. Direct Codex does not use Work Leaf. Normal Work
Leaf schedules its features concurrently. Runs are compared as groups, not paired by launch order.

The normal Work Leaf endpoint is timing-instrumented. After Work Leaf requested an interrupt for a
complete directive, the observer waited up to one second for provider usage before forwarding that
same request. Across 287 interrupts the combined wait was 15.0 seconds. This can add tokens and can
change later model behavior, so the endpoint represents normal Work Leaf logic under the recorded
measurement grace, not an entirely unobserved product run. The exact main protocol control lets
responses finish on both sides and does not depend on this grace.

## The Main Controlled Test

The most important control removes concurrency and the context-delivery optimizations before
comparing the two workflow loops:

- both workflows process features sequentially;
- both patch agents read files directly;
- both let provider responses finish instead of interrupting them;
- both linearizers receive compact exact commit targets;
- both retain normal focused-validation freedom and the same broad final checks;
- both include implementation, review, fixes, linearization, title work, and every provider thread.

The intended difference is direct Codex's normal native tool loop versus Work Leaf's structured
patch, write-command, ownership, and review protocol.

| Main control | Runs | Feature checks | Mean raw tokens | Raw range | Mean uncached tokens |
| --- | ---: | ---: | ---: | ---: | ---: |
| Compact direct Codex | 3 | 9/9 | 35,659,265 | 32.96M-40.58M | 1,547,137 |
| Sequential Work Leaf | 3 | 8/9 | 19,311,710 | 16.70M-23.56M | 1,785,694 |

Sequential Work Leaf used 16,347,554 fewer raw tokens, or 45.84% less than compact direct Codex.
All three Work Leaf results are below all three direct results. The pooled one-sided permutation
calculation is 0.05 under exchangeability of all six condition labels; it does not model the two
collection batches or establish causal-share precision. Both 3/3 Work Leaf runs are also below the
lowest direct result. That subset is selected after scoring and remains descriptive; two Work Leaf
observations do not establish quality equivalence. The full main-control sample uses 238,558 more
uncached tokens under Work Leaf, a 15.42% increase.

## What Produces The Difference

The saved provider histories show the procedure that reduces tokens:

1. A direct patch agent repeatedly calls native read, patch, and command tools in one long model
   thread. Every tool result triggers another model generation with the accumulated conversation.
2. A Work Leaf patch agent submits a complete structured edit or mediated write request.
3. `src/orchestrator.rs::handle_agent_directives_streaming` applies the operation and records a
   provisional commit outside the model thread.
4. `src/orchestrator.rs::render_patch_applied_prompt` returns a compact result and asks for focused
   validation or completion.
5. `src/workspace.rs::start_review_for_patch_agent` starts review from the recorded commit and routes
   findings to the owning patch agent.

The policy that asks agents for structured edits and mediated writes is
`src/agent.rs::PromptPolicy::for_read_permission`. Directive parsing is owned by
`src/orchestrator.rs::parse_agent_directives`.

The measured consequences per workflow are:

| Measurement | Compact direct | Sequential Work Leaf | Change |
| --- | ---: | ---: | ---: |
| Model generations | 311.00 | 198.00 | 36.33% fewer |
| Implementation generations | 186.67 | 80.33 | 56.96% fewer |
| Native patch calls by patch agents | 55.00 | 0.00 | replaced by structured edits |
| Structured edit submissions | 0.00 | 11.67 | orchestrator applies them |
| Implementation native commands | 279.33 | 153.00 | 45.23% fewer |
| Review native commands | 249.33 | 147.00 | 41.05% fewer |
| Review rounds | 6.00 | 10.67 | more, not omitted |

Across this transition, Work Leaf used 16.59 million fewer cached input tokens while using about
252,000 more fresh input tokens and about 9,500 more reasoning-output tokens. The raw-token
difference is concentrated in cached-context replay. The recorded reduction in model generations is
consistent with the structured handoff mechanism; these counts do not separately establish each
protocol action's causal contribution or an uncached-token saving.

## Allocation Of The Raw Saving

The normal Work Leaf endpoint is a range, so its gap and the affected bridge step are ranges too.
The exactly measured controls supply sample means. Every range below varies only the missing-usage
allowance while holding those sample means fixed. Sampling uncertainty, unequal quality, and
differences between collection batches are not represented by these bounds.

`analyze.py::bounded_endpoint_bridge` retains absolute token ranges when the endpoint difference
interval includes zero, but writes `share_of_endpoint_gap_percent: null` with status `undefined`.
`bounded_selected_causal_coverage` applies the same rule to grouped transitions. This applies to the
uncached endpoint: percentages evaluated at its two bounds cannot bound a ratio across a zero
denominator. `evidence.json::interpretation` identifies the remaining numeric ranges as descriptive
accounting bounds and records the unresolved causal-coverage and quality-equivalence conclusions.

| Controlled transition | Sample raw-token difference | Descriptive share of sample endpoint gap |
| --- | ---: | ---: |
| Compact exact linearization handoff | 457,117 fewer | 2.45%-2.80% |
| Work Leaf orchestration protocol | 16,347,554 fewer | 87.68%-99.74% |
| Concurrent scheduling | 87,912 more | -0.54% to -0.47% |
| Mediated reads plus early directive interruption under the recorded grace | 325,910 more to 1,928,090 fewer | -1.99%-10.34% |
| Total endpoint gap | 16,390,850-18,644,850 fewer | 100% |

At the recorded Work Leaf lower bound, the gap is 18.645 million tokens and the two Work Leaf
mechanism groups account arithmetically for 98.02%: 87.68% from orchestration and 10.34% from reads and
interruption. At the conservative Work Leaf upper bound, the gap is 16.391 million. The main-control
difference is 99.74% of that smaller gap, while the bounded read/interruption transition offsets 1.99%; their
net allocation is 97.75%. The full bridge telescopes to 100% by construction, so its zero remainder
does not test explanatory completeness. The exact controls support a substantial orchestration
effect, but this allocation does not establish at least 90% causal coverage or that reads plus
interruption independently save tokens.

The compact-linearization and scheduling sample differences nearly cancel. Their magnitudes are
small relative to variation between the recorded runs; their expected directions remain uncertain.

Inside the exact orchestration transition, implementation and fixes save 14.37 million raw tokens
and review saves 2.12 million. Linearization and Work Leaf's title session together use about
148,000 more, slightly reducing the net saving.

## Accounting Rule

The observer accepts same-turn terminal usage only when the cumulative total advances and its
nonzero `last` usage fits inside that advance; a repeated total or an unattributed advance is not
proof of the response. A later turn covers an earlier interrupted response only when, after
subtracting the previous total and the later response's own `last` usage, a nonzero increase remains
and exactly one unresolved interruption occupies that interval.

Five of the six normal Work Leaf runs contain 35 unresolved responses under these rules. Their
recorded totals are lower bounds. The raw streams prove that every uncovered tail contains exactly
one directive response and no intervening tool or input boundary. The conservative upper bound adds
386,400 raw tokens to each response: the frozen Codex 0.150.1 client enforces a 258,400-token hard
active-context limit and GPT-5.5 permits 128,000 output tokens. Every capture reports that active
context limit. The exact-normal accounting is documented in
`bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/FINAL-REPORT.md`.

The six main-control runs, the three completed-response control runs, and the three combined-control
runs have complete usage. Their controlled differences do not depend on the corrected normal
endpoint accounting.

## Supported Findings And Limits

The evidence supports these conclusions for this frozen benchmark:

- A large raw-token reduction remains after the maximum missing-token allowance.
- The controlled comparison supports a substantial raw-token effect from Work Leaf's orchestration
  protocol as one connected mechanism package.
- The main controls record fewer model/tool cycles and less cached-context replay during
  implementation and review, together with 15.42% more uncached tokens under Work Leaf.
- The completed-response control uses 2.79M-5.05M more raw tokens than the interrupted-response
  endpoint in the collected samples under the declared missing-usage ceiling.
- Mediated reads do not have a proven independent direction, and the joint read/interruption bridge
  remains unresolved because those interventions interact.
- The orchestration and read/interruption sample differences together net to 97.75%-98.02% of the
  observed raw-token gap in the descriptive bridge.

The evidence does not establish:

- an exact normal-workflow reduction percentage;
- an uncached-token reduction;
- at least 90% causal coverage with sampling uncertainty accounted for;
- formal equal-quality equivalence for either the endpoint or main-control groups;
- separate percentages for structured edits, write-command mediation, compact acknowledgements,
  ownership, and review routing inside the orchestration package;
- generalization to other repositories or task types.

## Conclusion

The main controls support changing the implementation and review loop as a source of raw-token
savings. Work Leaf replaces many native model/tool cycles with fewer structured handoffs that it applies and records
outside the model thread. This avoids repeatedly sending the growing cached conversation through
the model.

The exact main-control usage totals yield a 16.35 million-token difference in sample means. After
conservatively bounding the normal endpoint's 35 unresolved responses, orchestration plus the bounded mediated-read and
interruption transition net to 97.75%-98.02% of the observed raw-token difference as a descriptive
allocation. Normal-endpoint measurement remains incomplete, and the requested 90% causal-coverage
target is not established. A further study needs exact normal-workflow telemetry and predetermined
quality, sampling precision, and stopping criteria before collecting provider runs. The authorized
telemetry gate and first measurement batch are defined in
[`PROTOCOL.md`](../efficiency-measurement-gate-20260906/PROTOCOL.md).

Machine-readable values are in `evidence.json`; the detailed controlled chain is in
`05-CAUSAL-ANALYSIS.md`.
