# Independent ACK classification review: workflow 010

Reviewer: `/root/candidate_controls`. Scope: terminally published failed
`work-units-01-workflow-010` only, in ACK order under
`preflight/ACK-CLASSIFICATION-PROCEDURE.md`.

Disposition: accepted with no findings or classification disagreements. All ten
accepted ACKs remain classified despite the later contribution-gate failure.
Complete bodies, prior state and actual delivered triggers support the labels;
no independently remaining increment mixed into a repair, competing accepted
group or unresolved prior-state ambiguity is established here.

## Reviewed identities and coverage

- Phase manifest SHA256: `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
- Draft: `coding/drafts/work-units-01-workflow-010.json`, SHA256
  `4da7c1f9f97f86ffd88a54bb38a211715c08ca16884fae84de4045fff98cb424`.
- Existing packet: `coding/packets/work-units-01-workflow-010.json`, SHA256
  `4b5b55607a5f140aeee10193aef34f1750af8e0a2fffcccb40a78d4d69c3f696`.
- Separate observation: `coding/operational/WORKFLOW-010-AGENT-CONTRIBUTION.json`,
  SHA256 `d81c43925e8e8833f3353d875a8053e130ca55295e3c76a037466d513d45e49f`.
- Terminal receipt: `logs/work-units-01-workflow-010.exit.json`, SHA256
  `489ba5801916403da9ad07033aa7acdf7aa8d143c2ef2bbf788dc741e4838f6a`;
  completed invocation with launcher exit code 1 at
  `2026-09-07T04:12:08.170280+00:00`, not a successful workflow.

All 165 packet source hashes passed endpoint revalidation. The 125 draft evidence
entries across twelve canonical phase-local sources passed hash and locator
checks: 146 explicit physical JSONL references at 124 distinct locations,
including JSON pointers, string-typed RPC identities, thread/turn/item values
and explicitly named opposite-stream replies. The separate observation's
seventeen references also pass, including its sixteen explicit JSONL references,
baseline source range and final-state JSON pointers. All semantic source ranges
and claimed command/assertion order were inspected directly.

All twenty independently retained prior 001–009 packet/draft hashes match,
including original and revision-01 drafts for 003 and 005. The twenty-two tracked
prior coding artifacts—eleven original/revision drafts, two correction notes and
nine reviews—match frozen HEAD byte-for-byte. All nine prior packets match their
independently retained hashes.

The ten complete accepted bodies, original requests, relevant prior bundles and
actual review/validation prompts were read before reviewing the draft. Independent
raw indexing verifies every accepted ACK request/reply, exact forwarded trace
text, unique preceding complete structured edit, matching same-thread/turn
interrupt and exact receipt file set. All ten ACK-to-command/result/following-action
chains pass exact typed-turn joins. This does not manufacture a typed
`PatchApplied`-to-commit identity. The packet was not regenerated and no frozen
helper or whole-phase analyzer was rerun.

The table uses physical JSONL lines under
`runs/work-units-01-workflow-010/work-units-01-workflow-010-three-feature-bench-artifacts/observation/app-server/00000452179764996037-4005148/`.
`C` means `client-to-server.raw`, SHA256
`6ec02b6c28f91bce68e2cb5433edfb1cfdb9cb9e4327b37bdfa8de8ea6d07fd0`;
`S` means `server-to-client.raw`, SHA256
`30445cc6959e4fe8b8b0dd81cdac7eddfe0167eb232aea7aa43916745f7c0f87`.
Bodies are at `/params/item/text`, inputs at `/params/input/0/text`, and accepted
turns at `/result/turn/id`. Full identities remain in the hash-bound draft.

## Per-ACK semantic review

| ACK | Accepted RPC and request/reply | Preceding body / interrupt | Label and supported purpose |
| --- | --- | --- | --- |
| 1 | string `42`, C43/S27518 | S27508 / C42 | `remaining-work`: first completion question, yes/no handling, hiding/reopening state, terminal/harness projection and dependent tests. Prior rejected candidates are not accepted work. |
| 2 | string `52`, C53/S33530 | S33523 / C52 | `remaining-work`: first visual selection, motion, highlighting, clipboard output, adapters and dependent tests. Baseline lacks visual/yank state. |
| 3 | string `70`, C71/S34914 | S34905 / C70 | `review-repair`: C67 identifies missing clipboard emission in the non-cursor renderer; the group inserts its missing `copy_prefix` call. |
| 4 | string `79`, C80/S36330 | S36319 / C79 | `review-repair`: C78 identifies logical-line versus wrapped-row selection/copy; the group changes both source branches and adds a regression. |
| 5 | string `85`, C86/S38668 | S38661 / C85 | `review-repair`: C62 identifies loss of the closed chat's follow-up target; the group repairs routing/hiding and dependent terminal/harness/controller tests. |
| 6 | string `91`, C92/S39068 | S39061 / C91 | `validation-repair`: C88 demonstrates an overbroad user-2 absence assertion; the test-only group checks the hidden/reopened agent row instead of unrelated metadata references. |
| 7 | string `102`, C103/S40202 | S40195 / C102 | `review-repair`: C99 identifies remembered-closed-agent precedence over a selected visible chat; the group repairs terminal routing and adds a two-agent regression. |
| 8 | string `109`, C110/S40968 | S40961 / C109 | `review-repair`: C108 identifies an exact-width phantom row; the group changes display-row counts and extends the wrapping test. |
| 9 | string `120`, C121/S42157 | S42150 / C120 | `review-repair`: C117 identifies logical selected-agent index versus rendered metadata rows; the group calculates the actual left-pane row and adds a harness regression. |
| 10 | string `127`, C128/S42582 | S42575 / C127 | `review-repair`: C126 explicitly requests chunk iteration in the existing wrapping helper; the group performs that substitution and preserves empty lines. The alleged complexity is not independently endorsed. |

## Prior state, repair purpose and validation limits

For ACK1, delivered bundle 2 lines 3055–3074 retains summary-only review completion;
bundle 5 lines 1382–1399 sends only to the selected or command agent. The accepted
body connects the requested feature through those layers. C37 rejects S7563 for
a duplicate file header; C39 rejects S18266 for an old-block mismatch. For ACK2,
delivered bundle 0 lines 805–901 retains ordinary modes/navigation without visual
selection. C41 rejects S20120 for an ambiguous hunk. Frozen `src/patch.rs`, SHA256
`aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda`,
`GitPatcher::apply_edit`/`apply_edit_with_locks` lines 77–87 and 150–168, and
duplicate-header rejection at 590–602, supports rejection before partial writes.
No rejected attempt is relabeled as successful replay.

ACK1's S28280/C45/S28604 check/result/done chain passes one terminal close unit
test and one controller prompt/yes test. The `feature_done` filter excludes the
original differently named harness and controller reopen tests. ACK2's
S33561/C55/S33581 runs only the two visual-named harness tests, not the newly
added pure-UI or terminal integration tests. These are limited checks, not proof
of fully working initial implementations.

For ACK3, bundle 11 lines 590–622 confirms that only the cursor renderer emits
the pending copy. C67 is the actual repair trigger and reports failing pure-UI
tests; it is not falsely represented as a separate captured command result.
S34942/C73/S34960 passes both named pure-UI tests and returns done. For ACK4,
bundle 11 lines 1305–1322 and 1367–1381 retains the logical-line branches;
S34905 does not change them. S36606/C82/S36868 passes three visual tests after
the wrapping repair. The later C108 exact-boundary defect preserves this repair's
inadequacy without changing its evidenced purpose.

For ACK5, delivered bundle 10 lines 641–659 and 1828–1851 retains deselection on
hide and command-agent routing when selection is absent. Bundle 12 lines
186–204 and 305–329 confirms the harness's corresponding selected-agent path.
S38661 repairs the already attempted close/reopen behavior, including typing
after yes into the same closed agent, rather than a separately remaining request.
S38695/C88/S38728 is its failed check/result/read chain. The terminal unit passes;
the harness fails its broad user-2 absence assertion before post-close typing.
No later target or unreached assertion is treated as passing.

For ACK6, bundle 13 lines 2632–2692 retains the exact failing test order, and
lines 1934–1935 plus 2572–2588 establish the user-2 references in the remaining
parser agent's conflict/dependency metadata. The selected-agent inequality
precedes the demonstrated failure. S39061 changes only row-specific assertions.
S39095/C94/S39118 subsequently passes the terminal close/reopen unit, corrected
harness test and controller prompt/yes test; the differently named controller
reopen test remains filtered.

For ACK7, delivered bundle 14 lines 643–671 confirms the closed-agent fallback
precedes selected-agent routing. S40195 repairs only the terminal adapter and
adds its dependent test; it does not repair the analogous harness precedence.
S40229/C105/S40247 passes two terminal units, one harness completion test and one
controller prompt/yes test. Purpose is independent of unmodified edge cases.

For ACK8, bundle 13 lines 1597–1606 and S36319 preserve cursor-row arithmetic
used for display chunks. S40961 changes that helper and extends the existing
test. S40993/C112/S41011 passes three pure-UI visual tests, including the
exact-boundary scenario inside the extended test. For ACK9, delivered bundle 15
lines 964–971 and 1328–1367 establishes the selected-index/metadata-row mismatch.
S42183/C123/S42201 passes all three visual-named harness tests, including the new
rendered-row regression.

For ACK10, S40961 is the earlier helper and S42150 leaves it unchanged. C126 is
an explicit review request to replace skip/take with chunks; S42575 follows that
request and adds no feature or test. The reviewer alleges quadratic rescanning,
but this semantic audit neither proves Rust slice-iterator skip complexity nor
infers a performance/token improvement. S42607/C130/S42625 passes the three
existing pure-UI visual tests and returns done. Repair-purpose classification
does not require endorsing the reviewer's complexity diagnosis.

## Separate missing-contribution observation

The slash-agent observation is accepted with its explicit limitations. In thread
`01a079ef-5402-7f20-9a09-a340db878dfd`, C6/S17 starts the original request;
S135, S684 and S1755 request three mediated reads. S1594 explicitly notes the
literal `/status` command-prompt gap if that submission surface is in scope.
Bundle 3 lines 3627–3649 corroborates distinct parser and selected-chat routes.
S1904/C28, S1949/C30 and S2040/C32 execute three existing-path tests, each passing
one test. S2011's public conclusion is limited to the selected-agent slash checks
before the HTTP check. C32/S2047 accepts turn
`01a079f2-b534-7160-9ac1-e65e840e464c`; S2060 returns `@work-leaf done`.
No public edit/patch directive or accepted ACK appears in this thread.

The observation's final-state pointers identify user-2 at `/snapshot/sessions/3`,
done at `/lines/72`, the reported-done receipt at `/lines/73`, and null loading.
The ten retained commit subjects name user-1/user-3 only. Subjects are not an
independently replayed `Agent-ID` trailer inventory or typed ACK-to-commit join.
The observed sequence supports checks followed by done without a public edit,
not an explanation of hidden motivation or proof that the full request was
already satisfied. It does not prove absence of all native activity.

The committed `WORKFLOW-010-TERMINATION.md` gate account is separately corroborated
by the retained report header, SHA256
`2d08d4f214dd66954a044e97b0c21a2cc65654282b03e4767cfab71aeca980f5`,
and frozen driver SHA256
`d2487780c63c14021904b8a3c882d54fe231c5846f4a7f57fe955f50201f5644`.
Driver line 1190 counts distinct patch-agent trailers; lines 1206–1209 rejects
two rather than three before `force-linearize` at 1216. `fail_bench` lines
965–973 publishes exit 1. The `review_completed=yes` flag is not evidence that
all features passed review, and no unreached final-test failure is invented.

## Scope and disposition

Failure does not remove ACKs, zero-fill missing work or authorize replacement.
Labels describe purpose, not final quality, repair adequacy or instruction
compliance. The accepted ACK turn is the continuation after its acknowledged
body; provider turns and assistant items are not equated with completed model
responses. Reasoning bodies are excluded and perfect blinding is not claimed.

No 011+ semantic bodies, token totals, quality scores, condition contrasts,
p-values or whole-phase analysis were inspected. No treatment/token effect or
historical causal share is inferred. All-12 classification and its freeze remain
required before either mediator or token contrast CLI.

Only this review document is written by the reviewer. The immutable draft,
observation, packet, frozen helpers/protocol, captures, candidate checkouts and
prior reviews remain unchanged. This offline review affects no executable
behavior, architecture or public API; no implementation documentation update or
additional real-agent verification is required.
