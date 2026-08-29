# Endpoint Audit

## Result

The strongest existing quality-balanced comparison contains three normal direct Codex runs and
three normal concurrent Work Leaf runs. Both groups completed 8 of the 9 scored features. Work Leaf
used 60.25% fewer raw tokens and 37.58% fewer uncached tokens in this collected sample.

Every Work Leaf value is below every direct value for both measures:

| Workflow | Runs | Features | Mean raw | Mean uncached | Range, raw | Range, uncached |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Direct Codex | 3 | 8/9 | 35,196,786 | 1,739,208 | 28,877,983-41,035,124 | 1,328,068-1,982,580 |
| Work Leaf | 3 | 8/9 | 13,989,718 | 1,085,568 | 12,719,646-16,471,729 | 905,246-1,246,769 |

The exact one-sided permutation result is 1 of 20 possible group assignments, or 0.05. The
two-sided result is 0.10. This is strong descriptive evidence, but three observations per group are
too few for a precise population estimate.

## What Is Equal

`endpoint_audit.py::fairness_failures` verifies the following from every saved report and
environment record:

- base commit `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
- task SHA-256 `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`;
- GPT-5.5 with `xhigh` reasoning;
- the same final format, Clippy, and test gate;
- completed implementation, review, and linearization stages;
- no recursive provider sessions;
- direct Codex using its normal sequential workflow without Work Leaf;
- Work Leaf using its normal concurrent workflow with orchestrator-mediated reads.

The scorer tests the original three requests: visual selection and copy, `/status` forwarding, and
reviewed-patch close/reopen. `/fork` is not part of this comparison.

The observations are independent group samples. A result in one group is not paired with or removed
because of a result in the other group.

## Quality

The aggregate feature count is equal, but the feature mix is not:

| Workflow | Visual | `/status` | Close/reopen |
| --- | ---: | ---: | ---: |
| Direct Codex | 3/3 | 3/3 | 2/3 |
| Work Leaf | 2/3 | 3/3 | 3/3 |

This rules out the simple claim that Work Leaf used fewer tokens only because it completed fewer
features. It does not prove that each feature has equal implementation cost.

## Token Accounting

The authoritative values are final cumulative provider totals from the hash-verified Codex rollout
records. `bench-observer/src/lib.rs::supplement_usage_from_rollouts` verifies those records against
the captured provider threads, and `bench-observer/src/lib.rs::summarize_usage` sums one final total
per thread. All visible implementation agents, the three reviewers, the linearizer, and the hidden
title thread are included.

The Work Leaf controller counter is an audit source, not the billing authority.
`src/codex.rs::token_usage_from_params` reads the provider's per-turn `last` value and
`src/codex.rs::record_usage` adds it to the controller state. A new turn can initially repeat the
previous turn's `last` value, so summing those streamed values can overcount. This explains the
small controller/provider differences found during this audit; it does not indicate omitted
provider usage.

The three older Work Leaf captures originally failed a stricter rule that required a usage event
immediately after each interrupted directive. Their hash-verified rollout records contain later
cumulative totals for every provider thread. Reanalysis reports no unresolved provider thread and
uses no per-response token estimate.

## Hypotheses Checked

| ID | Explanation | Evidence for it | Other explanations checked | Result |
| --- | --- | --- | --- | --- |
| A | Work Leaf is undercounted | Some directives were interrupted before an immediate usage event. | Final rollout totals, hidden threads, and controller totals were inspected independently. | Rejected for these six runs. |
| B | Direct resume usage is counted twice | Direct Codex resumes the same conversation. | Saved reports use per-invocation totals; cumulative Work Leaf rules are not applied to direct runs. | Rejected for these six runs. |
| C | Direct Codex receives more required work | Direct runs contain more shell commands. | Task hash, validation freedom, final checks, review, and linearization match. Command representation differs by workflow. | No required-work mismatch found. |
| D | Work Leaf implements less | Some Work Leaf runs miss a feature. | Both admitted groups complete 8/9 features, and all outcomes are retained. | Rejected as the sole explanation. |
| E | Ordinary variation explains the gap | Both workflows vary substantially. | Every Work Leaf value is below every direct value in both token measures. | Unlikely for this sample; more runs are needed for a precise population claim. |
| L | Hidden Work Leaf threads are omitted | Work Leaf has a hidden title thread. | Provider-thread inventory contains eight threads and includes title, reviewers, and linearizer. | Rejected. |
| M | Model or reasoning drift explains it | CLI versions differ across collections. | Every admitted thread is GPT-5.5/`xhigh`; source hashes and CLI versions are recorded. | Model drift rejected; runner-version drift remains a limitation. |

## Limits

This is a historical sanity check, not a frozen current-version trial. The Work Leaf runner commits
differ across the three historical runs even though each records normal concurrent behavior. The
small sample also comes from one Rust repository and one task. These limitations prevent claiming
that 60.25% is a general or precise saving.

The result is sufficient to continue causal analysis: the observed gap is large, survives exact
provider accounting, and is not removed by balancing aggregate feature completion.

## Reproduce

```sh
python3 bench-results/efficiency-causal-validation-20260829T210343Z/test_endpoint_audit.py
python3 bench-results/efficiency-causal-validation-20260829T210343Z/endpoint_audit.py
```

The machine-readable result is `endpoint-evidence.json`. It records every source hash used by the
audit.
