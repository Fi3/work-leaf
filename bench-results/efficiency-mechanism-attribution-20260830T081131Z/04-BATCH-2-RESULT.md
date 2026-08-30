# Batch 2 Result

## Decision

Batch 2 is valid. All three workflows reached GPT-5.5 with `xhigh` reasoning, completed
implementation, review, linearization, final checks, and candidate replay, and produced exact
provider accounting. No workflow was retried.

## Results

| Run | Condition | Raw tokens | Uncached tokens | Feature score |
| --- | --- | ---: | ---: | ---: |
| `compact-direct-003` | Direct Codex with exact linearization targets | 32,961,130 | 1,407,978 | 3/3 |
| `sequential-work-leaf-combined-002` | Sequential diagnostic Work Leaf | 17,666,999 | 1,817,399 | 3/3 |
| `sequential-work-leaf-combined-003` | Sequential diagnostic Work Leaf | 23,563,172 | 1,957,540 | 3/3 |

Together with batch 1, compact direct scores 9/9 and sequential Work Leaf scores 8/9. The one
failed check is the completion close/reopen behavior in `sequential-work-leaf-combined-001`; that
result remains in the analysis.

## Instrument Checks

- Every expected provider thread has a hash-matched saved rollout.
- No expected, session-only, or same-checkout provider thread is missing.
- Every invocation completed, no provider response was interrupted, and no descendant provider
  session ran.
- Reports, observer totals, and Work Leaf controller streams reconcile.
- Every compact-direct linearizer prompt contains the exact reviewed commits grouped under the
  three original feature requests.
- Every sequential Work Leaf run started each later feature only after the preceding feature and
  review were terminal.
- Final formatting, Clippy, tests, candidate build, and candidate replay passed for every run.

## Group Result

The three compact-direct runs average 35,659,265 raw tokens. The three sequential Work Leaf runs
average 19,311,710, a difference of 16,347,554 tokens. The ranges do not overlap:

- compact direct: 32,961,130 to 40,578,525;
- sequential Work Leaf: 16,704,960 to 23,563,172.

Both fully correct sequential Work Leaf candidates remain below the lowest compact-direct result.
This makes lower implementation quality an inadequate explanation for the token difference.

The complete causal interpretation is in `05-CAUSAL-ANALYSIS.md`.
