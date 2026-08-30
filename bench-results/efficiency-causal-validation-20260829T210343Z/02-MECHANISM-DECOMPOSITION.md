# Mechanism Decomposition

## Status

An exact cycle/context decomposition of the six normal Work Leaf runs is not available. Ten
interrupted responses lack terminal usage. Those responses may contain both tokens and provider
generations that do not appear in the saved cumulative events, so dividing the recorded lower bound
by recorded generation counts cannot produce an exact allocation.

`decompose.py` checks the corrected endpoint before reading rollouts. When unresolved responses are
present, it returns `superseded_by_bounded_endpoint_analysis` and does not publish the former cohort
factorization. `decomposition-evidence.json` contains that status.

## What Remains Useful

The saved lower-bound histories show fewer recorded generations and less recorded input per
generation for Work Leaf. These observations helped choose the later causal control, but they are
descriptive and are not used as the final proof.

The later exact compact-direct versus sequential Work Leaf control directly tests the workflow
protocol without interrupted responses. It shows that Work Leaf's orchestration package reduces
model generations from 311 to 198 and raw tokens from 35.66 million to 19.31 million on average.
The bounded endpoint allocation is in
`bench-results/efficiency-mechanism-attribution-20260830T081131Z/FINAL-REPORT.md`.
