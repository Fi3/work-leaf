# Supplemental interrupted-tail audit

This is a post-hoc, conditional accounting supplement for the completed Work Leaf observation. The frozen primary result remains **`unbounded_accounting_gap`**: three tails satisfy its predicate and one does not. Neither this document nor its [machine-readable evidence](supplemental-tail-audit.json) replaces the frozen result, recovers exact missing usage, or establishes quality equivalence. Direct was still running at the time of this audit; this document makes no pairwise saving claim.

## Frozen result and separate conditional envelope

The recorded lower totals are 22,786,969 raw input-plus-output tokens and 1,719,449 uncached-input-plus-output tokens. The strict observer reports only `interrupted provider turn has no complete usage: count=4`, with all 36 invocations complete. The frozen helper records 76 turn starts, 242 unique completed raw response IDs, and no additional inventory errors; its call inventory is explicitly non-exhaustive.

All four tails belong to thread `01a07830-f0f9-7301-abbf-ee60f8ae9377`.

| Interrupted turn | Frozen tail predicate | Supplemental response upper count |
| --- | --- | --- |
| `01a07830-f16c-7ad2-b896-889ba67052b3` | Pass: 1 | 1, conditional |
| `01a07831-274a-7de1-b368-44f60b499ffe` | Reject: unknown | 1, conditional |
| `01a0783f-554e-7ec3-bb8b-c5b9a7ebbfb7` | Pass: 1 | 1, conditional |
| `01a0783f-769b-7a02-b9e7-81b5cc792567` | Pass: 1 | 1, conditional |

The separate post-hoc interpretation permits multiple assistant output items within the rejected tail. It assumes complete visibility of normal response/tool/compaction boundaries, no hidden retries or opaque upstream subresponses, and **no unobserved preemption**. Under those assumptions, each of the four tails contains at most one unaccounted response. This is conditional cardinality reasoning, not an exhaustive provider-call proof.

The frozen model policy supplies a deliberately loose per-response ceiling of 1,178,000 = 1,050,000 input + 128,000 output tokens, conditional on the captured subscription provider honoring the documented limits for the same gpt-5.5 model. It does not reuse the historical 386,400 ceiling or mistake the CLI compaction budget for a hard upstream input limit. See the [frozen policy](infrastructure/evidence/bench-results/efficiency-measurement-gate-20260906/BATCH-BOUND-POLICY.json) and [official model limits](https://developers.openai.com/api/docs/models/gpt-5.5).

| Metric | Recorded lower | Conditional post-hoc upper |
| --- | ---: | ---: |
| Raw input + output | 22,786,969 | 27,498,969 |
| Uncached input + output | 1,719,449 | 6,431,449 |

Each upper is its lower plus 4 × 1,178,000 = 4,712,000. The secondary envelope uses the same conservative cap because cache coverage for missing responses is unknown. These are accounting envelopes, not sampling confidence intervals or unconditional billing guarantees.

## Why the fourth tail fails the frozen predicate

All sequence numbers below are zero-based records in the pinned `server-to-client.raw`; physical line = sequence + 1. For turn `01a07831-274a-7de1-b368-44f60b499ffe`:

| Sequence | Evidence |
| --- | --- |
| 26316–26319 | A completed raw response, command item start/end, then cumulative usage form the prior accounted boundary. |
| 26326–26514 | A reasoning item starts and completes. |
| 26515–26571 | An ordinary commentary message starts and completes at 19:33:47.310 UTC. |
| 26572–26579 | Another reasoning item starts and completes. |
| 26580–26630 | The directive commentary message starts and completes at 19:33:56.738 UTC. |
| 26631–26632 | An unfinished reasoning item starts; cumulative usage repeats the prior total and last usage exactly. |
| 26636 | The turn completes interrupted at 19:33:56.764 UTC. |

Between sequences 26320 and 26636, the same-thread segment contains no completed raw response, tool/compaction boundary, foreign-turn event, warning/retry event, or unknown method. Four pre-directive items are paired, including **two** completed assistant messages. The frozen `tail_proof` requires exactly one completed assistant message, so it rejects this segment. The stale cumulative total 1,165,015 is not coverage for it.

Two message items alone do not prove two model responses. Conversely, absent a complete upstream request inventory, this stream does not unconditionally disprove hidden extra requests. The post-hoc conclusion therefore remains explicitly conditional.

## Same-capture identity and token-sum checks

Nine exact completed responses in this same capture attribute positive output tokens to two distinct completed assistant items. Each item matches the response's thread and turn, and each response ID is unique. The JSON evidence retains all response IDs, item IDs, identities, phases, event positions, and counts. In every row, the two attributed message outputs plus the separately reported reasoning output equal the response's entire output count.

| Response ID | Completed-response physical line | Message outputs + reasoning = response output |
| --- | ---: | --- |
| `resp_03fee1970559bc89016a9dbbfb5e2087d2952ae9134b5e95e5` | 926 | 61 + 37 + 0 = 98 |
| `resp_0a14ac765075b68e016a9dbcb5d7c487d29bf688bd64641fd3` | 3769 | 58 + 1729 + 1034 = 2821 |
| `resp_03fee1970559bc89016a9dbc64ffe887d295919db2bba583a5` | 9816 | 67 + 4097 + 10855 = 15019 |
| `resp_0df7b469066abcd8016a9dbe0719f087d2a441588788ddbdc8` | 20869 | 55 + 9 + 1190 = 1254 |
| `resp_03fee1970559bc89016a9dbd76b47c87d28294ed77b14dd34c` | 22993 | 64 + 4086 + 6207 = 10357 |
| `resp_03fee1970559bc89016a9dc1a948d887d2a7d06163923d548a` | 36304 | 60 + 1111 + 2588 = 3759 |
| `resp_0c79ac7c5de691aa016a9dc2200be887d2a53decfb24bea5ac` | 39225 | 49 + 136 + 0 = 185 |
| `resp_0c79ac7c5de691aa016a9dc2451af887d2ae0ef3c983e05b48` | 39697 | 72 + 119 + 1021 = 1212 |
| `resp_03fee1970559bc89016a9dc352d99087d2a1973b8307dd6792` | 43559 | 70 + 85 + 516 = 671 |

These examples establish the concrete message/response distinction in the installed CLI and actual capture. Their phases are commentary plus final answer; they do not themselves prove the cardinality of the disputed two-commentary tail.

## Source semantics and limitations

In the independently fetched [Codex rust-v0.153.4 turn loop](https://github.com/openai/codex/blob/rust-v0.153.4/codex-rs/core/src/session/turn.rs), the `run_turn` documentation begins at physical line 143; `client_session.stream` is at 2259; successive `OutputItemDone` handling is at 2347; `Completed` handling is at 2591 followed by the completion break at 2632. A stream can therefore contain several completed output items: an assistant item boundary is not itself a request boundary. The [Responses reference](https://developers.openai.com/api/reference/cli/resources/responses/methods/create) likewise defines response output as a variable-length array.

The `preempt_for_mailbox` path at physical line 2451 can exit before `Completed`. This prevents elevating the normal-loop interpretation to an unconditional proof: unobserved preemption, hidden retries, and opaque provider subresponses must remain excluded assumptions. The source review has no retained local source snapshot hash; the tag, exact URL and reviewed line anchors are recorded separately from hash-pinned local capture evidence.

A read-only native-log check for the same thread and submission in the uncovered time window found one `/responses` authentication-start record and no WARN/ERROR among 11 selected rows. Its local source annotations include `core/src/session/turn.rs` and `core/src/stream_events_utils.rs:419`. The selected row IDs are retained in JSON, without raw bodies or credentials. This is corroboration only: authentication logging and printed span names are not an exhaustive call inventory.

## Pins and safe replay

The JSON `source_sha256` map pins the report, strict analysis, both original capture directions, actual forwarded client bytes, rewrite decisions, grace records, response ledger, frozen helper, frozen model policy, and first-batch manifest. This supplement has no authority to alter those inputs.

From this batch directory, the following provider-free query checks all pins, reruns the frozen classification, verifies the four-tail arithmetic, and independently reproduces the nine identity-scoped multi-item examples. It prints only counts, status, and bounds; no message text, reasoning text, credentials, or native-log bodies.

```sh
python3 -B - <<'PY'
import hashlib, importlib.util, json
from pathlib import Path
b = Path.cwd()
e = json.loads((b / "supplemental-tail-audit.json").read_text())
for rel, expected in e["source_sha256"].items():
    assert hashlib.sha256((b / rel).read_bytes()).hexdigest() == expected, rel
g = b / "infrastructure/evidence/bench-results/efficiency-measurement-gate-20260906"
a = b / "runs/work-leaf-001/work-leaf-001-three-feature-bench-artifacts"
c = a / "observation/app-server/00000422153032163608-2899396"
spec = importlib.util.spec_from_file_location("frozen_tail_audit", g / "batch_analysis.py")
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
server = list(m.records(c / "server-to-client.raw"))
inv = m.inventory(list(m.records(c / "client-to-server.raw")), server,
                  list(m.records(c / "provider-usage-grace.jsonl")))
r = m.accounting(m.read_json(a / "observation/analysis.json"), inv,
                 m.read_json(g / "BATCH-BOUND-POLICY.json"))
assert r["status"] == e["strict_replay"]["status"] == "unbounded_accounting_gap"
assert inv["gaps"] == e["strict_inventory"]["gaps"]
assert sum(x["response_count_upper"] == 1 for x in inv["gaps"]) == 3
assert len(inv["gaps"]) == 4
seen, items, examples = set(), {}, []
for seq, event in enumerate(server):
    p = event.get("params") or {}
    item = p.get("item") or {}
    scope = p.get("threadId"), p.get("turnId")
    if event.get("method") == "item/completed" and item.get("type") == "agentMessage":
        items[item["id"]] = (scope, seq, item.get("phase"))
    if event.get("method") != "rawResponse/completed":
        continue
    rid = p["responseId"]
    assert rid not in seen
    seen.add(rid)
    attrs = (((p.get("usageMetadata") or {}).get("metadata") or {})
             .get("attribution") or {}).get("items") or {}
    outputs = []
    for ident, token_usage in attrs.items():
        count = token_usage.get("output_tokens")
        if ident in items and type(count) is int and count > 0:
            identity, done, phase = items[ident]
            assert identity == scope
            outputs.append(dict(item_id=ident, output_tokens=count,
                                completed_sequence=done, phase=phase))
    if len(outputs) > 1:
        u = p["usage"]
        subtotal = sum(x["output_tokens"] for x in outputs)
        examples.append(dict(response_id=rid, raw_response_sequence=seq,
            thread_id=scope[0], turn_id=scope[1], output_tokens=u["outputTokens"],
            reasoning_output_tokens=u["reasoningOutputTokens"],
            assistant_item_output_sum=subtotal,
            output_equals_item_sum_plus_reasoning=
                u["outputTokens"] == subtotal + u["reasoningOutputTokens"],
            items=outputs))
assert examples == e["multi_message_responses"]
assert len(examples) == 9
assert all(len(x["items"]) == 2 and x["output_equals_item_sum_plus_reasoning"]
           for x in examples)
s = e["supplemental"]
assert s["added_upper"] == 4 * s["per_response_raw_upper"] == 4712000
for metric, bounds in s["bounds"].items():
    assert bounds["lower"] == r["recorded_usage"][metric]
    assert bounds["upper"] == bounds["lower"] + s["added_upper"]
print(json.dumps(dict(frozen_status=r["status"], exact_multi_item_responses=len(examples),
                     supplemental_status=s["status"], conditional_bounds=s["bounds"])))
PY
```

This documentation-only audit does not change the benchmark protocol, helpers, product runtime, captured data, run count, or quality outcome.
