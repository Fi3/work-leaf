# Superseded Point-7 Result

## Status

This run is preserved but its token comparison is invalid. Later exact-accounting work proved that
normal Work Leaf interrupts model responses at orchestrator directives while the Codex transport
reports usage only for completed responses. The observer used here treated the remaining cumulative
totals as complete, so it systematically omitted tokens from interrupted Work Leaf responses.

The corrected investigation is in
`../efficiency-point7-exact-accounting-20260828T113610Z/FINAL-RESULT.md` and its transport evidence is
in that directory's `FAILURE-ANALYSIS.md`.

## Preserved Diagnostic Outcome

Both saved candidates passed the three frozen feature checks and their final repository checks:

| Workflow | Result | Features | Recorded raw tokens | Recorded uncached tokens |
| --- | --- | ---: | ---: | ---: |
| Concurrent Work Leaf | pass | 3/3 | 15,797,616 | 1,076,464 |
| Direct sequential Codex | pass | 3/3 | 30,999,451 | 1,459,739 |

The direct accounting remains usable. The Work Leaf accounting does not. Therefore the formerly
reported 49.039% raw and 26.256% uncached reductions are withdrawn and must not be used as evidence
of a normal-workflow token saving.

The quality result remains useful: this run produced two 3/3 candidates from the same frozen task,
base, model, reasoning level, normal validation policy, final checks, and scorer. It does not repair
the missing Work Leaf token measurements.

`PROVISIONAL-RESULT.md` and `result.json` preserve the scorer output generated before the accounting
defect was known. They are audit artifacts, not current conclusions.
