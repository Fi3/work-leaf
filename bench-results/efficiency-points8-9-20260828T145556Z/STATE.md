# Study State

## Status

Batches 1 and 2 and the first balanced execution batch are closed and inspected. `wl-100-001`,
`wl-000-003`, `wl-010-001`, `direct-003`, and `wl-110-001` completed. `wl-111-003` was stopped
after a systematic review-routing loop. `FAILURES.md` records the evidence and why the remaining
Git-reconstruction conditions will not be launched. The final balanced execution batch is
`direct-002` with `wl-000-002`.

## Frozen Inputs

- infrastructure commit: `cb14a74`
- isolated instrumentation commit: `4707ceb4903a09646857d1e316cb45acb15a3d07`
- candidate base: `c92a0b7060a36eac6db2d869b85e589a7a9480f9`
- model: GPT-5.5
- reasoning: `xhigh`
- task SHA-256: `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`
- schedule seed: `8917`
- maximum simultaneous top-level workflows: 2

## Progress

| Batch | Attempts | State |
| ---: | --- | --- |
| 1 | `wl-111-003`, `wl-100-001` | closed: `wl-100-001` passed; `wl-111-003` unusable interruption |
| 2 | `wl-000-003`, `wl-010-001` | closed: both passed workflow and 3/3 features |
| 3 | `wl-001-001`, `direct-003` | `wl-001-001` withheld; `direct-003` passed workflow and 2/3 features |
| 4 | `direct-002`, `wl-011-001` | `direct-002` pending; `wl-011-001` withheld |
| 5 | `wl-111-002`, `wl-110-001` | `wl-111-002` withheld; `wl-110-001` passed workflow and 2/3 features |
| 6 | `wl-000-002`, `wl-101-001` | `wl-000-002` pending; `wl-101-001` withheld |

Every attempt remains independent. A failure in one row does not remove or invalidate another row.

## Remaining Execution Order

The systematic Git-reconstruction failure removes one member from each original batch after batch
2. The remaining safe attempts run in two predeclared concurrent execution batches:

1. `direct-003` with `wl-110-001` (complete);
2. `direct-002` with `wl-000-002`.

This keeps the two-workflow parallelism while avoiding a schedule in which both direct runs happen
together and both Work Leaf runs happen later. The execution neighbors remain independent
observations, not analytical pairs. `SCHEDULE.tsv` remains unchanged so the original random order
and every withheld row stay visible.

## Batch 1 Inspection

`wl-100-001` completed review, linearization, final formatting, Clippy, full tests, candidate replay
build, and startup smoke. It used GPT-5.5/xhigh in eight provider threads, attempted no recursive
provider call, and activated the changed-reread full-file control. The observer recorded 13,954,487
raw tokens from completed responses and correctly marked the measurement incomplete because 57
interrupted responses have no terminal usage. Its conservative upper bound is 36,754,487 raw
tokens. The frozen scorer gives it 2/3 requested features: visual mode and `/status` pass;
completion close/reopen fails.

`wl-111-003` reached all feature and review threads, then repeated the same feature-2
`done`/`NO_FINDINGS` routing cycle until manually stopped. Signal cleanup removed its temporary
candidate and unpublished capture, so its quality and token values are missing. It remains an
admitted reliability failure and does not invalidate `wl-100-001`.

## Batch 2 Inspection

Both attempts completed review, linearization, final formatting, Clippy, full tests, candidate
replay build, and startup smoke. Each used GPT-5.5/xhigh in eight primary provider threads, with no
descendant provider threads. The frozen scorer gives both implementations 3/3 requested features.

Normal `wl-000-003` recorded 12,719,646 raw tokens from completed responses and 49 interrupted
responses. Its conservative upper bound is 32,319,646 raw tokens. It used digest delivery for one
verified unchanged-file reread.

Control `wl-010-001` recorded 19,080,233 raw tokens from completed responses and 59 interrupted
responses. Its conservative upper bound is 42,728,938 raw tokens. The usual 400,000-token allowance
per interruption was 48,705 tokens short of the stricter context-window, maximum-output, and
captured-prompt formula, so the larger amount is used. It delivered the full current
file for three verified unchanged-file rereads, so the declared control activated. The observed
values differ by 6,360,587 raw tokens, but their conservative ranges overlap. This one contrast
does not separate a repeatable digest effect from normal model-path variation.

## First Balanced Execution Batch

`direct-003` and `wl-110-001` completed the normal workflow, review, linearization, final
formatting, Clippy, full tests, candidate replay build, and startup smoke. Both used GPT-5.5/xhigh,
the frozen candidate base and task, and no recursive provider calls. The frozen scorer gives both
implementations the same 2/3 requested features: visual selection and `/status` pass; completion
close/reopen fails.

`direct-003` has a complete observer capture of 28,877,983 raw tokens and 1,906,975 uncached
tokens. `wl-110-001` recorded 11,052,832 raw tokens and 871,712 uncached tokens from completed
responses. Its 57 interrupted responses make the Work Leaf values incomplete; the conservative
raw-token range is 11,052,832 to 34,694,958.

The `wl-110` control delivered full content for both repeated-read types. It activated on eight
unchanged-file rereads and eight changed-file rereads. The unchanged-file events delivered 294,593
bytes where digest delivery would have used 452 bytes. A reconstructed changed-file diff was not
available for these eight full-file events, so their byte-level counterfactual is unknown. The
whole-workflow token ranges, rather than these byte counts alone, determine the causal screen.
