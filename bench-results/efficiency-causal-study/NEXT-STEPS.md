# Next measurement protocol

This specification governs a separate data collection. It does not report a new result. The compact
evidence does not establish exact token-gap allocation or formal quality equivalence.

This protocol has two distinct goals:

1. **Exact token-gap allocation.** Allocate the measured gap between normal direct Codex and normal
   Work Leaf to three Work Leaf mechanisms and a residual. A residual is the part not assigned to
   those mechanisms.
2. **Formal quality equivalence.** Test whether the two normal paths have practically equivalent
   quality for every requested feature.

Evidence for one goal is not evidence for the other. Token allocation without quality equivalence
is not an equal-quality efficiency result. Quality equivalence does not explain a token gap.

This protocol-writing step includes no paid run. It includes no model, provider, benchmark,
candidate, or candidate-binary run. Collection requires separate funding and authorization.

## Required work

### Freeze the comparison before collection

Create a pre-registration record before the first launch. A pre-registration is a plan that is
locked before outcomes are known. Give it a content hash and make it read-only. A content hash is a
short fingerprint that changes when the record changes.

The record must freeze all of the following:

- The exact task bytes and their hashes.
- The repository and exact starting commit.
- The frozen scorer, its fixtures, its tests, and their hashes.
- GPT-5.5 with `xhigh` reasoning.
- The exact direct Codex command and the exact Work Leaf command.
- Every executable path, executable hash, argument, argument order, working directory, input byte,
  environment setting, timeout, and version used by those commands.
- The Work Leaf build, direct Codex build, observer build, and factor settings.
- The number of blocks, randomization method, random seed, retry rule, admission rule, and stopping
  rule.
- The statistical code and confidence-interval method.
- The token scope and the rules for artifact and observer acceptance.

Store commands as literal argument arrays. Do not rely on aliases or an interactive shell profile.
Store standard input separately as exact bytes. A changed command, task, scorer, model, or reasoning
effort is a protocol deviation. Keep the run, but do not silently treat it as the frozen condition.

The accepted comparison profile is GPT-5.5 with `xhigh` reasoning. Results from any other model or
reasoning profile are display-only. They cannot enter the accepted comparison. A different profile
requires a separately funded and predeclared replication.

The path comparison is:

- Normal direct Codex, with the three feature tasks handled sequentially.
- Normal Work Leaf, with its agents handled concurrently.

Both paths use their normal product behavior. A sequentially scheduled Work Leaf condition is
outside this design.

### Use randomized complete blocks

A block is one set of closely matched runs. Every run in a block uses the same frozen task, starting
commit, profile, command versions, observer version, hardware class, and collection window. Each run
starts from a fresh checkout. Runs do not share a conversation or working tree.

Each block contains the direct condition and all eight Work Leaf factor cells. The direct condition
and `wl-000` form the normal-path pair. Pairing means that their difference is calculated within the
same block before differences are combined across blocks.

Run one condition at a time to avoid machine contention. Work Leaf still uses its normal internal
concurrency. Before collection, use a reproducible computer shuffle to create a different order of
the nine conditions for each block. Freeze the shuffle algorithm, seed, and full order list in the
pre-registration. Do not alter an order after seeing a failure, score, or token count.

### Run the complete mechanism design

A factor is one mechanism that can be set to normal behavior or a control behavior. A cell is one
particular combination of factor settings. A complete `2^3` factorial contains all eight
combinations of three two-setting factors.

The cell name uses three bits in this order:

1. Changed reread delivery.
2. Unchanged reread delivery.
3. Review provenance delivery.

For each bit, `0` means normal Work Leaf behavior. A changed reread returns a diff. An unchanged
reread returns a digest. A review receives exact provenance inline. A digest is a short stable
summary of unchanged content. Provenance is the exact source and commit information needed for a
review.

For each bit, `1` means the declared control. A changed reread returns the full current content. An
unchanged reread resends the full content. A reviewer reconstructs provenance from Git.

| Cell | Changed reread | Unchanged reread | Review provenance |
| --- | --- | --- | --- |
| `wl-000` | diff | digest | inline exact |
| `wl-001` | diff | digest | Git reconstruction |
| `wl-010` | diff | full resend | inline exact |
| `wl-011` | diff | full resend | Git reconstruction |
| `wl-100` | full current content | digest | inline exact |
| `wl-101` | full current content | digest | Git reconstruction |
| `wl-110` | full current content | full resend | inline exact |
| `wl-111` | full current content | full resend | Git reconstruction |

`direct` is the ninth cell in each block. It is normal sequential direct Codex. The Work Leaf factor
switches do not apply to it.

Use exact provider counters for every cell. Raw tokens equal input tokens plus output tokens. Cached
input remains in raw tokens. Uncached tokens equal input tokens minus cached input tokens plus
output tokens. Count every provider call caused by the condition. This includes implementation,
fixes, reviews, orchestration, and naming calls. Stop the scope at the frozen terminal report.
Offline scoring and analysis are outside the token scope.

Allocate raw and uncached tokens separately. For each metric and each complete block:

1. Calculate the whole gap as `direct` minus `wl-000`.
2. For each mechanism, calculate the saving from changing its control setting to its normal setting
   in every possible order in which the three mechanisms can be enabled. Average those savings.
3. Split interaction effects through that average. An interaction exists when the effect of one
   mechanism depends on another mechanism's setting.
4. Calculate the residual as `direct` minus `wl-111`.

This average-over-orders allocation makes the three mechanism amounts sum exactly to `wl-111`
minus `wl-000`. Adding the residual gives `direct` minus `wl-000`. Report negative amounts as
negative. Do not force every mechanism to appear beneficial.

The compact evidence has exact data for six of the nine required cells: `direct`, `wl-000`,
`wl-001`, `wl-010`, `wl-100`, and `wl-110`. Exact data are unavailable for `wl-011`, `wl-101`, and
`wl-111`. The missing combinations prevent all average-over-orders calculations. The missing
`wl-111` cell also prevents calculation of the residual. The current 6/9 cells therefore cannot
support exact allocation.

Do not fill a missing cell with a run from another block. That would mix collection windows and
observer identities. It would also break the within-block comparison.

The complete factorial is required unless the pre-registration gives a statistically valid
alternative. An alternative must be justified before collection. It must show which effects and
interactions are identifiable, state every assumption, include its own power simulation, and still
identify the residual against `direct`. If it cannot uniquely allocate the whole gap, label its
result as partial. Do not call it exact allocation.

### Freeze admission and retry rules

Assign every scheduled launch a unique block, condition, and attempt identifier. Save a manifest
entry before invoking its command.

A task observation is admitted when the frozen task reaches the first provider thread. Before that
point, allow at most one retry for one of these recorded reasons:

- The executable did not start.
- The checkout or frozen-hash preflight failed.
- The observer was not ready.
- The provider rejected the request before creating a thread.

No other reason permits a retry. In particular, do not retry because of a low score, high token use,
model refusal, test failure, partial implementation, workflow failure, interruption after a thread
starts, or a zero-feature result.

Give a retry its own attempt identifier. Record the original identifier in a `retry_of` field. Keep
the original launch and the retry. Never overwrite either record. Put a permitted retry in a frozen
retry slot after the nine scheduled positions in its block. Do not reorder the remaining conditions.

Retain every admitted success, partial result, failure, interrupted run, and zero-feature result.
Retain every pre-admission launch failure as operational evidence. Decide protocol exclusions only
from frozen integrity fields. These fields include the task hash, starting commit, model, reasoning
effort, command hash, factor setting, and observer readiness. Make the decision while scores and
token totals are hidden. Keep every excluded record with its reason.

Do not filter on workflow PASS. Do not filter on scorer results. Do not discard a run because exact
token counters are missing. Mark its token outcome as missing. A block without all nine exact token
outcomes cannot enter the exact within-block allocation.

### Give both paths the same validation budget

An implementation or fix turn is one agent response devoted to implementing or correcting one
feature. Permit exactly one focused Cargo validation in each such turn. Freeze the command-selection
rule, timeout, and output limit. Apply the same rule to direct Codex and Work Leaf.

After implementation and fixes, run the same final gate for both paths, in this order:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Do not give either path an extra manual check, correction, hint, or validation retry. This is fair
because both paths receive the same task feedback and the same chance to find errors. The design
then measures orchestration behavior instead of a larger test budget.

### Freeze quality scoring and capture acceptance

Score each requested feature separately:

- Vim-like visual selection and copying in both panes.
- Strict selected-agent slash-command execution, including `/status` as a backend command rather
  than an ordinary agent prompt.
- The reviewed-patch yes/no completion state, including close and reopen behavior.

Use the frozen scorer on every admitted artifact. Do not replace separate feature results with one
workflow PASS flag. Keep the total feature score as a secondary summary only.

`/fork` is supplemental evidence for original-task quality. It does not determine the primary slash
command feature result. It does not determine admission or equivalence.

An artifact is the saved candidate output used for scoring. It includes the frozen base identity,
the final commits or patch state, repository status, and validation logs. Define acceptance before
collection. Accept an artifact when its hashes match and it can be reconstructed from the frozen
base. A certified empty artifact is valid and scores zero for all features. A partial artifact is
valid and receives the feature results produced by the frozen scorer. A corrupt or unrecorded
artifact remains in the manifest and has a missing quality measurement.

An observer is the independent capture process that records provider threads and usage. Define its
acceptance before collection. Exact token acceptance requires a one-to-one mapping from the run to
every provider thread that it caused. Every thread must have complete input, cached-input, and
output counters through completion or interruption. The counters must reconcile with the provider
totals.
The observer may certify zero usage only when it also certifies that no provider thread started.
Missing or conflicting counters fail exact token acceptance. They do not remove the run.

Test the frozen scorer and observer acceptance rules on synthetic fixtures before any paid launch.
Blind scoring to the condition label and token totals.

### Predeclare the statistical decision

The primary token outcomes are the complete-scope raw and uncached gaps and their four allocation
amounts: changed rereads, unchanged rereads, review provenance, and the residual. Analyze raw and
uncached tokens as separate outcome families.

The primary quality outcomes are the three separate feature pass indicators. For each feature,
estimate the paired difference in pass probability between `wl-000` and `direct`.

Before collection, give each feature a numeric equivalence margin in percentage points. An
equivalence margin is the largest quality difference that the task owner considers practically
unimportant. Justify each margin from product requirements. Do not choose it from observed study
scores.

Run a sample-size and power simulation before collection. Power is the chance that the planned test
reaches the correct decision under stated assumptions. The simulation must use the frozen analysis
code and a fixed seed. It must represent paired block effects, correlation among the three feature
results, repeated use of the same task, interrupted runs, missing measurements, and linked retries.
It must also represent the planned random order and stopping rule.

Choose the number of blocks to provide at least 90% simulated power for the joint quality
equivalence decision when the paths are truly equal at the assumed rates. Also set a target maximum
confidence-interval width for each token allocation. Increase the planned block count until the
simulation meets both gates. Freeze the simulation inputs, code, output, block count, and any
reserve blocks before collection.

A confidence interval is a range that shows uncertainty around an estimate. Report confidence
intervals for every primary outcome. Use block-level resampling. Resampling repeatedly draws whole
blocks to measure how estimates vary. Keep each block's nine conditions, three feature results,
attempt families, and repeated observations together. This preserves their dependence. Do not count
feature checks or retries as independent runs.

Use simultaneous intervals with at least 95% joint coverage within each primary family. Apply the
frozen Bonferroni rule to the three quality intervals and separately to each token family. This rule
uses a smaller error allowance for each interval so that the full set keeps the stated coverage.

Claim formal quality equivalence only when every simultaneous quality interval lies wholly inside
its predeclared negative and positive margin. A failure to meet this rule means equivalence is not
established. It does not by itself prove that the paths differ.

Use a fixed stopping rule. Run exactly the predeclared blocks, reserve blocks, and permitted
pre-admission retries. Do not stop early because results look favorable. Do not add blocks after
viewing results. A safety, budget, or provider outage may stop collection early. Report that case as
incomplete.

Report absolute token amounts before percentages. Keep raw-token percentages separate from
uncached-token percentages. Keep whole-workflow results separate from treated-turn and
reviewer-thread results. Do not add percentages from different scopes. Do not present isolated
mechanism percentages as shares of the whole gap. Only the frozen complete-block allocation may
produce whole-gap shares.

### Preserve an auditable record

Keep full raw evidence outside rewritten master in an immutable archive. Keep the authoritative
machine-readable block, launch, attempt, retry, environment, artifact, observer, and analysis
manifests there as well. Include provider event streams, exact counters, command output, task bytes,
artifacts, and integrity hashes. Do not store credentials.

Keep a compact audit package in the repository. It should contain the frozen protocol, scorer,
analysis code, tests, normalized measurements, small decisive logs, archive inventory hashes, and
provenance pointers. Do not commit candidate binaries, raw provider streams, caches, temporary
checkouts, or large artifacts. Verify every external file used by the compact package with a
recorded cryptographic hash.

### Evidence required for each goal

Exact token-gap allocation is available only when the predeclared design is complete. Every used
block must have exact counters for `direct` and all eight Work Leaf cells. The frozen allocation
must produce the three mechanism amounts, the residual, the whole gap, and their confidence
intervals for raw and uncached tokens. A predeclared valid alternative must meet the same
identification standard.

Formal quality equivalence is available only when the numerical margins, power simulation, sample
size, scoring rules, stopping rule, and dependence-aware analysis were frozen first. Every admitted
normal-path artifact must enter the frozen scorer. All three simultaneous confidence intervals must
fall inside their margins.

Neither body of evidence is available until this protocol is funded, frozen, and run. The compact
study does not claim either result.

## Optional replication

Cross-project generalization requires replication on separately frozen projects. Each project needs
its own task bytes, starting commit, scorer contract, commands, block schedule, power simulation,
and archive. Predeclare any pooled analysis before those runs. Without that replication, conclusions
apply only to the frozen project and task.

A model or reasoning profile other than GPT-5.5 with `xhigh` reasoning is also a separate
replication. Fund it separately and predeclare it before collection. Otherwise, show its rows only
as display-only context.

An independent observer implementation or a second hardware environment can provide an optional
robustness check. Freeze it as a separate replication. Do not mix its cells into required blocks.
