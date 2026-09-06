# Work Leaf token-mechanism isolation

## Objective and authority

The objective is to identify the concrete mechanisms behind Work Leaf's observed raw-token
savings, using controlled interventions followed by independent confirmation. The user accepts
the historical approximately 50% raw-token reduction as sufficient endpoint evidence and assumes
no quality reduction for this investigation. This is not another Direct-versus-WL replication.
Quality artifacts and failures remain recorded; quality is not a post-hoc inclusion filter.

The current user authorization permits implementation changes exclusively in benchmark builds,
and requires that each intervention change only its declared experimental factor. It permits
continuing from screening into confirmation and further mechanism investigation. It supersedes
the completed two-observation pilot's pause for this separate study, not its historical record.
No Direct sequential run belongs to the initial screen or confirmation.

All generation uses the existing ChatGPT subscription, stored authentication, Codex CLI 0.153.4,
GPT-5.5, and `xhigh` reasoning. No API-key authentication, API-credit fallback, copied credentials,
or model substitution is permitted. Three complete workflows run concurrently in separate
checkouts and artifact directories whenever a wave contains three admitted observations.

## Experimental boundary

The nondefault Cargo feature `bench-experiments` contains private experimental code. Default
builds exclude that code and its hooks. In a feature-enabled build, runtime activation additionally
requires `WORK_LEAF_BENCH_EXPERIMENT=1`, an absolute manifest path in
`WORK_LEAF_BENCH_EXPERIMENT_MANIFEST`, and a matching `WORK_LEAF_BENCH_RUN_ID`.
Malformed, incomplete, reused, or mismatched activation fails before an agent launches.

Every active condition, including control, uses the same feature-enabled binary and the same
evidence recording. Conditions differ only in a manifest label and their specified transformation.
No experiment label, evidence metadata, or explanatory banner enters an agent prompt. Control
retains the original prompt bytes. The default build ignores experimental activation variables.

The two initial candidate conditions are:

1. `ack-validation-unlimited`: in `src/orchestrator.rs::render_patch_applied_prompt`, replace only
   “run at most one focused validation step that is relevant to files you touched or checks you
   added.” with “run the required focused validation steps that are relevant to files you touched
   or checks you added.”
   Keep the accepted-patch receipt, automatic commit information, ownership constraints,
   no-resend instructions, and completion conditions. This tests the incremental post-patch
   validation-cardinality cue. It does not remove the broader focused-validation policy in
   `src/agent.rs::PromptPolicy`.
2. `command-guidance-neutral`: omit only the next-directive/brief-explanation guidance in
   `src/orchestrator.rs::render_command_result`. Preserve command status, output, output
   compaction, lock state, pending diffs, parsing, and follow-up dispatch. This tests the guidance,
   not command-output truncation or permission to execute more commands.

Each transformation has a recording-backend test that fails without it and checks exact original
versus forwarded strings, including adversarial occurrences of the same words inside command
output. Control and non-target sites remain identical. Experimental evidence records both strings
and their byte difference; bytes are not represented as measured tokenizer counts.

No timing, cancellation, provider tool, message-count, task, validation-execution, or output-limit
change is part of either candidate. Removing or replacing an instruction inherently changes its
input tokens; that direct cost must be distinguished from subsequent changes in model behavior.
Artificial padding, forced busywork, synthetic repeated output, and task-specific orchestrator
branches are prohibited. Mediated commands and patches keep their normal safety/ownership rules.

## Frozen workflow and launch gates

The task is the original three-feature benchmark at base commit
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`, task-list SHA-256
`45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`.
All conditions use the normal concurrent `bench-three-features` driver, mediated reads, reviews,
fixes, linearization, final format/clippy/test checks, and the canonical three-feature scorer.
Neither task text nor quality checks are supplied differently between conditions.

The driver source is a clean snapshot. Runtime and observer binaries, their actual source inputs,
Cargo lockfiles, driver, wrapper, scorer, analysis helpers, protocol, schedule, and manifests are
hashed before provider admission. The driver source revision and experimental binary source
identity are separate facts; a clean driver does not imply an unmodified runtime binary.

Before the first launch: new study tests, `cargo fmt`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-targets --all-features` pass for affected crates. Default-build identity and
feature-control identity are verified. A real subscription-backed smoke verifies actual patch and
command handoffs with experimental evidence, plus ordinary interruption and raw continuation.
A fake backend is not the real-agent gate. Source review covers architecture and documentation.

Observer settings are identical across conditions: raw response metadata enabled, a maximum
1,000 ms pre-forward usage grace, and immediate release on resumed output (`forward`). The
experiment does not extend response completion waits. Stage limit is 7,200 seconds, busy stall
1,800 seconds, idle stall 300 seconds, outer supervisor 86,400 seconds, termination grace
20 seconds. Limits are failure bounds, not instructions to shorten useful agent work.

The supervisor strips inherited Work Leaf experimental/settings overrides and API authentication
or endpoint overrides before setting the frozen configuration. It preserves the existing
subscription home. It records full configuration hashes plus a secret-free safe-settings summary.
Only explicitly allowlisted notice bookkeeping is ignored in the behavioral hash. Other drift
stops new admissions; active observations reach recorded bounded outcomes. Recorded global
settings plus CLI overrides are not falsely labeled a complete proof of effective configuration.

## Discovery and independent confirmation

`screen-01` contains exactly three workflows in one concurrent wave: one control and one of each
candidate. System randomness assigns conditions to the three launch slots before admission.
Screening results are descriptive, not statistical confirmation. All outcomes remain visible.

After the screen, a candidate is eligible for confirmation only if its intended prompt difference
was actually delivered, non-target prompts were unchanged, and trace evidence shows the proposed
downstream behavior was possible. Among eligible candidates with a positive conservative raw-token
contrast, select the largest lower-bound increase over control. Ties use the fixed order
`ack-validation-unlimited`, then `command-guidance-neutral`. If neither has a positive lower bound,
the screen selects no candidate for this confirmation. Use its action traces to design a separate
targeted screen with a frozen hypothesis and observation count; do not repeatedly screen the same
factor until a favorable observation appears. Do not replace a failure.

Freeze a separate confirmation schedule before confirmation generation. It contains exactly
12 fresh workflows: six control and six of the selected variant. Two blocks each contain six
launch slots, split into two concurrent waves of three. Within each block, uniformly select among
the 18 allocations containing three control and three variant labels, with both labels present in
each wave. The two blocks are independently randomized, giving 324 possible joint allocations.
The frozen schedule records block, wave, slot, condition, and randomization rule. Launch-order
randomization inside a wave does not replace treatment allocation randomization.

The primary directional hypothesis is that removing the candidate constraint increases total raw
tokens. Use the mean variant-minus-control contrast and an exact one-sided randomization test over
the actual allowed allocations. The test assumes no treatment interference between whole
workflows. Checkouts are separate, but CPU, I/O, and subscription capacity are shared; record
resource/limit incidents and do not claim randomization alone eliminates that possible interference.
With interval-valued missing usage, report a conservative envelope
for the p-value and contrast, not a midpoint-derived exact value. A positive confirmation requires
the entire observed accounting contrast above zero and the conservative p-value upper bound at
most 0.05, as well as verified intervention integrity and the downstream trace chain. Failure to
meet that criterion is not proof of zero effect. Accounting intervals are not sampling confidence
intervals, and the randomization test is not a percentage-attribution interval.

Only one selected candidate receives this confirmatory test; screening selection uses independent
data. No peeking-based stop or sample extension belongs to that test. Further candidates,
interactions, or replications require a separate prelaunch phase record with their hypothesis,
allocation, fixed observation count, and multiplicity treatment. They remain within the user's
mechanism-investigation authority, but cannot retroactively become the first confirmation.

## Accounting and mechanism evidence

Primary tokens are input plus output, including cached input. Also report cached input, uncached
input, output, and reasoning output (a subset of output, never added twice). Include implementation,
review, fix, linearization, title/hidden workflow calls, failed work, and retries in the inventory.
Join by recorded invocation, agent, thread, turn, and response identifiers. Identical repeated
cumulative notifications are not new usage. Missing usage is never zero.

Compaction generation also belongs to that scope. A prelaunch historical audit identified a
native response associated with a Direct compaction that the old cumulative terminal total did
not include; its original study totals remain immutable. For this WL-only investigation, audit
the exact observer-matched native rollouts for compaction markers and per-response usage records
in every condition. Compare response identities with the app-server ledger before adding any
counter. An unmatched native record is not silently counted twice or silently discarded; report
its deduplication evidence, separate scope correction when proved, or unresolved coverage. Do not
claim a complete full-generation causal contrast while compaction accounting remains unresolved.

The prospective tail audit preserves predecessor identity, lifecycle, and boundary checks while
allowing multiple paired assistant items in one visible response boundary. It retains the strict
historical interpretation separately. Each bounded missing response uses the documented model
ceiling of 1,178,000 input-plus-output tokens (input bounded conservatively by the documented
1,050,000 context window, plus 128,000 maximum output; see the
[model specification](https://developers.openai.com/api/docs/models/gpt-5.5)) and explicitly
requires a defensible response inventory, no hidden retry/opaque response, and no intervening
preemption that invalidates that inventory. A matching CLI/schema and plausible transcript are not
unconditional proof of hidden provider behavior. An unsupported inventory remains unbounded.

For each exposure, retain the original and delivered prompt, agent identity, actual transcript
delivery, subsequent commands, validations, edits, completion directives, and usage boundaries.
Separate exact recorded response counts from cumulative-usage-advance proxies. Attribute stage
totals by lifecycle evidence, leaving unallocatable tails explicit. Include pre-exposure usage as
a diagnostic: a purported post-patch effect cannot explain work that preceded its first exposure.

The mechanical identity is total raw tokens = sum of input and output over generated responses.
The mechanism question is why the workflow changes the number of responses and/or their input
histories. A successful explanation must connect the manipulated instruction to an observed action
change (for example extra validation/repair/continuation cycles), then to additional responses and
their repeatedly supplied context. Cached-input arithmetic alone is not an explanation of why those
responses occurred. Stage differences alone are not causal shares.

Retain final quality scoring and available Git checkpoints. Any intermediate scoring is offline on
saved snapshots and is never fed back to a live condition. Do not exclude expensive failures or
low-quality artifacts to obtain an effect. The user's no-quality-loss assumption is a working
assumption, not a falsified scorer result to erase.

## Retention, progression, and reporting

Every admitted observation has one immutable identity and one launch attempt. Retain successful,
failed, incomplete, timed-out, and not-launched outcomes. Infrastructure failure stops new
admissions in that phase, not the recording of active work. No replacement is automatic. A failed
phase receives an integrity report and any repaired continuation has a separate frozen admission.

The stopping target is an evidence-backed mechanism explanation, not merely a positive endpoint
gap or a favorable screening value. A confirmed single cue establishes that cue's causal effect in
this benchmark context; it does not automatically explain the entire historical 50% difference.
If substantial causes remain unresolved, continue with targeted, isolated candidates, including
focused-validation policy, context refeeding, cohesive patch/commit handoff, review context, or
concurrent scheduling, in separate frozen phases. Multi-factor interventions require factorial or
otherwise explicit interaction analysis, not subtraction of unrelated sample means.

Final claims must name the source symbols, delivered treatment, downstream event chain, measured
effect, accounting assumptions, and reproducibility evidence. Do not promise mathematical certainty
about stochastic agent behavior or invent unique percentage shares from an underidentified design.
The historical approximately 50% endpoint evidence and this study's component effects remain
separate quantities unless a directly measured bridge justifies connecting them.
