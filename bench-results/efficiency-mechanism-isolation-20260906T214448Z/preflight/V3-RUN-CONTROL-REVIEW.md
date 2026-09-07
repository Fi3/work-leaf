# V3 run-control adapter review

Verdict: no blocking finding in the reviewed allocation/control adapter behavior. This is a provider-free source and synthetic-test review, not a provider admission, allocation draw, or real-agent verification.

Reviewed identities:

- `runner_untracked_reads.py`: `f09cf8dc42172d4662ee73eefb8f7cb9888a5d9d039505765082be3e53e74004`.
- `allocate_untracked_reads.py`: `fca994af5ccd9459d08be2d621d0c350c4c1efd040ab8a907ccb4459f22db610`.
- `test_untracked_run_control.py`: `adff0ba7e1010d6a2d8bc00f331c0b901f3fcae99eedfbd84486fd0133b42965`.

`checked_instance` compiles captured source bytes, retaining the actual engine `__file__`. The runner's private module instance changes only admitted conditions/schema and exact-plan/identity validation hooks; other imports and the original v2 files remain unchanged. `runner_sha256` intentionally identifies the actual `runner_work_units.py` engine, not the v3 adapter. `dependency_pins`, `verify_current_dependencies`, and `verify_manifest` separately bind the executing adapter, supervisor engine, allocation adapter, and finite-population engine to their exact frozen paths and hashes.

`allocate_untracked_reads.allocate` validates identifiers, creates a new phase directory and writes its claim before calling `SystemRandom.choice`; an existing or failed-draw directory cannot be reused. The private finite-population instance permits only the new condition. `runner_untracked_reads.validate_plan` reconstructs the entire twelve-row plan from its two recorded canonical block allocations and requires structural equality, apart from separately retained allocation provenance. Each block has 18 allowed balanced mixed-wave allocations; the independently exercised joint population has 324. No outcome-dependent selection or sample extension is present in this code.

`prepare` preserves the evidence-root tree and automatically includes both adapters and engines. The unchanged supervisor constructs v3 experiment manifests through the private schema parameter and preserves its normal runtime, subscription-only wrapper, environments, three-workflow waves, timeout/usage-grace policy, source checks, trust replay and create-new run admission. The saved plan, schedule, within-wave launch permutations and source inventory are revalidated through the actual engine. This review covers the adapter's use of those existing checks, not a new audit of every unchanged supervisor branch.

Independent verification: all seven `test_untracked_run_control.py` tests passed. These exercise every one of the 324 plans, unsupported conditions, changed populations/order/counts, claim-before-draw and failed-draw retention, exact nonfactor environment identity, unchanged independent v2 instances, and provider-free prepare/verify with both engine and adapter identities frozen. No provider process was launched by this review.

Complexity: no new outcome/event pairwise joins. Finite plan reconstruction is bounded at 12 rows; the unchanged supervisor's per-wave full evidence hashing remains O(W×E), explicitly documented with four waves. That inherited bound is not an added unbounded pairwise algorithm.

The resulting-state treatment design and `PROTOCOL-UNTRACKED-READ-INLINE.md` match this separation between preparation, allocation and provider admission. The private offline adapter needs no public API or architecture change. A real subscription-backed diagnostic of the v3 runtime and completed-source accounting replay remain separate mandatory prelaunch gates; synthetic adapter checks do not mark those gates green.
