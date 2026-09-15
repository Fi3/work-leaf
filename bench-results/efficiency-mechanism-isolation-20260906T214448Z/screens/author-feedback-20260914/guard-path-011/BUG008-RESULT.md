# BUG008 result — verified observer-directory identity

Verified 2026-09-15T00:11:00+00:00. The regression fails before the repair:
an executable reached through a sibling guard's proc fd is inaccessible after
entering the private user namespace. It passes when the exact same observation
directory is addressed by its canonical path, with matching device/inode before
and after resolution. Source inversion shows setup_observer is the only changed
function; prompts, host, observer binary, profile and all workflow stages remain
the original qualified sources. Normal WL and admitted original sources are intact.

Two new tests and all 21 earlier adapter tests pass. Required cargo fmt,
all-target/all-feature Clippy with denied warnings and all-target/all-feature
tests pass (exec session 3454, exit 0).

The actual retained artifact guard plus extracted full setup_observer plus
unchanged private input wrapper/observer/profile/native host route completes in
14.997763 seconds: two responses/turns, 32,338 raw. Native/public usage and exact
user/developer input joins pass; no incomplete tail, source drift or file changes.
The real host executes git status successfully and the same author receives
its actual result and returns DONE. Native full developer hash equals the
qualified reference under the same newline-joining convention.

Evidence: BUG008-RED.txt, BUG008-GREEN.txt, test_guard_path.py,
test_source_inverse.py, real/NATIVE-AUDIT.json, real/TERMINAL.json,
real/guard.stdout and real/host/events.jsonl. The diagnostic's hidden guard
stage remains retained after guard closure, not published as a benchmark.

All three original full attempts remain startup failures with empty stdout and
no native thread, not replaced or reclassified. This repair establishes setup,
not a scientific token effect or historical explained percentage. A subsequent
in-scope P09 attempt requires its own prospective admission and retains this
distinction explicitly. Path preparation is linear in path length; no new
quadratic operation is introduced.
