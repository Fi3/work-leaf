# Independent retained read-item recharge review

Verdict: **PASS, no blocking introduced finding in the reviewed offline scope.**
The qualified entry is the fixed-scope, single-operator CLI. This review does not
admit provider generation or run the extractor on retained workflows. Original
accounting, failures, phase flags and the earlier prepared scope remain unchanged.

## Exact reviewed identities

| File | SHA-256 |
| --- | --- |
| `recharge.py` | `4d6ac35d6bb857e6c34564020f035f6fdaf4a7f43fbe8d4e8d682fb78990ad0c` |
| `test_recharge.py` | `dd00cbf983c3f6d0f67b2b32bbcfa79cd6124d92283ea78a11535c291d6445d5` |
| `execute_recharge.py` | `3c2899d9bcda9e1a2e7c76bde4e45b2514bbb9247e7f36ddf4a9c378518eca42` |
| `test_execute_recharge.py` | `ad1442816c329681d7419011f5f9c39cd356a21f218c6c34dcea7b90e98465ee` |
| [README](README.md) | `72afcb9100eecb2d84875320cbc80192d5a1635dd765e7d2bda6ba735274c7fd` |
| [Final scope revision](../../RETAINED-READ-ITEM-RECHARGE-SCOPE.revision-01.json) | `b262d5bff186a80739dfcd90c73617d7d8ec2f7a002e92c2b3ab182773967224` |
| [Unchanged extractor](../../audit_input_attribution.py) | `ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740` |

Both helpers, both complete new test files, the README and revised scope were
read against this stable cut. The predecessor's `extract_records`, its exact
counter/identity rules and dependency-loading boundaries were inspected directly.
The relevant architecture/observer/benchmark evidence boundaries and the active
candidate/retained-read plans were also inspected. No runtime or public API is
modified; the private README and scope describe the offline workflow adequately.

## Source, identity and accounting boundary

`execute_recharge.execute` reads the exact scope SHA and verifies the executing
driver, adapter, frozen extractor and other common inputs. The adapter compiles
the captured pinned extractor bytes, without using cached bytecode or invoking
`dependencies`, `audit_run`, compaction audit or strict whole-workflow accounting.
The driver reconstructs the exact union of prior delivery/retrieval source maps,
checks its declared count/digest, and verifies those source hashes before and
after the row. Scope identity is checked again at the endpoint.

Native items use explicit direct or passthrough turn IDs, require agreement when
both exist, and require membership in all accepted turns. Failed and usage-less
turns remain owned; the latest `turn_context` cannot substitute for a missing ID.
Each target additionally matches its native source, physical line, payload hash
and turn. User-input and tool-output kinds stay distinct. Anonymous native items
receive no fabricated identity; conflicting identified items fail qualification.

The saved response ledger is checked by its canonical hash/count, exact accepted
thread/turn ownership, and each original native response's four integer counters.
The unchanged extractor separately joins raw response IDs and counters to this
ledger. Its unknown/conflicting attribution and signed/null residuals remain in
the projection. No response or tail is made exact by this adapter, and no missing
controller row or completed-response attribution becomes zero lifetime cost.

Deduplication is by `(thread_id, item_id)` and then completed response identity,
not bundle path. Multiple bundle references retain multiple associations on one
whole tool-output item; its input charge is not multiplied or divided among
archives. Direct read items include the complete delivered prompt, not a priced
snapshot substring. Actual eligibility and successful bundle-write metadata
distinguish bundle/inline classes from the selected tag alone. Failed retrievals,
unlinked read references and ineligible read deliveries remain present.

One raw capture per thread preserves physical within-thread response ordering.
First/subsequent positions mean the first/subsequent *observed charge* of that
item, not first execution, exhaustive lifetime use or cross-thread wall-clock
order. Native IDs lacking raw attribution and other coverage diagnostics remain
explicit. Full canonical extraction-result identity precedes bounded projection.

## Independent checks

From the repository root, with a fresh empty cache prefix:

```sh
recharge_review_cache=$(mktemp -d /tmp/work-leaf-recharge-final-review.XXXXXX)
python3 -B -X pycache_prefix="$recharge_review_cache" -m unittest discover \
  -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/read-item-recharge \
  -p 'test_*.py' -v
```

**26 PASS**, 0.007 seconds (`39a317`). The real unchanged extractor is called only
on synthetic records in this suite. Tests retain missing-writer-before-execution,
existing-attempt rejection, failed-result-publication/no-retry, duplicate keys,
closed JSONL tails, usage-less accepted turns, source drift, exact native counters,
explicit pre-context IDs, wrong-thread/prefix rejection, shared-output dedup,
partial attribution, body exclusion and row/byte ceilings. The author's recorded
RED cases and subsequent guards are documented in the README; this reviewer did
not modify tests or claim to have rerun their historical RED versions.

Two independent read-only metadata checks supplement the synthetic tests:

- `2941bc`: all **356** revised-scope source hashes pass before/after; six unique
  rows are exactly 001–006 in frozen order; all 11 common inputs and each row's
  report-union count/digest agree. Every saved response identity is globally
  distinct: **607** IDs across **42** owned threads. All 607 exact native physical
  witnesses match thread/turn/response and the four counters individually, with
  no sums or totals. All 12 declared result/attempt paths are distinct, absent,
  under the correct canonical per-run directory; `apply_patch` is available.
- `41fbdd`: exact-byte loading of only `native_inventory`, `targets` and
  `validate_targets` independently verifies **510** retained native target
  identities: 86 accepted read inputs and 424 retrieval outputs. Read classes
  are 34 eligible bundle manifests, 42 eligible inline candidates and ten other
  deliveries. All **54** delivery/retrieval/native source endpoints rehash.
  No retained `extract_records`, `project`, accounting function or provider call
  occurs in this metadata check; native bodies are hashed, not interpreted or
  exported.

These are source/counter equality and item-population checks, not new accounting,
comparison, token totals, per-item charge extraction or a causal result. All
declared outputs and attempts remained absent at this review's readiness check.

## Publication, resources and limits

`main` checks the writer and canonical existing destination parents before
execution, refuses existing results/attempts, and publishes an attempt marker
before the row's source/charge replay. Result publication is create-new and its
actual bytes must equal the serialization. The retained marker blocks a second
CLI execution after result-publication failure. Internal `execute` is not a
separately authorized retry entry. The root operator owns the exact six calls;
there is no simultaneous sibling extraction.

The documented assumption is one trusted operator, not atomic multiwriter or
crash-proof storage. Pre-result process failures require a separate operator
failure receipt; they do not authorize removing a marker or rerunning extraction.
Path/endpoint checks do not claim hostile-concurrent-writer containment.

Result metadata is allowlisted; message/argument/file/private-reasoning bodies
are absent. The 100,000-row and 32-MiB gates are postconstruction admission
checks; each source read/hash is bounded at 512 MiB. They are not a general peak
memory or in-flight workspace guarantee. Exceeding a bound fails instead of
clipping, selecting favorable rows or changing the predicate.

No introduced quadratic helper path was found. Source parsing, body hashing and
identity-map joins are linear in consumed bytes/metadata, with set/key sorting
and explicit associations counted as input/output work. There is no per-response
full native-file scan or per-bundle duplicate charge. These bounds do not assume
arbitrary metadata strings have constant size.

No Cargo build or real-agent verification is applicable: this is offline evidence
processing with no changed agent-facing behavior. No actual retained-run charge
extraction, whole-workflow audit, provider call, historical-report edit or runtime
mutation was performed by this review. Actual execution and six-row closure
remain the root operator's next separately controlled action.
