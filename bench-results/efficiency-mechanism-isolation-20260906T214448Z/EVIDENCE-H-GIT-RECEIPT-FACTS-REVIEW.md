# Independent H Git-receipt evidence review

Disposition: **no factual or mechanism-attribution correction required** at the
reviewed source cut. A nonblocking validator-complexity qualification and the
public-versus-native coverage distinction are explicit below.

Reviewed [report](EVIDENCE-H-GIT-RECEIPT-FACTS.md) SHA-256
`02ab02c37db327ca30bbf220dd78bd6304e0c07b861011c55ec281819915acdd` and
[index](EVIDENCE-H-GIT-RECEIPT-FACTS.json) SHA-256
`a0eea9a869a22c2acdd4216e88041946aa85fb00ee5f721ee2ba03766a0859b5`.
Only this review file is written. No original evidence, helper, runtime, C15
source, provider, benchmark or accounting result is changed.

## Coverage and source checks

The complete report and all three embedded extraction/validation sources were
read before execution. The exact indexed `validation.source` is read-only;
its independent replay passes **252 calls, 39 cycles, 53 ACKs and 166 source
endpoints** (tool receipt `876233`). The extraction scripts were not executed;
the inspected extractors themselves print derived public metadata only.

A separate direct source traversal checked all 18 author native files, rather
than accepting the indexed category and phase labels:

- Exactly 252 leading-`git` `function_call` records, each with its unique actual
  `function_call_output`, matching call ID, distinct native item IDs, argument
  hash, command text, output hash, physical lines and explicit same-turn metadata.
- All 30 distinct command strings: 107 status, 39 scoped diff bodies, 52 diff
  statistics, 40 whitespace checks, 11 ordinary path lists, two masked scoped
  documentation-path lists, and one combined statistics/body command.
- All 39 actual `task_started` and `task_complete` records equal the indexed
  cycle population. All 39 saved implementation/fix prompt files are covered,
  without an extra/missing native cycle or saved author prompt (`1e1288`).
- All 39 unique timeline joins have the exact feature and initial/fix-round
  identity. Initial receipts name an accepted commit; fix receipts explicitly
  say `result=committed`. Actual native completion timestamps precede these
  retained host receipts, using integer nanosecond conversion, not float ordering.
- All 252 Git outputs precede their own actual turn completion. All 336 successful
  native patch completions used by the before/after classification were checked
  against actual call/output hashes and success bodies. Every counted prior patch
  output completes before its classified Git call; issuance alone is not used.
- The 31 before-edit status outputs are clean; the other 76 contain unstaged
  ` M` entries. All 40 `git diff --check` outputs are empty with reported exit 0.
  The sole native-truncated diff, explicit `sed` ranges and two `|| true` commands
  remain distinguishable; compound shell exit 0 is not a proof of each subprocess.

These checks pass (`a0e44f`). They verify current-cycle ordering, not complete
source-state equality between separate commands or absence of intervening changes.

## ACK facts and the namespace distinction

The original index proves **53 typed capture request/reply/completed-public-user
joins**. It does not contain 53 native-rollout item locators and does not claim
that stronger scope. Public and native item IDs must not be treated as identical.

This review separately rechecked those 53 native joins using the six historical
rollout metadata files pinned in [H lifecycle evidence](EVIDENCE-H-LIFECYCLE.md),
SHA `556da269740ff8a33edd25d351ea913ec1a4f77bd2524f099eb26bd8753c55b2`.
For each ACK: exact captured thread/accepted turn, full input text and its SHA
match one native user item with explicit passthrough turn metadata; no preceding
turn-context inference supplies user ownership. All 18 native author files match
the declared thread, cwd, CLI 0.150.1 and source hash. All 24 supplemental
metadata/native endpoints and the original 166 endpoints rehash unchanged.
Completed public items have the same text and explicitly empty `text_elements`.

| H run | Verified ACK/native joins |
| --- | ---: |
| exact-normal-001 | 9 |
| exact-normal-002 | 7 |
| exact-normal-003 | 10 |
| exact-normal-004 | 5 |
| exact-normal-005 | 10 |
| exact-normal-006 | 12 |

For the report's first ACK, C30/S3612/S3616 additionally joins native N169 in
`rollout-2026-08-29T20-15-53-01a04ebc-3891-7172-bbb2-3bbdab8c9de7.jsonl`.
Its source SHA is `f152484b3e6602ecbd646b887fee2a68d0a1d5aceb152d17d42ef6e8c3be8159`.
Its native item is `msg_01a04ec0-5c2d-7ed1-81ec-17b614349a48`; its public item is
`01a04ec0-5c2d-7ed1-81ec-17cd78a2507c`. Both resolve the indexed accepted turn
`01a04ec0-5c10-7ce0-9745-ea593070f418` and input SHA `3ee791f8…`.
This supplemental check does not retroactively add native fields to the original
index or infer a per-ACK commit hash.

All 53 actual ACKs have the indexed common tail after the file-list line. The
success and provisional-commit facts are present; commit SHA/subject, diff body,
statistics, whole-tree status/untracked census, whitespace-check result and final
test-success facts are absent. The report's description is therefore accurate.

## Owning source and interpretation

Historical `src/orchestrator.rs::handle_agent_directives_streaming` collects
successful patch files at lines 792–808 and 864–880, then delivers
`render_patch_applied_prompt` at 982–985. The complete renderer is at 2638.
`src/patch.rs::GitPatcher::apply_edit_with_locks` (150–186) stages/commits before
returning success; the unified path has the corresponding commit order.
`src/agent.rs::PromptPolicy::for_read_permission` (227) supplies the distinct
orchestrator read route, without an invented Git-specific prohibition.

The linked retained P source files were compared with actual H Git object
`5b1d1ef9590850faed26052f909ddff7ff8f127d`. Full-byte hashes match:
orchestrator `f88aeea1…`, patch `aa9a0df5…`, agent `dd4464f9…`.
This equality does not substitute current split-driver source for the four
unavailable older Direct driver revisions. The historical Direct ordering is
supported by actual saved prompts and native/host receipts instead.

The conclusion is appropriately narrow: these calls inspect precommit status
and accumulated uncommitted work, not the same postcommit facts supplied by a
WL ACK. Partial file-list overlap does not identify an avoided model action.
Publication cadence, edit custody, continuation policy and access routing remain
interacting mechanisms. No individual factor, redundancy, quality-equivalence,
causal share or counterfactual saving follows from command counts alone.

## Complexity and readiness scope

**Nonblocking O(G×P) flag:** the embedded validator builds a fresh prior-patch
list for each Git call, where G is calls and P is accepted patches in its turn.
Its generic worst case is quadratic; the indexed extraction stores patches once
and does not have that per-call rescan. The index's linear-work description is
valid for extraction, not a blanket bound for its validator. The fixed evidence
replay passes; no frozen script is edited to optimize it.

Documentation/source references were reviewed for the behavior described here.
This is evidence-only qualification, with no changed public workflow or
agent-facing implementation. No Rust build, real-agent call, new controls,
percentage calculation or token retotal is required or performed.
