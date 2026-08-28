# Point 7 Result

## Abstract

Point 7 is complete. For the selected fair 3/3 observations, normal concurrent Work Leaf used fewer
raw tokens than direct sequential Codex even after charging the maximum conservative allowance for
every interrupted response. The saving also remains when all three suspected context-delivery
mechanisms are disabled in the selected observation. This proves that the observed raw saving is
real for these observations. It does not establish whether the three mechanisms explain the average
saving because one observation per condition cannot separate a repeatable effect from normal
run-to-run variation.

The exact reduction, uncached-token reduction, average reduction across repeated runs, and allocation
among the remaining causes are not established here.

## Result

| Condition | Feature score | Raw-token result versus exact direct |
| --- | ---: | ---: |
| Direct sequential Codex | 3/3 | 41,035,124 exact |
| Normal concurrent Work Leaf | 3/3 | at most 33,177,778, at least 19.15% lower |
| Work Leaf with all three controls disabled | 3/3 | at most 39,092,989, at least 4.73% lower |

The normal Work Leaf observer recorded 12,777,778 raw tokens and 51 interrupted turns. Charging the
full 400,000-token allowance to every interruption produces an upper bound of 33,177,778. This is
7,857,346 tokens below direct Codex.

The all-three-disabled observer recorded 10,292,989 raw tokens and 72 interrupted turns. The same
allowance produces an upper bound of 39,092,989. This is 1,942,135 tokens below direct Codex.

## Why The Bound Is Conservative

Every started app-server turn has exactly one terminal outcome in the saved transcripts:

| Condition | Started | Completed normally | Interrupted | JSON-RPC errors |
| --- | ---: | ---: | ---: | ---: |
| Normal Work Leaf | 63 | 12 | 51 | 0 |
| All three disabled | 80 | 8 | 72 | 0 |

Only the last provider response in an interrupted turn lacks terminal usage. The calculation still
charges a complete maximum-size response to every interruption.

All 210 normal and 180 all-three-disabled usage events report an effective context window of 258,400
tokens. The exact Codex 0.149.1 source treats this as a hard active-context cap and compacts when it
is reached. The [official GPT-5.5 documentation](https://developers.openai.com/api/docs/models/gpt-5.5)
lists 128,000 maximum output tokens. This gives 386,400 tokens before adding new-turn prompt text.

Rounding each response to 400,000 leaves 693,600 aggregate prompt headroom for normal Work Leaf and
979,200 for all-three-disabled Work Leaf. The captured prompt JSON totals only 481,131 and 785,051
bytes respectively. The declared allowances therefore exceed the context, maximum output, and
captured prompt upper bounds by 212,469 and 194,149 tokens.

The source-level context rule is in
[`context_window.rs` at Codex 0.149.1](https://github.com/openai/codex/blob/rust-v0.149.1/codex-rs/core/src/session/context_window.rs#L53-L76).
The exact transcript checks and source hashes are in `evidence.json`.

## Fairness

The three candidates use the same base commit, original task, GPT-5.5/xhigh profile, feature scorer,
and final `cargo fmt`, Clippy, and test gate. Direct Codex uses its normal sequential workflow without
Work Leaf. Work Leaf uses its normal concurrent workflow. No workflow is forced to run validation
after every iteration, and recursive provider-verification sessions are absent.

The all-three-disabled run changes only these delivery behaviors:

1. Changed-file rereads return full files instead of diffs.
2. Unchanged-file rereads return full files instead of digests.
3. Reviewers reconstruct the target from Git instead of receiving exact review context directly.

The candidates are independent observations, not a rule that one run must be discarded when another
fails. All three happen to pass 3/3, so implementation quality does not explain this comparison.

## What This Establishes

- A raw-token saving exists in these fair selected observations.
- The saving survives an intentionally excessive charge for all missing interrupted responses.
- No new paid model call was needed to reach this result.

## What Remains Unknown

- The exact raw-token saving is unknown because interrupted responses have no terminal usage.
- The uncached saving is unknown because the missing responses have no cached-input split.
- One observation per condition does not estimate the population average or normal variance.
- Whether the three tested delivery mechanisms explain the average saving remains unknown.
- This result does not allocate the remaining saving among fewer model/tool cycles, orchestration,
  command-output delivery, structured edits, linearization, or other workflow differences.

Those allocation and repetition questions belong to Points 8 and 9.
