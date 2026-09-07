# V3 runtime validation

Automatic runtime gates pass. Real-subscription status: **pending actual receipt review**.
This record does not admit benchmark observations. Two earlier diagnostic attempts remain retained:
their read handoffs completed, but the launch omitted the observer primary marker, so required
raw/grace capture was inactive. Those attempts are not passing primary-accounting diagnostics.

## Source identities

Repository-relative paths; hashes are SHA-256.

| Source | SHA-256 |
| --- | --- |
| `src/orchestrator.rs` | `71610cc88fef911fddd2099a1237166df51f8f31a9528268085e7f65477f792b` |
| `src/bench_experiment.rs` | `951c0a96795582c51473aaba59b1fc2b51ffe2a440414174221b3111441e40ad` |
| `docs/architecture.md` | `8377838c35e99cac89d7d67691ff9b10ff515b5270d32502c3bb6aa63e0b3ecd` |
| `tests/bench_read_inline.rs` | `e225f5a673458572278bd84e011c83ab678890baeccd64e7a7ec95084fbcdf78` |
| `src/bench_read_experiment_tests.rs` | `8e095c7a9f65bee8182b7803d116e22fa298935b9268a99508bff8dabeeb6c7e` |
| `tests/bench_read_inline_subscription_smoke.rs` | `8f80942005f3dc305cb29f8cef8ab8efe7b3e9604491f3fe5159f8e8557428ed` |

The smoke executable is `target/debug/deps/bench_read_inline_subscription_smoke-dd4acd1a51f6ca92`,
SHA-256 `89942613cf9065225e39b3ac73e1812fa6e9108db2e6266fb6fbbedb7ebe42d0`.

## RED and automatic gates

Observed RED executions, before their corresponding implementations:

- `cargo test --test bench_read_inline --all-features read_inline_variant_changes_only_successfully_bundled_untracked_component -- --nocapture`
  exited 101 with `unsupported manifest schema` under the original v1/v2-only implementation.
- The new read-budget test rejected the copied patch-ACK budget before its accepted boundaries were
  limited to exactly two ordinary file-read followups.
- `cargo test --test bench_read_inline_subscription_smoke --all-features teardown_rejects -- --nocapture`
  exited 101: both mixed-success/error and empty-turn-identity regressions failed.
- `cargo test --test bench_read_inline_subscription_smoke --all-features observer_preconditions -- --nocapture`
  exited 101: missing primary marker and incompatible observer configuration were not rejected.

Final GREEN commands:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --test bench_read_inline --no-default-features
cargo test --test bench_read_inline_subscription_smoke --all-features
git diff --check
```

The final full-suite execution completed with exit 0 after both smoke-verifier guards. The read
integration target has six passing tests plus one ignored subprocess entry; the private read module
has three passing tests. The default-build target has five passing tests plus one ignored entry.
The smoke target has eight passing automatic tests plus the separately opt-in real case.
No existing committed test was edited. Automatic tests do not substitute for real-agent verification.

Coverage includes default/inactive/control and v1/v2 identity; exact threshold boundaries; successful
and failed bundle writes and next allocation; coalescing, path order, mixed repeated/missing/explicit
bundle reads; UTF-8 and adversarial marker bodies; normal snapshot clearing after edits and failed-send
tracking; unchanged conflict refresh, policy, ACK and command text; owned ranges and write-error
propagation. Smoke preconditions check the matching nonempty primary marker, work-leaf condition,
absence of a parent invocation, raw capture `1`, grace `1000`, resume `forward`, and project inventory
`1` before fixture mutation or provider setup. Tests inject lookup functions without environment
mutation or credential-value output. Teardown requires actual forwarded interrupt bytes, a successful
typed acknowledgement, and the matching nonempty terminal turn; mixed result/error is not success.

## Independent pre-patch default reference

Root's provider-free reference lives at `/tmp/work-leaf-v3-default-reference.QjW6zT`; its checkout
HEAD is `9c1dedc6a57792c61f93e95a920d7586b23151b6`. The retained baseline executable hash is
`f5c14d64232246480ec112bcd99e2b7b294e6c7eb109f308a19eeee9b170503a`. The initial fixture snapshot
hash is `5233c35e5f3881a745a4e5a9f793615d7212262f97657c0011d33fdcc1a53118`.
The reference checkout's resulting two runtime files match the hashes above.

Independent read-only replay of the retained JSON confirms ten matrix prompts and two bundle-failure
prompts are identical after replacing only each fixture root and generated bundle directory with
shared placeholders. Bundle contents and per-step source inputs are also identical. This comparison
covers the initial twelve-prompt fixture, not later additional state/send-failure cases.

| Retained reference result | SHA-256 |
| --- | --- |
| `baseline-matrix/result.json` | `3a26ae7569554a465d3ca14967290d9b84372ecc3f274a824e9889d218480442` |
| `current-matrix/result.json` | `127adee7c8e5c5fe7e5699b3785ceec9d327474e7db29472ab11148fca0daa0c` |
| `baseline-bundle-failure/result.json` | `c9d1d215599c2664bdc80bb87cae6773bfda26b73d12705771b0954a218b4e03` |
| `current-bundle-failure/result.json` | `8416cdca1da238290dc80178deb46045c46a6e586de183547a0c2d9b15953cd4` |

## Ownership, documentation and remaining gate

`src/lib.rs:3` excludes the private experiment module without the nondefault feature. Read hooks and
metadata are similarly cfg-gated. `send_file_read_response`, its private renderer and the private
adapter preserve the existing orchestrator/provider ownership chain and public interfaces. Both v3
arms perform ordinary reads/bundle allocation once, construct both candidates, and differ only in
the owned successfully bundled component. No runtime crypto, second read, public configuration API,
provider turn, timing policy, command or validation-policy change belongs to the factor. Rendering
and three-way snapshot merging remain linear in held bytes and snapshot count.

The complete `docs/` catalog consists of architecture, benchmark operator policy and orchestrator
file workflow. Architecture documents the private v3 boundary. Ordinary file-workflow behavior and
the README's public launch/replay surfaces remain unchanged. Operator-policy and study STATE/protocol
admission references are parent-owned; this validation record does not edit their authority.

The real scenario is an actual three-response, one-session read of a >16 KiB generic diagnostic
fixture, an ordinary repeated `--force` read, and done. Control must deliver the bundle manifest;
treatment must deliver its exact inline candidate; the repeat must remain compact and identical.
Actual capture provenance, raw/grace/project-inventory activation, selected provider delivery and
terminal receipts require independent review before the real gate can be green. No provider call
or benchmark observation was launched by the implementing subagent.
