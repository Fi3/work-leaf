# Mechanism Decomposition

## Answer So Far

Work Leaf uses fewer tokens because it sends less accumulated input back through the model. Two
measured facts produce that result:

1. Work Leaf has fewer distinct provider usage changes, which are a practical count of model
   generation and tool cycles.
2. Each Work Leaf change carries a smaller average input context.

Both facts appear in the current detailed 6-vs-6 cohort and in the older quality-balanced 3-vs-3
cohort. This identifies the immediate source of the token difference. It does not yet prove which
Work Leaf feature causes fewer cycles or smaller contexts.

## Token Classes

The current detailed cohort has a mean raw gap of 18,644,850 tokens per workflow:

| Token class | Direct mean | Work Leaf mean | Direct minus Work Leaf |
| --- | ---: | ---: | ---: |
| Cached input | 34,507,669 | 16,128,128 | 18,379,541 |
| Uncached input | 1,398,316 | 1,147,149 | 251,167 |
| Output | 210,397 | 196,255 | 14,142 |
| Reasoning output | 101,770 | 109,976 | -8,206 |

Cached input accounts for 98.58% of the raw gap. Work Leaf actually emits slightly more reasoning
output in this cohort. The result is therefore not explained by Work Leaf doing little reasoning or
ending with unusually short answers. It comes from replaying much less accumulated input.

The older quality-balanced cohort gives the same direction: cached input accounts for 96.92% of
its 21,207,069-token raw gap.

## Cycles And Context

`decompose.py::rollout_usage_changes` reads every hash-verified Codex rollout and counts a change
only when the cumulative provider usage tuple changes. Repeated notifications with the same total
are ignored. This is more comparable than shell-command counts because both workflows report the
same provider event.

| Cohort | Direct changes | Work Leaf changes | Reduction | Direct input/change | Work Leaf input/change | Reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Current detailed, 6+6 | 320.17 | 212.33 | 33.68% | 112,148 | 81,359 | 27.45% |
| Historical balanced, 3+3 | 322.00 | 177.00 | 45.03% | 108,644 | 78,360 | 27.88% |

The two effects multiply. `decompose.py::summarize_cohort` splits their interaction equally so the
parts sum exactly to the current 18,630,708-token input gap:

| Descriptive factor | Tokens | Share of input gap |
| --- | ---: | ---: |
| Fewer provider usage changes | 10,433,254 | 56.00% |
| Smaller input context per change | 8,197,454 | 44.00% |

This is arithmetic attribution, not causal feature attribution. A feature that shortens context may
also prevent later cycles, so an isolated control is still required.

## Workflow Stages

| Stage | Direct raw | Work Leaf raw | Gap | Share of total raw gap |
| --- | ---: | ---: | ---: | ---: |
| Implementation and review fixes | 23,086,194 | 8,826,971 | 14,259,223 | 76.48% |
| Review | 6,575,109 | 4,802,117 | 1,772,992 | 9.51% |
| Linearization | 6,455,079 | 3,781,394 | 2,673,686 | 14.34% |
| Hidden title thread | 0 | 61,052 | -61,052 | -0.33% |

The main difference is produced while agents implement and respond to findings. Compact review
targets or linearization prompts may contribute, but neither stage is large enough to be the main
cause by itself.

## Compact Delivery Evidence

The normal Work Leaf captures contain direct byte measurements for some mechanisms:

- unchanged rereads: 7 verified events saved 245,578 bytes; 3 events could not be reconstructed;
- changed rereads: 40 verified events saved 1,348,829 bytes; 2 events could not be reconstructed;
- context bundles: 84 bundle paths avoided 2,049,881 bytes on the observed path;
- command results: 58 verified events saved 0 bytes compared with the recorded full result;
- linearization target compaction: the six runs lack a reconstructible byte counterfactual;
- directive interruption: all 50 measured terminal directives were interrupted, but no no-interrupt
  counterfactual exists.

The verified read and bundle differences total only a few hundred kilobytes per workflow before
conversation replay. They can be amplified when the same history is sent through many later model
cycles, but byte totals alone cannot assign a token fraction. Command-output compaction has no
measured byte effect in these runs and is not a promising first causal test.

## Command Counts

Direct Codex averaged 649 shell/tool commands and Work Leaf averaged 263 observed commands. The
counts cannot be compared as if they were the same operation. Direct Codex performs file reads as
provider tool calls; Work Leaf performs many reads as orchestrator directives and returns their
results in later prompts. The lower command count is consistent with fewer cycles, but it is not an
independent explanation.

## Hypothesis Challenges

| ID | Positive evidence | Alternatives checked | What would weaken it | Current judgment |
| --- | --- | --- | --- | --- |
| F: cached input dominates | 98.58% of the current raw gap and 96.92% of the balanced gap are cached input. | Missing output, hidden threads, and lower reasoning output were checked. | A cohort where output or uncached input explains most of the gap. | Proven for the collected samples. |
| G: fewer provider cycles | Both cohorts show 34-45% fewer distinct usage changes; all 114 rollout hashes match. | Duplicate usage notifications and incomparable command counts were removed. | A valid control with similar usage changes but no token movement. | Strong proximate explanation. |
| H: smaller context per cycle | Both cohorts show about 28% less input per usage change. | Different quality and interrupt delay were checked in the balanced uninstrumented cohort. | A valid control with similar context size but no token movement. | Strong proximate explanation. |
| I: compact delivery causes the change | Reread and bundle byte savings are directly observed. | Command outputs save zero measured bytes; review and linearization are minority stages. | Allowing direct reads leaves cycles and context unchanged. | Plausible, not isolated. |
| J: directive interruption causes the change | Fifty terminal directives are interrupted. | The older no-delay cohort shows the same result; output tokens are similar. | A no-interrupt control leaves cycle and context totals unchanged. | Possible contributor, unlikely sole cause. |
| K: parallel timing causes the change | Work Leaf implementation agents overlap in time. | Groups are independent, and the measured gap is accumulated input rather than wall time. | A valid concurrent control that removes mediated context also removes the saving. | Weak explanation. |
| D: lower quality causes the change | The current 6-vs-6 cohort has a 17/18 versus 13/18 quality gap. | The older 3-vs-3 cohort is balanced at 8/9 and reproduces both cycle and context effects. | Current-version balanced evidence shows no difference. | Cannot explain the whole gap. |
| M: version drift causes the change | Historical runner and CLI versions differ. | Current detailed runs use one frozen setup; both cohorts use GPT-5.5/`xhigh`. | A same-version controlled cohort eliminates the effect. | Remaining limitation, not the leading explanation. |

## Integrity

The analysis checked 114 local rollout files against the SHA-256 values saved at collection time.
There are no mismatches. Stage totals include implementation, review, linearization, and Work Leaf's
hidden title thread. Provider cumulative totals remain the token authority; streamed controller
totals are only a consistency signal.

## Next Decision

The highest-value isolated control is Work Leaf with direct file reads enabled through its existing
`--no-read-permission` mode. `src/agent.rs::PromptPolicy::for_read_permission` changes patch agents
from `@work-leaf read` to normal filesystem reads while preserving structured edits, mediated
write-producing commands, review routing, concurrent scheduling, task text, and final checks.

If that control increases context per usage change and raw tokens at comparable quality, it directly
supports orchestrator-mediated context delivery as a cause. If it does not, the next candidate is
directive interruption. The control must be audited against the launcher before any provider call.

## Reproduce

```sh
python3 bench-results/efficiency-causal-validation-20260829T210343Z/test_decompose.py
python3 bench-results/efficiency-causal-validation-20260829T210343Z/decompose.py
```

The complete per-run data and rollout hashes are in `decomposition-evidence.json`.
