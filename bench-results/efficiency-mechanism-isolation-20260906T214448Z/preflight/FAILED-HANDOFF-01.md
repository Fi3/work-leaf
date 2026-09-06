# Retained first handoff preflights

All three first handoff diagnostics fail their observer-teardown assertion. They are retained
diagnostics, not admitted screening observations and not successful real-agent verification.
Each reaches the patch acknowledgment and command-result prompt boundaries, applies the requested
fixture edit, records successful focused validation, and receives the final `@work-leaf done`.
Each app-server invocation lacks its finalized `end.json`.

| Condition / directory | App-server invocation | Final turn |
| --- | --- | --- |
| `handoff-control` | `00000432216842466071-3200270` | `01a078be-c800-76d1-9f1e-ab48dae3c348` |
| `handoff-ack-validation-unlimited` | `00000432216849107897-3200292` | `01a078be-c99e-7392-8741-98f7840539ff` |
| `handoff-command-guidance-neutral` | `00000432216846701214-3200285` | `01a078be-cab3-72b2-ad02-0ecc3328f8af` |

Directories are relative to this report's `preflight/` directory. Each contains `smoke.log`,
`prompt-events.jsonl`, and `observation/app-server/<invocation>/` capture files.

## Direct capture evidence

For all three invocations:

- `client-to-server.raw` contains three `turn/start` requests and three `turn/interrupt` requests.
  Its final request has string ID `"8"` and targets the final turn listed above.
- `client-to-server.forwarded.raw` contains three starts but only two interrupts. It ends at string
  ID `"7"`, the final `turn/start`; interrupt `"8"` has no successful-forward capture record.
- The final `provider-usage-grace.jsonl` entry records a 1,000 ms timeout. Its outcome text is a
  scheduling decision, not proof that the subsequent write to the provider succeeded.
- The server stream ends at the final assistant item rather than the final turn-completed event.
  The app-server `server-stderr.raw` files contain no reported provider error.

## Teardown ordering

`src/codex.rs::CodexAppServer::request_turn_streaming` returns from its interruptible path immediately
after `request_interrupt`; it does not wait for the observer to forward that request or for the
provider's turn-completed event. The diagnostic's original cleanup immediately invokes
`bench-observer stop-app-server`, which signals the captured provider child.

`bench-observer/src/lib.rs::forward_app_server_client_frame` persists the usage-grace decision
**before** writing and flushing the request to the provider. It writes the forwarded capture only
after that provider write succeeds. `run_captured_process` joins all stream pumps before publishing
`end.json`; a failed pump therefore prevents finalized invocation metadata.

Together, the immediate stop and missing final forwarded request identify a teardown/held-interrupt
race. Writing the pending request after the child was terminated is the source-supported failure
path. The exact operating-system error was not preserved: the original fixture discarded the
stop command's stderr, and the backend did not print its drained proxy stderr. This report does not
invent a captured `EPIPE` diagnostic.

The earlier successful `tests/observer_subscription_smoke.rs` finishes with a noninterruptible raw
`send_streaming` follow-up. That follow-up reaches turn completion before the same teardown sequence,
so it does not leave a final held interruption at the stop boundary.

## Diagnostic-only completion criterion

A bounded fixture cleanup can wait for the exact final interrupt's successful-forward record and
its matching turn-completed event before stopping the observer, while saving stop stdout/stderr.
The match must include the typed request ID, thread ID, and turn ID; neither a grace-decision label
nor another turn's completion satisfies it. No additional provider request is necessary.

This criterion changes only the diagnostic's observation/teardown ordering. It does not alter the
observer's existing 1,000 ms/`forward` policy, the product interrupt path, experimental prompts, or
the full benchmark driver. All three failed diagnostics remain retained, and any corrected
verification uses distinct preflight identities.
