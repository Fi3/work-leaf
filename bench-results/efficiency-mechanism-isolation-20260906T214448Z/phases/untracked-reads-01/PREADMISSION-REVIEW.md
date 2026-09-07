# Independent preadmission review

Prepared input/configuration/schedule verification passes. The complete supplemental Python
verification passes 265 tests on exact frozen source bytes. Direct discovery inside the prepared
read-only evidence tree remains a retained failed attempt; it is not relabeled as passing.

`PREADMISSION-TEST-RESULTS.json` contains the commands, source/copy hashes, exact test qualification
and final checks. Its SHA-256 is
`6da310209d3d348296bfe7589b232275608f10359c2ad267ab20871666cf1abf`.

## Prepared phase

- Manifest: `706c0b13d62376b3ea10c4604bb63957c271a1a7224d39b27db20fa24e2da263`.
- Schedule: `05fa9c780d2e2203af0454990597fb6369082ddefb49e46887b663341ed41687`.
- Original one-shot allocation: `01fea6d4f3ecc67f82bcbac7030103aaecc03817520b47923cb5242d2c85427d`.
- The 114 manifest identities comprise 84 frozen-evidence files (83 inputs plus their list),
  12 private experiment manifests, five source/five copied drivers, two release runtimes, observer,
  subscription wrapper, two external CLI identities, protocol and scorer config.
- Exactly twelve workflows remain: six control/six inline; two blocks of three per condition;
  four mixed waves of three. All private manifests, owned runtime paths and effective environment
  overrides match the frozen supervisor's construction.
- Frozen launch order is `003,001,002`; `006,004,005`; `007,008,009`; `012,010,011`, with prefix
  `untracked-reads-01-workflow-`. No allocation or launch-order redraw was performed.

The actual frozen `runner_untracked_reads.py::verify_manifest` passes before and after testing,
including every manifest file hash and the clean driver's HEAD/status. Driver commit is
`3c907f7266b63efa1558213f3e49587e999b899d`; the historical task base remains available. The release
runtime/observer hashes match the build and final qualification receipts. Generation settings remain
GPT-5.5/xhigh, existing subscription routing, maximum 1,000 ms usage grace with forward on resumed
output, raw metadata and the unchanged project-layer inventory/trust policy.

All primary, runtime and census execution dependencies are present in the actual prepared tree,
including the pinned accounting/legacy helpers, canonical scorer and fixtures. The separate
regression-only dependency below is not imported by those execution paths.

## Retained direct-discovery failure

Direct frozen study discovery reports 216 tests and 18 errors. One module cannot import
`allocate_work_units.py`; the other 17 errors are prepare fixtures requiring the canonical
subscription wrapper to be executable. The evidence copy is intentionally mode `0400`; the actual
phase runtime provider wrapper is correctly `0500`. Direct frozen predecessor-gate discovery passes
22 tests. Fresh bytecode-cache prefixes were used for each invocation.

The sole executable-source references to `allocate_work_units` in the frozen tree occur at
`test_analyze_work_units.py:13` and `:187`. They import the retained v2 allocation helper and call
`make_plan` with `lambda choices: choices[0]` for a deterministic unit fixture. The v3 allocator,
supervisor, primary analyzer and census do not import it. This is not a missing generation dependency.

## Supplemental exact-byte verification

The separate tree is `/tmp/work-leaf-v3-frozen-test.pbfpn1/evidence`. All 84 frozen evidence files
were materialized with path-preserving `apply_patch` additions and matched byte-for-byte. The only
extra source is the committed test-only `allocate_work_units.py`, SHA-256
`dcedab762a01b3983012adccac81caedc78d94f090796827be3fabe7ad1c0b71`. Its own predecessor
`allocate_confirmation.py` is already frozen. No provider or real-study allocation is invoked;
temporary allocation fixtures remain ordinary unit-test data.

Only the temporary canonical wrapper copy has executable permission `0500`; other copied inputs are
`0400`. No prepared source, test, permission, allocation or manifest was edited. The source hashes of
all 85 temporary inputs and their originals still match after testing.

The unchanged tests run with `python3 -B`, separate fresh empty `-X pycache_prefix` paths, and
discovery rooted in that temporary evidence tree. Result: 243 study tests plus 22 predecessor-gate
tests, all passing. This qualifies the frozen source bytes with an explicitly recorded test-only
materialization, not a claim that direct frozen discovery passes or that a provider was exercised.

## Final boundary

At `2026-09-07T07:12:55.936950+00:00`, the manifest remains unchanged; `RUN-ONCE` does not exist and
`/tmp/work-leaf-untracked-reads.jUcFKF` has no children. Global configuration still matches preparation:

- Raw: `e8aae9df8fdb7c91c473410aca9ed290d802f92f7e85d9cd030a91ffae09c3d8`.
- Parsed: `00be322d8c9b90bad8588e5f36438ee03b210c314bfc1f2e20ed0193b06589a1`.
- Behavioral: `215cca363181bec4286770a8811945c9fff35475782fc97f20f1cfefe19b202a`.

No global configuration contents, credentials, model messages or phase outcomes were displayed or
archived. No provider was launched and no real-phase run, primary, scoring or census mode was executed. Root
retains the subscription quota recheck and actual admission decision.
