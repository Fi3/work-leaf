# Fairness Contract

## Question

Can normal concurrent Work Leaf complete the same three requests as normal direct sequential Codex
while using fewer GPT-5.5/xhigh tokens?

This one-pair pilot checks whether the infrastructure can produce a trustworthy observation. It
cannot establish averages, variability, or statistical confidence by itself.

## Fixed Inputs

Both workflows use:

- source base `c92a0b7060a36eac6db2d869b85e589a7a9480f9`;
- GPT-5.5 with xhigh reasoning;
- task-list SHA-256 `45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`;
- the original three feature requests, including `/status` behavior and no `/fork` requirement;
- the same two-hour limit for each feature or linearization stage;
- the same final non-mutating format, Clippy, and complete test gate; and
- the same offline tests for visual selection, `/status`, and reviewed-feature close/reopen.

Neither launcher changes the three task strings. Neither launcher limits how many format, Clippy,
test, or other validation commands an agent may run.

## Workflow Difference

Work Leaf receives all three requests together and runs its normal concurrent patch, review, and
linearization workflow through the Codex app-server interface.

Direct Codex receives the same requests one after another. Each request has a normal implementation
conversation and a separate review conversation. Review fixes resume the corresponding
conversation. A final direct conversation linearizes the reviewed history. Work Leaf is not started
or used by this path.

This concurrent Work Leaf versus direct sequential Codex distinction is the intended experimental
difference.

## Recursive Provider Isolation

The repository normally asks an implementing agent to launch another real agent as a provider smoke
test. That recursive session does not test the requested feature and complicates provider accounting.

Both temporary benchmark checkouts therefore receive the same provider-isolation instruction:

- do not launch Codex or another provider from inside an agent turn;
- do not report the waived recursive verification as a review finding; and
- continue to run all relevant repository tests and validation checks.

The run-local Codex wrapper enforces this instruction. An attempted child Codex launch is blocked,
recorded, and makes the benchmark workflow fail. A successful pilot must contain an empty
`recursive-codex-attempts.log` for both workflows.

This exception affects only recursive provider smoke sessions. It does not remove implementation,
review, correction, linearization, Cargo validation, the final gate, or offline quality scoring.

## Model Profile

The run-local wrapper pins both `model="gpt-5.5"` and `model_reasoning_effort="xhigh"` on every
allowed Codex process. The launchers do not read or edit the user's `.codex/config.toml`. The observer
checks the recorded rollout profile for every primary implementation, review, correction, and
linearization conversation.

## Token Accounting

Work Leaf app-server updates are cumulative conversation totals. The observer keeps the final
cumulative value for each Work Leaf conversation.

Direct `codex exec` and `codex exec resume` commands report fresh totals for each CLI invocation. The
observer keeps one terminal value per invocation and adds every implementation, review, correction,
and linearization invocation. It independently checks that sum against the final value from every
`task_started`/`task_complete` epoch in the corresponding Codex rollout file.

The comparison reports:

- raw tokens: input plus output; and
- uncached tokens: input minus cached input, plus output.

Because recursive provider calls are prohibited, `primary_condition` and `total_workflow` should
contain the same provider conversations. Any difference must be explained before a token comparison
is accepted.

## Outcomes And Stop Rule

Every implementation is retained and scored, including partial implementations and workflow
failures. A quality difference is evidence, not a reason to delete or rerun a result. Token savings
are not presented as an equal-output comparison unless the saved implementations have comparable
feature completion.

The pilot launches one workflow per condition without retry. It stops after offline scoring. Larger
normal-workflow collection and mechanism allocation require explicit user approval.
