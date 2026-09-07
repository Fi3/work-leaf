# Post-run supplemental coverage and quality review

All twelve frozen workflow IDs are retained in both supplemental audits and the
canonical quality report, including workflow 010's recorded workflow failure.
The observed native/raw response inventories agree by identity for every run;
they do not establish complete whole-workflow usage. Workflow 003 has distinct
compaction, attribution-linkage and token-bridge diagnostics described below.

This review follows the completed classification freeze. It does not calculate
condition contrasts, revise classifications, or replace the frozen token ledger.

## Report identities

Paths are relative to
`/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01`.

| Report | SHA256 |
| --- | --- |
| `COMPACTION-AUDIT.json` | `bcec81dd2641dd27f83ae11eb71371062bb2df0c09333226379fccfede9fc823` |
| `INPUT-ATTRIBUTION.json` | `ed171a20cec3017a75d84219ce81f759725e86b95f3494daedc27868cecf92cd` |
| `QUALITY.json` | `9d7027bc3cc877ed18d8c9fd298aa7393de33267161c934648aa9251dcf7e1dd` |
| `WORK-UNIT-TOKENS.json` | `424fe4701e5407fdbf9f37ff87273b475e518c56377ae4d71cabe397443a7ae7` |

The supplemental and quality hashes are checked at both ends of this review.
Only workflow 003's error/status fields and the independent gate's error list are
inspected in the token report; its main comparison remains outside this review.

## All retained coverage results

Rows 0 through 11 in each supplemental `/runs` array correspond to suffixes 001
through 012. The response count below is independently identified completed
responses, not ACKs, provider turns, or exhaustive model-call counts. For each
row, `native_response_count`, `raw_response_count`, and
`identity_matched_response_count` are equal to the displayed value.

| Workflow | Launcher exit | Matched native/raw responses | Compaction markers | Observed counter scope | Input attribution |
| --- | ---: | ---: | ---: | --- | --- |
| 001 | 0 | 186 | 0 | matching_prefix | exact_observed_attribution |
| 002 | 0 | 172 | 0 | matching_prefix | exact_observed_attribution |
| 003 | 0 | 234 | 1 | different | unknown |
| 004 | 0 | 189 | 0 | matching_prefix | exact_observed_attribution |
| 005 | 0 | 192 | 0 | matching_prefix | exact_observed_attribution |
| 006 | 0 | 196 | 0 | matching_prefix | exact_observed_attribution |
| 007 | 0 | 201 | 0 | matching_prefix | exact_observed_attribution |
| 008 | 0 | 199 | 0 | matching_prefix | exact_observed_attribution |
| 009 | 0 | 201 | 0 | matching_prefix | exact_observed_attribution |
| 010 | 1 | 191 | 0 | matching_prefix | exact_observed_attribution |
| 011 | 0 | 216 | 0 | matching_prefix | exact_observed_attribution |
| 012 | 0 | 215 | 0 | matching_prefix | exact_observed_attribution |

For all compaction `/runs/<index>` rows:

- `status` is `observed_prefix_audited`, `errors` and retained thread errors are
  empty, and `app_raw_status` is `available`.
- `observer_threads_without_native_source`, `raw_only_response_ids`, and retained
  thread `native_only_response_ids` lists are empty. Input attribution's
  `/runs/<index>/attribution/native_response_ids_without_raw` lists are also empty.
- `observer_capture_complete` and `whole_workflow_coverage_established` remain
  false. Matching observed identities do not resolve interrupted tails or hidden
  work; no missing usage is zero-filled.

The compaction audit's `/runs/2/compaction_raw_coverage` is
`adjacent_candidates_identity_matched`; other rows are `not_exercised`.
No generated diagnostic reports missing or changed source files. This review
does not rerun the audits or independently rehash every native source; the exact
workflow 003 source files investigated below are independently rehashed.

## Workflow 003: three separate facts

### Source identities and explicit compaction link

The relevant source paths are:

- Raw capture: `runs/work-units-01-workflow-003/work-units-01-workflow-003-three-feature-bench-artifacts/observation/app-server/00000441344929587964-3537091/server-to-client.raw`, SHA256 `2f6949270f63a69423eee95ee0f7f866169c04cbffc027c9926862cca6909009`.
- Native source: `/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-43-01a07949-ee9c-7503-a486-0108cc6bb00d.jsonl`, SHA256 `c7df081e0badfbdff11fa7bcd5d920607b43db357d5db6906f5f1aeccf38e2bd`.

Both current hashes match the supplemental reports' source identities. The exact
thread is `01a07949-ee9c-7503-a486-0108cc6bb00d`, turn
`01a07975-91c7-71c0-9c81-344caa91ef43`, response
`resp_06dd34dd4bcd978a016a9e11d96e9487d2a73f239eca987fc2`.

Native line 623 is its identity-linked usage record. Line 624 is the `compacted`
record, timestamp `2026-09-07T01:24:10.430Z`. Besides the frozen audit's adjacency
link at `/runs/2/threads/0/compaction_markers/0`, the public metadata fields
`/payload/compaction_response_id` and
`/payload/latest_token_usage_record/response_id` explicitly name this same
response. No compaction message, replacement history, guardian history or private
reasoning content is inspected or reproduced.

The recorded `/runs/2/native_minus_observer` delta is input 228,168, cached input
18,816, output 5,223, reasoning output 0: raw input plus output 233,391 and uncached
input plus output 214,575. These equal this explicitly identified response's
usage. This is a scope difference between existing recorded ledgers, not an
authorization to add the amount to or otherwise retotal the primary ledger.

### Exact numerical reconciliation, unknown action/reasoning attribution

`INPUT-ATTRIBUTION.json /runs/2/attribution/responses/116` retains this response.
Its `/output_items/0/item_id` is
`cmp_06dd34dd4bcd978a016a9e11dc626887d299002841115c60e3`, with `/action: null`.
The identity-only scan of the matched native source finds no top-level
`response_item` with that exact ID. The frozen native-action index only indexes
those top-level records (`audit_input_attribution.py::native_item_inventory`,
lines 247–262); no substitute item identity is inferred.

The actual residual fields under `/runs/2/attribution/responses/116/residual` are:

| Component | Residual |
| --- | ---: |
| input_tokens | 0 |
| cached_input_tokens | 0 |
| output_tokens | 0 |
| cache_write_input_tokens | 0 |
| reasoning_output_tokens | null |

The same record's `/usage/reasoning_output_tokens` is recorded as 0, but its
item-level `/attributed_usage/reasoning_output_tokens` remains null because the
native action link is absent. Frozen `extract_records` marks that classification
unknown at lines 183–186; lines 200–203 propagate the null residual into the
general exact-reconciliation error. The unknown attribution status therefore
does not conceal a nonzero input/output numerical residual in this response.
The frozen status and errors remain unchanged.

### Separate token-bridge arithmetic rejection

The valid raw response above occurs at physical raw-capture line 63485:
`rawResponse/completed`, with input 228,168 + output 5,223 = total 233,391.

The immediately following line 63486 is `thread/tokenUsage/updated` for the same
thread/turn. Its `/params/tokenUsage/last` has `totalTokens: 38663` while
`inputTokens`, `cachedInputTokens`, `outputTokens`, and `reasoningOutputTokens`
are all zero. This nonadditive metadata object is independently verified here.
Its semantic meaning as a possible compaction/context-size estimate is not
established by its shape or timing alone; it is not reclassified as billed usage.

Frozen `batch_analysis.py::usage(camel=True)` rejects `totalTokens != input +
output` at lines 76–77. The prospective inventory consumes cumulative and last
usage notifications as well as completed raw responses, so this rejection is
different from the supplemental nullable-reasoning diagnostic.
`WORK-UNIT-TOKENS.json /observations/2/errors/0` consequently records
`provider total differs from input plus output`; `/measurement/status` is
`ineligible`, `/measurement/bounds` is null, and `exact_response_evidence` is
absent for this row. `/independent_v2_gate/integrity_errors` is empty.
No frozen parser, source notification or report is modified to make the gate pass.

## Exact canonical quality results

The frozen scorer runs the saved implementation against three feature fixtures:
`quality_visual_behavior`, `quality_status_behavior`, and
`quality_completion_behavior` (`score.py::FIXTURES`). Their results are separate
from the driver's workflow result and its repository-wide validation.

| Workflow | Recorded workflow result | visual | status | completion | Passed fixtures |
| --- | --- | --- | --- | --- | ---: |
| 001 | pass | pass | pass | pass | 3/3 |
| 002 | pass | pass | pass | fail | 2/3 |
| 003 | pass | fail | pass | fail | 1/3 |
| 004 | pass | pass | pass | fail | 2/3 |
| 005 | pass | fail | pass | fail | 1/3 |
| 006 | pass | pass | pass | fail | 2/3 |
| 007 | pass | pass | pass | fail | 2/3 |
| 008 | pass | pass | pass | fail | 2/3 |
| 009 | pass | pass | pass | fail | 2/3 |
| 010 | fail | pass | pass | pass | 3/3 |
| 011 | pass | pass | pass | pass | 3/3 |
| 012 | pass | pass | pass | fail | 2/3 |

This is 25/36 passed fixture checks: visual 10/12, status 12/12 and completion
3/12. All fixtures ran: passing checks have exit code 0, failing checks exit code
101. Every row has empty `materialization_notes`, and no `scoring_error` is
recorded. No workflow or failed check is excluded.

Exact pointers in `QUALITY.json` are `/runs/<index>/checks/<fixture>`,
`/completed_features`, and `/check_details/<fixture>/{status,exit_code,log,log_sha256}`
under that same row. Rows 0–11 map to workflow suffixes 001–012. Every one of the
36 retained logs at `scorer/logs/<run_id>/<fixture>.log` is independently rehashed
against its recorded `log_sha256`; all match. Log contents are not needed to
restate these canonical statuses, and no specific failed assertion is inferred.

Workflow 010 remains a workflow failure despite passing these offline fixtures:
its pre-linearize contribution-gate diagnosis is retained separately in
`coding/operational/WORKFLOW-010-TERMINATION.md`. Passing the fixtures does not
retroactively complete its linearization or remove its failed workflow outcome.

## Limits and preservation

Frozen helper identities are compaction
`dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153`,
input attribution
`ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740`,
strict accounting adapter
`dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3`,
and canonical scorer
`c0a4e951e96d7da53a6d414a7677176183cae6e30d0bbfab92069d5082865162`.

Quality is descriptive retained evidence, not a post-hoc inclusion filter or a
proof of equivalence. Observed identity reconciliation is not exhaustive usage;
compaction adjacency and explicit response identity do not establish an exact
causal savings share. The token bridge's ineligible row stays ineligible under
its frozen rule, and supplemental attribution stays unknown where recorded.

No provider is launched, no frozen helper/report is rerun or modified, no private
reasoning body is inspected, and no condition contrast is computed. The only
write is this create-new post-run review. No commit is made by the reviewer.
