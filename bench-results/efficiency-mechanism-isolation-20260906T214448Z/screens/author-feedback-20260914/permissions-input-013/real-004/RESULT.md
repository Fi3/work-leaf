# Exact project-trust attempt: input qualification failed

Completed2026-09-15 03:09:09.158355 UTC in14.329747s,47,776 raw tokens,
three distinct responses/two completed turns. No native/public/prompt accounting
error, missing tail, source-pin mutation or repository change. The first developer
input matches reference6278885a5704b96b3d52083d8e755d942c07bdde5b57b5d20a919ba0d4f81e9e;
resume still inserts the362-byte permissions message681b5ba580483b28751dbafd95e30f80ebb6ab9cc2d3ca4462f0f187d0784070.
No repaired integration input or token effect is established. Four failed small
probes total190,753 raw. Old outcomes and once-only native caches stay unchanged.

The subsequent zero-generation config/read check finds the attempted trust setting
was NOT installed at the intended project key. The pinned CLI splits the dotted
key despite quotes: it creates a project key beginning with a literal quote and
truncating at the period inside the temporary path. Its nested remainder contains
the trust value, while the exact target project's config remains absent. The pure
quoting test missed this CLI-specific parse behavior. This is failed setup
activation, not evidence that a correctly applied trust setting has no effect.
CONFIG-READ-SHAPE-20260915.json preserves the actual source-layer result and exact
RPC trace: initialize, initialized, config/read, zero thread/turn starts.

Any next setup qualification must first read back the effective complete config
and verify the exact trust key plus preservation of existing project entries.
Use no extra model call to test configuration parsing. A source-qualified actual
repair would still need its own bounded real admission and whole-input gate.
This remains the existing BUG009 repair, not another scientific task.
