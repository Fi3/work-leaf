# Fixed C08 original-accounting publication

Status: the separate `execute_accounting_once.py` implementation has provider-free synthetic coverage. Actual accounting execution requires independent review and a separately frozen source/output declaration. No actual receipt, response map, native payload, measurement or token effect was inspected or computed during implementation.

## Existing interface and smallest scope

The immutable `accounting_untracked_reads.py` has `audit_run(entry, frozen, sessions_root)` and no standalone CLI. `analyze_untracked_reads.py::measurement_for/analyze_manifest` imposes different v3 whole-phase/primary gates and does not provide six-receipt historical reuse or this fixed-three publication transaction. The provider-ledger qualifier calls `audit_run` again, and the compaction derivatives change the accounting predicate. Neither is an eligible launcher for this task. A small separate orchestration layer is needed; no accounting algorithm or new measurement interpretation is proposed.

One separately frozen scope authorizes only the three original calls, in fixed ID order:

```text
automatic-refresh-01-workflow-001
automatic-refresh-01-workflow-002
automatic-refresh-01-workflow-003
```

Each receives its complete unchanged eventual `score-manifest.json` row, the parsed object from the exact verified phase manifest, and canonical `/home/user/.codex/sessions`. The entry may use the runner's `id` field; do not synthesize a `run_id` replacement or omit failure fields. Each row is attempted once even when its workflow failed or its original accounting returns unknown. No baseline `audit_run`, observer analyzer/extractor, qualifier, derivative or provider invocation is authorized.

The six W IDs are `work-units-01-workflow-002/004/006/009/010/012`. W010 remains present with its original failed workflow outcome. Their exact receipt/map identities and counts are the tables in `POSTCAPTURE-ACCOUNTING-CHECKLIST.md`; those tables are source declarations, not freshly computed measurements here.

Private API:

```text
execute(scope_path, expected_scope_sha256) -> final orchestration receipt
compile_accounting(actual_path, captured_exact_bytes, parsed_manifest) -> module
invoke_accounting(actual_path, captured_exact_bytes, entry, parsed_manifest,
                  sessions_root, timeout_seconds, operation_root) -> unchanged full audit_run result
declared_outputs(output_root) -> exact literal scope output-path map
```

The CLI accepts only `--scope` and `--scope-sha256`; it has no alternative helper, row, baseline or retry option. `execute` requires typed `execution_authorized=true` before any marker or output reservation; read-only `preflight` also accepts a disarmed draft. Test monkeypatches use a clearly synthetic helper and invocation function, never a production CLI override. Child supervision is private to the single invocation function; there is no general execution service. `compile_accounting` qualifies exact source/module identity without invoking accounting; the production child's fixed bootstrap compiles the pinned source at its original `__file__` and makes the sole row call.

## Frozen declaration before any new result

The future scope contains exact refs to the phase manifest, schedule, `PHASE-RESULT.json`, terminal `score-manifest.json`, three completed per-run `.exit.json` receipts and a complete predeclared consumed-source SHA map. It binds all 175 immutable phase entries, the reviewed orchestrator/test/review cut, the exact helper and predecessor layout, plus six baseline receipt refs and six response-map supplement refs. `schedule` is a `{path,sha256}` ref, and `outputs` is exactly the complete `declared_outputs(output_root)` map. Fixed IDs/order, sessions root, per-row bound and historical descriptive limitations are literal fields. The final scope itself is create-new and hashed before execution; this document is not that execution declaration.

`runner_work_units.py` publishes `finished_at`, `manifest_sha256` and final `runs` in `PHASE-RESULT.json`; `score-manifest.json` repeats the final rows and exact `phase_manifest_sha256`. Completed `.exit.json` rows have `launch_status=completed`, `started_at`, `finished_at` and the actual `launcher_exit_code`. Preflight joins these actual schemas by exact ID/typed row content and the frozen schedule. No running row is admitted; a nonzero exit is not excluded. Source/config and delivery qualifications remain separate preserved inputs, not hidden conditions that alter the original helper's arithmetic.

The proposed deadline is 600 seconds per new invocation, sequentially, with a five-second termination grace for an accounting worker only. This is an upper bound, not permission to repeat. A worker exception, deadline or failed publication remains a retained execution outcome. `invoke_accounting` reports a typed private `WorkerFailure` containing its kind, actual exit disposition and `closed` evidence after supervision. Closed worker errors/timeouts allow the next fixed row, not a repeat of the failed row. An exception without proven child closure blocks later calls as `not_called_after_worker_uncertainty`; it cannot be described as a terminated worker merely because a timeout elapsed. If immutable/input-source drift makes later execution unsafe, subsequent declared IDs receive `not_called_after_source_failure`; the scope remains incomplete and does not authorize replacement calls.

The narrow internal `supervise_worker(argv, stdout_path, stderr_path, timeout_seconds, grace_seconds)` owns one process group, preserves full stdout/stderr in exclusive files, waits after termination/escalation and returns factual exit/timeout/closure metadata. The production invocation supplies only its fixed original-accounting bootstrap; there is no CLI alternative-helper/argv option. Its unit tests use a synthetic non-accounting Python child: one immediate nonzero exit and one bounded sleeper that ignores SIGTERM to exercise the short escalation path. Those tests invoke neither the original helper nor any provider/native executor. The shorter synthetic deadlines are test inputs to this private supervision function, not changes to the declared 600+5-second accounting limit.

## Exact source and module identity

Use only the actual helper path uniquely listed as `frozen-evidence` in the parsed manifest:

```text
P/infrastructure/evidence/bench-results/
efficiency-mechanism-isolation-20260906T214448Z/accounting_untracked_reads.py
```

Its SHA is `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`. Read/hash its exact bytes and compile with its actual canonical `__file__`, not the moving study path or a copied standalone module. Its `HERE/ROOT/PINS` derive `analyze.py`, `audit_compaction.py` and the sibling measurement-gate `batch_analysis.py`; preflight verifies the exact hashes and path-preserving tree in the checklist before any attempt. The module's existing exact loaders remain unchanged. Duplicate helper entries, a different role, symlink aliases, dependency drift or a manifest object reconstructed from selected fields fail closed.

Before each invocation, verify the complete frozen/source map. Every returned `source_sha256` path must resolve to its exact predeclared source identity and match before/after hashes; an unlisted returned source is a scope failure, not retrospectively admitted. Recheck all declared and previously returned sources at final closure. Preserve the unchanged helper result separately from caller-level source/publication errors. No source-integrity error is erased to expose a numerical result.

## Once-only publication

Outputs, all create-new under `P/postcapture`:

- `ACCOUNTING-ORIGINAL-ATTEMPT.json`: durable fixed-scope attempt marker before any helper invocation.
- `<new-ID>/COMMON-ACCOUNTING-ORIGINAL-FULL.json`: complete original helper result, canonical bytes/hash retained before projection.
- `<new-ID>/COMMON-ACCOUNTING-ORIGINAL.json`: full-result hash, exact entry, original diagnostics and lossless metadata projection; no arithmetic changes.
- `<new-ID>/worker/{INPUT.json,RESULT.json,stdout,stderr,EXECUTION.json}`: exact per-row worker input, full canonical worker result, original process streams and waited execution metadata. These five paths are explicitly declared; the worker opens each create-new and has no hidden temporary root.
- `COMMON-ACCOUNTING-ORIGINAL-RESPONSE-OWNERSHIP.json`: complete explicit returned/imported response ownership, with completeness limitations.
- `COMMON-ACCOUNTING-ORIGINAL.json`: all nine ordered references/projections, caller integrity/publication outcomes and the reused baseline identities; no contrast calculation.

Preflight all reserved paths and publication parents before the marker or a call: reject existing files/directories and dangling/ordinary symlinks, overlapping targets, missing/unwritable parent paths and any prior attempt. Establish write readiness with exclusive handles/owned reservation, not a post-call first attempt to create the output. The durable marker is never removed to enable a retry. A worker result is canonicalized and hashed in full before projection; reserved output handles retain partial/failed publication facts if writes fail. Publication is not a multi-file atomic transaction, and parent/worker failure must not be represented as a completed result. No repeated `audit_run` is allowed to repair output publication.

The worker invokes only the original helper once. It returns/publishes the complete exact object, with only metadata in console output. The parent preserves every received result and per-row execution record. A final consolidation is permitted only after all three declared rows have terminal invocation dispositions and all baseline/response/source checks have been attempted; missing rows remain explicit, never a shorter successful population.

## Existing W identity reuse and canonical checks

Read the six exact frozen `COMMON-ACCOUNTING-ORIGINAL.json` receipt files and their six separately pinned `PROVIDER-LEDGER-QUALIFICATION.json` identity supplements. The actual supplement schema is `work-leaf-admitted-provider-ledger-execution-v1`, with `result.schema=work-leaf-provider-ledger-qualification-v1`. Its full map is top-level `response_evidence`; `result.original.exact_response_evidence` is a count/hash projection, not the full map. Bind that map to both the supplement's original projection and the frozen measurement receipt's projection, plus the supplement's original-entry/run ownership. Its later qualification measurement is not reused. Enforce the checklist's prospectively pinned count/file hashes; the checklist's earlier shorthand `original.exact_response_evidence` is not the saved file's JSON access path. Recheck the retained original response-ownership ref as a metadata identity source if included in the final declaration.

Canonical identities use `json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True, allow_nan=False).encode('utf-8')`, matching the earlier canonical-result/map recipe. The six projected receipt bytes and their stored `exact_helper_result_sha256` remain immutable. The map supplement need not have the same whole-result source-path provenance as that earlier result: do not claim to recompute the older full-result digest from a projection or equate a later whole supplement result to it. File pin + exact map count/hash is the available reuse proof.

For each new full result, verify `schema=work-leaf-read-identity-accounting-v1`, exact run ID, typed status/errors/warnings and a complete response map object. Every response ID/thread/turn is a nonempty string. Each usage record retains the four nonnegative integer counters and source locators; derived usage consistency can be structurally checked without re-summing a run or changing bounds. Build one global response-to-run map across all nine returned/imported maps; any repeated response ID across runs fails uniqueness, even if usage agrees. Partial maps from unknown results remain partial: disjoint returned IDs do not prove unseen-response completeness.

The original strict measurement statuses are `exact`, `bounded`, `unbounded_accounting_gap` and `ineligible`; wrapper execution/integrity status is separate. Bounds are exactly `raw_input_plus_output` and `uncached_input_plus_output`, each `{lower,upper}` when available. Preserve integer endpoints, `upper=null`, whole `bounds=null`, reasons, warning/gap inventories, original observer ledger, corrected scope and `whole_workflow_hidden_call_completeness_proven=false`. No zero fill, response ceiling invention, midpoint, percentage, p-value, additive share or failure-as-equivalent-work interpretation belongs here.

The only projection substitutions are the established count/hash records for full response maps, `measurement.gap_inventory.raw_responses` and each `corrected_scope.captures[*].proof_view_server_lines`; the complete exact new result remains in its FULL sidecar. Preserve every other field. Reused baseline projections remain byte-/value-identical to their pinned receipts.

## Synthetic verification boundary

`test_accounting_once.py` uses only temporary synthetic source/result bytes, fake invocation callbacks and toy response maps; it never invokes real `audit_run`, a private command executor or a native source. Explicit invocation side-effect exceptions cover closed worker failure, closed timeout and unproven closure. Every completed test execution preserves nine disposition rows and a durable no-retry marker; blocked new rows also have separate disposition receipts. Cases cover exactly three normal new calls/no six baseline calls, unchanged parsed manifest/module path identity, schedule joins, explicit worker declarations, attempt/output replay/symlink failures, source drift, projected-baseline map mismatch, cross-run response-ID collisions, failed outcomes, null bounds and publication failure without retry. Pinned postcapture inputs may coexist with distinct reserved outputs; no source may inhabit a reserved worker subtree. Two bounded non-accounting child cases qualify only supervision/termination, not accounting. Actual accounting/source closure requires separately reviewed implementation and a frozen execution declaration after all workflows close.

The parent rehashes the fixed source union before each of three calls and at closure: O(K × E) for fixed K=3 and total declared source bytes E. Response-map validation/ownership uses indexed passes and canonical serialization linear in supplied metadata size under normal hash assumptions. Complete original JSON values and their projection are held in memory; no general peak-memory bound is asserted. Nothing here changes provider/agent-facing behavior, so no real-agent workflow is required or authorized for this offline publication layer.
