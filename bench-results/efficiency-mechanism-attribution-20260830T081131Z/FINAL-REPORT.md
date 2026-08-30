# Final Report: Why Work Leaf Uses Fewer Tokens

## Abstract

This study asks why normal concurrent Work Leaf used fewer GPT-5.5/`xhigh` tokens than a fair normal
direct sequential Codex workflow on the same three-feature Rust task. It uses controlled workflows
that turn Work Leaf mechanisms on in a fixed order while keeping the model, reasoning level, task,
source revision, validation scope, final checks, scorer, and token observer fixed.

The answer covers 98.02% of the observed 18.645 million-token difference. Work Leaf's orchestration
protocol explains 87.68%, and mediated file reads plus immediate interruption after a complete
directive explain 10.34%. Concurrency does not create the saving in this benchmark.

The main reason is concrete: Work Leaf makes patch agents return a small number of complete
structured edits and mediated write commands. The orchestrator applies and commits those changes,
returns compact results, and routes review from the recorded commits. Direct Codex performs the same
workflow through many more native edit, command, and review tool cycles. Each additional cycle asks
the model to generate again with the accumulated thread, so the same cached context is replayed
repeatedly.

## What Was Compared

The accepted endpoint contains six normal direct sequential Codex runs and six normal concurrent
Work Leaf runs. All use the original three requests, base commit
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`, GPT-5.5, `xhigh` reasoning, and the frozen three-feature
quality scorer.

| Normal workflow | Runs | Feature checks | Mean raw tokens |
| --- | ---: | ---: | ---: |
| Direct sequential Codex | 6 | 17/18 | 36,116,382 |
| Concurrent Work Leaf | 6 | 13/18 | 17,471,532 |

Work Leaf uses 51.62% fewer raw tokens at this endpoint. An older equal-quality cohort also shows a
60.25% reduction at 8/9 versus 8/9 feature checks, so the endpoint difference is not explained by
quality alone.

"Raw tokens" means all input plus output tokens, including cached input. "Uncached tokens" means
fresh input plus output. Cached input matters because every new model generation can replay a large
part of the existing conversation even when the provider charges cached input differently.

## The Main Controlled Test

The main test removes the explanations that could otherwise be confused with Work Leaf itself:

- both workflows process features sequentially;
- both patch agents read files directly;
- both let provider responses finish instead of interrupting them;
- both linearizers receive compact exact commit targets;
- both have normal focused-validation freedom and the same broad final checks;
- both include implementation, review, fixes, linearization, title work, and every provider thread.

The only intentional change is the direct Codex tool workflow versus Work Leaf's normal structured
patch, write-command, ownership, and review protocol.

| Main control | Runs | Feature checks | Mean raw tokens | Range |
| --- | ---: | ---: | ---: | ---: |
| Compact direct Codex | 3 | 9/9 | 35,659,265 | 32.96M-40.58M |
| Sequential Work Leaf | 3 | 8/9 | 19,311,710 | 16.70M-23.56M |

Sequential Work Leaf uses 45.84% fewer raw tokens. Every Work Leaf run is below every direct run.
The two fully correct Work Leaf runs also remain below the lowest direct run. The exact
three-versus-three one-sided permutation result is 0.05; with only the fully correct runs it is 0.10
because only two Work Leaf observations remain.

## Why It Happens

The controlled difference appears before concurrency and before Work Leaf's read and interruption
optimizations. It is caused by the orchestration protocol:

- Direct patch agents average 55 native `apply_patch` calls. Work Leaf patch agents use no native
  patch calls and average 11.67 complete structured edit submissions.
- Direct workflows average 311 measured provider generations. Sequential Work Leaf averages 198,
  a 36.33% reduction.
- The implementation stage falls from 186.67 to 80.33 generations.
- Native implementation commands fall from 279.33 to 153.00, and native review commands fall from
  249.33 to 147.00.
- Work Leaf still performs more review rounds, 10.67 versus 6.00, so review was not omitted.

The protocol transition saves 16.59 million cached input tokens while Work Leaf consumes about
252,000 more fresh input tokens and about 9,500 more reasoning-output tokens. Work Leaf is therefore
not saving by receiving less fresh task information or by reasoning less. It saves by avoiding
repeated generations that replay the growing cached conversation.

The source path is direct:

- `src/agent.rs::PromptPolicy::for_read_permission` defines structured edits and mediated writes.
- `src/orchestrator.rs::parse_agent_directives` and `handle_agent_directives_streaming` apply those
  requests and record provisional commits outside the model.
- `src/orchestrator.rs::render_patch_applied_prompt` returns the compact post-edit handoff.
- `src/workspace.rs::should_start_review` and `start_review_for_patch_agent` route review from the
  recorded commit.

## Allocation Of The Saving

| Cause | Raw tokens | Share of endpoint gap |
| --- | ---: | ---: |
| Work Leaf orchestration protocol | 16,347,554 fewer | 87.68% |
| Mediated reads plus immediate directive interruption | 1,928,090 fewer | 10.34% |
| Compact exact linearization handoff | 457,117 fewer | 2.45% |
| Concurrent scheduling | 87,912 more | -0.47% |
| Total | 18,644,850 fewer | 100.00% |

The first two mechanisms are the repeatable Work Leaf causes and jointly cover 98.02%. The compact
handoff and concurrency values nearly cancel and are small compared with run-to-run variation.

Inside the orchestration effect, implementation and fixes explain 77.09% of the full endpoint gap,
and review explains 11.38%. Linearization and Work Leaf's extra title session consume slightly more,
reducing the net saving by 0.80 percentage points.

## Reliability Checks

- Every admitted run preserves its quality result, including the one partial Work Leaf candidate.
- Every candidate passes final formatting, Clippy, the complete test suite, build, and replay.
- Every provider thread has exact cumulative usage and a hash-matched saved rollout.
- No accepted control has a missing, interrupted, hidden descendant, or recursively launched
  provider session.
- Work Leaf controller usage reconciles with provider events.
- The sequential and compact-target controls activated in every run.
- No Work Leaf implementation, original task, normal benchmark launcher, or frozen scorer was
  modified for the controls.

The machine-readable evidence is `evidence.json`. The complete hypothesis checks and source-level
causal chain are in `05-CAUSAL-ANALYSIS.md`; collection details are in `00-PROTOCOL.md`,
`03-BATCH-1-RESULT.md`, and `04-BATCH-2-RESULT.md`.

## Conclusion

For this benchmark, the token saving is real and its cause is identified to the requested level.
Work Leaf's structured orchestration protocol causes most of the reduction by replacing many native
model/tool cycles with fewer explicit edit, command, and review handoffs. Mediated reads and early
directive interruption add a smaller saving. Together these controlled mechanisms explain 98.02%
of the observed raw-token gap.

The study proves the orchestration protocol as one connected mechanism group. Separating the exact
share of structured edits, command mediation, compact command responses, ownership, and review
routing would require narrower follow-up controls. That finer split is not needed to establish why
the current Work Leaf workflow saves tokens or to exceed the 90% causal-coverage target.
