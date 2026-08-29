# Step 4 Failures

## Batch 2 Normal Work Leaf

`step4-normal-001` is an infrastructure failure and is not a normal Work Leaf efficiency
observation. Its admission and all captured evidence remain saved.

The launcher used the human-readable labels `diff` and `digest` as environment values. The frozen
Work Leaf parser accepts `normal` or `full` for those two switches. Three GPT-5.5/xhigh feature
turns reached the provider and returned orchestrator directives, then Work Leaf panicked while
processing each directive. No candidate, review, linearization, or final checks followed.

The attempt is not retried. Later scheduled normal Work Leaf attempts use `normal` for both
environment values while their admission records continue to describe the resulting behavior as
`diff` and `digest`. `test-study` asserts both the semantic labels and accepted environment values.

## Live Launcher Edit

The launcher mapping was corrected while batch 2 wrappers were still active. The running shells
later read the replaced file and exited with a parse error after their benchmark children returned.
This was an operator error. No launcher file may be edited while a batch is active.

The error did not invalidate the completed direct and corrected-control children:

- `step4-direct-001` published a passing report, exact complete observer usage, a candidate, final
  checks, and a successful candidate smoke check before its wrapper error.
- `step4-control-001` published a passing report, conservative interrupted-turn telemetry, a
  candidate, final checks, and a successful candidate smoke check before its wrapper error.

Their outer wrapper exit code remains `2` as evidence of the operator error. Their child reports,
observer records, candidates, and quality scores determine whether they are usable. No attempt is
rerun to replace the wrapper status.

## Effect On Collection

Batch 2 contributes one usable direct observation, no usable normal Work Leaf token observation,
and one usable corrected-control observation. The invalid normal attempt remains reliability
evidence but is excluded from token and feature-efficiency distributions because the requested
workflow never executed.

Batches 3 and 4 remain the only scheduled provider work. If both complete, the final groups contain
six usable direct observations, five usable normal Work Leaf observations, and six usable corrected
controls after historical endpoint observations are included. Step 5 supports unequal group sizes;
an extra provider batch is not justified solely to replace this operator-caused observation.

## Batch 3 Temporary Instruction Policy

`step4-normal-002` implemented and reviewed all three features, then failed after linearization. The
frozen scorer reconstructs its saved output and scores it 3/3. Its workflow report remains `fail`,
and its observed token count remains a lower bound because 41 interrupted turns lack final usage.

The failure came from benchmark infrastructure. To prevent recursive provider calls, the benchmark
temporarily appended a provider-isolation paragraph to tracked `AGENTS.md`. The visual-selection
feature legitimately updated `AGENTS.md`, and its final commit consequently contained both the real
documentation edit and the temporary paragraph. The old cleanup accepted only an entirely unchanged
file and stopped before final checks.

The batch-4 wrapper removes only the exact temporary paragraph from both the final file and its
commit. It rejects a moved or modified policy and rejects uncommitted instruction changes. The real
documentation edit is preserved, the commit count stays at three, and the final clean-tree check
still applies. The regression test reproduces the committed documentation-edit case. All 20 focused
benchmark-workflow tests pass. Production Work Leaf, the task, scorer, frozen binaries, model,
reasoning level, and measured workflow are unchanged.

The source revision for batches 1 through 3 is
`d217f3803ac0f417671e27cc8fb18064ff0f4ea9`; batch 4 uses
`72a9e507f57daf20a54bab5dcd6fe8f13f083d30`. This is a post-workflow cleanup correction and does not
alter provider prompts or model behavior.
