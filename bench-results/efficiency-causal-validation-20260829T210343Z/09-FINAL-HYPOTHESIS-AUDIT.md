# Final Hypothesis Audit

## Purpose

This audit challenges every explanation considered during the study before the final answer is
accepted. "Very unlikely" means the saved evidence directly contradicts the explanation for these
runs. It does not claim that the result automatically generalizes to every project.

## Audit

| Explanation | Why it could fit | Counterchecks | Judgment |
| --- | --- | --- | --- |
| Work Leaf tokens are missing | Interrupted provider output previously created incomplete totals. | Every admitted thread has exact cumulative usage, 594 combined app-server updates reconcile by component, all rollout hashes match, and no provider descendants exist. | Very unlikely. |
| Direct resume tokens are counted twice | Direct Codex launches and resumes the same thread several times. | Resume invocation totals are non-monotonic and sum exactly to the independent final total saved in each Codex rollout. They are per invocation, not repeated cumulative totals. | Very unlikely. |
| The direct workflow receives more required work | A stricter task or final gate would naturally cost more. | Both workflows use the same three requests, base commit, model, reasoning, focused implementation guidance, review duties, linearization contract, timeout, final formatting, Clippy, tests, build, replay, and scorer. | Very unlikely. |
| Work Leaf implements less | The current normal cohort scores 13/18 versus direct's 17/18. | An older equal-quality cohort scores 8/9 in both groups and preserves the gap. Combined Work Leaf scores 8/9, has comparable changed-line counts, performs more review rounds, and still sits below every direct run. | Cannot explain the saving. |
| Ordinary model variation | Individual runs vary by millions of tokens. | All six normal Work Leaf runs and all three combined runs are below all six direct runs. The current direct-versus-normal label permutation is `1/924`; direct-versus-combined is `1/84`. | Saving is strongly supported in this benchmark; exact percentage remains imprecise. |
| Codex CLI version drift | Three early direct rows use 0.149.1. | Restricting direct Codex to its three 0.150.1 rows gives 37.04M raw tokens and 9/9 features. Combined Work Leaf on 0.150.1 gives 19.40M and 8/9, 47.62% lower. | Very unlikely. |
| Hidden reviewer, title, or linearizer threads are omitted | Work Leaf has more internal sessions than direct Codex. | Every observed thread is inventoried. Work Leaf includes eight primary threads, including the hidden title thread; direct includes all implementation, review, and linearization invocations. | Ruled out. |
| Lower output or reasoning causes the gap | Stopping early could reduce generated text. | Combined Work Leaf emits 6,935 more output tokens and 14,402 more reasoning tokens than direct Codex while using 16.72M fewer raw tokens. | Ruled out for the residual gap. |
| Cached-token accounting creates an illusion | Most of the raw difference is cached input. | Raw tokens count cached input at full token volume. Combined Work Leaf also uses 448,000 more uncached input than direct Codex. The real difference is less repeated context, not missing fresh context. | Cached replay is the measured source, not an accounting error. |
| Mediated file reads are the main cause | Digests, diffs, and bundles reduce delivered file bytes. | Direct reads alone move 9.38% of the raw endpoint gap. In the combined control, reads and interruption together move only 10.34% because they overlap. | Real contributor, not the main raw cause. |
| Immediate directive interruption is the main cause | It prevents unnecessary post-directive generation. | Continued responses alone move 27.07% of the raw endpoint gap, but with direct reads they add only 179,000 raw tokens over direct-read Work Leaf. | Real contributor with strong overlap, not the main raw cause. |
| Command-output compaction is the main cause | Smaller tool output could shrink every later prompt. | Normal-run counterfactuals measured zero avoided command-output bytes, and direct versus combined command-output volume differs by less than 0.5 MB before replay. | Unlikely. |
| Less review is the main cause | Review loops can be expensive. | Combined Work Leaf averages 10.33 review rounds versus direct's 6.50, and its review stage uses slightly more raw tokens. | Ruled out. |
| Fewer provider generations cause the residual gap | Every extra generation replays accumulated context. | Combined Work Leaf has 38.26% fewer usage changes. The symmetric arithmetic split assigns 76.62% of the residual input gap to the lower count. | Strong proximate cause. |
| Smaller context per generation causes the residual gap | Shorter histories cost less on each generation. | Combined Work Leaf carries 13.47% less input per usage change. The arithmetic split assigns 23.38% of the residual input gap to this difference. | Strong proximate cause. |
| Structured workflow batching causes fewer generations | Work Leaf agents submit cohesive structured edits and mediated commands instead of many direct tool calls. | Direct averages 63.67 write submissions, combined Work Leaf 17.67; shell-tool calls fall from 634 to 429, repeated commands from 141 to 47, and validation commands from 58 to 14. Candidate size and review effort remain comparable. | High likelihood; the remaining mechanisms were tested only as a group. |
| Compact linearization causes part of the residual | Work Leaf gives the linearizer exact reviewed history rather than making it reconstruct the full workflow. | Linearization accounts for 3.42M raw tokens of the direct-minus-combined gap with only one fewer provider usage change, pointing to much smaller context rather than less linearization work. | High likelihood; not individually randomized. |
| Parallel scheduling alone causes the result | Concurrent agents may see different shared-worktree timing. | Raw-token totals measure model input, not wall time. The study does not run sequential Work Leaf because that is not a relevant product endpoint. Shared-worktree orchestration may contribute to batching, but no fraction is assigned to timing alone. | Possible part of the product workflow, not isolated. |

## Code Paths Behind The Interpretation

Normal direct Codex is defined by `bench-three-features-sequential` and
`bench-three-features-direct-common::run_feature_cycle`. Its implementation, reviewer, fix, and
linearizer prompts are produced by `implementation_prompt`, `review_prompt`, `fix_prompt`,
`linearize_plan_prompt_sequential`, and `linearize_accept_prompt_sequential`.

Normal Work Leaf agent policy is produced by `src/agent.rs::PromptPolicy::for_read_permission` and
`concurrent_work_leaf_interpretation`. Those functions require structured edits, mediated
write-producing commands, focused patch-agent checks, and final cross-feature reconciliation. The
direct-read control changes only `ReadPermission`; the continued-response control changes only the
observer's interrupt release policy.

Token totals come from `bench-observer/src/lib.rs::summarize_usage`. `decompose.py::analyze_run`
checks the saved rollout hashes and counts distinct cumulative usage changes. `analyze-combined.py`
independently reconciles app-server incremental and cumulative fields, direct analysis totals and
rollout metadata, provider action records, feature scores, review rounds, and the four control means.

## Boundary Of The Answer

The study causally isolates mediated reads, directive interruption, and their interaction. It proves
that the remaining Work Leaf workflow mechanisms collectively retain most of the raw advantage. It
does not assign separate causal percentages to structured edit batching, locked-command mediation,
focused validation policy, exact review targeting, and compact linearization.

Separating those mechanisms would require new experimental Work Leaf modes or altered prompts and
sandboxes. That would change the implementation under test and was therefore not done without user
authorization.
