# BUG010: inconsistent archive ordering in the accounting guard

Detected 2026-09-15T01:33:48+00:00 in the three closed P09 audits.
The aggregate template sorts initial files as pathlib.Path objects but compares
them with final files sorted as strings. Component ordering and string ordering
differ, for example for `candidate/file` and `candidate-build.log`. Unchanged
archives can therefore report `archive file population changed during audit`.
All three actual archives exhibit that ordering difference and retain their
original file counts, with no changed content hashes in the saved audit.

The two observer proxy symlinks resolve outside the archive in the content-hash
map; this explains their absence when filtering that map by archive prefix. It
is not evidence that the audit created those files. Postcapture output lies
outside the archived artifact and is not the cause.

Scope: a fail-first ordering regression, retained detection of real population
and content mutation, and an append-only supplemental aggregation using only
the existing once-audited per-thread cache. No native-core re-execution, provider
generation, edited original results, changed agent input or normal WL change.
Local bound: fifteen minutes from 01:35 UTC. This is a P09 accounting repair,
not an additional scientific task or evidence of a historical benchmark error.
