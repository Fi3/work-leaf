# BUG007 — recovered native diagnostic rejected as a terminal failure

Discovered: 2026-09-14T22:25:41+00:00. Related task: P03.

The saved P03-qualified002 first invocation exits 0 without timeout or
cancellation, emits one owned final and a turn.completed usage record, and
retains a 40,603-byte final (SHA-256
625204119a03f6dbd685e9dcb4a2c831ff05f832f16e79db062c536fe45fa006).
Its public stream also contains an earlier recoverable connection diagnostic.
The frozen standalone host's owned_final rejects every error event immediately,
so the host reports failure despite the subsequently successful native turn.
No proposed operation has been consumed; the checkout is still clean at its base.

Evidence: ../P03-qualified002/host/invocation-0001/{stdout.jsonl,exit.json,final.txt}
and ../P03-qualified002/host/result.json. The native turn records 575,731 raw
across ten completed responses. Preserve this full charge and the failed host
outcome; a recovered warning is not a failed provider response or zero usage.

The bounded repair is benchmark-local. Acceptance requires a successful child
exit, the unique complete owned turn/final and all existing ownership checks.
A diagnostic is acceptable only inside that successful turn; a terminal
turn.failed, incomplete/ambiguous ownership or mismatched final remains fatal.
No diagnostic-English-string or feature-specific recognition is permitted.
After fail-first tests, validate the saved real stream. A separately admitted
continuation may consume that original proposal once and resume its existing
thread with actual host feedback. It must not regenerate the first response or
send an operator-authored continuation prompt.
