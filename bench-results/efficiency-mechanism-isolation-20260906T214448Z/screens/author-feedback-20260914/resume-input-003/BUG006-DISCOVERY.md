# BUG006 — multi-turn diagnostic resource discovery

Discovered 2026-09-14 21:58 UTC while preparing the feature pair.
The saved resource samples report zero threads/usage because resource_sample.py
selects native.stdout.jsonl or host/invocation-*/stdout.jsonl, whereas the new
three-turn diagnostic writes turn-*/stdout.jsonl. The diagnostic's own 60-second
host deadline, per-completion 100k check and three-turn ceiling still applied;
the actual audited terminal cost is 48,827 raw, with no unknown tail. This defect
does not invalidate its source/input receipt, but the external usage monitor
did not observe that diagnostic's costs live. Retain those zero samples as
incorrect coverage, not as zero usage.

Repair with explicit generic public-output paths and possible rollout directories
supplied by the caller; include native-date rollover and duplicate-source guards.
Reuse the original distinct-response counting function and stop supervisor.
Fail-first tests and replay of this actual native diagnostic verify the repair
without another model turn. Existing admitted sources are not edited. This is
a newly noticed reporting/monitoring defect, not a scientific task or a reason
to pause the research. Record it in the separate bug counter before fixing.
