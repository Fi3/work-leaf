# Real normal-backend observer result

## Outcome

Observer integration: **passed**. Cancelled-response accounting: **incomplete**.
First benchmark batch: **not launched** (0 direct, 0 Work Leaf benchmark observations).

The single subscription-backed test passes in **11.53 seconds**. The actual Work Leaf directive
detector requests interruption of the first turn; the app server reports it as `interrupted`.
A raw follow-up in the same thread completes normally with the expected two-line answer.
The observer records one completed response with exact counters, but the interrupted response has
no response-scoped or cumulative usage. Both analyzer passes exit 2 and retain exactly one error:
`interrupted provider turn has no complete usage: count=1`.

This verifies the instrumentation and the existing interrupt/resume flow. It does **not** solve
the missing-token problem or demonstrate the accepted five-percentage-point precision target.
No repeat diagnostic or full benchmark follows this result.

## Exact configuration and command

`NORMAL-SMOKE-PROTOCOL.md` is the prelaunch protocol. `NORMAL-SMOKE-SOURCES.json` pins the source,
test, CLI launcher/native executable, and frozen observer binary digests. The separate observer
initialization in `observer-preflight-unused-001` contains no provider invocation or model turn.

Authentication is the existing ChatGPT login, confirmed locally before launch. The saved
`subscription-codex` wrapper removes API-key/alternate-endpoint variables and forces ChatGPT login,
the OpenAI subscription provider, GPT-5.5, and `xhigh`. No credential material is copied. The
observer's rollout audit confirms CLI 0.153.4, model GPT-5.5, effort `xhigh`, and the same scratch
working directory for its one captured thread; it finds no unobserved same-directory thread.

The command runs from `/home/user/src/work-leaf`:

```sh
smoke_study_dir=/home/user/src/work-leaf/bench-results/efficiency-measurement-gate-20260906
WORK_LEAF_OBSERVER_CONFIG="$smoke_study_dir/normal-workflow-smoke-001/observer-config.json" \
WORK_LEAF_OBSERVER_PRIMARY_MARKER="$(jq -r .primary_invocation_marker "$smoke_study_dir/normal-workflow-smoke-001/observer-config.json")" \
WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE=1 \
WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS=1000 \
WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME=forward \
WORK_LEAF_REAL_OBSERVER_SMOKE=1 \
WORK_LEAF_REAL_OBSERVER_CODEX_PROXY="$smoke_study_dir/normal-workflow-smoke-001/proxy-bin/codex" \
WORK_LEAF_REAL_OBSERVER_PROJECT_DIR=/tmp/work-leaf-observer-smoke.Loo6FC \
WORK_LEAF_REAL_OBSERVER_BIN="$smoke_study_dir/normal-smoke-runtime-001/bench-observer" \
env -u WORK_LEAF_CODEX_TRACE -u WORK_LEAF_OBSERVER_PARENT_INVOCATION \
cargo test --test observer_subscription_smoke \
  real_subscription_directive_interrupt_and_raw_follow_up -- --ignored --exact --nocapture
```

The run's stdout/stderr is retained in `normal-smoke-execution.log`. The test limits backend
requests to two turns, refuses mediated file/command requests, bounds execution to 180 seconds,
and stops the captured app-server child before shutting down the backend. The captured child exits
0 and its invocation has finalized start/end metadata. The scratch project remains empty. Captured
item types are only user messages, reasoning, and agent messages; no provider tool item is present.

## Observed interruption and usage

The configured grace ceiling is **1,000 ms**, with the original `forward` policy. Actual wait is
**84 ms**: resumed output releases the pending original interrupt, exactly as that policy specifies.
This is not a test of waiting a full second after forwarding cancellation.

Provider thread: `01a077ff-20c7-7be0-bdd2-76d206fbd292`.

| Turn | Provider outcome | Exact response usage |
| --- | --- | --- |
| `01a077ff-2142-7820-897c-6dd4c1a7d588` | interrupted | absent |
| `01a077ff-3d1d-79e1-a21a-7976e2bcba00` | completed | available |

The completed response ID is `resp_0f2f6987179b7cba016a9db1fc603087d2943c703c6c967e63`.
Its exact counters are input **15,828**, cached input **13,696**, uncached input **2,132**, output
**43**, including **23** reasoning tokens. Raw input plus output is **15,871**; uncached input plus
output is **2,175**. Reasoning is a subset of output, not an additional charge.

Those counts equal both the sole cumulative usage update and the final rollout usage. They are
exact for the completed follow-up and only a **lower bound for the entire two-turn diagnostic**.
Equality does not cover the earlier interrupted response. The new ledger's `matched_cumulative`
status therefore coexists correctly with whole-workflow accounting being incomplete.

## Evidence and checks

The app-server capture is under
`normal-workflow-smoke-001/app-server/00000419666713810516-2837703/`:

- `client-to-server.raw` and `client-to-server.forwarded.raw`: original and actual requests;
- `raw-response-rewrites.jsonl`: only initialization capabilities and thread-start metadata differ;
- `provider-usage-grace.jsonl`: the 1,000 ms/forward configuration and 84 ms release decision;
- `server-to-client.raw`, `frames.jsonl`, and `response-usage.json`: turn outcomes, timing, and counters.

Invocation start/end metadata pins provenance digests. The analyzer verifies both the hashes and
the allowed request transformation; no provenance error is present. `normal-smoke-analysis.json`
is the transport-only result; `normal-smoke-analysis-with-rollouts.json` includes the matching
rollout metadata and retains the same unresolved response. `normal-smoke-rollout-audit.json` has
one observed/matched thread, no missing thread, and no errors.

Formatting, warning-free clippy, and all-target/all-feature tests pass for the main crate and the
separate observer crate. The observer suite has 107 passing tests; the smoke harness has five
provider-free guard tests plus this explicitly run real test. The precision utility has 14 passing
tests. Independent reviews cover request integrity, response arithmetic, complexity, and evidence.

The earlier mechanism-report suite remains 23/24 because its committed status assertion requires
permission to correct under `AGENTS.md`. That separate readiness issue does not explain or erase
the real missing-usage result.
