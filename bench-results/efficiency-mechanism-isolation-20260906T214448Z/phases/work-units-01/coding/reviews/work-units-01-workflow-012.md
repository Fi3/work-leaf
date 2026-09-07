# Workflow 012 independent ACK classification review

Result: accepted without findings. All nine ordered labels agree with the
independent full-body, prior-state and actual-trigger review. No unresolved
mixed-purpose component was identified. This is classification review, not a
quality verdict or a token/condition comparison.

## Scope and immutable identities

The reviewed draft is `coding/drafts/work-units-01-workflow-012.json`, SHA-256
`2820cdba9cd3bd1ba168d314cb6804f45afa6c3c6ff2d7294965ebac109a6427`.
The existing packet is `coding/packets/work-units-01-workflow-012.json`, SHA-256
`911cce32fa464747d0eb64a713d8ed39c0e316f8727fffe07b64fc9efb96aff2`.
Both are relative to this review's phase directory, `phases/work-units-01`.

The phase manifest identity is
`5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
The per-run terminal receipt is
`logs/work-units-01-workflow-012.exit.json`, SHA-256
`f6f748f867ee005ca047f30e66ddb58656df2a288f9ac8fc1e9b7553db1ac921`:
completed, exit 0, finished `2026-09-07T04:12:17.451958+00:00`.

The frozen `preflight/ACK-CLASSIFICATION-PROCEDURE.md` governs the labels.
The packet was not regenerated. No provider, benchmark, aggregate analysis,
token/quality result or condition contrast was used in this review.

## Evidence identity and coverage

All 265 packet source identities pass SHA-256 verification. All 155 draft
evidence entries, covering ten unique cited files, pass canonical phase-local
path and hash checks. All 164 explicit physical JSONL references, representing
135 distinct locations, resolve. JSON pointers, typed RPC string IDs, native
public message IDs, thread IDs and turn IDs agree with their cited records,
including explicitly named opposite-stream references.

For each ACK, the forwarded trace bytes equal the actual accepted request. Its
preceding accepted same-thread turn contains one unique complete structured
edit; the exact receipt file set matches that body. The matching interrupt
precedes the ACK request. All nine complete-body line counts match the draft.
The ACK turn is the subsequent continuation, not the edit-generation turn.
This establishes the source-grounded processed-group witness; it does not
invent a typed PatchApplied-to-commit join.

Here `C` and `S` mean physical JSONL lines in the client and server raw streams
under the run's artifact directory:
`observation/app-server/00000452179765332046-4005151/`.
The client stream SHA-256 is
`7fd19919bc8e25cf5e31d6605eb06029e11e3610552fdc7bd3861d71dbd6640b`;
the server stream SHA-256 is
`2b8abf5678ef584aa2d053bdb2d5bf597901fb851054e546856c65d4bfc96493`.
The full phase-local paths, hashes and typed IDs are retained in every draft
evidence entry. `T` is a physical line in
`prompt-events/work-units-01-workflow-012.jsonl`, SHA-256
`126a90960dcf6a208f5618d8fd17c8edf440bd5e4aff1848bf7ec4d7c851f464`.

| ACK | Agent | Prior input / complete edit / interrupt | Accepted ACK request / reply / trace | Label |
| --- | --- | --- | --- | --- |
| 1 | user-2 | C18 / S3472 / C21 | C22 / S3479 / T5 | remaining-work |
| 2 | user-1 | C16 / S16827 / C39 | C40 / S16834 / T9 | remaining-work |
| 3 | user-2 | C50 / S22260 / C51 | C52 / S22267 / T12 | review-repair |
| 4 | user-3 | C64 / S30821 / C66 | C67 / S30828 / T15 | remaining-work |
| 5 | user-1 | C69 / S31929 / C74 | C75 / S31936 / T18 | review-repair |
| 6 | user-3 | C73 / S32303 / C78 | C79 / S32310 / T20 | validation-repair |
| 7 | user-3 | C89 / S32741 / C90 | C91 / S32748 / T24 | validation-repair |
| 8 | user-3 | C104 / S34856 / C105 | C106 / S34863 / T28 | review-repair |
| 9 | user-3 | C111 / S35691 / C112 | C113 / S35698 / T30 | review-repair |

## Independent semantic checks

1. **Initial slash disambiguation.** C6 requests slash followed by nonwhitespace
   to route to the selected backend. Delivered bundle 4, lines 2836–2842 and
   3088–3100, already routes immediately on slash. S1598/S1978 identify and plan
   the missing next-character check. The complete 184-line S3472 introduces
   pending state in both adapters and dependent whitespace-negative tests;
   it preserves the existing positive route. This is initial requested work,
   not a repair of an earlier accepted group from this agent. S3500 requests
   `cargo test slash`; C24 passes one invocation unit, three terminal tests and
   two harness tests; S3518 is done. This does not prove real-provider routing
   or a new command-prompt route.

2. **Initial visual selection.** C4 and the baseline bundle 0 establish the
   requested missing visual selection. The complete 864-line S16827 supplies
   private character/line/block state, both-pane coordinates, movement,
   highlight/copy and OSC52, adapter guards/Ctrl-V and three harness tests.
   S1800 explicitly preserves the public mode enum. S16865 requests the full
   harness target; C42 passes all 32 then-present tests and S16883 is done.
   C65 subsequently exposes an input-decoding defect; that limits adequacy,
   without converting this initial implementation into repair.

3. **Slash tab repair.** C36 delivers the concrete raw-tab finding. Bundle 7,
   lines 485–489, 808–821, 1505–1509 and 1819–1831, confirms that an unmapped
   byte can be discarded before pending-state handling. S22260's full
   153-line group cancels raw whitespace before mapping and adds two tab
   regressions. C50 rejects S19208 after concurrent visual context changes;
   rejected work is not an accepted replay. S22400/C54 is an invalid command
   with no tests. S22878/C56 is the corrected slash filter, passing one unit,
   four terminal and three harness tests; S23177 is done.

4. **Initial completion flow.** C8 and bundle 2, lines 4397–4410, establish
   the missing post-review question/local-answer handling. The full 556-line
   S30821 introduces controller state, local yes/no behavior, UI hide/show,
   terminal/harness paths and tests. C33 and C62 reject earlier candidates
   S13777 and S24825; C35/C64 provide concurrent source refreshes. They do not
   establish previously accepted completion work. S30915/C71 is invalid;
   S30980/C73 runs the corrected filter, passing the terminal unit but failing
   both own harness assertions before reaching workspace tests. The tests
   cover typing while pending or after no, not demonstrated typing after yes.

5. **Visual bytewise-arrow repair.** C65's finding is supported by bundle 9,
   lines 464–483, 880–890 and 1972–1982: immediate ESC exits visual state before
   a completed arrow sequence. S31929's full 106-line group extends both
   defer-escape predicates and supplies bytewise terminal/harness regressions.
   S31986/C77 is invalid, not validation. S32362/C81 runs and passes the two
   intended regressions; S32394 is done. No independent feature increment is
   mixed into the repair.

6. **Harness assertion repair.** C73 names the exact two failures. The full
   initial test bodies at S30821 lines 384–441 and bundle 9 lines 3488–3543
   show assertion order: yes has already deselected the target before the
   broad `user-2` absence check; typing preserves the target and its row before
   broad `READY` absence fails. Bundle 9 lines 2135–2151 establishes the other
   ready parser row and its metadata references to user-2. The entire
   37-line S32303 only scopes assertions to the intended row. S32374/C83
   selects zero tests with `--exact`; S32465/C89 passes both repaired harness
   tests but then fails the workspace test. These are separate events.

7. **Workspace assertion repair.** C89 fails the all-historic-sends assertion
   after question, local yes and marked-done assertions have passed. The
   original complete test is S30821 lines 462–502. S32527 explains the intended
   no-additional-send check; it is not an exact inventory of past send causes.
   The full 29-line S32741 snapshots the pre-yes count and compares it after
   yes, with no controller change. S32767/C93 passes one terminal, two harness
   and one controller test; S32785 is done. The differently named no/follow-up
   controller test is not selected by `feature_done`.

8. **Late-output review repair.** C102's finding and delivered bundle 12
   lines 653–690, 1073–1078 and 1752–1766 establish separate controller pending
   state and terminal last-line-only detection. The entire 94-line S34856 is
   terminal-only: backward boundary scanning and a synthetic late-output test.
   No broader controller work is inferred from planning. S34882/C108 passes
   two terminal, two harness and one controller test; S34900 is done. C111
   then identifies the introduced generic `> ` delimiter false match.
   Passing the first focused check does not establish complete repair.

9. **Quoted-output review repair.** C111 explicitly targets the incomplete
   preceding repair. S34856 body lines 14–41 contains the overbroad alternative;
   bundle 12 lines 1752–1766 records genuine session user lines as `user: `.
   The full 61-line S35691 removes the generic alternative and adds a quoted
   late-output terminal regression. S35717/C115 passes three terminal, two
   harness and one controller test; S35735 is done. C120 explicitly reports
   `NO_FINDINGS` despite the wrapper's generic issues header, and S35918 is
   done without another edit. No synthetic ACK is inferred from that follow-up.

Every cited bundle range and public-message locator was inspected. The
retained `GitPatcher::apply_edit`/`apply_edit_with_locks` source at
`infrastructure/evidence/src/patch.rs`, lines 77–87 and 150–168, confirms
complete computation before any write for these hunk failures. Its SHA-256
is `aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda`.

## Preservation and limits

All 24 expected packet/draft identities for workflows 001–011 pass: eleven
packets, eleven original drafts and both retained revision-01 drafts. All 29
tracked prior coding artifacts also match HEAD byte-for-byte, including
correction notes, reviews and uppercase operational observations. The
workflow 012 packet, draft, source hashes and terminal receipt remain intact.

Purpose is separate from correctness, instruction compliance and quality.
The cross-agent warning does not establish that another agent's tests failed.
Synthetic/fake-backend checks are not real-provider or full end-to-end proof.
Native activity is not assumed absent because of a public-only projection;
reasoning bodies are excluded and perfect blinding is not claimed.

This review changes no executable code or agent-facing workflow. No new
real-agent verification is applicable to this evidence-only artifact. The
existing frozen protocol, architecture and operator documentation require no
update for this review. The draft remains immutable; root owns its acceptance
and the separately reviewed assembly before any condition contrast.
