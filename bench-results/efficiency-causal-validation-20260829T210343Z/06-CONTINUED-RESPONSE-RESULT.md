# Continued-Response Control Result

## Answer From This Control

Stopping a provider response after a complete Work Leaf directive saves tokens in this scenario.
It is a contributor, not the whole explanation.

The three-run control allowed generation that had resumed after a directive to reach exact usage
before sending the same interrupt. Mean raw use rose from 17.47 million tokens for six normal Work
Leaf runs to 22.52 million, an increase of 5.05 million or 28.88%. That movement is 27.07% of the
18.64-million-token difference between direct sequential Codex and normal concurrent Work Leaf in
the current detailed cohort.

An exact label-permutation check over the six normal and three control observations gives `p=0.0238`
for a raw-token increase and `p=0.0357` for an uncached-token increase. These values describe this
small collected sample. The groups were collected in separate batches, and three controls are not a
precise population estimate.

## Runs And Quality

All three workflows passed implementation, review, linearization, final formatting, Clippy, tests,
candidate build, and candidate replay. The frozen feature scorer retained every result:

| Run | Continued responses | Timeouts | Features | Raw tokens | Uncached tokens |
| --- | ---: | ---: | ---: | ---: | ---: |
| `continued-response-001` | 2 | 0 | 2/3 | 23,312,848 | 1,824,976 |
| `continued-response-002` | 2 | 1 | 3/3 | 23,276,486 | 1,651,398 |
| `continued-response-003` | 4 | 0 | 1/3 | 20,964,172 | 1,419,852 |

The control completed 6/9 feature checks, compared with 13/18 for normal Work Leaf. Higher control
tokens were not caused by completing more scored features. The one full-quality control used 9.55
million more raw tokens than the two full-quality normal Work Leaf runs, but a one-versus-two subset
is too small for a useful effect estimate.

Run 002 had one read response that did not finish within the declared 120-second bound. The observer
forwarded the original interrupt, and later cumulative provider usage made the workflow total exact.
That turn is a partial activation, not a missing measurement and not a reason to retry the run.

## What Moved

The control added an average of 47 provider usage changes and 4,492 input tokens per change. A
symmetric arithmetic split of its 4.99-million input-token increase attributes:

- 3.93 million tokens to the greater number of provider usage changes; and
- 1.06 million tokens to larger accumulated context per change.

The stage increases were 2.65 million raw tokens in implementation, 1.68 million in linearization,
and 717,000 in review. The eight completed continuations occurred in two patch-agent turns and six
reviewer turns. The later linearization increase is therefore a downstream workflow effect, not
text generated directly inside a linearizer continuation.

This supports a concrete mechanism: an immediate interrupt prevents some continuation work from
entering provider history, which can also prevent later model cycles and context replay. It does not
mean that every directive saves tokens. In the six normal runs, 252 of 287 interrupts happened only
after the current provider response had already completed; 34 interrupted resumed output and one
timed out.

## Validity

Every run used the frozen Work Leaf binaries at commit
`5b1d1ef9590850faed26052f909ddff7ff8f127d`, GPT-5.5 with `xhigh` reasoning, normal mediated reads,
normal concurrent submission, normal validation freedom, the original three requests, and the
frozen `/status` scorer. The only proxy behavior changed was the release time of an interrupt after
output resumed.

All 24 provider rollout files match their recorded hashes. Each run has eight primary provider
threads, no descendants, exact cumulative totals, preserved client-to-server interrupt bytes, and no
recursive provider attempts. The custom observer also contains a later cumulative-usage analyzer
fix; that code runs after collection and does not alter proxy forwarding.

## Why The Percentages Cannot Be Added Yet

The direct-read control moved raw use by 9.38% of the endpoint gap, while this control moved it by
27.07%. Adding those figures would assume the mechanisms are independent.

They are not independent in the observed traces. Direct-read Work Leaf resumed output after 71 of
114 directives, while normal mediated-read Work Leaf did so after 34 of 287. Changing the read route
therefore changes how often the interruption mechanism can activate. A combined direct-read plus
continued-response control is required to measure that interaction.

Even with only continued responses changed, Work Leaf remained 37.65% below direct Codex in raw
tokens. That comparison has unequal feature totals and is not a new fair endpoint result; it only
shows that directive interruption cannot be the sole source of the original gap.

`continued-response-evidence.json` contains the individual rows, exact arithmetic, activation
records, hashes, stage totals, and permutation checks. `analyze-continued-response.py` rebuilds it
without launching a provider.
