# Study State

## Status

Batch 1 is closed and inspected. `wl-100-001` completed. `wl-111-003` was stopped after a
systematic review-routing loop. `FAILURES.md` records the evidence and why the remaining
Git-reconstruction conditions will not be launched.

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
| 2 | `wl-000-003`, `wl-010-001` | pending |
| 3 | `wl-001-001`, `direct-003` | pending |
| 4 | `direct-002`, `wl-011-001` | pending |
| 5 | `wl-111-002`, `wl-110-001` | pending |
| 6 | `wl-000-002`, `wl-101-001` | pending |

Every attempt remains independent. A failure in one row does not remove or invalidate another row.

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
