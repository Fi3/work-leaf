# Read-mechanism census: source and accounting review

The census helper has no remaining source/provenance, accounting, interface or redundant-complexity
finding at the identities below. This is the `study_harness` review scope; independent native-item,
delivery, bundle-reference semantics and privacy review belongs to `candidate_controls`.

## Reviewed identities

Paths in this table are relative to the study directory.

| Artifact | SHA-256 |
| --- | --- |
| `audit_read_mechanism.py` | `2ad81ed5dc27ccba621f21f087312f48ac26e7799073baa5da6130eb2f7119e9` |
| `test_audit_read_mechanism.py` | `61bd3c59b7ea8d09de5d7e78a92ba557984e4e0ac815c2b6ff341a1725c42127` |
| `DESIGN-READ-MECHANISM-CENSUS.md` | `c7c99b96afdd5fb3de613c62be742df5d9fe860c6bf7a3852b79fe668d70940e` |
| `preflight/READ-MECHANISM-DIAGNOSTIC-REPLAY.json` | `aab36258cf9f2cd340bde11d9cf80338feee9a1fca298ff2689175ccd9d4387d` |
| `preflight/READ-MECHANISM-DIAGNOSTIC-REPLAY-v2.json` | `faa20da012c3f63c6587d0a621d3db4ccd472f131ad86a4a8819fe9ad849ec65` |
| `preflight/replay_read_mechanism_diagnostics.py` | `2b74fde7960562591750fabca36ce930797f6638055f270632f2bd7e50f40969` |

The final helper delta extracts `recorded_access` and `replay_launched_sources` from the launched
workflow branch. `build_census` retains terminal phase/admission checks, frozen-file checks, complete
schedule coverage, exact per-run terminal receipt validation, prior-result equality and endpoint
rehashing before delegating. `replay_launched_sources` explicitly requires its caller to establish
terminal admission; it is not a replacement phase-admission gate.

## Evidence and accounting gates

`SourceIndex.read` requires canonical regular files, rejects symlink traversal, bounds source sizes,
checks opened-file identity through the read, validates every supplied lowercase SHA-256 and rejects
an indexed source that changes. A private sentinel distinguishes an independently inventoried source
from a malformed null expected digest. `verify` repeats endpoint checks; `publish` uses create-new
output. These are boundary checks, not a claim of continuous filesystem immutability.

`build_census` independently hashes the exact phase-local `logs/<run_id>.exit.json`, because the
primary analyzer does not consume those receipts. It checks run identity, integer exit code, start
and finish times, and terminal launch status against the retained phase row. A genuine nonzero exit
does not cause exclusion or zero-fill. Missing evidence produces an unavailable or partial row.

`dependencies` compiles exact pinned source bytes. Its attribution definitions come unchanged from
`audit_input_attribution.py` SHA
`ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740`.
`extract_records` counts top-level items plus separate request fields, not nested item content;
deduplicates exact response IDs; rejects conflicting identities; and retains signed residuals and
unknown output/reasoning attribution. The census builds references into this one response table.
It does not call the predecessor's cost-ranked CLI or recalculate the primary endpoint.

`replay_launched_sources` independently invokes the pinned v3 `measurement_for` accounting adapter,
checks its source identities and joins exact thread/turn/response usage. `build_census` requires that
fresh accounting record to equal the saved source-analysis record. Bounded and unbounded accounting
states and gaps remain explicit; no unsupported tail is completed, priced, bounded or replaced here.

## Regressions and complexity

The review independently observed failing tests for null expected hashes, the primary-report receipt
inventory mismatch, and repeated candidate/owner/path arrays before their implementations passed.
Shared native candidate, ownership-conflict and issued-path records replace repeated full arrays;
multiple issuers are ambiguous without rescanning them for each match. A single issuer uses a constant
time ordering check. UTF-8 pattern lengths are computed before matching.

Matching is output-sensitive. The scanner's bound is argument bytes plus emitted matches; pattern
preprocessing and serialization have their own costs. Many overlapping patterns can inherently emit
a superlinear number of matches, and serialized output costs its actual byte size. No claim of linear
total output under arbitrary overlap is made. The redundant quadratic scans and duplicated issuer
lists identified during review are absent at the reviewed helper hash.

Independent command, from the study directory:

```sh
python3 -B -m unittest test_audit_read_mechanism test_audit_input_attribution test_accounting_untracked_reads -q
```

Result: 83 tests passed, including 31 census tests. The launched-wrapper delegation regression covers
the extracted source branch. The earlier attribution/accounting suites are unchanged.

## Closed-diagnostic scope and exact-byte reproducer

All 47 source identities recorded by `READ-MECHANISM-DIAGNOSTIC-REPLAY-v2.json` were independently
rehashed and matched. Its two diagnostic rows have empty derivation and accounting error lists.
The report declares zero study observations, no provider calls, no primary invocation, and use of
the shared launched-source branch. It explicitly does not claim execution of a twelve-workflow phase
wrapper. This review did not rerun that replay or generate an agent response.

`replay_read_mechanism_diagnostics.py::compile_source` compiles and executes the captured census source
bytes and returns their SHA-256. Before replay, `SourceIndex.read` requires the source to match that
executed-byte hash. The stale-cache regression creates same-size source with its earlier modification
time and a conflicting valid cache; the exact-byte loader executes the source value and returns its
correct hash. This closes the diagnostic reproducer's source/import identity finding. The main census
helper is unchanged at its reviewed hash.

The original replay remains retained at its original identity. Its earlier reproducer hash was
`a94fa6f7ee274b57406285e97876cbb82c9c32464df2a74a5f2ab681b44ae88f`; it used a normal import before
source hashing. No actual stale code was found in that recording: the independently inspected cache
code object equaled compilation of the source, with cache SHA-256
`af3902cb4ea075ab95ffd81f4f0539d4f91fd1ff7f61dd59d0f72c7bfdeca701`.
The separate v2 replay carries the explicit executed-byte binding and leaves the earlier record intact.

No agent-facing workflow is affected by these offline reporting helpers: they read closed evidence
and write a separate create-new report. No new real-agent generation is necessary to exercise their
behavior. The architecture and benchmark operator policy require no ownership or provider changes;
the census design document describes the new offline workflow. No frozen helper/report, global
configuration, provider option, prompt, wait, allocation or study outcome was modified by this review.
