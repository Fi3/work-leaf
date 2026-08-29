# Direct-Read Control Result

## Answer From This Control

Orchestrator-mediated reads save tokens, but they do not explain most of the raw-token difference.

The read route has a clear effect on fresh, uncached context. Allowing the same concurrent Work Leaf
agents to read files directly raised uncached use by about 264,000 tokens per workflow. That increase
is almost the entire uncached difference between normal Work Leaf and direct sequential Codex in the
current detailed cohort.

The effect on raw tokens is smaller and less precise. Direct reads raised the three-run control mean
by 1.75 million raw tokens relative to the six normal Work Leaf runs. That is 9.38% of the current
direct-versus-normal raw gap. When only candidates that pass all three feature checks are compared,
the increase is 5.49 million tokens, or 23.04% of the corresponding full-quality raw gap. The sample
is too small to treat either percentage as a population estimate.

Most of the raw saving remains after direct reads are enabled. Direct-read Work Leaf still uses
46.78% fewer raw tokens than direct sequential Codex.

## Validity Gates

All three controls satisfy the declared gates:

- implementation, review, linearization, final formatting, Clippy, tests, candidate build, and
  candidate replay passed;
- the frozen external scorer passed visual selection, `/status`, and completion close/reopen for
  every candidate, giving 9/9 feature checks;
- each capture contains exactly eight GPT-5.5/`xhigh` provider threads and no descendant provider
  sessions;
- the original observer total and later cumulative reanalysis agree exactly;
- all 24 source rollout files match their recorded SHA-256 hashes;
- seven project-agent launch prompts per workflow enable direct reads, none contain the mediated-read
  restriction, and no agent emits `@work-leaf read`;
- direct read commands appear 308, 343, and 320 times; and
- recursive provider-attempt logs are empty.

The newer offline analyzer initially found zero threads because the published observer config still
named the removed staging directory. A derived config changes only the observation root to the final
published directory. The zero-thread result is not admitted. The corrected reanalysis has eight
threads per run and reproduces the original usage totals exactly; no benchmark was rerun.

## Group Results

The groups are independent. Concurrent collection is not statistical pairing.

| Group | Runs | Feature checks | Raw-token mean | Uncached-token mean | Usage changes | Input per change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Direct sequential Codex | 6 | 17/18 | 36,116,382 | 1,608,712 | 320.17 | 112,148 |
| Normal concurrent Work Leaf | 6 | 13/18 | 17,471,532 | 1,343,404 | 212.33 | 81,359 |
| Direct-read concurrent Work Leaf | 3 | 9/9 | 19,220,509 | 1,607,367 | 202.67 | 94,023 |

The raw ranges overlap: normal Work Leaf spans 13.21M to 21.80M, while the control spans 16.92M to
21.76M. The uncached ranges overlap less: 1.18M to 1.50M for normal Work Leaf and 1.45M to 1.75M for
the control.

An exact one-sided permutation check over all 84 possible three-versus-six assignments gives
`p=0.25` for the raw increase and `p=0.0357` for the uncached increase. These checks describe this
small collected sample; they do not turn three controls into a precise population result.

## Quality Check

The control did more measured feature work, not less. All three controls pass 3/3 features. Only two
of the six current normal Work Leaf candidates pass 3/3.

Restricting all groups to 3/3 candidates gives:

| Group | Runs | Raw-token mean | Uncached-token mean |
| --- | ---: | ---: | ---: |
| Direct sequential Codex | 5 | 37,564,061 | 1,549,060 |
| Normal concurrent Work Leaf | 2 | 13,728,957 | 1,183,741 |
| Direct-read concurrent Work Leaf | 3 | 19,220,509 | 1,607,367 |

The read-route increase becomes 5.49M raw and 424,000 uncached tokens in this subset. The minimum
possible one-sided permutation value is `p=0.1` because only ten assignments exist. This subset
rejects the explanation that direct-read tokens rose merely because those candidates implemented
less, but its size is too small for an exact allocation.

## What Changed Inside Token Use

Direct reads did not create more provider usage changes. They reduced the mean from 212.33 to
202.67, a movement in the opposite direction. They increased mean input context per change from
81,359 to 94,023 tokens.

A symmetric arithmetic split of the 1.78M input-token increase gives:

- 848,000 fewer input tokens from the lower usage-change count; and
- 2.63M more input tokens from larger context per change.

Those values sum to the observed input increase. This supports one narrow cause: mediated reads
keep each model cycle's context smaller. It does not support mediated reads as the cause of Work
Leaf's lower cycle count.

## Remaining Raw Difference

After direct reads replace mediated reads, Work Leaf still averages:

- 16.90M fewer raw tokens than direct sequential Codex;
- 117.5 fewer provider usage changes; and
- 18,125 fewer input tokens per usage change.

Its uncached mean is only 1,345 tokens below direct Codex, a difference of 0.084%. Therefore the
read route explains nearly all of the current uncached advantage but only a minority of the raw
advantage. The remaining raw difference is cached context replay across model cycles.

The next control targets immediate interruption after a complete orchestrator directive. Every
normal Work Leaf run uses that behavior, and it can prevent continued generation and additional
tool cycles. Command-output compaction has zero observed avoided bytes in the six normal runs, and
review plus linearization contain too little of the total gap to explain the majority alone.

`control-evidence.json` contains the exact rows, activation checks, hashes, permutation arithmetic,
and full-quality subset. `analyze-control.py` rebuilds it without launching a provider.
