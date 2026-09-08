# C08 automatic changed-refresh representation

Status: **source-bound prospective design only**. No implementation, provider admission or new history census belongs to this document. The sole proposed condition is `automatic-changed-refresh-full`; existing controls remain reused, not replenished.

## Question and observed scope

The [complete H recovery evidence](EVIDENCE-H-REFRESH-RECOVERY.json) and its [independent review](EVIDENCE-H-REFRESH-RECOVERY-REVIEW.md) establish14 delivered changed-file diffs, nine recovery chains, and five subsequent mediated read handoffs in four chains. None exercises the automatic48-KiB diff-omission or8-KiB untracked-full-text cap. Eleven diagnostics concern missing old blocks; three concern duplicate file headers. None of these counts establishes net savings or makes every rejection a concurrency failure.

The proposed question is narrower than disabling recovery: **does supplying full current text instead of an available compact automatic diff change reconstruction, rereading and repair work enough to offset its larger retained input?** Full text can reduce reconstruction/retrieval work; it can also increase first and repeated input charges, compaction, provider-request failures or distraction. A diff is not necessarily shorter in every case. Exact local bytes and later actions are mediators; neither their direction nor an ACK establishes whole-workflow saving.

## Existing owner and call chain

At the inspected source cutoff:

1. `src/orchestrator.rs` processes the ordinary `AgentDirective::Patch` or `Edit` through `GitPatcher::apply` / `apply_edit` (around798/883). The actual `PatchError::Conflict { files, diagnostic }` selects recovery. Already-applied, no-file and same-snapshot branches remain separate.
2. `patch_conflict_refresh_response` (1503) calls the existing `read_requested_files` using the existing locks/context-bundle service. Its normal comparison against `FileReadTracker::snapshot_for` determines whether a refresh exists. Read failures and missing snapshots keep their ordinary response paths.
3. `render_patch_conflict_prompt` / `render_structured_edit_conflict_prompt` (3144/3162) own the exact rejection/files/diagnostic and format-guidance wrapper. `append_file_refresh_response` (3270) appends `render_file_refresh_response` (2747).
4. The renderer owns each file header, current/prior digest, status and body. It calls `render_snapshot_diff` once for a changed tracked snapshot. That existing helper owns its temporary files, Git invocation and counter; no second diff or filesystem read is needed.
5. Both conflict branches **record the selected snapshots before the existing send** (around845/926), then use `send_agent_streaming_interruptible` and the normal followup/rejected-event handling. This ordering is different from requested reads, which record after send. It is a non-target invariant.

The architecture's private benchmark boundary permits a feature-gated renderer transformation, not changes to the public provider/controller/patch APIs or default behavior. A dedicated new private manifest schema/condition must be declared before implementation; old v1–v6 manifests, evidence and interventions are not relabeled or silently extended.

## Smallest transformation

Operate per file section, only when all of these source-owned facts hold:

- The existing automatic-recovery path actually selects this snapshot.
- A prior snapshot exists and its held text differs from current held text.
- The ordinary renderer produces a **nonempty diff within its existing48-KiB limit**.

Replace only that renderer-owned diff-body interval with:

```text
current full text:
<exact held current snapshot text>
```

The formatter may append its ordinary final newline when needed; the recorded body range excludes that formatter newline. Preserve the filename, current/prior digest strings, changed status, order and every other section. A file body containing marker-like text is data: ranges come from construction, never a search through copied source.

Only the minimal **response-local coherence spans** accompany the body replacement, and only when at least one section is replaced:

| Owned baseline span | Full-current candidate |
| --- | --- |
| `Rebase your patch against the compact file refresh below.` | `Rebase your patch against the file refresh below.` |
| `Rebase your exact edit blocks against the compact file refresh below.` | `Rebase your exact edit blocks against the file refresh below.` |
| The refresh intro beginning `This is a compact refresh, not a patch to submit.` and ending `Repeated full-text refreshes are intentionally avoided to keep the session compact.` | `This is a file refresh, not a patch to submit. Sections marked current full text contain the complete current file; all other sections keep their stated snapshot or diagnostic meaning.` |

The two wrapper alternatives correspond to the actual patch/edit path; they are not both inserted. Actual diagnostics, file lists, write-format guidance and any quoted strings inside those diagnostics remain byte-identical. Mixed changed/unchanged/untracked/failure responses remain truthful.

**No launch-policy change is proposed.** `src/agent.rs:485` says to treat compact refreshes and repeated-read digests as authoritative; it does not promise that every refresh is compact. The explicit full-current response identifies the other representation at its actual occurrence. Requested-read policy/syntax, relevance/limited-reread guidance and the `--force` contract remain intact. If implementation review establishes another truly contradictory owned instruction, it must be explicitly scoped before implementation rather than silently changing launch behavior.

## Eligibility, size limits and natural history

Keep the existing omitted-diff and unavailable-diff responses unchanged—even when current full text would fit. Keep untracked rendering, including its8-KiB cap, unchanged. An empty diff render remains identity and is reported separately; the current helper can return an empty string without exposing a usable diff, so do not invent an error cause or treat it as a full-diff witness. Same-snapshot, already-applied, no-file and read-failure-only responses remain identity.

**Do not apply the untracked8-KiB cap to tracked current text, or invent a new current-file threshold.** H's eligible files often exceed8KiB. A new cutoff would test a size-restricted policy, selectively remove exactly the larger reconstruction cases, and leave a different substantive question. The existing normal diff branch is a prospective exposure gate, not a workflow-inclusion gate: retain every workflow and every ineligible occurrence, with the actual reason. Never analyze only the exposed survivors or choose a threshold after seeing outcomes.

A large full candidate may exceed memory, trace-publication or provider context/request limits. Declare practical admission/resource bounds before generation; preserve any resulting failure without clipping, splitting, bundling, returning to delta, or substituting another observation. If resource safety requires a new full-size gate, it is a **different explicitly restricted factor**, not a hidden fallback under this design.

Natural workflows receive no forced stale edits, cross-agent scheduling barriers, sleeps, extra read requests, deliberately malformed bodies or manufactured conflicts. Three parser-error refreshes in H demonstrate why a changed snapshot must not be used to rename every rejection “concurrency conflict.” Zero automatic-refresh exposure remains a retained result.

## Minimal implementation and evidence seam

Keep ordinary rendering/diff construction once. While the renderer still has the same previous/current snapshots, construct the baseline and candidate plus ordered UTF-8 spans. Then preserve the existing `record_snapshots` point, and perform the private evidence/selection hook immediately before the existing send. Evidence failure blocks delivery; it must not move, undo or add snapshot advancement relative to that ordinary send attempt.

A small private renderer result can carry the candidate and metadata; the existing private owned-range validation/selection pattern in `src/bench_candidate_experiment.rs::forward_candidate` is a reference, not permission to reuse v4 identities. No public DTO or model-authored offset is needed.

Record both complete candidates and selected identity symmetrically in each active automatic-refresh occurrence, including identity-only occurrences. Required metadata is limited to actual patch/edit kind, source agent, original diagnostic/files, ordered snapshot paths/classes and ordinary FNV64 digest/byte metadata, normal diff disposition, exact component/body ranges and selected-candidate tag. Offline SHA-256 of exact spans and source artifacts supplies cryptographic identity; add no runtime crypto dependency solely for labels. Unchanged prefix/suffix equality must be mechanically checkable.

Candidate construction/range validation is linear in held bytes and sections, with the original diff work unchanged. Do not rescan all files for every section or append each full body repeatedly. Additional serialization can affect wall time and memory; keeping the ordinary wait/grace/scheduling policy unchanged is not a claim of identical elapsed execution. No extra model turn, native tool, bundle allocation, snapshot read or normal Git-diff invocation is part of the factor.

## Difference from the tested C02 contract

[C02's frozen protocol](PROTOCOL-CANDIDATE-SCREEN.md) changes **requested** reads of tracked changed **and unchanged** files to current full text and changes two launch-policy coherence sentences. It explicitly preserves automatic refresh. The current requested-read hook is at `src/orchestrator.rs:1140`, site `requested-repeat-read`.

C08 here acts only after an **actual rejection with an available automatic changed diff**. It leaves unchanged sections and all requested reads alone, and has no upfront launch-policy treatment. C02's closed screen does not identify this rejection-conditioned reconstruction/repair effect; nor does the existence of a distinct seam predict a positive result. C08's unexposed omission-limit explanation remains separate from its exposed delta representation.

## Qualification before readiness

Provider-free, test-first coverage must prove both real patch/edit recovery paths; exact default/inactive and legacy identity; mixed-section preservation; current text above8KiB; normal48-KiB boundary/unavailable/empty diff identity; untracked cap identity; UTF-8 and marker-like source bodies; unchanged read/diff/bundle calls and snapshot ordering; exact owned spans; and failure-before-send evidence behavior. Deterministic stale-snapshot fixtures belong to these tests, not to measured natural workflows.

The minimum actual-agent evidence is **one real same-author chain**: a normal rejected edit/patch selects an eligible automatic diff → the exact full-current candidate is accepted by the configured backend and appears in its explicit-turn native input → the agent repairs through the normal apply/ACK path → the normal focused check and completion are observed. Retain the real diagnostic reason, held text/digests, non-target bytes, ordinary timing/settings and every failure. This proves delivery/usable recovery, not net savings.

Use at most one separately frozen and admitted, bounded qualification scenario initially, with a fixed outer-call/time budget. A declared fixture may establish an older same-author snapshot and a subsequent genuinely accepted edit through the normal host path, so an actual stale submission exercises ordinary rejection and recovery. The real agent must receive the real rendered result, repair it and execute its ordinary check. The fixture stimulus is verification, not natural exposure or causal benchmark evidence; the runtime must not recognize its filenames, commands or responses. Do not fake rejection, acceptance, model output or validation. If the intended chain is unexposed or fails, retain that result without an automatic retry. Measured natural workflows retain the no-manufactured-conflicts rule above. No new control or Direct run is a qualification prerequisite.

This design admits nothing. [Operator policy](../../docs/benchmark-operator-policy.md) still governs subscription-only generation, unchanged global configuration, fixed attempts, retained failures and the prohibition on automatic baseline replenishment. Implemented agent-facing behavior would require updated architecture documentation, repository gates, independent review and the actual-agent receipt before readiness.

## Source cutoff

Inspected repository HEAD: `b8e928153674549e4877b5ae6b34546d8541d566`. These are design-source identities, not a future executable admission:

| Source | SHA-256 |
| --- | --- |
| `src/orchestrator.rs` | `e0ef8167bd1f51cd99d56ae58835ba5d6951549c9dc9a2ccdabf9b2e568ff0f7` |
| `src/agent.rs` | `5850fc7dd63a97249888dc8eeff7c525f749e4f2ae6e35c372cb7fc8d97ee800` |
| `src/bench_candidate_experiment.rs` | `e327d28fb05ceaa8f9c9465a78fc61069eddb56faa767b4909f29f83684d8b94` |
| `docs/architecture.md` | `20c1f2bd4cdec7f0ba4fd613eecc2fd4156554f4b4d46d53bdfbb4f6773df921` |
| `docs/benchmark-operator-policy.md` | `fcea97af92026b7374e1b72f072e7226fb4ca65dea41b22235a9b5f9cd4bda25` |
| `PROTOCOL-CANDIDATE-SCREEN.md` | `7b1865f2d7a21af6075a1b40148beb46120f54ecc1b815857f5aa8a0e3f50fdb` |
| `EVIDENCE-H-REFRESH-RECOVERY.json` | `6a83a13322b10afe25cb6cf3ba24248859133ba18f189c51b61f8c7d9537568f` |
| `EVIDENCE-H-REFRESH-RECOVERY-REVIEW.md` | `d48ddf698a9b76aa093bde72cc1772f0d9312a028fab2244186587a309322144` |

Only this new design file is written. No implementation, tests, historical evidence, frozen protocol, provider configuration or observation is changed; no provider, test, extractor, accounting or further historical census is run.
