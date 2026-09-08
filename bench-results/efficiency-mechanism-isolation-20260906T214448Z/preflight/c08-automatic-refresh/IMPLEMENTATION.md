# C08 detached implementation qualification

Status: implementation/source review in progress; not ready for provider admission.
All further Cargo/build checks are held during the root-owned three-workflow C15 wave.
No C08 provider, private executor, benchmark, extraction or accounting call is authorized
or performed by this implementation task.

## Source and ownership

The detached worktree is `/tmp/work-leaf-c08.04mP2m/repo`, rooted at
`b8e928153674549e4877b5ae6b34546d8541d566`. The root checkout's frozen C15 runtime,
bridge, driver and launch artifacts are not edited, restored or merged. Existing
committed tests are unchanged; all C08 tests are new files. No public API changes.

`src/orchestrator.rs` owns the automatic recovery rendering and held snapshot body.
`src/bench_automatic_refresh.rs`, compiled only under `bench-experiments`, owns ordered
candidate construction and evidence. `src/bench_experiment.rs` admits the one private
v7 condition and keeps ordinary continuation bytes unchanged. `docs/architecture.md`
describes that resulting private boundary. No provider, policy, patch, read tracker,
bundle implementation, command execution or diff helper is replaced.

The v7 manifest has only `schema`, `run_id`, `condition`, `evidence_path`, using schema
`work-leaf-bench-experiment-v7` and condition `automatic-changed-refresh-full`.
The existing strict manifest parser and environment/run identity requirements apply.
The v5 archive and v6 private-preview descriptors are not v7 fields.

## Delivery and evidence contract

Only an actual automatic patch/edit refresh with a tracked changed snapshot and a
nonempty ordinary diff of at most 48 KiB has a full-current replacement. The held text
has no new size gate. Renderer-owned wrapper/intro coherence changes accompany a body
replacement; otherwise both complete candidates are identical. Untracked text retains
the ordinary 8-KiB bound. Empty, unavailable and oversized diff results remain distinct.
Already-applied, no-file and nonstale rejection paths do not become refresh occurrences.

Each active rendered refresh records `event: automatic-refresh`, its v7 schema,
`site: automatic-patch-refresh|automatic-edit-refresh`, run/agent/process/sequence/time,
complete `original_prompt` and `candidate_prompt`, and `selected_candidate`.
`components` contain owned original/candidate byte ranges, component ID, optional
snapshot index and exact candidate body range excluding the formatter newline.
`metadata` retains patch/edit kind, original diagnostic/files/failures and ordered
snapshot classes, held/prior FNV64 digests and byte lengths, normal diff disposition,
baseline section bounds and available diff-body bounds. Source text is never searched
for these boundaries. Cryptographic source/native-input joins remain offline work.

The ordinary read and per-changed-file diff calls remain single. Snapshot advancement
still precedes delivery; the evidence/selection call follows that same advancement.
Evidence failure prevents the send without rolling tracking back. A failed provider
send also retains the ordinary advanced snapshot. There is no fallback or extra turn.
Candidate assembly/metadata work is linear in held bytes and sections; no new quadratic
history scan is introduced. Full-candidate allocation/serialization and provider limits
remain real resource/failure outcomes, not a reason to clip or reissue the response.

## Observed development checks

| Check | Retained result |
| --- | --- |
| New public workflow test before activation | `cargo test --offline --test bench_automatic_refresh --features bench-experiments -- --nocapture`; tool `eee4e2`: 1 PASS, 1 FAIL, 1 subprocess ignored. The selected fixture fails `unsupported manifest schema`; legacy baseline comparison passes. |
| New private helper tests before implementation | Tool `7697ef`: missing `Capture`/v7 helper interface produces compilation failure. |
| Initial integrated recovery | Tool `6bda98`: 2 PASS, 0 FAIL, 1 subprocess ignored. Public `AgentOrchestrator` exercises actual Git rejection for both patch formats, exact current text above 8 KiB, normal repair/ACK and failed-send snapshot order. |
| Helper ownership/evidence | Tool `b62ff5`: 4 PASS, 0 FAIL; ordered UTF-8 spans, marker-like data, identity-only coherence, malformed ranges and selected/identity evidence-write failures. |
| Renderer and helper matrix | Tool `309ee9`: 7 PASS, 0 FAIL; normal 48-KiB boundary and over-bound, empty/unavailable diff, untracked 8-KiB cap, mixed sections/failures, same snapshots, exact held bodies and one diff callback per changed file. Rendering alone leaves tracking unchanged. |
| Formatting before the test hold | `cargo fmt`, tool `a281a1`, exit 0. Subsequent narrow metadata/test/doc edits still require the final formatting gate. |

The last running Cargo process closed before the root's C15 launch. After the hold,
baseline section/diff-body metadata and additional legacy-v5/nonstale/no-file/already-
applied test cases are source-only edits awaiting execution. They are not claimed to
have passed the earlier cut's checks.

## Remaining gates

After the root confirms all three C15 workflows terminal: run the complete new targets,
default-build/prior-schema identity checks, `cargo fmt`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-targets --all-features`. Independently review exact final source/test
identities and all default/prior-schema and error-boundary claims.

Real-agent verification is pending. The design permits at most one separately frozen
and admitted bounded same-author fixture with genuinely accepted intervening state,
real rejection/full-refresh delivery, actual repair/ACK/check/completion and complete
typed native-input membership. This controlled fixture is qualification, not natural
benchmark exposure. No such admission or provider call occurs here; fake providers
qualify protocol plumbing only. No full-workflow saving or readiness is claimed.
