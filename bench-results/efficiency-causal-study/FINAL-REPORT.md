# Efficiency Study Status

## Abstract

This directory records an investigation into why concurrent Work Leaf appeared to use fewer model
tokens than direct sequential Codex on a three-feature Rust task. The historical collection found
several plausible mechanisms, including compact repeated-file responses and direct review context.
It does not establish a normal-product token-saving percentage.

The historical full-workflow runs used an artificial validation rule that limited feature agents to
one focused Cargo command. Normal Work Leaf usage has no such restriction. Some runs also used a
later slash-command request containing `/fork`, although the original task required only generic
slash-command forwarding and was scored with `/status`. Those runs answer questions about the
artificial protocol, not normal Work Leaf versus normal direct Codex.

Later normal-workflow runs corrected the task, model profile, validation policy, and direct resume
accounting. The final exact-accounting gate is stored in
`../efficiency-point7-exact-accounting-20260828T113610Z/FINAL-RESULT.md`. It found a remaining
transport limit: normal Work Leaf interrupts model responses at orchestrator directives, but the
ChatGPT Codex transport exposes usage only for completed responses. Earlier Work Leaf totals
therefore omitted interrupted-response tokens.

Point 7 is completed by a conservative bound in
`../efficiency-point7-bounded-accounting-20260828T142614Z/FINAL-RESULT.md`. It charges every omitted
response 400,000 raw tokens. Normal Work Leaf remains at least 19.15% below exact direct Codex, and
Work Leaf with all three tested mechanisms disabled remains at least 4.73% below. All selected
candidates passed 3/3.

Points 8 and 9 are completed in `../efficiency-points8-9-20260828T145556Z/FINAL-RESULT.md`. Three
direct and three normal Work Leaf observations have the same average feature count, 2.67 of 3.
Completed-response data show Work Leaf 60.25% lower, but a conservative allowance for every missing
interrupted response changes the possible difference to a range from 21.21 million fewer tokens to
3.33 million more. The repeated average saving is therefore not proven with current telemetry.

The changed-file and unchanged-file controls activated but did not produce a directional
whole-workflow result. Git review reconstruction repeatedly broke review routing, so review context
could not be estimated. Fewer command and validation cycles are the strongest remaining
explanation for the observed signal, but their exact token fraction was not isolated.

## Supported Findings

- The original three requests can be scored independently in saved candidate histories.
- `/status` is a concrete test of the original generic slash-command request. `/fork` is excluded.
- Workflow completion and requested-feature completion are different measurements and must both be
  reported.
- Changed-file diffs and unchanged-file digests save bytes in measured delivery events, but their
  conservative whole-workflow token ranges cross zero.
- Git review reconstruction is not behaviorally equivalent to normal inline review context in this
  workflow, so it cannot support a token-effect estimate.
- Normal Work Leaf averages far fewer total commands, repeated commands, and validation commands
  than direct Codex. This is the strongest observed explanation, but it is not an isolated causal
  result.
- Direct sequential Codex can be measured exactly by adding every completed launch and resume
  invocation.
- Normal Work Leaf cannot be measured exactly on the current ChatGPT Codex transport because usage
  for interrupted directive responses is unavailable.
- A conservative upper bound proves a raw-token saving in the selected fair Point 7 observations.
- Across three observations per endpoint, the average raw-token difference is inconclusive under
  the same conservative accounting.
- No current evidence supports a precise average reduction, an uncached reduction, an exact causal
  allocation, or formal population-level quality equivalence.

## Historical Names

Work Leaf conditions use `wl-XYZ`. `X`, `Y`, and `Z` identify changed-file reread, unchanged-file
reread, and review-context settings. `0` means normal Work Leaf behavior and `1` means the less
compact control. For example, `wl-110` returns full files for both reread types while still
supplying review context directly. Both the historical artificial-protocol archive and the current
normal-workflow study use these labels; results must be identified by their study directory and
must not be mixed.

`R19`, `step190`, and similar names are archive sequence labels, not model versions, feature counts,
or sample sizes. Raw artifacts, `evidence.json`, and `verify.py` remain available for audit. They
must not be mixed into a new normal-workflow result.

## Current Conclusion

The original headline reductions of 72.0567% raw tokens and 49.0420% uncached tokens are excluded
from the normal-workflow question. They describe one unequal-quality comparison collected under the
artificial protocol. The later 49.039% raw and 26.256% uncached Point 7 reductions and subsequent
attribution percentages are also withdrawn because their Work Leaf totals omit interrupted-response
usage.

Point 7 proves a raw-token saving for one selected 3/3 observation per endpoint after charging every
missing Work Leaf response the conservative maximum. That result remains valid for those selected
observations only.

The Points 8 and 9 study expands direct and normal Work Leaf to three observations each. Both groups
average 2.67 completed features. Direct averages 35,196,786 exact raw tokens. Work Leaf averages
13,989,718 observed raw tokens and has a conservative upper bound of 38,523,051. The possible mean
difference ranges from Work Leaf using 21,207,069 fewer tokens to 3,326,265 more. The repeated
average saving is therefore inconclusive under conservative accounting, even though every observed
completed-response total is substantially lower for Work Leaf.

Changed-file diff and unchanged-file digest controls do not establish a whole-workflow direction;
their conservative ranges cross zero. Review-context attribution is unavailable because the Git
control repeatedly changed workflow behavior. Work Leaf averages 57.79% fewer commands, 93.05%
fewer repeated commands, and 50.63% fewer validation commands. Fewer workflow cycles are the
best-supported explanation for a real saving, but no controlled cycle ablation assigns an exact
fraction.

The exact raw percentage, uncached percentage, population average, and causal allocation remain
unknown. `NEXT-STEPS.md` lists the evidence needed to resolve those limits.
