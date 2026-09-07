# C15 Rust integration: independent source review

Recorded at 2026-09-07 19:27:54 UTC. Scope: introduced benchmark-private Rust behavior and its tests/documentation, not a provider admission or a token-effect estimate. The source cut below is uncommitted relative to repository HEAD `1d7a418c31400b47ba9baed801e2c3798284e75c`.

## Disposition

No unresolved source/test correctness blocker was found at this cut. The source review is closed; overall agent-facing readiness is **not** established. Required final repository gates and a separately admitted real-agent verification with exact postcapture delivery/source checks remain outside this review. No provider calls, accounting calls, benchmark runs, implementation edits, or frozen-artifact edits were performed by this reviewer.

Independent command: `cargo test --lib --all-features private_test_first -- --nocapture` — **33 passed, 0 failed, 2 ignored** (execution `03708f`). The subprocess child is invoked by automatic parent tests; the separately gated actual confined-bridge qualification was not rerun. Earlier independent cuts passed 25 and then 28 automatic tests; those counts are not substituted for the final 33.

## Reviewed behavior and finding closure

- `src/agent.rs::PromptPolicy::inject_for_delivery` dispatches to the private renderer only for active v6. `src/bench_experiment.rs::load` admits only the explicit v6 condition/descriptor and delegates older manifest parsing unchanged. Default builds omit the module. Feature-enabled inactive and v1–v5 paths do not prepare author tickets, consume the private generation counter, initialize Python, or change ordinary user/system IDs, bundle counters, policy spans, ACKs, or command continuations. The extra private `Arc` field is not a model-visible allocation namespace.
- `src/cli.rs::prepare_agent_launch` establishes author provenance; `prepare_linearize_launch` explicitly establishes a non-author ticket. `launch_prepared_agent_streaming_with_ids` begins the exact prepared ticket before backend injection and commits only after successful launch. Public `AgentLaunch`, provider traits, and controller DTOs are unchanged. External preconstructed launches without a ticket fail closed in v6; this is not enrollment inferred from a name or prompt.
- `Registry` clones share one owner state. Process-unique generations prevent separate registries using the same agent ID from aliasing result tokens. The synchronous thread-local scope binds the exact launch, registry, generation, and role. The persistent explicit-author identity set denies unsupported cross-thread/deferred unowned injection; it does not grant an author role. Explicit non-author scopes remain non-author. The active resume guard rejects policy reinjection instead of treating a new session as a valid continuation.
- The initial cross-thread injection defect is closed: an author backend could previously return a baseline policy on another thread and fail only at commit. The actual synthetic-backend cross-thread and deferred-after-rollback regressions now reject before policy publication/delivery. These are automatic protocol tests, not real-provider evidence.
- Stop/shutdown coverage includes prepared tickets, provisional launches, committed authors before their first reservation, and executing previews. `mark_injected`, `begin`, and `LaunchGuard::commit` reject cancelled tickets; shutdown also rejects future preparation. The targeted/all-cancel three-stage regression and actual cloned `CommandChat` interrupt/shutdown-before-commit tests close the provisional-launch gap. Cancelled reservation results cannot open the apply gate or authorize feedback; uncertain subprocess closure remains explicitly false.
- `src/workspace.rs::apply_agent_title` and `set_pending_dependent_launch_prompt` revise the exact pending ticket. Failed revisions cancel preparation and preserve the visible rejection rather than launching an unowned mutation. The coverage finding is closed by `bench_private_test_first_controller_tests.rs` plus the integration parent: active title/dependency revisions retain the author role, and cancelled cases produce zero backend launches. The actual controller notice is rendered by the test-only `UiHarness` adapter with raw input/mode transitions. No new public harness API or committed-test mutation is needed.
- `parse_preview` retains the exact raw proposal body and typed metadata, rejects mixed/nested top-level envelopes, and does not require model-computed byte ranges or repeated bodies. A valid unified-diff context line containing literal `@work-leaf` source is not a top-level directive. Declared `test_paths` map to host-retained afterimages; neither the parser nor those file identities prove semantic test-only purpose.
- `src/orchestrator.rs::run_private_test_preview` reuses ordinary path normalization, other-agent test ownership, and failure-masking checks. The bridge only exposes validate/capture/test operations. Selection uses the existing shared `FileLockTable` root read lock; materialization and execution follow release, using the selected accepted objects and separately declared effective overlays. Private implementation is not a live operation.
- The Rust bootstrap executes exactly hash-checked Python/bridge bytes with pinned configuration and a cleared environment. The separately reviewed Python bridge checks its exact predecessor/capsule/source inventory. Result validation binds run, owner, generation, proposal, receipt identity, and closure; output or cancellation publication errors do not become successful test results. Watchdog/error handling reaps the direct child and retains `closed: false` if nested closure is uncertain; it does not delete uncertain trees.
- Only a closed executed test result successfully delivered in the same author session qualifies ordering. A preparation failure does not. Actual nonzero execution is retained as factual feedback, not labeled semantic RED by the host. The later shared combined patch, ordinary ACK, command, and review paths remain authoritative. Private execution does not promote files, record ordinary patch/command events, clear read snapshots, or allocate ordinary bundles.
- Exact proposal replay can resend saved feedback, never execute the command again. Changed bytes require a new explicit identity/revision; stale generations cannot settle a newer proposal. Executing or undelivered work blocks relaunch. Cancellation without delivered feedback leaves a blocked owner, not an automatically recoverable/retryable state.
- Host generation IDs and receipt path/hash fields remain in evidence rather than model feedback. The final automatic test checks both the omitted model-facing fields and retained evidence. Runtime send success is not claimed to be an exact native thread/turn/input join; that remains a postcapture qualification.

## Complexity and interpretation limits

The accumulating directive detector adds repeated C15 scans of growing completed-message text: O(K×B), worst-case quadratic across fragments. This is explicitly flagged, not described as a constant-pass whole-stream algorithm. The eight-MiB envelope limit is not a bound on the detector's accumulated memory. Registry lookup is logarithmic; proposal/body work is linear per invocation. Existing bounded O(M²) mount validation and O(D²) ancestor resolution, Git work, and per-proposal source hashing remain additional costs.

The architecture paragraph accurately distinguishes selected accepted objects, original live execution-view identity, overlays, sampled source endpoints, and cooperating lock coverage. The lock is not a proof against arbitrary concurrent external writers. Fresh private build state, cleared environment, extra feedback, and possible sandbox-only failures are part of this intervention; one successful target does not prove ordinary/private environment equivalence or a pure timing-only saving. Other unchanged architecture/operator/prerequisite documentation was inspected as required; no additional resulting-state documentation gap was identified.

The retained provider-free qualification at `/tmp/c15-rust-bridge-qualified-001` records private `sh check.sh` exit 1 followed by an ordinary shared ACK and command exit 0. This reviewer read its result/manifest and test-output metadata, not a new execution. Its earlier model-facing result included host receipt fields that are absent from the reviewed final cut, so it is **not** final-byte real-agent verification. Relevant retained hashes: result `0bd42a469eca544a6b83da18e9b5ac998a0cb42e9eb86f4a32c1b4ac974aa8ae`; manifest `f776feaef65de58839fadc4d38adc13c799fa023aabac5c4e0b5fb54ece2f697`; referenced test receipt `825b609825ea9d0e927e5ed31d4e88c52683471d57a0619388b7a362ed513243`.

Prerequisite source/confinement qualifications and their limits are retained in [INDEPENDENT-REVIEW.md](INDEPENDENT-REVIEW.md), [CONTRACT.md](CONTRACT.md), and the linked prerequisite reviews. They do not replace a real configured agent scenario.

## Exact reviewed source cut

Paths are relative to the repository root; SHA-256 values were checked after the final independent focused test.

| Path | SHA-256 |
| --- | --- |
| `src/agent.rs` | `5850fc7dd63a97249888dc8eeff7c525f749e4f2ae6e35c372cb7fc8d97ee800` |
| `src/bench_experiment.rs` | `658766748beca26424d23a3d95201ba9d1948d8fbcd7910dfc0f2f0e4f52bd94` |
| `src/cli.rs` | `98b08ef350616c2f9d33ca0dd6ca6916cf31a0a985e694370b0097acd4e867ad` |
| `src/orchestrator.rs` | `e0ef8167bd1f51cd99d56ae58835ba5d6951549c9dc9a2ccdabf9b2e568ff0f7` |
| `src/workspace.rs` | `9b05ae5bdae5f334fdceca1b81d638eff230c68edf2d3009d1c2e3bdf9fe07cc` |
| `src/ui_harness.rs` | `3d5b7a82aaa21fc0e2b8fb087ba0ce562d0fae1ca878046e9cdbbb0ebe153eb6` |
| `src/bench_private_test_first.rs` | `1700f26d5ff99271ca12acbfa783ff0d5525d3720c9ba47ff088c1aaa9d40d2f` |
| `src/bench_private_test_first_bridge.rs` | `923c23aad1d6af7ec9b1f168a2fa33f88967464dc40648fb228669e056a3e291` |
| `src/bench_private_test_first_tests.rs` | `82bfc571df12402c25e6a152a0fea0e478a8bc3dd3918199f8c4f32c91fd6e37` |
| `src/bench_private_test_first_integration_tests.rs` | `95a8943cb64c47c9cfe7b0f1dcee5e6671181422c2a3359cece9e78148d27ab2` |
| `src/bench_private_test_first_bridge_tests.rs` | `fdd186754e73c528e8b0d196b86db63f23303f94c4c5343a62e841922013d6ac` |
| `src/bench_private_test_first_controller_tests.rs` | `3705e6c148f5362bcb2e6cd9d2c25e388e64a4ec512c409aecbbc666324fb3c1` |
| `src/bench_private_test_first_ui_tests.rs` | `cee954c70597828cb6bb9cd9d641866a6647249f9f66fe79661771368e7f2abb` |
| `docs/architecture.md` | `20c1f2bd4cdec7f0ba4fd613eecc2fd4156554f4b4d46d53bdfbb4f6773df921` |
