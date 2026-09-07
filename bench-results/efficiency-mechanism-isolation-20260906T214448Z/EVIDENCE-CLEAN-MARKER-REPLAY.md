# Closed replay of clean-review marker detours

Recorded at 2026-09-07 12:21:48 UTC. Scope: existing W workflows 001–012 only, in run-ID and physical-request order, under [the active plan](PLAN-CANDIDATE-INVENTORY.md). This is a provider-free Stage B branch/action replay, not a new experiment, condition comparison, runtime fix or retotal.

## Result

All 52 accepted author-fix requests were replayed. Thirty-six contain a standalone `FINDINGS` marker; sixteen contain only one standalone `NO_FINDINGS` after commentary. No request has absent or conflicting standalone markers. Every one of the same sixteen cases in [EVIDENCE-BEHAVIOR.md](EVIDENCE-BEHAVIOR.md#L21) fails WL's first-nonempty-line classifier and passes the saved Direct first-explicit-marker classifier.

For each clean case, the entire routed review aggregate equals the ordered public reviewer messages joined with two newlines. Every constituent message matches its unique native assistant item by exact thread/item/turn identity and full text. None of these sixteen aggregates has a quoted/fenced marker or any preceding `FINDINGS`. This is not an inference from a review title, the last word alone, or token totals.

Each actual continuation then has exactly one author message, `@work-leaf done`, and exactly one reviewer message, `NO_FINDINGS`. The corresponding two accepted turns have no other public/native tool or edit action. Other concurrently executing agents are outside this pair-level absence claim. All 32 completed-response IDs are distinct and match native usage records exactly.

The observed pair responses sum to **3,068,616 raw tokens**: 3,063,601 input (including 2,609,152 cached) plus 5,015 output (including 4,703 reasoning). This is a demonstrated charged **WL overhead offset**, not a cause of WL's historical saving. Removing this deterministic branch would avoid routing that immediate pair on the same recorded aggregate; the complete future workflow, scheduling and retained contexts would then differ. No whole-counterfactual net saving, marginal share, uncertainty interval or frozen-total subtraction is asserted.

## Source rule and causal chain

The frozen W provider [streaming reply loop](phases/work-units-01/infrastructure/evidence/src/codex.rs#L277) appends each completed public agent message to `streamed_messages` and returns `join("\\n\\n")`. It therefore preserves inspection commentary before the final clean verdict.

The frozen W [`has_no_findings`](phases/work-units-01/infrastructure/evidence/src/review.rs#L411) is exactly first-nonempty-line recognition. [`review_commit_streaming_with_ids`](phases/work-units-01/infrastructure/evidence/src/cli.rs#L1260) tests that aggregate, sends it to the author as findings, processes the author's reply, then sends the reviewer a recheck. The recorded done-only/clean-only outputs instantiate that branch without new repair or verification evidence.

The saved Direct [`review_is_clean`](../efficiency-raw-token-pilot-20260906T185328Z/infrastructure/drivers/bench-three-features-direct-common#L1040) instead stops at the first standalone `NO_FINDINGS` or `FINDINGS` line. Its original embedded Python branch was compiled from the retained source and executed against in-memory captured aggregates; the shell driver and providers were not run. The WL replay is a source-equivalent predicate, not a claim that a newly built Rust binary was executed:

```python
# Source-equivalent WL predicate for these captured texts.
next((line.strip().lower() == "no_findings"
      for line in text.splitlines() if line.strip()), False)

# The saved Direct branch, unmodified:
for line in text.splitlines():
    marker = line.strip().upper()
    if marker == "NO_FINDINGS":
        sys.exit(0)
    if marker == "FINDINGS":
        sys.exit(1)
sys.exit(1)
```

The immediate source of the detour is review-verdict recognition (C22), interacting with review-fix/evidence routing (C23) and done/recheck continuation boundaries (C32/C38). It is not evidence that general protocol correction, evidence-only resolution or explicit completion lacks value.

## Complete retained case membership

All twelve workflows remain in scope, including failed workflow 010. It has seven FINDINGS-bearing fixes and no matching clean detour; it is not excluded or assigned zero workflow usage. The full 52-request census and launcher start times are bound by the prior evidence table.

C/S are physical client/server JSONL lines in the capture bound by [Appendix A](EVIDENCE-BEHAVIOR.md#L136). Each native N link identifies the exact native file and physical line. All rows below have the same replay disposition: WL rejects clean; Direct accepts clean; author done only; reviewer clean only; no pair-local tool/edit action. The aggregate SHA-256 binds the complete routed review text, excluding only the source-owned fix prefix and suffix.

| Case: fix/recheck | Prior final clean S → native N | Public messages in original aggregate | Exact aggregate SHA-256 |
| --- | --- | ---: | --- |
| 001 C38/C40 | S4007 → [N120](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-38-45-01a0794d-a0f0-7672-ab44-56ecb8046add.jsonl#L120) | 10 | `b377ebdfefd10975a655278b7904eed3ceef43e35356548882c8341ba211a6ae` |
| 002 C44/C46 | S8634 → [N136](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-39-21-01a0794e-2b36-76a2-a5c7-968cc7215198.jsonl#L136) | 10 | `172a41262f4aa67780b56f94c14c1e8ea6bb711a21841026fa14e3bf5f7e6628` |
| 002 C84/C86 | S55240 → [N185](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T03-15-54-01a0796f-a37f-7a73-b8ed-228ec6271e3f.jsonl#L185) | 3 | `217be272c0db9457e37fb887d0d96cd478428821f8824126745c9a922a583014` |
| 003 C34/C36 | S5407 → [N75](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-39-51-01a0794e-a1e1-7dd3-aa07-dbc1419b24be.jsonl#L75) | 4 | `d4707285bc0fb53421afdd248ee98217d55eaf953d41a3a5dfbf6d968ed5c7c7` |
| 003 C59/C61 | S42824 → [N128](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-51-10-01a07958-fc93-71a1-b4f2-718e496fbe9d.jsonl#L128) | 8 | `77133fc0a1732eae2fbed1a2f7ef2bda8969b7bc63c83f30a4b0dd6f2e0043b1` |
| 003 C170/C172 | S71269 → [N272](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T03-01-13-01a07962-2feb-7311-b75a-7f2dec62c706.jsonl#L272) | 2 | `b069a81e50fcd9198b4e3f487d61b0d3a319e5766219f84dac2184d46d1fdb31` |
| 004 C42/C44 | S11661 → [N164](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-02-43-01a0799a-7ec8-7020-adeb-ae261528de0f.jsonl#L164) | 8 | `2f4ce8792eb53827ebcb793fbda199569015fdd7e4bf7073c010ef8d37363ed2` |
| 004 C107/C109 | S70899 → [N123](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-26-11-01a079af-fc1a-7620-86e0-feb5e8648e34.jsonl#L123) | 8 | `ad68459b2ae7b653b9c2650cf25a14fd02f800a7c2ceb23af68d1331a54b434d` |
| 005 C40/C42 | S13890 → [N144](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-01-32-01a07999-6876-7f11-baa1-45df1e1825bd.jsonl#L144) | 8 | `01045f2ceba3c8f8c416db3fc01b756ba4fdca81f7390388269e219152c83d3e` |
| 006 C86/C88 | S44797 → [N179](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-14-26-01a079a5-3ac1-7bf0-a9f7-2d28f9903f8f.jsonl#L179) | 3 | `32de0ecd919678aaa4540ae3c8e9b0478929571b141e508904941b25ee149c4f` |
| 007 C36/C38 | S4741 → [N87](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-45-51-01a079c1-fc20-75b0-9858-ef66d66ead15.jsonl#L87) | 8 | `73c6da499e1cfbe45e41de37aeef79ba83defeee2cf9602aeb53f618582e0e99` |
| 007 C76/C78 | S27232 → [N154](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-49-59-01a079c5-c437-70d0-8187-49d88de2744e.jsonl#L154) | 3 | `f8351104cfd41f693ea044a2e2e8851fa972394c3f43acd373ee9b7e511a36d2` |
| 008 C71/C73 | S25229 → [N136](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-51-30-01a079c7-28ff-7d53-9fd0-6dca7bfe5481.jsonl#L136) | 8 | `f4e459d56a65a9ce1e0f4ff3093cdd33ce87e2a80acd54b04ac1134c61e0f5d9` |
| 009 C34/C36 | S14941 → [N117](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-47-57-01a079c3-ea25-7783-85a5-c9fc5c7ab4ec.jsonl#L117) | 7 | `735e403ed1c943aa92be2663f58585c7a842500e2818016e5ebde8feaa8b77f2` |
| 011 C42/C44 | S6867 → [N149](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T05-39-45-01a079f3-57bc-76d2-ac8d-4e6b1a02002e.jsonl#L149) | 5 | `60baa38a6d54c917ea8e0ddba63ad701f1d36b46b9169587946b982a56b8ec8b` |
| 012 C120/C122 | S35901 → [N226](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T05-52-50-01a079ff-5174-7073-9660-78c01867ebeb.jsonl#L226) | 2 | `1f6132536d79b5748d499af08b00a70da4d98502a936c84ee87aca2be1d0e28c` |

## Exact observed response records

I/C/O/R are original input/cached-input/output/reasoning-output fields. Neither cached input nor reasoning is added twice. Each row's native record agrees with its original raw response on response ID, thread, turn, all four fields and total input+output. Every field is a nonnegative integer with valid subset relationships. All original top-level input attribution items plus request-field inputs equal the recorded input: residual zero for every row. Nested content is not added to parent items.

The prior [whole-response evidence table](EVIDENCE-BEHAVIOR.md#L58) retains all response IDs and per-response context accounting; its preceding case table and linked packet/raw sources resolve the public output items and turn chains. The native locators below independently extend that evidence; they do not fill any unrelated missing tail or clear W's whole-workflow accounting flags.

| Case / role | Exact response ID | Raw S / native usage N | I / C / O / R |
| --- | --- | --- | --- |
| 001/C38 author | `resp_042ad8bf05f8964f016a9e0850d59487d28b45422516aa4eb9` | S4027 / [N177](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-48-01a0794a-0076-7363-bccc-7129dfc27e60.jsonl#L177) | 87579 / 86400 / 79 / 68 |
| 001/C38 recheck | `resp_05db3622b4ef8a4c016a9e08547e9c87d28f38d047bbd4c79e` | S4045 / [N133](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-38-45-01a0794d-a0f0-7672-ab44-56ecb8046add.jsonl#L133) | 47236 / 23936 / 525 / 516 |
| 002/C44 author | `resp_0b98cb4ba4a946c2016a9e0890c9fc87d2acf7a38c8cf92187` | S8654 / [N189](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-48-01a0794a-0079-7820-afde-772b691da5dc.jsonl#L189) | 87098 / 86400 / 233 / 222 |
| 002/C44 recheck | `resp_09d20ff1a35053c8016a9e0898801887d28be92fe2f96773ec` | S8724 / [N149](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-39-21-01a0794e-2b36-76a2-a5c7-968cc7215198.jsonl#L149) | 78696 / 23936 / 153 / 144 |
| 002/C84 author | `resp_0a27f35d0f41cb6d016a9e11dd7b4c87d28c5d78df965ab2e6` | S55260 / [N295](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-43-01a07949-eed9-79a3-82be-85eb0c4d10dc.jsonl#L295) | 122595 / 122240 / 139 / 128 |
| 002/C84 recheck | `resp_0507b0c91a4c58ea016a9e11e131b887d29efd5364e83346e9` | S55278 / [N198](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T03-15-54-01a0796f-a37f-7a73-b8ed-228ec6271e3f.jsonl#L198) | 148999 / 136576 / 268 / 259 |
| 003/C34 author | `resp_092a936e353885ad016a9e085aef6887d2bf574fc8ee5fed29` | S5635 / [N167](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-48-01a0794a-0078-7700-adb7-1dc143005ee8.jsonl#L167) | 73301 / 72064 / 127 / 116 |
| 003/C34 recheck | `resp_09c5c3a59d8ea046016a9e085ebbf887d2be4ed5040c287ba6` | S5928 / [N88](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-39-51-01a0794e-a1e1-7dd3-aa07-dbc1419b24be.jsonl#L88) | 51718 / 24960 / 191 / 182 |
| 003/C59 author | `resp_0ca3a081c7f26971016a9e0b7b119487d28278b79f39799fc1` | S43075 / [N225](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-53-01a0794a-144a-73c0-aa63-2586497de910.jsonl#L225) | 126239 / 125312 / 126 / 115 |
| 003/C59 recheck | `resp_0a9adefb55ea67b1016a9e0b7ed67c87d2b326b4a28e05f1a1` | S43462 / [N141](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-51-10-01a07958-fc93-71a1-b4f2-718e496fbe9d.jsonl#L141) | 79686 / 35200 / 283 / 274 |
| 003/C170 author | `resp_06dd34dd4bcd978a016a9e16d9075887d2a4c9bc0d1590a479` | S71287 / [N1233](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T02-34-43-01a07949-ee9c-7503-a486-0108cc6bb00d.jsonl#L1233) | 97822 / 96640 / 9 / 0 |
| 003/C170 recheck | `resp_0976c12539b2ff14016a9e16dcbc5487d2ae24fdc9dc11e547` | S71303 / [N283](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T03-01-13-01a07962-2feb-7311-b75a-7f2dec62c706.jsonl#L283) | 139716 / 138624 / 7 / 0 |
| 004/C42 author | `resp_03fe3376c0a76ef0016a9e1c2b978887d2815f2577b8dcd75b` | S11942 / [N216](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T03-56-43-01a07995-0233-77a3-8845-9171279da93e.jsonl#L216) | 118509 / 117120 / 117 / 106 |
| 004/C42 recheck | `resp_05116ced75888b13016a9e1c2f6ed487d286e2dac9fd7117b1` | S12457 / [N177](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-02-43-01a0799a-7ec8-7020-adeb-ae261528de0f.jsonl#L177) | 83423 / 25984 / 155 / 146 |
| 004/C107 author | `resp_04d82cf7fa2cac27016a9e2266f41887d28a9bb5564079e460` | S70919 / [N289](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T03-56-39-01a07994-f1c0-79c0-af7a-06582c33f38c.jsonl#L289) | 127714 / 126336 / 140 / 129 |
| 004/C107 recheck | `resp_0a33447e21b78ef3016a9e226d047c87d2b68e6a20f2843fd1` | S70937 / [N136](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-26-11-01a079af-fc1a-7620-86e0-feb5e8648e34.jsonl#L136) | 133822 / 80256 / 165 / 156 |
| 005/C40 author | `resp_0d7884c26304448b016a9e1baaa24c87d2afef92199484753c` | S14269 / [N219](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T03-56-43-01a07995-0234-73d1-8bfc-af39546b9b1a.jsonl#L219) | 125201 / 124288 / 104 / 93 |
| 005/C40 recheck | `resp_0602f3d57cf0ac33016a9e1bae068887d2a75a057fd9736342` | S14861 / [N157](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-01-32-01a07999-6876-7f11-baa1-45df1e1825bd.jsonl#L157) | 52610 / 24960 / 177 / 168 |
| 006/C86 author | `resp_0135db37de9183ef016a9e1fbbfe6087d2bfa4b554157f9ef6` | S44815 / [N297](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T03-56-39-01a07994-f1db-74f0-832e-d7bba9841ac3.jsonl#L297) | 109379 / 108928 / 9 / 0 |
| 006/C86 recheck | `resp_0b3511f8ae88b79e016a9e1fbe5bdc87d28b3ce3bacbbf1232` | S44833 / [N192](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-14-26-01a079a5-3ac1-7bf0-a9f7-2d28f9903f8f.jsonl#L192) | 107694 / 104832 / 132 / 123 |
| 007/C36 author | `resp_09446b0c2ac247dc016a9e25f047bc87d2bbf8ed80416bd436` | S5040 / [N226](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-41-08-01a079bd-a9a4-79a3-8a4a-f06dea1c32f0.jsonl#L226) | 111918 / 110976 / 147 / 136 |
| 007/C36 recheck | `resp_0b5289783867e568016a9e25f533f087d2b26baf2dd31de6ae` | S5372 / [N100](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-45-51-01a079c1-fc20-75b0-9858-ef66d66ead15.jsonl#L100) | 56808 / 24960 / 215 / 206 |
| 007/C76 author | `resp_000c5443f471f4c8016a9e28500c6487d281727064f53c6254` | S27252 / [N347](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-41-13-01a079bd-bd73-7dd3-b9fb-fda6f7621e7d.jsonl#L347) | 120805 / 120192 / 54 / 43 |
| 007/C76 recheck | `resp_02246e00974f8608016a9e285335bc87d298a97d88d031cb8f` | S27270 / [N167](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-49-59-01a079c5-c437-70d0-8187-49d88de2744e.jsonl#L167) | 64503 / 61824 / 203 / 194 |
| 008/C71 author | `resp_01d8a89fcc03e801016a9e27ca486c87d2b4ad2822132e2b0d` | S25249 / [N237](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-41-08-01a079bd-a9be-7c30-8828-e7f837a9ea46.jsonl#L237) | 103308 / 101760 / 205 / 194 |
| 008/C71 recheck | `resp_094fad0efbb602a1016a9e27d1d50887d2b2ec3f5a063e6688` | S25267 / [N149](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-51-30-01a079c7-28ff-7d53-9fd0-6dca7bfe5481.jsonl#L149) | 65009 / 34176 / 166 / 157 |
| 009/C34 author | `resp_09909ca4a2cdcdc1016a9e26a8fc1887d28f7464b3c9e3f0bc` | S15263 / [N249](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-41-08-01a079bd-a9a4-7db0-890b-873aa92a6f2f.jsonl#L249) | 91543 / 90496 / 156 / 145 |
| 009/C34 recheck | `resp_0a4046b6ab524458016a9e26ae677c87d2940987348beda89a` | S15602 / [N130](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T04-47-57-01a079c3-ea25-7783-85a5-c9fc5c7ab4ec.jsonl#L130) | 59854 / 25984 / 253 / 244 |
| 011/C42 author | `resp_017b8f140d194413016a9e32d3e3a087d2b061aa5c544806c7` | S7109 / [N246](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T05-35-22-01a079ef-540b-7e11-ac1d-e2fa6698f5e4.jsonl#L246) | 92573 / 91520 / 77 / 66 |
| 011/C42 recheck | `resp_01b065dae5c12243016a9e32d7df1887d28bad258d7e06ad44` | S7544 / [N162](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T05-39-45-01a079f3-57bc-76d2-ac8d-4e6b1a02002e.jsonl#L162) | 59051 / 25984 / 212 / 203 |
| 012/C120 author | `resp_0fb00e3420caa0e3016a9e37a8c99887d2ae115bdc361eaf1d` | S35919 / [N455](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T05-35-27-01a079ef-67d4-70b0-bf3c-de89984061cf.jsonl#L455) | 139654 / 138624 / 9 / 0 |
| 012/C120 recheck | `resp_0819ff32278f41da016a9e37ab59d887d2b29300bb0bb41745` | S35937 / [N239](/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T05-52-50-01a079ff-5174-7073-9660-78c01867ebeb.jsonl#L239) | 99538 / 97664 / 179 / 170 |

## Replay gates and limits

The read-only replay passed these gates:

1. Rehash twelve original packets and twelve client/server pairs against the prior census and packet inventory. Strictly decode complete JSONL; preserve physical lines. Enumerate every fix request, not only the sixteen selected cases.
2. Require each cited turn/start RPC's exact typed successful reply, with no mixed error/result and nonempty thread/turn IDs. Check complete aggregate reconstruction, one standalone clean marker, no FINDINGS marker and no quoted/fenced marker candidate.
3. Resolve only observer-matched native threads. Rehash the twelve rollout-metadata files and thirty-two needed native sources against retained metadata and the source inventory in the pinned [COMPACTION-AUDIT.json](phases/work-units-01/COMPACTION-AUDIT.json). Check session thread/cwd/CLI 0.153.4, and exact target turn contexts gpt-5.5/xhigh.
4. Join every original aggregate message and both continuation messages by exact native thread/item/turn IDs and full text; reject duplicate target item identities. The pair has one assistant message per turn. Inspect public item kinds and native response/event action kinds, including response items within the matching native turn-context interval, for tool/edit activity. Explicit message metadata supplies message identity; context order is only an additional activity-absence check, not an invented item-ID join. Native reasoning bodies are not interpreted or reproduced.
5. Require one original raw response per target turn, one matching native usage record per response ID, no repeated ID across pairs, exact four-field arithmetic and zero input-attribution residual. Rehash all 86 source files read by the replay at the endpoint; every hash remains equal.

These checks replay public actions and directly recorded response charges, not hidden model reasoning, an unobserved alternative workflow or the historical Direct–WL attribution denominator. The complete W phase's failed primary hypothesis, its untested fixed-sequence secondary, all original failures and accounting limitations remain intact. No parked R outcome or cost was read.

## Parser safety qualification

The exact source-rule replay also evaluated seven small in-memory edge texts; these are characterization checks, not new runtime tests or an implementation. Both rules reject absent and quoted-only markers, both accept plain clean, and both reject FINDINGS-before-clean. Only Direct accepts commentary-before-clean, as in the sixteen real cases.

**Neither existing rule is a general conservative verdict validator:** both accept a leading NO_FINDINGS followed by FINDINGS, and Direct also accepts a standalone NO_FINDINGS inside a code fence. The sixteen real aggregates contain neither shape. Therefore this result supports the exact observed branch mismatch, not blindly transplanting the Direct parser. Any separately authorized implementation must preserve ambiguous/conflicting/quoted/fenced cases as unverified and retain required review, prompts, targets and all non-clean behavior. No normal-WL parser is modified here.

## Source pins

The original capture/packet SHA-256 values are retained in [EVIDENCE-BEHAVIOR.md](EVIDENCE-BEHAVIOR.md#L136). Native file and rollout-metadata SHA-256 values resolve through the source inventory of the pinned compaction audit; only its provenance map is reused, not its totals as an accounting authority. The actual source bytes, public items and response counters were independently replayed.

| Source | SHA-256 |
| --- | --- |
| [Prior behavior evidence](EVIDENCE-BEHAVIOR.md) | `144eaa49395c8f2621e2f1e9b624f8ec54e34381f84daab9ca8441bd90067af2` |
| [PHASE-MANIFEST.json](phases/work-units-01/PHASE-MANIFEST.json) | `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950` |
| [COMPACTION-AUDIT.json](phases/work-units-01/COMPACTION-AUDIT.json) | `bcec81dd2641dd27f83ae11eb71371062bb2df0c09333226379fccfede9fc823` |
| [review.rs](phases/work-units-01/infrastructure/evidence/src/review.rs) | `12d70548086d053583e1fc2fc6b4f191dd262cf61b45637a1ee5d04931c919ee` |
| [cli.rs](phases/work-units-01/infrastructure/evidence/src/cli.rs) | `70d5107a5b022f4dba118e8567a5c2f54330a51319f5d97d84299380e6246c0d` |
| [codex.rs](phases/work-units-01/infrastructure/evidence/src/codex.rs) | `334eb3008f3b0a8db0d98bed2d3191f5cbf37052fb342dcf26a91ccd4ee502d8` |
| [bench-three-features-direct-common](/home/user/src/work-leaf/bench-results/efficiency-raw-token-pilot-20260906T185328Z/infrastructure/drivers/bench-three-features-direct-common) | `489289165601e00a545f76ab0631d5e0f48d644b928376488677e1875651e386` |
| [Frozen work-unit protocol](PROTOCOL-WORK-UNITS.md) | `8d7f29516d37a1e050e539d7098249a2550d61b7eb3b9c30c1f18f82086b9964` |
| [Active candidate-first authority](PLAN-CANDIDATE-INVENTORY.md) | `6092e9c2e9fe4bed7a4e9672c0a6051449e8472a2f4efd2134d14cd75c9bcd16` |

This document is a descriptive replay receipt. It changes no source, helper, fixture, admission, classification or report. No provider generation or new real-agent verification is needed for this read-only analysis.
