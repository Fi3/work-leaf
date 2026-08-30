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

The observer accepts a same-turn usage event only when its cumulative total advances and its nonzero
`last` usage fits inside that advance. It accepts a later turn as coverage for an interrupted
response only when the cumulative increase, after subtracting the later response's own `last`
usage, contains a nonzero unexplained increment. Five runs contain 35 unresolved responses; one run
is exact.

Each unresolved final response receives a conservative 1,000,000 raw-token allowance. The observed
258,400-token Codex context limit plus GPT-5.5's 128,000-token output limit permits at most 386,400
raw tokens for one response. The allowance leaves 613,600 tokens of extra headroom and is more than
five times the largest provider-reported response in the captures. `FINAL-REPORT.md`
explains the arithmetic and result. `evidence.json` contains the machine-readable comparison, and
`quality.json` preserves every candidate's feature score. Corrected replay outputs are named
`analysis-request-accounting.json`; `analysis-pre-same-turn-accounting.json` and
`analysis-cumulative.json` preserve the superseded analyses.

## Collection

`SCHEDULE.tsv` declares six independent observations. `run-batch` launched three at a time without
treating simultaneous runs as pairs. No workflow or quality result was discarded. The frozen source
and binary identities are recorded in `infrastructure/manifest.json`.
