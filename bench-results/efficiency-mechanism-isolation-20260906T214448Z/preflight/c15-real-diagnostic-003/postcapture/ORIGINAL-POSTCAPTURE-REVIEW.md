# Diagnostic 003 original postcapture

The unchanged original analyzer and prerequisite-gated rollout extractor each ran
once after root's terminal confirmation. Every original flag and complete output
is retained. No reanalysis, accounting, provider, private executor or retry ran.

[POSTCAPTURE-SCOPE.json](POSTCAPTURE-SCOPE.json), SHA
`e2b4a2b7eb4d3c1ced7399f0ae8503fb331750af4230f65899b2c14e8a94a4f4`,
was published before either command. Both used the admitted observer
`238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`,
an external 180-second bound plus five-second kill grace, and separate exclusive
attempt/stdout/stderr publications.

| Command | Actual UTC interval, 2026-09-07 | Exit | Original outcome |
| --- | --- | --- | --- |
| analyze | 22:09:16.739193–22:09:16.918110 | 2 | capture_complete false |
| extract-rollouts | 22:10:15.818857–22:10:15.913924 | 0 | two observed/matched threads; errors empty |

The analyzer published the summary required by extraction. Its stdout/summary
SHA remains `135c616cba117dc8ae1d5575fee36b35043429b04aa3b5334d5fd2e360bf21bc`
after extraction. The extractor stdout/audit SHA is
`73bc892aa052545fab211cee00e143f13ec35aec3ef6d6bfbfdbddc9bd4520e2`.
Both stderr streams are complete and empty.
Exact argv, timestamps, exit codes and source/output identities are in the
two execution receipts, SHA
`af9cb73e8a5919bd12cac56bc1fee0e51aee80715194032f67fea67cf424de88`
and `3910e95673bf3350867246f5724f42298004438c2a557306593feddfa61fdfe0`.

## Flags remain separate from workflow qualification

The distinct original flag is
`interrupted provider turn has no complete usage: count=1`.
The frozen `bench-observer::analyze` retains this error under required complete
usage (`bench-observer/src/lib.rs:4450`); its CLI returns exit 2 for incomplete
capture (`src/main.rs:105`). It is not a provider-crash conclusion, measured zero,
or complete-accounting proof, and is not explained by the proxy-marker finding.

The other eight flags are `bearer-token`, `OPENAI_API_KEY`,
`ANTHROPIC_API_KEY` and `refresh_token` in each of
`proxy-bin/codex` and `proxy-bin/sh`. Both complete ELF files equal the admitted
observer bytes. The unchanged regular-file scan includes these binary copies
(`lib.rs::collect_artifact_files_for_secret_scan`, line 6889).
This exact-file explanation does not clear `capture_complete: false`, waive
another path or suppress the independent usage flag.

The summary records two complete invocations and no controller reconciliation
rows. Extraction records no missing/session-only/unobserved-cwd thread or error.
These facts do not substitute for the separate accepted-input/native/private-test,
ordinary-workflow and semantic reviews. The diagnostic terminal's exit 0 remains
separate from original measurement completeness.

## Source retention

All 215 admission pins and 31 preexisting observer files matched before scope
publication. The union with retained terminal/process receipts matched at both
command boundaries: 248 endpoints for analysis, 249 including its published
summary for extraction. The final read-only check passed and confirmed that
extraction did not alter the original summary. Mutable initial project files are
not immutable postlaunch endpoints; later independently published reviews are
not represented as original admission pins.

[ORIGINAL-POSTCAPTURE-REVIEW.json](ORIGINAL-POSTCAPTURE-REVIEW.json), SHA
`9f25d5440d75234aa3aab1a5722a29ba0da6572479c2ed803671141dd7dfd9dc`,
retains all original flags, execution metadata and the six derived-report
identities without token totals or private reasoning. Publications are exclusive,
not a transactional multi-file guarantee. No original report was repaired or
replaced.
