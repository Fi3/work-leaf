# Independent ACK classification review: workflow 003

Reviewer: `/root/candidate_controls`. Scope: terminally published
`work-units-01-workflow-003` only, in ACK order under
`preflight/ACK-CLASSIFICATION-PROCEDURE.md`.

Disposition: revision 01 is accepted with no remaining findings and no classification
disagreements. All 15 ACKs retain one supported label. One source-owner naming error
in three evidence locators is resolved by a separately retained revision; no label,
rationale, source hash, line number or ACK identity differs from the original.

## Reviewed identities and resolved finding

- Phase manifest SHA256: `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
- Reviewed current draft: `coding/drafts/work-units-01-workflow-003.revision-01.json`,
  SHA256 `340d3df991be717e31bea8e6232dce904d63b8419e55a9d4f20701f4ba06322c`.
- Preserved original: `coding/drafts/work-units-01-workflow-003.json`, SHA256
  `11c3d3b46cdb5590d0ad6ab10e00846acfdb9505a24f0fad13c10fe07da61e26`.
- Packet: `coding/packets/work-units-01-workflow-003.json`, SHA256
  `328d15eea23d74a770aea571c7003f00d4d04b72065947c4d5996b78ccf892ce`.
- Terminal receipt: `logs/work-units-01-workflow-003.exit.json`, SHA256
  `3a2ae4f72aca03934205dfa3a7b55539a13bb149bb3386e0a139576013b34d22`;
  completed with launcher exit code 0.

The original ACK 2, 3 and 9 patch-source locators named nonexistent
`PatchApplier::apply_edit_with_locks`. The retained source declares `GitPatcher`
at line 54, its implementation at line 59, and the method at line 150. Independent
byte comparison verifies that revision 01 equals the original with exactly those
three owner-name substitutions to `GitPatcher::apply_edit_with_locks`; every other
byte is identical. The accompanying revision note preserves the reason and identities.

## Evidence and delivery coverage

All packet source hashes and 137 draft evidence entries across 17 unique canonical,
phase-local sources were revalidated. Every cited source passage and raw physical
location, including compound references, was checked against its stated role. The
complete successful edit bodies and original feature requests were inspected before
the draft was available; subsequent draft review agrees with that independent
assessment. Private reasoning was excluded.

Frozen pure `capture_provenance` and `prompt_inventory` replay passed for this
workflow: provenance verified, bidirectional inventory available with no errors,
15 ACKs in exact packet order, stable source hashes. No whole-phase analysis path
was invoked. Every ACK's string-typed RPC request/reply identifies its accepted
thread/turn; its preceding accepted same-thread turn contains exactly one complete
structured-edit group and one interrupt, with the exact ACK receipt file set.
No global timestamp, commit message or closest public item substitutes for this chain.

The table uses physical raw JSONL lines under
`runs/work-units-01-workflow-003/work-units-01-workflow-003-three-feature-bench-artifacts/observation/app-server/00000441344929587964-3537091/`,
the unchanged packet's canonical per-ACK `capture` directory. `C` denotes
`client-to-server.raw`, SHA256
`191f6d8611d77005d77728462a2cb0a5a1dfcecf8347ecc9fb5d0b6e24e3e025`;
`S` denotes `server-to-client.raw`, SHA256
`2f6949270f63a69423eee95ee0f7f866169c04cbffc027c9926862cca6909009`.
Bodies are at `/params/item/text`, request text at `/params/input/0/text`, accepted
turns at `/result/turn/id`. Full item/thread/turn identities and exact ACK IDs are
retained in the hash-bound draft and packet; table ordinals are that exact order.

## Per-ACK semantic review

| ACK ordinal | Accepted RPC and request/reply | Preceding edit / interrupt | Label and evidence-based purpose |
| --- | --- | --- | --- |
| 1 | string `23`, C24/S3281 | S3274 / C23 | `remaining-work`: C6 requests selected-backend slash routing. Bundle 4's terminal command handler and bundle 2's harness prompt parser lack that route. Implementation and dependent tests fill this requested gap. |
| 2 | string `43`, C44/S34362 | S34354 / C43 | `remaining-work`: C10 requests completion highlight/yes-no/close/reopen. Bundle 1's review-completion branch lacks confirmation state/prompt. The group implements that coherent feature; the earlier hunk rejection is not accepted prior work. |
| 3 | string `64`, C65/S57575 | S57568 / C64 | `remaining-work`: C4 requests visual selection/yanks. Bundle 0 lacks visual state/handlers. The complete group supplies modes, movement, rendering, clipboard integration and tests. Four earlier hunk rejections do not make this first accepted visual group replay or repair. |
| 4 | string `79`, C80/S60605 | S60599 / C79 | `review-repair`: C74 identifies logical-row versus rendered-wrap inconsistency in accepted selection. Bundle 10 corroborates that mapping. The edit introduces wrapped selection coordinates and a regression for that defect. |
| 5 | string `90`, C91/S61668 | S61659 / C90 | `review-repair`: C87 identifies exact-width row-count mismatch. Bundle 11 contains the conflicting count and chunk routines. The edit aligns existing cursor/selection counting and tests the edge case. |
| 6 | string `99`, C100/S62382 | S62375 / C99 | `review-repair`: C96 identifies character chunks versus word wrapping. Bundle 12 contains that implementation. Cursor-prefix mapping, wrapping and the new regression address the existing defect, not a new mode. |
| 7 | string `103`, C104/S62576 | S62569 / C103 | `validation-repair`: C102 reports status 101 and the existing wrapped-copy assertion's actual/expected mismatch. The entire edit changes that expectation; it adds no feature work. |
| 8 | string `112`, C113/S63295 | S63287 / C112 | `review-repair`: C109 requests wrap-whitespace consistency. Bundle 13 retains the custom wrapper. Renderer-backed row extraction and a whitespace regression address the accepted feature's copy/highlight behavior. |
| 9 | string `120`, C121/S64507 | S64500 / C120 | `validation-repair`: C115 reports three failures involving wrapped selection and lost prompt spaces. Bundle 14 retains prefix anchoring and trimmed rendered rows. Suffix anchoring/prompt-space handling repair those failures, though a later test still fails. |
| 10 | string `126`, C127/S65295 | S65288 / C126 | `validation-repair`: C123 reports the remaining wrapped-copy assertion failure. The entire edit raises the test viewport height from 10 to 14. C132 later identifies this as avoiding the production defect; inadequate repair remains repair-purpose evidence. |
| 11 | string `133`, C134/S65893 | S65886 / C133 | `review-repair`: C132 identifies clipped standalone prompt rendering and the masking viewport change. Bundle 15 and prior edit establish this state. Unclipped scratch rows and restoration of the small viewport repair that defect. |
| 12 | string `139`, C140/S66890 | S66883 / C139 | `validation-repair`: C136 reports continued wrapped-copy failure; bundle 16 retains the old history/prompt clipping. Visible-tail handling and a too-tall-prompt regression address accepted work's demonstrated failure. |
| 13 | string `150`, C151/S70299 | S70292 / C150 | `review-repair`: C145 identifies a normal-cursor regression caused by the previous visible-tail repair. Bundle 17 corroborates the old normal cursor and wrapped-tail paths. Passing full/visible content through cursor mapping and a regression repair that existing behavior. |
| 14 | string `158`, C159/S70800 | S70793 / C158 | `validation-repair`: the valid harness result C155 fails the new column-20 assertion. The edit changes only the expected column to 19; C161 still fails. Intervening shell error C157 is not runtime cursor evidence. |
| 15 | string `164`, C165/S71141 | S71134 / C164 | `validation-repair`: C161 and bundle 18 establish the failed column-19 assertion. The entire edit changes its expectation to 14. C167 passes, without turning the test-only repair into new requested functionality. |

## Prior-state, rejection and continuation checks

Retained prior-state passages include bundle 4 lines 2922–2930 and bundle 2 lines
3315–3361 for slash routing; bundle 1 lines 3055–3079 for completion; bundle 0's
initial UI state/dispatcher; and bundles 10–18 for successive accepted visual
implementations and tests. The reviewed draft gives their exact paths/hashes and
passage ranges. For steps without a fresh complete snapshot, the earlier accepted
edit plus the next delivered failure/review establishes the specific prior test or
implementation state; no unseen checkpoint is invented.

C42 rejects the initial completion candidate. C38/C40/C54/C63 reject the initial
visual candidates for absent old blocks. Later C78/C149 reject repeated file headers,
and C119 rejects an absent old block. These are rejected submissions, not successful
ACKs. Retained `infrastructure/evidence/src/patch.rs`, SHA256
`aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda`,
lines 150–168, computes all structured changes before the write loop for these
hunk failures. Prior refreshes preserve the accepted slash/completion work; no
competing accepted visual group is substituted into the initial classification.

The public post-ACK command/result/next-action chains were checked independently
from the preceding edits. Every listed next command belongs to its accepted ACK
turn; returned status/stdout/stderr are captured follow-up inputs. Review-trigger
labels use actual delivered findings, and validation-trigger labels use actual
preceding failures, not a later passing check. C153's malformed two-filter Cargo
invocation exits 1 before exercising tests; C155's valid harness run is the failure
supporting ACK 14. C157 exits 2 for an unmatched shell quote with empty stdout;
it supplies no successful probe result. The later failed expectation changes and
the viewport-masking attempt remain visible instead of being dropped or relabeled
as substantive work. C170's later public NO_FINDINGS is not used to retroactively
establish the quality or correctness of any earlier repair.

## Limits and disposition

No mixed repair-plus-independent-requested-work group or unresolved competing edit
candidate is supported by the inspected evidence. Repair labels describe the
documented purpose; they do not endorse every reviewer claim, certify the code or
tests, or require a repair to succeed. In particular, broad implementation changes
needed to repair wrapped selection and its induced ordinary-cursor regression are
not additional originally absent visual modes or independent user-requested work.

The driver lacks a separate typed `PatchApplied`-to-commit log. The exact accepted
same-thread chain, unique complete edit, interrupt, receipt files and source control
flow support each group without manufacturing commit joins. An assistant item or
accepted turn is not counted as a completed model response. Native body projection
is incomplete; perfect blinding and absence of native actions are not claimed.

No token totals, condition contrasts, p-values, quality scores or active workflow
004+ outcomes were inspected. This review establishes no treatment effect, token
effect or historical causal share. The final phase classification still requires
complete all-12 coverage and freezing before either mediator or token contrast CLI.

Only this review document is written by the reviewer. Draft revisions are retained
as separate coder artifacts; frozen helpers/protocol, captures, candidate checkouts
and previously reviewed drafts remain unchanged. This offline review affects no
executable or real-agent workflow and requires no additional provider verification.
