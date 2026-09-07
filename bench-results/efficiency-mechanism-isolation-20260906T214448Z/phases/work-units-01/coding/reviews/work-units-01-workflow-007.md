# Independent ACK classification review: workflow 007

Reviewer: `/root/candidate_controls`. Scope: terminally published
`work-units-01-workflow-007` only, in ACK order under
`preflight/ACK-CLASSIFICATION-PROCEDURE.md`.

Disposition: accepted with no findings or classification disagreements. All nine
ACKs have supported labels. Complete bodies, prior state and delivered triggers
do not establish an independently remaining feature increment mixed into a repair
or an unresolved candidate group.

## Reviewed identities

- Phase manifest SHA256: `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
- Draft: `coding/drafts/work-units-01-workflow-007.json`, SHA256
  `dc4f3ffc7df8f6ed49c3a5a944ec3f570058b51ef4f0e5e43ef381d8a693a64c`.
- Existing packet: `coding/packets/work-units-01-workflow-007.json`, SHA256
  `0540a467137676e5db6ef1b139e396026bd19d3496f1aa716f910e05c8818dc0`.
- Terminal receipt: `logs/work-units-01-workflow-007.exit.json`, SHA256
  `a2b9bcb6ff550aeaedc27b72350544a37e0c05faed9f045dec5c25faa9ea4d8a`;
  completed with launcher exit code 0 at `2026-09-07T03:20:55.003565+00:00`.

## Evidence and identity coverage

All 250 packet source hashes and 108 draft evidence entries across thirteen
unique canonical phase-local sources passed endpoint revalidation. Mechanical
checks covered 115 explicit JSONL references at 95 distinct physical locations,
including JSON pointers, string-typed RPC/thread/turn/item identities and the
explicit cross-stream reference in ACK9. Compound deliveries, source ranges and
semantic claims were checked manually. Phase, packet and receipt hashes match;
all fourteen earlier 001–006 packet/draft hashes remain unchanged.

All nine complete accepted bodies and original feature requests were read before
the draft review, together with actual failures/reviews and relevant prior
bundles. Independent raw-stream indexing verifies every accepted ACK request/reply,
exact trace-forwarded input, unique preceding submitted body and receipt file set.
ACKs 1–8 have matching same-thread interrupts. ACK9 has a naturally completed
preceding turn instead; no interrupt or structured-edit footer is invented.

The existing packet was not regenerated. No frozen helper or whole-phase analyzer
was rerun. This review rechecks source identity and semantic anchors, not token
accounting or the whole-phase admission gate.

The table uses physical JSONL lines under
`runs/work-units-01-workflow-007/work-units-01-workflow-007-three-feature-bench-artifacts/observation/app-server/00000448924886790377-3844443/`.
`C` denotes `client-to-server.raw`, SHA256
`c704dccf5fa07e5f1b7f68be3697a31ee07cfd5ae167072747d73ddb38846bb7`;
`S` denotes `server-to-client.raw`, SHA256
`ebc0e042336d8b722e8c34df555d8ee071ae8a224659d97f3334f51b157128a6`.
Submitted bodies are at `/params/item/text`, inputs at `/params/input/0/text`,
and accepted turns at `/result/turn/id`. Full identities remain in the hash-bound
draft and packet; ordinals denote exact ACK order.

## Per-ACK semantic review

| ACK | Accepted RPC and request/reply | Preceding body / boundary | Label and supported purpose |
| --- | --- | --- | --- |
| 1 | string `21`, C22/S2460 | S2453 / interrupt C21 | `remaining-work`: two initial tests express C6's missing command-prompt slash route. Retained terminal/harness prompt handlers lack the branch; no implementation is included in this group. C24 compiles and fails the new terminal assertion while existing chat/raw-slash tests pass. |
| 2 | string `25`, C26/S3081 | S3074 / interrupt C25 | `remaining-work`: first implementation of that requested prompt route, adding the predicate and selected-agent dispatch without modifying tests. This is the missing prerequisite exposed by ACK1, not repair of an earlier implemented route. |
| 3 | string `39`, C40/S8704 | S8697 / interrupt C39 | `remaining-work`: initial completion-disposition implementation and dependent tests, including question/session state, yes confirmation, review loading/ready behavior and terminal hiding. Baseline review and message paths lack the requested flow. |
| 4 | string `52`, C53/S22828 | S22800 / interrupt C52 | `review-repair`: C49 identifies duplicate completion-sentinel loss in the terminal cache. C51's bundle 8 retains that filter. The edit exempts the sentinel and adds a repeated-review regression for already submitted behavior. |
| 5 | string `58`, C59/S25646 | S25637 / interrupt C58 | `validation-repair`: C55 fails the second-question frame assertion with the reviewer selected, before sending yes. The only change reselects the patch chat after the second review; C57's bundle 9 preserves the original test and assertion order. |
| 6 | string `66`, C67/S26522 | S26515 / interrupt C66 | `remaining-work`: first visual-mode implementation, with state, movement, highlighting/copying, adapters and tests. C4 and initial bundle 0 establish requested work absent from the baseline. No earlier visual edit is accepted or rejected in this thread's captured sequence. |
| 7 | string `84`, C85/S28875 | S28868 / interrupt C84 | `review-repair`: C81 identifies wrong left rendered-row mapping, slash interception, right-wrap mismatch and formatting defects in the submitted visual feature. C83's bundle 11 retains those mechanisms. The edit repairs them and adds dependent regressions; no independent requested increment is present. |
| 8 | string `88`, C89/S29108 | S29101 / interrupt C88 | `validation-repair`: C87 exits 0 but emits an unused-parameter warning violating the delivered warning-free requirement. The entire edit removes only that parameter and call argument from ACK7's buffer-based repair. |
| 9 | string `100`, C101/S30918 | S30912 / natural completion S30917 | `formatting-only`: C98 requests remaining formatting repair. C100 captures and reverts the formatter's tracked changes; the submitted unified patch exactly reproduces that pending diff. All eight hunks are layout/formatting changes, including one optional trailing comma removal. |

## Tests-first and repair-purpose checks

ACK1 and ACK2 remain two actual accepted units. Bundle 5 lines 2922–2925 and
bundle 1 lines 3315–3354 show the missing prompt branches; the existing terminal
chat-view behavior is retained at bundle 5 lines 3884–3904. The complete S2453
body contains only new prerequisite tests. C24 fails the terminal prompt-route
assertion while its existing chat/raw-slash tests pass; the displayed execution
stops before a harness result. S3074 supplies only the missing first dispatch
implementation. Its FIX title and preceding failing test do not independently
establish repair purpose. No already implemented defect or separate repair
component is present. The accepted failing intermediate target remains a separate
execution/compliance fact, not an exclusion or a claim of global shared-tree failure.

S2497/C24/S3074 and S3110/C28/S3128 establish the slash check/failure/implementation
and subsequent passing check/done chains. For completion, bundle 2 lines
4396–4411 and bundle 4 lines 1382–1400 retain the initial review/send paths.
S8730/C42/S8750 is the initial filtered check/result/done chain. Its `feature_done`
filter does not run every newly named completion test; no full coverage is claimed.

ACK4's prior state is retained in S8697 and bundle 8 lines 648–674 and 789–804:
the controller allows repeated sentinels, the terminal derives pending closure
from its last cached line, and the cache drops duplicates. S22974/C55/S23730 is
the regression failure followed by a read, not a failed post-yes close.
Delivered bundle 9, SHA256
`f0c0524d4614d08e21cf83959837e45a8c27ed970d5f002ae132ca3789108e4c`,
lines 3744–3787 places the failing frame assertion before the yes send and later
close assertions. S25637 adds only the missing second patch-row click.
S25793/C61/S26052 is the passing repeat/done chain. This test-purpose label is
separate from whether the overall product should automatically select that chat.

ACK6's complete implementation supplies all named visual modes and their initial
tests. S26557/C69/S26575 is its filtered check/result/done chain. For ACK7,
delivered bundle 11, SHA256
`8da95f0b14e1c60f730c34d33363b960217e51126ab6d1e943a992d3d3f8f733`,
retains logical-row anchoring at 2330–2337, slash predicates at 823–825 and
3779–3781, and fixed-character wrapping at 3048–3068, alongside the wrapped
renderer at 2134–2141. Its behavior and formatting edits address the captured
review findings, not independent remaining feature work. The group is not
formatting-only because it contains substantive behavior repairs.

S28911/C87/S28975 separates passing selected tests from the unused `right_content`
warning. C4's delivered Required Checks clause prohibits build warnings; this
existing non-factor clause, the actual diagnostic and the complete one-parameter
edit support ACK8. S29144/C91/S29162 is the clean filtered repeat/done chain.
The `visual` filter omits the new right-yank test name, so those results are not
represented as complete coverage of ACK7's regressions.

## ACK9 natural completion and captured formatter output

C98's delivered review identifies remaining formatter differences and reports
earlier behavioral findings addressed. S29489 requests the scoped formatter;
C100 reports status 0 and explicitly states that tracked output was captured,
reverted to HEAD and retained as pending for the patch agent. The exact diff in
that input equals the body after S30912's unified-patch header byte-for-byte:
5,167 UTF-8 bytes, SHA256
`649c6ed7303debe223efa8a92c5c16f346f7ac68964a8cf9c12953c0ccbe7526`.
The complete eight-hunk body contains formatting only, including formatting of
concurrent completion-owned lines and removal of an optional trailing argument
comma. This is not falsely characterized as whitespace alone or a previously
applied no-op.

The unique preceding patch item is
`msg_0bbbcfd31d2a3182016a9e2b1119a887d28a436cb872edb2ac`, turn
`01a079d8-2c12-7251-a672-10c37edf0c06`, thread
`01a079bd-97b8-7481-9db2-f9d3e2a9b7a2`. S30917 is that exact turn's
`turn/completed` notification with status `completed`, before S30918 accepts the
ACK continuation. No `turn/interrupt` exists for the preceding patch turn.

Retained `src/orchestrator.rs`, SHA256
`11b1e131b8d6e6f50100575c4d43c5e245243fe501f9280b8f899f197f947ae0`,
lines 1179–1216 captures, reverts and returns pending tracked diffs;
lines 789–810 applies submitted unified patches and clears pending files.
Its parser at 1680–1708 accepts a nonempty unified body through end of reply;
an absent explicit end footer is not invented. Retained `src/cli.rs`, SHA256
`70d5107a5b022f4dba118e8567a5c2f54330a51319f5d97d84299380e6246c0d`,
lines 1035–1051 and 1079–1098 queues returned replies for directive processing
without an interrupt prerequisite. S30994/C103/S31012 is the scoped passing
format-check/result/done chain. This source-supported application chain is not
a separate typed patch-event-to-commit join.

## Limits and disposition

Labels describe evidenced purpose, not repair adequacy, final quality, test
equivalence or full instruction compliance. No title, file count, desired count,
condition identity or later approval determines the labels. The accepted ACK
turn is the continuation after the acknowledged body, not that body's generation
turn. Neither provider turns nor assistant items are equated with completed model
responses. Private reasoning is excluded, native body projection remains
incomplete and perfect blinding is not claimed.

No token totals, condition contrasts, p-values, quality scores or workflow 008+
outcomes were inspected. This review establishes no treatment effect, token
effect or historical causal share. Complete all-12 classification and its freeze
remain required before either mediator or token contrast CLI.

Only this review document is written by the reviewer. Draft, packet, frozen
helpers/protocol, captures, candidate checkouts and prior reviews remain unchanged.
This offline review affects no executable behavior, architecture or public API;
no implementation documentation update or additional real-agent verification is
required.
