# H author Git inspection versus ordinary applied-patch receipts

**The 252 Direct author Git calls are not postcommit history inspection duplicated by WL receipts.** Every call is a `git status` or `git diff` variant, and every output precedes its current author turn's completion and the driver's subsequent host-commit receipt. Ordinary WL ACKs do not supply most of the facts those commands inspect. The observed difference therefore remains coupled to edit/publication cadence and work selection, not established as a fact-rich receipt replacement.

The [source-bound index](EVIDENCE-H-GIT-RECEIPT-FACTS.json) covers all 252 previously counted leading-Git native calls across the accepted H six Direct workflows, all 18 authors and 39 author cycles (18 initial, 21 fix). It separately joins all 53 normal-WL ACK requests to typed successful replies and identical completed public user items. No provider, token total, quality filter, runtime change or fresh baseline is involved.

## What was actually inspected

These are native `exec_command` calls whose first parsed shell word is `git`, matched to their exact same-thread/same-turn outputs. Counts do not include reviewer or linearizer Git work, arbitrary Git inside another program, or model responses.

| Observed command/fact | Calls | Supplied by ordinary WL ACK? |
| --- | ---: | --- |
| `git status --short`: worktree/index status, including any untracked entries | 107 | No whole-tree status or cleanliness promise. The observed nonempty outputs contain unstaged ` M` entries. |
| Scoped `git diff -- <paths>`: actual accumulated uncommitted before/after code, sometimes through `sed` | 39 | No diff body. One returned diff is truncated and remains marked incomplete. |
| `git diff --stat`: changed-file list and change sizes | 52 | File-list overlap only; no stat/numstat. |
| `git diff --check`: whitespace-error check on the current uncommitted diff | 40 | No such result. All 40 recorded shell exits are zero and their outputs empty. Patch applicability and this check are different facts. |
| `git diff --name-only`: current uncommitted changed paths | 11 | Partial overlap: ACK lists its accepted group's paths, not the current turn's accumulated uncommitted delta. |
| `git diff --name-only -- <prose paths> ... || true`: restricted documentation-scope inspection | 2 | No negative statement that every other/prose path is unchanged. Both commands mask underlying Git failure; empty output is not silently promoted into an independently successful Git check. |
| `git diff --stat && git diff -- <paths> \| sed ...`: combined statistics/body | 1 | File-list overlap only; neither statistics nor body is supplied. |

All 252 reported shell exits are zero. A pipeline's exit and the two `|| true` commands do not independently certify each preceding subprocess. Every exact compound string and original output hash remains retained. There are 30 distinct command strings, but identical spelling is not evidence of identical source state or redundancy.

## The host-commit boundary is decisive

All 39 saved author prompts tell the model to leave changes uncommitted for the driver. Each prompt matches the corresponding native user item after terminal-newline-only normalization, and its saved thread file matches the established author identity. Every cycle's native `task_complete` precedes an actual retained `patch-accepted` or `review-fix-complete ... result=committed` timeline record. All 252 Git outputs occur before those current-cycle completion/commit boundaries.

| Position relative to accepted patches in the same author turn | Initial calls | Fix calls | Total |
| --- | ---: | ---: | ---: |
| Before any accepted own-turn patch | 16 | 15 | 31 |
| After at least one accepted own-turn patch | 137 | 84 | 221 |

The 31 before-edit calls are all clean `git status --short` observations. The other 76 status calls contain unstaged changes. These are entry-state and accumulated-work observations, not proof that the model redundantly queried a commit it had just been told about. A fix turn can follow an earlier host commit while still preceding the host commit for its **current** edits; the index preserves that distinction.

Accepted patch output lines precede the calls classified after them; classification does not substitute patch issuance for completed application. The accepted-patch table is stored once per turn and referenced by count/last locator. No intervening test or formatter is assumed inert, and the report supplies no byte-identical whole-tree claim.

## Deterministic first cycle

The first cycle is the first author in the established H order: point7-exact-direct visual, thread `01a04842-cbbf-7952-97d1-98fc8afc784a`, initial turn `01a04842-cc48-74c0-88e4-2e90d0bac62c`. Native locators resolve through index `authors[0].native_source`.

- N15→21: entry `git status --short` is clean, before any accepted own-turn patch.
- N300→303: scoped diff inspects modified UI/terminal/harness code after accepted edits. Its output is truncated; complete body coverage is not claimed.
- N301→305 and N345→354: status names modified terminal/UI/harness source and harness tests.
- N346→355: `git diff --name-only` returns those four paths.
- N423→428: later status also contains `tests/terminal_app.rs`, after additional accepted work.
- N424→429: diff statistics cover the five-file current delta; N427→431 reports a successful empty whitespace check.
- Native completion N470 is `2026-08-28T12:22:29.008Z`. The actual `patch-accepted` record, timeline line4, follows at Unix ns `1787919749881588396`, naming commit `f3033bbadf3a51b37a4888ece879f1a92ba62ec7`.

This cycle alone demonstrates why the inspection cannot be called duplicate postcommit receipt information. The complete 252-call extension confirms the same current-turn ordering rather than selecting only this example.

## What the actual 53 WL ACKs contain

All 53 captured inputs have exactly the same fixed body after their `files:` line. They provide:

- `work-leaf patch applied`;
- the accepted group's file list;
- the fact that the orchestrator already saved a provisional commit;
- instructions not to resend/restate the patch, followed by the ordinary validation, ownership, blocker and done guidance.

They contain no commit SHA or subject, no history, no diff body/statistics, no whole-worktree status, no untracked-file inventory, no whitespace-check result and no claim that final tests passed. A `PatchOutcome` or host event containing a commit hash is not automatically a model-delivered ACK field.

For example, H001 ACK C30 has string RPC ID `29`, successful reply S3612 and identical completed public user item S3616. Its author thread is `01a04ebc-3891-7172-bbb2-3bbdab8c9de7`; accepted turn is `01a04ec0-5c10-7ce0-9745-ea593070f418`. Its file line is `files: src/terminal_app.rs, tests/terminal_app.rs`; input SHA-256 is `3ee791f86e8a832e8fe221933c71d9d5c35733f96cbab8b302f5aa504b0e75a0`. All remaining ACKs receive the same exact source/reply/public-item checks.

The historical owning chain is `handle_agent_directives_streaming → GitPatcher::apply_edit/apply → apply_edit_with_locks/apply_with_locks → git_add/git_commit → PatchOutcome`, then applied-path collection and `send_agent_streaming_interruptible(render_patch_applied_prompt(...))`. [The retained renderer](../efficiency-raw-token-pilot-20260906T185328Z/source/src/orchestrator.rs:2638) emits the fields listed above; the ordinary [patch owner](../efficiency-raw-token-pilot-20260906T185328Z/source/src/patch.rs:77) commits before returning success. Both source files match actual H commit `5b1d1ef9590850faed26052f909ddff7ff8f127d` byte for byte.

The [historical restricted-read policy](../efficiency-raw-token-pilot-20260906T185328Z/source/src/agent.rs:227) also routes repository file access through the orchestrator, unlike Direct's direct workspace access. It does not justify inventing an explicit Git-specific prohibition, but it prevents attributing native inspection differences to receipt information alone. C06/C26 access-route policy remains a separate interaction.

## C17 disposition and the smallest boundary

The simple explanation **“WL supplies the same postcommit Git facts, so Direct needlessly reconstructs them” is unsupported and, for most inspected fields and every current-cycle commit boundary, contradicted by the actual payload/timing.** The file-list overlap is real but partial: current uncommitted accumulated paths and one accepted group's paths differ in both scope and time.

An ACK-file-list-only factor is technically narrow and has delivered-text exposure in all 53 ACKs. It would preserve the success/commit fact, all policy text, publication, routing and ACK count. This census, however, shows no specific avoided model action attributable to that field. Removing the file list would test that list's usefulness/input cost, not explain the whole 252-call difference. There is no evidence-based reason here to prioritize a fresh receipt-only run.

The remaining package is **who holds uncommitted edits, when they are accepted/committed, and what continuation directs the author to do next**. Committing every Direct patch earlier changes publication/granularity (C14/C17); it also changes what an ordinary worktree-versus-index `git diff` returns. Forcing a postcommit empty diff would not recreate the original inspected information. Adding full diffs/statistics to WL ACKs instead changes supplied content, not just custody or timing. Neither is an isolated explanation obtained by counting commands.

Keep C17's mechanical application/publication and C14/C15's work-unit/test-feedback boundaries distinct. Existing checks are not labeled waste, equivalent-quality work is not disputed, and no net token contribution or new baseline requirement follows from this evidence.

## Provenance and limits

The index records all four actual Direct driver commits, without replacing them with the current common driver. The old commit objects could not be resolved in the main checkout during this bounded source check, and no standalone archived driver script was found in the three retained study trees. Historical ordering instead rests on each actual saved prompt, thread/status file, native task boundary and host timeline receipt. The current common driver's `run_feature_cycle → run_direct_agent/resume → commit_if_changed` is consistent with this ordering but is not substituted as proof of identical historical source.

All 18 native and six report SHA pins match the established H evidence. Prompt/thread/status/timeline bytes are freshly hashed retained evidence with explicit joins, not retroactively called original frozen-manifest entries. All 166 recorded source endpoints rehash unchanged, and normal-WL agent/orchestrator/patch source matches the actual H Git revision. The embedded provider-free validator passes all 252 command/output identities, all 39 cycle boundaries, prior per-thread Git counts and all 53 typed ACK/public-item joins. Index SHA-256: `a0eea9a869a22c2acdd4216e88041946aa85fb00ee5f721ee2ba03766a0859b5`.

This is evidence-only documentation. No agent-facing code or public workflow is affected; no real-agent call or build is required or performed.
