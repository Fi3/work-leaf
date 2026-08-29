# Study State

Steps (1) through (5) are complete. The final groups contain 6 direct Codex observations, 5 normal
Work Leaf observations, and 6 all-disabled Work Leaf observations. All real workflow failures and
partial feature results remain included. The batch-2 normal attempt remains separate because invalid
environment values prevented the intended workflow from executing.

The benchmark-only source for batch 4 is
`72a9e507f57daf20a54bab5dcd6fe8f13f083d30`. Relative to the source used by the earlier batches, it
only sanitizes the exact temporary provider-isolation paragraph when a feature legitimately commits
an `AGENTS.md` edit. All 20 focused benchmark-workflow tests pass. The production Work Leaf
checkout, original task, frozen binaries, model settings, and frozen scorer are unchanged.

The initial three controls ran concurrently and exited successfully. Each completed review,
linearization, final formatting, Clippy, tests, candidate build, smoke check, and artifact capture.
Their frozen feature scores are 2/3, 2/3, and 3/3.

The collected-sample direct-minus-normal raw-token difference is 1.07M to 23.07M tokens, or 2.96%
to 63.87% fewer for normal Work Leaf. The descriptive bootstrap envelope still crosses zero, so a
population-average claim is not statistically established. The all-disabled-minus-normal interval
also crosses zero, so the three tested mechanisms receive no causal fraction.

`FINAL-REPORT.md` is the human-readable conclusion and `final-evidence.json` is its reproducible
machine evidence. `STEP4-FAILURES.md` records the launcher, cleanup, and final-test failures. No
counterpart was discarded.
