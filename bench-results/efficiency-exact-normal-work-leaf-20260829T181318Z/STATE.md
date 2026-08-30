# Study State

## Current Position

Collection and quality scoring are complete. The six saved Work Leaf provider streams were replayed
with corrected interrupted-response accounting. No provider workflow was rerun.

## Measurement State

- `exact-normal-004` has complete provider usage.
- Runs 001, 002, 003, 005, and 006 contain 2, 5, 22, 4, and 2 unresolved interrupted responses.
- Recorded usage is a lower bound for those five runs.
- A conservative 1,000,000 raw-token allowance is applied to every unresolved response.
- Each missing unit is the final response that produced a complete directive. The allowance exceeds
  its 386,400-token context-plus-output limit by 613,600 tokens.
- Same-turn totals prove coverage only when they advance with attributable nonzero `last` usage;
  later totals require an unexplained increase after their own `last` usage is removed.
- Corrected replay outputs are preserved as `analysis-request-accounting.json`.

## Result

Normal Work Leaf averaged 17,471,532-23,304,865 raw tokens. Direct Codex averaged 36,116,382 exact
raw tokens. The bounded Work Leaf reduction is 35.47%-51.62%.

The uncached result ranges from 346.12% more to 16.49% fewer tokens and is not established. Work
Leaf completed 13 of 18 scored features while direct Codex completed 17 of 18, so the all-run
average is not an equal-quality efficiency claim.

`FINAL-REPORT.md` is the human-readable authority and `evidence.json` is the machine-readable result.
