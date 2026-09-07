# Corrected C15 diagnostic result

The startup correction is verified with the real subscription backend. The complete intended diagnostic workflow is **partially qualified, not passed**: its harness exits 0 although the ordinary post-patch check and author completion never execute. No normal-WL control, Direct workflow or historical benchmark was rerun.

## Actual attempt

The single admitted command runs 2026-09-07 20:10:36.414477–20:11:37.374540 UTC, 60.960064835 seconds, exit 0. All 188 immutable dependency/provenance endpoints match. Five calls use two actual subscription threads: three author inputs and two reviewer inputs. Observer shutdown succeeds, all five accepted turns reach terminal records, and the watchdog does not fire. The original zero-provider diagnostic001 remains failed and unchanged.

## Verified behavior

The real author supplies its own three regression tests through the private-preview directive. The private source retains the original implementation. Cargo compiles and executes those tests: below-limit and equal-limit cases pass; the above-limit assertion fails with actual 12 versus expected 5. This is behavior-specific RED, not a compile, fixture or environment failure.

The complete private failure feedback reaches the same author thread. The author submits one ordinary combined implementation/test patch, accepted at commit `edd398ac604496e6ffbd0a9bf79262b357e1eb2e`. The implementation becomes `value.min(limit)`; all held test lines remain byte-identical. The normal patch ACK reaches the author unchanged. The reviewer subsequently requests the source and returns `NO_FINDINGS`. The [semantic review](postcapture/PRIVATE-TEST-SEMANTIC-REVIEW.md) retains exact held/accepted byte ranges and execution receipts; the [independent closure review](postcapture/INDEPENDENT-CLOSURE-REVIEW.md) verifies all invocation and terminal boundaries.

The [root source witness](postcapture/ROOT-SOURCE-WITNESS.json), SHA `e988125c9cc3aaf72b368816f0820ce1848c05d3e19e0089dd6943b44fe00e01`, verifies all five full inputs by typed accepted RPC, original/forwarded frame, public user item and explicit native thread/turn/item identity. There are no native tool actions beyond the captured public directive workflow. Thirteen forwarded frames differ only at the three permitted, journalled observer metadata sites. Author policy differs only in its owned span; reviewer policy and ordinary ACK remain unchanged. All 204 witness source endpoints match before and after inspection. No private reasoning body is used or exported.

## Missing normal check and exact harness cause

The author emits `@work-leaf locks run target -- cargo test --offline --locked` at public server line 548. There is no subsequent author input, command-result trace, locked-shell invocation, native execution or `@work-leaf done`. The reviewer reads the source but does not run this check either. No ordinary GREEN result is evidenced.

The fixture's `harness.rs:168` calls `CommandChat::with_max_review_rounds(2)`. Despite its review-oriented name, `src/cli.rs::process_agent_reply_streaming_result` also uses that field for author directive handling (`src/cli.rs:1173`). The two processed author directives are the private preview and combined patch. The third generated directive—the ordinary test command—hits the cap before dispatch. The transcript explicitly records `agent did not converge after 2 orchestrator rounds`.

This two-round cap belongs to the diagnostic fixture, not normal WL: `CommandChat::new` sets 80,000,000. The normal launch route returns an `AgentLaunched` transcript under optional completion. The fixture then starts review and accepts a resolved review plus transport closure without asserting an executed ordinary check or author DONE. Hence exit 0 is a harness false positive for its declared complete workflow. No source or admitted harness is changed after observing this result.

## Original accounting flags and interpretation

The unchanged analyzer runs once and exits 2, retaining one interrupted turn without complete usage plus eight marker flags located in pinned copied observer binaries. The unchanged extractor runs once and exits 0 with both native threads matched and no extraction errors. The [original postcapture review](postcapture/ORIGINAL-POSTCAPTURE-REVIEW.md) keeps these outcomes separate; no reanalysis or missing-usage-zero assumption is used.

This observation qualifies real private RED feedback followed by a test-preserving normal shared patch. It does not qualify ordinary post-patch validation/DONE, close the overall candidate plan, estimate a token effect, or change the accepted historical 45.38%–51.62% reduction.

The next required fixture correction is to avoid the accidental two-step author cap while retaining the declared external call/time bounds, and to reject incomplete check/DONE chains in the success predicate. Any further real diagnostic needs a separately approved admission; none is launched or silently substituted here. Normal WL and all previous benchmark outcomes remain unchanged.
