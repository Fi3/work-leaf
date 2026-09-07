# Independent materialization review

Recorded 2026-09-07 16:53 UTC. Verdict: no blocking finding in this generic,
provider-free prerequisite. Full C15 runtime and current-project admission remain
unqualified, as stated in [QUALIFICATION.md](QUALIFICATION.md).

The complete Python helper/tests, standalone Rust driver/Cargo target, detailed
source contract and qualification note were inspected. The driver reaches the
existing `GitPatcher::apply` / `apply_edit` and normal private locking/commit paths;
there is no alternate edit interpreter or shared-tree promotion.

## Independent verification

From this directory, `python -B -m unittest -v test_materialize.py` passed all 12
tests in 6.005 seconds using the already-built, exact-pinned private driver. No
root binary was rebuilt and no provider was launched. All 33 source/build entries
in [QUALIFICATION-001.json](QUALIFICATION-001.json) match their retained hashes.
Its four actual closed executions retain exit codes `0, 101, 0, 0`, with no stop
reason. The same test command and held test bytes produce behavioral RED followed
by GREEN after only the private implementation proposal.

`source_identity` checks actual tracked bytes against the accepted Git objects,
Git executable modes, index entries, hidden index flags, administrative hashes
and untracked/ignored input. The tests explicitly reject both clean and modified
skip-worktree/assume-unchanged cases before creating the private repository. They
also cover staged/dirty state, symlink/hardlink/submodule/filter inputs and source
drift. A clean status alone is not accepted as source equivalence.

`materialize` uses a disjoint create-new claim and independent no-hardlink local
clone, with a private index/object store rather than linked worktree metadata.
The selected commit/tree and actual copied tracked content are checked. Later
accepted shared changes do not rebase the held private proposal. The current
shared identity is separately bracketed and any observed drift remains failure.

`apply_proposal` preserves exact proposal bytes and declaration before execution,
uses the previously qualified namespace executor, and retains execution before
validating an unsupported after-image. The symlink after-image test demonstrates
that private effects and execution evidence survive qualification failure.
Duplicate proposal IDs and pre-execution publication failures do not apply work.
No result is a shared acceptance or an instruction to promote private Git state.

## Retained boundaries

This is an accepted-Git-tree contract, including Git executable modes, not a
complete clone of arbitrary filesystem permissions, ACLs, external configuration
or generated inputs. Local/global/provider configuration is not silently copied.
Tracked history and explicit tool mounts must be public and trusted. Unsupported
local inputs, especially the current benchmark's hidden instruction overlay,
remain outside admission. The dependency-free fixture and read-only toolchain
mount do not prove current-project dependency/cache/environment equivalence.

The standalone helper cannot hold a live daemon's shared `FileLockTable`.
`in_process_wl_lock_proved: false` remains correct. Source checks are endpoint
checks, not continuous immutable-host attestation; no shared lock is held during
the private check. Test-purpose ranges and later shared conflict/acceptance flow
remain future protocol responsibilities, not properties inferred from test names
or exit codes.

No new quadratic file join is present. Constant full-source passes plus changed
path sorting cost O(B + F log F), excluding existing Git operations. Repeated
proposals repeat source validation; Git history cloning/metadata are not covered
by the current-tree byte bound. The executor's previously reviewed bounded
O(M²) mount validation remains explicitly qualified, with at most 32 extra mounts.

The architecture and public APIs are unchanged. The local qualification note
describes the relevant private operating constraints; no production architecture
or operator-documentation edit is necessary. No agent-facing workflow is affected
by this offline prerequisite, so real-agent verification is a later integration
gate, not a missing provider run for these generic fixtures. Tracked runtime,
existing tests, Cargo files and architecture documentation had no worktree diff at
review closure; concurrent study-note work is not claimed immutable.

## Exact reviewed identities

| File | SHA-256 |
| --- | --- |
| `materialize.py` | `87546793b76bc7324af55479be47a1e34abbeffb8f2787547d021575a51efab2` |
| `test_materialize.py` | `a0e8f4ccf67e2fd82b976acb5194e7c34500c5083a8f92e4524b1a0632b1fba1` |
| `patch_driver.rs` | `f5560b0f4eb00b916f7b5a3b1434f5d8841898d2e8e7a0cc8fe8c3c60a2fe55b` |
| `Cargo.toml` | `948215227ca02fb3155ed9a3fa39367126282bd34ea7094f826f1b2d5b787b28` |
| `target/debug/c15-patch-driver` | `25005c324307383ad8dbe218f0ed08166e47afc42dfe0fb8ab86a2cd3a539c7c` |
| `QUALIFICATION.md` | `6d4da3d9ac68896673d0ebf2436ea1758431f318d5a432f6df04ea567d561f51` |
| `QUALIFICATION-001.json` | `f808f0ad9ac738d63c737f1a6de57bb2c38bbe591569c1b2985f4c8a9ba8d6f4` |

No reviewed source, test or original receipt was edited by this review.
