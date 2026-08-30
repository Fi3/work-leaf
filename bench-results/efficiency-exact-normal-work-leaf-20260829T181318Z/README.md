# Normal Work Leaf Token Follow-up

## Goal

This study compares six normal concurrent Work Leaf workflows with six existing normal direct
sequential Codex workflows on the frozen three-feature benchmark. The directory name records the
original intent to collect exact totals. The final result is bounded because cancelled Codex
responses do not always report usage.

## Fairness

Every run uses the same fixed task and base commit, GPT-5.5 with `xhigh` reasoning, normal validation
freedom, final checks, and feature scorer. Direct Codex runs without Work Leaf. Work Leaf uses its
normal concurrent workflow. `/status` is scored; `/fork` is not part of the task.

The observer setting `WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS=1000` delays an already requested
interrupt for at most one second while waiting for usage. It does not change the task, prompts, or
Work Leaf implementation, but it can permit extra post-directive generation. All captured work is
counted and incoming and forwarded request streams are retained.

## Accounting

The corrected observer accepts a later cumulative total as coverage for an interrupted response
only when the cumulative increase, after subtracting the later response's own `last` usage, contains
a nonzero unexplained increment. Five runs contain ten unresolved responses; one run is exact.

Each unresolved response receives a conservative 400,000 raw-token allowance. `FINAL-REPORT.md`
explains the arithmetic and result. `evidence.json` contains the machine-readable comparison, and
`quality.json` preserves every candidate's feature score. The old `analysis-cumulative.json` files
are superseded; corrected replay outputs are named `analysis-request-accounting.json`.

## Collection

`SCHEDULE.tsv` declares six independent observations. `run-batch` launched three at a time without
treating simultaneous runs as pairs. No workflow or quality result was discarded. The frozen source
and binary identities are recorded in `infrastructure/manifest.json`.
