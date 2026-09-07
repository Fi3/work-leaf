# Independent C15 bridge review

Verdict: no blocking finding in this private provider-free adapter cut. The
review covers the complete bridge, tests, [contract](CONTRACT.md), and
[validation note](VALIDATION.md), not the concurrent Rust integration or an
agent-facing readiness decision. No implementation, existing test, predecessor
helper, saved workflow, or provider was changed or invoked by this review.

## Exact reviewed cut and independent execution

| File | SHA-256 |
| --- | --- |
| `bridge.py` | `6af6c3f24f96a7b95514a0db702a775b895a8d89f5dc8a2db9aa1329db9d4d26` |
| `test_bridge.py` | `2a604d681b5bde70ad76b077b556a7577af7995524daff3887ccc4e0349c3f69` |
| `CONTRACT.md` | `4a4c109da4624ce8238472961e455c95ad50178b4ecb02e0fc25aae6092c56c0` |
| `VALIDATION.md` | `daf7bf6fd31d7cf42a00d50ee588bf9bb44324509947eeb991980f30162067a4` |

From this directory, `python -B -m unittest -v test_bridge.py` independently
passed 21 automatic tests in 12.599 seconds; the explicit once-only frozen-project
Cargo test was skipped. Its discovery count is 22, not 22 executed cases.
The suite exercises actual private Git/patch/sandbox paths, a shell-test failure
followed by unchanged-test qualification GREEN, later live HEAD advancement,
both hidden index flags, cancellation, exact-source isolated Python transport,
and negative source/owner/path/publication cases. Failure-injection cases remain
synthetic, not claims that an actual child escaped or a real timeout occurred.

The owner's final preflight regression demonstrated that a dangling internal
result path could otherwise allow private work before publication failed.
Reserved attempt/result/execution paths reject existing entries or symlinks
before the attempt. The independent final suite includes that regression.
Reverse-delta hashes establish that this guard and its single test are the only
source/test differences from the previously fully reviewed `3b87240a…` /
`37c37846…` cut, whose independent suite passed 20 cases with one skip.

The four exact predecessor helper hashes were rechecked after the suite:
executor `16e3238e…`, materializer `87546793…`, project inputs `b2f2905b…`, and
live selection `0614db9a…`. All match `HELPER_FILES`. Their complete qualification
and independent-review notes, the initial/detailed test-first designs, the prior
runtime-boundary review, architecture, and operator policy were read. Introduced
bindings were checked against the actual reused helper call paths.

## Source, transport and process boundaries

`validate_request` admits only the explicit runtime `validate`, `capture`, and
`test` operations. Canonical owner/generation/proposal fields and exact proposal
shape are retained. `test_paths` requires unique normalized paths; the model
supplies neither byte offsets nor a duplicate test-text field. `declared_images`
derives the actual complete private after-images, modes, byte lengths and hashes.
Those images do not certify semantic test boundaries or test-only purpose.

`validate_config` exact-compiles pinned helper bytes, checks dependency layout,
executables, declared public capsule files and directory population, overlays,
and disjoint protected inputs. The bridge's executing-source hash must match its
current file; the actual empty-environment `-I -B` bootstrap test exercises that
binding. Public system/toolchain distributions remain trusted dependencies,
not completely attested file inventories. No user/provider configuration or
credential tree is discovered or copied.

`validate_prior` binds test execution to the exact completed same-owner capture
receipt in the operation root. `live_selection.materialize_selected` uses the
pinned captured objects, not a later live HEAD. `bind_modules` preserves
`snapshot.shared.root` as the owned accepted clone and verifies the separate
`selected_origin` before setting only the executor view to the original live
path. Effective private files must equal the selected live census. Full overlay
flags survive separately from the older installer's explicitly nonexhaustive
single-flag argument. No live flag or live file is rewritten.

The patch driver applies the held proposal only in the independent private
repository through the existing `GitPatcher` semantics. The ordinary source,
ownership, read tracker and ACK path do not participate. Qualification-only
implementation requires the prior private test and identical declared test-file
images; runtime `main` rejects that operation and the qualification config mode.
This does not create a mandatory second private implementation preview.

`Budget` connects the exact owned cancel path to the qualified executor Event.
The scoped executor wrapper requires proven closure and retains uncertainty when
an exception occurs with execution pending. Git calls have their existing
30-second child bounds; budget checks occur between those calls. No hard bound
on arbitrary filesystem I/O is claimed. Original operational errors and source
endpoint errors remain separate. The complete command receipt is published
before later private-image validation, so a failed after-image check does not
erase executed work.

Create-new attempt/result publication forbids a transparent same-operation retry.
The trusted single caller owns publication paths; this is not crash-proof storage
or a concurrent-writer protocol. Post-execution publication failures can leave
partial retained artifacts and no successful outer response. The bridge neither
deletes uncertain trees nor falls back to ordinary unconfined commands.

## Remaining qualification and documentation limits

The private adapter is coherent with the documented core owners; its own
contract/validation describe its operating boundary. It does not independently
prove that Rust held the actual shared `FileLockTable`, enrolled the correct
author generation, preserved non-target prompts, or delivered an accepted native
input. Those integration checks and the prospectively documented architecture
remain the runtime owner's separate responsibility.

The optional frozen-project Cargo test was inspected but not executed here.
Its expected first GREEN qualifies execution feasibility, not a behavioral RED
or a timing effect. Private source/build/cache state, cleared environment,
snapshot preparation and result transport are real parts of this package; one
successful target cannot establish ordinary-host or all-command equivalence.
Actual agent-facing verification remains a separate required admission/gate,
not a green result supplied by this offline review.

No new all-history or all-file pairwise join was found. Source work uses constant
full byte passes and keyed/sorted inventories per proposal, with O(P × B) repeated
proposal work and additional Git history operations. Repository-required
complexity flag: the reused executor has bounded O(M²) mount checks, with at most
32 extra mounts. Output/metadata limits and postcreation bundle admission do not
establish a general peak-memory, disk-quota, or hostile-host guarantee.

No costs, new charge extraction, causal percentages, controls, or provider calls
were read or produced by this review.
