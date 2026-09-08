# C08 controlled stale-snapshot qualification

Status: source preparation only; no admission or provider execution. The three
C15 workflows are terminal; the build hold is released. The new guard contract
has observed missing-module RED and four passing implemented tests. Actual
CommandChat wrapper coverage, complete repository gates and independent exact-cut
review remain required. The separately reviewed C08 runtime cut is unchanged.

## One scenario, not a natural effect observation

The authority is the qualification section of
[the prospective design](../../../DESIGN-AUTOMATIC-REFRESH-REPRESENTATION.md)
and the root-approved controlled mutation point. There is at most one future
admitted real author, one owned thread, one initial launch and seven further
launch/send opportunities in total. There is no reviewer, new control, Direct
run, automatic retry or replacement. One unexposed or failed attempt is retained.

A fresh dependency-free Rust library starts with a function that returns its
first input. The natural request asks the author to implement the smaller-input
behavior, write its own tests, read the existing source through Work Leaf before
editing, run `cargo test --offline --locked` and finish normally. It supplies no
test bodies, desired patch, rejection, validation output or agent reply.

The harness owns exactly one stimulus:

1. The actual prepared author returns its own single mediated read directive for
   `src/lib.rs`. The entire actual reply is retained. Copied/fenced directives,
   another directive, another author or a mismatched full delivery do not authorize
   a mutation.
2. The backend wrapper receives the ordinary complete inline read response. It
   verifies the same author, previous actual reply, exact whole expected renderer
   output and exact initial project bytes. It records the complete input without
   changing a byte.
3. Before forwarding that input, exactly one normal `GitPatcher::apply_edit`
   accepts a semantically neutral `value` to `value + 0` edit. The actual accepted
   commit, before/after HEAD, patch request, source bytes and affected paths are
   retained. An attempt receipt precedes this operation; failures prohibit a
   second attempt. A distinct host fixture-mutator identity owns this commit;
   the real author's identity records whose held snapshot receives the stimulus.
   This host-owned commit is not attributed to agent-authored work.
4. The real backend receives the unchanged old read response and produces its own
   next action. Only an actual ordinary stale submission, actual conflict and
   eligible v7 full-current refresh can satisfy exposure. If its action happens
   not to be stale, the scenario is unexposed; no forced submission is substituted.
5. The same real author must repair through normal apply/ACK, execute the ordinary
   focused locked check and have its `@work-leaf done` processed by CommandChat.

The `value + 0` stimulus is equivalent for every `u8`; it does not implement the
requested smaller-input behavior. Final behavior and authored test purpose still
require independent source/semantic review. A passing command is not alone that
review and is not a net-saving observation.

## Timing, ownership and non-targets

`src/orchestrator.rs::read_requested_files` returns after
`FileLockTable::with_read_locks` releases the ordinary read lock.
`send_file_read_response` calls the backend before its existing
`FileReadTracker::record_snapshots`. The controlled stimulus runs inside that
backend call: it neither acquires nor updates the private tracker. The held old
snapshot is recorded by the original code after the real send returns. The
subsequent ordinary automatic-refresh branch advances its current snapshots at
the original before-send point. No extra runtime read, diff, bundle allocation,
tracking callback or provider API is introduced.

The synchronous fixture patcher uses a separate `FileLockTable` for the same
canonical project; it does not claim to share CommandChat's private lock table.
There is one author and no other admitted writer at this point. The source call
chain, not timing sleeps or a fake tracker, establishes the released-read-lock seam.

CommandChat retains its normal default round limit. The focused locked command
timeout is 30 seconds. One cooperative 240-second watchdog shuts down the cloned
CommandChat provider handle; a separately admitted outer argv must be exactly
bounded by `timeout --kill-after=5s 300s`. Neither timer grants another observation.
Call limits count real backend launch/send attempts, including failed attempts,
not shell commands, response frames or hypothetical billing units. No native tool
is prohibited or assumed absent by the harness; the complete native census remains
required. The fixture does not execute any extra validation command itself.

The prospective environment uses the subscription observer proxy, gpt-5.5/xhigh,
read-only provider sandbox, raw response forwarding and the existing 1000-ms grace
with output-resume forwarding. Explicit observer proxy-bin PATH instrumentation
must be recorded and pinned, including the shell/native executable chain. No
credential value belongs in a receipt. Existing observer warnings and original
analyzer/extractor results must remain unchanged. The exact paths/configuration,
source closure, global-config check and actual executable are future admission
inputs, not implicit defaults established by this source document.

## Evidence and success boundaries

Before any provider call, create-new `HARNESS-ATTEMPT.json` owns this attempt.
Every attempted delivery has a create-new `CALL-nnn-INPUT.json` with explicit
author/call/kind and full input, followed by a separate result or error receipt.
Mutation uses separate `MUTATION-ATTEMPT.json` and `MUTATION-ACCEPTED.json` files.
`HARNESS-RESULT.json` retains workflow return, call count, mutation state, watchdog,
observer stop exit/stdout/stderr and capture closure/error. Publication is
create-new, not a multi-file transaction; partial publication remains evidence of
failure. Full held/forwarded bytes and ordinary FNV metadata remain available for
separately computed offline SHA-256; no runtime crypto dependency is required.

The unchanged D3 `settled_turns` function can be reused through its private crate
dependency for accepted call cardinality and latest-turn transport settlement.
It is not a complete native/public lifecycle proof. No v6-only D3 completion,
private-preview or review predicate is reused for C08.

The C08 pure guard must join an owned eligible trace row to exactly one recorded
full input occurrence after the controlled read; validate the exact current body
through construction-owned UTF-8 spans and byte-identical non-target intervals;
then require an actual later owned ACK, status-zero non-timeout focused-command
result and sole final DONE reply. CommandChat must actually return after processing
that reply. These are local harness facts, not accepted/native proof or semantics.

After closure, a finite independent operator witness must verify every actual
call's original/forwarded RPC equality, unique accepted request/thread/turn IDs,
public user-item full bytes, explicit-turn native input full bytes, model/effort/
cwd and every usage-less thread. It must preserve all public/native action/output
identities and lifecycle gaps rather than infer absence from a missing public tool.
In particular it joins old read → accepted intervening commit → actual rejection
diagnostic → v7 eligible current body → real repair commit/ACK → actual locked
command/check result → processed DONE. Native compaction or missing evidence is
retained, not silently repaired. Original observer commands, if applicable, are
separately bounded and once-only; no whole-workflow accounting or percentages are
part of this qualification.

Only that complete source/public/native and semantic review may say qualification
passed. A harness exit zero, successful stop, fixture invocation count or a full
prompt containing copied marker text cannot establish it.

## Required source gates before admission

- Observe the new guard contract RED before implementing its missing module.
- Prove wrong-author, duplicate input, wrong trace/body, changed non-target bytes,
  wrong ordering, timeout, missing check, copied/fenced DONE and marker-only read
  rejection with provider-free tests. A source-only draft is not a passing test.
- Add a provider-free actual CommandChat wrapper test with the controlled accepted
  GitPatcher stimulus, proving untouched read forwarding and genuine recovery.
  This is separately labeled synthetic backend coverage, not real qualification.
- Run fmt, all-target/all-feature clippy and tests after the C15 hold, then obtain
  independent exact-cut review of the runtime and harness. No committed old test
  may be altered to obtain those gates.
- Freeze one fresh project/config/trace/evidence namespace and full source and
  executable chain before the sole provider admission. No reused result/attempt
  path is allowed. No current provider admission follows from this scope.

The per-call local trace scan is bounded by eight admitted calls. It may cost
O(K × T) for K calls and T trace bytes, with K ≤ 8 fixed here; it is not a reusable
unbounded-history analyzer. The runtime C08 renderer's linearity is unchanged.
