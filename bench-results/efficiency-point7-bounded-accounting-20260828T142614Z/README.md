# Point 7 Bounded Accounting

## Goal

This study answers whether normal concurrent Work Leaf can still be shown to use fewer raw tokens
than fair direct sequential Codex after accounting for the provider responses that Work Leaf
interrupts. It also checks a Work Leaf run with the three previously suspected saving mechanisms
disabled.

No provider workflow was launched for this analysis. It uses three saved candidates that all passed
the same three feature checks and the final repository checks.

## Inputs

- Direct sequential Codex: `point7-exact-direct`, with exact provider usage.
- Normal concurrent Work Leaf: `wl-normal-003`.
- Concurrent Work Leaf with all three controls disabled: `wl-all-off-002`.

All three used base commit `c92a0b7060a36eac6db2d869b85e589a7a9480f9`, GPT-5.5 with xhigh
reasoning, the same original task, and the same visual, `/status`, and completion scorer. Each
candidate scored 3/3 and passed the final Cargo checks.

The disabled Work Leaf controls return full changed files instead of diffs, return full unchanged
files instead of digests, and require reviewers to reconstruct context from Git instead of receiving
the exact review target directly.

## Method

The saved Work Leaf totals include every provider response that emitted usage. They omit the final
response of each interrupted turn. The analyzer charges each omitted response 400,000 raw tokens,
which is intentionally much larger than a normal response in these runs.

The saved app-server events consistently report a 258,400-token effective context window. Codex
0.149.1 treats that value as the hard active-context limit. The [official GPT-5.5 model page](https://developers.openai.com/api/docs/models/gpt-5.5)
documents a 128,000-token maximum output. Their sum is 386,400. The remaining 13,600 tokens per
interruption cover the captured new-turn prompt text in aggregate for both Work Leaf conditions.

`analyze_bounds.py` verifies source hashes, workflow settings, feature scores, provider profiles,
turn identities, terminal outcomes, interruption identities, context-window events, prompt
headroom, and the arithmetic. It writes `evidence.json` only after every check passes.

## Files

- `FINAL-RESULT.md`: human-readable result and limits.
- `evidence.json`: machine-readable inputs, checks, bounds, and source hashes.
- `analyze_bounds.py`: reproducible analysis.
- `test_analyze_bounds.py`: transcript and arithmetic regression tests.

Run the verification from this directory with:

```sh
python3 -m unittest -v test_analyze_bounds.py
python3 analyze_bounds.py
```
