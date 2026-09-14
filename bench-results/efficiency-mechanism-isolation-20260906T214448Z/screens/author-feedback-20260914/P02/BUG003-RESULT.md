# BUG003 — native resume dispatch verified

Fixed: 2026-09-14T20:45:30+00:00. The private P02 adapter consumes native exec
options before identifying a resume subcommand. Fresh launches alone use the
pinned P01 catalog view; resumes pass through with unchanged argv. Opt-in and
recursive-provider rejection remain, and unsupported/incomplete commands fail
closed. The admitted P01 source/tests are untouched.

BUG003-RED.txt reproduces six assertions failing in the original dispatch;
BUG003-GREEN.txt records all seven regressions passing after the correction.
BUG003-REAL-RESUME.json records the actual P04 invocation-0002: the host's
`exec --color never resume` reaches the same native thread and completes with
exit0, no timeout or cancellation. Its 5,038-byte developer profile appears in
the saved reference, but not at the corresponding author turn. P10's BUG005
boundary check rejects the earlier pooled-profile interpretation. Native
permission/model/effort settings match; historical resumed-input equivalence
does not. P03 also completes actual native resumes with that catalog limitation.

The planned real verification is supplied by the already-approved P03/P04
feature episodes, specifically P04's first completed resume; no extra diagnostic
is run. This verifies dispatch, not either complete feature or any token-saving
claim. Both episodes terminate at their independent walls; P10's final complete
input/source/usage audit preserves their failed catalog qualification. No normal
WL runtime or public API is affected.
