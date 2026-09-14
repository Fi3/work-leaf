# BUG006 — explicit output discovery

Fixed 2026-09-14T22:00:41+00:00. Five fail-first tests cover arbitrary public output
layouts, a native rollout on a later date, duplicate public paths, ambiguous
native identity, pending native records and no-yet-started providers. The actual
saved three-turn subscription diagnostic replays as 48,827 native raw and
48,827 public raw, three distinct responses and no pending thread. The original
four zero resource samples remain preserved as missing coverage, not zero cost.

output_monitor.py accepts caller-declared files/directories and reuses the
original native counting and stop mechanisms. It is operator observation only:
no agent-facing prompt, launch, resume, runtime or token path changes. Saved
real-native replay verifies this fix without another model turn. Its five
regressions and the original resource/monitor regressions are required before
the new feature admissions. See BUG006-RED.txt, BUG006-GREEN.txt and
BUG006-ACTUAL-REPLAY.json. No main research task completes from this repair.
