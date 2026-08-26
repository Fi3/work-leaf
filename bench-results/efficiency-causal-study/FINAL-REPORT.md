# Final efficiency report

## Abstract

This study asks whether normal concurrent Work Leaf uses fewer model tokens than doing the same
three-feature Rust task through normal sequential Codex sessions. It also asks whether any observed
saving comes from identifiable Work Leaf behavior rather than incomplete work, a different model,
or an accounting error. Both workflows use GPT-5.5 with `xhigh` reasoning, the same repository
starting point, the same task, the same per-feature validation limit, and the same final repository
checks.

The three scored features are Vim-like text selection and copying in both terminal panes; sending
`/status` to the selected backend agent and displaying its response; and highlighting a patch-agent
chat after review so the user can close or reopen that feature. `/fork` belongs to a later expansion
of the slash-command task and is not included in the three-feature score.

In the saved results, sequential Codex completes an average of 2.00 of the three requested features,
while concurrent Work Leaf completes 2.25. The one pair with complete, independently checked token
accounting shows Work Leaf using 72.0567% fewer total input-and-output tokens and 49.0420% fewer
tokens after cached input is removed. That pair is not equal in quality: Codex scores 3/3 and Work
Leaf scores 2/3. Separate controlled tests show that compact repeated-file responses and supplying
review context directly can each reduce tokens in the tested situation. The current data do not
show exactly how much each mechanism contributes to the complete workflow difference, do not prove
that the workflows are statistically equivalent in quality, and do not establish that the result
applies to other repositories.

## Names and terms

- **Sequential direct Codex** means using normal Codex sessions without Work Leaf. One feature is
  implemented and reviewed before work starts on the next feature.
- **Concurrent Work Leaf** means the normal Work Leaf workflow. Three patch agents work concurrently,
  followed by Work Leaf review and history cleanup. It does not mean the discarded experiment that
  submitted Work Leaf features one at a time.
- A **model profile** is the model name together with its reasoning setting. The comparison in this
  report uses GPT-5.5 with the `xhigh` reasoning setting for both workflows.
- **Candidate** or **row** means one saved implementation produced by one benchmark attempt. A
  **normal-product candidate** uses the normal sequential or concurrent workflow, not an experimental
  mechanism setting.
- A **cohort** is the complete group of candidates included in one comparison.
- **Frozen** means the task, scorer, or rule was fixed and identified by a file hash before this
  analysis. It is not silently changed between conditions.
- **Original-task quality** is the number of the three originally requested features that pass the
  frozen behavior tests. It ranges from 0/3 to 3/3.
- **`/status`** is the selected-agent backend command used to test the original slash-command
  feature. A pass requires a backend response to appear in the selected chat. `/fork` was added to a
  later task contract and is not part of the three-feature score in this report.
- **Workflow PASS** means the benchmark process completed its review and history-cleanup stages,
  produced the required commits, left a clean repository, and passed the required repository checks.
  It does not guarantee that all three requested features work. Workflow PASS and original-task
  quality are therefore reported separately.
- **Raw tokens** are input tokens plus output tokens. Cached input remains included.
- **Uncached tokens** are input tokens minus cached input tokens, plus output tokens. Raw and
  uncached totals answer different questions, so the report keeps both.
- **Exact accounting** means the saved provider events contain complete token counters and the
  offline checker independently reproduced the total.
- A measurement's **scope** states which work its token total covers. **Treated turn only** covers
  the one model response where a controlled behavior differs. **Complete reviewer thread** covers
  every model response in that fixed review conversation. Neither is a complete product workflow.
- **R19** is the name of the final saved full-workflow measurement batch. It is not a model version.
  The number 19 only distinguishes that collection iteration; it is not a sample count. **Attempt
  2** means the second saved execution of one condition in that batch.
- A **block** is a collection batch whose conditions share the same setup and time window. A
  **same-block pair** compares sequential Codex and concurrent Work Leaf from the same block and
  attempt number instead of combining unrelated runs.
- The **title agent** is Work Leaf's hidden helper that generates short chat titles. Its tokens are
  excluded from the R19 product comparison because naming chats is outside the three-feature work.
- A **controlled test** keeps a small task fixed and changes one Work Leaf behavior. The unchanged
  behavior is called the **normal setting**; the deliberately less compact alternative is called the
  **control setting**.
- **Review provenance** is the exact source, commit, and patch-agent history needed by a reviewer.
  **Inline exact** means Work Leaf supplies that information in the review prompt. **Git
  reconstruction** means the reviewer must recover it through Git commands.
- A **factor** is one behavior switched between its normal and control settings. A **factorial cell**
  is one combination of those switches.
- A **ledger** is the saved list of attempts and their outcomes. **Admission** means a saved attempt
  met the evidence requirements for a particular analysis; it does not mean that all requested
  features worked. A **retry cap** is a limit, declared before collection, on how many attempts a
  condition may receive.
- A **context bundle** is a file containing large tool output that the model can read by path instead
  of receiving the entire content directly in its prompt.
- **Patch acknowledgement** is Work Leaf's short response after accepting a structured edit.
  **Command-output compaction** means shortening long command output before returning it to the
  model. **Directive interruption** means stopping model generation as soon as a complete Work Leaf
  instruction has arrived. **Linearization compaction** means removing repeated context from the
  prompt used to turn reviewed provisional commits into final history.
- **Formal quality equivalence** would mean the quality difference is shown, with a predeclared
  statistical test, to stay inside a practically acceptable range. That range is the **equivalence
  margin**. A **descriptive interval** shows uncertainty in the saved sample but does not by itself
  prove equivalence. **Dependent observations** are runs that are not fully independent because they
  share collection batches, retries, or other setup.
- The **whole-workflow gap** is the token difference between a complete sequential Codex run and a
  complete concurrent Work Leaf run. **Whole-gap allocation** means measuring how much of that
  difference comes from each Work Leaf behavior, including interactions between behaviors.
- **Cross-project generalization** means showing that a result also occurs in repositories other
  than the single repository used by this study.
- A **SHA-256 hash** is a fixed identifier used to verify that an evidence file has not changed.
  Directory prefixes such as `step190` and `step228` are archive sequence numbers from the
  investigation; they do not denote benchmark features, quality levels, or model versions.

Work Leaf factorial conditions use the label `wl-XYZ`, where `wl` means Work Leaf. The three digits
always have this order:

| Digit | Behavior | `0`: normal Work Leaf | `1`: control setting |
| --- | --- | --- | --- |
| `X` | Rereading a changed file | Return only changed lines (a diff) | Return the full current file |
| `Y` | Rereading an unchanged file | Return only a short content hash (a digest) | Resend the full file |
| `Z` | Giving context to a reviewer | Supply exact context inline | Make the reviewer reconstruct it from Git |

For example, `wl-000` is normal Work Leaf with all three compact behaviors enabled. `wl-110`
returns full files for both kinds of reread but still supplies exact review context inline. `direct`
is the sequential Codex reference and has no three-bit Work Leaf setting. The complete design has
eight `wl-XYZ` combinations plus `direct`, for nine conditions in total.

## Supported result

The supported comparison is normal sequential direct Codex against normal concurrent Work Leaf.
Both use GPT-5.5 with `xhigh` reasoning on the fixed three-feature task. Other model profiles do not
enter the comparison.

The frozen original-task scorer retains every saved normal-product candidate. Sequential direct
Codex has six rows and a mean quality score of **2.00/3**. Concurrent Work Leaf has four rows and a
mean of **2.25/3**. `/status` succeeds in **4/6** sequential direct Codex rows and **4/4** concurrent
Work Leaf rows.

Four rows form same-block pairs. For each pair, subtracting the sequential score from the Work Leaf
score gives **-1, +2, +1, and -1** in frozen order. The paired condition means are also 2.00/3 and
2.25/3. The two sequential rows without a matched Work Leaf row remain in the overall average and
show the observed run-to-run variation.

Three older Work Leaf rows, used only to confirm that the scorer recognizes previously working
behavior, score **3, 2, and 2**. `/status` succeeds in all three. They used a sequential Work Leaf
schedule, so they are not part of the normal concurrent comparison and do not set an expected
current quality level.

The exact R19 attempt-2 pair counts model work for the requested workflow. It excludes title-agent
usage because naming chats is not part of implementing the three features.

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

The remaining tests do not supply another saving claim:

- Sending a large read through a context bundle instead of placing it directly in the prompt reduces
  uncached usage by **59.242%** but increases raw usage by **14.638%**. Because the two token measures
  move in opposite directions, this is a mixed result rather than another general saving claim.
- The short acknowledgement after accepting a patch has no observed raw-token or behavioral
  benefit. The two conditions also received different amounts of cached input, so this comparison
  cannot isolate the acknowledgement as the cause.
- The measured workflow never encountered a case where repeated linearization context could be
  removed, so that mechanism was inactive.
- The measured command output contained no bytes eligible for shortening.
- The model did not continue generating unnecessary text after a complete Work Leaf instruction in
  the measured case. The interrupted turn also has no token counter, so interruption cannot support
  a saving claim here.

## Outcomes, admission, and retries

The quality cohort includes saved workflow successes and failures. Scores of one, two, and three
features are present. No zero-feature row was observed, but the scorer keeps a zero score if one is
present. Saved workflow PASS and original-task feature score are different fields.

There was no uniform predeclared retry cap. Current comparison conditions have one or two saved
attempts, with at most two observed. No row is removed because it is a retry, a partial result, or a
saved workflow failure.

The R19 attempt list contains 15 completed attempts. Thirteen meet the historical workflow PASS rule.
Eleven have exact accounting, and nine of those also pass the workflow rule. During collection, each
paid launch required explicit authorization. Conditions with fewer attempts were run before
conditions with more attempts. Collection stopped retrying a condition once an attempt met the
evidence rule active at that time. Later rescoring changed how saved outcomes were interpreted, but
it did not invent attempts or fill in missing token counters.

`wl-110` attempt 2 was interrupted without a completed report. It contributes no outcome or token
value. No paid run is authorized by this study.

## Limits

Formal quality equivalence is unavailable. There are only four matched pairs. No equivalence margin
was declared before collection. Retries and dependent observations are present. The descriptive
intervals are wide: -0.833 to +1.250 for the aggregate difference and -1.00 to +1.50 for the paired
difference.

Exact whole-gap allocation is also unavailable. Only **6 of the 9** required R19 conditions have
exact accounting. `wl-111`, `wl-101`, and `wl-011` are missing. Substituting conditions from other
blocks would mix different collection times and token-capture setups, so that substitution would
not complete the same controlled design.

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
