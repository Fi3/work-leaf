# BUG009: unqualified integration-resume permission instruction

Detected by the whole-developer check at2026-09-15 01:19:42 UTC; verified against
native source by01:23 UTC. P09's run005 and006 integration-accept turns inject
an extra permissions instruction with SHA256
681b5ba580483b28751dbafd95e30f80ebb6ab9cc2d3ca4462f0f187d0784070.
The saved reference integration resume has no such developer message. The
recorded sandbox is danger-full-access and permission profile disabled in both;
effective permission equality alone does not establish equal model input.

Evidence: the qualified phase's LIVE-INPUT-BOUNDARIES-0119.json and
LIVE-DEVELOPER-CHECK-0119.json. The latter correctly reports passed=false;
catalog-only/legacy sandbox fields pass and are insufficient for this boundary.
The earlier real route diagnostic exercised read-only direct resume, not the
actual integration permission profile. All preceding passed checkpoints retain
their stated observed prefix, not a guarantee about later integration.

Generation receives Ctrl-C through its owned supervisor session at01:23 UTC.
PHASE-RESULT records all three terminal by01:23:16:004/005 exit1,006 exit0.
These statuses require report/accounting inspection; no clean causal full mean
is claimed. Every artifact, source endpoint, partial and completed response is
retained. No replacement workflow or integration continuation is admitted.

Existing P09 repair/qualification, with separate visible bug counter. Initial
local tracing/reproduction bound:20 minutes from01:24 UTC. No new provider call
before a specific locally verified repair and prospective bounded diagnostic.
Use the existing subscription only; no credential copies or normal-WL changes.
This is an experimental-input defect, not an explanation of historical savings.
