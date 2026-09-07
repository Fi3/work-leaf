# Workflow 011: visual test-first response/input-charge trace

This descriptive episode was selected from the frozen semantic review before examining its
token values. It is not selected by cost ranking. The phase primary test did not establish
a work-unit treatment effect; this trace supplies no causal savings share or counterfactual price.

## Scope and identity

The same thread, `01a079ef-4240-71b0-b2ce-42661747c9d2`, contains five selected accepted turns:
rebased test-only submission → ACK2/focused check → expected failing-check result and first
implementation → ACK3/focused check → passing-check result/done.
The prior rejected test generation and later review are outside this scope. Earlier retained
context is included wherever the selected responses charge it.

The frozen labels for ACK2 and ACK3 are both `remaining-work`. The acknowledged bodies are
at server lines **4866** and **25304**; the actual check directives are at **6009** and **25340**,
the delivered results at client lines **41** (status 101; two missing-mode assertions) and
**52** (status 0; both tests pass), and done at server line **25358**.
Client lines **37, 39, 41, 50, 52** are typed-string RPC IDs **36, 38, 40, 49, 51**,
respectively. Exact thread/turn/item IDs, request hashes, accepted replies and source locators
are retained in the JSON, not joined by timing or commit proximity.

## Matched completion records

R1–R8 below correspond to zero-based `responses[0..7]` in the
[complete charge inventory](WORKFLOW-011-VISUAL-TEST-FIRST-CHARGES.json).
All eight response IDs independently match `WORK-UNIT-TOKENS.json` and
`INPUT-ATTRIBUTION.json` and their exact `rawResponse/completed` source records.

| Record | Accepted input RPC | Raw server line | Input | Cached input | Output |
| --- | --- | ---: | ---: | ---: | ---: |
| R1 | 36 | 4252 | 87,520 | 81,280 | 5,070 |
| R2 | 36 | 4299 | 92,647 | 87,424 | 145 |
| R3 | 36 | 4867 | 94,382 | 92,544 | 621 |
| R4 | 38 | 6018 | 90,212 | 87,424 | 545 |
| R5 | 40 | 8846 | 90,850 | 89,472 | 2,377 |
| R6 | 40 | 25305 | 93,469 | 90,496 | 17,802 |
| R7 | 49 | 25341 | 98,701 | 90,496 | 27 |
| R8 | 51 | 25359 | 99,050 | 98,688 | 9 |

Observed sums for these IDs only: **746,831 input + 26,596 output = 773,427 raw tokens**.
Cached input is 717,824, leaving 29,007 uncached input; uncached input plus output is 55,603.
Reasoning-output count 18,472 is already part of output, not an additional charge.
These are realized accounting totals, not the cost that an alternative workflow would save.

Every selected response separately charges request fields: `tools` **5,926 input**
and `instructions` **4,348 input**, all cached here. Their eight-response sum is
**82,192 input**, already included in the totals above. The other input charges have exact
item identities. Top-level item charges and request fields reconcile to every response with
zero residual; nested content is not counted again.

## Repeated input, not repeated execution

The inventory retains **554 input-charge rows across 86 exact native-linked item identities**;
79 identities are charged in multiple selected responses. Indices and every per-response
input/cached/cache-write amount are preserved, not just these examples:

- The accepted test-only message `msg_0c1c83c94a0a57f2016a9e32b3216487d2afe4695f3b1751e0`
  is generated in R3 and charged **443 input tokens** in each of R4–R8.
  Those charges are uncached in R4/R5 and fully cached in R6–R8.
- The accepted implementation message `msg_0c1c83c94a0a57f2016a9e33b7c41887d2b253492b650d1fc9`
  is generated in R6 and charged **7,092 input tokens** in each of R7/R8:
  uncached in R7, cached in R8.
- The first `function_call_output` identity in chronological input order,
  `fco_01a079ef-a8e0-7380-b5b6-c264d143937f`, is charged **10,287 cached input tokens**
  in each of R1–R8. Its exact native link is at line 31 of `native_sources[0]`.
  This is repeated billing of one recorded result identity, **not evidence of eight executions**.

Some message IDs also have small input-wrapper charges in the response emitting their output.
The inventory preserves those values rather than calling them a replay of a complete earlier
message. Metadata alone does not establish unchanged text or tokenization in every charge.
Only safe native metadata (type/role/tool/call identity, source, line and payload hash) is
projected; no native message, tool-argument/result, or private reasoning bodies were inspected.

## Interruption and limits

All five selected turns end with `status: interrupted`. Each exact typed interrupt appears
in both original and forwarded client captures. The unchanged 1,000 ms usage-grace mechanism
records exact usage before forwarding after **21, 31, 27, 27 and 48 ms**, respectively.
The frozen conditional-gap inventory has **no row matching these five turns**. No missing
charge is assigned zero and no bound is revised. These facts do not prove there can be no
hidden response, omitted event, retry, or activity outside this episode, nor that interruption
has no effect in every workflow.

This realized path shows separate continuations charging retained input and request fields,
including cached context, while accepted test and implementation messages themselves become
later input. It does not identify the share of the historical workflow difference caused by
test-first packaging, the policy intervention, or any other factor.

## Provenance and checks

JSON SHA256: `e3b62774fadb50b5bedb3adee6a77ec69485bcfc872b9e0e603b3b8acec496c2`.

Its `source_sha256` map preserves 13 source endpoints: the two accounting reports,
frozen classifications and freeze receipt, reviewed 011 draft/review/packet, terminal receipt,
original/forwarded client capture, raw server capture, grace journal and selected native rollout.
Checks reconcile all eight response identities, all 554 charge rows, all 86 native metadata
links and all endpoint hashes. The selected classifications match the frozen file.
No provider call, source/helper change, aggregate reanalysis or counterfactual retotal is involved.
