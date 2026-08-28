# Hypotheses And Challenges

This file applies `../efficiency-residual-cause-20260828T070112Z/HYPOTHESIS-PROTOCOL.md` and records
how each challenge resolved.

## Repeated Overall Saving

Hypothesis: normal concurrent Work Leaf has a repeatable raw-token advantage over normal direct
sequential Codex on this task.

Credible alternatives:

- the Point 7 difference is ordinary model variation;
- one workflow completes less requested behavior;
- direct resume usage is counted differently;
- interrupted Work Leaf responses are still undercounted;
- the launchers differ in task, model, reasoning, validation, final checks, or recursive calls; or
- two simultaneous workflows create a condition-specific resource effect.

The repeated independent endpoint groups test a stable difference against one-run variation.
Feature scoring tests the less-work explanation. Saved provider metadata and the conservative
interruption bound address accounting. Frozen launch fields and per-batch inspection address
fairness. The bound can establish only a minimum raw saving, not an exact average reduction.

Outcome: both groups average 2.67 completed features. Work Leaf's observed mean is 60.25% lower,
but the conservative difference ranges from 21.21 million fewer to 3.33 million more raw tokens.
The repeated average saving remains inconclusive because interrupted-response headroom is wider
than the observed gap.

## Three Delivery Mechanisms

Hypothesis: changed-file diffs, unchanged-file digests, and exact inline review context reduce Work
Leaf token use.

Credible alternatives:

- a mechanism has no opportunity to activate in a run;
- model variation is larger than its effect;
- implementation quality or review loops cause the difference;
- mechanisms interact, so one-at-a-time comparisons are misleading; or
- missing interrupted-response usage is distributed differently across conditions.

The completed four-setting read screen exposes changed-file and unchanged-file directions. The
review-context controls could not complete without changing workflow behavior. The analysis reports
activation counts, quality, model/tool cycles, observed usage, and conservative bounds together.
It assigns no percentage when bounds overlap or a valid control is unavailable.

Outcome: both read controls activated, but their conservative whole-workflow ranges cross zero and
quality differs across cells. Git review reconstruction repeatedly broke review routing, so its
effect is not estimable. None of the three mechanisms receives a causal token percentage.

## Remaining Difference

Hypothesis: if `wl-111` remains lower than direct Codex across repeated comparable-quality runs, the
remaining advantage is mainly fewer model/tool cycles created by Work Leaf's broader orchestration
workflow rather than the three delivery mechanisms.

Credible alternatives:

- the endpoint difference is still normal variation;
- the conservative interrupted-response bound is too broad to distinguish the groups;
- app-server and direct-CLI transport differences cause the result;
- Work Leaf produces less robust behavior outside the frozen scorer; or
- several untested mechanisms jointly cause the remaining difference.

Endpoint repetitions can test a one-run explanation but cannot by themselves prove which untested
mechanism is causal. Stage-level cycle counts can support or contradict the fewer-cycle
explanation. A remaining gap stays unresolved unless a controlled condition changes that cycle
pattern and the token bounds in the predicted direction.

Outcome: normal Work Leaf averages 57.79% fewer commands, 93.05% fewer repeated commands, and
50.63% fewer validation commands. This strongly supports the fewer-cycle explanation, but no
controlled cycle ablation was run and the all-disabled endpoint was not replicated. The exact
cause and fraction remain unresolved.

## Interpretation Rules

- All outcomes remain evidence, including partial features and workflow failures.
- Batch neighbors are not analytical pairs.
- One condition observation cannot establish a stable mechanism effect.
- Percentages are calculated only after absolute token bounds and feature quality are shown.
- Exact uncached attribution is unavailable while interrupted responses lack usage.
- A result that fits both a mechanism effect and normal variation is inconclusive.
