# Final Report: Why Work Leaf Uses Fewer Tokens

## Abstract

This study asks why normal concurrent Work Leaf used fewer GPT-5.5/`xhigh` tokens than a fair normal
direct sequential Codex workflow on the same three-feature Rust task.

The main cause is Work Leaf's orchestration protocol. Patch agents return complete structured edits
and mediated write commands; Work Leaf applies and commits them, returns compact results, and starts
review from recorded commits. Direct Codex performs the same kind of work through many more native
edit, command, and review cycles. Every extra cycle asks the model to generate again with the growing
conversation, which repeatedly replays cached input tokens.

The controlled protocol comparison is exact: compact direct Codex averaged 35,659,265 raw tokens,
while sequential Work Leaf averaged 19,311,710, a 45.84% reduction. Direct Codex averaged 311 model
generations and sequential Work Leaf averaged 198. Most of the reduction occurs during
implementation and review.

The normal Work Leaf endpoint contains ten interrupted responses without terminal usage. Applying
the frozen conservative maximum of 400,000 raw tokens to each one puts the normal endpoint reduction
between 49.78% and 51.62%. With that correction, the orchestration protocol plus mediated reads and
early directive interruption explain between 97.95% and 98.02% of the observed raw-token gap. The
causal coverage remains above the requested 90% even under the maximum missing-token allowance.

This is not a formal equal-quality population estimate. The six normal direct runs completed 17 of
18 feature checks and the six normal Work Leaf runs completed 13 of 18. The exact main control is
closer at 9/9 versus 8/9, and both fully correct Work Leaf control runs used fewer tokens than every
direct control run.

## What Was Compared

The endpoint contains six normal runs from each workflow:

| Normal workflow | Runs | Feature checks | Mean raw tokens |
| --- | ---: | ---: | ---: |
| Direct sequential Codex | 6 | 17/18 | 36,116,382 exact |
| Concurrent Work Leaf | 6 | 13/18 | 17,471,532-18,138,199 |

"Raw tokens" means all input plus output tokens, including cached input. "Uncached tokens" means
fresh input plus output. The uncached endpoint is not conclusive because the ten missing responses
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

| Main control | Runs | Feature checks | Mean raw tokens | Range |
| --- | ---: | ---: | ---: | ---: |
| Compact direct Codex | 3 | 9/9 | 35,659,265 | 32.96M-40.58M |
| Sequential Work Leaf | 3 | 8/9 | 19,311,710 | 16.70M-23.56M |

Sequential Work Leaf used 16,347,554 fewer raw tokens, or 45.84% less than compact direct Codex.
All three Work Leaf results are below all three direct results. The exact one-sided permutation
result is 0.05 for this three-versus-three sample. Both 3/3 Work Leaf runs are also below the lowest
direct result, though two observations are too few for a precise quality-balanced estimate.

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

Across this transition, Work Leaf saved 16.59 million cached input tokens while using about 252,000
more fresh input tokens and about 9,500 more reasoning-output tokens. The saving is therefore not
less fresh task information or suppressed reasoning. It is fewer repeated model generations that
replay a growing cached conversation.

## Allocation Of The Raw Saving

The normal Work Leaf endpoint is a range, so its gap and the affected bridge step are ranges too.
The exact controlled steps remain single values.

| Cause | Raw tokens saved | Share of endpoint gap |
| --- | ---: | ---: |
| Compact exact linearization handoff | 457,117 | 2.45%-2.54% |
| Work Leaf orchestration protocol | 16,347,554 | 87.68%-90.93% |
| Concurrent scheduling | 87,912 more | -0.49% to -0.47% |
| Mediated reads plus early directive interruption under the recorded grace | 1,261,423-1,928,090 | 7.02%-10.34% |
| Total endpoint gap | 17,978,183-18,644,850 | 100% |

At the recorded Work Leaf lower bound, the gap is 18.645 million tokens and the two Work Leaf
mechanism groups cover 98.02%: 87.68% from orchestration and 10.34% from reads and interruption. At
the conservative Work Leaf upper bound, those groups cover 97.95%: the exact orchestration effect is
90.93% and the bounded read/interruption effect is 7.02%.

The compact-linearization benefit and small concurrency cost nearly cancel. They are minor compared
with the protocol effect and normal run variation.

Inside the exact orchestration transition, implementation and fixes save 14.37 million raw tokens
and review saves 2.12 million. Linearization and Work Leaf's title session together use about
148,000 more, slightly reducing the net saving.

## Accounting Correction

The earlier report treated any later cumulative provider total as proof that an earlier interrupted
response had been counted. The corrected observer requires arithmetic proof: after subtracting the
previous total and the later response's own `last` usage, a nonzero increase must remain and exactly
one unresolved interruption must occupy that interval.

Five of the six normal Work Leaf runs contain ten unresolved responses under this rule. Their
recorded totals are lower bounds. The conservative upper bound adds 400,000 raw tokens to every
unresolved response. The exact-normal correction is documented in
`bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/FINAL-REPORT.md`.

The six main-control runs, the three completed-response control runs, and the three combined-control
runs have complete usage. Their controlled differences do not depend on the corrected normal
endpoint accounting.

## What Is Proven And What Is Not

The evidence supports these conclusions for this frozen benchmark:

- A large raw-token reduction remains after the maximum missing-token allowance.
- Work Leaf's orchestration protocol causes the main reduction as one connected mechanism package.
- That package reduces repeated model/tool cycles and cached-context replay during implementation
  and review.
- Mediated reads and early directive interruption under the recorded measurement grace add a
  smaller bounded contribution.
- Together those two mechanism groups cover at least 97.95% of the observed raw-token gap.

The evidence does not establish:

- an exact normal-workflow reduction percentage;
- an uncached-token reduction;
- formal equal-quality equivalence for the full six-run groups;
- separate percentages for structured edits, write-command mediation, compact acknowledgements,
  ownership, and review routing inside the orchestration package;
- generalization to other repositories or task types.

## Conclusion

Work Leaf saves raw tokens here mainly by changing the implementation and review loop. It replaces
many native model/tool cycles with fewer structured handoffs that Work Leaf applies and records
outside the model thread. This avoids repeatedly sending the growing cached conversation through
the model.

The exact main control measures a 16.35 million-token protocol effect. After conservatively bounding
the normal endpoint's ten unresolved responses, orchestration plus mediated reads and interruption
still explain 97.95%-98.02% of the observed raw-token difference. This meets the requested 90%
causal-coverage target without hiding the endpoint's measurement and quality limits.

Machine-readable values are in `evidence.json`; the detailed controlled chain is in
`05-CAUSAL-ANALYSIS.md`.
