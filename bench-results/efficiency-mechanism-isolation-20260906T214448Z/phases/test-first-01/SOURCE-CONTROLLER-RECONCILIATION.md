# Closed controller reconciliation: five preview boundaries

All five flagged controller rows have the same exact explanation: the runtime interrupts after a valid v6 private-preview message, but the frozen observer does not recognize that directive. Offline replay therefore adds one post-interruption notification that the controller never receives. That notification repeats an earlier turn's entire `last` and cumulative `total`; it is not evidence of a newly charged response.

The omission also affects **live usage-grace eligibility**. Every listed interrupt is journalled `not-eligible`, `waited_ms: 0`, with configured grace 1000 ms and output-resume policy `forward`. The clean non-target grace contract is therefore not established for this batch. No claim follows about prevented generation or how much a corrected observer would have waited.

## Finite evidence

Only the five reported agent mismatches in the three closed original runs are inspected. [The JSON witness](SOURCE-CONTROLLER-RECONCILIATION.json) records 48 before/after source pins, full typed thread/turn/item/RPC identities, physical byte/line hashes, retained controller indices, trace joins and journal records. `S` means a physical line of the run's captured `server-to-client.raw`; `C` means `client-to-server.raw`, not an accounting response ID.

| Run / agent | Preview → carried usage | Host interrupt → next same-thread start | Grace journal line |
| --- | --- | --- | --- |
| 001 / user-2 | S16387 → S16388 | C40 → C41 | 14 |
| 001 / user-3 | S16144 → S16145 | C39 → none | 13 |
| 002 / user-3 | S13839 → S13840 | C40 → C41 | 13 |
| 003 / user-2 | S7781 → S7782 | C31 → C33 | 10 |
| 003 / user-3 | S16934 → S16935 | C50 → C51 | 17 |

For each row, all four fields of the original replay-minus-controller mismatch equal exactly the listed notification's `last` vector. Its `last` and `total` also equal the preceding same-thread notification, respectively at S15423, S13725, S12041, S5111 and S15465, each associated with a different prior turn. The existing `cumulative_usage_contains_last_response` predicate is false: cumulative increase is zero. These vectors are reconciliation witnesses, not totals, cost contrasts or fresh-charge attribution.

Each preview is the sole completed public agent message on its accepted turn, with a standalone typed JSON header, raw body and explicit terminal `@work-leaf end`. The only observer-recognizable directive lines in the full message are `test-preview` and its `end`; no copied, quoted or prefix-only match is used. Full initial policy input binds the captured thread to the trace-owned agent. Original and forwarded interrupt frames are byte-identical, each has a typed RPC acknowledgment, and each turn closes `interrupted`. Recorded chunk times order preview delivery, host interrupt, then the carried usage notification.

Four subsequent same-thread starts byte-match the corresponding `private-preview-delivery-attempt` prompt: trace lines 10/10/8/17 for 001-user-2, 002-user-3, 003-user-2 and 003-user-3. The matching proposal trace lines are 8/8/6/15. In 001-user-3 there is no subsequent same-thread start or private execution proposal: its retained controller line 74 reports `revision must name a delivered proposal in this launch generation`. Syntactically complete preview interruption is distinct from successful reservation/execution. Later controller errors remain in the witness and are not attributed backward to a different turn.

## Exact source chain

All locators below refer to `infrastructure/evidence/` under this phase, not mutable current source. The observer source SHA-256 is `188a1b4fd9913556c51024dcc0068be353813c44c688d6d22929da139f392b5f`, its frozen executable is `238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`, and the relevant frozen runtime source hashes match the phase manifest.

- Runtime `src/bench_private_test_first.rs::parse_preview` at 320 validates the complete envelope; `preview_terminal` at 395 accepts only that parser's success. `src/orchestrator.rs::should_interrupt_after_streamed_directive` at 2034 activates this predicate only under the v6 benchmark path. `DirectiveStreamInterruptDetector::observe` at 641 feeds `CodexAppServer`'s completed-message branch at `src/codex.rs:275`; it requests interruption, unregisters the turn and returns at 284–291, before the later usage event.
- Controller usage comes only from streamed `Usage`: `src/codex.rs:295` reads `tokenUsage.last`, `src/workspace.rs:2080` emits `WorkerEvent::Usage`, and 1237/1543 aggregate it into the session snapshot. The affected closed snapshot rows exactly equal retained `controller-usage.json`.
- Observer `bench-observer/src/lib.rs::assistant_text_completes_work_leaf_directive` at 1910 knows ordinary read/edit/patch/done/locked-command/send directives, not test-preview. Its only production callers are the **live** `ProviderUsageGrace::observe_server_value` at 793 and the **offline** `analyze_app_server` replay at 5364. Remaining references are unit tests at 8023–8034.
- Live capture enables the configured grace at 442/569. `bench-observer/src/raw_capture.rs:301` delegates interrupt forwarding to `forward_app_server_client_frame` at lib.rs:1159. The unrecognized preview leaves `directive_complete` false; `wait_before_interrupt` returns `NotEligible` at 819, and the original interrupt is forwarded. The retained journals prove that branch in these five cases.
- Offline replay at 5354 keeps accumulating `last` until its own directive predicate succeeds. At 5467 it combines those per-turn values; `reconcile_controller_usage` at 4524 compares them with the controller. Its separate fresh-usage/incomplete-turn gates at 5431 and 5477 are not replaced by this review.

## Retained limits and future boundary

Original `analysis.json` files remain `capture_complete: false`, with the controller mismatches and incomplete-turn counts 2/3/2. No native/raw response-ledger census, accounting helper, provider, test, Cargo or executor was run. This witness does not clear those flags, prove provider-ledger completeness, supply missing response identities or support a clean test-timing/token-effect claim. Exact carried public counters are not a substitute for raw/native accounting.

The smallest prospective repair is an explicit, default-off observer grammar selection bound to the admitted v6 invocation. The runtime's complete typed preview grammar—not a loose marker prefix—must be shared or mechanically source-bound for both the live grace predicate and offline delivery predicate. A valid preview then receives the existing 1000 ms / resume-forward eligibility; its fresh-usage checks, original interrupt bytes, wait outcomes and ordinary grammar remain unchanged. Legacy/default runs must retain their existing behavior. Malformed, quoted, fenced, indented or mixed envelopes must not gain eligibility. This is a future implementation/qualification boundary, not a modification of this batch or an admission.

Read-only checks: final five-row/source join `e118a2`, zero cumulative increase `768421`, identical prior `last`/`total` `5d92c0`. Initial check `94a50d` failed an incorrect freshness assumption; superseded in-memory projection `7f492a` retained a literal true. Neither was published as evidence. The final witness records the actual false freshness predicate for every row. All original artifacts remain unchanged.
