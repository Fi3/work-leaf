# Observer accounting result

This is the retained result of the earlier observer-only telemetry attempt. Current first-batch
authority and the accepted raw-primary accounting rules are in `PROTOCOL.md`; current execution
status is in `BATCH-RESULT.md`. The historical five-point cutoff below is not a current launch gate.

## Telemetry-attempt decision

First benchmark batch: **not launched**. The accepted target is at most five percentage points of
accounting uncertainty for both raw-token and uncached-input-plus-output saving. That target is not
demonstrated. Quality and sampling uncertainty are separate requirements.

## Historical replay

`accounting_precision.py` evaluates arbitrary nonnegative Work Leaf and positive Direct token
bounds, including uncertainty in the denominator. `historical-response-audit.json` records the
source paths and SHA-256 digests for the six historical normal Work Leaf observations and the
comparison evidence. Its results are reproducible without provider calls.

| Saving metric | Historical interval | Width before | Width after replay | Target |
| --- | --- | --- | --- | --- |
| Raw input plus output | 45.3834% to 51.6244% | 6.24094 pp | 6.24094 pp | ≤5 pp |
| Uncached input plus output | −123.6201% to 16.4920% | 140.11206 pp | 140.11206 pp | ≤5 pp |

All 35 missing responses remain unresolved. Thirty-two have a later cumulative advance equal to
the later response's own usage; three have no later advance. None of the six saved streams contains
`rawResponse/completed`. No additional counters can be recovered from those absent records.

## Instrumentation and limits

The external observer supports opt-in response-ID capture and a strict completed-response ledger.
`bench-observer/src/raw_capture.rs` owns metadata request instrumentation;
`bench-observer/src/response_usage.rs` owns response validation, deduplication, and reconciliation.
The ledger is never added to cumulative workflow totals. A cancelled tail stays unresolved even
when the completed response prefix matches cumulative usage exactly.

No product implementation in `src/`, model-facing prompt policy, normal interrupt detector, grace
decision rule, or benchmark task is part of this implementation. The observer's existing 1,000 ms
pre-forward grace and release on resumed output remain the diagnostic configuration. Metadata
traffic has overhead and is recorded as instrumentation, not assumed to be timing-neutral.

Official OpenAI documentation informed the app-server capability check; the installed CLI schema
defines the experimental notification fields. The [app-server guide](https://learn.chatgpt.com/docs/app-server)
documents subscription authentication and usage notifications, but does not establish exhaustive
cancelled-response accounting. No public API generation or authentication fallback is authorized.

## Real subscription verification

`NORMAL-SMOKE-RESULT.md` records the one real subscription check. The actual Work Leaf detector
interrupts the first turn, and the raw same-session follow-up completes normally in an 11.53-second
test. The original 1,000 ms/forward policy releases the interrupt after 84 ms of resumed output.
The observer captures exact follow-up usage (15,871 raw tokens, 2,175 uncached input plus output)
and reconciles it to cumulative and rollout totals. **The interrupted response remains missing.**
Whole-workflow analysis is incomplete with one unresolved response, despite the completed-response
ledger matching perfectly. Instrumentation integration passes; the accounting precision gate does not.

## Checks and remaining gate

The ledger's 13 tests include four regressions for review findings: directional RPC-ID collisions,
typed IDs, and cumulative regressions. The capture's 18 tests include six provenance checks.
Failing assertions were reproduced before implementation. The precision utility has 14 passing tests.

Both Rust crates' required formatting, clippy, and all-target/all-feature tests pass, including the
provenance checks and bounded real-test harness. The mechanism-report assertion has explicit user
approval and requires `incomplete_normal_endpoint_measurement`; all 24 study tests pass.

Metadata is not a claim of improved token totals or narrower bounds. This attempt did not satisfy
its original precision target and admitted no benchmark observations. It does not authorize a
runtime redesign. The separately authorized bounded raw-primary pilot retains this limitation.
