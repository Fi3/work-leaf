# Subscription tool-handoff result

## Verdict

The single synthetic handoff passes `HANDOFF-PROTOCOL.md` on Codex CLI 0.153.4 with the existing
ChatGPT subscription login, `gpt-5.5`, and `xhigh` reasoning. The exact usage of the response
containing the tool call arrives before the client returns the tool result. Codex then produces the
expected final answer, reports exact continuation usage, and completes normally.

The diagnostic sends one user turn, receives one valid dynamic tool request, and observes two
upstream responses. It sends no interruption, changes no Work Leaf runtime or public interface,
and launches no benchmark batch. No API-key model request or credential copy is part of this check.

## Observed ordering

Elapsed seconds are measured from diagnostic initialization; the full duration including cleanup
is 8.954498 seconds. These timestamps describe one run, not a latency estimate.

| Event | Elapsed seconds |
| --- | ---: |
| Subscription authentication verified | 1.369762 |
| Raw function-call item received | 5.251510 |
| Client receives `item/tool/call` | 5.256514 |
| Exact tool-call response usage received | 5.315033 |
| Client sends the synthetic tool result | 5.315203 |
| Dynamic tool completes successfully | 5.319021 |
| First cumulative thread-usage notification | 5.326088 |
| Expected final answer received | 8.873088 |
| Exact continuation-response usage received | 8.883291 |
| Final cumulative thread-usage notification | 8.887475 |
| Turn completes normally | 8.890274 |

The recorded result-withholding interval is 0.058597 seconds, well inside the 10-second limit.
The pre-result accounting signal is `rawResponse/completed`; the first cumulative
`thread/tokenUsage/updated` notification occurs after the result in this trace. The raw function
call and callback share `call_WAbBaNbNEFWFrF4jS1bqDkJf`, which is associated with the first exact
response completion. An earlier unrelated response is not used as proof of tool-call accounting.

The final text matches `WORK_LEAF_SUBSCRIPTION_HANDOFF_OK`. The normally completed turn, absence
of interruption, and process exit status 0 distinguish this outcome from the historical probe that
received exact response usage but did not finish its turn.

## Token reconciliation

| Response | Input | Cached input | Uncached input | Output | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tool call | 9,559 | 1,408 | 8,151 | 19 | 9,578 |
| Continuation | 9,599 | 8,576 | 1,023 | 13 | 9,612 |
| Sum and final thread total | 19,158 | 9,984 | 9,174 | 32 | 19,190 |

Both responses have valid exact usage and distinct response identities. Their sums equal the final
thread totals in all five mandatory fields. Provider-reported reasoning output is zero in both
responses; reasoning is a subset of output, not an additional token total. Uncached input plus
output is 9,206 tokens. None of these diagnostic counts represents a benchmark saving.

## Evidence and reproduction

`subscription-handoff-attempt-001.json` contains the complete filtered event record, exact command,
synthetic prompt, bounds, model/authentication confirmations, usage, and classification.
It retains no account email, credential, raw reasoning, or arbitrary tool content.

- Thread: `01a077c6-0e32-7933-a8e6-ca8b25492300`.
- Turn: `01a077c6-0e6e-71b1-8f1c-672a9143058d`.
- Helper SHA-256: `eaa8257ab896f873b8c14f3c3ededa56b830da0aac82177aca2d0d22c6b02dda`.
- Frozen protocol SHA-256: `5ba07591a233533aabb8f5e2f346eee93fe2601e2cd323f1e19dc67981916e94`.
- Result SHA-256: `fac01a9d3c0cae31e1095c03fc831754d877aa07d545a0208b61f5a2c7117cad`.
- Native Codex SHA-256: `56ef98ab4032d317ab26e9b5e5a175650717351edb16ed9cde0cb6d1734d62da`.

The executed command, from the repository root, is:

```sh
timeout --kill-after=25s 180s python3 -B \
  bench-results/efficiency-measurement-gate-20260906/handoff_probe.py \
  --codex /usr/bin/codex \
  --output bench-results/efficiency-measurement-gate-20260906/subscription-handoff-attempt-001.json
```

The helper refuses to replace an existing output. This command is provenance, not authorization
for another run. The diagnostic's scratch directory is retained; no repository files are supplied
to its model. Diagnostic-only feature restrictions are recorded in the command and protocol.

## Verification and limits

All 23 provider-free tests in `test_handoff_probe.py` pass. The initial authentication, environment,
and arithmetic tests failed before the helper existed. Further regression tests failed before
the schema, event buffering, identity scoping, and strict success checks were corrected. An
independent review confirms the recorded event ordering, reconciliation, and helper identity.

`cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-targets --all-features` pass after adding this diagnostic. These Rust checks cover
the unchanged product; the real subscription trace is the evidence for this new diagnostic.
The unrelated outstanding committed report-test expectation remains documented in `VERIFICATION.md`.

This is direct positive evidence for one synthetic subscription-preserving handoff. It does not
establish concurrent/parallel tool semantics, full orchestrator integration, general reliability,
restart/resume recovery, timeout/cancellation accounting, or recovery of historical missing counts.

A product integration would replace routine text-directive interruptions with structured tool
requests and results through a provider-neutral runtime contract. That is a separate design and
architecture decision requiring human approval, tests, and real Work Leaf verification. A benchmark
of that resulting workflow would measure a different protocol from the historical interrupted
workflow. The current exact-accounting gate in `PROTOCOL.md` is not satisfied by this synthetic
success and remains unchanged. Collection is paused with zero first-batch benchmark runs.
