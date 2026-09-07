# Candidate v4 real diagnostic review

This provider-free review covers only the three explicitly admitted subscription diagnostics in [the admission](CANDIDATE-DIAGNOSTIC-ADMISSION.json), SHA-256 `aae70eca25eaaa3fcfceddedfc9da71d500dec2ad38f6a21dcbd88ec37e86c82`, admitted 2026-09-07 11:55:05 UTC. It admits no replacements or benchmarks and makes no token-saving or percentage claim. Original terminal receipts, failed logs and original analysis errors remain evidence.

## Outcome and scope

| Diagnostic | Original harness outcome | Actual accepted turns / threads | Independent delivery evidence | Usage coverage |
| --- | --- | --- | --- | --- |
| [C02 repeat](candidate-v4-repeat-01/TERMINAL.json) | Exit 0, 16.99 seconds | 3 / 1 | Launch contract, first ordinary bundle response, exact current-text repeat, done | Three exact native/raw response IDs; corrected archive-scope analysis complete |
| [C16 format](candidate-v4-format-01/TERMINAL.json) | Exit 0, 10.12 seconds | 2 / 1 | Preferred unified-diff policy, actual patch acceptance, unchanged ACK, done | No native/raw response-usage records; two interrupted usage gaps |
| [C24 follow-up](candidate-v4-followup-01/TERMINAL.json) | **Exit 101**, 25.99 seconds | 6 / 2 | Actual initial edit/ACK/done, reviewer FINDINGS, request-resupplied author fix/ACK/done, reviewer NO_FINDINGS; final fixture value 2 | Two exact reviewer response IDs; four author turns have no native/raw usage records |

Durations above are the original test log's elapsed times, not the later receipt-collection clock. The turn counts are accepted outer provider turns, **not inferred model-response counts**. Runtime delivery is evidenced separately from token-accounting completeness. C24's original failed harness result is not relabeled successful by an offline review.

All eleven accepted requests have exact matching original/forwarded `turn/start` frames, one typed successful RPC reply, one completed public `userMessage`, and an exact model-native user item. Native `turn_context` records independently specify GPT-5.5, xhigh, approval never and read-only sandbox for every turn; session metadata records CLI 0.153.4 and the exact observed cwd/thread. No native function/tool/action response item occurs in these four native session files: only messages and reasoning-item metadata. Public item-start/completion types are userMessage, agentMessage and reasoning. Reasoning bodies are neither inspected nor reproduced. No response-count equality is inferred merely from this item inventory.

## Exact delivery and native input identities

C/S are physical lines of the diagnostic's original client/server raw stream; forwarded turn/start frames have the same C line and exact bytes. N is a physical line in the exact native session source listed below. Public user-item IDs and native user-item IDs are distinct namespaces: equality is not invented. The native join uses the session thread ID and `internal_chat_message_metadata_passthrough.turn_id`, with `content_item_kinds=["user.text"]`, then verifies the entire single input-text body against the accepted request. Generic turn-aborted messages are not request inputs.

| Scenario/turn | Client RPC / successful server reply | Exact provider turn ID | Public user item (server line) | Native user item (native line; context line) |
| --- | --- | --- | --- | --- |
| repeat/1 | C4, string `3` / S6 | `01a07bb8-d4ca-7770-8db2-5a015a1118c8` | `01a07bb8-e158-7b61-a2ab-76ef8c9c7af8` (S11) | `msg_01a07bb8-e157-72e1-b270-8ee06f6f347b` (N7; context N6) |
| repeat/2 | C6, string `5` / S27 | `01a07bb8-e97f-77f3-b574-2b04e89269ec` | `01a07bb8-e9a3-7a82-9dee-68b9ce48e598` (S31) | `msg_01a07bb8-e9a2-75d3-9b6f-36b0eb80aa54` (N20; context N19) |
| repeat/3 | C8, string `7` / S49 | `01a07bb8-fb18-79e2-bcbd-8d59f000e528` | `01a07bb8-fb3d-77f3-8c9c-2cc9947dc500` (S53) | `msg_01a07bb8-fb3b-7430-bb50-e0f965b913e7` (N31; context N30) |
| format/1 | C4, string `3` / S6 | `01a07bb8-d932-7791-9ee7-568ecadca0ae` | `01a07bb8-dff2-7af2-96a8-c3e79723550c` (S11) | `msg_01a07bb8-dff1-76b0-aaf0-fb92474c5a83` (N7; context N6) |
| format/2 | C6, string `5` / S89 | `01a07bb8-f5a9-7f33-8f1c-d0180c289e68` | `01a07bb8-f5c8-7c90-9dc0-425ae9e53e42` (S93) | `msg_01a07bb8-f5c8-7c90-9dc0-424fdd284576` (N21; context N20) |
| followup/1 | C4, string `3` / S6 | `01a07bb8-ddd5-7bf2-b88f-b6c3b0e21cc9` | `01a07bb8-e741-7c20-921d-f3929893f7c1` (S11) | `msg_01a07bb8-e741-7c20-921d-f38f76372d5e` (N7; context N6) |
| followup/2 | C6, string `5` / S74 | `01a07bb8-f351-74f2-98cf-64c8fbcf65aa` | `01a07bb8-f386-7550-9273-ccc5df74581e` (S78) | `msg_01a07bb8-f385-7c32-9795-871469d68858` (N19; context N18) |
| followup/3 | C9, string `8` / S95 | `01a07bb8-fc6b-70b1-9513-44fec58dd412` | `01a07bb9-02d5-71d0-a0e7-a41c7f0d6072` (S100) | `msg_01a07bb9-02d4-7322-bd41-84287da5f0ab` (N7; context N6) |
| followup/4 | C10, string `9` / S141 | `01a07bb9-1d53-70d1-89ba-edd291f473b0` | `01a07bb9-1d74-7263-a5fc-4edce129dbef` (S145) | `msg_01a07bb9-1d73-7623-8efe-eeec0a2ceb7b` (N29; context N28) |
| followup/5 | C12, string `11` / S207 | `01a07bb9-28f9-7ba0-9a7e-17e7a0e673ee` | `01a07bb9-2915-75f1-a159-e5d4252800f4` (S211) | `msg_01a07bb9-2914-7231-827c-6965e403de1f` (N39; context N38) |
| followup/6 | C14, string `13` / S222 | `01a07bb9-34db-7173-a29e-3183a0de1a22` | `01a07bb9-350b-7bc3-86ed-8a8e65f4e79c` (S229) | `msg_01a07bb9-350a-71e3-bdb7-03ee9dc83252` (N19; context N18) |

The repeat's original first-read prompt is 422 bytes and remains identical; its unchanged-repeat prompt is 257 baseline bytes versus 16,970 candidate bytes, containing the exact 16,890-byte held fixture snapshot at UTF-8 range `[80,16970)`. The normal untracked bundle allocation remains present. This demonstrates C02 requested-repeat delivery, not exposure to the parked untracked-read-inline factor. Trace records are [repeat lines 2–4](candidate-v4-repeat-01/prompt-events.jsonl).

C16's [policy record and ACK](candidate-v4-format-01/prompt-events.jsonl) deliver the preferred valid-unified-diff instruction including `@work-leaf end`. Actual patch item `msg_0a6e4ce63496c8d4016a9ea620d92c87d28e2bfefe42cd6cd5` completes at S83, followed by ACK request C6 / accepted S89; done item completes at S101. The ACK's complete original/forwarded bytes are identical. This scripted format exercise proves acceptance/delivery, not an unconstrained model preference or its token effect.

C24 [trace line 5](candidate-v4-followup-01/prompt-events.jsonl) supplies the original 1,433-byte launch request at candidate range `[743,2176)`, adding the truthful 52-byte header for a total 1,485-byte insertion. Everything before that insertion matches the baseline fix prompt. The exact request is native author input N29 and public input S145, accepted as C10 / S141. Both initial policies and both ACKs are identity boundaries. The real reviewer emits FINDINGS at S135, the author submits the corrected edit at S201, and the reviewer emits NO_FINDINGS at S236.

## Retained C24 verifier failure and offline correction

The launched harness SHA was `1c56c54987aada7c92f2824bd4830476541c8821479cfe61eff531a965167ab6`. Its final trace verifier required every selected text to occur exactly once among all requests. C24's two genuine ACKs have identical bytes because both successful edits target the same file. Thus each ACK matches **C6 and C12**; the original [run log](candidate-v4-followup-01/run.log) fails with `left: 2, right: 1`. This is not a rejected patch, extra call, failed reviewer fix, or ambiguous typed turn identity. The two ACKs have distinct accepted turns, distinct public user IDs and distinct native user IDs as listed above.

The separately versioned smoke verifier retains a RED regression reproducing the original global-text uniqueness rule. Its correction establishes each agent's thread from the exact owned policy delivery and consumes indexed `(thread, selected-text)` queues in occurrence order, rejecting repeated use, wrong-thread matching and order reversal. The exact planned site census and complete instrumented-request index sequence are also required. C24's only uninstrumented turn is its final reviewer recheck. A separate provider-free replay entrypoint checks the already-closed captures without constructing a backend or writing artifacts; the original FAILED receipt remains unchanged. The replay helper is not an authorization for another provider run.

Root executed the provider-free C24 replay successfully at 12:10 UTC, followed by successful repeat/format replays, at helper SHA `d5b4e004f5690de93b16b17aa4d7dc71efddec990f688942b613e0d57171713c`. A subsequent closed-tail regression reproduced that the live-poll decoder ignored a non-newline final fragment. The strict closed-reader version, SHA `f04b2eeac0128871ee0dd351ff36eb821bfc0128d1512b8f7dea2318e7bfc4f7`, requires complete final frames for original/forwarded/server/trace files while preserving permissive live polling. Root's final provider-free replays at those exact bytes all pass: requested-repeat-full 3 turns, unified-diff-preferred 2 turns, and review-fix-request-resupply 6 turns; each reports `1 passed; 9 filtered out`. No precise final replay execution timestamp was retained. The eight automatic smoke tests, required formatting, all-target/all-feature clippy and all-target/all-feature tests pass at the final source. Neither verifier correction alters the already generated requests, observed usage, original admission, or original FAILED receipt.

Both C24 threads reach matching terminal notifications: author final turn at S227 and reviewer final turn at S241. The final reviewer reply can be accepted at S222 before the author's S227 terminal notification, so checking only the globally last turn would be insufficient. The revised verifier requires each thread's latest accepted turn to terminate and any interrupt to be forwarded byte-for-byte and acknowledged by its exact typed RPC ID.

## Missing usage and extraction classification

Every capture start records primary=true, raw response usage=true, grace=1000 ms, output-resume policy=forward, and required project-layer inventory. The read diagnostic waits 21/68/10 ms for exact usage before its three interrupts. Format's two interrupts instead release at 0/0 ms on resumed output; C24 author's four interrupts release at 0/0/0/1 ms for the same reason. This is the existing `forward` policy, not a new grace change. The relevant exact decisions are retained in each capture's `provider-usage-grace.jsonl`.

For format, native `token_count.info` is null at N13/N25 and no `token_usage_record` exists. For C24 author, it is null at N11/N23/N33/N43 and no `token_usage_record` exists. No later exact native usage was found for either author. Those missing counters remain unknown, not zero, and this review does not invent finite ceilings or retotal their workflows.

The original extractor labels format author `01a07bb8-d8c8-7220-b106-64015e26b762` and C24 author `01a07bb8-dd63-73a3-8e50-2850d6def1dd` as cwd-sharing threads absent from process capture. Both are actually present in the typed raw capture and exact native metadata, not extra threads. Source chain: `bench-observer/src/lib.rs` builds `AnalysisSummary.threads` from `final_thread_observations(&observations)` (around 4393), and `extract_rollout_metadata` builds its expected set from that summary (around 7335). A thread with no usage observation falls out of that expected set and reaches the unobserved-cwd branch (around 7420). This explains the classification discrepancy without clearing the original extraction flags or restoring missing token data.

| Scenario | Exact response ID | Raw / native source line | Input | Cached input | Output | Reasoning output |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| repeat | `resp_0d24ec2ccc3ed4ae016a9ea61d72ec87d280768d34f3a251c3` | S21 / N11 | 14419 | 9600 | 11 | 0 |
| repeat | `resp_0d24ec2ccc3ed4ae016a9ea622213087d283084136de68325b` | S43 / N24 | 15898 | 13696 | 13 | 0 |
| repeat | `resp_0d24ec2ccc3ed4ae016a9ea625dea887d2ba07be5d6418ab22` | S63 / N37 | 21985 | 9600 | 160 | 149 |
| followup reviewer | `resp_017219a8c23f1e0d016a9ea6260e5487d2a41113f4ecd7fb3a` | S136 / N13 | 15739 | 9600 | 286 | 249 |
| followup reviewer | `resp_017219a8c23f1e0d016a9ea6332e8487d29e0cd2e938133947` | S237 / N25 | 16021 | 14720 | 73 | 64 |

All five response identities match native and app-server raw records by exact thread, turn and response ID; four counters are equal, nonnegative integers with cached/reasoning subsets contained in their parents and total=input+output. No summation here is a three-diagnostic token comparison, and the six usage-less author turns are not dropped from accounting.

## Postcapture archive error and evidence identity

The first observer analysis was polluted by an operator archive request over the project root instead of the actual bundle directory, attempting to interpret ordinary fixture/Git files as bundles. Each diagnostic retains `OPERATOR-ARCHIVE-ERROR.json`, its original `analysis.json` / `analysis-post-rollout.json`, and the recoverably quarantined erroneous archive described by the operator record. The separate corrected-archive-scope report clears these collection errors for repeat only; format and followup still retain their extraction and interrupted-usage gaps. This review does not suppress the original errors or modify captures, invocation receipts, native sessions, admission, or frozen historical reports.

| Diagnostic | Raw capture directory | Original client SHA-256 | Forwarded client SHA-256 | Original server SHA-256 |
| --- | --- | --- | --- | --- |
| repeat | [00000482167382942143-600957](candidate-v4-repeat-01/observation/app-server/00000482167382942143-600957) | `54ad699959110c15538720c14ff8d7d24afbdc6c5a85aed35965e7fca4018492` | `ec2e123ce1f192b4db6c487722f9810abd5f694ce33033acf7514ea1d19128ae` | `bd9472bb216ad2476d457dc3e54a954e2d7e890c0736703ba187cc1d2dc248c0` |
| format | [00000482168552413531-601296](candidate-v4-format-01/observation/app-server/00000482168552413531-601296) | `b4879bc19c0cdef52e730fec456528770cb88691c9775d7e9781f39a63a6e4ea` | `ae23ce0ec41e910f1e493b7a3ddf8f2b94c14bb4da021e60a61c1de29c15dc32` | `0ad4681679405eaa2316b379ed2c3d90480dd98d4e5f2adaa49ba59d938e7fda` |
| followup | [00000482169718276691-601648](candidate-v4-followup-01/observation/app-server/00000482169718276691-601648) | `225367ddcf8ada77a167c24b2bf0589d2524fae00baad26dfa2e2d1864651936` | `09e94b3ca1d4c7d1b0d86826c8f4c1ed350a8f40b439df9ba375c721924209e7` | `1098af066c814bb718ecdc97a39c30572f483e373fd856eb8ad0f3336ad446a2` |

| Native session source (exact observed thread) | SHA-256 |
| --- | --- |
| [rollout-2026-09-07T13-55-05-01a07bb8-d45b-7033-8c5a-720e19579bea.jsonl](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T13-55-05-01a07bb8-d45b-7033-8c5a-720e19579bea.jsonl) | `a1a71a8676692245fc0197d737946919b0e9a94e5338cb347d91f773fd43a874` |
| [rollout-2026-09-07T13-55-06-01a07bb8-d8c8-7220-b106-64015e26b762.jsonl](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T13-55-06-01a07bb8-d8c8-7220-b106-64015e26b762.jsonl) | `5d17880093d62273c1632506fb2b4c451e2f498173741cc4bc20388fcad3beec` |
| [rollout-2026-09-07T13-55-08-01a07bb8-dd63-73a3-8e50-2850d6def1dd.jsonl](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T13-55-08-01a07bb8-dd63-73a3-8e50-2850d6def1dd.jsonl) | `4b0dc97db8320ca46a315ce904c3dfe2dddefd7f72fc83e683c27a79c9a3a554` |
| [rollout-2026-09-07T13-55-15-01a07bb8-fc28-7692-8347-dc74d75d1cd3.jsonl](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T13-55-15-01a07bb8-fc28-7692-8347-dc74d75d1cd3.jsonl) | `58f1d1550a20051ac6edbd922a7e047196692accf683543d6285d575e654088f` |

Only the previously observed four exact thread IDs were used to resolve these native paths. Each native file's session ID/cwd/CLI identity was checked before its public items and usage metadata were inspected. All admitted runtime source hashes remain equal; the separately corrected test/replay helper is a prospective verifier change, not changed generated behavior. All 18 selected original diagnostic files and all four native session files pass the final endpoint SHA recheck. No provider calls, retries, new controls, parked-R cost analysis or frozen accounting edits are part of this work.
