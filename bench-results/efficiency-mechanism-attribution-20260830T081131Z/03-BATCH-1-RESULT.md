# Batch 1 Result

## Decision

Batch 1 is valid and batch 2 may run. All three workflows reached GPT-5.5 with `xhigh` reasoning,
completed their normal implementation, review, linearization, final checks, and candidate replay,
and produced exact provider accounting.

## Results

| Run | Condition | Raw tokens | Uncached tokens | Feature score |
| --- | --- | ---: | ---: | ---: |
| `compact-direct-001` | Direct Codex with exact linearization targets | 40,578,525 | 1,848,669 | 3/3 |
| `sequential-work-leaf-combined-001` | Sequential diagnostic Work Leaf | 16,704,960 | 1,582,144 | 2/3 |
| `compact-direct-002` | Direct Codex with exact linearization targets | 33,438,139 | 1,384,763 | 3/3 |

The sequential Work Leaf candidate passes visual selection and `/status`; it fails the requested
review-completion close/reopen behavior. The partial result remains evidence and is not retried.

## Instrument Checks

- Every expected provider thread has a hash-matched saved rollout. No expected, session-only, or
  same-checkout provider thread is missing.
- Every invocation completed. None was interrupted, and no descendant provider session ran.
- Report totals equal observer totals. The Work Leaf controller stream also reconciles with replayed
  app-server events for every visible agent.
- Both direct linearizer prompts contain the exact reviewed commit hashes grouped under the three
  original requests.
- Sequential Work Leaf started feature 2 only after feature 1 and its review were terminal, and
  started feature 3 only after feature 2 and its review were terminal.
- Final formatting, Clippy, tests, candidate build, and candidate replay passed for every run.

## Preliminary Reading

The first batch points to a large difference before concurrency: sequential Work Leaf used far less
raw input than either compact-direct run. This is not yet the conclusion because one Work Leaf run
cannot separate a repeatable mechanism effect from run variation, and its feature score is lower.
The second batch supplies the remaining independent observations required by the frozen protocol.
