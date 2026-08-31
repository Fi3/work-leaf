# Continued-Response Control Result

## Current Answer

The three continued-response controls are exact. Continued-response Work Leaf averaged 22,517,835
raw tokens. Relative to normal Work Leaf's 17,471,532-19,725,532 interval, allowing resumed output
to finish uses 2,792,303-5,046,303 more raw tokens. Early directive interruption therefore saves raw
tokens in these collected samples even after every unresolved normal response receives its maximum
allowance.

The bounded interruption contribution is 17.04%-27.07% of the endpoint gap. The upper percentage
uses only the recorded normal Work Leaf lower bound; the lower percentage uses the maximum allowance.
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

Those recorded event differences show that interruption changes later workflow behavior, not just
the final few output tokens. Direct reads also change how often output resumes, so read delivery and
interruption cannot be added as independent effects.

## Conclusion

Directive interruption is a proven raw-token saving in this bounded three-run control, with a
2.79M-5.05M effect relative to the collected normal endpoint. This does not make it the main cause:
even the exact continued-response group uses 37.65% fewer raw tokens than direct Codex, and that
comparison still includes the rest of the Work Leaf protocol.

`continued-response-evidence.json` preserves the exact runs, activation records, hashes, and the
lower-bound-only scenario. The combined control and final bounded interpretation are in
`08-COMBINED-CONTROL-RESULT.md` and
`bench-results/efficiency-mechanism-attribution-20260830T081131Z/FINAL-REPORT.md`.
