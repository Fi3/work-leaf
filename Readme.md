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
`0.0.1`, refreshes `Cargo.lock`, runs `cargo fmt`, `cargo clippy --all-targets --all-features -- -D
warnings`, `cargo test --all-targets --all-features`, and `./build-target`, then commits the version
bump and creates the matching `v<version>` tag. Pass `--push` to push `HEAD` and the tag to `origin`
after the local release is created.

`./smoke-three-features` builds the current release binaries, creates a temporary checkout at the
three-feature smoke-test base commit, and runs `./start` from that temporary checkout. The script
prints the three `:new` commands used by the real-agent smoke and removes the temporary checkout
when Work Leaf exits, fails, or is interrupted. Set `WORK_LEAF_SMOKE_BASE` to choose a different
base commit, or pass daemon options after `--`.

`./bench-three-features` runs the same three-feature scenario through the localhost HTTP API with
the real configured Codex backend. It creates a temporary clean Git checkout from the current
working directory, commits that snapshot as the benchmark base, uses the default mediated-read
workflow, records pass/fail, duration, review and linearize completion, commit churn, code-quality
checks, the Codex CLI path and version, resolved model, reasoning effort, and observed inefficiencies
under `bench-results`, and enables Codex child-process trace output in the saved daemon artifacts.
Busy agent silence and idle orchestrator silence have separate stall limits through
`WORK_LEAF_BENCH_BUSY_STALL_SECS` and `WORK_LEAF_BENCH_IDLE_STALL_SECS`.
Untracked benchmark output under `bench-results` remains saved output rather than source input for
the temporary checkout. The script always removes its temporary checkout. `./bench-dashboard` serves
saved benchmark reports from `bench-results`, including nested parallel-run result directories. When
`bench-results/baseline-manifest.json` is present, reports listed there form the regression baseline;
other discovered reports are shown separately as old or invalid data.

`work-leaf-orchestrator` owns the controller, agent backend, locks, review routing, and patch
workflow. It prints `WORK_LEAF_ORCHESTRATOR_URL=http://...` after binding its localhost HTTP API.
`work-leaf` connects to that URL through `WORK_LEAF_ORCHESTRATOR_URL`; when the variable is absent,
the CLI starts an embedded localhost controller on an ephemeral port and connects to it.
