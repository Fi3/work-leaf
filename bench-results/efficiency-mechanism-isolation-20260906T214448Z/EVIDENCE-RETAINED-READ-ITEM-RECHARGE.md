# Retained read content and repeated input charges

**Bundle deferral reduces the content placed in the initial read message; it does
not exempt subsequently retrieved content from retained-input charges.** In the
six saved R workflows, 506 of 510 identified read/output items appear in completed
response attribution; 497 appear in more than one response. Each observed item's
input-token charge is constant across its recorded appearances, although its cached
portion can change. One item's appearances include a separately retained unknown
response; four items have no observed charge, not a zero lifetime cost.

The [complete summary](EVIDENCE-RETAINED-READ-ITEM-RECHARGE.json), SHA-256
`800c181350b9b7c42adfe398fbb3463391b036ac9d910a365e24c2a7c703df9b`,
retains every run, category and predeclared deterministic example. All 510 complete
per-target charge lists are in the six source-bound `READ-ITEM-RECHARGE.json`
receipts linked by that summary. The [fixed scope](RETAINED-READ-ITEM-RECHARGE-SCOPE.revision-01.json)
and [independent pre-execution review](preflight/read-item-recharge/INDEPENDENT-REVIEW.md)
are committed at `48629d2`. This is local channel evidence, not another whole-workflow
total, comparison, percentage or historical causal allocation.

## Exact mechanism and its offset

The frozen R owner is
[`send_file_read_response`](phases/untracked-reads-01/infrastructure/evidence/src/orchestrator.rs:997)
→ [`render_file_read_response`](phases/untracked-reads-01/infrastructure/evidence/src/orchestrator.rs:2111)
→ [`render_bundled_file_read_response`](phases/untracked-reads-01/infrastructure/evidence/src/orchestrator.rs:2365).
The bundle contains exact held source text; the ordinary delivered message contains
its path, file/digest inventory and access guidance. The treatment substitutes the
exact inline candidate at that already-qualified span, retaining ordinary archive
creation and other delivery factors. [Source/retrieval evidence](EVIDENCE-RETAINED-READ-MECHANISMS.md)
establishes actual archive bytes, issued inputs and selected returned content.

The observed channel has three stages:

1. A small manifest or full-inline read enters the conversation as a user input item.
2. Native retrieval, when exercised, places returned text in a separate tool-output
   item. The tool output is not the entire archive merely because its command names
   that archive; multiple references and exact content witnesses remain separate.
3. Each item included in subsequent recorded requests contributes its own input
   charge again. Cached input remains included in the study's raw-token metric.

The unchanged [`extract_records`](audit_input_attribution.py:105) joins completed
raw responses to exact native response identities/counters and attributes top-level
items without double-counting their nested content. The new adapter only supplies
explicit-turn item identities and projects the already recorded whole-item charges.
Repeated charging is a common cost channel, not another independent saving factor.

The offset is concrete: selected retrieval output returns content to retained
context, and failed/no-content retrievals also leave charged result items. Retrieval
actions and continuations may add further work; this report does not allocate action
or response costs to individual archives or equate one tool call with one response.

## Complete item coverage

Counts below are items, not workflows or model responses. Repeated means at least
two observed charge records for the exact same thread/item identity.

| Delivered category | Targets | Observed | Repeated | Input tokens per observed item appearance |
| --- | ---: | ---: | ---: | --- |
| Eligible bundle manifests | 34 | 33 | 33 | 147–357, exact observed attribution |
| Eligible full-inline read inputs | 42 | 41 | 36 | 5,672–58,371 for 40 fully exact items; 58,666 for one item whose four appearances include the flagged response |
| Other/ineligible read inputs | 10 | 10 | 9 | 72–7,031, exact observed attribution |
| Selected-content retrieval outputs | 419 | 417 | 414 | 68–12,941, exact observed attribution |
| Failed retrieval outputs | 3 | 3 | 3 | 61–105, exact observed attribution |
| Executed search with no content | 1 | 1 | 1 | 43, exact observed attribution |
| Executed selection with no content | 1 | 1 | 1 | 44, exact observed attribution |

These ranges span different actual items, requested files, phases of work and
workflows. They are **not matched same-content intervention differences**. The
earlier controlled 116-versus-8,514 diagnostic remains its own same-source local
comparison; these natural-workflow ranges do not replace it or inherit its controls.

## Deterministic example, not a selected favorable comparison

The scope chooses the first fully qualified target in run-ID/delivery order for
each category. R001's first eligible manifest, native user N22, is charged **249
input tokens in 26 recorded responses**. Its first selected-content retrieval,
native tool output N29, is a distinct item charged **2,893 input tokens in 25
responses**. At raw S224 both items are present and charged separately. Later
cache hits change the cached portions, not these raw input charges.

The example's exact thread is `01a07add-961f-7793-b8ad-57061056d4c5`; manifest
item is `msg_01a07add-c09d-7c20-a884-0d436f4f3afc`, retrieval item
`fco_01a07add-fd0a-7721-928a-7be2e9aaa518`. The complete native/raw paths, hashes,
turns and response identities are in `deterministic_examples` of the summary.
This establishes retained retrieval cost, not the unobserved full-inline alternative
for that same decision sequence. R003's first qualified inline example has different
requested files and is not a matched comparison to R001.

## Closed execution and retained limitations

All six once-only CLI calls ran at 18:04:55–18:05:00 UTC. Each invoked the unchanged
extractor once; all published results and attempt markers are retained. The
[execution receipt](preflight/read-item-recharge/CLI-EXECUTION.json), SHA-256
`de68bdcce363e1ed054623f43046a605274620e0c0ace97867b34f1246be8f2c`,
records every original exit and output identity. All 356 scope sources match at
final consolidation, and all 607 qualified response IDs have raw coverage.

| R row | Recorded response identities | Attribution result |
| --- | ---: | --- |
| 001 | 192 | Exact observed attribution |
| 002 | 105 | Exact observed attribution |
| 003 | 29 | 28 exact responses; one retained unsupported-output-item scope |
| 004 | 84 | Exact observed attribution |
| 005 | 117 | Exact observed attribution |
| 006 | 80 | Exact observed attribution |

R003's original UNKNOWN concerns native `tool_search_call`, which is outside the
unchanged helper's recognized nonreasoning action kinds. The response retains
zero input/output/cached/cache-write residuals but an unknown reasoning residual;
the full exact-attribution assertion remains failed. No helper/predicate change,
waiver, replacement extraction or revised workflow accounting follows this fact.
The corresponding read item's `response_exact: false` record stays visible.

The four unobserved targets are one R002 retrieval output, one R003 inline input,
one R004 bundle manifest and one R004 retrieval output. Complete tail item coverage
is not proved. Original workflow failures, configuration/observer/controller and
source-membership exceptions remain linked through every execution receipt and
the pinned provider-ledger scope. Six unlaunched phase identities remain unlaunched.

The [closed qualified workflow comparison](EVIDENCE-RETAINED-READ-PROVIDER-LEDGER.md)
still does not demonstrate the proposed positive net increase from full-inline
reads. These item charges explain why smaller handoffs alone cannot establish that
increase: returned content and the subsequent decision/response sequence matter.
They neither overturn the accepted historical endpoint nor complete its allocation.

This is offline evidence processing. No provider, new control, Direct workflow,
normal runtime, public API or agent-facing behavior changes in this analysis.
