# Direct-Read Control Result

## Current Answer

The three direct-read Work Leaf controls are exact, but they do not establish the raw-token effect
of mediated file reads. Direct-read Work Leaf averaged 19,220,509 raw tokens. Normal Work Leaf is
bounded between 17,471,532 and 19,725,532, so direct reads range from using 505,023 fewer tokens to
using 1,748,977 more. The sign changes across the valid endpoint interval.

The 9.38% read-effect figure in `control-evidence.json` uses only the recorded normal Work Leaf
lower bound. It is a descriptive scenario, not the current causal estimate. The
uncached direction is also unresolved because missing normal responses do not report their cached
input split.

## Exact Control Data

| Group | Runs | Feature checks | Mean raw tokens | Mean uncached tokens |
| --- | ---: | ---: | ---: | ---: |
| Direct sequential Codex | 6 | 17/18 | 36,116,382 exact | 1,608,712 exact |
| Normal concurrent Work Leaf | 6 | 13/18 | 17,471,532-19,725,532 | 1,343,404-3,597,404 |
| Direct-read concurrent Work Leaf | 3 | 9/9 | 19,220,509 exact | 1,607,367 exact |

The three direct-read runs span 16.92M-21.76M raw tokens. All three complete every feature. Their
exact one-sided permutation test against the recorded normal lower-bound rows is `p=0.25` for raw
tokens and `p=0.0357` for uncached tokens, but those tests do not include the 35 unresolved normal
responses and therefore do not settle the bounded comparison.

## Validity

All three controls pass implementation, review, linearization, final formatting, Clippy, tests,
candidate build, replay, accounting, and feature scoring. Each capture contains eight
GPT-5.5/`xhigh` provider threads, no descendants, hash-matched rollouts, direct read permission, and
no `@work-leaf read` directives. No benchmark was rerun during offline reanalysis.

The controls changed only the read route: agents read through their normal read-only tools instead
of requesting file text from Work Leaf. Structured edits, mediated write-producing commands,
scheduling, review, linearization, validation freedom, final checks, and the scorer remain fixed.

## What The Control Still Shows

Direct-read Work Leaf uses 46.78% fewer raw tokens than direct sequential Codex while completing all
9 of its feature checks. This is not an isolated read-effect estimate because the two workflows
differ in the rest of the Work Leaf protocol. It does show that mediated reads are not necessary for
the large overall Work Leaf advantage.

Direct-read Work Leaf averages 202.67 provider usage changes and 94,023 input tokens per change.
Those measurements remain exact. Comparing them with normal Work Leaf's recorded lower-bound events
is useful for diagnosis but cannot produce an exact endpoint allocation.

## Conclusion

The current evidence does not prove that mediated reads independently reduce raw tokens. Their
ordered contribution is combined with directive interruption in the later bounded bridge, where it
ranges from a cost to a saving. The exact compact-direct versus sequential Work Leaf control, not
this read control, identifies the orchestration protocol as the dominant cause.

`control-evidence.json` preserves the exact control rows, activation checks, hashes, and the
lower-bound-only scenario. The current interpretation is in
`bench-results/efficiency-mechanism-attribution-20260830T081131Z/FINAL-REPORT.md`.
