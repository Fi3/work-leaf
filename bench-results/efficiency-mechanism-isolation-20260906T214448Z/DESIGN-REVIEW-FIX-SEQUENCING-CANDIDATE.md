# Review-fix test-feedback sequencing candidate

Status: source/design inspection only. This document admits no implementation,
executor, diagnostic, benchmark or additional control. The frozen initial-author
[C15 screen](PROTOCOL-TEST-FIRST-SCREEN.md) remains unchanged.

## Decision boundary

Review-fix sequencing is an exposed, distinct C14/C15 subfactor. The smallest owning
boundary is the **existing findings-to-author-fix delivery through that repair's first
ordinary shared publication**, not the initial author launch, reviewer launch, normal
ACK, or completion/recheck policy. Reuse the qualified private test-feedback machinery;
do not build a second executor, provider path or general review framework.

The current C15 requirement does not enforce this boundary. It opens one shared-apply
gate per successful author launch generation. After initial preview delivery, later
repair submissions remain eligible. The author can voluntarily request another
preview, and retained initial instructions can influence repairs, but that is not an
independently varied per-repair requirement.

There is one meaningful scope choice before implementation: **conditional sequencing
of genuinely needed new/revised tests, versus mandatory private feedback before every
first code/test repair publication**. These are not interchangeable. The latter can
force new test work when existing tests already demonstrate the defect. The historical
data do not authorize that substitution or a host heuristic that decides test necessity.
The smallest non-target-preserving candidate is the conditional request; any claim of
enforced ordering additionally needs an explicit eligibility/declaration contract.

## What the saved evidence establishes

[Historical review-driven work selection](EVIDENCE-H-REVIEW-WORK-SELECTION.md) retains
all 21 Direct fix cycles, all 17 genuine WL FINDINGS cycles, and nine separate already-clean
WL parsing detours. Both populations initially review all 18 feature targets and have
genuine findings on 13 targets. Every genuine cycle produces accepted code or test work;
there are no evidence-only cycles in that finite population. This does not remove the
general evidence-only route from the orchestrator's contract.

Twenty Direct cycles contain an executed failed check. D-17-01 is the important
exception: native N563→625 contains only the accepted No-path regression at N583;
focused N592 and full N602 checks pass first, without a production patch. D-11-01 mixes
first-GREEN coverage N541→556 with a different failing regression N547→555. A required
RED result would misclassify both. The index retains complete findings, typed source
identities and patch/output/recheck locators; these counts are neither response counts
nor savings.

WL's 17 genuine cycles contain 21 accepted repair groups and 23 mediated checks. Four
additional groups repair validation expectations/fixtures after accepted implementation.
For example, W-005-C42 accepts a routing/test group at ACK C44/edit S30544, encounters
the missing mock trait method at C46, repairs the mock at ACK C50/edit S31488, and passes
C52 before the clean C54 recheck. This is a concrete opportunity for differently timed
test feedback, not proof that the check or repair was unnecessary. W-006-C71→C114→C121
instead spans successive substantive completion-state repairs; different sequencing
cannot be assumed to eliminate later reviewer-requested behavior.

These are source-qualified historical H observations, not executions of current v6.
The evidence document preserves the H/P normal-source equality qualification and older
CLI identities. No new source census, test execution, quality scoring or token analysis
is part of this design inspection.

## Exact current call chain and prompt ownership

| Stage | Actual owner and consequence |
| --- | --- |
| Initial enrollment | `src/cli.rs::prepare_agent_launch` at line 962 prepares `Role::Author`; `launch_prepared_agent_streaming_with_ids` at 1026 surrounds the actual launch with the private ticket. Linearizer preparation explicitly uses `Role::Other`. |
| Initial policy and gate | `src/bench_private_test_first.rs::render_policy` at 401 replaces only owned author policy spans. `PRIVATE_TIMING` at 399 says “Before your first shared edit or patch”. A committed owner starts with `qualifying_delivered:false` at 830. |
| Private delivery | `src/orchestrator.rs::run_private_test_preview` at 1230 uses normal command safety predicates, `Registry::reserve`, exact selected-source execution, and same-author raw feedback. Only after send returns does `Registry::delivered` at 722 set `owner.qualifying_delivered |= qualifying` at 759. |
| Shared publication | `reject_unprepared_private_apply` consults `Registry::may_apply` at 573 before ordinary apply. The gate requires the delivered flag and an uncancelled owner; it contains no review-round state. |
| Actual review | `src/cli.rs::review_commit_streaming_with_ids` at 1307 renders the existing full cumulative scope and source context, launches or resumes the owning reviewer, and processes its reply. |
| Actual repair | The existing `while !has_no_findings(...)` branch at 1391 renders `fix_prompt` at 1396 and calls `send_agent_streaming_interruptible` at 1411 on **the same `commit.agent_id`**. There is no new author preparation, launch generation or preview reset. |
| Ordinary repair processing/recheck | `process_agent_reply_streaming` at 1420 processes real directives/followups. The host renders `recheck_prompt` at 1427 from actual processed repair/evidence and resumes the same reviewer. Completion handling and round limits are unchanged. |

The non-streaming `CommandChat::review` delegates to the same streaming review
function with a no-op event callback; it is not a separate repair implementation.
`bench_review_fix_prompt` at 1081 is the separate v4 C24 hook. Its `candidates_active`
guard is not a v6 repair-sequencing hook, and original-request resupply is not part of
this candidate.

The exact owned insertion point is after construction of the host's baseline
`fix_prompt`, before its actual send. Preserve byte-for-byte the `commit.hash`, complete
`review_text`, and existing instructions to fix code/test defects, supply exact evidence
or a blocker for non-code findings, defer prose, and emit `@work-leaf done`. An additional
owned timing paragraph must not replace or reinterpret any of those spans. Preserve
the reviewer launch and recheck templates, review scope and target, original feature
request, same author/reviewer sessions, ordinary ACK/check prompts and actual feedback.
Do not resend the full launch policy or search copied transcripts for a role/target.

The current branch predicate is not a semantic FINDINGS classifier: historical clean
marker detours demonstrate the distinction. Preserve its routing behavior; separately
record whether an observed handoff contained genuine findings. A new substring gate or
review-parser correction would combine another factor with sequencing.

## Minimum prospective intervention, not a selected arm

A conditional owned instruction could ask: “For new or revised tests needed to resolve
these findings, obtain their actual private test feedback before the ordinary combined
repair. Preserve the needed test obligations and disclose revisions. If existing tests
suffice or the finding needs evidence rather than code, do not invent a test patch.”
The precise grammar and ordinary safety/result instructions must still be available
through the source-owned private protocol, not assumed known from an unrelated launch.
No operator test body, assertion, failure status or implementation is supplied.

That is an **instruction treatment**, not proof of mandatory execution. If the intended
factor is enforced private ordering, the minimum extra state is a private repair epoch
attached to the existing author owner, reviewed target, reviewer and actual handoff
ordinal. It is not a new author generation. An eligible epoch can require one fresh
delivered test result before its first ordinary repair apply; it need not require a
preview for every later ACK/check within that repair. An old initial result or replay
cannot satisfy a fresh epoch. Failed sends, stale delivery, cancellation and duplicate
proposals retain their original outcomes and cannot reopen or silently rerun work.

The unresolved eligibility choice must not be hidden in that state machine. An
unconditional first-apply gate cannot distinguish a production-only repair using
existing coverage from a repair that needs a new test. Filenames, failure text, a
`FINDINGS` marker and whole-file hashes cannot resolve that semantic question. Either
retain conditional/advisory compliance as an observed outcome, or explicitly approve
the added author declaration/mandatory-check scope before designing its enforcement.
Do not require cosmetic test edits to make the existing nonempty proposal protocol fit.

For a **repair-only** factor, initial author prompt bytes and initial apply behavior
must remain baseline. Private role ownership may be retained internally, but initial
C15 policy injection/gating must not be silently combined with it. Adding a repair
requirement to v6 instead measures an initial-plus-repair package. A comparison of that
package with normal saved W cannot identify the repair increment alone. No new normal
control is required or authorized merely by recognizing this distinction.

## Held fixed and what may change

Hold fixed substantive findings, requested behavior/test obligations, cumulative review
scope, original target and same sessions; patch formats, shared no-known-red rule,
apply/commit/ACK, focused-check duties, locks/conflict refresh, completion/recheck,
linearization, provider/model/interruption/grace, normal timeouts and observation.
The selected source for a repair preview must be the accepted state at that repair,
not the task's original base or a guessed pre-fix snapshot. Concurrent accepted changes
and later patch conflicts remain ordinary outcomes.

The target can change the order of test construction/execution and implementation
publication, and therefore chosen repairs and subsequent continuations. Private
execution, fresh build state, its actual feedback and retained context remain treatment
components. The same tests are not assumed fixed across natural model trajectories;
prove held/final assertion identity from exact source where possible, retain revisions
and ambiguity elsewhere. No check count or first-GREEN result establishes wasted work.

C14's combined W policy result does not test this ordering requirement. C15's qualified
initial private chain establishes reusable machinery, not a repair-stage effect. C38's
DONE/recheck obligations remain non-target: skipping unresolved work, suppressing a
finding, ending at a private result, or omitting the ordinary check is not a saving.

## Authority and minimum further qualifications

[The recorded C15 authority](AUTHORITY-C15-C21-20260907T1806Z.md) approves the qualified
private test-first workflow family, with environment/feedback disclosed and fixed-count
admission after gates. It does not select conditional versus mandatory repair eligibility
or a per-review dose. The concrete current protocol explicitly targets an initial
author before its first implementation patch. Neither authority nor the successful
[diagnostic003](preflight/c15-real-diagnostic-003/RESULT.md) silently amends that frozen
scope: diagnostic003 has one clean review and **no findings/fix cycle**.

Therefore a new prospective scope decision is necessary, not another executor design.
The root must select the repair-only versus combined package and resolve the eligibility
choice explicitly; a materially broader compulsory-work requirement needs the user's
choice rather than inference from initial-author approval. The architecture already
owns private enrollment in `CommandChat` and execution in the orchestrator, so reuse
there need not require a public API extension. Any implementation needs a corresponding
current-state architecture/condition description and separate fixed admission. This
document neither blocks nor expands the already approved initial C15 screen.

The minimum incremental gates are:

1. Test-first actual review-branch coverage: complete an initial author, obtain an actual
   reviewer finding, route the owned fix, obtain private feedback if eligible, accept
   the ordinary combined repair, execute its normal check, process DONE and recheck the
   same target/reviewer. Exercise worker clones and the shared non-streaming delegate.
   Verify all non-owned prompt bytes, default/prior-schema behavior and read/ACK state.
2. If enforced, prove earlier preview/replay cannot qualify a new epoch, repeated delivery
   cannot execute twice, a new review round has its declared behavior, and failure,
   evidence-only/first-GREEN cases and cancellation cannot grant false apply authority.
   Do not reset the real launch generation or relax completion/round limits.
3. Qualify the **post-check** accepted source and ordinary overlay with existing pinned
   selection/materialization/executor, not only initial clean checkout. The exact
   `preflight/c15-live-selection/live_selection.py::census` rejects every
   `git ls-files --others -z` entry, including ignored build files. Ordinary
   `src/orchestrator.rs::run_shell_command` sets cwd and optional TMPDIR, not Cargo's
   target directory. A later build-output-bearing checkout may therefore be unsupported.
   Inspect actual declared environment/source boundaries; do not delete outputs, ignore
   them silently or relocate ordinary checks to manufacture eligibility. Any expanded
   source contract is a separate reviewed choice, not inherited qualification.
4. After explicit admission, one bounded real configured-agent **findings-to-repair**
   scenario must join full accepted/public/native inputs, owned target/turns, fresh
   source/overlay, closed result, held/final test mapping and ordinary ACK/check/DONE/
   recheck. Synthetic tests and diagnostic003's clean review cannot stand in for that branch.
   Natural GREEN, unsupported preparation and substantive unresolved findings stay
   outcomes; no automatic repeat to obtain RED or a clean verdict.

No new accounting framework is needed. Observe actual repair exposure and full retained
outcomes first; only a separately declared common scope can assess whole-workflow token
direction. Snapshot/hashing work remains per proposal, and existing keyed registry
lookup can remain logarithmic rather than scanning all earlier repairs. Repeated source
materialization is O(P×B), not constant-time. Existing detector/mount/path complexity
qualifications in `docs/architecture.md` remain; no algorithm is changed here.

## Inspection identities

These are the inspected source/document bytes, not identities attributed retrospectively
to H. Only this new design file is written; no providers, executors, tests, extractors or
accounting helpers are invoked for this task.

| Path | SHA-256 |
| --- | --- |
| `src/cli.rs` | `98b08ef350616c2f9d33ca0dd6ca6916cf31a0a985e694370b0097acd4e867ad` |
| `src/bench_private_test_first.rs` | `1700f26d5ff99271ca12acbfa783ff0d5525d3720c9ba47ff088c1aaa9d40d2f` |
| `src/orchestrator.rs` | `e0ef8167bd1f51cd99d56ae58835ba5d6951549c9dc9a2ccdabf9b2e568ff0f7` |
| `docs/architecture.md` | `20c1f2bd4cdec7f0ba4fd613eecc2fd4156554f4b4d46d53bdfbb4f6773df921` |
| `AUTHORITY-C15-C21-20260907T1806Z.md` | `42e40b84b255499d0877485e9fd36fadfbec3995a15c7bc25d8eba60f8170f0d` |
| `PROTOCOL-TEST-FIRST-SCREEN.md` | `4dcc17c5598666461f8e9a5fa7c9e17e56e2e439d3cefbf93830e6e16aa7057a` |
| `DESIGN-TEST-FIRST-ISOLATION.md` | `99d82be2c35c24dff89bded7eb786a9a18bf955ede11c0e7f68133ba6ffe2493` |
| `DESIGN-TEST-FIRST-ISOLATION-DETAIL.md` | `9e4911e63cbd5fac54475552dc883d15c4448eaf149a4e8e2a134df8328240ff` |
| `CANDIDATE-MAP.md` | `59928e936f3e7a9fad1349ce45b2bc9f9e3becfe33d041d2d5c125e900eaef9c` |
| `EVIDENCE-H-REVIEW-WORK-SELECTION.md` | `29185627ba22d52ab6c3f13a8e28806ea527d5e39e50201fd912c484dbb2b0d1` |
| `EVIDENCE-H-REVIEW-WORK-SELECTION.json` | `6c6b9ba63361e3db79ad3883f374d9ba6fe684ff4c01423c7d50edac192a043a` |
| `preflight/c15-real-diagnostic-003/RESULT.md` | `cc3f3973e255175949ce0121c185e922a829be0e3f75e9416fa7c1b78c64bfdc` |
| `preflight/c15-live-selection/live_selection.py` | `0614db9a5c874cd5f623997ce3433edbd2bc0ff3585c23913dd9ede9705d3a55` |
