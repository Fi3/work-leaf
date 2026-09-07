# C15 private runtime boundary review

The detailed design fits the current core owners, but it is not a prompt-only
factor and the standalone helpers are not a ready runtime path. The smallest
credible integration has explicit author enrollment, a distinct typed preview
envelope/state, immutable selected-source evidence, private execution and a
truthful same-session continuation. This review admits no implementation, provider,
baseline workflow or runtime change.

The reviewed [detailed design](../DESIGN-TEST-FIRST-ISOLATION-DETAIL.md) SHA-256 is
`9e4911e63cbd5fac54475552dc883d15c4448eaf149a4e8e2a134df8328240ff`.
Current source hashes match its listed `cli`, `orchestrator`, `locks`, `agent`,
`patch` and architecture identities. Additionally, `src/workspace.rs` is
`f93f2fe2b7481cc1689e5f0aa4ed295e8736af035555fbf84197024ffc80de17`.
Architecture's core-owner/API boundaries and the active operator policy were
inspected. The public API need not change; architecture must describe the new
private execution intervention if it is implemented and approved.

## Enrollment must precede injection and survive worker cloning

`CommandChat::launch_prepared_agent_streaming_with_ids` is **not author-only**:
`cli.rs::linearize` (1424–1427) and controller linearization also call it.
Therefore copying the v4 successful-launch registry would incorrectly enroll
linearizers. That registry also commits only after backend launch returns, while
`PromptPolicy::inject_for_delivery` executes inside that launch; it is too late
to select the author's initial treatment policy.

Concrete private role contract, without modifying public `AgentLaunch`:

1. A feature-only `Arc<Mutex<...>>` registry belongs to `CommandChat`, shared by its
   clones at `cli.rs:652–674`. It stores an instance/run identity, monotonic launch
   generation, typed role, exact launch value and transaction state. Author role
   is issued by `prepare_agent_launch`; `prepare_linearize_launch` issues a
   non-author ticket. Reviewer and system launch owners remain explicitly outside
   enrollment. No prefix, title, feature wording, filename or provider session
   inference supplies role.
2. The controller delegates author preparation to `CommandChat` at
   `workspace.rs:667–672`, then queues or defers the same launch before cloning
   the worker (`start_worker`, 1174–1192). Two legitimate mutation sites need
   private ticket revisions: `set_pending_dependent_launch_prompt` (833) and
   `apply_agent_title` (1444–1449). An exact old-value/new-value revision preserves
   role and generation; arbitrary public preconstructed/mutated values cannot
   silently acquire ownership. Canceled/deferred tickets remain distinguishable.
3. Immediately before the backend call, atomically consume the one matching
   prepared ticket and establish a scoped provisional delivery context carrying
   its owner/generation. An absent, reused or ambiguous ticket fails before the
   provider under active C15; inactive and legacy-schema calls are unchanged.
   A public embedding that bypasses preparation is unsupported for C15, not
   guessed to be an author. Exact copies of the authorized prepared value are
   equivalent data; public `AgentLaunch` provides no unforgeable object identity.
4. A benchmark-private scoped context can bridge the synchronous owning launch
   to `inject_for_delivery`, which already receives the exact ID/feature/prompt.
   Validate those bytes against the provisional ticket, transform only its owned
   timing spans, and consume its injection identity. Do not hold the registry
   mutex while calling the provider. The bridge must be tested at the actual
   injection sites; unsupported cross-thread/deferred injection must fail closed,
   not silently send baseline policy or rely on an unrelated agent-ID lookup.
5. Successful backend return commits that generation **before** processing its
   first reply; launch failure/panic rolls back only its provisional transaction
   and preserves the preceding committed owner. Evidence records attempted
   injection separately from successful launch. A successful relaunch gets a new
   generation; old preview success cannot authorize its first shared submission.

Queued title changes, dependency release, canceled preparation, clones created
before preparation, failed relaunch, same-ID collisions and non-author launches
are the minimum identity regressions. Ordinary promotion/send is not a fresh
launch; its treatment applicability must be explicit rather than silently
inventing another generation or resending the launch policy.

## Minimal core seams

| Owner | Private boundary and non-target requirement |
| --- | --- |
| `orchestrator.rs::AgentDirective`, `parse_agent_directives` (1701/1730) | Add only an active private preview envelope or separate private parse result. Validate the whole envelope and reject mixed ordinary side effects before any branch executes. Do not reinterpret the first ordinary edit as a test. |
| `DirectiveStreamInterruptDetector`, `should_interrupt_after_streamed_directive` (631/1844) | Complete preview framing gets one terminal handoff. Resolve eligibility from the owning context; preserve every ordinary directive's old stop behavior and provider grace. Incomplete/copied/fenced/malformed envelopes have no preview effect. |
| `DirectiveServices`, `handle_agent_directives_streaming` (242/685) | Pass a reference to the shared private owner/state service. Dispatch preview before normal apply branches, with state keyed by run/launch generation/agent/proposal. Same-ID changed bytes reject; repeated transport never executes twice. |
| `run_command_for_agent` (1176) | Reuse/factor only normalization, current ownership restrictions and failure-masking admission. Do not call its shared cwd runner, dirty-diff capture/revert, pending-change tracker or ordinary command-result/`CommandRun` event for a private preview. |
| `GitPatcher`, ordinary Patch/Edit branches (789/861) | Reuse exact existing patch semantics only in the independent private repository. Only later ordinary shared acceptance changes ownership, clears snapshots/pending diffs and enters the normal ACK group. |
| `AgentFollowUp`, `send_agent_streaming_interruptible`, CLI queue (618/657/1093) | Deliver one distinct preview-result prompt through the existing session/round budget. No fake applied ACK, ordinary command event, extra launch policy or counter reset. Persist result before sending; send failure keeps executed evidence and cannot rerun validation. |

The first-submission guard can mechanically gate ordinary Edit/Patch after
enrollment and before shared application. It cannot distinguish a “production”
patch from a test-only patch semantically. A delivered first GREEN still satisfies
ordering; failure status is not manufactured. Done without a qualifying proposal,
test revisions and nonactivation remain outcomes, not forced RED or automatic
replacement work. Claimed unchanged-test RED→GREEN requires actual retained test
units and accepted after-images, not a filename or exit-code heuristic.

Preview state has separate held, source-bound, executed and result-delivered
phases. A successful backend send is a runtime delivery witness; exact accepted
typed request/native input remains the later capture audit's stronger witness.
An uncertain send cannot be relabeled delivered merely because its text was
rendered. Private cancellation must reach the executor from the owning interrupt
and shutdown paths (`cli.rs:737/774`), without registering it as a provider or
altering ordinary provider shutdown. No mutex/root lock survives execution/send.

## Snapshot selection and scheduling

Use the **same cloned `FileLockTable`** in `DirectiveServices`, not a new table or
file lock. Its `with_read_locks(["."])` closure can select commit/tree and capture
source evidence against normal `GitPatcher` index/commit serialization, because
`patch_lock_paths` adds `.`. Commands lock exact requested keys; `.` does not
exclude a command holding only `target` or a test path, external writers, or all
transient live bytes. The live-overlay census must retain that limit.

The live-overlay owner proposes a pinned Git bundle captured in this actual
selection closure, then independent materialization after release. This avoids
the old standalone `materialize(shared, expected_commit, ...)` requirement that
live HEAD and full identity remain unchanged through cloning. Its runtime source
must be the captured selected objects, never a later live HEAD. Unexpected live
index/flag/blob/overlay differences reject, without clearing flags or retrying.

Git bundle creation while holding `.` still delays normal shared patch commits;
it is additional scheduling work, not a zero-duration pointer read. Bound and
record bundle bytes and lock duration. No full clone, validation, provider turn or
global all-writer barrier belongs inside that closure. The proposed standalone
Rust fixture may prove actual shared-table serialization, not product integration.

## Private environment is a real token mediator

Ordinary `run_shell_command` (1435) uses host `sh`, inherited environment and an
optional command TMPDIR. The qualified private executor clears environment,
disables network, changes HOME/Cargo/build state and supplies explicit read-only
public tool inputs. The project proof uses a fresh build and prints compilation
of 25 dependencies before 28 passing tests. Its same-path source view does not
make those environments or outputs equivalent.

Cold compilation can add retained compiler output and prompt charges; different
configuration/dependencies can instead cause setup/compile failures and repair
turns unrelated to the intended missing behavior. Ordinary later checks remain
unchanged, but the extra private preview includes these costs and may influence
subsequent model decisions. They cannot be labeled pure RED-timing savings.

The minimum further **provider-free qualification**, not another baseline
workflow, is a frozen existing test-only/implementation pair with exact test bytes
and command. Run both images in the supported confined environment and verify
the named behavioral RED followed by the same-test GREEN. Retain invocation,
compile/setup, zero-test and unrelated failures separately. An optional same-image
fresh-versus-already-built private replay can expose compiler-output/cache effects
without generating model continuations; keep it diagnostic, not silent runtime
prewarming, a new baseline run or a subtraction from workflow cost.

Source/environment/command/output identities and ordinary output-compaction rules
must be fixed prospectively. Arbitrary private commands receive their exact
declared shell text, not a known benchmark filter or hidden passing substitute.
Unsupported environment needs remain failures. No finite set of such checks
proves host-environment equivalence; the achievable factor is the explicitly
qualified **private test-first feedback package**. Existing baseline workflows
can supply the retained comparison without new controls, while observed preview
charges and repair chains remain separate descriptive evidence rather than a
counterfactual timing-only share.

The runtime still needs RED-first default/legacy identity, enrollment, framing,
state/replay, isolation/cancellation and normal final-ACK tests, plus an explicitly
admitted bounded real-agent scenario before readiness. This review performs none
of those future operations. Indexed per-owner/proposal state is appropriate;
snapshot/bundle/hash work is bounded per proposal and disclosed, not repeatedly
joined against every prior conversation or advertised as constant-time.
