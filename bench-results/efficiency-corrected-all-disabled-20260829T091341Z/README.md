# Corrected All-Disabled Work Leaf Study

## Goal

This study collects three independent observations of concurrent Work Leaf with these three context
delivery mechanisms disabled:

1. changed-file rereads send the full current file instead of a diff;
2. unchanged-file rereads resend the full file instead of a digest; and
3. reviewers reconstruct the exact review target from Git instead of receiving that context inline.

The third control preserves the normal review route. A reviewer may use mediated Git commands while
gathering context, but its final response places `NO_FINDINGS` or `FINDINGS` before the standard
`@work-leaf done` directive. This matches the parser contract and prevents a clean review from being
routed to the patch agent as findings.

## Scope

The production Work Leaf checkout, original three-feature task, and frozen feature scorer are not
modified. Batches 1 through 3 use isolated source commit
`d217f3803ac0f417671e27cc8fb18064ff0f4ea9`. Batch 4 uses
`72a9e507f57daf20a54bab5dcd6fe8f13f083d30`, whose only additional behavior removes the exact
temporary provider-isolation paragraph when a feature legitimately commits an `AGENTS.md` update.
Both revisions are based on the Points 8/9 instrumentation commit
`4707ceb4903a09646857d1e316cb45acb15a3d07`.

Every provider thread uses GPT-5.5 with `xhigh` reasoning. The concurrent Work Leaf workflow keeps
its normal validation behavior and final formatting, Clippy, and test gate. Recursive provider
verification is blocked by the existing benchmark profile.

Each collection batch runs three workflows concurrently with separate temporary roots, result
directories, observer identities, and run IDs. They are independent observations, not pairs. A
failure in one attempt does not remove or invalidate another attempt.

## Accounting

Completed response usage is observed directly. Interrupted Work Leaf responses use the established
conservative raw-token ceiling based on the emitted effective context window, maximum output, and
captured new-turn prompt size. No exact Work Leaf token percentage is reported when interrupted
response usage is missing.

## Results

The final independent groups contain 6 direct Codex runs, 5 normal Work Leaf runs, and 6
all-disabled Work Leaf controls. Direct Codex averages 36.12M raw tokens. Normal Work Leaf averages
between 13.05M known tokens and a 35.05M conservative upper bound, leaving a collected-sample saving
of 1.07M to 23.07M tokens, or 2.96% to 63.87%. A 3/3-feature subset gives the same direction.

The all-disabled control averages 8.85M to 43.38M raw tokens. Its difference from normal Work Leaf
crosses zero, and its feature score is lower. The three candidate context mechanisms therefore do
not receive a causal token fraction from this study. Fewer total, repeated, and validation command
cycles are the strongest remaining explanation, but that explanation is associated rather than
isolated.

`FINAL-REPORT.md` explains the final result in plain language. `final-evidence.json` contains every
included observation, source hash, feature score, token interval, and descriptive bootstrap result.
`step5-analyze.py` reproduces it without launching a provider. `PROVISIONAL-REPORT.md` and
`evidence.json` remain the Steps 1-3 checkpoint.

The complete attempts remain under `runs/` for local replay and reanalysis. Git stores their compact
reports and audits rather than the large raw app-server streams and duplicate binaries.

## Commands

`./test-study` validates the frozen source, binaries, schedule, and launcher contract without a
provider call. `python3 -m unittest test_step5_analyze.py` validates the final analysis, and
`python3 step5-analyze.py` rebuilds the final evidence. `python3 analyze.py` rebuilds the initial
three-control checkpoint. `./run-batch BATCH_ID` rejects all completed schedule rows and must not be
used to replay preserved attempts.
