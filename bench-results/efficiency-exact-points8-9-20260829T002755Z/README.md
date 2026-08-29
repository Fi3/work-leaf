# Exact Points 8 And 9 Study

## Goal

This study measures whether normal concurrent Work Leaf uses fewer tokens than fair normal direct
sequential Codex on the frozen three-feature task. It also checks whether three previously proposed
message-delivery mechanisms explain any difference.

The three Work Leaf groups are independent:

- `direct`: normal direct sequential Codex without Work Leaf;
- `wl-000`: normal concurrent Work Leaf; and
- `wl-111`: concurrent Work Leaf with changed-file diffs, unchanged-file digests, and inline exact
  review context replaced by their less compact study controls.

The `wl-111` review control reconstructs review context from Git. Earlier evidence found that this
control can disrupt review routing, so its implementation quality and workflow behavior must be
reported with its token result. It cannot be treated as a valid causal estimate when it changes the
workflow.

## Collection

`SCHEDULE.tsv` predeclares three runs per group. Runs are independent observations rather than
statistical pairs. At most two top-level workflows run simultaneously. Every success, partial
implementation, workflow failure, and measurement failure remains evidence.

The five batches are inspected in order. Collection stops before the next batch when the same
infrastructure problem makes two consecutive attempts unusable or when the frozen task, model,
reasoning level, workflow, scorer, or accounting route cannot be preserved.

## Fairness

Every run uses:

- candidate base `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
- frozen benchmark source `4707ceb4903a09646857d1e316cb45acb15a3d07`;
- the original generic selected-agent slash-command task, tested concretely with `/status` and no
  `/fork` requirement;
- GPT-5.5 with `xhigh` reasoning for every provider call;
- normal agent validation, review, linearization, time limits, and final repository checks;
- no recursive provider-verification sessions; and
- the frozen visual, `/status`, and completion quality checks.

Direct Codex uses `bench-three-features-sequential`. Normal and controlled Work Leaf use
`bench-three-features` with the concurrent feature schedule. The endpoint comparison changes only
the workflow. The `wl-111` comparison changes only the three declared study controls in an isolated
frozen Work Leaf build. Production Work Leaf, the task, and the evaluator are not modified.

## Exact Accounting

Both workflows run through
`../efficiency-exact-cancelled-usage-20260828T221936Z/infrastructure/run_with_exact_usage.py`.
That launcher routes every Codex invocation through one run-local Responses API proxy and pins
GPT-5.5/xhigh. The proxy stores provider responses, retrieves final usage after normal completion or
Work Leaf interruption, and rejects missing or duplicate response records.

The exact provider records are the token authority. Existing observer output remains useful for
commands, stages, and workflow activity, but its Work Leaf token total is incomplete after
interruptions. These runs use the OpenAI API route and are not merged numerically with the older
ChatGPT Codex runs.

## Interpretation

Point 8 is supported when direct and normal Work Leaf have comparable average feature completion
and exact repeated token totals show a stable difference.

Point 9 first asks whether `wl-111` moves token use toward direct Codex while retaining comparable
quality and workflow behavior. If normal and controlled Work Leaf remain similar, the three delivery
mechanisms do not explain the observed difference. The next causal control then targets the strongest
remaining hypothesis: Work Leaf performs fewer repeated command and validation cycles.
