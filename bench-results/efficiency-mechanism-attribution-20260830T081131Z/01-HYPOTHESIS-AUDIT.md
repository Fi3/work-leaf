# Pre-Run Hypothesis Audit

## Decision Rule

Each explanation is challenged before a control is built. A paid run is justified only when the
same observation cannot be explained more simply by accounting, quality, ordinary variation, or an
already measured mechanism.

## Hypotheses And Counter-Hypotheses

| Hypothesis | Why it could explain the gap | Strongest alternative | Check before action | Decision |
| --- | --- | --- | --- | --- |
| Work Leaf usage is undercounted | Interrupted turns can lack terminal totals. | A repeated same-turn total is stale, and an advancing event without attributable nonzero `last` usage does not prove the response; a later total recovers usage only when its arithmetic contains an extra unexplained increment. | Require advancing same-turn totals with attributable `last` usage, reconcile later cumulative `total` and `last`, require one unresolved turn per interval, prove each uncovered tail contains one response, and apply the frozen client/model maximum. | Confirmed for 35 normal-endpoint responses; the derived 386,400-token ceiling preserves the raw result. New controls remain exact. |
| Direct usage is overcounted | Resumed CLI sessions share a thread ID. | Each CLI invocation reports only its own use. | Sum invocation totals and reconcile them to each final saved rollout. | Already contradicted for all accepted direct runs. Keep as a gate. |
| Lower Work Leaf quality creates the saving | The current endpoint scores 13/18 versus 17/18. | A real workflow effect can coexist with different feature quality. | Retain all outcomes and report quality jointly with tokens. The main controls score 8/9 versus 9/9; passing subsets remain descriptive. | Equal-quality efficiency is not established; a future study requires a predetermined quality criterion. |
| Ordinary variation creates the saving | Individual runs vary by millions of tokens. | Complete rank separation across cohorts supports a large difference under exchangeability. | Preserve three observations per new condition and report ranges and pooled permutation calculations with their assumptions. | The main controls support a substantial package effect; expected-effect precision and small transition directions remain unresolved. |
| Compact linearization saves tokens | Direct linearization uses 3.42M more raw tokens in the prior stage decomposition. | The difference may be model variance or candidate-specific cleanup. | Change only the direct linearizer's target handoff and verify its prompt and provider actions. | Paid `L` control is justified. |
| Concurrent scheduling saves tokens | Work Leaf launches three independent feature agents together. | Parallelism changes wall time but may not change tokens. | Hold the Work Leaf protocol fixed and submit features one at a time in diagnostic `S`; compare with `C`. | Paid `S` control is justified. |
| The Work Leaf orchestration protocol saves tokens | Combined Work Leaf uses far fewer write submissions and implementation generations. | Those counts may merely reflect easier model trajectories or linearization differences. | Compare `L` with sequential diagnostic `S`, then split implementation/fix, review, and linearization sessions. | Paid bridge is justified; counts alone remain insufficient. |
| Work Leaf review is cheaper | Exact commit targets and source context may reduce reviewer exploration. | Prior combined runs used more review tokens and rounds. | Measure review sessions separately in `L` and `S`. | Expected not to be a saving; do not credit it without a positive controlled difference. |
| Focused validation is the cause | Work Leaf patch agents run fewer validation commands. | Both endpoint prompts allow focused checks and defer broad checks to the final linearizer. | Keep validation freedom unchanged and inspect whether the structured protocol changes validation behavior naturally. | Do not create a validation-limiting control; it would no longer represent normal use. |
| Command-output compaction is the cause | Smaller responses can shrink later prompts. | Prior normal traces recorded zero avoided command-output bytes. | Retain output-byte measurements in `S`; do not launch a separate control unless the mechanism activates. | Low likelihood and no separate paid run. |
| Mediated reads and interruption are the cause | Both mechanisms can reduce repeated context. | Their isolated effects overlap, and missing endpoint usage can reverse the ordered `C` to `W` sign. | Use the collected joint `C` to `W` transition and propagate the endpoint bound. | Joint direction is unresolved: 0.33 million more to 1.93 million fewer raw tokens in this intervention order. The separate completed-response control uses 2.79M-5.05M more raw tokens than the interrupted endpoint in these samples. |

## Remaining Risk

The `L` to `S` transition intentionally groups several parts of one protocol: structured edits,
write-command mediation, patch ownership, concise acknowledgements, review routing, and the
remaining compact-linearizer differences. The controlled comparison supports a raw-token effect of
this package. It does not assign a separate percentage to each instruction inside that protocol.
Sequential Work Leaf uses 15.42% more uncached tokens in these exact controls, so their raw-token
result does not imply an uncached-token saving.

Stage-specific usage and provider action records locate most of the main-control raw-token
difference in implementation/fixes and review. The 97.75%-98.02% bridge allocation is descriptive:
the endpoint bounds vary missing usage but omit sampling uncertainty, and the groups have unequal
quality. These observations do not establish at least 90% causal coverage. Exact normal-workflow
telemetry and predetermined quality and precision criteria remain requirements for further
measurement.
