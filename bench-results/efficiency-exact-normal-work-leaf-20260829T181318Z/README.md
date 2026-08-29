# Exact Normal Work Leaf Follow-up

## Goal

This study measures six normal concurrent Work Leaf workflows with exact provider token totals. It
replaces the broad per-interruption estimate used for earlier Work Leaf observations. The six new
observations are compared as a group with the six existing exact direct Codex observations in
`bench-results/efficiency-corrected-all-disabled-20260829T091341Z`.

## Fairness

Every run uses the fixed three-feature task and base commit from `bench-three-features`, GPT-5.5 with
`xhigh` reasoning, the normal concurrent Work Leaf implementation, normal validation freedom, the
same final gate, and the original quality scorer. No context-delivery experiment is enabled. The
direct comparison group uses the already-collected normal sequential Codex workflow; it does not use
Work Leaf and its token totals are exact.

The observer setting `WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS=1000` changes measurement timing,
not model work. After a complete Work Leaf directive is already visible, the observer waits briefly
for the matching provider usage event and then forwards the original `turn/interrupt`. If exact
usage does not arrive, the interrupt is forwarded at the deadline and the observation remains
explicitly incomplete. Incoming and forwarded client streams and every grace decision are retained.

## Collection

`SCHEDULE.tsv` declares six independent observations. `run-batch` launches exactly three at once,
so batches 1 and 2 provide the requested parallelism without treating simultaneous runs as
statistical pairs. A workflow failure, partial implementation, or measurement failure is retained
and does not remove another observation.

The immutable source commit is `5b1d1ef9590850faed26052f909ddff7ff8f127d`. Frozen binaries and
their hashes are recorded in `infrastructure/manifest.json`.
