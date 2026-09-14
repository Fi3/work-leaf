# BUG002 — accept valid progress advances, 2026-09-14

The actual-publication regression requires the live header's validated counts, not the
initial 74/16 pair forever. Its existing frozen-contract, append-only publication, visible
row and evidence checks remain intact. A valid P01-completed fixture (75/15) failed first
in BUG002-RED.txt and passes in BUG002-GREEN.txt after the one stale assertion is corrected.
The user's standing automatic bug-fix authority covers this defective-test correction.
No original 26 or old-extension 17 tests are changed. No agent-facing workflow or provider
observation is affected; this is a repository progress-test defect, not a scientific task.
