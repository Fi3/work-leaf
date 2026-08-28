# Result

## Conclusion

Exact usage recovery works for a normal completed Codex response and for a real Work Leaf turn that
disconnects immediately after `@work-leaf done`. The final interruption pilot has one started
response, one final response record, no missing usage, and no proxy error.

The infrastructure gate is complete. The next paid collection can use this mechanism for both
direct Codex and Work Leaf, but its result is a new OpenAI API comparison rather than a correction
of the old ChatGPT-backed candidates.

## Final Interruption Pilot

Evidence is under `preflight/real-agent-interrupted-0003` and its sibling transcript.

- Work Leaf user-visible result: `@work-leaf done`, followed by `agent user-1 reported done`.
- request model: `gpt-5.5`
- request reasoning: `xhigh`
- provider model snapshot: `gpt-5.5-2026-04-23`
- recovery trigger: downstream Codex connection closed
- recovery order: upstream stream closed, cancellation requested, usage retrieved
- exact input tokens: 17,206
- exact cached input tokens: 0
- exact output tokens: 233
- exact reasoning output tokens: 111
- exact raw input plus output: 17,439
- exact uncached input plus output: 17,439
- missing or duplicate response records: none
- launcher, proxy, and command status: successful
- API key in saved artifacts: absent

The call chain producing the interruption is
`src/orchestrator.rs::DirectiveStreamInterruptDetector::observe` to
`src/codex.rs::CodexBackend::request_interrupt`. `CodexBackend::request_turn_streaming` receives an
assistant message only on Codex `item/completed`, then invokes the detector and sends
`turn/interrupt`. The proxy records that downstream disconnect explicitly as
`recovery_trigger: downstream_disconnected`.

## Other Preserved Pilots

`real-agent-completed-0001` proved exact terminal usage before interruption was tested. Its Codex
model-list request received a harmless 404 because the first proxy version handled only response
requests. The model response and exact usage still completed successfully.

`real-agent-interrupted-0001` proved the close, cancel, and retrieve sequence before the explicit
recovery-trigger field existed.

`real-agent-interrupted-0002` intentionally requested 300 sentences after the directive. Work Leaf
received the whole assistant item because Codex exposes assistant text to Work Leaf at
`item/completed`, not token by token. The run confirmed that interruption occurs at the assistant
item boundary; it is diagnostic evidence and is not benchmark data.

## Automated Checks

Four tests pass:

- exact usage for a completed response;
- exact usage after a simulated downstream disconnect;
- model-list pass-through without adding token records; and
- custom Codex wrapper routing, model pins, and credential non-persistence.

The proxy and launcher also pass Python bytecode compilation. Full repository checks are recorded
in `STATE.md` after they complete.
