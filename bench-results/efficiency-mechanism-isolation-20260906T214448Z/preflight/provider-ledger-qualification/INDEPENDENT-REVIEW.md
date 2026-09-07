# Independent provider-ledger qualifier review

Scope: the new private offline helper, tests, source-bound diagnosis and README.
No actual saved workflow is passed to this helper during implementation review.
No provider-facing runtime or public API is affected.

Reviewed source identities:

- `qualify.py`: `ef4c28ed51030b775c6d75e956c6e1019f4fef655b48f968c4073d9cc6812e2e`.
- `test_qualify.py`: `9752aab1762697133a91fc6913ad89825e924a55591e600f24199033970bed94`.
- `README.md`: `30fae0e3a37ed89c0be001b0d6b5da59cf13ef52b913a3bbdf7f239d50b1e423`.
- `DIAGNOSIS.json`: `1b4d55f44c04bab3ccff88b723e225a53dfca3307db525f37fbc1e2f367a5243`.

Root read the complete helper/tests and relevant pinned observer
`assistant_text_completes_work_leaf_directive` and `analyze_app_server` paths.
The observer adds each pre-directive last-usage event, stops controller replay at
the complete directive, and publishes only turns with actual pre-directive usage.
The qualifier preserves those semantics and independently joins launch/input,
native, raw response, final-state and controller identities.

Root independently reproduced an introduced coverage bug: an additional accepted,
native-input/turn-context/public-input/terminal-failed turn with no generation events
escaped the missing-controller witness loop. The earlier helper incorrectly
qualified two accepted turns with only one supporting witness. The new regression
failed first, then exact accepted-key/witness-key equality closed that gap.
Eventless failed turns cannot silently satisfy the exception.

Complexity finding: per-turn membership in a list of missing agents introduced
O(turns × agents) work. The reviewed implementation uses a set for both membership
sites and a separate sorted output list. Event/source joins are keyed and the
additional scans are linear, with O(N log N) output ordering rather than an
unbounded quadratic conversation join.

Independent command:

`python3 -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/provider-ledger-qualification -p test_qualify.py -v`

Result: all 16 tests pass. They cover exact wrapper delegation, unchanged originals,
typed and source identities, all missing-controller exception rules, present-row
replay, rejected unrelated errors, finite versus unsupported tails, dependency
admission before execution, closed JSON and source-drift failure.

The wrapper must be executed only under a new fixed-population scope. It verifies
all source pins, calls the unchanged original exactly once, and requires the full
fresh original result hash to equal its declared predecessor. Qualification is a
separate result: controller absence is unavailable, not zero; original errors,
completeness, report-source exceptions and phase outcomes remain untouched.
Unsupported tails keep null upper endpoints. It cannot clear configuration flags,
prove hidden-provider completeness or convert a marginal effect into a historical
causal share.

No remaining blocking finding in this bounded implementation. The README supplies
its private operating contract; production architecture/operator behavior does not
change. No real-agent workflow is affected and no real-agent verification is
claimed. Numerical invocation and its complete outcome retention remain separate.
