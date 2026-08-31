# Final Hypothesis Audit

## Purpose

This audit challenges every explanation considered during the study before the final answer is
accepted. "Very unlikely" means the saved evidence directly contradicts the explanation for these
runs. It does not claim that the result automatically generalizes to every project.

## Audit

| Explanation | Why it could fit | Counterchecks | Judgment |
| --- | --- | --- | --- |
| Work Leaf tokens are missing | Interrupted provider output can lack terminal usage, and repeated cumulative notifications can be stale. | Strict same-turn freshness and later-turn arithmetic find 35 unresolved responses. Raw-event replay proves that each gap is one response with no intervening tool boundary. The direct-read, continued-response, and combined controls remain exact. | Confirmed for the normal endpoint and covered by the derived 386,400-token response ceiling. |
| Direct resume tokens are counted twice | Direct Codex launches and resumes the same thread several times. | Resume invocation totals are non-monotonic and sum exactly to the independent final total saved in each Codex rollout. They are per invocation, not repeated cumulative totals. | Very unlikely. |
| The direct workflow receives more required work | A stricter task or final gate would naturally cost more. | Both workflows use the same three requests, base commit, model, reasoning, focused implementation guidance, review duties, linearization contract, timeout, final formatting, Clippy, tests, build, replay, and scorer. | Very unlikely. |
| Work Leaf implements less | The current normal cohort scores 13/18 versus direct's 17/18. | The later exact main control scores 9/9 for compact direct and 8/9 for sequential Work Leaf. Both fully correct Work Leaf controls remain below every direct control, but only two Work Leaf observations are fully correct. | Cannot plausibly explain the full controlled effect; formal quality equivalence is not proven. |
| Ordinary model variation | Individual runs vary by millions of tokens. | In the exact main control, all three sequential Work Leaf totals are below all three compact-direct totals; the one-sided permutation result is `0.05`. The bounded normal endpoint also remains below direct on average under the maximum allowance. | Unlikely to explain the full effect in this benchmark; the sample is too small for a population estimate. |
| Codex CLI version drift | Three early direct rows use 0.149.1. | Restricting direct Codex to its three 0.150.1 rows gives 37.04M raw tokens and 9/9 features. Combined Work Leaf on 0.150.1 gives 19.40M and 8/9, 47.62% lower. | Very unlikely. |
| Hidden reviewer, title, or linearizer threads are omitted | Work Leaf has more internal sessions than direct Codex. | Every observed thread is inventoried. Work Leaf includes eight primary threads, including the hidden title thread; direct includes all implementation, review, and linearization invocations. | Ruled out. |
| Lower output or reasoning causes the gap | Stopping early could reduce generated text. | Combined Work Leaf emits 6,935 more output tokens and 14,402 more reasoning tokens than direct Codex while using 16.72M fewer raw tokens. | Ruled out for the residual gap. |
| Cached-token accounting creates an illusion | Most of the raw difference is cached input. | Raw tokens count cached input at full token volume. Combined Work Leaf also uses 448,000 more uncached input than direct Codex. The real difference is less repeated context, not missing fresh context. | Cached replay is the measured source, not an accounting error. |
| Mediated file reads are the main cause | Digests, diffs, and bundles reduce delivered file bytes. | Direct-read Work Leaf averages 19.22M exact, while normal Work Leaf is bounded at 17.47M-19.73M. The effect changes sign across that interval. | Not proven as an independent raw-token saving; not needed for the dominant protocol effect. |
| Immediate directive interruption is the main cause | It prevents unnecessary post-directive generation. | Continued-response Work Leaf averages 22.52M exact, 2.79M-5.05M above normal Work Leaf's 17.47M-19.73M bound. | Confirmed as a saving in this control, but smaller than the 16.35M orchestration effect and not the main cause. |
| Command-output compaction is the main cause | Smaller tool output could shrink every later prompt. | Normal-run counterfactuals measured zero avoided command-output bytes, and direct versus combined command-output volume differs by less than 0.5 MB before replay. | Unlikely. |
| Less review is the main cause | Review loops can be expensive. | Combined Work Leaf averages 10.33 review rounds versus direct's 6.50, and its review stage uses slightly more raw tokens. | Ruled out. |
| Fewer provider generations cause the residual gap | Every extra generation replays accumulated context. | Combined Work Leaf has 38.26% fewer usage changes. The symmetric arithmetic split assigns 76.62% of the residual input gap to the lower count. | Strong proximate cause. |
| Smaller context per generation causes the residual gap | Shorter histories cost less on each generation. | Combined Work Leaf carries 13.47% less input per usage change. The arithmetic split assigns 23.38% of the residual input gap to this difference. | Strong proximate cause. |
| Structured workflow batching causes fewer generations | Work Leaf agents submit cohesive structured edits and mediated commands instead of many direct tool calls. | The later exact main control holds scheduling, reads, response completion, and compact targets fixed. Sequential Work Leaf uses 16.35M fewer raw tokens and 113 fewer model generations than compact direct Codex. | Confirmed as the dominant orchestration package for this benchmark. |
| Compact linearization causes part of the residual | Work Leaf gives the linearizer exact reviewed history rather than making it reconstruct the full workflow. | The later exact `D` to `L` control saves 457,117 raw tokens, only 2.45%-2.80% of the bounded endpoint gap. | Small contributor, not the main cause. |
| Parallel scheduling alone causes the result | Concurrent agents may see different shared-worktree timing. | The later exact sequential-to-concurrent Work Leaf control changes raw use by only 87,912 tokens in the opposite direction. | Not the cause of the saving in this benchmark. |

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

The read, continued-response, and combined controls are exact. The read-only and combined effects
relative to normal Work Leaf change sign under the maximum missing-token allowance. The
continued-response control remains above normal across the interval, so early interruption has a
bounded positive saving in these samples. The later exact main control
proves that the Work Leaf orchestration package retains the dominant advantage when scheduling,
reads, response completion, and compact targets are held fixed.

Separating structured edit batching, write-command mediation, compact acknowledgements, ownership,
and review routing inside that package requires new experimental modes or altered prompts and
sandboxes. That was not done without user authorization.
