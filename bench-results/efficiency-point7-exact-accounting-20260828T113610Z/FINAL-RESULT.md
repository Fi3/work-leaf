# Exact-Accounting Attempt

This directory records the failed attempt to obtain exact Work Leaf provider usage without changing
normal directive handling. It is not the current Point 7 conclusion. The completed conservative
result is in
[`../efficiency-point7-bounded-accounting-20260828T142614Z/FINAL-RESULT.md`](../efficiency-point7-bounded-accounting-20260828T142614Z/FINAL-RESULT.md).

## Abstract

Point 7 attempted to compare fair normal direct sequential Codex with normal concurrent Work Leaf on
the same three-feature Rust task, then collect a Work Leaf control with all three suspected
context-saving mechanisms disabled. The direct run succeeded with exact accounting. The Work Leaf
measurement failed because exact accounting required delaying Work Leaf's normal immediate handling
of model directives. That delay changed the workflow. No valid Work Leaf token total, comparison
percentage, or all-three-disabled control was produced.

## Results

| Condition | Workflow | Frozen feature score | Token measurement |
| --- | --- | ---: | --- |
| Direct sequential Codex | pass | 3/3 | exact and usable |
| Normal Work Leaf attempt | fail during review | 2/3 partial candidate | incomplete and unusable |
| Work Leaf, all three controls disabled | not launched | not measured | not measured |

The direct observation used:

- 40,789,986 input tokens;
- 39,052,544 cached input tokens;
- 1,737,442 uncached input tokens;
- 245,138 output tokens;
- 41,035,124 raw input-plus-output tokens; and
- 1,982,580 uncached input-plus-output tokens.

All 32 observed processes completed, including 18 Codex CLI invocations across seven provider
threads, and the provider totals reconciled with the observer. The candidate passed the visual
selection/copy, selected-agent `/status`, and reviewed-feature close/reopen checks.

The Work Leaf report contains 9,329,970 raw and 973,874 uncached tokens, but those numbers omit two
interrupted review responses and must not be compared with the direct total. The preserved partial
candidate passed visual selection/copy and `/status`, but not reviewed-feature completion.

## Why Work Leaf Could Not Be Counted Exactly

Normal Work Leaf processes a complete `@work-leaf` directive immediately and interrupts the current
model response. The Codex app-server path is:

1. `src/cli.rs::launch_agent_streaming_interruptible` or
   `src/orchestrator.rs::send_agent_streaming_interruptible` recognizes the directive.
2. `src/codex.rs::CodexAppServer::request_turn_streaming` sends `turn/interrupt`.
3. Codex stops consuming the provider stream.
4. Exact usage would arrive only in the provider's `response.completed` event, which does not exist
   for that interrupted response.

The attempted benchmark mode waited up to 30 seconds for usage before interrupting. In a short probe,
the directive happened to be the final model output, so this appeared workable. In the full workflow,
two reviewers emitted a read directive and continued reasoning. Waiting delayed the read, timed out,
and stopped the reviews. The instrumentation therefore changed the behavior being measured.

Two real-provider immediate-interrupt probes, a newer CLI check, a server-side usage query, source
inspection, and stored/background response probes all reached the same conclusion. The detailed
evidence is in `FAILURE-ANALYSIS.md`.

## Conclusion

This study does not prove or disprove a real Work Leaf token saving. It proves that the previous Work
Leaf totals were systematically incomplete: interrupted directive responses were not counted. The
older 49.039% raw and 26.256% uncached Point 7 reductions, the later attribution percentages, and the
historical all-controls-disabled totals cannot answer the normal-workflow question.

The all-three-disabled run was deliberately not launched after the defect was known. Running it on
the same transport would spend model time while producing another invalid total.

The next valid study needs either provider telemetry for cancelled responses or an explicitly agreed
approximate measurement method. Switching both workflows to another provider transport would also
be a new study and must be declared before collection.
