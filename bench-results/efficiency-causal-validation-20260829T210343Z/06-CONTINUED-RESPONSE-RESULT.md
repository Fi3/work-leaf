# Continued-Response Control Result

## Current Answer

The three continued-response controls are exact, but the bounded normal endpoint does not establish
the direction of directive interruption by itself. Continued-response Work Leaf averaged 22,517,835
raw tokens. Relative to normal Work Leaf's 17,471,532-23,304,865 interval, allowing resumed output
to finish ranges from using 787,030 fewer tokens to using 5,046,303 more. The sign changes across
the valid endpoint interval.

The 27.07% interruption figure in `continued-response-evidence.json` uses only the recorded normal
Work Leaf lower bound. It is a descriptive scenario, not a current causal estimate.
The control also completed 6 of 9 feature checks versus 13 of 18 for normal Work Leaf, so its quality
does not provide a stronger matched comparison.

## Exact Control Runs

| Run | Completed continuations | Timeouts | Features | Raw tokens | Uncached tokens |
| --- | ---: | ---: | ---: | ---: | ---: |
| `continued-response-001` | 2 | 0 | 2/3 | 23,312,848 | 1,824,976 |
| `continued-response-002` | 2 | 1 | 3/3 | 23,276,486 | 1,651,398 |
| `continued-response-003` | 4 | 0 | 1/3 | 20,964,172 | 1,419,852 |

Every run passed implementation, review, linearization, final formatting, Clippy, tests, candidate
build, replay, and exact provider accounting. All 24 rollout files match their recorded hashes.
Each run has eight primary GPT-5.5/`xhigh` threads, no descendants, preserved interrupt bytes, and
no recursive provider attempts.

Run 002 had one read response that did not finish within 120 seconds. The observer forwarded the
original interrupt, and later cumulative usage made the workflow total exact. This is a partial
activation, not a missing measurement.

## Activation Evidence

The control allowed eight resumed responses to finish: two patch-agent responses and six reviewer
responses. It added an average of 47 recorded provider usage changes relative to the normal
lower-bound trace and moved tokens in implementation, review, and later linearization. This shows
that changing interruption timing can alter downstream workflow behavior, not merely append a few
output tokens to one response.

Those recorded event differences explain the lower-bound scenario but do not overcome the 35
missing normal responses. Direct reads also change how often output resumes, so read delivery and
interruption cannot be treated as independent effects.

## Conclusion

Directive interruption is an active mechanism, but its independent raw-token direction and share
are not proven by this cohort. Even the exact continued-response group uses 37.65% fewer raw tokens
than direct Codex; that comparison still includes the rest of the Work Leaf protocol and therefore
does not assign the saving to interruption.

`continued-response-evidence.json` preserves the exact runs, activation records, hashes, and the
lower-bound-only scenario. The combined control and final bounded interpretation are in
`08-COMBINED-CONTROL-RESULT.md` and
`bench-results/efficiency-mechanism-attribution-20260830T081131Z/FINAL-REPORT.md`.
