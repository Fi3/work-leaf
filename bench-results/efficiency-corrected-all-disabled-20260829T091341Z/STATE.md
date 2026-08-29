# Study State

Steps (1) and (2) are complete. The benchmark-only review control is frozen at
`d217f3803ac0f417671e27cc8fb18064ff0f4ea9`.

Automated verification passes formatting, Clippy, all Rust tests, and the study contract tests. The
real GPT-5.5/xhigh review fixture reconstructed its target with two Git commands, ran `cargo test`,
returned `NO_FINDINGS` before `@work-leaf done`, and completed in one review round without routing
findings to the patch agent.

Step (3) consists of the three attempts in `SCHEDULE.tsv`. All three belong to launch batch 1 and
run concurrently.
