# Mechanism Attribution Protocol

## Goal

Explain at least 90% of the observed raw-token difference between normal direct sequential Codex
and normal concurrent Work Leaf with controlled workflow substitutions.

The earlier causal study established that the difference exists for this benchmark, but it directly
explained only the joint read-and-interruption effect. Counts such as fewer model generations,
commands, or validations locate the saved tokens; they do not by themselves explain why those
actions did not occur. This study treats those counts as outcomes, not causes.

## Frozen Endpoint

The endpoint remains the original three-feature Rust benchmark at
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`, using GPT-5.5 with `xhigh` reasoning and the frozen
`/status` quality scorer.

The accepted endpoint cohorts are:

| Endpoint | Runs | Feature checks | Mean raw tokens |
| --- | ---: | ---: | ---: |
| Normal direct sequential Codex | 6 | 17/18 | 36,116,382 exact |
| Normal concurrent Work Leaf | 6 | 13/18 | 17,471,532-19,725,532 bounded |

Every implementation outcome remains evidence. A partial feature result is scored and retained; it
is not retried merely because its quality is lower.

## Causal Bridge

Five conditions form one ordered bridge. Adjacent conditions change one mechanism group.

| Symbol | Condition | Difference from the condition above |
| --- | --- | --- |
| `D` | Normal direct sequential Codex | Endpoint |
| `L` | Direct sequential Codex with compact exact linearization targets | Linearization handoff only |
| `S` | Sequential diagnostic Work Leaf with direct reads and completed responses | Work Leaf orchestration protocol |
| `C` | Concurrent Work Leaf with direct reads and completed responses | Scheduling only |
| `W` | Normal concurrent Work Leaf under the recorded one-second usage grace | Mediated reads and early directive interruption |

The allocation is calculated in this order:

```text
compact linearization          = D - L
Work Leaf orchestration        = L - S
concurrent scheduling          = S - C
reads plus interruption        = C - W
total                          = D - W
```

The four terms telescope exactly within either endpoint scenario. The normal Work Leaf lower bound
produces one bridge and its conservative upper bound produces the other. Percentages from another
intervention order must not be mixed into this bridge. This is an ordered causal decomposition, not
a claim that the mechanisms are independent.

`S` is not a proposed product workflow and must never enter the benchmark dashboard's normal Work
Leaf comparison. It exists only because holding the Work Leaf protocol fixed while changing the
feature schedule is the direct way to test whether concurrency causes the saving.

## What Each Substitution Holds Fixed

### `D` to `L`

Implementation agents, native Codex tools, feature order, reviewers, review/fix loops, model,
reasoning, timeouts, task, checks, and scorer stay unchanged. The linearizer receives an exact list
of the already reviewed provisional commits and their feature grouping instead of reconstructing
the target set from open-ended history inspection.

### `L` to `S`

Both workflows submit the three features sequentially, allow direct filesystem reads, let resumed
provider output finish, use compact exact linearization targets, and retain normal validation
freedom. The changed mechanism group is the Work Leaf orchestration protocol: structured edit
submissions, mediated write-producing commands, concise command/edit responses, patch ownership,
Work Leaf review routing, and the remaining difference between the two compact linearizer flows.

Implementation, review, and linearization provider sessions are measured separately. If review or
linearization consumes more under Work Leaf, it cannot be credited as a saving. This stage split
prevents a linearizer difference from being silently assigned to patch agents.

### `S` to `C`

Both use the same frozen Work Leaf binaries, prompts, direct-read policy, completed-response policy,
review flow, linearizer, observer, task, model, reasoning, and scorer. Only feature submission order
changes from one-at-a-time to all three together.

### `C` to `W`

This transition uses the completed controls from the prior study. It restores normal orchestrator
file reads and early interruption under the endpoint's recorded one-second usage grace. Their joint
effect is bounded because 35 normal endpoint responses lack provable terminal usage; every missing
response receives the event-audited 386,400-token maximum. The joint transition is retained
because the prior factorial test showed that the separate effects overlap.

## Required Gates

Every new observation must satisfy all of these gates:

1. The source, generated driver, Work Leaf binaries, observer, task, model, reasoning, and scorer
   hashes match the admission record.
2. Provider usage is exact and reconciles with saved provider rollouts for every new control. The
   normal endpoint remains bounded and must use its corrected conservative allowance.
3. There are no descendant or recursive provider sessions.
4. The intended substitution activates. Compact-target prompts must contain the exact provisional
   commits. Sequential Work Leaf sessions must not overlap feature implementation/review phases.
5. Workflow outcome, final checks, candidate replay, and all three feature scores are retained
   separately.
6. No normal Work Leaf source or product benchmark launcher is modified.

## Collection

Collect three observations for `L` and three for `S`. Use two mixed batches with at most three
top-level workflows running simultaneously. Inspect the first batch completely before admitting the
second. Do not pair outcomes; compare condition groups.

If a run reaches the provider, preserve it. Retry only an infrastructure failure that occurred
before the task reached a provider, and analyze the failure before deciding to retry.

## Completion Rule

The study is complete when valid observations permit the bridge to cover at least 90% of the
endpoint raw-token gap and quality remains comparable enough to interpret the transitions. If a
single grouped transition dominates, saved provider histories must show which stage and concrete
protocol actions changed. If the bridge cannot cover 90%, the report must name the unresolved gap
and the smallest additional control needed; it must not replace missing evidence with command-count
correlation.
