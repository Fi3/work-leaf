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

## Shallow-screen and resource gates

Research selection is adaptive: maximize verified explanation of the historical ~50% result per
unit of time and subscription usage. The live ranking in `../hypotesis.md` records the best current
candidates, supporting evidence, potential reach, next small check and stop/advance conditions.
Re-rank after material evidence; a promising interaction can precede unfinished individual checks.
Neither ID order nor inventory completion is a prerequisite for testing a high-value joint cause.
Keep screens short and representative; deprioritize weak results instead of repeatedly extending
them. More involved confirmation requires an actual mechanism signal and a separate bounded scope.

Hypothesis screening precedes expensive confirmation and substantial custom experimental machinery.
The operator first checks prior candidate outcomes and the smallest representative saved or local
case. A small real-agent screen is appropriate when the hypothesis concerns model behavior; local
replay alone cannot establish a behavioral token effect. Small means a bounded decision-relevant
scenario, not a toy happy path or a shortened task passed off as a full-workflow result.

Before generation, the prospective screen record specifies:

- The hypothesis, its prior evidence, and what distinguishes it from previous unsuccessful tests.
- One changed factor or an explicitly declared joint-factor set, the exact predicted behavior,
  and the non-target obligations held fixed.
- The selected scenario and observation count, with separate harness and mechanism success criteria.
- Explicit wall-time and usage budgets, their measurement source and cancellation/drain behavior.
  Delayed or incomplete usage requires a bounded independent fallback, such as wall time or
  invocation count; an observed-usage threshold does not guarantee against in-flight overshoot.
- Stop, shelve and escalation criteria, including representative rejection/retry/stale-state tests
  for any affected experimental host. A hard budget or safety stop does not await a whole wave.

Monitoring and resource stops are declared before admission and remain compatible with the phase's
analysis hold. They do not authorize optional live outcome analysis, adaptive tuning, deleting
expensive outcomes, silent replacements, or changing an already admitted protocol. Completed and
partial outcomes, missing usage and stop reasons remain in the record. A failure in experimental
machinery is not a falsified hypothesis, but neither does it automatically justify more machinery
or another full batch. A functioning smoke test is not evidence that the hypothesis is promising.

Only a relevant observed mechanism signal makes a larger experiment eligible for consideration;
the applicable study/user authority is still required. A positive shallow screen is not an exact
causal percentage, and an inconclusive one is not proof of no effect. Stop/deprioritize instead of
repeatedly spending until a desired reduction appears. The three-workflow default applies to
approved full batches; explicitly approved smaller screens need not fill three slots.

The primary hypothesis register is `../hypotesis.md`; `../ephemeral-note.md` is the operational
chronology. Read both before resuming mechanism work. At check start and after every result, failure,
stop or new hypothesis, maintain dated methods, evidence, effect qualifications and current activity
for affected individual and joint entries. Update before reporting progress; proposed or pending
checks are not running experiments. Keep all prior outcomes and original frozen source reports.
Keep the register's fixed-scope progress totals and explicit remaining-check list consistent with
its dated closure records. TODO means selected necessary pending work only. Completed negative,
inconclusive or deprioritized screens are DONE; proposals and scientific uncertainty are separate
from the work queue. A concrete new reason and explicit selection precede any scope extension;
an unproved zero effect is not a reason to repeat a screen. Do not treat shared evidence as
independent experiments. The user-approved 2026-09-14 finite task counter is
DONE+TODO=TOTAL=74: all 66 completed records plus eight approved tasks, R13–R20.
The header and visible rows derive from the ledger linked in `../hypotesis.md`.
The approved task contract freezes identities, questions, observation/preparation ceilings and
completion criteria. Reordering by promise is allowed; adding, splitting, replacing, reopening,
broadening or hiding extra work as an uncounted substep is not. A dated result with evidence and
honest limitations closes one task, including a failed-setup or inconclusive bounded outcome.
Those dispositions are not scientific disproof.

DONE cannot decrease and TODO cannot increase without subsequent explicit user approval.
Record newly necessary work as an unapproved scope issue, without executing it. At zero, stop:
report the supported answer or explicitly identify case (1) additional work beyond the list
and/or case (2) an initially incorrect/omitted checklist; list the exact proposed tasks and
requested increase, and ask permission before increasing TODO or doing that work. No self-approved
reset or hidden continuation is allowed. Preserve the zero checkpoint. Zero is checklist exhaustion,
not an automatic claim of verified causal attribution.

The stable-ID ledger and mandatory regression command are linked at the top of
`../hypotesis.md`; require a passing guard before every progress report or affected commit.
The sibling `PROGRESS-PUBLICATION-HISTORY.json` separately preserves the old 52/3 and 54/1
checkpoints, the once-approved 54/12 correction, the 66/0 checkpoint and approved 66/8/74 extension.
The pinned extension contract preserves the original zero ledger and publications byte-for-byte.
The additive `test_progress_extension.py` runs alongside the unchanged original regressions.
Each completion appends one matching
checkpoint and its result declaration. Never rewrite or truncate earlier checkpoints.
A pinned contract rejects task substitution or altered limits; comparing both histories rejects
ledger-only rollback. This is accidental-drift protection, not defense against an operator
deliberately rewriting the contract, both records and guard. Numeric/file validation does not
establish scientific truth. The finite final coverage review must flag necessary omissions instead
of performing new experiments inside an analysis row.
Record screen decisions and budget outcomes in the operational chronology. Operator guidance belongs
to the supervising investigation; preserve frozen measured-agent instructions and source snapshots.

## Current mechanism-investigation authority

The user's 2026-09-14 “ok you can now proceed with the new 8 task” authorizes R13–R20.
The sibling study record `AUTHORITY-EIGHT-TASK-EXTENSION-20260914.md` records this authority;
`PROGRESS-TASK-EXTENSION-20260914.json` freezes the eight scopes and their ceilings.
The task definitions and budgets remain fixed; listing a task is not a provider admission.
Earlier continuation authority below does not enlarge this scope.

The token-mechanism investigation reuses the existing normal-WL baseline. Additional normal-WL
control workflows or unmodified Direct sequential controls require fresh explicit user authorization. The
user's 2026-09-07 instruction prohibits further controls; an optional fresh randomized design is
not a prerequisite for investigating a benchmark-only factor against the saved baseline.

The active research pursues the highest-value mechanism or joint mechanism first, using the live
ranking and actual selected work in `../hypotesis.md`. The user's 2026-09-12 resumption permits
adaptive candidate selection and representative shallow checks; it is not an inventory-only pause
or a requirement to complete all single factors before interactions. Earlier candidate-first plans,
phase protocols and stopped-analysis scopes remain historical records. Current prospective records
must state each selected screen's exact observations, budgets, invariants and stop conditions before
generation; no past manifest admits another observation. Preserve the existing baselines and all
failed/partial outcomes. A new normal-WL or unmodified Direct control is not implied by resumption.

Compare modified WL with the existing baseline after checking source, configuration and accounting
compatibility from retained evidence. Keep non-target behavior unchanged. Further repetitions concern
the modified version when supported by the investigation, not automatic baseline replenishment.

A later user stop overrides future admissions from an earlier schedule. Preserve the original
manifest, completed and failed outcomes, and unlaunched identities; do not rewrite them as a completed
comparison. The live investigation ledger is `../ephemeral-note.md`.

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

## Raw-token pilot

`bench-results/efficiency-raw-token-pilot-20260906T185328Z/FIRST-BATCH-MANIFEST.json` admits exactly one
Direct sequential workflow and one normal concurrent Work Leaf workflow, followed by a pause.
Its two scheduled runs are a smaller pilot, not evidence of a two-workflow capacity restriction.
`bench-results/efficiency-measurement-gate-20260906/PROTOCOL.md` describes that pilot's study rules;
the admitted manifest's frozen copy identifies its actual prelaunch protocol.

## Mechanism-isolation study

`bench-results/efficiency-mechanism-isolation-20260906T214448Z/PROTOCOL.md` governs the separate
WL-only causal investigation authorized after the pilot result. Its initial screen contains one
control and two single-factor prompt variants in one three-workflow wave; a separately frozen
confirmation uses fresh observations. The study permits benchmark-only implementation controls
while preserving default Work Leaf behavior, subscription authentication, and non-target settings.
Each phase's `PHASE-MANIFEST.json` is its admission authority. Screening observations are not
confirmation evidence, failed observations remain retained, and the completed pilot's records are
not rewritten to cover the later study.

`bench-results/efficiency-mechanism-isolation-20260906T214448Z/PROTOCOL-WORK-UNITS.md` specifies a
distinct v2 work-unit-policy phase with twelve fresh WL workflows in four mixed waves of three.
Its automatic delivered-handoff outcome and fixed-sequence substantive-work test distinguish
causal continuation behavior from incomplete token-accounting totals. Allocation is not admission;
the phase requires its own frozen manifest, exact prompt-delivery audit and future-only verified
own-workflow trust-transition policy. The original screen's no-candidate decision, configuration
flag and unsuccessful diagnostics remain retained and are not replacement observations.

`bench-results/efficiency-mechanism-isolation-20260906T214448Z/PROTOCOL-UNTRACKED-READ-INLINE.md`
records the original v3 read-representation schedule: twelve fresh WL workflows, six per arm,
in four mixed waves of three. Its factor replaces only successfully bundled, currently untracked
project snapshots with their exact full-inline representation. Both arms retain ordinary bundle
creation and symmetric candidate evidence. Default builds and all non-target mechanisms remain
ordinary. A frozen phase manifest, real subscription delivery checks, strict future-only response
identity accounting and complete terminal outcomes are required before its whole-workflow raw-token
test. Earlier null phases remain separate; no favorable endpoint replacement or adaptive extension
is permitted.

The original v3 schedule is retained as evidence, not authority to launch further controls after the
user's stop. Its first six identities were launched; remaining admissions are held. The operator
record is `bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01/OPERATOR-CONTROL-STOP.json`.
