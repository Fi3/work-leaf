# Diagnostic Infrastructure

## Purpose

The two new conditions are generated from hash-pinned copies of the normal benchmark drivers. The
generator refuses an unknown source hash and writes each diagnostic driver into its run-local
runtime directory. The frozen source checkout remains clean.

No Work Leaf Rust source, normal benchmark launcher, task, model profile, observer accounting, final
gate, candidate replay, or scorer is changed.

## Compact Direct Linearizer

The generated direct driver differs from `bench-three-features-direct-common` in two operational
places:

1. It resolves shared helpers from the immutable source checkout because the generated file lives in
   an isolated runtime directory.
2. Its linearizer planning prompt lists the exact reviewed provisional commits under each of the
   three original feature requests and identifies the fixed stack base.

Implementation prompts, native Codex tools, feature sequence, reviewer prompts, review/fix loops,
acceptance prompt, timeouts, final checks, and reporting remain unchanged.

The generated driver SHA-256 is
`7d177e3c5798d2c321b761bbdc2c7270a01408c101e7938f6e277c1cd1eb08f9`.

## Sequential Diagnostic Work Leaf

The generated Work Leaf driver differs from `bench-three-features` in three operational places:

1. It resolves shared helpers from the immutable source checkout.
2. It labels the schedule `sequential-diagnostic` and submits feature 1 initially.
3. After a feature has a reviewed Work Leaf commit and no workflow is busy, it submits the next
   feature. Each newly submitted feature receives a fresh two-hour deadline, matching the direct
   driver's per-feature allowance.

The same frozen Work Leaf binaries still create agents, inject prompts, apply structured edits, run
locked commands, review patches, and linearize history. Direct reads and completed responses are
enabled through the same existing controls used by condition `C`.

The generated driver SHA-256 is
`88bc245b871343ae162ec34fe6bfade3528f6f4e6daeb3ab0a430060b489c936`.

## Launch Order

`SCHEDULE.tsv` contains six unique primary attempts. Each of two batches contains both conditions
and exactly three concurrent top-level workflows. Runs are independent groups, not pairs.

The launcher preserves every admitted run and refuses an existing attempt ID. It pins GPT-5.5,
`xhigh`, the observer and Work Leaf binary hashes, source hashes, generated driver hashes, and the
fixed benchmark base before each launch.

## Provider-Free Verification

The test suite verifies source immutability, exact source hashes, generated substitutions, shell
syntax, six unique schedule rows, mixed batches, maximum parallelism, profile settings, and a full
preflight that regenerates and rehashes both diagnostic drivers.

```sh
python3 -m unittest discover \
  -s bench-results/efficiency-mechanism-attribution-20260830T081131Z \
  -p 'test_*.py'
bench-results/efficiency-mechanism-attribution-20260830T081131Z/run-attribution-control --check
```
