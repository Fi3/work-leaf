# Fixed-three offline accounting publication qualification

The separately reviewed future execution scope, not this receipt, authorizes actual accounting. No original `audit_run`, baseline replay, observer/extractor, native payload analysis, provider or private command executor was called during implementation. This helper has no agent-facing behavior; real-agent verification and Rust gates are not applicable to the Python-only offline orchestration layer.

## Exact source cut

- `execute_accounting_once.py`: `0740c2fd41cecd4d1adc35ec49cc4e6317c14deebe7998566e87ec89a177ce06`.
- `test_accounting_once.py`: `73a2725f95e79552d2ce4434d59ef9cb501c929f1c3b466661c14a03423067ab`.
- `ACCOUNTING-ONCE-DESIGN.md`: `75adb92098f66851a59759215c75a349f3ed8e78498cea1e330485c7cc5988e0`.

## Observed test-first gates

All test commands used `/usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest -v ...` in this directory.

| Gate | Actual result | Tool receipt |
| --- | --- | --- |
| Initial 24-test draft, implementation absent | Exit 1, missing `execute_accounting_once` module, one loader error | `d2f274` |
| Initial implementation, synthetic fixtures and two actual non-accounting children | Exit 0, 24 PASS, 1.355s | `6145b2` |
| Four publication/schedule boundary checks | Exit 1: two assertions and two errors reproduce unchecked schedule/output declarations, rejected sibling witness and missing blocked-row receipt | `de3cb0` |
| Publication-boundary behavioral suite | Exit 0, 27 PASS, 1.400s | `ecbcce` |
| Root independent prior-cut suite | Exit 0, 27 PASS, 1.395s (attributed root result) | `2f3d12` |
| Actual saved-supplement-shaped synthetic fixture | Exit 1, `KeyError: original`, followed by three exact-envelope regression errors | `e330ee`, `aaa2fe` |
| Complete final suite with exact saved schema | Exit 0, 30 PASS, 1.412s | `0b375b` |
| Root independent envelope-cut suite | Exit 0, 30 PASS, 1.414s (attributed root result) | `dd7f6d` |
| Explicit disarmed-draft guard | Exit 1: `ValueError` not raised before synthetic calls | `666d68` |
| Final suite with typed execution-authorization gate | Exit 0, 31 PASS, 1.424s | `da28d7` |

The two actual children execute only synthetic Python statements: one emits fixed stdout/stderr and exits 7; the other ignores SIGTERM, emits a readiness string and sleeps until its 1s test deadline and 0.2s escalation. No test invokes the production accounting bootstrap. The remaining tests replace its private invocation callback with synthetic results or typed execution failures.

The root-authorized source-only assembly detected the supplement-envelope discrepancy before any original call (`44525e`). Metadata inspection `f4f62e` establishes the actual two schema tags, the top-level full `response_evidence` map and `result.original` count/hash projection. The exact wrapper-shaped synthetic tests bind both original projections and ownership, rather than importing the later qualification measurement. Original receipts, their canonical digests and all frozen helpers remain unchanged. That preparation inspects identity metadata only; it does not compute usage totals.

## Preserved boundaries

The fixed CLI has only scope/hash arguments. Its production bootstrap uses the original `c365aa86…` helper at its actual frozen path and preserves complete entry/manifest values and dependency-relative `__file__`; no alternative helper CLI exists. Each of three new IDs has exactly one possible call, and six pinned W receipts/maps are imported without baseline calls. Exact source/schedule/terminal joins, per-worker output declarations, pre-call reservations and a durable attempt marker precede execution. Closed errors/timeouts are retained; uncertain closure or source drift prevents subsequent calls and publishes their dispositions. Publications are create-new but not a multi-file transaction; partial outputs and the attempt remain, with no retry.

Canonical complete results precede projections. Original failures, diagnostics, partial response maps and null bounds remain separate from caller errors. A global indexed response-owner map rejects duplicates without changing accounting. Source union completeness remains a pre-execution operator obligation; returned extra sources cannot be admitted retrospectively. Fixed K=3 repeated source hashing is O(K×E); indexed metadata passes are linear under ordinary hash assumptions. Full result/projection memory is not generally bounded. Independent source review and actual frozen-scope execution remain pending.
