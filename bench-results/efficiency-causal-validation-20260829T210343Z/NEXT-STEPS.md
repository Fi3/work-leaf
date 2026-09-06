# Measurement Gate And Further Study

## Current Decision

The collected controls support a raw-token effect of the orchestration package on the frozen
three-feature Rust benchmark. Normal-endpoint measurement remains incomplete, the uncached-token
direction is unknown, and equal-quality efficiency and at least 90% causal coverage are not
established. The descriptive accounting bounds do not include sampling uncertainty.

The authorized next step is the exact-telemetry gate and first measurement batch in
[`PROTOCOL.md`](../efficiency-measurement-gate-20260906/PROTOCOL.md). A passing integrated real-agent
gate permits one direct sequential and one normal concurrent Work Leaf observation. That pilot
stops after the first batch regardless of the observed direction. An unmet telemetry gate keeps the
batch unlaunched; repeated bounded observations cannot supply missing exact usage.

## More Precise Percentages

Any collection after the pilot requires a separately frozen protocol. It must define a quality
estimand and numerical acceptance margin, a precision target, a finite sample limit, and a fixed
or valid sequential stopping rule before launch. Collect independent direct sequential and normal
concurrent Work Leaf observations under one frozen configuration with randomized order within
declared blocks; inference must respect those blocks. Use group means rather than fixed pairs,
retain every quality outcome, and report quality jointly with tokens.

Exact accounting is a prerequisite for that study. The primary metric is mean uncached input plus
output, with cached input, uncached input, output, and raw totals reported separately. Six runs per
group are not assumed sufficient for the selected precision or quality criterion. Normal-workflow
replication does not separate the remaining Work Leaf mechanisms.

## Exact Allocation Inside The Remaining Workflow

The current evidence groups structured edits, write-command mediation, focused concurrent
validation, exact review targeting, and compact linearization together. Separating them requires new
experimental controls because the production CLI has no switch that independently disables each
behavior.

The smallest useful controls would be:

1. A benchmark-only patch-agent mode with direct writes and direct write-producing commands while
   preserving Work Leaf scheduling, review, task text, and final checks.
2. A benchmark-only linearizer mode that receives the full uncompressed workflow history instead of
   the normal compact reviewed target.

Both controls alter Work Leaf behavior or prompts and are outside the authorized measurement pilot.
They require a separate experimental protocol, tests, an isolated build, and a fairness audit before
provider launch. Saved observations remain immutable.

## Statistical Confidence

The pilot produces descriptive artifact comparisons only. For any later study, exact telemetry,
quality acceptance, and sampling precision are separate gates. More runs cannot fix missing
interrupted-response telemetry, and collection must not continue merely until a favorable saving
appears. Causal-share inference also needs uncertainty for the controlled contrasts and endpoint;
a percentage allocation is undefined when the endpoint difference interval includes zero.

## Other Repositories

Repeat the frozen endpoint comparison in at least one unrelated Rust repository and one project
with a different toolchain. This tests whether the workflow-batching advantage is general or depends
on this repository's overlapping UI/controller features.
