# Step 4 Collection Plan

## Question

The first three corrected controls overlap both normal Work Leaf and direct Codex after missing
interrupted-turn usage is bounded conservatively. The next question is whether that overlap is
mainly ordinary run variation or a repeatable difference between the three workflows.

## Explanations Considered

- Ordinary variation can move any one workflow by several million tokens.
- The three disabled context mechanisms can have a repeatable combined effect.
- Direct Codex can remain higher because it performs more model and command cycles for reasons not
  covered by the three controls.
- Different completed feature quality can explain part of a token difference.
- Missing usage on interrupted Work Leaf turns can hide either a saving or a regression.
- A token-accounting or launcher mismatch can create an artificial difference.

Three more observations per group can challenge the first four explanations. Every new observation
must also reconcile its provider records and launcher settings so an accounting or fairness defect
is not silently treated as model variation. More runs cannot make interrupted-turn usage exact, so
both observed lower bounds and conservative upper bounds remain visible.

## Collection

Three batches add one observation from each independent group:

1. normal direct sequential Codex without Work Leaf;
2. normal concurrent Work Leaf; and
3. concurrent Work Leaf with changed rereads, unchanged rereads, and review-context delivery set to
   their less compact controls.

The three workflows in a batch launch simultaneously only to reduce elapsed time and spread the
same machine conditions across groups. They are not statistical pairs. A failure, partial feature
implementation, or high-token result in one group never removes any other observation.

Every workflow uses the frozen task, base commit, GPT-5.5 with `xhigh` reasoning, normal validation
freedom, final checks, scorer, source commit, and observer. No admitted provider workflow is retried.
Collection stops before another batch if a repeated infrastructure problem makes the results
unusable or if the frozen contract cannot be preserved.

After each batch, the audit checks model and reasoning settings, task and base identity, recursive
provider calls, final workflow state, feature quality, completed usage, interrupted turns, and the
conservative usage ceiling.

## Decision After Collection

The combined dataset contains six observations per group. Step 5 reports feature completion and
token distributions for the independent groups. It uses uncertainty intervals and the conservative
missing-usage bounds; it does not turn six runs into a claim of formal population equivalence.

- Repeated overlap between normal and controlled Work Leaf weakens the claim that the three context
  mechanisms explain the observed saving.
- A repeatably higher controlled group at comparable quality supports a combined mechanism effect,
  but does not allocate that effect among the three mechanisms.
- A direct group that remains higher than both Work Leaf groups supports a real workflow-level
  saving whose cause is mostly elsewhere.
- Broad or contradictory ranges leave the cause unresolved.
