# Typed end-record projection

Status: diagnosed source-contract mismatch; separate correction and synthetic
tests are described in [VALIDATION.md](VALIDATION.md). The original three
`SOURCE-RESULT.json` failures and admitted helper
`d05e7e7daa62601782fbcca2058f8c31a81d29ac5301407643474a09f4889f80`
remain unchanged. This draft authorizes no actual source-audit replay or
accounting/provider work.

## First failed predicate

`collect_closed_captures` compares `logged.end` to the complete raw `end.json`
before appending the invocation. For each run's first app-server invocation,
direct metadata inspection (`196012`) found no differing common fields and no
ledger-only field. Raw end has exactly two additional fields:
`raw_response_usage_sha256` and `raw_response_usage_start_sha256`.

| Run | First invocation | Original raw end SHA |
| --- | --- | --- |
| 001 | `00000568115683071173-564` | `a1a382eb6ef5c8be553cacd922d3f46a194281a7d4e190d9a84c4604065c70b1` |
| 002 | `00000568115683071173-565` | `2e217065b19a7c603488d7c6be42b8921557aa35bb2bf848c247e8aa8e95ffc8` |
| 003 | `00000568115683074735-566` | `d5c8f865139c610382b9d918c710003913e61ee12339b482deff9f607f8a2b3d` |

Each physical path is the corresponding scheduled artifact's
`observation/invocations/<ID>/end.json`; the exact source refs remain in its
original `SOURCE-INPUT.json`. The empty invocation/frame result is caused by
this introduced comparison, not evidence that the natural workflow had no
capture or failed delivery. Downstream orphan errors remain original outputs,
not independent evidence of missing native inputs.

## Source authority

`bench-observer/src/lib.rs` SHA
`188a1b4fd9913556c51024dcc0068be353813c44c688d6d22929da139f392b5f`:
`InvocationEnd` at340 has eight typed fields. `run_proxy` at646 passes that
value through `raw_capture::end_metadata` and writes the extended value to raw
`end.json` and `meta.json`. `ProcessInventoryRecord` at4176 instead contains
`Option<InvocationEnd>`; `read_process_inventory` at5052 deserializes that typed
record, and `analyze` at4270 publishes it to `process-invocations.jsonl`.

`bench-observer/src/raw_capture.rs` SHA
`a8397fa90fa8538cc74559c850401314bbb68726e0f1db03724133368c115a4a`:
`end_metadata` at31 appends exactly the two fields when raw capture is enabled;
the hash map contains three raw artifacts plus the grace journal when enabled.
`verify_capture` separately validates the start hash and exact map population.

Thus the typed ledger is not supposed to contain the raw supplements. The
unchanged old collector already compares selected typed end fields and checks
the raw-start/forwarded hashes separately; the new wrapper's whole-object
comparison introduced the failure.

## Separate source derivative

A separate helper keeps all original source and output bytes immutable. Only
the typed end projection admits the two source-owned supplement fields. It
requires the exact eight typed keys, rejects unrelated extra keys, and binds
the supplement start hash and exact four-entry raw/grace map to the declared
capture refs. Raw `end.json` and `meta.json` remain full objects; existing
raw-byte, source, frame, model and delivery checks remain unchanged. No
supplement is permitted without the raw-enabled app-server marker.

The separate file's dependency base must remain the original helper directory;
its own new source identity and new input/scope/output paths are explicit.
This is a prospective corrected source qualification, not an automatic retry
or a correction of any original observer/accounting result.

`test_source_projection.py` reuses byte-pinned original temporary fixtures but
not actual workflow records. Its six methods cover the original failing
projection, retained successful frame invocation, changed shared fields,
missing/wrong supplement hashes, unknown fields/map entries and unmarked raw
supplements, exact nonraw ends, and a complete corrected synthetic CLI with its
own source pin and no second execution. The initial reviewed draft and original
behavioral RED are retained in the validation receipt. Actual corrected inputs,
scope and one-time execution remain root-owned and separately authorized.
