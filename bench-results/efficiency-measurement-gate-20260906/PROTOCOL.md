# Raw-token measurement pilot and first batch

## Scope and current authority

All provider work uses the user's normal Codex subscription and existing subscription login.
Public API-key generation, API-credit purchases, and switching either benchmark to API-key
authentication are outside scope. The saved API diagnostics are excluded exploratory records and
are not a continuation path. Their helper must not be run as part of this study.

The authorized next benchmark batch contains one direct sequential Codex workflow and one normal
concurrent Work Leaf workflow. It is a measurement pilot. Collection pauses after both admitted
observations finish for a user-facing result. A measurement/infrastructure failure stops new
launches immediately; already active workflows reach bounded recorded outcomes. No second batch or
automatic replacement is authorized by this protocol.

Raw input plus output is the primary metric. Uncached input plus output is secondary and does not
block a raw-token comparison. The historical normal-workflow raw reduction is 45.38%–51.62% under
its documented missing-response limits; the much wider uncached interval is a different metric.
Missing-usage bounds describe the collected observations; they are not confidence intervals for repeated runs.
The exact sequential completed-response controls remain useful independent evidence. Their 45.84%
raw-token reduction coexists with a 15.42% increase in uncached input plus output.

## Conditions required before launch

1. The analysis regression tests, all relevant study tests, `cargo fmt`,
   `cargo clippy --all-targets --all-features -- -D warnings`, and
   `cargo test --all-targets --all-features` pass. A review confirms that the patch preserves the
   ownership and integration surfaces in `docs/architecture.md`.
2. The observer supplies authoritative input, cached-input, and output counts where available and
   explicit evidence-backed bounds for unresolved responses where possible. Finite bounds require
   both a defensible missing-response inventory and a ceiling valid for the actual model/version;
   historical CLI limits are not automatically transferable. Null fields and unchanged cumulative
   notifications are not exact usage. The previous dual-metric five-point cutoff is not a launch
   gate. Record the width and assumptions; an unbounded or zero-crossing raw contrast is inconclusive,
   not grounds to replace the observation or launch another batch.
3. A bounded real-agent scenario proves completed-response accounting, normal directive cancellation,
   and a raw follow-up in the same conversation. Available counts reconcile by response/thread identity and
   exclude double-counting resumed cumulative totals. Every model call, including hidden/title,
   review, fix, and linearization work, belongs to the workflow inventory.
4. Work Leaf handles the directive through its ordinary runtime path. Observer instrumentation
   preserves the historical 1,000 ms maximum pre-forward grace and immediate release on resumed
   output; response metadata does not change that policy. Extra generation from a different
   completion/wait policy is not classified as usage of the normal condition.
5. A launch manifest freezes both drivers, observer, binaries, task text, base commit, scorer,
   provider route/authentication mode (no credentials), model, reasoning effort, validation policy,
   timeout values, and any provider-specific measurement adapter. Both conditions use the same
   provider route, CLI version, GPT-5.5 model, and `xhigh` effort. An unavailable requested model is a
   blocked gate, not permission to substitute another model.

Historical `store`/`background` rejections and thread-usage null results are retained in their
original evidence. Measurement instrumentation must preserve the subscription provider route,
agent tool loop, cancellation, prompt history, and resume behavior. The batch requires the
integrated real-agent gate on that exact configuration.

The current provider availability checks are recorded in `TELEMETRY.md`. The real scenario in
`NORMAL-SMOKE-RESULT.md` verifies subscription launch, ordinary interruption, and raw follow-up;
it does not recover the interrupted response's usage. That accounting limitation remains explicit
in this pilot. `OBSERVER-PLAN.md` and `OBSERVER-RESULT.md` describe the earlier telemetry attempt,
not the current primary metric or admission threshold.
`HANDOFF-RESULT.md` records a successful synthetic tool-handoff feasibility check on the normal
subscription. It is not an integrated Work Leaf workflow or a cancelled-response accounting check,
and does not satisfy or waive these launch conditions.

## Frozen workflow and recording rules

The task remains the original three-feature Rust benchmark at
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`, with the existing `/status` feature scorer and the same
final non-mutating format, clippy, and test gate. Work Leaf submits its three features concurrently;
direct Codex implements them sequentially with its normal reviews and fixes. Validation remains
unrestricted within the existing workflow policies. No instrumentation changes the task text,
orchestrator prompts, feature order inside either condition, or scoring criteria.

Before launch, draw and save the order of the two condition labels using system randomness. Both
workflows may run concurrently in separate checkouts, with at most two top-level workflows active.
Launch order is provenance, not a statistical pairing. Save the concrete schedule and all artifact
identities before either task reaches a provider.

The existing drivers retain 7,200-second stage limits; Work Leaf retains its 1,800-second busy and
300-second idle stall limits. Direct review rounds retain the normal unlimited-round setting.
The outer supervisor has a frozen 86,400-second wall limit and a 20-second termination grace to
bound a pathological driver loop. Hitting that limit is a retained timeout, not permission to retry.

Keep every admitted outcome: full success, partial features, timeout, workflow failure, and missing
usage. A missing-usage observation is marked incomplete and excluded from exact token contrasts;
it may enter a bounded contrast only with the documented inventory and ceiling proof. Its existence
and quality result remain in the batch report even if finite bounds are unavailable. Do not replace an observation to
obtain a preferred score, complete telemetry, or a favorable token difference. A failure stops new
launches; active workflows are brought to a bounded recorded outcome rather than silently discarded.

## First-batch result and acceptance criteria

Report each workflow separately with:

- cached input, uncached input, output, and total tokens;
- provider-reported reasoning output as a subset of output, never an additional total;
- unresolved response count and reconciliation status;
- all three feature outcomes, final checks, workflow termination status, and elapsed time;
- exact versions, provider route, and retained artifact paths.

The pilot supports a directional raw-token result when both workflow inventories are accounted for
exactly or with supported bounds, the entire saving interval lies on one side of zero, and the
intended normal workflow is preserved. Report its actual width rather than a binary five-point
cutoff. Uncached uncertainty does not invalidate a supported raw contrast. Unresolved responses
remain explicitly inventoried. A
partial-quality result does not erase a telemetry pass, but prevents the pilot from supporting a
same-output saving claim. Such
a claim for the two observed artifacts requires all three features and final checks to pass in both.
This is an artifact criterion, not population-level quality equivalence.

With one observation per condition, report any token difference as descriptive. Do not calculate a
confidence interval, statistical significance, a population saving, or a percentage of causally
explained savings from this pilot. The stop is fixed at the first batch regardless of direction or
magnitude. A measured increase, a near-zero difference, and an inconclusive outcome are valid results.

## Requirements for any later study

Further collection needs a separately frozen protocol after the first-batch report. Its primary
metric remains raw input plus output; cached input, uncached input, and output remain separately
reported. Sampling precision and accounting precision are separate questions. Any numerical
sampling target must be chosen against available variation and a finite sample limit before launch;
six runs per group are not assumed sufficient.

The later protocol must choose a quality estimand and numerical acceptance margin before its data
are collected, account for clustering of the three features within a workflow, and specify either a
fixed sample size or an inference method valid for its sequential stopping rule. Randomization must
respect declared collection blocks, and inference must respect that design. It must not continue
sampling merely until savings become positive. An interval spanning a practically negligible
difference can be informative if it is narrow enough.

Causal follow-up should isolate candidate mechanisms, beginning with the implementation/fix handoffs
that account for the largest observed stage difference. The pilot records per-stage usage,
generation counts, tool activity, and quality to guide that design. Neither product-runtime
ablations nor further provider batches are part of this first-batch execution. Ordered
differences of sample means are descriptive allocations. A claim that at least 90% of a reproducible
effect is explained requires uncertainty for the contributing controls and denominator, plus stated
assumptions about intervention interactions. Percentage attribution is undefined when the endpoint
difference interval includes zero.
