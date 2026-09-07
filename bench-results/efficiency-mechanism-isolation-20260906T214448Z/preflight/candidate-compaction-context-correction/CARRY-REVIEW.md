# Independent repeated-compaction metadata review

Disposition: no actionable introduced correctness or scope finding. This review
covers the new sibling derivative, its tests and `CARRY-README.md`, not a new
measurement admission or a saved-workflow accounting result.

Reviewed exact source identities:

- `accounting_compaction_carry.py`: `47d28ef42c3243344cb28d6db680a531b04255cdc310653e45fc3ee81b8418a1`.
- `test_compaction_carry.py`: `efffcc4c03bb1b3d738bc83500dc4ca1c4ffdf9bf3b6e180ed6900286ca144b5`.
- Immutable context dependency: `513c8632803656d4f582a4ab714e35d9e0f95b56a4355b3b26c2927b07d2bfa3`.
- Original accounting source: `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`.

Independent command:

```sh
python -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/candidate-compaction-context-correction -p test_compaction_carry.py -v
```

Result: 26 tests PASS, exit 0. Only synthetic, pure predecessor, mocked wrapper
and unlaunched-scope cases ran. No actual saved-workflow `audit_run`, numerical
replay, provider call or current outcome inspection was performed by this review.

The derivation verifies exact original bytes and adds only capture-local carry
state plus a wrapper around the existing nonadditive metadata branch. An origin
can be established only after that unchanged branch proves one named native/raw
compaction omission with its complete public lifecycle. Repetition requires the
same capture/thread, a physical position after the originating lifecycle, exact
typed full metadata objects, and equality with both previous cumulative usage and
the independently maintained raw-prefix expectation.

Distinct raw responses, including zero-usage ones, new compactions, changed values
and unsupported thread/turn ownership events invalidate the origin. Exact duplicate
raw IDs do not create new charges. No unchanged-metadata event proves the absence
of ordinary item/tool generation. All five generated/interrupted synthetic tails
remain unbounded through the unchanged strict accountant; carried warnings remain
in `affected_turns` and cannot supply fresh usage or late-tail recovery.

The wrapper keeps original pure diagnostics separately. Exact physical row hashes
are checked against the retained client/server/grace files before capture-path
assignment; source hashes are rechecked at closure. A partial capture census cannot
claim an original whole-scope consequence. The original pre-cumulative outer guards
are explicitly distinct from a second original whole-run audit or an invented total.
Existing immutable helpers and original reports are not rewritten.

Added state work is one indexed forward pass and one origin per thread. Hashing
and typed JSON serialization are linear in the actual metadata/source bytes; large
additional JSON fields must not be treated as constant-size by an asymptotic claim.
Original and derived reconciliation each execute once per capture for the separate
diagnostics. There is no introduced event-history pair scan or O(E²) loop. The
documented predecessor compilation term and identity sorting remain qualified.

The new README correctly limits scope, preserves failures/unknown tails and requires
separate prospective authorization for any saved-workflow numerical invocation.
No runtime, public API, architecture or real-agent workflow is affected by this
offline sibling; no additional product documentation or provider verification is
required for its implementation qualification.
