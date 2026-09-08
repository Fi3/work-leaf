# Host binding qualification

This source cut contains only `runner_test_first.py`, its new tests, and the new
`preflight/c15-full-screen-launch` helper/tests/documents. The original engine,
driver scripts, runtime, observer, bridge and prerequisite helpers are unchanged.
No benchmark, provider, private executor or accounting operation belongs to these
checks. The finite operator scope is [CONTRACT.md](CONTRACT.md).

## Exact source cut

| File | SHA-256 |
| --- | --- |
| `../../runner_test_first.py` | `624fed61e6d8d311788827c0f6186348e56a0e5e8d7be64fa551d3fe5d3e124a` |
| `../../test_runner_test_first.py` | `42ee3720b70533f3730c903d004abdbf3069c4dda585ada3a4e235ea37264387` |
| `bind_daemon.py` | `aa1800d574df9c7dee1ad899619594060a68cd28fe61d5a2f0ed9d63e16e3155` |
| `test_binding.py` | `85cb48dae719196d1307834bd30a04edf818744ee52e9cf3c06df6cb2ae696cf` |
| Original `../../runner_work_units.py` | `2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e` |
| Original `../c15-runtime-bridge/bridge.py` | `6af6c3f24f96a7b95514a0db702a775b895a8d89f5dc8a2db9aa1329db9d4d26` |

## Test-first evidence

- Initial new binding tests: RED tool`7cac9c`,11 missing-module errors before
  implementation. Initial runner tests: RED`7bcfaa`,3 missing-module errors.
- First implementation: binding11PASS`a0636a`; runner3PASS`159adc`.
- Complete synthetic engine prepare/verify, immutable real-ELF/template pinning
  before the sole phase-manifest publication, replay/mutation refusal and loaded
  source drift: runner6PASS`e423e1`.
- Root's ID-length finding: RED`7b376c` proves a68-character phase produced a run
  beyond the bridge's80-character bound. The67-character limit is GREEN in the
  complete7-test runner suite`e7a6f4` (0.113s).
- Final binding suite`29b314`:15PASS (0.922s). It includes wrong cwd, exact observed
  Git-base/overlay mismatch, empty/disjoint private root, changed static template
  and source endpoint, source/output symlinks, replay, environment identity, exec
  failure, unchanged live HEAD/index/source, and actual bootstrap execution.

The final bootstrap test uses the exact source-pinned bridge's read-only
configuration validator and live census, then execs only `/usr/bin/true`. Its
private root and validation directory remain empty. Invalid `PYTHONPATH` is
ignored by isolated Python. It never calls `bridge.execute`, `capture_selection`,
the private executor, a provider or the benchmark driver.

Commands, from repository root:

```sh
python -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z -p test_runner_test_first.py
python -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-full-screen-launch -p test_binding.py
```

The fresh full-screen runtime's required Cargo gates belong to
`../test-first-screen/ROOT-GATES.json` and the parent's build attestation; this
Python-only adapter does not relabel those gates or the003 diagnostic as real
verification of its new binding. Independent source review is separate. The first
admitted natural benchmark must establish the actual binding/provider chain,
retaining failures without a replacement. No ready-for-generation claim is made
by fake execution, and no token-saving result is part of this receipt.
