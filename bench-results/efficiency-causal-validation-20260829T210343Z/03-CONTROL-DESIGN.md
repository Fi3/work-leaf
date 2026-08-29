# Direct-Read Control Design

## Question

The endpoint audit shows lower token use in normal concurrent Work Leaf, and the decomposition shows
where the measured difference appears: Work Leaf accumulates less input across fewer provider usage
changes. Those observations do not identify which Work Leaf behavior causes the difference.

This control asks one narrower question:

> Does orchestrator-mediated file reading reduce model input compared with allowing the same Work
> Leaf agents to read project files directly?

The control is useful because it changes the route used to obtain repository text while retaining
the normal concurrent Work Leaf workflow. It does not attempt to imitate direct sequential Codex.

## Production Call Path

The benchmark launcher sets `WORK_LEAF_BENCH_NO_READ_PERMISSION=1`. The environment-variable name is
historical and easy to misread: value `1` means **direct agent reads are enabled**.

The setting follows this existing path:

1. `bench-three-features` appends `--no-read-permission` to the daemon arguments.
2. `src/cli.rs::parse_process_args` maps that option to
   `src/agent.rs::ReadPermission::DirectFilesystem`.
3. `src/cli.rs::selected_agent_backend` constructs the selected backend, and
   `src/cli.rs::codex_backend` builds its prompt with
   `PromptPolicy::for_project_with_read_permission`.
4. `src/agent.rs::PromptPolicy::for_read_permission` tells patch and review agents to use direct,
   read-only filesystem inspection instead of `@work-leaf read`.
5. The same policy still requires structured edits and orchestrator-mediated write-producing
   commands.

The linearization agent already receives direct workspace access in both conditions through
`src/agent.rs::PromptPolicy::inject`, so this control primarily changes patch-agent and reviewer
reads. It does not turn Work Leaf into the direct sequential workflow.

The relevant files in the active checkout are byte-for-byte unchanged from frozen source commit
`5b1d1ef9590850faed26052f909ddff7ff8f127d`, whose binaries produced the six detailed normal Work
Leaf reference runs.

## Fixed Conditions

| Property | Normal Work Leaf reference | Direct-read control |
| --- | --- | --- |
| Base and three-feature task | Frozen benchmark task | Same |
| Workflow | Concurrent Work Leaf | Same |
| Feature schedule | Normal concurrent submission | Same |
| Model | GPT-5.5 | Same |
| Reasoning effort | `xhigh` | Same |
| Agent validation freedom | Repository instructions | Same |
| Write path | Structured edits and locked commands | Same |
| Review and linearization | Normal Work Leaf | Same |
| Final checks and scorer | Frozen benchmark checks | Same |
| Recursive provider sessions | Disabled | Same |
| Provider-usage grace | 1,000 ms | Same |
| Timeout | 7,200 seconds | Same |
| Read route for patch/review agents | `@work-leaf read` | Direct read-only tools |

Run IDs, output paths, runtime directories, and operator notes necessarily differ. They do not alter
the task or model workflow.

## Why This Is the First Control

Three independent observations point to file-context delivery:

- Cached input accounts for 98.58% of the detailed raw-token gap.
- Work Leaf has both fewer provider usage changes and less input context per change in the current
  six-versus-six cohort and in an older equal-quality cohort.
- Normal Work Leaf recorded compact unchanged rereads, changed-file diffs, and context bundles. The
  observed byte counters show that these mechanisms activate, while command-result compaction
  recorded zero avoided bytes in the six normal runs.

This does not prove the hypothesis. The same pattern could come from ordinary run variation,
different feature outcomes, interruption after directives, or a general change in agent behavior
caused by the direct-read prompt. The control therefore measures the complete read-route effect; it
cannot assign that effect among digest replies, diffs, bundles, and any resulting change in agent
planning.

## Checks Against Alternative Explanations

### Missing or incorrect token accounting

Both conditions use provider rollout totals. Each admitted result must have a terminal cumulative
total for every provider thread, matching rollout hashes, and zero unresolved usage. A control with
incomplete accounting is retained but does not enter an exact token mean.

### Different task or validation burden

The launcher reuses the frozen source checkout, benchmark script, binaries, model profile, timeout,
and final checks from the exact normal Work Leaf study. It removes old experiment variables and does
not set a validation budget. The only workflow switch is direct read permission.

### Different implementation quality

Every feature result is retained. Token movement is interpreted causally only alongside the frozen
three-feature score. Lower tokens caused by implementing less work do not count as an efficiency
gain.

### Ordinary model variation

Earlier one-run read-delivery experiments were non-monotonic: forcing full changed rereads used
13.95 million raw tokens, forcing full unchanged rereads used 19.08 million, and forcing both used
11.05 million. Their quality was respectively 2/3, 3/3, and 2/3. Those isolated observations cannot
establish an effect. This study runs three new independent controls and compares the group with all
six current normal Work Leaf observations. It reports ranges and individual rows, not only means.

### Immediate interruption after a directive

All normal Work Leaf runs interrupted terminal directives, so interruption remains a possible cause
of fewer provider cycles. It is not changed in this control. If direct reads do not account for the
gap, interruption remains a candidate for a later, separately audited control.

### Review or linearization compaction

Implementation and review-fix work contains 76.48% of the measured raw gap. Review and
linearization are too small to explain the endpoint result alone. They remain fixed here.

## Activation Gates

Each control result must satisfy all of these before causal interpretation:

1. Its saved report records direct agent file reads.
2. Agent launch prompts permit direct reads and do not contain the mediated-read restriction.
3. Direct read-only inspection appears in the provider trace, while `@work-leaf read` directives
   disappear or fall to an explained exceptional case.
4. Model strata contain only GPT-5.5 with `xhigh` reasoning.
5. Every observed provider thread is inventoried and exact token accounting reconciles.
6. The frozen scorer can evaluate all three requested features, including partial and failed work.

Failure of an activation gate is evidence about the control, not permission to discard the run or
silently retry it.

## Analysis

Three control workflows run concurrently with unique runtime and result directories. Concurrency is
only a collection optimization; the observations are not paired with any normal run.

The control group is compared with the six current normal Work Leaf runs using:

- raw and uncached provider tokens;
- cached input, uncached input, output, and reasoning tokens;
- distinct provider usage changes;
- input context per usage change;
- workflow stage totals;
- all three feature scores and workflow outcomes.

If direct reading repeatedly raises tokens while quality stays comparable, mediated context delivery
causes at least part of the saving. The observed increase divided by the direct-Codex versus normal
Work-Leaf gap is reported as a descriptive sample fraction, with no claim of population precision.

If direct reading overlaps normal Work Leaf, the read route is not supported as a material cause by
this batch. The next control must target a different mechanism; the benchmark implementation is not
changed merely to force a preferred result.
