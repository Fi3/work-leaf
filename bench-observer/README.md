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
the real Codex, shell, and Cargo executables. It places `codex` and `sh` proxy links under
`observation/proxy-bin`. The driver puts that directory first in `PATH` and exports the generated
`WORK_LEAF_OBSERVER_CONFIG`.

The Codex proxy records noninteractive `app-server` and JSON `exec` invocations. It forwards output
and signals and returns the real process status. Version, help, and local schema-generation commands
pass through with informational metadata. Other unclassified pass-through commands make a capture
ineligible.

The shell proxy records only the locked-command shape emitted by
`src/orchestrator.rs::run_shell_command`. Other shell processes run through the configured real
shell. Cargo runs directly; its recorded identity and observed command events are descriptive
evidence rather than an enforcement mechanism.

`stop-app-server` stops only the real child of the primary observed Work Leaf app server. It waits
for the proxy to write completion metadata before the driver shuts down the product daemon.

## Validation recording

The observer records Cargo command activity that appears in captured agent and shell events. It does
not place a Cargo executable in the proxy directory, impose a per-turn command budget, reject broad
commands, or change a workflow result because of its validation pattern. This preserves each
workflow's normal behavior: Work Leaf agents follow the orchestrator's regular implementation,
review, and linearization prompts, while direct Codex sessions validate as needed and leave broad
cross-feature checks to their final linearizer.

After linearization, both drivers independently verify the saved implementation with
`bench-validation-common::bench_run_final_gate`. That non-mutating gate runs:

```sh
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

The gate stops at the first failed command and returns that failure even when its caller tests the
function's status in a shell condition. It records whether the result meets the repository checks;
it does not repair the implementation or replace the agents' own validation.

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

An interrupted app-server response without terminal usage is not made complete merely because the
thread later reports another cumulative total. The observer subtracts the previous cumulative total
and the later event's `last` usage. It accepts the interrupted response only when a nonzero remainder
is attributable to exactly one unresolved interruption in that interval. Otherwise `analyze` keeps
the recorded total as a lower bound and marks provider usage incomplete.

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
mismatch, or a credential marker rejects the capture. A capture failure marks token measurement as
unusable but does not rewrite the workflow's implementation result. Raw prompts and streams use
user-only permissions. Credential scanning runs before artifacts are admitted.

The dashboard fits only the accepted GPT-5.5/xhigh profile. Reports from another model or reasoning
profile remain visible in a separate raw-comparison section and cannot train or be compared with the
accepted fit.
