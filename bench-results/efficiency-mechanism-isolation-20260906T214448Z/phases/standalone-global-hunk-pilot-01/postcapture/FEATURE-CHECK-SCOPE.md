# Single completed-pilot feature-check scope

2026-09-13 00:29 UTC. One provider-free scoring execution covers only
standalone-global-hunk-pilot-001 after successful terminal closure. It uses the
unchanged batch_analysis.py score entry point, frozen BATCH-SCORER.json and its
three canonical fixtures. The report says PASS and its saved pass/commits.bundle
exists; scorer/fixture hashes match. No absent-bundle fallback is allowed.

Source and fixtures are unchanged. The scorer materializes the actual saved
commit in a fresh temporary checkout and runs its deterministic fixtures, not
the configured provider. Per-command timeout600seconds; outer timeout1900seconds
with20-second termination escalation. One attempt only; all results/errors are
retained, without filtering token observations or editing a frozen verifier.

The existing visible-close wording limitation remains explicit if encountered;
unreached assertions cannot be credited as passing. These checks are separate
from the actual real-agent workflow and native token audit.

Command:

```sh
timeout --kill-after=20s 1900s python3 -B bench-results/efficiency-measurement-gate-20260906/batch_analysis.py score --manifest /home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/standalone-global-hunk-pilot-01/score-manifest.json --scorer-config bench-results/efficiency-measurement-gate-20260906/BATCH-SCORER.json --output /home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/standalone-global-hunk-pilot-01/postcapture/quality.json --timeout-seconds 600
```
