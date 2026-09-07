# C21 source-bound review-evidence audit

`audit_review_evidence_sources.py` is a provider-free collector for one closed
workflow or diagnostic. It verifies exact held/archive bytes, source-owned prompt
replacement, accepted public/native input identities, and narrowly supported
native archive reads. It does not estimate tokens, exclude failed workflows,
infer quality, assign causal shares, admit observations, or launch a provider.
The inline condition supports retained identity fixtures, not authorization for
a new control run. Runtime, observer and older audit helpers are untouched.

## Source identities and checks

| Source | SHA-256 |
| --- | --- |
| `audit_review_evidence_sources.py` | `d3fd8c80746bf4bce565cb5f0a2e2eab29681b3aa40f89196cf95ebf344ed9ee` |
| `test_review_evidence_sources.py` | `f5c39b75d2ddc1c2493bf8d01f3821163a0e74d696360e812468ff097e425bc3` |
| Pinned `audit_review_evidence.py` | `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257` |
| Actual `preflight/BUILD-ATTESTATION-REVIEW-EVIDENCE-V5.json` | `a1302c135721d5df640331ab8df6f00cac5a68235604f90fe71fd3987e75beb6` |

Commands run from this study directory:

```sh
python -m unittest test_review_evidence_sources test_review_evidence_audit -v
python -m py_compile audit_review_evidence_sources.py test_review_evidence_sources.py
```

The final combined run passes **36 tests: 21 collector and 15 unchanged primitive
tests**. The actual provider-free CLI subprocess writes a new fully source-bound
synthetic result with the input-manifest hash, then refuses the same output path
without changing its bytes. Python compilation passes. No Rust source is part of this collector;
the root operator owns the separately recorded runtime fmt/clippy/full-suite and
real-agent gates. No agent-facing workflow is changed by executing this audit.
Actual source-bound diagnostic replay follows the operator's separately admitted,
closed diagnostic; synthetic collector tests are not that real verification.

The actual build attestation was read through `Sources` with its recorded hash,
validated against all eight required runtime-source identities and rechecked at
the endpoint. Its real release/smoke build exits are both zero. This establishes
the supplied source/build record, not that an arbitrary process executed that
binary; executable admission remains the operator's separate evidence.

## Input and invocation

The input schema is `work-leaf-review-evidence-input-v1`. Each source reference is
exactly `{"path": "/absolute/canonical/path", "sha256": "64 lowercase hex"}`.
Required fields:

- `run_id`, `condition`, `helper_sha256` bind the run, selected v5 condition and
  the executing collector source. The primitive is compiled from its exact pinned
  source bytes; Python bytecode caches are not used for that dependency.
- `runtime_sources` maps the eight logical source paths in `RUNTIME_PINS` to exact
  references. `build_attestation` references schema
  `work-leaf-review-evidence-build-attestation-v5`, including matching logical
  `source_files`, successful build commands/exits and recorded binary digests.
- `activation` is the actual strict v5 experiment manifest. Its `evidence_path`
  must match `trace`, and its canonical `review_evidence_root` owns all archives.
- `terminal` is either the actual completed workflow receipt (`id`, `run_id`,
  `condition`, `launch_status: completed`, timezone-bearing `started_at` and
  `finished_at`, integer `launcher_exit_code`) or a separately source-bound
  diagnostic projection with schema `work-leaf-review-evidence-terminal-v1`,
  `run_id`, `condition`, the same timestamps and integer `exit_code`.
  Nonzero exits remain reported; zero is not an inclusion gate.
- `invocations` references the observer's `process-invocations.jsonl`.
  `captures` lists every app-server invocation as `{path, clients, forwarded,
  servers, start, end}`. The five references must identify its canonical original,
  forwarded, server and start/end artifacts. Terminal stream digests and the
  observer invocation census must reconcile exactly.
- `native_sessions` contains `{thread_id, source}` for every captured native
  session, including title and usage-less threads. Input joins use explicit native
  user turn identity, never an inferred preceding `turn_context`.
- `archives` contains `{payload, manifest}` references. Every issued archive must
  match the typed manifest and owned context bytes. Untraced supplied publications
  remain hashed, unresolved records. Unlisted directory files are named as a
  provenance failure without reading unpinned contents.

```sh
python audit_review_evidence_sources.py \
  --input /absolute/closed-run-input.json \
  --input-sha256 INPUT_SHA256 \
  --output /absolute/new-derived-result.json
```

The output uses exclusive creation, refuses aliased parents/existing targets,
and retains failures or unknowns. Sources reject duplicate JSON keys, nonfinite
numbers, malformed hashes, symlinks, hardlink aliases, incomplete JSONL tails,
and changing file identity/bytes. Physical JSONL line maps survive blank lines.
Source and archive-directory identities are checked again after derivation.
Resource bounds fail closed rather than clipping records.

## Evidence and limits

The collector checks the actual typed archive field, context byte ranges,
`project_snapshots_applicable: false`, independent archive sequence/path, FNV64,
offline SHA-256, source-owned prefix/suffix, exact receipt and all prompt byte
fields. Normal v5 policy/ACK/command prompts must remain identity deliveries.
The pinned primitive joins accepted original/forwarded requests to public user
items and exact native users, preserving distinct IDs and repeated occurrences.
First title launch is an ordinary owned policy; later raw title sends retain
that thread owner and are not skipped.

Native archive reads require a supported `function_call` executor and
`function_call_output`, exact reviewer thread/call identity, consistent explicit
or inherited same-turn metadata, and a call after the actual delivered native
user item. Exact literal issued-path lookup precedes the primitive's supported
`cat`/`sed`/`head` and exact successful output comparison. A path mention, custom
tool with a misleading executor name, indirect/unsupported program, missing or
conflicting output, or unsupported metadata is unresolved, not read proof.

Classifications are `complete`, `verified_partial`, `absent`, or `unresolved`.
Absence is restricted to the complete supplied reviewer session's observed
activity; it is not OS-level nonaccess. Public native actions without an exact
native identity witness prevent an absence claim. A separately verified complete
read can coexist with unresolved additional activity. Safe shared group references
retain those gaps without copying unknown lists under every archive. No message,
command argument, output or reasoning body is exported. No response IDs or token
totals are synthesized.

Joins use keyed paths, call identities and per-thread indexes, not an archive ×
native-stream scan. One explicit complexity limitation remains: the pinned read
primitive can scan an archive of size B for R supported calls, **O(R×B)**, including
short partial outputs. This can be quadratic when both grow; it is not described
as output-linear. The root accepted this bounded primitive cost for qualification.

## Observed RED gates

Before implementation, importing the new collector failed with
`ModuleNotFoundError`. Subsequent individual regressions reproduced and then
closed: duplicate outputs incorrectly proving complete retrieval; conflicting
native item IDs retaining one earlier call claim; unlisted partial publications;
untraced archive omission; nonempty public text metadata; unallowlisted build-body
export; unsupported custom-call masquerading; and public-only native activity
incorrectly yielding absence.

Independent review identified an extra LF in the collector receipt, masked by a
self-derived fixture. An independent pinned-Rust literal first failed the full
source-bound fixture, then passed with the source-exact receipt. Both actual
provider-free Rust adapter receipt spans in
`/tmp/work-leaf-review-evidence-adapter-67-1788789878145914766-9/trace.jsonl`
were compared byte-for-byte and passed, with endpoint trace hash unchanged at
`21252df8dc9c9ccf3a932426d7af255b56f796c8cb240ff522e1fd1f3d36d8a6`.
Those adapter records prove receipt parity, not real native retrieval or a full
source-bound diagnostic replay. Independent collector review by `study_harness`
closed on the exact helper/test hashes above with all 36 tests independently
passing and no remaining introduced correctness or isolation finding. The
O(R×B) primitive cost and separate actual diagnostic/source-replay limitation
remain explicit.
