# V3 runtime qualification

The private read factor is qualified by same-source automatic tests, independent default identity,
real subscription-backed handoffs and separately identified release outputs. This is prelaunch
engineering evidence, not an observation or claim about whole-workflow savings.

Runtime source commit: `9751fa44d8301eb4749b10ce571b672dabdba8c6`. The root reviewer independently
rehashes all 27 original/snapshot compiler inputs and both release executables against
`BUILD-ATTESTATION-UNTRACKED-READS.json`, plus the unchanged observer executable. All match.
Comparison to `BUILD-ATTESTATION-WORK-UNITS.json` finds 21 identical shared inputs and exactly two
changed shared runtime files: `src/orchestrator.rs` and `src/bench_experiment.rs`. The three newly
inventoried web assets equal their bytes at the previous compiled commit. The additional Rust
input is `src/bench_read_experiment_tests.rs`, included only under `cfg(test)`.

The implemented factor is confined to the successfully bundled untracked component. Its baseline
renderer executes once in each arm, retaining ordinary bundle creation and failure behavior.
Both full candidates are constructed and recorded symmetrically. Tracking, refresh, prompts,
commands, validation, provider settings, cancellations and waits outside this component are
unchanged. `V3-RUNTIME-VALIDATION.md` and `V3-READ-RUNTIME-REVIEW.md` retain the RED-first tests,
passing required Rust checks and complete twelve-prompt pre-patch default identity comparison.

`READ-INLINE-PRIMARY-DIAGNOSTIC-REVIEW.md` and its JSON establish the actual real-agent gate.
Both conditions complete the prescribed one-thread, three-response read/repeat/done sequence,
with the same runtime source bytes and GPT-5.5/xhigh subscription backend. Actual primary/raw
capture, 1,000 ms grace, forward-on-resumed-output and project inventory settings match. All
six native/raw response identities and attribution rows reconcile; policy and repeated-read
inputs match exactly. The root reviewer independently rehashed all 44 retained review sources.
The initial missing-marker failures remain retained and are not green verification.

The real scenario uses the debug integration executable identified in that admission, not either
release executable. The release outputs are independently tied to the exact same source inputs;
there is no `cfg(debug_assertions)` runtime branch in these sources. The separate
`V3-RELEASE-STARTUP-CHECK.json` tests the actual release executables in thirteen provider-free
cases, including default-off/control/treatment activation and fail-closed invalid activation
before a backend or listener. Binary hashes remain:

- `work-leaf`: `469491dd93cb7a42a4e32d76d577d1eab4fe4da135f86e2242084c2cc9f45a09`.
- `work-leaf-orchestrator`: `7f2707efb4c4630e61fc57bfc0555f404f0d3aa9f6c16235e304c37a1b415b9e`.
- Unchanged observer: `238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`.

The earlier build note's pending exact-release real check is a review-cutoff note, not a claim
that these release binaries performed the real diagnostic. The gate is the same-source real
scenario plus explicit release identity/startup qualification; no additional provider diagnostic
or benchmark observation follows from it. Census helper review, final phase freeze and the
subscription resource wait remain separate pre-admission requirements.
