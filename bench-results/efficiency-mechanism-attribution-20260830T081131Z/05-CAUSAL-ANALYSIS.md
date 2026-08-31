# Causal Analysis

## Answer

Two controlled Work Leaf mechanism groups explain 97.75%-98.02% of the observed raw-token
difference on the frozen three-feature benchmark:

1. The Work Leaf orchestration protocol saves exactly 16,347,554 raw tokens in its controlled
   comparison. This is 87.68%-99.74% of the bounded normal endpoint gap.
2. Mediated reads plus early interruption after a complete directive, under the recorded one-second
   measurement grace, range from costing 325,910 tokens to saving 1,928,090. This is
   -1.99%-10.34% of the endpoint gap, so the direction of this joint contribution is unresolved.

The range exists because 35 responses in the normal Work Leaf endpoint were interrupted without
provable terminal usage. Raw-event replay proves that each gap contains one response and no
intervening tool boundary. Charging each response the derived 386,400-token maximum changes the
endpoint gap from 18,644,850 to 16,390,850 raw tokens. It does not change the exact main control.

The separate completed-response control also becomes conclusive under this bound: allowing those
responses to finish uses 2,792,303-5,046,303 more raw tokens than early interruption. Interruption is
therefore a real saving in these samples, although the joint read-plus-interruption bridge can still
change sign because mediated reads and interruption interact.

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
| Raw range | 32.96M-40.58M | 16.70M-23.56M |
| Model generations | 311.00 | 198.00 |
| Implementation generations | 186.67 | 80.33 |
| Patch-agent native patch calls | 55.00 | 0.00 |
| Structured edit submissions | 0.00 | 11.67 |
| Review rounds | 6.00 | 10.67 |

All three Work Leaf totals are below all three direct totals. Both fully correct Work Leaf runs are
also below every direct run. The exact one-sided three-versus-three permutation result is 0.05.

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

This procedure replaces many small native tool loops with fewer complete handoffs. The saved token
classes confirm the result: Work Leaf saves 16.59 million cached input tokens across the protocol
transition while consuming more fresh input and slightly more reasoning output.

## Bounded Allocation

| Transition | Raw-token effect | Share of endpoint gap |
| --- | ---: | ---: |
| Compact linearization | 457,117 fewer | 2.45%-2.80% |
| Work Leaf orchestration | 16,347,554 fewer | 87.68%-99.74% |
| Concurrent scheduling | 87,912 more | -0.54% to -0.47% |
| Mediated reads and interruption under the recorded grace | 325,910 more to 1,928,090 fewer | -1.99%-10.34% |
| Endpoint total | 16,390,850-18,644,850 fewer | 100% |

The bridge adds exactly within either endpoint scenario. The ranges are correlated: choosing the
maximum Work Leaf allowance produces both the smaller total gap and a negative read/interruption
effect. Under that most conservative scenario, the exact orchestration saving is larger than the
endpoint gap and the bounded `C` to `W` transition offsets part of it. The two Work Leaf mechanism
groups still net to 97.75% of the gap, but the data do not prove that both groups save tokens
individually.

## Alternative Explanations

| Explanation | Check | Result |
| --- | --- | --- |
| Missing Work Leaf tokens | Thirty-five normal-endpoint responses remain unresolved. | Every uncovered tail is one response with no tool boundary; the derived 386,400-token maximum is included and the raw conclusion survives. |
| Direct resume accounting | Direct invocations reconcile with their rollout epochs. | No unexplained direct overcount was found. |
| Ordinary variation | Main-control ranges do not overlap across three runs per group. | Unlikely to create the protocol effect; small bridge steps remain noisy. |
| Lower quality | Main control scores 9/9 versus 8/9; both 3/3 Work Leaf runs remain below every direct run. | Cannot plausibly explain the full protocol effect, but formal equivalence is not proven. |
| Less review | Work Leaf performs 10.67 review rounds versus 6.00. | Rejected. |
| Skipped validation | Every control candidate passes the same final format, Clippy, test, build, and replay checks. | Rejected. |
| Concurrency | The main protocol control is sequential; the separate scheduling transition is small and negative. | Not the cause of the saving. |
| Compact linearization | Both main-control workflows receive compact exact targets. | Not the main cause. |

## Scope

The controlled result identifies the orchestration protocol as one package. It does not separately
measure structured edits, write-command mediation, compact acknowledgements, ownership, and review
routing. It applies to this frozen Rust benchmark; cross-project generalization and formal
equal-quality precision require separate studies.
