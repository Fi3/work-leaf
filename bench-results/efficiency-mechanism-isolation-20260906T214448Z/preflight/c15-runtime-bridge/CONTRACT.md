# C15 private runtime bridge

The bridge is a benchmark-private adapter over the exact qualified executor,
materializer, project-input helper, and live selector. It captures accepted Git
objects plus a declared effective overlay, applies a held test only in an owned
preview, and returns factual confined command feedback. It neither labels a
failure as semantic RED nor promotes files, commits, ownership, or ACKs. The live
ordering gate requires a delivered test preview before ordinary shared patch
submission. Private implementation replay is qualification-only, never that gate.

## Transport and identities

The Rust owner invokes a pinned canonical Python executable with `-I -B` and a
static exact-source bootstrap. Before `exec(compile(...))`, the bootstrap sets
`__file__` to the admitted bridge path and `__compiled_sha256__` to the SHA-256 of
the exact compiled bytes. It then calls `main([input_path, output_path])`. The
bridge compares that executing-source digest with its current file and retains
the source pin. A standalone direct script call derives its initial source hash;
the admitted runtime uses the stronger explicit compiled-byte binding.

Input and output are distinct direct children of the canonical operation root.
Output publication is create-new. Each operation creates its own attempt marker
before selection, materialization, or execution; an existing attempt/result forbids
a rerun. A partial publication or failed send does not authorize another command.
The trusted single caller owns the directory. This is not atomic multiwriter,
crash-recovery, or hostile-host storage. Earlier evidence is never deleted.

Input schema is `work-leaf-private-preview-bridge-v1`. Every request has exactly:

```text
schema, operation, config_path, config_sha256, project_root, operation_root
```

`validate` has only those fields. It checks source/config/public-input identities,
publishes an admission receipt, and selects or executes no project source. Its
operation directory may already contain the input file.

`capture` and `test` additionally require:

```text
run_id, agent_id, launch_generation, proposal_id, revision_of,
proposal, cancel_path
```

Run/proposal/revision IDs use `[A-Za-z0-9_.-]{1,80}`. `revision_of` is null or a
different prior identifier. `agent_id` is the already canonical owned `AgentId`
string, without whitespace/control characters; no role is inferred from it.
`launch_generation` is a positive integer, not a boolean. Rust owns enrollment,
revision history, normal command/lock admission, and the exact actual role.

`proposal` has exactly `format` (`edit` or `patch`), `reason`, `body`,
`test_purpose`, `test_paths`, `command`, and `lock_paths`. `test_paths` is a nonempty
list of unique normalized relative paths. The model supplies no byte offsets and
does not duplicate its test text in metadata. The host retains exact complete
after-images, byte lengths, Git modes and SHA-256 for these declared paths after
the actual held patch. These are file identities, not semantic assertion boundaries
or proof that a mixed production/test file is test-only. Later held/final-source
mapping must preserve ambiguous co-located changes instead of equating a changed
whole-file hash with a changed test. The ordinary held body and command are not
parsed into a replacement implementation.

`test` also has `selection: {path,sha256}`, referencing the successful
`CAPTURE-RESULT.json` in the same operation root. Capture/test retain the exact
same owner, generation, proposal, config and project identity. Their source
selection is not repeated. Rust calls `capture` synchronously inside its actual
shared `FileLockTable` `.` read-lock closure, then releases that lock before `test`.
Python records that it cannot prove the caller's in-process lock by itself.

Each operation writes `<OP>-ATTEMPT.json` and `<OP>-RESULT.json`. Successful capture
also creates `capture/SELECTED.json` and `capture/accepted.bundle`. Test materializes
under `test/accepted` and `test/preview`; unchanged predecessor artifacts remain
inside those roots. The complete command result is also `TEST-EXECUTION.json`.
The outer response contains `status`, `closed`, `exit_code`, `stop_reason`,
`stdout`, `stderr`, `result_path`, `result_sha256`, and applicable run/agent/
generation/proposal IDs. Capture returns its result reference again as `selection`.

`main` exits 0 for a published `completed` result, including an actual nonzero test
exit; 1 for a published `failed` result; 2 for an input/admission/publication error.
The runtime must inspect the exact returned receipt, not treat exit 0 alone as a
passing test. A failed result is factual evidence, not a qualifying delivered test
execution. Uncertain process closure is explicitly false, never assumed from a
Python exception. Complete output is retained; Rust owns its prospectively fixed
ordinary output compaction for the model-facing continuation.

## Pinned configuration

Config schema is `c15-runtime-bridge-config-v1`, with exactly these fields:

```text
helpers: {
  executor: {path,sha256}, materializer: {path,sha256},
  project_inputs: {path,sha256}, live_selection: {path,sha256}
}
executables: {
  git: {path,sha256}, bwrap: {path,sha256}, patch_driver: {path,sha256},
  toolchain_cargo: {path,sha256}, toolchain_rustc: {path,sha256}
}
toolchain_root: /canonical/public/installed/toolchain
cargo_capsule: {path: /canonical/public/capsule/CAPSULE.json, sha256}
overlays: [complete unchanged live_selection declarations]
timeout_seconds: positive finite number <= 300
operation_timeout_seconds: positive finite number <= 900
max_output_bytes: positive integer <= 16777216
qualification_implementation: false
```

The four helper hashes are the constants in `HELPER_FILES`; their relative
dependency layout must agree. Modules execute captured, verified source bytes,
not cached bytecode. Git and bubblewrap identities must match the qualified
predecessors. Cargo and rustc must be the pinned `bin/cargo` and `bin/rustc` under
the declared toolchain root. Trusted executables may have packaging hardlinks
(Cargo's `target/debug` and `deps` entries share an inode); their hashes bracket
execution. Source, config, capsule, and receipt files remain independent regular
files without symlink or hardlink aliases.

The existing public capsule at
`../c15-project-qualification/PROJECT-001/public-cargo/CAPSULE.json` is reusable by
exact digest, including for a dependency-free fixture. The bridge copies no Cargo
home or registry directory, fetches nothing, and validates every declared capsule
file plus the absence of undeclared files/directories. Missing archives remain
the capsule's original explicit limitation. Cargo/rustc are pinned, but the entire
installed public toolchain/system-library distribution remains a trusted input,
not a complete file-by-file attestation or proof of arbitrary command dependencies.

A tiny Rust fixture can commit its own minimal package, source, and lockfile;
use `overlays: []` if its actual tracked/index census is clean. Agent-selected
`command: "cargo test --offline --locked --test <declared-target>"` reaches exactly:

```text
/usr/bin/env RUSTC=/toolchain/bin/rustc PATH=/toolchain/bin:/usr/bin:/bin \
  /bin/sh -c <unchanged command>
```

Only the explicit public toolchain/capsule are additional read-only mounts.
The executor supplies private HOME, Cargo home, build and scratch, fixed locale,
cleared environment and isolated network/process/filesystem namespaces. These are
real environment/feedback costs, not ordinary-host equivalence or pure timing.

## Selected source, overlays and original view

`live_selection.capture_selection` performs its complete sampled census and
immutable bundle capture once. `materialize_selected` restores those pinned
objects even if normal shared HEAD advances afterward. The original materializer
fields are preserved: `snapshot.shared.root` remains the **owned accepted clone**;
`snapshot.selected_origin.live_root` names the **original project**. The bridge
checks both identities against the selected receipt/census before setting the
executor's view to that original canonical project path. It never relabels the
owned accepted root as the live source or clones a later live HEAD.

Scoped bindings on separately exact-byte-loaded modules connect the predecessors:
`M.execution_spec` preserves the original specification except that validated view;
`M.executor` supplies the same qualified executor with cancellation/budget;
`L.M`/`L.git` and `P.materializer` use that owned module instance. No old helper file
or patch interpreter is changed. These adapter bindings are new qualified bridge
behavior, not a claim of entirely unchanged execution semantics.

Overlay installation uses `P.install_private_overlays`. Complete L declarations,
including both index-flag booleans, remain in the bridge receipt. The older P
interface receives one **actually true** flag as an explicitly nonexhaustive
projection. Both-true is supported and recorded; a nonempty overlay with neither
flag true is unsupported before capture. Empty overlays need no installation.
The final private file inventory must exactly equal the selected census's effective
images. No live flag is cleared or source file replaced. The neutral private Git
feature label is `Private test preview`; actual author feature provenance belongs
to the Rust outer trace. Private metadata commits are never shared acceptance.

## Failure, cancellation, bounds and qualification

The exact owned absent cancel-file path is polled into the executor's Event.
Checks between Git operations and a bounded executor timeout enforce a cooperative
operation budget. Each existing Git child has its own 30-second bound; a child
already in progress can take that long to return after cancellation. Kernel I/O,
trusted hashing, and postcreation bundle storage are not hard wall-clock/disk
bounds. The caller must preserve roots and report uncertainty if its outer
supervision cannot establish closure; there is no unconfined fallback or cleanup
of uncertain trees. Source pins are checked after successful and failed operations,
with endpoint errors retained separately from the original failure.

`execute(request, qualification=True)` is a provider-free qualification API with a
separately pinned config where `qualification_implementation` is true. Runtime
`main` rejects that mode and rejects `implementation`. Qualification implementation
requires `prior_test: {path,sha256}` for the same successfully executed test result plus
`implementation: {format,reason,body}`; it reuses that exact private image/held test,
verifies unchanged complete declared file images, applies only the supplied private
implementation, and repeats the original command. Its qualification fixture uses
a separate test-only file; a co-located production edit cannot be called an
unchanged-test proof from whole-file equality alone. It is not a live operation, rerun authorization,
or gate requiring an implementation preview before normal shared submission.

Source work uses constant byte passes per proposal, one read per declared file
and keyed/sorted inventories. Repeated proposals cost O(P × B); existing Git/history
operations remain additional. The predecessor's at-most-32-mount O(M²) check remains
explicit. File bounds, output caps and bundle postcreation limits retain their
original qualifications. None proves general peak memory, hostile-host containment,
continuous live-source immutability or the semantics of a failed test.

Bridge automatic/provider-free tests qualify this adapter only. Root Rust gates,
actual lock/role/protocol/delivery integration, exact frozen-project Cargo execution,
and a separately admitted real-agent workflow remain distinct readiness evidence.
