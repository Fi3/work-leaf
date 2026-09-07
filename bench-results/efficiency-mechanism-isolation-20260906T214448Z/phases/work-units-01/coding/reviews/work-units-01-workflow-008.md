# Independent ACK classification review: workflow 008

Reviewer: `/root/candidate_controls`. Scope: terminally published
`work-units-01-workflow-008` only, in ACK order under
`preflight/ACK-CLASSIFICATION-PROCEDURE.md`.

Disposition: accepted with no findings or classification disagreements. All five
ACK labels are supported by the complete accepted bodies, prior state and actual
delivered triggers. No independently remaining feature increment mixed into a
repair, or unresolved candidate group, is established by this evidence.

## Reviewed identities

- Phase manifest SHA256: `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
- Draft: `coding/drafts/work-units-01-workflow-008.json`, SHA256
  `199be3f2f208b0cd432a88751383ec08310ba21daeda62b2d59905f0e42562ff`.
- Existing packet: `coding/packets/work-units-01-workflow-008.json`, SHA256
  `725ad4937df4eb5d1a3066f5ea25447ee52488043d614fe6870b7ed5a016af83`.
- Terminal receipt: `logs/work-units-01-workflow-008.exit.json`, SHA256
  `bacca3209ea51981de47eaa8c8454ab8a78b46101b23f64ec7475f3e8d7aebf9`;
  completed with launcher exit code 0 at `2026-09-07T03:31:46.734979+00:00`.

## Evidence and identity coverage

All 350 packet source hashes and 78 draft evidence entries across eleven unique
canonical phase-local sources passed endpoint revalidation. Mechanical checks
covered 84 explicit JSONL references at 74 distinct physical locations, including
JSON pointers, string-typed RPC/thread/turn/item identities, accepted-turn reply
joins and explicitly named opposite-stream references. All cited source ranges,
compound deliveries and semantic claims were also inspected directly.

All sixteen earlier 001–007 packet/draft hashes remain unchanged, including both
original drafts and revision-01 files for 003 and 005. In particular, workflow
007's packet remains `0540a467137676e5db6ef1b139e396026bd19d3496f1aa716f910e05c8818dc0`
and draft remains `dc4f3ffc7df8f6ed49c3a5a944ec3f570058b51ef4f0e5e43ef381d8a693a64c`.
Sixteen is the verified complete prior packet/draft inventory, not the fourteen
files covering only 001–006.

The five complete accepted bodies and original requests were read before the
draft review, together with relevant prior bundles, refreshes and actual command
and review inputs. Independent raw-stream indexing verifies each accepted ACK
request/reply, exact forwarded trace text, unique preceding complete structured
edit, same-thread/turn interrupt and exact receipt file set. The five subsequent
check/result/done chains and the final visual-review request/reply also pass
exact typed-turn joins. These checks do not fabricate a typed patch-event-to-commit
join.

The existing packet was not regenerated and no frozen helper or whole-phase
analyzer was rerun. This is a source and semantic-anchor review, not a replay of
the whole-phase accounting or admission gate.

The table uses physical JSONL lines under
`runs/work-units-01-workflow-008/work-units-01-workflow-008-three-feature-bench-artifacts/observation/app-server/00000448924902156465-3844489/`.
`C` denotes `client-to-server.raw`, SHA256
`14d5026a3abd0f508992c096b425b4b44256e876ad6fef69363819f7accc30d2`;
`S` denotes `server-to-client.raw`, SHA256
`110d43e43feb58b7408478c3f39d6e1288a3621b2da39495e26ec8d910b7419c`.
Submitted bodies are at `/params/item/text`, inputs at `/params/input/0/text`,
and accepted turns at `/result/turn/id`. Full identities remain in the hash-bound
draft and packet; ordinals denote exact ACK order.

## Per-ACK semantic review

| ACK | Accepted RPC and request/reply | Preceding body / interrupt | Label and supported purpose |
| --- | --- | --- | --- |
| 1 | string `25`, C26/S6567 | S6560 / C25 | `remaining-work`: first completion-question, review-wait/ready and yes-hide/type-reopen implementation, with dependent tests. Baseline review/message paths lack the requested flow; existing ready/bell and manual hiding are reused. |
| 2 | string `47`, C48/S14300 | S14293 / C47 | `remaining-work`: first command-prompt slash routing implementation and dependent tests. Existing selected-chat slash behavior remains distinct. Earlier submissions are rejected, not previously accepted or replayed. |
| 3 | string `59`, C60/S20660 | S20650 / C59 | `review-repair`: C42 identifies patch-agent loading left busy on automatic review error in ACK1. The group clears that loading and tests the error/follow-up path; no independent feature increment is included. |
| 4 | string `76`, C77/S41795 | S41788 / C76 | `remaining-work`: first visual-selection state, rendering, movement, yank/clipboard and adapter implementation with tests. Earlier failed submissions remain rejected. C79's subsequent failure concerns the completion test's initial bell assertion, not a prior implemented visual defect. |
| 5 | string `101`, C102/S44307 | S44300 / C101 | `review-repair`: C98 objects to the public `UiKey::CtrlV` API break. The group keeps Ctrl-V internal through private input variants and a crate-private selection setter, preserving the existing feature; it does not repair the earlier completion-test failure. |

## Prior-state, rejected-group and trigger checks

For ACK1, delivered bundle 1 lines 3019–3069 preserves the baseline review/loading
and summary-only paths; lines 725–737 and 1348–1365 preserve ready/bell and manual
hide primitives. Bundle 4 lines 719–736 forwards chat without completion-answer
interception. The complete S6560 group supplies the missing requested flow rather
than repairing a previously accepted completion implementation.

For ACK2, bundle 2 lines 3794–3805 and 3203–3238 distinguishes the Work Leaf
command parser from selected-chat handling. Bundle 3 lines 2180–2202 preserves
the same distinction in the terminal adapter. C35 and C39 reject S8570 and
S11485 for missing old blocks. C41 supplies the concurrent completion changes;
S14293 preserves completion interception while adding slash dispatch. The
controller selection cache belongs to the initial requested route, not an
independently repaired feature. C71's generic review wrapper contains actual
`NO_FINDINGS`; S25248 is done, not another edit.

For ACK3, C42's actual finding and bundle 8 lines 648–688 and 701–708 identify
the asymmetric automatic-review completion/error handlers after ACK1. The
specific missing clear is on `WorkerEvent::ReviewError`; other branches already
clear loading when no reviewer starts or startup returns an error. S20650 adds
only that missing clear and its dependent reviewer-launch-error regression.
C58 rejects the earlier S15893 candidate after a concurrent slash-test insertion;
the accepted resubmission is not a no-op.

For ACK4, bundle 0 lines 477–488 and 778–873 preserves the ordinary key handling
without visual selection/yank support. S25049 is rejected by C68 for an unmatched
harness hunk; C70 refreshes concurrent completion/slash state. S33385 is rejected
by C75 for an ambiguous UI old block. S41788 is the first accepted full visual
group. The retained `src/patch.rs`, SHA256
`aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda`,
`GitPatcher::apply_edit_with_locks` lines 150–168 computes all structured-edit
changes before any write. These hunk failures do not establish partial accepted
work or an accepted replay.

For ACK5, C98's concrete review request and delivered bundle 13 lines 1542–1554,
1825–1846 and 884–890 establish the existing public variant and its input uses.
The complete S44300 body removes that public variant and substitutes private
Ctrl-V inputs plus a crate-private setter. It changes neither completion bell
setup nor its test, and introduces no independently remaining visual capability.
Bundle 13 lines 3400–3405 preserves the completion helper unchanged. C106's
accepted reviewer continuation produces S44374 `NO_FINDINGS`; that later approval
is not the basis for the repair-purpose label.

## Validation limits retained separately from purpose

S7255/C28/S8010 is ACK1's filtered check/result/done chain. The `feature_done`
filter runs only the terminal unit test, not the new completion harness test.
S14342/C50/S14360 is ACK2's corresponding chain: `slash_command` runs three
terminal and two harness tests but zero workspace tests, omitting the new
`controller_routes_slash_prompt_to_selected_agent` regression. Neither filter
establishes complete coverage of its submitted group.

S20934/C62/S21181 runs the one exact review-error regression successfully and
then returns done. This supports the narrow observed repair validation, not
every completion behavior.

S41820/C79/S41840 runs the whole harness target, exits 101 and then returns done.
All five visual tests pass; the sole failure is
`scripted_harness_closes_reviewed_feature_on_yes_and_reopens_on_typing` at its
initial `ready_frame.starts_with` bell assertion, before yes or reopening.
The full result and cross-agent validation guard remain visible; this is neither
a full-suite success nor evidence that the final API-only edit repairs that
failure. Test failure, adequacy and possible instruction compliance remain
separate from acknowledged-group purpose.

S44338/C104/S44356 is ACK5's filtered check/result/done chain. Its
`scripted_harness_visual` filter passes four tests but excludes the separately
named Ctrl-V-entry test and the failing completion test. No complete post-repair
harness validation is claimed.

## Limits and disposition

Labels describe evidenced purpose, not repair adequacy, final quality, complete
test coverage or instruction compliance. No title, file count, desired count,
condition identity or later reviewer approval determines a label. The accepted
ACK turn is the continuation after the acknowledged body, not its generation
turn. Provider turns and assistant items are not equated with completed model
responses. Reasoning bodies are excluded; the native body projection remains
incomplete and perfect blinding is not claimed.

No token totals, condition contrasts, p-values, quality scores or workflow 009+
semantic bodies/outcomes were inspected. This review establishes no treatment
effect, token effect or historical causal share. Complete all-12 classification
and its freeze remain required before either mediator or token contrast CLI.

Only this review document is written by the reviewer. Draft, packet, frozen
helpers/protocol, captures, candidate checkouts and prior reviews remain unchanged.
This offline review affects no executable behavior, architecture or public API;
no implementation documentation update or additional real-agent verification is
required.
