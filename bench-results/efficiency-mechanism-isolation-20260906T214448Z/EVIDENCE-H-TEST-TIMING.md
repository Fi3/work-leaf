# Historical H test-timing exposure

This provider-free C15 audit retains the accepted six normal-WL and six Direct observations in [H evidence](../efficiency-exact-normal-work-leaf-20260829T181318Z/evidence.json). It inspects public actions and returned test output, not token costs, private reasoning, parked R, quality scores or new provider runs. No control or intervention is admitted.

## Finding and limits

**H exhibits a real test-timing difference.** Each of the 18 Direct author initial feature sequences publishes a new test before its first accepted production-code edit. Fourteen have a demonstrated new behavioral-test failure before that edit; four visual-mode sequences instead stop compiling because a newly proposed feature API is absent. The latter are a separately identified compiler-contract RED, not executed behavioral assertions and not merely syntax errors.

WL's 58 delivered mediated command results all follow an accepted implementation patch in the same author thread. Seventeen of the 18 initial author patch groups contain new tests and production changes together. The remaining visual group, H002 S54889/C65, has a context-only test-file section with **zero added/deleted test lines**; new visual tests arrive later with a review repair. No separately accepted new-test-only group preceding its corresponding implementation was identified among the 53 ACK groups. Later failing checks and test repairs remain visible and are not relabeled as initial tests-first RED.

This establishes an exercised action/continuation difference, not a causal share of the historical saving. Direct's pre-implementation checks create actual public failure-result boundaries followed by model-authored implementation. WL omits that particular published-RED boundary in these traces, but adds mediated edit/ACK/command continuations and still performs repair. Fewer of one kind of action does not by itself prove fewer total responses or net tokens. The checks are largely focused in both systems; [validation breadth](DESIGN-VALIDATION-CADENCE.md) is a separate factor.

## Audit boundary

Population/role and exact public/native delivery identities use [EVIDENCE-H-LIFECYCLE.md](EVIDENCE-H-LIFECYCLE.md). Direct role files identify all 18 author threads; public `apply_patch` input/output pairs establish accepted writes, and `exec_command` output establishes the actually reached tests/status. Rejected patch attempts, invalid Cargo invocations, missing fixture helpers and already-existing tests are not treated as successful new-test RED. `src/*.rs` can contain test-only inline additions, so pathname alone does not establish production implementation.

The census reads every initial Direct feature action sequence through first production edit and its earlier test outputs; it is not a classification of every later Direct review repair. Across all 53 WL groups, complete edit receipts and changed production/test sections were inspected for timing. The 58 command handoffs are joined to the exact preceding same-thread public locked-command directive and accepted request/reply, without an invented ACK-to-commit join. A public native-command scan in the 18 WL author sessions found no additional native Cargo/rustc test execution; this is not a claim about hidden provider internals.

Locators below are physical JSONL/raw lines. `N` identifies the exact native source listed in the final table; `C/S` identify the H workflow's client/server capture. An arrow in the Direct table means tool call → its exact tool output, **not** two provider turns or response IDs. In the WL tables it means public edit → accepted ACK request → typed reply, or public command → accepted result request → typed reply. These old H sessions have no exact `rawResponse/completed` / native `token_usage_record` response IDs; no newer token attribution is manufactured.

## All 18 Direct initial author sequences

Rows retain H's accepted-evidence order, never cost order. Features 1/2/3 are visual selection, slash routing and completion respectively. The “production” column is the first accepted production edit, not a claim that it already completes the feature or passes all checks.

| Run / feature | Accepted new-test input→output N | Demonstrated RED call→output N | First production input→output N | Disposition |
| --- | --- | --- | --- | --- |
| point7-exact-direct / 1 | 138→140 | 144→146 | 159→161 | API compile: missing `clipboard_text`. No visual assertion reached. |
| point7-exact-direct / 2 | 124→126;130→132 | 138→141;139→143 | 149→151 | Behavior: prompt slash routing/reply absent. |
| point7-exact-direct / 3 | 140→142;144→146 | 160→163;161→165 | 188→190 | Behavior: completion prompt/author selection absent. |
| direct-003 / 1 | 140→142;144→146 | 150→153;151→155 | 161→163 | API compile: missing `visual_mode` / `copied_text`; N144 is a test-only source-file addition. |
| direct-003 / 2 | 136→138;168→170 | 174→176 | 202→204 | Behavior: valid harness command remains in Command mode. Excludes the terminal fixture's missing Esc. |
| direct-003 / 3 | 187→189;193→195;201→203 | 209→213;210→215 | 223→225 | Behavior: missing completion prompt/wrong selection. Separate harness missing-API compile failure at211→217. |
| direct-002 / 1 | 120→122;174→176;180→182 | 188→190;196→198 | 208→210 | Behavior: new visual tests cannot find the expected OSC52 output. |
| direct-002 / 2 | 115→117;119→121 | 127→129;135→137 | 179→181 | Behavior: new prompt route remains in Command mode. |
| direct-002 / 3 | 145→147;153→155 | 167→169;173→175 | 189→191 | Behavior: new terminal/harness tests fail. Existing workspace test's new assertion at161→163 is not the sole witness. |
| step4-direct-001 / 1 | 115→117 | 121→123 | 148→150 | Behavior: new visual tests fail because visual mode is absent. |
| step4-direct-001 / 2 | 203→205 | 211→214;212→216 | 233→235 | Behavior: new terminal/harness route/reply absent. |
| step4-direct-001 / 3 | 206→208;212→214 | 220→223;221→225 | 231→233 | Behavior: new completion prompt/author selection absent. |
| step4-direct-002 / 1 | 116→118 | 124→126 | 160→162 | API compile: missing `UiHarness::clipboard_text`. |
| step4-direct-002 / 2 | 342→344;346→348;350→352;354→356 | 362→367;363→370;364→371;365→373 | 379→381 | Behavior: new CLI/workspace/terminal/harness routing absent; CLI unwrap fails on actual unknown-command result. |
| step4-direct-002 / 3 | 126→128;130→132 | 138→141;139→143 | 186→188 | Behavior: new completion prompt absent. |
| step4-direct-003 / 1 | 134→136 | 142→144 | 155→157 | API compile: missing `last_yank`. |
| step4-direct-003 / 2 | 114→116 | 120→122 | 128→130 | Behavior: new controller test records no `/status` send. |
| step4-direct-003 / 3 | 227→229;233→235 | 240→245;265→267 | 279→281 | Behavior: workspace prompt and corrected terminal fixture fail before code. Missing fixture helper at241→247 is excluded. |

Important counterexamples to mechanical failure counting:

- point7 feature3 N152→154 passes two Cargo filters incorrectly; exit1 is invocation failure, not RED. N160→163 and161→165 are the actual new-test failures.
- direct-003 feature2's first terminal test lacks the Esc needed after creating the chat. N243→245 corrects that fixture. The independently valid harness test N168→170 and its actual failure N174→176 are the initial behavioral witness; existing slash tests N96→98 already pass.
- direct-002 feature3 N161→163 extends an existing workspace test; its distinct newly added terminal/harness tests supply the new-test witnesses.
- step4-direct-003 feature3 N241→247 cannot compile because the newly added fixture calls a missing `git` helper. Fixture repair N259→261 precedes the real terminal behavioral failure N265→267, still before production N279→281.
- Direct also writes later tests alongside implementation or after earlier code; the initial-sequence finding does not mean every test in every thread was observed RED first.

## Complete WL ACK and command population

For each H run the capture directory is `../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-NNN/exact-normal-NNN-three-feature-bench-artifacts/observation/app-server/<capture-id>/`. Files are `client-to-server.raw` and `server-to-client.raw`. Every listed ACK has one preceding complete structured edit in its own accepted author turn. Every listed command result follows at least one prior same-author ACK; commands from another concurrent author are not used to establish order.

| H run | All accepted edit S→ACK C→reply S | All locked-command S→result C→reply S (status) |
| --- | --- | --- |
| exact-normal-001 | 3606→30→3612; 19783→46→19789; 19996→52→20002; 30642→67→30648; 55661→84→55670; 58186→90→58192; 61264→102→61270; 63217→111→63223; 63525→119→63531 | 3653→32→3659 (0); 19837→48→19843 (1); 19870→50→19876 (101); 20027→54→20033 (0); 30682→69→30688 (0); 56216→86→56452 (101); 58230→92→58236 (1); 58325→94→58420 (0); 61399→104→61467 (0); 63264→113→63270 (1); 63290→115→63296 (101); 63550→121→63556 (0) |
| exact-normal-002 | 3192→22→3198; 34076→42→34082; 35907→46→35913; 37313→50→37321; 54889→65→54895; 55578→73→55584; 57956→86→57962 | 3230→24→3236 (0); 34816→44→34959 (101); 36066→48→36087 (101); 37496→52→37516 (0); 54918→67→54927 (0); 55632→75→55638 (1); 55706→79→55713 (0); 57984→88→57990 (0) |
| exact-normal-003 | 5190→28→5196; 5305→32→5311; 28177→54→28186; 37718→60→37727; 38395→65→38401; 38998→74→39004; 39275→80→39281; 40845→93→40852; 41562→99→41569; 42262→108→42268 | 5217→30→5223 (101); 5330→34→5336 (0); 28989→56→29065 (101); 38022→62→38135 (0); 38440→70→38461 (101); 39040→76→39046 (101); 39311→82→39317 (0); 40887→95→40898 (101); 41602→101→41609 (0); 42298→110→42304 (0) |
| exact-normal-004 | 3478→24→3484; 3681→28→3687; 14403→42→14412; 22906→58→22912; 26078→74→26084 | 3513→26→3519 (101); 3716→30→3722 (0); 14809→44→14911 (0); 22938→60→22944 (0); 26122→76→26128 (0) |
| exact-normal-005 | 3683→26→3689; 30544→44→30553; 31488→50→31496; 43457→63→43466; 44910→69→44916; 60929→86→60935; 65971→92→65977; 66140→96→66146; 68245→109→68251; 69664→120→69670 | 3717→28→3723 (0); 30817→46→30941 (101); 31665→52→31793 (0); 44040→65→44129 (101); 44940→71→44946 (0); 61444→88→61523 (0); 65998→94→66004 (101); 66178→98→66184 (0); 68283→111→68289 (0); 69702→122→69708 (0) |
| exact-normal-006 | 3434→28→3440; 17189→44→17199; 19206→50→19215; 21033→54→21044; 22126→58→22138; 56332→83→56339; 57308→86→57314; 58133→95→58139; 58486→103→58492; 60092→116→60098; 61782→126→61788; 63326→135→63332 | 4266→30→4674 (0); 17550→46→17639 (0); 19876→52→20090 (101); 21160→56→21255 (101); 22243→62→22249 (0); 57043→87→57319 (101); 57340→89→57347 (101); 58160→97→58166 (0); 58565→105→58571 (1); 58609→107→58615 (0); 60120→118→60130 (0); 61824→128→61830 (0); 63353→137→63359 (0) |

The initial cohesive groups are H001 C30/C46/C84; H002 C22/C42 (C65 is the exception); H003 C28/C54/C65; H004 C24/C42/C58; H005 C26/C63/C92; H006 C28/C50/C83. This is a body/timing classification, not a quality score or an assurance that submitted patches were buildable. The 53/58 inventory retains wrong assertions, actual source defects, compile failures and later repairs.

Examples of non-RED failures: H003 C30 fails because the freshly submitted workspace test calls `.unwrap()` on a unit-returning method; production was already delivered in C28. H004 C26 passes the new positive slash route but fails its new negative test's display assertion; C28 repairs only that assertion. H001 C48/C92/C113, H002 C75 and H006 C105 are invalid multi-filter Cargo invocations, not executed RED tests. A command containing several `&&` checks does not establish execution of checks after the first failure.

## Three exact action chains

### A. Genuine Direct RED, followed by implementation and GREEN

In step4-direct-003 feature2, native thread `01a04de5-49d9-72f1-a87d-af9f50224ac8`, initial outer turn `01a04de5-4a31-7c10-9b59-7c05a115f2c7`:

1. N114 adds only `controller_routes_slash_command_lines_to_the_selected_agent` to `tests/workspace.rs`. Accepted result N116 belongs to call `call_IWhX3r6KURvJOSmGMIn2LLML`; input item `ctc_015e87a8dcf773b3016a92eb3d9c8c87d2bcc74091726d4795`.
2. N120 runs `cargo test --test workspace controller_routes_slash_command_lines_to_the_selected_agent`, call `call_OzLT1X6fALrWjI0E1P3siJzt`, output item `fco_01a04de7-0e48-7f52-b566-2df82fe9a008` at N122. It actually executes one test and fails: no send versus expected `(user-1, /status)`. No prior production edit supplies that route.
3. N128/N136 implement selected-agent routing/helper logic in `src/workspace.rs` (accepted N130/N138). N144 repeats the exact focused command; N146 passes one test, output `fco_01a04de7-901a-7011-b861-b8809439e151`.
4. The author then separately adds terminal/harness tests N169/N185, sees their actual failures N179/N193, and implements those adapters N208/N216. This is a real additional test-first increment inside one outer author turn, not evidence that it was one provider response.

### B. Cohesive WL delivery still has an assertion-repair loop

H004 slash author thread `01a04ee8-0c81-7b72-a923-f14e71ee8c69` emits S3478, item `msg_0c9aa7c21f7029aa016a932de1bb4087d2b462ecbfdd79ec59`: terminal/harness routing plus four new positive/negative tests. ACK C24 (RPC23) is accepted at S3484 as turn `01a04eeb-a67c-7c23-863b-f80c9c3d9274`. S3513 requests the focused terminal/harness suite, and C26/S3519 delivers status101 as turn `01a04eeb-f08c-7602-8cef-162fd1c33144`.

The positive terminal slash test passes; the negative test incorrectly expects the controller error in the selected-agent frame. S3681 changes that test to inspect the controller transcript; C28/S3687 accepts it. S3716 requests the same suite; C30/S3722 returns status0, followed by S3733 done. The failing-output→repair continuation exists, but it is **after implementation**, not deliberate pre-implementation RED. Its presence prevents “cohesive patches eliminate repair” from being inferred.

### C. The initial WL visual-test omission is retained

H002 visual author thread `01a04ebc-2c3d-7741-8e87-8e1d30282cad` emits S54889, item `msg_0073345b0eb7bcfb016a9325b75bc887d29c28dbea3dd6cc26`, text SHA-256 `5d1e40092fd476d3ffb400c1809610c08c58aa02fbd4e1e15ed7e78a2c5cd870`. The `tests/ui_harness.rs` section contains only unchanged context for an existing mouse-wheel test. It is not a new visual test and not an adaptation. C65/S54895 accepts the receipt, S54918 requests the full harness suite, C67/S54927 returns31 passing tests, and S55037 says done.

Later S57956, item `msg_0073345b0eb7bcfb016a9327dd09a487d28a7243195b312559`, changes visual routing/escape handling and adds four visual tests together. C86/S57962 accepts it; S57984 requests the harness suite; C88/S57990 returns35 passes and S58001 says done. This is implementation-first then cohesive review repair, not tests-first RED. No omission is removed from the population or converted into a quality exclusion.

## C15 isolation and shared-tree safety

The actual H launch source is `PromptPolicy::for_read_permission` at retained `src/agent.rs:267` and `concurrent_instruction_translation` at :364. Both buildable-test sentences occur exactly once in **all18 accepted author launch inputs**, C4/C6/C8 of each WL capture. The full root AGENTS body still includes both literal RED-first requirements; the explicit WL translation changes their ownership/timing application. This is a delivered source-level mechanism, not a condition-name inference.

Current corresponding source is `src/agent.rs:346–349` and :514–524. Existing renderer-owned private ranges `policy-buildable-work-unit` and `instruction-tests-work-unit` provide narrow textual seams, but the frozen W treatment changed incremental/cohesive **buildable work-unit preference**, not mandatory executed failing tests. Its failed primary gate does not establish C15 absent or refute this observed test-ordering difference.

A true “execute the new failing test before production implementation” intervention cannot simply replace these sentences while claiming all other normal-WL rules unchanged. The shared-tree policy forbids knowingly failing intermediate patches; manual test writes must use the edit protocol, and write-producing commands are mediated. Publishing a deliberate failing-test-only patch would contradict the preserved safety clause and expose other authors/reviewers to that state. Removing safety/ownership restrictions is a different factor, not a pure timing ablation.

A scratch/transactional test preview or private checkout could execute RED without publishing it, but requires an explicit implementation surface, test-body transport, snapshot/lock semantics and extra work. Those additions are not an existing one-line C15 knob. A wording-only test-design cue is safer but does not ensure executed RED and must not be described as such. No source implementation or fresh run is justified by this audit alone. A prospective study would first need an independently reviewed safe boundary and actual ordering exposure; no additional control is a gate here.

The supported mechanism candidate is therefore specific: **normal WL's buildable shared-tree translation is associated with skipping a publicly executed pre-implementation new-test failure stage, while Direct actually exercises it**. The strength is exact ordering evidence across the accepted population, not an isolated causal estimate or a claim that test correctness/coverage are equivalent.

## Reproducible source identities

All 18 Direct native files, 12 WL raw streams and 18 WL author native files were rehashed at audit end: 48 sources, no mismatch against their previously recorded source identities. All 84 call/output pairs in the Direct census were independently checked for exact nonempty call identity and the stated accepted-write status0 or failed-check status101. The source inventory and exact native role resolution remain in [EVIDENCE-H-LIFECYCLE.md](EVIDENCE-H-LIFECYCLE.md); the Direct native files below make each N locator directly recoverable. Public text/tool payloads were inspected; private reasoning bodies were not used. These source files contain pre-existing logs, not new benchmark observations.

| Direct run / feature | Exact native source / SHA-256 |
| --- | --- |
| point7-exact-direct / 1 | [01a04842-cbbf-7952-97d1-98fc8afc784a](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T14-05-32-01a04842-cbbf-7952-97d1-98fc8afc784a.jsonl:1) — `81d8532466ee761bc4f6a35cef95851b85eb14512eaca5578b0a8346e4539358` |
| point7-exact-direct / 2 | [01a04875-77e0-7f80-90af-3806070505ad](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T15-00-53-01a04875-77e0-7f80-90af-3806070505ad.jsonl:1) — `ab78e672321c49aed7695cfa1aab778c0803d0825e5e10590aef912c3a2f0f72` |
| point7-exact-direct / 3 | [01a0487f-0996-7c23-8a6d-0f9221b8907f](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T15-11-20-01a0487f-0996-7c23-8a6d-0f9221b8907f.jsonl:1) — `cba2438b5b5628d084f7b95ff7cd5372586d2666451e11c5fd4a9af430c7ec7d` |
| direct-003 / 1 | [01a0494e-baf3-7ff3-b586-c33247ada806](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T18-58-11-01a0494e-baf3-7ff3-b586-c33247ada806.jsonl:1) — `23ae078f807f81ec24ec3d263f7e4863280a9915c5d989f67462fc1c69558b45` |
| direct-003 / 2 | [01a04967-43d2-7e50-b40f-b1640464e5d7](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T19-24-59-01a04967-43d2-7e50-b40f-b1640464e5d7.jsonl:1) — `2141a3860ed4f400e468fda3ccd56a3f0bd53191209eace4d86f453bb6616f62` |
| direct-003 / 3 | [01a04970-dc4a-7840-9d16-eafd8b0f6254](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T19-35-28-01a04970-dc4a-7840-9d16-eafd8b0f6254.jsonl:1) — `eb51da97759feefc4d9b5d1e001b1acc36799e2d003f3aa9f667b96276264c3b` |
| direct-002 / 1 | [01a04990-b481-7702-b8b3-0ee09945eb44](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T20-10-15-01a04990-b481-7702-b8b3-0ee09945eb44.jsonl:1) — `d61f6577a67a6bf0f92912c06ec1e9c6efec563378cd76f2562d0d64a241020b` |
| direct-002 / 2 | [01a049ab-e9de-7591-8b86-9c3e318cd179](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T20-39-58-01a049ab-e9de-7591-8b86-9c3e318cd179.jsonl:1) — `cb40b4af3fe88117220d0e4f816cbec13c87e7cdd51ca9f8b29664f2baf93a23` |
| direct-002 / 3 | [01a049b8-1424-7e01-9df3-c2cd7624f4bc](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T20-53-15-01a049b8-1424-7e01-9df3-c2cd7624f4bc.jsonl:1) — `b667ae4fa14365a6954d5de751c8160769e53431f028090fe802aeb4ad8ce848` |
| step4-direct-001 / 1 | [01a04d33-45af-7411-a12a-111fe0f03105](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T13-06-40-01a04d33-45af-7411-a12a-111fe0f03105.jsonl:1) — `8d2b2df855c997392d6e696c8e23c9b2f4853559d530e3e3a4f296c0549947bc` |
| step4-direct-001 / 2 | [01a04d47-fb77-77a3-99b8-0f57b5363c0c](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T13-29-18-01a04d47-fb77-77a3-99b8-0f57b5363c0c.jsonl:1) — `c3a863d62e16b460447fb1e5bec79415eb077c72748a9126e387f5c75d1203bf` |
| step4-direct-001 / 3 | [01a04d59-55ea-7ee3-a052-5f7b85458611](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T13-48-15-01a04d59-55ea-7ee3-a052-5f7b85458611.jsonl:1) — `40e56c693d10eacb7bf8a0cc4401d20b78dc6d760e639fffa517730c72dad628` |
| step4-direct-002 / 1 | [01a04d7b-f0bf-7431-9df3-34c50a105afc](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T14-26-03-01a04d7b-f0bf-7431-9df3-34c50a105afc.jsonl:1) — `82e8751306f3eeed450b4237235a7f1bf29c2c963168613d3bda2d5df996c054` |
| step4-direct-002 / 2 | [01a04d98-18eb-75a2-bd33-28cc248f79e6](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T14-56-48-01a04d98-18eb-75a2-bd33-28cc248f79e6.jsonl:1) — `0d3e7ea1c709106d9a91caa05cfbb4b4e564d65d51312d7cfd0e4adcfbb3cdca` |
| step4-direct-002 / 3 | [01a04da9-279b-7802-9a1b-020b3ca56a47](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T15-15-26-01a04da9-279b-7802-9a1b-020b3ca56a47.jsonl:1) — `979621cf26a312fa8ce5eca8df893e5cd0987cce6bdb3ad462f71800764fb8d5` |
| step4-direct-003 / 1 | [01a04dcf-deef-7d92-b055-901622f6ffe1](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T15-57-43-01a04dcf-deef-7d92-b055-901622f6ffe1.jsonl:1) — `927b08099024acfcb2b5c4dc131228159ea73e97ef3305d84439dab02690d2ab` |
| step4-direct-003 / 2 | [01a04de5-49d9-72f1-a87d-af9f50224ac8](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T16-21-07-01a04de5-49d9-72f1-a87d-af9f50224ac8.jsonl:1) — `f06ac070bb68fbaf3c11c137d1548193bf1c59840418a43b5dfb6d293ecd4217` |
| step4-direct-003 / 3 | [01a04df7-cd85-78a3-acd1-2dfeb365fcea](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T16-41-20-01a04df7-cd85-78a3-acd1-2dfeb365fcea.jsonl:1) — `67fe88ad49d8477cd4c9be38d6ea71ada4bc26ace2ee6fbd3bf45ed30b766b5e` |

| WL run | Capture ID | Client SHA-256 | Server SHA-256 |
| --- | --- | --- | --- |
| exact-normal-001 | [00002676642578624406-641](/home/user/src/work-leaf/bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-001/exact-normal-001-three-feature-bench-artifacts/observation/app-server/00002676642578624406-641/client-to-server.raw:1) | `90d9e5a624d9f04bb2c7809cb73d1af18b787a0cd2d270718c64c659bf906e54` | `42379ffc5bd1c929738a69982f70f1439ed20a3c2774a751d9d1b29fbbb7d339` |
| exact-normal-002 | [00002676642578623219-639](/home/user/src/work-leaf/bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-002/exact-normal-002-three-feature-bench-artifacts/observation/app-server/00002676642578623219-639/client-to-server.raw:1) | `3bb9f3541b8f5d89ddf5ea15dfcd9de817a432f6b9aa9d54ed7e0d21e8861d42` | `670459b291165a08996e8db5943a0be3b801d1d198d4fd065650ec725528154f` |
| exact-normal-003 | [00002676642578620844-642](/home/user/src/work-leaf/bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-003/exact-normal-003-three-feature-bench-artifacts/observation/app-server/00002676642578620844-642/client-to-server.raw:1) | `784dc35bb6bd618896304125ee891448b396a0c3dd37c0731bee32cbe0e55c31` | `d7c9e8b7f35ff152e56c0ebe25240ec5de6f95ce0440667b7a5fd5cda0a689ba` |
| exact-normal-004 | [00002679513471323065-646](/home/user/src/work-leaf/bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-004/exact-normal-004-three-feature-bench-artifacts/observation/app-server/00002679513471323065-646/client-to-server.raw:1) | `598ce84b96e05232df0257f8b7304746c8d1aaa219d64d264c4ea1412e50b61a` | `353021d33cb79748afa810f1d128cd09bf18ed93d13f7b583e0f0e8347053eed` |
| exact-normal-005 | [00002679513471141406-639](/home/user/src/work-leaf/bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-005/exact-normal-005-three-feature-bench-artifacts/observation/app-server/00002679513471141406-639/client-to-server.raw:1) | `8c444063cad8ad04e1f2be9224266e7947660621380c32ad08dc6c70ad418bdd` | `c35711c5398eff5bff47bc3c763db5813ae4ae18606854f6d8870927d1cdcac1` |
| exact-normal-006 | [00002679513471141406-640](/home/user/src/work-leaf/bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-006/exact-normal-006-three-feature-bench-artifacts/observation/app-server/00002679513471141406-640/client-to-server.raw:1) | `e2b4ed1812205251473e8e012da25e1fd329c10d39dfe492f02f184cb369b113` | `e0e473c2b3c548dbfafd0abad88abe71466e7c3138bc4f329889cf53d12bd454` |

Pinned historical source `5b1d1ef9590850faed26052f909ddff7ff8f127d`, retained `/home/user/.codex/work-leaf-exact-normal-source-20260829T181318Z/src/agent.rs` SHA-256 `dd4464f94d887b30ad603f331743d18e69936d0f898c74383595c34f43d4cb74`; its exact normal source equality qualification is in the lifecycle note. Current `src/agent.rs` SHA-256 `a9e6065a450d05bf299202a2d6f44dd2dae33a3324e8c70e9f91d36a22b242fd`. Current architecture ownership is `docs/architecture.md:416`, SHA-256 `87f1a62d991991635a758f3efcc6925253eb07800acdf8ca594319ef9e0d24dd`; no architecture or public API change is proposed. C10 companion SHA-256 `4723e0abfa11ebfd60578e6980146e15d768ee31105c11f11610b95e87854b20`.

This document changes no agent-facing workflow. No Rust/runtime verification or provider generation is needed for the read-only evidence report; no frozen report, helper, protocol, source or original outcome was modified.
