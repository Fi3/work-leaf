# C15 closed-source qualification plan

This plan authorizes no execution by itself. Root owns the one separately admitted
diagnostic and its terminal publication. No collector, extractor, analyzer,
private executor or provider was invoked to prepare this source-only mapping.
The scope is factual workflow/transport/private-test qualification, not whole-run
accounting, a token comparison, a percentage or a replacement observation.

Let `D` be this directory's absolute path:

```text
/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-real-diagnostic
```

## Original terminal publication

Retain `ADMISSION.json`, its exact once-only attempt, complete process output,
the supervised command return code and `HARNESS-RESULT.json` when published.
Missing harness output after a startup error is an outcome, not permission to
rerun. A quiet stream is not process/capture closure. Preserve watchdog/kill,
scenario, observer stop and source-qualification facts separately.

The source-reviewed `bounded_launch.py` publishes schema
`work-leaf-single-diagnostic-terminal-v1` and these explicit fields:

```text
run_id: private-test-first-diagnostic-001
id: private-test-first-diagnostic-001
condition: private-test-first
observer_condition: work-leaf
launch_status: completed
started_at: actual timezone-aware ISO-8601 command start
finished_at: actual timezone-aware ISO-8601 supervised-command terminal observation
launcher_exit_code: actual integer supervised-command return code
```

`launch_status: completed` means the launched supervised command is terminal,
including exit 101 or a timeout-wrapper failure; it does not mean workflow success.
Record a genuine spawn failure as such instead of manufacturing a completed
process. Preserve native child exit/signal/unknown separately from a timeout
wrapper's return code. An outer kill must not be represented as a proved closed
private child. Additive `exit_code`, `terminal_observed_at`, durations and original
failure fields may remain, but do not substitute them for `finished_at` and the
typed launcher field. Use `2026-09-07T…+00:00`/`…Z`, not a space-separated `UTC`
label that the pinned legacy `datetime.fromisoformat` contract cannot read.

The launcher waits for the exact admitted supervised argv; its actual terminal
exit and source-endpoint verdict remain separate. Create-new attempt/output files
prohibit retries, including after partial publication. This is exclusive, flushed
publication, not an atomic-rename/crash-transaction guarantee. The admitted argv
supplies the external timeout; the launcher does not construct provider arguments
or infer a bound for an arbitrary command.

This is the generic workflow branch of
`audit_review_evidence_sources.py::terminal`, not a fabricated v5 review terminal.
The harness's existing `condition: work-leaf` is its observer/workflow-family
label; preserve that file unchanged and explicitly map the diagnostic factor
condition in the outer receipt. Prior C21-002 terminal publication lacked the
required condition/id/status/finished/launcher fields and used another time
format. Its original failure remains untouched. Correct C15 publication does not
make the C21 collector itself applicable to C15.

## Once-only original command order

After actual closure, pin every admitted source/executable and all closed raw
captures, start/end/child metadata, prompt trace and private operation evidence.
Resolve all outputs before work, reserve a create-new attempt per command and
verify publication paths/writer access. Each command has one attempt and a
180-second operator bound with five-second final kill grace. A timeout or failed
publication remains failed; neither authorizes the command to run again.

The exact original observer is:

```text
/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/review-evidence-native-diagnostic-002/infrastructure/bin/bench-observer
SHA256 238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386
```

1. Run that binary with exactly
   `analyze --config D/observation/observer-config.json`. Retain verbatim stdout in
   `D/postcapture/OBSERVER-ANALYZE-ORIGINAL.json`, stderr in its `.stderr`, and
   argv/start/end/status/output hashes in
   `OBSERVER-ANALYZE-ORIGINAL-EXECUTION.json`.
2. If `observation/mechanism-summary.json` was actually published, run the same
   binary exactly once with `extract-rollouts --config D/observation/observer-config.json
   --sessions-root /home/user/.codex/sessions`. Retain analogous
   `OBSERVER-EXTRACT-ORIGINAL.json`, `.stderr` and
   `OBSERVER-EXTRACT-ORIGINAL-EXECUTION.json`. If the prerequisite is absent,
   record the not-launched reason, not an empty successful extraction.
3. Freeze the resulting original reports and derived observer source inventory.
   Build the separately reviewed finite C15 witness input from those exact closed
   artifacts and all accepted native threads. Only after its own implementation,
   RED/GREEN tests, independent review and explicit invocation scope may a new
   witness execute. Preserve every original flag beside its narrower findings.

Here `D/...` is a path placeholder to expand before recording literal argv, not a
shell variable that a helper resolves. Outputs use exclusive create-new streams;
stdout is never parsed/rewritten to simulate JSON when a failed command returned
empty or malformed output. Full streams stay on disk; operator stdout can project
only status/path/hash, without displaying the observer's token totals.

`bench-observer::analyze` must precede `extract_rollout_metadata`: it writes
`process-invocations.jsonl`, `mechanism-summary.json`, `counterfactuals.jsonl` and
`capture-audit.txt`; extraction requires that summary and writes
`rollout-metadata.jsonl` and `rollout-audit.json`. These are explicitly allowed
derived publications, not immutable raw source. Verify they do not already exist
as a previous postcapture attempt, except a preexisting process inventory that is
retained by hash before its documented regeneration. Do not run analyze again
after extraction: it can consume the new rollout audit and supplement usage,
which would replace the original report rather than preserve it.

Both original commands apply to a Work Leaf app-server capture without recognizing
the C15 directive. Their mechanism taxonomy does not understand private previews;
private shell work is separate from ordinary observer locked-shell work. The
harness publishes no controller-state usage file. The frozen reconciliation
function returns an empty reconciliation when that file is absent; do not invent
a controller mismatch or claim a performed reconciliation. Ordinary interrupted
usage or rollout mismatches remain original flags. None is automatically a
provider crash, private-test failure or missing generation equal to zero.

## Source compatibility and permitted reuse

The frozen observer source copies and current observer source are byte-identical:
`bench-observer/src/lib.rs` SHA-256
`188a1b4fd9913556c51024dcc0068be353813c44c688d6d22929da139f392b5f`,
`src/main.rs` SHA-256
`6e7dc30c6166f8e3a050480a265d06476590f8ec0eca96b18c9a9473e85d4643`.
The owning functions inspected are `analyze`, `extract_rollout_metadata`,
`reconcile_controller_usage`, and the CLI's analyze/extract exit decisions.

The unchanged C21 collector is **not** an applicable original C15 CLI:

- `audit_review_evidence_sources.py` SHA-256
  `d3fd8c80746bf4bce565cb5f0a2e2eab29681b3aa40f89196cf95ebf344ed9ee`
  admits only v5 review conditions, eight fixed older runtime sources, a v5
  build attestation and opaque-review archive activation. C15 has schema v6,
  another source/build cut, private operation receipts and no opaque archive.
- `audit_review_evidence.py::join_review_inputs` (source SHA-256
  `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257`)
  requires first-policy original/forwarded identity. C15 intentionally varies only
  the explicitly enrolled author's policy spans. A v5 conversion would be false.
- The C21 frame derivative CLI still invokes those v5 checks. Do not run it on a
  fabricated v5 source input or waive its fixed runtime/policy/archive gates.

Small pure pieces are reusable by exact captured-source compilation:
`Sources`/strict JSON framing, `typed_equal`, `one_text`, `native_user_index`, and
the frame correction's `prove_frames(original_raw, forwarded_raw, decisions,
settings)`. The latter source is
`preflight/review-evidence-native-diagnostic-001/postcapture/observer-frame-correction/audit_observer_frames.py`,
SHA-256 `ccfb4cc3fe2a24c6496f4a749e6c95f147d6b5fefa7fd4e91dedc6f40be1961f`.
It imports exact pinned collector/primitive sources at their original relative
paths. `prove_frames` alone proves only journalled initialize/thread-start
metadata changes and exact identity for every other frame. It neither invokes
the collector nor establishes native, model or private execution coverage.
Its wrapper's source/config/start/end/settings/journal bindings must be retained
in the new witness, not replaced by an unverified saved boolean.

## Complete finite witness requirements

Retain zero-call startup failures, all actual 1–8 admitted outer slots and both
intended roles. A failed run may have only one or no accepted thread. Success
requires the intended author/reviewer scope, but never discard partial outcomes
or force a four-turn transcript. Distinguish outer admission slots, accepted
turns, public messages, native items and provider response IDs.

1. **Source and closure:** bind final admission/build/helper/config/manifest,
   terminal/harness/process receipts, complete invocation population and every
   canonical capture. Verify start/end/child identities, raw digests, complete
   physical JSONL tails and no unlisted invocation. Hash before and after.
   Keep failed/partial private directories and every trace occurrence. Recheck
   every actual project-layer inventory and its required pre-spawn-before-child
   ordering. The current harness has no base/pre-linearize/final checkpoint calls:
   do not invent those snapshots or continuous configuration immutability.
2. **Accepted inputs and native membership:** index typed RPC IDs, accepted
   thread/start IDs and successful turn/start replies. Every accepted full input
   must equal its original/forwarded request, complete public `userMessage` and
   native `response_item/message/user` by exact full text and explicit matching
   thread/turn. Preserve separate public/native item IDs and physical locators.
   Consume duplicate prompt occurrences once in order, not a global text set.
   Reject duplicate/conflicting IDs, Boolean/numeric thread IDs and malformed
   explicit/passthrough metadata; require both turn fields to agree when present.
   Do not infer user ownership from preceding turn_context alone.
3. **All native sources:** enumerate exact accepted thread IDs including usage-less
   threads, not only `rollout-metadata.jsonl` rows. Require unique matching native
   session_meta/cwd/CLI and all applicable turn-context model/effort/sandbox values.
   Account explicitly for contextual AGENTS/environment items versus raw user
   input, and retain duplicate item locators or unexplained items. Inspect exact
   native function-call/output pairs and public action coverage. A missing public
   tool event is not proof of absent native work; unsupported action kinds,
   asynchronous output or unmatched IDs remain unresolved, not silent absence.
4. **Policy and ownership:** first-input joins use schema-v6 `policy-injection`
   rows and actual `owned_role`/`launch_generation`, with strict types. Check the
   ordered renderer-owned original/candidate UTF-8 spans and exact unowned bytes
   against final admitted policy source; only explicit author spans may differ.
   Other-role policy remains byte-identical. Attribute the reviewer through the
   actual harness review route and owned first policy, not an Agent-ID prefix.
   Known-session followups remain raw; record any extra policy or role as a gap.
5. **Every private proposal:** join the public generated directive/held body to
   `private-preview-proposal` by exact author/generation/id/revision/format/body/
   command/test_paths/purpose/locks. Preserve malformed requests and blocked shared
   apply prompts, not just successful proposals. Follow each operation's exact
   `capture-input/output`, `CAPTURE-RESULT`, selected receipt/bundle, `root-lock.json`,
   test input/output, `TEST-RESULT` and `TEST-EXECUTION` source references. Verify
   selected commit/tree/effective census, full overlay declarations, no duplicate
   execution for a reused ID, and the accepted-clone `shared.root` versus actual
   `selected_origin.live_root`/private mount distinction.
6. **Private feedback delivery:** bind `private-preview-result` to its exact result
   path/SHA, truthful exit/closed/status and full stdout/stderr; independently check
   the recorded ordinary output-compaction transformation. Join each
   `private-preview-delivery-attempt` to the next exact accepted/public/native
   input for that owner and then its `private-preview-delivered` occurrence.
   `runtime_send_returned: true` explicitly is not a native-delivery assertion.
   Replay delivery may occur without a new execution; preserve both counts.
   A failed preparation or uncertain closure does not qualify the apply gate.
7. **Normal accepted work:** require actual qualified delivered preview before the
   first successful shared patch, then join real patch-applied ACK/commit/files,
   normal focused command directive, exact observer locked-shell stdin/argv/cwd/
   locks/result, completed author and actual review/fix/recheck chain. Preserve all
   extra checks, partial work and failures. Private execution is not an ordinary
   ACK or proof the accepted tree passes; a clean review marker alone does not
   establish the requested ordinary validation command was executed.
8. **Held/final test semantics:** retain exact initial, every held after-image and
   accepted/revised Git source plus public patch locators. Review the actual
   assertion/function/test portions: classify first GREEN, behavior-specific RED,
   compile/fixture/environment failure, revised/removed/ambiguous tests. For a
   separate test-only file, whole-file equality is useful exact evidence. For
   co-located production and tests, changed whole-file hashes are inconclusive;
   give explicit held/final byte/line mapping without claiming automatic semantic
   equality. The preview must not contain the missing implementation to support
   preimplementation RED. Do not execute another check to obtain a desired color.

Ordinary raw followups without a dedicated trace site remain inventoried as actual
public/native inputs, not fabricated trace matches. A missing generated directive,
unmatched delivery or source gap is explicit. No private reasoning is inspected or
exported; safe reports retain public item identities, source ranges, byte lengths,
hashes and factual command outcomes.

## Minimal proposed witness, not yet implementation or admission

If automation is selected, use one new `audit_private_preview.py` with a small
pure `audit_sources(input, sources)` plus create-new CLI:

```text
--input D/postcapture/PRIVATE-PREVIEW-SOURCE-INPUT.json
--input-sha256 <actual closed input SHA>
--output D/postcapture/PRIVATE-PREVIEW-SOURCE-WITNESS.json
```

Input schema `work-leaf-private-preview-source-input-v1` references exact final
admission/build/runtime/helper pins, outer terminal and harness, activation/trace,
observer config/invocations/captures including settings/rewrite journals, every
explicit native session and every actual private operation directory/source ref.
It contains no usage totals and no precomputed passed boolean. Output separates
source integrity, input delivery, private execution, normal workflow evidence,
original observer flags and unresolved rows. Human semantic source mappings belong
in a separately pinned `PRIVATE-TEST-SEMANTIC-REVIEW.md`; automation does not decide
whether an arbitrary failure is the intended RED.

Write tests before implementation: valid 1–8-turn variable/review-fix paths;
zero/partial failed outcomes; typed IDs and duplicate identical prompt occurrences;
owned author versus unowned/cross-thread policy; allowed observer rewrite versus
mutated content; usage-less native membership; missing/public-only tool coverage;
proposal revision and reused-ID/no-second-execution; changed result hash, stdout,
source/overlay or original-live mapping; delivered flag without native witness;
private result mistaken for ACK; natural first GREEN; same test versus co-located
changed production; incomplete tails, missing receipts and create-new output retry.
Record RED, then GREEN and independent review before a single actual witness call.

Use per-thread/proposal/item maps and ordered queues: linear source-byte passes
plus indexed joins, not per-proposal rescans of every native stream. The old C21
archive-read classifier's O(R×B) scan is unnecessary here and is not imported as a
private-test mechanism. Do not add a token ledger, change original accounting,
force fixed turn counts, rewrite old collectors, or expand into a general framework.
