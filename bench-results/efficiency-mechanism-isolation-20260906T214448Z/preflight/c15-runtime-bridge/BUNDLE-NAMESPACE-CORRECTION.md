# Private preview namespace qualification

The v6 evidence root must be canonical and disjoint from the ordinary bundle parent. An ordinary parent may already exist, be empty or populated, or have an absent descendant suffix. Namespace validation does not construct a bundle store, allocate bundle identities, create directories or remove evidence. Default builds and v1–v5 do not enter this private bridge.

## Retained failure and cause

`preflight/c15-real-diagnostic/TERMINAL.json` retains `private-test-first-diagnostic-001`, launcher exit 101, duration 0.014849746 seconds, and matching admitted source endpoints. `PROCESS.stderr:2–3` records the error at the actual `harness.rs:170` call to `CommandChat::prepare_agent_launch`: the private evidence root allegedly overlaps or aliases the ordinary bundle namespace. This diagnostic is not a successful provider or private-test workflow; no retry is part of this correction.

The exact call path is `src/cli.rs::CommandChat::prepare_agent_launch` → `private_test_first::active` → `bench_experiment::active/load` → `bridge::initialize` → `validate_bundle_separation`. In the original resolver, an existing ordinary parent is itself the nearest existing ancestor. `strip_prefix` yields an empty suffix; `PathBuf::join` appends a separator. The strict `as_os_str` comparison then rejects those different bytes despite the canonical sibling directories being disjoint. The comparison is appropriate for rejecting aliases; appending an empty suffix is the defect.

`src/bench_private_test_first_bridge.rs::validate_bundle_separation` preserves the canonical ancestor bytes and pushes only a nonempty suffix. The existing absolute-path/depth/parent-component gates, byte-exact alias comparison and bidirectional overlap check remain authoritative. This is not a normalization relaxation, an ordinary bundle lifecycle change or a new filesystem read. Added work is a constant empty-suffix branch; existing bounded ancestor-resolution complexity is unchanged.

## Test-first evidence

The new automatic tests are confined to `src/bench_private_bundle_path_tests.rs`, included only in Linux feature-enabled unit tests. No committed test was edited.

- Clean direct RED: `cargo test --lib --all-features bundle_path_tests -- --nocapture`, tool `cece86`, exit 101; existing empty and populated sibling parents both fail, missing descendants and alias/overlap negatives pass (2 passed, 2 failed, 1 isolated child ignored).
- Actual-startup RED: the original resolver was restored before compiling the new startup test. Session `73738`, completion `232950`, exit 101; 2 passed, 3 failed, 2 isolated children ignored. The third failure is the actual `CommandChat::new` → `prepare_agent_launch` startup call, with the same namespace error as diagnostic 001. Its retained fixture is `/tmp/work-leaf-private-bundle-path-295-1788810793629074549-1`.
- Final focused GREEN: session `47210`, completion `21f56d`, exit 0; all 5 automatic tests pass, with 2 child-entry tests ignored by the outer harness and invoked automatically by their parent tests. Both empty and populated startup cases execute successfully.

Direct tests cover existing empty/populated parents, missing descendants, equal/containing/contained roots, existing and missing-descendant symlink aliases, and a relative parent. Before/after fixture inventories preserve project/ordinary file bytes and do not follow symlinks. Counter probes verify consecutive ordinary store identities and an unchanged held store's debug state; the initializer does not secretly allocate an ordinary store. The actual startup case counts the one store owned by `CommandChat` separately.

The startup fixture uses the real v6 manifest loader, initializer and exact-source Rust Python bootstrap. Its small pinned validation-only sentinel checks the actual `validate` request and executed-source hash, then returns a closed configuration result. A backend whose launch/send methods panic is never invoked. Only the validation directory and activation row exist: there is no proposal, private execution, shared apply or policy delivery. This qualifies the exact startup boundary, not the real executor or provider. It is not an independently written preflight validator bypassing Work Leaf startup.

An earlier developmental test run (`5e3ad0`) also had an unrelated fixture inventory failure: dropping its ordinary store removed an empty configured test parent through the existing `ContextBundleStoreInner::drop`. The final isolated child uses `ManuallyDrop` for its harmless unwritten stores and does not construct stores in rejected namespaces. It therefore measures validator effects without ordinary teardown interference. That fixture failure is not attributed to the runtime validator. Successful new ephemeral fixtures are removed; RED fixtures and the original diagnostic remain retained.

## Gates and identities

`BUNDLE-NAMESPACE-CORRECTION-GATES.json` retains the exact captured RED/GREEN output, standalone fmt/clippy/default receipts and complete full-suite output with all 44 target summaries.

| Gate | Actual result |
| --- | --- |
| `cargo fmt` | `48b6cd`, exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | session `79261` → `0d12e4`, exit 0, no warnings |
| `cargo test --all-targets --all-features` | session `95818` → `aa2c67`, exit 0; 534 passed, 0 failed, 22 ignored, 44 targets |
| `cargo test --lib --no-default-features` | `3e6e4c`, exit 0; 50 passed |
| `git diff --check` | `a1de7c`, exit 0 |

Final source hashes:

```text
a8395dfe870345056a64f4a53691248238a7875ed4e6a799212ec48126db6fbc  src/bench_private_test_first_bridge.rs
bd2bd0817d12dd6dfbe6f50876d6cb610d58f70af8dfb3d5c67158e429f22bc2  src/bench_private_bundle_path_tests.rs
```

Retained predecessor hashes:

```text
316e10157dbd68f113ddc562210857771fa83ebfb5ff1d1ecb85a627671ece0c  preflight/c15-real-diagnostic/TERMINAL.json
b09cd5ce93358ba84f4ad25c43c8cf1796717fdd5379acf9c26baa37da51f540  preflight/c15-real-diagnostic/PROCESS.stderr
72fc8e1c1cc7bf1b819baeca109ac70b6bc1620f0ab9f0b98a4098c79286b85a  preflight/c15-real-diagnostic/ADMISSION.json
67a79895b8bacd26f2b41710ca4f8ad64fbc408abecb530da2a3d2f855744563  preflight/C15-RUNTIME-IMPLEMENTATION-VALIDATION.md
```

The documentation catalog consists of `docs/architecture.md`, `docs/benchmark-operator-policy.md` and `docs/orchestrator-file-workflow.md`. Their architecture/authority/ordinary-bundle contracts remain unchanged; this private implementation correction requires no product documentation edit or public API change. Architecture SHA remains `20c1f2bd4cdec7f0ba4fd613eecc2fd4156554f4b4d46d53bdfbb4f6773df921`. Only the bridge resolver and its new test-module include modify tracked runtime source.

Independent review and any later source-frozen real-agent qualification are separate gates. No provider call, diagnostic replacement, old prerequisite rewrite, admitted binary rebuild or commit was performed by this correction task. The failed diagnostic is not converted into a passing run, and automated bootstrap coverage is not a green real-agent verification.
