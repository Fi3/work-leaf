# Final Report: Work Leaf Token Savings

## Abstract

This study asks whether normal concurrent Work Leaf uses fewer GPT-5.5/`xhigh` tokens than a fair
normal direct sequential Codex workflow for the same three Rust features, and what causes any
difference.

The saving is real in the collected benchmark. Across the current six-versus-six cohort, direct
Codex averages 36.12 million raw tokens and Work Leaf averages 17.47 million, a 51.62% reduction.
Every Work Leaf run is below every direct run. An older equal-quality three-versus-three cohort also
shows a 60.25% reduction. This supports the result for this repository and workflow; it is not yet a
cross-project theorem or a precise population estimate.

The immediate token source is less repeated input. Cached input accounts for 98.58% of the current
raw gap. Work Leaf reaches that result through fewer model/tool generations and smaller accumulated
context per generation.

Controlled tests show that mediated file reads and immediate directive interruption are real but
overlapping contributors. Disabling both together moves only 1.93 million raw tokens, 10.34% of the
endpoint gap. The remaining Work Leaf workflow still uses 16.72 million fewer raw tokens than direct
Codex. Saved provider histories tie that residual mainly to much less iterative implementation work
and a smaller linearization context: fewer shell calls, write submissions, repeated commands, and
validation commands.

## What Was Compared

The endpoint comparison uses two normal workflows:

- **Direct sequential Codex:** a normal direct coding agent implements each request, a separate
  direct agent reviews it until clean, and a final direct agent linearizes the three reviewed
  requests.
- **Concurrent Work Leaf:** three normal Work Leaf patch agents implement concurrently, Work Leaf
  runs normal review and fix routing, and a final Work Leaf linearizer creates the reviewed history.

Both receive the same three requests, base commit, GPT-5.5 model, `xhigh` reasoning, focused-check
guidance, review responsibility, linearization requirement, timeout, final formatting, Clippy,
tests, candidate build, replay, and frozen three-feature scorer. `/status` tests the requested slash
command. `/fork` is not part of the task and is not scored.

"Raw tokens" means input plus output, counting cached input at full token volume. "Uncached tokens"
means uncached input plus output. Groups are independent; concurrent launch is only a speedup, not a
pairing rule. Every success and partial feature result is retained.

## Is The Saving Real?

### Current detailed cohort

| Workflow | Runs | Feature checks | Mean raw tokens | Raw range | Mean uncached tokens |
| --- | ---: | ---: | ---: | ---: | ---: |
| Direct sequential Codex | 6 | 17/18 | 36,116,382 | 28,877,983-43,257,690 | 1,608,712 |
| Normal concurrent Work Leaf | 6 | 13/18 | 17,471,532 | 13,214,206-21,800,967 | 1,343,404 |

Normal Work Leaf uses 51.62% fewer raw tokens and 16.49% fewer uncached tokens. The raw ranges do
not overlap. A descriptive exact label permutation gives `p=1/924`; batches were not prospectively
randomized, so the rank separation matters more than treating that value as a formal trial result.

### Equal-quality sanity check

An older admitted cohort has three direct and three Work Leaf runs with 8/9 feature checks in each
group. Direct averages 35,196,786 raw tokens and Work Leaf 13,989,718, a 60.25% reduction. Uncached
use falls from 1,739,208 to 1,085,568, a 37.58% reduction.

This counters the strongest quality objection. The exact percentage still varies between cohorts,
so the evidence supports a large saving, not a single universal percentage.

## Where The Tokens Go

The current 18.64-million-token raw gap consists of:

| Token class | Direct minus Work Leaf | Share of raw gap |
| --- | ---: | ---: |
| Cached input | 18,379,541 | 98.58% |
| Uncached input | 251,167 | 1.35% |
| Output | 14,142 | 0.08% |
| Reasoning output | -8,206 | reported separately; Work Leaf is higher |

Work Leaf is not saving tokens by suppressing reasoning or returning much shorter answers. It is
replaying much less accumulated input.

Direct Codex averages 320.17 distinct provider usage changes and 112,148 input tokens per change.
Normal Work Leaf averages 212.33 changes and 81,359 input tokens per change. Fewer generations and
smaller context therefore multiply together.

## Controlled Causes

Four Work Leaf conditions isolate file reads, response interruption, and their overlap:

| Condition | Mean raw tokens | Change from normal | Share of endpoint raw gap |
| --- | ---: | ---: | ---: |
| Normal Work Leaf | 17,471,532 | n/a | n/a |
| Direct file reads only | 19,220,509 | +1,748,977 | 9.38% |
| Continued responses only | 22,517,835 | +5,046,303 | 27.07% |
| Both changes together | 19,399,622 | +1,928,090 | 10.34% |

The first two percentages must not be added. Their combined result is 4.87 million raw tokens below
that additive prediction. Direct reads make resumed post-directive output much more common; allowing
that output to finish then removes some later reads and model generations. These mechanisms overlap
and substitute for some of the same work.

Together they more than remove the endpoint's small uncached advantage: combined Work Leaf uses
719,770 more uncached tokens than normal Work Leaf and 454,462 more than direct Codex. Yet it still
uses 46.29% fewer raw tokens than direct Codex. The remaining advantage is therefore repeated-context
efficiency elsewhere in Work Leaf.

## Main Cause Of The Remaining Saving

Comparing direct Codex with the combined Work Leaf control leaves 16.72 million raw tokens. Combined
Work Leaf receives 448,000 more fresh input and emits 6,935 more output tokens, but replays 17.17
million fewer cached input tokens.

The residual input gap breaks down as follows:

- **76.62% from fewer provider generations:** 320.17 per direct workflow versus 197.67.
- **23.38% from smaller context per generation:** 112,148 input tokens versus 97,044.

Most of the gap occurs during implementation and review fixes, 13.46 million raw tokens. Final
linearization contributes another 3.42 million. Review itself contributes no saving in this control;
combined Work Leaf spends about 95,000 more raw tokens there and completes more review rounds.

Hash-verified provider actions show the practical workflow difference:

| Operation per workflow | Direct Codex | Combined Work Leaf | Reduction |
| --- | ---: | ---: | ---: |
| Shell-tool calls | 634.17 | 429.00 | 32.35% |
| Separate write submissions | 63.67 | 17.67 | 72.25% |
| Repeated commands | 140.67 | 47.33 | 66.35% |
| Validation commands | 57.83 | 13.67 | 76.37% |

The best-supported explanation is workflow batching. Work Leaf makes patch agents submit cohesive
structured edits, mediates write-producing commands, scopes validation to concurrent ownership,
routes exact reviewed patches, and gives the final linearizer a compact reviewed target. The agents
reach comparable output with fewer autonomous tool/model cycles, and each later cycle carries less
history.

The study proves those remaining Work Leaf mechanisms collectively retain 89.66% of the sample's raw
endpoint gap after reads and interruption are disabled. It does not assign exact individual
percentages among structured edits, command mediation, focused validation, review targeting, and
linearization compaction because no existing production switch separates them.

## Checks Against False Results

- **Work Leaf undercounting:** all admitted provider threads have exact cumulative totals; app-server
  incremental components reconcile, rollout hashes match, and no descendant provider sessions exist.
- **Direct overcounting:** each resume reports a per-invocation total; their sums independently match
  the final per-thread Codex rollout totals.
- **CLI drift:** the three direct Codex 0.150.1 runs average 37.04M raw tokens at 9/9 quality; combined
  Work Leaf 0.150.1 averages 19.40M at 8/9.
- **Less code:** candidate changed-line ranges overlap, combined Work Leaf reviews more rounds, and
  full-feature combined runs remain below every direct run.
- **Hidden work:** Work Leaf's title, patch, reviewer, and linearizer threads are included. Direct
  implementation, resume, review, and linearization invocations are included.
- **Command-output compaction:** measured normal-run counterfactuals save zero command-output bytes;
  it is not the main cause.

The full challenge table is in `09-FINAL-HYPOTHESIS-AUDIT.md`.

## Conclusion

For this three-feature Rust workflow, normal concurrent Work Leaf has a real and large raw-token
advantage over fair normal direct sequential Codex. The observed reduction is about 52% in the
current detailed cohort and 60% in an older equal-quality cohort.

About 10% of the current raw gap is the net combined effect of mediated reads and immediate
directive interruption. Most of the advantage remains when both are disabled. It comes from Work
Leaf's broader orchestration behavior producing far fewer iterative provider/tool cycles and less
context replay, especially during implementation and final linearization.

The exact percentage should not be generalized beyond this repository without more runs and other
projects. The existence, direction, token class, workflow stage, and dominant operational mechanism
are supported by the saved evidence.

## Reproduce The Analysis

No provider call is needed:

```sh
python3 -m unittest discover \
  -s bench-results/efficiency-causal-validation-20260829T210343Z \
  -p 'test_*.py'
python3 bench-results/efficiency-causal-validation-20260829T210343Z/endpoint_audit.py
python3 bench-results/efficiency-causal-validation-20260829T210343Z/decompose.py
python3 bench-results/efficiency-causal-validation-20260829T210343Z/analyze-control.py
python3 bench-results/efficiency-causal-validation-20260829T210343Z/analyze-continued-response.py
python3 bench-results/efficiency-causal-validation-20260829T210343Z/analyze-combined.py
```

`combined-evidence.json` contains the complete run rows, hashes, activation records, quality checks,
four-condition arithmetic, operation counts, review rounds, and accounting counterchecks.
