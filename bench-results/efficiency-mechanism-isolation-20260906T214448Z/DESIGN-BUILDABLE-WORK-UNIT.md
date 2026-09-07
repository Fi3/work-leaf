# Prospective buildable work-unit policy

Status: treatment specification implemented under the nondefault `bench-experiments` feature;
not an admission record. The completed initial screen and its frozen sources, helpers, results,
and settings are independent of this design. A later phase requires a separate hypothesis,
allocation, observation count, analysis rule, source freeze, and admission decision.

Condition identifier: `buildable-work-unit-incremental`. Comparison: ordinary `control`
in the same newly frozen feature-enabled binary. This is one semantic factor: the preferred
unit of implementing and handing off requested feature work. The treatment favors naturally
separable buildable increments and permits remaining requested work after an accepted increment;
normal WL favors a cohesive submitted patch followed by focused validation and review readiness.

## Intended mechanism and limits

The hypothesis is:

natural buildable increments and continued remaining feature work → different submitted work
units → additional or differently ordered edit/check/read continuations → different repeated
conversation input and total raw tokens.

This is not strict RED-before-implementation, per-file editing, or compulsory fragmentation.
Required tests, conditional test design before implementation, no-known-red shared states,
and all final acceptance requirements remain mandatory. Mutually dependent tests and source
changes stay together. There is no minimum number of patches, maximum files per patch,
mandatory extra check, empty patch, or manufactured intermediate failure.

A submission is not a model call. `src/orchestrator.rs::handle_agent_directives_streaming`
collects successful edit paths and can issue one acknowledgment for multiple directives from
one response. Actual response identities and continuation boundaries, not commit counts alone,
are the mechanism evidence.

The earlier `DESIGN-BATCHING.md` is a narrower neutral-preference design. It leaves the
acknowledgment's “another edit only for a validation defect” restriction intact. This specification
treats that remaining-work restriction consistently with both launch-policy copies. Its effect
would belong to the complete work-unit policy, not independently to either sentence or RED timing.

## Exact treatment spans

All offsets refer to renderer-owned UTF-8 byte spans in the original full delivered prompt.
Original repository instructions and user text remain data even when they contain identical words.
Do not globally search-and-replace the assembled prompt.

### A. Shared non-linearizer policy: `policy-buildable-work-unit`

Owner: `src/agent.rs::PromptPolicy::for_read_permission`, the second sentence of the final
shared-worktree instruction. Preserve its first sentence exactly:

```text
Keep the shared worktree usable for the other patch agents: do not submit known-red, compile-breaking, or deliberately failing intermediate patches.
```

Old span:

```text
Design tests before implementation when required, but submit a cohesive patch that includes the test and the implementation needed for the shared tree to build.
```

New span:

```text
Design tests before implementation when required. Prefer naturally separable, independently buildable feature increments, keeping mutually dependent test and implementation changes together. Submit each completed increment and continue any remaining requested feature work. Do not split inseparable work or create otherwise unnecessary patches.
```

### B. Test-instruction translation: `instruction-tests-work-unit`

Owner: `src/agent.rs::concurrent_instruction_translation`, only the existing `topics.tests`
branch. Preserve the prefix `- Test requirements remain mandatory. ` exactly.

Old span:

```text
Design the needed tests, but submit tests with the implementation needed to keep the shared worktree buildable.
```

New span:

```text
Design the needed tests, and prefer naturally separable, independently buildable feature increments, keeping mutually dependent tests and implementation together. Do not split inseparable work or create otherwise unnecessary patches.
```

Emit and treat this span once per instruction-file translation already selected by
`InstructionTopics::detect`. Do not change topic detection, instruction loading, original
instruction text, or the checks/docs/commits/reviews/real-agent-verification translations.

### C. Successful patch acknowledgment: `patch-applied-remaining-work`

Owner: `src/orchestrator.rs::render_patch_applied_prompt`, its final completion/repair paragraph.

Old span:

```text
After the focused validation passes, or after you report an external blocker, emit a top-level `@work-leaf done` so review can start. Send another edit only if validation found a concrete issue in your own patch.
```

New span:

```text
After the focused validation passes, or after you report an external blocker, continue with another independently buildable increment if requested feature work remains and can proceed without taking over another agent's work. Otherwise emit a top-level `@work-leaf done` so review can start. Send another edit only to implement remaining requested feature work or to repair a concrete issue that validation found in your own patch.
```

Every other acknowledgment byte remains unchanged. In particular, retain exactly:

```text
run at most one focused validation step that is relevant to files you touched or checks you added.
```

The provisional-commit receipt, lock-command syntax, other-agent test prohibition, report-once
external-blocker rule, and prohibition on unrelated repair remain intact. “Another increment”
does not authorize taking over another feature or leaving a known-red shared state.

## Fixed nonfactor behavior

- Preserve read permissions, read batching, snapshots/digests/diffs/context bundles, command-output
  rendering, tool availability, structured-edit representation and parsing.
- Preserve `GitPatcher` application, immediate provisional commits, patch ownership, lock
  acquisition/rejection, command timeouts, pending-diff capture/reversion, and error handling.
- Preserve validation scope and cardinality: the normal at-most-one focused check after an
  accepted submission remains. More submissions can naturally cause more such checks; that is
  an observed consequence of the work-unit factor, not a separately strengthened test policy.
- Preserve command-result guidance, failure/ownership guards, review requirements, reviewer
  source context, review/fix routing, linearization, final gates, and feature requests.
- Preserve provider model/effort, session reuse, launch/send branch selection, subscription
  authentication, concurrency, interruption detection, and 1,000 ms/`forward` timing.
- Preserve each role's existing injection scope. Every non-linearizer policy injection receives
  A and any existing B copies, including reviewers and hidden agents. Existing linearizer
  detection and its distinct prompt remain unchanged. Do not infer patch-only scope from run
  labels, feature names, or ad hoc agent-ID conventions.

Whole-workflow effects can include downstream review behavior. Claims about implementation-only
mediation require stage/agent evidence; those roles are not silently excluded from the treatment.

## Private fallible policy-delivery contract

The public `PromptPolicy::inject(&self, agent_id, feature, prompt) -> String` signature and
normal rendering remain unchanged. It remains an infallible baseline renderer, with no
environment-controlled mutation or evidence I/O. No new public re-export or provider API is needed.

Use one private rendering implementation shared by `inject` and a new crate-private
`PromptPolicy::inject_for_delivery(...) -> Result<String, AgentError>` method. Its rendered
value contains text plus feature-gated owned-span metadata. Construct A's span where the preamble
fragment is assembled; carry offsets through joins/appends. Construct B's spans at the existing
translation emission sites and offset them when assembling the full policy. Do not rediscover
spans by searching copied instruction or prompt text.

For a default build, `inject_for_delivery` simply returns the normal rendered text and compiles
out experiment access and span bookkeeping. For a feature build, it validates and forwards the
rendered text through the private `bench_experiment` owner; evidence failures become the existing
`AgentError::Io` before the provider's turn request. There is no ignored write failure, extra
model message, or semantic policy implementation inside a provider adapter.

Both configured providers already have four normal policy-injection sites:

- `src/codex.rs::CodexBackend`: `launch_streaming`, `launch_streaming_interruptible`,
  and the no-existing-session branches of `send_streaming` and
  `send_streaming_interruptible`.
- `src/claude.rs::ClaudeBackend`: the same four sites.

Route only those existing injection sites through the private fallible method. Known-session
follow-ups stay raw. Preserve the same operation guards, session lookup, sandbox choice, and
provider request calls. The shared renderer owns the experiment; provider adapters only propagate
the existing error type. A custom external provider calling public `inject` keeps baseline
behavior and is not silently enrolled in this private benchmark mechanism.

Extend the private successful-acknowledgment metadata to identify C while retaining the original
validation-cardinality span. Do not transform the whole acknowledgment through a generic provider
filter. The command-result boundary remains identical.

## Phase-local admission and evidence contract

The existing nondefault Cargo feature `bench-experiments` remains the compile gate.
Use the existing explicit environment marker and absolute manifest path, require
`WORK_LEAF_BENCH_RUN_ID` equality, and create-new evidence files. No benchmark environment
can activate a variation in a default product build.

The proposed new phase uses manifest/trace schema `work-leaf-bench-experiment-v2`, with the same
four manifest fields: `schema`, `run_id`, `condition`, `evidence_path`. Its permitted
conditions are `control` and `buildable-work-unit-incremental`. Version 1 manifests and their
two existing interventions retain their current semantics and trace shape; new injection hooks
must not add version 2 records to a version 1 study. Frozen initial-screen helpers are not edited
to reinterpret this phase.

Version 2 records one full-prompt evidence row per existing policy-injection, successful-patch,
or command-result delivery boundary, identically for control and treatment. Each row includes
the existing run/agent/process/sequence/time identities, original and forwarded text and byte
counts, and an ordered `spans` array. Each span includes its stable span identifier, original
UTF-8 `cue_start`/`cue_end`, declared original/replacement text, changed flag and byte delta.
The policy row can contain A plus several B spans; a linearizer row has no treatment spans.
The acknowledgment row includes C and can retain the unchanged cardinality span as an explicit
integrity check. Only A/B/C can differ under this condition.

Validate ordered, disjoint, in-range spans and exact original text before writing or forwarding.
Construct forwarded text in one pass over original gaps and replacements. Do not repeatedly
clone the full prompt or call `replace_range` per instruction file: that would introduce
O(number-of-spans × prompt-size) work, potentially quadratic. Required rendering/logging cost
is O(original bytes + replacement bytes + span count), with full text logged once per boundary.

New phase-specific analysis must validate the declared spans and bidirectional
trace ↔ actual request coverage, including the no-existing-session policy-injection fallback
and zero-span linearizer identity. Known-session follow-ups remain raw. Do not treat one trace
span as an extra response.
Keep the frozen version 1 analyzer and results untouched.

For these exact ASCII strings, A is +184 bytes, each B is +122 bytes, and C is +219 bytes.
A non-linearizer injection with `k` existing test translations changes by `184 + 122k` bytes.
Each successful acknowledgment changes by 219 bytes. These are prompt byte deltas, not tokenizer
counts or a causal share. Report them without padding; direct length effects and downstream
behavioral effects must not be conflated.

## RED-first acceptance gates before any admission

Add new tests without modifying committed tests unless separately authorized. Observe their
failure before implementation, then verify:

1. Entire default/control prompts are byte-identical to baseline across both read permissions,
   no instructions, multiple instruction files, tests/no-tests translations, Unicode, and
   duplicate cue text embedded in original instructions/user data.
2. A, every existing B, and C change only their declared spans. All preserved safety, test,
   focused-check, at-most-one cardinality, lock, receipt, and unrelated translation text remains
   exact. Linearizer text and known-session raw follow-ups remain unchanged.
3. Public `inject` stays infallible/baseline. The private delivery path reaches all eight actual
   provider injection sites without changing turn/session topology; evidence/configuration
   failures propagate before generation. Version 1 activation/records remain unchanged.
4. Version 2 control and treatment share instrumentation and validate ordered span arrays,
   aggregate byte arithmetic, exact request identities, and reverse exposure coverage.
5. The full required format/clippy/test checks pass. A bounded real configured-agent smoke
   verifies actual policy delivery plus structured edit → unchanged-cardinality acknowledgment
   → remaining requested work → completion, without extra artificial stress or revised grace.
   Diagnostic scaffolding is excluded from mechanism observations.

Before implementation/admission, review the resulting private API wiring, phase-local schema,
documentation implications, and source freeze. No new provider work is authorized by this file.
Normal user workflows are unchanged unless a later explicit benchmark manifest enrolls them.

## Mechanism evidence for the eventual phase

Retain every admitted outcome. Link first policy exposure to actual submitted source/test
grouping, meaningful feature increments, accepted/rejected edits, acknowledgments, checks,
remaining-work continuation, readiness, review/fix rounds and unique response IDs. Report raw,
cached, uncached and output usage with unresolved generation coverage separate.

Observed extra fragmentation alone is insufficient: verify it implements genuinely remaining
requested work and does not arise from an unrelated failure or forced busywork. No packaging
movement means this factor has not demonstrated its proposed mediator in that phase. A positive
effect estimates this complete work-unit policy in the frozen environment, not automatically
the entire historical Direct–WL difference and not the effect of strict test-first development.
