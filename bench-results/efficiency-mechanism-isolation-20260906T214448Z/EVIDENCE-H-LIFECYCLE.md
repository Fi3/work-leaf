# Historical endpoint lifecycle and action evidence

Scope: the accepted H population in [evidence.json](../efficiency-exact-normal-work-leaf-20260829T181318Z/evidence.json): six normal-WL and six Direct workflows, with original identities and outcomes retained. This provider-free Stage B audit covers C09, C24–C30, C32–C33 and C36–C38. It does not pool H with P/S/W, inspect parked R costs, recompute token totals, require new controls or change any source/helper.

## Main dispositions

- **Actual extra WL input/work:** every one of 48 WL launch threads receives the root instruction body both through native AGENTS loading and inside WL's launch policy; all six workflows generate three titles in one persistent title thread. These are offsetting costs, not explanations for WL saving.
- **Actual narrower WL follow-up packaging:** all 21 Direct fix prompts repeat the original feature request; normal WL's 26 fix prompts use the findings/evidence continuation without a separate original-request section. Both systems reuse author/reviewer sessions. The appended-request difference is real; its net token effect remains the separately isolated C24 hypothesis.
- **Actual clean-review detour:** nine of H's 26 WL fix requests route an already-clean aggregate to the author, which replies only done, followed by reviewer-only NO_FINDINGS. This extends the observed branch mismatch to H; no response-cost subtraction is made.
- **Cohort-specific exclusions:** no native compaction in the six H WL workflows; only two compacted records in point7-exact-direct. No accepted WL inter-agent send, command-agent launch, explicit compact request, ownership-block continuation, command-classification continuation, protocol-correction continuation or extra linearizer completion-request turn occurs.
- **Still unresolved:** interruption's behavioral saving, permission/tool-definition effects, scheduling/conflict attribution and integration-work relocation. Source existence, waiting time, command count and outer-turn count do not establish net saving.

The native exec-versus-write_stdin polling investigation is root-owned and separate from this audit. No function-call-count contrast is used here.

## Identity, source and version boundary

The [H infrastructure manifest](../efficiency-exact-normal-work-leaf-20260829T181318Z/infrastructure/manifest.json) pins normal WL commit `5b1d1ef9590850faed26052f909ddff7ff8f127d`; [score-manifest.json](../efficiency-exact-normal-work-leaf-20260829T181318Z/score-manifest.json) resolves its six artifact directories. All six client/server SHA pairs match `evidence.accounting.capture_bound_audits`.

The map's source-equality qualification was independently replayed: `src/{agent,cli,codex,review,linearize,workspace,instructions,locks,orchestrator}.rs`, `bench-three-features-direct-common` and `bench-agent-profile-common` are byte-identical between that H commit, the retained H source checkout and [P's saved source](../efficiency-raw-token-pilot-20260906T185328Z/source) at `41b6418ef420fbce6aab93706657bd77dba3ce51`. Links to P source below therefore describe those H normal-WL source bytes.

That qualification does **not** make every earlier Direct driver identical. The six Direct reports name four older driver commits; the split common driver does not exist at that path in those revisions. Actual Direct claims here instead use each retained prompt, role/thread file, invocation, exec JSONL and native rollout. The two indirect observation sources resolve through [points8-9 evidence](../efficiency-points8-9-20260828T145556Z/evidence.json), selecting only `direct-003` and `direct-002`, never its WL variants.

All H normal-WL native sessions use CLI **0.150.1**. The first three Direct observations use **0.149.1**; the three step4 Direct observations use **0.150.1**. All inspected actual target turn contexts are gpt-5.5/xhigh. This is not P/S/W's CLI 0.153.4, and no newer response-metadata capability is retroactively assumed.

### Delivery audit

All **355** WL turn/start requests have a unique typed successful reply, nonempty exact thread/turn identities and a unique completed public user item with identical text. Extra public `text_elements: []` metadata is checked separately, not mistaken for a content mismatch. Every accepted input also matches one native user item by explicit thread/turn and full text. Each of the 48 native sessions matches its recorded cwd/thread/CLI and saved source SHA.

All **90** Direct prompt files match a unique native user message within the role's exact saved thread ID, and that thread agrees with the captured exec stream's thread.started record. All 90 captured provider turns complete successfully. In CLI 0.149.1, `content_item_kinds` is absent: the join uses exact role-thread identity, full user text and explicit native turn metadata, not an inferred message-kind label or prefix. Native and public item IDs are not assumed interchangeable.

These are accepted outer model turns, **not completed-response counts**: native tool interactions can entail additional model responses inside an outer turn. Direct's repeated CLI processes include intentional resumes, not automatically restarts or retries. Nested repository test commands also appear in observer invocation logs and are not provider workflows.

## Whole-population lifecycle census

WL rows follow run-ID order. Every row has 8 sessions: three authors, three reviewers, one title session, one linearizer. “Author / review / title / linear” counts accepted outer turns by the owned launch role; all counts retain failures and repairs. “Fix / clean” means all accepted fix requests / the clean-marker subset.

| WL run | Accepted turns: author / review / title / linear | Fix / clean | Edit-rejected: changed-refresh / same-snapshot | Interrupts: exact-usage / resumed-output / timeout |
| --- | --- | --- | --- | --- |
| exact-normal-001 | 49 / 13 / 3 / 2 = 67 | 5 / 2 | 2 / 5 | 53 / 2 / 0 |
| exact-normal-002 | 33 / 12 / 3 / 2 = 50 | 4 / 2 | 2 / 2 | 34 / 5 / 0 |
| exact-normal-003 | 43 / 10 / 3 / 2 = 58 | 3 / 1 | 2 / 1 | 26 / 21 / 1 |
| exact-normal-004 | 27 / 10 / 3 / 2 = 42 | 3 / 2 | 1 / 0 | 32 / 0 / 0 |
| exact-normal-005 | 46 / 14 / 3 / 2 = 65 | 5 / 1 | 4 / 2 | 49 / 4 / 0 |
| exact-normal-006 | 55 / 13 / 3 / 2 = 73 | 6 / 1 | 3 / 4 | 58 / 2 / 0 |

Direct rows retain accepted-evidence order. Each has seven native sessions: three authors, three reviewers and one linearizer. The named title/command-agent roles do not occur. The step4-direct-001 outer-launcher failure after report publication remains an original outcome note; successful child provider turns do not erase it.

| Direct run | Provider turns: implement / fix / review / linear | Intentional resume invocations | CLI | Native compacted records |
| --- | --- | ---: | --- | ---: |
| point7-exact-direct | 3 / 5 / 8 / 2 = 18 | 11 | 0.149.1 | 2 |
| direct-003 | 3 / 3 / 6 / 2 = 14 | 7 | 0.149.1 | 0 |
| direct-002 | 3 / 3 / 6 / 2 = 14 | 7 | 0.149.1 | 0 |
| step4-direct-001 | 3 / 3 / 6 / 2 = 14 | 7 | 0.150.1 | 0 |
| step4-direct-002 | 3 / 4 / 7 / 2 = 16 | 9 | 0.150.1 | 0 |
| step4-direct-003 | 3 / 3 / 6 / 2 = 14 | 7 | 0.150.1 | 0 |

## Exact exposed mechanisms and limits

### C09: native and WL instruction duplication

[`PromptPolicy::inject`](../efficiency-raw-token-pilot-20260906T185328Z/source/src/agent.rs#L293) appends the concurrency translation and complete loaded root instruction files; [`load_project_instructions`](../efficiency-raw-token-pilot-20260906T185328Z/source/src/instructions.rs#L11) owns that file selection. Every H WL first launch contains the same retained AGENTS body, **10,937 UTF-8 bytes excluding its trailing newline(s)**, SHA-256 `ca7e12ef03c75037e15b7aa655908889e4e3328a585c3648eca797247e2fab51`. The same exact body already occurs in a separate native instruction user item at physical line 4 of each session, including title and linearizer sessions.

For example, H001 author user-1 starts at C4 / native N7, thread `01a04ebc-2a9e-7662-8f06-5d0cfe7c08fd`; its separate instruction item is N4, ID `msg_01a04ebc-396f-7fe0-bfd8-479675418d81`, in [the exact native source](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T20-15-49-01a04ebc-2a9e-7662-8f06-5d0cfe7c08fd.jsonl#L4). This is actual duplicate delivery, not a guess that Codex might load AGENTS.

Direct has one native AGENTS item per session (42 total), and none of its 90 driver prompt files copies the full root guide. Known-session resumes in either workflow do not create a fresh WL policy/native AGENTS copy on every follow-up in this cohort. Native developer/skills updates are separate items; identical complete provider input is not asserted.

The source boundary supports a possible instruction-deduplication study, but H has no exact per-item token-attribution metadata. Byte duplication is established; its retained-input charges and net workflow effect are not quantified here. Removing the translation rather than only duplicated text would change a different factor.

### C24: exact request resupply versus shared session reuse

Direct's retained `sequential-feature-N-fix-M.prompt.txt` contains an explicit `Request N:` block. All 21 blocks equal the feature text from that run's initial implementation prompt, and all 39 review prompts likewise repeat that original request within their review scope. Direct also appends actual prior author evidence to rechecks. WL's [review-fix/recheck source](../efficiency-raw-token-pilot-20260906T185328Z/source/src/cli.rs#L1260) carries findings and author evidence but does not add a separate original feature-request section to its 26 fix prompts.

A directly linked example is [step4-direct-001 fix1](../efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/runs/sequential-feature-1-fix-1.prompt.txt): the 189-byte original visual request is repeated before actual reviewer findings and focused-check guidance. Its corresponding [review2 prompt](../efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/runs/sequential-feature-1-review-2.prompt.txt) repeats the request and supplies the author's actual fix/check evidence. Direct therefore does not lack focused checks or evidence-aware review.

Both systems reuse the same original author/reviewer threads. WL H001 C40 and C42 resume author `01a04ebc-3891-7172-bbb2-3bbdab8c9de7` and reviewer `01a04ec0-b298-7e90-ad37-3dfc626bc56c`; their native input locators are N190 and N80 respectively. Source ownership of raw known-session followups is `CodexBackend::send_streaming[_interruptible]`; launch-only injection is distinct. Session reuse itself is shared, while repeated request packaging is an observed differential input mechanism. The v4 C24 factor is a separately admitted test of that narrower contract, not permission to alter review scope or sessions.

### C25–C27: interruption, permissions and native compaction

All six WL primary captures record a 1,000-ms configured grace. The actual decisions are the table above, not an assumption that each interrupt waited the full second. Only H003 has a timeout; resumed-output release occurs in five runs. Captured request methods contain one initialization, eight thread/start calls, the listed turn starts and interrupts; no thread/compact/start or thread/resume occurs. Native session reuse does not require a thread/resume RPC inside the same app-server.

The [streaming directive boundary](../efficiency-raw-token-pilot-20260906T185328Z/source/src/codex.rs#L277) returns the accumulated public reply after requesting interruption. These records establish the interruption/grace action, not how many tokens a hypothetical delayed interrupt would add or remove. No timing change is part of this audit, and observed usage publication is not equated with new generation.

Actual native permissions match the transport declarations: WL authors/reviewers/title read-only, Direct authors workspace-write and reviewers read-only, both linearizers danger-full-access, approval never. Native tool operations exist despite policy-mediated WL project reads; allowed read-only tools and mediated writes are distinct. Complete tool-definition sets/charges are not available from H's outer request text. H contains no `rawResponse/completed` capture records or native `token_usage_record` events. Its cumulative/last-response accounting is left unchanged; newer P/S/W exact response-ID attribution is not manufactured for H.

All 48 WL native files lack compacted/compaction events. Two `compacted` records occur in point7-exact-direct: [author N810](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T14-05-32-01a04842-cbbf-7952-97d1-98fc8afc784a.jsonl#L810), thread `01a04842-cbbf-7952-97d1-98fc8afc784a`, and [reviewer N447](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T14-22-30-01a04852-53e3-7352-baeb-43d24b1e5000.jsonl#L447), thread `01a04852-53e3-7352-baeb-43d24b1e5000`. The other five Direct native populations have none. This is native compaction exposure, not proof of its trigger or a response-cost correction. Compaction cannot explain WL savings through an exercised WL compactor in H.

### C28/C38: linearization and explicit completion

Each H workflow has exactly one linearizer session and exactly two admitted outer turns: plan, then acceptance/execution. WL's actual target context contains reviewed commit/scope information under `LinearizePlanner::interactive_prompt`; Direct's saved plan/acceptance prompts also require final history, documentation and checks. Both use direct filesystem access.

H001 WL's exact chain is C130/S63717 (plan accepted), C131/S64665 (execution accepted), final message S67960 including `@work-leaf done`, thread `01a04ed8-c2dc-7d00-9299-b143d400ade1`. The native user inputs are [N7 and N83](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T20-47-03-01a04ed8-c2dc-7d00-9299-b143d400ade1.jsonl#L83). Its public execution messages retain actual docs/history work and repeated validation repair, so “two turns” does not mean two generated responses or no integration repair.

No WL extra completion-request prompt occurs in H; every final linearizer turn contains its own done directive and reaches captured completion. The generic completion-correction mechanism is therefore not exposed through an additional linearizer turn in this cohort. The nine reviewer detours below are a different completion-only path.

### C29/C32/C36/C37: coordination, rejection and retry

The accepted client corpus partitions completely into the named categories: 98 file-read responses, 58 command results, 53 successful patch ACKs, 28 edit-rejection continuations, 26 fixes, 26 rechecks, 48 policy launches, 12 title followups and six plan acceptances. These categories total 355; none is dropped as “other.”

Exactly 14 edit-rejection prompts deliver a changed-file refresh and 14 state that the touched files still match the agent's snapshot. These are actual recovery branches; a changed snapshot alone does not identify which agent changed it or prove a cross-agent conflict. H001 C26/S2654, thread `01a04ebc-3891-7172-bbb2-3bbdab8c9de7`, is the same-snapshot branch; the author then explains overly loose edit context at S2712 and submits a tighter edit at S3606. This is format/context repair, not demonstrated concurrent interference.

No accepted ownership-block, failure-masking command rejection, command classification, protocol correction or `Message from <agent>` send continuation occurs. No command-agent or unexpected provider session appears in the captured role inventory. The ordinary concurrency/lock implementation remains active; absent specific conflict messages do not prove zero shared-state effects.

Each WL workflow uses one captured primary app-server invocation with eight established threads and no RPC error reply. Direct's provider invocations all complete successfully; the 48 resume invocations are deliberate same-thread fix/recheck/linearize continuation, not evidence of retrying failed generation. Hidden provider-internal retries cannot be excluded by these public logs, and unsupported retry counts are not inferred.

### C30: title generation is an actual auxiliary offset

Each WL workflow has one title session and three completed accepted turns; Direct's seven-session topology has no title role. H001's title inputs are C36/C56/C96, all on `01a04ec0-b27c-7912-9618-f020a72c88e8`; C36 is a full policy launch, and C56/C96 are raw title followups. The first actual title is `slash-command-agent-routing` at S3886, ID `msg_053d7ff1c47c81a4016a932304c43c87d2bc1e72bf6b4c4920`. [Native N7](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T20-20-46-01a04ec0-b27c-7912-9618-f020a72c88e8.jsonl#L7) retains the exact launch.

`WorkLeafController::start_title_worker → CommandChat::generate_chat_title → run_system_agent_turn` is therefore exercised, not merely source-available. All 18 title turns belong to H's observed workflow; they are neither discarded hidden work nor a token-saving cause. Their exact standalone response costs are not invented from title word counts.

## Nine historical clean-marker chains

The complete 26-fix census has nine clean-only aggregates and seventeen FINDINGS-bearing aggregates, with no absent/conflicting marker class. All nine clean aggregates reconstruct from complete ordered public reviewer messages. Every following author turn contains only `@work-leaf done`; the next exact same-reviewer recheck contains only `NO_FINDINGS`, with no other public action. This is an H-only descriptive extension of [the W closed replay](EVIDENCE-CLEAN-MARKER-REPLAY.md), not a transfer of its 32 response IDs or charges.

C/S locators resolve through the corresponding pinned H capture below.

| H run | Fix / recheck C | Original clean marker item at S | Author done S → reviewer clean S |
| --- | --- | --- | --- |
| exact-normal-001 | 40 / 42 | `msg_02c327ae4a82adba016a932384c35c87d29af231fd82d25f4b` at 11648 | 11988 → 12287 |
| exact-normal-001 | 126 / 128 | `msg_00cbf714282e9f30016a93291a49d087d29e8e429a9cf59e39` at 63676 | 63692 → 63709 |
| exact-normal-002 | 32 / 34 | `msg_07e29c7dd8dc6f9e016a93238f2bac87d293651a2ae858944c` at 14983 | 15371 → 15943 |
| exact-normal-002 | 93 / 95 | `msg_049c9db5bf9238cb016a93287fd75887d295a2a8778f2cc844` at 58181 | 58197 → 58214 |
| exact-normal-003 | 50 / 52 | `msg_070c0b8c5a758307016a9324677a3c87d28b4c832433fb1759` at 23193 | 23216 → 23233 |
| exact-normal-004 | 38 / 40 | `msg_0db97db96baabb41016a932e8bffa487d2adef5c9d9b6f69b0` at 5109 | 5466 → 5847 |
| exact-normal-004 | 70 / 72 | `msg_0af475e9d1d24949016a93310d3b5487d2880e94e59ce9368a` at 23965 | 23983 → 24013 |
| exact-normal-005 | 59 / 61 | `msg_04ef81f2ac7f49fa016a9330ac90d887d2a08fc257cca0d2e9` at 38387 | 38817 → 39287 |
| exact-normal-006 | 131 / 133 | `msg_0ec0b261bdb448c0016a9333d1b7cc87d2bb30c3a8da61f6a0` at 62247 | 62511 → 62821 |

The source recognition mismatch is `review.rs::has_no_findings` applied to `CodexAppServer`'s complete public-message aggregate. These clean pairs supply no newly generated verification evidence. They should not be labeled successful evidence-only resolution. The conservative parser-safety caveat in the W replay remains applicable; this audit implements no replacement parser.

## Provenance and retained limits

All 135 sources retained in the lifecycle provenance projection were SHA-rechecked at the endpoint: manifest/report/raw/grace/metadata/native sources. The supplemental Direct exact-prompt/exec joins and H clean chains also rehashed their inputs after replay. Native files are resolved only from the corresponding saved rollout-metadata entry and must equal its source SHA; no broad native-session discovery or private reasoning-body inspection is used.

| Authority | SHA-256 |
| --- | --- |
| [efficiency-exact-normal-work-leaf-20260829T181318Z/evidence.json](../efficiency-exact-normal-work-leaf-20260829T181318Z/evidence.json) | `4fc87b42b1a5f4a12f74195c6a16754e46e6a6f2605de32cdc79002d361ee246` |
| [efficiency-exact-normal-work-leaf-20260829T181318Z/score-manifest.json](../efficiency-exact-normal-work-leaf-20260829T181318Z/score-manifest.json) | `f46ec7dfa267d50fa9904a5a9a50ae782011209b56dae5cc6f79badcd603b452` |
| [efficiency-points8-9-20260828T145556Z/evidence.json](../efficiency-points8-9-20260828T145556Z/evidence.json) | `87b826fd951dbe11486d2e9dc0925e11ca1c145ce9411fa24f0e3383d4f0a236` |
| [H infrastructure manifest](../efficiency-exact-normal-work-leaf-20260829T181318Z/infrastructure/manifest.json) | `121c4ed265e967e517aaa14f1b6c71a123ddc3a5ca6c5ebd8d84e5a47cfaaa03` |

The following exact report/metadata links resolve each actual historical workflow, including the two indirect evidence observations. Metadata supplies the complete matched native source path/SHA inventory. Client/server pins for all six WL rows additionally reside in the accepted H evidence's `capture_bound_audits`; this audit revalidated all twelve raw hashes.

| Run | Actual report SHA-256 / source | Native metadata SHA-256 / source |
| --- | --- | --- |
| exact-normal-001 | [report](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-001/exact-normal-001-three-feature-bench-artifacts/report.json) `f7697ded0326681b039027bbc83eae92551fcb0edcfc20c800844858495c6694` | [metadata](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-001/exact-normal-001-three-feature-bench-artifacts/observation/rollout-metadata.jsonl) `16f913d0e8ebf5aea1811fd5764dd63d0e534000632173282b47e3ddb679f379` |
| exact-normal-002 | [report](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-002/exact-normal-002-three-feature-bench-artifacts/report.json) `70e80066156aab07c0ef64fc2ec4daecbacf8f20445044f37c445bac78e28e1e` | [metadata](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-002/exact-normal-002-three-feature-bench-artifacts/observation/rollout-metadata.jsonl) `435cf27c98279d0baacda553764d8d87c95e6b2acbd5a2996f041864c50cb49e` |
| exact-normal-003 | [report](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-003/exact-normal-003-three-feature-bench-artifacts/report.json) `2204c623092127c272aefafa2d3ae01f4a3c6640be325bbcd614e5e478a4f30e` | [metadata](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-003/exact-normal-003-three-feature-bench-artifacts/observation/rollout-metadata.jsonl) `805848bead73a46b10be95a092c4aa8127465e639fa4af272b47548c40542845` |
| exact-normal-004 | [report](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-004/exact-normal-004-three-feature-bench-artifacts/report.json) `062fa443d5241276a812adc405ec6802060ab9cca1602344115d56d428a176c1` | [metadata](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-004/exact-normal-004-three-feature-bench-artifacts/observation/rollout-metadata.jsonl) `15f7083dc489eea75df5c7e294946aa985c614eeb4df63ad246d0ec4f5df0cff` |
| exact-normal-005 | [report](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-005/exact-normal-005-three-feature-bench-artifacts/report.json) `60171a9173a99bc27756a0d48de5a33e054cc71cc929d3d298cefec3b6b391e1` | [metadata](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-005/exact-normal-005-three-feature-bench-artifacts/observation/rollout-metadata.jsonl) `481400398bd7122a322be03e38e46e972e19ef9889cabc25b3e1f0a2614f5113` |
| exact-normal-006 | [report](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-006/exact-normal-006-three-feature-bench-artifacts/report.json) `a1b24490b68b7bd74c1e43cc50f7fdeabbf5ec9a76ad1e518afb269b20f7fbca` | [metadata](../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-006/exact-normal-006-three-feature-bench-artifacts/observation/rollout-metadata.jsonl) `7d00c37dbebdb8e25290d047b9ef881c8eb063c97944df2f171e6be1cdc9c49e` |
| point7-exact-direct | [report](../efficiency-point7-exact-accounting-20260828T113610Z/runs/direct/point7-exact-direct-three-feature-sequential-bench-artifacts/report.json) `99075f4699c8ed01f36c794d607a685381e05daf0f6c35b158ac06e47eba8f61` | [metadata](../efficiency-point7-exact-accounting-20260828T113610Z/runs/direct/point7-exact-direct-three-feature-sequential-bench-artifacts/observation/rollout-metadata.jsonl) `4f60d3e8b856b1ff80be2104a9321a39aeccd8703fb602893d46eb41f4f418fe` |
| direct-003 | [report](../efficiency-points8-9-20260828T145556Z/runs/direct-003/points89-direct-003-three-feature-sequential-bench-artifacts/report.json) `01ebb52993811f16649c36fbb4efb418e8e66aaad898a55c27e7ed70be117b31` | [metadata](../efficiency-points8-9-20260828T145556Z/runs/direct-003/points89-direct-003-three-feature-sequential-bench-artifacts/observation/rollout-metadata.jsonl) `687bb2fa49c807dd92ae9be350f1ebeb762b0e16657191ce9a5ae644ac81326c` |
| direct-002 | [report](../efficiency-points8-9-20260828T145556Z/runs/direct-002/points89-direct-002-three-feature-sequential-bench-artifacts/report.json) `ae5d803eeaf49252ad65c6d3be0f5916b3ca148354e89187f446a811a6f0a66a` | [metadata](../efficiency-points8-9-20260828T145556Z/runs/direct-002/points89-direct-002-three-feature-sequential-bench-artifacts/observation/rollout-metadata.jsonl) `23057794f65f822155ab3e7a15deedb237ea1793fe954291bcabc8cb098467a1` |
| step4-direct-001 | [report](../efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/report.json) `976fa6dfa18e1ff9d355b6affa0c5c38438dbf1446fdad3fa564d60af5138eb8` | [metadata](../efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/observation/rollout-metadata.jsonl) `17b58543fec95e35214926a9a5d0952f69094e3541bdecf1605684787e79e7b0` |
| step4-direct-002 | [report](../efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-002/corrected-study-step4-direct-002-three-feature-sequential-bench-artifacts/report.json) `048e9a4905b3937f38e48899dbfea03ef6db3563f2d21dd63ceb714fada95c61` | [metadata](../efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-002/corrected-study-step4-direct-002-three-feature-sequential-bench-artifacts/observation/rollout-metadata.jsonl) `3a93254e9248731914d6f89c4669a3500021044c72880a85b58b78ec45f4568d` |
| step4-direct-003 | [report](../efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-003/corrected-study-step4-direct-003-three-feature-sequential-bench-artifacts/report.json) `0db7c97ba40998923f6aab4a8ba9f8572fc5e69bc85579d79b64a7f14e21ce62` | [metadata](../efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-003/corrected-study-step4-direct-003-three-feature-sequential-bench-artifacts/observation/rollout-metadata.jsonl) `1ef92ce87014eb04e64098607f26ffff447acf73de6e77d78035c4b18242c131` |

No quality inclusion filter, missing-usage zero-fill, old ceiling revision, provider admission or new baseline requirement is part of this artifact. The older CLI/source scope and incomplete fine-grained accounting are limits on attribution, not reasons to discard the accepted historical endpoint. The concrete next checks are the separately isolated request-resupply factor, instruction/title offsets if later selected, and root's independent command-wait/poll action chains; no new controls are a gate.
