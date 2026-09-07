# Prospective response-accounting validation

This is an offline validation of `accounting_untracked_reads.py` against synthetic fixtures and unchanged completed source records. It is not retrospective admission or a replacement analysis of the work-unit phase. The replay supplies a helper-pin entry in an in-memory copy of the closed phase manifest; no manifest, report, provider setting, wait, configuration or credential is written. No provider call occurs, and no replacement old-phase total or contrast is published here.

## Checked identities and tests

- Helper SHA-256: `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`.
- Test SHA-256: `93f958ae241b7c59d2dd531aa2fc87f70bee0d268f3c380758ab965177a9bdf7`.
- Closed source phase manifest SHA-256: `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
- Command: `python3 -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z -p test_accounting_untracked_reads.py`.
- Result: all 32 tests pass. Demonstrated RED stages were the absent helper, absent physical response-locator index, incorrectly accepted foreign cumulative/native-compaction identities, and absent late-terminal recovery. Negative cases cover malformed completed usage, cumulative arithmetic/regression, omitted ordinary responses, strict typed identities, conflicting response IDs, native model/cumulative/compaction identity, incomplete compaction lifecycle, invalid-last components, and late-response recovery followed by new generation or mismatching usage.

The adapter independently compiles exact hash-pinned predecessor source bytes. It reuses unchanged conditional tail-proof and accounting definitions, selecting only the required pure definitions from the prospective helper rather than executing its cached module loader. Exact identities and source locations are indexed; there is no response-by-whole-stream locator scan or compaction-by-whole-stream rescan.

## Completed ordinary source replay

`work-units-01-workflow-001` returns `status: validated`, no integrity errors, and `measurement.status: bounded`. It has no nonadditive-last warning or late-terminal recovery. All retained gaps have the predecessor's conditional finite tail proof. Its source observer ledger is unchanged.

The primary captured source is `phases/work-units-01/runs/work-units-01-workflow-001/work-units-01-workflow-001-three-feature-bench-artifacts/observation/app-server/00000441344929586777-3537090/server-to-client.raw`, SHA-256 `f050910cd73200db4ee4bfb9ee63080ca20c42bddd253d467f34f3177550e243`. The original observer analysis SHA-256 is `cd49e49fac799a9d5ed15abd475e8363d0eb02bb2d6d9f7bda8baef2513b8ff7`. The independent adapter checks every source hash, all native metadata references, accepted captures and cumulative/raw response prefixes before closing its result.

## Completed compaction source replay

`work-units-01-workflow-003` returns `status: validated`, no integrity errors, and **`measurement.status: unbounded_accounting_gap`**. The compaction scope proof passes, but an unrelated unsupported tail prevents a finite whole-workflow upper bound. These are separate facts, not an exception that forces a green total.

Primary capture: `phases/work-units-01/runs/work-units-01-workflow-003/work-units-01-workflow-003-three-feature-bench-artifacts/observation/app-server/00000441344929587964-3537091/server-to-client.raw`, SHA-256 `2f6949270f63a69423eee95ee0f7f866169c04cbffc027c9926862cca6909009`.

Native source: `/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-43-01a07949-ee9c-7503-a486-0108cc6bb00d.jsonl`, SHA-256 `c7df081e0badfbdff11fa7bcd5d920607b43db357d5db6906f5f1aeccf38e2bd`.

### Exact compaction omission

Thread `01a07949-ee9c-7503-a486-0108cc6bb00d`, turn `01a07975-91c7-71c0-9c81-344caa91ef43`:

- Native line 623 is the additive response-usage record for `resp_06dd34dd4bcd978a016a9e11d96e9487d2a73f239eca987fc2`.
- Native line 624 explicitly identifies that same response through `compaction_response_id` and the complete matching `latest_token_usage_record`. Adjacency alone is not the proof.
- Raw line 63485 supplies the same exact response identity, thread, turn and additive usage.
- Raw lines 63484 and 63488 bound the complete `contextCompaction` lifecycle for item `01a07975-b799-7ad0-a97d-9b3d6ad0d2a0`.
- Raw line 63486 has nonadditive `tokenUsage.last`, but its strict additive cumulative total is unchanged. The raw-prefix comparison identifies precisely that compaction response as omitted from the observer cumulative scope. The separate observed-scope correction contains that response exactly once; already included compactions receive no second addition.

The original `last` metadata is retained as `unusable_compaction_last_metadata`, not a completed response, zero-cost response, context-size estimate, fresh-usage marker or tail-recovery proof. The adapter checks component types/parents even for this metadata exception. It excludes only this separately proved unusable notification from the predecessor's proof view, retaining original physical locators and withholding any affected turn's tail proof.

### Exact late terminal response

Turn `01a0796a-4f96-7762-8655-158353fc7788` has original grace outcome `forwarded-after-output-resumed`. The saved 1,000 ms grace and 457 ms elapsed value remain unchanged and are not an inference premise.

Raw directive line 61659 is followed by the exact completed response `resp_06dd34dd4bcd978a016a9e0f0f110887d28e6fe8a986cfbdc3` at line 61662, matching additive fresh usage at line 61663, and the same interrupted terminal at line 61667. The native response identity/usage agrees. No later same-thread generation/item/unknown notification intervenes after the raw completion and before terminal. This creates a separately retained `exact_late_terminal_response_usage` proof; it does not change the original grace outcome or count the response twice.

This explains the distinction between 32 originally unresolved grace decisions and 31 retained observer missing-usage turns. The response completed after its earlier grace decision; elapsed time alone would not establish that fact.

### Unsupported tail remains unknown

Turn `01a07973-5e12-77b2-9bb3-619c47929a3b` remains an unresolved gap with `response_count_upper: null` and `proof: null`. Its directive is at raw line 63287, followed by a completed post-directive reasoning item at 63288–63289, a usage notification at 63290 and interrupted terminal at 63294. It does not meet the predecessor's isolated unfinished-response predicate, and no exact same-turn completed raw response establishes recovery. No new response cardinality, observed-maximum ceiling or midpoint is invented.

## Interface and interpretation

`audit_run(entry, frozen, sessions_root)` returns the original observer ledger/diagnostics, separate corrected scope and named compaction IDs, exact completed response evidence, gap inventory and bound policy, warnings, and source SHA-256 map. It requires the helper's unique `frozen-evidence` entry. Phase allocation, configuration/trust, delivery and cross-workflow response-identity gates belong to the independent v3 analyzer; a saved adapter success boolean is not their substitute.

`status: validated` means the independent evidence/scope gates passed. It does not imply finite accounting: the caller must inspect `measurement.status` and both interval endpoints. Every row remains retained, including workflow failures and unknown/unbounded measurements. The model-ceiling bounds retain the documented visible-boundary/no-hidden-call assumptions; matching native/raw prefixes do not prove that hidden provider work cannot exist.

Official [Codex App Server documentation](https://learn.chatgpt.com/docs/app-server) documents the `contextCompaction` item lifecycle and thread usage notifications. It does not establish the arithmetic semantics of this nonadditive `last` field; the validation makes no such claim. OpenAI Docs was used to check that limitation.

The helper affects only offline accounting, not any agent-facing workflow. Completed actual subscription-source replays verify the real captured ordinary and compaction paths; no new real-agent generation is necessary for this offline-only helper. Runtime v3 delivery verification remains the owning implementation agent's separate requirement.
