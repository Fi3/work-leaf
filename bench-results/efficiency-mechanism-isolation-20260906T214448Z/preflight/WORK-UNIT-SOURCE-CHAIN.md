# Work-unit policy: source chain and measurable mechanism

Scope: the v2 `control` versus `buildable-work-unit-incremental` phase in
`PROTOCOL-WORK-UNITS.md`. Runtime references identify commit
`2a69f863a294c224d137120ea0a21834cc88210d`; line numbers refer to that source.
This is a provider-free source audit, not an observation, admission, or causal result.

## Execution chain

1. **Policy reaches the existing launch boundary.**
   `src/agent.rs:300` constructs A's cohesive-work-unit sentence;
   `concurrent_instruction_translation` at `src/agent.rs:412` emits B only for each
   already-detected test instruction. `PromptPolicy::render` at `src/agent.rs:353`
   retains original repository instructions and user text, and excludes linearizers
   from A/B. `inject_for_delivery` at `src/agent.rs:339` delegates only feature-enabled
   delivery to `bench_experiment::forward_policy` (`src/bench_experiment.rs:205`).
   `render_spans` at `src/bench_experiment.rs:218` transforms only owned A/B/C spans.
   Public `inject` remains baseline; default builds compile out the experiment.
   Codex's existing launch sites are `src/codex.rs:1544` and `src/codex.rs:1582`;
   missing-session send sites are `src/codex.rs:1638` and `src/codex.rs:1705`.
   Claude has the same four delivery sites at `src/claude.rs:407`, `421`, `459`, `494`.

2. **A streamed response becomes orchestrator work.**
   `DirectiveStreamInterruptDetector::observe` (`src/orchestrator.rs:637`) detects
   complete directives in agent messages. `CodexAppServer::request_turn_streaming`
   handles completed message items and returns accumulated streamed text after the
   ordinary interrupt request (`src/codex.rs:276`).
   `CommandChat::process_agent_reply_streaming_result` (`src/cli.rs:1035`) queues
   returned replies, calls `handle_agent_directives_streaming` (`src/orchestrator.rs:685`),
   and queues nonempty follow-up replies (`src/cli.rs:1140`). This is ordinary control
   flow in both arms; the intervention does not change interruption or response waits.

3. **Successful edits aggregate before the ACK.**
   `handle_agent_directives_streaming` creates one `applied_patch_files` set per
   processed reply (`src/orchestrator.rs:717`). Successful `GitPatcher::apply` and
   `apply_edit` results extend that set and emit `PatchApplied` events at
   `src/orchestrator.rs:794` and `866`. The patcher owns locks and provisional commits
   (`src/patch.rs:64`, `77`, `150`); unchanged structured edits return the distinct
   already-applied conflict (`src/patch.rs:173`). After processing the reply, a nonempty
   set and `!run.completed` produce **one** successful-patch ACK (`src/orchestrator.rs:982`).
   Several successful directives can therefore share an ACK. A file, directive,
   provisional commit, ACK, provider turn, and provider response are not interchangeable
   counts. A reply that already reaches accepted `done` can bypass this ACK branch.

4. **The ACK changes remaining-work guidance, not validation cardinality.**
   `render_patch_applied_prompt` (`src/orchestrator.rs:2676`) keeps the provisional-commit
   receipt, at-most-one focused check (`2682`), locks and ownership guards. Only C's
   completion/remaining-work paragraph (`2692`) varies through
   `ContinuationPrompt::finish` (`src/orchestrator.rs:2396`). The existing sender
   (`src/orchestrator.rs:657`) starts the next interruptible interaction. Additional
   submissions may lead to additional focused checks, reads, or repairs; no extra
   submission or check is enforced by code. The distinct `work-leaf patch already applied`
   renderer (`src/orchestrator.rs:2666`) is not this successful ACK.

5. **A known agent continues its existing provider thread.**
   `CodexBackend::send_streaming_interruptible` (`src/codex.rs:1677`) retrieves the
   existing session/thread, forwards only the new prompt when that session exists
   (`1701`), and passes its thread ID to `request_turn_streaming` (`1715`). That method
   retains the ID or loads it with `thread/resume` if necessary (`src/codex.rs:195`,
   `492`), then sends `turn/start`. `turn_start_params` (`src/codex.rs:1122`) contains
   that thread ID and the new text, not a rebuilt launch-policy transcript. Local
   message recording (`src/codex.rs:1426`) is bookkeeping, not proof of provider billing.

6. **Recorded response attribution can establish repeated input charges.**
   The supplemental `audit_input_attribution.py::audit_run` (`:292`) verifies raw-capture
   provenance and matched native scope. `extract_records` (`:105`) joins exact
   response/thread/turn identities and four usage fields; it sums top-level attribution
   items plus `request_fields` without summing nested copies (`:168`). Its
   `(thread_id, item_id)` index records charges across completed responses (`:207`).
   A linked native item supplies safe metadata; an unmatched identity stays unlinked.
   Same-session execution alone does **not** prove which earlier items remain in later
   model input, their tokenization, or their charge. Only recorded attribution establishes
   those facts for covered responses; compaction, pruning and missing metadata limit coverage.

## Direct prompt-size component

Exact ASCII replacements are defined at `src/bench_experiment.rs:20`; `render_spans`
records UTF-8 byte arithmetic at `:251`, and `forward_v2` records full-prompt lengths
once per boundary at `:282`.

| Owned span | Treatment minus control bytes |
| --- | ---: |
| A: `policy-buildable-work-unit` | +184 per non-linearizer injection |
| B: `instruction-tests-work-unit` | +122 per existing test translation |
| C: `patch-applied-remaining-work` | +219 per successful-ACK prompt |

An injection with `k` B copies differs by `184 + 122k` bytes. Total first-delivery
text delta is `184 × A exposures + 122 × B exposures + 219 × C exposures`.
Use delivery-linked exposures; a written trace alone does not prove acceptance.
These are bytes, not token prices. Their later retained-input charges and behavioral
consequences are separate measurements. Raw response tokens are input plus output;
cached input is a subset of input, not an additional quantity or a deduction
(`audit_input_attribution.py:70`). No padding or approximate bytes-to-tokens subtraction
identifies a causal direct-prompt cost.

## What must come from outcomes

The source establishes a concrete possible path:
work-unit guidance → selected submissions → successful-ACK continuations → same-thread
turns → observed repeated input charges. It does not establish that models choose more
increments, that those increments are substantive, or that total tokens increase.

`analyze_work_units.py::prompt_inventory` (`:222`) verifies owned spans and bidirectional
trace/request coverage. It counts successful ACKs only after an accepted typed request
(`:322`), not after a trace write, rejection, or already-applied receipt. Its explicit
output does not claim an exhaustive model-call inventory (`:334`). The frozen randomized
primary tests whether the complete policy changes delivered ACK counts; the fixed-sequence
substantive-work outcome distinguishes useful increments from other accepted handoffs.
Stage/agent/response attribution must then connect the observed continuations to actual
charges. A positive result identifies an effect of this complete policy under the mixed-wave
assignment regime, not separate effects of A/B/C, strict tests-first timing, an assumption
that every ACK creates one model response, or a quantified explanation of the entire historical
approximately 50% WL-versus-direct reduction. Missing interrupted-response counters stay
explicitly bounded/unknown; they are not inferred from assistant-item counts or later input cost.

Attribution helper SHA256:
`ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740`.
No runtime behavior or executable helper is affected by this document, so it requires no
additional real-agent generation.
