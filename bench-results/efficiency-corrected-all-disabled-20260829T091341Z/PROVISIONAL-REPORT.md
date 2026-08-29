# Corrected Three-Mechanism Control: Provisional Report

## Abstract

This study asks whether three Work Leaf context-delivery mechanisms explain the previously observed
token difference between normal concurrent Work Leaf and direct sequential Codex. It disables all
three mechanisms together while preserving normal concurrent Work Leaf behavior, then runs the
original three-feature task three times with GPT-5.5 at `xhigh` reasoning.

The corrected control works: all three workflows completed review, linearization, formatting,
Clippy, tests, and candidate capture. Their independent feature scores are 2/3, 2/3, and 3/3. The
mean conservative raw-token range is 8.15M to 41.61M. That overlaps both the prior normal Work Leaf
range of 13.99M to 38.52M and the prior direct Codex mean of 35.20M.

The result does **not** identify the cause of the token difference. It also does not show that the
difference disappears when these three mechanisms are disabled. Interrupted Work Leaf turns make
the token ranges too wide, and this control averaged 2.33/3 features versus 2.67/3 in each prior
endpoint group.

## What “All Disabled” Means

Only these three candidate mechanisms are disabled:

1. A changed file is resent in full instead of being delivered as a diff.
2. An unchanged file is resent in full instead of being replaced by a digest.
3. A reviewer reconstructs its target with mediated Git commands instead of receiving the exact
   target inline.

Work Leaf itself remains enabled and concurrent. Its ordinary agent workflow, reviews,
linearization, validation opportunities, and final repository checks remain in place.

## Fairness

The control uses:

- the same candidate base, `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
- the same original task, identified by SHA-256
  `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`;
- GPT-5.5 with `xhigh` reasoning for every provider thread;
- the normal concurrent Work Leaf feature schedule;
- the normal formatting, Clippy, and test behavior, without the discarded one-test restriction;
- no recursive verification provider calls; and
- the same frozen visual, `/status`, and completion scorer used by the prior endpoint study.

The production Work Leaf checkout, original task, and scorer are unchanged. The control exists only
in the isolated benchmark source at `d217f3803ac0f417671e27cc8fb18064ff0f4ea9`. Its review prompt
requires `NO_FINDINGS` or `FINDINGS` before `@work-leaf done`, matching the existing parser contract.

## Feature Quality

| Run | Visual | `/status` | Completion | Total |
| --- | --- | --- | --- | ---: |
| `corrected-all-disabled-001` | pass | pass | fail | 2/3 |
| `corrected-all-disabled-002` | fail | pass | pass | 2/3 |
| `corrected-all-disabled-003` | pass | pass | pass | 3/3 |

The control mean is 2.33/3. The prior direct Codex and normal Work Leaf groups each average 2.67/3.
All outcomes are retained; the two partial implementations are evidence, not discarded runs.

## Mechanism Activation

| Run | Changed-file full resends | Unchanged-file full resends | Git-based reviews |
| --- | ---: | ---: | --- |
| `001` | 0 | 2 | all three reviewers |
| `002` | 4 | 5 | all three reviewers |
| `003` | 4 | 11 | all three reviewers |

The changed-file control had no matching reread opportunity in run `001`; its setting was active,
but there was no event to transform. It was exercised in runs `002` and `003`. The unchanged-file
and Git-review controls were exercised in every run. Every final review response put its review
marker before `@work-leaf done`, so the old false findings loop did not recur.

## Token Accounting

Work Leaf interrupts Codex immediately after a complete orchestrator directive. Those interrupted
turns do not expose final provider usage, so the observed token count is a lower bound rather than
an exact total.

The upper bound charges 400,000 raw tokens to every interrupted turn. For each run, this exceeds the
captured prompt size plus the 258,400-token effective context window and 128,000-token maximum
output. This is intentionally excessive; it prevents an unsupported exact-saving claim.

| Run | Observed raw lower bound | Conservative raw upper bound | Interrupted turns |
| --- | ---: | ---: | ---: |
| `001` | 8,921,177 | 50,121,177 | 103 |
| `002` | 7,051,423 | 32,251,423 | 63 |
| `003` | 8,467,571 | 42,467,571 | 85 |
| **Mean** | **8,146,724** | **41,613,390** | **83.7** |

Exact uncached totals are also unavailable. The mean observed uncached lower bound is 755,662
tokens, but it is not used for a reduction claim.

## Comparison

| Group | Runs | Mean features | Mean raw-token range |
| --- | ---: | ---: | ---: |
| Direct sequential Codex | 3 | 2.67/3 | exactly 35,196,786 |
| Normal concurrent Work Leaf | 3 | 2.67/3 | 13,989,718 to 38,523,051 |
| Corrected three-mechanism control | 3 | 2.33/3 | 8,146,724 to 41,613,390 |

The control-minus-normal difference ranges from 30.38M fewer to 27.62M more raw tokens. The
direct-minus-control difference ranges from 6.42M more to 27.05M fewer raw tokens for the control.
Both ranges cross zero.

The low observed values do not prove a saving because they omit unknown interrupted-turn usage.
The conservative upper values do not prove a regression because they deliberately assume a
near-maximum response for every interruption.

## Observer Warnings

The saved audit verifies the warnings rather than silently ignoring them:

- unfinished process records are Git review commands, not provider invocations;
- rollout threads absent from identity mapping contain only interrupted turns, all charged by the
  conservative ceiling; and
- the reviewer identity mismatch in runs `002` and `003` has its usage already included in the
  provider total under the corresponding feature-agent row.

Every app-server start, completion, and interrupt reconciles, and no recursive provider call exists.

## Conclusion

Steps (1), (2), and (3) are complete. The corrected control is usable as reliability and quality
evidence, but the three mechanisms’ combined token contribution remains unknown. Additional runs
may improve sampling, but they will not by themselves remove the broad uncertainty caused by
interrupted-turn accounting.

Steps (4) and (5) remain pending user review. Any future collection treats direct Codex, normal Work
Leaf, and this control as independent groups; a failure in one group never discards another group’s
observation.
