# Completion fixture: bounded saved-source diagnostic

2026-09-13 00:44 UTC. Read-only saved evidence; no fixture/source edits, test execution, provider generation or new benchmark.

## Finding

**The pilot's line-23 failure is an author-chat selection/presentation gap. Completion prompting itself exists. The same failure predates this treatment in historical WL runs 004 and 006.** This is not the earlier line-41 visible-close wording failure, and the raw frozen result remains FAIL.

The original task still requires the patch chat to be highlighted and ask whether the feature is done, with yes closing and later input reopening it. Nothing in the delivered task removes those behaviors. The available evidence does not establish whether the treatment affected the probability of this implementation mistake; recurrence before the treatment is not proof of causal independence.

## Exact pilot chain

Paths in this section are relative to this phase's `runs/standalone-global-hunk-pilot-001/standalone-global-hunk-pilot-001-three-feature-sequential-bench-artifacts/`.

- `runs/sequential-feature-3-implement.host/invocation-0001/prompt.txt:4` retains the original feature request. Lines 20–26 retain task/tests/review obligations while explicitly changing test-execution timing; therefore unchanged feature wording is not proof that every implementation outcome must be identical.
- Final `patches/pass/format-patch/0003-ADD-review-completion-confirmation-so-clean-patches-.patch:362` modifies `WorkLeafController::apply_agent_result`: resolved reviews call `request_feature_completion_confirmation`.
- The same patch at lines 384–388 defines that helper: set the reviewed author's completion state to pending, then append the question to that author's transcript. It emits no `AgentSelected` event. `set_session_completion` at lines 402–435 emits a status update, not selection. Review startup retains reviewer selection.
- The test `terminal_app_review_completion_highlights_patch_chat_and_yes_closes_until_next_message` checks the author's READY highlight, **clicks the author row**, then checks the question (patch lines 526–537). Its close/reopen assertions follow that manual selection. `checks.log:216` records this test PASS; the controller close/reopen test also passes at line 308.
- The frozen fixture `bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z/scorer/fixtures/quality_completion.rs:19` waits for idle and checks the currently rendered frame **without clicking**. Line 23 requires the question already visible; lines 24–27 then require selected agent `user-1`. The pilot remains on `review-user-1`, so it fails before close/reopen assertions.

Thus the concrete difference is **question available in the author chat after selecting it** versus **question immediately visible with the author already selected**. The fixture's failure does not mean no completion question was implemented. Nor can its later unexecuted assertions be counted as passing.

## Historical recurrence and comparison

Let `H=bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z`.

The completion fixture SHA-256 is identical in historical `H/quality.json` and the pilot `postcapture/quality.json`:
`4d6a19f4c6f515b9a97f184a056d08dc2882041d40d26dd82011180a949c5c87`.

Historical `H/scorer/logs/exact-normal-004/completion.log:14` and `exact-normal-006/completion.log:14` both fail the same line 23. Their rendered frame bodies are **byte-identical to the pilot's**: selected reviewer, highlighted READY rows, empty chat. Frame SHA-256, from after the initial bell/home escape through the frame before the backtrace note:
`65ab10558983d071595d8a36dbb5277dd09b69eef8a06fa89b5861bde8101e30`.

This is also source-grounded, not only visual coincidence:

- Historical 004's saved checkout is `/home/user/.codex/work-leaf-exact-normal-score-20260829T181318Z/all-six/checkouts/exact-normal-004`. HEAD `ace5a6713d4c47dc14fb806d5de49d52c5a87565` matches `H/quality.json:215`; working `src/workspace.rs` blob matches HEAD exactly (`a7bbccb4afbf8eab9a5979a393cd7acba3a2dafb`).
- Its `src/workspace.rs:402` selects the reviewer on startup; `apply_agent_result` at lines 708–716 calls `prompt_feature_done`; that helper at lines 766–770 appends the author question without selecting the author.
- Historical 006's saved `src/workspace.rs::request_feature_completion` at lines 783–790 likewise appends the question without a selection event. Its saved failure is the same observable mode; no new execution was needed.

The actual six historical Direct identities are listed in `H/evidence.json:2098`. Their saved completion logs show five PASS (`point7-exact-direct`, `direct-002`, `step4-direct-001/002/003`); `direct-003` reaches and fails **line 41**, after displaying the author question. Therefore this exact line-23 failure occurs in historical WL, but not in the six historical Direct observations.

Direct log roots: `bench-results/efficiency-point7-exact-accounting-20260828T113610Z/scorer/logs/`, `bench-results/efficiency-points8-9-20260828T145556Z/quality/logs/`, and `bench-results/efficiency-corrected-all-disabled-20260829T091341Z/scorer/logs/`.

## Evidence identities and inference limit

Pilot final completion patch SHA-256: `f487ef7f3c9ef8b05cf454de1673831a4331cbadcfae0a3d5daf322f95b2136d`.

Completion log SHA-256:
- Pilot: `68069fbeeb85892d47ce00b253c45891e9841cc7e0e3d6f3c6efbe4f0c385bcd`.
- Historical WL 004: `267227778596b328a68eb0bea74c2520e40ff699623acb86218a7ee1f411e957`.
- Historical WL 006: `5bc18fce4cc562e234d2c424f4f31dcd5a379546c97b7754605cc97575db73af`.

The narrow answer is a recurring implementation/fixture presentation mismatch, not evidence that the pilot omitted the whole feature or that its treatment explicitly reduced functional scope. Whether automatic author selection is intended by “highlighted and ask” is not silently resolved here: the frozen fixture requires it, the implementation supplies highlight plus a question in a nonselected chat, and FAIL is preserved. No token cost is attributed to this difference; the user's no-quality-loss working assumption and the separate accounting population remain unchanged.

