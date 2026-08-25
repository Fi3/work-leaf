# Final efficiency report

## Supported result

The supported comparison is normal sequential direct Codex against normal concurrent Work Leaf.
Both use GPT-5.5 with `xhigh` reasoning on the fixed three-feature task. Other model profiles do not
enter the comparison.

The frozen original-task scorer retains every saved normal-product candidate. Sequential direct
Codex has six rows and a mean quality score of **2.00/3**. Concurrent Work Leaf has four rows and a
mean of **2.25/3**. `/status` succeeds in **4/6** sequential direct Codex rows and **4/4** concurrent
Work Leaf rows.

Four rows form same-block pairs. Their Work Leaf-minus-sequential differences are **-1, +2, +1,
and -1** in frozen order. The paired condition means are also 2.00/3 and 2.25/3. Two unmatched
sequential rows remain reliability evidence.

Three historical Work Leaf sanity rows score **3, 2, and 2**. `/status` succeeds in all three. They
used a sequential Work Leaf schedule. They are not comparison rows and do not define a quality
floor.

The exact R19 attempt-2 pair has a narrower accounting scope. Title-agent usage is excluded.

| Condition | Raw tokens | Uncached tokens | Original-task quality |
| --- | ---: | ---: | ---: |
| Sequential direct Codex | 43,009,498 | 2,105,178 | 3/3 |
| Concurrent Work Leaf | 12,018,293 | 1,072,757 | 2/3 |

For that pair, concurrent Work Leaf reduces raw tokens by **72.0567%** and uncached tokens by
**49.0420%**. This is exact accounting for one unequal-quality pair. The broader quality cohort is
needed to describe average task completion.

## Mechanism evidence

A controlled test holds a small fixed task constant and changes one declared mechanism. Three such
tests support token savings.

| Normal Work Leaf mechanism | Measured scope | Raw reduction | Uncached reduction |
| --- | --- | ---: | ---: |
| Changed rereads return a diff | Treated turn only | 14.8824% | 85.0587% |
| Unchanged rereads return a digest | Treated turn only | 15.2067% | 87.2081% |
| Reviews receive exact provenance inline | Complete fixed reviewer thread | 75.3157% | 36.8717% |

These percentages have separate scopes. They cannot be added. They are not shares of the whole
workflow gap.

The remaining screens do not supply another saving claim:

- Under the requested feature-off direction, uncached input increased **59.242%**, while raw input
  decreased **14.638%**. The archived source uses bundle minus inline. In that source direction,
  the bundle reduces uncached usage by 59.242% and increases raw usage by 14.638%. The result is
  mixed.
- Patch acknowledgement has no observed raw or behavioral benefit. The comparison is cache
  confounded and is not a formal causal pair.
- Linearization compaction is inactive in the measured workflow.
- Command output offers zero bytes for compaction in the measured rows.
- Directive interruption shows no observed continuation benefit. The interrupted turn has no usage
  counter, so it cannot support a token claim.

## Outcomes, admission, and retries

The quality cohort includes saved workflow successes and failures. Scores of one, two, and three
features are present. No zero-feature row was observed, but the scorer keeps a zero score if one is
present. Saved workflow PASS and original-task feature score are different fields.

There was no uniform predeclared retry cap. Current comparison conditions have one or two saved
attempts, with at most two observed. No row is removed because it is a retry, a partial result, or a
saved workflow failure.

The R19 ledger contains 15 completed attempts. Thirteen meet the historical workflow PASS rule.
Eleven have exact accounting, and nine of those also pass the workflow rule. During collection, each
paid launch required explicit authorization. Pending conditions were interleaved by their attempt
count. A condition stopped after its first admission under the active collection gate. The offline
workflow interpretation does not create attempts or missing counters.

`wl-110` attempt 2 was interrupted without a completed report. It contributes no outcome or token
value. No paid run is authorized by this study.

## Limits

Formal quality equivalence is unavailable. There are only four matched pairs. No equivalence margin
was declared before collection. Retries and dependent observations are present. The descriptive
intervals are wide: -0.833 to +1.250 for the aggregate difference and -1.00 to +1.50 for the paired
difference.

Exact whole-gap allocation is also unavailable. Only **6 of 9** exact R19 factorial cells exist.
`wl-111`, `wl-101`, and `wl-011` are missing. Substituting cells from other blocks would mix time
blocks and observer identities, so that substitution is invalid.

Cross-project generalization is deferred. The controlled tests use one repository and one pair per
mechanism.

## Paths investigated

The compact scorer comes from
`/home/user/.codex/work-leaf-investigation-archive-20260824/step1-recovery-archive/payload/original-worktree/bench-results/efficiency-causal-study/step228-original-task-quality-rescore`.
Its seven primary file hashes and all 64 log hashes are recorded in `provenance.json` and the frozen
`result.json`.

Workflow and token accounting come from
`step1-recovery-archive/payload/original-worktree/bench-results/efficiency-causal-study/step226-historical-pass-reanalysis/result.json`
under the same external root. Its SHA-256 is
`1b1c623c6b204065584e48d75e9d38bf2b334317ac0036db71b7f9bba77d1dff`.

The controlled source files are the step-56 changed-reread comparison, step-63 unchanged-reread
comparison, and step-86 review-provenance comparison under the archived study tree. Their SHA-256
values are `a04de0bb918568f3bc40d5c0ae311bd2c3a8d4d1699513b8cdbbb7e950e18c9f`,
`900e68bf26acfedd0a8f19624b41cc100e9ba0627615d84e7ab435d8590561ea`, and
`f1690e1a817b8b97436fc575d19ccc2a84a55961b828075d3ecc8e13b1901774`.

Step 3 replay evidence is at
`/home/user/.codex/work-leaf-investigation-archive-20260824/step3-final-replay-evidence`.
Its replay ledger SHA-256 is
`3826465c132f93fdd31ad99a0af7cc24dcf01f98ae6497460318378e4195626d`. That replay exercised 66
bounded saved candidates. It permitted no real agent or model execution. It is replay evidence, not
quality or token evidence.
