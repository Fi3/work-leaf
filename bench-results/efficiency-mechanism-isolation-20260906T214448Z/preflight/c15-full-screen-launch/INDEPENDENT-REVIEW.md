# Independent C15 full-screen launch review

Verdict: **no unresolved introduced source/test blocker** in the exact cut below.
The new host binding's real-agent verification remains pending at the first separately
admitted full workflow; neither these tests nor diagnostic003 proves that new seam.
No benchmark/provider, private executor, accounting operation or Cargo gate was run
by this review. Only the new provider-free Python tests were executed.

## Exact reviewed cut

| Source | SHA-256 |
| --- | --- |
| `../../runner_test_first.py` | `624fed61e6d8d311788827c0f6186348e56a0e5e8d7be64fa551d3fe5d3e124a` |
| `../../test_runner_test_first.py` | `42ee3720b70533f3730c903d004abdbf3069c4dda585ada3a4e235ea37264387` |
| `bind_daemon.py` | `aa1800d574df9c7dee1ad899619594060a68cd28fe61d5a2f0ed9d63e16e3155` |
| `test_binding.py` | `85cb48dae719196d1307834bd30a04edf818744ee52e9cf3c06df6cb2ae696cf` |
| `CONTRACT.md` | `99811f0b4481d9fd02316c76bd3a60846a2fe0985f0e907c5634d8a34585eac2` |
| `VALIDATION.md` | `08a486e41ee202277d5fb2c2db1a3a6b383420a39a485635b35508cf24c4b82f` |
| `../../PROTOCOL-TEST-FIRST-SCREEN.md` | `4dcc17c5598666461f8e9a5fa7c9e17e56e2e439d3cefbf93830e6e16aa7057a` |

Read complete new implementations/tests, protocol, contract/validation and current
architecture/operator policy; inspected the other documentation/catalog for affected
workflows. The study protocol/private contract describe the added host operating path.
Ordinary public API, user workflow and runtime ownership are unchanged; no additional
general-product documentation edit is required by this private launcher.

## Source and isolation conclusions

- `runner_test_first.load_source` exact-compiles the unchanged engine
  `2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e`.
  Overrides are confined to v6/single-condition plan validation, preparation and
  supplemental verification. `run_environment` and `run_phase` retain the original
  engine functions, including trust/config gates, three-workflow schedule, one-attempt
  claim, supervision and outcome retention. The 67-character phase bound fits the
  qualified bridge's 80-character suffixed run IDs.
- `prepare → template_for → write_template` publishes all copied launch inputs and
  dependency identities before the engine's sole phase-manifest write. The static
  template is explicitly not an active v6 manifest. `verify_manifest` reconstructs
  the exact templates/bootstrap and checks required frozen adapter identities,
  shared nonfactor inputs and the complete engine file inventory.
- `bootstrap → bind_daemon.main → bind` verifies exact-byte sources, Python and
  run/template identity, then checks the actual driver cwd/base/index/live overlay.
  It calls only the pinned bridge's read-only configuration validator and the live
  selector's two sampled censuses—not capture/materialization or private execution.
  The actual driver remains responsible for installing its ordinary overlay.
- The immutable driver creates `<runtime>/<allocated>/repo`, its sibling tmp and
  context-bundle paths, then invokes the shim at
  `infrastructure/driver-source/bench-three-features:1116`.
  Its overlay formatter/skip-worktree operation is
  `bench-agent-profile-common::bench_install_no_recursive_agent_policy`.
  Binding preserves every argument and environment value except the experiment
  manifest path, which points to the independently published actual v6 descriptor.
  Environment values/credentials are not copied into binding receipts.
- `ATTEMPT.json`, actual `EXPERIMENT.json` and `BINDING.json` use separate create-new
  publication. After a claimed failure, `ERROR.json` is retained where publication
  is possible. Unknown/unowned runs and reused/aliased roots fail without overwriting
  another attempt. Partial publication is neither transactionality nor retry permission;
  `ready_for_exec` does not claim successful exec/provider work.
- The real daemon ELF resides at `launch-inputs/work-leaf-orchestrator-real`,
  separate from both private state and the driver's removable checkout. The driver's
  `copy_artifacts` copies the **shim** into `runner-bin/work-leaf-orchestrator`;
  that file is not mislabeled as the actual ELF. Both identities must remain in
  postcapture proof. `execve` replaces the shim with the pinned ELF while retaining
  argv, streams, process group and daemon PID; there is no new provider child.
- Path/inode/hash checks are bounded non-hostile-operator safeguards, not continuous
  filesystem immutability or descriptor-exec protection against hostile races.
  No new public sandbox/security guarantee is claimed.

## Independent checks and retained limits

Commands from repository root:

```sh
python3 -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z -p test_runner_test_first.py -v
python3 -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-full-screen-launch -p test_binding.py -v
```

Both passed: **7 runner tests** (tool `03e3dd`, test report 0.132s) and
**15 binding tests** (`89585a`, 0.927s). This includes complete synthetic prepare/verify
with dependency population inspected before phase publication, immutable/replay/source
failures, real Git/index-overlay mismatch tests, exact argument/environment forwarding,
and actual isolated bootstrap execution of **only `/usr/bin/true`**. The latter uses
the qualified configuration checker and keeps private/validation roots empty.
Synthetic ELF-shaped preparation inputs are not asserted to be real runtime execution.

The implementer's retained test-first evidence is in `VALIDATION.md`: initial
missing-module REDs `7cac9c`/`7bcfaa`, subsequent GREENs, and the concrete excessive
run-ID RED `7b376c` before the 67-character fix. This review inspected those receipts;
it did not recreate older source or rerun the RED cut.

Read-only endpoint check `035567` rehashed **119** actual inputs: 47 attested runtime
source files, two release binaries, the engine, five saved driver scripts, exact
qualified CONFIG, its helper/executable/capsule identities and 53 public capsule
files. All match. The preserved root gate receipt's 44 summaries reconcile to
534 passed / 0 failed / 22 ignored; these are inspected prior gate receipts, not
independently rerun Cargo results. Configuration reconstruction is read at its recorded
cutoff, not claimed as a new current-global reconstruction. Final admission still owns
fresh transport, configuration, runtime and phase-source checks.

Complexity is bounded but not universally constant: hashing/census work is linear
per fixed pass, with sorting/path/Git work inherited from the qualified dependencies.
The engine's O(W×E) repeated frozen-input checks can be quadratic if both waves and
per-run metadata grow; this exact plan fixes one wave and three rows. Existing bounded
O(M²) mount and O(D²) path-resolution qualifications are not removed. The new binder
has no event-pair or workflow-pair scan.

Diagnostic003 qualifies the underlying C15 private-preview → shared patch/check/DONE/
review path. **Required real verification of this new dynamic launch binding is not
yet observed.** The protocol assigns it to the first admitted complete benchmark,
preserving failure without replacement. Source review and `true` execution do not
waive that requirement or authorize generation.
