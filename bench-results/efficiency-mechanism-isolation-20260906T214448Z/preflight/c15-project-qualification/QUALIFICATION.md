# Frozen-project offline Cargo feasibility

**Qualified:** the frozen benchmark project builds its locked Linux dependencies
and passes all 28 existing `ui_harness` tests inside the qualified private executor.
The actual command exits 0, closes without timeout and takes 5.618 seconds. This
is one provider-free prerequisite, not a benchmark or a test-first causal result.
No production/test file in the frozen project is changed for this command.

## Exact evidence

[QUALIFICATION.json](QUALIFICATION.json) pins the implementation/tests, original
source input locations, command/source admissions, execution and overlay/capsule
receipts. Full artifacts remain under `PROJECT-001/`; their trees are excluded
from accidental Git staging, not deleted. `PROJECT-001/RESULT.json` SHA-256 is
`8b8e509b9359542f99bc6e306836b445173391ccd8622ccbb6a72e5564d53d9f`.

The accepted base is `c92a0b7060a36eac6db2d869b85e589a7a9480f9`, tree
`acb6583fbb0e25347f70ee44cbdacafd6df95e37`, selected from driver HEAD
`3c907f7266b63efa1558213f3e49587e999b899d`. The command is:

```text
/usr/bin/env RUSTC=/toolchain/bin/rustc PATH=/toolchain/bin:/usr/bin:/bin /toolchain/bin/cargo test --offline --locked --test ui_harness
```

The selected test target and `src/ui_harness.rs::UiHarness::{new,execute_prompt}`
at that commit use fixture state/transcript operations, not a configured backend.
There are no provider calls or agent launches. The 28 exact test names and their
PASS results are retained in the command output; all 25 Linux crate archives were
compiled from the locked versions with private Cargo/build state.

## Source overlay and public cache boundary

The effective public instruction bytes are derived exactly from the frozen
`bench-agent-profile-common:73–99` formatter: original tracked bytes, two LF bytes,
and its fixed here-document policy. Their declaration and private before/after
Git/file identities are retained. The generic adapter uses the declared path,
base/effective hashes and modes; it has no AGENTS-name branch. A private metadata
commit records the overlay so the existing clean-image/patch semantics can be
reused. It is explicitly not an accepted shared commit. The accepted base remains
separate, and the accepted source plus the private overlaid source are unchanged
by the actual check.

This is a **frozen accepted base plus declared private overlay**, not a successful
census of a live flagged checkout. `observed_index_flag` in this declaration names
the source-documented driver flag, not a captured live-index observation. The
unchanged materializer still rejects skip-worktree/assume-unchanged inputs. Live
flagged-source admission and actual same-table `.` lock selection remain separate
prerequisites; no live flag is cleared and no shared file is overwritten.

The capsule has 29 locked versions in **27 distinct sparse-index files**, 25
checksum-matched archives, and one public registry configuration: 53 files total.
The original [contract](CONTRACT.md)'s wording “29 … sparse-index … entries” is a
physical-count error; it conflates versions with distinct names. Its original
bytes/hash are retained. Missing Windows archives are `clipboard-win-5.4.1`,
`error-code-3.3.2`, `windows-link-0.2.1`, and `windows-sys-0.61.2`; none is fetched.
That omission does not prevent this actual Linux target, but is not a Windows or
all-target dependency-completeness claim.

Only explicitly selected public index/cache files are copied into a create-new
capsule. Host Cargo source directories, Cargo home/config/credentials, rustup
configuration, shell setup and provider state are never copied or mounted. The
capsule and installed public toolchain directory are read-only mounts. Private
Cargo-home symlinks point to the capsule; extraction and package locks remain in
private scratch and builds under `/build`. Source, selected registry/capsule
files and Cargo/rustc executable hashes pass the recorded before/after checks.
The installed toolchain/system distribution remains a trusted public dependency;
the receipt does not claim a complete shared-library/system-file attestation.

## Tests and source versions

The new adapter has seven passing tests (1.025 seconds). Initial missing-module
RED and an actual unlisted-parent-read regression RED precede their fixes.
Coverage includes wrong registry/archive/index/config identity, missing archives,
alias/hardlink exclusion, credential non-copying, exact private Cargo links,
duplicate/mismatched overlay rejection and accepted-source preservation.

The actual Cargo execution used helper `b020421c…` and tests `95148d93…`; exact
bytes are retained under `source-at-project-001/`. Final helper `b2f2905b…` and
tests `ff5d86d0…` also reject a forged capsule receipt before any parent-path read.
That added pre-read guard accepts the exact completed capsule in
[FINAL-GUARD-REPLAY.json](FINAL-GUARD-REPLAY.json); it does not rerun Cargo or alter
the original execution. Archived sources are identity evidence, not modules to
execute using the archive directory as their original `__file__`.

All qualified executor/materializer bytes remain unchanged. This private Python
prerequisite changes no Rust/runtime source, existing tests, public APIs or
documented workflow; no additional real-agent gate or duplicate root Cargo suite
is asserted. A future agent-facing integration still requires its own gates.

## Smallest remaining contract

The actual frozen project's Linux offline build is feasible with these public
inputs; a generic no-dependency fixture is no longer the only feasibility witness.
The full C15 runtime is still unadmitted. A narrow live-overlay validator must
compare the complete actual tracked/index/flag census against the accepted tree
plus predeclared overlays, rejecting every undeclared difference without touching
live source. Then the private owner can select the base through the actual lock
table, retain the held proposal and validation result, and leave the later normal
shared patch authoritative.

Private HOME, isolated namespaces, stripped environment, private Cargo/build
state and the reduced configuration are real differences from the host benchmark
runner. One target's success is not all-command, timing, quality, cache or
environment equivalence. This qualification supplies no token cost or savings.
The new input adapter uses indexed package identities and constant byte passes;
no new all-pairs scan exists. The predecessor executor's bounded mount `O(M²)`
check remains, and Git/materializer history-copy and repeated snapshot-validation
costs retain their already documented bounds and limitations.
