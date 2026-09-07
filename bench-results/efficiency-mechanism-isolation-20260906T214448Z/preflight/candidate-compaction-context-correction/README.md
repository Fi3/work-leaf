# Separate pre-turn compaction context accounting derivative

`accounting_compaction_context.py` supplies a narrowly scoped, provider-free
context attestation for an explicitly identified pre-turn compaction response.
Original native records, captures, observer ledgers, accounting failures and frozen
helpers remain immutable. Eligibility is not a corrected workflow total or a
mechanism conclusion. The corrected nine-row accounting scope requires its own
declaration and independent review before `audit_run` is invoked.

## Source and predicate boundary

The derivative compiles the exact original `accounting_untracked_reads.py`
(`c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`) and its
pinned dependencies from the candidate phase's frozen evidence tree. The native
checker is `audit_compaction.py`
(`dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153`).

One exact source substitution extends only the native response model/effort
predicate. A response with an already valid preceding context takes precisely
the original path. The fallback is specific to a proved response ID, physical
line, complete response payload hash, whole native-row hash and complete native
metadata hash. The derived native source digest is
`24cfcd6d5de51a5ceaab6ef7299a24867223a4116a33c47cbc0e0ceae8504bce`.

The original `native_ledger` function's code executes with a private derived
native module binding. No source row is inserted, reordered, normalized on disk,
or stripped. Physical locators, unique-response inventory, cumulative arithmetic,
exact native compaction/latest-record rules, original observer reconciliation,
raw usage rules, interrupted-tail bounds and outcome retention are unchanged.
Conflicting later context records remain failures. There is no generic
permission to infer a response model from a later record.

## Required proof

The fallback requires all of the following together:

- One native response with the exact session/thread/turn identity, immediately
  followed by an explicit `compacted` marker naming that response and repeating
  the complete native latest-usage record exactly.
- One unique later context for that turn, with the required model, effort and
  owned cwd. No generation or ambiguous intervening boundary may occur before
  that context. The exact native `user.text` item follows it.
- Typed, accepted public thread and turn identities, exact complete original and
  forwarded input, and exact public/native user text for that same turn. Explicit
  public model/effort/cwd overrides must agree; item identifiers retain their
  separate public and native namespaces.
- A unique public context-compaction start, matching raw response, matching
  compaction completion and public user, in that order after turn acceptance.
  Raw/native response usage must match exactly under the original additive rules.
  Another response inside the compaction window makes the proof ambiguous.
- Hash-bound closed capture/native sources and endpoint equality. The unchanged
  pinned `capture_provenance` verifies the complete observer rewrite journal and
  raw-metadata settings before the internal capture representation permits the
  existing `thread/start.experimentalRawEvents=true` rewrite. `turn/start` remains
  unchanged. This internal result is derived from source, not a saved boolean.

Ordinary metadata, duplicate or conflicting identities, unsupported custom
activity, different source bytes, missing or malformed hashes and absent evidence
cannot produce the fallback. Explicit expected SHA-256 values must be lowercase
64-hex strings; `null` cannot mean an unpinned source.

## Eligibility-only invocation

`ELIGIBILITY-INPUT.json` pins this helper, the original phase manifest and the final
score manifest, and names exactly candidate workflow 001. The CLI has no accounting
mode and does not invoke the original whole-workflow `audit_run`:

```sh
python3 -B accounting_compaction_context.py \
  --input /absolute/ELIGIBILITY-INPUT.json \
  --input-sha256 87afef6df5a35cc0aa28cbd5e620c4f4f8aaea583780a65f5c4b6dc2a6191dd6 \
  --output /absolute/new-eligibility-result.json
```

The result exports source identities, exact proof locators and both original and
corrected native-prefix diagnostics, never token values or message bodies. Native
prefix arithmetic is checked internally; no whole-workflow total is calculated.
Output creation is exclusive and cannot overwrite the original or another result.

The actual closed-source `ELIGIBILITY-001.json` is eligible with one response proof,
eight retained native sources and 25 rechecked source files. The native response at
line 276 and marker 277 precede their matching context 284 and user 285. Public
RPC string `64` is accepted at server line 43734; compaction starts at 43737,
the exact raw response occurs at 44094, completion at 44097 and user at 44099.
The original nine native-prefix diagnostics remain present in the eligibility
receipt. The original common-accounting UNKNOWN remains unchanged.

## Validation and limits

The new tests initially fail because the derivative module is absent. Additional
observed RED gates cover intervening generated events, explicit public scope
contradictions, metadata identity binding, malformed empty text metadata, extra
responses inside a compaction window, null source digests and repeated public
index construction. An initial actual eligibility preflight rejects the ordinary
observer metadata rewrite; its exact permitted rewrite has a separate RED-to-GREEN
fixture and the source-bound closed replay passes.

`python3 -B -m unittest test_compaction_context -v` passes 22 tests. Coverage includes
already-valid report identity, unchanged original failure, physical native ledger
locators, no cumulative-rule waiver, exact-byte loading, retained failed outcomes,
eligibility-only CLI output and refused overwrite. No committed test or runtime
source is modified. This offline helper does not affect an agent-facing workflow;
no new real-agent verification or provider call is applicable to its execution.

Public indexes are shared across all qualifying native threads, avoiding repeated
capture scans. Successful response-to-context spans are disjoint; source and
payload work is linear in bytes plus records. Fixed dependency compilation and
hashing is O(T × D), for T native threads and D pinned dependency bytes. The module
does not claim hostile-writer containment or continuous source immutability.
Independent review is a separate qualification gate; corrected all-nine totals
remain prohibited until the separately declared corrected scope.
