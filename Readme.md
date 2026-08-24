I often find my self juggling trough the many codex session opened across even more tmux panes for a
single project. I love my ssds and I'm always short on available space, so many working trees are
kinda scary. The goal of work-leaf is to replicate what instruments like claude-squad offers, but
without using the git work-tree functionality. What I want is an highly opinionated agent orchestrator
for coding. The highly opinionated part is the flow of work: open many agent that change the code
with atomic commits, review with an agent every single commit, patch them, rewrite git history to
have the smallest diff possible and not too many commit, review again agent/human.

## Try it

## The flow

## When to use it

## When not to use it

## Installing

`install.sh` installs the release binary for Linux or macOS. It detects the local OS and CPU, resolves
the latest GitHub release, downloads the matching `work-leaf-<target>.tar.gz` archive and
`.sha256` checksum file, verifies the checksum, and installs `work-leaf` into `/usr/local/bin`.

Set `WORK_LEAF_INSTALL_DIR` to install into another directory, such as `$HOME/.local/bin`. Set
`WORK_LEAF_INSTALL_VERSION` to install a specific release tag. When the target directory already
contains the requested version, the installer exits without downloading another archive.

The installed binary supports `work-leaf --version` for update checks. Runtime machines need Codex
available on `PATH` for the default agent, or Claude available on `PATH` when launched with
`--agent claude`.

## Running

## Benches

`./start` builds the `work-leaf` binary in release mode and renders the terminal CLI. Set
`WORK_LEAF_START_SKIP_BUILD=1` to reuse an existing binary, and set `WORK_LEAF_START_BIN_DIR` to run
`work-leaf` from a different binary directory. Pass `-d` or `--daemon` to run only the localhost HTTP
API and web UI daemon, or pass `-c` or `--cli` with an API URL to attach the terminal CLI to an
existing daemon.

`./start --bench` lists saved benchmark artifact directories that contain executable Work Leaf
binaries, newest first by the timestamped artifact name, and prompts for the benchmark to run. The
selected artifact's `bin/work-leaf` is executed with any remaining arguments, so the session uses the
binaries saved by that benchmark instead of binaries built from the current checkout. Set
`WORK_LEAF_START_BENCH_RESULTS_DIR` to search a results directory other than `bench-results`.

`./build-target` packages the `work-leaf` binary for the current Rust host target and writes it under
`dist/work-leaf-<target>`. Set `WORK_LEAF_BUILD_TARGETS` to an explicit whitespace-separated target
list when running release automation. When `rustup` is available, the script installs missing Rust
targets before building each package. The release-binaries GitHub Actions workflow uses native
Ubuntu, macOS, and Windows runners for the Linux, Darwin, and MSVC packages.

`./release` prepares a patch release from a clean git worktree. It increments the package version by
`0.0.1`, refreshes `Cargo.lock`, verifies the refreshed lockfile with `cargo check --locked`, runs
`cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`,
`cargo test --all-targets --all-features`, and `./build-target`, then commits the version bump and
creates the matching `v<version>` tag. Pass `--push` to push `HEAD` and the tag to `origin` after the
local release is created.

`./smoke-three-features` builds the current release binaries, creates a temporary checkout at the
three-feature smoke-test base commit, and runs `./start` from that temporary checkout. The script
prints the three `:new` commands used by the real-agent smoke and removes the temporary checkout
when Work Leaf exits, fails, or is interrupted. Set `WORK_LEAF_SMOKE_BASE` to choose a different
base commit, or pass daemon options after `--`.

`./bench-three-features` runs the fixed three-feature workload through the localhost HTTP API with
the real configured Codex backend. It launches the three Work Leaf requests together.
`./bench-three-features-sequential` runs the same frozen requests one after another through normal
direct Codex sessions, without Work Leaf. Both drivers use the fixed base commit, require the passive
benchmark observer, give every implementation or fix cycle exactly one focused Cargo validation,
and run the same final `cargo fmt`, Clippy, and test gate once after linearization. Busy agent silence
and idle orchestrator silence have separate limits through `WORK_LEAF_BENCH_BUSY_STALL_SECS` and
`WORK_LEAF_BENCH_IDLE_STALL_SECS`.

Reports and admitted candidate runtimes are written under `bench-results`. The driver always removes
its temporary checkout. `./materialize-bench-candidate` can reconstruct one verified historical
candidate from a saved report, Git bundle, and patch evidence when no admitted runtime was retained.
`./bench-dashboard` serves saved reports. Its fitted baseline uses only listed GPT-5.5/xhigh rows
that explicitly record the normal concurrent Work Leaf Codex app-server path. Its product summary
compares those rows with explicitly recorded direct sequential Codex rows. Other modes stay visible,
and other model or reasoning profiles remain in a raw-comparison section without fitted judgments.

`work-leaf-orchestrator` owns the controller, agent backend, locks, review routing, and patch
workflow. It prints `WORK_LEAF_ORCHESTRATOR_URL=http://...` after binding its localhost HTTP API.
`work-leaf` connects to that URL through `WORK_LEAF_ORCHESTRATOR_URL`; when the variable is absent,
the CLI starts an embedded localhost controller on an ephemeral port and connects to it.
