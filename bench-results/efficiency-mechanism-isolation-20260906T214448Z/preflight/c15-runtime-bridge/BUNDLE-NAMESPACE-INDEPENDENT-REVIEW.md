# C15 ordinary bundle namespace: independent review

Recorded 2026-09-07 19:57:40 UTC. Scope is the diagnostic-001 startup failure and the two-file, benchmark-private correction. No provider, private execution qualification, or diagnostic retry was performed by this review.

## Verdict

No remaining introduced source blocker was found at the reviewed hashes below. Independent `cargo test --lib --all-features private_test_first -- --nocapture` passed **38 tests, 0 failed, 4 ignored** (tool `607746`, exit 0). Five new parent tests are included; their two isolated subprocess children are automatically invoked by those parents. The other ignored tests are the existing subprocess child and separately gated confined qualification. The exact-source required repository gate receipts are green and independently reconciled below. Real-agent verification is **not green**: diagnostic 001 failed before provider launch and remains the original outcome. The parent reports explicit user approval for one separately recorded corrected diagnostic; this source review neither executes nor itself admits that observation.

| Reviewed source | SHA-256 |
| --- | --- |
| `src/bench_private_test_first_bridge.rs` | `a8395dfe870345056a64f4a53691248238a7875ed4e6a799212ec48126db6fbc` |
| `src/bench_private_bundle_path_tests.rs` | `bd2bd0817d12dd6dfbe6f50876d6cb610d58f70af8dfb3d5c67158e429f22bc2` |

## Retained failure and exact cause

[Diagnostic 001 terminal](../c15-real-diagnostic/TERMINAL.json) retains exit 101, `0.014849746017716825` seconds, and the recorded interval 19:42:55.818266–19:42:55.833120 UTC. Its [stderr](../c15-real-diagnostic/PROCESS.stderr) identifies `harness.rs:170:66`, the `prepare_agent_launch` unwrap, and `private evidence root overlaps or aliases the ordinary bundle namespace`.

The frozen source chain is `harness.rs:170` → `cli.rs::CommandChat::prepare_agent_launch` → benchmark activation/load → `bench_private_test_first_bridge.rs::initialize` → `validate_bundle_separation`. The original resolver canonicalized the first existing ancestor and unconditionally joined the remaining suffix. For an already-existing canonical ordinary parent, that suffix is empty. Rust's `PathBuf::join("")` adds a trailing separator, so the subsequent **OsStr byte** comparison rejects the same canonical directory. An independent standard-library-only probe produced `parent="/tmp"`, `suffix=""`, `resolved="/tmp/"`, `equal_bytes=false`, `equal_path=true` (tool `6638cb`). This is a representation mismatch, not an actual overlap.

The six relevant frozen harness/runtime source identities were independently recovered from selected commit `22a38282d2ae12f184c833de11d377241a18fca0` and matched the original admission (tool `bb32ca`). That commit includes runtime commit `39d2d1cdab11b665b00c267c794f86aa087eda79`. The root's original terminal endpoint check reports all 167 admitted pins matching before subsequent live-source correction; this review does not relabel the modified live source as those original bytes.

The rejection occurs before the private `validation` directory, Python bridge invocation, activation trace creation, harness budget assignment/watchdog, or backend launch. `CodexBackend::new` constructs `CodexAppServer` with no process; actual spawning requires the later `ensure_started` path. The retained diagnostic has no `observation/app-server`, `observation/process-invocations.jsonl`, `prompt-events.jsonl`, or `HARNESS-RESULT.json`; its private preview root remains empty. Together with the exact panic location and call chain, these establish no app-server or private test work for this attempt, not merely missing usage evidence.

One ordinary lifecycle side effect is retained: the precreated empty ordinary bundle parent is absent after panic unwind. `CommandChat::new` owns an ordinary `ContextBundleStore`; existing `ContextBundleStoreInner::drop` at `src/orchestrator.rs:337` removes its store directory and then attempts to remove the empty parent. Thus the original ordinary parent cannot be described as unchanged-empty. This is ordinary cleanup, not a private command or provider call; no retry against the resulting missing parent occurred.

## Correction and automatic coverage

The resolver appends the suffix only when nonempty. It retains the original absolute/path-depth/parent-component restrictions, canonical byte alias rejection, and both-direction namespace separation. Existing canonical empty and populated parents pass; missing descendants remain uncreated; equal/contained/containing, symlink alias, and relative paths remain rejected. No prompt, parser, normal command, store counter, public API, or provider behavior is otherwise altered.

The new startup fixture drives actual `CommandChat::new` → `prepare_agent_launch` for both precreated empty and populated ordinary parents. Its backend panics on any launch/send. The real Rust bootstrap exact-loads an explicitly synthetic, validation-only sentinel: this proves startup/control-flow admission, **not** qualified confinement or provider behavior. Assertions retain project/ordinary inventory, one validation directory and activation event, and the ordinary store counter sequence without an extra hidden allocation. Test-local `ManuallyDrop` keeps the ordinary fixture stores alive during inventory checks; it is not runtime behavior.

The owner reports a clean combined RED after restoring only the old resolver: session `73738`, completion `232950`, **2 passed / 3 failed / 2 ignored**, including the actual startup error. Final focused GREEN is session `47210`, completion `21f56d`, **5 passed / 2 ignored**. Independent full private-test-first coverage above passed at the exact final hashes, rechecked in tool `4fd910`.

The complete [correction note](BUNDLE-NAMESPACE-CORRECTION.md), SHA `842efb1f834c86d1d62f92d6024e32821e7f4eb7ae30d008a9ae04f5f058e73b`, and [gate receipt](BUNDLE-NAMESPACE-CORRECTION-GATES.json), SHA `d8ad6a78c2673ca6f889045e9e9ace6fa668cba23750f9dd9c6aaca1a5c8dda8`, retain exact-source fmt (`48b6cd`), clippy (`0d12e4`), full all-target tests (`aa2c67`), and default library tests (`3e6e4c`), each exit 0. Independent parsing of the retained 56,991-character complete all-target output reproduces all 44 indexed summary lines and **534 passed / 0 failed / 22 ignored**; default library coverage is 50 passed. These are owner-executed gate receipts, not an additional independent full-suite run. Their bootstrap/confinement/provider limitations agree with this review.

This is a constant-size conditional inside the existing ancestor resolution; it introduces no quadratic scan or new allocation namespace. The previously disclosed bounded path-depth/ancestor work is unchanged. The architecture/operator/private-preview contracts still describe the intended canonical, disjoint namespace; no public architecture or workflow documentation change is required for this correction. Existing committed tests and frozen diagnostic artifacts are not modified.

## Original artifact preservation

The following original identities and the admitted executable were rechecked after the new source review (tool `4fd910`):

| Original artifact | SHA-256 |
| --- | --- |
| `ADMISSION.json` | `72fc8e1c1cc7bf1b819baeca109ac70b6bc1620f0ab9f0b98a4098c79286b85a` |
| `ATTEMPT.json` | `996237fe5b000c53712292799665226cd11c25c56cbe50b2a1ec092fe8dee4f0` |
| `TERMINAL.json` | `316e10157dbd68f113ddc562210857771fa83ebfb5ff1d1ecb85a627671ece0c` |
| `PROCESS.stdout` | `db7689ba315ba2fc3122f9579e87e340d13554108fe0e1c24e4d5f17262f4365` |
| `PROCESS.stderr` | `b09cd5ce93358ba84f4ad25c43c8cf1796717fdd5379acf9c26baa37da51f540` |
| `/tmp/c15-real-runtime.DBQaUo/private-test-first-diagnostic` | `ffa981f8bf1e9cf105d5199284bf18e3ce14a8412912245dfcea7a9e27c7d182` |

Prior qualification used a missing ordinary parent, not this precreated-directory startup shape. Its passing receipts remain valid within that narrower coverage; neither they nor the synthetic startup tests replace the still-missing real-agent result.
