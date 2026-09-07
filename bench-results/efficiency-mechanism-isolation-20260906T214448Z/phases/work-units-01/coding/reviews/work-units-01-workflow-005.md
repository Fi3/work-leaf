# Independent ACK classification review: workflow 005

Reviewer: `/root/candidate_controls`. Scope: terminally published
`work-units-01-workflow-005` only, in ACK order under
`preflight/ACK-CLASSIFICATION-PROCEDURE.md`.

Disposition: revision 01 is accepted with no remaining findings or classification
disagreements. All ten ACKs have supported labels. Two evidence-description errors
are resolved in a separately retained revision; ACK identities, order and labels
are unchanged.

## Reviewed identities and resolved findings

- Phase manifest SHA256: `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
- Current draft: `coding/drafts/work-units-01-workflow-005.revision-01.json`, SHA256
  `0aebe4480810dceb3d418be72fb2c0c5481c855a8f54602c9ba8ba9e737383cc`.
- Preserved original: `coding/drafts/work-units-01-workflow-005.json`, SHA256
  `e9428e27ac96e2152aa184863d150ac77f4b543ea71ff43f0845b45eae60ffe5`.
- Existing packet: `coding/packets/work-units-01-workflow-005.json`, SHA256
  `5da59338e82c8152ed09ce648a689e053087534bb72ba2a0ecbea5b6c6528dda`.
- Terminal receipt: `logs/work-units-01-workflow-005.exit.json`, SHA256
  `45c6026787c127e81025a9987d163e59c09d9bfa3618d645210683da28f780a4`;
  completed with launcher exit code 0 at `2026-09-07T02:32:37.581934+00:00`.

The original ACK7 description implied that both second-cycle question and DONE?
assertions preceded the failing reviewer-summary assertion. The complete test
actually checks initial-cycle DONE?, then after reopening the second question,
then the summary, then second-cycle DONE?. The failure does not establish that
the last assertion ran. Revision 01 distinguishes these observations explicitly.

The original ACK9 reference associated client line 96 with bundle 11. That exact
request delivers bundle 12. Revision 01 cites the delivered bundle's path, hash
and corresponding source lines. The prior-state code supports the same repair
purpose; the delivery association was wrong, not the label.

Independent recursive comparison verifies exactly six changed JSON values:
ACK7 rationale and one prior-test locator; ACK9 delivery locator and the prior
bundle's path/hash/locator. No other field differs. The adjacent revision note
preserves these exact paths and both draft identities.

## Evidence and identity coverage

All 258 packet source hashes and all 120 draft evidence entries across ten unique
canonical phase-local sources passed endpoint revalidation. Every cited source
passage and physical locator, including compound references, was checked against
its claimed role. Mechanical checks covered 111 distinct explicitly numbered
physical JSONL locations, including explicitly named cross-stream replies, JSON
pointers and typed RPC/thread/turn/item identities. Bundle deliveries, source
ranges, command failures and additional compound references were inspected manually.
All ten complete successful edit bodies and original feature requests were read
before the final draft review, together with relevant prior state and triggers.

The existing packet was not regenerated and no frozen helper or whole-phase
analyzer was rerun. Independent raw-stream indexing verifies each ACK's accepted
string-typed request/reply, exact trace-forwarded input, preceding accepted
same-thread turn, unique complete structured edit, matching interrupt and exact
receipt file set. This is not a claim that an accepted provider turn or assistant
item equals a completed model response. The packet's existing provenance remains
preserved; this review independently rechecks its source identities and semantic
delivery anchors, not token accounting or the whole-phase integrity gate.

The following table uses physical JSONL lines under
`runs/work-units-01-workflow-005/work-units-01-workflow-005-three-feature-bench-artifacts/observation/app-server/00000446260581883146-3719549/`.
`C` denotes `client-to-server.raw`, SHA256
`0ceaabfa316a0089b1eee2f503d4aa897295f47009581a9e1931192b5f39e55b`;
`S` denotes `server-to-client.raw`, SHA256
`21ea08aea1cdc94c109e4e2446db90cc9b298d2cb951a77ca1bcb85533676414`.
Edit bodies are at `/params/item/text`, inputs at `/params/input/0/text`, and
accepted turns at `/result/turn/id`. Full ACK, thread, turn and item identities
are retained in the hash-bound draft and packet; table ordinals are exact ACK order.

## Per-ACK semantic review

| ACK | Accepted RPC and request/reply | Preceding edit / interrupt | Label and supported purpose |
| --- | --- | --- | --- |
| 1 | string `23`, C24/S2813 | S2806 / C23 | `remaining-work`: first requested completion-disposition tests, explicitly planned as failing coverage at S1505. No completion implementation exists in initial bundles 2/5; C26 exposes the proposed missing DTO/field. The accepted tests-only state remains a separate execution/compliance fact. |
| 2 | string `29`, C30/S5158 | S5151 / C29 | `remaining-work`: C6 requests selected-backend slash routing. Bundle 3's terminal/harness handlers lack the route; implementation and dependent tests provide it. C32's selected slash tests pass. |
| 3 | string `43`, C44/S14947 | S14938 / C43 | `remaining-work`: first controller/terminal/UI completion implementation, with its prerequisite test changed to the chosen session-line representation. S1505/C26/S2904/S2971 establish intended RED and the public-API-compatible design choice. The mixed-purpose question is resolved below, not by initial position alone. |
| 4 | string `49`, C50/S18177 | S18170 / C49 | `validation-repair`: C46 demonstrates existing terminal-test substring/history assertion failures. The entire edit narrows patch-agent row matching and removes the historical-question absence assertion; no new requested feature is supplied. |
| 5 | string `53`, C54/S18390 | S18383 / C53 | `validation-repair`: C52 identifies the existing harness test's exact ANSI substring failure. The entire edit checks reverse styling and DONE? text separately. |
| 6 | string `70`, C71/S31686 | S31676 / C70 | `review-repair`: C67 identifies terminal-cache duplicate filtering suppressing a second completion prompt after reopening. Bundle 10 retains that filter/state derivation. The marker exemption and two-cycle test repair the existing lifecycle. |
| 7 | string `74`, C75/S33307 | S33294 / C74 | `validation-repair`: C73 fails the newly submitted two-cycle test at its reviewer-summary assertion. The entire edit removes that assertion while retaining second-cycle question/DONE? checks. The latter DONE? assertion was not reached in the failing run. |
| 8 | string `80`, C81/S38844 | S38837 / C80 | `remaining-work`: C4 requests visual modes and yanks. Bundle 0 lacks selection state/CtrlV. The complete group provides modes, movement, highlighting/copying, terminal/harness routing and tests. C48/C64 reject earlier candidates rather than establish accepted prior visual work. |
| 9 | string `97`, C98/S40813 | S40806 / C97 | `review-repair`: C94 identifies logical-row selection versus wrapped display rows. C96's bundle 12 retains the exact inconsistent snapshot/styling/renderer. Wrapped-row mapping and its regression repair existing visual behavior. |
| 10 | string `103`, C104/S41231 | S41224 / C103 | `validation-repair`: C102's valid command fails two existing visual harness expectations after wrapping changes. The edit changes only those line/block expectations; C106 subsequently passes without establishing broader quality. |

## ACK1/ACK3 tests-first and mixed-purpose adjudication

The possibility of mixed remaining work plus test repair was examined explicitly.
ACK3 revises an already accepted test, so absence of prior implementation alone
would not justify a pure remaining-work label. The following concrete evidence
resolves its purpose in this case:

1. Before ACK1, S1505 expressly proposes failing controller/terminal coverage
   first. The complete S2806 group introduces a proposed DTO/field assertion and
   two end-to-end completion tests, not a previously working implementation.
2. C26 reports only the absent prospective DTO/field at this initial seam. S2904
   identifies that failure as intended. It is not evidence of an already
   implemented close/reopen transition behaving incorrectly.
3. S2971 then explicitly chooses a session-line representation because adding
   a required public session field would break external struct literals. The
   complete S14938 group implements that representation and changes precisely
   the prerequisite import/assertion to express the same completion-question
   design. The other changes implement the requested first controller/UI flow
   and dependent harness coverage.

The test-design adjustment and implementation therefore have one dependent
initial-feature purpose. No separately demonstrated previously implemented
behavioral defect or independent repair component is present in that group.
Calling the abandoned DTO assumption a design error describes this same initial
representation choice, not an additional evidenced repair episode. This judgment
does not assert that the two tests are equivalent, that the implementation is
correct, or that every tests-first failure should receive this label. Later
assertion defects and repeated-prompt behavior have distinct actual triggers and
remain repairs. A genuinely independent repair component would require `unresolved`
under the frozen mixed rule; no such component is supported here.

ACK1 and ACK3 remain two actual accepted ACK units. They are not merged because
they concern one feature, nor subdivided by file or test count. C26 establishes
that ACK1's accepted shared-checkout workspace-test target did not compile. That
non-buildable intermediate target and possible instruction-compliance issue are
retained separately from purpose and quality; neither is an exclusion. C32's
different slash-target command passes, so the evidence does not establish that
all shared-tree validation was blocked.

## Prior-state, rejection and continuation checks

Retained initial source passages include bundle 2's controller completion branch,
bundle 5's terminal message handling, bundle 3's slash handlers and bundle 0's
UI state/dispatcher. Bundle 10 preserves the existing terminal completion cache
and first-cycle tests. Delivered bundle 12, SHA256
`6cf8cb47c031dc8902c79375cfdc2721837c47d21ba3c8210bedf32890efd409`,
lines 817–824, 1112–1119 and 1197–1202, preserves ACK9's wrapped renderer and
logical-row snapshot/styling. Earlier complete accepted edits establish the
intervening test assertions when there is no fresh whole-file snapshot.

C48/C64 reject visual old-block matches; C66 supplies changed mediated context.
The rejected groups are not accepted no-ops. Retained `src/patch.rs`, SHA256
`aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda`,
declares `GitPatcher` at lines 54/59 and computes structured changes before the
write loop at lines 150–168. These hunk failures do not partially write a candidate.

Post-ACK commands and public continuations were checked separately from the
preceding acknowledged edits. S2849/C26 leads to the initial representation
decision/read; S5189/C32/S5213 is the slash check/result/done chain.
S15587/C46/S18170 and S18198/C52/S18383 lead to the two assertion repairs;
S18411/C56/S18431 is their later passing focused check/done.
S32010/C73/S33120/S33294 ties the two-cycle assertion failure to its repair;
S33542/C77/S33739 is the subsequent passing check/done.
Visual commands S38908/C83 and S39014/C85 fail Cargo argument parsing before
tests; S39050/C87/S39068 is the corrected passing check/done.
After the wrapping repair, S40869/C100 is another invalid-filter attempt;
S40905/C102 is the valid failing harness run supporting ACK10, followed by
S41258/C106/S41276's passing run/done. Malformed commands are not runtime-test
failures, and later passing checks do not create earlier repair triggers.

## Limits and disposition

Labels describe evidenced purpose, not repair adequacy or final quality. No
desired count, title, file count, later reviewer approval or treatment identity
determines the labels. The driver lacks a separate typed `PatchApplied`-to-commit
log; no such join is manufactured. Unique complete edits, same-thread accepted
sequence, interrupts, receipts and retained source flow support the groups.
Private reasoning is excluded. Native body projection is incomplete and perfect
blinding is not claimed.

No token totals, condition contrasts, p-values, quality scores or workflow 006+
outcomes were inspected. This review establishes no treatment effect, token
effect or historical causal share. Complete all-12 classification and its freeze
remain required before either mediator or token contrast CLI.

Only this review document is written by the reviewer. The original and revised
drafts are preserved coder artifacts. Frozen helpers/protocol, captures, candidate
checkouts and prior reviews remain unchanged. This offline review affects no
executable or real-agent workflow and requires no additional provider verification.
