# Recorded developer setting does not restore the resumed input

Checked: 2026-09-14T23:22:00+00:00.

The diagnostic finishes in 25.987904 seconds with three completed turns,
three distinct responses and 47,166 raw tokens. Source endpoints, exact prompt
joins and all four native/public usage fields match; no unfinished turn,
duplicate or compaction is recorded. The once-only native audit is retained in
NATIVE-AUDIT.json with its exact command and input hashes.

The local fresh-input preview contains the expected extra 1,014-byte message.
Actual same-thread resumes contain no developer update on either turn two or
three. Therefore this setting fails the intended resumed-input gate. A fresh
preview is not proof of resumed delivery. No full workflow is admitted from it.
The exact saved text is not silently removed or declared behaviorally irrelevant.

This configuration approach is parked without another diagnostic. The closed
native thread remains immutable. Both failed metadata-delivery approaches are
setup outcomes, not scientific nulls, and neither changes the research counter.
RECORDED-MESSAGE.txt has a file-format trailing newline; the configuration uses
the exact original message without that newline. Its file hash is not asserted
to equal the 1,014-byte model-message hash.

The next useful in-scope action reuses historical work classifications for P13's
transfer/reach analysis before building further P09 machinery. This does not
reopen old tests or the accepted historical saving premise.
