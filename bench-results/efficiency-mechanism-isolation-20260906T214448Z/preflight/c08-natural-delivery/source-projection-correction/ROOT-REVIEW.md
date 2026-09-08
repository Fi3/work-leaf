# Typed-end correction root review

The complete derivative diff and six final regressions were read in `3cb8e3`.
Root independently ran `python3.14 -B -m unittest -q test_source_projection`
under a 30-second timeout: six passed in0.085 seconds, receipt `dc1bef`.
The reviewed source is `7ea311907b975f25b02215c3e750eb8d69b49311c30247db6967862fe44ed092`;
tests are `b2fcf172c2d30690c3e15d3f7fa30eaed6a946aa8455f5e6ddc6a5e073dc43c3`.

No implementation blocker was found. `typed_end` admits only the exact eight
typed fields and two source-owned raw supplements, requires the raw-enabled
app-server relation, binds the full four-entry hash map and raw start digest,
and leaves raw end/meta objects intact. Shared-field changes, unknown fields,
missing/wrong supplements and unmarked raw metadata remain failures. Nonraw
invocations retain exact typed identity. The capture lookup is indexed, with
no introduced quadratic scan.

The original helper/dependencies are unchanged and remain explicitly pinned;
the derivative's own source identity is separately checked. The complete
synthetic CLI uses a real typed ledger fixture, retains original failed-workflow
and observer flags, verifies full result hashing and rejects a second call to
the same output. Original failures remain immutable and are not replaced by
this review. A new explicit corrected-source scope is required before actual
execution; no accounting, observer replay or provider call is authorized here.

This correction is offline-only and affects no agent-facing workflow. Rust or
real-agent verification is inapplicable. Source availability will still not
establish complete token accounting, repair semantics or causal contribution.
