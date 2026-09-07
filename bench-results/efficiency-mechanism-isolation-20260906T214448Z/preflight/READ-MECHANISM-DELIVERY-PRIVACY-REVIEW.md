# Read-mechanism census: delivery and privacy review

Verdict: CLOSED, no remaining finding in the assigned scope. This is a provider-free
review of the new descriptive helper, not a benchmark analysis or causal result.

Reviewed identities:

- `audit_read_mechanism.py`: `2ad81ed5dc27ccba621f21f087312f48ac26e7799073baa5da6130eb2f7119e9`.
- `test_audit_read_mechanism.py`: `b9258351aa626425970b8d29c94cece0708d80fc560f86a70ea67276dde0f75f`.
- `DESIGN-READ-MECHANISM-CENSUS.md`: `c7c99b96afdd5fb3de613c62be742df5d9fe860c6bf7a3852b79fe668d70940e`.

## Scope and findings

The reviewed chain is `build_census` → `replay_launched_sources` → `derive_run`,
with `safe_read` for trace projection and `native_inventory` for public native items.
The independent source/accounting/resource-complexity review is separately owned by
the study-harness reviewer; this note does not replace that closeout.

The two reproduced P2 findings are resolved and covered by RED-to-GREEN regressions:
extra nested snapshot/bundle/component fields cannot export bodies, and a globally
invalid trace identity cannot make later physical read-response rows disappear.
The census enumerates physical trace records rather than relying on successful
inventory completion or overwritable sequence keys.

Native input joins use exact verified thread, explicit turn and byte-equal selected
text; direct and nested passthrough turn fields must agree. The documented unscoped
unique-text fallback does not infer user ownership from preceding turn context.
Ambiguous identities and one-item/multiple-input conflicts retain shared evidence
groups without granting charge attribution. Rejected or unverified delivery is not
an accepted exposure. Native-to-attribution charge references use exact item IDs.

Tool calls/results use exact thread/call identity. A bundle argument substring and
its source ordering are only a retrieval candidate: `actual_read_proven` remains
false. Longer filenames, indirect commands and unknown tool behavior still require
source inspection. Neither an unselected candidate path nor absence of a substring
proves that an agent received or did not retrieve a bundle. No content body, tool
arguments/results, or private reasoning body is exported by the safe projections.

For a valid terminal phase the wrapper retains all twelve declared workflows in
run-ID order, including unavailable/unlaunched outcomes; it does not zero-fill them.
Unexpected source-result IDs receive safe marker rows. Missing/duplicate source
analysis for a declared ID leaves that workflow unavailable; it is not treated as
another admitted workflow or silently accepted. Invalid phase population/closure
fails publication. Primary values are not consumed or copied; no condition contrast,
cost ranking, alternative endpoint or causal share is calculated.

## Independent verification

All 30 new tests pass independently at the hashes above. The final mechanical
`recorded_access`/`replay_launched_sources` extraction preserves the reviewed branch.
An independent replay of that actual source/native/capture/accounting branch on both
corrected, admitted and completed diagnostics reproduces the coder's entire derived
outputs exactly: four read records, six response records, all four previously
verified native input IDs, and zero attribution residuals. All 48 consumed source
identities match at closure. No synthetic twelve-workflow phase is used.

Retained replay: `READ-MECHANISM-DIAGNOSTIC-REPLAY.json`, SHA-256
`aab36258cf9f2cd340bde11d9cf80338feee9a1fca298ff2689175ccd9d4387d`.
The diagnostic-only reproduction script was read at SHA-256
`a94fa6f7ee274b57406285e97876cbb82c9c32464df2a74a5f2ab681b44ae88f`.
The source replay uses the actual diagnostic admission, successful smoke logs and
invocation start/end receipts. These smokes prohibit natural bundle retrieval;
their read/item links are plumbing evidence, not workload savings evidence.

No provider calls, whole-phase primary calls, frozen-source edits or commits occurred.
No study-phase outcomes were inspected. The helper affects only offline derivation, not
agent launch, prompts, tools or runtime behavior; no additional real-agent generation
is required for this private helper. The reviewed design documents its behavior;
no production architecture or operator-workflow documentation change is needed.
