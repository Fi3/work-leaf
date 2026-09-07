# Workflow 010 operational termination

The retained outcome for `work-units-01-workflow-010` is a completed driver
invocation with exit code 1, classified as `recorded_workflow_failure`. Its
recorded failure reason is:

> expected all three patch agents to produce reviewed commits before linearize; got 2

The narrow operational cause is the frozen driver's pre-linearize contribution
gate. It counts distinct patch-agent commit authors and requires all three before
starting linearization. This record does not establish why any particular agent
had no counted commit.

## Retained identities and locators

All relative paths below are within
`/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01`.

| Source | SHA256 | Relevant locators |
| --- | --- | --- |
| `logs/work-units-01-workflow-010.exit.json` | `489ba5801916403da9ad07033aa7acdf7aa8d143c2ef2bbf788dc741e4838f6a` | `/finished_at` (line 7), `/launch_status` (line 9), `/launcher_exit_code` (line 10), `/outcome_classification` (line 11) |
| `runs/work-units-01-workflow-010/work-units-01-workflow-010-three-feature-bench-artifacts/report.json` | `2d08d4f214dd66954a044e97b0c21a2cc65654282b03e4767cfab71aeca980f5` | `/comment`, `/result`, `/workflow_result`, `/review_completed`, `/linearize_completed` |
| `infrastructure/drivers/bench-three-features` | `d2487780c63c14021904b8a3c882d54fe231c5846f4a7f57fe955f50201f5644` | commit-author count at line 1190; pre-linearize gate at line 1206; failure at line 1209; `fail_bench` at line 965 |

The receipt records `started_at: 2026-09-07T03:35:17.463755+00:00` and
`finished_at: 2026-09-07T04:12:08.170280+00:00`.
The report records `result: fail`, `workflow_result: fail`,
`review_completed: yes`, and `linearize_completed: no`.
The human-readable counterpart
`runs/work-units-01-workflow-010/work-units-01-workflow-010-three-feature-bench.md`
has the same failure comment at line 44 and completion flags at lines 31 and 32;
the machine report above is the hash-bound authority for these values.

## Frozen driver call chain

1. At line 1190, the driver reads commit bodies after the frozen base, extracts
   `Agent-ID: user-[0-9]+` trailers, deduplicates them and counts the distinct
   patch-agent identities. This value is not a count of ACKs or total commits.
2. At line 1206, the driver enters the pre-linearize branch only while
   linearization has not started, the orchestrator is idle, the three user
   sessions are terminal under its existing predicate, and ready users cover
   the counted commit authors.
3. It sets `review_completed="yes"` at line 1207, then requires the counted
   patch-agent authors to equal three. The recorded value of two triggers
   `fail_bench` at line 1209. The review flag is therefore a driver control-flow
   marker here, not evidence that every requested feature passed review.
4. `fail_bench` at line 965 records the failure reason and `result="fail"`,
   attempts the final state/status/log snapshots, calls `finish_observation 1`
   at line 971, publishes the report at line 972, and exits 1 at line 973.
5. This failure precedes the `force-linearize` command at line 1216. It does not
   enter the later accepted-linearization/final-checks branch.

## Scope and exclusions

The separate recorded usage incompleteness is not the condition that triggers
this failure branch. No token totals or usage-gap counts are reproduced here.
No final-test failure is inferred: the identified branch terminates before the
driver reaches linearization and its final repository gate. The report's quality
section and actual test outcomes are not inspected.

This diagnosis uses terminal metadata, report header fields, and frozen source
control flow only. It does not inspect semantic messages, assign ACK labels,
identify an agent's underlying reason for missing a counted commit, or compare
conditions. The failed workflow remains retained; no replacement or zero-fill is
performed. The existing 010 and 012 packets are unchanged.

No provider call, active-workflow inspection, frozen-file edit, or commit is made.
The sole write for this preservation task is this create-new operational report.
