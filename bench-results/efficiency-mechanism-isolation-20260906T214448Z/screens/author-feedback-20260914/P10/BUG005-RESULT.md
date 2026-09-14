# BUG005 — turn-bound catalog qualification

Completed: 2026-09-14T21:07:45+00:00.

The source-linked closed-input comparison requires the catalog effective at each
selected author outer turn, with inheritance when no developer replacement occurs.
Profile membership or profile-order equality across the whole native thread is
insufficient: that thread also contains later review repairs.

## Reproduction and verification

The extracted former pooled-profile algorithm fails seven of eight boundary
regressions in BUG005-RED.txt. The corrected catalog_boundary.py passes all eight
in BUG005-GREEN.txt. CATALOG-BOUNDARY-QUALIFICATION.json applies it to the retained
actual native records without another usage audit or model call.

Reference author turns 1–4 retain the 5,813-byte catalog; turn 5 receives the
5,038-byte version. P03 drops to 5,038 at turn 2 and keeps it. P04 drops at turn 2,
restores 5,813 at turn 3 and drops again at turn 5. Both startup objects are exact,
but the subsequent causal comparisons are not fully input-qualified.

FINAL-INPUT-SOURCES.json, the live resume receipts and P02/BUG003-REAL-RESUME.json
remain preserved. Their profile-equality fields establish observed-profile
membership, not corresponding-turn equality. The matching-resume command parser
is fixed; the earlier assertion of full historical resume fidelity is unsupported.

## Limits

This is a reporting/qualification fix, not a fix to the admitted native runs and
not proof that catalog drift caused their extra work or the historical token gap.
No original record, accounting result, admitted source or frozen budget changes.
Both affected screens remain blocked for clean attribution, with all charges and
stopped tails retained. The extra sixth turn has no matching reference-author turn.
An ordinal check detects this mismatch; it does not establish semantic equivalence
between model actions in otherwise matching turns.

No real-agent workflow is modified: this utility reads already-recorded events.
Verification replays the exact actual native records; no additional generation is
appropriate for an analysis-only correction. The eight deterministic regressions
cover inherited state, early transitions, later-stage leakage and malformed mappings.
