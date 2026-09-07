# Diagnostic 002 original postcapture

The original analyzer and prerequisite-gated rollout extractor each ran once.
The analyzer's exit 2 and all nine flags remain retained; the extractor's exit 0
does not clear them. No reanalysis, provider, private executor, accounting or retry
was performed.

Scope [POSTCAPTURE-SCOPE.json](POSTCAPTURE-SCOPE.json) SHA
`66d30797e5f02ad7d4c84ebf40f4b9d935202c4cfdbbec9ddf87eab83546aab9`
was published before either command.
The unchanged observer SHA is
`238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`.
Both commands used the declared external 180-second timeout plus five-second
final kill grace and exclusive separate attempts/stdout/stderr.

| Original command | Actual UTC interval | Exit | Result |
| --- | --- | --- | --- |
| analyze | 20:14:23.326324–20:14:23.496907 | 2 | capture_complete false |
| extract-rollouts | 20:15:26.496190–20:15:26.587679 | 0 | two observed/matched threads; errors empty |

Analyze published the summary required by the extractor. Its output SHA remains
`192fc73372e9923cbf7a2247f1fabe7a2e9fb6676720cc0ac184fd411798b871`
after extraction. The rollout-audit/stdout SHA is
`73bc892aa052545fab211cee00e143f13ec35aec3ef6d6bfbfdbddc9bd4520e2`.
Complete stderr streams are retained and empty. Exact argv, actual status,
timestamps and output hashes are in the two `*-EXECUTION.json` receipts;
their SHAs are `77ad220d43d87881f33d567e097ae1cf216a97add8f648313b2127e0f19c907e`
and `027a6428b7bb15751e1472893da598a8fbe1e8b62d3875321b5cb6c9e6283a66`.

## Preserved original flags

The distinct usage flag is:
`interrupted provider turn has no complete usage: count=1`.
The frozen analyzer requires complete provider usage and retains that flag when
its interrupted-turn counter is nonzero
(`bench-observer/src/lib.rs::analyze`, line 4450).
This review does not classify it as a provider crash, zero usage or a complete
accounting result. It is not explained by the binary-marker finding.

The other eight flags are `bearer-token`, `OPENAI_API_KEY`,
`ANTHROPIC_API_KEY` and `refresh_token` in each of
`proxy-bin/codex` and `proxy-bin/sh`.
Both entire files are ELF binaries with the exact admitted observer SHA above.
The unchanged regular-file marker scan includes those copied binaries
(`lib.rs::collect_artifact_files_for_secret_scan`, line 6889).
This narrowly establishes the flagged file provenance; it does not clear
`capture_complete: false`, waive another file or suppress the separate usage flag.
The original CLI returns exit 2 for incomplete capture
(`bench-observer/src/main.rs:105`).

The summary records one complete invocation and no controller reconciliation
rows. No performed controller reconciliation is inferred from an absent file.
Extraction retains empty missing/session-only/unobserved-cwd lists and errors,
but its two matched threads do not certify complete native-input, private-test,
semantic or quality coverage. Those finite witnesses belong to the separate
root/independent reviews.

## Source boundaries and retention

All 188 admitted immutable inputs and 21 preexisting observer files were verified
before declaration. Each command also bound the original admission/terminal and
process receipts; all 211 union endpoints matched around analysis, and all 212
including its summary matched around extraction. The final read-only recheck
passed with the original summary unchanged. Mutable project files are not
postlaunch immutable endpoints.

[ORIGINAL-POSTCAPTURE-REVIEW.json](ORIGINAL-POSTCAPTURE-REVIEW.json), SHA
`4544b9e293ba2f42e1b23dd01d4dc545784b1cd07055b6a5ed747f68a537aac8`,
retains every original flag, execution receipt and all six derived-report
identities. It contains no token totals or private reasoning.
The supervised diagnostic terminal exit 0 is separate from analyzer completeness
and all remaining real-workflow/source/semantic qualifications. Publications are
exclusive, not a transactional multi-file guarantee.
