# Independent retained read-item recharge result review

## Scope and conclusion

The six saved results pass the independent source, identity, coverage, category, and per-item charge-join review. Five retain `exact_observed_attribution`; workflow 003 retains `unknown`. These are observed-response attribution statuses, not whole-workflow completeness claims or causal savings estimates. All failures and original phase statuses remain present.

Authority is [RETAINED-READ-ITEM-RECHARGE-SCOPE.revision-01.json](RETAINED-READ-ITEM-RECHARGE-SCOPE.revision-01.json), SHA-256 `b262d5bff186a80739dfcd90c73617d7d8ec2f7a002e92c2b3ab182773967224`. The parent executed the six declared CLIs once during 2026-09-07 18:04:55–18:05:00 UTC. This independent review reads only their saved results, attempts, and pinned sources. It makes **zero** `extract_records`, accounting, audit, or provider calls and computes no token totals, contrasts, or percentages.

The prior implementation qualification is [preflight/read-item-recharge/INDEPENDENT-REVIEW.md](preflight/read-item-recharge/INDEPENDENT-REVIEW.md), SHA-256 `78261f5c6f766c27299e04d6910074b36965e5b552bf6f13cd65831d595592b1`. The result review does not replace or rerun that qualification.

## Saved population and publication identities

Paths below share `phases/untracked-reads-01/postcapture/untracked-reads-01-workflow-<suffix>/`; result and attempt names are respectively `READ-ITEM-RECHARGE.json` and `READ-ITEM-RECHARGE-ATTEMPT.json`.

| Suffix | Saved status | Target items | Response IDs | Targets observed / not observed in completed-response attribution |
| --- | --- | ---: | ---: | ---: |
| 001 | exact_observed_attribution | 184 | 192 | 184 / 0 |
| 002 | exact_observed_attribution | 134 | 105 | 133 / 1 |
| 003 | unknown | 13 | 29 | 12 / 1 |
| 004 | exact_observed_attribution | 142 | 84 | 140 / 2 |
| 005 | exact_observed_attribution | 22 | 117 | 22 / 0 |
| 006 | exact_observed_attribution | 15 | 80 | 15 / 0 |

Exact result SHA-256 identities:

```text
001 ffc4eed18025409c9ca01871265549b48ec22573475af9e90b0ddf8d4982b500
002 29ac450fd333ce71a4ab73816ca2fe9a0c55192df9ecb3781c0f4da5924a573f
003 fd176cdf78263d55804c1b647f5434ab451456306d660358b16368e15ef9203f
004 7a3502a8889880dc0615e6f07c5f3f4bf04bbef83fb200b123234261ee1713b1
005 ecbef8ef12b8b2075fe6704da0ff2594c73bf4a2ffb217d5a00591b8e5ae59c4
006 8fd92733fed2028ed0f60712bf4e91cf483b306e1fd9ca4a7f9a19ea5243e3e7
```

Exact attempt SHA-256 identities:

```text
001 4ac6c6f84da604102173026c7c1312e964bff5c7cc9b8eff7107aa9176bc6d38
002 1857ff1ab03f8063453b74df1bd73d0747a1e42b763df7c64942ce6f40d886f4
003 9fa378560cbdaa1bb98b4ddd2943a6945fdfe0babc00fdf63f11a19308850f29
004 564ed3664a357c2da7e2bc111af206e272dd762b10520fccebc2ffa44aa50254
005 ec1b78b2ca84d287bd0a78968c0036d4055e480aa3d80bb11097d5e9dbf22c4f
006 2aee815089fc800ac3fdcac1dd33d01c199adcc37f76ee859f6a0572ab4b7e08
```

Every attempt retains `state: attempt-started`, the exact scope/run identity, and `retry_authorized: false`. Each saved result names its matching attempt/hash, reports one extraction, no provider or whole-workflow audit calls, and matching source endpoints. `original_run` and `preserved_status` match the declared row exactly. The six source-union counts are respectively 175, 168, 172, 170, 177, and 174; independently reconstructed union hashes match their saved identities.

## Independent checks

The bounded verification completed at approximately 18:09 UTC, exit 0, with `PASS` (tool receipt `605bd7`; elapsed 1.321 seconds). Its checks are:

- All 356 declared source hashes and all 12 result/attempt hashes match before and after review; scope bytes are unchanged.
- Exactly the six declared workflow IDs survive, with no replacement or omitted failure. Qualified response-ledger hashes and counts match the frozen scope. All 607 response IDs are globally distinct.
- Each qualified response's exact native source and physical line identifies the same response/thread/turn and four integer usage counters. Every counter matches individually; no counter sums or new ledger totals are computed.
- Raw `rawResponse/completed` evidence agrees on response/thread/turn, usage, attribution hash, source line, first physical occurrence, and duplicate count. Duplicate raw occurrences require equal identity/usage/attribution; response-set differences match saved coverage rather than disappearing.
- All 510 target `(thread, item)` identities are distinct and match exact native source, physical line, explicit turn, and canonical payload hash. All 86 declared read indices and 424 retrieval-candidate indices have their retained associations. This is per-item ownership, not a count or charge per bundle.
- Actual read delivery metadata reproduces 34 eligible bundle-manifest deliveries, 42 eligible inline-candidate deliveries, and 10 other/ineligible deliveries. Classification uses bundle publication and eligibility metadata, not a selected-condition tag alone. Trace/sequence/selection, snapshot classes, hashes, and bundle identities match saved source records.
- Retrieval classifications, call identities, path references, witness lists, and complete-path lists match the retained retrieval review.
- All 6,949 saved item/response charge records match their original raw attribution entry individually: thread/item/response, turn, source/physical line, input/cached/cache-write fields, occurrence order, and exact/unknown flag. `(thread, item, response)` is unique. Reverse coverage verifies every positive raw input-attribution entry for a target has exactly one saved charge record; shared bundle references do not multiply charges.
- Saved item coverage and charge exactness agree with those joined records. Targets without an observed completed-response charge retain their coverage and null exactness rather than becoming measured zeroes.

The recorded full-extractor-result digest/length and projection metadata are internally consistent. This review deliberately does **not** reconstruct that full extractor result by rerunning extraction; independent assurance here comes from the frozen implementation review and the direct saved-record/source joins above.

## Workflow 003: exact retained uncertainty

The sole unknown response is `resp_09b466b7761d5b41016a9e742c230487d289767a995c683721`, in thread `01a07aee-59c8-7761-9ed2-a0b65b5bf88a`, turn `01a07af5-c9b3-75f1-bf6a-d5543eae214f`.

Raw witness: `phases/untracked-reads-01/runs/untracked-reads-01-workflow-003/untracked-reads-01-workflow-003-three-feature-bench-artifacts/observation/app-server/00000467799012547114-183422/server-to-client.raw`, physical line 22229. The attribution SHA-256 is `8d023d53fac0956c9dddf05482e333c692a13d6d03404fa62d590d82ee2bbe59`.

Its retained errors are `unmatched/unsupported output item scope: tsc_09b466b7761d5b41016a9e7440b1f087d2a86f4766085223fd` and `attribution does not exactly reconcile to recorded usage`.

Native witness: `/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T10-13-56-01a07aee-59c8-7761-9ed2-a0b65b5bf88a.jsonl`, SHA-256 `80dc86990ed615fe1ac9735774aa85e2f4581b7cc731023c025443273e4b66f9`, physical line 41. Its public output metadata type is `tool_search_call`, with the exact `tsc_...` item ID and the same explicit nested turn. Canonical payload SHA-256: `bf7bd5997bd75596d0f86fea8d9d979bd48910742ce269c27770521f0e4de95b`. No arguments, private reasoning, or message bodies are reproduced here.

The frozen [audit_input_attribution.py](audit_input_attribution.py) (`ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740`) excludes `tool_search_call` from `NONREASONING`; `extract_records` therefore leaves that output's reasoning split unverified. Saved input, output, cached-input, and cache-write residuals are each zero; the reasoning-output residual is null. Consequently the generic reconciliation error does **not**, by itself, demonstrate a numerical counter discrepancy. The unsupported output-action classification and associated false exactness flags remain intact; nothing is waived, reclassified, or re-extracted.

## Completeness and inference limits

Four target items have no observed completed-response attribution: workflow 002 `/evidence/items/23` (retrieval output), workflow 003 `/evidence/items/6` (direct read input), and workflow 004 `/evidence/items/5` (direct read input) plus `/evidence/items/88` (retrieval output). Their native identities and source hashes match; they remain retained, not priced at zero. A run's `exact_observed_attribution` status can coexist with such an unobserved target.

The evidence establishes exact observed item-charge associations where classified, with the explicit unknown above. It does not establish whole-workflow coverage, missing-tail usage, byte-identical counterfactual state, causal attribution to bundle representation, or a percentage contribution to savings. Repeated item charges are observed entries, not independent experiments. No normal runtime, provider settings, frozen helper, scope, source, result, or attempt file is modified by this review.

## Independent summary and prose check

The subsequent [complete summary](EVIDENCE-RETAINED-READ-ITEM-RECHARGE.json), SHA-256 `800c181350b9b7c42adfe398fbb3463391b036ac9d910a365e24c2a7c703df9b`, and [explanation](EVIDENCE-RETAINED-READ-ITEM-RECHARGE.md), reviewed SHA-256 `b49d460094db1b0da216f0c694f4e8ccb6161cd36adba0b0b32ce8d240cc828d`, also pass. This bounded second pass derives only the expressly declared item/category ranges and coverage from the saved six receipts; it performs no extraction, whole-workflow accounting, total, contrast, or provider call.

All per-run category buckets, exact/unknown/not-observed partitions, first and later input/cached/cache-write ranges, response-count ranges, and repeated-item counts independently reproduce. The 510 targets comprise 506 observed and four not observed; 497 have multiple charge records. Among observed targets, 505 have only exact responses and one includes the retained unknown response. Every observed item has a constant input charge across its own recorded appearances; this does not require a constant cached portion. The seven prose category rows agree exactly:

| Category | Targets / observed / repeated | Exact observed input range | Mixed-unknown observed input range |
| --- | ---: | --- | --- |
| Eligible bundle manifest | 34 / 33 / 33 | 147–357 | None |
| Eligible inline candidate | 42 / 41 / 36 | 5,672–58,371 | 58,666 |
| Other/ineligible read | 10 / 10 / 9 | 72–7,031 | None |
| Selected-content retrieval | 419 / 417 / 414 | 68–12,941 | None |
| Failed retrieval | 3 / 3 / 3 | 61–105 | None |
| Executed search, no content | 1 / 1 / 1 | 43 | None |
| Executed selection, no content | 1 / 1 / 1 | 44 | None |

All seven deterministic examples are independently reconstructed as the first fully qualified target in frozen run-ID order and that category's delivered read/candidate order. Their complete saved association, native locator, first/later response identities, count, and charge ranges match. In particular, the first R001 manifest is 249 input tokens in 26 observed responses and its first selected-content retrieval is a distinct item at 2,893 input tokens in 25 responses. Their shared raw line 224 is the same source and response ID, with separate item entries. This verifies the example without selecting a favorable later event.

The prose explicitly keeps different requested files and natural workflow ranges separate from a matched same-content intervention. It distinguishes item charges from tool-call/response counts, retains unknown and missing-tail limits, does not allocate an output among referenced archives, and makes no percentage or historical causal-share claim. No blocking factual or scope finding remains. The independent summary verification is `PASS`, tool receipt `647981`; summary source bytes and six result hashes remain unchanged.
