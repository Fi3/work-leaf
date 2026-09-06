# Causal Analysis

## Answer

Two controlled Work Leaf mechanism groups account arithmetically for 97.75%-98.02% of the observed
raw-token difference on the frozen three-feature benchmark. This is a descriptive allocation of the
collected sample means, not a confidence interval or established at-least-90% causal coverage:

1. The exactly measured main controls differ by 16,347,554 raw tokens in their sample means. This is
   87.68%-99.74% of the bounded normal endpoint gap.
2. Mediated reads plus early interruption after a complete directive, under the recorded one-second
   measurement grace, range from costing 325,910 tokens to saving 1,928,090. This is
   -1.99%-10.34% of the endpoint gap, so the direction of this joint contribution is unresolved.

The range exists because 35 responses in the normal Work Leaf endpoint were interrupted without
provable terminal usage. Raw-event replay proves that each gap contains one response and no
intervening tool boundary. Charging each response the derived 386,400-token maximum changes the
endpoint gap from 18,644,850 to 16,390,850 raw tokens. It does not change the exact main-control usage
totals. Sampling uncertainty in the cohort means is not included in these ranges, and normal-endpoint
measurement remains incomplete.

The separate completed-response control uses 2,792,303-5,046,303 more raw tokens than the
interrupted-response endpoint in these samples under the declared ceiling. This supports an
interruption effect, although the joint read-plus-interruption bridge changes sign within the bound
and the unequal-quality small cohorts do not establish the precision of expected effects.

## Main Causal Control

The main control compares compact direct Codex with sequential Work Leaf. Both process the same
features one at a time, read files directly, let responses finish, use compact exact linearization
targets, retain normal validation freedom, and run the same final checks. The intended difference is
the native direct tool loop versus Work Leaf's structured edit, command, ownership, and review
protocol.

| Measurement | Compact direct | Sequential Work Leaf |
| --- | ---: | ---: |
| Runs | 3 | 3 |
| Feature checks | 9/9 | 8/9 |
| Mean raw tokens | 35,659,265 | 19,311,710 |
| Mean uncached tokens | 1,547,137 | 1,785,694 |
| Raw range | 32.96M-40.58M | 16.70M-23.56M |
| Model generations | 311.00 | 198.00 |
| Implementation generations | 186.67 | 80.33 |
| Patch-agent native patch calls | 55.00 | 0.00 |
| Structured edit submissions | 0.00 | 11.67 |
| Review rounds | 6.00 | 10.67 |

All three Work Leaf raw totals are below all three direct totals. Both fully correct Work Leaf runs
are also below every direct run; this subset is selected after scoring and remains descriptive.
The pooled one-sided permutation calculation is 0.05 under exchangeability of all six condition
labels. It does not model collection batches or establish causal-share precision. Sequential Work
Leaf uses 15.42% more uncached tokens in the full main-control sample, despite using 45.84% fewer raw
tokens. Neither the endpoint nor the main-control groups establish quality equivalence.

## Causal Procedure

Direct Codex repeatedly reads, edits, and validates through native tools in a long model thread.
Each tool result leads to another model generation with the accumulated conversation.

Work Leaf changes that loop:

1. `src/agent.rs::PromptPolicy::for_read_permission` requests structured edits and mediated writes.
2. `src/orchestrator.rs::parse_agent_directives` recognizes those operations.
3. `src/orchestrator.rs::handle_agent_directives_streaming` applies them, records ownership, and
   creates provisional commits outside the model thread.
4. `src/orchestrator.rs::render_patch_applied_prompt` returns a compact result and focused next step.
5. `src/workspace.rs::start_review_for_patch_agent` reviews the recorded commit and routes findings
   to its owner.

This procedure replaces native tool loops with structured handoffs. The saved histories record fewer
model generations and 16.59 million fewer cached input tokens across the protocol transition, while
Work Leaf consumes more fresh input and slightly more reasoning output. The control supports the
protocol package; these observations do not separately identify each action's causal share.

## Bounded Allocation

| Transition | Sample raw-token difference | Descriptive share of sample endpoint gap |
| --- | ---: | ---: |
| Compact linearization | 457,117 fewer | 2.45%-2.80% |
| Work Leaf orchestration | 16,347,554 fewer | 87.68%-99.74% |
| Concurrent scheduling | 87,912 more | -0.54% to -0.47% |
| Mediated reads and interruption under the recorded grace | 325,910 more to 1,928,090 fewer | -1.99%-10.34% |
| Endpoint total | 16,390,850-18,644,850 fewer | 100% |

The bridge adds exactly within either endpoint scenario by telescoping; a zero arithmetic remainder
does not test causal completeness. The ranges are correlated: choosing the maximum Work Leaf
allowance produces both the smaller total gap and a negative read/interruption difference. Under
that scenario, the main-control difference is 99.74% of the endpoint gap and the bounded `C` to `W`
transition offsets part of it. The two Work Leaf mechanism groups net to 97.75% of the sample gap.
These bounds hold cohort means fixed, so neither sampling uncertainty nor quality differences enter
the allocation. They do not establish at least 90% causal coverage or that both groups save tokens
individually in the expected workflow.

## Alternative Explanations

| Explanation | Check | Result |
| --- | --- | --- |
| Missing Work Leaf tokens | Thirty-five normal-endpoint responses remain unresolved. | Every uncovered tail is one response with no tool boundary; the derived 386,400-token maximum is included and the raw conclusion survives. |
| Direct resume accounting | Direct invocations reconcile with their rollout epochs. | No unexplained direct overcount was found. |
| Ordinary variation | Main-control ranges do not overlap across three runs per group. | Supports a large package effect; expected-effect precision and small transition directions remain unresolved. |
| Lower quality | Main control scores 9/9 versus 8/9; both 3/3 Work Leaf runs remain below every direct run. | The passing subset is descriptive; equal-quality efficiency is not established. |
| Less review | Work Leaf performs 10.67 review rounds versus 6.00. | Rejected. |
| Skipped validation | Every control candidate passes the same final format, Clippy, test, build, and replay checks. | Rejected. |
| Concurrency | The main protocol control is sequential; the separate scheduling sample difference is small and negative. | The main-control saving does not require concurrency; the expected scheduling effect remains uncertain. |
| Compact linearization | Both main-control workflows receive compact exact targets. | A large raw-token difference remains with this handoff held fixed. |

## Scope

The controlled comparison supports an effect of the orchestration protocol as one package. It does
not separately measure structured edits, write-command mediation, compact acknowledgements, ownership, and review
routing. It applies to this frozen Rust benchmark; cross-project generalization and formal
equal-quality precision require separate studies. Exact normal-workflow telemetry and predetermined
quality, precision, and stopping criteria are required before a further measurement study. The
requested causal-coverage target remains unresolved.
