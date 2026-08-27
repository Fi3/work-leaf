# Practical follow-up measurement plan

This plan governs a separate data collection. The evidence and conclusions in `FINAL-REPORT.md`
remain unchanged.

The follow-up has two goals:

1. Measure how much of the token difference comes from changed-file diffs, unchanged-file digests,
   direct review context, and everything not explained by those three mechanisms.
2. Improve the estimate of token use and implementation quality for normal sequential Codex versus
   normal concurrent Work Leaf.

Cross-project replication is deferred. This study concerns only the frozen three-feature Rust task.

## The unit of evidence

Each completed candidate is one observation in its assigned condition. The primary analysis compares
all observations in one condition with all observations in another condition.

Do not create one-to-one pairs after collection. With many separately launched runs, any such pairing
would be arbitrary and would not add information. A collection batch is only a practical way to
spread the conditions across time and detect operational failures early. It is not the statistical
unit, and a run is not discarded merely because another condition is missing from the same batch.

Record launch time and collection batch so time-related or machine-related effects can be checked
separately. Those checks are secondary to the randomized condition-group comparison.

## Fixed comparison

Every accepted run uses:

- Candidate base commit `c92a0b7060a36eac6db2d869b85e589a7a9480f9`. Commit `3fdf54f` is a
  documentation commit and is not the candidate base.
- The existing three-feature benchmark task and frozen original-task scorer.
- GPT-5.5 with `xhigh` reasoning.
- The same current Codex CLI version across conditions in this collection. Record its version; it
  does not have to match the historical CLI version.
- A fresh checkout, conversation, observer identity, and output directory.

The product comparison uses the existing benchmark launchers unchanged:

- `direct`: `bench-three-features-sequential`, which runs normal direct Codex and handles the three
  requested features sequentially without Work Leaf.
- `wl-000`: `bench-three-features`, which runs normal Work Leaf and handles its feature agents
  concurrently.

Do not schedule Work Leaf features sequentially. Do not modify Work Leaf, either workflow's prompt,
the task, the scorer, the validation budget, or the evaluator to improve outcomes. The benchmark
must measure the products as they actually behave.

The finalized study under `bench-results/efficiency-causal-study` is immutable evidence. The rejected
planning attempt formerly stored at
`bench-results/efficiency-token-allocation-follow-up-20260826T144239Z` launched no provider workflow
and is absent from the worktree. Do not recreate or use its 16,400-run calculation, generated
controller, statistical assumptions, or storage plan.

## Mechanism conditions

The Work Leaf condition name has three bits in this order:

1. Changed-file reread delivery.
2. Unchanged-file reread delivery.
3. Review-context delivery.

`0` is normal Work Leaf behavior. `1` replaces that behavior with its less compact control.

| Condition | Changed-file reread | Unchanged-file reread | Review context |
| --- | --- | --- | --- |
| `wl-000` | diff | digest | supplied directly |
| `wl-001` | diff | digest | reconstructed from Git |
| `wl-010` | diff | full file | supplied directly |
| `wl-011` | diff | full file | reconstructed from Git |
| `wl-100` | full file | digest | supplied directly |
| `wl-101` | full file | digest | reconstructed from Git |
| `wl-110` | full file | full file | supplied directly |
| `wl-111` | full file | full file | reconstructed from Git |

`direct` is the ninth condition. Work Leaf factor switches do not apply to it.

## Collection stages

### 1. Run the first retained production batch

Launch one run in each of the nine conditions in a randomized order. These are real study
observations and remain in the final condition groups; they are not disposable preflight or pilot
runs. At most two top-level workflows may run at once. Each workflow must have its own checkout,
build directory, observer identity, and result directory. Work Leaf keeps its normal internal
concurrency.

Use the existing launchers, frozen factor controls, frozen binaries, and frozen observer directly.
Do not run a separate launcher audit or preflight, build a controller, replay historical candidates,
run a power simulation, create storage infrastructure, or execute candidate fixtures before this
batch.

### 2. Inspect before spending more

After the first batch, inspect every factor setting, token counter, candidate artifact, scorer result,
failure, and missing field. Preserve failed attempts and explain their causes in plain language. Stop
and report if failures share a systematic cause, if exact counters cannot be recovered, or if running
two workflows at once creates resource contention.

Do not fix an apparent problem by changing Work Leaf, the task, the prompts, or the scorer. Do not
rerun a model merely because its implementation is incomplete or scores poorly.

### 3. Set the larger sample from observed data

The first batch does not set or prove the final sample size. Combine its observations with the saved
historical variation only to propose practical options for additional independent runs per condition.
For each option, state the expected uncertainty, elapsed time, and provider use. Obtain the study
owner's approval for one explicit option before launching a larger collection. Do not infer approval
from an example number mentioned in conversation, and do not derive a huge run count from the
rejected formal plan.

Run an approved expansion in small randomized operational batches, with at most two top-level
workflows at once. Inspect each batch before starting the next. The batches are not statistical
blocks and do not create pairs.

Once the mechanism estimates are reasonably stable, additional evidence for the normal product
comparison needs only `direct` and `wl-000`. The other seven factor conditions do not need to be run
again solely to increase confidence in the normal-product comparison.

## Admission and retries

Assign every scheduled launch a unique condition, collection-batch, and attempt identifier before it
starts. Preserve every attempt.

A retry is allowed only when no task reached a provider thread because the executable did not start,
the checkout failed its fixed-identity check, the observer was not ready, or the provider rejected
the request before creating a thread. Give the retry a new identifier and link it to the original.

Once a provider receives the task, the observation remains evidence even if it is interrupted,
implements nothing, fails validation, or scores zero. Do not retry or remove it because of its result,
token use, or quality. Missing token counters remain recorded as missing; they do not erase the
quality result.

## Fair workflow treatment

Both normal paths receive the same task information, implementation-turn allowance, focused Cargo
validation allowance, timeout policy, and final checks. Each implementation or correction turn may
run one focused Cargo validation. The final gate is the same for both paths:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Do not give either path an extra hint, manual correction, validation retry, or hidden test. Workflow
PASS and feature quality remain separate results.

## Quality and token capture

Score every admitted candidate on the three original features:

- Vim-like visual selection and copying in both panes.
- Selected-agent `/status` execution as a backend command, with its response shown in that chat.
- Reviewed-patch close and reopen behavior.

`/fork` is not part of this task's score. It may be recorded separately but cannot affect admission,
the slash-command result, or the comparison.

Use exact provider counters. Raw tokens are input plus output tokens. Uncached tokens are input minus
cached input plus output. Count every provider call caused by the workflow through its frozen final
report. Keep implementation, orchestration, fix, review, and history-cleanup calls in scope. Apply
the same title-agent rule to every normal-path run.

Offline scoring and analysis are outside the token count. Blind the scorer to condition names and
token totals. A valid empty or partial artifact receives the score it earned; it is not discarded.

## Analysis

### Normal product comparison

Use all admitted `direct` observations as one group and all admitted `wl-000` observations as the
other group. For each group report:

- Mean and median raw tokens and uncached tokens.
- The difference and percentage difference between group means.
- Pass rate for each of the three features.
- Mean total feature score out of three.
- Confidence intervals and the full result distribution.

Do not pair runs after collection. Launch-round comparisons may be shown only as a secondary check
for time or machine effects. Retain unequal-quality outcomes and show tokens together with quality;
an incomplete implementation is useful evidence, not a reason to discard a run.

The practical study reports the uncertainty supported by the collected sample. It does not claim
formal `+/-5` percentage-point quality equivalence unless the resulting confidence intervals truly
support that claim. Failure to prove equivalence does not prove that the workflows differ.

### Token allocation

For raw and uncached tokens separately, calculate the mean for each of the eight Work Leaf condition
groups and for `direct`. Apply the average-over-orders allocation to those condition means:

1. The whole gap is mean `direct` minus mean `wl-000`.
2. Each mechanism receives its average marginal change across every order in which the three normal
   mechanisms can be enabled.
3. Those three amounts include their shared interactions and sum to mean `wl-111` minus mean
   `wl-000`.
4. The residual is mean `direct` minus mean `wl-111`.

The three mechanism amounts plus the residual must reproduce the whole gap. Report negative amounts
as negative. Do not force every mechanism to save tokens.

Estimate uncertainty by resampling complete observations within each condition group. The primary
calculation does not require equal sample counts or complete nine-run blocks. Also show a secondary
check by launch period to reveal drift, without constructing arbitrary pairs.

Report absolute token amounts before percentages. Keep raw and uncached results separate. Do not add
the earlier treated-turn percentages or reviewer-thread percentages; their scopes differ from a
whole workflow.

## Records and completion

Keep raw provider streams, candidate artifacts, manifests, commands, environment values, versions,
timestamps, factor settings, counters, scorer outputs, failures, and hashes in the external archive.
Keep a compact human-readable report, normalized measurements, analysis code, tests, and archive
hashes in a new study directory.

The follow-up is complete when it provides:

- A reproducible group-level allocation of the whole raw and uncached token gaps, with uncertainty.
- A group-level normal sequential versus concurrent Work Leaf comparison for tokens and all three
  feature outcomes, with uncertainty.
- A plain-language account of failures, missing data, sample size, runtime, cost, and remaining
  limits.

Historical candidates remain a sanity check only. Do not mix them into the new randomized groups.
Other repositories, models, and reasoning settings remain future studies.
