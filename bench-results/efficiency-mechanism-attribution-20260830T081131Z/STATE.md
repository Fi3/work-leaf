# Study State

## Current Position

The study has exact main-control measurements and a descriptive allocation of the collected raw-token
gap. Normal-endpoint measurement remains incomplete: 35 interrupted responses have no provable
terminal usage. The requested conclusion about at least 90% causal coverage is not established.

## Valid Measurements

- The three compact-direct and three sequential Work Leaf main-control runs have exact usage.
- The main protocol transition is 35,659,265 minus 19,311,710, or 16,347,554 raw tokens.
- The three completed-response and three combined-control Work Leaf runs have exact usage.
- The normal six-run Work Leaf endpoint has 35 unresolved interrupted responses across five runs.
- Raw-event replay proves that each unresolved gap contains one response and no intervening tool
  boundary. The derived 386,400-token maximum per response produces a normal Work Leaf mean of
  17,471,532-19,725,532.

## Result

The normal endpoint reduction is 45.38%-51.62% raw tokens in the collected samples under the declared
missing-response ceiling. The main-control sample means differ by 16,347,554 raw tokens. Its ordered
share is 87.68%-99.74%. Mediated reads plus directive interruption range from 325,910 more to 1,928,090
fewer tokens. The two mechanism groups
net to 97.75%-98.02% of the endpoint gap; their individual endpoint shares are not both positive across
the full bound. These ranges describe missing-usage uncertainty with sample means held fixed; they
are not confidence intervals for expected effects or for causal coverage.

The normal-endpoint uncached direction is not established. The exact main controls use 15.42% more
uncached tokens under sequential Work Leaf despite using fewer raw tokens. The normal groups score
17/18 for direct Codex and 13/18 for Work Leaf; the main controls score 9/9 and 8/9. Neither comparison
establishes equal-quality efficiency.

Exact normal-workflow telemetry, a predetermined quality criterion, and a sampling precision and
stopping rule are prerequisites for the next measurement study. No further provider runs are part of
this saved analysis.

The authorized gate and first measurement batch are defined in
[`PROTOCOL.md`](../efficiency-measurement-gate-20260906/PROTOCOL.md).

`FINAL-REPORT.md` is the human-readable authority. `evidence.json` contains the bounded endpoint and
allocation scenarios.
