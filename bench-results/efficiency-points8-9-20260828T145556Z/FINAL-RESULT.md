# Points 8 And 9 Final Result

## Abstract

This study compared normal direct sequential Codex with normal concurrent Work Leaf on the same
three-feature Rust task, then tested three proposed causes of Work Leaf's apparent token reduction.
Every workflow used the same candidate base, original task, GPT-5.5/xhigh profile, final checks, and
frozen feature scorer. Partial implementations and failures remain in the result.

The completed-response data show a large Work Leaf advantage: the direct group averaged 35.20
million raw tokens, while the observed part of the normal Work Leaf group averaged 13.99 million,
or 60.25% less. That percentage is not an exact saving. The current transport omits usage for
interrupted Work Leaf responses. Charging every missing response the study's conservative maximum
raises the Work Leaf mean to 38.52 million. The defensible difference therefore ranges from Work
Leaf using 21.21 million fewer tokens to 3.33 million more. The repeated group result is
inconclusive under conservative accounting.

The two groups completed the same average number of requested features: 2.67 of 3. Direct missed
completion close/reopen once; Work Leaf missed visual copying once. This rules out a simple claim
that one group consistently did less work, but it is not formal quality equivalence.

The changed-file diff and unchanged-file digest controls activated, but their whole-workflow token
ranges overlap zero. Exact inline review context could not be tested against Git reconstruction
because the Git control repeatedly broke review routing. The three proposed delivery mechanisms
therefore do not receive a causal percentage.

The strongest remaining explanation is the observed workflow shape. Normal Work Leaf averaged 58%
fewer commands, 93% fewer repeated commands, and 51% fewer validation commands than direct Codex.
Those differences can plausibly reduce context growth and token use, but this study did not isolate
them as a controlled ablation. They are strong supporting evidence, not an exact causal allocation.

## Plain-English Answer

- There is a strong signal that Work Leaf uses fewer raw tokens in this task.
- The current telemetry cannot prove the average saving because cancelled Work Leaf responses have
  no exact usage record.
- The observed 60.25% reduction is descriptive, not a valid exact percentage.
- Changed-file diffs and unchanged-file digests are too small or too noisy to explain a measured
  fraction of the whole-workflow gap with the available runs.
- Review-context savings are not measurable from this study because that control changed workflow
  behavior and repeatedly failed.
- Fewer command, repetition, and validation cycles are the most likely source of a real saving, but
  the exact fraction remains unknown.

## Fair Comparison

The launch contract is implemented by `run-condition` in this directory:

- direct calls the frozen `bench-three-features-sequential` launcher without Work Leaf;
- Work Leaf calls the frozen concurrent `bench-three-features` launcher;
- both start from `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
- both receive the original generic selected-agent slash-command task, with no `/fork`
  requirement;
- every provider thread uses GPT-5.5 with `xhigh` reasoning;
- both use their normal validation behavior and the same final formatting, Clippy, and full-test
  checks;
- recursive provider-verification sessions are forbidden and verified absent; and
- feature quality is scored by the frozen visual, `/status`, and completion fixtures; `/status` is
  the concrete slash command used to test the original generic requirement.

Direct and Work Leaf observations are independent groups. Running two workflows at the same time
only shortened collection; neighboring runs are not treated as statistical pairs. A failure in one
workflow never removes another workflow.

The causal conditions use an isolated instrumentation build at commit
`4707ceb4903a09646857d1e316cb45acb15a3d07`. The production Work Leaf checkout and the frozen task
were not modified. `evidence.json` records and hashes the admissions, reports, observer analyses,
mechanism summaries, recursive-call logs, scorer logs, and Point 7 inputs.

## Overall Workflow Result

The endpoint groups contain these observations:

| Workflow | Run | Features | Raw-token result |
| --- | --- | ---: | ---: |
| Direct | `point7-exact-direct` | 3/3 | 41,035,124 exact |
| Direct | `direct-003` | 2/3 | 28,877,983 exact |
| Direct | `direct-002` | 3/3 | 35,677,252 exact |
| Normal Work Leaf | `wl-normal-003` | 3/3 | 12,777,778 observed; 33,177,778 upper bound |
| Normal Work Leaf | `wl-000-003` | 3/3 | 12,719,646 observed; 32,319,646 upper bound |
| Normal Work Leaf | `wl-000-002` | 2/3 | 16,471,729 observed; 50,071,729 upper bound |

Group summaries:

| Measure | Direct | Normal Work Leaf |
| --- | ---: | ---: |
| Observations | 3 | 3 |
| Mean completed features | 2.67 | 2.67 |
| Mean raw tokens observed | 35,196,786 exact | 13,989,718 incomplete |
| Mean conservative raw upper bound | 35,196,786 | 38,523,051 |

Using only captured completed responses, Work Leaf is 21,207,069 tokens, or 60.25%, below direct.
Using the conservative missing-response maximum, Work Leaf can be 3,326,265 tokens, or 9.45%, above
direct. Because this interval crosses zero, the repeated result does not prove an average raw-token
saving. It also cannot establish an uncached-token reduction.

The missing-usage allowance is intentionally broad. Work Leaf interrupts a response immediately
after a complete orchestrator directive, while the current ChatGPT Codex transport reports usage
only for completed responses. The upper bound charges at least 400,000 raw tokens for every such
interruption and uses a larger aggregate when context-window, maximum-output, and captured-prompt
headroom require it. This prevents undercounting but makes the result imprecise.

## Delivery Controls

Condition names are `wl-XYZ`. `X` controls changed-file rereads, `Y` controls unchanged-file
rereads, and `Z` controls review context. `0` is normal Work Leaf behavior; `1` is the less compact
control. For example, `wl-110` sends full content for both reread types while keeping exact inline
review context.

### Changed Files

Normal changed-file rereads returned diffs. The verified `wl-000` trace delivered 22,241 bytes and
measured 350,297 bytes avoided across nine events. Full-file delivery activated in `wl-100` and
`wl-110`, with 15 and 8 events respectively.

The balanced four-condition screen estimates full-file minus diff usage between 25.02 million
fewer and 19.82 million more raw tokens. The interval crosses zero, and quality differs across
cells. The study cannot establish the direction or percentage of a changed-file diff effect.

### Unchanged Files

Normal unchanged-file rereads returned a digest. The two normal-digest traces measured 45,348 and
20,678 avoided bytes. Full-file delivery activated three times in `wl-010` and eight times in
`wl-110`, adding 106,112 and 294,141 bytes relative to their measured digest counterfactuals.

The balanced four-condition screen estimates full resend minus digest usage between 19.47 million
fewer and 25.37 million more raw tokens. The interval crosses zero. The byte-level saving is real
where measured, but its whole-workflow token effect is unresolved and cannot explain a precise
fraction of the observed gap.

### Review Context

The normal path supplies exact review context inline. The control asked reviewers to reconstruct
context from Git. That control repeatedly entered a review-routing loop, including `wl-111-003`.
The remaining Git-control conditions were withheld under the predeclared stop rule.

This is useful reliability evidence: Git reconstruction is not behaviorally equivalent to the
normal review path in this workflow. It is not a valid token comparison, so review context receives
no causal estimate.

## Strongest Remaining Explanation

Across the three endpoint observations per group, the workflow summaries are:

| Activity | Direct mean | Normal Work Leaf mean | Work Leaf reduction |
| --- | ---: | ---: | ---: |
| Commands | 657.0 | 277.3 | 57.79% |
| Repeated commands | 148.7 | 10.3 | 93.05% |
| Validation commands | 53.3 | 26.3 | 50.63% |

These differences are consistent with Work Leaf keeping feature work scoped, avoiding repeated
reads and checks, and consolidating the reviewed stack once. They are large enough to plausibly
explain much more of the token signal than the measured reread byte reductions.

This is still an association. The study did not run a normal Work Leaf control that deliberately
reproduced direct Codex's command and validation cycle pattern while holding everything else fixed.
The exact token fraction caused by fewer cycles is therefore unknown.

## Retained Failures And Limits

- `wl-111-003` is a reliability failure caused by the Git-review control loop. Its candidate and
  usage were lost during signal cleanup, but its admission and full driver log remain.
- `wl-001-001`, `wl-011-001`, `wl-101-001`, and `wl-111-002` were withheld after the same control
  failure became systematic.
- `wl-100-001` and `wl-110-001` completed 2/3 features; they remain in the causal screen.
- `direct-003` and `wl-110-001` both completed the same 2/3 features.
- `direct-002` completed 3/3 while `wl-000-002` completed 2/3; both remain in the endpoint groups.
- The all-three-disabled endpoint has one completed historical observation and one current
  reliability failure, so its residual saving was not replicated.
- Exact Work Leaf raw and uncached usage is unavailable on the current transport.

## Reproduction

The machine-readable result is `evidence.json`. It is generated and verified by:

```sh
python3 -m unittest -v bench-results/efficiency-points8-9-20260828T145556Z/test_analyze.py
python3 bench-results/efficiency-points8-9-20260828T145556Z/analyze.py
```

Schedule checks use:

```sh
python3 bench-results/efficiency-points8-9-20260828T145556Z/test_schedule.py
```

## Conclusion

The study finds a large and repeated observed token signal, with comparable average feature count,
but it cannot prove the average saving under conservative missing-response accounting. The two
read-delivery mechanisms do not have a measurable whole-workflow effect in this sample, and the
review-context control is invalid because it changes workflow behavior. Fewer command, repetition,
and validation cycles are the best-supported explanation for a real Work Leaf advantage, but the
exact fraction and exact overall saving remain unknown.
