# Candidate v4 prelaunch review

The introduced source and diagnostic-verifier review is closed with no remaining findings.
This is a provider-free prelaunch review, not a record of successful real-agent diagnostics.
The [screen protocol](PROTOCOL-CANDIDATE-SCREEN.md) admits three modified workflows only;
the existing read-factor phase remains parked and no fresh control or Direct run is authorized.

## Reviewed boundaries

- `src/agent.rs::PromptPolicy` collects separate owned candidate-policy spans. Public `inject`
  remains baseline; the private delivery path preserves existing injection sites and raw known-session
  followups. Default rendering and v1–v3 evidence contracts remain separate from v4.
- `src/bench_experiment.rs` admits only the three v4 conditions. Its private candidate adapter
  checks ordered UTF-8 components and identical intervening bytes, records both complete strings,
  selects only the applicable factor and propagates recording errors before delivery.
- `src/orchestrator.rs::bench_repeat_read_candidate` uses held tracked snapshots and one owned
  repeat section. Ordinary read/diff rendering and untracked bundle allocation occur once; failures,
  explicit-bundle prefixes, refreshes and tracker advancement/clearing remain ordinary. This author's
  C02 implementation received a separate clean source review from `study_harness`.
- `src/cli.rs::CommandChat` retains successful owned launches across worker clones and supplies an
  exact original-request insertion only at the author review-fix boundary. Failed launches cannot
  replace provenance; absent provenance cannot be inferred from a provider transcript.
- `runner_candidates.py` compiles the byte-pinned original supervisor in a private module. Exact-three
  validation, adapter/engine identity binding and no-control/no-replacement scope are checked;
  environment, subscription route, timing and trust/outcome handling retain the engine's behavior.

The documentation catalog contains architecture, operator-policy and file-workflow documents.
The architecture's v4 paragraph describes these private ownership boundaries; ordinary file-workflow
documentation remains accurate for default behavior. There is no public API expansion. Added prompt
work is linear in held bytes and snapshot/span count, with logarithmic keyed launch lookup. The
diagnostic's repeated capture scans have at most six admitted starts and two threads. The runner's
inherited O(W×E) evidence verification remains bounded here by one wave; no unbounded quadratic
algorithm is introduced by these candidate boundaries.

## Diagnostic admission and receipt checks

| Modified diagnostic | Admitted outer provider turns | Threads | Actual path |
| --- | ---: | ---: | --- |
| Requested-repeat full | 3 | 1 | launch → mediated read → repeated full-current read → done |
| Unified-diff preferred | 2 | 1 | launch → actual unified patch/application/ACK → done |
| Original-request resupply | 6 | 2 | author edit/ACK/done → reviewer finding → author fix/ACK/done → reviewer acceptance |

`tests/bench_candidate_subscription_smoke.rs` uses actual `CommandChat` and `CodexBackend`.
Admission rejects unexpected agent IDs, launch/send shape, prompt boundary or an extra outer call;
reply checks constrain each diagnostic action. Outer turns are not asserted to equal all native
response generations. No built-in tools, commands or subagents are requested by the scenario.
The public item allowlist rejects tool/action types and unknown shapes rather than trusting that
instruction alone. Git fixture setup is local and precedes provider launch.

The prelaunch guard requires the observer's exact primary marker, work-leaf condition, no parent
invocation, raw usage, 1,000-ms grace, immediate forward-on-output and project-layer inventory.
The configured backend uses GPT-5.5 and the supplied absolute reviewed proxy; root's admission must
bind the subscription-only wrapper and xhigh settings. The fixture copies no auth/config material
and does not select another provider. Provider execution has a 118-second watchdog, bounded shutdown,
a 15-second receipt poll and a separately bounded observer-stop command.

Receipt checks join exact original/forwarded turn-start frames, unique typed accepted replies and
one public user item per exact (thread, turn), comparing the entire one-text input. Each matched user
identity is consumed once. The last accepted turn of each thread must have a terminal notification;
its recorded interrupt, when present, must be forwarded and acknowledged. Trace-selected text joins
an actual accepted input, repeated-read body bounds recover the full fixture text, and resupply bounds
recover the exact original request. These checks inspect public payloads, not private reasoning.

The C16 preferred patch instruction retains explicit `@work-leaf end`, so format preference does
not also remove the normal streamed handoff cue. Observed RED regressions cover that cue, missing
public/forwarded delivery or forbidden tool activity, a missing author terminal despite reviewer
completion, and reuse of one accepted turn balanced by an unrelated user item. All are GREEN.

## Independent checks and source identities

`cargo test --features bench-experiments --test bench_candidate_subscription_smoke --test
bench_candidate_policy --test bench_candidate_activation --test bench_candidate_followup --test
bench_repeat_read` passed: **23 tests**, six intentionally ignored subprocess/real-provider fixtures.
The private adapter target passed three tests; `python3 -m unittest test_runner_candidates.py`
passed three. Earlier default read targets passed 13 tests with three ignored subprocess fixtures.
Root owns the final required fmt/clippy/all-target/all-feature and release gates for admitted bytes.

```text
src/agent.rs                       a9e6065a450d05bf299202a2d6f44dd2dae33a3324e8c70e9f91d36a22b242fd
src/bench_experiment.rs            8c9db45eda3a10acabe441259e901617d44bf3d92f800231099ba84036de9a89
src/bench_candidate_experiment.rs  e327d28fb05ceaa8f9c9465a78fc61069eddb56faa767b4909f29f83684d8b94
src/orchestrator.rs                027c35a9eaf24e99335001e68f7d0612a262831af80f8677874525db95e9332b
src/cli.rs                        e68988f004ac9a61d25c0565b41dd46a371ebcf62a2415c0aabc4d6d83a96763
tests/bench_repeat_read.rs         adf1abf6740b937d408a267897587dbe394c9d65fe5a322af002470cf7fa6775
tests/bench_candidate_policy.rs    866ad275b4e8390b9608bec7057c98fbb3b03e3fb3de33e2e6ceae07818db4d2
tests/bench_candidate_subscription_smoke.rs 1c56c54987aada7c92f2824bd4830476541c8821479cfe61eff531a965167ab6
runner_candidates.py              ecac7753fb535c59974deedf6a04f0ce11b6e05a19f8c4112416ab7e29b7cb03
test_runner_candidates.py         7deb0411164578dfcd8caedff7545490ba57ad729a37bfe68868e45600fb1f3e
```

## Required actual evidence

The three separately admitted real-subscription diagnostics remain pending actual receipts at this
review's closeout. Their failures must be retained, not silently retried. Passing this fixture alone
does not establish native rollout provenance, actual model/effort, complete response accounting or
absence of unobserved work; root's closed-capture/native audit must verify those before benchmark
admission. Scripted plumbing success is not natural representation-choice evidence, a net saving,
a causal share of the historical difference or permission for another control.
