# Study State

## Current Position

Collection and quality scoring are complete. The six saved Work Leaf provider streams were replayed
with corrected interrupted-response accounting. No provider workflow was rerun.

## Measurement State

- `exact-normal-004` has complete provider usage.
- Runs 001, 002, 003, 005, and 006 contain 2, 5, 22, 4, and 2 unresolved interrupted responses.
- Recorded usage is a lower bound for those five runs.
- A derived 386,400 raw-token allowance is applied to every unresolved response.
- The raw streams prove that each missing unit is one response that produced a complete directive:
  all 35 uncovered tails contain zero intervening tool boundaries.
- The allowance is the frozen client's 258,400-token hard active-context limit plus GPT-5.5's
  128,000-token maximum output. Every capture reports that active-context limit; Codex derives it
  by applying its 95% factor to the 272,000-token catalog window.
- Same-turn totals prove coverage only when they advance with attributable nonzero `last` usage;
  later totals require an unexplained increase after their own `last` usage is removed.
- Corrected replay outputs are preserved as `analysis-request-accounting.json`.

## Result

Normal Work Leaf averaged 17,471,532-19,725,532 raw tokens. Direct Codex averaged 36,116,382 exact
raw tokens. The bounded Work Leaf reduction is 45.38%-51.62%.

The uncached result ranges from 123.62% more to 16.49% fewer tokens and is not established. Work
Leaf completed 13 of 18 scored features while direct Codex completed 17 of 18, so the all-run
average is not an equal-quality efficiency claim.

`FINAL-REPORT.md` is the human-readable authority and `evidence.json` is the machine-readable result.
