# Independent v2 bridge review

Verdict: no introduced blocking source finding in this bounded bridge delta.
The local automatic suite passes. This is not full-project Cargo qualification,
actual-agent verification, observer/grace qualification or future admission; those
remain explicit readiness gates. Original C15 outcomes and v1 artifacts are not
corrected, replaced or relabeled.

## Exact reviewed cut

| Source | SHA-256 |
| --- | --- |
| `bridge_v2.py` | `8d2896dc2ce6bf2bc3085318ce3b303adf8e2b59623a088372481ec72bc57b42` |
| `test_bridge_v2.py` | `9e96b4c1e709dcde544e0256dfa42035afa2cb8f18764d29a2562c4297f6926e` |
| Pinned sibling selector | `ddfa7b6eb99bf713669caf5d3766aaa11843b24057b86329fd6985b83733dd5f` |
| Selector tests | `9adae3746498dec664e107f08a7e81c5fcb668a9a3917ed9367f86dc724aaf5d` |
| Original bridge | `6af6c3f24f96a7b95514a0db702a775b895a8d89f5dc8a2db9aa1329db9d4d26` |
| Original selector | `0614db9a5c874cd5f623997ce3433edbd2bc0ff3585c23913dd9ede9705d3a55` |
| Derived-output design | `605cefb6665385833d14ff0d82d7e3213cef985c1b7eb8e60f4ae3fa8cc0b504` |

The complete 374-line bridge and 335-line test file, full bridge/predecessor diff,
derived-output design, selector contract/validation and separate selector review
were read. Relevant original bridge, materializer and current v6 bootstrap paths
were inspected. The reviewer implemented the selector, so this is independent
review of the root-authored **bridge delta**, not a second independent certification
of the reviewer's own selector. The selector has its separately authored review.

Independent `python3 -B -m unittest -q test_bridge_v2.py` from this directory
passed **26 tests in 14.852 seconds**, tool `cf8a81`, exit 0. The two printed CLI
errors are expected negative tests for duplicate JSON keys and output reuse.
Tests use owned temporary Git repositories and the qualified patch/namespace
executor; no Cargo, provider, saved-workflow extraction or accounting is invoked.
Before/after endpoints match, including the original v1 source identities
(`347874` / `1d219e`). Owner-reported RED `c73c4b` was not recreated by source
reversion during review. Only this new review note was written by the reviewer.

## Introduced behavior

- `HELPER_FILES` binds the exact new selector source, with the existing materializer,
  executor and project-input pins/layout unchanged. Sources still execute captured
  verified bytes rather than cached bytecode, with endpoint verification on both
  success and failure.
- The wire schema remains `work-leaf-private-preview-bridge-v1`, matching current
  `src/bench_private_test_first_bridge.rs` requests. The new strict configuration
  is `c15-runtime-bridge-config-v2` with mandatory `derived_output_roots`; the result
  is `c15-runtime-bridge-result-v2` and selected source is separately v2. No public
  Rust API, model prompt, command, timeout or ACK contract is modified here.
- `validate_config` validates the declaration's type, bounds, normalized paths,
  overlap and existing directory/ancestor safety before an attempt. It intentionally
  passes no accepted-file population in this source-free validation stage; the
  complete accepted/staged/overlay exclusion check occurs in the selector's actual
  capture census. A successful validate receipt is not a successful source capture.
- `execute` forwards the exact configured roots to the once-only capture. Before
  any test attempt or materialization, the owned prior capture must have the new
  selected schema, exactly matching declarations and the pinned selector digest.
  Existing prior-owner/proposal/config reference checks and the selector's exact
  receipt/bundle validation remain active. Missing/old schemas, changed declarations
  and v1 selector bytes cannot masquerade as the new contract.
- The original-live versus owned-accepted mapping, both-flag overlay provenance,
  private cold build/scratch, held-patch semantics, command text and factual result
  handling remain the qualified predecessor behavior. Declared output stays in the
  live project and out of the selected/private image. The actual fixture command's
  nonzero exit is retained, not treated as preparation failure or semantic RED.
- Runtime `main` still excludes qualification-only implementation; cancellation,
  uncertain executor closure, source failures, reserved publication paths and
  create-new/once-only failures keep their original meanings. Publication is not
  an atomic multi-file transaction or hostile-writer recovery mechanism.

The 26-case suite includes the retained predecessor behavior plus explicit output
preservation/noncopying, unlisted output failure, malformed declaration rejection,
v1 config/selector rejection and changed prior selected-declaration rejection.
It exercises the actual local private shell command in the declared-output case;
it does not claim a full benchmark project's post-check state is qualified.

## Complexity, documentation and remaining gates

No new population-sized quadratic join appears in the bridge delta. Selector root
and ancestor validation is bounded and indexed; constructing/stringifying ancestor
prefixes can still require O(D²) character work in path depth, explicitly bounded
to D ≤ 64. The inherited at-most-32-mount executor O(M²) checks, Git/history work,
postcollection output bounds and postcreation bundle-storage limits remain flagged.
Repeated source passes are not represented as zero additional cost.

The new configuration/result/selection identities and remaining gates are
documented here and in the derived-output design/selector validation. The original
v1 contract stays immutable. Future executable freeze documentation must name this
distinct bridge/configuration rather than silently reuse the v1 qualification.

Before an agent-facing ready claim, the coordinated future private package still
needs full-project post-check capture/private-command qualification, the corrected
observer `test-preview` parsing/grace contract, exact v6 runtime integration and a
separately fixed/admitted actual-agent scenario. No such admission or real-agent
result follows from this source review or the 26 local tests.
