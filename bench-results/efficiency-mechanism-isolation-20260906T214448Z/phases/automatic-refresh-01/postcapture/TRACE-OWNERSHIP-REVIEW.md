# Finite C08 trace ownership and selected-input review

All 17 selected trace texts have exact original-request, forwarded-request,
accepted RPC, public user-item and explicit-turn native user-item witnesses.
The six ordinary rows are physically delivered; the saved `ambiguous` statuses
reflect the pure helper's narrower ownership-anchor rule, not missing delivery.
Original and corrected source results, errors and accounting flags remain intact.

## Six ordinary rows

Each run's trace lines 2/3 are unchanged `patch-applied`/`command-result` inputs
for `user-2`. Each has one trace occurrence, one global input occurrence and a
saved fully joined input. Their input indices are 10/12, 10/11 and 13/14 for
001/002/003. The initial accepted input on each matching thread independently
contains the source-rendered `Agent-ID: user-2` / `Feature:` suffix before
`User prompt:`; complete original/forwarded/public/native bytes and typed turn
identities were checked for those three initial inputs too.

This is not ownership inferred from an arbitrary marker in a later reply.
Frozen `agent.rs::PromptPolicy` appends that launch suffix at467;
`orchestrator.rs` uses the same `agent_id` for continuation rendering and send
at1015 and1494; `bench_experiment.rs::forward_v2` records that explicit owner
with the exact forwarded text at320–353. The two independent source relations
agree for each ordinary pair.

`join_automatic_refresh_delivery.py::join_occurrences` creates owner claims
only from singleton `automatic-refresh` events. None belongs to these three
user-2 threads, so their owner is unknown under that rule and their ordinary
owner-specific queues are empty. No helper is modified or rerun for this
finite source qualification.

## Eleven eligible deliveries

Every eligible trace is a singleton automatic-refresh anchor with its recorded
owner and a joined input. Direct checking also verified each held full body
against its byte count/FNV digest and all non-owned prompt ranges against the
original renderer text. These are source/delivery facts, not yet Git-state or
repair-success facts.

| Run | Trace line | Owner | Client / public / native line | Held path | Full-body bytes |
| --- | --- | --- | --- | --- | --- |
| 001 | 4 | user-3 | 39 / 8936 / 189 | src/ui_harness.rs | 21903 |
| 001 | 9 | user-1 | 53 / 23093 / 193 | src/ui.rs | 43723 |
| 002 | 4 | user-3 | 38 / 7962 / 206 | src/ui_harness.rs | 22027 |
| 002 | 13 | user-1 | 78 / 30335 / 244 | src/ui.rs | 43697 |
| 002 | 19 | user-1 | 102 / 55047 / 280 | src/terminal_app.rs | 51593 |
| 002 | 23 | user-1 | 115 / 66949 / 308 | src/terminal_app.rs | 53644 |
| 002 | 28 | user-3 | 138 / 81305 / 868 | src/terminal_app.rs | 54140 |
| 002 | 31 | user-3 | 163 / 101958 / 981 | src/terminal_app.rs | 54438 |
| 003 | 4 | user-3 | 41 / 12653 / 212 | src/terminal_app.rs | 43136 |
| 003 | 9 | user-1 | 69 / 50685 / 192 | src/ui_harness.rs | 24100 |
| 003 | 14 | user-3 | 89 / 63040 / 482 | src/terminal_app.rs | 47624 |

All 11 are automatic structured-edit refreshes. Ten record an old-block-not-found
diagnostic; run002 trace13 records repeated file headers. There are no recorded
snapshot failures. These body bytes establish the representation actually
delivered, not a saving claim.

## Verification boundary and next finite check

Tool receipt `304825` checked the 17 selected rows and three initial launch
inputs directly, with 31 physical source SHA endpoints before/after. The
[metadata-only index](TRACE-OWNERSHIP-REVIEW.json) retains exact paths, original
status, typed IDs, full-input hashes, byte ranges and all pins. No private
reasoning or prompt body is exported. No helper, observer, extractor, accounting,
provider or test command was executed.

The next source-semantic pass is exactly these 11 rows: connect the preceding
actual edit submission and its rejected hunk, the last held digest/source and
intervening accepted changes, then the refreshed response, next actual repair
submission and accepted ACK/rejection/check. Same-thread/call ordering and
existing Git/object receipts must establish state; a repeated path, diagnostic
or subsequent successful workflow alone does not. Preserve additional reads,
repeated rejections and missing repair evidence explicitly; deduplicate any
overlapping chains. This is a bounded existing-evidence pass, not another
observation or a new collector.
