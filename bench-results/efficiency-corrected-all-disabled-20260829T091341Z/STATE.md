# Study State

Steps (1), (2), and (3) are complete. Step (4) batches 2 and 3 have ended and batch 4 is pending.
Batch 3 contributes a passing direct result, a passing corrected-control result, and a normal Work
Leaf result that implemented 3/3 features but failed during temporary-policy cleanup. Step (5)
begins after batch 4 and its audit finish.

The benchmark-only source for batch 4 is
`72a9e507f57daf20a54bab5dcd6fe8f13f083d30`. Relative to the source used by the earlier batches, it
only sanitizes the exact temporary provider-isolation paragraph when a feature legitimately commits
an `AGENTS.md` edit. All 20 focused benchmark-workflow tests pass. The production Work Leaf
checkout, original task, frozen binaries, model settings, and frozen scorer are unchanged.

All three attempts in `SCHEDULE.tsv` ran concurrently and exited successfully. Each completed
review, linearization, final formatting, Clippy, tests, candidate build, smoke check, and artifact
capture. The frozen feature scores are 2/3, 2/3, and 3/3.

The mean conservative raw-token range is 8,146,724 to 41,613,390. It overlaps the prior normal Work
Leaf range and direct Codex mean, so the combined token effect of the three disabled mechanisms is
not known. `PROVISIONAL-REPORT.md` explains the result in plain language; `evidence.json` is the
machine-readable audit.

`STEP4-FAILURES.md` records the batch 2 launcher errors and the batch 3 temporary-policy cleanup
failure. All failed and partial outcomes remain in the evidence; no counterpart is discarded.
