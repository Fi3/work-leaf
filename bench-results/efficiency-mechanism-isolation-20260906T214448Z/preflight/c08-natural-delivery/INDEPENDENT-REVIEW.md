# Independent pure v7 validator review

No blocking finding in the reviewed in-memory implementation. This is not a
closed-workflow delivery, native membership, conflict-completeness or accounting
qualification. No actual run payload or historical native source was read.

## Exact reviewed cut

| File | SHA256 |
| --- | --- |
| `validate_automatic_refresh.py` | `c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f` |
| `test_delivery.py` | `9ac394e07c4d5ee04c6f3d03f02bbb775ef1ac5cd7679c1d40ade9b3ab071400` |
| `DESIGN.md` | `206c7b7b76ff310741d3401f22a154a4d9665b98ff1869e4e4fda9aa2c7e3095` |
| `VALIDATION.md` | `77dd25b9ec85648964bbfdb88491ae12d04fc581ec974afad0f741efd351020a` |

All four files were read completely and rehashed (`f95d00`). Source comparison
covered `src/bench_automatic_refresh.rs::Capture::{candidate,forward_with}`,
`src/orchestrator.rs::{RefreshPrompt,append_refresh_guidance,
render_file_refresh_response_with_diff,render_untracked_refresh_snapshot,
render_patch_applied_prompt,read_requested_files}`, the continuation/activation
shapes in `src/bench_experiment.rs`, normalization in `src/locks.rs`, and the
resulting v7 architecture paragraph. Exact renderer identities are retained in
the design; the pure helper does not load those files or attest its own provenance.

## Findings and limits

- Both automatic rejection formats use the actual cue/newline/format/header
  structure. Construction-owned offsets determine the two coherence spans and
  each eligible body; copied headings or diagnostics cannot select those spans.
  The candidate walk checks all intervening and trailing non-owned bytes.
- Available changed diffs require 1–49,152 bytes; eligible full-current text has
  no introduced 8-KiB cap. Empty, unavailable, oversized, unchanged, small/large
  untracked and failure branches retain their exact source framing. Only carried
  bodies receive length/FNV/SHA proof. Every previous body and metadata-only
  current body remain `not-carried`, not empty or independently verified text.
- Key/type/range checks reject boolean integer aliases and split UTF-8 ranges.
  Invalid events keep unknown eligibility. Census preserves every later physical
  row, rejects activation/process/sequence inconsistencies, and uses null physical
  locators for invalid line maps instead of dropping records. A valid zero-event
  census still has `complete_delivery_proven=false`.
- Ordinary identity continuations retain exact source span literals and byte
  equality. The command check proves framing only: multiline dynamic fields
  prevent unique recovery of executed command/status/output from this prompt.
  No v4 policy ownership, title heuristic, native identity or successful-command
  claim is imported. The design explicitly reserves these for a later scope.
- Production validation has constant-pass prompt/metadata work and indexed path
  uniqueness under ordinary Python hash-table assumptions. Component order and
  nonoverlap are checked before body copying/hashing; valid body intervals are
  disjoint. No quadratic history/prefix scan was found. Resource admission for
  external files remains outside this pure helper.
- Complexity flag, test-only: `test_delivery.py::event_fixture` repeatedly
  concatenates immutable byte strings while assembling snapshots/components.
  Generalizing it to unbounded equal-sized sections would be quadratic. Current
  calls use finite small section populations (at most seven snapshots); it is
  not the production validator or an approved external-source collector.

The final design and validation note describe the resulting pure scope and its
remaining gates accurately. No runtime/public API or agent-facing workflow is
affected by this helper, so a provider run is neither required nor authorized
for this review. Natural-run delivery and numerical readiness are not granted.

## Independent execution

In this directory:

```text
/usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest -v test_delivery
```

Receipt `eb6a9e`: exit0, all18 tests PASS, 0.018 seconds. A separate bounded
in-memory mutation check (`b1895e`, exit0) replaced each field of five synthetic
event/identity fixtures with twelve JSON-type/value alternatives: 2,328 cases,
no uncaught exceptions. It read no captures, exported no bodies and asserted no
source semantics beyond safe malformed-input handling.

The owner's missing-module RED `1969e2`, overlap-before-hash RED `240092`, and
subsequent GREEN receipts remain separately attributed in `VALIDATION.md`; they
were not relabeled independent executions. No Cargo, Git, provider, executor,
observer, extraction or accounting call was made here.
