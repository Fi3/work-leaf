# Independent typed-end projection review

No introduced source blocker was found. This is a bounded review of the separate metadata projection and its integration, not an actual-run source qualification. No saved capture/native payload audit, observer/extractor/accounting call, provider, Cargo job or private executor was run. The original failed source results, workflow outcomes and accounting UNKNOWNs remain unchanged.

## Exact cut

| File | SHA-256 |
| --- | --- |
| `audit_automatic_refresh_sources_corrected.py` | `7ea311907b975f25b02215c3e750eb8d69b49311c30247db6967862fe44ed092` |
| `test_source_projection.py` | `b2fcf172c2d30690c3e15d3f7fa30eaed6a946aa8455f5e6ddc6a5e073dc43c3` |
| `DIAGNOSIS.md` | `30835f02aaabf6ba7c263fe23eba37a059740c266221963ef7b6ea22c99ae572` |
| `VALIDATION.md` | `06b6127275dd635d8e9c30c3c59ad0c41e154af1b2d5ad74e64b1e7ac7505fc1` |
| `SOURCE-INPUT-SCHEMA.md` | `adf2fcdb37723c622c7deb4f130fb8589ad35bcc1630eb92467bbdcae48064ad` |
| Original helper | `d05e7e7daa62601782fbcca2058f8c31a81d29ac5301407643474a09f4889f80` |
| Original tests | `25c532f240bc49f070f318029a967e531d15e6ce1a055923e6fa94ec62470502` |

The complete original-to-derivative diff and all six correction tests were read (`a95ead`). Resulting diagnosis and validation were read completely (`1f9522`), as was the separate corrected input-schema note (`a04601`), which accurately requires new source-bound inputs/scope/output paths without replacing originals. The underlying serialization chain was independently checked (`c83d4e`): `bench-observer/src/lib.rs::{InvocationEnd,ProcessInventoryRecord,run_proxy,read_process_inventory}` at SHA `188a1b4fd9913556c51024dcc0068be353813c44c688d6d22929da139f392b5f`, and `raw_capture.rs::{end_metadata,verify_capture}` at SHA `a8397fa90fa8538cc74559c850401314bbb68726e0f1db03724133368c115a4a`.

## Reviewed behavior

`typed_end` compares only the eight source-defined typed ledger fields, but does not discard the raw end. A raw-enabled app-server must have exactly the two source-owned supplements, with its exact start-file SHA and exactly four declared forwarded/settings/journal/grace digests. Unknown raw or ledger fields, wrong/missing hashes and unmarked supplements fail. Nonraw ends retain complete eight-field identity. Subsequent exact full meta/raw/source/frame checks still receive the original full end object.

`collect_closed_captures` builds a unique indexed capture map and supplies the owned start/capture references before comparing the typed ledger. The correction does not edit other original checks, returned full invocation metadata, pure delivery validation, generated-profile qualification, output ceilings or create-new/no-retry publication. The dependency base stays at the original directory while `execute` independently pins the corrected source and original helper. No source assumptions are silently redirected by moving the file.

The diagnosis's serialization explanation matches the source: `raw_capture::end_metadata` appends the two raw supplements; deserialization into `ProcessInventoryRecord.end: Option<InvocationEnd>` yields the eight-field published ledger. Therefore whole-object raw-versus-typed equality is not the correct source relation. This conclusion does not convert a prior failed audit into a success.

## Independent checks and limits

From this directory, the declared bounded command was executed once:

```sh
PYTHONPATH=/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c08-natural-delivery /usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest -q test_source_projection test_sources test_join_delivery test_delivery
```

Tool `7ec830`: **79 passed**, exit 0, 0.419 s. This comprises six separate correction tests plus the unchanged 73 original source/join/event tests; it is not a claim that every original test was rebound to the derivative. The correction's complete synthetic CLI verifies the actual derivative, both source pins, retained nonzero outcome/original flag, canonical publication digest and prevention of a second execution. The source-level RED receipts in `VALIDATION.md` are attributed to the implementer, not independently re-created.

The added capture index is linear in capture references, and each projection handles a fixed number of fields. No introduced O(n²) pass was found. Existing output ceilings remain postconstruction checks, not a peak-memory guarantee. The scoped resulting docs accurately preserve original failures and separate source qualification from delivery, measurement and savings claims; no architecture/runtime documentation change is required for this offline helper.

Root owns the exact corrected input/scope freeze and any subsequent actual source qualification. This review admits no replay itself. No real-agent workflow is affected by this metadata-only derivative, so a new provider verification is neither required nor justified for this change.
