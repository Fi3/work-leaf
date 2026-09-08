# C08 original accounting gaps: source diagnosis

Both original outcomes remain `unknown` / measurement `ineligible` with null bounds. This note explains their exact source predicates; it does not repair a result, infer zero generation, remove a diagnostic, or publish corrected accounting. Inspection used saved JSON/frame metadata and exact input-byte comparisons only. No `audit_run`, observer/analyzer/extractor, qualifier, provider, or private command was executed. No token totals or contrasts were recomputed.

The once-only consolidation is `COMMON-ACCOUNTING-ORIGINAL.json`, SHA `9895317a00a9e9964953c4d36c6e11e271d021b8652877e4fe81afa3dc3069b1`, under scope `804e16e28df323f70123743afc7bb04fad908de04afab8083a467d3aaebb21d6`. It retains three new call attempts, zero baseline calls and no publication/execution/identity/integrity errors. The run-level accounting failures below are separate from successful publication.

## 001: a captured, accepted usage-less failed turn

The retained `rollout-audit.json` has six expected/matched threads, no missing expected thread and one `unobserved_cwd_threads` identity: `01a080e4-6f65-7f62-a9d4-86b3df91bcbc`. That identity is **not absent from the saved process stream**.

For `observation/app-server/00000568115683071173-564`, physical lines are:

| Witness | Exact locator |
| --- | --- |
| Original and forwarded request, fully equal parsed frame | Both client files line 51, string RPC ID `50` |
| Accepted response | Server line 14546, turn `01a080e4-6fce-73a0-8c06-05220ceda0dc`, no RPC error |
| Actual user delivery | Server 14550/14551, item `01a080e4-7606-7e01-8ac3-60fe1b42c98b` |
| Native explicit user | Native line 7, item `msg_01a080e4-7604-7cb1-ae06-d2f9d4005ee9`, typed passthrough `user.text` for that exact turn |
| Actual failure | Server 14554 error and 14555 `turn/completed`, status `failed`, `codexErrorInfo=serverOverloaded` |

Original request, forwarded request, public user and native user have the same complete 107,076 UTF-8 bytes, SHA `aeeef3fc9ad40e6e212420885abf5ac904b4a025553bffed6a714d8ab446563a`. The terminal message is “Selected model is at capacity. Please try a different model.” It is a retained provider failure, not a protocol or source correction.

The exact native file is `/home/user/.codex/sessions/2026/09/08/rollout-2026-09-08T14-00-49-01a080e4-6f65-7f62-a9d4-86b3df91bcbc.jsonl`. Its ten rows contain session identity (1), task start (2), supplied context (3–5), exact `gpt-5.5` / `xhigh` context and matching cwd (6), the user input (7–8), `token_count` with `info:null` (9), and task completion (10). There is no native `token_usage_record` or assistant/function-call response item. The saved public thread has no `rawResponse/completed`, cumulative-usage notification or generated assistant/tool item. These are finite observed-source facts, not proof of hidden-provider completeness or zero charge.

The frozen observer explains the misleading diagnostic:

1. `bench-observer/src/lib.rs::analyze_app_server` builds `generation_threads` from requested turns, but inserts `UsageObservation` / thread accounting only after an actual cumulative usage event (`extract_usage`, lines 5400–5454). This turn has no such event. Because a turn was requested, it is not a session-only thread either (lines 5281–5291).
2. `extract_rollout_metadata` constructs `expected` from `summary.threads` (7317–7354), supplemented by the separate exec capture path, not every accepted app-server turn. A same-cwd native header outside that expected set and within the run-time window becomes `unobserved_cwd_threads` (7422–7435). The message at 7544–7549 says “absent from process capture,” although this captured usage-less app-server input is present.
3. `accounting_untracked_reads.py::audit_run` retains the original observer errors, then `batch_analysis.py::accounting` (294–303) permits only the exact interrupted-usage count message. The rollout message therefore remains an ineligibility reason. `audit_run` clears bounds on any error (its final error branch).

The three original interrupted-usage gaps remain separate and untouched. The usage-less overloaded turn is not silently counted as another measured response, deleted, or zero-filled.

## 003: pre-context compaction rejected by the existing prefix rule

The exact failing native source is `/home/user/.codex/sessions/2026/09/08/rollout-2026-09-08T13-47-43-01a080d8-7297-7fe1-93b0-c344d213fd2c.jsonl`. Thread identity is the UUID in that filename; the affected turn is `01a080f6-8ab1-7393-80d8-fd919ee39544`.

| Native physical line | Saved identity / boundary |
| --- | --- |
| 525 | Actual task start for the affected turn |
| 526 | `token_usage_record`, response `resp_0476b42dd55e23d4016a9ffd96c13487d2af0254c64712a954` |
| 527 | `compacted`, same explicit `compaction_response_id`; `latest_token_usage_record` equals the complete line-526 payload |
| 528–533 | Settings/usage metadata, same compaction completion, supplied context messages and world state; no generated assistant/tool item |
| 530 | `ContextCompaction` completion, item `01a080f6-8ac7-7e72-8298-3e9e9b0addf8` |
| 534 | The unique exact-turn context: `gpt-5.5`, `xhigh`, matching recorded project cwd |
| 535 | Native `user.text` input, item `msg_01a080f7-f2f7-79f2-baef-31e2cb7b14f1`, explicit affected-turn passthrough |
| 541 | First following ordinary response-usage record |

In `observation/app-server/00000568115683074735-566`, string RPC ID `94` is at original/forwarded client line 95; the complete two parsed request frames are equal. Server 66118 accepts the exact affected turn, 66121 starts the compaction item above, 66254 publishes the exact line-526 response, 66257 completes that compaction item, and 66258/66259 delivers the user item `01a080f7-f2f7-79f2-baef-31f41676cf28`. The four raw/native response counters are individually equal with exact JSON integer types; no totals were emitted or recalculated.

That input is 979 UTF-8 bytes, SHA `1a294fa41e934bd5af25aa94f98e201a646bf5b65b731422561f916304edb6af`, equal across request/public/native. **The same text also appears in earlier RPC `64` at client line 65, accepted as another turn at server 48952.** Full-text equality alone is therefore not unique ownership. The affected occurrence is RPC `94`, accepted-turn identity and same-turn compaction/user lifecycle; it is not a timestamp FIFO or arbitrary first hash match.

The frozen `audit_compaction.py::audit_rollout` maintains contexts only as it traverses earlier rows. Line 92 rejects response 526 because its turn has no matching context yet. That exception occurs before inserting the response into `records`, adding its prefix contribution, or setting `previous`. Consequently native line 527 gets `unlinked compaction marker`. The retained result subsequently reports cumulative-prefix errors at native lines 541, 552, 565, 578, 591, 615 and 626. Their location is consistent with the skipped earlier record; this diagnosis does **not** claim that a corrected complete prefix ledger or whole-run audit has passed.

`accounting_untracked_reads.py::native_ledger` raises on any `audit_rollout` error before returning that ledger (121–126); `audit_run` therefore exits while iterating the third native source. The original `corrected_scope` is null. It does not reach raw reconciliation, the remaining native metadata rows, or a new measurement result. Five original interrupted-usage gaps remain recorded; this source explanation cannot establish their eventual bounds.

## Smallest separate qualification, if explicitly authorized

- **001:** retain a source-only membership qualification joining the exact failed accepted turn above across original/forwarded/public/native and closed invocation inventory, including the genuine overload status and missing usage. This resolves what the particular “absent from process capture” message describes, not the broader accounting completeness question. Do not delete the error from the original helper or infer a measured zero. Any differently scoped numerical treatment would need its own declared policy and is not authorized by this note.
- **003:** the existing separate `preflight/candidate-compaction-context-correction/accounting_compaction_context.py::prove_future_contexts` already expresses the required narrow proof (SHA `513c8632803656d4f582a4ab714e35d9e0f95b56a4355b3b26c2927b07d2bfa3`). A new eligibility-only invocation could bind the exact original metadata row and unchanged native source, prove complete frame rewriting separately, then check unique explicit response/marker/future context, exact accepted turn, same-turn full input and compaction lifecycle. Its public index keys inputs by `(thread, turn)`, so the repeated text in RPC `64` must remain a distinct occurrence. No `audit_run`, counter adjustment, reordered stream or broad future-context inference is needed to publish that source qualification. The existing function was read, not invoked here; eligibility is not assumed from resemblance to an earlier case.

Full delivery/source provenance remains owned by the separately reviewed C08 wrapper. Neither qualification alone changes the two original unknown results or proves a saving.

## Source identities

Paths below are relative to the phase unless absolute. Let `O1` and `O3` denote `runs/automatic-refresh-01-workflow-00N/automatic-refresh-01-workflow-00N-three-feature-bench-artifacts/observation` for N=1 and N=3. `C1`/`C3` denote their app-server directories named above. Endpoint checks `3c7804` match the original consolidation pins for all referenced analysis, rollout-metadata, client/forwarded/server and native files. `rollout-audit.json` is separately inspected retained metadata, not falsely described as an original accounting-read endpoint.

| Source | SHA-256 |
| --- | --- |
| `postcapture/automatic-refresh-01-workflow-001/COMMON-ACCOUNTING-ORIGINAL-FULL.json` | `bf554e48d74cba26600a2d8e08dd3fe4be69f19b994ab7e1795a5cbe272553f2` |
| `postcapture/automatic-refresh-01-workflow-003/COMMON-ACCOUNTING-ORIGINAL-FULL.json` | `a05bd96ca91935d26d2d2d156a836cfe5ccd5e5df4eef761f654861d21d18c44` |
| `O1/analysis.json` | `f21ee4984c9bad1db4f6fdd5ce84ec5b9dde07da8b5fd45c72c365b230181990` |
| `O1/rollout-metadata.jsonl` | `fcaa0318b2877b4c89b49dd9a968f262aefa5c5e3f51c1b6c07f645ea33b1f0c` |
| `O1/rollout-audit.json` | `891a1db7c38c723633c7a1609df00216a23c2719e0bc0bcf4ea197dc9aa60083` |
| `C1/client-to-server.raw` | `46658e1f97c1e0faa0b703792106095aac285a3307afd88630a50ff0ef63ab3f` |
| `C1/client-to-server.forwarded.raw` | `98c370cd7e10ca4dde8a5854b9ed98a4399245ee2e7ad11f4942335dc2faf064` |
| `C1/server-to-client.raw` | `54ba8ad16cbd62081edc805a97f891dfc6f0e9dbda422569cec5e5fce6165a87` |
| Absolute native 001 file above | `b3ed6e43a5cabcb055e1b48b6d2b5ce53a457b4118f506c7f519a9778d84df8c` |
| `O3/analysis.json` | `0a63ef36046888d40501c8a3f923840d13fbcc2176788065b4e3be302edb670d` |
| `O3/rollout-metadata.jsonl` | `8f712bba09775bf703b262881f0a59045cb7ce2f5ba1641fa02898092a6c3245` |
| `O3/rollout-audit.json` | `d8d98b3db0e478ddf1054497a7e451598b290712a9df8f835e1da9380d80036d` |
| `C3/client-to-server.raw` | `df92b39c1fb718436814f05afa0d938e12ab42c18535e5f4ac95d526f7408768` |
| `C3/client-to-server.forwarded.raw` | `bae925acb43c825d8043f5b81af8768493522871d9a115e585148aa6937b4686` |
| `C3/server-to-client.raw` | `a8a9d823054adb3c8d0b7a347559a7710d39730540799da3c609c93052e1d36f` |
| Absolute native 003 file above | `c50fdac72aaaf558fe378e4a572b06910a6acc1b6c9a6fedddc100c09802b6b6` |

Frozen code is under `infrastructure/evidence/`, with each repository-relative path preserved: `bench-observer/src/lib.rs` SHA `188a1b4fd9913556c51024dcc0068be353813c44c688d6d22929da139f392b5f`; study `accounting_untracked_reads.py` SHA `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`; study `audit_compaction.py` SHA `dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153`; `bench-results/efficiency-measurement-gate-20260906/batch_analysis.py` SHA `dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3`. Their phase pins match; the inspected current observer source is byte-identical to that frozen copy. Tool locators `344b3e`, `3c7804` and `073677` retain the finite metadata/input comparisons, not private provider reasoning.
