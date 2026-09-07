# Independent ACK classification review: workflow 001

Reviewer: `/root/candidate_controls`. Scope: only terminally published
`work-units-01-workflow-001`, under `preflight/ACK-CLASSIFICATION-PROCEDURE.md`.

Disposition: no findings or classification disagreements. All five accepted ACKs
have one supported draft label, in the exact inventory order. This is review of one
workflow's draft, not the final phase classification or a condition comparison.

## Reviewed identities and integrity

- Phase manifest SHA256: `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
- Draft: `coding/drafts/work-units-01-workflow-001.json`, SHA256
  `e19e434f51357ae1ae05d6311c6fcff447ea734dddbd697613b4adc841505410`.
- Packet: `coding/packets/work-units-01-workflow-001.json`, SHA256
  `ac492a5659e12893468f2f4990df92ba45ce1427b9cdd11aca31a9d11db697a7`.
- Terminal receipt: `logs/work-units-01-workflow-001.exit.json`, SHA256
  `a9ff4ea17473140e486dfc679741f4897eba6ed511e5293c5854ebad1820b6d7`;
  completed with launcher exit code 0.

Every packet source hash and all 41 draft evidence entries across nine unique
canonical, phase-local source files were independently revalidated. Every cited
physical JSONL location and compound reference was checked against its stated role.
The frozen pure `prompt_inventory` and `capture_provenance` functions were replayed
for this workflow only: provenance verified, bidirectional inventory available with
no errors, five ACKs, exact packet order, and unchanged replay source hashes.
Neither whole-phase analysis entry point was invoked.

For each ACK, the accepted typed string RPC request/reply fixes the thread and ACK
turn. Its preceding same-thread accepted turn contains one complete structured edit
group, followed by that turn's interrupt; the group's file set equals the ACK receipt.
The review uses this source-grounded processed-reply chain, not global event timing.

The table below uses one-based physical lines from this phase-local capture:
`runs/work-units-01-workflow-001/work-units-01-workflow-001-three-feature-bench-artifacts/observation/app-server/00000441344929586777-3537090/`.
`C` means `client-to-server.raw`, SHA256
`748135038446e5397f8514c0e0a70609ff1020b1070e64d227c97d096c011d9d`;
`S` means `server-to-client.raw`, SHA256
`f050910cd73200db4ee4bfb9ee63080ca20c42bddd253d467f34f3177550e243`.
Edit text is at `/params/item/text`; request text is at `/params/input/0/text`;
accepted turn identity is at `/result/turn/id`.

## Per-ACK review

| ACK, in inventory order | Exact processed edit and delivery | Supported classification and basis |
| --- | --- | --- |
| `ack:afb6bb7626dfc40f6028232bade28f1dd5fd53fbc60b18adff8948c67dc96438` | Edit S3239, item `msg_042ad8bf05f8964f016a9e0770b3c487d2924c7d081249df82`; interrupt C27; ACK RPC string `27`, C28/S3246. | `remaining-work`: original request C6 requires selected-backend slash routing. Bundle 1's controller/command route lacks it; the submitted controller method and regression implement that requested route, without a preceding repair trigger. |
| `ack:f22c2155d5d61e22efe2605034da6e0375f6af38975f7ac6814477378686f53a` | Edit S10798, item `msg_037b7c0faffa07c0016a9e085f845887d2bd3df03a663e5678`; interrupt C41; ACK RPC string `41`, C42/S10808. | `remaining-work`: original request C8 requires reviewed-feature highlighting, yes/no completion, close/reopen behavior. Bundle 2 lacks the completion prompt/state. The group's controller, terminal, UI, harness and tests implement that coherent feature; the later routing defect does not relabel the initial work. |
| `ack:9ecec00c6655f15438c74c7ccc7168702ab408454b10ca427aedd65d546c7e54` | Edit S19202, item `msg_037b7c0faffa07c0016a9e09fdda4c87d2bdc1a79205d43d97`; interrupt C54; ACK RPC string `54`, C55/S19209. | `review-repair`: delivered finding C53 identifies the completion prompt in the patch chat while the reviewer remains selected, so yes targets the wrong session. Bundle 10 corroborates this existing implementation and the manual-click test. The edit selects the patch agent and removes the test's masking click; it repairs that behavior rather than implementing a separate requested component. |
| `ack:05365cf8d2d7d91184cccd1750447af14daa0d41de1f2a11260e71f86b4b7e28` | Edit S39197, item `msg_0c11383d22d629fd016a9e0b67f63c87d29093dcbe5552d8ea`; interrupt C64; ACK RPC string `64`, C65/S39204. | `remaining-work`: original request C4 requires focused-pane visual modes/yanks. Bundle 0 lacks visual state/key handling. The complete group supplies selection, rendering, clipboard integration, harness support and tests. Earlier hunk-mismatch rejections C52/C61 are not accepted prior visual work or evidence of a no-op. |
| `ack:f1a2596e8167925e44db747276a67c4acc8272f53f100f2d510e7b73663189d5` | Edit S41679, item `msg_0c11383d22d629fd016a9e0d8fcf3487d28ab045330972e185`; interrupt C77; ACK RPC string `77`, C78/S41686. | `review-repair`: delivered final findings C74 concern existing visual selection's left-pane detail-row indexing and right-pane wrapped cursor position. Bundle 12 contains both defective implementations. The edit repairs those two mappings with focused regressions; no independent new mode/yank feature is mixed in. |

Full ACK thread/turn identities, trace locations, request hashes and file receipts
remain in the reviewed draft/packet; the table does not replace that exact inventory.
The prompt trace hash is
`17a2273f64dabdee7d2ac88f2ad4dae13d30bd576fa6f7135d3c1ad2cf4c1aa7`.

## Prior-state and continuation checks

Prior-state checks include bundle 1 lines 3661–3680 and 3800–3836; bundle 2 lines
1254–1278 and 3703–3711; bundle 10 lines 4273–4281 and 5813–5826; bundle 0 lines
666–690 and 830–925; and bundle 12 lines 767–825, 1199–1210 and 1422–1450.
Their phase-local paths and full-file hashes are the verified draft evidence entries.
Relevant public read commands/results and complete submitted edit bodies were inspected;
private reasoning was excluded.

The rejected visual candidates at S18453 and S29150 received explicit old-block-not-found
messages at C52 and C61. Retained `infrastructure/evidence/src/patch.rs`, SHA256
`aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda`,
lines 150–168, establishes that structured changes are computed before writes for
these hunk failures. The subsequent forced read S29194 is answered at C63.
Thus a failed earlier submission alone does not make ACK four replay/no-op or
repair of an already accepted visual implementation.

Post-ACK validation and done were checked separately: S3278/C30/S3296,
S11476/C44/S11854, S19233/C57/S19251, S39230/C67/S39248, and
S41712/C80/S41730. Each command-result prompt reports status 0. These subsequent
checks corroborate continuation, not the cause of an earlier edit. No later passing
test or final quality score supplies a fabricated preceding repair trigger.

## Limits and disposition

No separate typed `PatchApplied`/commit receipt exists in this evidence. The unique
preceding processed edit group is supported by exact same-thread sequence, interrupted
reply, receipt file set and source control flow; reviewer commit prose is contextual,
not a manufactured ACK-to-commit join. One assistant item or provider turn is not treated
as one completed model response. Native public projection is incomplete and coding is
not claimed perfectly blinded.

No mixed-purpose or unresolved candidate is supported by the inspected evidence in
these five groups. This finding is specific to workflow 001 and does not establish a
condition effect, token effect, causal share, or phase-level inference. Other workflow
outcomes, condition contrasts, token totals and quality scores were not inspected.
The draft remains unchanged. All 12 workflows still require complete final coverage
and a frozen classification file before either mediator or token contrast command.

This review-only document affects no executable or real-agent workflow; no provider
verification or runtime change is involved. Frozen helpers, protocols, captures,
checkouts and classification drafts are unmodified.
