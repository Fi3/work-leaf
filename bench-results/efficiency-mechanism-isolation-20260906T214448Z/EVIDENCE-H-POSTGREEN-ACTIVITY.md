# H author work after the initial GREEN

The first passing new-feature checks are not the end of implementation. The retained Direct authors subsequently add regression coverage, repair production behavior, correct test fixtures, run scoped checks and formatting, and respond to review. Twenty of the 21 later author fix turns contain an executed failing test. The remaining fix turn adds missing `no`-answer coverage that passes immediately, without a production patch. This is actual additional work, not simply a count of repeated passing commands.

Scope: all 18 author sessions and all 21 later author fix turns in the accepted historical H six Direct workflows. The [source-bound index](EVIDENCE-H-POSTGREEN-ACTIVITY.json) retains every test-leading call outside the prior initial windows, every postcutoff validation command and patch attempt, exact command/output hashes, public review-fix input identities, failure episodes, and 23 source hashes. The prior [cadence report](EVIDENCE-H-VALIDATION-CADENCE.md), all normal-WL outcomes, historical token endpoint and frozen runtime remain unchanged. No provider, token analysis, quality exclusion or new observation is involved.

## Complete population and corrected window boundary

The prior cadence identifies 92 initial-window test-leading calls out of 420 whole-author test-leading calls. Its remaining 196 initial-turn calls are **192 actually after the initial GREEN plus four earlier baseline checks**. The 132 later-fix calls are separate. Exact same-thread call-ID sets prove the partition `92 + 4 + 192 + 132 = 420`, without omissions or overlap.

The four earlier calls are direct-003 slash N96→98 and step4-direct-002 slash N132→139, N133→140 and N137→142. All pass; none is post-GREEN. N locators throughout this document are one-based physical native input → matching output lines, resolved through the index's per-author `native_source`. They are not response IDs or provider call counts.

| Disposition of the 328 retained remainder test-leading commands | Calls |
| --- | ---: |
| Passing scoped/confirmation checks after GREEN or in fix turns | 276 |
| Earlier passing baseline checks | 4 |
| Newly introduced regression-test RED before its associated repair | 25 |
| Modified existing-test regression RED | 4 |
| Mixed newly introduced and modified-test RED | 2 |
| Executed failure after partial implementation, followed by production repair | 4 |
| Test-fixture or expected-output repair | 5 |
| Removed/replaced failing reproduction, not repaired unchanged | 2 |
| Existing-suite failure followed by passing reruns without an intervening accepted patch | 4 |
| Invalid Cargo invocation, no tests executed | 2 |

These are mutually exclusive command dispositions, not numbers of tests or independent causes. Forty source-linked episodes cover all 48 nonzero test-leading commands **and one additional fmt-leading compound failure**. Several commands fail multiple tests; a single repair can address multiple commands. A printed test name passing later does not prove identical test bodies.

All 56 fmt-leading and 25 Clippy-leading Cargo commands fall after these initial cutoffs, as do 26 `rustfmt`-leading commands. They are retained separately. All 13 `&&` Cargo compounds remain exact; two return 101. The failed fmt-leading step4-direct-002 visual N556→558 reaches a large-output terminal assertion. The failed test-leading N786→788 reaches a missing-file-followup assertion. Neither author is credited with executing the later `&&` clauses after failure. The command counts must not be expanded into assumed subprocesses or model responses.

## Every author and later fix turn

The counts below are post-GREEN initial / later-fix **test-leading** calls. Formatting-leading compounds remain in the index rather than being silently added to this denominator. The two earlier-baseline authors are marked explicitly.

| Direct run / feature | Calls; fix turns | Observed additional work |
| --- | --- | --- |
| point7-exact-direct / visual | 12 / 36; 5 | Initial lint repair N258→260 / patch266, status/rendering refinement and positive coverage. Review increments address wrapped rows/slash interception; bytewise arrows; lone Esc; split reads; Ctrl-V in text buffers. The initial lone-Esc reproductions fail N948→951/N949→953 and are removed N999/N1003; a distinct actual-terminal-path test fails N1019→1021 before repair. |
| point7-exact-direct / slash | 4 / 0; 0 | Four passing target checks around format-only assertion-layout patch N232. No additional behavioral RED. |
| point7-exact-direct / completion | 9 / 0; 0 | Passing checks, format-check failure N321→325, formatting-only patches N331/N335/N343/N347, and an expected-string local-variable cleanup N413. No additional behavioral RED. |
| direct-003 / visual | 16 / 14; 2 | Positive uppercase-Y/OSC52 tests and rendering refinements. Later-agent metadata anchor: test502, RED508→510, production516, passing526/527. Bytewise arrows: test602, RED608→610, production616/620, passing640/641. |
| direct-003 / slash | 6 / 0; 0; plus 1 earlier baseline | N243 repairs the missing-Esc fixture from the earlier initial-window failure; focused checks and full targets pass. No new postcutoff behavioral failure. |
| direct-003 / completion | 9 / 8; 1 | Existing streaming-output assertion fails N336→342, then focused351/full360 pass without an accepted patch between. Review adds two cache/reconstructed-state regressions N472; RED478→482/479→483, production489/509/515, passing523/524. |
| direct-002 / visual | 7 / 15; 2 | Positive initial visual coverage. Review adds arrow-routing/uneven-block regressions N523/N531, RED539→541/545→547; later Ctrl-V regressions N701/N709/N717/N723 fail731→733/737→739. Both groups receive production repairs and matching printed passes. |
| direct-002 / slash | 12 / 0; 0 | Negative display assertion fails206→208, transcript expectation repair214, pass222. Additional controller-routing tests361 fail369→371, production377/385, pass393. Initial implementation therefore contains another real regression/repair increment. |
| direct-002 / completion | 7 / 5; 1 | Existing prompt-history failures305→307 lead to harness/input-routing repair330 and pass336. Review modifies existing completion tests481 to exercise `no`; RED489→491/495→497, production503, passing511/517. |
| step4-direct-001 / visual | 6 / 2; 1 | Positive visual/OSC52 coverage, rendering and selection-cache refinement, scoped compound gates. Metadata-row regression576 fails584→586; production592/600, pass608. |
| step4-direct-001 / slash | 8 / 6; 1 | Passing checks around formatting-only patches354/362/370. Scripted CommandChat routing test498 fails506→508; production514 introduces selected-agent routing, pass522. |
| step4-direct-001 / completion | 16 / 7; 1 | Old READY assertion317→325 is corrected339. Additional hidden-chat test384 follows implementation378: fixture failure393→398, expectation repair406, actual reopening failure412→414, production420, pass429. Review's repeated-prompt regression547 fails555→558 and is repaired; separate `no` coverage541 passes first556. |
| step4-direct-002 / visual | 6 / 10; 3 | Positive initial OSC52/yy and arrow coverage. Three review increments: bytewise arrows526→534→542/548; wrapped-right cursor658→666→674; long-left metadata cursor748→756→764/770. Each has later matching printed passes. Two existing-suite failures in compounds remain separate from those regressions. |
| step4-direct-002 / slash | 14 / 0; 0; plus 3 earlier baselines | Passing focused/target checks around formatting repair496; positive-only HTTP slash coverage544 first passes552. No extra behavioral RED or review-fix turn. |
| step4-direct-002 / completion | 19 / 15; 1 | Positive harness coverage, then existing repeated-review/linearize failures412→417; production426/432/436 permits ordinary followups, pass450. Review first adds controller repeated-prompt regression570 (RED578), then terminal regression657 (RED665), with distinct fixes. Invalid two-filter command705 and existing backend assertion731 remain separate. |
| step4-direct-003 / visual | 15 / 8; 1 | Positive terminal/highlight coverage, cursor/routing refinements and private API cleanup. Review revises existing tests593/597 to use the actual nonzero chat cursor; RED605→608/606→610, production616/624/642, passing656/657. Rejected patch628 remains retained. |
| step4-direct-003 / slash | 8 / 4; 1 | New selected-agent routing tests169/185 fail177→179/191→193. Production208/216 still leaves fixture failure224→226; reply-queue repair232, pass240. Review's stale-controller-selection test433 fails439→441; coordinated controller/terminal/HTTP repair447/453/461 and additional coverage475/483 precede pass497. |
| step4-direct-003 / completion | 18 / 2; 1 | Postimplementation hidden-row fixture failure362→368, expected-row repair384, pass390; duplicate-prompt refinements and gates. Existing directive assertion462→468 later passes476/489 without accepted patch. Review only requests missing `no` coverage: test583 passes first592 and suite602; no production patch or invented RED. |

The index preserves full command strings and exact output locators for every shorthand check/patch number in this table, including rejected patch attempts and formatter commands. All 169 retained postcutoff patch attempts remain identified; none is substituted for an absent source snapshot.

## What the extra work actually is

The clearest continuation mechanism is **review finding → targeted regression → observed failure → production repair → focused confirmation → scoped regression gates**. It occurs on concrete gaps such as bytewise terminal input, wrapped/expanded UI geometry, duplicate confirmation delivery and selected-agent routing. Those checks target behavior not covered by the first passing tests. They cannot be labeled repeated validation of already-demonstrated behavior merely because the Cargo target string recurs.

There is also test construction and integration recovery. Examples are an extra fake reply ahead of the expected reply (step4-direct-003 slash N224→226, repair232), incorrect rendered-row expectations, and a hidden-chat test that first exposes a fixture mistake and subsequently a production routing failure. The two point7 lone-Esc reproductions are explicitly discarded before a different core-path reproduction; treating every failure as a successful unchanged-test cycle would be wrong.

The five existing-suite failure calls require a narrower conclusion: direct-003 completion336; step4-direct-002 visual556/786; step4-direct-002 completion731; step4-direct-003 completion462. Their outputs name missing rendered messages or rows, and their subsequent focused/full or serialized commands pass. No accepted patch occurs between each failure and those reruns. That does **not** establish flakiness, a timing cause, byte-identical source, or absence of formatter/test side effects. The observations remain failed commands followed by passing rechecks, with exact provenance.

Finally, the 276 passing post-GREEN/fix calls include both confirmation and surrounding scoped gates, including checks around newly added positive-only coverage, production refinements, and formatting. They are not all proven necessary, and they are not all proven redundant. The earlier cadence's sole public-action-stable repeat candidate remains the same narrow source-state-unproven pair; this purpose classification supplies no new whole-tree identity proof.

## Mechanism boundary and verification

This completes the retained Direct remainder's purpose classification; it does not establish that WL avoids every kind of later repair. The prior complete 58 WL mediated author checks already contain behavioral, compile and invalid-command failures. Their window differs from this Direct remainder and cannot be subtracted. The actual historical source/CLI distinction also remains: normal WL source `5b1d1ef…` and CLI0.150.1; first three Direct workflows CLI0.149.1 and step4 three CLI0.150.1. Newer study metadata is not assigned retrospectively.

The supported explanation is finer than “Direct repeats tests”: the evidence distinguishes additional test/implementation increments, review-driven regressions, expectation repairs, format-separated checks, and unresolved failure/recheck episodes. Their net token contribution is not measured here. No C15 implementation, provider-ledger result, percentage, or causal-effect claim follows automatically from this classification.

The index embeds its exact read-only extraction and validation recipes. Validation passes: 18 distinct author threads, 21 fix turns, exact disjoint 92/328 whole-420 call-ID coverage, all 48 test-leading nonzero outcomes assigned once, every patch/command/output identity rehashed, all matched later-pass names checked against original output, all 13 compounds retained, and all 23 source endpoints unchanged before/after. Index SHA-256: `3a172941dfd645d977c66937a3057756927b20772ed88cdce98856d65c59e6d5`.

This is evidence-only documentation. No agent-facing code or public workflow is affected, and no real-agent generation or Cargo build is required or performed for this source classification.
