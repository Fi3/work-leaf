# C21 source-audit independent review

The bounded source review has no remaining introduced correctness or isolation
blocker at the identities below. It covers the complete new collector and tests,
the pinned primitive interfaces, and the collector qualification documentation.
It does not qualify provider execution, token accounting, quality, causal savings,
or the completeness of an arbitrary external capture.

| Reviewed source | SHA-256 |
| --- | --- |
| `audit_review_evidence_sources.py` | `d3fd8c80746bf4bce565cb5f0a2e2eab29681b3aa40f89196cf95ebf344ed9ee` |
| `test_review_evidence_sources.py` | `f5c39b75d2ddc1c2493bf8d01f3821163a0e74d696360e812468ff097e425bc3` |
| Pinned `audit_review_evidence.py` | `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257` |
| `test_review_evidence_audit.py` | `d5a58b1a001c9fd909a34b0371a614e5b194aac65277a82fc9d027e401da3afe` |
| `preflight/C21-SOURCE-AUDIT-VALIDATION.md` | `d6ef3ed8463cf71ac1e1d547c33000b6fea12814151307715c398f365f286eff` |

## Independent checks

From the study directory, the final command passes **36 tests: 21 collector and
15 primitive tests**:

```sh
python3 -B -m unittest test_review_evidence_sources test_review_evidence_audit -v
```

Coverage includes exact-source CLI execution, input-manifest hashing, exclusive
output creation and unchanged output bytes after a refused overwrite. Source
checks cover typed identities, strict closed JSONL and physical lines, invocation
and native-session membership including title/usage-less sessions, held/archive
bytes, immutable typed publication records, endpoint hashes, failed terminal
outcome retention, and unresolved unsupported activity.

Both actual provider-free Rust adapter receipt spans in
`/tmp/work-leaf-review-evidence-adapter-67-1788789878145914766-9/trace.jsonl`
match the collector's receipt bytes exactly. The trace hash is unchanged at
`21252df8dc9c9ccf3a932426d7af255b56f796c8cb240ff522e1fd1f3d36d8a6`.
This checks the actual runtime renderer interface, not real native retrieval.

## Resolved review findings

- `receipt` matches the Rust single-LF boundary. Its independent literal fixture
  does not derive its expected text from the collector formatter.
- `native_tools` requires the supported function-call/output kind pair; a custom
  tool with an executor-like name cannot establish retrieval.
- `public_activity` preserves public native actions lacking an exact native
  identity witness. Such gaps prevent `absent`; a separately verified read can
  coexist with explicitly unresolved additional activity.
- Conflicting native item IDs invalidate both claimed call IDs. Duplicate or
  conflicting calls/outputs cannot leave an earlier claim eligible for retrieval.

Each defect has a retained regression; the implementation owner's validation
receipt records the observed RED-to-GREEN gates. The public/native activity join
is an identity witness only, not a claim that command/output semantics are equal.
Exact read bytes are checked separately.

## Limits and remaining gates

The pinned partial-read primitive can scan B archive bytes for R supported reads:
**O(R × B)**, potentially quadratic as both grow. This cost is explicitly flagged;
the collector uses keyed paths/calls and shared unknown groups rather than an
archive-by-native-stream scan or duplicated per-archive unknown inventories.

Retrieval scope extends from archive delivery through the supplied reviewer
session, not necessarily only until that review's verdict. Endpoint hashes and
directory checks are not continuous-immutability or hostile-writer proofs.
Build/source attestation does not independently establish the executable used by
an arbitrary process. Runtime admission, subscription/model/settings, observer,
native accounting and global configuration remain separate operator gates.

The actual admitted C21 real-agent diagnostic and its closed source-bound replay
remain outstanding at this review cutoff. No provider was launched, no benchmark
cost was inspected, and no runtime or frozen helper was modified during review.
Closed candidate 001/003 postcapture preparation remains held until the operator
confirms all-three closure and explicitly triggers the recorded audit workflow.
