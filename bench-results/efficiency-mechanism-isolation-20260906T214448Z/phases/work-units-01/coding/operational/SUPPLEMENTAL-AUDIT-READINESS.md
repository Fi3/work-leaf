# Supplemental audit readiness

Both supplemental reports retain all twelve unique frozen workflow IDs, including
failed workflow 010. Their hashes match the identities recorded immediately after
generation and remain unchanged at the end of the bounded metadata-only review.
The reports expose one unresolved output-action attribution in workflow 003;
neither report establishes whole-workflow completeness.

All relative paths below are within
`/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01`.

## Report and helper identities

| File | SHA256 |
| --- | --- |
| `COMPACTION-AUDIT.json` | `bcec81dd2641dd27f83ae11eb71371062bb2df0c09333226379fccfede9fc823` |
| `INPUT-ATTRIBUTION.json` | `ed171a20cec3017a75d84219ce81f759725e86b95f3494daedc27868cecf92cd` |
| Frozen `audit_compaction.py` | `dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153` |
| Frozen `audit_input_attribution.py` | `ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740` |

The frozen helpers are under
`infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z/`.
Their invocation identities were checked before report generation. Both commands
exited 0; process success alone is not a claim that every row has exact coverage.

In each report, `/runs/0/run_id` through `/runs/11/run_id` map in order to
`work-units-01-workflow-001` through `work-units-01-workflow-012`. This inventory
matches the frozen schedule exactly, with neither duplicates nor omitted runs.

## Workflow 003 attribution limitation

In `INPUT-ATTRIBUTION.json`:

- `/runs/2/status` and `/runs/2/attribution/status` are `unknown`.
- `/runs/2/attribution/responses/116/status` is `unknown`.
- `/runs/2/attribution/responses/116/errors/0` identifies unmatched/unsupported
  output item scope for `cmp_06dd34dd4bcd978a016a9e11dc626887d299002841115c60e3`.
- `/runs/2/attribution/responses/116/errors/1` records that attribution does not
  exactly reconcile to recorded usage.
- `/runs/2/attribution/responses/116/output_items/0/action` is `null`.
- `/runs/2/errors` preserves these response-qualified errors. The other workflow
  rows have `exact_observed_attribution` status and empty errors.

The affected response identity is
`resp_06dd34dd4bcd978a016a9e11d96e9487d2a73f239eca987fc2`.
Its saved source locator is
`runs/work-units-01-workflow-003/work-units-01-workflow-003-three-feature-bench-artifacts/observation/app-server/00000441344929587964-3537091/server-to-client.raw`,
physical line 63485. The source body is not opened in this review.

`audit_input_attribution.py::extract_records` looks up each output item's native
action at lines 180–181. Missing action or unsupported scope leaves reasoning
classification unknown at lines 183–186. Lines 200–201 then preserve that unknown
component as `None`; the exact-reconciliation predicate at lines 202–203 rejects
it because `None != 0`. Consequently, the recorded reconciliation error does not
alone prove a numerical discrepancy. It also does not establish a changed source
file. No residual or token value is inspected to make a stronger claim.

The uncertainty stays retained for workflow 003; it is not a reason to omit the
workflow, replace it, zero-fill it, or alter the primary accounting ledger.

## Observed source coverage and its limits

For every row in `COMPACTION-AUDIT.json`:

- `/runs/<index>/status` is `observed_prefix_audited` and
  `/runs/<index>/errors` is empty; retained thread errors are also empty.
- `/runs/<index>/app_raw_status` is `available`.
- `/runs/<index>/observer_threads_without_native_source` and
  `/runs/<index>/raw_only_response_ids` are empty. Retained thread
  `native_only_response_ids` lists are empty as well.
- `/runs/<index>/observer_capture_complete` and
  `/runs/<index>/whole_workflow_coverage_established` remain `false`.

In `INPUT-ATTRIBUTION.json`, every
`/runs/<index>/attribution/native_response_ids_without_raw` list is empty, while
the top-level `/whole_workflow_coverage_established` remains `false`.

No generated source diagnostic reports missing or changed source files. These are
recorded audit findings: this readiness review verifies the report bytes, not a
fresh independent rehash of every underlying native source.

`audit_compaction.py::audit_run` checks the native path boundary and saved source
SHA at lines 211–217 and flags missing observer/native coverage at lines 250–252.
Its `observed_prefix_audited` status at line 271 means that this observed-prefix
inventory has no recorded errors. It does not override the observer completeness
flag copied at line 262 or the explicit whole-workflow limitation. Matching
observed identities therefore does not resolve interrupted tails, hidden work,
or exhaustive call coverage.

## Review scope

Only report hashes, workflow identities, statuses, errors, source-coverage flags,
the unmatched action's identity/null linkage, and frozen helper control flow are
examined. No token totals, residual values, repeated-item rankings, quality
figures, semantic bodies, ACK labels or condition comparisons are inspected or
reproduced. No contrasts are computed.

The supplemental reports and helpers are unchanged. No helper is rerun and no
provider is called. The sole write for this preservation task is this create-new
operational review; no commit is made.
