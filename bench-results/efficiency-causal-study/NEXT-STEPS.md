# Next Steps For A Valid Efficiency Study

## Goal

Measure normal concurrent Work Leaf against fair normal direct sequential Codex on the same three
requests, then identify which Work Leaf mechanisms account for any real token difference. Preserve
every success, partial implementation, failure, and missing measurement. Analyze direct and Work
Leaf runs as independent groups rather than discarding one run when another fails.

## Current Evidence

The corrected Point 7 study is in
`../efficiency-point7-exact-accounting-20260828T113610Z/FINAL-RESULT.md`.

- Direct sequential Codex completed all three requested features. Its exact total is 41,035,124 raw
  and 1,982,580 uncached tokens.
- The Work Leaf accounting attempt changed normal directive handling and failed during review. Its
  partial candidate scored 2/3, but its token total is incomplete and unusable.
- No valid direct-versus-Work-Leaf percentage was produced.
- The planned Work Leaf run with all three suspected saving mechanisms disabled was not launched
  after the shared measurement defect was confirmed.

All earlier normal-workflow and attribution percentages are withdrawn. Their Work Leaf observers
counted completed-response usage but omitted responses interrupted at orchestrator directives. The
historical artificial-validation runs remain useful only as qualitative mechanism traces.

## What Is Verified

The fair benchmark setup can hold these controls constant:

1. the original task with `/status` and without `/fork`;
2. base commit `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
3. GPT-5.5 with xhigh reasoning for every provider thread;
4. normal validation behavior and the same final repository checks;
5. no recursive provider-verification sessions;
6. concurrent Work Leaf versus sequential direct Codex; and
7. the same frozen three-feature scorer, with every quality outcome retained.

Direct token accounting is also verified: every completed initial and resumed Codex invocation is
added once and reconciled with saved provider records.

## Remaining Blocker

Normal Work Leaf immediately interrupts a provider response after a complete orchestrator directive.
On the current ChatGPT Codex transport, exact usage exists only on `response.completed`; interruption
produces no exact or cumulative usage for that response. Waiting for completion changes Work Leaf's
normal behavior and is not a fair accounting fix.

Real GPT-5.5/xhigh probes reproduced this with Codex CLI 0.149.1 and 0.150.1. Server-side account
usage was unavailable, and the same endpoint rejected stored and background responses. The complete
evidence and Codex source call chain are in
`../efficiency-point7-exact-accounting-20260828T113610Z/FAILURE-ANALYSIS.md`.

## Required Decision

No more paid runs should start until one measurement contract is chosen:

1. **Exact provider measurement:** use a transport that reports usage for cancelled responses. Both
   workflows must use the same declared account and transport. This is a new study because it is not
   the current normal ChatGPT Codex path.
2. **Estimated local measurement:** preserve the current normal workflows and estimate interrupted
   response usage from captured request and response content. The report must label raw and uncached
   values as estimates, document uncertainty, and never mix them with exact provider totals.

After the measurement method is frozen and verified, run a small gate containing one direct run, one
normal Work Leaf run, and one Work Leaf run with all three candidate mechanisms disabled. The
all-three-disabled run is mandatory because it checks whether the suspected mechanisms explain the
overall difference. Only a green gate should proceed to repeated normal-workflow runs and the full
eight-setting attribution study.

Cross-project replication and other model profiles remain future work.
