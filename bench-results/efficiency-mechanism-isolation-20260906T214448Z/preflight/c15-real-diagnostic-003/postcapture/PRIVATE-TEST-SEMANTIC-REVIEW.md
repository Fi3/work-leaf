# C15 diagnostic003 semantic qualification

The closed third diagnostic demonstrates the complete bounded private-test-first chain: a genuine behavioral RED, same-session feedback, an ordinary combined patch retaining the exact tests, an actual successful focused command, author DONE, and source review ending NO_FINDINGS. No executor, test, provider, extractor or accounting command was run during this review.

Let `O` be `/tmp/c15-real-diagnostic-003.lCT2VA/preview/preview-0000000000000001` and `P` its sibling live project `/tmp/c15-real-diagnostic-003.lCT2VA/project`. Source hashes and physical records are retained in [PRIVATE-TEST-SEMANTIC-WITNESS.json](PRIVATE-TEST-SEMANTIC-WITNESS.json).

## All private operations and test identity

The entire nine-record trace contains one proposal, one result, one delivery attempt, one delivered record, one ordinary ACK, one command result and two policy launches. The preview root contains only startup `validation` and the single numbered preview. There is no proposal revision, replay, second preview or private implementation check in the observed chain.

Public generated record S293 declares `cap-tests-v1`, generation 1, no revision, test path `src/lib.rs`. Parsing its complete JSON header and exact patch body gives exactly trace line 3's proposal; every declared field and body matches both capture and test requests. All three operation request/attempt/result/output links validate: startup validation and capture complete without a shell exit, and the test completes with exit 101. Their bridge processes exit 0 without cancellation. The private patch-driver receipt exits 0 and matches the held proposal/body and nested test result. The result file SHA is `a428bfed54ad7835ffc438691a7f2094058df57133879752072d95b8ba4cc96f`; its explicit trace reference and stdout/stderr agree exactly.

The held 398-byte file retains the original 53-byte implementation returning `value` and adds three tests. The actual private `cargo test --offline --locked` compiles and runs them: below-limit `capped(4,7)==4` and equality `capped(7,7)==7` pass; above-limit `capped(9,7)==7` fails at held line 19, actual 9/expected 7. This is a behavior assertion failure, not a compile, dependency, preparation or timeout failure. `TEST-EXECUTION.json` records closed=true, exit=101, no cancellation/timeout/stop reason, the exact command, and the private repository mounted at the original live path with separate writable build/scratch. No shared apply or promotion is claimed by this receipt.

Ordinary commit `f1579e1c5ec801b90c4fe6dda887a558a0c7439c` follows selected initial commit `77548ba9ba808f4e0f1187834d0081ea56b85587` and changes only `src/lib.rs`, mode 100644. Its blob equals the retained 412-byte live file, SHA `59672ffef5d56da0238ee741f9b1400a27b3dfad442c41a92fec33c6de637c84`. The implementation returns `value.min(limit)`. Held bytes **[53,398)** and accepted bytes **[67,412)**, respectively lines 2–21 and 4–23, are identical: 345 bytes, SHA `e8a24fd51fbc9d0d1519efd7f794d0bdac11575cf9671d417165620c0f1b3f22`. These bytes contain all three tests/assertions; none was revised or removed. The held full-file SHA is `7468ffbc6ca9c9fd4bdc7b42ca35674a446df20008b70769389ac4600a88e7e6`. Whole-file hashes alone are not the test-equivalence proof.

## Actual normal workflow

C/S denote physical lines in `observation/app-server/00000518819721475357-11/client-to-server.raw` and `server-to-client.raw`. All six typed accepted inputs are byte-identical to forwarded requests and complete public user text with empty text metadata. Each has exactly one public generated message; exact IDs are in the witness.

| Handoff | Client / accepted / public input / generated |
| --- | --- |
| Author launch → held preview | C4 / S6 / S11 / S293 |
| Private RED feedback → combined patch | C6 / S298 / S302 / S523 |
| Ordinary ACK → focused command request | C8 / S530 / S534 / S550 |
| Actual command result → author DONE | C10 / S557 / S561 / S568 |
| Reviewer launch → source read | C13 / S578 / S583 / S595 |
| Exact final source → NO_FINDINGS | C15 / S602 / S606 / S613 |

Trace lines 4–6 bind private feedback delivery; line 7 is the normal ACK. The sole ordinary locked-command invocation `00000518848757187114-865` has an actual start/end receipt, live project cwd, exit 0 and no terminating signal. Its raw stdout/stderr hashes and complete contents match trace line 8's command-result prompt: **3 tests passed, 0 failed**, followed by zero doc tests. Its observed execution occurs after the ACK and before check-result delivery. The command string is unchanged from the private preview. S568 is the author's actual standalone `@work-leaf done`, before reviewer acceptance S578; the retained orchestrator transcript also records the processed AgentDone event. The reviewer reads the exact accepted source and returns NO_FINDINGS, one resolved round.

This ordering follows the frozen source seams: `src/orchestrator.rs::run_private_test_preview` (1230) sends factual private feedback; ordinary patch/command/AgentDone processing remains authoritative. D3 `harness.rs:225–240` requires the source-bound author completion guard before entering review. `guards.rs::diagnostic_chat` (18) uses the normal CommandChat round default, not D2's accidental two-round harness cap.

## Scope and provenance

Read-only operator assertions passed at tool `f1daf7`, including 41 source-file endpoint hashes, exact proposal/receipt chains, held/final Git and byte identities, six public input/output joins, and ordinary command output. Earlier operator drafts incorrectly treated an attempt pathname as an object (`845f87`) and expected a timestamp on a v6 policy event (`5673dc`); these checker-shape errors were corrected without changing artifacts or runtime. They are not diagnostic failures.

[ROOT-SOURCE-WITNESS.json](ROOT-SOURCE-WITNESS.json) separately supplies the complete source/native proof; the public item IDs here are not native-rollout IDs. The root lock receipt records the actual caller table's `.` capture interval, 0.362094994 seconds, but does not claim global live-tree atomicity. The Python materializer's owned accepted root remains distinct from the original live mount path.

The retained terminal is exit 0, 55.0549667 seconds, six outer calls/two roles, no watchdog and resolved review. Qualification is for this small dependency-free diagnostic and this complete observed chain—not a full benchmark, universal test quality, pure timing-only comparison, or token-saving estimate. Private build/environment differences and extra feedback remain treatment work. Diagnostic001's startup failure and diagnostic002's incomplete normal check/DONE qualification remain unchanged.
