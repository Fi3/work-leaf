# Provisional investigation ledger

Last checked: **2026-09-07 11:20:54 Europe/Rome (09:20:54 UTC)**.

Scope: the entire overnight mechanism investigation, with all **27 declared benchmark identities** (3 screen + 12 work-unit + 12 read-factor), all **16 diagnostic attempts**, and the elapsed-time breakdown. The earlier two-run pilot and accepted historical twelve-run cohort are listed separately below. The live count is investigation-wide; the smaller current-batch count is explicitly labeled.

## Goal and current experiment

**User instruction, 11:12 Rome: no more control workflows.** The supervisor alone is verified suspended at 11:12:28, preventing every further admission. Active treatments 005 and 006 continue in their own process groups and were not signaled. Controls 007, 010 and 012 will not run. Unlaunched treatments 008, 009 and 011 are held, not silently launched under the superseded mixed schedule. Existing baseline runs will be reused; the interrupted batch is not a completed twelve-run randomized comparison. [Operator stop receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/OPERATOR-CONTROL-STOP.json).

Explain the exact mechanism behind WL's token savings. The historical 45.38%–51.62% reduction is accepted for this investigation, under the user's assumption of no quality reduction; this phase does not rerun Direct sequential.

The current benchmark-only factor replaces eligible bundled, untracked source-read responses with their full inline contents. Both arms preserve the normal bundle creation and all non-target behavior. Ordinary WL remains unchanged. Generation uses the existing ChatGPT subscription, not API-key authentication or API credits.

“Arm” means one version under comparison: six complete benchmarks with normal WL read delivery (control), and six with the benchmark-only inline-read treatment. Each benchmark is the entire three-feature workflow, including review, fixes, linearization and final checks; it is not one feature agent. Neither version is Direct sequential. Waves contain three complete benchmarks and mix the versions.

The six-per-version allocation above is the superseded design, not the user's continuing requirement. There is no implementation reason requiring a fresh baseline. The additional controls were the agent's optional experimental-design choice; they are prohibited by the user's latest instruction. Continuing work compares modified WL with the existing WL baseline, verifies compatibility from saved artifacts, and investigates the exact mechanism without new control runs.

## Live state — whole investigation

- Whole mechanism investigation: **19 workflows with terminal receipts** = 3 initial screen + 12 work-unit + 4 current read-factor workflows. Current active: **1**; current scheduled but not launched: **6**.
- Driver exited, awaiting supervisor receipt: 006 (raw wait status 0).
- These are terminal outcomes, not all successes. Diagnostics and earlier historical/pilot runs are listed separately below; they are not added to the benchmark count.

### Current read-factor batch

- Fixed allocation: 12 workflows — six normal-delivery controls and six inline-read treatments — in four waves of three concurrent complete benchmarks, each with its own checkout and artifacts.
- Original admission: 2026-09-07 07:55:36 UTC. Supervisor is SUSPENDED by user-directed control cancellation. No further workflow can launch; active treatment children continue.
- Completed: **4/12**. Workflow 001: success (exit 0, recorded_workflow_success); Workflow 002: failure (exit 1, recorded_workflow_failure); Workflow 003: failure (exit 1, recorded_workflow_failure); Workflow 004: failure (exit 1, recorded_workflow_failure). All outcomes are retained, with no replacement.
- Workflows 002, 003 and 004 failed at the frozen 300-second idle limit.
- Running: **wave 2**. Workflow 005 — [11:19:49] elapsed=2449s busy=true review-user-1:WaitingForReply:-:lines=6:title=review (Rome timestamps).
- Future controls 007, 010 and 012: CANCELED by user. Future treatments 008, 009 and 011: HELD while current treatments finish; the original mixed-wave continuation is disabled.
- Last configuration snapshot: 2026-09-07T09:12:18.576940+00:00; verified_own_workflow_trust_transition. The supervisor hold pauses this journal; do not treat its old admission flag as permission to launch or claim continuous monitoring during the hold.
- No identification/authentication error was found in the inspected live logs; none has blocked this batch.

### Every current-batch workflow

All times below are on 2026-09-07, Europe/Rome (UTC+02:00). A running row uses its first workflow-log timestamp until the terminal receipt supplies exact launcher times.

| Run | Wave | Condition | Start | Finish | Outcome | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 001 | 1 | control | 09:55:36 | 10:39:00 | success | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/logs/untracked-reads-01-workflow-001.exit.json) |
| 002 | 1 | control | 09:55:36 | 10:25:37 | failure | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/logs/untracked-reads-01-workflow-002.exit.json) |
| 003 | 1 | untracked-read-inline | 09:55:36 | 10:31:25 | failure | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/logs/untracked-reads-01-workflow-003.exit.json) |
| 004 | 2 | control | 10:39:00 | 11:04:41 | failure | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/logs/untracked-reads-01-workflow-004.exit.json) |
| 005 | 2 | untracked-read-inline | 10:39:01 | — | running | [live log](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/logs/untracked-reads-01-workflow-005.log) |
| 006 | 2 | untracked-read-inline | 10:39:01 | — | driver exited; receipt pending | [live log](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/logs/untracked-reads-01-workflow-006.log) |
| 007 | 3 | control | — | — | canceled — not launched | frozen schedule |
| 008 | 3 | untracked-read-inline | — | — | held — not launched | frozen schedule |
| 009 | 3 | untracked-read-inline | — | — | held — not launched | frozen schedule |
| 010 | 4 | control | — | — | canceled — not launched | frozen schedule |
| 011 | 4 | untracked-read-inline | — | — | held — not launched | frozen schedule |
| 012 | 4 | control | — | — | canceled — not launched | frozen schedule |

## Earlier benchmark workflows — complete ledger

All times are 2026-09-07 Europe/Rome (UTC+02:00). A successful workflow exit is distinct from passing a causal hypothesis or every feature-quality check. Every failed workflow remains an observation.

### Initial screen: 3/3 terminal, 3 successful workflows

Execution: **00:21:31–01:08:14 (46m 43s)**, three workflows concurrently. The two initial prompt-policy candidates did not qualify under the frozen screening rule; the original configuration flag remains recorded. These observations are not pooled into later confirmation. [Phase result](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/screen-01/PHASE-RESULT.json).

| Run | Wave | Condition | Start | Finish | Duration | Workflow outcome | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| screen-01-001 | 1 | ack-validation-unlimited | 00:21:31 | 01:06:27 | 44m 56s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/screen-01/logs/screen-01-001.exit.json) |
| screen-01-002 | 1 | control | 00:21:31 | 01:06:13 | 44m 42s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/screen-01/logs/screen-01-002.exit.json) |
| screen-01-003 | 1 | command-guidance-neutral | 00:21:31 | 01:08:14 | 46m 43s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/screen-01/logs/screen-01-003.exit.json) |

### Work-unit experiment: 12/12 terminal, 11 successful workflows and 1 failure

Execution: **02:34:42–06:18:49 (3h 44m 07s)**, four waves of three. The policy/handoff hypothesis was not confirmed (46 control versus 50 treatment accepted acknowledgments; exact p = 5/12). Workflow 010 failed the pre-linearization contribution gate and was retained. The token comparison remains unavailable in the frozen report because of the documented post-compaction accounting rejection; it was not silently repaired or excluded. [Phase result](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/PHASE-RESULT.json), [study interpretation](bench-results/efficiency-mechanism-isolation-20260906T214448Z/STATE.md).

| Run | Wave | Condition | Start | Finish | Duration | Workflow outcome | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| work-units-01-workflow-001 | 1 | buildable-work-unit-incremental | 02:34:42 | 03:15:11 | 40m 29s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-001.exit.json) |
| work-units-01-workflow-002 | 1 | control | 02:34:42 | 03:31:44 | 57m 1s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-002.exit.json) |
| work-units-01-workflow-003 | 1 | buildable-work-unit-incremental | 02:34:42 | 03:56:38 | 81m 55s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-003.exit.json) |
| work-units-01-workflow-004 | 2 | control | 03:56:38 | 04:41:02 | 44m 24s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-004.exit.json) |
| work-units-01-workflow-005 | 2 | buildable-work-unit-incremental | 03:56:38 | 04:32:37 | 35m 59s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-005.exit.json) |
| work-units-01-workflow-006 | 2 | control | 03:56:38 | 04:32:47 | 36m 9s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-006.exit.json) |
| work-units-01-workflow-007 | 3 | buildable-work-unit-incremental | 04:41:02 | 05:20:55 | 39m 52s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-007.exit.json) |
| work-units-01-workflow-008 | 3 | buildable-work-unit-incremental | 04:41:02 | 05:31:46 | 50m 44s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-008.exit.json) |
| work-units-01-workflow-009 | 3 | control | 04:41:02 | 05:35:17 | 54m 15s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-009.exit.json) |
| work-units-01-workflow-010 | 4 | control | 05:35:17 | 06:12:08 | 36m 51s | failure (exit 1) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-010.exit.json) |
| work-units-01-workflow-011 | 4 | buildable-work-unit-incremental | 05:35:17 | 06:18:49 | 43m 32s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-011.exit.json) |
| work-units-01-workflow-012 | 4 | control | 05:35:17 | 06:12:17 | 37m 0s | success (exit 0) | [receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/logs/work-units-01-workflow-012.exit.json) |

## Where the elapsed time went

This reconstructs wall-clock periods from run receipts, retained reviews and commit timestamps; it is not a claim that every minute was benchmark generation. Reviews and diagnostics sometimes overlapped running benchmarks.

| Rome time, 7 September | Activity and retained result |
| --- | --- |
| Before 00:21 | Source inspection, initial benchmark-only prompt controls, accounting and launch preparation; first retained real diagnostic starts at 00:02. |
| 00:21–01:08 | Initial three-workflow screen ran in parallel; no qualifying candidate. |
| 01:08–02:34 | Analysis of the screen; implementation, diagnostics and validation of the distinct work-unit experiment. |
| 02:34–06:18 | Twelve work-unit workflows ran in four waves of three; source-based handoff classification/review also proceeded during this period. |
| 06:18–07:28 | Finish complete classification and independent reviews; preserve the failed primary and compaction accounting issue; trace an exact retained-input episode. The classification freeze is at 07:05 Rome. |
| 07:28–09:13 | Source-exposure audits; implement the benchmark-only bundled-read versus inline-read factor; future-only accounting and complete read-census helpers; release build, real diagnostics and required tests/reviews. Final prepared-source test review is at 09:12:55. |
| 09:08–09:55 | Subscription-capacity gate: 95% weekly usage at 09:07:52, reset scheduled 09:53:49, 0% verified 09:54:43, batch admitted 09:55:36. This interval overlaps the end of preparation; it must not be added twice. |
| 09:55–10:39 | Current read-factor wave 1 ran; one success and two retained idle-limit failures. |
| From 10:39 | Current read-factor wave 2 and subsequent fixed waves; live status and all twelve rows are above. |

Some overhead came from operator/test-setup errors, not useful benchmark generation: failed diagnostic setups and their qualified follow-ups, including my omission of the primary observer marker in the first two read diagnostics; and 18 direct-discovery test-packaging errors, followed by the separately verified exact-byte test materialization. These are retained, not hidden or counted as successful benchmark workflows. The investigation is not complete merely because its infrastructure is checked.

Evidence: [study state](bench-results/efficiency-mechanism-isolation-20260906T214448Z/STATE.md), [classification freeze](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/coding/operational/CLASSIFICATION-FREEZE.json), [failed read diagnostics](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/READ-INLINE-REAL-DIAGNOSTIC-OUTCOMES.md), [prepared-source review](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/PREADMISSION-REVIEW.md), [pre-reset check](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/QUOTA-PREWAIT.json), [post-reset check](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/QUOTA-POSTRESET.json). Implementation/evidence commit anchors: `21b34d0`, `2a69f86`, `d4d999c`, `7625efa`, `e87c281`, `9751fa4`, `5322562`, `4c6a4dd`, `e47003f`, `ffea5b1`, `4b594b7`.

## All real-agent diagnostic attempts in this investigation

**16 separate scenario attempts: 9 PASS and 7 FAIL**, not additional full benchmark workflows. Each has one app-server process. The observer contains 22 invocation records because six other invocations are local locked-validation commands. Five failed app-server captures lack an end receipt. Offline replays, test executions and audits are not new provider runs.

All times are **2026-09-07 Europe/Rome**. These are provider-process start/end timestamps, not invented scenario completion times. A missing end stays missing even when the test log records its failure duration.

| Diagnostic | Provider start | Provider end | Scenario outcome | Explanation | Evidence |
| --- | --- | --- | --- | --- | --- |
| handoff-control | 00:02:33.728 | missing receipt | FAIL | final-interrupt teardown race | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/handoff-control/observation/invocations/00000432216842466071-3200270/start.json) |
| handoff-command-guidance-neutral | 00:02:33.732 | missing receipt | FAIL | final-interrupt teardown race | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/handoff-command-guidance-neutral/observation/invocations/00000432216846701214-3200285/start.json) |
| handoff-ack-validation-unlimited | 00:02:33.735 | missing receipt | FAIL | final-interrupt teardown race | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/handoff-ack-validation-unlimited/observation/invocations/00000432216849107897-3200292/start.json) |
| normal-transport-01 | 00:06:37.802 | 00:06:43.342 | PASS | normal launch/continuation | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/normal-transport-01/observation/invocations/00000432460915850916-3206189/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/normal-transport-01/observation/invocations/00000432460915850916-3206189/end.json) |
| handoff-v2-ack-validation-unlimited | 00:12:19.496 | 00:12:36.026 | PASS | corrected handoff fixture | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/handoff-v2-ack-validation-unlimited/observation/invocations/00000432802610573558-3228084/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/handoff-v2-ack-validation-unlimited/observation/invocations/00000432802610573558-3228084/end.json) |
| handoff-v2-control | 00:12:19.499 | 00:12:30.840 | PASS | corrected handoff fixture | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/handoff-v2-control/observation/invocations/00000432802612767710-3228092/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/handoff-v2-control/observation/invocations/00000432802612767710-3228092/end.json) |
| handoff-v2-command-guidance-neutral | 00:12:19.502 | 00:12:34.478 | PASS | corrected handoff fixture | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/handoff-v2-command-guidance-neutral/observation/invocations/00000432802616251282-3228105/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/handoff-v2-command-guidance-neutral/observation/invocations/00000432802616251282-3228105/end.json) |
| work-units-control | 01:51:51.423 | 01:52:04.140 | FAIL | fixture round limit; provider exit 0 is not scenario success | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-control/observation/invocations/00000438773369467731-13/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-control/observation/invocations/00000438773369467731-13/end.json) |
| work-units-incremental | 01:51:51.437 | 01:52:06.048 | FAIL | fixture round limit; provider exit 0 is not scenario success | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-incremental/observation/invocations/00000438773383260704-13/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-incremental/observation/invocations/00000438773383260704-13/end.json) |
| work-units-control-v2 | 01:56:25.343 | 01:56:41.234 | PASS (plumbing) | observer-identity qualification below | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-control-v2/observation/invocations/00000439047289343048-13/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-control-v2/observation/invocations/00000439047289343048-13/end.json) |
| work-units-incremental-v2 | 01:56:25.354 | 01:56:39.807 | PASS (plumbing) | observer-identity qualification below | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-incremental-v2/observation/invocations/00000439047300303196-13/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-incremental-v2/observation/invocations/00000439047300303196-13/end.json) |
| work-units-trust | 02:03:46.714 | 02:04:01.133 | PASS | final-observer trust check; two sessions | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-trust/observation/invocations/00000439488660812093-20/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-trust/observation/invocations/00000439488660812093-20/end.json) |
| read-inline-untracked-read-inline | 08:06:59.679 | missing receipt | FAIL | operator omitted primary capture marker | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/read-inline-untracked-read-inline/observation/invocations/00000461281625912714-74284/start.json) |
| read-inline-control | 08:06:59.680 | missing receipt | FAIL | operator omitted primary capture marker | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/read-inline-control/observation/invocations/00000461281626217853-74285/start.json) |
| read-inline-primary-control | 08:11:55.222 | 08:12:13.787 | PASS | corrected settings; complete capture verified | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/read-inline-primary-control/observation/invocations/00000461577168765160-98337/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/read-inline-primary-control/observation/invocations/00000461577168765160-98337/end.json) |
| read-inline-primary-untracked-read-inline | 08:11:55.222 | 08:12:11.515 | PASS | corrected settings; complete capture verified | [start](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/read-inline-primary-untracked-read-inline/observation/invocations/00000461577168762786-98336/start.json), [end](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/read-inline-primary-untracked-read-inline/observation/invocations/00000461577168762786-98336/end.json) |

The initial three handoff tests stopped the provider while its final interruption was held; their teardown failures remain recorded. The first two work-unit fixtures allowed four responses when the scenario required five, so they failed despite provider exit 0. The corrected work-unit scenarios passed their plumbing checks, but all four work-unit captures retain the later observer-executable identity flag: that qualified PASS is not full accounting readiness. The separate trust diagnostic supplies final-observer verification. The first two read-inline diagnostics are my missing-primary-marker error; the corrected two are separate attempts, not overwritten predecessors.

Outcome authorities: [handoff inventory](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/REAL-VERIFICATION.json), [work-unit fixture diagnosis](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/FAILED-WORK-UNIT-SMOKE-01.md), [work-unit observer scope](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/WORK-UNIT-OBSERVER-VERIFICATION.md), [work-unit delivery audit](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/WORK-UNITS-V2-DELIVERY-AUDIT.json), [trust result](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/work-units-trust/RESULT.json), [failed read diagnostics](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/READ-INLINE-REAL-DIAGNOSTIC-OUTCOMES.md), [corrected read verification](bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/READ-INLINE-PRIMARY-DIAGNOSTIC-REVIEW.md).

## Earlier pilot and accepted historical cohort

These executions precede the overnight mechanism study. They are listed separately, not added to its live 3 + 12 + 12 phase counter and not pooled with the current experiment. This is not a lifetime total obtained by summing every report: later analyses reuse earlier executions.

### Previous evening's two-workflow pilot

Both ran on **2026-09-06, Europe/Rome**, through the subscription, in separate checkouts. The explicitly approved pilot had two observations, so it did not authorize a third.

| Run | Mode | Start | Report publication | Outcome |
| --- | --- | --- | --- | --- |
| work-leaf-001 | Normal concurrent WL | 21:14:48 | 22:02:26 | Workflow and final checks pass; exit 0 |
| direct-001 | Direct sequential | 21:14:48 | 22:58:49 | Workflow and final checks pass; exit 0 |

The runner recorded both terminal rows at pair closure around 22:58:49; WL's earlier report publication above is documented separately. Driver durations, excluding final candidate replay/publication, are WL 46m 26s and Direct 1h 43m 42s. The frozen comparison is inconclusive; a separately labeled conditional supplement gives 41.51%–51.53%, not the historical interval and not a substitute for the frozen result. [Admission](bench-results/efficiency-raw-token-pilot-20260906T185328Z/RUN-ONCE/admission.json), [two-run ledger](bench-results/efficiency-raw-token-pilot-20260906T185328Z/FIRST-BATCH-RESULT.json), [result/timing explanation](bench-results/efficiency-raw-token-pilot-20260906T185328Z/RESULT.md).

### Accepted historical 6 + 6 cohort

These twelve retained endpoint executions underpin the accepted **45.38%–51.62%** bounded raw-token reduction. They were already complete before this continuation. The user's no-quality-loss assumption applies to the current causal investigation; original feature scores remain in their source evidence. No new sequential run is being added by this ledger.

| Run | Group | Workflow outcome | Source evidence |
| --- | --- | --- | --- |
| point7-exact-direct | direct | pass | [source](bench-results/efficiency-point7-exact-accounting-20260828T113610Z/runs/direct/point7-exact-direct-three-feature-sequential-bench-artifacts/report.json) |
| direct-003 | direct | pass | [source](bench-results/efficiency-points8-9-20260828T145556Z/evidence.json) |
| direct-002 | direct | pass | [source](bench-results/efficiency-points8-9-20260828T145556Z/evidence.json) |
| step4-direct-001 | direct | pass | [source](bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/report.json) |
| step4-direct-002 | direct | pass | [source](bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-002/corrected-study-step4-direct-002-three-feature-sequential-bench-artifacts/report.json) |
| step4-direct-003 | direct | pass | [source](bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-003/corrected-study-step4-direct-003-three-feature-sequential-bench-artifacts/report.json) |
| exact-normal-001 | normal_work_leaf | pass | [source](bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-001/exact-normal-001-three-feature-bench-artifacts/report.json) |
| exact-normal-002 | normal_work_leaf | pass | [source](bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-002/exact-normal-002-three-feature-bench-artifacts/report.json) |
| exact-normal-003 | normal_work_leaf | pass | [source](bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-003/exact-normal-003-three-feature-bench-artifacts/report.json) |
| exact-normal-004 | normal_work_leaf | pass | [source](bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-004/exact-normal-004-three-feature-bench-artifacts/report.json) |
| exact-normal-005 | normal_work_leaf | pass | [source](bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-005/exact-normal-005-three-feature-bench-artifacts/report.json) |
| exact-normal-006 | normal_work_leaf | pass | [source](bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-006/exact-normal-006-three-feature-bench-artifacts/report.json) |

Authorities: [historical report](bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/FINAL-REPORT.md), [twelve-row evidence](bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/evidence.json). [Later mechanism synthesis](bench-results/efficiency-mechanism-attribution-20260830T081131Z/evidence.json) reuses these endpoint runs; it is not twelve additional executions.

## Evidence already established

The corrected subscription diagnostics verified exact delivered inputs and response-level accounting: the bundled-read input item was charged 116 input tokens per subsequent response, versus 8,514 for the full-inline item in that diagnostic. This demonstrates retained-input charging, not the net effect or the causal share of the historical savings. Whole-workflow confirmation is still running.

The implementation and frozen study inputs are committed. Required Rust checks passed (449 tests passed, eight ignored); the frozen-source study/gate test materialization passed 265 tests. Diagnostic failures and the original test-packaging failure remain recorded alongside their qualified follow-up results.

## Next steps

1. Let active treatments 005 and 006 reach their bounded outcomes; keep future control admissions blocked. Preserve every completed, failed and unlaunched original identity.
2. Finalize the stopped supervisor safely after those active children exit. Retain the operator-induced configuration-monitoring gap and incomplete schedule; do not run or claim the complete twelve-workflow randomized primary.
3. Use the existing WL baseline and retained treatment evidence to investigate the mechanism. Keep exact source/response accounting, missing measurements and scope limits explicit; any partial-phase analysis is labeled separately from the frozen primary.
4. Inspect actual bundle retrievals and exact input-to-response charges, then report the mechanism and supported effect without launching additional control workflows.

## Evidence locations

- [Study state](bench-results/efficiency-mechanism-isolation-20260906T214448Z/STATE.md)
- [Frozen phase commands](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/COMMANDS.md)
- [Admission receipt](bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/RUN-ONCE/admission.json)
- Phase artifacts: `bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/`
- Runtime checkouts: `/tmp/work-leaf-untracked-reads.jUcFKF/`
- Pre-stop admission commit: `4b594b7` (subscription-capacity and admission evidence).

This is a provisional live status note, not a final result. No interim arm totals or token contrasts are being inspected.
