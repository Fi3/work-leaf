# Closed candidate-screen mechanism evidence

Scope: the three admitted modified workflows in `phases/candidate-screen-01`, after all three closed on 2026-09-07. This is a public-action/source census, not token accounting, a percentage estimate, or a counterfactual explanation of the historical difference. No provider was invoked and no frozen artifact was modified for this audit.

[The indexed evidence](EVIDENCE-CANDIDATE-SCREEN-MECHANISMS.json) supplies 54 exact source paths/SHA-256s, physical 1-based JSONL locators, native call/output IDs and hashes, all top-level public edit/patch submissions, all changed trace handoffs, and their byte spans. `sNN:N` below means physical line N in that indexed source. Native `msg_…` IDs and public app-server item IDs are separate namespaces. Private reasoning bodies are neither inspected for conclusions nor exported. Selected C16 call/output pairs are witnesses, not a claim that every tool call has the same purpose.

## Admission, outcomes, and delivered exposure

The frozen source is the phase-local `infrastructure/evidence/src` copy, not a claim based on the later v5 checkout. Source keys s01–s08 pin `agent.rs`, `bench_experiment.rs`, `bench_candidate_experiment.rs`, `cli.rs`, `orchestrator.rs`, `patch.rs`, `codex.rs`, and `review.rs`. Activation manifests are s16/s31/s44; the prospective protocol is s09. The independent postcapture membership reports s10/s25/s38 bind the exact captured launch/input identities and native sources. Their complete source inventories remain authoritative for the underlying raw captures.

| Workflow / factor | Original terminal result (UTC) | Policy / read / ACK / command-result / fix trace rows | Actual changed delivery |
| --- | --- | --- | --- |
| 001 / C02 requested-repeat-full | Exit 0; 13:14:17.145326; s15 | 8 / 18 / 8 / 8 / 4 | Seven policies, −50 UTF-8 bytes each; four requested-repeat handoffs |
| 002 / C16 unified-diff-preferred | Exit 1; 14:29:25.479789; s30 | 6 / 14 / 4 / 4 / 2 | Six policies, +3 UTF-8 bytes each; no changed read, ACK, result, or fix handoff |
| 003 / C24 review-fix-request-resupply | Exit 0; 13:15:50.880783; s43 | 8 / 19 / 9 / 9 / 4 | Four actual author-fix handoffs; all policy/read/ACK/result rows unchanged |

Each trace also has exactly one activation row. These are delivered-site counts, not response counts or causal outcome comparisons. The original frozen delivery audits s12/s27/s40 retain the title-classifier failure; the separately named title-corrected reports s11/s26/s39 are available with no delivery errors on the same captured inputs. They do not erase the original reports. Membership covers 8/6/8 threads and 61/39/61 accepted public/native user inputs respectively. Workflow 002 still has no final public terminal for user-1 turn `01a07c45-46bf-71f3-b048-656ba9810bf2`; the failed/incomplete outcome is not filtered out.

The exact 002 report states **feature/review stage timed out after 7200s** (s54:44–45). Its last busy snapshots still show user-1 waiting (s53:212–214). There was no replacement run, successful-only selection, or presumed successful final repair.

## C02: the larger requested-repeat representation actually reached users

The source chain is frozen `agent.rs::PromptPolicy` owned spans → `bench_candidate_experiment.rs::forward_candidate_policy` (154–202), plus `orchestrator.rs::render_file_read_response` (2186), `bench_repeat_read_candidate` (2293), and `send_file_read_response` (997; selection at 1118). The factor includes **two policy-coherence sentences**, not a renderer-only intervention. The held tracked snapshots supply full-current bodies; the ordinary untracked/bundle branch and compact automatic refresh contract remain unchanged. `forward_candidate_with` (s03:60–108) records both candidates and selects only the condition's owned spans.

All changed read components have byte-identical surrounding text. Body bounds and held-text SHA-256s are recorded in the index; no file is inferred from its marker alone.

| Trace (s14) | Owned receiver / exact native input | Snapshot classes | Selected-minus-original bytes | Next public action in that thread |
| --- | --- | --- | --- | --- |
| 13 | user-3; s19:198 | Seven unchanged files | +234,786 | Public output at 207 contains a malformed/prose-prefixed attempted edit; later top-level edit at 217 |
| 24 | user-3; s19:264 | Four changed, three unchanged | +244,409 | Top-level exact edit at 269 |
| 31 | user-3; s19:345 | One unchanged test file | +18,747 | Focused test-assertion edit at 350, accepted receipt at 358 |
| 46 | review-user-3; s23:91 | Six changed files | +195,556 | `NO_FINDINGS` at 96 |

The repeated request therefore has a demonstrated **local input-byte expansion**, including an unchanged-only read and a reviewer read; it is not merely an unexposed launch cue. Its subsequent behavior is not uniformly successful: the earlier user-3 exact edits at s19:217/230/243 receive old-block-not-found continuations at 225/238/251 before the full-current handoff at 264. That chronology does not show that full repeats prevented every repair, that compact snapshots are intrinsically sufficient, or that these byte deltas equal provider token charges. Initial bundle reads, actual later native retrievals, and session reuse remain separate possible costs; this census does not retotal them.

## C16: actual representation generation and recovery

The five coherent owned substitutions are frozen `bench_candidate_experiment.rs::policy_pair` (s03:125–149): preferred directive, required representation, remaining exact-edit alternative, manual-write wording, and commit-reason wording. Both formats remain accepted. All six delivered policies select those changes (s29:2/3/4/16/17/22). The +3-byte net policy difference is not the substantive mechanism: it changes the requested output representation.

All three authors first combined the preferred **`@work-leaf patch` directive with an exact-block `*** Begin Patch` body**. Actual typed continuations reject the missing unified headers:

- Visual: s32:236 → 244.
- Slash: s33:188 → 196.
- Completion: s34:198 → 206.

The complete top-level public-message inventory contains ten C16 patch submissions: those three mixed-format attempts and seven unified bodies. Four accepted ACKs remain in the trace. No top-level exact-edit directive occurs in this C16 inventory; this is not a statement about arbitrary quoted code or private reasoning. By contrast, the other two modified workflows' top-level edit/patch messages use exact-edit bodies; they are **not controls**, and that observation does not identify a comparative effect.

The visual agent's actual native tool chain is particularly concrete. Source s32 has SHA-256 `0673699a9af3a9df4c6124e53c05103b63e8a384d7ab4dccde98c26e23149378`; the index retains each selected call's exact `call_id`, argument SHA, output SHA, timestamp and physical call/output pair.

| s32 call → output (or public message → input) | Observed mechanism, not merely the agent's explanation |
| --- | --- |
| 250→254 and 252→256 | Bundle-marker grep attempts omit `--`; both actually fail because the marker is parsed as an option. This is retrieval plumbing, not itself a unified-diff failure. |
| 351→354; 360→363; 376→379; 385→388 | Python reads the issued bundle-0/bundle-5 text, extracts file sections, builds modified strings, and calls `difflib.unified_diff`. The last actual output has the native truncation marker. |
| 417→424, 418→425, 419→426 | Three bundle-parser attempts actually raise `KeyError` for the requested source filenames. |
| 497→500 | The generator runs `rustfmt --edition 2024 --emit stdout` on in-memory modified text. It fails because a generated Rust newline character literal is not escaped. |
| 507→510 | The next generator/formatter returns exit 0, but its displayed diff is truncated; the retained output also visibly concatenates file/hunk header lines. Its script passes `lineterm=''` and prints generated lines without inserting missing separators. |
| 525→528; 534→537; 543→546; 552→555 | The source generation is split into a `src/ui.rs` diff and a remaining-file group. The first remaining-file formatter attempt fails on corrupted string/quote syntax in `tests/terminal_app.rs`; subsequent calls emit the remaining-file diffs successfully. These are real additional commands and outputs, not necessarily additional model responses. |
| 560→567 | Submitted unified patch fails at `src/ui_harness.rs:150`; the delivered continuation includes a changed-snapshot compact refresh with concurrent source edits. |
| 583→586; 720→723; 729→732 | Subsequent scripts reconstruct current text from issued bundles plus hard-coded replacements corresponding to refreshed source, then generate diffs. The 720 script explicitly omits test refresh reconstruction and reports original test line counts; its `OK` output is not proof of full reconstruction correctness. The 729 output is a source-files-only diff. |
| 650→652; 667→670 | A long heredoc/output-redirection attempt fails with read-only-filesystem temporary-file creation; a later `/dev/shm` probe succeeds. Do not claim the entire workflow performed no temporary writes merely because many generators were in memory. |
| 675→683 | Actual `read --force` still receives the ordinary unchanged-digest repeat response. C16 does not activate the C02 full-repeat representation. |
| 745→748 | A reconstruction attempt raises `Exception: missing ui current review`. |
| 776→784 | Another unified patch fails in `tests/ui_harness.rs:272`, but the continuation explicitly says touched files still match the latest snapshot. This is distinct from the earlier concurrent-source refresh. |
| 791→799; 812→815 | A narrower test reread remains unchanged-status-only; the next reconstruction script actually fails to parse a Rust `\u{…}` string as a Python Unicode escape. |
| 821→824; 828→831; 838→846 | An intermediate raw-text command returns empty output; an explicit hunk-count probe reports old/new/added lines. The subsequent complete patch finally receives the accepted receipt for all five files. |
| 851→859; 864 | The focused `cargo test --test ui_harness` returns status 101: 32 pass, one own visual-line-selection assertion fails at `tests/ui_harness.rs:299`. The agent acknowledges the own-test issue publicly, but the original workflow times out without a verified completed next repair. |

The completion author independently has a shorter instance of the same representation/refresh interaction: s34:216→219 generates a unified diff; the submission at 224 receives a changed-snapshot conflict at 232; generators 240→243 and 249→252 precede accepted submission 257→265. A later actual reviewer repair uses another generator 338→341, then patch 346→354. The slash author instead reaches an accepted unified patch at s33:238→246 after its first mixed-format rejection. The adverse long visual trajectory must not be generalized to every feature.

`patch.rs::GitPatcher::apply_with_locks` (s06:90–147) runs `git apply --recount --check` before applying and committing. The failed forward checks above do **not** establish partially applied earlier hunks. In particular, the visual agent's public suggestion after 784 that source hunks had been accepted “far enough” is not an authoritative write receipt. The compact-refresh and unchanged-snapshot messages establish different recovery inputs; they do not identify an entire response generation's cost.

Supported mechanism: this admitted preference exposed incompatible directive/body combinations and real extra output-representation work, with concrete syntax, truncation, source-refresh and hunk-context obstacles. **Unsupported**: that every native call was unnecessary, that exact-edit preference would have avoided all of them, that the whole timeout was caused by C16, or that this trajectory explains a numerical share of the historical WL/Direct difference. The test defect, long source reconstruction, concurrent work, and renderer/native-tool boundaries interact.

## C24: original-request resupply is delivered, not a fabricated fix join

Frozen `cli.rs::CommandChat::launch_prepared_agent_streaming_with_ids` retains only a successful prepared launch in the shared owner registry (s04:980–1012). `bench_review_fix_prompt` (1019–1057) appends the exact stored request to the existing findings prompt and records the owned zero-length baseline insertion. `forward_candidate_with` selects it only for this condition. This is an author fix handoff, not a new session, Git-only reconstruction, or modification of the reviewer finding itself.

| Trace (s42) | Receiver / exact native input | Added bytes | Actual public continuation |
| --- | --- | --- | --- |
| 19 | user-2; s46:286 | 204 request + 52 framing = 256 | `done` at 291, with no new edit. The existing wrapper contains progress prose followed by `NO_FINDINGS`, then fix instructions: this is the known clean-marker detour, not a demonstrated code repair caused by resupply. |
| 32 | user-3; s47:374 | 149 + 52 = 201 | Read at 379, bundle input 387, concrete terminal-cache repair edit 417, ACK 425, focused command 428/result 436, done 439. |
| 41 | user-1; s45:245 | 189 + 52 = 241 | Forced read 250/input 258, row-anchoring/arrow-routing edit 324, ACK 332, focused command 335/result 343, done 346. |
| 46 | user-1; s45:354 | 189 + 52 = 241 | Read 357/input 365, row-offset rendering repair edit 379, ACK 387, command 390/result 398, done 401. |

The two identical visual requests are distinct typed deliveries, not one global-text-unique event. Their exact request spans, owner IDs, launch-source tag and native item IDs are in the index. T32 follows a concrete duplicate-completion-cache finding; T41 follows concrete review of visual anchoring/routing; T46 follows the review's remaining row-offset complexity finding. The last item's **repair purpose** does not independently endorse every complexity claim in reviewer prose. These actual findings already exist in the unchanged baseline prompt; successful downstream edits cannot be credited uniquely to the appended request.

## Checks and interpretation boundary

All 54 indexed source hashes were checked before derivation and again at the endpoint. Every inspected v4 candidate has valid UTF-8 component boundaries and byte-identical non-owned gaps/prefix/suffix; changed status and selected byte delta reconcile. Every changed C02 snapshot body length and its independently computed SHA-256 is retained. Complete per-run delivery/native membership is reused from the separately pinned postcapture audits, not reconstructed from apparent text chronology or assistant claims. Source line references into tool commands are command-text lines nested inside the stated physical native record.

The local observations support three different dispositions: C02 genuinely expands requested-repeat input while preserving the automatic-refresh interaction; C16 genuinely changes representation behavior and exposes specific generation/recovery failures; C24 genuinely resupplies immutable requests at four actual fix boundaries, including a non-code clean-marker detour. None is an outcome counterfactual or a token saving estimate. Native tool call counts are not model-response counts; one output retained in later context is not evidence of re-execution. Complete accounting, incomplete-tail treatment, final quality and comparisons remain separate parent-owned analyses. No provider, runtime, frozen helper, or original audit was changed for this evidence-only task.
