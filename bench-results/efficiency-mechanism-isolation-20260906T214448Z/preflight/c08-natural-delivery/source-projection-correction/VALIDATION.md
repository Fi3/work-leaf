# Separate end-projection qualification

Status: six correction tests and all 73 original source/pure tests pass.
Independent review and a separately frozen corrected source scope remain
required before any actual run. No actual capture/native payload audit,
observer/extractor/accounting replay, provider call, Cargo job or private
executor was run for this correction.

## Exact source and scope

- Corrected helper: `7ea311907b975f25b02215c3e750eb8d69b49311c30247db6967862fe44ed092`.
- Correction tests: `b2fcf172c2d30690c3e15d3f7fa30eaed6a946aa8455f5e6ddc6a5e073dc43c3`.
- Original helper remains `d05e7e7daa62601782fbcca2058f8c31a81d29ac5301407643474a09f4889f80`.
- Original tests remain `25c532f240bc49f070f318029a967e531d15e6ce1a055923e6fa94ec62470502`.

The source diff (`38e9e7`) contains the separate module description/dependency
base, an explicit original-helper source pin, one indexed capture-reference map,
and the typed end projection. Dependencies retain their exact original paths;
the derivative's own source path is separately read and verified by `execute`.
No original helper, fixture, input, attempt or result is rewritten.

The projection admits exactly eight typed `InvocationEnd` fields. Only a
raw-enabled app-server may additionally carry the two source-owned supplements;
the original start hash and exact four raw/grace digest refs must agree before
typed comparison. Unknown fields, unrelated shared-field differences, absent or
wrong supplement hashes, extra digest entries and unmarked supplements fail.
Raw end/meta objects and all original downstream source/frame/delivery checks
remain full and unchanged. Nonraw ends preserve their complete typed identity.

## Test-first receipts

The reviewed draft was tests `05bf3a35…` / diagnosis `7bdace50…`; both were
unexecuted when reviewed. A complete synthetic CLI case preceded implementation.

| Stage | Tool receipt | Outcome |
| --- | --- | --- |
| Corrected module absent | `7ccf0d` | Missing-module RED, exit 1 |
| Separate original-behavior copy with original dependency base | `cd3e01` | Five tests, three pass and two fail; actual typed-end collection and complete CLI reproduce the original comparison failure |
| Exact projection implementation | `75d307` | Five tests pass, 0.079s |
| Final correction plus unchanged original/pure gates | `bc916e` | 79 tests pass, 0.415s; exit 0 |

The behavioral regression calls the original collector over independent
temporary records and explicitly verifies `capture-population-or-source-invalid`
with no appended invocation. Its corrected counterpart calls the actual pinned
full-frame function, not a supplied saved success. The complete CLI test uses
typed ledger ends, verifies new and original source pins plus canonical result
hash, retains a synthetic nonzero workflow outcome and original observer flag,
and proves a second invocation does not reach `execute`.

Final command, from this directory:

```sh
PYTHONPATH=/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c08-natural-delivery /usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest -q test_source_projection test_sources test_join_delivery test_delivery
```

## Limits and next authority boundary

The new capture index is built once; each end examines a fixed eight-key record
and at most four supplement entries. This introduces no quadratic work. Original
source/output limits, publication reservation and no-retry behavior remain
unchanged. No general peak-memory or crash-proof storage guarantee is claimed.

This is an offline metadata qualification, not an agent-facing runtime change;
a new real-agent workflow is neither affected nor appropriate. Original failed
source results and all natural-workflow/observer/accounting flags remain intact.
A future separately recorded source result can qualify only the new relation;
it cannot turn the original three failed calls into successes or authorize a
numerical or model replay.
