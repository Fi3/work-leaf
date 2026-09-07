# Prior repeat-read and review-context screens

This provider-free Stage B review covers the earlier points-8/9 and corrected-all-disabled studies, plus an independent check of [H context evidence](EVIDENCE-H-CONTEXT.md). It inspects no parked R costs, launches nothing, changes no frozen evidence, and does not reopen the accepted historical H endpoint. The older studies' accounting conclusions belong to their own populations and measurement rules; they do not replace H's accepted result. No quality-based exclusion or new percentage is proposed.

## What was tested

Paths beginning `P89/` below mean `../efficiency-points8-9-20260828T145556Z/`; `CAD/` means `../efficiency-corrected-all-disabled-20260829T091341Z/`.

| Prior factor | Actual source boundary | Present-factor relevance |
| --- | --- | --- |
| Changed requested reread: diff versus current full text | `P89/infrastructure/instrumentation.patch:109`, `render_file_read_response_with_repeats`; new `render_changed_repeat_full_current_snapshot` at patch line 222 | Directly exercises C02's changed-repeat representation subfactor. It is not an untested family. |
| Unchanged requested reread: digest versus current full text | Same renderer, patch line 140; full-current body appended after the ordinary path/digest receipt | Directly exercises C02's unchanged-repeat representation subfactor. |
| Inline exact review context versus Git reconstruction | `P89/infrastructure/instrumentation.patch:50`, `CommandChat::review_commit_streaming_with_ids` | A different, compound intervention from C21's proposed on-demand access to the same complete evidence. |
| Corrected all-disabled | Both full-repeat switches plus Git reconstruction; `CAD/infrastructure/corrected-control.patch:16` adds an arm-specific verdict/completion instruction | Tests a combined alternative workflow, not three independently identified effects or a pure evidence-location factor. |

The repeat switches accept `normal` or `full`: `WORK_LEAF_EXPERIMENT_CHANGED_REPEAT_DELIVERY`, `WORK_LEAF_EXPERIMENT_UNCHANGED_REPEAT_DELIVERY`, with shared fallback `WORK_LEAF_EXPERIMENT_READ_DELIVERY` (`P89` patch lines 101–105, 172–215). They modify repeat response rendering, not first-read thresholds, read tracking, automatic refresh, or the author launch policy. The source patch also includes deterministic changed/unchanged full-body fixtures and a Git-reconstruction review fixture; their existence is not a substitute for the actual exposures below.

There is a concrete launch-contract difference from current C02. In `P89/runs/wl-100-001/points89-wl-100-001-three-feature-bench-artifacts/observation/app-server/00002578722918249004-405/client-to-server.raw`, physical **C4**, RPC `3`, thread `01a048e6-0787-7f72-8ded-919ea1506299`, the full-repeat arm still receives:

> If you request a file you already received, Work Leaf compares digests and returns either unchanged status or a diff from your last snapshot; do not use repeated reads to reload whole files.

It also retains the normal repeated-digest authority sentence. Current `src/bench_candidate_experiment.rs::policy_pair` at lines 115–123 changes those two owned sentences to describe full-current requested reads truthfully. Current C02 is therefore a requested-repeat delivery **contract**, not a byte-identical rerun of the earlier renderer-only manipulation. This distinction does not erase the earlier exposure or its unresolved result, and does not establish that the policy difference explains it.

## Points 8/9: exposed, but unresolved

The fixed four-cell screen selected these rows (`P89/evidence.json#/factorial_screen/cells`, in condition order). Counts are delivered file events from the retained observer summaries, not accepted provider-turn counts or input-token prices.

| Condition / row | Changed-repeat events | Unchanged-repeat events | Qualification |
| --- | ---: | ---: | --- |
| `000 / wl-000-003` | 9 diff | 1 digest | All ten component counterfactuals verified. |
| `100 / wl-100-001` | 15 full | 1 digest | Full changed bodies exposed; their counterfactual diffs were not reconstructed. |
| `010 / wl-010-001` | 6 diff | 3 full | One changed-diff reconstruction invalid; five verified. All three unchanged full deliveries verified. |
| `110 / wl-110-001` | 8 full | 8 full | Both subfactors exposed; changed counterfactual diffs unavailable. |

The additional normal row `wl-000-002` is retained in the endpoint group, not silently substituted into this four-cell screen. Its 17 changed events include one invalid diff reconstruction; its 15 unchanged events include four requiring snapshot resolution (`P89/evidence.json#/observations/wl-000-002/delivery_events`).

Both recorded whole-workflow factor intervals cross zero (`factorial_screen.changed_reread_full_minus_diff.raw_token_interval` and `unchanged_reread_full_minus_digest.raw_token_interval`). The disposition is **inconclusive under those bounds and that sample**, not proof of no effect, insufficient exposure, or a demonstrated small causal contribution. Verified local avoided/resupplied bytes are real representation differences; they are not a whole-workflow causal token allocation.

The original twelve-row schedule yielded seven completed attempts, one admitted reliability failure, and four withheld rows. `wl-111-003` repeated the feature-2 `done`/`NO_FINDINGS` routing cycle and was stopped. Its temporary candidate and unpublished usage capture were lost during signal cleanup; its admission and driver log remain. `wl-001-001`, `wl-011-001`, `wl-101-001`, and `wl-111-002` were withheld, not zero-exposure successful observations. These facts are preserved in `P89/STATE.md`, `SCHEDULE.tsv`, and `FINAL-RESULT.md`; this note does not manufacture exact delivery/usage evidence for the lost capture.

## Review reconstruction is not C21's same-evidence on-demand boundary

The old Git arm removes both `commit.context` and `render_review_source_context` from the inline prompt. It asks the reviewer to start at the latest commit, include immediately preceding **contiguous same-Agent-ID** commits, stop at a different/missing Agent-ID, and obtain full commit messages and their cumulative diff through `@work-leaf locks run . -- git ...` (`P89` patch line 86).

That supplies a reconstruction procedure, not the same complete archived Work Leaf scope, chat history, logs, verification results, and blocker evidence behind a read-only pointer. It cannot be assumed that noncommitted evidence is recoverable from Git, or that this contiguous-author rule produces the exact normal scope. It also introduces explicit Git command work. Those are actual source-level differences; no claim is made that every possible information loss occurred in every run.

The corrected arm explicitly orders `NO_FINDINGS`/`FINDINGS` first in the **final review response**, before `@work-leaf done`. This is an additional review-completion prompt intervention, not merely a storage relocation (`CAD` patch line 21). The corrected source does not replace WL's first-nonempty-line classifier or establish that all commentary-before-marker aggregates become clean. The independent clean-marker evidence remains [a separate parsing mechanism](EVIDENCE-CLEAN-MARKER-REPLAY.md).

Accordingly, the original routing failures are relevant reliability evidence against that Git alternative. They do not disprove useful exact-evidence bundling. Conversely, a later complete Git-arm run does not establish information equivalence to C21 or permit attributing the compound result only to evidence delivery.

## Corrected all-disabled: compound exposure, not independent identification

`CAD/README.md` and `STEP4-COLLECTION-PLAN.md` retain source `d217f3803ac0f417671e27cc8fb18064ff0f4ea9` for batches 1–3 and `72a9e507f57daf20a54bab5dcd6fe8f13f083d30` for batch 4's post-workflow instruction-cleanup correction. The base, task hash, model, effort, final checks and scorer are documented as fixed. The final dataset is six Direct, five normal WL, and six corrected all-disabled workflows—not H's later six-normal-WL cohort.

All six disabled rows remain: `corrected-all-disabled-001`, `002`, `003`, followed by `step4-control-001`, `002`, `003`. `final-evidence.json#/control_activation` records **12 changed-full events in four rows, 26 unchanged-full events in all six, and Git-review exposure in all six**. The first three checkpoint rows have changed/unchanged counts respectively `0/2`, `4/5`, `4/11`; their nine reviewer sessions retain mediated Git commands and final clean markers (`CAD/evidence.json#/runs/*/review_control`). Missing changed exposure in a particular workflow is not a failed activation of the unchanged/review factors.

The final combined all-disabled-minus-normal interval crosses zero. That is not an isolated C02 or C21 test and does not establish either a causal fraction or zero effect. The report's command-frequency association is also not an independent mediator intervention or exact upstream response census.

All recorded failures remain relevant (`CAD/STEP4-FAILURES.md`):

- `step4-normal-001` reached three feature provider turns, then invalid `diff`/`digest` environment values stopped the intended workflow. It is a retained infrastructure attempt, not a normal-WL observation or an unlaunched row.
- A live launcher edit caused outer parse-error exits after the `step4-direct-001` and `step4-control-001` children published. Their outer failures remain alongside the complete child evidence.
- `step4-normal-002` implemented/reviewed/linearized the requested work, then failed temporary-instruction cleanup. Its workflow failure remains in the normal group.
- `step4-control-003` failed a final terminal test. Its genuine workflow failure and saved candidate remain in the all-disabled group; it was not replaced.

No quality-filtered subset is used here to rescue or reject a candidate.

## Independent H-context review and limits

At the reviewed H-context note hash `3b588ad431f4ea2624d8d111b6d29f50e786aa992d8e4dfc9cfdb9b4f9ba98d3`, I independently checked the cited H renderer/tracker Git object and the exact raw locators for: H001 `C12/S131/S137/S192` selective bundle output; H004 `C52/S17091` refresh then `C56/S17280` unchanged reminder; H001 `C109/S62117`, `C111/S63223`, `S63312`, `C117/S63318` own-edit invalidation/reread; H005 mixed `C58`; and H001 `C16/S425` plus H006 `C22/S575` small inline responses. The reported IDs, lengths and prompt/output hashes agree.

I also independently reconstructed **all 84 archived bundles** (15/11/16/11/16/15) using the old source's exact prefix, ordered file frames, recorded UTF-8 body lengths, FNV digests and newline rules, verifying every complete bundle SHA-256. H001's selected `sed` output equals exactly the archive's first 220 lines. This confirms that witness's returned-text selection, not selective disk I/O, complete provider-input contents, or a token price. The earlier [H lifecycle audit](EVIDENCE-H-LIFECYCLE.md) independently covers all accepted turn/public-user joins. This review does not claim a second independent replay of every command-output comparison in the sibling note.

The H-context note at its accepted hash `44e285a20cf78848f66db6eaa18e4a5fa1a78d520d48cedfc77559a874e36a0b` names current C02's truthful requested-repeat contract, including the two policy spans, and distinguishes the older renderer-only knobs. The underlying source evidence is identical to the reviewed cutoff. No source-locator or archived-body discrepancy was found in the reviewed checks.

## Provenance and disposition

The historical JSON source inventories were rehashed without regeneration: 39 checked sources for points-8/9, 19 for the corrected initial checkpoint, and 55 for its final evidence; all matched. Counts can overlap between inventories. The additional points-8/9 full-arm launch capture above has SHA-256 `49785db8dc3b4ca6206bb1dff25982179ff6c66d8f121c34d18056841bbfce1c`.

| Retained source | SHA-256 |
| --- | --- |
| `P89/evidence.json` | `87b826fd951dbe11486d2e9dc0925e11ca1c145ce9411fa24f0e3383d4f0a236` |
| `P89/FINAL-RESULT.md` | `01587157b8d97246a0fbdb446dacfa984dbdf9c557ed28859fc7eed98973c70a` |
| `P89/infrastructure/instrumentation.patch` | `3d90048d6ed15c474aa5447946b507884ead5eb2929502d00674cb4116511e21` |
| `P89/SCHEDULE.tsv` | `bc80fd0f8d2a4bc494f2d27181fbf842059810e7291dd205ad31a027c3fcd135` |
| `CAD/evidence.json` | `e6b7d465f4a46feb91fd09cdc0f8cc7f1944516b7157030f8bdff38b1cf9eae2` |
| `CAD/final-evidence.json` | `ef7854807123f5a26a620c2347b2ba5093a5acb43be0c0086aa000f35a316638` |
| `CAD/FINAL-REPORT.md` | `1013e6c48bb04e0dab686e066396cb90aee1fb14c21bc6bd0c8b3e7dce30aa04` |
| `CAD/infrastructure/corrected-control.patch` | `5c462e7c42210d3695b8061d9e13dfd0fc1c1006c716fe70051862c379bc03f7` |
| `CAD/STEP4-FAILURES.md` | `e3b1b69cad1a18899178552688fdca12e3de30520a620b69128681a7f0b4ade5` |
| Current `src/bench_candidate_experiment.rs` | `e327d28fb05ceaa8f9c9465a78fc61069eddb56faa767b4909f29f83684d8b94` |

Practical conclusion: retain C02 as an **already-exposed, unresolved representation family**, distinguishing its current truthful contract. Retain C21 exact-evidence on-demand as **not answered by the old Git-reconstruction control**, while preserving the latter's real routing failures and compound corrected result. Neither conclusion authorizes new controls, a provider launch, or a percentage allocation. This document affects no agent-facing workflow and requires no real-agent generation.
