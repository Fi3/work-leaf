# Recorded complete plugin-state transition qualified

Checked: 2026-09-14T23:28:09+00:00.

The diagnostic completes in 23.936574 seconds with 49,968 raw tokens, three
distinct responses and three complete native/public turns. All source endpoints,
exact prompt joins and all four usage fields match; no recorded compaction,
duplicate or unfinished turn remains. NATIVE-AUDIT.json is the once-only result.

Actual developer input is exactly the saved full initial payload, followed by
only the 5,038-byte short catalog, followed by only the 1,014-byte plugin-usage
instruction. The native world-state records true -> false -> true as predicted.
INPUT-COMPARISON.json uses newline-joined item hashes; its fresh combined payload
has 7,817 bytes, also exactly equal to the reference under that convention.
The earlier fresh-preview convention concatenates four content items without
three separators and therefore reports 7,814 bytes. There is no text discrepancy.

This establishes the distinct setup cause: disabling individual plugin entries
did not disable plugin support, while features.plugins=false produces the needed
state transition. No recorded developer text is manually injected or silently
dropped. The flags live only in private benchmark configuration views; stored
authentication, original provider/native binary and normal/global configuration
remain unchanged. The closed diagnostic thread will not be resumed.

This is input qualification, not a causal effect or full-workflow verification.
P09 still requires tested full-stage routing, exact A/B source preservation and
prospective admission of its three modified workflows. Prior failed diagnostics
remain retained and are not replaced by this result.
