# C15 complete-workflow diagnostic result

The real subscription workflow completes the intended chain: **private behavioral RED → same tests with implementation → ordinary shared patch/ACK → normal GREEN → author DONE → resolved review**. This is a workflow qualification result, not a C15 token-saving estimate.

The sole admitted command runs **2026-09-07 22:05:57.550833–22:06:52.605792 UTC**, **55.055 seconds**, exit **0**, with **six outer calls and two real roles**. All 215 admitted immutable endpoints match. No watchdog fires, no replacement runs, and no historical benchmark/control/Direct observation is rerun. [Terminal](TERMINAL.json), [harness result](HARNESS-RESULT.json), [admission](ADMISSION.json).

## Actual behavior

The agent supplies its own three regression tests, with no supplied test body or edited task. Its sole preview retains the original implementation. Below-limit and equal-limit tests pass; the above-limit test fails at `capped(9, 7)`, actual 9 versus expected 7. The private command exits 101 with closed children, no cancellation or timeout. This is a behavior-specific assertion failure, not a compilation/environment error.

The full private failure reaches the same author. It submits one ordinary combined patch, accepted as `f1579e1c5ec801b90c4fe6dda887a558a0c7439c`. The implementation uses `value.min(limit)`. The entire 345-byte held test suffix is unchanged: held `src/lib.rs` bytes `[53,398)` equal accepted bytes `[67,412)`, SHA256 `e8a24fd51fbc9d0d1519efd7f794d0bdac11575cf9671d417165620c0f1b3f22`. No test is revised or removed. [Semantic review and exact operation links](postcapture/PRIVATE-TEST-SEMANTIC-REVIEW.md).

The actual normal locked shell executes `cargo test --offline --locked` against the accepted project and returns status 0: three tests pass. That result reaches the author, which emits a processed top-level `@work-leaf done`. The ordinary reviewer reads accepted source and returns `NO_FINDINGS`; one review round resolves. The private execution is not substituted for this normal validation. [Independent closure review](postcapture/INDEPENDENT-CLOSURE-REVIEW.md).

| Exchange | Actual accepted input / generated outcome |
| --- | --- |
| Author launch | Private test proposal |
| Private failure feedback | Combined tests-and-implementation patch |
| Normal patch ACK | Focused locked-command directive |
| Normal command result, status0 | Author DONE |
| Ordinary reviewer launch | Source read directive |
| Source read result | NO_FINDINGS |

All six complete model inputs match their typed accepted request, original/forwarded transport, complete public user message and explicit-turn native item. All six generated public messages match their native items exactly. Both native threads retain gpt-5.5/xhigh/read-only configuration; their complete response-item population contains messages/reasoning and no native tool actions. Reasoning bodies are not used or exported. Only the explicitly owned author policy span differs; normal ACK/check prompts and known-session followups remain unchanged. [Root source witness](postcapture/ROOT-SOURCE-WITNESS.json) pins 258 sources and retains exact physical locators.

## Meaning of the harness correction

The predecessor's `.with_max_review_rounds(2)` also bounded its author directive loop in `src/cli.rs::process_agent_reply_streaming_result`. Preview and patch consumed that fixture cap before the normal check could execute. Normal `CommandChat::new` has the separate default 80,000,000; no normal WL implementation or default changes in003.

This private crate's `guards.rs::diagnostic_chat` retains the ordinary default and the existing locked-command timeout. `harness.rs::author_capture` and `guards.rs::author_completion` require the actual owned successful check plus real author completion **before** `Budget::begin_review`. The independent eight-call/240-second cancellation/300-second outer bounds remain. Tests reproduce the saved002 false success and the actual capped CommandChat sequence before the correction. [Validation](HARNESS-CORRECTION-VALIDATION.md), [independent source review](HARNESS-INDEPENDENT-REVIEW.md).

Required unchanged-runtime gates pass 534 tests with zero failures and clean fmt/Clippy. The private crate passes 18 tests with zero failures/two ignored and clean fmt/Clippy; independent testing repeats that result. The real scenario above supplies the actual-agent verification rather than a synthetic substitute.

## Measurement boundary and retained outcomes

The unchanged original analyzer runs once and exits 2; the prerequisite-gated extractor runs once and exits 0 with two matched threads and no extraction errors. The original analyzer retains **one interrupted turn with incomplete usage** and eight marker hits in the two exact pinned observer ELF copies. The latter exact-file explanation does not clear the usage flag or `capture_complete:false`. No reanalysis, new accounting or token percentage is computed. [Original postcapture](postcapture/ORIGINAL-POSTCAPTURE-REVIEW.md).

Original001 remains a zero-provider startup failure;002 remains its partial outcome with an unexecuted normal check and missing DONE. This successful003 neither rewrites those records nor establishes which fraction of the accepted historical 45.38%–51.62% savings C15 causes. C15's real workflow feasibility is established; causal direction/contribution and the broader mechanism plan remain separate unfinished work.
