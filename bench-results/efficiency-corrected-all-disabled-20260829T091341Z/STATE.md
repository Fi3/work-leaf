# Study State

Steps (1), (2), and (3) are complete. Step (4) batch 2 has ended. Its direct and corrected-control
children completed; its normal Work Leaf row is an infrastructure failure caused by invalid launcher
switch values. Batches 3 and 4 have not started. Step (5) begins only after collection and audit
finish.

The benchmark-only review control is frozen at
`d217f3803ac0f417671e27cc8fb18064ff0f4ea9`. Its automated checks and bounded real-agent review
verification pass. The production Work Leaf checkout, original task, and frozen scorer are
unchanged.

All three attempts in `SCHEDULE.tsv` ran concurrently and exited successfully. Each completed
review, linearization, final formatting, Clippy, tests, candidate build, smoke check, and artifact
capture. The frozen feature scores are 2/3, 2/3, and 3/3.

The mean conservative raw-token range is 8,146,724 to 41,613,390. It overlaps the prior normal Work
Leaf range and direct Codex mean, so the combined token effect of the three disabled mechanisms is
not known. `PROVISIONAL-REPORT.md` explains the result in plain language; `evidence.json` is the
machine-readable audit.

`STEP4-FAILURES.md` records the batch 2 launcher errors, why the failed normal row is unusable, why
the two completed child reports remain usable despite outer wrapper errors, and why no attempt is
retried.
