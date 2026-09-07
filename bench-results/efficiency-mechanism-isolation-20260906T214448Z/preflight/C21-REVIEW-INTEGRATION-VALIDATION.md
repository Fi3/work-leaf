# C21 reviewer-context integration validation

The private integration follows [DESIGN-REVIEW-EVIDENCE-ON-DEMAND.md](../DESIGN-REVIEW-EVIDENCE-ON-DEMAND.md), SHA-256 `914244ea8f37cf1b12a95486f685ebe87146f73be973c903351afc57cb5f3877`. Automatic integration checks and independent adapter review pass. Full root gates and separately admitted real-agent verification remain prerequisites; this receipt does not admit generation or declare the actual native-read workflow verified.

## Source boundary

`src/cli.rs::CommandChat::review_commit_streaming_with_ids` obtains its existing optional backend session snapshot once and calls the unchanged `review.rs::render_review_source_context` once. The review prompt retains its exact prefix and suffix. Construction records the UTF-8 interval occupied by that held source-context string; the private `bench_experiment::forward_review_context` receives the interval before the ordinary new/reused reviewer branch. Adapter errors propagate as `CliError::Io` before the affected provider launch/send.

The source context is not searched for headings, footer text or file markers. Author fix and reviewer recheck construction are unchanged. No renderer, public API, provider, orchestrator read route, tracker, lock, bundle counter or observer is extended here. The interval variables and adapter invocation compile out without `bench-experiments`; older/inactive schemas retain their existing behavior. Added prompt construction is linear in held prompt bytes, with no per-message rescan or quadratic operation.

Source SHA-256:

- Prepatch `HEAD:src/cli.rs`: `e68988f004ac9a61d25c0565b41dd46a371ebcf62a2415c0aabc4d6d83a96763`.
- Integration `src/cli.rs`: `a87be2707fcb873d1cdfba319d9e6d1b07ab6aa7aacdf8e5aa6ff25f98ab1025`.
- New `tests/bench_review_evidence.rs`: `dc5b1d9e58d153049b29c42dae4bf7b38ef8dcde91faac0a615598ebfd8442fc`.
- Unchanged `src/review.rs`: `12d70548086d053583e1fc2fc6b4f191dd262cf61b45637a1ee5d04931c919ee`.

## RED-first evidence and automatic scope

Before the CLI hook or schema-5 adapter existed, `cargo test --test bench_review_evidence --features bench-experiments -- --nocapture` returned exit101: **1 passed, 2 failed, 1 ignored**. Both new-schema cases failed at the unsupported `review_evidence_root` manifest field before provider delivery. The independent old-formatter reference already passed eight whole-prompt comparisons: new and reused reviewer for missing, empty, populated and fix-loop source-session cases.

The reference copies the prepatch formatting contract, not the changed renderer's output. Its populated cases retain all four public message roles, Unicode, trimmed trailing whitespace and copied Agent-ID, User prompt, source-context and file/bundle markers. Each ordinary review takes exactly one source-session snapshot. A later commit/session mutation produces a new exact archive while the earlier payload and typed manifest remain unchanged after reviewer reuse and `CommandChat` drop.

Selected and nonselected schema-5 tests check both complete candidates, exact archived context, renderer-owned byte intervals, identical outside-span bytes, source/reviewer identities, distinct archive paths and actual selected forwarding. Inline selection preserves all complete provider deliveries. Default builds ignore even supplied schema-5 environment; inactive feature builds and versions 1–4 preserve the tested review/fix/recheck bytes and allocate no review evidence. The legacy-v4 fix fixture supplies a genuine successful owned author launch, as required by that existing adapter; backend-only transcript state is not substituted for launch provenance.

The same `CommandChat` reviewer interleaves review handoffs with ordinary large reads, changed and unchanged repeats, an explicit ordinary bundle-file read, `done` tracking clear and a subsequent fresh read. Baseline and both new-schema paths have identical ordinary bundle contents and paths `bundle-0.md`, `bundle-1.md`, `bundle-2.md`, with normal cleanup. Opaque-only and mixed small-project/opaque requests retain the ordinary unavailable-path result and do not disclose opaque evidence. The selected/inline cases use the actual emitted archive path. The inactive reference uses a fixture-owned external file containing the same bytes to exercise the identical pre-existing refusal route.

A publication-error case preserves the first archive under a renamed fixture directory, obstructs its original path, and verifies that the second review fails before any reused-reviewer send. Both selected and nonselected schema-5 conditions fail closed. Archive security, descriptor identity and trace-write failure tests belong to the separate private adapter owner; this receipt does not replace those checks.

Two intermediate fixture issues were corrected without runtime changes: the legacy-v4 case needed an actual owned launch before its existing fix adapter, and the explicit-bundle fixture needed to select the real file-text envelope instead of a copied bundle marker inside public chat. Their failed assertions remain development checks, not admitted provider observations.

## Exact final commands

- `rustfmt --check --edition 2024 src/cli.rs tests/bench_review_evidence.rs` — exit0.
- `git diff --check -- src/cli.rs tests/bench_review_evidence.rs` — exit0.
- `cargo test --test bench_review_evidence --features bench-experiments -- --nocapture` — exit0, **5 passed, 1 ignored** subprocess entry point.
- `cargo test --test bench_review_evidence --no-default-features -- --nocapture` — exit0, **4 passed, 1 ignored** subprocess entry point.
- `cargo test --test cli --features bench-experiments command_chat_ -- --nocapture` — exit0, **19 passed**, 15 filtered out; existing review reuse, reviewer directives and stale-refresh checks included.
- `cargo clippy --test bench_review_evidence --no-default-features -- -D warnings` — exit0.
- `cargo clippy --test bench_review_evidence --all-features -- -D warnings` — exit0 after the adapter owner resolved its in-progress `collapsible_if` warning.

The ignored test is an explicitly invoked, bounded provider-free subprocess fixture, not a real provider scenario. Tests set experiment environment only on child processes; no parallel process-global environment mutation or credentials are involved.

Root owns `cargo fmt`, full all-target/all-feature Clippy and tests, architecture/operator/protocol updates, complete offline archive/native audit and any later real diagnostic admission. The relevant architecture was read completely before implementation; its module ownership and public interfaces are preserved. No committed test, active frozen runtime/helper/report, release binary or provider configuration was edited. No provider generation, new control or benchmark was launched by this integration task.

## Independent private-adapter review

The integration owner independently reviewed the other owner's `src/bench_experiment.rs` delta, complete `src/bench_review_evidence.rs`, and new `src/bench_review_evidence_tests.rs`. The review covers source isolation, schema/identity and owned-span validation, selected/nonselected symmetry, archive lifetime/failure propagation, default/legacy preservation and complexity; it is not an independent review of this owner's CLI patch.

One P2 finding is closed: the original separation guard handled only an explicit `WORK_LEAF_CONTEXT_BUNDLE_DIR`, permitting an opaque root under the ordinary default bundle tree when that variable was absent. The adapter owner reports its isolated-TMPDIR regression failed before the fix. The reviewed implementation uses the same effective parent as `ContextBundleStore::new`, including `temp_dir()/work-leaf-context-bundles`, without constructing a store or consuming a counter. The new exact/nested-default regression independently passes.

The descriptor-based publication verification retains the original create-new file handle, rejects symlinks/replaced final inodes and compares bytes through that handle. It does not claim hostile concurrent-writer containment or continuous immutability. Missing ordinary-bundle ancestors retain explicitly flagged **O(D²)** filesystem component resolution in path depth D; held context/candidate work is linear, with no session-history quadratic copying or earlier-archive rescanning.

`cargo test --lib --features bench-experiments bench_experiment::review_evidence -- --nocapture` independently passes: **10 passed, 1 ignored**, 59 unrelated tests filtered out. No introduced correctness/isolation finding remains at activation SHA-256 `d23fe2475f645fb6d91db29d54048be87125f3e1ec17cee04dee393aed33e7e6`, archive adapter `0d7ef8db0b0f7264fb1e11aa1e87327b3faab668681924bd03f9d18cbcef84a1`, and private adapter tests `c12178b6b8543dd92a77e18d8f4b2cae468f0fc5fd7a89fd4f53193ad368a3ec`. Root's architecture paragraph must describe the private opaque evidence type before admission; no public API or observer change is required.
