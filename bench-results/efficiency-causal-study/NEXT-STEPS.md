# Next Steps For Exact Efficiency Attribution

## Goal

Turn the strong observed Work Leaf token signal into an exact average result, then isolate how much
of any real saving comes from fewer workflow cycles. Preserve normal direct Codex and normal Work
Leaf as the endpoint workflows; causal controls must be reported separately.

## Current Evidence

The completed normal-workflow study is
`../efficiency-points8-9-20260828T145556Z/FINAL-RESULT.md`.

- Three direct and three normal Work Leaf observations each average 2.67 completed features.
- Direct averages 35,196,786 exact raw tokens.
- Work Leaf averages 13,989,718 observed raw tokens, 60.25% below direct, but interrupted responses
  make that value incomplete.
- The conservative Work Leaf upper bound averages 38,523,051 raw tokens. The defensible difference
  ranges from 21,207,069 fewer tokens to 3,326,265 more.
- Changed-file and unchanged-file delivery controls activated, but both whole-workflow effect ranges
  cross zero.
- Git review reconstruction repeatedly broke review routing, so it is not a valid review-context
  token control.
- Work Leaf averages 57.79% fewer commands, 93.05% fewer repeated commands, and 50.63% fewer
  validation commands. This is the strongest remaining explanation, but it is not an isolated
  causal fraction.

The selected Point 7 observations still prove a bounded saving for those runs. They do not prove
the repeated average.

## 1. Obtain Complete Work Leaf Usage

The current ChatGPT Codex transport reports usage only when a response completes. Normal Work Leaf
interrupts a response after receiving a complete orchestrator directive, so exact usage for that
response is absent. The conservative allowance is wider than the observed group difference.

The next exact study requires one of these without changing normal Work Leaf behavior:

1. provider-side usage records that include cancelled responses;
2. cumulative thread usage available after cancellation; or
3. transport telemetry that reports final usage on the cancellation path.

Do not make Work Leaf wait for unnecessary model text merely to obtain usage. That would change the
workflow being measured.

When complete telemetry exists, rescore the saved runs if the provider records can be linked to
their recorded thread and turn identities. Otherwise collect a small fresh batch with the same
frozen task, base, model, reasoning level, scorer, and independent-group rules.

### Available API Route

`../efficiency-exact-cancelled-usage-20260828T221936Z/README.md` defines a separate API-key route
that has passed automated checks and a real Work Leaf interruption. It stores each GPT-5.5/xhigh
response, closes the provider stream when Codex disconnects, requests cancellation, and retrieves
exact usage. The final pilot has one started response, one exact final record, and no missing usage.

This route cannot recover the old ChatGPT Codex runs. New direct and Work Leaf observations must
both use it, and their result must be labeled as an OpenAI API comparison. The existing benchmark
observer can still capture commands and workflow activity, but the exact provider records from the
new study are the token authority because the app-server report still lacks terminal usage for the
interrupted turn.

## 2. Isolate Workflow Cycles

After accounting is exact, test the most supported cause: fewer command, repetition, and validation
cycles. Predeclare a control that changes only cycle policy in an isolated benchmark build. Keep the
normal direct and normal Work Leaf endpoint workflows unchanged.

The control must record:

- provider turns and threads;
- total commands;
- repeated commands;
- validation commands;
- prompt and command-output bytes;
- exact raw and uncached tokens; and
- all three feature outcomes.

The causal claim is supported only if the control changes cycle counts in the intended direction,
quality remains comparable, and exact token use moves with the cycle change across repeated
independent observations.

## 3. Add Formal Precision Only If Needed

More runs improve confidence in the overall direct-versus-Work-Leaf average; they do not fix missing
usage. Run them only after complete accounting exists.

Use independent groups rather than fixed statistical pairs. Predeclare a practical precision target
and begin with a small batch so infrastructure or quality problems are visible before committing to
long collection. Keep every success, partial implementation, workflow failure, and missing result.

## Optional Review Follow-Up

The Git-reconstruction control is not behaviorally equivalent to normal inline review. A future
review-context study needs a new control that changes only the amount or form of review context
without changing routing or completion behavior. It must pass a real-agent pilot before paid
replication. This is optional and separate from proving the overall saving.

## Do Not Reuse

- Do not use the historical artificial-validation percentages as normal-product results.
- Do not report completed-response Work Leaf usage as an exact total.
- Do not add percentages from different scopes to allocate the whole-workflow gap.
- Do not discard one workflow because another workflow in the same launch batch fails.
- Do not rerun a saved candidate because an offline report formatter fails after fixture results
  were written.
