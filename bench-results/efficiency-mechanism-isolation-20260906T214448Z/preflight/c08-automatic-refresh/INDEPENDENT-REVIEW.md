# C08 automatic-refresh: independent source review

Status: **source review complete at the cut below; no runtime defect found. Not ready for admission.** Two automatic-coverage gaps, final source gates and real-agent qualification remain open. No Cargo/build/test, private executor, provider, extraction or accounting call was made during the active C15 hold.

The full modified orchestrator/activation diff, new helper, all three new test files, implementation note, amended design and affected architecture paragraph were read. The diagnostic draft received only the narrow ownership/completion check requested by root; it is not a complete harness review. Implementation and existing tests were not edited.

## Findings / required closeout

1. **P2 — v6 identity is not in the public legacy matrix.** `tests/bench_automatic_refresh.rs::legacy_refresh_bytes_remain_baseline` covers v1–v5 only. A valid v6 descriptor and its bootstrap/role conditions require a distinct fixture; a plain v6 schema string is not coverage. The source filter appears correct, but the requested default/v1–v6 identity gate remains incomplete. Add narrow coverage after the hold; do not represent the earlier matrix pass as covering v6.
2. **P2 — evidence-write failure is not exercised through actual recovery.** `src/bench_automatic_refresh_tests.rs::source_evidence_failure_blocks_selected_and_identity_results` calls `Capture::forward_with` directly. The public recovery fixture exercises failed **provider send**, not failed evidence publication after tracker advancement. The source orders `record_snapshots → finish/evidence → send` correctly in both rejection branches. The actual-path regression must demonstrate that a failed write suppresses backend delivery while a subsequent same-author read observes the advanced snapshot, without an extra read/diff or rollback. This is missing invariant coverage, not a discovered ordering bug.
3. **Readiness gate — actual-agent C08 recovery remains pending.** Synthetic/public fixtures do not establish configured-provider delivery or usable real recovery. Final metadata/test/doc edits also await fmt/Clippy/full tests. The implementation note correctly keeps those facts separate from earlier7 private and2 public passes.

These findings were sent to root and the implementer before this note. No test execution or source modification is requested during the hold.

## Source checks

- V7 activation has one condition and follows the existing private manifest/environment/run-ID loader. Default builds exclude the helper. Prior schemas select no capture; their continuation selection is otherwise unchanged. No public API, provider, launch-policy or requested-read change is introduced.
- Both actual Patch/Edit conflict arms use the same ordinary `patch_conflict_refresh_response` read and normal selected snapshots. Already-applied, no-file and nonstale branches remain independent. Rebuilt baseline wrapper/newline strings match the preceding source by inspection; current-version baseline comparisons alone are not an independently frozen predecessor executable test.
- The original changed-file diff callback remains single. Its normal48-KiB inclusive boundary, omitted, unavailable and empty outcomes remain distinct. Only a nonempty within-bound diff gets a current-text replacement. Full current text has no8-KiB limit; unchanged and untracked sections retain ordinary status/text/cap behavior.
- Wrapper and intro replacements are source-owned and take effect only with an eligible body. Body ranges preserve held UTF-8 text, exclude any formatter newline and do not search marker-like source. Mixed-section tests preserve diagnostics/failures and unchanged segments. Current/prior digest and byte metadata remain ordinary FNV64, not cryptographic identity.
- Automatic snapshot advancement precedes `RefreshPrompt::finish`, including evidence publication, then the original send. It is not moved to requested-read's after-send ordering. Failed provider send retains that advanced state; evidence failure source ordering is correct but still needs finding2's integration coverage.
- `Capture::candidate` makes a constant number of walks over ordered replacements and held bytes. Rendering traverses snapshots once; metadata comparisons/clones and candidate serialization add bounded linear byte work. No new O(n²) history/file/span scan, extra diff process, snapshot read, bundle allocation or provider turn was found. Existing Git-diff and filesystem costs are not relabeled constant.
- The architecture paragraph describes current ownership, original limit branches, failure ordering, linear construction and pending actual-agent requirements. No documented public-interface change or broader architecture expansion is present.

## Narrow diagnostic draft check

The amended design permits one fixed **controlled** older-snapshot fixture using an actually accepted host edit; it does not permit fake rejection/output/checks or a natural-exposure claim.

At `diagnostic/harness.rs::BoundedBackend::before`, the host stimulus currently calls `GitPatcher::apply_edit` with the real author's AgentId. Although prose labels it harness-owned, that ID enters actual commit metadata. A distinct declared fixture-mutator ID, retained separately from the sole real author, is safer and avoids accidental attribution without adding a provider role. Root and implementer were notified.

The draft `processed_done` suffix predicate needs its promised actual CommandChat-path automatic test before admission. The missing guard module and source-first negative fixtures are intentionally unexecuted; they are not a passing qualification. This note does not clear native/public joins, watchdog/call limits, fixture semantics, observer configuration or real exposure.

## Reviewed cut

Detached worktree: `/tmp/work-leaf-c08.04mP2m/repo`, base `b8e928153674549e4877b5ae6b34546d8541d566`. All seven implementation identities matched the stable handoff on source recheck:

| File | SHA-256 |
| --- | --- |
| `src/bench_experiment.rs` | `cb7e1326d1f598a81df6f1544804c6b6285df185aee3021b15a01b6cdf7a3b28` |
| `src/orchestrator.rs` | `e19e477a4d9eccbdf479c75589da35dda623a6236c7fdd6194748342841015fe` |
| `src/bench_automatic_refresh.rs` | `dc09943eb17ed9d02bbed4d938908fe5dbbd4fe74aaa8ee8204844f9ae406ec2` |
| `src/bench_automatic_refresh_tests.rs` | `ecfd4b2ffa79b9ad58c44020972b2b7664428278a014aaa3a3a3a3aad670a717` |
| `src/orchestrator_automatic_refresh_tests.rs` | `40c1c9aeacd3c0ac1052352cfa60b402e7a124f1750d97b91dc40ad17b34a439` |
| `tests/bench_automatic_refresh.rs` | `78a76c7da1441737319e24b76df5e53ffdaf95838f51a3576fe80b733decc0f1` |
| `docs/architecture.md` | `ee443aeead88d6b70125eccdc3c3a7ce9268f3e33dd59876eaa242a692862670` |

Amended design SHA-256: `7649ef1359126714e511a9b2b05b2641622a27620f230ee8613f6bc1a778bf20`. Earlier results and the unchanged root C15 source/launch are not modified or upgraded by this review. Only this new detached-worktree note is written.
