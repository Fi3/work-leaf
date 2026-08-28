# Preflight

The implementation and observer regression suites reproduce the old interrupted-turn undercount and
exercise the corrected exact-response accounting. The complete repository and observer formatting,
Clippy, and test suites pass.

Attempt 1 started a GPT-5.5/xhigh app-server thread with exact raw events enabled and interrupted it
immediately after a complete Work Leaf directive. The server emitted six exact raw response-item
events and acknowledged the interrupt, but it emitted neither `rawResponse/completed` nor
`turn/completed`. Exact usage therefore cannot be recovered after the immediate interrupt. The
transcript is `preflight/attempt-0001-immediate-interrupt.jsonl`; its SHA-256 is
`a6400444886131499c74c1455ec804558b728c8dad0bde64885c049ea53c10dd`.

Attempt 2 tests the narrower counter-hypothesis that exact usage is emitted if the already-complete
directive response is allowed to finish its protocol notifications. The model is not asked for more
work and no additional turn is started. If this succeeds, benchmark telemetry can wait for response
completion after detecting the directive without changing prompts, tools, generated content, or
validation behavior.

Attempt 2 emitted one non-null exact usage event immediately after the completed directive. It then
waited indefinitely for a later `turn/completed` notification and was stopped. The useful finding is
preserved, but the attempt is not a passing smoke. The transcript is
`preflight/attempt-0002-wait-for-completion.jsonl`; its SHA-256 is
`e0fc50c3dae0ed1d140c3f5807351945b2ce7bb26fdb9fa5cf2362d81fc13ecf`.

Attempt 3 uses the bounded benchmark sequence: wait only for the exact usage event after the complete
directive, then send the normal interrupt. It does not wait for `turn/completed`.

Attempt 3 passed with GPT-5.5/xhigh. One non-null exact event reported 14,742 input tokens, 9,600
cached input tokens, 35 output tokens, and 24 reasoning output tokens. The interrupt was sent and
acknowledged after that event. The transcript is
`preflight/attempt-0003-interrupt-after-usage.jsonl`; its SHA-256 is
`82be11f1c2d2bb8babb4f66cb816323ddfdfaf3b413847ab910a7e59162bf7e0`.
