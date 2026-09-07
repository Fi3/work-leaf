# C15: test-first isolation without publishing a known-red shared tree

Status: provider-free design, not an implemented condition or admission. Normal WL,
existing controls, current candidate inputs and parked read-factor outcomes are untouched.
No new control or Direct run is proposed as a prerequisite.

## Actual distinction to investigate

[The complete H timing census](EVIDENCE-H-TEST-TIMING.md) supplies the specific evidence:
all 18 Direct initial author sequences install a new test before their first production
edit; 14 reach a failing new behavioral assertion and four fail on absent target APIs.
These are not 18 syntax failures, failures of only existing tests or invalid Cargo commands.
All 58 WL author command handoffs follow accepted implementation in their own thread.
Seventeen initial WL groups add code and new tests together; H002's visual group initially
adds no new test lines and remains an explicit exception. All 53 accepted groups and
later assertion/source repairs are retained.

The smallest clear historical witness is step4-direct-003's slash-author thread
`01a04de5-49d9-72f1-a87d-af9f50224ac8`: native N114→116 adds the new workspace routing
test; N120→122 runs it and fails on the missing `/status` send; N128→130 and N136→138
implement routing; N144→146 reruns the same test successfully. Exact call/output IDs
and source SHA are in timing census chain A. H004 chain B instead delivers routing and
four new tests at S3478→C24→S3484, then encounters an assertion defect at C26 and repairs
it at S3681→C28. This is a real timing distinction, not proof that cohesive delivery
eliminates repairs. Both systems use focused checks.

The delivered WL policy is also evidenced, not inferred: both shared-tree/test-translation
sentences occur in all 18 H author launches, while the literal root RED requirements
remain in the copied instructions. [C15's disposition](CANDIDATE-DISPOSITIONS.md) therefore
remains substantial but unisolated. The W A/B/C work-unit test concerned independently
buildable increments, not mandatory executed RED. Its null result is not a C15 test.
[The earlier batching design](DESIGN-BATCHING.md) and
[mechanism-priority note](DESIGN-MECHANISM-PRIORITIES.md) explicitly make that distinction.

## Existing seams and the hard safety boundary

| Existing owner | What it actually permits | What it does not provide |
| --- | --- | --- |
| `src/agent.rs::PromptPolicy::for_read_permission`, lines 346–353 | Owned `policy-buildable-work-unit` sentence, after the preserved no-known-red sentence | An executable private RED preview |
| `concurrent_instruction_translation`, lines 514–524 | Owned `instruction-tests-work-unit` span for every detected test instruction | Permission to drop tests or publish intentionally failing shared states |
| `src/orchestrator.rs::handle_agent_directives_streaming`, Edit branch around 858 | Parses a submitted edit, calls `GitPatcher::apply_edit`, then records ownership/clears snapshots and collects an ACK | A held test-only draft that has not been applied |
| `src/patch.rs::GitPatcher::apply_edit` / `apply_edit_with_locks`, lines 77/150 | Takes write locks, computes file changes, writes, stages and commits them | Test execution before shared publication or automatic rollback after a failing test |
| `run_command_for_agent`, lines 1176–1282 | Runs the requested command in `services.locks.root()` under ordinary ownership/lock rules, returns actual output | An isolated workspace selected merely by changing validation wording |
| `send_agent_streaming_interruptible`, line 657; ACK grouping at 982 | Existing backend continuation and genuine applied-group ACK path | A truthful ACK for a draft that was never committed |

Changing only the two policy sentences to “run RED first” would leave contradictory
shared-tree safety in force. Removing that safety sentence, using command locks for
manual test writes, suppressing/ignoring the new assertion, temporary known-red shared
commits, or granting author native writes are not isolated C15 implementations.
`CARGO_TARGET_DIR` alone isolates build output, not source/test publication.

## Smallest safe first test: fixed-artifact replay, no model generation

Use the exact step4-direct-003 chain above as a bounded private regression fixture:

1. Resolve its saved feature-1-complete checkpoint and the actual slash-author starting
   tree, not merely the study's common base or the later final rewritten commit.
   The retained artifact has `observation/git-checkpoints/files/feature-1-complete/`
   head/status/index/worktree records, public native edits and `patches/pass/commits.bundle`.
   Verify reconstruction before execution; a final bundle alone does not prove the
   provisional starting tree is available. Missing exact reconstruction remains a limit,
   not permission to substitute a similar tree.
2. In a new disposable **private** checkout, apply only the captured accepted test delta
   N114 and run the exact N120 focused command. Require the named new test to run and
   fail for the captured missing behavior. An invocation failure, unrelated assertion,
   syntax/helper error or zero tests is not this fixture's RED.
3. Apply the captured first implementation deltas N128/N136 without changing the held
   test bytes; run that same command and require its GREEN result. Verify the test body,
   assertion and command are identical across the two executions.
4. Assert the original repository and saved artifacts remain byte/HEAD/index unchanged.
   Give the private checkout its own build/scratch output, retain both command receipts,
   and treat these as diagnostic executions—not new benchmark workflows or token data.

This is the smallest implementable check of the essential safety/ordering claim. It
does not require a generic preview subsystem or provider call. It proves that genuine
RED→implementation→GREEN can be represented privately using the same actual test;
the existing capture already establishes that the historical author performed it.
No replay or checkout mutation has been performed as part of this planning task.

## Minimum genuine future WL factor, if separately authorized

A natural executed-test-first treatment needs a **benchmark-private held test preview**,
not another cohesion cue. Its minimum state is an explicit author/test-proposal identity,
an exact clean source snapshot, the held test body, one selected focused command and its
actual result. The first test-only proposal is evaluated in a private copy; it must not
reach normal shared `GitPatcher` publication, ownership, tracker clearing or ACK grouping.
The author receives a truthful typed preview-result continuation, not a fake applied ACK,
then submits tests plus implementation through the ordinary shared edit/commit path.

The plausible owning boundary is the private benchmark adapter before the existing
Edit application and ordinary backend continuation in `handle_agent_directives_streaming`.
This is a **new seam/state operation**, not an available switch. Test proposal/command
transport must be explicit and source-owned; do not recognize current benchmark filenames,
agent-ID prefixes, reason strings or assume `tests/` paths prove test-only semantics.
Adding a public directive/enum variant or provider API is not silently authorized by this
design. A private hook/manifest and an exact role source require separate architecture
review before implementation. The two existing policy spans would describe that actual
preview contract; the original instructions and all non-target policy remain intact.

Hold these invariants:

- No deliberately failing shared state, native author write permission, different
  ownership policy, provider, interruption/grace, scheduling or review/final-gate change.
- One declared preview snapshot per eligible proposal; no live-tree rereads during its
  execution, shared build-output writes, hidden retries or inferred success. If the live
  tree advances, ordinary final-patch conflict handling remains authoritative.
- Preserve every test proposal/result and the final test delta. Fixed-artifact tests can
  require exact test identity; natural model-generated test repairs need explicit evidence,
  not silent deletion, a passing replacement test or exclusion of a failed workflow.
- A GREEN first preview or an invalid/unrelated failure remains nonactivation/unsupported
  RED, not manufactured failure. Preserve whole workflow outcomes and all preview work.
- Keep the unchanged host apply/ACK/check/review sequence after publication. Preview
  output is a distinct source-linked result; it is not ordinary file content, command
  compaction or a previously accepted patch receipt.

## Achievable question and decision

The concrete question is: **does requiring actual private pre-implementation test
feedback change subsequent chosen implementation/repair work and whole-workflow input
replay, while preserving the public tree and requested test obligations?** Inspect exact
test/result→next response→implementation/check/repair chains and all outcomes, including
extra preview transport, snapshot/context cost and later repair avoided or introduced.

This treatment necessarily couples test timing with isolated execution and its feedback
transport. A saved-baseline directional comparison can inform that specific private
test-first workflow; it cannot identify timing alone, price an ungenerated continuation,
or assign an additive fraction of the historical gap. Holding tests fixed by supplying
historical tests in the launch prompt would instead be a test-assistance/information
treatment and is not an interchangeable shortcut.

Recommendation: preserve C15 as the evidenced package mechanism, qualify the fixed-artifact
private replay before considering a preview implementation, and use the already selected
C16/C21 evidence to narrow remaining work. Do not generate an unsafe or exposure-free
prompt ablation merely to fill this ledger row. No new provider observation is admitted.

## Inspection pins

- Timing census: `894130bacdce03adb1b86173a05e03162a57f1b3c91ebbefc54205a692659901`.
- Dispositions: `ed420cd72547e935a2fa4ba8d6393d76db4debc91da8b23fafc7b9532c42aab6`.
- Batching / priority notes: `545ad7dcb398e36f859cad098e8de21f5f7e3c333edba75135dba87561356754` /
  `9ce8feb3164321f65ebe8b14b25e1b737875b40608429ce8bc690efa67300e48`.
- Current agent / patch / orchestrator sources: `a9e6065a450d05bf299202a2d6f44dd2dae33a3324e8c70e9f91d36a22b242fd` /
  `aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda` /
  `027c35a9eaf24e99335001e68f7d0612a262831af80f8677874525db95e9332b`.

This design changes no runtime or agent-facing workflow and requires no real-agent
verification itself. Historical and current CLI/source qualifications remain distinct;
no active costs, parked read costs, new totals or percentages were inspected/computed.
