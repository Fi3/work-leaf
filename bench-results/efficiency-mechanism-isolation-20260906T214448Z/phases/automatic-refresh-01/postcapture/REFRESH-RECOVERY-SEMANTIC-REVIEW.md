# C08 finite refresh-to-repair evidence

All 11 eligible refreshes have exact delivered-input evidence and lead to eight distinct accepted repair ACKs. Eight reported old-block failures depend on changed held snapshots; two old blocks were already absent from the prior snapshot; one rejection is a duplicate file header. The eight first post-ACK checks are captured executions: six return 0 and two return 101.

This is a source/action result, not a saving estimate. The original failed workflow 001, successful workflows 002/003, and every original observer/extractor/accounting qualification remain intact. In particular, the two ordinary trace ambiguities per run remain in the corrected source result; their independent singleton ownership witnesses are in [TRACE-OWNERSHIP-REVIEW.md](TRACE-OWNERSHIP-REVIEW.md).

## Fixed population and source chain

The [metadata index](REFRESH-RECOVERY-SEMANTIC-REVIEW.json) covers exactly trace lines 4/9 (001), 4/13/19/23/28/31 (002), and 4/9/14 (003). It records 188 physical source hashes, complete typed input locators, separate public/native generated item IDs, held file-body offsets/hashes, and exact captured command/output sources. All 25 distinct public edit submissions in the eight bounded windows have one full-text public-completion/native-message match in the same explicit turn.

The frozen renderer supplies the actual current body; the independently checked current/prior body digests match the selected path and byte counts. Eight prior states are backed by an issued bundle with its archived snapshot. The remaining three are the author's exact prior requested-repeat path/digest block (002 T19, native N256) or earlier full refresh (002 T23 from N280; T31 from N868). A bundle pointer does not by itself prove a model opened its body.

Frozen `infrastructure/evidence/src/patch.rs` prepares file changes before publication, applies exact structured old blocks in hunk order, and rejects duplicate file headers. The finite comparison uses each actual rejected target-file hunk through the reported failing hunk, applying preceding unique hunks in memory. It does not run GitPatcher, tests, native tools, or the source/accounting helpers. Prior-snapshot match versus current-snapshot nonmatch establishes the narrow failure dependency, not whether full text was necessary.

## Dispositions

Native line numbers below refer to the source path and exact item IDs in each index row. Multiple refreshes sharing an ACK are not counted as separate repair successes.

| Run / trace | Rejection and held-state evidence | Accepted repair / first captured check |
| --- | --- | --- |
| 001 T4 | UiHarness hunk 5 matches prior text, not current slash-routing text. | N194 → ACK N202; `cargo test completion --all-targets --all-features`, exit 101. |
| 001 T9 | UI hunk 2 omits the pre-existing `StatusNotice` declaration between `PendingKey` and constants. It matches neither snapshot; the concurrent change is elsewhere. | N210 → ACK N218; `cargo test --test ui_harness`, exit 0. |
| 002 T4 | UiHarness hunk 1 matches prior routing, not the other author's current routing. | N215 → ACK N223; `cargo test feature_done`, exit 0. |
| 002 T13 / T19 / T23 | T13 has duplicate `src/ui.rs` file headers. Later terminal-app hunk 1 and hunk 4 respectively match their prior states but not the subsequent import/visibility changes. | Shared N317 → ACK N324; `cargo test --test ui_harness --test terminal_app visual`, exit 101. |
| 002 T28 / T31 | Terminal-app hunks 3 and 6 respectively match prior states but not later visual-selection routing/cursor changes. | Shared N986 → ACK N994; `cargo test feature_done`, exit 0. |
| 003 T4 | Terminal-app hunk 2 matches the prior import block, not the added slash-command import. | N234 → ACK N242; `cargo test feature_done`, exit 0. |
| 003 T9 | UiHarness hunk 1 matches prior imports, not the completion-related imports/state. | N214 → ACK N222; `cargo test --test ui_harness visual -- --nocapture`, exit 0. |
| 003 T14 | Terminal-app hunk 1 assumes a duplicate `let Some(agent_id)` line absent in both states. The final edit omits terminal-app production changes and edits workspace plus terminal-app tests. | N519 → ACK N535; `cargo test feature --lib --test terminal_app --test workspace --test ui_harness`, exit 0. |

Every case also has an intervening other-owner ACK whose exact file list names the changed file, with typed input locators between the selected prior delivery and refresh. These ACKs and held-body comparisons are not a reconstruction assigning every changed byte to one Git commit.

## Recovery work and limits

Across the eight initial-rejection-through-ACK windows there are 25 edit submissions: eight accepted and 17 rejected. Eleven rejections have the eligible full refresh; six additional rejections do not. Six additional mediated read inputs occur in four windows. Thus the actual full refresh did not guarantee a one-response repair or eliminate further context requests.

Seven native calls occur inside these windows: six archive range reads and one literal base64 calculation. Each of the six returned range outputs exactly equals the selected bytes of its pinned archive, including line numbering where requested. This is positive returned-content evidence, not an inference from a path mention. It does not establish absence of native activity outside these finite windows.

For each first post-ACK check, one captured locked-shell invocation matches the complete command wrapper, project cwd, exit status, stdout, and stderr delivered back to that author. All eight output pairs appear without an output-compaction marker. The two 101 outcomes are retained as failures; later repair acceptance is not a complete feature-quality or unchanged-test proof.

The useful C08 observation is therefore actual exposure to complete changed current text (21,903–54,438 bytes), followed by both simple and multi-step repairs under normal acceptance/check rules. Whether that representation saves net tokens compared with the ordinary delta remains a separate comparison with the declared accounting and quality limits. No provider observation, baseline replay, counterfactual percentage, or new accounting call is part of this review.
