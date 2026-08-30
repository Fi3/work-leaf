# Causal Validation Protocol

## Current Accounting Constraint

The normal Work Leaf endpoint is bounded, not exact. Thirty-five interrupted responses lack
provable terminal usage.
Any protocol step below that describes the normal endpoint as exact is superseded by
`ACCOUNTING-STATUS.md`. The direct-read, continued-response, and combined controls retain exact
usage and remain valid.

## Goal

Answer two separate questions for the frozen three-feature Rust benchmark:

1. Is the lower token use a real difference between normal direct sequential Codex and concurrent
   Work Leaf, rather than missing accounting, unequal tasks, or ordinary run variation?
2. Which observable part of the workflow produces the difference, and how much of the collected
   sample can be assigned to it without inventing precision?

The endpoint workflows remain unchanged. Direct Codex does not use Work Leaf. Work Leaf uses its
normal concurrent implementation. The task, base commit, GPT-5.5 model, `xhigh` reasoning, normal
validation freedom, final repository gate, and three-feature scorer stay fixed. `/status` tests the
generic slash-command request; `/fork` is excluded.

## Evidence Rules

- Groups are independent. A failure in one group never removes another observation.
- Every success, partial implementation, workflow failure, and infrastructure failure is retained
  under its own identity.
- Raw and uncached tokens are reported separately. Cached input is never treated as free.
- A token total is exact only when every provider thread has a final cumulative total or each direct
  invocation has terminal usage. Estimates do not enter exact comparisons.
- Workflow completion, feature quality, and token completeness are separate fields.
- Existing evidence is admitted only after its task, base, model, reasoning, validation policy,
  recursive-provider policy, accounting, and scorer are verified from saved artifacts.
- Historical artificial-validation runs and Git-reconstructed review controls are never used as
  normal-product or causal results.
- Concurrent launches are a scheduling optimization, not statistical pairs.

## Hypothesis Challenge

Each hypothesis must pass three checks before an action is based on it:

1. Positive evidence: identify the exact saved measurement or code path that supports it.
2. Alternatives: identify at least two other explanations that could produce the same observation.
3. Falsifier: state what saved evidence or controlled result would make the hypothesis unlikely.

An observation is not called a cause merely because it correlates with tokens. In particular,
command counts are not directly comparable when direct Codex reads through shell commands and Work
Leaf reads through orchestrator directives.

## Hypotheses

| ID | Candidate explanation | Initial status | Required check |
| --- | --- | --- | --- |
| A | Work Leaf usage is undercounted | serious alternative | Reconcile every provider thread and reanalyze uninstrumented captures through later cumulative totals. |
| B | Direct Codex usage is overcounted on resume | serious alternative | Reconcile each invocation with rollout epochs and reject cumulative/per-invocation mixing. |
| C | The benchmark gives direct Codex more work | serious alternative | Compare exact task bytes, validation freedom, model profile, time limits, review duties, and final gates. |
| D | Work Leaf saves tokens by implementing fewer features | serious alternative | Keep all scores, compare feature distributions, inspect full-feature subsets, and model quality explicitly. |
| E | The difference is ordinary run variation | plausible alternative | Use all admissible independent observations, report ranges and uncertainty, and avoid matched-pair logic. |
| F | The raw result is mostly a cache-accounting effect | highly likely | Split the exact gap into cached input, uncached input, and output tokens. |
| G | Work Leaf performs fewer provider generation cycles | likely | Count distinct cumulative usage changes from hash-verified rollout files, not shell-command events. |
| H | Work Leaf carries a smaller context through each generation cycle | likely | Divide exact input usage by distinct cumulative usage changes and inspect stage-level totals. |
| I | Compact rereads, command results, bundles, edit acknowledgements, review targets, or linearization targets directly explain the gap | possible but limited | Compute verified byte counterfactuals and run a control only when the mechanism activates without changing routing or validation. |
| J | Immediate directive interruption explains the gap | possible | Compare counted post-directive work and, only if needed, use an isolated no-interrupt control with identical prompts and validation. |
| K | Parallel execution itself changes token use | weak | Check timing/cache contention evidence; do not use sequential Work Leaf because it is not a relevant product endpoint. |
| L | Hidden title, reviewer, or linearizer threads are omitted | serious accounting alternative | Inventory every observed thread and include hidden workflow threads in totals. |
| M | Provider or CLI drift explains the difference | plausible alternative | Admit only GPT-5.5/`xhigh` rows and record CLI versions; CLI version may differ only when accounting semantics reconcile. |

## Execution Order

1. Reanalyze the three earlier fair, uninstrumented Work Leaf endpoint captures with cumulative
   accounting. This is the strongest available check because those three Work Leaf outputs and the
   three direct outputs have equal average feature completion.
2. Build a durable offline decomposition of all admitted exact endpoints by token class, stage,
   distinct provider-usage changes, context per change, tool activity, and feature quality.
3. Audit every proposed control against production call paths. Reject controls that alter task
   wording, validation freedom, review routing, feature schedule, or scorer behavior.
4. If offline evidence leaves a material ambiguity, run one small causal batch of three concurrent
   workflows. All three must use the same isolated control and unique identities. Analyze them
   before authorizing another batch.
5. Continue only when the control activated, accounting is exact, workflow behavior stayed valid,
   and quality did not collapse. Otherwise stop and report the failed hypothesis.

## Planned Outputs

- `01-ENDPOINT-AUDIT.md`: bounded endpoint result and fairness audit.
- `02-MECHANISM-DECOMPOSITION.md`: explanation of why the endpoint decomposition is not exact.
- `03-CONTROL-DESIGN.md`: accepted and rejected causal controls with code paths.
- `04-PILOT-RESULT.md`: first paid causal batch or the reason no paid run is justified.
- `FINAL-REPORT.md`: plain-language answer, uncertainty, rejected explanations, and remaining work.

## Stop Conditions

Stop paid collection and preserve state when the same infrastructure problem repeats, a control
does not activate, exact accounting is unavailable, the control changes normal validation or review
routing, or quality falls enough that token movement cannot be interpreted. Diagnose the failure
before any retry.
