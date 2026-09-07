# Candidate postcapture runbook

This is an offline procedure for the frozen three-workflow candidate screen.
No command starts a provider, prepares another phase or changes a frozen input.
Do not execute accounting or inspect candidate costs until **all three terminal
outcomes** are published. A failed workflow remains an observation; success is not
the release criterion. Missing publication or integrity failure remains explicit
and cannot justify replacement, zero-fill or a changed accounting rule.

## Fixed identities and locations

From repository root:

```sh
candidate_phase=/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/candidate-screen-01
candidate_helpers="$candidate_phase/infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z"
timeout --kill-after=5s 120s python3 -B "$candidate_helpers/runner_candidates.py" verify --phase-root "$candidate_phase"
```

Expected `PHASE-MANIFEST.json` SHA-256:
`935a32719e0c91e2afb8fc101788db44bb955a3d85993d2c28327cc44325acb1`.
Frozen launch order is 002 → 001 → 003, one concurrent wave; analysis retains
run-ID order and does not turn launch order into a treatment/control allocation.

| Run ID | Condition | Artifact relative to phase root |
| --- | --- | --- |
| `candidate-screen-01-workflow-001` | `requested-repeat-full` | `runs/candidate-screen-01-workflow-001/candidate-screen-01-workflow-001-three-feature-bench-artifacts` |
| `candidate-screen-01-workflow-002` | `unified-diff-preferred` | `runs/candidate-screen-01-workflow-002/candidate-screen-01-workflow-002-three-feature-bench-artifacts` |
| `candidate-screen-01-workflow-003` | `review-fix-request-resupply` | `runs/candidate-screen-01-workflow-003/candidate-screen-01-workflow-003-three-feature-bench-artifacts` |

For each exact run ID `rid`, the report is `<artifact>/report.json`, terminal
receipt `logs/<rid>.exit.json`, experiment manifest `experiments/<rid>.json`, and
host trace `prompt-events/<rid>.jsonl`. These are manifest-owned paths, not guessed
staging locations. Runtime roots under `/tmp/work-leaf-candidate-screen.WgKBtD`
are not postcapture evidence substitutes; driver cleanup may remove them.

Frozen helper hashes are delivery `ab9570bb0e87674fa738aa0dc6e51702ef7ac6ab97b17092d1f546a60b3889e1`,
accounting `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`,
trust `37122d613c96b21fc4cd078ad8dfd7784e0fa60c1f430bb2ff78cf6d82ea88b8`,
and candidate runner `ecac7753fb535c59974deedf6a04f0ce11b6e05a19f8c4112416ab7e29b7cb03`.
Use their frozen path-preserving tree, not current mutable siblings or cached `.pyc`.

## Closure, source and configuration gate

Require `PHASE-RESULT.json`, `score-manifest.json`, `config-history.json` and
`trust-final.json`. Join the exact three IDs against the original schedule and
admission hash; no row may still be running. Check each retained receipt against
its final score row, including exit, status and report/artifact paths. In the normal
terminal branch, `runner_work_units.py::run_phase` writes `launch_status: completed`
and an exit receipt even for a workflow failure. A supervisor-stopped, unlaunched or
unpublished row is preserved as such, not rewritten into that branch to satisfy a
delivery helper.

For interpretable comparisons, require no frozen-input integrity errors, no
unexplained config drift and no pending attestation at closure. Do **not** require
the legacy behavioral-drift flag to be false or erase it. Exact-compile the pinned
`trust_work_units.py` and call `audit(candidate_phase)` immediately while its required
external final-global bytes still match. Retain its complete safe result as a NEW
`FINAL-GLOBAL-TRUST-REPLAY.json` using `apply_patch`; preserve any failure unchanged.
Re-run the frozen runner's `verify` command before and after this read-only audit.
Later global changes can prevent independent replay; do not copy configuration or
credentials or invent a self-contained proof from hashes alone.

The trust replay's prefix identities, native permission witness, config remainder
and actual project inventories are distinct from delivery and token evidence.
Check each actual pre-spawn project inventory and all available checkpoints; they
are boundary snapshots, not continuous-immutability proof. Retain missing final
checkpoints on failed workflows rather than filling them in.

## Delivery input assembly and create-new audit

Only after closure, construct each NEW
`postcapture/<rid>/DELIVERY-AUDIT-INPUT.json` with this exact schema. `ref(path)` means
the canonical absolute regular file plus its current SHA-256, computed read-only;
the frozen auditor independently rehashes before and after replay.

```python
{
  "schema": "work-leaf-candidate-delivery-input-v1",
  "run_id": row["run_id"], "condition": row["condition"],
  "helper_sha256": "ab9570bb0e87674fa738aa0dc6e51702ef7ac6ab97b17092d1f546a60b3889e1",
  "runtime_sources": {name: ref(phase / "infrastructure/evidence" / name)
                      for name in delivery.RUNTIME_PINS},
  "trace": ref(Path(row["prompt_trace"])),
  "terminal": ref(phase / "logs" / (row["run_id"] + ".exit.json")),
  "invocations": ref(observation / "process-invocations.jsonl"),
  "captures": [{
    "path": str(observation / "app-server" / invocation_id),
    "clients": ref(observation / "app-server" / invocation_id / "client-to-server.raw"),
    "forwarded": ref(observation / "app-server" / invocation_id / "client-to-server.forwarded.raw"),
    "servers": ref(observation / "app-server" / invocation_id / "server-to-client.raw"),
    "start": ref(observation / "invocations" / invocation_id / "start.json"),
    "end": ref(observation / "invocations" / invocation_id / "end.json")
  } for invocation_id in every_app_server_id]
}
```

Here `phase` is the exact phase above; `row` comes from the frozen schedule and must
match the final score row; `observation = Path(row["artifact"]) / "observation"`.
`every_app_server_id` comes from the complete observer process inventory, checked
against actual app-server directories, with duplicates rejected. Include every
capture, not only successful or usage-bearing ones. `delivery.RUNTIME_PINS` names
exactly `src/{agent,bench_experiment,bench_candidate_experiment,orchestrator,cli,chat_title}.rs`;
their six frozen source paths and hashes must also match the phase file inventory.
Hash the trace's experiment manifest and verify exact schema/run/condition/evidence
path against the frozen row. Save assembled JSON via `apply_patch`, refusing an
existing destination. If assembly cannot resolve evidence, retain a separate
per-run unavailable/error record and continue preserving the other rows.

For each of the three IDs, with the newly saved input's exact SHA substituted:

```sh
candidate_run=candidate-screen-01-workflow-001
candidate_input="$candidate_phase/postcapture/$candidate_run/DELIVERY-AUDIT-INPUT.json"
sha256sum "$candidate_input"
candidate_input_sha="REPLACE_WITH_EXACT_INPUT_SHA256"
timeout --kill-after=5s 180s python3 -B "$candidate_helpers/audit_candidate_delivery.py" \
  --input "$candidate_input" --input-sha256 "$candidate_input_sha" \
  --output "$candidate_phase/postcapture/$candidate_run/DELIVERY-AUDIT.json"
```

Use 001, 002 and 003, once each; do not substitute another run if a report is
unverifiable. The auditor's `write_new` refuses overwrite. Exit 0 means its delivery
scope is available; it does not change the original workflow exit or establish
native/accounting/config/binary completeness. Fields named `native_user_*` here are
**public app-server user-item identities**, not native rollout IDs.

## Native membership and common accounting, all nine retained rows

Before treating a delivery as a native input, join the actual accepted local thread
and turn to the source-native session metadata and explicit native input turn ID;
compare the complete public input text. Do not equate public and native item IDs.
Resolve only captured thread IDs under `/home/user/.codex/sessions`, verify retained
native path hashes, session ID/cwd/CLI and model/effort, and preserve every extra,
missing or unaccounted thread. A usage-less thread must not vanish merely because
the observer's usage-derived thread summary omits it. The unchanged accounting
helper rejects unsupported scope; delivery success is not permission to bypass it.

Declare a NEW `CANDIDATE-COMMON-ACCOUNTING-SCOPE.json` before reading totals. Pin the
phase/helper/dependencies, both score manifests and all nine exact IDs: candidate
001/002/003 plus W control 002/004/006/009/010/012. W010 stays included with exit 1.
The existing W common report and baseline compatibility note are provenance, not
permission to rewrite the original W admission or import a randomized p-value.

The bounded provider-free execution is `timeout --kill-after=5s 600s python3 -B -`.
Inside that one-shot command, exact-compile the frozen accounting bytes after checking
their unique `frozen-evidence` manifest pin, retain their actual `__file__`, and call:

```python
candidate_rows = final_candidate_score_manifest["runs"]  # all three terminal
control_ids = {f"work-units-01-workflow-{i:03d}" for i in (2, 4, 6, 9, 10, 12)}
control_rows = [r for r in original_W_score_manifest["runs"] if r["run_id"] in control_ids]
assert len(candidate_rows) == 3 and {r["run_id"] for r in control_rows} == control_ids
results = [(entry, accounting.audit_run(entry, frozen_candidate_manifest,
             Path("/home/user/.codex/sessions"))) for entry in candidate_rows + control_rows]
```

Do not call the old v2/v3 whole-phase analyzer with invented conditions or fabricated
trace rows. Retain every returned status, original observer ledger, exact response
evidence, gap, warning and correction. Reject duplicate response IDs across distinct
workflows; rehash every returned source and frozen input at the endpoint. Missing
upper bounds stay unbounded, errors stay unknown/ineligible, and partial/failed
results are neither zero-filled nor excluded. An early helper/process failure must
produce explicit not-audited rows for the remaining IDs, not a nine-row success claim.

Save a NEW `CANDIDATE-COMMON-ACCOUNTING.json` via `apply_patch`, using full results or
the explicitly labeled source/digest projection demonstrated by `COMMON-W-ACCOUNTING.json`.
Keep large tool stdout out of chat with result projection; do not redirect over old
artifacts. After all three mechanism/exposure checks, the descriptive contrast for
each candidate is `[candidate.lower - mean(control.upper), candidate.upper -
mean(control.lower)]`, retaining unbounded endpoints. Report all three, no p-value,
midpoint, invented price, automatic repetition, quality gate or share of the historic
Direct–WL gap. No command in this runbook authorizes further generation.
