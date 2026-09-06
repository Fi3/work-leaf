# Prelaunch implementation verification

The benchmark runtime uses the private nondefault `bench-experiments` feature. Both experimental
sites preserve control prompt bytes; default builds cannot activate the experimental conditions.
The actual release source and binary hashes are in `BUILD-ATTESTATION.json`.

## Automated gates

- Main crate: `cargo fmt -- --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo test --all-targets --all-features --quiet` pass. Logs: `fmt.log`, `clippy.log`,
  `cargo-test.log`. `cargo fmt` also passes during implementing-agent verification.
- Observer crate: format check, all-target/all-feature warning-free clippy, and all-target/all-feature
  tests pass: 33 library, 29 analyzer, and 45 proxy tests. Its code is unchanged by this study.
- Phase supervisor: 23 offline tests pass (`runner-tests.log`).
- Prospective analyzer: 22 offline tests pass (`analysis-tests.log`).
- Default-product experiment tests: three pass; feature-enabled tests: four pass, including
  control identity, exact treatment spans, output/pending-diff preservation, and invalid activation.
- `git diff --check` passes. Tests exercise RED before implementation for each experimental feature,
  accounting/runner behavior, reverse trace coverage, and the diagnostic teardown correction.

## Independent review

Rust review finds no introduced public-API, dependency-direction, provider-neutrality, or
single-factor-isolation issue. Architecture documentation describes the private benchmark
boundary; normal file-workflow documentation requires no change because normal behavior is
preserved. Operator policy identifies the separate causal study and its admission manifests.

The analyzer's initial missing reverse-coverage check has a RED/GREEN regression: every captured
eligible handoff must have its trace entry, as well as every trace entry having a captured handoff.
Independent replay of 2,916 compatible exact token assignments across 80 bounded examples keeps
every exact randomization p-value inside the reported conservative envelope.

Complexity flags: exact randomization enumeration is combinatorial, O(A × n), explicitly capped
at 200,000 allocations; the confirmation design has A=324 and n=12. Full frozen-input verification
costs O(waves × frozen evidence bytes), including potentially quadratic run-manifest metadata work
as an unrestricted schedule grows. This study's phases have fixed three- or twelve-workflow bounds.
Prompt transformations are linear in prompt bytes; event joins use indexed identities.

## Real backend

The existing subscription login reports `Logged in using ChatGPT`; CLI version is 0.153.4.
The ordinary two-turn interruption/raw-resume diagnostic passes in 5.56 seconds.
Three corrected experimental diagnostics pass in 11.36 seconds (control), 16.55 seconds
(acknowledgment variant), and 15.01 seconds (command-guidance variant). Each performs one actual
patch, one actual locked validation, and completion in three turns of one agent session.
Their source-evidence audit is recorded separately in `REAL-VERIFICATION.json`.

Three earlier diagnostic attempts remain failed and retained due to the fixture teardown race
documented in `FAILED-HANDOFF-01.md`. The correction waits for the already queued final interrupt
to be delivered and settled before observer shutdown. It changes only the diagnostic fixture,
not the benchmark runtime, grace policy, provider prompt, or provider-call count. Diagnostic
usage is excluded from the study's screening and confirmation observations.
