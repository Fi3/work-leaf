# WL token-saving candidate map

Status: broad source/evidence inventory, not a list of proven savings or a percentage attribution.
The [candidate-first plan](PLAN-CANDIDATE-INVENTORY.md) governs continuation. Existing read-factor
treatments are parked; this map does not analyze their costs or admit any benchmark.

## Evidence vocabulary and scope

- **Source candidate**: an implemented boundary can change input, output or response count.
- **Exposed**: retained public actions/deliveries exercise it in the named cohort; not causal proof.
- **Local mechanism**: exact delivery/charge evidence establishes a local cost pathway, not net savings.
- **Not demonstrated**: the stated isolated test did not establish its intended effect, not proof of zero.
- **Not exposed / shared / accounting**: a cohort-specific exclusion or a non-causal explanation.
- **Parked**: already collected variant data remains intact, with analysis deferred.

No entry receives a demonstrated whole-workflow saving label merely from source code or an episode.
The source inventory covers candidate reductions and offsetting overhead. Direction can differ by
workload; it is important to retain mechanisms that might increase WL's usage too.

Evidence keys below refer to these saved records, not a freshly pooled sample:

| Key | Population and evidence |
| --- | --- |
| H | Accepted earlier six-WL/six-Direct endpoint: [report](../efficiency-exact-normal-work-leaf-20260829T181318Z/FINAL-REPORT.md). Its accepted reduction is not a factor attribution. |
| P | September 6 two-workflow pilot, distinct from H: [episodes](preflight/HISTORICAL-EPISODES.md), [command-output census](preflight/COMMAND-OUTPUT-MECHANISM-EXPOSURE.md), [saved source](../efficiency-raw-token-pilot-20260906T185328Z/source). |
| S | Three-workflow narrow-cue screen: [state](STATE.md), [delivered ACK chains](phases/screen-01/ACK-CYCLES-DESCRIPTIVE.md). Neither cue qualified under that screen's rule. |
| W | Twelve-workflow work-unit phase: [independent mediator review](phases/work-units-01/PHASE-MEDIATOR-REVIEW.md). The declared handoff test did not establish its proposed increase; all failures remain. |
| D | Bounded real read-delivery diagnostics: [review](preflight/READ-INLINE-PRIMARY-DIAGNOSTIC-REVIEW.md). Exact local item charges; tool/bundle retrieval was deliberately prohibited. |
| R | Parked six-outcome read-factor phase: [terminal result](phases/untracked-reads-01/PHASE-RESULT.json). No completed factor cost comparison is asserted. |

Paths and symbols in candidate entries are repository-relative. Source inspection uses HEAD
`e2a0d00` before these documentation commits, including private benchmark hooks. Normal-mechanism
claims are cross-checked against P's source commit `41b6418ef420fbce6aab93706657bd77dba3ce51`.
The inspected normal sources at historical WL commit `5b1d1ef9590850faed26052f909ddff7ff8f127d`
and that P commit have no Git differences for `src/{agent,cli,codex,review,linearize,workspace,
instructions,locks,orchestrator}.rs`, `bench-three-features-direct-common` and
`bench-agent-profile-common`. This binds source behavior, not every earlier Direct invocation.
Earlier provider versions differ; P/S/W exposure is not automatically H exposure.

## Source-coverage matrix

| Boundary | Inspected owner/call chain | Candidate coverage |
| --- | --- | --- |
| Launch and ordinary continuation | `src/agent.rs::PromptPolicy`; `src/instructions.rs`; `src/cli.rs::launch_agent_streaming_interruptible`; `src/codex.rs::CodexBackend` launch/send methods | C09, C24–C27, C37–C38 |
| Mediated and native reads | `handle_agent_directives_streaming → send_file_read_response → read_requested_files → render_file_read_response`; `FileReadTracker`; `ContextBundleStore` in `src/orchestrator.rs` | C01–C06, C34–C35 |
| Read/refresh/result representations | Repeat and refresh renderers; `render_command_output`; native tool-result evidence | C02, C06–C08 |
| Checks, blocked commands and patch acknowledgments | `src/agent.rs` policy; `run_command_for_agent`, ownership tracker, result/ACK renderers in `src/orchestrator.rs` | C10–C13, C17–C20, C32 |
| Edit application and repair | `src/patch.rs::GitPatcher`; accepted/rejected directives and pending changes in `src/orchestrator.rs` | C14–C19 |
| Review and evidence handoff | `src/workspace.rs::start_review_for_agent`; `src/review.rs::GitHistory` and `render_review_source_context`; `src/cli.rs::review_commit_streaming_with_ids` | C21–C23, C31 |
| Integration and final history | `WorkLeafController::start_linearize → CommandChat::prepare_linearize_launch`; `src/linearize.rs` prompt/target assembly; Direct plan/resume prompts | C28 |
| Parallelism and dependencies | `bench-three-features`; `src/workspace.rs` worker/queue/stop paths; `src/locks.rs`; per-agent backend serialization | C29, C36 |
| Auxiliary agent calls | `WorkLeafController::start_title_worker/start_command_agent_worker → CommandChat::run_system_agent_turn` | C30, C33 |
| Direct comparator | Saved `bench-three-features-direct-common`: implementation/fix/review prompts, `commit_if_changed`, `run_feature_cycle`, `run_direct_agent_resume`, `run_sequential_bench` | Shared-factor checks and all differential claims |
| Host/UI/measurement boundaries | Controller event/DTO delivery, driver final gate, observer raw/native/grace accounting | Exclusions X01–X07; no speculative extra model calls |

This is a coverage inventory, not proof that every surface activated in every run. The controller
benchmark uses the `CommandChat` review loop; the public `ReviewCoordinator` is an alternative
entry, not another reviewer in addition. Likewise `LinearizePlanner::launch_linearizer` is an
alternative public entry, not an extra benchmark stage.

## Source and context candidates

### C01 — Deferred large-source delivery through bundles

`orchestrator.rs::send_file_read_response → split_repeated_file_reads →
render_file_read_response → should_bundle_file_read_response → render_bundled_file_read_response`
attempts complete source bundles when untracked snapshots exceed strict 16-KiB single / 24-KiB
combined UTF-8 thresholds. A successful write delivers the manifest; a failed write falls back to
ordinary full inline text. Threshold eligibility is not proof of manifest delivery.
Smaller initial input can remain smaller across
later responses; retrieving the bundle can add tool responses and reintroduce all its content.
**Status:** D demonstrates local representation/recharge; P shows real bundle reads; R parked.
**Boundary/next:** existing private successful-untracked-read inline factor, preserving bundle
creation and non-target suffixes. Defer its cost analysis. Interacts with C02–C06, C24 and C27.

### C02 — Requested repeated reads: unchanged reminders and changed diffs

`split_repeated_file_reads → render_file_read_response_with_repeats →
render_changed_repeat_read_snapshot` uses per-agent snapshots, not another full source delivery.
It can reduce retained input, but insufficient diffs/reminders can induce more inspection or repair.
**Status:** source candidate and exact unchanged-repeat delivery in D; not demonstrated net.
**Boundary/next:** independently audit unchanged and changed exposures, then actual-current-snapshot
delivery at this boundary if needed. Preserve first reads and tracker state; reconcile contradictory
repeat guidance. `--force` does not bypass this classifier. Distinct from C01 and C08.

### C03 — Coalescing adjacent read directives into one handoff

`handle_agent_directives_streaming` merges consecutive parsed reads before one
`send_file_read_response`. One combined exchange can avoid multiple model continuations; a bigger
combined payload can cross a bundling threshold or include less-needed material.
**Status:** source candidate; actual same-reply coalescing exposure needs classification.
**Boundary/next:** inspect emitted directives and accepted sends first. A later coalescing-only
variant must preserve request contents, ordering and locks. Do not infer coalescing from file count.

### C04 — Deduplicating normalized project paths within a request

`read_requested_files → FileLockTable::normalize_path → BTreeSet` reads each normalized project
path once in that response. Avoided duplicate contents can reduce input independently of C03.
**Status:** source candidate; duplicate-path exposure not established for H.
**Boundary/next:** check actual repeated/alias paths before any experiment. Explicit owned bundle
reads use a separate path and must not be assumed to have this deduplication.

### C05 — Snapshot lifetime, per-agent tracking and explicit-bundle bypass

`FileReadTracker::record_snapshots/snapshot_for/clear_files/clear_agent`, accepted edit branches
and `ContextBundleStore::read` determine whether content is fresh, repeated or separately inlined.
A tracked manifest does not prove the model opened its bundle. Own edits clear paths, permitting
another untracked read; explicit bundle requests bypass ordinary project-repeat classification.
These rules can avoid duplicate context or reintroduce it and change subsequent inspection choices.
**Status:** source candidate with invariant tests; no isolated net effect.
**Boundary/next:** classify lifetime/bypass events before proposing distinct subfactors. Do not
silently change source freshness, cross-agent ownership or tracking truth to suppress tokens.

### C06 — Access route and selective retrieval of source/tool results

`agent.rs::PromptPolicy::for_read_permission` governs mediated project access and permits native
reads of owned bundles; Direct uses native filesystem tools. Actual `rg/sed` operands and matched
results determine what content enters context. Selected excerpts can save input; exploration,
truncation and reconstruction can add responses. Both paths can also perform large native reads.
**Status:** P shows both native truncation and WL bundle inspection, not a proven saving.
**Boundary/next:** inspect public call/result pairs, not path mentions. Selective returned text is
not selective disk I/O. Native output-size choices and search breadth are submechanisms, not separate
independent shares of C01. Direct-access permission is a broader factor, not a bundle-only switch.

### C07 — WL mediated command-output compaction

`run_command_for_agent → render_command_result → render_command_output →
compact_blank_runs/compact_long_lines/compact_total_chars` shortens actual stdout/stderr.
Smaller retained output may save input, but omitted diagnostics can require more work.
**Status:** **not exposed in P's 14 delivered mediated results**: zero removed output. H-wide
exposure is not inferred from P. Native tool results bypass this renderer.
**Boundary/next:** saved-output exposure census for other relevant cohorts before a toggle.
Only compare actual original output with its renderer output; never fabricate verbose commands.

### C08 — Bounded automatic conflict-refresh representation

`patch_conflict_refresh_response → render_file_refresh_response →
render_snapshot_diff/render_untracked_refresh_snapshot` supplies changed context, omitting
automatic diffs over 48 KiB and untracked full text over 8 KiB. This can avoid full rereads but also
force reconstruction. Requested changed-repeat diffs (C02) do not use this same bound.
**Status:** source candidate, protocol tests and recorded repair episodes; no isolated net effect.
**Boundary/next:** separate already-held refresh representation from whether a conflict exists
(C18). Preserve exact snapshots, rejection truth and permissions.

### C09 — Policy/instruction packaging and duplicated context

`PromptPolicy::inject/inject_for_delivery → concurrent_work_leaf_interpretation →
concurrent_instruction_translation` includes WL interpretation and original project instructions.
Extra framing costs input while guidance can prevent later work. Provider-added instruction
duplication is a separate hypothesis requiring actual native/request evidence, not assumed.
**Status:** source-confirmed explicit packaging; differential net effect untested.
**Boundary/next:** compare exact delivered layers with saved Direct launches. Keep semantic policy
changes (C10–C15) separate from equivalent-information packaging, and never pad token lengths.


## Work-selection and action candidates

### C10 — Complete validation-scope guidance

`agent.rs::concurrent_work_leaf_interpretation/concurrent_instruction_translation` and
`orchestrator.rs::render_patch_applied_prompt` reinforce scoped validation. This can avoid broad
check/repair cycles, or merely move them to integration. **Status:** source/exposed episodes in P;
whole policy not isolated. Direct's `normal_validation_guidance` already requests focused checks.
**Boundary/next:** compare actual compliance, not labels; enumerate all owned scope spans while
preserving mandatory final checks. Distinct from stop rules, cardinality and runtime enforcement.

### C11 — Stop retrying external blockers and avoid unrelated repairs

Policy plus `append_cross_agent_validation_guard` and ACK wording direct report-once/stop behavior.
This can prevent repeated failures and accumulating context; incorrect blocker attribution can
defer real owned work. **Status:** S/W expose blocker-related actions; complete factor untested.
**Boundary/next:** classify whether failures really are external, then isolate coherent stop wording
without weakening ownership or truthful failure reporting. Interacts with C10, C12, C29.

### C12 — Runtime other-agent test-command gate

Accepted edits feed `PatchOwnershipTracker::record_patch`; `run_command_for_agent →
other_agent_test_locks → render_other_agent_test_command_prompt` can reject a command before
execution. Avoided cross-agent work may save tokens; rejection/correction can add them.
**Status:** exposed rejection in S and protocol coverage, not an isolated saving.
**Boundary/next:** inspect avoided commands and subsequent actions. Removing enforcement changes
shared-tree safety; a wording-only variant does not measure the gate itself.

### C13 — Post-patch validation cardinality

`render_patch_applied_prompt` requests at most one focused validation step after accepted edits.
Limiting continuations can save repeated input; more checks may catch issues and reduce later work.
**Status:** S's exact `ack-validation-unlimited` cue did not establish a qualifying saving.
**Boundary/next:** retain that result and its precise scope; do not call the complete validation
family disproved or automatically rerun this cue. Existing private span preserves other behavior.

### C14 — Cohesive work units and permission to continue remaining work

The cohesion and translated-test spans in `PromptPolicy` plus ACK remaining-work wording can
change submission granularity and edit/check/read cycles. Larger edits can also increase rejection
and rework. **Status:** W tested the combined buildable-work-unit policy; its declared mediator
test did not demonstrate the increase. ACK count is not response count.
**Boundary/next:** preserve that result. A differently scoped cohesion-only or completion-only
factor needs its own definition; do not reinterpret W as those separate tests.

### C15 — Test timing and observable RED before implementation

Original project instructions and `concurrent_instruction_translation(topics.tests)` interact
with WL's no-known-red shared-tree rule. Test→failure→implementation can add continuations but
prevent wasted implementation or later repairs. **Status:** different actual sequences in P;
no isolated strict-RED effect. W is not such a test.
**Boundary/next:** inspect genuine failures, not the word “incremental.” Publishing known-red shared
states or adding private test workspaces changes safety/architecture and requires a separate design.

### C16 — Structured edits versus other patch representations

`PromptPolicy` prefers structured edits; directive parsing routes to
`GitPatcher::apply_edit → StructuredEditPatch::parse/apply_structured_hunk` or
`GitPatcher::apply → git apply --recount`. Exact blocks may avoid hunk calculations/full re-emission;
ambiguous/stale blocks can cause costly resubmission. **Status:** both formats implemented and
exposed in saved work; P retains a rejected large structured edit. No isolated net effect.
**Boundary/next:** a format-preference-only variant can preserve parsers/application/locks.
Do not assume Direct rewrites whole files: native tools also support patch-style edits.

### C17 — Host apply/stage/commit work and compact receipts

`GitPatcher::{apply,apply_edit}` applies and commits; orchestrator records ownership, clears
snapshots and groups accepted paths for ACK. Delegating mechanical work may avoid tool/planning
responses, but WL handoffs and provisional history also cost tokens.
**Status:** real handoffs in S/W. Direct also host-commits with `commit_if_changed`, so automatic
committing alone is not a differential saving.
**Boundary/next:** compare actual host/agent responsibilities and cadence. Receipt wording is an
isolatable subfactor, but cannot establish the effect of removing host apply/commit. That change is
architecture-coupled; commits, ACKs, turns and responses remain separate quantities.

### C18 — Conflict discrimination and already-applied/no-resend guidance

`patch_conflict_refresh_response` and `render_*_conflict_prompt/
render_already_applied_patch_prompt` distinguish stale source, malformed edits and replay.
Accurate guidance can avoid needless rereads/rebases or repeated bodies; correction still adds work.
**Status:** source/protocol tests and retained P/W repair episodes; no isolated saving.
**Boundary/next:** separate truthful classification from guidance representation and C08's supplied
refresh. Never fake acceptance or suppress a genuine failure.

### C19 — Command-produced diff capture, rollback and acceptance

`run_command_for_agent` captures tracked changes, restores the tree and records pending diffs;
`render_command_result_with_pending_changes/render_pending_command_changes_prompt` return them,
and `Done` rejects unresolved changes. Isolation can prevent interference, but diff
input→resubmitted patch output→ACK can duplicate content and create responses.
**Status:** W workflow 007 contains an actual formatter-diff handoff; protocol tests cover state.
**Boundary/next:** classify pending-diff lifecycles. Shortening representation differs from removing
capture/rollback/acceptance, which changes write isolation and review provenance.

### C20 — Command-result next-action and brevity guidance

`run_command_for_agent → render_command_result → ContinuationPrompt::finish` recommends a next
directive and brief explanation. It can reduce explanation/exploration, or stop useful investigation.
**Status:** S's exact `command-guidance-neutral` cue did not qualify; no broader conclusion.
**Boundary/next:** preserve the narrow result. Any distinct follow-up factor must keep actual
command/status/output, locks, failure guards and pending diffs; do not conflate it with C07.

## Review, lifecycle and coordination candidates

### C21 — Eager reviewer evidence versus on-demand access

`WorkLeafController::start_review_for_agent → CommandChat::review_commit_streaming_with_ids →
review.rs::render_review_source_context` supplies commits and all recorded author chat messages.
This can avoid reconstructive reads but duplicate source/patch/result evidence in another context.
Recorded chat is not the complete native tool or private reasoning history.
**Status:** source-established and review exposure in P/S/W, not a net saving.
**Boundary/next:** preserve equivalent access to actual checks/blockers and review scope.
Git alone cannot recreate all evidence. Metadata-only redundancy is narrower (C31).

### C22 — Review target selection and suppression of redundant reviews

`GitHistory::latest_agent_review_commits/agent_review_commit`, controller reviewed/in-progress
state, `should_start_review` and `has_unreviewed_agent_commit` choose review scope after done.
Avoiding duplicate reviews can save calls; larger cumulative scope can increase reviewer work.
**Status:** source/tests; full differential exposure needs classification.
**Boundary/next:** map actual target commits and rechecks. Preserve complete review of changed
behavior; skipping required review is not a valid saving.

### C23 — Evidence-only review resolution versus unnecessary code fixes

`review_commit_streaming_with_ids` routes findings to the original author and evidence back to
the reviewer, explicitly accepting real verification evidence/blockers without cosmetic patches.
This may avoid edit/check/review cycles; inadequate evidence can lengthen exchanges.
**Status:** implementation and W classified repairs/evidence; not isolated.
**Boundary/next:** separate evidence-resolution wording from routing, target scope and C24 message
packaging. Direct also resumes authors/reviewers and carries evidence; reuse alone is shared.

### C24 — Known-session follow-up packaging and retained context

`codex.rs::send_streaming[_interruptible]` sends raw follow-ups to known threads without another
WL policy injection. Direct `run_direct_agent_resume` also reuses threads, but fix/review templates
restate different task/guidance/evidence fields. Repeated additions can inflate later input;
missing context can require clarification. **Status:** source-established difference in templates;
actual redundant-item contribution unmeasured.
**Boundary/next:** compare exact per-role appended fields. Do not reset threads, change cache policy
or treat retained-context charging itself as an independent WL-only mechanism.

### C25 — Directive interruption versus additional generation before handoff

`DirectiveStreamInterruptDetector::observe → should_interrupt_after_streamed_directive →
CodexAppServer::request_turn_streaming/request_interrupt` requests interruption and returns the
accumulated directive reply to the orchestrator, without waiting for provider terminal completion.
Stopping additional output could save generation; more fragmented continuations could increase it.
**Status:** generation boundary exists; no general token-saving conclusion follows from early return.
**Boundary/next:** inspect existing continuation/interruption experiments and response identities.
Keep measurement publication separate. The current 1,000-ms/forward policy is fixed, not a free knob
to alter while “repairing accounting.” No timing experiment is admitted by this map.

### C26 — Effective tools, permissions and transport-specific input

WL `spawn_app_server_process/thread_start_params/turn_start_params` and Direct CLI launch/resume
use different transport/sandbox paths. Tool/instruction request fields add input; available
operations alter chosen actions. **Status:** comparison surface requiring exact effective evidence,
not proof of a saving or a model mismatch. Same requested model does not establish identical tools.
**Boundary/next:** compare saved launch/native/request evidence. Treat native tool truncation as
outside WL's renderer. A combined transport+sandbox+tool change would not isolate one factor.

### C27 — Provider context compaction

Native compaction changes later retained context and has its own generated response; WL's
`CodexAppServer::command` also supports explicit `thread/compact/start`. It can reduce later
input while spending compaction tokens or causing reconstruction.
**Status:** P has one Direct compaction and none observed in WL; not a WL-specific policy proof.
**Boundary/next:** inventory actual triggers and exact response identities. Automatic provider
compaction differs from explicit commands and from C01/C07. Deduplicated accounting is necessary
but is not itself a saving. C37 separately covers retries/restarts.

### C28 — Linearizer target context and work moved into integration

`WorkLeafController::start_linearize → CommandChat::prepare_linearize_launch →
LinearizePlanner::interactive_prompt → compact_linearize_targets` supplies reviewed scope,
metadata and history instructions. Target grouping does not summarize away all constituent
context. Less reconstruction may save calls; author work deferred here may merely shift costs.
**Status:** both WL and Direct have direct-filesystem linearizers, accepted plans, final docs/checks.
**Boundary/next:** separate target representation from actual repair/history work and count the
whole role. Removing final checks/linearization changes the task. Explicit completion is C38.

### C29 — Within-workflow scheduling, locks and shared-state coordination

`bench-three-features` launches concurrent feature work; Direct `run_sequential_bench` serializes
features. Controller workers/queues and `FileLockTable` govern simultaneous access and state seen.
Overlap may avoid duplicate work or cause stale edits, blockers and integration repairs.
**Status:** source/exposure, direction not isolated. Lock waiting and speedup alone are not tokens.
**Boundary/next:** classify conflicts/queued work before a scheduling-only WL design. Preserve locks;
changing enforcement needs safety review. Distinguish this from three top-level benchmark workflows.
Dependency/fork/promotion APIs need explicit exposure before any cohort attribution.

### C30 — Auxiliary title-agent generation

`WorkLeafController::start_title_worker → CommandChat::generate_chat_title →
run_system_agent_turn` uses a persistent system session. This is actual extra model work, not a
saving by itself; follow-up context is retained too.
**Status:** one title thread exists in P's WL topology; Direct has none in that pair.
**Boundary/next:** retain all charges as a possible offset. Do not exclude hidden work or infer one
thread per title. A title-only variant would need to preserve all feature behavior.

## Additional independently distinguishable boundaries

### C31 — Repeated metadata inside review context

The outer review prompt in `cli.rs::review_commit_streaming_with_ids` and
`review.rs::render_review_source_context` both supply commit/feature/reason/scope fields.
Equivalent-information deduplication could reduce input without switching all evidence to lazy access.
**Status:** source candidate; no delivered duplicate-field census or net test.
**Boundary/next:** isolate exact redundant metadata, preserving all facts and ordering semantics.
Different from C21; its marginal effect overlaps the larger review-context representation.

### C32 — Protocol correction and command classification exchanges

`parse_agent_directives → needs_protocol_correction/render_protocol_correction_prompt` and
`render_command_classification` can add corrective/advisory follow-ups. A predictable protocol may
avoid failed actions; syntax repairs/classification requests can add output and retained input.
**Status:** source/tests, cohort-specific exposure needs enumeration.
**Boundary/next:** separate malformed-protocol recovery, command classification and rejection
guidance. Do not count host parsing itself as generation, weaken path/write checks, or suppress
real errors to make a run appear cheaper. Interacts with C16, C20 and C25.

### C33 — Natural-language command-agent routing and transcript resupply

`send_command_agent_message/start_command_agent_worker →
interpret_command_agent_message → run_system_agent_turn` interprets command-chat input and
supplies recent transcript lines. It may avoid user coordination but adds separate model calls.
**Status:** no command-agent thread in P; ordinary host-parsed `new` and `force-linearize` do not
require it. Source existence is not benchmark exposure.
**Boundary/next:** check actual invocation inventory before testing; preserve routing semantics.
Do not confuse this with title generation or native model tools.

### C34 — Small untracked eager-inline delivery and threshold selection

`render_file_read_response_inline` eagerly delivers below-threshold snapshots; the same inline
path also supplies above-threshold snapshots when bundle creation fails. This can avoid a
retrieval turn but retains full text; changing the cutoff affects which groups use C01.
**Status:** source candidate, W exposure; P had no small-untracked-inline response in its saved
read census. R only tests successful bundled untracked components, not all small reads.
**Boundary/next:** inspect exposure across the existing cohorts. Threshold choice and formatting
are separate possible factors; byte cutoffs are not token budgets.

### C35 — Snapshot consistency versus reading the live shared tree

`read_requested_files` takes locked UTF-8 snapshots; `ContextBundleStore::write` gives later
retrievals a captured version, whereas native project reads see the evolving tree.
A stable version may prevent inconsistent inspection, but stale evidence may cause repair.
**Status:** source semantics; exact archived-version/consumption coverage incomplete.
**Boundary/next:** establish which version each actual read saw. Changing freshness is not merely
a token-format switch and must preserve truthfulness and shared-tree correctness.

### C36 — Explicit inter-agent message routing

`handle_agent_directives_streaming` handles `AgentDirective::Send` by forwarding content to the
target agent, which can generate another response. Shared evidence may prevent duplicate discovery
or add overlapping context and coordination exchanges.
**Status:** source candidate; exposure not counted in this inventory.
**Boundary/next:** locate actual sends and target responses, not UI notifications or copied Agent-ID
text. Do not assume all feature agents automatically share one conversation.

### C37 — Provider/session restarts and genuine generation retries

`ensure_thread_loaded`, per-agent session lookup, `spawn_app_server_process` and caller error
paths determine resume/relaunch behavior. A genuine re-execution can repeat work or policy input.
**Status:** source boundary; no inferred hidden retry count. Pre-generation `ETXTBSY` spawn retries
are not model responses.
**Boundary/next:** inspect exact native thread/response and error lifecycle before counting any
retry. Missing-session fallback differs from known-session follow-up and from C27 compaction.
Do not manufacture failures or change credentials/provider to activate a candidate.

### C38 — Completion protocol and additional “done” continuations

`CommandChat::process_agent_reply_streaming_result`, `AgentCompletionPolicy` and
`render_linearize_completion_required_prompt` can require another exchange before a task closes.
Explicit completion may prevent premature review/integration, while extra confirmation costs input
and output. **Status:** source/tests; net differential effect untested.
**Boundary/next:** classify actual completion corrections versus substantive remaining work.
The benchmark linearizer has direct tools but uses interruptible launch/send helpers; it is not
universally an uninterruptible backend path. Never equate silent early termination with completion.


## Shared factors and measurement explanations that must not masquerade as savings

| ID | Disposition and concrete basis |
| --- | --- |
| X01 | **Fresh per-feature contexts are shared.** [Session audit](preflight/SESSION-SCOPE-SOURCE-AUDIT.md): both drivers launch separate authors/reviewers, resume fixes/rechecks, and use a separate final linearizer. WL's shared app-server is not a shared conversation; Direct does not carry all features through one author conversation. |
| X02 | **Several host responsibilities are shared.** Saved Direct `normal_validation_guidance/commit_if_changed/run_feature_cycle/run_sequential_bench` already provide focused-check guidance, host commits, reviewer reuse and final docs/checks with plan acceptance. Exact policy, cadence and evidence packaging can still differ; those are the candidates above. |
| X03 | **Cached input and retained-input charging are accounting channels.** Input including cache counts in raw usage; reasoning is included in output. Counting caching as a discount would change the metric. Repeated item charges are not repeated executions and amplify several upstream causes rather than define another independent share. |
| X04 | **Missing or duplicated telemetry is not a generation mechanism.** `codex.rs::token_usage_from_params/record_usage`, observer ledgers and frozen accounting helpers can differ after interruption or compaction. Deduplicate exact response identities, preserve unsupported gaps, and include auxiliary/retry/failed work. Never call an omitted completion a saving. |
| X05 | **Display shortening and host-only work are not model input reductions.** `codex.rs::activity_from_item/compact_text`, controller events/DTOs and terminal rendering shorten display/status. Driver `bench_run_final_gate` is a local check. Only an actual provider input or generated response connects these to raw tokens; do not infer that from UI size, CPU time or polling. |
| X06 | **Model/version/configuration/resources are comparability conditions.** Retained requested model/effort are not permission to switch providers or credentials. Earlier CLI versions differ from P's 0.153.4; one earlier Direct driver object was unavailable in the local source audit. Keep source/configuration/exposure provenance explicit instead of importing P's findings into all of H. |
| X07 | **Inapplicable entry paths are not observed calls.** Public coordinator/planner alternatives, fork/dependency/promotion APIs, natural-language command routing and the unused Claude provider require actual cohort exposure. Do not count all reachable APIs as work performed by the benchmark. Wall-clock speedup and three-top-level-run capacity do not by themselves change token totals. |

## Interactions and independently testable subfactors

The 38 numbered entries are candidate boundaries/families, not 38 independent or positive effects.
Within C02, unchanged suppression and changed diffs require separate exposure and can be isolated
separately. Within C05, record-after-manifest, invalidation and explicit-bundle bypass are distinct
state subfactors. Within C07, blank-run, long-line and whole-stream compaction are separate
transformations. C32 likewise separates classification, malformed-protocol recovery and rejection
guidance. A later experiment must name the precise subfactor rather than change an entire family
and assign the result to one member.

- C03/C04 can change C01's threshold eligibility; C05/C35 affect which snapshots need refreshing.
- C01/C02/C06/C07/C08 change material retained by C24 and can trigger C27 or more retrieval work.
- C10–C15 change checks/edit packaging; C16–C19 affect rejection and acceptance cycles.
- C21/C23/C24/C31 can retain the same evidence multiple times across author/reviewer contexts.
- C14/C17/C22 influence C28's provisional scope and integration work; reduced author work may shift.
- C29 changes exposure to ownership gates, stale snapshots, conflicts and external blockers.
- C25/C32/C38 affect continuation boundaries; a completed turn, ACK and completed response are not
  interchangeable. C30/C33/C37 can add work outside the visible feature-agent totals.

No percentages or additive allocation are assigned during this inventory.

## Coverage disposition and next work

The first broad source pass covers all families and model-delivery/lifecycle entry paths in the
matrix, with cohort exclusions rather than silent omissions. Exposure and causal verification
remain explicitly incomplete where stated. Source coverage does not establish that these 38 entries
exhaust all future workload behavior or all hidden provider implementation details.

The next stage is an **evidence-status pass across the map**, not a return to the parked
large-read percentage analysis:

1. For every entry/subfactor, record named-cohort exposure, supporting public source locators and
   the existing comparison result. Reuse S/W/P evidence without reclassifying those cohorts as H.
2. Separate local deterministic reductions, behavioral hypotheses, shared mechanisms, unexposed
   paths and measured offsets. Carry existing nulls and failures forward; no favorable selection.
3. Resolve the smallest missing source/exposure facts offline before designing new generation.
   A no-exposure result in one cohort is not permission to invent a verbose workload.
4. Propose precise benchmark-only directional tests only for unresolved exercised mechanisms,
   with existing baseline compatibility and fixed observation counts. No new controls or Direct.
   Architecture/safety-coupled factors require a design that preserves those boundaries, or the
   required explicit human authorization before implementation.
5. Reuse parked R data only when its candidate is reached under the broad plan; quantify shares
   only in Stage C after actual mechanism identification. Do not create an automated launcher from
   the 38-entry list, or launch one run for every entry merely to fill slots.

## Verification scope

This map and plan are documentation-only. No runtime, provider integration, prompt, policy
implementation, accounting helper, frozen manifest or saved benchmark report is modified.
No real-agent workflow is affected; source/evidence inspection requires no new subscription call.

Repository checks on 2026-09-07: `cargo fmt`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-targets --all-features` passed. Explicit real-subscription tests remain ignored;
these ordinary checks are not claimed as a new real-agent verification. Existing implementation's
real diagnostics remain recorded separately. Markdown references and whitespace are checked before
the documentation checkpoint.
