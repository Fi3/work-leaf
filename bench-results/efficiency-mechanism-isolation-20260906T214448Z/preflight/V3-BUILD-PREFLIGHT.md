# V3 isolated release-build preparation

This is a provider-free preparation plan, not a build receipt, allocation, admission or launch. No command below was executed by this review. Existing frozen phase artifacts remain read-only.

## Checked current state

- The reusable driver repository is `infrastructure/driver-source`, clean under `git status --porcelain --untracked-files=all`, at `3c907f7266b63efa1558213f3e49587e999b899d`. It contains benchmark base commit `c92a0b7060a36eac6db2d869b85e589a7a9480f9`. It remains the future runner's `--source-repo`; the experimental compiler snapshot is a separate object.
- The five driver digests match the closed phase: `bench-three-features` `d2487780c63c14021904b8a3c882d54fe231c5846f4a7f57fe955f50201f5644`; `bench-candidate-common` `2971cb9729626ddc7a12bd688df8d80ee20f904bb48d52ab846c1e9f9751711f`; `bench-validation-common` `235ef644d692098a670c4e61c3572f60f8830c445896f3639a56430c57781dca`; `bench-agent-profile-common` `de9803658bc8ac41a9356436be2c8b8edfce9d7bef9176841e067b3478633d4f`; `bench-progress-common` `8d8977f3d79ad721f09973f883b662f0aaf2f804b522390ff564337398b292f5`.
- At inspection, repository HEAD was `c315a0680ba099e5d6e91566e4774118f00f76ba`, with uncommitted v3 changes in `src/bench_experiment.rs` and `src/orchestrator.rs`. That HEAD alone therefore does not identify the finished v3 source. Wait for the implementation owner's final source/review/commit identity before archiving.
- Existing `target/release/work-leaf` is still SHA-256 `07c403c5290371a7e1700cb9d03127b9b9a8d384c55a98692115b2fbf01fe1db`; `work-leaf-orchestrator` is `9f9a6c88a45bd39a99b6c7eeae82e0037575592ed75d8a41a013d3792ade95fa`. These are the v2 release identities, not evidence of a v3 build.
- Both current `bench-observer/target/release/bench-observer` and the closed phase's frozen observer have SHA-256 `238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`. Reuse that exact executable as the future `--observer-bin`; do not rebuild or change its semantics for this factor.
- The subscription wrapper is unchanged: `bench-results/efficiency-measurement-gate-20260906/subscription-codex`, SHA-256 `0977db361b4477e2bf68ea08c05571a4a35fb1e5c958cc9a953f2cc738de7078`. Its actual launcher `/usr/lib/node_modules/@openai/codex/bin/codex.js` remains `61b0194f3bb6534439c8d26a3ed57d0805f84b884588b761795323eeb92fcf70`; native CLI `/usr/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/bin/codex` remains `56ef98ab4032d317ab26e9b5e5a175650717351edb16ed9cde0cb6d1734d62da`. These are identity-only external files, never copied credentials.

## Exact compiler inputs

The root package has two binary names in `Cargo.toml`: `work-leaf` and `work-leaf-orchestrator`. Compile both with `bench-experiments` in the same release build. The observer is a separate Cargo package and is not a root workspace member.

The runtime snapshot must contain `Cargo.toml`, `Cargo.lock`, every `src` source, **and** `web-ui/index.html`, `web-ui/styles.css`, `web-ui/app.js`. `src/http_controller.rs:645`, `:649`, `:653` embeds those three assets with `include_bytes!`; a `Cargo + src` archive alone cannot build the actual runtime. The prior 31-file attestation did not list these asset inputs individually. The v3 attestation should identify them explicitly without rewriting any prior attestation.

Do not copy the working tree wholesale: `.codex/config.toml`, `.gitignore`, credentials, user configuration, old run captures and build outputs are not inputs to the selected compiler snapshot. The archive must come from the exact final reviewed source commit, not the currently dirty checkout. Preserve that commit plus the archive/input hashes, command, toolchain, output binary hashes and excluded paths in a create-new `preflight/BUILD-ATTESTATION-UNTRACKED-READS.json`.

## Build commands after final source approval

These commands require the operator to supply `V3_SOURCE_COMMIT` as the final reviewed 40-character commit. They create a fresh build directory under the study infrastructure so allowlisted source evidence remains inside the runner's required repository-relative evidence root. They do not use or overwrite the old `target/release` directory.

```bash
set -euo pipefail
: "${V3_SOURCE_COMMIT:?Set the final reviewed source commit}"
[[ "$V3_SOURCE_COMMIT" =~ ^[0-9a-f]{40}$ ]]
V3_STUDY=/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z
V3_BUILD_ROOT=$(mktemp -d "$V3_STUDY/infrastructure/untracked-read-build.XXXXXX")
mkdir "$V3_BUILD_ROOT/source"
git -C /home/user/src/work-leaf cat-file -e "$V3_SOURCE_COMMIT^{commit}"
git -C /home/user/src/work-leaf archive --format=tar "$V3_SOURCE_COMMIT" \
  Cargo.toml Cargo.lock src web-ui/index.html web-ui/styles.css web-ui/app.js \
  | tar -x -C "$V3_BUILD_ROOT/source"
cargo build --manifest-path "$V3_BUILD_ROOT/source/Cargo.toml" \
  --target-dir "$V3_BUILD_ROOT/target" --release --locked --offline \
  --features bench-experiments --bin work-leaf --bin work-leaf-orchestrator
sha256sum "$V3_BUILD_ROOT/target/release/work-leaf" \
  "$V3_BUILD_ROOT/target/release/work-leaf-orchestrator"
rustc -Vv
cargo --version
```

Use the ordinary recorded Cargo/toolchain configuration. If an offline dependency is unavailable, stop and record it before choosing an explicit dependency-fetch step; do not improvise a credential copy or silently switch toolchains. The selected exact source archive has no `build.rs`, path dependencies or other embedded files at the inspected source state. Recheck that assertion against the final v3 source commit before building.

Each archived input requires its SHA-256 in the attestation. Retain the source directory and the two resulting binaries until phase freeze. Do not label a build as reproducible merely because its inputs were pinned; the actual output hashes identify the admitted binaries. Required format, all-target/all-feature clippy/tests and the owning agent's real subscription v3 delivery checks must already pass against the same final source. The final release binaries require their own bounded runtime verification before admission if existing real checks used a different executable identity.

## Runner preparation wiring

`runner_untracked_reads.py::prepare` delegates to the unchanged, hash-pinned `runner_work_units.py::prepare`. It requires a complete fresh allocated schedule, a clean driver Git source, an explicit frozen task SHA, the reviewed subscription wrapper and a readable global configuration snapshot. It does not infer a compiler source attestation from the supplied binaries. The operator must supply that evidence explicitly.

Required arguments and exact source choices:

| Argument | Input |
| --- | --- |
| `--source-repo` | `$V3_STUDY/infrastructure/driver-source`, unchanged clean clone |
| `--bin-dir` | The new `$V3_BUILD_ROOT/target/release` containing the selected two runtime names |
| `--observer-bin` | `$V3_STUDY/phases/work-units-01/infrastructure/bin/bench-observer`, after matching the unchanged observer hash |
| `--subscription-wrapper` | `/home/user/src/work-leaf/bench-results/efficiency-measurement-gate-20260906/subscription-codex` |
| `--protocol` | `$V3_STUDY/PROTOCOL-UNTRACKED-READ-INLINE.md`, after final prelaunch freeze |
| `--scorer-config` | The existing frozen canonical scorer configuration, SHA-256 `4e35a729f53f030eb1ebf73eaa8f27bd6b66fed67411a94f5339dcfb0562ab70` |
| `--task-list-sha256` | `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a` |
| `--global-config` | `/home/user/.codex/config.toml`; hash/safe summary only, no copy |
| `--identity-file` twice | Actual Codex JS launcher and native binary paths listed above |
| `--evidence-root` | `/home/user/src/work-leaf`, required by the v3 adapter |
| `--phase-root`, `--runtime-root`, `--schedule` | Fresh operator-selected paths and the one-shot prospective allocation; not supplied or created by this review |

Add each allowlisted source input, new build attestation, finalized v3 analyzer/accounting/tests/design/protocol and required predecessor/scorer/fixture helpers via repeated `--evidence`. Preserve their original repository-relative hierarchy. The v3 adapter automatically includes its own runner/allocation dependencies; the engine additionally includes its trust helper. It does **not** automatically add the v3 analyzer/accounting module or compiler inputs. Keep the old scorer/analyzer dependency hierarchy complete when frozen under `infrastructure/evidence`; copying a top-level scorer JSON alone is insufficient.

Preparation copies runtime files to new phase `infrastructure/bin/{work-leaf,work-leaf-orchestrator}`, the unchanged observer to `infrastructure/bin/bench-observer`, the wrapper to `infrastructure/provider/codex`, driver copies to `infrastructure/drivers`, and the evidence allowlist below `infrastructure/evidence`. It writes create-new manifests and separately shuffles launch order; `RUN-ONCE` is only claimed by the later explicit `run` mode. A failed partial prepare must be retained and investigated, not overwritten in place. This plan deliberately does not allocate, prepare or run a phase.

The actual admission gate must rehash the final build receipt, copied binaries, full source inventory including web assets, unchanged driver/observer/provider identities, finalized helpers and fresh configuration baseline. The source/binary freeze must precede every provider observation; no v2 binary or pending implementation hash is a substitute.
