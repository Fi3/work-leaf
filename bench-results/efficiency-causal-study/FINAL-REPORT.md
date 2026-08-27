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

The normal-workflow pilot is stored in
`../efficiency-fair-normal-workflow-pilot-20260827T115642Z/PROVISIONAL-RESULT.md`. It used the exact
original task and normal product workflows, but it also produced no supported token percentage:
Work Leaf completed 2/3 requested behaviors while direct Codex completed 3/3, descendant verification
calls escaped the fixed GPT-5.5 profile, and direct resume accounting used the wrong accumulation
rule.

## Supported Findings

- The original three requests can be scored independently in saved candidate histories.
- `/status` is a concrete test of the original generic slash-command request. `/fork` is excluded.
- Workflow completion and requested-feature completion are different measurements and must both be
  reported.
- Changed-file diffs, unchanged-file digests, and direct review context remain plausible token-saving
  mechanisms in their small controlled traces.
- Percentages from those small traces have different scopes and cannot be added or allocated to a
  whole-workflow difference.
- No current evidence supports a precise normal-workflow token reduction or formal quality
  equivalence.

## Historical Names

Historical Work Leaf conditions use `wl-XYZ`. `X`, `Y`, and `Z` identify changed-file reread,
unchanged-file reread, and review-context settings. `0` means normal Work Leaf behavior and `1`
means the less compact control. For example, `wl-110` returns full files for both reread types while
still supplying review context directly. These labels describe archived artificial-protocol runs;
they are not current normal-product results.

`R19`, `step190`, and similar names are archive sequence labels, not model versions, feature counts,
or sample sizes. Raw artifacts, `evidence.json`, and `verify.py` remain available for audit. They
must not be mixed into a new normal-workflow result.

## Current Conclusion

The original headline reductions of 72.0567% raw tokens and 49.0420% uncached tokens are excluded
from the normal-workflow question. They describe one unequal-quality comparison collected under the
artificial protocol. The newer pilot's diagnostic arithmetic is also not a result until its profile
and resume-accounting defects are fixed and comparable-quality observations exist.

The required work before another paid batch is listed in `NEXT-STEPS.md`.
