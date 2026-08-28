# Points 8 And 9 Efficiency Study

## Goal

This study asks whether normal concurrent Work Leaf repeatedly uses fewer raw tokens than fair
normal direct sequential Codex on the frozen three-feature task, and whether three previously
identified Work Leaf delivery mechanisms account for that difference.

The mechanisms are:

1. changed-file rereads return a diff instead of the full current file;
2. unchanged-file rereads return a digest instead of the full file; and
3. reviewers receive exact source context instead of reconstructing it from Git.

The study retains every workflow and feature result. Conditions are independent groups. Two runs
sharing a launch batch are not treated as a pair, and one run is never discarded because another
run fails.

## Starting Evidence

Point 7 supplies one fair 3/3 observation for each endpoint:

- direct sequential Codex: 41,035,124 exact raw tokens;
- normal Work Leaf: at most 33,177,778 raw tokens under the conservative interruption bound; and
- Work Leaf with all three mechanisms disabled: at most 39,092,989 under the same bound.

Those are single observations. They prove only the selected-run bounds and do not estimate normal
variation or a population effect.

## Conditions

Work Leaf conditions are named `wl-XYZ`. A `0` uses normal Work Leaf behavior and a `1` uses the
less compact control.

| Digit | `0` | `1` |
| --- | --- | --- |
| X: changed reread | diff | full current file |
| Y: unchanged reread | digest | full resend |
| Z: review context | exact context inline | reconstruct from Git |

`wl-000` is normal Work Leaf. `wl-111` disables all three mechanisms. `direct` is normal direct
sequential Codex without Work Leaf.

## Collection

`SCHEDULE.tsv` contains twelve predeclared independent attempts. Seven completed. One
Git-reconstruction attempt is a retained reliability failure, and the four remaining
Git-reconstruction attempts were withheld after that failure became systematic. The completed
conditions include two direct attempts, two normal Work Leaf attempts, and a four-condition screen
of changed-file and unchanged-file delivery. Point 7 supplies one additional observation for each
endpoint group.

At most two top-level workflows ran concurrently. Each attempt used a separate checkout, temporary
root, observer identity, result directory, and run ID. Batch neighbors remain independent
observations rather than analytical pairs.

## Fairness

Every condition uses:

- candidate base `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
- the original generic selected-agent slash-command task, without a `/fork` requirement;
- GPT-5.5 with `xhigh` reasoning for every provider thread;
- normal validation behavior and identical final repository checks;
- no recursive provider-verification sessions; and
- the frozen visual, `/status`, and completion scorer, where `/status` is the concrete test of the
  generic slash-command requirement.

Direct Codex uses the repository's normal sequential benchmark without Work Leaf. Work Leaf uses
the normal concurrent benchmark. The only changes in `wl-001` through `wl-111` are the declared
delivery controls. The production Work Leaf checkout is not modified; the controls exist only in
the isolated study build.

## Accounting

Direct Codex usage is exact when every launch and resume invocation reconciles. Work Leaf responses
interrupted after complete orchestrator directives lack terminal provider usage on the current
transport. Each Work Leaf result therefore reports:

- observed usage from completed responses;
- the number of interrupted responses; and
- a conservative raw-token upper bound. Missing usage is charged at least 400,000 tokens per
  interrupted response. If that does not also cover the effective context window, maximum output,
  and captured new-turn prompt bytes, the larger aggregate amount is used.

The study must not report an exact Work Leaf raw percentage or any uncached-token reduction. A
single factorial contrast is a causal screen, not a population estimate. Repeated endpoint groups
are used to judge whether apparent residual differences exceed ordinary run variation.

## Result

The direct and normal Work Leaf endpoint groups each contain three observations and average 2.67
completed features. Direct averages 35.20 million exact raw tokens. Normal Work Leaf averages 13.99
million observed raw tokens, but its conservative upper bound averages 38.52 million. The implied
difference ranges from Work Leaf using 21.21 million fewer tokens to 3.33 million more, so the
repeated saving is not proven under conservative accounting.

The changed-file and unchanged-file whole-workflow controls are inconclusive because both effect
ranges cross zero. Review context is not estimable because its Git-reconstruction control
repeatedly broke review routing. Normal Work Leaf nevertheless averages substantially fewer total,
repeated, and validation commands than direct Codex. That cycle difference is the strongest
remaining explanation for the observed token signal, but it is not an isolated causal estimate.

`FINAL-RESULT.md` explains the result in plain language. `evidence.json` contains the verified
machine-readable evidence.

## Stop Rule

No admitted provider attempt is retried because of quality, token use, model behavior, test failure,
or workflow failure. A pre-provider infrastructure failure may receive one separately named retry.
Collection stops for analysis if the same infrastructure problem makes two consecutive attempts
unusable or if the frozen model, task, base, controls, or accounting scope cannot be preserved.
