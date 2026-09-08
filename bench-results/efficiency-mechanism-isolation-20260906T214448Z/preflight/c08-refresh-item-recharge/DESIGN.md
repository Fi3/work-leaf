# C08 fixed full-refresh item attribution

This pure offline adapter qualifies observed whole-input-item charges for exactly the 11 full automatic refreshes already identified in `TRACE-OWNERSHIP-REVIEW.json`. Its population is 001 T4/T9, 002 T4/T13/T19/T23/T28/T31, and 003 T4/T9/T14. It does not compare workflow costs, read baseline measurements, collect sources, launch providers, or claim a file-body substring price.

Status: synthetic qualification only. Actual input preparation, a root-reviewed source/limit/one-attempt declaration, a qualified lower-response result, and root execution are required before any saved item extraction. There is no standalone CLI or automatic invocation.

## Materialized interface

`prepare(trace_report, source_results, lower_result, native_rows, raw_rows, target_payload_hashes)` returns three ordered prepared objects. It performs no extraction or workflow arithmetic.

- `trace_report` is the exact existing ownership JSON, with all three run records and every eligible/noneligible record retained. No R-read or bundle schema is fabricated.
- `source_results` maps the three run IDs to their exact `SOURCE-RESULT-TYPED-END.json` objects. Successful frame/stream/native/project subproofs and the two known ordinary trace ambiguities are required. Every accepted input is indexed against the complete explicit context population.
- `lower_result` is the reviewed, qualified lower-result object. Only global/candidate qualification, the complete candidate `response_evidence` maps, count/hash identities and retained errors/gaps are consumed. Recorded-use totals, bounds and baseline records are not interpreted or recomputed.
- `native_rows` maps each of the six selected authors' exact source paths to the complete ordered `(physical_line, decoded_row)` sequence. The existing explicit-turn inventory sees all accepted turns, including failed/no-usage turns, and all named native items; it never uses last-context inference. Other threads remain in upstream accepted/source and lower-map coverage but need no native item inventory.
- `raw_rows` maps each run to every original `rawResponse/completed` event in its one qualified app-server capture, with `_audit_source` and `_audit_line`. Input assembly preserves the original physical lines and complete metadata; it must not supply only item hits.
- `target_payload_hashes` is a separate 11-row list of `{run_id, trace_line, payload_sha256}`, frozen from the actual native payloads. It does not modify the ownership report. Canonical hashing follows the unchanged recharge convention (sorted compact JSON, ASCII escaping), whereas the existing `input_sha256` hashes full UTF-8 text.

The caller owns canonical path/SHA checks, bounded closed reads, full source/terminal/response-map provenance, the exact input population, pre/post endpoint verification, one-attempt reservation, and exclusive bounded publication. These pure functions do not turn supplied dictionaries into independent file provenance. The future caller should reuse the already qualified source inputs/results, not execute another collector.

## Target and response identities

Each target must exactly equal its accepted source input; its native item must be a `message` with `role=user`, matching the physical locator, explicit turn, complete payload hash and complete text hash/byte count. Renderer-owned body ranges must be ordered, nonoverlapping UTF-8 ranges with their saved byte/hash identities. The native item ID, not the distinct public user-item ID, is the attribution key.

The category is `automatic_refresh_input`. It is not `direct_read_input`; the adapter owns the explicit user-kind check, while the unchanged `validate_targets` supplies generic locator equality.

Per author, the response window begins after its earliest target's exact public user-item line and extends through the supplied closed capture. Every later completed response is retained, even with no target hit, missing attribution or an unknown response ID. Malformed unassignable records after the earliest relevant boundary remain unknown. An owned response ID carrying a contradictory thread cannot disappear through filtering. The native response map uses the corresponding post-introduction native window, including later compactions, and exact native response-ID/thread/turn/four-counter witnesses.

`extract(prepared_run, max_rows=100000, max_bytes=33554432)` calls only unchanged `load_extractor(...)(servers, native, ledger)` once, then unchanged `project`. It preserves the full canonical extractor hash/size before projection. Missing native/raw identities, duplicate/conflicting metadata, signed residuals and per-response exactness remain visible. Projection rejection returns unknown with no clipped evidence and retains the full extraction hash; it does not authorize another call.

A target charged before its own exact introduction is marked unknown, including every projected charge from that response. The original frozen-extractor hash remains the hash before this explicit temporal qualification. A target with no positive observed charge remains `not_observed_in_completed_response_attribution`, not zero lifetime cost. Missing raw attribution and re-identification through compaction cannot establish absence of retained context.

## Immutable code dependencies

- `preflight/read-item-recharge/recharge.py`: `4d6ac35d6bb857e6c34564020f035f6fdaf4a7f43fbe8d4e8d682fb78990ad0c`.
- `audit_input_attribution.py`: `ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740`.

The adapter reuses `native_inventory`, `load_extractor`, `validate_targets` and `project` without edits. It does not call the old six-R execution driver, `targets()`, `native_item_inventory`, `dependencies()`, `audit_run`, a compaction auditor, or the lower-bound arithmetic. All top-level attribution items and request fields within each selected response remain present for exact reconciliation; nested content counters are not counted again.

## Limits and interpretation

There are constant-pass identity/hash joins over the supplied rows and bytes, plus canonical-key/identity sorting. No per-response native-file search, pairwise history scan, per-body token allocation or quadratic production join is introduced. The two-author-per-run minima are fixed small operations.

The row/byte ceilings govern the unchanged full/projected evidence admission checks after construction, not peak memory or source-file loading. The future caller must also bound the complete published success/failure envelope and source reads. Failure metadata is not a clipped successful projection.

Repeated positive attribution establishes that the same recorded whole refresh-input item incurred observed input/cache/write charges again. It does not separately price its larger file-body component, prove identical byte retention after compaction, or establish net savings. All original workflow failures, missing usage, unbounded endpoints and ordinary trace ambiguities remain independent qualifications.
