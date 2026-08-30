# Combined Read And Continued-Response Control

## Question

How much do mediated reads and immediate response interruption overlap?

The completed controls show that both affect token use, but the read route also changes how often
provider output resumes after a directive. Their separate percentages cannot be added without a
combined condition.

## Four Conditions

| Condition | File reads | Resumed output after a directive |
| --- | --- | --- |
| Normal Work Leaf | Orchestrator-mediated | Interrupt immediately |
| Direct-read control | Direct filesystem reads | Interrupt immediately |
| Continued-response control | Orchestrator-mediated | Wait for exact usage, then interrupt |
| Combined control | Direct filesystem reads | Wait for exact usage, then interrupt |

The combined condition is diagnostic. It is not presented as normal product behavior.

## Fixed Behavior

The combined control keeps the same frozen Work Leaf binaries, source checkout, concurrent feature
submission, three requests, GPT-5.5/`xhigh` profile, validation freedom, structured writes, locked
commands, review, linearization, final checks, exact accounting, and frozen `/status` scorer used by
the prior controls.

It combines only two already-tested switches:

1. `WORK_LEAF_BENCH_NO_READ_PERMISSION=1` gives patch agents and reviewers direct read-only file
   inspection. Despite its historical name, this is the existing direct-read mode.
2. `WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME=wait-for-usage` holds an interrupt through
   resumed output until exact usage or the 120-second bound.

Work Leaf source and prompts are not otherwise changed. The observer forwards the original
interrupt bytes and does not execute or interpret agent work.

## Analysis

Three independent combined workflows run concurrently. The four group means provide:

- the direct-read effect under normal interruption: direct-read minus normal;
- the continued-response effect under mediated reads: continued-response minus normal;
- the direct-read effect when responses continue: combined minus continued-response;
- the continued-response effect with direct reads: combined minus direct-read; and
- the interaction: combined minus direct-read minus continued-response plus normal.

The combined displacement from normal is compared with the direct-Codex versus normal-Work-Leaf
gap. Individual feature results, ranges, stages, usage changes, context per change, and timeouts are
reported. The arithmetic is a sample decomposition, not a population estimate.

## Counterchecks

- Every run must contain direct-read launch prompts, direct read commands, no mediated read
  directives, and at least one completed continued response.
- Every provider thread must reconcile to exact hash-verified usage. A bounded timeout remains a
  partial activation and is retained.
- The frozen scorer keeps successes, partial candidates, and failures. Lower quality cannot be
  treated as efficiency.
- The combined runs are not paired with prior runs. Collection concurrency is only an operational
  speedup.
- If the interaction is large, the separate percentages remain non-additive. If it is small, the
  sample effects may be combined arithmetically with the interaction term shown explicitly.
- No retry is allowed merely because the result is noisy, non-directional, or unfavorable.
