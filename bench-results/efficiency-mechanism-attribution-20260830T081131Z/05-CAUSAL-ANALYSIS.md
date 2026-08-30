# Causal Analysis

## Answer

Two controlled mechanism groups explain 98.02% of the observed raw-token difference between normal
direct sequential Codex and normal concurrent Work Leaf:

1. The Work Leaf orchestration protocol explains 87.68%.
2. Work Leaf's mediated file reads plus immediate interruption after a complete directive explain
   another 10.34%.

The first group is the main answer. It was tested before concurrency: both sides processed the same
three features one at a time, read files directly, let provider responses finish, received compact
exact linearization targets, used GPT-5.5 with `xhigh` reasoning, and ran the same final checks. The
only intentional difference was direct Codex tools versus Work Leaf's normal patch, command,
ownership, and review protocol.

## Why The Protocol Saves Tokens

A direct Codex agent reads, edits, and validates through native tools inside a long provider thread.
After each tool result, the model generates again with the accumulated thread as input. Repeated
small edits and commands therefore replay an increasingly large cached prompt.

Work Leaf changes that loop in four connected ways:

1. `src/agent.rs::PromptPolicy::for_read_permission` requires patch agents to return a structured
   edit instead of writing through native tools, and requires write-producing commands to go through
   `@work-leaf locks run`.
2. `src/orchestrator.rs::parse_agent_directives` parses those requests, while
   `handle_agent_directives_streaming` applies the edit, records ownership, and creates the
   provisional commit outside the provider thread.
3. `src/orchestrator.rs::render_patch_applied_prompt` returns a compact acknowledgement, directs the
   agent to one relevant focused validation step, and then asks it to finish. Broad formatting,
   Clippy, and test checks still run at final linearization for both workflows.
4. `src/workspace.rs::should_start_review` and `start_review_for_patch_agent` start review from the
   recorded provisional commit and route findings back to the owning patch agent.

This protocol makes an edit, validation, and review handoff a small number of explicit workflow
steps. Direct Codex is free to perform the same work through many native tool cycles. The controls
show that replacing the direct loop with this protocol causes the token reduction; the provider
histories show the concrete path through which it happens.

## Controlled Evidence

| Measurement per workflow | Compact direct | Sequential Work Leaf |
| --- | ---: | ---: |
| Runs | 3 | 3 |
| Feature checks | 9/9 | 8/9 |
| Mean raw tokens | 35,659,265 | 19,311,710 |
| Raw-token range | 32.96M-40.58M | 16.70M-23.56M |
| Distinct provider usage changes | 311.00 | 198.00 |
| Implementation-stage usage changes | 186.67 | 80.33 |
| Patch-agent native `apply_patch` calls | 55.00 | 0.00 |
| Patch-agent structured edit submissions | 0.00 | 11.67 |
| Implementation native command calls | 279.33 | 153.00 |
| Review native command calls | 249.33 | 147.00 |
| Review rounds | 6.00 | 10.67 |

Each distinct cumulative provider-usage change corresponds to another measured model generation in
the saved rollout. Work Leaf reduces these generations by 36.33%. The largest reduction is in
implementation, where patch agents replace an average of 55 native patch calls with 11.67
orchestrator-applied structured edits.

The review result rules out omitted review work. Sequential Work Leaf performs more review rounds,
not fewer, while still using 2.12 million fewer review-stage raw tokens. Linearization also cannot
explain the saving: sequential Work Leaf uses 87,004 more tokens in that stage.

The token classes identify what those avoided cycles save. Across the protocol transition, Work
Leaf saves 16.59 million cached input tokens while using about 252,000 more fresh input tokens and
about 9,500 more reasoning-output tokens. The saving is repeated context replay, not less fresh task
information or suppressed reasoning.

## Complete Allocation

The ordered bridge uses five conditions:

- Normal direct sequential Codex.
- Direct sequential Codex with compact exact linearization targets.
- Sequential diagnostic Work Leaf with direct reads and completed provider responses.
- Concurrent Work Leaf with direct reads and completed provider responses.
- Normal concurrent Work Leaf.

Adjacent conditions change one mechanism group. Their mean differences add exactly to the 18.645
million-token endpoint gap.

| Controlled transition | Raw tokens | Share of endpoint gap |
| --- | ---: | ---: |
| Compact exact linearization handoff | 457,117 fewer | 2.45% |
| Work Leaf orchestration protocol | 16,347,554 fewer | 87.68% |
| Concurrent rather than sequential scheduling | 87,912 more | -0.47% |
| Mediated reads plus immediate directive interruption | 1,928,090 fewer | 10.34% |
| Total | 18,644,850 fewer | 100.00% |

The two repeatable Work Leaf mechanisms together account for 18,275,644 tokens, or 98.02% of the
endpoint gap. The remaining net 1.98% is the small compact-handoff benefit offset by the small
concurrency cost. Those two small values are within ordinary run variation and are not needed for
the 90% causal-coverage goal.

Within the orchestration transition, the saved tokens occur in these stages:

| Stage | Raw tokens | Share of endpoint gap |
| --- | ---: | ---: |
| Implementation and fixes | 14,374,206 fewer | 77.09% |
| Review | 2,121,012 fewer | 11.38% |
| Linearization | 87,004 more | -0.47% |
| Work Leaf title session | 60,660 more | -0.33% |

Implementation and review therefore account for the entire protocol saving; linearization and the
extra title session slightly reduce it.

## Alternative Explanations

| Explanation | Check | Conclusion |
| --- | --- | --- |
| Token-accounting error | Every provider rollout is hash-locked and reconciled; there are no missing, interrupted, or descendant sessions. | Contradicted. |
| Ordinary model variation | All three sequential Work Leaf results are below all three compact-direct results; the exact one-sided three-versus-three permutation result is 0.05. | Unlikely to create the observed effect. |
| Lower implementation quality | The groups score 9/9 and 8/9. Both 3/3 Work Leaf runs remain below every direct run. | Cannot explain the effect. |
| Less review | Work Leaf averages 10.67 review rounds versus 6.00. | Contradicted. |
| Skipped final validation | Every candidate passes the same final formatting, Clippy, full test, build, and replay gates. | Contradicted. |
| Concurrency | The main control is sequential; changing only scheduling costs 0.47% in this bridge. | Not the saving. |
| Compact linearization | Both main-control workflows receive exact compact targets; the stage itself costs Work Leaf slightly more. | Not the main saving. |
| Read and interruption optimization | The prior joint control measures 10.34%. | Real but secondary. |
| Work Leaf orchestration protocol | Replacing only the direct workflow loop produces complete range separation and 16.35M fewer tokens. | Main causal explanation. |

## Scope

The controlled result proves the orchestration package as a group. It does not assign separate
causal percentages to structured edits, write-command mediation, compact command responses, patch
ownership, and review routing because those parts were not disabled one at a time. The saved
histories identify structured patch and command cycles as the dominant path, but a finer percentage
split would require additional controls.

The conclusion applies to this frozen three-feature Rust benchmark and normal workflows represented
here. Cross-project generalization and tighter statistical confidence require separate follow-up
studies; they are not prerequisites for the current 98.02% causal-coverage result.
