# Continued-Response Control Design

## Question

Does Work Leaf save a material number of tokens by interrupting a provider response when the model
has emitted a complete `@work-leaf` directive but has started generating again?

This is narrower than disabling interruption entirely. The saved Codex app-server probe in
`../efficiency-point7-exact-accounting-20260828T113610Z/preflight/attempt-0002-wait-for-completion.jsonl`
received exact usage after a final `@work-leaf done` response but never received `turn/completed`.
Waiting for natural turn completion can therefore hang the workflow and is not a valid control.

## Evidence Before Launch

The six current normal Work Leaf traces contain 287 directive-triggered interrupts:

| State when the interrupt was released | Count | Share |
| --- | ---: | ---: |
| Provider response had completed and exact usage arrived | 252 | 87.80% |
| Provider output resumed after the directive | 34 | 11.85% |
| One-second accounting wait expired | 1 | 0.35% |

The 252 completed responses cannot have saved tokens inside their current provider response: the
response and its usage existed before the interrupt was sent. Only the 35 remaining cases can
truncate provider generation. By directive type, resumed output occurred after 22 reads, six edits,
four locked-command requests, and two `done` directives. The timeout followed a read.

This rules out the broad claim that every Work Leaf directive saves a large continuation. It leaves
a narrower claim: a small number of interrupted continuations might be individually expensive and
could still explain a material part of the mean gap.

## Isolated Change

The control uses the normal concurrent Work Leaf binary, task, prompts, read mediation, structured
edits, locked commands, validation, review, linearization, scorer, GPT-5.5/`xhigh` profile, and exact
accounting. A benchmark-observer option changes only what happens after output resumes following a
complete directive:

1. Normal evidence forwards the pending interrupt as soon as output resumes.
2. The control keeps the same interrupt pending until the current provider response emits exact
   usage, then forwards the original interrupt bytes.
3. A bounded timeout forwards the interrupt unchanged and records that the control did not activate
   fully for that turn.

Work Leaf still receives and processes only the first complete directive. Any extra provider text is
not interpreted by the orchestrator, but it remains in the provider thread history. That history
effect is part of disabling early response truncation and must not be hidden from later model turns
or token accounting.

The observer must not drop interrupts, start turns, alter prompts, execute commands, or change Work
Leaf source. Client requests issued while an interrupt is held remain queued behind it, preventing a
new turn from overtaking the still-active provider response.

## Counterchecks

### Ordinary variation

Three independent controls run concurrently and are compared as a group with all six current normal
Work Leaf observations. Their ranges and every individual outcome remain visible. Three runs can
identify a large effect; they cannot estimate a precise population percentage.

### Different implementation quality

The frozen scorer evaluates visual selection, selected-agent `/status`, and reviewed-feature
close/reopen for every candidate. Partial and failed candidates remain evidence. A token reduction
caused by completing less work is not an efficiency result.

### Token undercounting

Every provider thread must reconcile to a terminal cumulative total from a hash-verified rollout.
The delayed responses and their extra text must appear in the observer stream and cumulative total.
Any unresolved provider usage excludes only the exact token interpretation, not the workflow result.

### Transport deadlock

The observer regression test must prove that resumed output can finish, exact usage can arrive, and
the original interrupt is then forwarded. A bounded real-agent smoke must prove the same sequence on
the configured Codex app server before three full workflows launch. Failure stops the control rather
than triggering repeated paid retries.

### Read-delivery overlap

This control retains normal mediated reads. It measures the additional effect of truncating a
continuing provider response in the normal product workflow. It is not added to the direct-read
effect as if the two estimates were independent.

## Interpretation

If the control repeatedly raises raw tokens at comparable quality, early response interruption is a
cause. The mean increase divided by the current direct-versus-normal Work Leaf gap is reported only
as a descriptive fraction for this sample.

If the control overlaps normal Work Leaf, or few turns activate despite exact telemetry, early
response interruption is not supported as a major cause. The remaining explanation must be sought
in the number and accumulated size of provider cycles, especially implementation and review-fix
work; the benchmark must not be altered to manufacture a directional result.
