# Fair Normal-Workflow Pilot

This study checks whether the benchmark infrastructure can make one usable comparison between:

- normal concurrent Work Leaf, with all three requests submitted together; and
- normal direct Codex work, with the same requests handled one after another without Work Leaf.

Both workflows use GPT-5.5 with xhigh reasoning and the same fixed source base. The pilot records
total provider use, tests the three requested behaviors in the saved implementations, and keeps
failures and partial implementations. It does not estimate statistical confidence or attribute the
token difference to individual Work Leaf mechanisms.

The exact rules are in `FAIRNESS-CONTRACT.md`. `run-pilot` runs at most the two top-level workflows
at once and stops after writing `PROVISIONAL-RESULT.md` and `result.json`. Larger replication and
mechanism runs require a separate user decision.

`SCORER-VALIDATION.md` records the provider-free positive and negative checks of the three quality
fixtures. In particular, the literal `/status` behavior already passes at the fixed benchmark base;
the task is kept unchanged for both workflows rather than replaced with a stricter later task.

The earlier efficiency-study percentages are excluded because those runs imposed an artificial
one-Cargo-command rule and used a later slash-command task containing `/fork`. Their raw artifacts
remain available only as an audit trail.
