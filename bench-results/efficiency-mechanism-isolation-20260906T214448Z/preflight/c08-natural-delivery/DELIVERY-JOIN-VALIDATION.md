# Pure closed-input join validation

Scope: in-memory v7 trace/request/public/native identity joins, with two exact supplied code-byte dependencies. No source/capture collector, frame-proof invocation, native tool/commit analysis, accounting or provider operation belongs to this unit. `exposure_qualified` is always false and full-frame provenance is always `upstream-full-frame-proof-required`.

## Exact cut

| File | SHA-256 |
| --- | --- |
| `join_automatic_refresh_delivery.py` | `614ca2faab8790a24ec02b20cec5dcb2997ed7ae79273ec86e5304570565a402` |
| `test_join_delivery.py` | `8b8d80419fde82e1a0a9b2ea312b258e9580538a0f6ba54cd17cb745038517d7` |
| Immutable event validator | `c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f` |
| Immutable event tests | `9ac394e07c4d5ee04c6f3d03f02bbb775ef1ac5cd7679c1d40ade9b3ab071400` |
| Immutable review primitives | `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257` |

`code_module` hashes the exact two supplied byte strings before compilation and preserves their original code filenames. It performs no file I/O. The synthetic tests read those two named code dependencies once at module initialization; every protocol payload is constructed in memory. Ordinary Python module imports provide the local fixture definitions. No cached bytecode is treated as source authority for the invoked dependency functions.

## Test-first observations

Source-only draft review identified three required guards: a repeated native item ID must preserve its whole typed payload, a reply with both result and error must stay ambiguous, and an owner anchor requires singleton trace as well as request occurrence counts. All three regressions were present in the initial RED source cut; the duplicate-anchor fixture also includes a subsequent identity ACK that must not acquire ownership.

The owner's three unit-suite commands ran from this directory with `/usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest ...`. Root used `timeout 30s /usr/bin/python3.14 -B -m unittest -v test_join_delivery test_delivery` from the same directory. Independent commands belong to their separately attributed receipts below. No test launches a subprocess or reads an actual capture fixture.

| Tool receipt | Selector | Result |
| --- | --- | --- |
| `55c005` | `-v test_join_delivery` | RED, exit1; `ModuleNotFoundError: No module named 'join_automatic_refresh_delivery'`; one loader error before implementation. |
| `6dcee1` | `-v test_join_delivery` | GREEN, exit0; 24 tests, 0.193 seconds. |
| `0ffdbf` | `-q test_join_delivery test_delivery` | GREEN, exit0; 42 tests, 0.213 seconds (24 join + 18 unchanged event tests). |
| `d86c43` (root) | Combined join/event suite | Root-reported GREEN; 42 tests, 0.210 seconds. |
| `fbebdb` (candidate_controls) | Combined join/event suite | Independently reported GREEN; 42 tests, 0.210 seconds. |
| `e47750` (candidate_controls) | Bounded synthetic JSON-type mutations | Independently reported 792 cases PASS without uncaught exceptions, 3.77 seconds. |

The final cut retains native-source input order for the safe thread inventory rather than sorting it. Per-thread occurrence ordering comes from unique native physical user lines, not source-list order or timestamps. Shared text groups store candidate/input indices once; no all-capture rescan per event or duplicated whole candidate list per ambiguous row is used. Work/memory are linear in supplied bytes plus indexed record counts under ordinary hash-table assumptions and absent an adversarial SHA collision. A detected same-hash/different-text group is ambiguous; exact text comparisons remain required.

## Preserved distinctions

Every supplied turn request remains visible as joined, rejected, missing, ambiguous or mismatched, including accepted rows lacking public/native witnesses. The pure thread census contains native source declarations and `turn/start` request thread IDs, including auxiliary and usage-less threads represented by those witnesses, without a guessed factor owner. Accepted `thread/start` responses with zero turns and no native source declaration require the later full-frame wrapper census. Successfully indexed unmatched users have individual orphan records; an invalid native source is retained as an `unavailable-source` placeholder/error, not a complete per-user invalid-source inventory. The wrapper retains the full original source population. One bad source does not erase another source's valid inputs. Native replay deduplication reaches the pinned primitive only after full typed-payload consistency is checked.

Original and forwarded complete frame populations remain counted, with safe differing-frame locators/digests. Exact turn settings and payloads must agree. The pinned `rpc_key` permits an empty string RPC identity, while mandatory upstream `prove_frames` rejects it; the pure join intentionally does not replace that stricter raw-frame gate. No synthetic metadata proof, discarded differing frame or saved valid boolean establishes the independent `prove_frames` gate. Trace writes without delivery, conflicting owners, excess repeated occurrences and cross-thread ambiguity cannot establish qualified exposure. Safe output exports identities, locators, lengths, hashes, statuses and fixed errors, not full prompts, source bodies, diagnostics, command output or reasoning.

This offline helper affects no real agent-facing runtime workflow; no new real-agent verification is claimed or needed for this pure source-consistency unit. Independent helper source review reports no blocker at the exact code/test cut above, with the explicitly attributed test and mutation checks. Natural source/frame/native membership closure, actual native actions/commit semantics and any numerical scope remain separately authorized work. Frozen validators, runtime, driver, active phase and prior reports are unchanged.
