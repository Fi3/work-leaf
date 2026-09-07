# V3 read-runtime review

Source/automatic-test verdict: no remaining blocking finding in the reviewed factor implementation. Agent-facing readiness remains conditional on the separately retained real subscription diagnostic and final required-check receipts; this review does not label an ignored real-agent test as passed.

## Reviewed identity and scope

| Source | SHA-256 |
| --- | --- |
| `src/orchestrator.rs` | `71610cc88fef911fddd2099a1237166df51f8f31a9528268085e7f65477f792b` |
| `src/bench_experiment.rs` | `951c0a96795582c51473aaba59b1fc2b51ffe2a440414174221b3111441e40ad` |
| `src/bench_read_experiment_tests.rs` | `8e095c7a9f65bee8182b7803d116e22fa298935b9268a99508bff8dabeeb6c7e` |
| `tests/bench_read_inline.rs` | `e225f5a673458572278bd84e011c83ab678890baeccd64e7a7ec95084fbcdf78` |
| `tests/bench_read_inline_subscription_smoke.rs` | `8dc73505572a482c5f497fa3dcb9f16dc1f6b1338271568d40d1e7121f3d8c88` |
| `docs/architecture.md` | `8377838c35e99cac89d7d67691ff9b10ff515b5270d32502c3bb6aa63e0b3ecd` |

The review covers the complete changed renderer/extension code, both new read test files, the new real-smoke fixture and its automatic tests, and the architecture addition against the source-representation design/protocol. Existing public controller/provider interfaces, Codex launch implementation, tracker, locking, command and patch rules remain outside the changed behavior.

## Isolation checks

`send_file_read_response` calls the existing read/split path once and retains the same downstream send, tracker recording and event sequence. The private `FileReadGroups`/`RenderedFileRead` boundary exposes evidence without changing a public API. `render_file_read_response` retains one ordinary threshold decision and bundle write attempt; its successful bundle path proves eligibility. Both active v3 arms build the inline candidate from the same held snapshots and append the once-rendered repeated/failure suffix. Bundle failure retains the ordinary inline fallback and counter progression.

The final composition shifts component/body ranges past an explicit-bundle prefix. A pure explicit-bundle request correctly has no delivered project component. `forward_read_with` checks UTF-8 slice validity and exact outside-component identity, serializes both candidates for both arms, records selection, and returns the selected string only after successful evidence writing. It introduces no additional read, diff render, bundle allocation, model-visible label, forced call or wait. Large-input failure remains a treatment outcome rather than a clipping branch.

The v3 admission allowlist cannot activate either earlier cue/work-unit treatment. Non-read v3 boundaries use baseline static spans; v1/v2 schemas and transformation logic remain intact. Default/inactive reads do not construct v3 candidates or metadata. Snapshot order uses the existing `Path` ordering; the fixed three-class merge performs at most three candidate comparisons per output snapshot. Rendering, body-offset tracking, digest work and candidate copying are linear in the retained bytes/snapshot count, with no new O(n²) event or snapshot join.

The architecture text describes the resulting private benchmark boundary and its evidence/compatibility limitations. No public API or architectural ownership change needs authorization. The distinct protocol retains future-only admission, the single raw-token endpoint, failures, and no historical causal-share inference.

## Automatic checks and resolved finding

Independently executed:

- `cargo test --features bench-experiments --test bench_read_inline`: six passed, one subprocess entry point ignored by the parent harness.
- `cargo test --features bench-experiments --lib bench_experiment::read_tests`: three passed.
- `cargo test --features bench-experiments --test bench_read_inline_subscription_smoke`: six automatic tests passed; the real provider test remained explicitly ignored.
- `git diff --check` for changed renderer/extension/architecture files: passed.

The integration cases cover default/control and legacy identity, strict threshold boundaries, Unicode/no-final-newline/empty source, coalescing, mixed untracked/repeated/missing/explicit-bundle responses, separate agents, ordinary bundle failure/counter/cleanup, failed send without tracker advancement, own-edit tracker clearing, conflict refresh and ACK/command identity. Private tests cover owned-span failures and evidence-write failure in both arms. The implementer separately reports clean format/clippy and full-suite checks; final admission must retain their completed receipts.

One P2 finding concerned only the new smoke teardown verifier: a mixed success/error turn-start reply or empty turn ID could satisfy its witness. The implementer demonstrated two new RED regressions and restricted the predicate to a success without an error member and a nonempty string ID. This review independently inspected the narrow correction and reran the six automatic smoke tests successfully. The factor's two runtime source hashes are unchanged.

## Real-agent gate

The required scenario is one actual subscription-backed session with three bounded turns: the model requests a large fixture read, receives the selected bundle/inline response, requests a repeated read, receives the unchanged tracked response, then emits done. Both active arms require exact trace/session/request equality and actual final-interrupt forwarding, typed acknowledgement, terminal turn and observer closure. Its prohibition on extra inspection is diagnostic-only, not a benchmark policy. Provider/tool actions and capture provenance still require inspection of the retained real evidence.

No provider call was made by this reviewer. Real scenario receipts were not yet supplied at this source-review cutoff, so overall agent-facing readiness is pending rather than green.
