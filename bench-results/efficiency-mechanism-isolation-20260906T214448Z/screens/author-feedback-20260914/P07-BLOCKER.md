# P07 — joint author/refresh/continuation boundary

Checked: 2026-09-14T21:13:06+00:00; scope P07-P08-LOCAL-SCOPE.md. No generation.
Status: BLOCKED, required experiment remains in TODO.

The declared A+B+C08+C25 package lacks a faithful existing common boundary:

- src/bench_experiment.rs::Manifest stores one schema/condition; load accepts
  one listed condition for each schema. C08's Capture::forward_with in
  src/bench_automatic_refresh.rs explicitly requires the single v7 condition.
- C25's ProviderUsageGrace::wait_before_interrupt in bench-observer/src/lib.rs
  is a separate observer seam. It can coexist structurally with C08; that is not
  the obstacle and does not qualify the missing historical execution state.
- A's shared test-only publication and actual RED differ from the ordinary
  shared-buildable policy translated by src/agent.rs around lines 513–534.
  v6 private test-first preview retains normal shared publication, not A's
  actual operation/state change.
- The retained standalone host invokes native exec/resume and serializes final
  replies. author_inverse.py::main calls only the initial-author stage. It has
  neither WL's file tracker/automatic refresh nor C25's directive interruption.

The saved EXISTING-WL-JOINT-VARIANT-FEASIBILITY.md under the global-hunk pilot's
postcapture directory independently retains these design limitations. P06's
exact first refreshed file can be reconstructed, but its complete runtime/
continuation checkpoint is not qualified.

Adding a compound condition alone would not repair the publication semantics,
restore the historical multi-agent state or qualify actual provider input.
The permitted bounded adapter work does not include a new orchestration/
historical-state restoration framework. No absent-path or zero-effect exclusion
is justified. The local check stops on those source-backed blockers.
