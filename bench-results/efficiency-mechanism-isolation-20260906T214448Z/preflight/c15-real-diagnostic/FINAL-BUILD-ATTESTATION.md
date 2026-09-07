# Final diagnostic build

The private diagnostic executable is
`/tmp/c15-real-runtime.DBQaUo/private-test-first-diagnostic`, SHA-256
`ffa981f8bf1e9cf105d5199284bf18e3ce14a8412912245dfcea7a9e27c7d182`.
It is an independent read-only executable copy of the final private Cargo test
binary; both bytes match (`fbbfa7`). Its source/build cut is commit
`22a38282d2ae12f184c833de11d377241a18fca0`, including runtime commit `39d2d1c`.
The relevant source, Cargo manifests/locks and architecture match that commit.
Unrelated user configuration/ignore edits remain outside the patch.

Final private diagnostic commands:

```sh
cargo test --offline --locked --manifest-path bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-real-diagnostic/Cargo.toml --all-targets --all-features
cargo clippy --offline --locked --manifest-path bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-real-diagnostic/Cargo.toml --all-targets --all-features -- -D warnings
cargo fmt --manifest-path bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-real-diagnostic/Cargo.toml -- --check
```

Tests pass nine guards with the real target ignored (`632129`/`412c3e`, 3.14-second
build); clippy passes without warnings (`3c84c0`/`d0fedf`, 1.38 s); format passes
(`6e9830`). These use exact root dependency records in the private lock, not newer
cached versions. The complete repository gates and final-byte confined runtime
qualification remain in `../C15-RUNTIME-IMPLEMENTATION-VALIDATION.md`.

`FINAL-SOURCES.json` binds 159 current source/config/public/executable inputs,
including the actual native provider executable and unchanged global-config hash.
The installed system/toolchain distribution is trusted, not exhaustively pinned.
`FINAL-COMMAND.json` names the exact single bounded command and selected environment.
The observer's older `experiment_commit: pending-root-runtime-build-attestation`
value remains an unchanged preparation label; this separate source/build receipt
supplies the actual compiled commit. It is not a claim that the old label was a
verified commit. No observer reinitialization or original preparation rewrite occurs.

No real test has run. Independent final admission/resource checks and a single
create-new admission remain prerequisites. Actual native/source/semantic delivery
qualification follows the original outcome; no exit status alone closes it.
