# Existing work-unit policy: current common-scope token direction

## Result

All twelve saved W workflows have validated response-identity accounting in this
separately declared current scope. Eleven have finite raw-token bounds. W003 has
a validated recorded ledger but one unsupported response tail, so its upper bound
remains unbounded. No workflow is excluded, including failed control W010.

The resulting treatment-minus-control **mean total raw-token interval** is
`[-21,027,146 / 6, +∞)`, or approximately **[-3,504,524.33, +∞)** tokens per
workflow. It establishes neither an increase nor a reduction for the incremental
work-unit policy. It is not a sampling confidence interval.

The recorded-ledger means are 15,676,675.33 for incremental policy and
14,861,866.33 for control, a recorded difference of +814,809. That subtraction
of recorded lower bounds is **not** a bound on the actual total difference.
There is no midpoint substitution, finite ceiling assigned to the unsupported
tail, percentage attribution or new p-value.

## All retained raw bounds

Raw means input plus output, including cached input; reasoning is an output subset.
Each row is a complete retained top-level workflow attempt, not an agent or response.
An upper value marked unbounded is not zero or a missing workflow.

| W suffix | Original condition | Exit | Raw lower | Raw upper |
| --- | --- | ---: | ---: | ---: |
| 001 | incremental policy | 0 | 12,788,930 | 13,966,930 |
| 002 | control | 0 | 12,288,138 | 18,178,138 |
| 003 | incremental policy | 0 | 19,888,093 | unbounded |
| 004 | control | 0 | 15,514,170 | 21,404,170 |
| 005 | incremental policy | 0 | 14,762,773 | 18,296,773 |
| 006 | control | 0 | 13,875,604 | 15,053,604 |
| 007 | incremental policy | 0 | 15,104,799 | 17,460,799 |
| 008 | incremental policy | 0 | 15,760,054 | 26,362,054 |
| 009 | control | 0 | 16,236,327 | 19,770,327 |
| 010 | control | 1 | 15,653,269 | 19,187,269 |
| 011 | incremental policy | 0 | 15,755,403 | 22,823,403 |
| 012 | control | 0 | 15,603,690 | 21,493,690 |

| Original group | Rows | Total raw lower | Total raw upper | Mean raw interval |
| --- | ---: | ---: | ---: | --- |
| Control | 6 | 89,171,198 | 115,087,198 | [89,171,198 / 6, 115,087,198 / 6] |
| Incremental policy | 6 | 94,060,052 | unbounded | [94,060,052 / 6, +∞) |

Finite upper bounds use the unchanged helper's documented per-response limits and
its supported isolated-normal-response-tail predicate. A model limit alone does
not establish how many responses lie in a gap. The frozen interpretation remains
conditional on the observed boundary scope; it is not an exhaustive hidden-call
guarantee.

## What the separate scope resolves

W003's original observer ledger remains **19,654,702** raw tokens. Its new named
common-scope ledger is **19,888,093**, including exactly once the previously omitted
compaction response `resp_06dd34dd4bcd978a016a9e11d96e9487d2a73f239eca987fc2`:
228,168 input plus 5,223 output equals **233,391** raw tokens. The unchanged helper
requires its exact native/raw response identity, additive usage, matching captured
compaction lifecycle and cumulative reconciliation. This is an accounting-scope
correction, not additional generated work or a saving mechanism.

The separate nonadditive `tokenUsage.last` at original server line 63,486 remains
a warning: component counters are zero while `totalTokens` is 38,663. That
notification is neither billed response usage nor a proven context-size estimate.
It is not used for fresh-usage or tail recovery. Exact original warning, lifecycle
IDs and source locators remain in the new JSON.

The helper also retains one independently established late terminal recovery:
response `resp_06dd34dd4bcd978a016a9e0f0f110887d28e6fe8a986cfbdc3`,
raw line 61,662, fresh usage 61,663 and terminal 61,667. Its original grace outcome
`forwarded-after-output-resumed` is preserved; no elapsed-time inference or
additional response charge is applied.

A different gap remains unsupported: thread
`01a07949-ee9c-7503-a486-0108cc6bb00d`, turn
`01a07973-5e12-77b2-9bb3-619c47929a3b`,
reason `isolated response tail not established`. Neither resolving compaction
scope nor proving the other late completion establishes this gap's response
cardinality. W003 therefore retains `unbounded_accounting_gap`.

## Frozen scope, provenance and reproducibility

[COMMON-W-ACCOUNTING-SCOPE.json](COMMON-W-ACCOUNTING-SCOPE.json), SHA-256
`5fca3fa4d06d7dc91790e5a39d6b32c75155412974277259fcff462cab52125d`,
was saved before inspecting totals. It explicitly includes all twelve original
IDs and forbids exclusions, historical-report edits, new observations and
confirmatory reinterpretation.

[COMMON-W-ACCOUNTING.json](COMMON-W-ACCOUNTING.json), SHA-256
`893aceede9e03c67c0841cabb7b04c7c5fbd36911c7f396ff7cc6a0c675ce284`,
contains every result/outcome, original observer diagnostics, corrected ledger,
bounds, gaps, warnings, complete source-hash map, and exact helper-result digests.
Bulky duplicated response-record and proof-line arrays have canonical-JSON
SHA-256/count identities; their original raw/native files and unchanged helper
reproduce them. No rewritten provider stream is an evidence source.

The replay found **2,392 unique recorded response IDs**, no cross-workflow
duplicates and **230 distinct source identities**, all rehashed at closure
2026-09-07T12:34:59.172477+00:00. Full source-map SHA-256 is
`f5efac0c436fa5818676c2f9d9101f5464a855eac14eb3c1c1330e8078975cfb`.
This is the exact reconciled recorded response population, not a claim that every
missing or hidden provider call has been observed.

Execution used exact-compiled frozen
`candidate-screen-01/infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z/accounting_untracked_reads.py`,
SHA `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`,
and its three unchanged pinned dependencies. Each call was
`audit_run(original_W_score_entry, candidate_frozen_manifest, Path("/home/user/.codex/sessions"))`.
The candidate manifest hash is
`935a32719e0c91e2afb8fc101788db44bb955a3d85993d2c28327cc44325acb1`;
it supplies current helper identity only, not a new enrollment of W observations.
Original W manifest
`5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`
and score-manifest entries remain unchanged.

Workflow/source/config admission is a separate caller obligation, grounded here in
[PREFLIGHT-CANDIDATE-BASELINE.md](PREFLIGHT-CANDIDATE-BASELINE.md) and the original W
closure/source/mediator records. No global configuration contents or credentials
were copied, and no active candidate result or parked-read cost was inspected.

## Original mediator interpretation remains intact

The [independent original mediator review](phases/work-units-01/PHASE-MEDIATOR-REVIEW.md)
retains its inclusive one-sided **p = 5/12**, with the primary not rejected and the
fixed-sequence secondary untested. Actual policy exposure does not turn the
recorded-ledger difference into confirmation of a policy-to-handoff-to-token
mechanism. This current descriptive accounting scope does not retest, replace,
refund or relabel that endpoint.

The original `phases/work-units-01/WORK-UNIT-TOKENS.json`, SHA
`424fe4701e5407fdbf9f37ff87273b475e518c56377ae4d71cabe397443a7ae7`,
retains its old strict-bridge failure. Its failure is not silently removed.
These new separately named files close the supported common-scope accounting
facts while keeping the unresolved tail explicit. No quality gate, provider call,
new control, Direct run or percentage claim belongs to this replay.

