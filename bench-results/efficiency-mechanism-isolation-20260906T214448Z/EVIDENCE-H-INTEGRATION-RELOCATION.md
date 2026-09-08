# H integration work: finite relocation evidence

Documentation and final validation work is demonstrably assigned to, and performed by, the linearizer in **both** historical conditions. Some WL integration work repairs inherited author lint/test debt; other work resolves history-reordering conflicts or failures that arise inside integration. This establishes relocation and additional integration work, not its counterfactual cost or a WL-specific saving.

## Population and identity

The finite population is the **12 accepted execution turns** of the six normal-WL and six Direct linearizer sessions in [H evidence](../efficiency-exact-normal-work-leaf-20260829T181318Z/evidence.json). Each session also has a separate planning turn, excluded from these action counts. Plan inputs and their exact reviewed-commit target lists remain provenance at native N7. Every execution input begins with the actual plan acceptance, carries an explicit native turn ID, and is distinct from interrupted-turn instrumentation.

[The companion JSON](EVIDENCE-H-INTEGRATION-RELOCATION.json) pins 33 source files and retains all **666** ordered native call/output pairs: physical lines, exact call and item IDs, supported type pairing, argument/output SHA-256, exit status, classification, all patch targets, and selected exact command strings. It includes the 12 full execution inputs, public-message identities, and preceding author witnesses. No private reasoning body was inspected or exported. Existing [H native identity/lifecycle evidence](EVIDENCE-H-LIFECYCLE.md) supplies the full accepted-input/public/native joins; this note does not rerun that workflow audit.

H WL is source commit `5b1d1ef9590850faed26052f909ddff7ff8f127d`, CLI 0.150.1. The saved source links below use [P source](../efficiency-raw-token-pilot-20260906T185328Z/source), whose relevant H bytes are independently equality-qualified in the lifecycle note. That qualification does not recover the four older Direct driver trees. Actual Direct launch/acceptance payloads, not modern driver substitution, support the Direct claims. The first three Direct runs use CLI 0.149.1; the step4 runs use 0.150.1.

| Execution source (link opens acceptance) | Condition | Accept N | exec / patch / plan calls | Failed / negative inspections |
| --- | --- | ---: | ---: | ---: |
| [point7-exact-direct](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T15-27-20-01a0488d-b117-7ae1-b7c1-8a12b45f52f0.jsonl:333) | Direct | 333 | 54 / 6 / 6 | 1 / 1 |
| [direct-003](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T19-57-50-01a04985-5746-7ff0-afb2-ee5d8053428c.jsonl:304) | Direct | 304 | 30 / 4 / 0 | 1 / 0 |
| [direct-002](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T21-15-42-01a049cc-a2aa-7b82-9135-6f22e9508149.jsonl:302) | Direct | 302 | 26 / 3 / 0 | 0 / 0 |
| [step4-direct-001](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T14-07-40-01a04d6b-1b2c-75d3-8070-ffc49bb85808.jsonl:321) | Direct | 321 | 53 / 7 / 4 | 2 / 0 |
| [step4-direct-002](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T15-40-10-01a04dbf-cd8e-7bf0-9343-dc8d622d0c0f.jsonl:184) | Direct | 184 | 44 / 4 / 7 | 1 / 0 |
| [step4-direct-003](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T16-58-35-01a04e07-9709-7d70-82fe-fcced31e06df.jsonl:318) | Direct | 318 | 41 / 4 / 0 | 0 / 1 |
| [exact-normal-001](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T20-47-03-01a04ed8-c2dc-7d00-9299-b143d400ade1.jsonl:83) | WL | 83 | 61 / 11 / 0 | 5 / 0 |
| [exact-normal-002](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T20-44-30-01a04ed6-6ddd-7b20-837d-d489a829ceec.jsonl:93) | WL | 93 | 42 / 5 / 0 | 0 / 0 |
| [exact-normal-003](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T20-41-07-01a04ed3-5558-7720-b25c-4f7b6b6ce78f.jsonl:77) | WL | 77 | 55 / 4 / 4 | 0 / 0 |
| [exact-normal-004](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T21-25-10-01a04efb-a797-72c1-9517-30bfc584d0d5.jsonl:113) | WL | 113 | 60 / 6 / 6 | 2 / 2 |
| [exact-normal-005](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T21-44-19-01a04f0d-3117-7b22-b2e3-042ccdf0088d.jsonl:71) | WL | 71 | 51 / 4 / 0 | 2 / 1 |
| [exact-normal-006](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T21-33-54-01a04f03-a5d8-7421-931f-5c4538cedaf1.jsonl:63) | WL | 63 | 55 / 4 / 5 | 1 / 0 |

Execution totals are WL **324 / 34 / 15**, Direct **248 / 28 / 17**. These are native tool calls, not shell subcommands, model responses, tokens or net work. A combined command is one call. The original whole-linearizer action index also includes planning; its counts have a different boundary.

## Declared obligations and their actual exposure

The H normal-WL chain is [`PromptPolicy`](../efficiency-raw-token-pilot-20260906T185328Z/source/src/agent.rs#L235) → patch-agent docs prohibition and [focused-check/cross-agent guidance](../efficiency-raw-token-pilot-20260906T185328Z/source/src/agent.rs#L328) → [`linearize_preamble`](../efficiency-raw-token-pilot-20260906T185328Z/source/src/agent.rs#L481) → [`build_interactive_linearize_prompt`](../efficiency-raw-token-pilot-20260906T185328Z/source/src/linearize.rs#L170). The final role owns deferred prose, reviewed-history rewriting and required checks until passing. WL does **not** simply waive all author validation: focused/changed tests remain mandatory, and its broad-check exception concerns external integration blockers.

At least one exact initial author launch per run is pinned in the JSON (12 N7 witnesses). Every inspected Direct launch explicitly defers docs and broad cross-feature formatting, Clippy and full-suite checks to the final linearizer, with an exception when necessary to finish the feature. Every inspected WL launch explicitly defers docs and contains its narrower focused-check/cross-agent contract. All 12 accepted execution prompts explicitly require final checks. Thus this responsibility split is shared, although the launch wording is not identical.

All 12 execution turns perform documentation patches and required final checks. **51 of 62 patch attempts are documentation-only**: 46 succeed and five fail context matching. The other 11 successful patch calls are completely enumerated below. Routine `cargo fmt`, Git replay/reset/amend/fixup and final verification are separate from new semantic feature implementation. No other execution write program was found in the complete command strings: exec calls begin with Git, Cargo, rg, sed, nl, or a Git sequence-editor assignment.

## Concrete inherited and integration-created chains

N locators in each paragraph refer to that row's exact native source above; author sources are separately linked.

### WL001: inherited lint debt, then changed test assertions

The [visual author's accepted patch N174](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T20-15-49-01a04ebc-2a9e-7662-8f06-5d0cfe7c08fd.jsonl:174), acknowledged at N180, contains the helper/loop shapes later rejected by Clippy. Its last retained focused `cargo test visual --lib` result is status 0 at N326; author DONE is N339. The linearizer's N7 target list and successful replay preserve that reviewed visual commit chain. Integration Clippy **N247→248, exit 101** reports nine lint errors; **N269→271** groups viewport arguments and rewrites range/loop shapes in `src/ui.rs`. Clippy then succeeds at **N282→283**. This is inherited validation debt repaired at integration, not demonstrated completion of a missing visual feature.

The [review-done author's patch N156](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T20-15-58-01a04ebc-4c57-7081-aade-5bc50a4fcd61.jsonl:156), ACK N163, introduced the exact combined `ready_frame` assertion and broad `user-1` absence assertion. Its focused command results at N206/N277 and `cargo test feature_done --all-targets` N401 pass before DONE N404; these command names do not prove complete coverage of the later failing test.

Integration full-suite failures **N287→288**, **N337→338**, and combined gate **N372→373** expose `scripted_harness_review_done_prompt_yes_closes_and_typing_reopens_chat`. Successful patches **N314→316**, **N360→362**, **N378→380** first split the expected row/status text, then move row assertions from the full frame to `render_left_pane()`, then narrow the absence match from `user-1` to `parser user-1`. **N390→392** reapplies the repaired test while placing it in the intended review feature commit; it is not a fourth independent defect. Final combined fmt/Clippy/full suite **N422→423** succeeds.

These are substantive **test-oracle/coverage changes**, not production feature fixes or proof of unchanged tests. In particular, checking a pane helper is not identical to checking the rendered frame. The record supports neither silently discarding these failures nor claiming test-quality equivalence.

### WL006: inherited visual lint, no separate feature repair

The [visual author's accepted patch N171](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T21-03-40-01a04ee7-fa7c-7331-bd53-8621568fea10.jsonl:171), ACK N178, contains the old index-based `character_selection_ranges` loop. Focused `cargo test --test ui_harness visual_` results at author N266 and N303 pass; DONE is N316. The linearizer's target list includes the reviewed visual commit and follow-up line-copy/mapping fixes, successfully replayed at N128→129.

Integration Clippy **N240→241** fails on that loop. **N250→252** converts it to enumerate/take/skip iteration. Formatting, fixup/autosquash, Clippy **N284→285**, full tests **N290→291**, and final fmt **N308→309** succeed. This is another exact inherited lint-to-final-validation chain; no new requested behavior is identified in this patch.

### WL004: replay-order conflicts, not an unfinished feature

The N7 target list separates visual commits `0d97d489…`/`7ac32e92…`, review-done `72dc9a49…`, and slash-routing `b3106756…`/`50110664…`. During reordered replay, visual cherry-pick **N175→176** and review cherry-pick **N226→227** each exit 1 with a conflict in `src/ui_harness.rs` imports. Patches **N195→197** and **N244→246** resolve those exact import blocks, first preserving visual imports and then combining the visual and review imports as the corresponding commit is replayed. The rg exit-1 checks **N200→203** and **N249→253** mean no remaining conflict markers, not failed repairs. Final Clippy **N361→362** and full suite **N366→367** pass.

This work is concretely produced by rewriting the reviewed history. It is not evidence that either author left its requested behavior unimplemented, nor proof of a concurrent-worktree failure during authoring.

### WL005: failed check followed by successful retries without a repair

After documentation and formatting/history work, full tests **N293→294** fail `terminal_app_answers_agent_file_requests_even_when_one_requested_path_is_missing` on a frame-text assertion. The exact focused retry **N300→303** and later full-suite retry **N309→310** succeed, with no intervening code patch. The public message calls the failure timing/flaky; the observed evidence establishes non-reproduction on these two checks, not its root cause. No specific author deferral is inferred for this failure.

The earlier `git diff -w --exit-code` **N277→281** exits 1 and shows differences; it is a negative identity inspection, not a successful equality proof. Public claims that the remaining difference is “formatting-only” are not substituted for semantic equivalence.

### Direct step4-001: deferred help/test support, then an integration-created regression

The [slash author's final public message N582](/home/user/.codex/sessions/2026/08/29/rollout-2026-08-29T13-29-18-01a04d47-fb77-77a3-99b8-0f57b5363c0c.jsonl:582) reports the selected-agent routing fix and focused checks, states that docs/prose were not changed, and leaves Clippy/full-suite work to final validation. That is a public author statement, not a replacement for its underlying command evidence. Integration's N7 target list and replay identify the reviewed routing work.

Patches **N427→429** and **N445→447** add CLI help text describing slash routing and corresponding help assertions/tests. This is additional user-facing support/test work for the already implemented routing, not another routing implementation. After Clippy **N565→567** passes, full suite **N573→575** fails two PTY startup visibility tests. **N581→583** shortens the help text added during this same integration turn (and an existing description); later Clippy **N646→648** and full tests **N652→654** pass. The public explanation attributes the failure to wrapped help pushing the header out of view; the exact local intervention and before/after outcomes support that integration-regression chain, not a new author defect.

An additional fixup command **N595→597** fails with an invalid commit reference (exit 128), followed by a corrected reference and successful history rewrite. The original outer-launcher failure after report publication remains part of H; successful linearizer checks do not erase it.

## Remaining executions and complete failure disposition

The other seven rows—point7 Direct, Direct003, Direct002, step4 Direct002, step4 Direct003, WL002 and WL003—have **no non-documentation apply_patch call**. Their inspected execution actions consist of docs, history assembly/rewrite, formatting, inspection and final gates. All 11 non-doc patch calls across the population are accounted for by the four repair/support sections above (WL001 five; WL004 two; WL006 one; step4 Direct001 three). No separate missing-feature implementation is identified in this finite public-action census.

The **15 genuine failed actions** are retained: five documentation patch context failures; two lint-check calls (WL001/WL006); five test-containing failed calls (WL001 three, WL005 one, step4 Direct001 one); two WL004 cherry-pick conflicts; and one Direct fixup failure. This counts calls, including a combined fmt/Clippy/test command, not independent failing tests.

The five additional negative inspections are kept separately: point7 Direct N628 and step4 Direct003 N485 have no rg matches; WL004 N200/N249 have no conflict markers; WL005 N277 reports a nonempty diff. None is silently converted to a successful repair or discarded. All 12 executions eventually record successful required integration checks; this local fact is not a new workflow-outcome or quality filter.

## Supported conclusion and limits

This closes the narrow relocation gap: **author-only work is an incomplete boundary** because final docs, checks and some repairs occur later. Docs/global validation are shared assignments in both conditions. Exact inherited WL lint/test debt, rewrite-generated conflicts, a non-reproduced test failure and a Direct integration-created help regression must remain distinct.

The evidence does **not** establish that apparent author savings are *merely* moved work, that WL saves because Direct lacks a linearizer, or that counting/subtracting these actions yields a causal saving. Planning is outside these execution counts; source reading, input size, generation and repeated histories are different surfaces. This note computes no token totals, percentages, quality-adjusted inclusion, current-screen outcomes or hypothetical alternative prices.

The companion index preserves 33 SHA-pinned endpoints and all 666 call/output pairs without a new runnable collector or model work. Source records were rechecked at the endpoint, including exact argument/output hashes, the complete post-acceptance call census and selected detail strings. Earlier read-only oversized projections were truncated and one JSON parse failed on that truncation; complete per-run projections and targeted public-source reads supplied the retained evidence. Neither tooling failure changed a source, ran a test or discarded a workflow.
