# Private-test-first diagnostic guard qualification

These are provider-free fixture guards and a compiled ignored real-agent harness,
not an admission or completed real run. The private crate uses the current Work
Leaf runtime with `bench-experiments`; its lockfile derives from the exact root
lock plus the private fixture package. No dependency upgrade or root lock edit
is part of the fixture. Guards test explicit controller-owned roles/call bounds,
activation/subscription/observer settings, natural task input and public closure.

Initial RED: `cargo test --offline --manifest-path .../c15-real-diagnostic/Cargo.toml
--test guard_contract` fails before `guards.rs` exists (`3fd700`, exit 101).
The implementation then passes five tests (`4c969b`). A second RED reproduces
the initial fixture guard's rejection of the ordinary known-author review-fix
route (`c9ef3c`, exit 101). The guard retains that normal send route within the
same finite overall call ceiling; duplicate launches and unknown third roles
remain rejected. Existing committed repository tests are unchanged.

The natural-task test fails before `fixture_request` exists (`2f530e`) and then
passes with seven total tests (`410d73`). The terminal-cut tests fail before
`settled_turns` exists (`62ac81`), then pass with nine tests (`2ee92d`). The real
target initially fails compilation because its harness file is absent (`ca4d37`);
it compiles after implementation but remains ignored without explicit admission.

Private-crate gates at the initial harness cut:

```sh
cargo fmt --manifest-path bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-real-diagnostic/Cargo.toml
cargo clippy --offline --locked --manifest-path bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-real-diagnostic/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --offline --locked --manifest-path bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-real-diagnostic/Cargo.toml --all-targets --all-features
```

Format passes (`e22b62`); clippy finishes cleanly (`10cd9a`/`56b709`);
all nine tests pass and the real test is ignored (`ce87de`/`3199d0`). The earlier
six-test pre-harness gates (`5feddc`, `2547df`/`eefa80`, `42c8a3`) remain historical.
Per-call role state is constant-sized; public frames use indexed joins with at
most eight admitted turn starts. No quadratic algorithm or public API extension
belongs to this private fixture. Independent harness review, final source/build
freeze, root-repository gates and actual real-agent verification remain required;
these guard results do not replace them.
