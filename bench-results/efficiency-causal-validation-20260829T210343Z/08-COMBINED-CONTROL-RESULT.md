# Combined Control Result

## Abstract

This control measures the overlap between two Work Leaf behaviors that independently changed token
use: orchestrator-mediated file reads and immediate interruption after a complete directive. Three
new GPT-5.5/`xhigh` Work Leaf workflows used direct file reads and allowed resumed output to finish.
All three passed the workflow, final repository checks, candidate build, replay, accounting, and
activation gates. They completed 8 of 9 requested feature checks.

The combined condition averaged 19.40 million raw tokens, 1.93 million above normal Work Leaf. That
movement is 10.34% of the current 18.64-million-token direct-Codex versus normal-Work-Leaf gap. The
two isolated effects cannot be added: their raw-token interaction is negative 4.87 million tokens.
Together they explain the uncached difference, but only a minority of the raw difference.

## Conditions In Plain English

| Name | File access | Behavior after a complete directive |
| --- | --- | --- |
| Normal Work Leaf | Work Leaf returns requested file context | Work Leaf interrupts unnecessary resumed output |
| Direct-read Work Leaf | Agents read files with normal read-only tools | Work Leaf interrupts unnecessary resumed output |
| Continued-response Work Leaf | Work Leaf returns requested file context | Resumed output finishes before the original interrupt |
| Combined Work Leaf | Agents read files with normal read-only tools | Resumed output finishes before the original interrupt |

The combined condition is a diagnostic control, not a proposed product mode. The original requests,
concurrent Work Leaf scheduling, frozen binaries, structured writes, command locks, review,
linearization, final checks, GPT-5.5 model, `xhigh` reasoning, and `/status` scorer remain fixed.

## Results

| Group | Runs | Feature checks | Mean raw tokens | Mean uncached tokens |
| --- | ---: | ---: | ---: | ---: |
| Direct sequential Codex | 6 | 17/18 | 36,116,382 | 1,608,712 |
| Normal concurrent Work Leaf | 6 | 13/18 | 17,471,532 | 1,343,404 |
| Direct-read Work Leaf | 3 | 9/9 | 19,220,509 | 1,607,367 |
| Continued-response Work Leaf | 3 | 6/9 | 22,517,835 | 1,632,075 |
| Combined Work Leaf | 3 | 8/9 | 19,399,622 | 2,063,174 |

The combined runs were:

| Run | Features | Raw tokens | Uncached tokens | Completed continuations | Timeouts |
| --- | ---: | ---: | ---: | ---: | ---: |
| `combined-control-001` | 3/3 | 17,187,752 | 2,106,408 | 21 | 0 |
| `combined-control-002` | 3/3 | 23,861,127 | 2,354,567 | 24 | 0 |
| `combined-control-003` | 2/3 | 17,149,987 | 1,728,547 | 11 | 0 |

Every combined run used fewer raw tokens than every direct run. The combined group also completed
more review rounds on average than direct Codex, 10.33 versus 6.50, so fewer review rounds cannot
explain its lower token total.

## Interaction

Changing only file reads raised raw use by 1.75 million tokens. Changing only interruption raised
raw use by 5.05 million. Adding those values would predict a 6.80-million increase, but changing both
together raised raw use by only 1.93 million.

The difference, negative 4.87 million tokens, is the interaction. Direct reads cause provider output
to resume after directives much more often. When that output is allowed to finish, some later reads
and model turns no longer occur. The two controls therefore replace some of the same future work.

For raw tokens, the combined movement is 10.34% of the endpoint gap. For uncached tokens, it is
271.30% of the endpoint gap: the control more than removes normal Work Leaf's small uncached
advantage. This means the large remaining raw advantage is entirely about avoiding repeated cached
context, not hiding fresh input or producing less output.

## Remaining Difference

Direct Codex still used 16.72 million more raw tokens than combined Work Leaf, a 46.29% reduction
relative to direct Codex. That remaining difference consists of:

- 17.17 million fewer cached input tokens in combined Work Leaf;
- 448,000 more uncached input tokens in combined Work Leaf; and
- 6,935 more output tokens in combined Work Leaf.

Combined Work Leaf therefore received more fresh input and emitted slightly more output while using
far less replayed input.

Direct Codex averaged 320.17 provider usage changes; combined Work Leaf averaged 197.67, a 38.26%
reduction. Mean input carried by each change fell from 112,148 to 97,044 tokens, a 13.47% reduction.
Splitting their multiplication symmetrically attributes 76.62% of the remaining input gap to fewer
provider generations and 23.38% to smaller context per generation. This is arithmetic attribution,
not a separate randomized control.

The remaining raw gap occurs in implementation and final linearization:

| Stage | Direct minus combined raw tokens |
| --- | ---: |
| Implementation and review fixes | 13,455,640 |
| Linearization | 3,416,550 |
| Review | -94,797 |
| Hidden title thread | -60,632 |

Review itself used slightly more tokens in combined Work Leaf, again ruling out less review as the
source.

## Operation Evidence

The hash-verified provider histories show how Work Leaf reaches fewer generations:

| Operation per workflow | Direct Codex | Combined Work Leaf |
| --- | ---: | ---: |
| Shell-tool calls | 634.17 | 429.00 |
| Separate write submissions | 63.67 | 17.67 |
| Repeated commands | 140.67 | 47.33 |
| Validation commands | 57.83 | 13.67 |

Combined Work Leaf's write figure includes every observed structured edit submission, including
duplicates and rejected submissions, plus linearizer `apply_patch` calls. It does not use the UI's
de-duplicated transcript count.

These records strongly support a workflow-batching explanation: structured patches, mediated
write-producing commands, focused concurrent-agent validation, exact review scopes, and compact
linearization inputs lead to fewer autonomous tool/model cycles and less context replay. They do not
assign an exact fraction to each member of that remaining group.

## Validity

All three runs used frozen Work Leaf binaries from
`5b1d1ef9590850faed26052f909ddff7ff8f127d`, Codex CLI 0.150.1, GPT-5.5/`xhigh`, the original three
requests, and the frozen scorer. The 24 primary provider threads have no descendants. Every rollout
matches its saved SHA-256, every total reconciles, and the original interrupt bytes are preserved.

One app-server notification repeated a stale convenience value in `last.totalTokens` while all
authoritative input, cached-input, output, and reasoning components remained unchanged. The next
event advanced normally. The anomaly is retained in `combined-evidence.json`; it does not affect any
reported token total.

`analyze-combined.py` rebuilds the full evidence without launching a provider.
