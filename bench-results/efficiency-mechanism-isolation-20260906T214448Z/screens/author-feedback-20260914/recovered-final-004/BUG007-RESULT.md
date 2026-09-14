# BUG007 — completed-turn recovery is verified

Verified: 2026-09-14T22:39:53+00:00.

recover_host.py accepts an in-turn diagnostic only when the original strict
unique-thread/turn, final-item and matching-final-file checks all succeed.
The unchanged invoke path still requires child exit 0, no timeout and no
cancellation. Terminal turn.failed remains fatal. Public diagnostic text is
retained in host evidence. The frozen original host, admitted P03/P04 sources
and normal Work Leaf implementation remain untouched.

BUG007-RED.txt records two failures against the old acceptance branch, including
the actual saved native response. RECOVERY-RED.txt records four fail-first
restoration cases. BUG007-GREEN.txt records all eleven tests passing, including
terminal failure, incomplete/duplicate ownership, stale data, consumed work,
wrong prompt, prefix drift and once-only recovery. Cargo fmt, strict Clippy and
all-target/all-feature cargo test also pass.

Actual agent scenario: P03-recovered004 consumes the original completed final
and resumes native thread 01a0a1f2-54aa-7983-b2d5-c80b87ce2ab9 at invocation 2.
BUG007-ACTUAL-BOUNDARY.json establishes that the original final/evidence and
1,259,214-byte native prefix remain intact, and the real provider receives the
actual B-off host feedback byte-for-byte. P03-BOUNDARIES-001.json verifies the
expected native model/effort, read-only/never, base/developer content and first
two catalog states. No new launch/operator prompt or repeated first response.
The resumed turn is still active at this verification checkpoint; this repair
result is not an author completion or a scientific causal result.

Research remains 77/13/90. The separate bug ledger closes BUG007. Original
1,154.27-second failed-host interval and all original 575,731 raw are retained.
Final continuation accounting and mechanism analysis follow its own closure.
