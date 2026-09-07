# Independent prospective accounting review

The root reviewer read all 528 helper lines and all 255 test lines of the future-only v3
accounting adapter. Review covers only this adapter, not the immutable older reports.

The helper SHA-256 is `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`;
the test SHA-256 is `93f958ae241b7c59d2dd531aa2fc87f70bee0d268f3c380758ab965177a9bdf7`.
The independent command `python3 -B -m unittest discover -s
bench-results/efficiency-mechanism-isolation-20260906T214448Z -p
test_accounting_untracked_reads.py` passes all 32 tests.

No blocking finding remains. Raw/native identities, accepted local turn scope, component arithmetic,
cumulative prefixes, exact compaction lifecycle/identity and duplicate suppression are checked.
Unusable notification metadata is distinct from completed response usage. Late completion recovery
requires exact same-turn raw/native usage and an additive fresh update before terminal, without a
later generation/item/unknown boundary. Original grace decisions remain intact. The correction and
physical evidence locators are retained separately from the original observer ledger. Source scans
and joins are indexed; no newly introduced whole-response-by-whole-stream quadratic scan remains.

The reviewer independently replayed the completed ordinary 001 and compaction 003 source records
using an in-memory helper identity entry, without writing any old manifest or replacement total.
001 validates 186 exact responses across 23 sources and retains finite conditional bounds.
003 validates 234 exact responses across 23 sources and the separately proved compaction exception,
but retains an unrelated unbounded accounting gap. Both have no integrity errors. This distinction
is required behavior, not permission to omit 003 or assert complete usage.

`READ-ACCOUNTING-VALIDATION.md` retains the exact raw/native physical locators and source digests
for these real subscription-source replays. Offline accounting changes no provider calls, input,
runtime timing, interruption behavior or credentials. Its actual-source verification is therefore
the real-data replay; runtime read delivery has a separate real-agent gate.
