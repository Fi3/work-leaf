# Prior protocol-package and interruption evidence

Scope: provider-free verification of the saved August 29–30 studies, not new generation,
a new endpoint, a quality-selected subset or a token retotal. All three modified
continued-response workflows and all six main package-comparison workflows remain
evidence. The old term “control” names those existing observations; it authorizes no
new control, timing experiment or replacement.

Authorities: [continued-response design](../efficiency-causal-validation-20260829T210343Z/05-INTERRUPT-CONTROL-DESIGN.md),
[result](../efficiency-causal-validation-20260829T210343Z/06-CONTINUED-RESPONSE-RESULT.md),
[continued evidence](../efficiency-causal-validation-20260829T210343Z/continued-response-evidence.json),
[main package report](../efficiency-mechanism-attribution-20260830T081131Z/FINAL-REPORT.md)
and [main evidence](../efficiency-mechanism-attribution-20260830T081131Z/evidence.json).
Their sample comparisons are retained below without importing the old causal-share
allocation or treating its telescoping bridge as explanatory completeness.

## C25: which response can interruption affect?

The six accepted normal-H captures contain 287 directive-triggered interrupts:
252 released after usage, 34 after resumed output and one after the one-second timeout.
This audit independently read each evidence-pinned raw/grace stream, located the first
complete public directive in each of the 252 corresponding thread/turns, and checked
the following usage payload. All 252 have nonzero `last` usage contained componentwise
in an advancing input/cached-input/output/reasoning cumulative delta; none fails that
check. This is a payload check, not reliance on an outcome label alone.

Example: H `exact-normal-001`, thread
`01a04ebc-2a9e-7662-8f06-5d0cfe7c08fd`, turn
`01a04ebc-2ae8-7572-b7f5-bd0107d310a9`, complete directive S93 → usage S95.
Its raw and grace paths/hashes resolve through
`efficiency-exact-normal-work-leaf-20260829T181318Z/evidence.json`
`accounting.capture_bound_audits[0]`. The same check covers all six entries, not a
selected low-cost episode.

Those 252 forwarded interrupts cannot erase tokens from the **already measured current
response**. They do not establish that interruption never affects later response starts,
retained history, scheduling or future work. The remaining 35 cases are possible current
generation-truncation boundaries, not 35 proved large savings or additional calls.

### What the continuation treatment actually does

Both studies retain normal WL source `5b1d1ef9590850faed26052f909ddff7ff8f127d`.
The continued-response launcher selects observer source
`8eb26a4bf77b7fcbffbb60682f81069cd28880a9`, grace 120000 ms and
`WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME=wait-for-usage`; normal H uses
1000 ms and forwards on resumed output. This is the saved **bounded wait policy**, not
an unbounded removal of all interruptions. The longer cap also applies to a still
pending directive with no usable completion; keep that timeout-policy difference in scope.

At that observer commit, `ProviderUsageGrace::observe_server_value` (lib.rs:721)
records complete directive, resumed output and usage state;
`wait_before_interrupt` (747) waits through resumed output in the selected policy.
`forward_app_server_client_frame` (1084–1118) then writes the original request bytes;
its serial input pump prevents queued subsequent requests from overtaking that held
interrupt. `pump_app_server_stdout_with_usage_grace` (1120) keeps forwarding/capturing
provider output while the request waits.

The old observer's `exact_usage_seen` flag means a post-directive usage notification,
not by itself modern advancing-counter proof. This audit therefore checked the actual
payloads separately. All 162 usage-released decisions across the three continued runs
have advancing, attributable nonzero `last` usage after the directive. The remaining
one decision is the retained timeout below; no per-response cost is invented for it.

Normal WL's `src/codex.rs:276–290` sends an interrupt, unregisters that turn and returns
the accumulated public messages immediately when the detector accepts the directive.
Later provider text is therefore **not interpreted as another WL operation** in that
turn, although it is retained in the provider history. Delaying the interrupt can alter
later prompts/context and behavior; it is not equivalent to adding a fixed suffix price.

### Complete retained continuation population

Each saved workflow passes its implementation/final-gate workflow record and retains
its independently scored features. No incomplete-feature workflow is excluded.

| Run | Usage-completed resumed continuations | Timeout | Feature checks | Ordinary usage-release / total decisions |
| --- | ---: | ---: | --- | --- |
| continued-response-001 | 2 | 0 | visual/status pass; completion fail | 54 / 56 |
| continued-response-002 | 2 | 1 | all three pass | 55 / 58 |
| continued-response-003 | 4 | 0 | status pass; visual/completion fail | 45 / 49 |

The eight resumed continuations are two author and six reviewer responses. The
following physical locators retain every activation, in run/decision order. G denotes
`provider-usage-grace.jsonl`, C original client, S server, N the exact matched native
file. Each extra public message matches its native assistant item by exact item ID,
thread/explicit turn and full text; no private reasoning body is inspected.

| Run / G | Wait ms | First directive S → extra public S → usage S | Interrupt C / string RPC | Extra native N / item ID |
| --- | ---: | --- | --- | --- |
| 001 / 22 | 79891 | 39816 → 39897 → 39898 | 56 / `55` | 14 / `msg_03a045ed7e7e6714016a936969eb8487d29326640c10690202` |
| 001 / 33 | 454 | 55127 → 55147 → 55148 | 81 / `80` | 126 / `msg_03a045ed7e7e6714016a936ba8da7c87d29a29397d086a8e45` |
| 002 / 11 | 11487 | 4854 → 4891 → 4892 | 32 / `31` | 14 / `msg_0ec19313ac32cab5016a93671946dc87d2b20fe3634c32c41f` |
| 002 / 44 | 3020 | 64393 → 64599 → 64600 | 103 / `102` | 103 / `msg_04a970eb7b753673016a936cdb0b3487d2be326dce907b6f15` |
| 003 / 5 | 7208 | 555 → 598 → 599 | 17 / `16` | 41 / `msg_00212eaa420095b4016a9365dfcb2887d281945e30def4002e` |
| 003 / 8 | 847 | 3287 → 3316 → 3317 | 23 / `22` | 116 / `msg_00212eaa420095b4016a93669868e087d2b018dc9735190e95` |
| 003 / 11 | 58634 | 3466 → 3590 → 3597 | 32 / `31` | 14 / `msg_0c9b27e10f87f706016a9366eb1b7087d2af7c49aa4fb0570d` |
| 003 / 30 | 1347 | 38579 → 38630 → 38631 | 76 / `75` | 14 / `msg_003514ebe838cb69016a936a4d109087d284f78bf9e68a7df8` |

The omitted ninth activation is not omitted from the population: run 002 G36 waits
120000 ms after read S58405 and has no usage event in that turn. Its thread is
`01a04fda-9234-7f53-8347-77150b91774f`, turn
`01a04fda-928c-7b03-8a3f-18975aeefe4f`. The original interrupt is forwarded.
The saved cumulative/rollout accounting later makes that workflow total exact; this
does not retroactively turn the timed-out treatment into a completed activation.

Capture suffixes under each continued run's `*-three-feature-bench-artifacts/observation/app-server/`:
001 `00002694005645597466-675`; 002 `00002694005645598654-677`;
003 `00002694005645602216-676`. Native full paths/SHA values are retained in
`continued-response-evidence.json::rollout_integrity.files` (24 files, all rehashed).
For each activation, G supplies its exact thread/turn and thus the matched native source;
the table supplies the exact native item, never a timestamp-only join.

The saved all-three sample comparison is 2.79M–5.05M more raw tokens under the bounded
continued-response policy than normal H, across the documented normal missing-usage
interval. It is directional evidence **for this collected timing-policy contrast**,
not a population precision result or an independently additive contribution. The
older result's percentages and “proven contribution” wording do not establish a causal
share. Retain the timeout, different scored outcomes and nonrandom historical reference.

## Exact main package comparison: three plus three

The saved main contrast is `compact-direct-001..003` versus
`sequential-work-leaf-combined-001..003`, not normal concurrent WL versus Direct.
All six remain in the comparison. The exact saved workflow totals, report/analysis
hashes and 45 native source hashes were rechecked without rerunning either analyzer.
All 69 native files across this main comparison and the continued group identify CLI
0.150.1; do not import current CLI capabilities or response-ID attribution into them.

The manifest pins the same unmodified WL binaries/base/model/effort and separately
generated drivers. `generate_diagnostic_driver.py::generate` changes only the
declared driver sections after requiring exact predecessor hashes:

- Sequential WL: `SEQUENTIAL_WORK_LEAF_START` (28) and `SEQUENTIAL_ADVANCE` (54)
  post the next feature only after existing agents/review reach the terminal gate.
  The run-local driver and launch-log hashes match all three admissions; each actual
  log has three feature starts and the two prior-terminal gates. This is real schedule
  activation, not merely a `sequential` report label.
- Compact Direct: `COMPACT_DIRECT_LINEARIZE_PLAN` supplies original requests and exact
  reviewed provisional hashes/groupings, instead of open-ended target reconstruction.
  Its three actual plan payloads contain three targets and respectively 7/4/7 commits.
  Each whole saved prompt matches the unique native linearizer user input at N7,
  including its exact bytes. No new summary agent is involved.
- Sequential WL's launcher sets `WORK_LEAF_BENCH_NO_READ_PERMISSION=1` and the same
  120000-ms `wait-for-usage` observer policy. Actual first inputs show direct-read
  policy on seven non-linearizer threads in each run, with eight threads total.
  Structured writes, lock-mediated commands, ownership and WL review remain active.

The compact-plan native identities are respectively
`msg_01a05203-9559-7d11-934b-ba9ba11dba1c`,
`msg_01a051fd-bcc5-7510-9b30-fc870ad1998b`, and
`msg_01a0524d-071e-7890-a282-f18b7b634ec9`; exact native paths/SHA values and prompt
SHA values are in `evidence.json::groups.L.run_rows[*].rollout_integrity/activation`.

| Retained main run | Workflow record | Feature checks | Actual activation |
| --- | --- | --- | --- |
| compact-direct-001 | pass | 3/3 | compact 3-target / 7-commit plan |
| compact-direct-002 | pass | 3/3 | compact 3-target / 4-commit plan |
| compact-direct-003 | pass | 3/3 | compact 3-target / 7-commit plan |
| sequential-work-leaf-combined-001 | pass | visual/status pass; completion fail | sequential, direct reads, bounded continuation |
| sequential-work-leaf-combined-002 | pass | 3/3 | sequential, direct reads, bounded continuation |
| sequential-work-leaf-combined-003 | pass | 3/3 | sequential, direct reads, bounded continuation |

### Important response-versus-turn terminology

The main report's `interrupted_provider_turns=0` is **not zero interrupt requests**.
At observer commit 8eb26a4, `analyze_app_server` (lib.rs:5341–5358) counts interrupted
turns lacking accepted usage under its accounting rule, not every transport interruption.
The three sequential-WL raw captures actually contain 38/27/40 `turn/interrupt` requests
and 33/23/35 `turn/completed` records whose status is `interrupted`.

Their grace decisions are respectively 31+7, 15+12 and 14+26 ordinary-usage plus
resumed-output-usage releases, with no timeout. An independent raw-payload check finds
advancing attributable usage after the directive for all 105 released interrupts.
Thus “let responses finish” describes those **usage-completed model responses before
the retained interrupt**, not natural outer-turn completion or disabled interruption.
The old accounting field alone is not proof of response completeness; the saved native
workflow reconciliation and these payload checks are separate evidence.

### Supported package result, not individual identification

The published exact means are 35,659,265 raw tokens for compact Direct and 19,311,710
for sequential WL; all three WL results are below all three Direct results. These are
the original whole-group observations, not a reselected quality subset. The report
records fewer cumulative-usage changes (311 versus 198 per workflow, described there
as model generations), concentrated in implementation/review, alongside more uncached
tokens for WL. Those older usage-change counts are not modern exact response-ID ledgers.
Native command counts also need not equal model generations: one response can request
several tools, and native calls can perform reads, writes or polling.

This supports a substantial **connected orchestration-package** difference in the
collected samples. Source chain: launch policy → emitted edit/write directive →
orchestrator application/commit → ACK/check/done → owned review/fix/recheck. It does
not individually identify cohesive work packaging, structured edit syntax, validation
scope/cardinality, write mediation, ownership, review-context delivery, evidence-only
resolution or original-request resupply. These C10–C24 factors remain coupled in that
comparison, along with residual native permission/tool and auxiliary-context differences.
The later single-cue screens cannot be replaced by a claim that this package already
identified every component. No uncached saving, exact mechanism share or formal
quality equivalence is inferred.

## Broad candidate dispositions

| Candidate | Existing exposure and supported direction | Remaining limit |
| --- | --- | --- |
| C25 interruption | 252 normal interruptions follow measured current-response usage; 8 actual continued responses and one timeout are retained. The saved all-three timing-policy contrast moves raw tokens upward. | Not every interruption saves its current response; 120-s waiting/history/downstream effects are part of this contrast. No fixed suffix price or additive share. |
| C26 read route | Exact direct-read WL observations preserve a large package advantage without ordinary mediated reads; actual source policy/direct-read activation is recorded. | The saved direct-read-minus-normal interval crosses zero. It changes policy/access/read packaging and continuation opportunities together, not only one file representation or generic tool definitions. |
| C28 compact targets | Three actual Direct plan prompts deliver exact target lists; whole-payload native joins pass. The saved compact-handoff sample mean is slightly lower. | Small difference relative to run variation; expected sign unresolved. It neither proves a major saving nor removes other linearizer-context differences. |
| C29 scheduling | Three actual sequential-WL runs pass prior-feature terminal gates, while source preserves the normal feature/review sequence. | The saved scheduling sample mean is slightly higher, not evidence that concurrency materially caused the endpoint gap. No precision or population sign claim. |
| C10–C24 package | Three versus three exact main workflows retain a large raw-token difference and different implementation/review activity. | Coupled policy/protocol package, not component-wise identification or causal allocation. Preserve all outcomes and test narrower factors only under separate authority. |

The direct-read evidence's bounded interval crosses zero, as does the combined
read-plus-continuation transition. The saved combined evidence reports a negative
interaction across its missing-usage interval. These are reasons **not to add** separate
read and timing sample differences as independent effects. The package remains
informative without asserting the old bridge's descriptive percentages as causal shares.

## Provenance and verification boundary

110 unique referenced files passed direct SHA checks: declared evidence dependencies,
analysis/report payloads, generator and source drivers, runtime binaries, six generated
drivers, activation plans/logs and 69 native source files. Continuation raw client/server
hashes additionally match their original terminal capture metadata; each complete
forwarded client stream equals the original bytes. Normal-H raw/grace hashes match
all six accepted `capture_bound_audits`. Source inspection uses actual Git blobs and
the retained clean source checkout, not current implementation as historical proof.

| Source | SHA-256 |
| --- | --- |
| Continued design | `e56dcbc34d6c8bd3923653e8f541aff2a54092f5c118ab765d4f381bec57d908` |
| Continued result | `560d4dc252dd02c2367d3f504d866e783fbed689c6f29068b4fc8ae67c7abe6c` |
| Continued evidence | `9549ccb59dab003b4d091d997462264168a792886abb3e5cac9dd79664ff20f8` |
| Continued manifest | `40b62c6ac567033eb6585cdf9475574bc4ddd2173d5fd9f178da9fc1f22bef79` |
| Continued launcher | `42bfeedb886163908a63c618f7661f260f2c5edaf09c81d5b54cb2cdfab16f45` |
| Main final report | `491eb2f6c4c81e44ede0f6f161ec0b10b16fe79d62451674d710c1fba900e53d` |
| Main evidence | `aeb9a2d569d2552e01de13dd9a8679dfd5a3bb0dffc7cf8b3c9345b7ae579c22` |
| Main infrastructure manifest | `3e64a0274b560356a02c5f3c5c66a2bed5035e33611889abd0f0525a895bb31b` |
| Diagnostic generator | `eab5a8dd441b6815c2aa645b7d26b317b3c11678d190d0048b786bd3d90198c8` |
| Observer lib.rs at 8eb26a4 | `6281adfec446552d7c08951a12dd654cb54a270013b0f844720e688e34532a11` |

The main manifest's binary hashes, all independently verified, are observer
`000ba67baf4a8af56e140be0fd7f2342f8f93b6b5bdd8c434026db45249d1e0f`, WL
`1958080ce5dda6852e947e3639a5edfda1fe12a403b5d65de7ca733c377299e6`, orchestrator
`e1b74e1e2519fea07015c8b35037b224ebfa06d57d8da70e6225e4a289c37620`.
Generated sequential-WL driver: `88bc245b871343ae162ec34fe6bfade3528f6f4e6daeb3ab0a430060b489c936`;
compact-Direct driver: `7d177e3c5798d2c321b761bbdc2c7270a01408c101e7938f6e277c1cd1eb08f9`.

Continuation raw client/server/decision SHA triplets:

- 001: `95f4f07d93111c093c1a304fab062292bfed5c83385f9817b78355a6f8c70f2d` /
  `7749e13e3ffb4810aa15c10b68e1bc21f8d386b205acb47f11a4248bde6639cb` /
  `5dbedb3ee868ad5598dbb1b5680db0536b22c38b4acae29deea1da68a1c4853e`.
- 002: `b91fe4a42ccf3a7bfaaa46e261bd2f43dfbef709e437174cc5381d54680af7c0` /
  `0d74683b9272e461831124b0571d88dba42fbc6981a68f6c9abc1ae90414319e` /
  `50ce80915d8f0f41f607b78d59c9071dfda2f5db34c4726fd6906535441ccadc`.
- 003: `6af51eb9e08cc797b7a36fc5f2f377810917f6a11175da1c33ac817f348202d9` /
  `0502941805fcc000af0358fb1df8161f32418a12be1719eda6a30d6ce6aab451` /
  `124f5493ea6642bf1e9fcb5b20cbc123e5a19cfe4a19bbfee46caaf572fced3a`.

This descriptive source/payload audit changes no code, frozen report, measurement
ceiling or admission. No private reasoning bodies, current candidate costs or parked
large-read outcomes were inspected. No provider workflow is affected or launched.
