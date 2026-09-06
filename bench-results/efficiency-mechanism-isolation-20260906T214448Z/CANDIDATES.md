# Mechanism candidates

The initial screen contains only the first two candidates. Remaining candidates are prospective
options, not admitted observations or explanations already established by data.

## Initial isolated cues

1. Post-patch validation cardinality. Successful patch handling in
   `src/orchestrator.rs::handle_agent_directives_streaming` sends
   `render_patch_applied_prompt` through the ordinary interruptible continuation. The normal
   acknowledgment requests at most one focused validation step and then completion or an owned
   repair. The variant relaxes only that cardinality sentence. Expected mediator: additional
   validation/repair/continuation cycles after an accepted patch. This is not removal of all
   focused-validation policy; `src/agent.rs::concurrent_work_leaf_interpretation` and
   `concurrent_instruction_translation` also guide focused checks and external-blocker handling.
2. Command-result next-action guidance. `src/orchestrator.rs::run_command_for_agent` renders the
   completed command and sends its continuation. `render_command_result` includes a directive
   recommendation and brevity cue. The variant omits only that renderer-owned span. Expected
   mediator: longer explanation or additional exploratory action before the next directive.
   Command status/output, pending diffs, execution, ownership, and output limits remain unchanged.

## Further candidates requiring separate designs

3. Focused-validation policy as a whole. `src/agent.rs::PromptPolicy::for_read_permission`,
   `concurrent_work_leaf_interpretation`, and `concurrent_instruction_translation` contain
   overlapping scoped-check and stop-retrying instructions, reinforced by the patch receipt.
   A complete policy intervention must enumerate those spans without removing ownership safety
   or silently changing the required final gate. A small acknowledgment-only result cannot be
   generalized to this larger factor.
4. Cohesive patch batching. `src/agent.rs::PromptPolicy::for_read_permission` asks for tests and
   implementation together so the shared checkout stays buildable. Separately it prefers
   structured edits over manually calculated unified diffs. Batching and edit representation are
   different factors: changing format does not test batching, and permitting known-red shared
   states could change concurrency safety. A valid design must preserve that safety boundary.
5. Supplied reviewer context. `src/review.rs::render_review_source_context` supplies commit
   metadata and recorded chat history to the review launch assembled by `src/cli.rs`. The
   candidate is the cost of supplying that context versus reconstructing it, keeping exact review
   scope, target markers, routing, and review requirements. Removing review itself is not this test.
6. Compact repeated reads. `src/orchestrator.rs::split_repeated_file_reads` classifies snapshots
   by per-agent digest; `render_file_read_response_with_repeats` delivers compact refreshes.
   `--force` does not disable the digest classifier. A full-read variant needs the actual current
   snapshot at the same boundary, not reconstructed stale content or artificial padding. Expected
   mediator: more input per later response, even without additional actions.
7. Command-output compaction. `src/orchestrator.rs::render_command_output` applies
   `compact_blank_runs`, `compact_long_lines`, and `compact_total_chars`. A valid control delivers
   actual captured command output with identical execution and status. It cannot manufacture
   verbose output to force the desired direction. Expected mediator: additional retained context
   on subsequent responses, potentially offset by fewer follow-up inspections.
8. Automatic patch/commit handoff and review routing. `src/patch.rs::GitPatcher` applies and
   commits accepted edits; `src/orchestrator.rs` records ownership; `src/workspace.rs` and
   `src/cli.rs` select review targets and completion flow. This is real delegated work, not only
   wording. Removing it requires preserving truthful receipts, state transitions, ownership, and
   eventual work completion. A message pretending the host did not commit is not a valid ablation.
9. Concurrent scheduling. The normal driver starts three feature workflows together. A
   harness-only WL sequential schedule can isolate overlap while retaining WL's mechanisms,
   unlike `bench-three-features-sequential`, which is the Direct implementation. Concurrency can
   interact with scoped validation and integration blockers; one-factor effects need not add.

Existing direct-read and continued-response timing controls answer narrower questions and have
their own historical records. The normal 1,000 ms/forward interruption policy remains fixed in
this study. Waiting longer is a generation-policy intervention, not a free accounting repair.

For each admitted candidate, the evidence target is a source boundary, a delivered single-factor
intervention, its downstream behavior, and the resulting response-count/context contribution to
raw tokens. Historical cached-input and stage-total differences guide inspection but are not
substitutes for that causal chain.
