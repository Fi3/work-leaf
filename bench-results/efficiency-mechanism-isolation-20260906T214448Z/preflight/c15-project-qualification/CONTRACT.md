# C15 frozen-project offline qualification contract

This is a provider-free feasibility prerequisite, not a benchmark, test-first
effect estimate, complete runtime implementation or environment-equivalence claim.
The qualified executor and materializer remain separate, unchanged dependencies.

## Frozen source and discovered inputs

Driver checkout `infrastructure/driver-source` has HEAD
`3c907f7266b63efa1558213f3e49587e999b899d`.
`bench-three-features:62` fixes accepted base
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`, tree
`acb6583fbb0e25347f70ee44cbdacafd6df95e37`; `:405–409` independently clones
and detaches that base. Its `Cargo.toml` specifies Rust 2024, the work-leaf library
and two binaries, and rustyline/serde/serde_json/tui dependencies. The pinned
`Cargo.lock` SHA-256 is
`ffd7c0afbc27f9599345589178c6385974179458ac908161cff60f8611946758`.
All 29 external locked packages name the ordinary crates.io registry; no path/Git
dependency, base `.cargo` configuration, `build.rs`, or toolchain override is
present. Twenty-five cached `.crate` archives match their lock checksums; four
Windows-package archives are absent. This inventory is not proof that resolution
or Linux compilation succeeds.

`bench-agent-profile-common:73–99` retains original instruction bytes, constructs
effective public policy bytes, sets skip-worktree and replaces the tracked file.
It does not create a Git commit for that overlay. The original/effective distinction
must remain explicit; status-clean is insufficient. This qualification must not
clear live flags, overwrite the shared file, or fabricate an accepted overlay commit.

## Generic immutable overlay boundary

An overlay declaration must precede execution and name a normalized tracked path,
accepted blob/mode and original-byte SHA, exact effective bytes/SHA/mode, and the
expected index flag (`skip-worktree` for the observed driver). It is data, not a
filename-specific exception. Duplicate paths, undeclared hidden flags, unexpected
effective bytes, a changed staged blob/mode, aliases/hardlinks, an unknown flag,
untracked input or unrecorded source differences remain unsupported.

The accepted Git snapshot is materialized independently through the unchanged
qualified path. Exact declared overlay bytes may then be installed only in the
private image, with before/after identities and a separate overlay receipt. If a
private metadata commit is needed to reuse `GitPatcher`'s clean-image checks, it is
explicitly not an accepted shared commit; the immutable accepted base stays in the
snapshot record. Held proposals still use the existing private Git patch semantics
and no artifact is promoted. A live-index adapter requires its own source census
and tests; a frozen-base plus explicit overlay demonstration does not supply that
proof. The first project execution may qualify this narrower frozen-input case.

## Public-only dependency and tool inputs

Never mount or copy Cargo home, Cargo/global configuration or credentials, rustup
home, provider state, shell startup files, or the broad user home. A create-new
public capsule may contain only the selected crates.io index `config.json`, the
29 explicitly named sparse-index cache entries, and checksum-verified locked crate
archives that actually exist. Unsupported registries, ambiguous cache locations,
symlinks, hardlinks, malformed identities and checksum mismatches fail closed.
Missing target-inactive archives are recorded rather than fetched or fabricated.

Mount that capsule read-only at `/cargo-public` and the exact installed public
toolchain directory read-only at `/toolchain`. The qualified executor does not
allow nested mounts within its writable `/tmp`; instead, private Cargo-home
registry/index and cache symlinks may point into `/cargo-public`. Cargo extraction,
lock files and registry/src live in private scratch; builds use private `/build`.
No host Cargo cache receives writes. Capsule/source/tool executable hashes bracket
the execution; no network fetch or unconfined fallback is allowed.

Use the actual frozen project's `tests/ui_harness.rs` focused target, first as an
offline/locked build-and-test feasibility check. Its retained source uses the
deterministic `UiHarness`, not a configured provider. Record the exact command,
selected test identities/output, exit status, timeout/closure and source endpoints.
No tests or implementation in the accepted project are edited for this check.

## Environment and interpretation

The private source is shown at the supplied original absolute cwd. Explicit Cargo
and rustc executables come from the pinned toolchain; `RUSTC`, `PATH`, Cargo home,
target and temporary locations are declared. The ordinary benchmark's
`bench-validation-common:3–15` instead uses host PATH/Cargo and a child TMPDIR.
Private HOME, fresh extraction/builds, isolated process/network namespaces, reduced
configuration and read-only tool inputs are real non-target differences. One
successful focused Linux check proves feasibility for those exact inputs, not
all commands, platforms, Git-dependent tests, timing, cache behavior or environment
equivalence. Unavailable dependencies and actual failed checks remain recorded.

## Test-first and authority boundary

New tests must fail before the new capsule/overlay adapter exists. They must cover
wrong lock checksum, unapproved registry/config fields, missing index identity,
path alias/hardlink, duplicate/undeclared/mismatched overlay, and provider/Cargo
credential exclusion. The actual frozen-source execution follows those unit gates.
No provider, control run, active runtime mutation, existing-helper edit or current
benchmark admission follows from this document. Current-project runtime admission
still requires a reviewed live-overlay adapter and actual owned-lock integration.
