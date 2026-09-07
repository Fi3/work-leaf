# C15 live selection qualification

The provider-free prerequisite passes its bounded source/lock/object checks.
It does not authorize a runtime preview, benchmark, strict-timing intervention,
provider launch or promotion of private work. The user-facing choice between a
private workflow and strict test timing remains separate.

## Source identities

| Source | SHA-256 |
| --- | --- |
| `live_selection.py` | `0614db9a5c874cd5f623997ce3433edbd2bc0ff3585c23913dd9ede9705d3a55` |
| `test_live_selection.py` | `f2998b3e8724317486b8622efb8182313e19532714a2429006395de8f3ed59d2` |
| `lock_selection.rs` | `6af0713734f822fd41cbeca0008371c98812d519b194955d68c802dd8a5b49ab` |
| `target/debug/c15-lock-selection` | `0894c041e5b8d949a368347c58c26fb727e5abc6a3f3b9b1f0c726f1a840d4ea` |
| `Cargo.toml` | `1d489f72d261498a5e6dd0f72413d1a0a6fe44dc8ce23fa82aa924c1f82c45a5` |
| `Cargo.lock` | `c1ff25fefe936c308714ee8dd7de872cbdf4098e3200d737a7bf5efe3fec2ba4` |
| `DESIGN.md` | `e56863a887575ce5dab64277766a225f372d96b533c635e102d76ebdbeef311d` |
| `QUALIFICATION-001.json` | `b99dde0702b6e9d5153707bc250ca5a56a493a2b934c3e02e5f6f4e3417a890f` |

The JSON receipt pins 37 source/build files and 14 actual evidence files. It
includes the existing materializer `87546793…`, all current Rust source files,
the actual caller executable, architecture, detailed C15 design and frozen
benchmark installer. All endpoints were rehashed after materialization. Earlier
executor, materializer, project-cache/overlay helpers and their outputs are
unchanged. No current runtime source or public interface is modified.

## RED and automatic gates

Initial discovery failed because `live_selection` did not exist (tool receipt
`d65791`). The initial real-lock test subsequently failed because its caller
executable did not exist (`ee4627`). A separate pre-fix test demonstrated that
materialization could write beneath the live source (`902eca`, one failed test).
The explicit timeout, endpoint bundle-size and original-view metadata tests
failed before their guards (`900f10`, three errors): no timeout failure receipt,
missing size field and missing selected-origin record. Their focused GREEN was
four tests (`2020b4`).

The initial broad fixture also exposed a test-cleanup mistake: combining two
`git update-index --no-*` options left one flag set. The new fixture clears them
with separate commands; this was not a source-census defect. No committed test
was changed. Final automatic command, from the repository root:

```sh
python -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-live-selection -p test_live_selection.py -v
```

Result: **14 PASS**, 3.131 seconds (`a0c0e9`/`502bff`). Cases cover the actual
installer overlay, generic alternate filename, hidden flags and dirty bytes,
staging/modes/untracked/ignored input, malformed/duplicate declarations, aliases,
hardlinks, Git-directory indirection, filter/attribute/symlink/submodule rejection,
selection drift, bundle/receipt tampering, unsafe destinations, Git timeout,
postcreation size rejection, later live HEAD advancement, and real lock success
and failure release. Failure fixtures inspect retained `failure.json` and
`CALLER.json`; test-owned temporary directories are cleaned after assertions.

Using this directory's `Cargo.toml`, `cargo fmt`, `cargo build --offline`,
`cargo clippy --offline --all-targets --all-features -- -D warnings`, and
`cargo test --offline --all-targets --all-features` all exit 0. The Rust test
target contains zero standalone unit tests; the 14 Python tests execute the
real caller and Git operations. Existing root Rust gates are not represented as
a fresh run here. No agent-facing behavior changes, so no real-agent workflow
is affected or invoked by this qualification.

## Retained actual project selection

`ACTUAL-001` is a new owned clone of frozen benchmark base
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`, tree
`acb6583fbb0e25347f70ee44cbdacafd6df95e37` (47 tracked files). The exact frozen
`bench-agent-profile-common::bench_install_no_recursive_agent_policy` function
(lines 73–99, SHA `de980365…`) produced the declared effective instruction text
and skip-worktree flag there. The generic census matches accepted blob/mode,
actual effective body/mode and index flags; it recognizes no instruction filename.

The real caller shares one `FileLockTable` with an actual `GitPatcher`. While
holding `with_read_locks(["."])`, it captures the selected bundle; the writer
remains pending, then adds a declared fixture witness after release. The selected
bundle is **287,837 bytes**, SHA `ec1c907aa703d68e19b655c3389d5c6ad66398a4a314b945cb738c2a351cdc73`.
The recorded root-lock duration is **0.269297205 s**; selection operation duration
is **0.177350779005792 s**. These are one qualification execution's timings, not
benchmark performance estimates or a worst-case bound.

The live HEAD then equals `6bde788fa9d701c20e7639be6fdebbedf75c9f12`. Materialization
still restores the earlier selected tree from its pinned bundle, without the
later witness. Complete live census endpoints before/after materialization match.
The effective live instruction body remains intact. The materialized image has
the original accepted instruction body: this receipt **does not install overlays**.

## Caller contract and remaining limits

`live_selection.py::census` (74) checks actual index flags and bytes, not `status`
cleanliness. `capture_selection` (152) belongs synchronously inside the **actual**
orchestrator's existing shared-table `.` read-lock closure, with `locks.root()`
bound to the selected canonical live root. `src/patch.rs::patch_lock_paths` (382)
includes the same root key; `src/locks.rs::locks_for` (105) keys exact normalized
paths. A new lock table or OS lock file cannot establish daemon serialization.
The standalone caller proves only its fixture's table and cooperating writer.

The caller must release the lock before `materialize_selected` (190), overlay
installation, proposal execution, build/checks or agent sends. The immutable
bundle prevents live HEAD substitution; ordinary later patch application remains
authoritative. No retries, fake ACK, promotion or lock-held test are implicit.

The old materializer's `shared.root` remains the **owned accepted clone**, and
`selected_origin.live_root` separately names the original checkout. Its existing
executor specification therefore defaults to the owned clone's path. A future
same-path execution route must verify the original view independently; it cannot
rename those provenance fields. Installation of declared overlays remains a
separate qualified operation, including explicit handling of combined flags.

The `.` lock does not exclude ordinary commands on other keys or external writes.
Census endpoints do not prove atomic live-tree equality. Inputs and publication
directories are trusted/exclusively owned; conversion filters, external source
dependencies and untracked inputs remain unsupported. No private configuration,
credentials or broad Cargo/home snapshot is copied. Public reachable Git history
is copied, so history can exceed selected tree size. The **512 MiB bundle bound
is postcreation admission only**; each Git child has 30 seconds, not a whole
operation deadline. Failures retain size/duration but do not promise recovery
from process death, disk exhaustion or inability to write the failure receipt.

The introduced census uses indexed path/blob maps and one batched blob read:
linear file/body work plus path-resolution work, JSON key sorting and Git's own
history traversal/packing. It has no per-file full-tree rescan. The frozen
materializer/executor retain their earlier declared complexity and limits.
Architecture and detailed design were read fully; this private qualification
does not require a public API or architecture-document change. Runtime wiring,
same-path overlay execution and the final C15 behavioral contract remain outside
this prerequisite's go decision.
