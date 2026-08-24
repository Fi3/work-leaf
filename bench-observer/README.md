# Work Leaf Benchmark Observer

The benchmark observer records the process traffic and token use of a benchmark run. It is
measurement tooling. It is not linked into the `work-leaf` crate or either product binary.

## Build and driver use

```sh
cargo build --release --manifest-path bench-observer/Cargo.toml --bin bench-observer
```

The normal benchmark drivers require the observer for every non-dry run. If
`WORK_LEAF_BENCH_OBSERVER_BIN` is unset, each driver builds and uses
`bench-observer/target/release/bench-observer`. An explicit path must name an executable observer.

`bench-three-features` measures three Work Leaf features launched together.
`bench-three-features-sequential` invokes `bench-three-features-direct-common` and measures the
same three requests handled one after another by direct Codex sessions. Both use the same fixed
base commit, task text, observer, validation rules, and final repository gate.

## Process capture

`bench-observer init` creates a run-owned artifact directory and writes immutable identities for
the real Codex, shell, and Cargo executables. It places `codex`, `sh`, and `cargo` proxy links under
`observation/proxy-bin`. The driver puts that directory first in `PATH` and exports the generated
`WORK_LEAF_OBSERVER_CONFIG`.

The Codex proxy records noninteractive `app-server` and JSON `exec` invocations. It forwards output
and signals and returns the real process status. Version, help, and local schema-generation commands
pass through with informational metadata. Other unclassified pass-through commands make a capture
ineligible.

The shell proxy records only the locked-command shape emitted by
`src/orchestrator.rs::run_shell_command`. Other shell processes run through the configured real
shell. The Cargo proxy always executes the immutable real Cargo path recorded by `init`.

`stop-app-server` stops only the real child of the primary observed Work Leaf app server. It waits
for the proxy to write completion metadata before the driver shuts down the product daemon.

## Fair validation

Each implementation or review-fix cycle must run exactly one focused Cargo validation command.
A command is focused only when it names a package with `-p` or `--package`, names one Cargo target
with `--bin`, `--test`, `--bench`, or `--example`, or supplies a named test filter. Target-kind
switches such as `--lib` and `--doc`, and package-set switches such as `--workspace` and `--all`, do
not provide focus by themselves.
The test filter may appear before or after `--`; runner flags such as `--nocapture` and empty target
or filter values do not provide scope. Validation discovery recognizes direct shell command
segments, leading environment assignments, and `env` or `command` wrappers. Redirection targets,
including file-descriptor duplication, and shell comments are ignored before commands and filters
are counted. Dynamic `eval` commands and command substitution are ineligible because the audit
cannot prove that they start exactly one validation process. `env` split-string execution and
heredocs are ineligible for the same reason.
Validation discovery follows at most four nested `sh -c` payloads.
`cargo nextest` must use its `run` action; `--filter-expr` can provide named-test focus. A focused
`cargo fmt` must also use
`--check`. Broad switches such as `--all-targets` and `--all-features` do not provide focus by
themselves. For example, `cargo test --all-targets --all-features done` is focused by the `done`
filter, while plain `cargo fmt`, `cargo clippy`, `cargo check`, `cargo build`, `cargo test`, and
`cargo test -- --nocapture` are not. Discovery skips Cargo's global options before the subcommand
and recognizes the built-in `b`, `c`, `d`, and `t` aliases for build, check, doc, and test. Cargo
does not define built-in short aliases for Clippy or fmt, so none are assumed.

The Cargo proxy rejects a broad first command before the real Cargo process starts. It also blocks a
second validation inside one captured process allowance. For direct Codex,
`bench-audit-agent-validation` reads each implementation or fix JSONL log and requires exactly one
focused command. For Work Leaf, the observer groups the initial request and each reviewer-requested
fix into validation cycles. Analysis requires exactly one focused command across each cycle, even
when its reads, edits, and commands span several provider turns.

Both drivers disable the implementation budget before linearization. Linearizers rewrite the
reviewed commits without running Cargo. After linearization, both drivers call
`bench-validation-common::bench_run_final_gate` exactly once. That gate runs:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

The gate stops at the first failed command and returns that failure even when its caller tests the
function's status in a shell condition.

This keeps the comparison plain: concurrent Work Leaf and direct sequential Codex receive the same
per-cycle validation allowance and the same final gate.

## Accounting and analysis

`bench-observer timeline` records ordered workflow events. `git-checkpoint` records repository HEAD,
status, diffs, commit graph, and reflog at named boundaries. `archive-bundles` saves regular context
bundle files with byte lengths and SHA-256 digests.

`analyze` reads captured provider messages, command results, process lineage, checkpoints, and token
snapshots. It reports separate token scopes for visible agent roles, the primary benchmark path, and
the total workflow. Dashboard comparisons use the total-workflow scope only when capture completion,
run identity, and token arithmetic all validate.

Codex reports cumulative usage for a thread. The observer counts a captured generation only when the
same transport contains a matching turn start. Resume or fork metadata without a generated turn does
not create token usage. Direct-driver accounting also deduplicates cumulative totals by thread.

`extract-rollouts` starts from captured thread IDs. It saves only thread identity, working directory,
model, reasoning effort, CLI version, final cumulative usage, source digest, relative source path,
and scope labels. It does not copy prompts, messages, reasoning text, authentication state, or
unrelated sessions.

## Artifact checks

Raw streams and invocation start/end records are the capture authority. Message indexes, usage
tables, and summaries are reproducible outputs. JSON-looking text inside command output is not
treated as a provider event.

`capture-audit.txt` records completeness. Missing process endings, malformed framing, token usage
without a thread ID, rollout mismatch, an unobserved same-directory thread, model or reasoning
mismatch, a validation violation, or a credential marker rejects the capture. Raw prompts and
streams use user-only permissions. Credential scanning runs before artifacts are admitted.

The dashboard fits only the accepted GPT-5.5/xhigh profile. Reports from another model or reasoning
profile remain visible in a separate raw-comparison section and cannot train or be compared with the
accepted fit.
