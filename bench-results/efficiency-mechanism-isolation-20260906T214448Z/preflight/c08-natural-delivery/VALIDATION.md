# Pure v7 event validation

Scope: `validate_event` and `census_trace` only, on in-memory synthetic JSON values. The validator has no filesystem, subprocess, provider, runtime-executor or accounting dependency. Frozen phase/runtime/driver/helper bytes are outside its write scope. No real agent-facing workflow is affected by this offline helper; no real-agent, actual capture or natural-exposure qualification is claimed.

## Source cut

- `validate_automatic_refresh.py`: `c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f`.
- `test_delivery.py`: `9ac394e07c4d5ee04c6f3d03f02bbb775ef1ac5cd7679c1d40ade9b3ab071400`.
- Renderer/activation/guidance/normalization source identities and future delivery limitations are in `DESIGN.md`; the pure helper neither imports nor executes those sources.

## Exact development observations

Before the first recorded RED, the uncommitted fixture's guidance separator was reconciled with `src/orchestrator.rs::append_refresh_guidance`: the complete prefix is cue + newline + format guidance + two newlines + refresh header. Original draft `1628198730bbe24688ba6069c9bb1c62ecae00bbfcaaeff4a67b47c88cc5d1f8`; corrected-only draft `0d60167f165745f48f66c410588cdedf70909e64550d318c03dff06fb70f35b8` (hash receipt `0d8c87`). No failure is attributed to the earlier unexecuted draft. Further preimplementation census/identity tests produced `c1208a7eea6d15c731c65978192e5f16527c262837dcddc86b71be1002526bd0` (receipt `c43d42`).

All commands ran in this directory, with `/usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest -v ...`; no test reads a filesystem fixture or invokes a subprocess.

| Tool receipt | Test selector | Exit and observation |
| --- | --- | --- |
| `1969e2` | `test_delivery` | 1; initial missing-module RED, `ModuleNotFoundError: No module named 'validate_automatic_refresh'`, one loader error. |
| `469b66` | `test_delivery` | 0; initial implementation 14 tests PASS, 0.014 seconds. |
| `240092` | `test_delivery.EventTests.test_overlapping_components_are_rejected_before_any_body_hashing` | 1; explicit adversarial RED: overlapping components reached `body_proof` twice before rejection. |
| `485f07` | `test_delivery` | 0; final 18 tests PASS, 0.017 seconds, including prehash overlap rejection and safe malformed population retention. |

The overlap fix performs noncopying range validation and ordered baseline/candidate component checks before reading or hashing any carried body. The command-header check scans fixed separators once, without nested variable-width regex alternatives. No introduced quadratic history scan or repeated large-body hashing is claimed acceptable.

## Qualification boundaries

Tests cover patch/edit framing, both coherence spans, body/snapshot ownership, all seven representation branches, inclusive diff limits, large full-current bodies, UTF-8 and optional terminal newlines, copied marker-bearing diagnostics, failure suffix retention, strict JSON keys/types, normalized successful paths and duplicates. Identity continuation tests require typed source-owned ACK/command spans and source framing. Census tests require activation/run/process/sequence consistency and retain malformed/unknown rows under missing or invalid physical-line maps.

Available current bytes are independently FNV/length/SHA checked. Metadata-only current bodies and every previous body remain `not-carried`; no preimage or current checkout substitutes for them. Returned data contains safe counts, hashes, classes, indices and fixed errors, never prompt/source/diagnostic/reasoning bodies. `complete_delivery_proven` is always false. Even a completely valid trace is not proof of delivery, accepted patches, Git state, tool execution, complete natural conflict exposure or token savings.

Independent pure-helper review is pending. Full closed-source ownership/delivery/native membership and any numerical scope require separate authorization. No Cargo, Git, namespace/private executor, provider, live/saved capture audit or accounting call was run in this development unit.
