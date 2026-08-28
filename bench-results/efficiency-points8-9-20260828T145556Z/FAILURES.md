# Failures

## `wl-111-003`: Repeated Review Routing

`wl-111-003` reached the provider and is retained as an admitted attempt. It was manually stopped
after 51 minutes because feature 2 repeated the same cycle:

```text
@work-leaf done
codex: reviewer findings routed back for fixes
reviewer findings:
@work-leaf done
NO_FINDINGS
```

The reviewer transcript independently repeated:

```text
@work-leaf done
NO_FINDINGS
```

The final live inspection showed multiple identical cycles, while the feature and reviewer line
counts continued growing. This is not a slow implementation or an unresolved code finding. The
Git-reconstruction control requires a reviewer to use Work Leaf commands before returning its
decision, and the completed `done` plus `NO_FINDINGS` response is routed back as another finding.

The same failure occurred in the preserved historical all-disabled attempt documented in
`../efficiency-residual-cause-20260828T070112Z/STATE.md`. The repeated independent occurrence makes
this a systematic control/workflow incompatibility, so the remaining scheduled conditions with
Git reconstruction (`wl-001`, `wl-011`, `wl-101`, and the second `wl-111`) must not be launched.
Changing Work Leaf's review parser or changing the experimental reviewer prompt would alter the
frozen implementation or control. Neither change is permitted in this study.

The manual stop exposed an operational preservation defect. The driver removed the temporary
candidate and unpublished observer directory during signal cleanup, while `run-condition` recorded
exit code 0. The admission record and complete driver log remain, but candidate quality and token
usage are unavailable. The generated zero exit code is not treated as success and is not rewritten.

If another active attempt must be stopped, its runtime root and unpublished artifact directory must
be copied to a separately named interruption archive before signaling the driver. A manual
interruption record must then identify the signal, process, copied paths, and reason. This procedure
changes no benchmark behavior; it prevents another loss of failure evidence.

## Consequence For The Study

The endpoint repetition and the changed/unchanged reread controls remain usable. The complete
eight-setting factorial is not attainable without changing Work Leaf or the review control. Review
context therefore cannot receive a new whole-workflow token percentage from this study. Its prior
isolated one-commit result remains limited evidence, and the new loop is reliability evidence that
Git reconstruction is not behaviorally equivalent to exact inline context in this workflow.

## Batch 1 Scorer Formatting

The frozen scorer ran all three feature fixtures for `wl-100-001` and wrote
`quality/batch1.json`, then exited while formatting a one-row Markdown comparison because the
comparison object has no `token_measurements_usable` field. This is the previously observed
single-condition formatting defect. The fixture results and logs were complete before the error:
visual and `/status` pass, while completion close/reopen fails. The candidate was not rerun.
