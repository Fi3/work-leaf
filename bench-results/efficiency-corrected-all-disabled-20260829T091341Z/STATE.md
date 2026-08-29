# Study State

Steps (1), (2), and (3) are complete. Steps (4) and (5) have not started.

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
