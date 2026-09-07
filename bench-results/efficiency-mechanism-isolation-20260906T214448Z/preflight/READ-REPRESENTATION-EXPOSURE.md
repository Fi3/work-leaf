# Read-representation exposure

This is a read-only descriptive audit of the historical raw-token pilot and the closed work-unit phase. It changes no run, frozen helper, accounting total, provider setting or protocol. All historical rows and all twelve current workflows, including the recorded workflow failure, remain present. This is not a treatment contrast or a causal share of the historical WL-versus-Direct savings.

## Scope and source identities

The historical attribution report is `preflight/HISTORICAL-INPUT-ATTRIBUTION.json`, SHA-256 `586533b10c4ffa94b9b2dbaf5ebf91762f8ea103b500238ae6b6e1265f0100c6`. The current report is `phases/work-units-01/INPUT-ATTRIBUTION.json`, SHA-256 `ed171a20cec3017a75d84219ce81f759725e86b95f3494daedc27868cecf92cd`. Paths in this note are relative to this study unless absolute.

Every native source and raw client/server capture used in this extraction was independently rehashed against the relevant report's `/runs/i/source_sha256` inventory: 135 distinct source files. The SHA-256 of the combined path-to-hash map, serialized with sorted keys and compact JSON separators, is `547a36e0e1d34faacba5e4c840753c445d0bc5869813c9a6ed2d5aabab393ae6`. These source maps retain the complete physical locations; this note does not copy native bodies or credentials.

“Untracked” means absent from the existing per-agent file-read tracker, not necessarily the first chronological read of a path. Successful own edits/patches can clear tracked paths. The source chain is `src/orchestrator.rs::send_file_read_response` → `split_repeated_file_reads` → `render_file_read_response` / `render_file_read_response_with_repeats`. The untracked bucket alone controls ordinary bundling: over 16 KiB for any single file, or over 24 KiB summed UTF-8 text (`should_bundle_file_read_response`). Changed repeats use a diff and unchanged repeats use digest references; automatic conflict refresh is a separate path.

Accepted read responses are original `turn/start` requests containing the renderer's exact file-text prefix, joined to a successful typed RPC reply and its nonempty accepted turn ID. Untracked manifests, inline snapshots, changed and unchanged repeat sections are classified from the captured renderer-owned representation. Request text plus thread identity must uniquely match a native public user-message item; its exact `(thread_id,item_id)` joins the existing attribution ledger. A text match is only used to recover that identity, never to estimate a token price. Ambiguous identities remain unassigned.

## Observed exposure

Historical Direct has no captured app-server read requests and its attribution status is unknown: `no completed raw response attribution; coverage unknown`. This is not zero file reads or zero input charges, and no synthetic Direct item pricing is supplied.

Historical WL has 20 accepted read responses: 18 untracked bundle manifests, two changed-repeat responses covering nine repeated files, no observed small inline untracked response, and no unchanged-repeat response. Both repeat responses exceed the inline limits.

The current phase has 195 accepted read responses: 160 untracked bundle manifests, five inline untracked responses, and 35 repeat-containing responses. Changed and unchanged sections overlap in mixed responses. All 35 repeat responses exceed the inline limits based on their repeated-file sizes alone; a repeat intervention restricted to those limits has **zero observed eligible exposure**. The 160 bundle manifests occur in every workflow.

| Workflow suffix | Accepted reads | Bundle manifests | Inline untracked | Repeat responses | Changed sections | Unchanged sections | Mixed with untracked | Repeated files | Bundle tool calls | Maximum bundled snapshot bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 001 | 14 | 13 | 0 | 2 | 1 | 2 | 1 | 7 | 136 | 250655 |
| 002 | 14 | 12 | 1 | 1 | 1 | 0 | 0 | 3 | 106 | 250272 |
| 003 | 22 | 19 | 0 | 4 | 2 | 3 | 1 | 11 | 173 | 236169 |
| 004 | 14 | 11 | 0 | 3 | 2 | 2 | 0 | 6 | 120 | 281266 |
| 005 | 15 | 13 | 0 | 2 | 2 | 2 | 0 | 10 | 134 | 246701 |
| 006 | 13 | 13 | 0 | 0 | 0 | 0 | 0 | 0 | 134 | 296418 |
| 007 | 14 | 12 | 0 | 2 | 2 | 1 | 0 | 9 | 120 | 298701 |
| 008 | 23 | 14 | 2 | 7 | 3 | 6 | 0 | 20 | 139 | 229001 |
| 009 | 14 | 12 | 0 | 2 | 2 | 2 | 0 | 8 | 149 | 261231 |
| 010 | 19 | 16 | 1 | 4 | 2 | 4 | 2 | 16 | 123 | 240891 |
| 011 | 16 | 12 | 1 | 3 | 3 | 0 | 0 | 7 | 144 | 289588 |
| 012 | 17 | 13 | 0 | 5 | 5 | 2 | 1 | 18 | 135 | 257060 |

“Repeated files” counts occurrences, not distinct project paths. The mixed column denotes a response containing repeat sections and an untracked representation, so categories are not additive.

## Every bundle request's exact snapshot-byte sum

Each entry below is `physical client-to-server.raw line : sum of current snapshot UTF-8 bytes`. It sums the manifest's per-file `bytes:` fields, not the manifest prompt, bundle headers, tokenizer output or maximum context. Each listed run has one primary captured app-server directory, identified by its source report's client/server paths. All 18 historical and 160 current manifest requests are retained.

- Historical WL: `9:148848, 12:199346, 14:161973, 16:109558, 18:184827, 20:164957, 33:152816, 37:134297, 54:83573, 60:298575, 74:182892, 77:177924, 81:24833, 84:84448, 99:96781, 112:198051, 118:88877, 128:50711`.
- 001: `10:161973, 12:199346, 14:179555, 16:141975, 18:43352, 20:138449, 22:126645, 24:50711, 26:63843, 37:116472, 50:250655, 73:207077, 76:80919`.
- 002: `8:160267, 12:172444, 14:148848, 16:216516, 20:43352, 22:107477, 33:250272, 43:188582, 49:122390, 67:162635, 81:190765, 83:50760`.
- 003: `8:201052, 12:160267, 14:201052, 16:47551, 18:145063, 20:50711, 22:142436, 33:83185, 52:236169, 73:160455, 76:109264, 89:86045, 98:86594, 111:88806, 117:90001, 125:69530, 138:89272, 147:89266, 163:92095`.
- 004: `10:161973, 12:199800, 14:181715, 16:121037, 18:43352, 20:151801, 22:50760, 35:186518, 41:132432, 58:281266, 106:207028`.
- 005: `10:148848, 12:177966, 14:162427, 16:149178, 18:67167, 20:131674, 22:77284, 28:63843, 39:238642, 62:246701, 69:92067, 93:165220, 96:96605`.
- 006: `8:148848, 12:199346, 14:160721, 16:56477, 18:114246, 20:189267, 22:40125, 24:63843, 35:140186, 51:296418, 56:89169, 72:154720, 75:154720`.
- 007: `10:161973, 12:201052, 14:201506, 16:96433, 18:136743, 20:192686, 35:235687, 48:298701, 51:90405, 57:134584, 75:210748, 83:210748`.
- 008: `10:120004, 12:160267, 14:199800, 16:170189, 18:74015, 20:45058, 22:50711, 37:225661, 44:70473, 46:75938, 56:229001, 85:178722, 87:50711, 100:152483`.
- 009: `10:162427, 12:199346, 14:179555, 16:94806, 18:95979, 20:150295, 31:207821, 52:261231, 57:142304, 81:207376, 84:207376, 97:65084`.
- 010: `9:160721, 12:199800, 14:160267, 16:240891, 18:55294, 20:101090, 22:93151, 26:27729, 51:233595, 61:205865, 64:202863, 69:62171, 77:23496, 90:105968, 101:67676, 119:94051`.
- 011: `10:161973, 12:186221, 14:162427, 16:200579, 18:96433, 20:198144, 24:50711, 35:208854, 58:289588, 69:155426, 77:275879, 89:50790`.
- 012: `10:148848, 12:201506, 14:172444, 16:96433, 18:192686, 20:164760, 31:125132, 38:125132, 48:174812, 69:134538, 87:44601, 99:257060, 104:121583`.

The current sums range from 23,496 to **298,701 bytes**. The maximum is workflow 007, client line 48: nine snapshots of 26,239; 22,497; 44,053; 43,878; 23,347; 42,270; 46,352; 17,875; and 32,190 bytes. The largest single bundled file anywhere in the current phase is 70,191 bytes. The first observed manifest per workflow ranges from 120,004 to 201,052 snapshot bytes. Historical WL ranges from 24,833 to 298,575 bytes, with a largest single file of 67,534 bytes.

These manifest values bind the held snapshot's declared byte lengths and FNV64 digests through the captured accepted prompt. They do not independently prove a later on-disk bundle remained immutable. A future inline factor can save the held candidate's exact bytes and offline SHA identity; absent historical archive coverage must not be silently replaced with that stronger claim.

## Actual bundle consumption and observed charges

A bundle tool call is a native public `function_call` whose arguments explicitly mention an issued bundle path. Each counted call was checked to follow that bundle's manifest in the same native thread/source; call and output identities are unique by `(thread_id,call_id)`. All counted calls have a matching native `function_call_output` item and recorded input-attribution charges. Commands can contain multiple operations, so these are tool-call occurrences, not independent file-open counts. Full tool-output items may include wrappers and other command output.

The table reports observed input charges of those **whole output items**, summed across completed responses that include them. “Repeated” excludes the first observed charge of each item. It is neither incremental causal cost nor uncached-only usage. Shapes are overlapping lexical command categories, not an exclusive semantic coding.

| Workflow | Tool calls / charged outputs | sed ranges | awk selection | regex search | head/tail | python | cat | Observed output-item input charges | Repeated-after-first input charges |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Historical WL | 170/170 | 99 | 57 | 29 | 5 | 1 | 0 | 11571336 | 11101115 |
| 001 | 136/136 | 88 | 16 | 31 | 0 | 3 | 0 | 4340852 | 4038476 |
| 002 | 106/106 | 52 | 37 | 18 | 0 | 0 | 0 | 4498469 | 4163029 |
| 003 | 173/173 | 139 | 10 | 28 | 1 | 0 | 0 | 5569439 | 5218749 |
| 004 | 120/120 | 73 | 32 | 18 | 0 | 11 | 0 | 7382135 | 6966135 |
| 005 | 134/134 | 104 | 12 | 18 | 0 | 0 | 0 | 5824949 | 5477741 |
| 006 | 134/134 | 101 | 11 | 26 | 4 | 0 | 0 | 4894689 | 4589465 |
| 007 | 120/120 | 71 | 29 | 20 | 3 | 0 | 0 | 6806076 | 6455003 |
| 008 | 139/139 | 86 | 31 | 21 | 2 | 0 | 2 | 5346003 | 5034709 |
| 009 | 149/149 | 115 | 12 | 24 | 0 | 0 | 0 | 5833805 | 5506711 |
| 010 | 123/123 | 71 | 27 | 29 | 5 | 0 | 0 | 8430572 | 8112647 |
| 011 | 144/144 | 92 | 27 | 25 | 5 | 0 | 0 | 6041110 | 5702909 |
| 012 | 135/135 | 84 | 49 | 19 | 0 | 0 | 0 | 6750422 | 6416905 |

Historical WL's 170 outputs carry 11,571,336 observed input charges, including 11,101,115 repeated-after-first charges. The current 1,613 outputs carry 71,718,521 and 67,682,479 respectively. All cases remain included; no Direct comparison is possible from the available item ledger. Workflow 003's supplemental report retains an unknown output-action/reasoning attribution, so these input-component joins must not be described as universally complete response-level attribution or whole-workflow accounting.

The compact manifest user items themselves carry 112,540 observed input charges historically (108,341 repeated), and 954,702 currently (908,937 repeated). All 18 and 160 have unique charged item joins. The five current inline untracked items carry 96,022 observed input charges (88,111 repeated). These figures describe the actual different items; they do not price the same hypothetical text under another representation.

Two workflow 008 unchanged-repeat requests, client lines 93 and 97, have the same text hash `65a74a131a1cb3e6f14666e28e6b6e7f5d9f7b1370b453d4a4cd509435738f9b` and two matching native item IDs in thread `01a079d3-eb84-7a10-82c1-ea662bfae269`. They remain counted as responses but their item charges are not uniquely assigned by this conservative join. Each requests a 48,125-byte repeated file. Changed/unchanged categories can occupy the same user message, so category-level charge sums must not be added.

### Exact representative joins

- Historical manifest: `/home/user/src/work-leaf/bench-results/efficiency-raw-token-pilot-20260906T185328Z/runs/work-leaf-001/work-leaf-001-three-feature-bench-artifacts/observation/app-server/00000422153032163608-2899396/client-to-server.raw:9` → `server-to-client.raw:64`, RPC string `8`, thread `01a07825-10a9-77b3-bd6a-f75800663294`, accepted turn `01a07825-37bf-7793-b533-674e10dc8d7a`. Native `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-14-50-01a07825-10a9-77b3-bd6a-f75800663294.jsonl:21`, item `msg_01a07825-3835-7771-b542-98668543fec0`, canonical payload SHA-256 `100b2f218f5c2495247d3c3d40a294280dfc56259913e93536f0c9d5eb653b2a`. Exact text SHA-256 `9e858607207396a8dfcfe263c2c44e7f6bcae8f961cdd05fac4a9cd72f174e28`; 777 prompt bytes; 44 recorded response charges, 12144 input tokens including 11868 after the first charge.
- Current 001 manifest: `/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/runs/work-units-01-workflow-001/work-units-01-workflow-001-three-feature-bench-artifacts/observation/app-server/00000441344929586777-3537090/client-to-server.raw:10` → `server-to-client.raw:74`, RPC string `9`, thread `01a07949-eeb4-77b0-b7e6-12dd24809355`, accepted turn `01a0794a-1582-7731-9551-e7e664b46082`. Native `/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-43-01a07949-eeb4-77b0-b7e6-12dd24809355.jsonl:22`, item `msg_01a0794a-15a7-7750-b85b-12ad401e1e07`, canonical payload SHA-256 `9831b4130e4ca57e73d9f8cb3b3edc7f8f6252ba4f35e665cb8c5649594be6dc`. Exact text SHA-256 `8a42eb283a5acaeec65d5b239ba1e904493633f8a356679d7e502a0fcd1f9900`; 857 prompt bytes; 32 recorded response charges, 9856 input tokens including 9548 after the first charge.
- Current 001 mixed changed/unchanged repeat: `/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/runs/work-units-01-workflow-001/work-units-01-workflow-001-three-feature-bench-artifacts/observation/app-server/00000441344929586777-3537090/client-to-server.raw:63` → `server-to-client.raw:29201`, RPC string `62`, thread `01a07949-eeb4-77b0-b7e6-12dd24809355`, accepted turn `01a0795a-d141-7cc0-a283-b23162153c85`. Native `/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-43-01a07949-eeb4-77b0-b7e6-12dd24809355.jsonl:228`, item `msg_01a0795a-d16a-7de1-818b-7f1b869c97bd`, canonical payload SHA-256 `8b23d3f73ec80b2808f67b890bc5a1a39a0cb44e06aaf04ffae9a347144fbe96`. Exact text SHA-256 `5ca302c76a59278d22b2097ceda1e9afb27a6b408014997c7320ec63d37493a1`; 7647 prompt bytes; 10 recorded response charges, 19390 input tokens including 17451 after the first charge.
- Historical narrower retrieval: `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-14-50-01a07825-10a9-77b3-bd6a-f75800663294.jsonl:27`, call `call_sI2MBJG1qlOYRi5kcwHCxszH`, arguments SHA-256 `980262b7b7bb6a152bd3886e71c3d1bcc44178a9e4c4c7e35f104b12fcc812ad`, is a `sed -n '1,220p'` request on the issued bundle. Its output at line 30, item `fco_01a07825-6e80-7383-acfa-13ab6676391c`, payload SHA-256 `50673835520070b516976d72d27a590c6f3f720226ca53f558f20f205efe061d`, is 13204 UTF-8 bytes and reports exit 0 without a truncation marker. The complete issued snapshot sum was 148848 bytes. The output item carries 124399 input charges across 43 responses, including 121506 repeated-after-first charges.
- Current 001 narrower retrieval: `/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-43-01a07949-eeb4-77b0-b7e6-12dd24809355.jsonl:28`, call `call_k7kMNWN29XlbYvWgK7g6BmZY`, arguments SHA-256 `ae260272675f5975d99b077c066d9dfe41ce6b030927e74b2545c10b8a88287d`, is a `sed -n '1,240p'` request on the issued bundle. Its output at line 31, item `fco_01a0794a-4959-7310-8818-6ee5d5203694`, payload SHA-256 `b306f4960c8fb9d25a3dd2670e0c1e8ad5cb370c3d180f5b20086cbf923c7750`, is 14282 UTF-8 bytes and reports exit 0 without a truncation marker. The complete issued snapshot sum was 161973 bytes. The output item carries 96937 input charges across 31 responses, including 93810 repeated-after-first charges.

These concrete outputs establish that actual retrieval can return much less than the whole snapshot bundle. They do not prove that all later retrievals collectively remain partial, that every lexical range command is narrow, or that all output bytes belong only to the bundle. No mediated full bundle read-back was observed in the accepted WL file-text responses; the counted consumption is via native tools.

## Feasibility and confounding boundary

The untracked, successfully bundled component is a well-exposed distinct factor. Returning its held full text inline tests eager delivery versus a compact manifest plus agent-chosen later retrieval. Keep ordinary first-read tracking, repeated sections, automatic conflict refresh, bundle creation/ownership/counters, threshold decision and write-failure fallback unchanged. The factor may enlarge retained context while removing retrieval calls; forcing the retrieval calls to match would suppress a causal pathway, not improve isolation. It cannot establish a fraction of the historical WL-versus-Direct gap without additional comparative evidence.

A repeated-read full-text factor remains separate. An inline-threshold-limited version is not useful in this observed workload because all repeats exceed the threshold. Treating large repeats inline creates much larger prompts and replaces diff/unchanged semantics. Using a separate repeat-owned bundle instead tests deferred retrieval plus representation, not the same inline intervention; it must not move repeat snapshots into the existing untracked bundle bucket or perturb that bucket's threshold and path allocation.

The observed maximum is almost 299 kB of source text before response framing and prior conversation. Bytes are not tokens; these data alone cannot certify context headroom. Any prospective resource/admission ceiling must be declared before launch, not inferred to be safe from this sample or used to drop large/failing observations afterward. Large-request rejection, compaction, missing tails and tool-cycle changes remain outcomes. This audit proposes no new arm, sample, provider call or accounting correction.

## Accounting limits retained

The observed charge inventories are native-item/component evidence, not a replacement whole-workflow total. The existing work-unit token report remains unchanged and unavailable for its whole contrast because workflow 003 has a nonadditive `tokenUsage.last` notification. Its raw completed response usage is arithmetically valid; the separate nullable output-action attribution does not itself establish a numerical input discrepancy. Missing usage tails and compaction coverage stay explicit. No private reasoning was read or reproduced for this exposure audit.

