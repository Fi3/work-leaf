# First raw-token pilot result

## Outcome and stop

The admitted pair is complete and collection is paused. Exactly one Direct sequential workflow and
one normal concurrent Work Leaf workflow ran through the existing ChatGPT subscription, with
GPT-5.5 / `xhigh` and Codex CLI 0.153.4. Both launchers exited 0, completed their reviews and
linearization, and passed final format, clippy, test, and candidate-startup checks. No observation
was replaced and no additional benchmark is queued. [Runner result](FIRST-BATCH-RESULT.json).

The frozen accounting comparison is **inconclusive**: Direct is exact, while one of Work Leaf's
four unresolved interruption tails does not satisfy the frozen cardinality rule. A separately
identified post-hoc audit supports a **conditional 41.51%–51.53% raw-token reduction** for this pair.
That supplement does not replace the frozen result or recover the missing usage. The artifacts
also differ on one frozen quality check, and global configuration drift limits experimental control.

| Result | Direct sequential | Normal concurrent Work Leaf |
| --- | ---: | ---: |
| Workflow result and final repository checks | Pass | Pass |
| Frozen feature checks | 3/3 | 2/3 |
| Driver-reported workflow duration | 6,222 s (1 h 43 m 42 s) | 2,786 s (46 m 26 s) |
| Recorded raw input + output | 47,013,757 | 22,786,969, lower bound |
| Frozen raw upper | 47,013,757 | Unknown |
| Separate conditional post-hoc raw upper | 47,013,757 | 27,498,969 |
| Complete observer invocations | 37/37 | 36/36 |
| Unresolved interruption tails | 0 | 4 |

The driver durations exclude final candidate replay build/smoke/publication. Work Leaf's report was
published at 20:02:26 UTC and Direct's at 20:58:49 UTC on 2026-09-06; the runner records both terminal
rows when the pair finishes, not Work Leaf's earlier actual completion time. Both started at
19:14:48 UTC in isolated checkouts. The randomized saved launch order is WL then Direct.

## Token accounting

| Recorded component | Direct | Work Leaf lower bound |
| --- | ---: | ---: |
| Input | 46,758,502 | 22,553,632 |
| Cached input, included in input | 45,107,712 | 21,067,520 |
| Uncached input | 1,650,790 | 1,486,112 |
| Output | 255,255 | 233,337 |
| Reasoning output, included in output | 127,926 | 149,924 |
| Raw input + output, primary | 47,013,757 | 22,786,969 |
| Uncached input + output, secondary | 1,906,045 | 1,719,449 |

The [frozen analysis](pair-analysis.json) retains `unbounded_accounting_gap` for WL and an unknown
lower saving endpoint. Three tails satisfy its one-message predicate; the fourth contains two
completed assistant messages. Multiple messages can belong to one response, but the frozen rule
does not admit that shape. Available raw completions and unchanged cumulative usage do not account
for that missing tail.

The [tail audit](SUPPLEMENTAL-TAIL-AUDIT.md) and
[separate pair supplement](SUPPLEMENTAL-PAIR-ANALYSIS.md) retain the source hashes and assumptions.
Conditional on complete visibility of normal response/tool/compaction boundaries, no hidden retries,
opaque subresponses or unobserved preemption, and the subscription model honoring its documented
limits, each of the four tails has at most one missing response. Each receives the deliberately
loose 1,178,000-token model ceiling: 1,050,000 input plus 128,000 output. Missing usage remains unknown;
the upper adds 4 × 1,178,000, not a guessed typical response length.

The separate raw envelope is 22,786,969–27,498,969, versus Direct's exact 47,013,757. Using the full
Direct workflow as denominator gives **41.5087%–51.5313% fewer raw tokens**, width **10.0226 percentage
points**, under those assumptions. This entire conditional interval is positive. It is not a
confidence interval, an unconditional provider-call inventory, or a same-quality saving claim.

The secondary envelope is 1,719,449–6,431,449 uncached-input-plus-output tokens, giving
**−237.4238% to +9.7897% savings**. Its direction is unresolved. This different metric does not widen
or invalidate the conditional raw interval. Neither interval supersedes the historical
45.38%–51.62% raw result, which concerns different observations and a different CLI/version-specific
bound; this pilot is not pooled with that sample.

## Saved-artifact quality

| Frozen check | Direct | Work Leaf |
| --- | --- | --- |
| Visual selection | Pass | Fail |
| `/status` routing | Pass | Pass |
| Completion acknowledgement and reopening | Pass | Pass |

[Quality evidence](quality.json) pins the unchanged scorer and each retained log. WL's visual test
exits 101 at `tests/quality_visual.rs:26`: the rendered left-pane character-selection state does
not satisfy `has_visual_status(..., "left", "char")`. This is the specific observed assertion
failure, not a finding that every visual-selection behavior is absent. There is no retry or candidate
repair. [Failure log](scorer/logs/work-leaf-001/visual.log).

These are three narrow fixture checks on each saved artifact, not three independent benchmark
observations or comprehensive correctness tests. Passing the candidate's own final repository
checks does not override this independent failure. This pair does not meet the protocol's
matching-output criterion; it does not establish whether that difference persists across runs.

## Descriptive mechanism evidence

| Stage | Direct recorded raw | WL recorded raw lower | Direct usage advances | WL usage advances |
| --- | ---: | ---: | ---: | ---: |
| Implementation and fixes | 30,149,576 | 14,444,366 | 247 | 133 |
| Review | 9,500,669 | 4,319,091 | 98 | 58 |
| Linearization | 7,363,512 | 3,962,985 | 43 | 48 |
| Title | 0 | 60,527 | 0 | 3 |
| Total | 47,013,757 | 22,786,969 | 388 | 242 |

The largest recorded stage gap is implementation/fixes: 15,705,210 raw tokens. Of the full
24,226,788-token recorded gap, 24,040,192 is cached input, 164,678 uncached input, and 21,918 output.
This arithmetic points to repeated cached-context processing as the dominant recorded component;
it is not a causal attribution. The missing tails are not allocated into this stage table.

Usage advances are a reported-generation proxy, not an exhaustive model-call count. Captured
provider tool actions are 745 `exec_command` / 73 `apply_patch` for Direct versus 250 / 5 for WL.
WL also delegates work through orchestrator commands, so these are not comparable totals of all
repository reads, edits, or validation operations. The stage and rollout provenance is retained in
`pair-analysis.json`; no percentage of causally explained savings is inferred from these counts.

## Integrity, verification, and continuation boundary

The [post-run integrity audit](POST-RUN-INTEGRITY-AUDIT.md) retains a failed final identity check:
`/home/user/.codex/config.toml` differs from its prelaunch hash. Its observed birth/modification/change
timestamps fall within collection. The writer and exact content difference are not established; no credential or
configuration copy is retained. The other pinned files, frozen manifest/schedule and clean source
snapshot match. This is an unresolved global-configuration control limitation, not a green full
post-run verification. Matching captured model/effort does not establish equality of every setting.

Prelaunch required format, all-target/all-feature clippy and tests pass for both crates, as recorded
in [readiness](READINESS.md). All 76 provider-free measurement tests pass, including 13 runner and
22 batch-analysis tests. The approved report-test correction passes all 24 mechanism-study tests;
the predecessor report suite passes all four tests. Product runtime files under `src/`, production
drivers, prompts, and the existing 1,000 ms/forward interruption policy have no implementation changes
in this work. The actual subscription pair supplies real-agent launch, resume, review, fix and
linearization verification; it does not turn incomplete WL accounting into an exact measurement.

Independent final review finds no reporting or implementation blocker. A separate provider-free
replay verifies 20 supplemental source pins, all 15 rollout hashes, six scorer-log hashes, stage sums,
interval arithmetic, both retained exit-0 outcomes and the paused flag. All local report links resolve
and `git diff --check` passes. The unresolved configuration identity deviation remains explicit.

Before more provider data, a proposed next protocol should resolve the tail-rule coverage and
configuration-control gaps, then choose a finite sample and quality criterion for mechanism work.
That is a recommendation, not an executed fix or approved additional batch. No runtime ablation is
part of this pilot. Collection remains paused for the user-facing result.

The user's standing default of **three complete benchmarks concurrently** is preserved in
`../../AGENTS.md` and [the operator policy](../../docs/benchmark-operator-policy.md), with the exact
other-chat authority. It applies when at least three observations are approved; it does not add a
third observation to this explicitly admitted two-run pilot or waive the pause.
