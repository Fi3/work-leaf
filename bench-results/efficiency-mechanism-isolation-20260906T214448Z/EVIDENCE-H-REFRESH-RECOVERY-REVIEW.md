# H refresh-recovery: independent finite review

Recorded 2026-09-08T09:23:01.558636+00:00. **PASS: no introduced factual finding in this finite scope.**

Reviewed completely:

- [Recovery report](EVIDENCE-H-REFRESH-RECOVERY.md), SHA-256 `6dbc262c74ed3428fad8e62862ab2fc3d0f947ab1a26796e5dd2da931b097afa`.
- [Fourteen-input / nine-chain index](EVIDENCE-H-REFRESH-RECOVERY.json), SHA-256 `6a83a13322b10afe25cb6cf3ba24248859133ba18f189c51b61f8c7d9537568f`.
- [H source qualification](EVIDENCE-H-LIFECYCLE.md), SHA-256 `556da269740ff8a33edd25d351ea913ec1a4f77bd2524f099eb26bd8753c55b2`.
- The qualified [normal-WL renderer source](../efficiency-raw-token-pilot-20260906T185328Z/source/src/orchestrator.rs), SHA-256 `f88aeea1e910504eeb4434291e662ff2455e64edb5b2d45f87f5c316c6aafe24`, particularly `patch_conflict_refresh_response` at1226 and `render_file_refresh_response` / its limits at2210–2297.

## Exact identity and completeness

All14 full original refresh inputs were read, including their complete visible diffs. All12 pinned raw-source endpoints match before/after review. Fifty-seven distinct referenced inputs—the chain inputs plus prior snapshot witnesses and intervening ACKs—match their exact full text SHA/byte count, typed successful RPC reply, accepted thread/turn, and unique complete public user item. No input-text substring substitutes for accepted identity.

For each of the nine chains, every same-author turn/start from its first refresh through its first subsequent patch-applied ACK equals the indexed input list. There is no missing intermediate same-author input and no earlier successful ACK in that interval. All44 indexed complete agent messages match exact physical line, item ID, full text hash/bytes and the actual directive lines; this includes the response to the endpoint ACK, not a claim that a later command result was captured inside the interval. Shared suffixes are counted once by eventual ACK.

The five actual requested-read deliveries are003/C49,004/C54,C56,005/C56 and006/C77: five handoffs in four distinct chains. Same-snapshot/malformed-edit inputs002/C40,005/C81 and006/C41,C75 remain inside the windows without becoming additional changed refreshes.

## Prior file/digest ownership

The prior-digest check is stronger than a path-plus-digest substring search. Thirteen witnesses are exact entries in the host-rendered bundled-file list. The remaining witness,005/C48, is the exact `src/terminal_app.rs` block's **current** digest at earlier refresh005/C37. Every entry includes the recorded byte length. Each actual refresh block binds that same prior digest and path to its separately recorded current digest; no match comes from another file, arbitrary source text or a quoted prior header.

C/S locators use the exact client/server sources and typed IDs in the reviewed index. The final column lists actual accepted same-file groups strictly after the prior snapshot input and before the refresh. Their file lists match exactly; their threads belong to different owned author launches, with explicit Agent-ID/Feature/User-prompt bindings rather than a guess from an arbitrary ID string.

| Refresh | Exact file | Earlier owned snapshot witness | Intervening other-author ACKs |
| --- | --- | --- | --- |
| 001/C65 | `src/ui_harness.rs` | C10: exact bundled-file entry; `fnv64:770bd2bb00b376ed` | C46 |
| 001/C78 | `src/terminal_app.rs` | C20: exact bundled-file entry; `fnv64:25dc74bb7b8daeba` | C30, C46, C67 |
| 002/C36 | `src/ui_harness.rs` | C14: exact bundled-file entry; `fnv64:770bd2bb00b376ed` | C22 |
| 002/C62 | `tests/ui_harness.rs` | C10: exact bundled-file entry; `fnv64:01ef102fdde22996` | C22, C42, C46, C50 |
| 003/C43 | `src/terminal_app.rs` | C18: exact bundled-file entry; `fnv64:25dc74bb7b8daeba` | C28 |
| 003/C47 | `src/ui_harness.rs` | C14: exact bundled-file entry; `fnv64:770bd2bb00b376ed` | C28 |
| 004/C52 | `src/terminal_app.rs` | C20: exact bundled-file entry; `fnv64:25dc74bb7b8daeba` | C24, C42 |
| 005/C37 | `src/terminal_app.rs` | C22: exact bundled-file entry; `fnv64:25dc74bb7b8daeba` | C26 |
| 005/C39 | `src/workspace.rs` | C14: exact bundled-file entry; `fnv64:001c1f08f5fa26bd` | C26 |
| 005/C48 | `src/terminal_app.rs` | C37: exact rendered file block; `fnv64:6997dff12c8fca1b` | C44 |
| 005/C67 | `src/ui.rs` | C10: exact bundled-file entry; `fnv64:9ee9f02c5bc43f7d` | C63 |
| 006/C39 | `src/terminal_app.rs` | C20: exact bundled-file entry; `fnv64:25dc74bb7b8daeba` | C28 |
| 006/C60 | `src/ui.rs` | C10: exact bundled-file entry; `fnv64:9ee9f02c5bc43f7d` | C50 |
| 006/C70 | `src/ui_harness.rs` | C10: exact bundled-file entry; `fnv64:770bd2bb00b376ed` | C28, C44, C50 |

## Semantics and limits

The14 complete inputs contain11 “old block was not found” diagnostics and three duplicate-file-header diagnostics:005/C37 and006/C39,C60. The changed snapshots and other-author accepted activity are real, but they do not prove that concurrency caused each rejection. In006/C60 the visible `hide_agent` addition coexists with an explicit duplicate-header parser error; the author's subsequent public explanation preserves that distinction.001/C65 andC78 actually deliver the completion/harness and terminal/slash changes described in the report, and the public responses explicitly discuss rebasing around them. These are visible source/author statements, not hidden-reasoning evidence.

Every target is a tracked changed-diff response. None contains a diff-omission, unavailable-diff or untracked full-text-omission branch. The qualified renderer applies the48-KiB limit to an automatic diff and the8-KiB limit to untracked full text; neither omission branch is exercised here. “Complete diff delivery” denotes the intact renderer branch and captured payload, not a new reconstruction/execution of every prior/current source tree.

The next ACK proves accepted repair, not preserved test semantics, a passing check, all-native-tool absence or avoided generated work. The report's five chains without another mediated read do not imply that the model never retrieved an issued bundle natively. No token total, response-cost subtraction, causal saving or quality judgment follows from these joins.

The remaining full-versus-delta automatic-refresh representation boundary is correctly separate from requested-repeat C02. Any such factor must preserve rejection, snapshot advancement, locks/permissions and truthful guidance; removing those mechanisms or deliberately causing conflicts is a different intervention. This review admits no experiment.

Only this new review note is written. No reviewed artifact, source, runtime, helper, protocol, configuration or historical outcome is changed. No provider, benchmark, test execution, extraction or accounting call is made; no private reasoning body is inspected or exported. No reusable collector or executable patch is introduced, so there is no new algorithmic-complexity or agent-facing readiness claim.
