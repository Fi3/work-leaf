# Final Report

## Abstract

This study asks two questions:

1. Does normal concurrent Work Leaf use fewer raw tokens than normal sequential Codex on the same
   three-feature task?
2. If it does, is the saving caused by compact changed-file rereads, compact unchanged-file rereads,
   and inline review context?

The final dataset contains 6 direct Codex runs, 5 normal Work Leaf runs, and 6 Work Leaf runs with
all three candidate mechanisms disabled. The workflows use the same task, starting commit,
GPT-5.5 model, `xhigh` reasoning, validation opportunities, and final feature scorer. Runs are
independent observations rather than pairs, and failed or partial implementations remain in their
groups.

In this collected sample, direct Codex averaged 36.12M raw tokens. Normal Work Leaf averaged between
13.05M known tokens and 35.05M tokens under the deliberately extreme missing-token ceiling. The
sample therefore shows between 1.07M and 23.07M fewer raw tokens for Work Leaf, or 2.96% to 63.87%.
The lower end assumes every interrupted Work Leaf response consumed the full 400,000-token ceiling.

The saving also remains in the supporting subset where every requested feature passed: 5 direct
runs averaged 37.56M, while 3 normal Work Leaf runs averaged between 11.78M and 30.58M. That leaves
at least 6.98M fewer raw tokens for Work Leaf in that subset.

This is strong evidence of a raw-token saving in the collected benchmark sample. It is not yet a
formal population result: a descriptive bootstrap that includes both run variation and the extreme
missing-token ceiling spans -7.62M to +27.38M tokens. The uncached-token interval also crosses zero.

Disabling all three candidate mechanisms did not remove the apparent saving or produce a consistent
token increase. The all-disabled group averaged between 8.85M and 43.38M raw tokens, and its
difference from normal Work Leaf spans -26.20M to +30.33M. Its average feature score was also lower,
2.0/3 versus 2.6/3. These data do not establish that the three mechanisms explain any specific
fraction of the saving.

The strongest remaining explanation is fewer model/tool cycles in the Work Leaf workflow. Existing
matched traces show 57.79% fewer total commands, 93.05% fewer repeated commands, and 50.63% fewer
validation commands for Work Leaf. Those measurements explain a plausible route to lower token use,
but they are associated workflow differences, not isolated causal fractions.

## Direct Answers

**Did normal Work Leaf save raw tokens here?**

Yes in the collected sample. Its worst-case bounded average is 1.07M tokens, or 2.96%, below direct
Codex. The result also survives when only 3/3-feature implementations are inspected. More runs are
needed before calling this a statistically established population average.

**Do the three tested context mechanisms explain the saving?**

No such conclusion is supported. Their combined effect interval crosses zero by a wide margin, and
the all-disabled group completed fewer features on average.

**What most likely explains the remaining difference?**

Work Leaf performed far fewer total, repeated, and validation command cycles. This is the best
supported explanation in the current traces, but the exact token fraction caused by those reduced
cycles is unknown.

**Is the exact reduction percentage known?**

No. Interrupted Work Leaf responses do not expose final provider usage. The defensible raw-token
range for this sample is 2.96% to 63.87% fewer. The uncached-token saving is not proven.

## Terms

- **Direct Codex** means a normal sequential workflow without Work Leaf. It implements, reviews,
  fixes, and integrates each feature in order.
- **Normal Work Leaf** means the normal concurrent orchestrator workflow with three feature agents,
  their reviewers, and final integration.
- **All-disabled Work Leaf** means the same concurrent workflow, except changed rereads return full
  files, unchanged rereads return full files, and reviewers reconstruct their target from Git.
- **Raw tokens** are input tokens including cached input, plus output tokens.
- **Uncached tokens** are uncached input plus output tokens.
- **Known tokens** are usage emitted by completed provider responses.
- **Conservative upper bound** adds up to 400,000 tokens for every interrupted response whose final
  usage is unavailable. It is a ceiling, not an estimate of likely usage.
- **Descriptive bootstrap** repeatedly resamples the collected runs to show how sensitive the result
  is to ordinary run-to-run variation. It is not formal proof about every future run.

## Fairness

All included groups use:

- starting commit `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
- task hash `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`;
- GPT-5.5 with `xhigh` reasoning;
- the same three feature checks for visual selection, `/status`, and reviewed-patch completion;
- normal workflow validation freedom and final repository checks;
- blocked recursive provider launches; and
- the same observer accounting rules.

The intended workflow difference remains intact: direct Codex is sequential and does not use Work
Leaf, while normal Work Leaf is concurrent. No production Work Leaf implementation, task, or scorer
was changed for this study.

Batch 4 uses a benchmark-only cleanup correction. The benchmark temporarily adds a paragraph to
`AGENTS.md` to block recursive provider calls. When a feature legitimately edits that file, cleanup
removes only the exact temporary paragraph and preserves the feature edit. This happens after model
work and does not change prompts, feature behavior, token use, or final scoring.

The groups are analyzed independently. A failure in one group never removes a run from another
group, and no run is paired with or discarded because of another run's outcome.

## Dataset

| Workflow | Runs | Reported workflow passes | Passed requested features | Mean raw tokens |
| --- | ---: | ---: | ---: | ---: |
| Direct Codex | 6 | 6 | 17/18 | 36.12M exact |
| Normal Work Leaf | 5 | 4 | 13/15 | 13.05M to 35.05M |
| All-disabled Work Leaf | 6 | 5 | 12/18 | 8.85M to 43.38M |

The normal Work Leaf non-pass completed implementation, review, and integration and scored 3/3. It
failed only when the benchmark tried to remove its temporary `AGENTS.md` paragraph. It remains in
the group with its failed workflow status and bounded token usage.

The all-disabled non-pass completed the provider workflow and scored 2/3. Its final repository test
suite found one failing terminal interaction test. It also remains in its group.

One additional normal Work Leaf attempt is not in the five-run group. Invalid environment values
caused the orchestrator to stop as soon as its first directives arrived, before implementation,
review, or integration. Its evidence is retained as an infrastructure failure, but it is not a
normal Work Leaf observation.

## Token Result

### Entire dataset

| Comparison | Conservative mean difference |
| --- | ---: |
| Direct minus normal Work Leaf | 1.07M to 23.07M more for direct |
| All-disabled minus normal Work Leaf | 26.20M fewer to 30.33M more |

The first row is positive across the collected-sample interval. The second crosses zero and cannot
identify a combined mechanism effect.

The known-token means differ by 23.07M tokens, but that is the optimistic end rather than the final
answer. The 1.07M difference is the deliberately pessimistic end after every interrupted Work Leaf
response receives its full ceiling.

### Full-feature subset

The primary result keeps every partial and failed workflow. A separate check uses only 3/3-feature
outputs to challenge the explanation that Work Leaf saved tokens by doing less work:

| Workflow | 3/3 runs | Mean raw tokens |
| --- | ---: | ---: |
| Direct Codex | 5 | 37.56M exact |
| Normal Work Leaf | 3 | 11.78M to 30.58M |

The conservative difference is 6.98M to 25.78M fewer tokens for normal Work Leaf. This supports the
main sample result but does not replace the complete dataset or prove formal quality equivalence.

### Statistical uncertainty

The point estimate and the population question are different:

- The collected-sample worst case is positive: 1.07M fewer raw tokens for Work Leaf.
- The descriptive 95% bootstrap envelope is -7.62M to +27.38M because only 6 direct and 5 normal
  observations exist and Work Leaf has wide interruption ceilings.
- The feature-score difference is 0.23 features in favor of direct, with a descriptive interval of
  -0.30 to +0.80.

Therefore the current result demonstrates the collected sample but does not provide a formal
population-confidence claim or cross-project generalization.

## Mechanism Result

All six control runs configured all three candidate mechanisms as disabled. The controls were
actually exercised:

- changed-file full rereads occurred 12 times across 4 of 6 runs;
- unchanged-file full rereads occurred 26 times across all 6 runs; and
- Git-based review reconstruction occurred in all 6 runs.

Despite this activation, the all-disabled versus normal interval crosses zero. Its known-token mean
is 4.20M lower, not higher, but its average quality is also 0.60 features lower and its conservative
upper bound is broad. The result neither proves a saving from the mechanisms nor proves that they do
nothing. It establishes that this experiment cannot assign them the observed workflow-level saving.

## Likely Source

The existing normal-workflow traces show:

| Measured workflow activity | Direct mean | Work Leaf mean | Fewer with Work Leaf |
| --- | ---: | ---: | ---: |
| All commands | 657.0 | 277.3 | 57.79% |
| Repeated commands | 148.7 | 10.3 | 93.05% |
| Validation commands | 53.3 | 26.3 | 50.63% |

This pattern is consistent with Work Leaf reducing repeated reasoning and tool work by splitting the
features across concurrent agents, preserving compact state, and reviewing exact commits. It is the
best current explanation for the residual token gap. Because those cycle counts were observed in
the complete workflows rather than changed one at a time, they do not provide an exact causal token
allocation.

## Reproduction

These commands are offline and launch no provider:

```sh
cd bench-results/efficiency-corrected-all-disabled-20260829T091341Z
python3 -m unittest test_step5_analyze.py
python3 step5-analyze.py
./test-study
```

`final-evidence.json` contains every observation, source hash, interval, feature score, bootstrap
result, and conclusion used above. `STEP4-FAILURES.md` preserves the infrastructure and workflow
failures. Raw local captures remain under `runs/` for audit and are not required to read this report.
