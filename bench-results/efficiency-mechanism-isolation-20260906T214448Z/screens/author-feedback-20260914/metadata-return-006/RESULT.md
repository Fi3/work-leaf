# Metadata-return profile does not reproduce the recorded update

Checked: 2026-09-14T23:04:15+00:00.

The diagnostic completes in 24.24 seconds with 47,169 raw, three completed
native/public turns, all marker replies and all source/usage/prompt joins.
Fresh short-with-plugins matches the saved 7,039-byte developer input exactly.
Both subsequent turns emit no developer update. In particular, re-enabling
the two plugins does not emit the expected isolated 1,014-byte usage instruction.
The existing short/full/short qualification is not invalidated; this separate
metadata-return expectation fails.

This approach is parked without a repeat. No causal/token-saving conclusion
follows from a setup diagnostic. A full batch remains unadmitted behind this
specific known input boundary and stage routing. The reference's extra
instruction remains evidence; it is not silently dropped or called harmless.
The native thread is closed and immutable; future diagnostics cannot resume it.

The next setup alternative, if used, must restore the exact saved instruction
through an explicit benchmark-only documented input setting and verify its
actual delivery. It cannot inject arbitrary operator guidance or claim that
the natural CLI metadata transition was reproduced.
