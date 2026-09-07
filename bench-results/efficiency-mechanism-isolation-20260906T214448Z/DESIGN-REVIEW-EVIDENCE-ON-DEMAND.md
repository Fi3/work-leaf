# C21: exact reviewer evidence through native on-demand reads

Status: source-grounded implementation design under the user's private benchmark-only
authority. This document does not admit generation, a new control or a replacement.
The selected factor is
**eager inline delivery versus native on-demand access to the same immutable reviewer
source context**. The opaque review artifact is not registered as an ordinary mediated
project read or project context bundle. This is not Git reconstruction, evidence removal,
a different review standard or a claimed cause of historical savings. The advertised
native access route and its short instructions belong to this representation factor.

## Owned boundary and retained evidence

The benchmark path is `WorkLeafController::start_review_worker` →
`CommandChat::review_commit_streaming_with_ids` in `src/cli.rs:1245`.
Lines 1252–1255 obtain the existing backend session snapshot and call
`src/review.rs::render_review_source_context:177`. Lines 1258–1266 insert the returned
string into the initial review request, before the existing new/reused reviewer split at
1267. This single renderer-owned interpolation is the proposed treatment boundary.
Collect its UTF-8 range during construction, never by searching copied message contents.

`render_review_source_context:177–208` includes commit metadata, review scope, the Git
commit log and **every recorded session message**, with role and order. This is the
current rendered public session context, including existing `trim_end` behavior; it is
not the complete native tool/private-reasoning history. Render it once from the same
existing snapshot in every admitted arm. Preserve those exact resulting bytes in the
archive; do not reconstruct them later from Git, changed session state or a summary.
Missing/empty session state retains the ordinary rendered unavailable/empty marker.

The public `ReviewCoordinator` type exposes `review_latest_agent_commits`, which reaches
its private `review_commit` at `src/review.rs:256` through a separate integration path.
The experiment scopes the private hook to the actual `CommandChat` benchmark path; it
does not claim coverage of that coordinator or change its API.

## Treatment contract

- Baseline candidate: the complete ordinary review prompt, byte-identical.
- Selected candidate: replace only the owned `source_context` body with a short, truthful
  receipt for an exact immutable Work Leaf review-evidence payload, identifying its
  supplied absolute path, byte length and evidence identity. Do not frame it as a project
  file or an ordinary `ContextBundleStore` bundle.
- The reference says it contains the **same complete commit/log/recorded-chat evidence**
  and remains available for the review. It explicitly permits native read-only inspection
  of this supplied path and states that the opaque artifact is not served by
  `@work-leaf read`. Consult relevant archived evidence before declaring required evidence
  missing. Neither a read nor full consumption is forced; the reference prescribes no
  conclusion. Its necessary access guidance and direct prompt-byte difference are recorded.
- Keep the outer target Agent-ID, latest commit, feature, reason and full review scope,
  all verdict/verification/docs rules, existing reads/tools/locks and agent profile.
  Keep the existing author-fix prompt at `cli.rs:1323` and reviewer recheck at 1354
  unchanged. Do not reinject the archive or launch policy on ordinary rechecks.
- Reviewer reuse remains reuse. Each later ordinary review launch/reuse handoff gets
  its own exact context snapshot/reference; earlier references remain readable. Never
  overwrite a previous review's evidence with a more recent snapshot.

Construct both full prompt candidates and materialize the archive symmetrically for
any newly admitted schema's selected/nonselected conditions. There is no new control
run requirement. Default builds, inactive feature builds and every older manifest
version must perform no archive allocation or prompt change. The source-owned UTF-8
range is exactly the `source_context` interpolation, excluding the existing preceding
heading and following review instructions. Capture its start/end during construction;
do not locate it by searching copied context. A first reviewer launch still receives its
ordinary backend policy, and reused reviewers still receive the ordinary raw handoff.

## Private storage and unchanged read routes

`src/orchestrator.rs::ContextBundleStore::write:271` consumes the ordinary project-read
counter and wraps `FileSnapshot` values in file markers. Calling it for reviewer context
would perturb later `bundle-N.md` paths and misdescribe the payload. Do not use that
writer or its counter for this factor.

Even a separately numbered file in the same bundle tree is insufficient:

- `bench-three-features::archive_observation_bundles:297` archives the whole ordinary
  context-bundle root.
- `bench-observer/src/lib.rs::archive_context_bundles:6696` copies every regular file;
  `parse_context_bundle_snapshots:6742` requires project-file framing and snapshots.
- `load_bundle_observations:4838` treats malformed/non-project bundle payloads as errors;
  `MechanismAnalyzer` also reports unannounced archived bundles at 3321. The ordinary
  announcement classifier at 2402 expects a `work-leaf file text` envelope and project
  manifest vocabulary. Faking those markers would contaminate project-read evidence.
- The existing v3 read census indexes issued project bundle paths from its own read
  traces (`audit_read_mechanism.py:274–297`). It cannot be relabeled as a census of
  opaque reviewer evidence or treated as proof of its retrieval.

The selected storage boundary is a separate **benchmark-only opaque-review-evidence namespace**,
outside the ordinary project context-bundle archive tree, with its own explicit
manifest and independent sequence. A future private manifest schema must admit its
exact canonical root; no new general-purpose file permission is implied. Persist the
exact payload with create-new/no-follow semantics and a retained schema tag such as
`review-source-context`, source agent, reviewer, target hash, local delivery sequence,
payload byte length/hash and source-owned prompt range. Mark snapshot classifications
as inapplicable, not an empty or invented project-file snapshot list.

There is **no opaque-path lookup in the orchestrator**. Leave `parse_read_request`,
`read_requested_files`, `send_file_read_response`, `ContextBundleStore`, `FileReadTracker`
and automatic conflict refresh unchanged. The standard bundle counter, allocation,
framing, archive and parser retain their existing semantics.

The current contract does not promise mediated reads of arbitrary supplied artifacts.
`docs/architecture.md:616–622` describes the ordinary large project-read bundle type;
`ContextBundleStore::owns_bundle_path` admits only that store's own `bundle-*.md` paths.
The native-read exception in `src/agent.rs:277` permits explicitly supplied temporary
context-bundle files; it is not a general external-file permission. The new receipt
identifies this particular temporary read-only review context, its opaque type and
exact native-only access permission. That narrow access instruction belongs to the
factor. It grants no repository reads, arbitrary external paths, writes, new tools or
different provider sandbox settings. Actual native accessibility must be verified in
the configured backend, not inferred solely from a `readOnly` label.

If a reviewer nevertheless requests `@work-leaf read project.rs /exact/opaque/path`,
the existing `read_requested_files` reads the ordinary project path and retains
`FileAccessError::PathEscapesRoot` for the absolute opaque path, then renders their
ordinary combined result in one handoff. The opaque body is never embedded there.
An opaque-only request retains the ordinary unavailable-path result. These route
mistakes, subsequent recovery and any additional responses remain observations;
there is no special retry, invented successful retrieval or hidden fallback. Ordinary
project and genuine context-bundle paths retain their exact previous behavior.

The opaque payload is inspected through a native read command and its captured native
tool output, not through a host-generated file-text envelope. Its separate typed archive
manifest is consumed only by supplementary exposure analysis. The existing observer's
`resolve_bundles` indexes only announced/archived ordinary project bundle paths, so the
new external namespace does not become a fake project bundle or require observer changes.
The ordinary observer, raw streams and token-accounting rules remain unchanged; their
reports do not automatically prove opaque retrieval or complete native input attribution.
Existing text-based review counters need not recognize the new reference, so a missing
legacy review marker is not evidence that no review occurred. The separately typed
archive/delivery audit owns that exposure claim; it does not suppress observer errors.

The exact bytes live in a private admitted per-run root outside both the driver's
disposable checkout and ordinary context-bundle tree. The phase retains that root after
success, failure and normal teardown; it is not subject to `ContextBundleStore` cleanup.
Use independent create-new identities and read-only payload permissions, reject symlink
or aliased storage roots, and never overwrite an earlier review. Keep a typed manifest
entry with source/reviewer/commit/sequence, byte length and the explicitly labeled FNV64
runtime checksum. The separately pinned offline audit supplies SHA-256 identity and
exact byte equality against the trace-held source-context span; FNV64 alone is not
cryptographic proof or sufficient audit acceptance. No new crypto dependency or external
hashing process belongs to the runtime factor.
No silent truncation, Git-only fallback or deletion after first read is permitted.
Archive creation/identity/evidence-write failure prevents the affected delivery. Endpoint
and actual-read hashes detect unsupported mutation; ordinary read-only permissions do
not establish hostile-process containment or continuous immutability.

Only private benchmark storage and the owned review-prompt construction need runtime
implementation. A focused architecture paragraph must describe this separate evidence
type before admission. No public provider API, orchestrator event/DTO, parser, archive
API or observer extension is part of the selected design.

## Rejected broader-route alternatives

Registering the opaque artifact as a mediated source would require a new mixed-response
contract, not merely a new filename. An opaque-first envelope hides the normal project
bundle announcement from `MechanismAnalyzer::observe_prompt_at`; project-first framing
can expose copied chat/file markers to its substring and snapshot parsers. Its unconditional
omitted-refresh parsing and review/patch/linearizer markers also require typed isolation.
`analyze_app_server` subsequently runs `extract_agent_id` over raw prompts, so copied
author metadata in a mediated opaque body could overwrite reviewer ownership.

A safe broader design would require source-bound typed section ranges at observer
ingestion, exact ordinary project slices, full actual prompt counting once and opaque
ownership/classification separate from project snapshots. That is a rejected extension
for this first factor, not a blocker for the selected native-only artifact. Base64/escaped
payloads, fake project snapshot markers, changing ordinary bundle counters, suppressing
observer errors or forcing a second mediated turn are not substitutes for that contract.

## RED-first gates for a future implementation

1. Whole ordinary review prompts must match the prepatch renderer for new and reused
   reviewers, missing/empty/populated sessions, arbitrary roles, Unicode and embedded
   fake footer/file/context markers. Default and all older schemas retain identity.
2. Selected/nonselected new-schema tests require both complete candidates and an exact
   archive of the same held context. Only the owned body may differ; prefix/suffix and
   all unrelated launch/fix/recheck prompts are byte-identical. Exercise two reviews of
   one agent with mutation of later session/Git state: the earlier archive stays exact.
3. Interleave ordinary small/large reads, repeats, refreshes and review handoffs. Ordinary
   bundle paths/counters/content and tracker contents/clearing must equal the baseline
   reference, including failure paths. Test opaque-only and mixed project/opaque
   `@work-leaf read` attempts against the unchanged renderer: preserve its unavailable
   path and normal project result, without disclosing opaque bytes. No duplicate
   rendering, filesystem reread, diff work or extra host-imposed turn.
4. Exercise failed/create-existing archive, symlink/path alias, missing or altered bytes,
   wrong run/agent/reviewer, failed trace write and any predeclared size limit. The
   affected publication or audit fails closed without inventing successful retrieval,
   removing evidence or silently falling back. Earlier snapshots survive later session
   mutation, reviewer reuse and ordinary orchestrator/driver cleanup.
5. Archive tests retain arbitrary opaque bytes, including valid-looking project bundle
   and Agent-ID markers, without parsing project snapshots. The external opaque namespace
   does not enter the ordinary archive manifest; ordinary observer bundle processing
   stays identical and retains malformed-file failures. Supplementary retrieval joins
   require the issued archive identity, actual reviewer thread/turn, native tool/call/output
   and observed output bytes. A mentioned path alone is not proof of retrieval. Distinguish
   complete, verified partial, absent and unresolved access; repeated input charges are
   not extra reads. Keep aliases/indirect commands unsupported unless independently joined.
6. Run `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and
   `cargo test --all-targets --all-features`, plus existing provider-free observer
   regression checks without changing its source. Added work should be linear in held
   context/candidate bytes, with keyed ownership lookup; flag any quadratic copying or
   per-message rescanning.

## Eventual bounded real verification

Only after explicit admission: a small actual author patch → ACK → done, followed by
an actual reviewer launch that reads the supplied archive using the configured native
read-only tool, requests one focused locked check, and returns `NO_FINDINGS` after its
result. The expected path is four outer turns across two threads; native tool activity
within a turn is not another outer turn or proof of one provider response. A hard
six-turn ceiling, fixed wall-time bound and exact final per-thread interrupt/natural
terminal settling belong in the diagnostic's admission record. The diagnostic may
require a full retrieval to verify plumbing; the benchmark factor does not force one.

Keep non-Git public evidence in the held context and verify that its exact archived
bytes occur in the actual native tool output. Preserve the native call/output IDs,
turn ownership, tool wrapper and any truncation/partial result. Retain raw
original/forwarded requests, typed accepted turns/public items, exact native input
identities, archive hashes and all failures. A default/predecessor whole-prompt
reference and automatic nonselected-path tests do not authorize a new provider control.

This scenario verifies plumbing and recoverability, not net saving or reviewer quality
equivalence. A later complete outcome census must distinguish initial deferred input,
actual full/partial retrieval, repeated retained input charges, extra response work and
repair/verification outcomes. It cannot price an unobserved workflow by subtracting the
initial context, or attribute a percentage of the accepted historical difference.

## Source and test ownership

Implementation in the root checkout is separate from the active screen's frozen source
copies and executables. It cannot change an admitted screen input. Generation remains
held until the three screen outcomes and their audits are complete and any later
diagnostic or benchmark has its own fixed prelaunch admission.

| Owner | Exclusive implementation scope | RED-first proof |
| --- | --- | --- |
| Private adapter owner | Proposed `src/bench_review_evidence.rs`, new-schema activation/root validation in `bench_experiment.rs`, and new archive/guard tests | Create-new exact bytes/typed identity, independent sequence, old-schema inactivity, failed publication, lifetime and mutation detection |
| Review integration owner | `src/cli.rs` owned `source_context` span and proposed `tests/bench_review_evidence.rs`; keep `src/review.rs` renderer unchanged | Exact held snapshot once; new/reused reviewer baseline identity; selected replacement only; fix/recheck identity; no ordinary read/tracker/bundle-counter changes |
| Offline audit owner | Proposed `audit_review_evidence.py` and `test_audit_review_evidence.py` in this study, reusing pinned existing raw/native/accounting readers | Source/role/span membership, exact output joins, complete/partial/unresolved retrieval, original failures retained, create-new output, no inferred response or token totals |
| Root/operator | Architecture paragraph, prospective phase wrapper/manifest-root contract, protocol/freeze/source attestations, independent review coordination and sole real-provider admission | Required Rust/observer regressions, unchanged driver/observer/provider identities, bounded actual native retrieval, all terminal outcomes retained |

`src/orchestrator.rs`, `src/locks.rs`, `src/agent.rs`, `src/codex.rs`, ordinary bundle
archives, the driver and observer need no implementation edits for this selected route.
If implementation appears to require such a change, return to design review rather
than silently expanding the factor. Freeze the new supplemental audit separately;
do not rewrite any earlier audit or pretend its schema already covers opaque evidence.

## Source identity

Inspected current source SHA-256 values:

- `src/review.rs`: `12d70548086d053583e1fc2fc6b4f191dd262cf61b45637a1ee5d04931c919ee`
- `src/cli.rs`: `e68988f004ac9a61d25c0565b41dd46a371ebcf62a2415c0aabc4d6d83a96763`
- `src/orchestrator.rs`: `027c35a9eaf24e99335001e68f7d0612a262831af80f8677874525db95e9332b`
- `src/agent.rs`: `a9e6065a450d05bf299202a2d6f44dd2dae33a3324e8c70e9f91d36a22b242fd`
- `bench-observer/src/lib.rs`: `188a1b4fd9913556c51024dcc0068be353813c44c688d6d22929da139f392b5f`
- `bench-three-features`: `d2487780c63c14021904b8a3c882d54fe231c5846f4a7f57fe955f50201f5644`
- `audit_read_mechanism.py`: `2ad81ed5dc27ccba621f21f087312f48ac26e7799073baa5da6130eb2f7119e9`

These identify the proposed current boundary, not proof that every historical driver
had identical source. Historical evidence keeps the qualifications in C21 of
`CANDIDATE-MAP.md` and the cohort-specific evidence notes.
