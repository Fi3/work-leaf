# C08 observed-charge lower qualification: synthetic validation

The separate offline helper passes 39 synthetic tests. Actual study execution and numerical qualification are not performed for this validation. All original accounting/source/eligibility artifacts remain unchanged. A root-frozen source declaration and independent final review are required before the new source-bound call.

## Exact source cut

| File | SHA-256 |
| --- | --- |
| `qualify_lower.py` | `4793a42f845191f01dc9f0cef78b9dedb3823ce6bc2b5adcad36652ddc62e4c5` |
| `test_qualify_lower.py` | `af26be5c4a42148354baddfd2669a1cbf092cecfd9d42fdfefb47569af1de766` |
| `test_sources_lower.py` | `40eb731f7f49c65ebda76eee1d14139cac98da21cfaca9f33cc667e5d82020f4` |

The pure arithmetic definitions are compiled without modification from whole SHA-pinned `accounting_untracked_reads.py` (`c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`) and `batch_analysis.py` (`dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3`). Only `checked_usage`, `add`, `integer`, `usage` and their field constants execute. No original accounting/observer/source/frame/context auditor is imported and called, and no baseline usage is summed.

## Recorded RED and GREEN evidence

- Initial `470835`: 22 tests fail in 0.002 s because the new module is absent, before implementation.
- First implementation `807ed4`: 21 pass / 1 new-fixture failure. The duplicate-ID fixture changed its original map without the corresponding raw map, so an earlier valid rejection prevented reaching its intended global-ID assertion. Correcting that uncommitted fixture gives `1b2744`: 22 pass, 0.038 s.
- Independent review identified sums before complete population proof and a quadratic native-path membership scan. `0753bd` reproduces two premature sums and loss of baseline receipts on failure. Indexed source-path membership and staged all-nine map validation give `760265`: 24 pass. Independent reproduction `d6f0d3` is attributed to the reviewer.
- Root's blank-versus-literal-empty-object case fails at `33aec0`, then passes with distinct server blank sentinels (`3f5e7c`: 25 pass).
- Wrapper initial `b8263c`: five errors for its absent materialization API, one existing source-union test passes. `cc8cb8`: 31 pass after source/reference and exclusive-publication implementation.
- Root's raw physical metadata hash versus canonical object hash case fails at `2b257e`; the independent identities pass at `267b82` (34 tests).
- The Markdown preparation contract fails at `8fe6bb`; raw pinned reading passes at `6fd1ff` (35 tests).
- Duplicate original disposition rows fail to be rejected at `cf22aa`; exact list/map cardinality closes the case at `d8b3fc` (36 tests). Reviewer reproduction `deb14e` is attributed separately.
- The complete actual temporary-files pipeline, with no materialization mock, fails at `905c62`; direct arithmetic-reference admission was incomplete. The explicit fixed arithmetic references and full nested-source pipeline pass at `da4199` (37 tests).
- Final `ea9597`: **39 pass, zero failures, 0.117 s**, including failed result publication retaining its no-retry attempt and source drift after arithmetic invalidating every bound. No new behavior was required by those last two passing regression tests.

The owner command from this directory is:

```sh
/usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest -q test_qualify_lower test_sources_lower
```

The tests read only named immutable code inputs and their own synthetic/temporary files. The complete pipeline fixture constructs three candidate and six baseline records, original/full/projection relations, closed-source subproof references, spaced/reordered raw metadata and exact helper pins. It uses real file reads, endpoint hashes, the unmocked materializer, unchanged arithmetic definitions and create-new publication. Its source/membership/eligibility documents are explicitly synthetic prior proofs, not a claim that old auditors or real providers ran. Separate in-memory tests target malformed and missing records; separate publication tests inject only local write/drift failures.

Independent review has already reported 25 pure tests passing (`c090c7`) and 37 combined tests passing (`8e180e`, 0.097 s). Those are attributed checkpoints, not a claim that final independent review is complete.

## Result and interface qualifications

`execute(scope_path, scope_sha256, output_path)` requires the declared `work-leaf-c08-observed-lower-scope-v1` schema and literal `execution_authorized:true`. The preparation reference is Markdown. Arithmetic references are `arithmetic.accounting` and `arithmetic.strict`. Three candidate references bind their original FULL/projection and corrected SOURCE INPUT/RESULT; only 003 binds the existing eligibility input/result and sidecar. Six baseline refs come from the original pinned accounting scope. The CLI is:

```sh
python3 -B qualify_lower.py --input /absolute/new-scope.json \
  --input-sha256 <exact-sha256> --output /absolute/new-result.json
```

Output and `<output>.ATTEMPT.json` must not exist; a retained attempt cannot be retried. Stdout contains only status, row counts and zero original-auditor/provider call counts. Source/hash failures and unknowns are retained, not converted to zero. Full original statuses, errors, observer ledgers, phase flags and measurements accompany separate observed lower endpoints. All candidate uppers remain null. The baseline projection is copied unchanged from its pinned consolidation/receipt, with both exact response-map projections checked; its own ledger or mean is never recomputed here.

The native source-path membership check uses an explicit set. All nine map proofs precede aggregation. Other passes are indexed and linear in bytes/records except canonical key sorting; no new O(N²) source or response join remains. Test fixtures are finite and make no unbounded throughput or hostile-host guarantee. Source/output byte limits apply after the explicitly described reads/construction, not to all peak memory.

No Work Leaf runtime, observer behavior, provider/backend integration, driver or public API is modified. This private saved-data helper affects no agent-facing workflow, so no real-agent verification or Rust/Cargo gate is asserted. The admitted actual source-bound lower qualification remains root-owned; this receipt neither computes its totals nor authorizes additional generation.
