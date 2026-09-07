# Retained read-item recharge evidence

This private offline adapter measures observed input charges of already identified native items. Its fixed population is the six actual R workflows in run-ID order: 86 accepted read user items and 424 retrieval-candidate output items. The prepared [scope revision](../../RETAINED-READ-ITEM-RECHARGE-SCOPE.revision-01.json) owns source pins, output paths and preserved outcomes. Root review and explicit execution authority precede actual extraction; preparation uses no retained-run attribution calls.

## Evidence boundary

`recharge.py` exact-byte loads the unchanged `audit_input_attribution.py::extract_records`, SHA-256 `ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740`. No `dependencies`, `audit_run`, compaction audit, strict accounting or whole-workflow arithmetic is called. Existing `PROVIDER-LEDGER-QUALIFICATION.json /response_evidence` supplies the previously qualified response ledger. The driver verifies its canonical identity and each response's exact native thread/turn/response identity and four original counters without summing workflow totals.

The frozen `native_item_inventory` is not used. Its latest-context inference is incompatible with saved developer/input messages that precede their matching `turn_context`. The new inventory uses explicit direct or nested passthrough turn IDs only, requires agreement when both occur, and requires membership in **all accepted input turns**, including failed and usage-less turns. No item-prefix, text, latest-context or call-ID inference supplies a missing item/turn identity. Missing, boolean, conflicting or unowned identities fail qualification.

Each target must match its saved native source path, physical line, canonical payload hash and turn. Native bodies are hashed but never exported or semantically inspected here. The projection carries allowlisted identities, locators, retrieval classifications/associations and counters only. Full message, command argument, file, private reasoning and credential bodies are absent.

A direct read item contains the complete delivered input, not a priced snapshot substring. The 86 read inputs contain 76 eligible deliveries (34 bundle manifests, 42 inline candidates) and ten other/ineligible deliveries; eligibility, actual saved bundle-write metadata and snapshot classes remain distinct from the selected-candidate tag. A retrieval output is charged once per response as one whole native item even if its command references multiple archives. The 424 candidates retain 419 selected-content and five failed/other dispositions. Referenced archive paths and specifically witnessed/complete paths are separate associations. A failed retrieval output remains a failed candidate; unlinked read references remain unlinked. Neither a mere path substring nor an output witness from archive A prices archive B.

## Execution and retention

`execute_recharge.py` has one operator-owned CLI. The root agent owns all six actual calls; no sibling agent executes them. After final review, use the exact final scope SHA:

```sh
python -B preflight/read-item-recharge/execute_recharge.py \
  --scope RETAINED-READ-ITEM-RECHARGE-SCOPE.revision-01.json \
  --scope-sha256 <reviewed SHA-256> \
  --run-id untracked-reads-01-workflow-001
```

Continue only the same fixed rows 002–006, once each. The driver resolves `apply_patch` and checks the declared output/attempt paths before execution. It creates a retained attempt marker before source/charge replay, then publishes the result create-new through `apply_patch`. An existing result or attempt rejects a second CLI call. This assumes one trusted operator and no concurrent invocation of the same row; it is not atomic multiwriter or crash-proof storage. A failed publication does not authorize a transparent rerun. Operator execution failure receipts remain necessary if a process ends before publishing its result.

For each row, the driver verifies common inputs and the union of the existing source-delivery/retrieval source maps before/after. Raw response frames retain physical source lines and one capture per thread, so first/subsequent charges mean observed response order within that item’s thread, not lexicographic response-ID order. It inventories complete raw frames, deduplicates through the unchanged extractor, preserves conflicting/unknown responses, and reports exact qualified-ID versus raw-ID coverage. Native records without raw item attribution and all unknown tails remain explicit; there is no invented item charge for them.

The full canonical extractor-result hash is recorded before projection. Both full and projected evidence have 100,000-metadata-row and 32-MiB admission ceilings per run; source files are bounded at 512 MiB per read/hash. The result ceilings are postconstruction checks, not a general peak-memory or workspace resource bound. Exceeding a bound fails rather than clipping. No extraction retry or predicate change follows a difficult result. First-attribution absence is `not_observed_in_completed_response_attribution`, never zero lifetime cost. Charges whose response fails the unchanged full reconciliation retain `response_exact: false` and signed/unknown residual evidence.

The report retains original workflow failures, controller unavailability, original observer/configuration and source-membership exceptions by exact pinned references. The six unlaunched phase identities remain unlaunched. No stopped randomized primary, new control, provider workflow, accounting total, contrast, percentage or causal effect is reconstructed.

After all six close, the predeclared summary covers every target: population, observed/not-observed and exact-status coverage; per-target first and subsequent observed input/cache charges; per-category repeated-item counts and min/max ranges across those observed target values. Unknown responses and absent attribution are separately labeled, not filtered into an exact population. A deterministic first fully qualified target in delivered order per category may illustrate the mechanism only after complete closure. It cannot establish net counterfactual savings.

## Tests and review scope

Twenty-six provider-free tests pass. Initial missing-module REDs precede both new modules. Additional REDs cover exact target locator qualification, missing optional complete-path evidence on a failed candidate, null read references, partial raw metadata, bundle-bound delivery classification and the missing-writer-before-execution guard. The missing-writer regression first demonstrated that execution was reached before publication readiness. The retained-attempt and failed-publication tests confirm no second execution is reached after a marker exists.

The synthetic integration uses the real pinned extractor, checking nested-content exclusion, exact response deduplication, conflicting duplicates and partial-metadata unknown status. Other tests cover explicit pre-context turns, boolean/conflicting IDs, wrong-thread/prefix nonjoins, native counter equality, multi-archive whole-item charging, no body export, signed residual retention and row/byte ceilings. Source-only preparation independently matched all 510 actual target identities; it did not call the extractor on actual workflow metadata.

Algorithmic work is linear parsing/hash-map joins plus sorting identity/path sets. There is no per-response full native-file scan or per-bundle copy of an output charge. The implementation affects no agent-facing workflow, runtime, public API or operator benchmark admission, so no Cargo build or real-provider verification belongs to this offline helper qualification. Frozen helpers, source captures, qualification receipts and historical reports remain unchanged.
