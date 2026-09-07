# C15 accepted-tree materialization qualification

## Decision and scope

**GO for this provider-free prerequisite on the admitted clean, trusted regular-file
repository contract. NO-GO for full C15 runtime or current-project admission yet.**
The actual generic test passes the required private test RED → unchanged-test plus
private implementation GREEN sequence. No provider, normal Work Leaf workflow,
shared production patch, shared ACK, promotion, or causal/cost observation exists.

The source contract is deliberately narrower than “git status is clean.” A current
benchmark checkout with an effective instruction overlay hidden by skip-worktree
is unsupported, regardless of filename. The generic fixture has no external
dependencies; it does not establish current-project Cargo cache/dependency,
generated-input, instruction-overlay, or environment equivalence.

## Source boundary

`materialize.py::materialize` requires a canonical, disjoint, exclusively owned
empty destination and an exact expected accepted commit/tree. `source_identity`
checks staged object IDs/modes against that tree, actual tracked file bytes against
Git blob IDs, regular-file modes, administrative file hashes, and the complete
untracked/ignored census. Skip-worktree and assume-unchanged flags are rejected even
with equal bytes. It rejects linked Git administration, alternates, path aliases,
external hardlinks, symlinks, submodules, conversion attributes and unsupported
local include/filter/worktree configuration. There is no filename exception.

An independent `clone --no-checkout --no-hardlinks --local` and detached checkout
materialize the selected objects. The private index and object files are separate
inodes without external hardlinks; no linked worktree is used. Clone copies Git
objects/history, not merely the selected tree, and keeps a local origin path. It
does not copy the source's local configuration/hooks or user/global/provider
configuration. Admitted repository contents and explicit tool mounts must already
be public/trusted: this helper is not a secret detector for tracked Git history.

The source checks bracket materialization. They are endpoint checks, not an atomic
live-worktree snapshot or proof of a lock held by an independent process. Existing
`src/locks.rs:105::FileLockTable::locks_for` keys exact paths;
`src/patch.rs:382::patch_lock_paths` includes `.` to serialize patch Git work.
Commands can lock different keys. Future in-process source selection must use the
same owned lock table; this standalone qualification explicitly records
`in_process_wl_lock_proved: false`. No shared lock is held during private tests.

`patch_driver.rs::execute` calls the existing public
`src/patch.rs:64::GitPatcher::apply` or `:77::apply_edit` on the private repository.
Unified diffs retain the ordinary `git apply --recount --check` path at `:90`;
structured edits retain `compute_structured_edit_changes` before writes at `:150`.
There is no second edit interpreter. The normal private metadata commit is an
implementation detail, never a shared acceptance or promoted result. Existing
architecture ownership (`docs/architecture.md:844`) and public APIs are unchanged.

## Proposal, confinement and evidence

`apply_proposal` retains the exact typed declaration/body in a create-new proposal
directory before execution. It records before/after SHA-256, bytes and Git mode
for each changed file plus private commit/tree identities. A later accepted shared
commit never replaces the held private base. The current shared identity is checked
before/after each qualification and recorded separately; concurrent drift fails
qualification without rebasing or overwriting it. This static endpoint condition
is not a proposed global barrier for ordinary runtime concurrency.

The exact-pinned [qualified executor](../c15-private-executor/QUALIFICATION.md)
provides the filesystem/process boundary. The original absolute source path has
the private repository mounted over it; only owned source/build/scratch are
writable. The driver and exact proposal directory are explicit read-only mounts.
Git author metadata is synthetic; global/system configuration and hooks are
disabled. There is no provider launch, credential copy or inherited auth setup.

Every completed patch-driver execution is retained before after-image validation.
Unsupported after-images, including a privately created symlink, retain the
execution and failure receipt; they are not reported as accepted snapshots. A
publication failure before proposal execution executes nothing. A later evidence
failure can leave private effects and partial artifacts, which remain available;
there is no rollback/promotion claim. Duplicate IDs never execute twice. The
helper never deletes owned evidence. Unit-test fixtures alone use their normal
temporary-directory cleanup after capturing the qualification receipt.

Test purpose is explicitly declared, not inferred from a filename, patch success,
compiler failure or nonzero command status. Exact test-unit semantic/range evidence
and subsequent normal shared acceptance remain future protocol responsibilities.
External inputs are not silently reconstructed: unsupported dependencies/network
or missing read-only mounts produce their actual confined failures.

## Verification

[QUALIFICATION-001.json](QUALIFICATION-001.json) binds 33 source/build inputs and
the executable, before/after hashes, exact generic proposal bodies, file/mode
identities and four real confined executions. The patch-driver/test/driver/test
exit codes are `0, 101, 0, 0`. Both Cargo checks use the same exact command and test
bytes; RED names the expected assertion (`left: 1`), and GREEN names its success.
The shared HEAD, tree, index/admin hashes and tracked contents remain identical;
the held test is absent from the shared tree. All processes are closed.

Twelve Python integration tests pass (6.615 seconds), including dirty/index/hidden
flags, ignored/untracked input, alias/hardlink/symlink/submodule/filter rejection,
snapshot drift, duplicate ID, evidence failure, exact-block/path failures, later
accepted HEAD and bounded reads. The receipt retains initial missing-module RED
and subsequent useful regression REDs before their corresponding fixes. The new
private Cargo target passes offline build, fmt, all-target/all-feature clippy with
`-D warnings`, and its test build (zero Rust unit cases; Python exercises the real
binary). No root Rust source or existing test is modified by this prerequisite.

The wrapper performs a constant number of full file/byte passes per snapshot or
proposal, plus `O(F log F)` changed-path ordering and Git's existing operations.
Cloning costs the copied Git object/history size, not only current-tree bytes;
each Git subprocess has a 30-second bound. No new all-pairs file scan exists.
Repeated proposals revalidate the complete snapshot; this is `O(P × B)`, not an
incremental cache claim. The executor's previously flagged bounded `O(M²)` mount
overlap check (at most 32 explicit mounts) remains unchanged. Tracked input is
bounded at 100,000 files/256 MiB; file bodies are bounded before reading. Git
metadata/history and admitted tool mounts are trusted inputs, not a disk-quota or
hostile-host-race qualification.

## Smallest subsequent contract

Keep this prerequisite separate from runtime admission. First qualify the exact
current project's source contract (or a separately declared immutable generic
overlay), offline dependencies/read-only caches and requested test command.
Then an owned private orchestrator operation can hold proposal identity, select
the accepted base using its actual lock table, invoke the confined path, retain
the real result and deliver one truthful preview continuation. Ordinary later
patch application/conflict handling stays authoritative. No public API or
architecture change is needed for this offline prerequisite; real-agent
verification is required before any future agent-facing integration is ready.
