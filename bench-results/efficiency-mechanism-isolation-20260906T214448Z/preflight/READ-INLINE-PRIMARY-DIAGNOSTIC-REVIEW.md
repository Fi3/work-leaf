# V3 real read-delivery diagnostic review

Verdict: PASS for the bounded real-agent delivery/accounting scenario, not a benchmark
result or proof of whole-workflow token savings. Both corrected diagnostics finish with
`WORK_LEAF_READ_INLINE_SMOKE_OK`, three turns, one session and launcher exit 0.
The two earlier missing-primary-marker failures remain separately retained in
`READ-INLINE-REAL-DIAGNOSTIC-OUTCOMES.md`; no failed observation is replaced or retotalled.

Evidence: `READ-INLINE-PRIMARY-DIAGNOSTIC-REVIEW.json`, SHA-256
`96bc50d281826e0abc3c7005cf0c9b46d960a58cf2c182688aaea2bfc8d027d9`
(114,209 bytes; 44 source identities, all rehashed at closure). It contains complete
safe six-response attribution, four read boundaries, exact typed request/reply and
native item locators, and every retained accounting/scope limitation.

## Verified chain

The unchanged observer's `analyze --config <diagnostic>/observation/observer-config.json`
returns exit 0 for each closed capture. Its exact JSON output is retained create-new at
`observation/analysis.json`. `DIAGNOSTIC-REPORT-PROJECTION.json` explicitly copies this
diagnostic's model/effort and observed ledger for the accounting helper's report interface;
it is not an independently generated benchmark report. The accounting helper's exact
self-source pin is diagnostic-only in memory, not a phase admission manifest.

`analyze_untracked_reads.py::capture_provenance` and `prompt_inventory` validate
both capture byte streams and 3/3 accepted typed requests in each diagnostic:
unchanged policy, one eligible untracked read, and one unchanged repeated read.
Actual invocation settings are primary=true, raw response usage=true, grace=1000 ms,
resume=forward, project-layer inventory=true, with gpt-5.5/xhigh and CLI 0.153.4.
All six captured terminals are interrupted; grace reconciliation retains no unresolved
usage gap. Policy bytes and repeat-response bytes are identical within and across arms.
The eligible candidate transformation is confined to its renderer-owned component.
This real fixture has no patch-ACK or command-result exposure; those sites' identity
is covered by the separately reviewed automated tests, not asserted as real exposure.

`accounting_untracked_reads.py::audit_run` validates exactly three native/raw response
identities per diagnostic, no compactions, no tail recovery or missing-response bound.
Original observer and corrected scope are identical. `audit_input_attribution.py::audit_run`
validates all six responses' top-level items plus request fields: every signed residual
(input, cached, cache-write, output and reasoning output) is zero; nested content is not
counted twice. No whole-phase primary, contrast, p-value or hypothesis selection is run.

## Exact selected-input evidence

Each selected read string equals exactly one public native user-item body in its verified
thread. Its explicit native passthrough turn ID equals the typed accepted turn ID.
Raw attribution then joins the exact thread/item identity, never text similarity.

| Diagnostic | Native source line | Exact initial-read item ID | Charges in response 2; response 3 |
| --- | ---: | --- | --- |
| control | 22 | `msg_01a07a7e-c77c-7802-bddc-6ec213511e75` | 116; 116 input tokens |
| untracked-read-inline | 22 | `msg_01a07a7e-bcda-7450-a62f-5f543263b693` | 8,514; 8,514 input tokens |

These are observed charges of each entire selected user item, including framing.
They are not a snapshot-only token allocation, randomized contrast, net savings estimate,
or causal share of earlier benchmarks. The unchanged repeat item has 71 input tokens
in each diagnostic's third response. The JSON retains all exact response IDs and
physical source locators, including cache counts and separately charged request fields.

## Scope and source cutoff

The controlled smoke explicitly prohibits bundle opening and tool use and requests the
same read/read/done sequence. It validates delivery and measurement, not natural retrieval
behavior. Normal teardown removes the temporary bundle files, and this smoke does not
archive them. The control observer's mechanism subsection therefore retains its H1 snapshot
resolution and missing archived-bundle diagnostics. Capture/accounting errors remain empty;
no archived bundle byte-equality claim or natural-workload mechanism inference follows.

Reviewed source identities: smoke `8f80942005f3dc305cb29f8cef8ab8efe7b3e9604491f3fe5159f8e8557428ed`;
delivery analyzer `30a58a9312e2f9c641643698f53e0592c392fa8ed5fa3c0d01d9cc61ff427c3e`;
accounting `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`.
All remaining runtime, helper, capture and native hashes appear in the JSON.

Protocol SHA-256 `84fa9227ba9bdafa947a244b6611c8445db6a9eed6d83e34ccc009939582af49`:
its late-terminal recovery paragraph agrees with the implemented exact-identity,
fresh-additive-usage and no-later-boundary requirements, while retaining original grace
and excluding compaction/unusable-last turns. These diagnostics need no such recovery.

No provider was launched by this reviewer. No frozen helper, capture, metadata, old
report or protocol was edited; only new diagnostic projections and review artifacts
were created. The reviewed runtime sources are in root commit `9751fa4`.
