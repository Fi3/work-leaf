# C15 private test-preview boundary

Status: source-grounded design only. No condition, code, test, process, provider call,
or observation is admitted by this document. The existing
[C15 design](DESIGN-TEST-FIRST-ISOLATION.md) and its
[qualified fixed-artifact replay](EVIDENCE-TEST-FIRST-REPLAY.md) remain unchanged.

## Decision

A held test proposal, explicit command, truthful private result and later ordinary
patch submission fit the current **core-workflow ownership** without a public API
extension. They are not implementable by a prompt replacement alone. The minimum
credible intervention needs a new benchmark-private operation, immutable source
materialization and a separately qualified preview process boundary.

The current shell runner does not supply the last guarantee. A temporary cwd, a
different Cargo target directory, a root read lock, or a disposable Git worktree
does **not** establish that arbitrary project validation cannot write to the shared
checkout. Consequently, an implementation that merely redirects the existing
runner is not an admissible safe C15 factor. This is a concrete missing mechanism,
not uncertainty about whether private test-first execution can be built.

## Existing call seams and what they prove

Paths and line numbers refer to the source identities at the end of this document.

| Owner / exact seam | Relevant behavior and proposed boundary |
| --- | --- |
| `src/orchestrator.rs:1701`, `AgentDirective`; `:1730`, `parse_agent_directives` | Both are private. A feature-gated explicit preview envelope can be parsed separately; no public enum variant is necessary. Ordinary edit/patch syntax must retain its present meaning. |
| `src/orchestrator.rs:631`, `DirectiveStreamInterruptDetector`; `:1844`, `should_interrupt_after_streamed_directive` | A preview must interrupt only after its complete envelope, using the existing stop-detector mechanism. Ordinary terminal detection and provider/grace handling remain unchanged. |
| `src/orchestrator.rs:685`, `handle_agent_directives_streaming` | Preview dispatch belongs here, before either shared apply branch (`:789` patch, `:861` edit). It must not masquerade as a submitted production patch. |
| `src/patch.rs:77`, `GitPatcher::apply_edit`; `:150`, `apply_edit_with_locks`; `:187`, `compute_structured_edit_changes` | Exact-block matching is computed before file writes, but public `apply_edit` also stages and commits. Preview needs a crate-private prepare/apply-to-private-root path owned by this module, not a duplicate edit interpreter and not a shared temporary commit. |
| `src/patch.rs:64`, `GitPatcher::apply`; `:90`, `apply_with_locks` | Unified diffs have a `git apply --recount --check` boundary before application, then ordinary staging/commit. A private preview can reuse those semantics against a separate repository; it must not reuse the shared index or a linked worktree's administrative state. |
| `src/orchestrator.rs:1176`, `run_command_for_agent` | Normal execution applies path normalization, other-agent-test ownership and failure-masking checks, then runs in `locks.root()`. Preview must preserve relevant checks but cannot call this whole path: it also captures/reverts shared changes and creates an ordinary command-result continuation. |
| `src/orchestrator.rs:1435`, `run_shell_command` | Uses `current_dir`, inherited environment, optional command TMPDIR and a process group; no filesystem isolation is installed. `terminate_child` at `:1503` is timeout cleanup, not proof that arbitrary descendants or external writes are contained. |
| `src/orchestrator.rs:618`, `AgentFollowUp`; `:624`, `DirectiveRun`; `:657`, `send_agent_streaming_interruptible` | A distinct preview-result prompt can use the same session and follow-up queue without a new backend method or fabricated `PatchApplied`/`CommandRun` event. Existing status stream events can describe preview activity truthfully. |
| `src/cli.rs:1093`, `process_agent_reply_streaming_result` | Processes real follow-ups through the normal queue/round limit. Preview adds its actual turn; it must not reset that limit, create a hidden agent, resend launch instructions, or skip the next reply. |
| `src/orchestrator.rs:795–809`, `:867–881`, `:982` | Only actual normal shared acceptance clears tracked snapshots/pending changes, records ownership and contributes to the applied-file ACK group. A held test proposal does none of these. |

`FileLockTable::locks_for` (`src/locks.rs:105`) keys independent locks by exact
normalized path. `patch_lock_paths` (`src/patch.rs:382`) adds `.` for patch index
serialization. Commands lock their requested keys (`src/orchestrator.rs:1235`),
not necessarily `.`. Holding `.` therefore does not atomically snapshot every
live source file. Nor does lexical path normalization at `src/locks.rs:124`
provide symlink or command-execution confinement. These are limits of reuse,
not proposed changes to normal locking.

## Concrete private protocol and state

The prospective schema selects one `test-first-preview` condition and explicitly
enrolls author launches by their owning launch call, not an agent-name prefix,
feature text, filename, failure string or observed cost. Reviewers, title/system
sessions and linearizers are not preview subjects. The `CommandChat` launch and
shared-clone pattern at `src/cli.rs:977–1012` can carry a separate private owner
record; the version-4 request registry must keep its existing contract.

One new terminal envelope contains: a proposal identity; format `edit` or `patch`;
the ordinary body in that format; the author's test-purpose declaration and
declared test units; the exact focused shell command; and normalized requested
write paths. A single-line typed metadata header followed by the existing raw
patch body avoids requiring a second, JSON-escaped copy of the entire patch.
An explicit `@work-leaf test-preview …` / `@work-leaf end` framing is a proposed
private grammar, not an existing directive. Malformed, incomplete, duplicate-ID
or mixed preview-plus-ordinary-side-effect envelopes have no apply/execute effect.
No first-edit heuristic silently treats normal production work as a test proposal.

State is keyed by run, owned launch generation, agent and proposal, with phases
`held → snapshot-bound → executed → result-delivered`. Repeated transport of the
same identity cannot execute twice; changed bytes under that identity are an
error. A distinct revised proposal preserves its predecessor and consumes real
time/turns. There is no automatic rerun until a desired RED appears.

The author first submits the test proposal, receives its actual result, then
submits tests plus implementation through the unchanged ordinary edit/patch path.
A treatment-only gate before shared application rejects an enrolled author's
first production submission without a delivered preview result; it does not
apply the withheld patch or generate a fake ACK. The gate concerns ordering,
not a predetermined failure status. It cannot certify that an arbitrary patch
is semantically test-only or that a generic command actually ran a new assertion.
Those claims require the retained complete proposal, command/output and subsequent
test/implementation evidence; `tests/` filenames or nonzero exit alone are inadequate.

## Source snapshot: a bounded implementable contract

The existing benchmark primitive is reusable in scope, not as a whole benchmark:
`bench-three-features:405–409::prepare_base_checkout` performs independent
`clone --no-checkout --no-hardlinks` plus detached checkout. Select the exact
accepted source identity instead of its workload-specific fixed base; do not
invoke the driver, provider setup, frozen feature scripts or final three-check
gate. `bench-validation-common:3–15::bench_run_final_gate` supplies only cwd,
TMPDIR and host Cargo execution. It is not a confined executor. The driver's
`tiny_pid_namespace` predicate (`:118–120`) detects existing process conditions
for supervision; it installs no namespace. The inspected source/scripts contain
no general confinement primitive to reuse for this guarantee.

The smallest snapshot contract is **one immutable accepted Git tree**, identified
by exact commit/tree object IDs, rather than an allegedly atomic copy of the live
worktree. Select that identity under the existing shared `.` lock; materialize
from those objects into a create-new independent repository. Never use hardlinks
to mutable files, shared indexes, `git worktree add`, stash, reset, or temporary
shared commits. No lock is held while validation or agent reasoning runs.

This explicitly tests the accepted state at capture, not all transient filesystem
bytes or a later HEAD. A dirty tracked/index state or required source outside the
materialized tree is an unsupported snapshot, not silently discarded input.
Untracked/ignored fixtures, local path dependencies, submodules, symlinks,
attributes/filters, generated inputs and Git-dependent tests need an explicit
predeclared materialization contract. The minimal version rejects unsupported
cases before execution; it does not guess that omitted files are irrelevant.
Any allowed immutable overlay has exact source/byte/mode identities and is
recorded separately from the Git tree. No credentials or user configuration are
copied into the snapshot. Environment-dependent checks remain qualified, not
declared equivalent because the source hashes match.

The patch module computes held changes against this base and applies them only
inside the preview repository. Its manifest records input body bytes, format,
base identity, every touched before/after image and mode, plus declared test-unit
ranges/hashes. All matching failures occur before the validation command starts.
The runner does not promote a private index, commit, build output or generated
file to the shared tree. A concurrent accepted change can make the later normal
submission stale; ordinary conflict feedback and refresh remain authoritative.
No automatic rebase, refreshed preview or silent retry substitutes a new base.

An alternative complete live-tree snapshot would require a private barrier around
**all** cooperative mutators, including commands, plus handling external writers.
That changes scheduling and is deliberately outside this minimum intervention.
The immutable-tree contract avoids that barrier but must honestly retain its
unsupported-input and accepted-state-versus-live-state limits.

## Execution, failure and cleanup contract

For the strong requirement that preview execution cannot publish a failing shared
state, a new private executor must be qualified before implementation admission:

- Its writable source/build/scratch paths are exclusively the owned preview tree.
  The real shared checkout, Git administration, retained evidence and provider
  configuration are not writable from that process. Absolute paths, symlink
  traversal, subprocesses and shell scripts must not bypass this boundary.
- A supported platform isolation mechanism must establish that property, not
  command-name classification. No such general executor exists in the inspected
  source or dependencies. Platform-specific confinement may sit behind a private
  core helper; an unavailable facility means `unsupported`, never an unconfined
  fallback. The mechanism and its executable identity require a separate decision
  and tests; this document does not claim an installed sandbox has been verified.
- Keep the exact agent-selected command, ordinary failure-masking rejection and
  existing five-minute maximum unless separately authorized. Do not inject a
  known-workload test, automatically run baseline checks, alter assertions, relax
  failure exit codes or substitute a passing filter. Bind tool/executable and
  effective execution-environment identities without recording secret values.
- The private source location and cold/private build outputs can affect tests and
  duration. Tests requiring unsupported absolute cwd, network, mutable external
  state or shared caches retain that limitation. The treatment is private
  test-first feedback, not a claim of identical execution environments or timing
  alone. A same-path filesystem view would need its own qualified implementation.
- Timeout/cancellation owns the whole preview process lifetime. Require a closed
  child/descendant boundary and drained output before cleanup or result delivery;
  do not treat the current shell process-group helper as proof against escape.
  Leave the provider registry and normal command shutdown behavior unchanged.

Publish create-new proposal/source/result artifacts before the result is sent.
They identify exact command, status/signal/timeout, stdout/stderr bytes, partial
capture flags, private source changes made by validation, elapsed time and
execution/source digests. The user-facing continuation includes only its distinct
preview identity, source base, exact command, status and ordinary compacted output;
complete output remains retrievable as explicitly scoped preview evidence. It
contains no repeated launch policy, unrelated source bundle or reviewer history.
Preview artifacts use a separate namespace/counter, never ordinary project-read
snapshots or `ContextBundleStore` allocations. Do not auto-open or resend them.

An initial GREEN is a real GREEN. Absent API/compile failure, named behavioral
assertion failure, invocation failure, zero-test execution, unrelated failure,
timeout and unsupported execution remain distinguishable evidence states. The
generic runtime returns facts, not a fabricated semantic RED classification.
Failed preparation, evidence publication or delivery cannot become a successful
preview receipt. Delivery failure preserves the executed result and does not
implicitly rerun its command on resume.

Temporary execution trees and durable evidence have separate lifetimes. Cleanup
can remove only the verified owned temporary directory after all processes exit;
replacement, symlink, inode mismatch or uncertain process state preserves the
directory in quarantine and records failure. Parent paths, live checkout and
earlier proposals are never cleanup targets. Retained evidence outlives agent
completion and normal orchestrator teardown. Cleanup errors are not hidden by a
passing test result.

Final publication uses the ordinary submitted patch, not an automatic merge of
held test files with generated implementation. Preserve all final test deltas and
revisions. Exact unchanged test units can be proved by their byte ranges/hashes
in the accepted after-images; changed/deleted/ambiguous units remain explicit.
Do not impose a language-specific test parser or treat whole-file hash changes
from co-located production code as proof that an assertion changed. This design
does not forbid a necessary later test correction, but that correction cannot
count as an unchanged-test RED→GREEN chain.

## Non-target identity and implementation gates

The existing no-known-red shared-tree sentence remains literal. Only the owned
test-timing/work-unit sentences (`src/agent.rs:346–354`, `:514–524`) and a new
owned preview grammar section describe the private ordering contract. Original
repository instructions, focus/ownership restrictions, checks, edits, commits,
locks, review/fix/recheck/linearization and completion obligations stay present.
The altered cohesive-publication clause still requires tests plus implementation
for the final shared patch; it distinguishes that publication from a private
proposal. The baseline acknowledgment and command-result instructions are not
ablated. No v2/v4 work-unit, format, read or review-context treatment is combined.

Before any real-agent qualification, new tests must first demonstrate failures
and then cover:

1. Default compile-out, feature-enabled inactive and every older schema: identical
   full launch/follow-up bytes, directive interpretation, stop boundaries,
   event order, bundle paths/counters, snapshot/ownership/pending-change state and
   normal apply/check/review behavior. No preview filesystem access or allocation.
2. Active explicit author ownership and complete envelope handling; copied marker
   data, Unicode, both formats, multiple/duplicate IDs, reordered states,
   malformed/mixed bodies and reviewer/system misuse cannot create side effects.
3. Exact immutable snapshot and held-test application; dirty/unsupported inputs,
   path aliases/symlinks, private/shared Git identity, changed HEAD and live
   concurrent commands cannot silently substitute source or publish test bytes.
4. Actual process-isolation adversaries: absolute-path writes, symlinks, child
   processes, timeout, cancellation and replacement during cleanup. A source
   comparison after execution detects drift but is not a substitute for preventing
   an observable failing shared interval.
5. A generic declared-test missing-behavior RED followed by normal implementation
   and same-test GREEN; first-GREEN, missing API, invalid/zero-test command,
   unrelated failure and test revisions retained without relabeling. No current
   benchmark feature or filename is recognized by production code.
6. No apply/ACK/tracker/ownership effect for preview; ordinary final acceptance,
   replay/already-applied, conflicts/refresh, one applied-group ACK and normal
   focused follow-up remain authoritative. Evidence-write and send failure,
   duplicate delivery, terminal shutdown and retained artifacts are exercised.

Use the existing owning-module test shapes (`tests/patching.rs`,
`tests/orchestrator_protocol.rs`, `tests/workspace.rs`) without editing committed
tests. Required format/clippy/all-target/all-feature checks and independent default
reference comparison remain mandatory. A separately admitted bounded actual-agent
scenario must demonstrate natural explicit proposal → real private RED → same
session implementation → ordinary acceptance/check/review, shared-tree identity
and closed process/evidence receipts. A scripted deterministic replay qualifies
plumbing only; it cannot replace that actual workflow. No such run is authorized
or claimed here.

## Ownership and authorization conclusion

`docs/architecture.md:844–870` assigns directives/commands to the orchestrator,
locking to `locks`, and patch computation to `patch`; `:950–972` permits new core
behavior in its owner and defines the actual authorization boundary. A private
preview module called by `orchestrator`, with crate-private patch preparation,
private `CommandChat` owner/state wiring and provider-neutral existing sends,
does not require a different UI/controller/provider integration path. No public
`AgentDirective` exists. Do not add preview variants to public `OrchestratorEvent`,
`PatchOutcome`, `PatchError`, `AgentStreamEvent` or backend traits merely to obtain
logging: that would create a separate API-compatibility question.

Therefore **human architecture authorization is not inherently required just to
implement a private preview in these owners**. Explicit approval of this new
benchmark workflow, its source/execution contract and a prospective architecture
paragraph is still required before code or admission. The current architecture's
prompt-only intervention description is not permission to present an execution
intervention as a string-only change. If implementation needs public API changes,
provider-owned execution, UI-owned workflow logic, global lock/scheduling changes
or stronger product-wide sandbox promises, stop for the applicable authorization.

The unresolved implementation choice is the concrete confined executor and its
supported materialization/environment contract, not whether a shell exit proves
RED or a temporary directory is safe. Until that choice is approved and qualified,
the safe fixed-artifact replay is evidence for feasibility, not a ready natural
C15 intervention. This document computes no tokens, counterfactual prices, causal
shares or additional observations; private feedback necessarily includes its own
transport, snapshot/execution and context-retention effects.

## Inspection identities

The inspected current files had no Git diff against HEAD
`9d4d87c939f2f825f71005b2d0a9be21350c53c9`; these are current implementation
identities, not a claim that the historical H cohort ran version-5 code.

| Path | SHA-256 |
| --- | --- |
| `docs/architecture.md` | `e992c2335acd3c25e42c1c98df4e6aa45b3423de30af9625445cd4389b8da511` |
| `src/orchestrator.rs` | `027c35a9eaf24e99335001e68f7d0612a262831af80f8677874525db95e9332b` |
| `src/locks.rs` | `0590b3409c5f8ad9cd44b0320d24b153d4485a26a8c065a877f8742f0b029697` |
| `src/patch.rs` | `aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda` |
| `src/cli.rs` | `a87be2707fcb873d1cdfba319d9e6d1b07ab6aa7aacdf8e5aa6ff25f98ab1025` |
| `src/agent.rs` | `a9e6065a450d05bf299202a2d6f44dd2dae33a3324e8c70e9f91d36a22b242fd` |
| `src/agent_runtime.rs` | `cae5903d6b34271613fe2ef3a8033eeda3c294a602f2b4b9206b3649881babb0` |
| `src/bench_experiment.rs` | `d23fe2475f645fb6d91db29d54048be87125f3e1ec17cee04dee393aed33e7e6` |
| `src/lib.rs` | `ee1f20cb7d80ceca4903138a67e34de469ad0b7ce70fe79f3f776c5d170389c0` |
| `Cargo.toml` | `09d25465a66dfd04f1c4deddc573e485e886843baf1d7c7a785cae3f2f7b8e72` |
| `bench-three-features` | `d2487780c63c14021904b8a3c882d54fe231c5846f4a7f57fe955f50201f5644` |
| `bench-validation-common` | `235ef644d692098a670c4e61c3572f60f8830c445896f3639a56430c57781dca` |
| `DESIGN-TEST-FIRST-ISOLATION.md` | `99d82be2c35c24dff89bded7eb786a9a18bf955ede11c0e7f68133ba6ffe2493` |

Expected new census work is indexed by proposal/agent, with one snapshot
materialization and byte hashing per proposal, not one scan of all prior proposals.
Repeated whole-tree materialization is O(P×B) for P proposals and B source bytes;
it must be disclosed, bounded and not advertised as constant-time. Existing patch
matching and test execution costs remain additional. No runtime or tests were
changed or executed for this design; no agent-facing behavior is affected by
writing it.
