# Declared derived-output selector qualification

This is a provider-free selector review cut, not a bridge, runtime or actual-agent
qualification. The original v1 selector and bridge, completed C15 runs, captures,
outcomes and ordinary working trees remain immutable. No Cargo, private command
executor, provider, accounting or admission operation belongs to this receipt.

## Source identities

| File | SHA-256 |
| --- | --- |
| `live_selection.py` | `ddfa7b6eb99bf713669caf5d3766aaa11843b24057b86329fd6985b83733dd5f` |
| `test_selection.py` | `9adae3746498dec664e107f08a7e81c5fcb668a9a3917ed9367f86dc724aaf5d` |
| Original `../c15-live-selection/live_selection.py` | `0614db9a5c874cd5f623997ce3433edbd2bc0ff3585c23913dd9ede9705d3a55` |
| Original `../c15-runtime-bridge/bridge.py` | `6af6c3f24f96a7b95514a0db702a775b895a8d89f5dc8a2db9aa1329db9d4d26` |

The new selector begins with an exact predecessor copy made through `apply_patch`.
Its explicit classification and source-comparison branches are local code, not
forged Git output, an alternate index, cleanup or production monkey patching.
It retains the exact qualified materializer and Git executable pins. The tests'
bounded Git hooks perform real local commands before injecting actual output
growth or source/admin drift; they do not fabricate Git classifications.

## Observed test sequence

The command is `python3 -B -m unittest -q test_selection.py` from this directory.
All temporary Git repositories belong to the automatic fixtures and are independent
of admitted workflow checkouts. No test executes Cargo, bubblewrap or a provider.

| Gate | Actual result |
| --- | --- |
| Root initial absent-module RED, tool `34a02b` | 10 tests, original strict reproduction passes; 19 missing-module errors |
| Independent initial RED, tool `d56b2b` | Same 10 tests / 19 missing-module errors; original ignored-output rejection passes, 0.604 s |
| Ignore-provenance interface RED, tools `160de5` / `eb1893` | Missing keyword contract; actual untracked rule source separately verified after correcting the fixture's path to a directly matched leaf |
| Initial intended contract GREEN, tool `0633f0` | 16 passed, 4.399 s |
| Within-census ignore-admin drift RED, tool `ef6ac3` | Expected rejection was absent |
| Rule-authority snapshot GREEN, tool `fa0874` | 17 passed, 3.907 s |
| Unused untracked `.gitignore` RED, tool `e1a651` | Expected administration rejection was absent |
| Final selector cut GREEN, tool `64bc02` | **18 passed**, 3.546 s |

The new 18-test population covers actual v1 strict rejection, unchanged default
census, explicit ignored-only opt-in, unknown/nonignored/negated input, tracked
descendants, unsafe/duplicate/overlapping roots, symlinked roots/ancestors and
ignored file aliases, output growth, actual accepted-source/admin drift, absence
of derived files from real private materialization, exact ignore-rule provenance,
external policy rejection and untracked ignore administration. Existing tests
and original helper bytes are unchanged; these are not actual-command or
agent-facing verification results.

## API and source contract

`census(root, expected_commit, overlays, *, derived_output_roots=None)` preserves
the exact original returned census and strict rejection when the keyword is absent.
An explicit list opts into the separate classification. Empty lists permit no
ignored output. There is no default inferred build directory.

`capture_selection(..., derived_output_roots=[...])` keeps accepted-tree/index/
overlay/administrative equality and immutable Git bundle creation. Its new schema
is `c15-caller-owned-selected-source-v2`. `live_census` retains the first complete
sample; `derived_output_endpoints.before/after` separately retain roots, actual
ignored paths, size/mtime metadata and per-path typed ignore-rule witnesses.
Only the `derived_outputs` field is excluded from the accepted-source equality
comparison. Body growth, creation or removal may differ; every ordinary source
or administration difference still fails. This is sampled endpoint evidence,
not hostile-concurrency exclusion or continuous atomicity.

Each excluded leaf must be under a declared directory, independently regular and
reported as ignored by actual Git. The separate verbose NUL-delimited witness
must name an accepted/overlay-tracked `.gitignore` or `.git/info/exclude`; negated
patterns cannot establish ignored status. Tracked rule bytes, config and exclude
identities are sampled around classification and again at census completion.
Local `core.excludesFile` and every untracked `.gitignore` are unsupported rule
administration. No accepted excluded body is read or copied by the selector.

`materialize_selected` accepts only the new selected schema and restores the
unchanged pinned bundle through the qualified materializer. The materializer's
`shared.root` remains the owned accepted clone; the separate `selected_origin`
still identifies the original live root. Derived outputs never enter that bundle
or private image and are not removed or rewritten in the original project.

## Bounds, costs and remaining gates

The explicit declaration bound is 64 roots. Derived/protected paths are bounded
to 64 components and 4096 UTF-8 bytes; the classified untracked population is at
most 100,000 paths. Parent membership uses sets and bounded ancestor walks, not
pairwise file/history scans. Added work is O((F+E) × D), plus sorted inventories
and actual Git traversal/rule work, where D ≤ 64; no O(E²) implementation is
introduced. Rule-source hashing is a constant number of additional byte passes.
The preserved 256-MiB source and postcreation 512-MiB bundle qualifications still
apply. Git command output is checked after collection, not a general peak-memory
bound; each inherited Git child remains individually bounded to 30 seconds.

The result's selector hash records its source file at publication. An admitted
caller must still execute exact pinned source bytes and verify all dependency
endpoints; this selector receipt alone is not executable admission provenance.

A v2 bridge/configuration, exact declaration/helper/selection binding, original
live-view integration and post-check private-command qualification remain pending.
Bridge integration is deliberately stopped for coordinated review of the future
private-preview package, including its observer parser/grace contract. No old
failed preview is corrected or replayed, and no modified workflow is admitted by
this provider-free selector result.
