# C21 implementation and provider-free review

Verdict: no outstanding introduced source blocker in the reviewed runtime, integration,
bounded diagnostic fixture or offline primitives below. This is **not** real-agent
verification or a completed source-collector review. No provider was launched, no release
was built and no benchmark or control was admitted by this review.

## Exact reviewed inputs

| Source | SHA-256 |
| --- | --- |
| `src/bench_experiment.rs` | `d23fe2475f645fb6d91db29d54048be87125f3e1ec17cee04dee393aed33e7e6` |
| `src/bench_review_evidence.rs` | `06101576abc6166d57497d8806b48357e38c0611b6d8d46f15a5e6285078b725` |
| `src/bench_review_evidence_tests.rs` | `c12178b6b8543dd92a77e18d8f4b2cae468f0fc5fd7a89fd4f53193ad368a3ec` |
| `src/cli.rs` | `a87be2707fcb873d1cdfba319d9e6d1b07ab6aa7aacdf8e5aa6ff25f98ab1025` |
| `tests/bench_review_evidence.rs` | `dc5b1d9e58d153049b29c42dae4bf7b38ef8dcde91faac0a615598ebfd8442fc` |
| `tests/bench_review_evidence_subscription_smoke.rs` | `b94bcae247d704ecc3c913522848d2597a1a1dcdde132c4da9f66d10f4cda345` |
| `audit_review_evidence.py` | `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257` |
| `test_review_evidence_audit.py` | `d5a58b1a001c9fd909a34b0371a614e5b194aac65277a82fc9d027e401da3afe` |

The adapter author produced this receipt. Its runtime was independently reviewed by
the integration owner, including the default-bundle-namespace correction. That review
closed at adapter SHA `0d7ef8db0b0f7264fb1e11aa1e87327b3faab668681924bd03f9d18cbcef84a1`.
The final `06101576…` differs only in the new cfg(test) temporary-name counter: an
in-memory reversal reproduces the reviewed hash exactly; the entire runtime prefix
before cfg(test) is byte-identical. The CLI integration, complete 650-line new integration
fixture, complete real-diagnostic fixture and Python primitives were independently read
and checked by this reviewer. No file owned by those implementing agents was edited.

## Runtime and integration boundaries

`CommandChat::review_commit_streaming_with_ids` retains the existing single backend
session snapshot and `render_review_source_context` call. It constructs the ordinary
review prefix, appends the held source context once, then appends the unchanged suffix.
Only the two append-time UTF-8 offsets reach the private adapter. New/reused reviewer
dispatch, author fixes, reviewer rechecks, profiles and public integration APIs remain
outside the factor. Default and old-schema full-prompt comparisons use an independent
prepatch formatter, including missing, empty, populated and adversarial marker/Unicode
contexts and reused reviews.

Schema-5 selected and inline test paths archive the same held bytes with independent
numbering. Old schemas strictly reject the new root field, including null; inactive
and old-schema calls perform no new archive work. Canonical project/bundle separation
includes the ordinary temporary-directory default. Payload/manifest creation is
exclusive and read-only; descriptor-based readback and Unix path/inode checks reject
replacement identities. Failed archive or trace publication prevents delivery and
retains available evidence. The integration fixture interleaves large bundle allocations,
changed/unchanged repeats, explicit bundle reads, done invalidation and unsupported
opaque-only/mixed reads. Ordinary bytes/counters and cleanup match the baseline fixture;
earlier opaque files survive later reviews and ordinary store cleanup.

## Bounded diagnostic scope

The diagnostic permits the actual author edit → ACK/done → reviewer full native read
and locked check → clean result path: four ordered outer handoffs, two owned threads,
an independent six-call ceiling and a 118-second watchdog. It does not rewrite the
post-adapter prompt, emulate a reviewer or forge a successful retrieval. The captured
public command must be exactly the issued-path read, with exact complete output and
one read action; same-output shell suffixes are rejected. Accepted/public inputs are
consume-once, original/forwarded turn starts agree, and both final thread turns must
settle. Strict closed JSONL rejects incomplete tails. The scenario result is printed
before later audit assertions and cannot be replaced by a postcapture verdict.

The public capture assertion is intentionally narrower than native rollout/source/model/
subscription/accounting proof. The separately pinned collector must establish those
facts. A public path mention is not native retrieval evidence. Missing real verification
remains a readiness gate, not a clean automatic-test substitute.

## Resolved review and validation findings

- Default bundle separation: an unset override originally left the normal default tree
  unchecked. A subprocess-owned TMPDIR regression failed first and passes for both the
  exact default root and a nested opaque root, without creating a normal bundle store.
- Publication identity: path-based readback accepted a different same-byte regular file.
  The RED regression passes only with the held create-new descriptor and Unix final-path
  identity check. Owner-only creation/read-only sealing is separately tested.
- New test isolation: simultaneous timestamp-only unit-test roots collided in a full
  suite. A cfg(test)-only atomic component resolves it; no runtime bytes change.
- Diagnostic failure preservation: generic stop failure hid captured observer stderr.
  A new failing-then-passing formatter test retains stop stdout/stderr and logs settling
  independently, without altering provider behavior or replacing the original outcome.
- Shell proof: unquoted operators/expansion in a declared pathname incorrectly counted
  as literal retrieval. Quote-aware validation covers both outer wrapper and inner
  command; correctly quoted literals remain eligible, unsupported evaluation unresolved.
- Native identity: malformed present false/zero/empty turn IDs could fall back to a nested
  valid value. Each non-null present identity is independently validated before joining.
- JSON types: ordinary Python equality aliased a forwarded Boolean RPC ID to integer 1
  and a Boolean manifest sequence to integer 1. Recursive type-preserving equality
  rejects those independently reproduced provenance errors.

Checks at the hashes above: **15 Python primitive tests** and **4 smoke automatic tests**
pass independently; the real smoke is ignored and unlaunched. Final `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings` and
`cargo test --all-targets --all-features` pass: **496 passed**, zero failed, 18 ignored,
44 targets. The private adapter subset passes 10 automatic tests. The recorded REDs
and earlier interrupted full-suite qualification attempts remain failures in their
validation history, not retrospectively green observations.

## Complexity, documentation and remaining gates

Held-byte transformation and input joins use linear traversal with indexed ownership
and queues. The adapter's missing-directory ancestor lookup explicitly flags **O(D²)**
filesystem component resolution in path depth D, not transcript/archive count. Individual
partial-read classification scans its supplied payload; a collector calling it R times
over one B-byte archive incurs O(R×B) unless it indexes LF boundaries once. A complete
collector must not claim whole-census linearity while repeatedly rescanning histories.

The current architecture/private-design ownership is consistent. The prospective
`PROTOCOL-REVIEW-EVIDENCE-NATIVE.md` was read at SHA
`4010c02d3b653c240ad9d65396ecb5c559ee49a1c527b51fe46f3728b70e03df`.
Its stated nonadmission, original-screen closure gate, subscription-only setup and
separate native/accounting proof match the source. The root received a narrow wording
correction: exclusive/read-only publication prevents host reuse/overwrite, **not**
hostile external mutation or proof of continuous immutability; no new runtime crypto
dependency or external hashing process is required. The detailed design already states
those limitations. This does not represent a runtime blocker or waived evidence gate.

The complete source-bound collector/CLI, its actual native call/output inventory and
source-pinned postcapture proof remain a separate unfinished review scope. Real configured
subscription retrieval is unlaunched. Earlier closed candidate 001/003 preparation
remains preserved without regenerated input manifests or executed postcapture audits;
the all-three-closure gate remains in force. No active candidate cost or contrast was
inspected for this review.
