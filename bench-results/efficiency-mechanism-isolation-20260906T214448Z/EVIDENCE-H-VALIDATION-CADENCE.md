# Historical validation cadence: timed work, repairs and rechecks

Scope: the accepted H six Direct and six normal-WL workflows, using only closed public/native evidence. [The source-bound index](EVIDENCE-H-VALIDATION-CADENCE.json) records the 18 Direct initial chains, their complete bounded command windows, all 58 WL mediated author checks, successful-repeat candidates, exact call/item/turn identities and 37 source hashes. No provider, source/runtime change, quality exclusion, parked R analysis, token total or percentage is involved.

## What the evidence establishes

The initial test-first stage is real, but it is not the whole explanation for repeated validation. Every one of the 18 Direct initial author sequences has a captured GREEN for the exact command previously used for its documented initial RED. The four missing-feature-API visual sequences remain compiler-contract RED, not executed behavioral failures. The other 14 author sequences have the behavioral witnesses already reviewed in [H test timing](EVIDENCE-H-TEST-TIMING.md).

Within the bounded initial windows, **14 post-implementation executed-test failure calls lead to test-fixture/assertion repairs**. All their failed test names were introduced by the same author. Complete accepted repair bodies show expected row/title text, missing Esc, `iyes` versus `yes`, or visual-selection inputs/expectations being adjusted. The two additional post-implementation compile failures are different: a new VisualBlock enum variant lacks match arms, and actual production adapters are repaired. None of these later failures is relabeled deliberate pre-implementation RED.

WL also has substantial post-implementation recovery: its complete 58 author command results include 16 behavioral failures, two compile failures and five invalid Cargo invocations. Cohesive delivery does not eliminate repair. One result mixes its own visual-test failure with another feature's completion-test failure; the mixed case stays explicit.

These are action-path explanations, not isolated causal attribution to C15 or a net saving estimate. In particular, the Direct and WL count windows below differ and must not be subtracted.

## Exact initial windows, all 18 authors

An initial window begins at the first accepted new-test patch identified by the pinned timing census. Its cutoff is the last first-GREEN output for that census's initial RED commands. Every leading-`cargo` call issued by that cutoff is retained with its actual output, including a call issued in the same batch whose output arrives after the cutoff. No other exec command mentioning Cargo occurs inside these windows. Later additional test-first increments and post-GREEN checks are outside this bounded semantic classification, not discarded from H.

The 92 included calls are all `cargo test` calls, not formatting or Clippy:

| Actual disposition | Calls |
| --- | ---: |
| Initial executed new-test behavioral RED | 27 |
| Initial missing-feature-API compile RED in the four visual sequences | 5 |
| Additional missing-feature-API compile failure in direct-003 completion | 1 |
| Existing-test new assertion, not a newly introduced test | 1 |
| Invalid initial fixture, missing fixture helper, invalid Cargo invocation | 3 |
| Post-implementation executed GREEN | 39 |
| Post-implementation own-new-test behavioral failure | 14 |
| Post-implementation missing-match-arm compile failure | 2 |

A same-command GREEN reports the initially failed test names passing; it does **not** prove their bodies remained identical. The index preserves intervening accepted patch locators. Several tests evolve before GREEN. The independently reconstructed identical-test example remains the narrower [fixed-artifact test-first replay](EVIDENCE-TEST-FIRST-REPLAY.md), not a property assigned to all these chains.

Each arrow is native tool input → exact same-call output, not a provider response count. N locators resolve to the same per-run/per-feature native source used in the reviewed timing table and the index's `native_source`.

| Direct run / feature | First production patch N | Initial RED N / first exact-command GREEN N |
| --- | --- | --- |
| point7-exact-direct / 1 | 159→161 | 144→146 / 223→225 |
| point7-exact-direct / 2 | 149→151 | 138→141 / 196→199; 139→143 / 197→201 |
| point7-exact-direct / 3 | 188→190 | 160→163 / 238→242; 161→165 / 275→280 |
| direct-003 / 1 | 161→163 | 150→153 / 268→272; 151→155 / 250→254 |
| direct-003 / 2 | 202→204 | 174→176 / 233→237 |
| direct-003 / 3 | 223→225 | 209→213 / 288→292; 210→215 / 289→294 |
| direct-002 / 1 | 208→210 | 188→190 / 334→336; 196→198 / 342→344 |
| direct-002 / 2 | 179→181 | 127→129 / 195→198; 135→137 / 196→200 |
| direct-002 / 3 | 189→191 | 167→169 / 243→245; 173→175 / 277→279 |
| step4-direct-001 / 1 | 148→150 | 121→123 / 262→264 |
| step4-direct-001 / 2 | 233→235 | 211→214 / 249→254; 212→216 / 250→256 |
| step4-direct-001 / 3 | 231→233 | 220→223 / 275→278; 221→225 / 294→296 |
| step4-direct-002 / 1 | 160→162 | 124→126 / 271→273 |
| step4-direct-002 / 2 | 379→381 | 362→367 / 428→433; 363→370 / 429→435; 364→371 / 430→437; 365→373 / 431→439 |
| step4-direct-002 / 3 | 186→188 | 138→141 / 272→275; 139→143 / 307→309 |
| step4-direct-003 / 1 | 155→157 | 142→144 / 286→288 |
| step4-direct-003 / 2 | 128→130 | 120→122 / 144→146 |
| step4-direct-003 / 3 | 279→281 | 240→245 / 301→304; 265→267 / 332→334 |

The explicitly excluded initial cases remain source-linked: point7 feature3 N152→154 is an invalid two-filter command; direct-003 feature2 N142→144 uses the missing-Esc fixture; direct-002 feature3 N161→163 changes an existing test assertion; step4-direct-003 feature3 N241→247 lacks a test helper. Direct-003 feature3 N211→217 is an additional missing proposed harness API, separately retained from its executed workspace/terminal REDs.

## The actual later repair bodies

The following groups cover all 14 executed-test failure calls inside those windows. A repair may address more than one failing check, so neither failure count nor patch count is a response count. There are 14 distinct accepted test-repair patch bodies in this table.

| Direct author | Failed checks N | Accepted repair patches N | Actual repair |
| --- | --- | --- | --- |
| point7 / completion | 239→244; 240→246 | 262→264; 266→268 | Match the actual working-chat row instead of any occurrence of the agent ID in the pane. |
| direct-003 / visual | 226→229; 227→231; 249→252 | 237→239; 245→247; 260→262 | Correct character versus line selection input, expected selection text and compact row identification. N245 edits an inline test inside `src/terminal_app.rs`, not production logic. |
| direct-003 / slash | 232→235 | 243→245 | Insert Esc before the colon slash command in the newly created chat fixture. Its GREEN N251→255 lies just beyond this row's valid-harness initial-window cutoff and is retained as a repair follow-up. |
| direct-002 / visual | 320→322 | 328→330 | Correct the block-selection expected text from the feature label to the actually displayed user row. |
| direct-002 / completion | 249→251; 263→265 | 257→259; 271→273 | Remove the extra insert-mode `i`; identify the complete chat row rather than any agent-ID occurrence. |
| step4-direct-001 / completion | 276→280 | 286→288 | Narrow the hidden/reopened-row assertion to the full feature/agent label. |
| step4-direct-002 / completion | 273→277; 291→293 | 283→285; 299→301 | Assert the actual full left-pane row and then the exact closed row identity. |
| step4-direct-003 / completion | 302→306; 318→320 | 312→314; 326→328 | Inspect the untruncated left-pane representation and the complete hidden/reopened label. |

The separate direct-003 visual N203→206 and N204→208 compile failures name unhandled `HarnessInput::VisualBlock` and `TerminalAppInput::VisualBlock`. Accepted N214→216 and N218→220 add those missing production match arms. Later fixture corrections and their own checks remain separate. “Compile RED” after partial implementation is not the initial absent-feature contract.

These fixture repairs demonstrate actual additional validation/recovery work. They do not establish that every changed assertion was the best test or that a different test-design policy would necessarily prevent the error. The complete initial RED→GREEN chain often combines implementation and test-design evolution; no automatic precedence or quality-equivalence claim is introduced.

## All WL checks remain after accepted implementation

The index independently rechecks every existing S-command → C-result → typed S-reply identity against the raw captures and the prior command input hashes. Every one has an earlier accepted same-author implementation group. The nearest own ACK and intervening other-author ACKs remain distinct; an ACK is a submitted-work witness, not a complete byte snapshot at command execution.

| Whole H WL author result population | Results |
| --- | ---: |
| Executed passing tests | 35 |
| Executed behavioral test failures | 16 |
| Compile failures | 2 |
| Invalid Cargo invocations | 5 |

Seven passing results, ten behavioral failures, one compile failure and one invalid invocation occur while only the first own ACK has been accepted; later own groups account for the rest. This is accepted-group order, not an inferred initial/review-stage classification.

Fifteen behavioral-result rows name only tests added by the same author. H006 C87 names both the visual author's own new test and the completion author's test; only the own visual name belongs to its earlier accepted additions. Name membership identifies an owned test, **not** whether implementation or expected output is at fault.

Concrete retained counterparts are H004 S3478→C24/S3484 (cohesive slash implementation/tests), C26's failing negative display assertion, S3681→C28/S3687 (test repair), and C30/S3722 GREEN. H002 completion C44 and C48 fail successive assertions before C52 passes. H002 visual C65 is the reviewed no-new-test initial group, and its later new visual tests are not silently invented at C65. These cases prevent successful-only or “WL never repairs tests” conclusions.

## Successful repeats: do not equate no patch with unchanged source

Across the **complete** 18 Direct author native sessions, 14 successive successful occurrences of identical leading-`cargo test` strings have no accepted `apply_patch` between them. Thirteen pairs nevertheless contain a write-capable `cargo fmt` or `rustfmt` command. Even a failed formatter can have affected files; these pairs are source-state unresolved, not proven redundant checks.

The sole remaining candidate is step4-direct-002 slash N428→433 and N448→456, both `cargo test --test cli command_chat_routes_agent_slash_command_to_selected_agent -- --nocapture`, in the same exact outer author turn. The six intervening native actions are other focused Cargo tests: N429/430/431 and N445/446/447. No intervening source-edit or formatter action is recorded. This is a **public-action-stable repeated successful check**, not proof of byte-identical tested trees: there is no complete source-hash boundary around both commands, and possible native/test-side effects must not be inferred absent from command spelling. The index deliberately sets `byte_identical_source_state_proven: false` for every candidate.

This exact sample is more informative than the old same-string repeat count, but it does not establish that the other repeated checks were unnecessary or classify all later author work.

## Whole-author Cargo breakdown, without conflating operations

The complete H native author census's 501 leading-`cargo` calls splits by the **first** Cargo subcommand:

| Actual author phase | Test | Fmt | Clippy | Check | Other | Native calls |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Initial author turn, including work after the bounded first-GREEN window | 288 | 36 | 15 | 0 | 0 | 339 |
| Later same-author fix turns | 132 | 20 | 10 | 0 | 0 | 162 |
| Total | 420 | 56 | 25 | 0 | 0 | 501 |

These are 18 initial and 21 actual later author turns, joined by explicit native turn identity and the prior saved driver-role audit. `rustfmt`-leading commands are outside the 501 and are not falsely counted as Cargo calls. The 92 initial-window test calls are a subset of the 288 first-author-turn test-leading calls; the other 196 are retained but not fully purpose-classified here.

Thirteen of the 501 commands contain multiple `&&`-linked Cargo invocations: six are test-leading and seven fmt-leading. Consequently, 420 is **not** the count of all executed Cargo test subprocesses. The index retains every compound string and its exact output. Two compounds return 101, so later `&&` clauses must not be credited with execution merely because they occur in the command text. No compound is expanded into assumed provider calls or repeated-response charges.

## Disposition and bounded next check

The unresolved C15/cadence space is narrower: initial executed RED, fixture-construction repair, partial-implementation compiler repair, formatter-separated rechecking, and one source-unproven repeated successful check are different mechanisms. Broad all-target validation is not supported as the common explanation. Normal WL's buildable shared-tree translation remains associated with omitting the initial published-RED stage; the fixed-artifact replay verifies one real behavioral dependency without changing the shared tree or adding a model observation.

The concrete next **offline** check is the already retained post-first-GREEN author activity, prioritizing complete per-feature source-state/formatter and actual failure-result witnesses rather than another broad command count. Any stronger “same source, same passing check” claim requires complete before/after source identity or a faithfully reconstructed pinned tree; this audit does not supply one. No further control, runtime intervention, replacement or provider admission follows automatically. Private test-first isolation remains subject to [its explicit shared-tree safety design](DESIGN-TEST-FIRST-ISOLATION.md).

## Provenance and exclusions

Index SHA-256: `c9f7b02c439843d1d76b3782bea4c437f84db25acfc014917b11b19d80ce6918`. All 18 selected Direct native files and all 12 WL raw streams match their earlier accepted pins; the seven source/protocol/evidence references bring the endpoint inventory to 37, with no drift. The index contains no private reasoning or usage-counter fields.

The historical source/CLI distinction remains [H's qualification](EVIDENCE-H-LIFECYCLE.md): actual normal-WL source `5b1d1ef…`, WL CLI0.150.1, Direct CLI0.149.1 for the first three and0.150.1 for the step4 three. Newer P/S/W metadata and response IDs are not retroactively assigned. Original outcomes and the accepted historical token endpoint are untouched.

This is evidence-only documentation: no agent-facing code or public workflow is changed, so no real-agent generation is required or performed.
