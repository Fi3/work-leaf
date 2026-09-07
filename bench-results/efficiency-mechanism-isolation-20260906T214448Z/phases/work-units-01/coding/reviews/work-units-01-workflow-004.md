# Independent ACK classification review: workflow 004

Reviewer: `/root/candidate_controls`. Scope: terminally published
`work-units-01-workflow-004` only, in ACK order under
`preflight/ACK-CLASSIFICATION-PROCEDURE.md`.

Disposition: accepted with no findings or classification disagreements. All seven
ACKs retain one supported label; no mixed-purpose or competing-group uncertainty
requiring `unresolved` was identified in the inspected evidence.

## Reviewed identities and evidence coverage

- Phase manifest SHA256: `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
- Draft: `coding/drafts/work-units-01-workflow-004.json`, SHA256
  `0787fe815c86504b0ad6fc1259c43678200e513c8d4fd75eabd8ccabe1ca5937`.
- Packet: `coding/packets/work-units-01-workflow-004.json`, SHA256
  `48bfda978756f39057a4b05ed71bea09330e6a1e148bfbacad840583e98f0e50`.
- Terminal receipt: `logs/work-units-01-workflow-004.exit.json`, SHA256
  `a0e7cc78c90a1dfc483c5f965e25c858b4ccdac12a3b5f046e61f165c4df13bd`;
  completed with launcher exit code 0 at `2026-09-07T02:41:02.382116+00:00`.

All packet source hashes and all 70 draft evidence entries across nine unique
canonical phase-local sources passed hash revalidation, including end-of-review
checks. Every cited passage and physical source locator, including compound
references, was checked for its stated evidentiary role. The mechanical check
covered 61 distinct explicitly numbered JSONL references, first JSON pointers,
string-typed RPC values and stated thread/turn/item identities; manually inspected
compound references include the additional bundle deliveries and rejected edits.
All seven complete successful edit bodies and original task requests were inspected
before the draft review, as were relevant prior snapshots and repair triggers.

Frozen pure `capture_provenance` and `prompt_inventory` replay passed: no provenance
or inventory errors, complete bidirectional delivery coverage, seven ACKs in exact
packet order and stable source hashes. This replay checked 18 source identities
and did not invoke a whole-phase inference path. Each accepted ACK has an exact
string-typed request/reply and a preceding accepted same-thread turn containing
one complete structured edit and one matching interrupt. The edit's file set equals
the ACK receipt. The accepted ACK turn is the continuation, not the edit it acknowledges.

The table uses physical JSONL lines under
`runs/work-units-01-workflow-004/work-units-01-workflow-004-three-feature-bench-artifacts/observation/app-server/00000446260578462469-3719532/`.
`C` denotes `client-to-server.raw`, SHA256
`3a4cca10e1f1a24d139a4bbb2c1e2fae7e8a8b06267900a7e0a08cdc2130e4ff`;
`S` denotes `server-to-client.raw`, SHA256
`2fca9c847c064306ac995712f483b67448966ba9b3e78c82a8ac93949288725f`.
Edit text is at `/params/item/text`, request text at `/params/input/0/text`, and
accepted turn identity at `/result/turn/id`. Exact ACK/item/thread/turn IDs remain
in the hash-bound draft and packet; ordinals below preserve their exact ACK order.

## Per-ACK semantic review

| ACK | Accepted RPC and request/reply | Preceding edit / interrupt | Label and supported purpose |
| --- | --- | --- | --- |
| 1 | string `25`, C26/S4842 | S4835 / C25 | `remaining-work`: C6 requests selected-backend slash routing. Bundle 3's unconditional terminal command handler and bundle 1's harness command parser lack that route. The complete group implements routing and dependent tests. C24 rejects the earlier hunk rather than establishing accepted prior work. |
| 2 | string `36`, C37/S10318 | S10311 / C36 | `remaining-work`: C8 requests review-completion highlight, yes/no, close and reopen. Bundle 2's controller review-completion branch lacks that state/prompt. The group introduces the coherent controller/terminal/harness implementation and tests. Its readiness-test replacement is part of that initial attempt; later defects do not retroactively make this a repair. |
| 3 | string `45`, C46/S12918 | S12909 / C45 | `validation-repair`: C39 reports status 101 for two prompt-history failures and the new completion test's missing bell. Bundle 8 shows review auto-activating chat and the selected parser already ready. Removing that transition and selecting the non-ready fixture agent repair these demonstrated failures. |
| 4 | string `49`, C50/S14124 | S14116 / C49 | `validation-repair`: C48 reports the remaining global READY assertion failure. The previous edit selects user-2, while bundle 8's parser remains independently ready. The entire group narrows the existing assertion to the completed agent; no new feature work is present. |
| 5 | string `62`, C63/S32635 | S32626 / C62 | `review-repair`: C61 identifies duplicate completion questions being dropped by the terminal cache. Bundle 8 retains unconditional line deduplication; the accepted initial controller already includes reopening and repeated markers. Marker exemption and a repeated question/closed/question test repair that lifecycle defect. |
| 6 | string `85`, C86/S52062 | S52051 / C85 | `review-repair`: C72 identifies ordinary idle-reply readiness suppressed by the initial pending-question projection. The group restores idle readiness while respecting marked-done state, adds a dependent state helper/regression and adjusts the question test's initial state. These repair existing behavior, not independently remaining requested work. |
| 7 | string `97`, C98/S70339 | S70332 / C97 | `remaining-work`: C4 requests both-pane visual selection and yanks. Bundle 0 lacks selection state/dispatch. The complete group introduces modes, movement, pane geometry, highlighting, clipboard output, terminal/harness routing and three tests as one dependent implementation. Earlier rejected candidates are not an accepted visual implementation. |

## Prior-state, rejection and continuation checks

The draft's exact prior-state passages were inspected: bundle 3 lines 2974–2978
and bundle 1 lines 3290–3334 for slash routing; bundle 2 lines 3773–3794 for
completion; bundle 8's review mode transition, fixture readiness, original test,
cache deduplication and readiness projection; and bundle 0's initial UI state and
complete key dispatcher. Earlier accepted edits supply the intervening state when
there is no fresh complete snapshot. No unseen checkpoint or commit-to-ACK join
is required or claimed.

C24 rejects a slash hunk. C60/C71/C78/C96 reject four visual candidates for repeated
file headers or absent old blocks. C74/C80/C82/C84 reject four completion-repair
candidates for ambiguous old blocks. C76 reports unchanged terminal digests;
the full C71/C78/C96 refreshes contain accepted slash/completion changes rather
than an earlier accepted visual implementation. Retained `src/patch.rs` declares
`GitPatcher` at lines 54/59 and computes all structured changes before writes at
lines 150–168. These matching failures are not partially applied successful groups.
Rejected submissions remain distinct from successful ACKs and no-op/replayed work.

Post-ACK continuation commands, returned statuses and subsequent public actions
were checked separately from preceding edit identity. S4879/C28/S4897 is the
slash check/result/done chain. S10353/C39/S10393 is the initial completion check,
failure and mediated reread; S13160/C48/S14116 leads to the next assertion repair.
S14282/C52/S14403 is the passing harness/workspace check and done.
S32823/C65/S33049 exercises the named repeated-marker regression and then done;
the other selected test binaries run zero filtered tests, not additional coverage.
S52311/C88 is an invalid multiple-filter Cargo invocation with status 1 and empty
stdout. S52636/C90/S52788 is the subsequent valid terminal-test run with status 0
and done. S70364/C100/S70382 is the visual harness check/result/done chain.
The invalid invocation is not evidence of a failed runtime assertion or a passing
test. Later passing checks corroborate continuation, never establish an earlier
repair trigger by themselves.

## Limits and disposition

Repair labels concern purpose, not adequacy or final quality. In particular, the
initial implementation's replaced readiness behavior and the dependent restoration
are not treated as separate newly requested features. No title, file count, later
NO_FINDINGS response or desired number of units determines a label.

The driver has no separate typed `PatchApplied`-to-commit event log. Unique complete
edit, same-thread accepted sequence, interrupt, receipt files and retained source
control flow support the groups without fabricated commit joins. An assistant item
or accepted turn is not a completed model response. Private reasoning was excluded;
native body projection is incomplete and perfect blinding is not claimed.

No token totals, condition contrasts, p-values, quality scores or workflow 005+
outcomes were inspected. This review establishes no treatment effect, token effect
or historical causal share. Complete all-12 classification and its freeze remain
required before either mediator or token contrast CLI.

Only this review document is written by the reviewer. Drafts, frozen helpers and
protocol, captures, candidate checkouts and previously reviewed artifacts remain
unchanged. This offline review affects no executable or real-agent workflow and
requires no additional provider verification.
