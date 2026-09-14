# BUG004 — separate admission bookkeeping from experimental adaptation

Fixed: 2026-09-14T20:54:21+00:00. This is a report-data correction, not a validator or
budget change. The existing publication regression reproduces the incorrect
364-second experimental-preparation entries and passes with their original
zero values. BUG004-RED.txt and BUG004-GREEN.txt preserve both outcomes.

screens/author-feedback-20260914/ADMISSION-BOOKKEEPING.json retains the whole
20:28:03-20:34:07 interval once: 364 shared wall seconds for two parallel
admissions. It lists the clone/policy setup, frozen-input materialization,
source pinning, commit and monitor launch. The adapter/factor qualification
was already complete in P02; no extra experimental adaptation occurred.
The interval is not erased, doubled into independent labor, charged as model
usage or used to expand the immutable preparation/model ceilings.

All five publication/repair guard suites pass without modifying their rules.
The final author workflows remain incomplete at their original 900-second
walls. This correction cannot make a scientific task complete or admit another
observation. No runtime or agent-facing behavior is affected by report data.
