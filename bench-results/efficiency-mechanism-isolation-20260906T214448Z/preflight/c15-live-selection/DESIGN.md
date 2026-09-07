# Live overlay census and caller-owned selection

This provider-free prerequisite keeps all earlier qualified code/output unchanged.
Its accepted image is immutable Git objects plus predeclared overlay bytes, not
an atomic copy of every live file. Runtime enrollment, preview directives and
agent delivery are outside scope.

`capture_selection(root, owned, expected_commit, overlays)` is a synchronous private
operation for a caller already inside its actual shared
`FileLockTable::with_read_locks(["."])` closure. It samples the complete live
tracked/index/flag/file census twice, checks exact accepted blobs/modes and every
overlay's original/effective identity, and writes a create-new independent Git
bundle of selected HEAD between those endpoints. It fails on any undeclared
dirty/hidden/untracked input, alias, unsupported repository feature or endpoint
drift. The bundle cannot be inside the shared root/Git administration. No live
index flag, file, ref or Git configuration is written.

The opaque bundle hash and selected commit/tree are fixed before returning.
Subsequent materialization verifies those bytes and objects and does not read
live HEAD. A later ordinary accepted commit cannot silently substitute the base.
Clone/build/proposal execution and agent sends take place after the caller releases
the lock. No retry or new selection is implicit in a failed preparation.

`src/locks.rs:105::locks_for` keys exact normalized paths;
`src/patch.rs:382::patch_lock_paths` includes `.`. The prospective in-process
orchestrator caller must use `DirectiveServices::locks`, not construct a new lock
table or OS lock file. The qualification Rust caller intentionally creates a
standalone fixture table shared with a competing real `GitPatcher`; its proof is
limited to that owned fixture and does not claim a lock in a running daemon.

The `.` read lock excludes cooperative patch/index mutation, not ordinary commands
on different keys or external processes. Repeated live census establishes only
the recorded endpoints. Git objects and exact declared overlay bytes define the
selected image even if transient unrelated writes escape those samples. Bundle
creation copies reachable history inside selection; the 512 MiB limit admits or
rejects the completed bundle, not its in-flight storage use. Reachable history
can exceed the selected source tree's 256 MiB bound. The pinned Git subprocess
has a 30-second timeout per command, not a whole-operation deadline. The receipt
records selection duration and endpoint bundle size on success and failure; the
standalone caller separately records actual root-lock duration. Full
clone/test/agent time is excluded from the critical section. This is not
product-wide atomicity or an unchanged-scheduling claim.

Overlay declarations are generic normalized tracked paths with accepted blob,
SHA-256 and Git mode, exact effective text/SHA/mode and explicit skip-worktree /
assume-unchanged booleans. All declared paths must appear once; every nondefault
index flag and every effective-byte difference needs that exact declaration.
No AGENTS name, project feature, model or outcome is recognized by the helper.
Tests may run the pinned benchmark policy installer on a new fixture to prove
that the real documented overlay is representable.

Input and evidence roots are trusted, canonical and exclusively owned. Symlinks,
external hardlinks, linked Git state, submodules, conversion/filter configuration,
untracked/ignored input and unresolved source bounds remain unsupported. Only
public repository objects and explicit overlay data are retained; no global Cargo,
provider configuration or credentials are copied. Existing materializer/executor
limitations continue to apply.

`materialize_selected` preserves the earlier materializer's fields: its `shared`
source is the owned accepted clone, not the original live checkout. A separate
`selected_origin` identifies the original live path and immutable selection
receipt; it explicitly does not establish same-path execution or install overlay
bytes. The future caller must separately verify the original live view before
using it in an executor mount. Existing `execution_spec` otherwise uses the owned
accepted clone's path. Original-source census, privately installed overlays and
ordinary later shared patch acceptance remain distinct identities.

Required new RED→GREEN cases include exact actual benchmark overlay, an arbitrary
other filename, undeclared hidden edits/flags, wrong blob/byte/mode/index identity,
duplicate/unused overlays, source or index drift, dirty/untracked inputs, bundle
tampering/alias/unsafe destination, later HEAD advancement, and actual same-table
patch exclusion/release. The full runtime still requires its separate source,
default/legacy identity and real-agent gates.
