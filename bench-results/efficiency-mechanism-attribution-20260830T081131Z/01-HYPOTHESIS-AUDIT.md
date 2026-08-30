# Pre-Run Hypothesis Audit

## Decision Rule

Each explanation is challenged before a control is built. A paid run is justified only when the
same observation cannot be explained more simply by accounting, quality, ordinary variation, or an
already measured mechanism.

## Hypotheses And Counter-Hypotheses

| Hypothesis | Why it could explain the gap | Strongest alternative | Check before action | Decision |
| --- | --- | --- | --- | --- |
| Work Leaf usage is undercounted | Interrupted turns previously lacked immediate totals. | Later cumulative totals may already recover all usage. | Require exact cumulative totals, component reconciliation, rollout hashes, and no descendants. | Already contradicted by the accepted endpoint and combined controls. Keep as a gate. |
| Direct usage is overcounted | Resumed CLI sessions share a thread ID. | Each CLI invocation reports only its own use. | Sum invocation totals and reconcile them to each final saved rollout. | Already contradicted for all accepted direct runs. Keep as a gate. |
| Lower Work Leaf quality creates the saving | The current endpoint scores 13/18 versus 17/18. | A real workflow effect can coexist with noisy feature quality. | Retain all outcomes and compare the older equal-quality 8/9 versus 8/9 cohort. Require each new condition's quality. | Quality cannot explain the whole gap, but remains a required covariate. |
| Ordinary variation creates the saving | Individual runs vary by millions of tokens. | Complete rank separation across cohorts is unlikely under exchangeability. | Preserve three observations per new condition and report ranges and exact rank tests descriptively. | Cannot explain the endpoint direction; it can blur smaller bridge transitions. |
| Compact linearization saves tokens | Direct linearization uses 3.42M more raw tokens in the prior stage decomposition. | The difference may be model variance or candidate-specific cleanup. | Change only the direct linearizer's target handoff and verify its prompt and provider actions. | Paid `L` control is justified. |
| Concurrent scheduling saves tokens | Work Leaf launches three independent feature agents together. | Parallelism changes wall time but may not change tokens. | Hold the Work Leaf protocol fixed and submit features one at a time in diagnostic `S`; compare with `C`. | Paid `S` control is justified. |
| The Work Leaf orchestration protocol saves tokens | Combined Work Leaf uses far fewer write submissions and implementation generations. | Those counts may merely reflect easier model trajectories or linearization differences. | Compare `L` with sequential diagnostic `S`, then split implementation/fix, review, and linearization sessions. | Paid bridge is justified; counts alone remain insufficient. |
| Work Leaf review is cheaper | Exact commit targets and source context may reduce reviewer exploration. | Prior combined runs used more review tokens and rounds. | Measure review sessions separately in `L` and `S`. | Expected not to be a saving; do not credit it without a positive controlled difference. |
| Focused validation is the cause | Work Leaf patch agents run fewer validation commands. | Both endpoint prompts allow focused checks and defer broad checks to the final linearizer. | Keep validation freedom unchanged and inspect whether the structured protocol changes validation behavior naturally. | Do not create a validation-limiting control; it would no longer represent normal use. |
| Command-output compaction is the cause | Smaller responses can shrink later prompts. | Prior normal traces recorded zero avoided command-output bytes. | Retain output-byte measurements in `S`; do not launch a separate control unless the mechanism activates. | Low likelihood and no separate paid run. |
| Mediated reads and interruption are the cause | Both mechanisms reduce some repeated context. | Their isolated effects overlap heavily. | Use the already completed joint `C` to `W` transition. | Proven contributor, jointly 10.34% in the prior intervention order. |

## Remaining Risk

The `L` to `S` transition intentionally groups several parts of one protocol: structured edits,
write-command mediation, patch ownership, concise acknowledgements, review routing, and the
remaining compact-linearizer differences. It answers whether the Work Leaf orchestration protocol
causes the missing saving. It does not assign a separate percentage to each instruction inside that
protocol.

If that grouped transition explains most of the endpoint, stage-specific usage and provider action
records will identify whether implementation/fixes or review owns the effect. A deeper split is
justified only after that result; building every hypothetical switch in advance would create more
confounding infrastructure than evidence.
