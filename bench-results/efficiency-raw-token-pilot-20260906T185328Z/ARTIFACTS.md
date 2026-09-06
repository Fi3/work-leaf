# Artifact retention

Git retains the study protocol and analysis helpers, top-level manifests and result summaries,
the two workflow reports and strict observer analyses, admission/exit records, driver logs, and
the six final-artifact quality logs. Selected small report files inside ignored generated
directories remain tracked explicitly. The measurement helpers and their tests live in
`../efficiency-measurement-gate-20260906`; their exact prelaunch identities are recorded in
`FIRST-BATCH-MANIFEST.json`.

The local study directory also retains raw provider streams, invocation captures, candidate
binaries and source bundles, the clean source clone, frozen executable/helper copies, and unused
prelaunch preparation records. These generated artifacts are excluded from Git; they are not
deleted or replaced by committing the reports. The measurement gate's raw smoke capture likewise
remains local and ignored.

The committed analysis and supplemental evidence contain source hashes and exact artifact paths.
Replaying their full capture-level checks requires the corresponding retained local evidence;
a fresh Git checkout alone does not contain that evidence. The frozen files must not be regenerated
under the same names to fill missing inputs. Missing raw evidence is an explicit replay limitation,
not permission to infer exact cancelled-response usage or replace an admitted observation.

The existing user configuration and authentication state are outside the study commits. The global
configuration identity discrepancy remains documented in `POST-RUN-INTEGRITY-AUDIT.md`. Collection
is paused; artifact retention and committing results do not authorize another provider run.
