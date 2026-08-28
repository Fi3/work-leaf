# Point 7 Failure Analysis

## Goal

Point 7 was intended to compare fair normal direct sequential Codex with fair normal concurrent
Work Leaf using complete token measurements. A separate Work Leaf run with all three candidate
context-delivery causes disabled was also planned.

The comparison cannot produce a valid token percentage with the available Codex transport. Normal
Work Leaf interrupts a model response as soon as it emits a complete orchestrator directive. Codex
does not report token usage for that interrupted response. Waiting for usage changes what the model
does and is therefore not a fair measurement of normal Work Leaf.

## What Happened

The first normal Work Leaf attempt enabled benchmark-only code that waited up to 30 seconds for an
exact usage event before interrupting a directive response. Two reviewers emitted a file-read
directive as an intermediate message and then continued reasoning. No usage event existed at the
directive boundary. The 30-second wait timed out, the reviews failed, and the candidate never
reached linearization or final checks.

This was a measurement failure, not evidence that normal Work Leaf fails the task. The attempt is
preserved under `runs/wl-000`; its report marks both the workflow and measurement incomplete.

## Checks Performed

### Immediate interruption on Codex CLI 0.149.1

A fresh GPT-5.5/xhigh turn was interrupted immediately after `@work-leaf done`. The app server was
kept alive for 60 seconds after acknowledging the interrupt. It emitted `turn/completed` but no
`rawResponse/completed` event and no `thread/tokenUsage/updated` event for the interrupted response.

Evidence:

- `preflight/attempt-0004-immediate-interrupt-post-ack.jsonl`
- transcript SHA-256: `e422028b1fc811456ba34b352b4cc95811125eefb481e67f1e594718767ea728`
- summary SHA-256: `84480cf528d378c3262c5153824bfa75d383b8c9d78cfc39c68b3ec2fa87e701`

### Immediate interruption on Codex CLI 0.150.1

The same GPT-5.5/xhigh check was repeated with the newer Codex CLI release available from npm. It
also emitted neither exact nor cumulative usage during the 60 seconds after interruption.

Evidence:

- `preflight/attempt-0005-cli-0.150.1-immediate-interrupt.jsonl`
- transcript SHA-256: `707d18bafdda52ac03ab23393e128bfb49e596027387202da2e55c9de1885571`
- summary SHA-256: `8185794dd725d7a4ce0957ae8ef3a3f22442ba89a293a4a3f5e3d7cfbdf31667`

### Server-side usage query

Codex app server exposes `account/usage/read` for a conversation when the account's billing route
supports it. The query returned no conversation usage for both new interrupted probes and eight
older Work Leaf conversations. It cannot recover the missing measurements on this account.

### Stored and background response recovery

Two requests tested whether the same ChatGPT Codex endpoint could retain or cancel a response and
then return its usage. Neither request reached the model:

- `store=true` was rejected with HTTP 400: `Store must be set to false`;
- `background=true` with `store=false` was rejected with HTTP 400: `Unsupported parameter: background`.

The filtered request and response records are
`preflight/attempt-0006-chatgpt-stored-response.json` and
`preflight/attempt-0007-chatgpt-background-response.json`. They contain no credentials or response
content. Public Responses API cancellation cannot be substituted silently because this benchmark
uses the ChatGPT Codex endpoint and account, not the API-key transport.

### Codex source inspection

The inspected source is the official `rust-v0.150.1` tag at commit
`90854393966b21e9ebfd21b122334eb09a20c93d`.

- `codex-rs/core/src/client.rs::map_response_events` stops polling the provider when the response
  consumer is dropped by interruption. It records cancellation without token usage.
- `codex-rs/codex-api/src/sse/responses.rs` creates token usage only while handling the provider's
  `response.completed` event.
- `codex-rs/core/src/session/turn.rs` emits `RawResponseCompletedEvent` and updates cumulative usage
  only after receiving that completed event.
- `codex-rs/app-server/src/bespoke_event_handling.rs` can expose exact or cumulative usage only when
  those core events exist.

This call chain matches both real probes. There is no hidden local token total to recover after a
normal immediate interruption.

## Rejected Alternatives

- **Read longer after interrupt:** rejected by the 60-second probes; completion can arrive without
  usage.
- **Use a newer Codex CLI:** rejected by the 0.150.1 probe and source inspection.
- **Use the conversation usage endpoint:** rejected for new and historical local conversations.
- **Cancel and retrieve a stored/background response:** rejected by the same ChatGPT Codex endpoint.
- **Wait for the response before interrupting:** rejected because reviewers continue generating and
  can change their answer before Work Leaf receives the directive.
- **Use the partial cumulative total:** rejected because it omits the interrupted response and would
  systematically undercount Work Leaf only.

## Point 7 Status

Point 7 cannot establish a valid token-saving percentage with this Codex account and transport. The
direct observation completed all three features and remains valid on its own because direct Codex
receives a terminal usage record for every invocation. Its exact totals are 41,035,124 raw and
1,982,580 uncached tokens. The failed Work Leaf observation is retained as infrastructure evidence
and must not be compared with the direct total.

The all-three-disabled Work Leaf observation was not launched after this failure. It uses the same
immediate-interrupt transport and would have the same unmeasured-token problem, so it could not add
valid evidence about token savings.

Completing the intended comparison requires one of these capabilities:

1. provider or Codex telemetry that reports input, cached input, and output usage for cancelled
   responses; or
2. an explicitly approved study based on documented estimates or conservative bounds instead of
   exact token totals.

The benchmark-only exact-usage delay was removed from the active code after this result. Normal Work
Leaf again interrupts immediately when it receives a complete directive.
