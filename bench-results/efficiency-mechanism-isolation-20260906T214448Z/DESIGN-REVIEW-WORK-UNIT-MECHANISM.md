# Independent work-unit design review

Scope: source and prospective-design review of `DESIGN-BUILDABLE-WORK-UNIT.md` and
`PROTOCOL-WORK-UNITS.md`, informed by completed historical and screen-01 traces. This note admits
no workflows, chooses no new allocation, and changes no runtime or frozen analyzer. Suggested
protocol decisions below require resolution before admission; they are not implicit amendments.

## Source isolation

The treatment is grounded in three actual renderer-owned spans:

- `src/agent.rs::PromptPolicy::for_read_permission`: cohesive test/implementation submission.
- `src/agent.rs::concurrent_instruction_translation`, existing `topics.tests` branch: the second
  copy of test/buildability timing, emitted once per selected instruction file.
- `src/orchestrator.rs::render_patch_applied_prompt`: readiness after validation versus permission
  to implement remaining requested work.

This is one complete work-unit/remaining-work policy factor, not separately identifiable cohesion,
stopping, or strict RED-before-implementation effects. The original no-known-red, ownership, read,
structured-write, focused-validation cardinality, and command-result policies remain fixed.
The documented reviewer/hidden-agent injection scope is accurate; implementation-only attribution
cannot silently disregard those exposures. Four existing injection calls in each configured
Codex/Claude provider match the delivery plan. The public baseline renderer and existing-session
raw followups need not change behavior.

`src/orchestrator.rs::handle_agent_directives_streaming` aggregates successful edit paths and sends
one ACK after processing the directives. Therefore commits, files, assistant items, and full
ACK-to-done episodes are all unsuitable substitutes for delivered ACK count. Several natural
increments can occur before one `done` under the proposed condition.

## Primary count: exact admission/delivery rules

The automatic all-successful-ACK outcome in the proposed protocol is a defensible primary mediator.
Use one unique actual accepted `turn/start` carrying the exact successful-ACK prompt, with a typed
thread/turn identity and a successful `PatchApplied` group. Deduplicate capture/RPC and actual turn
identities, not prompt text. Two distinct accepted deliveries of identical text count twice.

A trace row written before a failed send is not delivery. Neither a rejected request, the distinct
`work-leaf patch already applied` reply, a queued-but-never-sent ACK, nor duplicate notifications add
to the count. If an accepted no-op or replay emits a genuine `work-leaf patch applied` ACK, that
actual ACK does count in the primary; its substantive label remains no-op/replayed unless evidence
establishes new remaining work.
Conversely, an accepted ACK counts even if its subsequent generation or workflow later fails.
Missing usage for that generation does not invalidate an otherwise verified delivery. Retain every
admitted workflow; an unverifiable inventory is unknown rather than zero or an exclusion.

Reconcile both directions against actual captured requests and successful patch events. This is a
finite protocol-event inventory, not a claim to count hidden model retries or provider responses.

## A bounded causal analysis that does not require tighter token ceilings

The fresh fixed twelve-workflow design, six per condition, two independently randomized mixed-wave
blocks, has 18 × 18 = 324 permitted allocations. Use workflows as units, not their three feature
agents or individual ACKs. The frozen one-sided mean-count contrast and alpha 0.025 can test the
policy's effect on delivered handoffs. A null result is inconclusive about smaller effects; it does
not authorize extension or replacement.

Primary significance establishes an effect on all handoffs, including repair handoffs. To support
the more specific remaining-work mechanism, resolve one of these interpretation rules before launch:

1. Keep the single automatic primary test and explicitly describe substantive-work classifications
   and exact retained-item chains as mechanistic corroboration, not a separately established causal
   effect on the substantive subset.
2. Preferably, preregister a fixed-sequence secondary exact test of genuinely-remaining-work ACK count
   at the same 0.025 threshold, performed only after the automatic primary passes. This retains the
   phase's 0.025 family-wise allocation. Freeze the classification rule before outcomes: inspect
   actual patches and prior states, separate format/validation/review repair, retain mixed or unknown
   cases, and use conservative finite count intervals for unresolved labels. Do not filter runs or
   claim significance from only resolved/favorable classifications. Independent condition-blind
   adjudication where practical protects the semantic outcome; patch titles alone are insufficient.

The present qualitative rule that a result cannot be caused *only* by repairs is weaker: a large
repair difference plus one genuine increment example would satisfy it without establishing a causal
increase in genuine increments. The claim must match the chosen rule.

Report all distinct actual provider continuation turns separately from exact completed response IDs.
For each newly accepted handoff, follow the next action, any locked-command/read result, subsequent
edit or `done`, and exact retained-item charges. Extra handoff continuations can be counterbalanced
by fewer inspection, repair, review, or other responses elsewhere; no local chain establishes a net
whole-workflow response or token increase. Missing output tails do not prevent the primary event-count
test, but they do prevent promoting captured-response input sums into exhaustive token totals.

The screen-01 user-2 chain already demonstrates the accounting step exactly: an earlier `sed`
result, `fco_01a078d2-189b-7252-a3ec-4e451f275b65`, is charged 8,896 input tokens in each of five
ACK/check continuations. That is evidence of retained content being charged again, not an estimate
of the cue's causal share. The new randomized test supplies the policy-to-handoff evidence; exact
source/response/item chains supply the accounting explanation. Neither component alone identifies
a natural indirect effect or the percentage of historical approximately 50% savings.

## Interference and scope

Shared CPU/I/O/subscription capacity can make one workflow's assigned policy affect its wave peers.
The constrained Fisher test can test the sharp null of no assignment effects, but interpreting the
mean difference as an isolated individual-workflow direct effect additionally requires assumptions
about interference. Report the observed causal contrast in the frozen mixed-wave allocation regime;
do not claim a direct-versus-spillover decomposition. Independent checkouts do not settle this issue.

## Recommended progression

Complete and freeze the exact v2 implementation, delivery checks, real subscription smoke, inventory,
allocation, and prospective interpretation rule. Run the twelve admitted workflows in the specified
three-workflow waves, preserve every outcome, then pause. No additional measurement subsystem is
needed to test the primary mediator. Whole-workflow raw-token bounds remain a secondary report.

If the policy produces no verified difference in meaningful packaging, it has not demonstrated this
mediator in these tasks. Do not make fragmentation mandatory after looking at the data. A subsequent
separately admitted fixed-work continuation experiment is an optional distinct question, not a rescue
run or a substitute for this phase's result. No microexperiment is needed before this viable test.
