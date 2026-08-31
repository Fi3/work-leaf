# Combined Control Result

## Current Answer

Three exact Work Leaf controls used direct file reads and allowed resumed output to finish. They
averaged 19,399,622 raw tokens and completed 8 of 9 feature checks. The controls are valid, but their
movement relative to normal Work Leaf is bounded because the normal endpoint is
17,471,532-19,725,532 raw tokens.

The combined transition therefore ranges from using 325,910 fewer tokens to using 1,928,090 more.
Its ordered share of the endpoint gap ranges from -1.99% to 10.34%. The sign changes, so this study
does not prove that mediated reads plus interruption independently save raw tokens.

## Conditions

| Condition | File access | Behavior after a complete directive |
| --- | --- | --- |
| Normal Work Leaf | Work Leaf returns requested file context | Work Leaf requests interruption |
| Direct-read Work Leaf | Agents read with normal read-only tools | Work Leaf requests interruption |
| Continued-response Work Leaf | Work Leaf returns file context | Resumed output finishes first |
| Combined Work Leaf | Agents read with normal read-only tools | Resumed output finishes first |

The combined condition is diagnostic, not a proposed product mode. The original requests,
concurrent scheduling, frozen binaries, structured writes, command mediation, review,
linearization, validation freedom, final checks, GPT-5.5/`xhigh`, and scorer remain fixed.

## Exact Results

| Group | Runs | Feature checks | Mean raw tokens | Mean uncached tokens |
| --- | ---: | ---: | ---: | ---: |
| Direct sequential Codex | 6 | 17/18 | 36,116,382 exact | 1,608,712 exact |
| Normal concurrent Work Leaf | 6 | 13/18 | 17,471,532-19,725,532 | 1,343,404-3,597,404 |
| Direct-read Work Leaf | 3 | 9/9 | 19,220,509 exact | 1,607,367 exact |
| Continued-response Work Leaf | 3 | 6/9 | 22,517,835 exact | 1,632,075 exact |
| Combined Work Leaf | 3 | 8/9 | 19,399,622 exact | 2,063,174 exact |

The combined runs are:

| Run | Features | Raw tokens | Uncached tokens | Completed continuations |
| --- | ---: | ---: | ---: | ---: |
| `combined-control-001` | 3/3 | 17,187,752 | 2,106,408 | 21 |
| `combined-control-002` | 3/3 | 23,861,127 | 2,354,567 | 24 |
| `combined-control-003` | 2/3 | 17,149,987 | 1,728,547 | 11 |

Every combined run passed the workflow, final repository checks, candidate build, replay,
accounting, and activation gates. Their 24 primary provider threads have no descendants, and every
rollout matches its recorded SHA-256.

## Interaction

Using the recorded normal Work Leaf lower bound gives the former interaction result: direct reads
add 1.75M raw tokens, completed responses add 5.05M, their combination adds 1.93M, and the
interaction is -4.87M. Under the conservative normal upper bound, the interaction is -2.61M. The
valid interaction interval is therefore -4.87M to -2.61M. Its sign remains negative, proving that
the read and continuation effects are not additive in these samples.

The lower-bound traces still show why a simple addition was inappropriate: direct reads made output
resume after directives more often, and completed continuations changed later turns. This is useful
mechanism evidence, but the missing normal usage prevents an exact percentage.

## Exact Residual Comparison

Direct Codex used 16.72M more raw tokens than combined Work Leaf, a 46.29% reduction. This exact
comparison is not an isolated read/interruption effect because combined Work Leaf retains the rest
of the orchestration protocol. It shows that disabling both candidate mechanisms does not remove
the large overall difference.

The exact token classes are:

- combined Work Leaf uses 17.17M fewer cached input tokens;
- it uses 448,000 more uncached input tokens; and
- it emits 6,935 more output tokens.

Combined Work Leaf averages 197.67 provider usage changes versus direct Codex's 320.17. Its mean
input per change is 97,044 versus 112,148. These are descriptive outcomes that motivated the later
exact orchestration control; they are not themselves the causal intervention.

The provider histories also record 17.67 versus 63.67 write submissions, 429 versus 634 shell-tool
calls, 47 versus 141 repeated commands, and 14 versus 58 validation commands per workflow. The
combined runs perform 10.33 review rounds versus 6.50 for direct Codex, so less review does not
explain the difference.

## Conclusion

The combined control proves that a large Work Leaf advantage remains when direct reads and completed
responses are both enabled. It does not prove the direction of their joint contribution relative to
normal Work Leaf. The later exact compact-direct versus sequential Work Leaf control identifies the
orchestration protocol as the dominant cause without relying on this bounded transition.

`combined-evidence.json` preserves the exact control data and the recorded-lower-bound scenario.
The current allocation is in
`bench-results/efficiency-mechanism-attribution-20260830T081131Z/FINAL-REPORT.md`.
