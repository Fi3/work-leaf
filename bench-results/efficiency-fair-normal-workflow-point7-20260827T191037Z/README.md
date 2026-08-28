# Superseded Fair Normal-Workflow Point-7 Gate

## Important Result Status

The candidates and quality scores in this directory remain valid, but its Work Leaf token total is
incomplete. Normal Work Leaf interrupted directive responses whose usage was not reported by the
Codex transport. The recorded 49.039% raw and 26.256% uncached reductions are withdrawn. See
`../efficiency-point7-exact-accounting-20260828T113610Z/FINAL-RESULT.md` for the corrected result.

## Goal

Run exactly one normal concurrent Work Leaf workflow and one fair direct sequential Codex workflow
against the same frozen three-feature task. Both workflows use GPT-5.5 with xhigh reasoning. The gate
checks that the repaired benchmark infrastructure can complete both workflows and produce usable
token measurements without changing Work Leaf, the task, or the quality scorer.

This study replaces only the failed infrastructure gate recorded in
`../efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z`. It does not replace or delete that
evidence.

## Stop Rule

The study admits one workflow from each condition with no automatic retry. It preserves and scores
every outcome, including partial or failed implementations. Steps 8 and 9 remain stopped until the
user reviews this point-7 result.

## Fixed Inputs

- benchmark base: `c92a0b7060a36eac6db2d869b85e589a7a9480f9`
- task-list SHA-256: `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`
- model: GPT-5.5
- reasoning: xhigh
- Work Leaf schedule: three requests submitted concurrently
- direct schedule: the same requests handled sequentially without Work Leaf

`PREFLIGHT.md` records the infrastructure checks and real write smoke. `FINAL-RESULT.md` explains
which evidence remains usable. `PROVISIONAL-RESULT.md` and `result.json` preserve the original
generated output, and `runs/` contains both complete workflow records.
