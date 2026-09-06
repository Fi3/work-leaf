# Benchmark operator policy

## Concurrency and scope

The user's default is three complete benchmark workflows concurrently. Use up to three separate
checkouts and artifact directories when the approved schedule contains at least three runs. This is
top-level concurrency: three feature agents inside one Work Leaf workflow still constitute one
benchmark run. Three parallel benchmarks does not mean three Direct/WL pairs.

Concurrency capacity and the authorized observation count are distinct. A specifically approved
two-run pilot uses two slots; a remaining smaller batch can also use fewer. Do not invent a third
observation, change an admitted comparison, silently replace a failed run, or bypass a pause to fill
capacity. A new or expanded schedule requires the applicable study authority and a prelaunch record.
Never rewrite an admitted manifest or its frozen evidence to imply a different launch plan.

Codex benchmark generation uses the existing ChatGPT subscription and stored subscription login.
API-key authentication, API credits, provider switching, and credential copying are not permitted
continuation paths. The study protocol governs its model, effort, validation, interruption policy,
measurement assumptions, outcome retention, and stop condition.

## Recorded user authority

The source conversation is `01a05746-e2dc-7e62-a7fb-a276ef17bbb4`. Its locally retained history is:

```text
/home/user/.codex/sessions/2026/08/31/rollout-2026-08-31T12-04-18-01a05746-e2dc-7e62-a7fb-a276ef17bbb4.jsonl
```

- Line 211545 contains the explicit user instruction: “Alwasy run bench in paralle 3 at time”.
- Line 211537 contains the preceding proposal for up to three top-level workflows concurrently,
  preserving every outcome.
- Line 211590 clarifies that the three are complete benchmark runs running simultaneously.
- Line 211598 contains the user's confirmation to continue.

These records have imported replay timestamps around `2026-08-31T10:04:28Z`; those timestamps are
not asserted to be original execution times. The user reaffirmed this concurrency instruction in
the current conversation on 2026-09-06 and requested a durable repository record. An earlier
two-slot proposal in the history is not the general concurrency limit.

## Current admitted pilot

`bench-results/efficiency-raw-token-pilot-20260906T185328Z/FIRST-BATCH-MANIFEST.json` admits exactly one
Direct sequential workflow and one normal concurrent Work Leaf workflow, followed by a pause.
Its two scheduled runs are a smaller pilot, not evidence of a two-workflow capacity restriction.
`bench-results/efficiency-measurement-gate-20260906/PROTOCOL.md` describes that pilot's study rules;
the admitted manifest's frozen copy identifies its actual prelaunch protocol.
