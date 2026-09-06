# Raw-token first-batch result

Status: **complete; collection paused** at 2026-09-06 20:58:49 UTC. Exactly one Direct sequential
workflow and one normal concurrent Work Leaf workflow ran on the existing ChatGPT subscription,
GPT-5.5 / `xhigh`, CLI 0.153.4. Both workflows and final repository checks pass; no outcome is
replaced and no additional benchmark is queued. Admission was at 19:14:48 UTC.

The full [pilot result](../efficiency-raw-token-pilot-20260906T185328Z/RESULT.md) retains reports,
quality logs, source hashes, stage usage, and the stop record.

| Result | Direct | Work Leaf |
| --- | ---: | ---: |
| Recorded raw input + output | 47,013,757, exact | 22,786,969, lower bound |
| Frozen raw upper | 47,013,757 | Unknown |
| Separate conditional post-hoc raw upper | 47,013,757 | 27,498,969 |
| Frozen feature checks | 3/3 | 2/3: visual check fails |
| Driver-reported workflow duration | 6,222 s | 2,786 s |

The frozen primary comparison is inconclusive (`unbounded_accounting_gap`): one of four unresolved
WL tails fails the predeclared single-message rule. The separately labeled post-hoc audit allows
multiple assistant items in one response and supplies a **conditional 41.51%–51.53% raw reduction**,
width 10.02 percentage points. Its model-limit and complete-visible-boundary assumptions, including
no hidden retries or unobserved preemption, remain explicit. Missing usage is not recovered and the
frozen result is not overwritten.

Uncached input plus output is secondary: Direct 1,906,045; WL conditional envelope
1,719,449–6,431,449, savings −237.42% to +9.79%. That separate metric does not invalidate the positive
conditional raw interval. The historical 45.38%–51.62% raw result and its distinct 140-point uncached
uncertainty concern older observations; this pilot does not supersede or pool with them.

The largest recorded raw stage gap is implementation/fixes (15,705,210). Cached input constitutes
24,040,192 of the 24,226,788 recorded full-workflow gap; reported usage advances are 388 versus 242.
These are descriptive component and activity counts, not causal shares or exhaustive model-call
counts. One workflow per arm and the 3/3 versus 2/3 quality result do not establish same-quality
population savings.

Final identity verification reports global `/home/user/.codex/config.toml` hash drift during
collection. Other pinned artifacts match, but the writer/content difference is unresolved. The
pilot retains that experimental-control limitation in its post-run integrity audit.

All 76 provider-free measurement tests, all 24 mechanism-study tests, and all four predecessor report
tests pass; required Rust checks for both crates pass before admission. Product runtime, production
drivers, prompts, and the 1,000 ms/forward interruption-observer policy have no implementation changes.
The successful real subscription pair and the earlier two-turn smoke verify the agent-facing paths;
they do not establish exact interrupted-response accounting.

Further collection requires a separate protocol after this pause. The user's default of three
complete benchmarks concurrently is recorded in `../../AGENTS.md` and
`../../docs/benchmark-operator-policy.md`; it applies when at least three observations are approved,
not as authority to expand this two-run pilot.
