# Frozen private catalog input disappeared before admission

Discovered 2026-09-15 03:59 UTC while preparing P09 integration continuation.
The manifest guard fails before admission/generation because four pinned files
under `/tmp/work-leaf-p01-cache-view.Xc6Oc6/cache/openai-curated-remote/deep-research-work/0.1.15`
are absent: `.app.json`, `.codex-plugin/plugin.json`, `skills/deep-research/SKILL.md`
and `skills/deep-research/agents/openai.yaml`. Every remaining checked cache pin
matches. All four public installed counterparts match their original hashes.

The old P01 `private_catalog.enter_view` bind-mounts the shared pinned cache
writable inside each provider namespace. Several non-generating app-server
configuration checks preceded the discovered deletion, but the precise writer
has not been identified; temporal proximity is not proof of its cause.
The current defect is that immutable study inputs are exposed to mutation.

Research remains 80/10/90. This is an unforeseen setup bug inside P09, not a new
token-saving hypothesis. The bug ledger records it separately as BUG011 before
repair. The partially prepared phase has no manifest, admission or model call;
its frozen partial files and failed prepare command remain retained.

## Prospective bounded repair

2026-09-15 04:03 UTC, twenty local minutes. Restore only the four exact missing
non-credential files from their hash-matching installed counterparts. Do not edit
the installed/public catalog, invent skill contents, alter old pins, copy auth
material or change normal WL. Preserve the old writable-mount implementation;
qualify a benchmark-only immutable namespace view using fail-first mount tests
and actual pinned-client readback before any generated verification.

If the local repair qualifies, a separate admission may use one fresh two-turn
60-second/100k subscription probe of the same integration-input boundary. No full
replacement batch or extra scientific observation is authorized by this scope.
All earlier diagnostics remain outcomes. Check actual complete input, preserved
cache/config/source hashes and owned accounting; a working mount is not a
mechanism result. Stop this attempt at its bound and continue useful in-scope work.
