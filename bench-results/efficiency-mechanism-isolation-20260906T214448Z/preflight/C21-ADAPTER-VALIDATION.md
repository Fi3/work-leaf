# C21 private archive adapter validation

Scope: provider-free validation of schema-5 activation and the private review-context
archive adapter. This receipt does not admit generation or an inline control. The real
configured-agent retrieval check belongs to the root operator and remains required
before agent-facing readiness. No provider, observer, ordinary bundle store, driver,
public API or committed test is part of this adapter patch.

## Source identities

| Input | SHA-256 |
| --- | --- |
| `src/bench_experiment.rs` | `d23fe2475f645fb6d91db29d54048be87125f3e1ec17cee04dee393aed33e7e6` |
| `src/bench_review_evidence.rs` | `06101576abc6166d57497d8806b48357e38c0611b6d8d46f15a5e6285078b725` |
| `src/bench_review_evidence_tests.rs` | `c12178b6b8543dd92a77e18d8f4b2cae468f0fc5fd7a89fd4f53193ad368a3ec` |

`DESIGN-REVIEW-EVIDENCE-ON-DEMAND.md` defines the factor. The root owns its architecture
paragraph, protocol/admission, the separate offline SHA-256/exact-byte audit and real
retrieval verification. CLI integration and its independent formatter/read-route tests
have a separate owner and validation receipt.

## Contract

`forward_review_context(project_dir, source_agent, reviewer, target_commit,
baseline_prompt, context_span)` is crate-private. Only schema
`work-leaf-bench-experiment-v5` enters archive validation. Its conditions are
`review-evidence-native` and `review-evidence-inline`; the latter is an automatic
identity-test condition, not authority for a new control observation.

The required `review_evidence_root` identifies a pre-created, absolute, exact canonical
directory. Its Unix device/inode identity remains checked at each publication. The
root must not overlap the project or the effective ordinary context-bundle directory,
including the ordinary temporary-directory default when its override is absent and
a bundle directory whose final components do not yet exist.
Older schemas deserialize through their original strict manifest type and reject the
new field even when its value is null. Default/inactive and schema 1–4 hook calls
return the held baseline before new span/root validation or archive allocation.

Each schema-5 handoff holds the caller-owned UTF-8 slice once and preserves it exactly
in independent sequence-numbered `.txt`/`.json` files. The selected and inline paths
perform identical archive/candidate work. Files use exclusive creation, owner-only
Unix creation permissions and final read-only mode `0400`. Publication reads back
from the original read/write descriptor and checks final-path regular-file/device/inode
identity. It never overwrites or truncates an earlier file. Partial publication and
trace failures remain retained and prevent returning a deliverable prompt.

The typed manifest schema is `work-leaf-review-evidence-v1`; its `archive` object repeats
the prompt trace's kind, run/source/reviewer/commit identities, independent sequence,
payload and manifest paths, byte length and explicitly named `fnv64:<16hex>` checksum.
It also retains the source-owned context range and
`project_snapshots_applicable: false`. FNV is not cryptographic identity.

The prompt event is `review-context`, site `review-source-context`, with global prompt
sequence and timestamp; original/candidate/forwarded complete strings; source and
candidate UTF-8 ranges; selected candidate; byte lengths/delta; and the exact typed
archive object. Only the context interpolation differs. The receipt authorizes native
read-only access to the exact supplied opaque path and says it is not served by
`@work-leaf read`. It does not force retrieval or prescribe a verdict. Schema-5 policy,
ACK and command traces retain their existing normal bytes through the identity renderer.

## RED and GREEN evidence

1. `cargo test --lib --features bench-experiments review_evidence_tests -- --nocapture`
   first failed with missing `forward_review_context` at the new adapter assertions.
   The integration owner independently recorded the preceding behavioral RED:
   schema-5 requests failed on unknown `review_evidence_root` before CLI integration.
2. The owner-only publication assertion reproduced `0444` instead of `0400`, then
   passed with explicit owner-only creation. Its first invocation encountered a
   test-directory PID reuse collision; a timestamped test identity removed that fixture
   collision before the actual permissions RED was recorded.
3. `cargo test --lib --features bench-experiments publication_verifies -- --nocapture`
   reproduced acceptance of a different regular file with identical bytes under the
   path-based readback. Descriptor-bound identity verification makes that assertion
   pass. The same test rejects a replacement symlink and altered expected bytes.
4. The independent review's default-bundle-namespace fixture reproduced acceptance
   with an unset bundle override and a subprocess-owned `TMPDIR`. Both the exact
   effective default directory and its descendants are rejected before trace creation.
   No ordinary store is allocated to determine that parent.
5. Final focused adapter run: **10 passed**, one isolated subprocess fixture ignored as
   a standalone test. Coverage includes all old schemas/inactive invalid new spans,
   exact Unicode/marker-bearing opaque bytes, two immutable snapshots, retained lifetime
   after checkout removal, condition symmetry, invalid/missing/aliased roots, bundle and
   project overlap, wrong conditions/unknown fields, invalid UTF-8 ranges, reused payload
   and manifest paths, symlinks, replaced root identity, FNV vectors and failed trace writes.
6. Final library test result: **69 passed**, one isolated fixture
   ignored. The eight integration targets `bench_experiments`, `bench_work_unit`,
   `bench_read_inline`, `bench_repeat_read`, `bench_candidate_activation`,
   `bench_candidate_policy`, `bench_candidate_followup` and `bench_review_evidence`
   passed **36 automatic tests**, with their standalone child fixtures ignored.
7. `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings` passed.
   The first full `cargo test --all-targets --all-features` attempt reached the adapter
   and integration targets successfully, then encountered the separate smoke owner's
   two intentional in-progress RED tests. A subsequent attempt exposed a same-timestamp
   collision between new parallel adapter test roots; a cfg(test)-only atomic identity
   counter preserves unique fixture directories. Reversing only that test delta exactly
   recreates independently reviewed adapter SHA `0d7ef8db0b0f7264fb1e11aa1e87327b3faab668681924bd03f9d18cbcef84a1`;
   the runtime prefix is byte-identical. The final full suite passes **496 tests across
   44 targets**, zero failures and 18 ignored real/subprocess fixtures, at smoke fixture
   SHA `b94bcae247d704ecc3c913522848d2597a1a1dcdde132c4da9f66d10f4cda345`.
   Final `cargo fmt --check` and full clippy also pass. Earlier failed checks remain
   documented, not relabeled as successful runs.

## Complexity and limits

Payload hashing, candidate construction, serialization and descriptor readback are
linear in held context/prompt bytes; prior archives and session-message histories are
not rescanned. The independent archive sequence and global trace use fixed mutex state.
The missing-bundle-ancestor validation loop can require **O(D²) filesystem component
resolution in path depth D**. This introduced metadata complexity is explicitly flagged;
it is not quadratic in reviewer history or archive count.

Read-only permissions, canonical checks and Unix descriptor identity checks are not a
hostile-concurrent-writer sandbox or proof of continuous immutability. Earlier payloads
are not repeatedly reread at every handoff. The separately pinned final offline audit
must verify retained payloads, manifests and trace slices with SHA-256 and exact bytes.
Non-Unix platforms lack the Unix inode/device identity check. Actual native accessibility,
retrieval completeness and response/token accounting are not proved by these fake-backend
or private adapter tests. No new provider calls, release build or benchmark admission
were performed for this receipt.
