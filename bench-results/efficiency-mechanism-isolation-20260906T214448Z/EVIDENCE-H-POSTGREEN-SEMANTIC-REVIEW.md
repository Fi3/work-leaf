# H post-GREEN activity: independent semantic review

Recorded 2026-09-07 17:28:36 UTC. Verdict: **PASS, no introduced factual finding in the reviewed scope.** This is a source-only review of historical evidence, not a benchmark, token calculation, or implementation qualification.

Reviewed artifacts:

- [Activity report](EVIDENCE-H-POSTGREEN-ACTIVITY.md), SHA-256 `7843f545e1af9f18082dd32d1e771793db879152bf1f1122b4a5f5c57fc3a7a9`.
- [Source-linked index](EVIDENCE-H-POSTGREEN-ACTIVITY.json), SHA-256 `3a172941dfd645d977c66937a3057756927b20772ed88cdce98856d65c59e6d5`.

The report was read completely. All 40 failure episodes were independently checked in author/run order against the full referenced public test, production and removal patches, their actual failed command outputs, and the delivered review findings where applicable. This covers 97 distinct accepted patch bodies and 49 distinct failed call/output pairs: the 48 test-leading failures and the separately retained fmt-leading compound failure. All 56 distinct subsequent-check outputs were revalidated for the exact claimed printed passing names and recorded command exit status. Twenty-three pinned source endpoints and both report identities match before/after this read-only verification.

`aN` below means index `authors[N]`; `Nxxx` means a one-based physical native JSONL line in that author's pinned `native_source`, not a model response number. The index supplies exact paths, item/call/turn identities, output lines and input/output hashes.

## Semantic checks

| Episodes / source anchors | Independently supported distinction |
| --- | --- |
| 01–08, a0 | Invalid multiple Cargo filters are separate from executed failures. New wrapped-row/slash regressions and revised expectations coexist in the first group. The partial implementation still fails before further repair. Bytewise arrows and Ctrl-V introduce separate behavioral regressions. The lone-Esc tests N936/N940 fail N948/N949 and are removed N999/N1003; the later N1011 test failing N1019 is a distinct core-path reproduction. N1165 revises the lone-Esc expectation while adding split-read coverage, so N1173 remains mixed new/revised-test RED. |
| 09–14, a3/a5/a6 | Expanded left-row geometry, bytewise arrows, reconstructed completion state, block-selection geometry and Ctrl-V buffer routing have actual targeted failures followed by relevant production patches. The a5 existing streamed-reply failure N336 remains separate from the new review regressions N472→478/479. |
| 15–20, a7/a8/a9/a10 | A negative slash test's rendered-frame assertion is repaired to inspect the transcript. Additional controller slash routing is a real initial-turn test/implementation increment, not repeated checking alone. Existing prompt-history failures lead to a mixed production/input-fixture repair. The `no` path is added to existing completion tests; later metadata-row and scripted CommandChat tests introduce distinct behavioral regressions. |
| 21–24, a11 | The earlier READY assertion is reversed as an expectation repair. Hidden-chat implementation N378 precedes test N384: N393 first fails an overbroad row-label assertion; after N406 corrects it, N412 reaches a separate reopening failure repaired at N420. Repeated terminal prompt delivery fails after new test N547. Separate `no` coverage N541 passes first, and is not credited as RED. |
| 25–29, a12 | Bytewise arrows, wrapped-right cursor geometry and bounded left-pane cursor geometry are distinct review-driven regressions. Compound N556 prints the targeted regression passing but exits 101 at a later existing terminal assertion. N786 fails another existing assertion. Later `&&` clauses are not presumed executed. |
| 30–34, a14 | Existing review-reuse/linearization failures lead to normal-followup routing repairs. Controller duplicate suppression and terminal duplicate suppression are two successive new regression/fix cycles. The invalid two-filter invocation and the later existing backend assertion remain separate. |
| 35–38, a15/a16 | Nonzero visual cursor coverage revises existing tests rather than introducing untouched tests. Selected-agent routing has real preimplementation failures; after routing repair, N224 reaches a different failure caused by the fake reply queue, corrected at N232. The stale-controller-selection test then exposes another real routing defect. |
| 39–40, a17 | A postimplementation hidden-row test has an incorrect expected rendered label, repaired at N384. The existing directive-output assertion is a separate failure/recheck case. Its later review turn adds only the missing `no` coverage N583; N592 and N602 pass and no production patch is present in that turn. |

The four authors without failure episodes were also checked for the report's specifically cited patch claims: a1 N232 is assertion layout only; a2 N331/N335/N343/N347 are formatting and N413 factors the same expected string; a4 N243 inserts the missing Esc in the fixture; a13 N496 is formatting and N544 extends the HTTP fixture's slash coverage. These checks do not claim a second full semantic review of every one of the index's 169 postcutoff patch attempts. The complete failure-episode review and selected nonfailure claims supplement the separate mechanical population/identity review.

## Limits preserved

For the five existing-suite failure intervals, the retained action index has no accepted `apply_patch` between failure and the cited passing reruns: a5 N336–360; a12 N556–586 and N786–802; a14 N731–747; a17 N462–489. This is deliberately **not** byte-stable source evidence: native commands, formatting, tests, or other side effects are not excluded by that narrower observation. No flakiness or timing cause is established.

A matching later printed test name does not establish identical test bytes, and a relevant production patch does not prove it alone caused the later pass. The removed/revised tests, overlapping repair groups and first-GREEN-only coverage are retained. Additional implementation and review-driven validation are supported; necessary-versus-redundant checks, avoided WL work, whole-workflow token contribution and causal percentages are not established by this classification. The historical CLI/source qualifications and the different WL/Direct observation windows remain intact.

No provider, Cargo execution, accounting helper, runtime mutation, semantic relabeling, or owner-artifact edit was performed. This evidence-only review affects no agent-facing workflow and requires no new real-agent verification. It introduces no executable algorithm or public documentation/API change.
