# C15 diagnostic 002: independent closure review

Recorded 2026-09-07 20:17:41 UTC. This bounded review checks source endpoints, actual invocation/capture closure, typed accepted-turn boundaries and prepared-role evidence. It runs no provider, private/ordinary test, analyzer, extractor or accounting call. Full native-input mapping and held/final test semantics are separate root/semantic reviews; no reasoning bodies are exported.

**Closure/source checks pass; the complete intended workflow is not qualified.** The outer command genuinely returned 0 after 60.960064835 seconds, 20:10:36.414477–20:11:37.374540 UTC. That result is retained separately from missing ordinary validation/author completion and original capture flags.

## Independently checked closure

- All **188 admitted immutable endpoints** match (`2de632`), with exact admission-to-terminal digest. The separate initial project receipt is not mistaken for immutable editable source.
- Physical `observation/invocations` and `observation/app-server` populations each contain exactly `00000511898560227962-11`. The subsequently published `process-invocations.jsonl` has that same single row; every projected start/end field equals the actual metadata (`38285c`). No shell invocation is present.
- `meta.json` exactly contains the separate `start.json` and `end.json`. End status is 0, signal null. The start digest, original client/server/stderr digests, forwarded client digest, grace/rewrite/raw-usage artifact digests all match the end receipt. Original streams have complete contiguous chunk partitions: 13 stdin chunks, 557 stdout chunks, 0 stderr chunks, with exact range hashes and bounded recorded times (`2de632`). This does not claim complete provider usage.
- Start metadata records primary=true, raw-response capture=true, grace=1000 ms, output-resume=forward, required project inventory=true, and the admitted subscription wrapper/cwd. The sole pre-spawn inventory and baseline are byte-identical; its completion `511898572702971` precedes actual child start `511898576872785`. It follows the proxy start, so proxy `start.json` is not used as the child-spawn boundary. This is one recorded project-layer boundary, not continuous configuration proof.
- Original/forwarded client streams each have 13 complete JSONL frames, server 598, trace 8. All five typed string-ID turn/start requests are exactly forwarded and have unique accepted replies. Every accepted turn has one matching terminal notification. All four interrupts are exactly forwarded and acknowledged. Closed JSONL tail checks also pass for all ten observer `.jsonl` files (`ffc350`, `38285c`).

## Role and terminal witnesses

Here C/F/S are physical lines of `client-to-server.raw`, `client-to-server.forwarded.raw`, and `server-to-client.raw` in the sole capture. RPC IDs below are strings, not numeric coercions.

| Role/thread | Start C=F / RPC | Accepted reply S | Terminal S / actual status |
| --- | --- | --- | --- |
| author `01a07d7e-7ef2-7160-96ec-a257694e3a76` | 4 / `"3"` | 6 | 281 / interrupted |
| same author | 6 / `"5"` | 282 | 527 / interrupted |
| same author | 8 / `"7"` | 528 | 554 / interrupted |
| reviewer `01a07d7f-0aa4-7182-9e41-f3cf724c389b` | 11 / `"10"` | 558 | 581 / interrupted |
| same reviewer | 13 / `"12"` | 582 | 598 / completed |

Interrupts C=F 5/7/9/12, RPC `"4"`/`"6"`/`"8"`/`"11"`, have replies S279/525/552/579 and exact matching thread/turn identities. Thus the last author turn is terminal **interrupted**, not naturally completed or an observed `done`; the reviewer last turn is completed.

Trace line 2's exact selected policy equals C4 and records explicit `owned_role: author`, generation 1. Line 8 equals C11 and records `owned_role: other`, generation 0, unchanged policy. The roles come from those owned records and the actual harness `prepare_agent_launch` → launch → `handle_line("review")` flow, not names alone. The harness retains five outer calls, two roles, settled/closed counts of five, no watchdog, and observer stop exit 0/stdout `1\n`.

## Preserved incomplete workflow and analyzer result

`HARNESS-RESULT.json` retains the diagnostic fixture's two-round nonconvergence message after the author's generated `@work-leaf locks run target -- cargo test --offline --locked`. The fixture's `harness.rs:168` selects `with_max_review_rounds(2)`; `src/cli.rs:1173` also passes that limit to author orchestration. The ordinary default is 80,000,000 rounds (`src/cli.rs:705`), not two. The eight-row trace has private proposal/result/delivery evidence, an ordinary patch ACK, and reviewer launch, but no ordinary command-result event. There is no ordinary shell capture. Root's source witness agrees: `ordinary_command_executed: false`, `author_done_observed: false`, `full_intended_workflow_qualified: false`. A clean review and harness exit 0 do not establish an executed focused GREEN check or author completion. This review does not reinterpret private exit 101's semantic cause; that belongs to the held-test source review.

The once-only original analyzer returned **2**, with `capture_complete: false`: one interrupted provider turn lacks complete usage, plus eight marker findings in the two complete copied observer ELF proxies. Original extraction returned **0**, matching two observed native threads with no extraction errors. Controller reconciliation is an empty array because no controller rows exist; it is not a proved controller/native reconciliation. These flags remain unchanged. No original marker or usage error is cleared by this review.

The root witness's 204 source pins independently match; its role/closed-work limitations agree with the independent checks. Root owns its five complete original/forwarded/public/native input joins and native model/action assertions; this review does not falsely claim to repeat that full mapping. The analyzer and extraction stream hashes match their execution receipts, and analyzer stdout still exactly matches `mechanism-summary.json` after extraction.

## Evidence identities

| Artifact | SHA-256 |
| --- | --- |
| `../ADMISSION.json` | `fed9f20f3a26ca584413284ff52be9719c589b6d5bc94781b57f0315370383e0` |
| `../TERMINAL.json` | `d37e86b1572007f0b97caec01d72559b8ad5339d612ae2268a77a66fbc386b34` |
| `../HARNESS-RESULT.json` | `c6196f98968882e49de48dc64303e1ddea4ac746dee202ace6d69ef753a29e90` |
| `../prompt-events.jsonl` | `53460ad1898a142b5e77b212ee2d6e3edff6a216c2278135e51bbd60befd68e8` |
| `../observation/process-invocations.jsonl` | `af86d041d5ccc7217aa8067690d0323984a94050cdc16e1d62e0871339d45137` |
| [ROOT-SOURCE-WITNESS.json](ROOT-SOURCE-WITNESS.json) | `e988125c9cc3aaf72b368816f0820ce1848c05d3e19e0089dd6943b44fe00e01` |
| [OBSERVER-ANALYZE-ORIGINAL.json](OBSERVER-ANALYZE-ORIGINAL.json) | `192fc73372e9923cbf7a2247f1fabe7a2e9fb6676720cc0ac184fd411798b871` |
| [OBSERVER-EXTRACT-ORIGINAL.json](OBSERVER-EXTRACT-ORIGINAL.json) | `73bc892aa052545fab211cee00e143f13ec35aec3ef6d6bfbfdbddc9bd4520e2` |

Original diagnostic 001, all raw artifacts and all current outcomes are preserved. This is not an effect estimate, whole-workflow green claim, retry authorization or new benchmark observation.
