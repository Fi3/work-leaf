# Corrected Point 7 Study

## Goal

This study checks whether normal concurrent Work Leaf uses fewer tokens than fair normal direct
sequential Codex on the same three-feature task. It also planned one concurrent Work Leaf run with
all three previously studied context-delivery mechanisms disabled.

All conditions use GPT-5.5 with xhigh reasoning, base commit
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`, the original task containing `/status` and not
`/fork`, normal validation behavior, no recursive provider calls, and the same offline quality
scorer. Direct and Work Leaf outcomes are retained independently.

## Outcome

The execution gate is closed, but it did not produce a valid Work Leaf token total or a token-saving
percentage.

- Direct sequential Codex completed all three features. Its exact total is 41,035,124 raw tokens and
  1,982,580 uncached tokens.
- The normal Work Leaf attempt used an exact-accounting experiment that delayed directive handling.
  That changed normal Work Leaf behavior, caused two review turns to time out, and left a partial
  two-feature candidate. Its recorded token total is incomplete and unusable.
- The all-three-disabled Work Leaf run was not launched after the shared accounting defect was
  confirmed. It would have produced another knowingly incomplete Work Leaf total.

Normal Work Leaf interrupts a provider response immediately after a complete orchestrator directive.
The ChatGPT Codex transport used here reports exact usage only when that response completes, and it
does not expose usage after interruption. Waiting for completion is not a measurement-only change:
the model can continue generating and change the workflow before Work Leaf handles the directive.

`FINAL-RESULT.md` gives the plain-language result. `FAILURE-ANALYSIS.md` records the real-provider
probes and source call chain. `BATCH1-RESULT.md` and `batch1-result.json` are the frozen scorer output,
and `runs/` preserves both attempted workflows.

## Frozen Inputs

- candidate base: `c92a0b7060a36eac6db2d869b85e589a7a9480f9`
- task SHA-256: `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`
- model and reasoning: GPT-5.5/xhigh
- attempted accounting implementation: `db5fe21`

The accounting implementation is recorded for provenance only. It was removed from the active
benchmark code after this study because it does not preserve normal Work Leaf behavior.
