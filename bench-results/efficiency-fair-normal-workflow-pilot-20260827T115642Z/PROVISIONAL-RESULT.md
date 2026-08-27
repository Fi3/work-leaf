# Provisional Pilot Result

## Abstract

This pilot ran exactly one normal concurrent Work Leaf workflow and one fair direct sequential
Codex workflow on the same three requests. Both started from the same commit at 14:52:08 on
2026-08-27. Both used the normal product workflow, with no artificial limit on Cargo commands, and
neither was retried. The intended profile was GPT-5.5 with `xhigh` reasoning.

The pilot does **not** produce a valid token-saving percentage. Work Leaf implemented two of the
three requested behaviors, while direct Codex implemented all three. Two measurement defects were
also detected: descendant verification calls inherited GPT-5.6 Sol instead of GPT-5.5, and direct
Codex resume usage was interpreted with the wrong accumulation rule. These are infrastructure
findings, not reasons to discard either implementation.

## Saved Implementations

| Workflow | Driver gate | Visual | `/status` | Completion | Features | Frozen token result |
| --- | --- | --- | --- | --- | ---: | --- |
| Concurrent Work Leaf | pass | pass | pass | fail | 2/3 | not usable |
| Direct sequential Codex | fail once | pass | pass | pass | 3/3 | not usable |

The Work Leaf completion failure is repeatable. Its saved candidate failed the same frozen
close/reopen check in all five additional offline runs because the required completion question was
not displayed after review.

The direct driver gate failed on
`terminal_app_mouse_wheel_scrolls_chat_history`. The unchanged saved candidate passed that exact
test in all ten additional offline runs. This makes the initial failure likely flaky, but the
original failed gate remains recorded in `report.json` and `checks.log`.

## Why Tokens Are Not Comparable

Three independent reasons prevent a supported token-efficiency claim from this pair:

1. Output quality differs: Work Leaf scored 2/3 and direct Codex scored 3/3.
2. Real-agent verification commands launched by the benchmarked agents did not specify a model.
   They inherited the machine default, producing four GPT-5.6 Sol descendant threads in Work Leaf
   and eight in direct Codex. Primary threads remained GPT-5.5/`xhigh` in both workflows.
3. `codex exec resume` reports a new per-invocation total. The observer treated those values as
   cumulative thread totals and retained only the largest invocation for each resumed thread. Its
   rollout checker detected the mismatch and marked the direct capture incomplete.

The frozen scorer therefore rejected both token measurements. The values in `result.json` are
retained as the scorer's original output, but its direct total of 23,048,981 raw tokens is not an
authoritative workflow total.

## Diagnostic Arithmetic

The raw direct capture contains 27 terminal per-invocation usage events. Summing each captured
invocation once gives this diagnostic, non-admitted view:

| Workflow | Raw input + output | Uncached input + output |
| --- | ---: | ---: |
| Concurrent Work Leaf, current observer total | 7,996,153 | 741,753 |
| Direct sequential Codex, sum of captured invocations | 35,947,089 | 1,353,041 |

Those numbers imply an apparent Work Leaf reduction of 77.756% raw and 45.179% uncached. They are
**not a study result** because the implementations have unequal quality, descendant calls used a
different model, and the corrected direct accumulation rule is not yet part of the frozen,
regression-tested observer.

## What The Pilot Established

- The repaired launchers exercised normal concurrent Work Leaf and normal direct sequential Codex.
- The exact original task was used; `/fork` was absent.
- Both primary workflows were GPT-5.5/`xhigh` and completed without a provider or launcher retry.
- Both candidate histories and all raw outcomes are saved and independently scorable.
- The quality scorer distinguishes the two saved implementations.
- The pilot exposed two measurement defects before a larger paid batch.

## Required Before More Paid Runs

1. Pin both model and reasoning effort for every descendant Codex call without editing the user's
   global configuration.
2. Make direct-exec accounting retain one terminal usage value per invocation and sum resumed
   invocations, while keeping Work Leaf app-server thread totals cumulative.
3. Add regression coverage based on the observed resume pattern and verify the saved pilot capture
   offline.
4. Correct the live state display so it reports admitted workflows immediately.
5. Run another one-pair pilot before any larger batch.

One pair cannot estimate normal variability, average implementation quality, statistical
confidence, or mechanism allocation. The study stops here pending user review; steps 8 and 9 have
not started.
