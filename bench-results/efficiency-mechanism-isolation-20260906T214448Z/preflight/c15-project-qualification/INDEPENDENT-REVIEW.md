# Independent frozen-project input review

No blocking finding in this bounded private-input qualification. The complete new
helper, seven tests, contract, qualification note and execution receipts were
reviewed. This is a frozen accepted project plus a declared private overlay and
public Cargo capsule, not live-overlay admission or runtime integration.

Reviewed identities:

- `project_inputs.py`: `b2f2905b3ffe9a0e356577d78eae2249b6c16a6919b0a359d731ab2e2ed0f862`.
- `test_project_inputs.py`: `ff5d86d02ad4b2037a42f4ff94da2f018ea7a2341f389d1a62eb6abea0779842`.
- `QUALIFICATION.md`: `444745b392c0993dbc072733006c270cf9387cc2cc252903fe06fe61779d7163`.
- `QUALIFICATION.json`: `2b9d55936e7d4f1a279899979a76517ac8d9dc5e2446e73ee38c0295f176c456`.
- `PROJECT-001/RESULT.json`: `8b8e509b9359542f99bc6e306836b445173391ccd8622ccbb6a72e5564d53d9f`.

## Source and isolation checks

`prepare_capsule` selects locked crates.io package identities, exact public registry
configuration, corresponding sparse-index files and available checksum-matched
archives. It does not enumerate/copy Cargo home, credentials, configuration or
Cargo source trees. Unsupported registries, invalid identities, checksum drift,
aliases and hardlinks are rejected. Missing archives remain named outcomes.
`prepare_cargo_home` requires the published receipt and normalized relative
index/cache paths before reading entries, then creates only the two private
registry links to `/cargo-public`. The executor still supplies the read-only mount;
these links alone are not a sandbox.

`install_private_overlays` uses the pinned materializer's complete accepted-image
identity before validating every exact declaration. It records the immutable
declaration, private effective bytes/modes and separate private metadata commit.
The shared accepted commit remains distinct; no flag is cleared, live checkout is
read as if clean, or private commit promoted. The helper has no AGENTS-filename
exception. Trusted precreated private roots and caller serialization remain the
predecessor interface assumptions, not a hostile-host concurrency guarantee.

The actual declaration independently equals the retained
`bench-agent-profile-common::bench_install_no_recursive_agent_policy` formatter:
original bytes, two LF bytes and the exact policy here-document. Base SHA-256 is
`e72abfe8ed19bf1f01684cca2dc540172956c9935151a4e9a38e0c1442985973`;
effective SHA-256 is
`5f6ba65e9708299c97fa6856d6eb4e9ac09434ca19d7add22b90862fb20d652c`.
Its source-documented flag is not falsely described as an actual live-index census.

## Independent verification

Both ordinary discovery and an exact-source-byte compiled test run pass all seven
tests. The latter passes in 1.012 seconds and cannot execute stale Python bytecode.
The tests exercise actual private Git materialization/overlay operations and
synthetic public-cache safety checks; no provider or Cargo command is launched by
this review.

Independent endpoint checks pass all eleven qualification file pins, all ten
original input identities, all 53 capsule files and all 53 selected registry source
files. The original execution's two earlier helper/test hashes resolve to the
explicitly retained `source-at-project-001/` bytes; they are not relabeled as the
final source. The final guard's separate actual-capsule replay is retained without
pretending the original Cargo command ran under later source bytes.

The actual execution receipt retains exit 0, closed processes/pipes, no timeout,
the exact offline/locked command, 28 individual PASS test lines and matching
stdout/stderr hashes. The current accepted Git/file identity equals the saved
accepted image; the current private identity equals the saved overlaid after-image.
The review does not rerun the actual Cargo workload or infer broader test coverage.
The documented correction from 29 versions to 27 index files is explicit; the
original contract's physical-count error is retained, not silently overwritten.

## Remaining boundaries

This passes the precise frozen-project Linux feasibility question. Live flagged
source selection, actual shared `FileLockTable` ownership, proposal/runtime
integration and agent verification remain separate and unadmitted. Private HOME,
fresh Cargo/build state, stripped configuration, namespaces and public read-only
tool inputs differ from the ordinary benchmark environment. The receipt pins
executables and selected sources, not every toolchain library/system file. No
all-command, timing, cache, quality, token-saving or causal-equivalence claim follows.

The new package/overlay joins are indexed, with linear byte scans and sorting;
there is no new unbounded O(N²) scan. The predecessor executor's explicitly bounded
mount-pair validation and materializer costs retain their prior review limits.
Private documentation describes this resulting interface and its limitations;
production architecture, public APIs, operator workflow and real-agent behavior
are unchanged. No real-agent verification is required or claimed for this offline
prerequisite. No existing helper, runtime source, test or original receipt was edited.
