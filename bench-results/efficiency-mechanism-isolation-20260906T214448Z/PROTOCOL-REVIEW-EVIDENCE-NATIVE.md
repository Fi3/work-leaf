# Exact review evidence: private C21 verification protocol

Status: prospective qualification, **no provider admission**. This protocol supports
Stage B of [the active plan](PLAN-CANDIDATE-INVENTORY.md), not a control arm, replacement,
new sequential comparison or an automatic complete-workflow schedule. The current three
candidate outcomes and their postcapture audits precede any C21 provider admission.

## Factor and isolation

The factor is eager inline versus on-demand native read-only access to the exact same
rendered reviewer source context. `src/cli.rs` holds the ordinary source context once;
the private `src/bench_review_evidence.rs` hook receives its exact UTF-8 byte span after
ordinary review-prompt construction. The span includes the original commit/log/recorded
chat evidence, with no Git reconstruction, summary, deletion or content-marker parsing.
The candidate replaces only that span with a typed path/length/checksum receipt.

Activation requires `bench-experiments` and a valid schema-5 experiment manifest.
Condition `review-evidence-native` selects the receipt. `review-evidence-inline` exists
only for provider-free identity tests and is not generation authority. The separate
canonical archive root must not overlap the project or ordinary context-bundle root,
including its default location. Exclusive publication, read-only sealing, independent
archive numbering and exact bytes prevent host overwriting or reusing prior snapshots
and ordinary bundle-sequence effects. These endpoint safeguards do not establish
continuous immutability or containment against hostile concurrent writers.
Runtime FNV64 is a labeled identity check; offline SHA-256 and exact byte comparison
independently bind every archive. No new runtime cryptographic dependency or external
hashing process is required.

Default/inactive/v1–v4 paths remain unchanged and allocate no opaque evidence. Author
policy, ACK, commands, review obligations, fix/recheck, ordinary reads/bundles, provider,
observer and interruption behavior remain fixed. Native access to this exact immutable
artifact is the factor's permission; broader file access is not introduced. Ordinary
`@work-leaf read` does not serve the opaque evidence. Full design and confounds are in
[DESIGN-REVIEW-EVIDENCE-ON-DEMAND.md](DESIGN-REVIEW-EVIDENCE-ON-DEMAND.md).

## Qualification gates

Before admission, retain failing-before-fix tests, full default/legacy prompt identity,
both selection paths, publication/failure/namespace checks and the ordinary read route.
Require `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-targets --all-features`; required provider tests remain explicitly
ignored until separately admitted. Independent review covers the private runtime,
real diagnostic fixture and source-bound offline collector. The documented runtime
missing-ancestor path resolution is O(D²) in path depth D; no transcript-history
quadratic path is introduced. The collector must avoid archive × transcript rescans.

A source-bound build record uses schema
`work-leaf-review-evidence-build-attestation-v5`, canonical `source_repo`, logical
`source_files` and `binaries` lists of `{path, sha256}`, exact command
`cargo build --release --features bench-experiments --bins`, integer-zero build exit,
unchanged-observer proof and committed runtime identity. The diagnostic additionally
pins its actual test executable in `smoke_test_binary`, its build command/exit and
fixture source. Actual artifacts, not anticipated hashes, populate this record.

## Single bounded real-subscription diagnostic

The sole prospective diagnostic identity is `review-evidence-native-diagnostic-001`.
A separate create-new admission must pin exact sources, build, subscription wrapper,
observer, model/effort, fixture configuration, archive root and output paths before
generation. Exactly one attempt is allowed; no retries or replacements are automatic.
It is not a complete benchmark observation and supplies no causal token estimate.

The fixture's ignored `real_subscription_review_evidence_handoffs` test drives actual
`CommandChat` launch/edit/ACK/done/review/locked-check/clean-verdict flow. A one-line
supplied fixture changes VALUE from 0 to 1; the author records fixed non-Git public
evidence. The reviewer reads the complete issued archive with one native `cat --`
call, requests the existing focused check, then returns `NO_FINDINGS` on actual success.
These diagnostic-only instructions precede ordinary review rendering and do not
rewrite delivered output after the factor. Complete benchmark reviewers must not be
forced to read all context: actual selective, complete or absent retrieval is measured.

Expected outer generation is four turns in two threads, with a hard six-call ceiling
and 118-second workflow/settling watchdog. An outer operator timeout preserves failure
and all captures; it is not retry authority. Fixed ordered handoffs reject extra calls.
The final accepted turns must settle before capture closure and inspection.

Only the existing stored ChatGPT subscription is allowed, through the pinned forced-
subscription launcher: no API keys/credits, credential copies or model/provider changes.
Model/effort remain `gpt-5.5`/`xhigh`, with original observer raw usage, 1000-ms grace,
resumed-output `forward` and project-inventory settings. Pre-agent initialization/auth
failure is a retained blocked diagnostic, not green real verification or a retry loop.

## Postcapture proof and subsequent work

Retain original process exit, workflow result and later audit failures separately.
The new source-bound collector must independently hash inputs before and after replay;
join every accepted public input to exact native thread/explicit turn/full text; retain
usage-less sessions; verify archive manifests, receipt rendering and source-owned ranges;
and join the actual native read call/output after the archive's issued review input.
Public command text alone or a path mention does not prove retrieval. Complete, exact
partial, absent-with-complete-inventory and unresolved are distinct outcomes.

Actual subscription/backend/model/effort and project inventories need independent
evidence, not declarations. Final byte identity and complete native access qualify
agent-facing plumbing; they do not demonstrate net savings. Missing evidence and failed
verification remain visible and preclude a green readiness report.

Complete-workflow generation requires a later fixed-count, source-frozen admission
after the original three candidate results and this diagnostic. It reuses the saved
six normal-WL observations, including failed 010, under identical accounting scope.
No new baseline or Direct run is implied. Subsequent directional evidence includes all
outcomes and unbounded gaps; only a separately specified confirmation can add modified
repetitions. A marginal C21 difference is not automatically its share of the historical
45.38%–51.62% saving or an additive component of other factors.
