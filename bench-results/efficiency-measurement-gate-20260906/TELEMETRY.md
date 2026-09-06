# Exact Usage Availability Gate

## Provider Constraint

The study uses the normal Codex subscription exclusively. The public API diagnostics below are
excluded exploratory attempts outside that constraint. Their records and helper remain for audit;
their reproduction commands are historical and must not be executed for this study. API balance
or API-key access is not a prerequisite or proposed remedy for continuing the benchmark.

## Result

`HANDOFF-RESULT.md` records a passing, isolated subscription-backed dynamic-tool experiment.
Exact response usage precedes the synthetic tool result, continuation completes normally, and
response totals reconcile. That proposed-protocol feasibility result is separate from the current
immediate-interruption accounting gate, which remains unmet.

The configured Codex CLI identifies itself as `codex-cli 0.153.4`. Its
`account/usage/read` responses contain `threadUsage: null` for each of the three
historical threads queried by the existing usage-recovery script. The command exits
successfully with status 0. The responses also contain null daily usage buckets and
null summary fields. No usable token groups are available from this check, so the
exact-accounting gate is not satisfied.

This is a query of existing conversations under the configured Codex authentication.
It launches zero model turns, performs no provider switch, and runs no benchmark.
The query succeeds without an initialization error; `codex doctor` is unnecessary.
The evidence contains usage responses and thread identifiers, with no credentials,
prompts, or model output.

`thread-usage-codex-0.153.4.json` is the machine-readable result. The accompanying
version and digest checks have a UTC recording time of `2026-09-06T09:44:10Z`.

## Reproduction

From `/home/user/src/work-leaf`:

```sh
codex --version
command -v codex
timeout 40s python3 bench-results/efficiency-point7-thread-usage-20260828T140623Z/query_thread_usage.py \
  --codex /usr/bin/codex \
  --cwd /home/user/src/work-leaf \
  --output bench-results/efficiency-measurement-gate-20260906/thread-usage-codex-0.153.4.json \
  --timeout 30 \
  01a04842-cbbf-7952-97d1-98fc8afc784a \
  01a044a5-8d25-7261-b56d-0a144a74c1d9 \
  01a0486e-f091-7dd3-a664-1179b5e8f889
```

The script refuses to replace an existing output file. A repeat requires a distinct
output name. The script sends `initialize`, `initialized`, and `account/usage/read`;
it sends neither `thread/start` nor `turn/start`.

## Evidence Identity

| Item | SHA-256 |
| --- | --- |
| `/usr/bin/codex` launcher, resolving to `/usr/lib/node_modules/@openai/codex/bin/codex.js` | `61b0194f3bb6534439c8d26a3ed57d0805f84b884588b761795323eeb92fcf70` |
| Native executable `/usr/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/bin/codex` | `56ef98ab4032d317ab26e9b5e5a175650717351edb16ed9cde0cb6d1734d62da` |
| `../efficiency-point7-thread-usage-20260828T140623Z/query_thread_usage.py` | `46e0505842dfee7392bf78942b6f99accf94f1acd5a7e255fae497ee2346aa2d` |
| `thread-usage-codex-0.153.4.json` | `0a3e788b75a44812d1cbe26491bc996a7b44ad47895bd4da8cadd36dde0cfd3c` |

## Scope and Next Gate

The historical 60-second immediate-interruption probes use Codex 0.149.1 and
0.150.1, as recorded in
`../efficiency-point7-exact-accounting-20260828T113610Z/FAILURE-ANALYSIS.md`.
Those probes establish missing cancellation telemetry on those inspected versions
and transport. This 0.153.4 check establishes only that the historical-thread usage
query returns no usable token groups under the currently configured authentication.
It does not establish the behavior of an immediately interrupted 0.153.4 model turn
or of a public Responses API request.

A usable exact-usage route requires non-null per-thread or per-response token groups
and reconciliation against completed responses before a bounded real cancellation
and resume smoke. That smoke must prove coverage of input, cached input, and output
usage for the interrupted response, preserve immediate provider interruption, and
attribute every response without duplication. A query that returns null cannot
satisfy this gate. A different provider transport is a separate experimental
condition; the current result does not authorize or validate such a substitution.

The observer ownership and interruption behavior are documented in
`../../docs/architecture.md`. Its interrupt grace setting affects timing and can
permit extra generation; it does not supply passive exact telemetry for a response
that lacks usage.

## Public API Diagnostic

The existing `OPENAI_API_KEY` environment variable is present. The environment has
no explicit `OPENAI_BASE_URL`, `OPENAI_API_BASE`, or `CODEX_BASE_URL` endpoint. A
read-only `GET https://api.openai.com/v1/models/gpt-5.5` returns HTTP 200 with model
ID `gpt-5.5`; `api-model-availability.json` records that check. Model listing access
does not establish permission or available balance for generation.

The bounded background-response diagnostic requests `gpt-5.5` with `xhigh`
reasoning and `max_output_tokens: 2048`. Its fixed synthetic input asks for integers;
it supplies no tools or repository content. It targets the official API using the
existing key in memory, rejects redirects, and never saves authorization headers or
raw provider error messages. Background generation and cancellation are documented
in the [official background-mode guide](https://developers.openai.com/api/docs/guides/background);
the requested reasoning level is supported by the
[GPT-5.5 model documentation](https://developers.openai.com/api/docs/models/gpt-5.5).
These capabilities do not imply complete usage for a cancelled response on this
account.

`api-cancel-attempt-001.json` records one request reaching `response.in_progress`
and then `response.failed` before any output-text delta. Its usage is null. The
first diagnostic retains only a response-ID digest and omits the terminal error
object; it cannot support a diagnosis of the failure or a subsequent retrieval by
ID. The preserved record states this evidence limitation.

`api-cancel-attempt-002.json` records the single follow-up diagnostic authorized to
resolve that omission. It retains response ID
`resp_007123270208bbd0006a9d37fb89b087d2a47171e7bc072318` and actual model
`gpt-5.5-2026-04-23`. The response fails with error code
`credit_balance_exhausted`, null usage, and no output-text delta. Three bounded
retrievals return the same terminal failure. The cleanup cancellation request
returns HTTP 400 with error type `invalid_request_error` after the response is
already terminal. The evidence does not exercise successful cancellation during
generation. That API request is blocked by balance. The entire API route is outside the
subscription-only study regardless of balance, and no further API attempt is permitted.

There are two API generation attempts in this diagnostic sequence. Neither is a
benchmark run or a substituted Codex provider. The second request finishes in
14.412 seconds. Its HTTP calls use a 15-second socket timeout, its generation-read
loop has a 45-second deadline, and cleanup includes cancellation and at most three
retrievals. Cleanup is outside that read-loop deadline. The reproduction commands
below include a 180-second outer bound around the entire operation.

`api_probe.py` implements the bounded diagnostic. The cancellation gate requires a
recorded in-progress event, an output-text delta, final status `cancelled`, and
non-null integer input, cached-input, output, and total token counts. Cached input
cannot exceed input; total must equal input plus output; a supplied reasoning
count must fit within output. A completed, failed, zero-work, or arithmetic-invalid
response cannot pass. The gate is a diagnostic check, not an integrated benchmark
accounting adapter or proof of full workflow/resume coverage.

`test_api_probe.py` contains four provider-free tests covering evidence redaction,
response identity and safe error retention, actual-cancellation prerequisites,
and token arithmetic. The tests fail before the helper exists; six arithmetic
cases also fail before their gate fix. All four tests pass after implementation.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 bench-results/efficiency-measurement-gate-20260906/test_api_probe.py
# Read-only access check, using a distinct output name:
timeout 180s python3 bench-results/efficiency-measurement-gate-20260906/api_probe.py model \
  --output bench-results/efficiency-measurement-gate-20260906/api-model-next-check.json
# Historical reproduction of the excluded diagnostic; not authorized for this study:
timeout 180s python3 bench-results/efficiency-measurement-gate-20260906/api_probe.py cancel \
  --output bench-results/efficiency-measurement-gate-20260906/api-cancel-next-attempt.json
# Read-only retrieval of the identified failed response:
timeout 180s python3 bench-results/efficiency-measurement-gate-20260906/api_probe.py retrieve \
  --response-id resp_007123270208bbd0006a9d37fb89b087d2a47171e7bc072318 \
  --output bench-results/efficiency-measurement-gate-20260906/api-retrieve-next-check.json
```

## Current Codex Immediate-Interruption Smoke

`codex-0.153.4-immediate-interrupt.json` records a real configured-agent turn on
Codex 0.153.4, resolving model `gpt-5.5` and reasoning effort `xhigh`. The model emits
the requested complete commentary directive at 7.540 seconds. The diagnostic sends
`turn/interrupt` at the same recorded time, without a usage grace interval. At
7.550 seconds the app server acknowledges the request, changes the thread to idle,
and emits `turn/completed` with status `interrupted`. No `rawResponse/completed`
event precedes the interrupt. Neither exact-response nor cumulative-thread usage
arrives during the following 60 seconds.

The smoke finishes in 67.566 seconds and uses one model turn. It is a failed
exact-accounting gate on the currently configured transport. No initialization or
authentication error occurs. There is no resume probe because the interrupted
response lacks the usage required by this gate. No benchmark batch is launched.

The parent diagnostic has a 180-second shell timeout. Its per-request response
waits are bounded to 15 seconds, its directive wait to 45 seconds, its post-acknowledgement
drain to 60 seconds, and process cleanup to 5 seconds. The exact provider command is:

```sh
/usr/bin/codex --disable apps --cd /home/user/src/work-leaf \
  --sandbox read-only --ask-for-approval never \
  -c 'model="gpt-5.5"' -c 'model_reasoning_effort="xhigh"' \
  app-server --listen stdio://
```

The diagnostic removes the same child environment variables listed by
`src/codex.rs::REMOVED_CODEX_CHILD_ENV`. It sends the following requests over stdio,
waiting for each response before the dependent request. The thread and turn IDs
come from the corresponding start responses:

```json
{"id":"init","method":"initialize","params":{"clientInfo":{"name":"work_leaf_usage_gate","title":"Work Leaf usage gate","version":"1"},"capabilities":{"experimentalApi":true}}}
{"method":"initialized"}
{"id":"thread","method":"thread/start","params":{"approvalPolicy":"never","cwd":"/home/user/src/work-leaf","sandbox":"read-only","model":"gpt-5.5","experimentalRawEvents":true}}
{"id":"turn","method":"turn/start","params":{"threadId":"<thread-id>","model":"gpt-5.5","effort":"xhigh","approvalPolicy":"never","sandboxPolicy":{"type":"readOnly"},"input":[{"type":"text","text":"Send a commentary message containing exactly @work-leaf done. Then, without using tools or reading or modifying files, list the integers from 1 to 1000 in your final answer."}]}}
{"id":"interrupt","method":"turn/interrupt","params":{"threadId":"<thread-id>","turnId":"<turn-id>"}}
```

The interrupt is triggered by the complete `agentMessage` whose trimmed text is
exactly `@work-leaf done`. The diagnostic retains method timing, identifiers, turn
status, and usage fields; it omits message content and raw stderr. It keeps the app
server alive for the bounded post-acknowledgement drain before cleanup. This
direct current-version observation supplements the historical probes; it is not
an inference that all future providers or Codex versions must behave identically.
