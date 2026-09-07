# Saved lifecycle and continuation evidence

## Scope and disposition

This Stage B pass covers C24–C30, C33, C36–C38 and X01–X07 in
[CANDIDATE-MAP.md](CANDIDATE-MAP.md), with a C32 protocol-correction cross-reference.
It is an exposure and source-identity inventory, not a token percentage comparison.
The parked R read-factor costs are neither read nor analyzed. P, S and W remain distinct
cohorts; W/010's workflow failure remains in every table.

The strongest exercised differential here is **C24: incremental WL follow-ups versus Direct
resupply of the original request and role-specific instructions on resumed review/fix turns**.
Separate persistent author/reviewer threads are shared, not a WL-only saving mechanism.
Titles are an exposed WL overhead; natural-language command-agent routing, explicit agent-to-agent
send, known-session policy reinjection, and captured session restart are unexposed in P/S/W.
Interruption and compaction require separate generation and accounting treatment.

The source baseline is P's saved tree at commit
`41b6418ef420fbce6aab93706657bd77dba3ce51`, with the historical source-equality qualification
in the map. P/S/W's requested/native model is GPT-5.5/xhigh; provider versions and phase policies
are not retroactively treated as identical to every H observation.

## Method and complete retained population

Read-only extraction uses the P first-batch manifest and S/W score manifests, each workflow's
single primary app-server invocation, complete raw client/server JSONL, interrupt-grace journal,
and observer native-rollout metadata. Native source bytes are rehashed against that metadata.
Only public user/assistant/tool type metadata, typed RPC identities, thread/turn/item/response IDs,
safe effective-setting fields and lifecycle records are inspected. Private reasoning bodies and
credentials are not interpreted or reproduced.

A call below means an accepted `turn/start` request, joined to its exact typed string RPC reply
and returned turn ID. It is **not** a completed upstream response, ACK, native tool call or
token charge. Roles come from the first renderer-owned Agent-ID/Feature/User-prompt header of the
initial delivery, never an embedded header copied inside review evidence. The scan finds no
unaccepted/unreplied turn/start request. Every one of the 16 WL workflows has exactly one retained
primary app-server invocation, one app-server capture overall and a terminal invocation receipt.

The grace column is exact-usage / output-resumed / timeout **decision counts**, all under the
unchanged 1,000-ms maximum and `forward` policy. It is not a missing-token count or a claim of
whole-workflow accounting completeness.

| Workflow | Threads | Accepted turns | Author 1/2/3 turns | Reviewer 1/2/3 turns | Title turns | Linearizer turns | Fix/recheck deliveries | Grace decisions |
| --- | ---: | ---: | --- | --- | ---: | --- | --- | --- |
| P/WL | 8 | 76 | 21/7/25 | 7/4/7 | 3 | 2 | 7/7 | 58/4/0 |
| S/001 | 8 | 98 | 31/9/42 | 4/3/4 | 3 | 2 | 4/4 | 49/38/0 |
| S/002 | 8 | 77 | 28/6/20 | 10/3/5 | 3 | 2 | 8/8 | 58/4/0 |
| S/003 | 8 | 59 | 20/7/18 | 3/3/3 | 3 | 2 | 3/3 | 31/18/0 |
| W/001 | 8 | 43 | 13/7/9 | 3/3/3 | 3 | 2 | 3/3 | 32/1/0 |
| W/002 | 8 | 45 | 14/6/9 | 4/3/4 | 3 | 2 | 3/3 | 30/5/0 |
| W/003 | 8 | 91 | 57/6/7 | 10/3/3 | 3 | 2 | 9/9 | 43/31/1 |
| W/004 | 8 | 58 | 11/7/22 | 3/3/6 | 3 | 3 | 4/4 | 41/5/0 |
| W/005 | 8 | 57 | 17/7/18 | 4/3/3 | 3 | 2 | 3/3 | 44/3/0 |
| W/006 | 8 | 46 | 14/5/13 | 4/2/3 | 3 | 2 | 3/3 | 35/1/0 |
| W/007 | 8 | 56 | 15/8/13 | 7/3/5 | 3 | 2 | 5/5 | 41/2/0 |
| W/008 | 8 | 55 | 13/9/12 | 9/3/4 | 3 | 2 | 3/3 | 36/9/0 |
| W/009 | 8 | 55 | 18/6/14 | 5/3/4 | 3 | 2 | 4/4 | 41/3/0 |
| W/010 | 6 | 68 | 24/7/20 | 8/—/6 | 3 | not reached | 7/7 | 53/3/0 |
| W/011 | 8 | 51 | 14/7/12 | 5/3/5 | 3 | 2 | 3/3 | 35/6/0 |
| W/012 | 8 | 64 | 10/11/23 | 4/4/7 | 3 | 2 | 5/5 | 47/5/0 |

P/WL = `work-leaf-001`; S/NNN = `screen-01-NNN`; W/NNN =
`work-units-01-workflow-NNN`. W/010 never reaches reviewer 2 or linearization; missing roles
are not zero-work successful stages. The other workflows have three authors, three reviewers,
one title agent and one linearizer. There is no additional coordinator/planner provider call
on top of this inventory.

## C24: exact incremental-versus-resupplied context exposure

The saved Direct driver has `run_feature_cycle → run_direct_agent_resume`.
`fix_prompt` repeats the original request, role constraints and `normal_validation_guidance`;
`review_prompt`, including each resume, repeats the request, base/scope and review contract,
then adds the prior author's new evidence. The reviewer template does **not** contain the
author's focused-validation paragraph.

P/Direct contains 16 CLI task invocations in seven persistent native threads: three author,
three reviewer and one final linearizer thread. All 16 saved `runs/*.prompt.txt` files match
their corresponding native user-message text byte-for-byte. Four author-fix and four reviewer
resume deliveries exercise resupply; the ninth resumed delivery is linearizer acceptance.
This is not a claim that Direct opens a fresh conversation on every fix.

WL's actual `CommandChat::review_commit_streaming_with_ids` supplies the new findings with
a fixed fix/evidence-resolution wrapper, then the author's new reply with a recheck wrapper.
`CodexBackend::send_streaming_interruptible` uses that raw prompt when a session exists.
Across P/S/W, every first policy envelope belongs to the role's initial thread and no subsequent
accepted turn begins with a re-injected launch-policy envelope. Copied evidence can still contain
old text: this excludes a new provider policy injection, not all duplication inside review content.

Concrete P identities (physical JSONL line numbers):

- WL author fix: primary client line 38, RPC string `37`, thread
  `01a07825-2392-7d82-983c-78f2b5be16c9`, accepted turn
  `01a0782e-ab7a-73c3-a93c-c1cba680a435`, server reply line 20877.
  Exact delivered text: 1,974 UTF-8 bytes,
  SHA-256 `e41ed0b55165877e405cbf38a967fe241f09e00cf01702eac1f92fc788d47f89`.
  It matches native user item `msg_01a0782e-abb7-7c73-8e6e-c4b3b26c9f81`, line 255,
  in the same explicitly identified native turn.
- WL reviewer recheck: client line 40, RPC string `39`, thread
  `01a0782b-8100-71f1-a935-ca076774eabb`, turn
  `01a0782e-bf3f-7aa0-8d2b-61035d91282d`, server reply line 21457.
  Its 594-byte text SHA is
  `2035d0e983ddf549c3d6af93cef6300bcdb945200107a249b50f3962c679c865`;
  native line 191/user item `msg_01a0782e-bf6a-7832-a677-da81fa76775d` is identical.
- Direct feature-1 fix: saved `sequential-feature-1-fix-1.prompt.txt`, SHA
  `830a7f25d24dd349cc3619be8490ab352a0595b5ef84dc74848e71d802300101`,
  matches native line 618/user item `msg_01a07839-7235-7b92-afd2-dc2c0a9f3783`;
  thread `01a07825-1052-7cd0-a87e-507552c13d67`, turn
  `01a07839-6bc3-7c01-9ac9-c241de3142bd`.
- Direct feature-1 reviewer resume: `sequential-feature-1-review-2.prompt.txt`, SHA
  `eb16dbe838b5b40eea016fcf301fe2549bb239636233d5ae107892ee631c2109`,
  matches native line 246/user item `msg_01a0783c-4225-7c62-aec8-4f3f68afc545`;
  thread `01a07833-e768-7030-a8ae-2b7c5237fa32`, turn
  `01a0783c-3b6b-7900-8d2c-31d57157b67f`.

These are examples of exact delivery, **not matched semantic-work pairs or a byte-to-token
conversion**. WL can send substantially more evidence too: P's recheck RPC string `91`,
client line 92, sends 16,994 bytes to reviewer 3, SHA
`5ad242a14ac4021b004238e76ecc1393a03138803ad8aa3304b0ae47045e6fd4`.
Therefore “raw follow-up” does not mean universally small follow-up.

**Smallest directional factor:** at actual author review-fix delivery only, resupply the exact
original feature request once, using owned launch state. Keep findings, resolution wrapper,
new evidence, thread identity, review schedule and every non-target prompt unchanged.
Do not re-inject AGENTS/full policy, add validation rules, resupply on every ACK/read, reset
sessions or pad to a desired size. This isolates one exercised component of C24, not the whole
Direct template. Observe exact appended text/native-item exposure and downstream responses;
net effect and repeated-input amplification are subsequent outcomes, not assumptions.
The existing P/S/W fixation exposure supports this narrow candidate without inventing extra tasks.
A run with no review-fix exposure is retained as unexposed, not replaced.

## C25/C27/C37: generation, publication and restart are different

### Interruption

`DirectiveStreamInterruptDetector → CodexAppServer::request_turn_streaming/request_interrupt`
returns the directive to WL without waiting for provider terminal completion. The observer's
1,000-ms ceiling can release earlier when output resumes. Every P/S/W workflow has at least one
such output-resumed decision. This does not establish how many tokens interruption avoided.

P's grace line 29 is `forwarded-after-output-resumed` at recorded wait 0 ms, thread
`01a07830-f0f9-7301-abbf-ee60f8ae9377`, turn
`01a07830-f16c-7ad2-b896-889ba67052b3`. Its public read-directive item
`msg_056b6b9b26cbfc93016a9dbec1c1f487d289ea83b6ffdb5691` is at server line 25193;
terminal `interrupted` is at 25198. There is no exact raw completed-response record for
this turn in that closed capture. The neighboring grace line 31/turn
`01a07831-274a-7de1-b368-44f60b499ffe` has multiple earlier exact response records;
those do not prove usage for its interrupted final response.

W/003's grace line 31 reaches the 1,000-ms ceiling, thread
`01a07949-ee9c-7503-a486-0108cc6bb00d`, turn
`01a07968-8cdc-7aa3-a981-3cc59eba0745`. Public edit item
`msg_06dd34dd4bcd978a016a9e0e7e20fc87d299897df5d1fd4f3d` is at server line 60599,
terminal `interrupted` at 60604, with no raw completed-response record for that turn.
Elapsed waiting alone is not a completion/usage proof.

Older saved checks separate two propositions:

- [Normal smoke](../efficiency-measurement-gate-20260906/NORMAL-SMOKE-RESULT.md):
  unchanged 1,000-ms/forward policy releases at 84 ms; first turn is interrupted without exact
  response usage, same-thread follow-up completes with an explicit response ID.
- [Immediate-interruption record](../efficiency-measurement-gate-20260906/TELEMETRY.md):
  CLI 0.153.4 emits an interrupted terminal event but no exact/cumulative usage through the
  recorded 60-second post-ack interval.
- [Historical wait-for-usage attempt](../efficiency-point7-exact-accounting-20260828T113610Z/FAILURE-ANALYSIS.md):
  two review handoffs time out under a different up-to-30-second policy. Its failed workflow
  is retained; it is not a normal-WL or paired causal saving observation.

Thus C25 is exercised and may affect generation, but neither “interrupted means zero usage”
nor “one extra second always publishes every response” is supported by these captures.
No timing or native-tool-protocol change is proposed as an accounting fix.

### Compaction

There is no client `thread/compact/start` request in P/S/W. Explicit native compaction
markers occur once in P/Direct and once in W/003; none occur in P/WL, S or W's other 11 workflows.

- P/Direct author 3 native line 838 has named response
  `resp_0b59e72c1bec5db3016a9dcce4c60c87d29322821e8d60b29f`.
  Native source SHA `45ac06df86e00002dce1e2d103500e6d275bbf7679750852d7e57056741bcebb`.
- W/003 author 1 native line 624 names
  `resp_06dd34dd4bcd978a016a9e11d96e9487d2a73f239eca987fc2`.
  Matching app-server context-compaction item
  `01a07975-b799-7ad0-a97d-9b3d6ad0d2a0` is at server line 63488 in turn
  `01a07975-91c7-71c0-9c81-344caa91ef43`.

W/003 also has the separately documented unusable/nonadditive `tokenUsage.last`
notification at line 63486. That is not itself a billed completed response or proof of a provider
context-size-estimate contract. The frozen accounting failure, named compaction scope and
unsupported interruption gaps remain separate facts. No old report is retotalled.
See [supplemental readiness](phases/work-units-01/coding/operational/SUPPLEMENTAL-AUDIT-READINESS.md)
and the preserved W compaction report.

Compaction is an exercised provider-context mechanism, not an established WL-specific setting.
Its response cost and its effect on later input must be treated together.

### Restarts and retries

The P/S/W client inventories contain only initialization, thread/start, turn/start and
turn/interrupt methods: no thread/resume, explicit compact, fork or restart command.
One app-server invocation per workflow and one first policy envelope per role support
**no observed transport/session restart or missing-session policy fallback in these captures**.
No native error-event marker or changed native-source hash is found; raw completed-response IDs
are unique within each workflow.

This does not prove that a provider has no internal HTTP retry invisible to the retained
protocol. Different response IDs alone are not evidence of retries. Pre-generation ETXTBSY
spawn retry handling in `spawn_app_server_process` is not a generated response.
No restart/retry factor has eligible exposure here; do not manufacture failures to fill a slot.

## C26/C28–C30/C33/C36/C38: roles, settings and coordination

- **C26:** native turn-context settings are GPT-5.5/xhigh/approval-never throughout P/S/W.
  WL authors/reviewers/title use read-only; linearizers use danger-full-access.
  P/Direct authors use workspace-write, reviewers read-only and linearizer danger-full-access.
  Public native tool names observed are `exec_command` and `apply_patch`; W/001 additionally
  records `request_user_input`. W/010 only records `exec_command`.
  These are observed tool-name sets, not a complete available tool catalog. Native model equality
  does not establish identical tools, injected instructions, sandbox policy or hidden transport
  request overhead. The exact available/requested tool-schema comparison remains unresolved;
  the next safe check is retained public request metadata/schema, not new provider generation.
- **C28:** the actual controller path is
  `start_linearize → prepare_linearize_launch → launch_prepared_agent_streaming_with_ids`,
  followed by accepted-plan send. It uses interruptible helpers despite direct filesystem tools.
  The standalone `LinearizePlanner::launch_linearizer` is an alternative API, not another
  benchmark call. Fourteen P/S/W workflows have two linearizer calls; W/004 has an additional
  protocol correction; W/010 never reaches the role. Direct P has separate plan and accepted-plan
  calls in one persistent thread. Both retain final documentation/check/history responsibilities.
  Work deferred into integration is not a saving until the whole role is counted.
- **C29:** P provides concrete overlap: author 1's first turn is accepted at server line 7 and
  author 2's at line 17, before author 1's terminal at line 60. These are separate threads,
  `01a07825-10a9-77b3-bd6a-f75800663294` and
  `01a07825-2392-7d82-983c-78f2b5be16c9`.
  Feedback/rejection exposure is present in the call inventory; separate behavioral evidence owns
  whether each was a stale edit, ownership gate or genuine external blocker. Host lock waiting,
  status messages and three top-level workflow capacity do not themselves consume model tokens.
  A scheduling factor would change shared-state exposure and needs to retain locks and task scope.
- **C30:** each WL workflow has three actual title calls in one persistent title thread, including
  W/010. P's title thread is `01a0782b-80e7-7401-897c-f604f237763b`.
  Its resumed call RPC string `55`, client line 56, joins turn
  `01a0782f-9e7e-7191-9bad-dab68ace6804`, native user item
  `msg_01a0782f-9ec9-7b73-afcb-5e0da35b79ec` at native line 19 and completed response
  `resp_0463d806d5d96eae016a9dbe5e854c87d2b71043a9760b8de1` at server line 23933.
  Direct P's seven-thread inventory has no title role. This is real auxiliary work, an offset,
  not a reason for claiming lower WL generation.
- **C33/C36:** no role header identifies a command-agent, no accepted follow-up begins with the
  owned command-agent interpretation template, and no accepted follow-up has the actual
  `Message from {agent_id} about {feature}:\n{message}` routing envelope.
  Host-parsed new/linearize commands and reviewer-fix routing are not these separate APIs.
  Natural-language command routing and explicit inter-agent send remain unexposed here.
- **C38/C32:** no accepted P/S/W delivery begins with
  `render_linearize_completion_required_prompt`'s completion-required text.
  The one extra W/004 linearizer exchange is instead the **387-byte generic protocol correction**:
  client line 112, RPC string `111`, thread
  `01a079b6-a60e-7e10-914d-e5b8b3f2e317`, turn
  `01a079b8-b321-7822-b2fa-e585627c3dcc`, server reply line 71828.
  Payload SHA `65e08806a33fa1c024d0181924f8c303144875c84098d6df38a06b81811b7a23`;
  completed response `resp_0f5f1f17c036a8c5016a9e2301516887d28a60c9100bf81df8`,
  server line 71896. Do not label this as the absent post-accept done-correction factor,
  or suppress a real protocol failure to reduce counts.

## H metadata cross-check and exclusions

H's six WL observer role inventories all contain the same eight roles as successful P/S/W
workflows, including title-agent. Their report headers identify app-server/GPT-5.5/xhigh,
CLI 0.150.1 and benched binary commit `5b1d1ef9590850faed26052f909ddff7ff8f127d`.
Five retain missing-interrupted-response diagnostics; one is marked capture-complete.
Those are original measurement dispositions, not lower generation.

Four H Direct report/observer pairs checked here contain seven persistent role threads.
The point-7 row reports CLI 0.149.1; three later step-4 rows report 0.150.1. Two other Direct
rows resolve through the earlier points-8/9 aggregate evidence rather than a per-run header in
this pass; their individual effective settings remain unexpanded. H metadata is not used to
claim that P/S/W's zero-exposure or exact call counts hold for every H workflow.

| Exclusion | Evidence disposition |
| --- | --- |
| X01 | Separate feature/reviewer contexts and within-role resume are shared; exact P Direct user-item joins confirm this. WL's one app-server is not one shared conversation. |
| X02 | Direct already supplies focused checks, host commits, persistent review and plan/accept linearization. Exact cadence/packaging, not their mere existence, remains differential. |
| X03 | Retained/cached input charging amplifies delivered context. Caching is not a raw-token discount; repeated charges are not repeated generation executions. |
| X04 | All original usage gaps and W/003 arithmetic/scope diagnostics remain. Terminal completion, raw response identity and usage publication are separate. |
| X05 | Display compaction, status polling, locks and local final-gate execution are not provider-input reductions without a delivered-message join. |
| X06 | Native settings and H version differences above are comparability evidence, not interchangeable cohorts. No account/provider/configuration changes occur in this pass. |
| X07 | The role/RPC census excludes additional public coordinator/planner, command-routing, fork/promotion/dependency and Claude calls from the observed inventory, not from general WL capabilities. |

This completes the named candidates' saved lifecycle exposure pass. Remaining unknowns are explicit:
full provider tool/schema/hidden-request equivalence; invisible internal provider retries; causal
net effects of packaging, interruption, compaction, scheduling and integration. Their next checks
are exact saved public request/schema or isolated exercised boundaries, not an automatic rerun list.

## Source locators and identities

For P/WL, the artifact root is
`../efficiency-raw-token-pilot-20260906T185328Z/runs/work-leaf-001/work-leaf-001-three-feature-bench-artifacts`.
For S/W it is
`phases/<phase>/runs/<run-id>/<run-id>-three-feature-bench-artifacts`.
In the following table, client/server are
`<artifact>/observation/app-server/<invocation>/client-to-server.raw` and
`server-to-client.raw`. All SHA values identify the closed bytes read in this pass.

| Workflow | Invocation | Client SHA-256 | Server SHA-256 | Rollout-metadata SHA-256 |
| --- | --- | --- | --- | --- |
| P/WL | `00000422153032163608-2899396` | `3be1a143ed5f66960daa92816800a8ac1ec63ae9af9ed43f1eca93ba90ad7ece` | `701a611f120e7718f1ebabfac8d4d676a0ab76da237c8c3201d0b2b76a65eb9f` | `d4692c2fdfe8639bacabd931dae54886139fe45a9bf74a1110b2d02c92e80e9e` |
| S/001 | `00000433355029717882-3253359` | `d16bb323ac97513015edbee59a8a0596068e0877520833107fd41e1ffe0244d0` | `1282fe6fa7d051c570ff465cc9f839acba3c3c37b4402f9f783491c1f68e714f` | `bbbd9795916980dc46cc8438fd690e47bc807a128ff59bd145342fe801147464` |
| S/002 | `00000433355029374750-3253355` | `c7ad3fa9465d8de5cdf10bbca7a76f3b8a75be72e6361879d904241685df9719` | `a6a22f5efb21e85c218df5c2b3c53ee9f316cdc774fb29e1429a77421e56da52` | `226761369e49ac0e7333d942414e40a2398bdfd8fe6d4e82181a15b3d23f0f87` |
| S/003 | `00000433355029867482-3253361` | `23ed496344322387b6c251beced0e7e4df86bfe2fe3bd0789318129d12ea57a1` | `edb12fac72ea51860524a4e8391ca076e0317af62348b5d970c3d9d34d526ed2` | `579534a4c8a3928bbc4f2a667f1a508302d50e99f8d19e3f8108f8f1cbc2d6cc` |
| W/001 | `00000441344929586777-3537090` | `748135038446e5397f8514c0e0a70609ff1020b1070e64d227c97d096c011d9d` | `f050910cd73200db4ee4bfb9ee63080ca20c42bddd253d467f34f3177550e243` | `b9d4a48f6d17a4a4f245037b345a9b70f15cbc65364cc882cad832c40ae01286` |
| W/002 | `00000441344930516444-3537099` | `53c7a7b31a9cd648d4eb5ce260ed3f2031b15d98d8311a3ad54b810703aa090e` | `a152470a637b44655803bfd0dc4a6ba128c32104b1e8e5a6f76cef0790f55951` | `aebc1b75eaaef16918eeb890273159977a380fa587703e396b16b9967c6b0918` |
| W/003 | `00000441344929587964-3537091` | `191f6d8611d77005d77728462a2cb0a5a1dfcecf8347ecc9fb5d0b6e24e3e025` | `2f6949270f63a69423eee95ee0f7f866169c04cbffc027c9926862cca6909009` | `95dec4d922346f4f784b018cae7d914b1331c3ba16bd0ee2909dcd1d8f49542f` |
| W/004 | `00000446260578462469-3719532` | `3a4cca10e1f1a24d139a4bbb2c1e2fae7e8a8b06267900a7e0a08cdc2130e4ff` | `2fca9c847c064306ac995712f483b67448966ba9b3e78c82a8ac93949288725f` | `f07cf28d3140784e979a852eecc4ec2ff567a00cd317976b20ecdb92879e6f43` |
| W/005 | `00000446260581883146-3719549` | `0ceaabfa316a0089b1eee2f503d4aa897295f47009581a9e1931192b5f39e55b` | `21ea08aea1cdc94c109e4e2446db90cc9b298d2cb951a77ca1bcb85533676414` | `060ae62e2c13c994efeac7d9478217c3a7f33907e3cef3a377b197337ddd4e4a` |
| W/006 | `00000446260578466031-3719533` | `d1891ed7420f705c02b266c2b2254747b63e2d0cc4cea3b092dbe3c5230f4d8d` | `60d872168e1077b321262f8df8f93e4a1c197d961e3098333d1b10d53744cfc3` | `462e15eb2b326527a1588b911b1c7a4dca115b988a96a822c342318ceaac244a` |
| W/007 | `00000448924886790377-3844443` | `c704dccf5fa07e5f1b7f68be3697a31ee07cfd5ae167072747d73ddb38846bb7` | `ebc0e042336d8b722e8c34df555d8ee071ae8a224659d97f3334f51b157128a6` | `e48ce40c4da1d38a4887e1d092d8bcc50643d793d17e09b408218a1ef4640676` |
| W/008 | `00000448924902156465-3844489` | `14d5026a3abd0f508992c096b425b4b44256e876ad6fef69363819f7accc30d2` | `110d43e43feb58b7408478c3f39d6e1288a3621b2da39495e26ec8d910b7419c` | `a1767fbf6b4fd3a859004bfdc5984c6c86bf74eb75d8e0220082be5d67809efb` |
| W/009 | `00000448924886506612-3844440` | `df85039a73a9ab64b03fa2195b76cebb9baab2701a1a97701bba7acf3a113f4d` | `60535906551d65a45091b6d97c466d882d86521be23d13e39f0e12db84f3363c` | `0e0971cdd91d85572209e64ae7a573ce9afe8f878723796d5f31b2d1c47c9a7c` |
| W/010 | `00000452179764996037-4005148` | `6ec02b6c28f91bce68e2cb5433edfb1cfdb9cb9e4327b37bdfa8de8ea6d07fd0` | `30445cc6959e4fe8b8b0dd81cdac7eddfe0167eb232aea7aa43916745f7c0f87` | `a2e68a51965d0137a44542a2e9b1c7cc38f0c1ce79a849e91fa34a1e357988de` |
| W/011 | `00000452179770293816-4005169` | `f14c383551536cdfee8b02f5875a674cc9be2a26050f17f509f8eac1273cc708` | `cbb8f6580c03be7f8f8bbdf139fe25e1af7c5934b46894024a450db4b716a247` | `7ed329b32ae719efad9ae19f1edbb3e2f3ae4ec816f5f9b3ed76cf047fc6a341` |
| W/012 | `00000452179765332046-4005151` | `7fd19919bc8e25cf5e31d6605eb06029e11e3610552fdc7bd3861d71dbd6640b` | `2b8abf5678ef584aa2d053bdb2d5bf597901fb851054e546856c65d4bfc96493` | `4e8face36ef75bd93d70e109c0e43b160b5994f39ca68a9e7d99c4f5a80379d0` |

Each rollout-metadata JSONL is at `<artifact>/observation/rollout-metadata.jsonl` and binds the
native thread, original sessions-relative path and native source SHA. Every referenced native
file passed that equality check. Exact native example paths and full-file hashes:

- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-14-54-01a07825-2392-7d82-983c-78f2b5be16c9.jsonl`: `a5d48cfb9de2529efd7cfa458969fd3836ab2aee836a1f295e1d3e8f181c2691`.
- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-21-52-01a0782b-80e7-7401-897c-f604f237763b.jsonl`: `a09121a0d90285786e86c5c68b3d5662f2d3672b3364e1582b5dfd97e681fefb`.
- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-21-52-01a0782b-8100-71f1-a935-ca076774eabb.jsonl`: `4466b583555f74cc83dac00dfc69d7803ca4112b96a029abce7bf5e73baed68c`.
- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-14-50-01a07825-1052-7cd0-a87e-507552c13d67.jsonl`: `301f3c9d4b3b8ea9bf063bfa549f8ad1767e40757b05ba7abb41827f94b3474f`.
- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-31-02-01a07833-e768-7030-a8ae-2b7c5237fa32.jsonl`: `b6b424afe8fb0c265c6475732da35f54732096f1d6520c7d1728af507ea01e51`.
- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-42-08-01a0783e-120a-7480-8659-cd65cfb1eb5c.jsonl`: `14f5959eaa66281bf97e49176fbfa2e2f78944604b9489c73d21a155abaf7d8d`.
- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-50-03-01a07845-5050-7a72-854d-77422792fc8d.jsonl`: `93290f27d14b9ffc09a7eea39336a340d3d730fa9c2aa4db031082b46c83363c`.
- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T22-00-16-01a0784e-abd2-7803-a0b5-e83fd25653a8.jsonl`: `45ac06df86e00002dce1e2d103500e6d275bbf7679750852d7e57056741bcebb`.
- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T22-18-43-01a0785f-8e68-77b3-bf39-c8125edf1e14.jsonl`: `c1f9b93ac21c83f8c2e98b92408e69eeb82b56c7abe31910b53721fd4327f6c1`.
- `/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T22-48-26-01a0787a-c2e6-7b52-aa52-14c14803b153.jsonl`: `c8129196bfd552438d9e45c7b9e00e0b3f485e114051f475d5693fc5de3df026`.

Interrupt grace journals use the same app-server directory as client/server. Example journal hashes:

- `work-leaf-001`: `418be3775d695528208342c62ff66abbc1ea440f874482159dede7650c50a738`.
- `work-units-01-workflow-003`: `13edd26fbf0f12ee84faaa22ef4427186c547bf526829c31473d7196f5e35bf1`.

P Direct prompts live under
`../efficiency-raw-token-pilot-20260906T185328Z/runs/direct-001/direct-001-three-feature-sequential-bench-artifacts/runs`.
The remaining exercised resumed prompt identities, in addition to feature-1 examples above:

- `linearize-accept.prompt.txt`: `78122a39e19eacc045addd976eabf1373ac12d53663ef285b125644bc6ed0546`.
- `sequential-feature-2-fix-1.prompt.txt`: `5a213185780e64869570552e045192fcf4750bed0313256de948f8a8142b7a59`.
- `sequential-feature-2-review-2.prompt.txt`: `c04f951b127688a6310c81f31febfed825bbe65b66ba8e105ea30d96e2eb0d7f`.
- `sequential-feature-3-fix-1.prompt.txt`: `1f865f9da23dd4ef38eaf1fb9f8b6d138f8a971dc7e5ba4ac2f7f496b376f366`.
- `sequential-feature-3-fix-2.prompt.txt`: `ee130e32ed54df5125c83cc0337438918d42141ee23a1824226d039d37b45a04`.
- `sequential-feature-3-review-2.prompt.txt`: `fd55f8c02c9b5175af4232c9fdf13eeec68d074797016382995b61727ec5ec17`.
- `sequential-feature-3-review-3.prompt.txt`: `1a9f83ea4325ed32d026309b4439f9ccad9ad01fbe7061b7ba5f5d1d0517364c`.

## Verification limit

This report is provider-free read-only evidence work. Only this new evidence document is written;
no runtime, saved observer report, classifier, accounting helper or frozen cohort is changed.
No real-agent workflow is affected or newly invoked. Whole-workflow accounting remains owned by
its frozen reports; no token retotal, new comparison or causal percentage is produced here.
