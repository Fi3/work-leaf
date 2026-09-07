# C15 diagnostic002 semantic qualification

The real private preview demonstrates the intended behavior-specific RED, exact feedback delivery, and a subsequent ordinary shared patch preserving the same tests. It does **not** demonstrate the complete normal focused-check/done cycle: the author's requested check was left unprocessed at the harness's two-round limit. The original exit 0 and clean reviewer verdict remain retained, with this narrower qualification.

Scope: the closed `private-test-first-diagnostic-002` attempt only. Read-only inspection followed `BUILD-AND-EXECUTION-SCOPE.md` and the inherited finite postcapture operator plan. No executor, test, provider, extractor or accounting command was run; no test was altered to force a color. Native-rollout membership and whole-invocation closure remain root's separate source review, not a claim made from public item IDs here.

## Held and accepted tests

Let `O` denote `/tmp/c15-real-diagnostic-002.aD9RpS/preview/preview-0000000000000001`; `P` denotes `/tmp/c15-real-diagnostic-002.aD9RpS/project`.

The complete held file `O/test/preview/repo/src/lib.rs` is 410 bytes, SHA-256 `071c0586d9a7e6a4170049f1a0639f1318d3ae19ae390a5d5662a37cb2af9137`. Its first line exactly equals the 53-byte initial file in `O/test/accepted/src/lib.rs` and the ordinary commit's parent: `capped` returns `value` and ignores `_limit`. The held patch adds only the test module; it does not include the missing minimum implementation. The public proposal is server record 277, exactly matching trace record 3 and `TEST-RESULT.json:/request/proposal` after parsing its declared envelope and ordinary end marker. Proposal identity is `capped-tests-1`, generation 1, no revision.

The private execution is `cargo test --offline --locked`, carried unchanged as the shell command in `TEST-EXECUTION.json:/argv`. The same receipt records the private repository mounted at the original live project path, a separate writable build directory, `closed:true`, `exit_code:101`, `timed_out:false`, `cancelled:false`, and no stop reason. `TEST-RESULT.json` reports the completed execution, not a preparation failure, and retains `shared_accepted:false` and `promoted:false`.

All three tests compiled and ran. Below-limit `capped(4,9)==4` and equal-limit `capped(7,7)==7` passed. Above-limit `capped(12,5)==5` failed at held `src/lib.rs:19:9`, actual left 12/right 5. This is a behavioral assertion RED, not a compile, fixture, dependency or timeout error. Result and execution stdout/stderr match exactly. The result file SHA `6327d69da6d179f84a244131429f997225776f87a2c62fcc14f91b27ea64f132` matches the trace's explicit result reference. This semantic verdict is grounded in the assertion and actual output, not simply the nonzero exit.

The accepted commit is `edd398ac604496e6ffbd0a9bf79262b357e1eb2e`; its `src/lib.rs` blob equals the current 420-byte file in `P`, SHA `e5ed089211ceeecc514beb275dbb74d15199b99a0b7e07ef83d1e257aecad4b9`. Only implementation line 1 differs from the held file: it accepts `limit` and returns `value.min(limit)`. Held bytes `[53,410)` and accepted bytes `[63,420)`—lines 2–21 in each—are identical, 357 bytes, SHA `c0e6faca8cceabeff0aa9598d715697a1f893b4cd068d84632bfdbed3ef0c40f`. That exact range contains all three test functions/assertions; no test revision or removal appears. Whole-file hashes differ because production and tests share a file; those whole-file hashes alone are not the equivalence proof.

## Actual public delivery and workflow boundary

`C` and `S` are physical lines in `observation/app-server/00000511898560227962-11/client-to-server.raw` and `server-to-client.raw`; forwarded client records have the same line numbers and exact typed `turn/start` objects. All five accepted inputs match their complete public `userMessage` text with empty `text_elements`. `PRIVATE-TEST-SEMANTIC-WITNESS.json` records every exact thread, turn, typed RPC ID, public item ID, text hash and source file hash; these public IDs are not native-rollout IDs.

| Handoff | Exact records | Observed meaning |
| --- | --- | --- |
| Author launch → held preview | C4, accepted S6, public input S11, generated S277 | Natural regression proposal; trace line 3 |
| Private result → ordinary combined patch | Trace 4–6, C6, accepted S282, public input S286, generated S521 | Exact private failure text delivered in the same author thread; ordinary patch follows |
| Real patch ACK → focused-check request | Trace 7, C8, accepted S528, public input S532, generated S548 | Normal ACK for `src/lib.rs`; model emits `@work-leaf locks run target -- cargo test --offline --locked` |
| Reviewer launch → source read | Trace 8, C11, accepted S558, public input S563, generated S575 | Ordinary review route; requests `src/lib.rs` |
| Read response → clean verdict | C13, accepted S582, public input S586, generated S593 | Exact accepted source delivered; reviewer returns `NO_FINDINGS` |

The eight-row trace contains one proposal, one result, one delivery attempt, one delivered record, one ordinary ACK and the two owned policy launches. There is no command-result continuation and no author `@work-leaf done`. The public input census contains no ordinary check-result input. The reviewer requests source and then gives a clean verdict; it does not request a normal focused command in this saved public chain.

The concrete source boundary explains this absence: frozen `harness.rs:168` sets `with_max_review_rounds(2)`. This is a diagnostic-harness cap, not a normal Work Leaf two-round limit: `CommandChat::new` defaults to 80,000,000, and the same field limits author directive processing as well as review. In `src/cli.rs::process_agent_reply_streaming_result`, lines 1169–1182 check the round bound before dispatching the pending response. Processing the preview and then the combined patch consumes those two rounds; the next returned response contains the command request but is not dispatched. The retained author transcript says `agent did not converge after 2 orchestrator rounds`. Frozen `harness.rs:187–205` then calls `review` after an `Ok` author return and records the resolved review without requiring the missing author completion/check links. This is a harness budget/qualification gap and a false-positive exit for the complete required chain, not evidence that the requested shared check failed or passed. No normal shared-test GREEN or complete author convergence is claimed.

## Provenance and disposition

The companion witness retains 25 exact source-file hashes and their sizes, rechecked after inspection (tool `3c2db3`, exit 0). It verifies the held afterimage against its receipt, parent/accepted Git blobs against the retained files, public proposal fields/body against host-held fields, exact private result reference/output, all five typed accepted/public input joins, and the full held/final test region. The source-only inspection also confirms the CLI and frozen harness/guard hashes are members of `FINAL-SOURCES.json`. It does not rerun the bridge's dependency qualification or replace root's complete 188-input endpoint audit.

`TERMINAL.json` SHA `d37e86b1572007f0b97caec01d72559b8ad5339d612ae2268a77a66fbc386b34` records launcher exit 0, 60.960064835 seconds, all admitted source endpoints matching. `HARNESS-RESULT.json` SHA `c6196f98968882e49de48dc64303e1ddea4ac746dee202ace6d69ef753a29e90` records five outer calls, two roles, settled capture, no watchdog, and one resolved review round. Neither field substitutes for the missing ordinary check/done links.

Disposition: private test-first startup, real behavioral RED, same-session feedback, exact test retention and ordinary application are demonstrated in this diagnostic. Full requested shared-validation/convergence qualification remains incomplete. This record supplies no token-saving estimate, percentage, counterfactual, or authority for a replacement attempt. Diagnostic001 and all original002 artifacts remain unchanged.
