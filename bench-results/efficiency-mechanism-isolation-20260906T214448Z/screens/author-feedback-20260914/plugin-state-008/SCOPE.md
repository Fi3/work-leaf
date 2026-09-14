# P09 recorded plugin-state transition qualification

Prospective scope: 2026-09-14T23:25:00+00:00; ten local minutes and one
three-turn, sixty-second, 100,000-recorded-raw subscription input diagnostic.
No causal screen or control is admitted. Existing scheduler/driver/monitor and
stored ChatGPT login remain unchanged; no normal/global configuration changes.

Concrete distinction from the two parked approaches: the reference native
world_state at line371 explicitly sets plugins_instructions=false; line385 sets
it true. Input006 disabled two individual plugins but its native world_state
never became false. Input007's developer_instructions setting was ignored on
resume. Neither tested the complete plugins feature boundary. A local preview
with features.plugins=false removes exactly the 1,014-byte plugin instruction
while retaining the short skill catalog. The installed pinned CLI lists that
feature as stable. [Official setting](https://learn.chatgpt.com/docs/config-file/config-reference)
and [managed configuration](https://learn.chatgpt.com/docs/enterprise/managed-configuration)
describe the plugins flag; actual delivery, not documentation, is the gate.

Use private profiles full -> short/no-plugins -> short/with-plugins. Expected
actual developer payloads are original full launch, the short-catalog-only
update (5,038 bytes), and exactly the plugin-usage-only update (1,014 bytes).
The latter two reproduce the reference third-author turns5/6 transition after
turn4's full catalog. Other base/model/effort/permissions/tools/task and real
feedback remain unchanged. A diagnostic marker is not a feature result.

First qualify exact parsed-config diff and private-file preview. Then one fresh
native thread plus two resumes; no old audited thread is resumed. Reject extra
or missing developer input, source drift, ownership failures or budget breach.
Save all outcomes and once-only accounting. Failure parks this concrete approach;
no automatic repeat or full-batch launch. Success qualifies only this input seam,
not full-stage routing or the A+B workflow contrast.
